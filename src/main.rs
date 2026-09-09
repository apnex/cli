//! The launcher selects storage and presentation; operation words come from loaded data.

use programmable_cli::authoring_error::AuthoringError;
use programmable_cli::authoring_frontend::{run_authoring_stream, run_authoring_terminal};
use programmable_cli::authoring_protocol::AuthoringResponse;
use programmable_cli::authoring_runtime::{AuthoringRuntime, checked_response_bytes};
use programmable_cli::cli_definition::CliCapabilityId;
use programmable_cli::cli_file_read::{JsonFileReadGrants, invalid_json_read_grant};
use programmable_cli::document_value::MAX_SCALAR_BYTES;
use programmable_cli::operation_definition::OperationDefinition;
use programmable_cli::session_storage::read_regular_file_bounded;
use programmable_cli::storage_faults::StorageFaultControl;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

const LAUNCH_HELP: &str = "cli run <spec.json> [context ... command arguments ...]  (use cli run --help for options)\n\ncli --definition <operations.json> --session <checkpoint> [--create --intent-file <task>] [--compose] [--constraints] [--grant-json-read <capability> <file>] [--machine | --commands]\n\nOmit --create to reopen a saved draft. --create requires original task text in --intent-file.\n--compose enables CLI definition activation, contextual discovery, simulated commands, and declared connected reads.\n--grant-json-read grants one logical capability access to one local JSON file for this process only; repeat for distinct capabilities (at most 32). Requires --compose.\n--constraints enables explicit schema attachment, contextual guidance, validation, and constrained commits.\nUse the same profile when reopening a checkpoint; read grants must be supplied again for fresh connected invocations.\n--machine reads JSON request lines; --commands reads terminal command lines and emits JSON events.\nWithout either flag, a terminal gets interactive editing and other input gets plain command output.\nInside the session, use help to discover operations from the loaded definition.\nCtrl-D closes the session; every acknowledged edit is already checkpointed.\n";

fn launch_authoring() -> Result<i32, AuthoringError> {
    let invalid = || {
        AuthoringError::new(
            "INVALID_REQUEST",
            "Invalid launcher arguments.",
            LAUNCH_HELP,
        )
    };
    let mut arguments = std::env::args_os().skip(1);
    let mut definition = None;
    let mut session = None;
    let mut intent_file = None;
    let mut create = false;
    let mut machine = false;
    let mut commands = false;
    let mut compose = false;
    let mut constraints = false;
    let mut json_read_targets = Vec::new();
    while let Some(argument) = arguments.next() {
        match argument.to_str() {
            Some("--help" | "-h") => {
                print!("{LAUNCH_HELP}");
                return Ok(0);
            }
            Some("--definition") if definition.is_none() => {
                definition = Some(PathBuf::from(arguments.next().ok_or_else(invalid)?))
            }
            Some("--session") if session.is_none() => {
                session = Some(PathBuf::from(arguments.next().ok_or_else(invalid)?))
            }
            Some("--intent-file") if intent_file.is_none() => {
                intent_file = Some(PathBuf::from(arguments.next().ok_or_else(invalid)?))
            }
            Some("--create") if !create => create = true,
            Some("--machine") if !machine && !commands => machine = true,
            Some("--commands") if !machine && !commands => commands = true,
            Some("--compose") if !compose => compose = true,
            Some("--constraints") if !constraints => constraints = true,
            Some("--grant-json-read") => {
                let capability = arguments
                    .next()
                    .and_then(|arg| arg.into_string().ok())
                    .ok_or_else(|| {
                        invalid_json_read_grant(
                            "JSON read grant requires a UTF-8 capability identifier.",
                        )
                    })?;
                let target = arguments.next().map(PathBuf::from).ok_or_else(|| {
                    invalid_json_read_grant("JSON read grant requires a target file.")
                })?;
                if json_read_targets.len() >= 32 {
                    return Err(invalid_json_read_grant(
                        "JSON read launcher exceeds the 32-grant limit.",
                    ));
                }
                json_read_targets.push((CliCapabilityId(capability), target));
            }
            _ => return Err(invalid()),
        }
    }
    if create != intent_file.is_some() {
        return Err(invalid());
    }
    if !compose && !json_read_targets.is_empty() {
        return Err(invalid_json_read_grant(
            "JSON read grants require the composition profile.",
        ));
    }
    let mut json_read_grants = JsonFileReadGrants::default();
    for (capability, target) in json_read_targets {
        json_read_grants.grant_json_file_read(capability, &target)?;
    }
    let definition = definition.ok_or_else(invalid)?;
    let declaration = OperationDefinition::load_kernel_profile(
        &definition,
        programmable_cli::kernel_profile::KernelProfile::from_layer_flags(compose, constraints),
    )?
    .with_json_read_grants(json_read_grants);
    let intent = intent_file
        .map(|path| {
            read_regular_file_bounded(&path, MAX_SCALAR_BYTES, "INVALID_REQUEST").and_then(
                |bytes| {
                    String::from_utf8(bytes).map_err(|_| {
                        AuthoringError::new(
                            "INVALID_REQUEST",
                            "Original task file is not UTF-8.",
                            "Supply UTF-8 original task text.",
                        )
                    })
                },
            )
        })
        .transpose()?;
    let mut runtime = AuthoringRuntime::open_authoring_session(
        &session.ok_or_else(invalid)?,
        declaration,
        intent,
        StorageFaultControl::from_test_environment()?,
    )?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    let result = if !machine && !commands && stdin.is_terminal() && stdout.is_terminal() {
        run_authoring_terminal(&mut runtime)
    } else {
        run_authoring_stream(
            &mut runtime,
            &mut stdin.lock(),
            &mut stdout.lock(),
            machine,
            machine || commands,
        )
    };
    if let Err(error) = result {
        // A delivery failure can follow durable mutation. Do not fabricate an operation rejection.
        eprintln!(
            "Authoring transport stopped: {error}. Reopen and inspect the saved receipt before retrying."
        );
        return Ok(1);
    }
    Ok(if runtime.session_is_uncertain() { 1 } else { 0 })
}

fn main() {
    if std::env::args_os().nth(1).is_some_and(|word| word == "run") {
        std::process::exit(programmable_cli::cli_run_launcher::launch_cli_run(
            std::env::args_os().skip(2),
        ));
    }
    match launch_authoring() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            let response = AuthoringResponse::operation_failure(None, None, None, error);
            if let Ok(bytes) = checked_response_bytes(&response) {
                let _ = io::stdout().write_all(&bytes);
            }
            std::process::exit(1);
        }
    }
}
