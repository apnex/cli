//! Terminal tokens compile through the loaded declaration into the same requests as machine input.

use crate::authoring_error::{AuthoringError, authoring_limit_error};
use crate::authoring_protocol::{
    AuthoringRequest, RequestIdentity, SessionCheckpoint, SessionRevision,
};
use crate::document_path::parse_terminal_document_path;
use crate::document_value::{MAX_BATCH_EDITS, MAX_REQUEST_BYTES};
use crate::operation_definition::{DeclaredTerminalForm, HandlerArgumentKind, OperationDefinition};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// A terminal token retains byte positions so completion replaces exactly the token being edited.
#[derive(Clone, Debug)]
pub struct TerminalToken {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

/// Split completed tokens from a possibly unfinished quoted token at a UTF-8 cursor boundary.
pub fn terminal_completion_fragment(
    line: &str,
    position: usize,
) -> Option<(Vec<TerminalToken>, String, usize)> {
    if line.len() >= MAX_REQUEST_BYTES || !line.is_char_boundary(position) {
        return None;
    }
    let before = &line[..position];
    let (mut quoted, mut escaped, mut start) = (false, false, 0);
    for (index, character) in before.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && character == '\\' {
            escaped = true;
            continue;
        }
        if character == '"' {
            quoted = !quoted;
        }
        if !quoted && character.is_ascii_whitespace() {
            start = index + character.len_utf8();
        }
    }
    let previous = tokenize_terminal_input(&before[..start]).ok()?;
    let raw = &before[start..];
    let prefix = if raw.starts_with('"') {
        serde_json::from_str(&if quoted {
            format!("{raw}\"")
        } else {
            raw.into()
        })
        .ok()?
    } else {
        raw.into()
    };
    Some((previous, prefix, start))
}

fn terminal_syntax_error(message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        "INVALID_REQUEST",
        message,
        "Use whitespace-separated tokens and JSON string quoting for whitespace or control characters.",
    )
}

/// Tokenize without expansion or shell interpretation, decoding only complete quoted JSON strings.
pub fn tokenize_terminal_input(line: &str) -> Result<Vec<TerminalToken>, AuthoringError> {
    if line.len() + 1 > MAX_REQUEST_BYTES {
        return Err(authoring_limit_error(
            "Terminal input exceeds the 2 MiB line limit.",
        ));
    }
    let bytes = line.as_bytes();
    let mut tokens = Vec::new();
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position].is_ascii_whitespace() {
            position += 1;
            continue;
        }
        let start = position;
        let text;
        if bytes[position] == b'"' {
            position += 1;
            let mut closed = false;
            while position < bytes.len() {
                match bytes[position] {
                    b'\\' => position = (position + 2).min(bytes.len()),
                    b'"' => {
                        position += 1;
                        closed = true;
                        break;
                    }
                    _ => position += 1,
                }
            }
            if !closed {
                return Err(terminal_syntax_error(
                    "Terminal quoted token is unfinished.",
                ));
            }
            text = serde_json::from_str(&line[start..position]).map_err(|_| {
                terminal_syntax_error("Terminal quoted token has invalid JSON escapes or Unicode.")
            })?;
            if position < bytes.len() && !bytes[position].is_ascii_whitespace() {
                return Err(terminal_syntax_error(
                    "Terminal quoted token must end at whitespace or end of input.",
                ));
            }
        } else {
            while position < bytes.len() && !bytes[position].is_ascii_whitespace() {
                if bytes[position] == b'"' {
                    return Err(terminal_syntax_error(
                        "Terminal quotes must enclose the complete token.",
                    ));
                }
                position += 1;
            }
            text = line[start..position].to_owned();
        }
        tokens.push(TerminalToken {
            text,
            start,
            end: position,
        });
    }
    Ok(tokens)
}

