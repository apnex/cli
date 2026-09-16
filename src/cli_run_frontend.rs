//! Direct commands, piped lines, and terminal input share one lowering and delivery path.

use crate::authoring_error::AuthoringError;
use crate::authoring_frontend::read_authoring_line;
use crate::authoring_protocol::{AuthoringRequest, AuthoringResponse, RequestIdentity};
use crate::authoring_runtime::checked_response_bytes;
use crate::cli_definition::{CliBehaviorBinding, CliCommand};
use crate::cli_run_routes::{CliRunTarget, RUN_CONTROLS, run_usage_error};
use crate::cli_run_session::CliRunSession;
use crate::cli_terminal::compile_cli_terminal_arguments;
use crate::cli_verb_tree::{cli_binding_label, render_run_verb_tree};
use crate::cli_view_frontend::{
    attach_output_presentation, discover_output_views, require_output_view,
    select_command_output_view,
};
use crate::document_value::MAX_RESPONSE_BYTES;
use crate::terminal_input::{TerminalToken, tokenize_terminal_input};
use reedline::{
    ColumnarMenu, DefaultPrompt, DefaultPromptSegment, Emacs, KeyCode, KeyModifiers, MenuBuilder,
    Reedline, ReedlineEvent, ReedlineMenu, Signal, default_emacs_keybindings,
};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

/// Metadata is printable text, never an escape sequence interpreted by the terminal.
pub fn escape_run_metadata(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}

pub fn run_error_exit(error: &AuthoringError) -> i32 {
    match error.code.as_str() {
        "INVALID_RUN_COMMAND"
        | "INVALID_CLI_ARGUMENTS"
        | "UNKNOWN_CLI_COMMAND"
        | "UNKNOWN_CLI_CONTEXT"
        | "UNKNOWN_OUTPUT_VIEW"
        | "OUTPUT_VIEW_REQUIRED"
        | "INVALID_REQUEST" => 2,
        _ => 1,
    }
}

fn run_request(session: &CliRunSession, operation: &str, arguments: Value) -> AuthoringRequest {
    let state = session.runtime.session_checkpoint();
    AuthoringRequest {
        request_id: RequestIdentity::new_request_identity(),
        session_id: state.session_id.clone(),
        operation: operation.into(),
        arguments: serde_json::from_value(arguments).expect("Run constructs argument objects"),
        expected_revision: session
            .runtime
            .operation_definition()
            .operations
            .get(operation)
            .is_some_and(|operation| operation.effect != "read")
            .then_some(state.revision),
    }
}

fn run_view(session: &CliRunSession, operation: &str, result: Value) -> AuthoringResponse {
    AuthoringResponse::operation_success(
        &run_request(session, operation, json!({})),
        session.runtime.current_session_header(),
        result,
        "none",
    )
}

fn command_help(session: &CliRunSession, word: &str, command: &CliCommand) -> Value {
    let requirement = match &command.binding {
        CliBehaviorBinding::Connected {
            provider,
            capability,
        } => {
            json!({"provider":provider, "capability":capability, "granted":session.runtime.operation_definition().has_read_capability(provider, capability)})
        }
        CliBehaviorBinding::Unbound { reason } => json!({"reason":reason}),
        _ => Value::Null,
    };
    let mut result = json!({"word":word, "id":command.id, "help":command.help, "parameters":command.parameters, "binding":cli_binding_label(&command.binding), "requirement":requirement});
    if let Some(view) = &command.view {
        result["view"] = json!(view);
    }
    result
}

