//! A recipient-only recorder exposes no completion recipe or acceptance oracle.

mod trial_manifest;
mod trial_process;

use crate::trial_manifest::verify_trial_manifest;
use crate::trial_process::{TrialProcess, TrialResult, require_trial, write_trial_json};
use programmable_cli::session_storage::SessionStorage;
use programmable_cli::storage_faults::StorageFaultControl;
use serde_json::json;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Instant;

fn receive_continuation(directory: &Path, mode: &str) -> TrialResult<()> {
    let directory = directory.canonicalize()?;
    verify_trial_manifest(&directory)?;
    let work = directory.join("work.session.json");
    if !work.exists() {
        // Reuse the existing exclusive, synchronized publication boundary for the first copy.
        let mut storage =
            SessionStorage::acquire_session_storage(&work, StorageFaultControl::default())?;
        storage.publish_session_checkpoint(
            &fs::read(directory.join("handover.session.json"))?,
            true,
        )?;
    }
    let trace = format!("logs/{}", uuid::Uuid::new_v4());
    if mode == "probe" {
        let before = fs::read(&work)?;
        let wrong = TrialProcess::start(
            &directory.join("cli"),
            &directory,
            "work.session.json",
            "incompatible-operations.json",
            None,
            true,
            &format!("{trace}-dependency"),
        )?;
        println!("{}", wrong.header);
        require_trial(
            wrong.header["error"]["code"] == "DEFINITION_MISMATCH",
            "Expected incompatible dependency rejection",
        )?;
        let wrong_metrics = wrong.close()?;
        require_trial(
            wrong_metrics["exit_code"] == 1,
            "Wrong declaration did not fail opening",
        )?;
        require_trial(
            fs::read(&work)? == before,
            "Dependency rejection changed the checkpoint",
        )?;
        let mut stale = TrialProcess::start(
            &directory.join("cli"),
            &directory,
            "work.session.json",
            "operations.json",
            None,
            false,
            &format!("{trace}-stale"),
        )?;
        println!("{}", stale.header);
        let response = stale.send_wire(&fs::read(directory.join("abandoned-request.json"))?)?;
        println!("{response}");
        require_trial(
            response["error"]["code"] == "REVISION_CONFLICT" && response["mutation"] == "none",
            "Expected stale request rejection",
        )?;
        let metrics = stale.close()?;
        require_trial(
            metrics["exit_code"] == 0,
            "Stale request process failed to close",
        )?;
        require_trial(
            fs::read(&work)? == before,
            "Stale request changed the checkpoint",
        )?;
    } else {
        let mut process = TrialProcess::start(
            &directory.join("cli"),
            &directory,
            "work.session.json",
            "operations.json",
            None,
            true,
            &trace,
        )?;
        println!("{}", process.header);
        require_trial(
            process.header["event"] == "session_open",
            "Recipient session did not open",
        )?;
        for line in io::stdin().lock().lines() {
            let line = line?;
            if !line.trim().is_empty() {
                println!("{}", process.command(&line)?);
            }
        }
        let metrics = process.close()?;
        require_trial(metrics["exit_code"] == 0, "Recipient CLI process failed")?;
    }
    Ok(())
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [mode, directory] if mode == "run" || mode == "probe" => {
            let started = Instant::now();
            let result = receive_continuation(Path::new(directory), mode);
            let measurement = json!({"mode":mode,"elapsed_receiver_ms":started.elapsed().as_millis().to_string(),
                "status":if result.is_ok() {"ok"} else {"error"},
                "includes":"manifest hashing, initial publication, process lifetime, input wait, and recording",
                "overlap":"Contains the nested process durations; do not add them to this duration."});
            let record =
                Path::new(directory).join(format!("logs/{}.receiver.json", uuid::Uuid::new_v4()));
            if Path::new(directory).is_dir() {
                if let Err(error) = write_trial_json(&record, &measurement) {
                    eprintln!("Receiver timing could not be retained: {error}");
                    std::process::exit(1);
                }
            }
            result
        }
        _ => Err("Usage: receiver (run|probe) <package-directory>".into()),
    };
    if let Err(error) = result {
        eprintln!("Continuation receiver failed: {error}");
        std::process::exit(1);
    }
}
