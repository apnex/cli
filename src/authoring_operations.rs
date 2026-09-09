//! Native operation meanings are shared by terminal input, machine requests, and atomic batches.

use crate::authoring_error::AuthoringError;
use crate::authoring_protocol::SessionCheckpoint;
use crate::authoring_result::AuthoringResultEntries;
use crate::document_path::changed_document_paths;
use crate::document_path::{
    DocumentPath, DocumentPathBase, DocumentSegment, append_document_value, delete_document_value,
    diff_document_values, insert_document_value, read_document_path, render_document_pointer,
    set_document_value,
};
use crate::document_value::{DocumentValue, MAX_REQUEST_BYTES};
use crate::operation_definition::{
    HandlerArgumentKind as Kind, HandlerParameter, OperationDefinition, RegisteredAuthoringHandler,
    ValidatedArgument, ValidatedArguments,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

/// A handler returns data or an export intention; only the runtime performs publication.
pub enum HandlerOutcome {
    Data(Value),
    Export(String),
    ExportDocument { destination: String, bytes: Vec<u8> },
}

fn parameter(name: &'static str, kind: Kind) -> HandlerParameter {
    HandlerParameter {
        name,
        kind,
        optional: false,
    }
}

/// Build the actual native dispatch map whose completeness is checked against loaded data.
pub fn registered_authoring_handlers() -> BTreeMap<String, RegisteredAuthoringHandler> {
    type Function = fn(
        &mut SessionCheckpoint,
        &ValidatedArguments,
        &OperationDefinition,
    ) -> Result<HandlerOutcome, AuthoringError>;
    let specifications: Vec<(&str, &str, bool, Vec<HandlerParameter>, Function)> = vec![
        (
            "authoring.status",
            "read",
            false,
            vec![],
            execute_session_status,
        ),
        (
            "authoring.show",
            "read",
            false,
            vec![
                parameter("path", Kind::DocumentPath),
                parameter("view", Kind::DocumentView),
            ],
            execute_document_show,
        ),
        (
            "authoring.help",
            "read",
            false,
            vec![HandlerParameter {
                name: "operation",
                kind: Kind::Text,
                optional: true,
            }],
            execute_operation_help,
        ),
        (
            "authoring.complete",
            "read",
            false,
            vec![
                parameter("path", Kind::DocumentPath),
                parameter(
                    "offset",
                    Kind::Unsigned {
                        minimum: 0,
                        maximum: None,
                    },
                ),
                parameter(
                    "limit",
                    Kind::Unsigned {
                        minimum: 1,
                        maximum: Some(100),
                    },
                ),
            ],
            execute_document_completion,
        ),
        (
            "authoring.diff",
            "read",
            false,
            vec![],
            execute_candidate_diff,
        ),
        (
            "authoring.edit",
            "state",
            false,
            vec![parameter("path", Kind::DocumentPath)],
            execute_context_edit,
        ),
        ("authoring.up", "state", false, vec![], execute_context_up),
        ("authoring.top", "state", false, vec![], execute_context_top),
        (
            "authoring.set",
            "state",
            true,
            vec![
                parameter("path", Kind::DocumentPath),
                parameter("value", Kind::ValueConstructor),
            ],
            execute_value_set,
        ),
        (
            "authoring.append",
            "state",
            true,
            vec![
                parameter("path", Kind::DocumentPath),
                parameter("value", Kind::ValueConstructor),
            ],
            execute_value_append,
        ),
        (
            "authoring.insert",
            "state",
            true,
            vec![
                parameter("path", Kind::DocumentPath),
                parameter(
                    "index",
                    Kind::Unsigned {
                        minimum: 0,
                        maximum: None,
                    },
                ),
                parameter("value", Kind::ValueConstructor),
            ],
            execute_value_insert,
        ),
        (
            "authoring.delete",
            "state",
            true,
            vec![parameter("path", Kind::DocumentPath)],
            execute_value_delete,
        ),
        (
            "authoring.batch",
            "state",
            false,
            vec![parameter("operations", Kind::PrimitiveEdits)],
            execute_primitive_batch,
        ),
        (
            "authoring.commit",
            "state",
            false,
            vec![],
            execute_candidate_commit,
        ),
        (
            "authoring.discard",
            "state",
            false,
            vec![],
            execute_candidate_discard,
        ),
        (
            "authoring.import",
            "state",
            false,
            vec![parameter("source", Kind::Text)],
            execute_document_import,
        ),
        (
            "authoring.save",
            "export",
            false,
            vec![parameter("destination", Kind::Text)],
            execute_candidate_export,
        ),
    ];
    specifications
        .into_iter()
        .map(|(id, effect, batchable, parameters, invoke)| {
            (
                id.into(),
                RegisteredAuthoringHandler {
                    effect,
                    batchable,
                    parameters,
                    invoke,
                },
            )
        })
        .collect()
}

fn path_argument(arguments: &ValidatedArguments) -> &DocumentPath {
    match &arguments["path"] {
        ValidatedArgument::Path(path) => path,
        _ => unreachable!("Registered path codec"),
    }
}
fn constructor_argument(arguments: &ValidatedArguments) -> DocumentValue {
    match &arguments["value"] {
        ValidatedArgument::Constructor(value) => value.clone(),
        _ => unreachable!("Registered constructor codec"),
    }
}
fn unsigned_argument(arguments: &ValidatedArguments, name: &str) -> usize {
    match arguments[name] {
        ValidatedArgument::Unsigned(value) => value,
        _ => unreachable!("Registered integer codec"),
    }
}
fn text_argument<'a>(arguments: &'a ValidatedArguments, name: &str) -> &'a str {
    match &arguments[name] {
        ValidatedArgument::Text(value) => value,
        _ => unreachable!("Registered text codec"),
    }
}

