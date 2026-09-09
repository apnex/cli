//! The run launcher loads a portable specification and hands untouched argument strings to its surface.

use crate::authoring_error::AuthoringError;
use crate::authoring_protocol::AuthoringResponse;
use crate::cli_definition::CliCapabilityId;
use crate::cli_file_read::{JsonFileReadGrants, invalid_json_read_grant};
use crate::cli_run_frontend::{
    execute_run_tokens, run_cli_stream, run_cli_terminal, write_run_response,
};
use crate::cli_run_routes::run_usage_error;
use crate::cli_run_session::CliRunSession;
use crate::document_value::MAX_REQUEST_BYTES;
use crate::storage_faults::StorageFaultControl;
use crate::terminal_input::TerminalToken;
use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

pub const RUN_LAUNCH_HELP: &str = "cli run [--json] [--session <checkpoint>] [--grant-json-read <capability> <file>] <spec.json> [context ... command arguments ...]\n\nLoad a definition, assembled definition, or interface export and use its commands directly.\nOmit the command for a contextual shell (or command lines on stdin).\nUse --help after the specification, a context, or a command for generated help.\nUse -- before a command argument to pass a literal --help value.\n--session retains state across launches; use - as the specification to reopen it without its source.\nControls: :help :tree :up :top :status :export <file> :exit\nExit status: 0 success, 2 usage/arguments, 1 execution/storage. Mocks are labeled on stderr.\n";

fn launch_run(
    arguments: impl Iterator<Item = OsString>,
    structured: &mut bool,
) -> Result<i32, AuthoringError> {
    let mut arguments = arguments.peekable();
    let mut source = None;
    let mut checkpoint = None;
    let mut grants = Vec::new();
    let mut words = Vec::new();
    let mut options = true;
    while let Some(argument) = arguments.next() {
        match argument.to_str() {
            Some("--help" | "-h") if options && source.is_none() => {
                io::stdout()
                    .write_all(RUN_LAUNCH_HELP.as_bytes())
                    .map_err(|error| {
                        AuthoringError::new(
                            "RUN_DELIVERY_FAILED",
                            format!("Cannot write launcher help: {error}"),
                            "Check the output destination and retry.",
                        )
                    })?;
                return Ok(0);
            }
            Some("--json") if options && !*structured => *structured = true,
            Some("--session") if options && checkpoint.is_none() => {
                checkpoint = Some(PathBuf::from(arguments.next().ok_or_else(|| {
                    run_usage_error("--session requires a checkpoint path.")
                })?));
            }
            Some("--grant-json-read") if options => {
                let capability = arguments
                    .next()
                    .and_then(|arg| arg.into_string().ok())
                    .ok_or_else(|| {
                        invalid_json_read_grant(
                            "JSON read grant requires a UTF-8 capability identifier.",
                        )
                    })?;
                let file = PathBuf::from(arguments.next().ok_or_else(|| {
                    invalid_json_read_grant("JSON read grant requires a target file.")
                })?);
                if grants.len() >= 32 {
                    return Err(invalid_json_read_grant(
                        "JSON read launcher exceeds the 32-grant limit.",
                    ));
                }
                grants.push((CliCapabilityId(capability), file));
            }
            Some("--") if options => {
                options = false;
                if source.is_some() {
                    words.push(argument);
                    words.extend(arguments);
                    break;
                }
            }
            _ if source.is_none() => {
                if options
                    && argument
                        .to_str()
                        .is_some_and(|text| text.starts_with('-') && text != "-")
                {
                    return Err(run_usage_error(
                        "Unknown run launcher option. Use cli run --help.",
                    ));
                }
                source = Some(PathBuf::from(argument));
            }
            _ => {
                words.push(argument);
                words.extend(arguments);
                break;
            }
        }
    }
    let source = source.ok_or_else(|| {
        run_usage_error("Run mode requires a specification path. Use cli run --help.")
    })?;
    let mut size: usize = 0;
    let words: Vec<TerminalToken> = words
        .into_iter()
        .map(|word| {
            let text = word
                .into_string()
                .map_err(|_| run_usage_error("Configured command arguments must be UTF-8."))?;
            size = size.saturating_add(text.len() + 1);
            if size > MAX_REQUEST_BYTES {
                return Err(run_usage_error(
                    "Configured command exceeds the 2 MiB argument limit.",
                ));
            }
            Ok(TerminalToken {
                start: 0,
                end: text.len(),
                text,
            })
        })
        .collect::<Result<_, AuthoringError>>()?;
    let mut authority = JsonFileReadGrants::default();
    for (capability, file) in grants {
        authority.grant_json_file_read(capability, &file)?;
    }
    let mut session = CliRunSession::open_cli_run(
        &source,
        checkpoint.as_deref(),
        authority,
        StorageFaultControl::from_test_environment()?,
    )?;
    let result = if !words.is_empty() {
        execute_run_tokens(
            &mut session,
            &words,
            true,
            *structured,
            &mut io::stdout().lock(),
            &mut io::stderr().lock(),
        )
        .map(|(code, _)| code)
    } else if io::stdin().is_terminal() && io::stdout().is_terminal() {
        run_cli_terminal(&mut session, *structured)
    } else {
        run_cli_stream(
            &mut session,
            *structured,
            &mut io::stdin().lock(),
            &mut io::stdout().lock(),
            &mut io::stderr().lock(),
        )
    };
    match result {
        Ok(code) => Ok(code),
        Err(error) => {
            // State may already be durable; this is a transport stop, not a rejected command.
            let _ = writeln!(
                io::stderr(),
                "Run transport stopped: {}. Reopen a persistent session and inspect :status before retrying.",
                crate::cli_run_frontend::escape_run_metadata(&error.to_string())
            );
            Ok(1)
        }
    }
}

pub fn launch_cli_run(arguments: impl Iterator<Item = OsString>) -> i32 {
    let mut structured = false;
    match launch_run(arguments, &mut structured) {
        Ok(code) => code,
        Err(error) => {
            let response = AuthoringResponse::operation_failure(None, None, None, error);
            write_run_response(
                None,
                &response,
                structured,
                &mut io::stdout().lock(),
                &mut io::stderr().lock(),
            )
            .unwrap_or(1)
        }
    }
}
