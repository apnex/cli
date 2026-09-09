//! One validated route projection supplies direct parsing, help, trees, and completion.

use crate::authoring_error::AuthoringError;
use crate::cli_definition::{CliContextParent, CliDefinition};
use crate::terminal_input::TerminalToken;
use std::collections::BTreeMap;

/// Context words map to stable identities; assembling changes the words, never stored identities.
#[derive(Clone, Debug)]
pub struct CliRunRoutes {
    children: BTreeMap<String, BTreeMap<String, String>>,
}

/// Parsing stops at a command so its remaining argument strings stay untouched.
#[derive(Debug, PartialEq)]
pub enum CliRunTarget {
    Context {
        id: String,
        help: bool,
    },
    Command {
        context: String,
        word: String,
        arguments_start: usize,
    },
}

pub const RUN_CONTROLS: [(&str, &str); 7] = [
    (":help", "Describe this context or a command"),
    (":tree", "Print the complete configured verb tree"),
    (":up", "Enter the parent context"),
    (":top", "Return to root"),
    (":status", "Inspect state, grants, and historical outcome"),
    (":export", "Create a portable interface export"),
    (":exit", "Close this run"),
];

pub fn run_usage_error(message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        "INVALID_RUN_COMMAND",
        message,
        "Use :help for available contexts, commands, and ordered arguments.",
    )
}

impl CliRunRoutes {
    /// Reject ambiguous surfaces before creating a session or executing any command.
    pub fn from_cli_definition(definition: &CliDefinition) -> Result<Self, AuthoringError> {
        let mut children: BTreeMap<String, BTreeMap<String, String>> = definition
            .contexts
            .keys()
            .map(|id| (id.clone(), BTreeMap::new()))
            .collect();
        for (id, context) in &definition.contexts {
            let CliContextParent::Context(parent) = &context.parent else {
                continue;
            };
            let word = if definition.assembly.is_some() {
                id.strip_prefix(&format!("{parent}.")).unwrap_or(id)
            } else {
                id
            };
            let siblings = children.get_mut(parent).expect("Validated context parent");
            if definition.contexts[parent].commands.contains_key(word)
                || siblings.insert(word.to_owned(), id.clone()).is_some()
            {
                return Err(AuthoringError::new(
                    "AMBIGUOUS_RUN_ROUTE",
                    format!("CLI context {parent} has more than one route named {word}."),
                    "Rename the conflicting command or child context before using this definition in run mode.",
                ));
            }
        }
        Ok(Self { children })
    }

    pub fn context_children(&self, context: &str) -> &BTreeMap<String, String> {
        &self.children[context]
    }

    /// Resolve relative words using the same child names exposed by every generated view.
    pub fn resolve_run_target(
        &self,
        definition: &CliDefinition,
        start: &str,
        tokens: &[TerminalToken],
    ) -> Result<CliRunTarget, AuthoringError> {
        let mut context = start;
        let mut index = 0;
        while index < tokens.len() {
            let literal = tokens[index].text == "--";
            if literal {
                index += 1;
            }
            let word = tokens
                .get(index)
                .ok_or_else(|| {
                    run_usage_error("Argument separator needs a following context or command.")
                })?
                .text
                .as_str();
            if !literal && word == "--help" && index + 1 == tokens.len() {
                return Ok(CliRunTarget::Context {
                    id: context.to_owned(),
                    help: true,
                });
            }
            if definition.contexts[context].commands.contains_key(word) {
                return Ok(CliRunTarget::Command {
                    context: context.to_owned(),
                    word: word.to_owned(),
                    arguments_start: index + 1,
                });
            }
            context = self.context_children(context).get(word).ok_or_else(|| {
                run_usage_error(format!(
                    "Unknown command or context {word:?} under {context}."
                ))
            })?;
            index += 1;
        }
        Ok(CliRunTarget::Context {
            id: context.to_owned(),
            help: false,
        })
    }

    /// Render the actual usable context path, omitting the implicit root.
    pub fn context_path(&self, definition: &CliDefinition, context: &str) -> String {
        let mut words = Vec::new();
        let mut current = context;
        while let CliContextParent::Context(parent) = &definition.contexts[current].parent {
            let word = self
                .context_children(parent)
                .iter()
                .find(|(_, id)| id.as_str() == current)
                .expect("Projected child")
                .0;
            words.push(word.as_str());
            current = parent;
        }
        words.reverse();
        words.join(" ")
    }
}