fn execute_session_status(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    Ok(HandlerOutcome::Data(
        json!({"intent_text":state.intent_text,"last_receipt":state.last_receipt,"origin":"hand-authored-bootstrap","real_hooks_available":false}),
    ))
}

fn execute_document_show(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let path = path_argument(arguments);
    let view = text_argument(arguments, "view");
    if view == "accepted" && path.base != DocumentPathBase::Root {
        return Err(AuthoringError::new(
            "INVALID_PATH",
            "Accepted document reads require an explicit root-based path.",
            "Supply a root path because draft context may identify different accepted content.",
        ));
    }
    let absolute = path.absolute_segments(&state.context);
    let document = if view == "accepted" {
        &state.accepted
    } else {
        &state.candidate
    };
    Ok(HandlerOutcome::Data(
        json!({"path":DocumentPath::from_absolute(absolute.clone()),"view":view,"json_text":read_document_path(document,&absolute)?.compact_document_json()}),
    ))
}

fn execute_operation_help(
    _: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    definition: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let name = arguments
        .get("operation")
        .map(|_| text_argument(arguments, "operation"));
    Ok(HandlerOutcome::Data(definition.operation_help(name)?))
}

fn execute_document_completion(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let absolute = path_argument(arguments).absolute_segments(&state.context);
    let node = read_document_path(&state.candidate, &absolute)?;
    let offset = unsigned_argument(arguments, "offset");
    let limit = unsigned_argument(arguments, "limit");
    let describe = |segment: DocumentSegment, value: &DocumentValue| {
        let mut location = absolute.clone();
        location.push(segment.clone());
        json!({"segment":segment,"kind":value.document_kind(),"terminal_path":render_document_pointer(&location),"path":DocumentPath::from_absolute(location)})
    };
    let mut entries = AuthoringResultEntries::default();
    for (segment, value) in crate::document_path::document_child_locations(node)
        .skip(offset)
        .take(limit)
    {
        entries.push_result_entry(describe(segment, value))?;
    }
    let total = match node {
        DocumentValue::Object(values) => values.len(),
        DocumentValue::Array(values) => values.len(),
        _ => 0,
    };
    let entries = entries.into_result_entries();
    let next_offset = if offset.saturating_add(entries.len()) < total {
        Some(offset + entries.len())
    } else {
        None
    };
    let mut result = json!({"entries":entries,"next_offset":next_offset});
    if let Some(constraint) = &state.active_constraint {
        result["guidance"] = serde_json::to_value(crate::schema_guidance::describe_schema_path(
            constraint, &absolute,
        )?)
        .unwrap();
    }
    Ok(HandlerOutcome::Data(result))
}

fn execute_candidate_diff(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    Ok(HandlerOutcome::Data(
        json!({"changes":diff_document_values(&state.accepted,&state.candidate)?}),
    ))
}

fn execute_context_edit(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let path = path_argument(arguments).absolute_segments(&state.context);
    read_document_path(&state.candidate, &path)?;
    state.context = path;
    Ok(HandlerOutcome::Data(json!({"path":state.context})))
}

fn execute_context_up(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    state.context.pop();
    Ok(HandlerOutcome::Data(json!({"path":state.context})))
}

fn execute_context_top(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    state.context.clear();
    Ok(HandlerOutcome::Data(json!({"path":state.context})))
}

fn changed_document_result(
    before: &DocumentValue,
    state: &SessionCheckpoint,
) -> Result<HandlerOutcome, AuthoringError> {
    state.candidate.validate_document_limits()?;
    let paths = changed_document_paths(before, &state.candidate)?;
    Ok(HandlerOutcome::Data(json!({"changed_paths":paths})))
}