/// Quote a complete token when literal insertion could change parsing or terminal presentation.
pub fn quote_terminal_token(text: &str) -> String {
    if text.is_empty()
        || text
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '"' || c == '\\')
    {
        serde_json::to_string(text).expect("Terminal strings serialize")
    } else {
        text.into()
    }
}

/// Compile a terminal operation using declaration-owned positional binding and registered codecs.
pub fn compile_terminal_request(
    line: &str,
    definition: &OperationDefinition,
    state: &SessionCheckpoint,
) -> Result<AuthoringRequest, AuthoringError> {
    let tokens = tokenize_terminal_input(line)?;
    let command = tokens
        .first()
        .ok_or_else(|| terminal_syntax_error("Terminal operation is empty."))?;
    let operation = definition.operations.values().find(|operation|matches!(&operation.terminal,DeclaredTerminalForm::Tokens { command:word,.. } if word == &command.text))
        .ok_or_else(||AuthoringError::new("UNKNOWN_OPERATION","Terminal operation is not declared.","Inspect help for available commands."))?;
    let positional = match &operation.terminal {
        DeclaredTerminalForm::Tokens { positional, .. } => positional,
        _ => unreachable!(),
    };
    let mut arguments = BTreeMap::new();
    let mut position = 1;
    for name in positional {
        let argument = operation
            .arguments
            .iter()
            .find(|argument| &argument.name == name)
            .expect("Validated positional declaration");
        if definition.argument_kind(argument)? == HandlerArgumentKind::CliArguments {
            let active = state
                .active_interface
                .as_ref()
                .ok_or_else(crate::cli_interface::inactive_cli_error)?;
            let word = arguments
                .get("command")
                .and_then(Value::as_str)
                .ok_or_else(|| terminal_syntax_error("CLI invocation requires a command word."))?;
            let values = crate::cli_terminal::compile_cli_terminal_arguments(
                active,
                word,
                &tokens[position..],
            )?;
            arguments.insert(name.clone(), values);
            position = tokens.len();
            continue;
        }
        if position == tokens.len() {
            break;
        }
        let token = &tokens[position].text;
        position += 1;
        let value = match definition.argument_kind(argument)? {
            HandlerArgumentKind::Text | HandlerArgumentKind::DocumentView => json!(token),
            HandlerArgumentKind::Unsigned { .. } => {
                let number: usize = token.parse().map_err(|_| {
                    terminal_syntax_error("Terminal integer argument must be nonnegative.")
                })?;
                if number.to_string() != *token {
                    return Err(terminal_syntax_error(
                        "Terminal integer argument must use canonical decimal spelling.",
                    ));
                }
                json!(number)
            }
            HandlerArgumentKind::DocumentPath => json!(parse_terminal_document_path(
                token,
                &state.candidate,
                &state.context
            )?),
            HandlerArgumentKind::ValueConstructor => match token.as_str() {
                "object" | "array" | "null" => json!({"kind":token}),
                "string" | "number" | "boolean" => {
                    let payload = tokens.get(position).ok_or_else(|| {
                        terminal_syntax_error("Terminal value constructor is missing its payload.")
                    })?;
                    position += 1;
                    if token == "boolean" {
                        let value: bool = payload.text.parse().map_err(|_| {
                            AuthoringError::new(
                                "INVALID_VALUE",
                                "Terminal boolean constructor requires true or false.",
                                "Supply a boolean literal after the boolean kind.",
                            )
                        })?;
                        json!({"kind":token,"value":value})
                    } else {
                        json!({"kind":token,"value":payload.text})
                    }
                }
                _ => {
                    return Err(AuthoringError::new(
                        "INVALID_VALUE",
                        "Terminal value constructor kind is not supported.",
                        "Choose object, array, string, number, boolean, or null.",
                    ));
                }
            },
            HandlerArgumentKind::PrimitiveEdits => {
                return Err(terminal_syntax_error(
                    "Terminal primitive edits require declared batch framing.",
                ));
            }
            HandlerArgumentKind::CliArguments => {
                unreachable!("CLI arguments consume the remaining positional tokens")
            }
        };
        arguments.insert(name.clone(), value);
    }
    if position != tokens.len() {
        return Err(terminal_syntax_error(
            "Terminal operation has extra positional tokens.",
        ));
    }
    Ok(AuthoringRequest {
        request_id: RequestIdentity::new_request_identity(),
        session_id: state.session_id.clone(),
        operation: operation.name.clone(),
        arguments,
        expected_revision: (operation.effect != "read").then_some(state.revision),
    })
}

