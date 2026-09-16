//! Terminal suggestions follow the same routes as execution and refresh only acknowledged context.

use crate::cli_interface::ActiveCliInterface;
use crate::cli_run_frontend::escape_run_metadata;
use crate::cli_run_routes::{CliRunRoutes, CliRunTarget};
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
        let control = previous.first().map(|token| {
            self.routes
                .control_aliases
                .get(&token.text)
                .unwrap_or(&token.text)
                .as_str()
        });
        let help = control == Some(":help");
        let words = if help { &previous[1..] } else { &previous[..] };
        let mut choices = Vec::new();
        if control == Some(":endpoint")
            && self.routes.endpoint_control
            && ((previous.len() == 2 && previous[1].text == "clear")
                || (previous.len() == 3 && previous[1].text == "set"))
        {
            choices.push((
                "--save".into(),
                "Save this change for future launches".into(),
            ));
        }
        if previous.len() == 1 && control == Some(":endpoint") && self.routes.endpoint_control {
            choices.extend(
                crate::cli_operator_settings::CLI_ENDPOINT_ACTIONS
                    .map(|(word, help)| (word.into(), help.into())),
            );
        }
        if previous.len() == 1
            && control == Some(":render")
            && let Some(views) = &active.definition.views
        {
            choices.extend(
                views
                    .iter()
                    .map(|(id, view)| (id.clone(), view.help.clone())),
            );
        }
        if previous.is_empty() || (help && previous.len() == 1 && self.routes.operator.is_some()) {
            choices.extend(self.routes.control_aliases.iter().map(|(alias, target)| {
                (
                    alias.clone(),
                    self.routes
                        .available_run_controls()
                        .find(|(control, _)| control == target)
                        .map(|(_, description)| description.to_owned())
                        .unwrap_or_default(),
                )
            }));
            choices.extend(
                self.routes
                    .available_run_controls()
                    .filter(|_| self.routes.operator.is_none() || prefix.starts_with(':'))
                    .map(|(word, description)| (word.to_owned(), description.to_owned())),
            );
        }
        if let Ok(target) =
            self.routes
                .resolve_run_target(&active.definition, &active.context, words)
        {
            match target {
                CliRunTarget::Context { id, help: false } => {
                    if let Some(profile) = &self.routes.operator {
                        choices.extend(
                            profile
                                .context_listing
                                .iter()
                                .filter(|word| {
                                    !active.definition.contexts[&id].commands.contains_key(*word)
                                        && !self.routes.context_children(&id).contains_key(*word)
                                })
                                .map(|word| (word.clone(), "List this context".into())),
                        );
                    }
                    choices.extend(self.routes.context_children(&id).iter().map(|(word, id)| {
                        (word.clone(), active.definition.contexts[id].help.clone())
                    }));
                    choices.extend(
                        active.definition.contexts[&id]
                            .commands
                            .iter()
                            .filter(|(_, command)| {
                                id != "root"
                                    || !prefix.is_empty()
                                    || !self.routes.operator.as_ref().is_some_and(|profile| {
                                        profile.command_aliases.contains_key(&command.id)
                                    })
                            })
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