fn execute_value_set(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let before = state.candidate.clone();
    let path = path_argument(arguments).absolute_segments(&state.context);
    set_document_value(
        &mut state.candidate,
        &mut state.context,
        &path,
        constructor_argument(arguments),
    )?;
    changed_document_result(&before, state)
}

fn execute_value_append(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let before = state.candidate.clone();
    let path = path_argument(arguments).absolute_segments(&state.context);
    append_document_value(&mut state.candidate, &path, constructor_argument(arguments))?;
    changed_document_result(&before, state)
}

fn execute_value_insert(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let before = state.candidate.clone();
    let path = path_argument(arguments).absolute_segments(&state.context);
    insert_document_value(
        &mut state.candidate,
        &mut state.context,
        &path,
        unsigned_argument(arguments, "index"),
        constructor_argument(arguments),
    )?;
    changed_document_result(&before, state)
}

fn execute_value_delete(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let before = state.candidate.clone();
    let path = path_argument(arguments).absolute_segments(&state.context);
    delete_document_value(&mut state.candidate, &mut state.context, &path)?;
    changed_document_result(&before, state)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimitiveRequest {
    operation: String,
    arguments: BTreeMap<String, Value>,
}

fn execute_primitive_batch(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    definition: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let before = state.candidate.clone();
    let edits = match &arguments["operations"] {
        ValidatedArgument::PrimitiveEdits(edits) => edits,
        _ => unreachable!("Registered primitive edits codec"),
    };
    for (index, edit) in edits.iter().enumerate() {
        let mut apply = || -> Result<(), AuthoringError> {
            let request: PrimitiveRequest = serde_json::from_value(edit.clone()).map_err(|_| {
                AuthoringError::new(
                    "INVALID_REQUEST",
                    "Batch entry must contain only operation and arguments.",
                    "Supply a primitive operation request.",
                )
            })?;
            let operation = definition
                .operations
                .get(&request.operation)
                .filter(|operation| operation.batchable)
                .ok_or_else(|| {
                    AuthoringError::new(
                        "INVALID_REQUEST",
                        "Batch operation is not an eligible primitive edit.",
                        "Use declared batchable set, append, insert, or delete operations.",
                    )
                })?;
            let arguments =
                definition.validate_operation_arguments(operation, &request.arguments)?;
            if arguments.values().any(|argument| matches!(argument,ValidatedArgument::Path(path) if path.base != DocumentPathBase::Root)) {
                return Err(AuthoringError::new("INVALID_PATH","Batch paths must be root-based.","Address each primitive edit from the root of the evolving batch document."));
            }
            definition.invoke_authoring_handler(operation, state, &arguments)?;
            Ok(())
        };
        apply().map_err(|error| error.at_batch_index(index))?;
    }
    changed_document_result(&before, state)
}

fn execute_candidate_commit(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    if let Some(constraint) = &state.active_constraint {
        constraint.require_valid_instance(&state.candidate)?;
    }
    state.accepted = state.candidate.clone();
    state.accepted_revision = state.revision;
    Ok(HandlerOutcome::Data(
        json!({"accepted_revision":state.accepted_revision}),
    ))
}

fn execute_candidate_discard(
    state: &mut SessionCheckpoint,
    _: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    state.candidate = state.accepted.clone();
    state.context.clear();
    Ok(HandlerOutcome::Data(json!({"dirty":false})))
}

fn execute_candidate_export(
    _: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    Ok(HandlerOutcome::Export(
        text_argument(arguments, "destination").into(),
    ))
}

/// Import one complete document into the draft; acceptance and interface activation remain explicit.
fn execute_document_import(
    state: &mut SessionCheckpoint,
    arguments: &ValidatedArguments,
    _: &OperationDefinition,
) -> Result<HandlerOutcome, AuthoringError> {
    let source = text_argument(arguments, "source");
    let bytes = crate::session_storage::read_regular_file_bounded(
        Path::new(source),
        MAX_REQUEST_BYTES,
        "IMPORT_FAILED",
    )?;
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        AuthoringError::new(
            "INVALID_VALUE",
            "Document import source is not valid UTF-8.",
            "Supply a UTF-8 JSON document with unique decoded keys.",
        )
    })?;
    let document = DocumentValue::parse_document(text)?;
    let changed_paths = changed_document_paths(&state.candidate, &document)?;
    state.candidate = document;
    state.context.clear();
    Ok(HandlerOutcome::Data(json!({
        "changed_paths":changed_paths,
        "source":source,
        "source_sha256":format!("{:x}", Sha256::digest(&bytes)),
        "source_bytes":bytes.len()
    })))
}