/// Terminal input may submit a request, report unsent framing, or reject syntax without mutation.
pub enum TerminalInputOutcome {
    Request(AuthoringRequest),
    InputEvent(Value),
    Rejected {
        operation: Option<String>,
        error: AuthoringError,
    },
    Empty,
}

struct PendingTerminalBatch {
    operation: String,
    expected_revision: SessionRevision,
    preview: SessionCheckpoint,
    submit: String,
    cancel: String,
    edits: Vec<Value>,
    first_error: Option<AuthoringError>,
}

/// An unsent batch buffer never owns or persists authoritative authoring state.
#[derive(Default)]
pub struct TerminalInputBuffer {
    pending: Option<PendingTerminalBatch>,
}

impl TerminalInputBuffer {
    /// Expose the unsent preview for completion while retaining the authoritative checkpoint separately.
    pub fn completion_checkpoint<'a>(
        &'a self,
        acknowledged: &'a SessionCheckpoint,
    ) -> &'a SessionCheckpoint {
        self.pending
            .as_ref()
            .map(|pending| &pending.preview)
            .unwrap_or(acknowledged)
    }
    /// Return the unsent edit count for a prompt that cannot be confused with saved draft state.
    pub fn pending_edit_count(&self) -> Option<usize> {
        self.pending.as_ref().map(|pending| pending.edits.len())
    }

    /// Keep framing errors attached to an unsent batch so end cannot publish a valid prefix.
    pub fn reject_input_line(&mut self, error: AuthoringError) -> TerminalInputOutcome {
        if let Some(pending) = &mut self.pending {
            let error = error.at_batch_index(pending.edits.len());
            if pending.first_error.is_none() {
                pending.first_error = Some(error.clone());
            }
            return TerminalInputOutcome::Rejected {
                operation: Some(pending.operation.clone()),
                error,
            };
        }
        TerminalInputOutcome::Rejected {
            operation: None,
            error,
        }
    }

    /// Cancel transient batch input without submitting any state operation.
    pub fn cancel_pending_input(&mut self) -> Value {
        self.pending = None;
        json!({"event":"batch_input","state":"cancelled","count":0})
    }

    /// Compile ordinary commands or assemble a declared batch while retaining its starting revision.
    pub fn accept_terminal_line(
        &mut self,
        line: &str,
        definition: &OperationDefinition,
        state: &SessionCheckpoint,
    ) -> TerminalInputOutcome {
        if line.trim().is_empty() {
            return TerminalInputOutcome::Empty;
        }
        let tokens = match tokenize_terminal_input(line) {
            Ok(tokens) => tokens,
            Err(error) => {
                if let Some(pending) = &mut self.pending {
                    if pending.first_error.is_none() {
                        pending.first_error =
                            Some(error.clone().at_batch_index(pending.edits.len()));
                    }
                }
                return TerminalInputOutcome::Rejected {
                    operation: self
                        .pending
                        .as_ref()
                        .map(|pending| pending.operation.clone()),
                    error,
                };
            }
        };
        if let Some(mut pending) = self.pending.take() {
            if tokens.len() == 1 && tokens[0].text == pending.cancel {
                return TerminalInputOutcome::InputEvent(
                    json!({"event":"batch_input","state":"cancelled","count":0}),
                );
            }
            if tokens.len() == 1 && tokens[0].text == pending.submit {
                if let Some(error) = pending.first_error {
                    return TerminalInputOutcome::Rejected {
                        operation: Some(pending.operation),
                        error,
                    };
                }
                let request = AuthoringRequest {
                    request_id: RequestIdentity::new_request_identity(),
                    session_id: state.session_id.clone(),
                    operation: pending.operation,
                    arguments: BTreeMap::from([("operations".into(), json!(pending.edits))]),
                    expected_revision: Some(pending.expected_revision),
                };
                return TerminalInputOutcome::Request(request);
            }
            let index = pending.edits.len();
            if pending.first_error.is_none() {
                let compile = || -> Result<(Value, SessionCheckpoint), AuthoringError> {
                    if index >= MAX_BATCH_EDITS {
                        return Err(authoring_limit_error(
                            "Terminal batch exceeds 256 primitive edits.",
                        ));
                    }
                    let request = compile_terminal_request(line, definition, &pending.preview)?;
                    let operation = &definition.operations[&request.operation];
                    if !operation.batchable {
                        return Err(terminal_syntax_error(
                            "Terminal batch accepts only declared primitive edits.",
                        ));
                    }
                    let arguments =
                        definition.validate_operation_arguments(operation, &request.arguments)?;
                    if arguments.values().any(|argument|matches!(argument,crate::operation_definition::ValidatedArgument::Path(path) if path.base != crate::document_path::DocumentPathBase::Root)) { return Err(AuthoringError::new("INVALID_PATH","Terminal batch paths must be root-based.","Use absolute pointers inside the batch.")); }
                    let edit = json!({"operation":request.operation,"arguments":request.arguments});
                    let mut preview = pending.preview.clone();
                    definition.invoke_authoring_handler(operation, &mut preview, &arguments)?;
                    Ok((edit, preview))
                };
                match compile() {
                    Ok((edit, preview)) => {
                        pending.edits.push(edit);
                        if serde_json::to_vec(&pending.edits).unwrap().len() > MAX_REQUEST_BYTES {
                            pending.first_error = Some(
                                authoring_limit_error(
                                    "Terminal batch buffer exceeds the request byte limit.",
                                )
                                .at_batch_index(index),
                            );
                        } else {
                            pending.preview = preview;
                        }
                    }
                    Err(error) => pending.first_error = Some(error.at_batch_index(index)),
                }
            }
            let event = json!({"event":"batch_input","state":"unsent","count":pending.edits.len(),"revision":pending.expected_revision,"error":pending.first_error});
            self.pending = Some(pending);
            return TerminalInputOutcome::InputEvent(event);
        }
        if let Some(operation) = definition.operations.values().find(|operation|matches!(&operation.terminal,DeclaredTerminalForm::Block { start,.. } if start == &tokens[0].text)) {
            if tokens.len() != 1 { return TerminalInputOutcome::Rejected { operation:Some(operation.name.clone()),error:terminal_syntax_error("Terminal batch start takes no positional arguments.") }; }
            let (submit,cancel) = match &operation.terminal { DeclaredTerminalForm::Block { submit,cancel,.. } => (submit.clone(),cancel.clone()), _ => unreachable!() };
            self.pending = Some(PendingTerminalBatch { operation:operation.name.clone(),expected_revision:state.revision,preview:state.clone(),submit,cancel,edits:Vec::new(),first_error:None });
            return TerminalInputOutcome::InputEvent(json!({"event":"batch_input","state":"unsent","count":0,"revision":state.revision}));
        }
        match compile_terminal_request(line,definition,state) {
            Ok(request) => TerminalInputOutcome::Request(request),
            Err(error) => TerminalInputOutcome::Rejected { operation:definition.operations.values().find(|operation|matches!(&operation.terminal,DeclaredTerminalForm::Tokens { command,.. } if command == &tokens[0].text)).map(|operation|operation.name.clone()),error },
        }
    }
}
