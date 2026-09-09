//! Constraint operations share the authoring transaction and keep schema interpretation explicit.

use crate::authoring_error::AuthoringError;
use crate::authoring_operations::HandlerOutcome;
use crate::authoring_protocol::SessionCheckpoint;
use crate::document_value::DocumentValue;
use crate::operation_definition::{
    HandlerArgumentKind as Kind, HandlerParameter, OperationDefinition, RegisteredAuthoringHandler,
    ValidatedArgument, ValidatedArguments,
};
use crate::schema_constraint::{MAX_SCHEMA_BYTES, SchemaConstraint};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;

/// Extend either actual dispatch map with the independently selected constraint layer.
pub fn registered_constraint_handlers(
    compose: bool,
) -> BTreeMap<String, RegisteredAuthoringHandler> {
    type Function = fn(
        &mut SessionCheckpoint,
        &ValidatedArguments,
        &OperationDefinition,
    ) -> Result<HandlerOutcome, AuthoringError>;
    let parameter = |name, kind, optional| HandlerParameter {
        name,
        kind,
        optional,
    };
    let entries: Vec<(&str, &str, Vec<HandlerParameter>, Function)> = vec![
        (
            "constraints.attach",
            "state",
            vec![parameter("source", Kind::Text, true)],
            execute_schema_attachment,
        ),
        (
            "constraints.detach",
            "state",
            vec![],
            execute_schema_detachment,
        ),
        (
            "constraints.inspect",
            "read",
            vec![],
            execute_schema_inspection,
        ),
        (
            "constraints.validate",
            "read",
            vec![parameter("view", Kind::DocumentView, false)],
            execute_schema_validation,
        ),
        (
            "constraints.guide",
            "read",
            vec![parameter("path", Kind::DocumentPath, false)],
            execute_schema_guidance,
        ),
    ];
    let mut handlers = if compose {
        crate::cli_composition::registered_composition_handlers()
    } else {
        crate::authoring_operations::registered_authoring_handlers()
    };
    for (name, effect, parameters, function) in entries {
        handlers.insert(
            name.into(),
            RegisteredAuthoringHandler {
                effect,
                batchable: false,
                parameters,
                invoke: function,
            },
        );
    }
    handlers
}

/// Require an attachment before making any claim about schema validity or guidance.
pub fn active_schema_constraint(
    state: &SessionCheckpoint,
) -> Result<&SchemaConstraint, AuthoringError> {
    state.active_constraint.as_ref().ok_or_else(|| {
        AuthoringError::new(
            "NO_ACTIVE_SCHEMA",
            "No schema is attached.",
            "Author a schema or name a schema file, then use constrain to attach it explicitly.",
        )
    })
}

fn execute_schema_attachment(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let schema = if let Some(ValidatedArgument::Text(source)) = arguments.get("source") {
        let bytes = crate::session_storage::read_regular_file_bounded(
            Path::new(source),
            MAX_SCHEMA_BYTES,
            "INVALID_SCHEMA",
        )?;
        let text = std::str::from_utf8(&bytes).map_err(|_| {
            AuthoringError::new(
                "INVALID_SCHEMA",
                "Schema file is not UTF-8.",
                "Supply a UTF-8 JSON schema document.",
            )
        })?;
        DocumentValue::parse_document(text)
            .map_err(|error| error.with_error_code("INVALID_SCHEMA"))?
    } else {
        state.candidate.clone()
    };
    let attachment = SchemaConstraint::from_schema_document(schema)?;
    let header = attachment.constraint_header();
    state.active_constraint = Some(attachment);
    state.constraint_mode = "json-schema-2020-12".into();
    Ok(HandlerOutcome::Data(json!({"active_constraint":header})))
}

fn execute_schema_detachment(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    state.active_constraint = None;
    state.constraint_mode = "unconstrained".into();
    Ok(HandlerOutcome::Data(json!({"active_constraint":null})))
}

fn execute_schema_inspection(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let constraint = active_schema_constraint(state)?;
    Ok(HandlerOutcome::Data(
        json!({"active_constraint":constraint.constraint_header(),"schema_json_text":constraint.schema.compact_document_json(),"format_policy":"annotation-only","draft_policy":"editable; full validation on commit","reference_policy":"in-document JSON Pointer, acyclic, no retrieval"}),
    ))
}

fn execute_schema_validation(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let constraint = active_schema_constraint(state)?;
    let view = match &arguments["view"] {
        ValidatedArgument::Text(view) => view,
        _ => unreachable!("Validated document view"),
    };
    let document = if view == "accepted" {
        &state.accepted
    } else {
        &state.candidate
    };
    Ok(HandlerOutcome::Data(
        json!({"view":view,"validation":constraint.validate_schema_instance(document)?}),
    ))
}

fn execute_schema_guidance(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let path = match &arguments["path"] {
        ValidatedArgument::Path(path) => path.absolute_segments(&state.context),
        _ => unreachable!("Validated document path"),
    };
    if let Some((segment, parent)) = path.split_last() {
        let node = crate::document_path::read_document_path(&state.candidate, parent)?;
        if !matches!(
            (segment, node),
            (
                crate::document_path::DocumentSegment::Key { .. },
                DocumentValue::Object(_)
            ) | (
                crate::document_path::DocumentSegment::Index { .. },
                DocumentValue::Array(_)
            )
        ) {
            return Err(AuthoringError::new(
                "TYPE_MISMATCH",
                "Guidance path segment does not match its existing parent container.",
                "Inspect the parent and supply an object key or array index matching its type.",
            )
            .at_document_path(crate::document_path::DocumentPath::from_absolute(path)));
        }
    }
    Ok(HandlerOutcome::Data(
        json!({"guidance":crate::schema_guidance::describe_schema_path(active_schema_constraint(state)?, &path)?}),
    ))
}
