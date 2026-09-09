//! Composition operations stage interface changes inside the existing authoring transaction.

use crate::authoring_error::{AuthoringError, authoring_limit_error};
use crate::authoring_operations::{HandlerOutcome, registered_authoring_handlers};
use crate::authoring_protocol::SessionCheckpoint;
use crate::cli_definition::CliDefinition;
use crate::cli_interface::{
    ActiveCliInterface, CliInterfaceExport, CliInterfaceOrigin, inactive_cli_error,
};
use crate::cli_verb_tree::render_cli_verb_tree;
use crate::document_value::{MAX_CHECKPOINT_BYTES, MAX_SCALAR_BYTES, decode_authoring_json};
use crate::operation_definition::{
    HandlerArgumentKind as Kind, HandlerParameter, OperationDefinition, RegisteredAuthoringHandler,
    ValidatedArgument, ValidatedArguments,
};
use crate::session_storage::read_regular_file_bounded;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

/// Extend the actual native handler map without duplicating document or persistence operations.
pub fn registered_composition_handlers() -> BTreeMap<String, RegisteredAuthoringHandler> {
    type Function = fn(
        &mut SessionCheckpoint,
        &ValidatedArguments,
        &OperationDefinition,
    ) -> Result<HandlerOutcome, AuthoringError>;
    let argument = |name, kind, optional| HandlerParameter {
        name,
        kind,
        optional,
    };
    let entries: Vec<(&str, &str, Vec<HandlerParameter>, Function)> = vec![
        (
            "composition.assemble",
            "state",
            vec![],
            execute_cli_assembly,
        ),
        (
            "composition.activate",
            "state",
            vec![argument("source", Kind::Text, true)],
            execute_cli_activation,
        ),
        (
            "composition.enter",
            "state",
            vec![argument("context", Kind::Text, false)],
            execute_cli_context,
        ),
        (
            "composition.discover",
            "read",
            vec![],
            execute_cli_discovery,
        ),
        ("composition.tree", "read", vec![], execute_cli_verb_tree),
        (
            "composition.invoke",
            "state",
            vec![
                argument("command", Kind::Text, false),
                argument("values", Kind::CliArguments, false),
            ],
            execute_cli_invocation,
        ),
        (
            "composition.export",
            "export",
            vec![argument("destination", Kind::Text, false)],
            execute_cli_export,
        ),
    ];
    let mut handlers = registered_authoring_handlers();
    for (id, effect, parameters, invoke) in entries {
        handlers.insert(
            id.into(),
            RegisteredAuthoringHandler {
                effect,
                batchable: false,
                parameters,
                invoke,
            },
        );
    }
    handlers
}

fn composition_text<'a>(arguments: &'a ValidatedArguments, name: &str) -> &'a str {
    match &arguments[name] {
        ValidatedArgument::Text(text) => text,
        _ => unreachable!("Registered composition text codec"),
    }
}

fn execute_cli_assembly(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let candidate = crate::cli_assembly::assemble_cli_document(&state.candidate)?;
    let changes = crate::document_path::changed_document_paths(&state.candidate, &candidate)?;
    state.candidate = candidate;
    state.context.clear();
    Ok(HandlerOutcome::Data(json!({"changed_paths":changes})))
}

fn execute_cli_activation(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let bytes = if arguments.contains_key("source") {
        read_regular_file_bounded(
            Path::new(composition_text(arguments, "source")),
            MAX_CHECKPOINT_BYTES,
            "INVALID_CLI_DEFINITION",
        )?
    } else {
        state.candidate.compact_document_json().into_bytes()
    };
    let active = parse_cli_interface_source(
        &bytes,
        state.active_interface.as_ref(),
        CliInterfaceOrigin {
            intent_text: state.intent_text.clone(),
            revision: state.revision,
        },
    )?;
    let header = active.cli_interface_header();
    state.active_interface = Some(active);
    Ok(HandlerOutcome::Data(json!({"active_interface":header})))
}

/// Decode one portable source for authoring activation and direct execution with identical validation.
pub fn parse_cli_interface_source(
    bytes: &[u8],
    previous: Option<&ActiveCliInterface>,
    origin: CliInterfaceOrigin,
) -> Result<ActiveCliInterface, AuthoringError> {
    let envelope: Value =
        decode_authoring_json(bytes, MAX_CHECKPOINT_BYTES, "INVALID_CLI_DEFINITION")?;
    if envelope["format"] == "cli-interface-export-v1" {
        let bundle: CliInterfaceExport =
            decode_authoring_json(bytes, MAX_CHECKPOINT_BYTES, "INVALID_CLI_DEFINITION")?;
        if bundle.source_intent_text.len() > MAX_SCALAR_BYTES {
            return Err(authoring_limit_error(
                "CLI export source task exceeds 64 KiB.",
            ));
        }
        bundle.interface.validate_cli_interface()?;
        Ok(bundle.interface)
    } else {
        let definition = CliDefinition::parse_cli_definition(bytes)?;
        Ok(match previous {
            Some(active) if active.definition_sha256 == definition.cli_definition_digest() => {
                active.clone()
            }
            _ => ActiveCliInterface::initialize_cli_interface(definition, origin),
        })
    }
}

fn execute_cli_context(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let active = state
        .active_interface
        .as_mut()
        .ok_or_else(inactive_cli_error)?;
    active.enter_cli_context(composition_text(arguments, "context"))?;
    Ok(HandlerOutcome::Data(
        json!({"active_interface":active.cli_interface_header()}),
    ))
}

fn execute_cli_discovery(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    definition: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    Ok(HandlerOutcome::Data(
        state
            .active_interface
            .as_ref()
            .ok_or_else(inactive_cli_error)?
            .discover_cli_interface(definition.json_read_grants()),
    ))
}

fn execute_cli_verb_tree(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let interface = state
        .active_interface
        .as_ref()
        .ok_or_else(inactive_cli_error)?;
    Ok(HandlerOutcome::Data(json!({
        "interface": interface.cli_interface_header(),
        "verb_tree_text": render_cli_verb_tree(&interface.definition)
    })))
}

fn execute_cli_invocation(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    definition: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let values = match &arguments["values"] {
        ValidatedArgument::CliArguments(values) => values,
        _ => unreachable!("Registered CLI arguments codec"),
    };
    let outcome = state
        .active_interface
        .as_mut()
        .ok_or_else(inactive_cli_error)?
        .invoke_cli_command(
            composition_text(arguments, "command"),
            values,
            definition.json_read_grants(),
        )?;
    Ok(HandlerOutcome::Data(json!({"invocation":outcome})))
}

fn execute_cli_export(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let interface = state
        .active_interface
        .as_ref()
        .ok_or_else(inactive_cli_error)?;
    let export = CliInterfaceExport {
        format: "cli-interface-export-v1".into(),
        interface: interface.clone(),
        source_intent_text: state.intent_text.clone(),
        source_revision: state.revision,
    };
    let mut bytes = serde_json::to_vec(&export).unwrap();
    bytes.push(b'\n');
    if bytes.len() > MAX_CHECKPOINT_BYTES {
        return Err(authoring_limit_error("CLI interface export exceeds 8 MiB."));
    }
    Ok(HandlerOutcome::ExportDocument {
        destination: composition_text(arguments, "destination").into(),
        bytes,
    })
}
