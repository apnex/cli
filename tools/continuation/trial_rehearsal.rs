//! Rehearse the literal receiver workflow without reporting a scripted replay as a fresh actor.

use crate::trial_process::{
    TrialProcess, TrialResult, require_trial, trial_commands, trial_file, write_trial_json,
};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

/// Run the same recorded compatibility, discovery, and invalid-commit controls before each method.
pub fn begin_continuation_trial(
    package: &Path,
    terminal: bool,
    trace: &str,
) -> TrialResult<TrialProcess> {
    let probe = Command::new(package.join("receiver"))
        .current_dir(package)
        .args(["probe", "."])
        .stdout(Stdio::from(trial_file(
            &package.join("logs/probes.console.txt"),
        )?))
        .stderr(Stdio::from(trial_file(
            &package.join("logs/probes.stderr.txt"),
        )?))
        .status()?;
    require_trial(
        probe.success(),
        "Receiver compatibility or stale request probe failed",
    )?;
    let mut process = TrialProcess::start(
        &package.join("cli"),
        package,
        "work.session.json",
        "operations.json",
        None,
        terminal,
        trace,
    )?;
    require_trial(
        process.header["event"] == "session_open",
        "Rehearsal could not reopen the handover",
    )?;
    let start = fs::read_to_string(package.join("START.md"))?;
    let inspection = start
        .split_once("./receiver run . <<'CLI'\n")
        .ok_or("Recipient inspection block missing")?
        .1
        .split_once("\nCLI\n")
        .ok_or("Recipient inspection block is unterminated")?
        .0;
    let before = fs::read(package.join("work.session.json"))?;
    for line in inspection.lines() {
        let response = process.command(line)?;
        if line == "commit" {
            require_trial(
                response["error"]["code"] == "SCHEMA_VIOLATION" && response["mutation"] == "none",
                "Invalid draft did not reject its commit",
            )?;
        } else {
            require_trial(
                response["status"] == "ok",
                &format!("Documented inspection failed: {line}: {response}"),
            )?;
        }
        require_trial(
            fs::read(package.join("work.session.json"))? == before,
            "Inspection or rejected commit changed the handover",
        )?;
    }
    Ok(process)
}

/// Exercise the shipped probe recorder and the documented inspection before a known completion recipe.
pub fn rehearse_continuation(root: &Path, package: &Path, terminal: bool) -> TrialResult<()> {
    let started = Instant::now();
    let mut process = begin_continuation_trial(package, terminal, "logs/rehearsal")?;
    trial_commands(
        &mut process,
        &fs::read_to_string(root.join("docs/reuse/acceptance/queue-finish.commands"))?,
    )?;
    require_trial(
        process.close()?["exit_code"] == 0,
        "Rehearsal did not close cleanly",
    )?;
    write_trial_json(
        &package.join("logs/rehearsal.wall.json"),
        &json!({
        "elapsed_rehearsal_ms":started.elapsed().as_millis().to_string(),
        "actor":"scripted","includes":"receiver probes and preparation overhead, inspection, rejected commit, repair, completion, and exports",
        "overlap":"Contains nested process and receiver durations; do not add them together."}),
    )?;
    Ok(())
}
