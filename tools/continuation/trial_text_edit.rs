//! Measure a literal file-editing baseline without reaching into authoring checkpoints.

use crate::trial_manifest::trial_file_digest;
use crate::trial_process::{TrialResult, require_trial, trial_file, write_trial_json};
use serde::Deserialize;
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrialTextEdit {
    find: String,
    replace: String,
}

/// Check every literal edit anchor before writing; retain exact input, recipe, output, and rejection.
pub fn apply_trial_text_edits(
    document: &Path,
    recipe: &Path,
    evidence: &Path,
) -> TrialResult<Value> {
    let started = Instant::now();
    fs::create_dir(evidence)?;
    let before = fs::read(document)?;
    let recipe_bytes = fs::read(recipe)?;
    trial_file(&evidence.join("before.json"))?.write_all(&before)?;
    trial_file(&evidence.join("recipe.json"))?.write_all(&recipe_bytes)?;
    let before_digest = trial_file_digest(document)?;
    let attempt = (|| -> TrialResult<(String, usize)> {
        let edits: Vec<TrialTextEdit> = serde_json::from_slice(&recipe_bytes)?;
        require_trial(!edits.is_empty(), "Text edit recipe has no edits")?;
        let mut after = String::from_utf8(before.clone())?;
        for edit in &edits {
            require_trial(
                !edit.find.is_empty() && after.match_indices(&edit.find).count() == 1,
                "Text edit anchor must occur exactly once",
            )?;
            require_trial(
                edit.find != edit.replace,
                "Text edit must change its matched fragment",
            )?;
            after = after.replacen(&edit.find, &edit.replace, 1);
        }
        Ok((after, edits.len()))
    })();
    let (after, edits) = match attempt {
        Ok(result) => result,
        Err(error) => {
            require_trial(
                fs::read(document)? == before,
                "Rejected text edits changed document bytes",
            )?;
            write_trial_json(
                &evidence.join("result.json"),
                &json!({
                    "status":"error", "error":error.to_string(), "document_unchanged":true,
                    "document_bytes_read":before.len(), "recipe_bytes":recipe_bytes.len(),
                    "document_bytes_written":0, "before_sha256":before_digest,
                    "elapsed_editor_ms":started.elapsed().as_millis().to_string()
                }),
            )?;
            return Err(error);
        }
    };
    fs::write(document, after.as_bytes())?;
    require_trial(
        fs::read(document)? == after.as_bytes(),
        "Text edit output differs from written file",
    )?;
    trial_file(&evidence.join("after.json"))?.write_all(after.as_bytes())?;
    let report = json!({
        "status":"ok", "edits_applied":edits, "recipe_bytes":recipe_bytes.len(),
        "document_bytes_read":before.len(), "document_bytes_written":after.len(),
        "before_sha256":before_digest, "after_sha256":trial_file_digest(document)?,
        "elapsed_editor_ms":started.elapsed().as_millis().to_string(),
        "scope":"Literal text replacement on an exported document; no checkpoint write or product transaction is implied.",
        "timing_overlap":"Included in method and open-process wall time; do not add these timers."
    });
    write_trial_json(&evidence.join("result.json"), &report)?;
    Ok(report)
}
