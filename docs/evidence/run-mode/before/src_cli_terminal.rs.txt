//! Configured scalar signatures drive terminal lowering and contextual completion.

use crate::authoring_error::AuthoringError;
use crate::cli_interface::ActiveCliInterface;
use crate::document_value::DocumentValue;
use crate::terminal_input::TerminalToken;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Lower positional terminal values to the active command's named JSON scalar arguments.
pub fn compile_cli_terminal_arguments(
    active: &ActiveCliInterface,
    word: &str,
    tokens: &[TerminalToken],
) -> Result<Value, AuthoringError> {
    let command = active.resolve_cli_command(word)?;
    let invalid = || {
        AuthoringError::new(
            "INVALID_CLI_ARGUMENTS",
            "CLI terminal values do not match the declared scalar signature.",
            "Use discover to inspect the command's ordered required parameters.",
        )
    };
    if tokens.len() != command.parameters.len() {
        return Err(invalid());
    }
    let mut values = BTreeMap::new();
    for (parameter, token) in command.parameters.iter().zip(tokens) {
        let value = match parameter.value_type.as_str() {
            "string" => json!({"kind":"string","value":token.text}),
            "boolean" => match token.text.as_str() {
                "true" => json!({"kind":"boolean","value":true}),
                "false" => json!({"kind":"boolean","value":false}),
                _ => return Err(invalid()),
            },
            "number" => {
                let value = DocumentValue::parse_document(&token.text).map_err(|_| invalid())?;
                if value.document_kind() != "number" || token.text.trim() != token.text {
                    return Err(invalid());
                }
                json!({"kind":"number","value":token.text})
            }
            _ => unreachable!("Validated CLI scalar type"),
        };
        values.insert(parameter.name.clone(), value);
    }
    active.validate_cli_arguments(word, &values)?;
    Ok(json!(values))
}

/// Complete only command words or boolean values available in the active interface context.
pub fn complete_cli_invocation(
    active: &ActiveCliInterface,
    previous: &[TerminalToken],
) -> Vec<(String, String)> {
    if previous.len() == 1 {
        return active.definition.contexts[&active.context]
            .commands
            .iter()
            .map(|(word, command)| (word.clone(), command.help.clone()))
            .collect();
    }
    let Ok(command) = active.resolve_cli_command(&previous[1].text) else {
        return Vec::new();
    };
    if command
        .parameters
        .get(previous.len() - 2)
        .is_some_and(|parameter| parameter.value_type == "boolean")
    {
        return vec![
            ("true".into(), "boolean".into()),
            ("false".into(), "boolean".into()),
        ];
    }
    Vec::new()
}