fn run_help(
    session: &CliRunSession,
    context: &str,
    command_word: Option<&str>,
) -> AuthoringResponse {
    let active = session.active_interface();
    let context_definition = &active.definition.contexts[context];
    let commands: Vec<_> = context_definition
        .commands
        .iter()
        .filter(|(word, _)| command_word.is_none_or(|wanted| wanted == word.as_str()))
        .map(|(word, command)| command_help(session, word, command))
        .collect();
    let children: Vec<_> = if command_word.is_none() {
        session.routes.context_children(context).iter().map(|(word,id)| json!({"word":word,"id":id,"help":active.definition.contexts[id].help})).collect()
    } else {
        Vec::new()
    };
    let path = session.routes.context_path(&active.definition, context);
    let mut text = format!(
        "{}{}\n{}\n",
        active.definition.id,
        if path.is_empty() {
            String::new()
        } else {
            format!(" / {path}")
        },
        escape_run_metadata(&context_definition.help)
    );
    for child in &children {
        text.push_str(&format!(
            "  {}/  {}\n",
            child["word"].as_str().unwrap(),
            escape_run_metadata(child["help"].as_str().unwrap())
        ));
    }
    for command in &commands {
        text.push_str(&format!("  {}", command["word"].as_str().unwrap()));
        for parameter in command["parameters"].as_array().unwrap() {
            text.push_str(&format!(
                " <{}:{}>",
                parameter["name"].as_str().unwrap(),
                parameter["type"].as_str().unwrap()
            ));
        }
        text.push_str(&format!(
            " [{}]\n    {}\n",
            command["binding"].as_str().unwrap(),
            escape_run_metadata(command["help"].as_str().unwrap())
        ));
        if let Some(capability) = command["requirement"]["capability"].as_str() {
            let instruction = session.capability_help.clone().unwrap_or_else(|| {
                if command["requirement"]["provider"] == "json-file-read-v1" {
                    format!("Requires --grant-json-read {capability} <file>")
                } else {
                    format!("Requires HTTP capability {capability} from the application launcher")
                }
            });
            text.push_str(&format!(
                "    {}; granted: {}\n",
                escape_run_metadata(&instruction),
                command["requirement"]["granted"]
            ));
        }
        if let Some(reason) = command["requirement"]["reason"].as_str() {
            text.push_str(&format!("    {}\n", escape_run_metadata(reason)));
        }
        if let Some(view) = command["view"].as_str() {
            text.push_str(&format!(
                "    Output view: {} (use --table)\n",
                escape_run_metadata(view)
            ));
        }
        if command_word.is_some() {
            for parameter in command["parameters"].as_array().unwrap() {
                text.push_str(&format!(
                    "    {}: {}\n",
                    parameter["name"].as_str().unwrap(),
                    escape_run_metadata(parameter["help"].as_str().unwrap())
                ));
            }
        }
    }
    text.push_str("Controls:");
    for (word, _) in RUN_CONTROLS {
        text.push(' ');
        text.push_str(word);
        if word == ":export" {
            text.push_str(" <file>");
        }
        if word == ":render" {
            text.push_str(" <view>");
        }
    }
    text.push('\n');
    if let Some(help) = &session.launch_help {
        text.push_str(help);
        text.push('\n');
    }
    if !session.routes.control_aliases.is_empty() {
        text.push_str("Shortcuts:");
        for (alias, target) in &session.routes.control_aliases {
            text.push_str(&format!(" {alias}={target}"));
        }
        text.push('\n');
    }
    run_view(
        session,
        ":help",
        json!({"help_text":text,"definition_id":active.definition.id,"context":context,"path":path,"commands":commands,"contexts":children,"controls":RUN_CONTROLS,"control_aliases":session.routes.control_aliases}),
    )
}

fn dispatch_run_tokens(
    session: &mut CliRunSession,
    tokens: &[TerminalToken],
    one_shot: bool,
) -> Result<Option<AuthoringResponse>, AuthoringError> {
    let first = tokens
        .first()
        .map(|token| token.text.as_str())
        .unwrap_or(":help");
    let first = session
        .routes
        .control_aliases
        .get(first)
        .map(String::as_str)
        .unwrap_or(first);
    let start = if one_shot {
        "root".to_owned()
    } else {
        session.active_interface().context.clone()
    };
    let exact = |count| {
        if tokens.len() == count {
            Ok(())
        } else {
            Err(run_usage_error(
                "Control received an incorrect number of arguments.",
            ))
        }
    };
    match first {
        ":views" => {
            exact(1)?;
            return Ok(Some(run_view(
                session,
                ":views",
                discover_output_views(&session.active_interface().definition),
            )));
        }
        ":render" => {
            exact(2)?;
            let name = &tokens[1].text;
            require_output_view(&session.active_interface().definition, name)?;
            let invocation = session
                .active_interface()
                .last_invocation
                .as_ref()
                .ok_or_else(|| {
                    crate::cli_output_view::output_view_error(
                        "NO_OUTPUT_RESULT",
                        "There is no retained invocation result to render.",
                    )
                })?;
            let mut response = run_view(
                session,
                ":render",
                json!({"historical":true,"invocation":invocation}),
            );
            attach_output_presentation(&session.active_interface().definition, name, &mut response);
            return Ok(Some(response));
        }
        ":exit" => {
            exact(1)?;
            return Ok(None);
        }
        ":tree" => {
            exact(1)?;
            let text =
                render_run_verb_tree(&session.active_interface().definition, &session.routes);
            return Ok(Some(run_view(
                session,
                ":tree",
                json!({"verb_tree_text":text}),
            )));
        }
        ":status" => {
            exact(1)?;
            let request = run_request(session, "discover", json!({}));
            return Ok(Some(session.runtime.execute_authoring_request(request)));
        }
        ":up" | ":top" | ":export" => {
            exact(if first == ":export" { 2 } else { 1 })?;
            let (operation, arguments) = match first {
                ":export" => ("export-interface", json!({"destination":tokens[1].text})),
                ":top" => ("enter", json!({"context":"/"})),
                _ => ("enter", json!({"context":".."})),
            };
            let request = run_request(session, operation, arguments);
            return Ok(Some(session.runtime.execute_authoring_request(request)));
        }
        _ => {}
    }
    let help_only = first == ":help";
    let words = if help_only && !tokens.is_empty() {
        &tokens[1..]
    } else {
        tokens
    };
    let target =
        session
            .routes
            .resolve_run_target(&session.active_interface().definition, &start, words)?;
    match target {
        CliRunTarget::Context { id, help } => {
            if one_shot || help_only || help || words.is_empty() {
                return Ok(Some(run_help(session, &id, None)));
            }
            let request = run_request(session, "enter", json!({"context":id}));
            Ok(Some(session.runtime.execute_authoring_request(request)))
        }
        CliRunTarget::Command {
            context,
            word,
            arguments_start,
        } => {
            let arguments = &words[arguments_start..];
            if help_only {
                if !arguments.is_empty() {
                    return Err(run_usage_error(
                        "Help expects a context or command path without argument values.",
                    ));
                }
                return Ok(Some(run_help(session, &context, Some(&word))));
            }
            if arguments.len() == 1 && arguments[0].text == "--help" {
                return Ok(Some(run_help(session, &context, Some(&word))));
            }
            let arguments = if arguments.first().is_some_and(|arg| arg.text == "--") {
                &arguments[1..]
            } else {
                arguments
            };
            let target = format!("{context}/{word}");
            let view = select_command_output_view(
                &session.active_interface().definition,
                &session.active_interface().definition.contexts[&context].commands[&word],
                &session.output_selection,
            )?;
            let values = compile_cli_terminal_arguments(session.active_interface(), &target, arguments)
                .map_err(|mut error| {
                    if error.code == "INVALID_CLI_ARGUMENTS" {
                        error.recovery = "Use :help followed by the command path to inspect its ordered required parameters.".into();
                    }
                    error
                })?;
            let request = run_request(session, "invoke", json!({"command":target,"values":values}));
            let mut response = session.runtime.execute_authoring_request(request);
            if let Some(view) = view {
                attach_output_presentation(
                    &session.active_interface().definition,
                    &view,
                    &mut response,
                );
            }
            Ok(Some(response))
        }
    }
}

