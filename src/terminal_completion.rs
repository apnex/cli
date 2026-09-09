//! Interactive completion reads declaration metadata and the same typed child enumeration as requests.

use crate::authoring_protocol::SessionCheckpoint;
use crate::document_path::{
    document_child_locations, parse_terminal_document_path, read_document_path,
    render_document_pointer,
};
use crate::document_value::MAX_REQUEST_BYTES;
use crate::operation_definition::{DeclaredTerminalForm, HandlerArgumentKind, OperationDefinition};
use crate::terminal_input::{quote_terminal_token, terminal_completion_fragment};
use reedline::{Completer, CompletionResult, Span, Suggestion};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// The editor reads an acknowledged snapshot, refreshed only after canonical dispatch.
#[derive(Clone)]
pub struct AuthoringCompleter {
    definition: OperationDefinition,
    snapshot: Arc<Mutex<SessionCheckpoint>>,
    pending_batch: Arc<AtomicBool>,
}

impl AuthoringCompleter {
    /// Bind discovery to the loaded operation declaration and the initial session state.
    pub fn new_authoring_completer(
        definition: OperationDefinition,
        state: SessionCheckpoint,
    ) -> Self {
        Self {
            definition,
            snapshot: Arc::new(Mutex::new(state)),
            pending_batch: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Refresh document-aware completion without giving the editor access to authoritative mutation.
    pub fn refresh_completion_context(&self, state: &SessionCheckpoint) {
        *self.snapshot.lock().expect("Completion snapshot mutex") = state.clone();
    }

    /// Mark transient batch completion so the menu exposes submission, cancellation, and primitives.
    pub fn set_batch_completion(&self, pending: bool) {
        self.pending_batch.store(pending, Ordering::Relaxed);
    }

    fn document_path_suggestions(&self, prefix: &str) -> Vec<(String, String)> {
        let (parent, stem, filter) = if prefix.is_empty() {
            (".", "./", "")
        } else if let Some(slash) = prefix.rfind('/') {
            (&prefix[..slash], &prefix[..=slash], &prefix[slash + 1..])
        } else if prefix == "." {
            (".", "./", "")
        } else {
            return Vec::new();
        };
        let snapshot = self
            .snapshot
            .lock()
            .expect("Completion snapshot mutex")
            .clone();
        let Ok(path) = parse_terminal_document_path(parent, &snapshot.candidate, &snapshot.context)
        else {
            return Vec::new();
        };
        let Ok(node) = read_document_path(
            &snapshot.candidate,
            &path.absolute_segments(&snapshot.context),
        ) else {
            return Vec::new();
        };
        let mut children = std::collections::BTreeMap::new();
        for (segment, value) in document_child_locations(node) {
            let escaped = render_document_pointer(&[segment]);
            children.insert(escaped[1..].to_owned(), value.document_kind().to_owned());
        }
        if let Some(constraint) = &snapshot.active_constraint
            && let Ok(guidance) = crate::schema_guidance::describe_schema_path(
                constraint,
                &path.absolute_segments(&snapshot.context),
            )
        {
            for child in guidance.children {
                let escaped = crate::schema_constraint::escape_schema_component(&child.key);
                children.entry(escaped).or_insert_with(|| {
                    format!(
                        "schema: {}{}",
                        child.types.join(" or "),
                        if child.required { "; required" } else { "" }
                    )
                });
            }
        }
        let mut suggestions = Vec::new();
        let mut bytes = 0;
        for (component, description) in children {
            if component.starts_with(filter) {
                let suggestion = format!("{stem}{component}");
                let size = quote_terminal_token(&suggestion).len();
                if suggestions.len() == 100 || bytes + size > MAX_REQUEST_BYTES {
                    suggestions.push((
                        prefix.into(),
                        "More matching paths; narrow the prefix or use paged complete.".into(),
                    ));
                    break;
                }
                bytes += size;
                suggestions.push((suggestion, description));
            }
        }
        suggestions
    }

    fn constructor_schema_guidance(
        &self,
        handler: &str,
        previous: &[crate::terminal_input::TerminalToken],
    ) -> Option<crate::schema_guidance::SchemaPathGuidance> {
        if !["authoring.set", "authoring.append", "authoring.insert"].contains(&handler) {
            return None;
        }
        let snapshot = self.snapshot.lock().expect("Completion snapshot mutex");
        let constraint = snapshot.active_constraint.as_ref()?;
        let mut path = parse_terminal_document_path(
            &previous.get(1)?.text,
            &snapshot.candidate,
            &snapshot.context,
        )
        .ok()?
        .absolute_segments(&snapshot.context);
        if handler == "authoring.append" {
            let crate::document_value::DocumentValue::Array(array) =
                read_document_path(&snapshot.candidate, &path).ok()?
            else {
                return None;
            };
            path.push(crate::document_path::DocumentSegment::Index { index: array.len() });
        } else if handler == "authoring.insert" {
            path.push(crate::document_path::DocumentSegment::Index {
                index: previous.get(2)?.text.parse().ok()?,
            });
        }
        crate::schema_guidance::describe_schema_path(constraint, &path).ok()
    }
}

impl Completer for AuthoringCompleter {
    fn complete(&mut self, line: &str, position: usize) -> CompletionResult {
        let Some((previous, prefix, start)) = terminal_completion_fragment(line, position) else {
            return CompletionResult::fresh(Vec::new());
        };
        let mut choices: Vec<(String, String)> = Vec::new();
        let mut configured_choices = false;
        if previous.is_empty() {
            for operation in self.definition.operations.values() {
                if self.pending_batch.load(Ordering::Relaxed) {
                    if let DeclaredTerminalForm::Block { submit,cancel,.. } = &operation.terminal {
                        for (word,help) in [(submit,"Submit the unsent batch"),(cancel,"Cancel the unsent batch")] { if word.starts_with(&prefix) { choices.push((word.clone(),help.into())); } }
                    }
                    if !operation.batchable { continue; }
                }
                let command = match &operation.terminal { DeclaredTerminalForm::Tokens { command,.. } => command, DeclaredTerminalForm::Block { start,.. } => start };
                if command.starts_with(&prefix) { choices.push((command.clone(),operation.help.clone())); }
            }
        } else if let Some(operation) = self.definition.operations.values().find(|operation|matches!(&operation.terminal,DeclaredTerminalForm::Tokens { command,.. } if command == &previous[0].text)) {
            if operation.handler == "composition.invoke" || operation.handler == "composition.enter" {
                configured_choices = true;
                let snapshot = self.snapshot.lock().expect("Completion snapshot mutex");
                if let Some(active) = &snapshot.active_interface {
                    choices = if operation.handler == "composition.invoke" {
                        crate::cli_terminal::complete_cli_invocation(active, &previous)
                    } else if previous.len() == 1 {
                        active.definition.contexts.iter().map(|(id,context)|(id.clone(),context.help.clone())).chain([("/".into(),"Root interface context".into()),("..".into(),"Parent interface context".into())]).collect()
                    } else { Vec::new() };
                }
            } else {
            let schema_guidance = self.constructor_schema_guidance(&operation.handler, &previous);
            let positional = match &operation.terminal { DeclaredTerminalForm::Tokens { positional,.. } => positional, _ => unreachable!() };
            let mut consumed = 1;
            for name in positional {
                let argument = operation.arguments.iter().find(|argument|&argument.name == name).unwrap();
                let kind = self.definition.argument_kind(argument).expect("Validated argument codec");
                if consumed == previous.len() {
                    choices = match kind {
                        HandlerArgumentKind::DocumentPath => self.document_path_suggestions(&prefix),
                        HandlerArgumentKind::ValueConstructor => {
                            if let Some(guidance) = &schema_guidance && !guidance.types.is_empty() {
                                guidance.types.iter().map(|kind| (if kind == "integer" { "number".into() } else { kind.clone() }, "schema type suggestion; validate the complete draft".into())).collect::<std::collections::BTreeMap<_,_>>().into_iter().collect()
                            } else { self.definition.operation_help(None).unwrap()["value_kinds"].as_array().unwrap().iter().map(|kind|(kind.as_str().unwrap().into(),"value kind".into())).collect() }
                        },
                        HandlerArgumentKind::DocumentView => argument.values.as_ref().unwrap().iter().map(|value|(value.clone(),"document view".into())).collect(),
                        HandlerArgumentKind::Text if operation.handler == "authoring.help" => self.definition.operations.values().map(|operation|(operation.name.clone(),operation.help.clone())).collect(),
                        _ => Vec::new(),
                    };
                    break;
                }
                if kind == HandlerArgumentKind::ValueConstructor && ["string","number","boolean"].contains(&previous[consumed].text.as_str()) {
                    if consumed+1 == previous.len() {
                        if let Some(guidance) = &schema_guidance {
                            configured_choices = true;
                            choices = guidance.values.iter().filter(|value| value["kind"].as_str() == Some(&previous[consumed].text)).filter_map(|value| {
                                let text = value["value"].as_str().map(str::to_owned).or_else(|| value["value"].as_bool().map(|value| value.to_string()))?;
                                Some((text, "schema literal suggestion; validate the complete draft".into()))
                            }).collect();
                        }
                        if choices.is_empty() && previous[consumed].text == "boolean" { choices = vec![("true".into(),"boolean".into()),("false".into(),"boolean".into())]; }
                        break;
                    }
                    consumed += 2;
                } else { consumed += 1; }
                if consumed > previous.len() { break; }
            }
            }
        }
        if configured_choices {
            let mut bounded = Vec::new();
            let mut byte_count = 0;
            for (value, description) in choices
                .into_iter()
                .filter(|(value, _)| value.starts_with(&prefix))
            {
                let size =
                    quote_terminal_token(&value).len() + quote_terminal_token(&description).len();
                if bounded.len() == 100 || byte_count + size > MAX_REQUEST_BYTES {
                    bounded.push((
                        prefix.clone(),
                        "More configured choices; narrow the prefix or use discover.".into(),
                    ));
                    break;
                }
                byte_count += size;
                bounded.push((value, description));
            }
            choices = bounded;
        }
        CompletionResult::fresh(
            choices
                .into_iter()
                .filter(|(value, _)| prefix.is_empty() || value.starts_with(&prefix))
                .map(|(value, description)| Suggestion {
                    value: quote_terminal_token(&value),
                    description: Some(quote_terminal_token(&description)),
                    span: Span::new(start, position),
                    append_whitespace: true,
                    ..Suggestion::default()
                })
                .collect::<Vec<_>>(),
        )
    }
}
