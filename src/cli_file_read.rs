//! Connected JSON file reads require transient grants that cannot be reconstructed from saved data.

use crate::authoring_error::{AuthoringError, authoring_limit_error};
use crate::cli_definition::{CliCapabilityId, CliConnectedProvider, validate_cli_name};
use crate::document_value::{DocumentValue, MAX_DOCUMENT_BYTES};
use rustix::fs::{Mode, OFlags, open, openat};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

/// File observations retain byte identity and exact output identity, never a target path or grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonFileReadObservation {
    pub provider: CliConnectedProvider,
    pub capability: CliCapabilityId,
    pub source_sha256: String,
    pub source_bytes: usize,
    pub output_sha256: String,
}

impl JsonFileReadObservation {
    /// Validate source-free observation consistency; historical source bytes are not remeasured.
    pub fn validate_file_observation(&self, capability: &CliCapabilityId, output: &str) -> bool {
        self.provider == CliConnectedProvider::JsonFileRead
            && self.capability == *capability
            && self.source_bytes > 0
            && self.source_bytes <= MAX_DOCUMENT_BYTES
            && self.source_sha256.len() == 64
            && self
                .source_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            && self.output_sha256 == format!("{:x}", Sha256::digest(output.as_bytes()))
    }
}

#[derive(Clone)]
struct JsonFileReadGrant {
    directory: Arc<File>,
    name: OsString,
}

/// Runtime-only read grants pin parent directories and grant one filename each; no serialization exists.
#[derive(Clone, Default)]
pub struct JsonFileReadGrants {
    files: BTreeMap<CliCapabilityId, JsonFileReadGrant>,
}

/// Invalid grant arguments fail before session creation without changing a prior grant.
pub fn invalid_json_read_grant(message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        "INVALID_CAPABILITY_GRANT",
        message,
        "Use --compose and --grant-json-read <unique-capability> <file> with an existing parent directory; at most 32 grants are supported.",
    )
}

impl JsonFileReadGrants {
    /// Grant JSON file reads by logical identity, resolving the parent once at launch without reading the file.
    pub fn grant_json_file_read(
        &mut self,
        capability: CliCapabilityId,
        target: &Path,
    ) -> Result<(), AuthoringError> {
        validate_cli_name(&capability.0).map_err(|error| invalid_json_read_grant(error.message))?;
        if self.files.len() >= 32 || self.files.contains_key(&capability) {
            return Err(invalid_json_read_grant(
                "JSON read grant is duplicated or exceeds the 32-grant limit.",
            ));
        }
        let name = target
            .file_name()
            .ok_or_else(|| invalid_json_read_grant("JSON read grant requires a file name."))?
            .to_os_string();
        let parent = target
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let directory = open(
            parent,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| {
            invalid_json_read_grant(format!("JSON read grant parent cannot be opened: {error}"))
        })?;
        self.files.insert(
            capability,
            JsonFileReadGrant {
                directory: Arc::new(File::from(directory)),
                name,
            },
        );
        Ok(())
    }

    /// Report granted authority only; discovery never probes whether the target currently exists.
    pub fn has_json_read_grant(&self, capability: &CliCapabilityId) -> bool {
        self.files.contains_key(capability)
    }

    /// Read a fresh bounded regular file through an explicit grant, with no-follow open relative to its pinned directory.
    pub fn read_granted_json_file(
        &self,
        capability: &CliCapabilityId,
    ) -> Result<(DocumentValue, JsonFileReadObservation), AuthoringError> {
        let grant = self.files.get(capability).ok_or_else(|| AuthoringError::new(
            "CAPABILITY_NOT_GRANTED",
            format!("JSON read capability is not granted: {}", capability.0),
            &format!("Reopen with --grant-json-read {} <file>; imported definitions and observations grant no access.", capability.0),
        ))?;
        let failed = |message: String| {
            AuthoringError::new(
                "CONNECTED_READ_FAILED",
                message,
                "Inspect the granted local file; it must be a readable regular file. Retry with a new request to observe current bytes.",
            )
        };
        let descriptor = openat(
            grant.directory.as_ref(),
            grant.name.as_os_str(),
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| {
            failed(format!(
                "Connected JSON file open failed for {}: {error}",
                capability.0
            ))
        })?;
        let file = File::from(descriptor);
        if !file
            .metadata()
            .map_err(|error| failed(format!("Connected JSON file inspection failed: {error}")))?
            .is_file()
        {
            return Err(failed(
                "Connected JSON file target is not a regular file.".into(),
            ));
        }
        let mut bytes = Vec::new();
        file.take(MAX_DOCUMENT_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| failed(format!("Connected JSON file read failed: {error}")))?;
        if bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(authoring_limit_error(
                "Connected JSON file exceeds the 1 MiB source limit.",
            ));
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| {
            AuthoringError::new(
                "INVALID_CONNECTED_DOCUMENT",
                "Connected JSON file is not UTF-8.",
                "Supply a valid UTF-8 JSON document in the granted file.",
            )
        })?;
        let document = DocumentValue::parse_document(text).map_err(|error| {
            if error.code == "LIMIT_EXCEEDED" {
                error
            } else {
                error.with_error_code("INVALID_CONNECTED_DOCUMENT")
            }
        })?;
        let observation = JsonFileReadObservation {
            provider: CliConnectedProvider::JsonFileRead,
            capability: capability.clone(),
            source_sha256: format!("{:x}", Sha256::digest(&bytes)),
            source_bytes: bytes.len(),
            output_sha256: format!(
                "{:x}",
                Sha256::digest(document.compact_document_json().as_bytes())
            ),
        };
        Ok((document, observation))
    }
}