/// Delivery errors after publication stop the transport; they cannot masquerade as rejected edits.
pub fn write_run_response(
    session: Option<&CliRunSession>,
    response: &AuthoringResponse,
    structured: bool,
    output: &mut impl Write,
    errors: &mut impl Write,
) -> io::Result<i32> {
    let bytes = checked_response_bytes(response).map_err(io::Error::other)?;
    if response.status == "ok"
        && response.mutation == "applied"
        && !response.replayed
        && let Some(session) = session
    {
        session.runtime.before_response_delivery()?;
    }
    if let Some(error) = &response.error {
        if structured {
            errors.write_all(&bytes)?;
        } else {
            writeln!(errors, "{}", escape_run_metadata(&error.to_string()))?;
        }
        errors.flush()?;
        return Ok(session
            .and_then(|session| session.exit_codes.get(&error.code).copied())
            .unwrap_or_else(|| run_error_exit(error)));
    }
    let result = response.result.as_ref().expect("Successful result");
    let presentation_failed = result["presentation"]["status"] == "error";
    if structured {
        output.write_all(&bytes)?;
    } else {
        let invocation = &result["invocation"];
        if result["historical"] == true {
            writeln!(errors, "[historical] retained result; no command invoked")?;
        }
        if invocation["binding"] == "simulated" {
            writeln!(
                errors,
                "[simulated] {} {}",
                invocation["context"].as_str().unwrap(),
                invocation["operation_id"].as_str().unwrap()
            )?;
            errors.flush()?;
        }
        if presentation_failed {
            let error = &result["presentation"]["error"];
            writeln!(
                errors,
                "{}: {} Invocation outcome is retained; rendering failed. Inspect :status or use :render with a compatible view; do not repeat the invocation to repair its display.",
                escape_run_metadata(error["code"].as_str().unwrap_or("OUTPUT_VIEW_FAILED")),
                escape_run_metadata(error["message"].as_str().unwrap_or("Output view failed."))
            )?;
            errors.flush()?;
            return Ok(session
                .and_then(|session| session.exit_codes.get("OUTPUT_VIEW_FAILED").copied())
                .unwrap_or(1));
        }
        let text = if let Some(text) = result["presentation"]["table_text"].as_str() {
            text.to_owned()
        } else if let Some(text) = invocation["output_json_text"].as_str() {
            format!("{text}\n")
        } else if let Some(text) = result["help_text"]
            .as_str()
            .or_else(|| result["verb_tree_text"].as_str())
        {
            text.to_owned()
        } else if response.operation.as_deref() == Some("enter") {
            String::new()
        } else {
            format!("{}\n", serde_json::to_string_pretty(result).unwrap())
        };
        // Pretty indentation can expand a bounded response; use bounded compact JSON in that case.
        let text = if text.len() > MAX_RESPONSE_BYTES {
            format!("{result}\n")
        } else {
            text
        };
        output.write_all(text.as_bytes())?;
    }
    output.flush()?;
    Ok(if presentation_failed {
        session
            .and_then(|session| session.exit_codes.get("OUTPUT_VIEW_FAILED").copied())
            .unwrap_or(1)
    } else {
        0
    })
}

