//! A stable sidecar lock and synchronized atomic publication own local checkpoint durability.

use crate::authoring_error::{AuthoringError, authoring_limit_error};
use crate::document_value::MAX_CHECKPOINT_BYTES;
use crate::storage_faults::StorageFaultControl;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn storage_failure(code: &str, message: impl Into<String>) -> AuthoringError {
    let recovery = match code {
        "PERSISTENCE_UNCERTAIN" => {
            "Close and reopen the session before another write; inspect the recovered receipt."
        }
        "EXPORT_UNCERTAIN" => "Inspect or repeat the export at the same revision and destination.",
        "SESSION_CHANGED" => "Preserve the external file and reopen the session before retrying.",
        "SESSION_LOCKED" => "Wait for the current session owner to close, then reopen.",
        "EXPORT_EXISTS" => {
            "Choose a new destination; different existing bytes will not be replaced."
        }
        _ => {
            "Inspect the local file or filesystem error and retry without replacing unrelated content."
        }
    };
    AuthoringError::new(code, message, recovery)
}

fn reject_nonregular_target(
    path: &Path,
    code: &str,
    allow_missing: bool,
) -> Result<(), AuthoringError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(storage_failure(
            code,
            format!(
                "Storage target must be a regular file, not a symlink or special file: {}",
                path.display()
            ),
        )),
        Err(error) if allow_missing && error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(storage_failure(
            code,
            format!(
                "Storage target cannot be inspected: {}: {error}",
                path.display()
            ),
        )),
    }
}

fn canonical_target_path(path: &Path, code: &str) -> Result<PathBuf, AuthoringError> {
    let name = path
        .file_name()
        .ok_or_else(|| storage_failure(code, "Storage target requires a file name."))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = parent.canonicalize().map_err(|error| {
        storage_failure(
            code,
            format!("Storage parent directory cannot be resolved: {error}"),
        )
    })?;
    let target = parent.join(name);
    if target.to_str().is_none() {
        return Err(storage_failure(
            code,
            "Storage target must have a UTF-8 path for its receipt.",
        ));
    }
    Ok(target)
}

/// Read a bounded regular file without following a symlink or allocating its unchecked length.
pub fn read_regular_file_bounded(
    path: &Path,
    maximum: usize,
    code: &str,
) -> Result<Vec<u8>, AuthoringError> {
    reject_nonregular_target(path, code, false)?;
    let input = File::open(path).map_err(|error| {
        storage_failure(
            code,
            format!("Storage file open failed: {}: {error}", path.display()),
        )
    })?;
    if !input
        .metadata()
        .map_err(|error| storage_failure(code, error.to_string()))?
        .is_file()
    {
        return Err(storage_failure(
            code,
            "Opened storage target is not a regular file.",
        ));
    }
    let mut bytes = Vec::new();
    input
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| storage_failure(code, format!("Storage file read failed: {error}")))?;
    if bytes.len() > maximum {
        return Err(authoring_limit_error(
            "Storage input exceeds the supported byte limit.",
        ));
    }
    Ok(bytes)
}

struct TemporaryPublication {
    path: PathBuf,
    file: File,
}

