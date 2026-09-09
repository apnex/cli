//! Inventory immutable continuation inputs independently of mutable recipient work and logs.

use crate::trial_process::{TrialResult, require_trial, write_trial_json};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

pub const TRIAL_INPUT_FILES: [&str; 11] = [
    "cli",
    "receiver",
    "operations.json",
    "incompatible-operations.json",
    "source.session.json",
    "handover.session.json",
    "abandoned-request.json",
    "queue-policy.json",
    "CATALOG-TASK.md",
    "TASK.md",
    "START.md",
];

/// Hash file bytes with bounded memory, including executable inputs whose volume is reported separately.
pub fn trial_file_digest(path: &Path) -> TrialResult<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

/// Write a complete allowlisted inventory; its hashes establish integrity relative to this manifest only.
#[allow(dead_code)] // The recipient binary consumes verification only.
pub fn create_trial_manifest(directory: &Path, dependencies: Value) -> TrialResult<()> {
    let mut files = BTreeMap::new();
    for name in TRIAL_INPUT_FILES {
        let path = directory.join(name);
        files.insert(
            name,
            json!({"sha256":trial_file_digest(&path)?,"bytes":fs::metadata(&path)?.len()}),
        );
    }
    write_trial_json(
        &directory.join("manifest.json"),
        &json!({
            "format":"cli-continuation-trial-v1", "platform":{"os":std::env::consts::OS,"arch":std::env::consts::ARCH},
            "files":files, "dependencies":dependencies,
            "mutable_files":["work.session.json","completed.definition.json","completed.interface.json","logs/"],
            "integrity_scope":"Accidental drift relative to this manifest; not authentication or task fidelity.",
        }),
    )
}

/// Reject missing, modified, or non-regular inputs before a receiver begins its mutable continuation.
pub fn verify_trial_manifest(directory: &Path) -> TrialResult<Value> {
    let manifest: Value = serde_json::from_slice(&fs::read(directory.join("manifest.json"))?)?;
    require_trial(
        manifest["format"] == "cli-continuation-trial-v1",
        "Unsupported continuation manifest",
    )?;
    let files = manifest["files"]
        .as_object()
        .ok_or("Continuation file inventory is missing")?;
    require_trial(
        files.len() == TRIAL_INPUT_FILES.len(),
        "Continuation file inventory is incomplete",
    )?;
    for name in TRIAL_INPUT_FILES {
        let path = directory.join(name);
        let expected = files
            .get(name)
            .ok_or("Continuation file inventory is incomplete")?;
        let metadata = fs::symlink_metadata(&path)?;
        require_trial(
            metadata.file_type().is_file(),
            "Continuation input must be a regular file",
        )?;
        require_trial(
            expected["bytes"] == metadata.len() && expected["sha256"] == trial_file_digest(&path)?,
            &format!("Continuation input changed: {name}"),
        )?;
    }
    require_trial(
        manifest["platform"]["os"] == std::env::consts::OS
            && manifest["platform"]["arch"] == std::env::consts::ARCH,
        "Continuation executable platform differs from this receiver",
    )?;
    Ok(manifest)
}