/// Return exit intent separately from status; every frontend uses this exact dispatch function.
pub fn execute_run_tokens(
    session: &mut CliRunSession,
    tokens: &[TerminalToken],
    one_shot: bool,
    structured: bool,
    output: &mut impl Write,
    errors: &mut impl Write,
) -> io::Result<(i32, bool)> {
    let mut response = match dispatch_run_tokens(session, tokens, one_shot) {
        Ok(None) => return Ok((0, true)),
        Ok(Some(response)) => response,
        Err(error) => session.runtime.rejected_terminal_input(None, error),
    };
    if session.context_help
        && response.status == "ok"
        && response.operation.as_deref() == Some("enter")
    {
        let help = run_help(session, &session.active_interface().context, None);
        response.result.as_mut().unwrap()["help_text"] = help.result.unwrap()["help_text"].clone();
    }
    let code = write_run_response(Some(session), &response, structured, output, errors)?;
    Ok((
        code,
        response
            .error
            .as_ref()
            .is_some_and(AuthoringError::publication_uncertain)
            || session.runtime.session_is_uncertain(),
    ))
}

fn execute_run_line(
    session: &mut CliRunSession,
    line: Result<Vec<u8>, AuthoringError>,
    structured: bool,
    output: &mut impl Write,
    errors: &mut impl Write,
) -> io::Result<(i32, bool)> {
    let tokens = line.and_then(|bytes| {
        let line = std::str::from_utf8(&bytes)
            .map_err(|_| run_usage_error("Command input must be UTF-8."))?;
        tokenize_terminal_input(line)
    });
    match tokens {
        Ok(tokens) if tokens.is_empty() => Ok((0, false)),
        Ok(tokens) => execute_run_tokens(session, &tokens, false, structured, output, errors),
        Err(error) => Ok((
            write_run_response(
                Some(session),
                &session.runtime.rejected_terminal_input(None, error),
                structured,
                output,
                errors,
            )?,
            false,
        )),
    }
}

pub fn run_cli_stream(
    session: &mut CliRunSession,
    structured: bool,
    input: &mut impl BufRead,
    output: &mut impl Write,
    errors: &mut impl Write,
) -> io::Result<i32> {
    let mut exit = 0;
    let mut seen = false;
    while let Some(line) = read_authoring_line(input)? {
        seen = true;
        let (code, stop) = execute_run_line(session, line, structured, output, errors)?;
        exit = exit.max(code);
        if stop {
            break;
        }
    }
    if !seen {
        exit = execute_run_tokens(session, &[], true, structured, output, errors)?.0;
    }
    Ok(exit)
}

pub fn run_cli_terminal(session: &mut CliRunSession, structured: bool) -> io::Result<i32> {
    let completer = crate::cli_run_completion::CliRunCompleter::new_run_completer(
        session.active_interface().clone(),
        session.routes.clone(),
    );
    let menu = ColumnarMenu::default().with_name("run_completion");
    let mut keys = default_emacs_keybindings();
    keys.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("run_completion".into()),
            ReedlineEvent::MenuNext,
        ]),
    );
    let mut editor = Reedline::create()
        .with_completer(Box::new(completer.clone()))
        .with_menu(ReedlineMenu::EngineCompleter(Box::new(menu)))
        .with_edit_mode(Box::new(Emacs::new(keys)));
    let mut exit = 0;
    loop {
        let active = session.active_interface();
        completer.refresh_run_completion(active);
        let path = session
            .routes
            .context_path(&active.definition, &active.context);
        let prompt = DefaultPrompt::new(
            DefaultPromptSegment::Basic(format!(
                "{} [{}]",
                active.definition.id,
                if path.is_empty() { "/" } else { &path }
            )),
            DefaultPromptSegment::Empty,
        );
        match editor.read_line(&prompt)? {
            Signal::Success(line) => {
                let (code, stop) = execute_run_line(
                    session,
                    Ok(line.into_bytes()),
                    structured,
                    &mut io::stdout().lock(),
                    &mut io::stderr().lock(),
                )?;
                exit = exit.max(code);
                if stop {
                    break;
                }
            }
            Signal::CtrlD => break,
            Signal::CtrlC => {}
            _ => {}
        }
    }
    Ok(exit)
}