impl TemporaryPublication {
    fn create(target: &Path) -> io::Result<Self> {
        let path = target.parent().unwrap().join(format!(
            ".{}.{}.tmp",
            target.file_name().unwrap().to_string_lossy(),
            uuid::Uuid::new_v4()
        ));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(Self { path, file })
    }
}
impl Drop for TemporaryPublication {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Session storage retains a stable lock inode while each checkpoint inode is replaced.
pub struct SessionStorage {
    path: PathBuf,
    _lock: File,
    observed_bytes: Vec<u8>,
    faults: StorageFaultControl,
}

impl SessionStorage {
    /// Acquire an exclusive nonblocking session lock without unlinking it at process exit.
    pub fn acquire_session_storage(
        path: &Path,
        faults: StorageFaultControl,
    ) -> Result<Self, AuthoringError> {
        let path = canonical_target_path(path, "PERSISTENCE_FAILED")?;
        reject_nonregular_target(&path, "INVALID_SESSION", true)?;
        let lock_path = path.parent().unwrap().join(format!(
            ".{}.lock",
            path.file_name().unwrap().to_string_lossy()
        ));
        reject_nonregular_target(&lock_path, "PERSISTENCE_FAILED", true)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|error| {
                storage_failure(
                    "PERSISTENCE_FAILED",
                    format!("Session lock file cannot be opened: {error}"),
                )
            })?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err(storage_failure(
                    "SESSION_LOCKED",
                    "Another process owns this session's stable lock.",
                ));
            }
            Err(error) => {
                return Err(storage_failure(
                    "PERSISTENCE_FAILED",
                    format!("Session file lock failed: {error}"),
                ));
            }
        }
        Ok(Self {
            path,
            _lock: lock,
            observed_bytes: Vec::new(),
            faults,
        })
    }

    /// Load the authoritative checkpoint only; leftover temporary files are never promoted.
    pub fn read_session_checkpoint(&mut self) -> Result<Vec<u8>, AuthoringError> {
        let bytes = read_regular_file_bounded(&self.path, MAX_CHECKPOINT_BYTES, "INVALID_SESSION")?;
        self.observed_bytes = bytes.clone();
        Ok(bytes)
    }

    /// Synchronize a validated reopened checkpoint before reporting its state as durable.
    pub fn synchronize_reopened_checkpoint(&self) -> Result<(), AuthoringError> {
        let synchronize = || -> io::Result<()> {
            self.faults.hit_storage_boundary("reopen_file_sync")?;
            File::open(&self.path)?.sync_all()?;
            self.faults.hit_storage_boundary("reopen_directory_sync")?;
            File::open(self.path.parent().unwrap())?.sync_all()
        };
        synchronize().map_err(|error| {
            storage_failure(
                "PERSISTENCE_FAILED",
                format!("Reopened checkpoint synchronization failed: {error}"),
            )
        })
    }

    /// Detect prior outside changes without claiming compare-and-swap against a simultaneous writer.
    pub fn verify_unchanged_checkpoint(&self) -> Result<(), AuthoringError> {
        match read_regular_file_bounded(&self.path, MAX_CHECKPOINT_BYTES, "SESSION_CHANGED") {
            Ok(bytes) if bytes == self.observed_bytes => Ok(()),
            _ => Err(storage_failure(
                "SESSION_CHANGED",
                "Checkpoint bytes no longer match the session's last observed file.",
            )),
        }
    }

    /// Publish a complete checkpoint and receipt before the runtime acknowledges their transition.
    pub fn publish_session_checkpoint(
        &mut self,
        bytes: &[u8],
        create: bool,
    ) -> Result<(), AuthoringError> {
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(authoring_limit_error(
                "Serialized checkpoint and receipt exceed 8 MiB.",
            ));
        }
        if !create {
            self.verify_unchanged_checkpoint()?;
        }
        reject_nonregular_target(&self.path, "PERSISTENCE_FAILED", create)?;
        if create && self.path.exists() {
            return Err(storage_failure(
                "PERSISTENCE_FAILED",
                "Session creation refused because the checkpoint already exists.",
            ));
        }
        let mut published = false;
        let mut publish = || -> io::Result<()> {
            self.faults
                .hit_storage_boundary("before_checkpoint_write")?;
            let mut temporary = TemporaryPublication::create(&self.path)?;
            temporary.file.write_all(bytes)?;
            self.faults.hit_storage_boundary("after_checkpoint_write")?;
            temporary.file.sync_all()?;
            self.faults
                .hit_storage_boundary("after_checkpoint_file_sync")?;
            if create {
                fs::hard_link(&temporary.path, &self.path)?;
            } else {
                fs::rename(&temporary.path, &self.path)?;
            }
            published = true;
            self.faults
                .hit_storage_boundary("after_checkpoint_rename")?;
            File::open(self.path.parent().unwrap())?.sync_all()?;
            self.faults
                .hit_storage_boundary("after_checkpoint_directory_sync")?;
            Ok(())
        };
        if let Err(error) = publish() {
            return Err(storage_failure(
                if published {
                    "PERSISTENCE_UNCERTAIN"
                } else {
                    "PERSISTENCE_FAILED"
                },
                format!("Checkpoint publication failed: {error}"),
            ));
        }
        self.observed_bytes = bytes.to_vec();
        Ok(())
    }

    /// Export bounded document or interface bytes using create-only publication and an observed-byte receipt.
    pub fn export_document_bytes(
        &self,
        destination: &Path,
        bytes: &[u8],
        revision: &str,
    ) -> Result<Value, AuthoringError> {
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(authoring_limit_error(
                "Document export envelope exceeds 8 MiB.",
            ));
        }
        let target = canonical_target_path(destination, "EXPORT_FAILED")?;
        reject_nonregular_target(&target, "EXPORT_FAILED", true)?;
        let mut created = false;
        if target.exists() {
            if read_regular_file_bounded(&target, MAX_CHECKPOINT_BYTES, "EXPORT_FAILED")
                .ok()
                .as_deref()
                != Some(bytes)
            {
                return Err(storage_failure(
                    "EXPORT_EXISTS",
                    "Export destination already contains different bytes.",
                ));
            }
        } else {
            let mut published = false;
            let mut publish = || -> io::Result<()> {
                let mut temporary = TemporaryPublication::create(&target)?;
                temporary.file.write_all(bytes)?;
                temporary.file.sync_all()?;
                fs::hard_link(&temporary.path, &target)?;
                published = true;
                self.faults.hit_storage_boundary("after_export_publish")?;
                Ok(())
            };
            if let Err(error) = publish() {
                if !published && error.kind() == io::ErrorKind::AlreadyExists {
                    if read_regular_file_bounded(&target, MAX_CHECKPOINT_BYTES, "EXPORT_FAILED")
                        .ok()
                        .as_deref()
                        != Some(bytes)
                    {
                        return Err(storage_failure(
                            "EXPORT_EXISTS",
                            "Export destination appeared with different bytes.",
                        ));
                    }
                } else {
                    return Err(storage_failure(
                        if published {
                            "EXPORT_UNCERTAIN"
                        } else {
                            "EXPORT_FAILED"
                        },
                        format!("Document export publication failed: {error}"),
                    ));
                }
            } else {
                created = true;
            }
        }
        let synchronize = || -> io::Result<()> {
            File::open(&target)?.sync_all()?;
            File::open(target.parent().unwrap())?.sync_all()
        };
        synchronize().map_err(|error| {
            storage_failure(
                "EXPORT_UNCERTAIN",
                format!("Document export synchronization failed: {error}"),
            )
        })?;
        let observed = read_regular_file_bounded(&target, MAX_CHECKPOINT_BYTES, "EXPORT_UNCERTAIN")
            .map_err(|error| error.with_error_code("EXPORT_UNCERTAIN"))?;
        if observed != bytes {
            return Err(storage_failure(
                "EXPORT_UNCERTAIN",
                "Export bytes changed before their receipt could be verified.",
            ));
        }
        Ok(
            json!({"revision":revision,"destination":target.to_str().unwrap(),"sha256":format!("{:x}",Sha256::digest(&observed)),"publication":if created { "created" } else { "already_present" }}),
        )
    }
}
