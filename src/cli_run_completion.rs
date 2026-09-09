//! Terminal suggestions follow the same routes as execution and refresh only acknowledged context.

use crate::cli_interface::ActiveCliInterface;
use crate::cli_run_frontend::escape_run_metadata;
use crate::cli_run_routes::{CliRunRoutes, CliRunTarget, RUN_CONTROLS};
use crate::document_value::MAX_REQUEST_BYTES;
use crate::terminal_input::{quote_terminal_token, terminal_completion_fragment};
use reedline::{Completer, CompletionResult, Span, Suggestion};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct CliRunCompleter {
    active: Arc<Mutex<ActiveCliInterface>>,
    routes: CliRunRoutes,
}

impl CliRunCompleter {
    pub fn new_run_completer(active: ActiveCliInterface, routes: CliRunRoutes) -> Self {
        Self {
            active: Arc::new(Mutex::new(active)),
            routes,
        }
    }
    pub fn refresh_run_completion(&self, active: &ActiveCliInterface) {
        *self.active.lock().expect("Run completion mutex") = active.clone();
    }
}

impl Completer for CliRunCompleter {
    fn complete(&mut self, line: &str, position: usize) -> CompletionResult {
        let Some((previous, prefix, start)) = terminal_completion_fragment(line, position) else {
            return CompletionResult::fresh(Vec::new());
        };
        let active = self.active.lock().expect("Run completion mutex");
        let help = previous.first().is_some_and(|token| token.text == ":help");
        let words = if help { &previous[1..] } else { &previous[..] };
        let mut choices = Vec::new();
        if previous.is_empty() {
            choices.extend(
                RUN_CONTROLS.map(|(word, description)| (word.to_owned(), description.to_owned())),
            );
        }
        if let Ok(target) =
            self.routes
                .resolve_run_target(&active.definition, &active.context, words)
        {
            match target {
                CliRunTarget::Context { id, help: false } => {
                    choices.extend(self.routes.context_children(&id).iter().map(|(word, id)| {
                        (word.clone(), active.definition.contexts[id].help.clone())
                    }));
                    choices.extend(
                        active.definition.contexts[&id]
                            .commands
                            .iter()
                            .map(|(word, command)| (word.clone(), command.help.clone())),
                    );
                }
                CliRunTarget::Command {
                    context,
                    word,
                    arguments_start,
                } if !help => {
                    let arguments = &words[arguments_start..];
                    let arguments = if arguments.first().is_some_and(|token| token.text == "--") {
                        &arguments[1..]
                    } else {
                        arguments
                    };
                    if active.definition.contexts[&context].commands[&word]
                        .parameters
                        .get(arguments.len())
                        .is_some_and(|parameter| parameter.value_type == "boolean")
                    {
                        choices.extend([
                            ("true".into(), "boolean".into()),
                            ("false".into(), "boolean".into()),
                        ]);
                    }
                }
                _ => {}
            }
        }
        let mut suggestions = Vec::new();
        let mut bytes = 0;
        for (value, description) in choices
            .into_iter()
            .filter(|(word, _)| word.starts_with(&prefix))
        {
            let value = quote_terminal_token(&value);
            let description = escape_run_metadata(&description);
            bytes += value.len() + description.len();
            if suggestions.len() == 100 || bytes > MAX_REQUEST_BYTES {
                suggestions.push(Suggestion {
                    value: quote_terminal_token(&prefix),
                    description: Some(
                        "More matching choices; narrow the prefix or use :help.".into(),
                    ),
                    span: Span::new(start, position),
                    ..Suggestion::default()
                });
                break;
            }
            suggestions.push(Suggestion {
                value,
                description: Some(description),
                span: Span::new(start, position),
                append_whitespace: true,
                ..Suggestion::default()
            });
        }
        CompletionResult::fresh(suggestions)
    }
}
