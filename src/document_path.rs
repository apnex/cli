//! Typed locations keep object keys distinct from positional array indices.

use crate::authoring_error::AuthoringError;
use crate::document_value::DocumentValue;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// An absolute location segment never infers its kind from the spelling of an object key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum DocumentSegment {
    Key { key: String },
    Index { index: usize },
}

/// A request resolves its path against either the root or the acknowledged working context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentPathBase {
    Root,
    Context,
}

/// A document path carries its base explicitly, including for an empty segment list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentPath {
    pub base: DocumentPathBase,
    pub segments: Vec<DocumentSegment>,
}

impl DocumentPath {
    /// Resolve a relative location once against the current session context.
    pub fn absolute_segments(&self, context: &[DocumentSegment]) -> Vec<DocumentSegment> {
        let mut path = match self.base {
            DocumentPathBase::Root => Vec::new(),
            DocumentPathBase::Context => context.to_vec(),
        };
        path.extend(self.segments.clone());
        path
    }

    /// Wrap an absolute segment sequence for machine results and structural differences.
    pub fn from_absolute(segments: Vec<DocumentSegment>) -> Self {
        Self {
            base: DocumentPathBase::Root,
            segments,
        }
    }
}

fn path_failure(
    code: &str,
    message: impl Into<String>,
    path: &[DocumentSegment],
) -> AuthoringError {
    AuthoringError::new(
        code,
        message,
        "Inspect the parent value and correct the typed document path.",
    )
    .at_document_path(DocumentPath::from_absolute(path.to_vec()))
}

/// Read an existing value; a numeric-looking object key remains an object key.
pub fn read_document_path<'a>(
    document: &'a DocumentValue,
    path: &[DocumentSegment],
) -> Result<&'a DocumentValue, AuthoringError> {
    let mut current = document;
    for segment in path {
        current = match (current, segment) {
            (DocumentValue::Object(values), DocumentSegment::Key { key }) => {
                values.get(key).ok_or_else(|| {
                    path_failure(
                        "MISSING_PATH",
                        format!("Document member missing: {key:?}"),
                        path,
                    )
                })?
            }
            (DocumentValue::Array(values), DocumentSegment::Index { index }) => {
                values.get(*index).ok_or_else(|| {
                    path_failure(
                        "INDEX_OUT_OF_BOUNDS",
                        format!(
                            "Document array index outside length {}: {index}",
                            values.len()
                        ),
                        path,
                    )
                })?
            }
            _ => {
                return Err(path_failure(
                    "TYPE_MISMATCH",
                    "Document path segment does not match its parent container.",
                    path,
                ));
            }
        };
    }
    Ok(current)
}

fn edit_document_parent<'a>(
    document: &'a mut DocumentValue,
    path: &[DocumentSegment],
) -> Result<&'a mut DocumentValue, AuthoringError> {
    let mut current = document;
    for segment in path {
        current = match (current, segment) {
            (DocumentValue::Object(values), DocumentSegment::Key { key }) => {
                values.get_mut(key).ok_or_else(|| {
                    path_failure(
                        "MISSING_PATH",
                        format!("Document edit parent missing: {key:?}"),
                        path,
                    )
                })?
            }
            (DocumentValue::Array(values), DocumentSegment::Index { index }) => {
                values.get_mut(*index).ok_or_else(|| {
                    path_failure(
                        "INDEX_OUT_OF_BOUNDS",
                        format!("Document edit parent index missing: {index}"),
                        path,
                    )
                })?
            }
            _ => {
                return Err(path_failure(
                    "TYPE_MISMATCH",
                    "Document edit parent is incompatible with its typed path.",
                    path,
                ));
            }
        };
    }
    Ok(current)
}

/// Replace a value or create a final object member; missing ancestors and sparse arrays fail.
pub fn set_document_value(
    document: &mut DocumentValue,
    context: &mut Vec<DocumentSegment>,
    path: &[DocumentSegment],
    value: DocumentValue,
) -> Result<(), AuthoringError> {
    if let Some((last, parent_path)) = path.split_last() {
        let parent = edit_document_parent(document, parent_path)?;
        match (parent, last) {
            (DocumentValue::Object(values), DocumentSegment::Key { key }) => {
                values.insert(key.clone(), value);
            }
            (DocumentValue::Array(values), DocumentSegment::Index { index }) => {
                let destination = values.get_mut(*index).ok_or_else(|| {
                    path_failure(
                        "INDEX_OUT_OF_BOUNDS",
                        "Set requires an existing array index; use append or insert.",
                        path,
                    )
                })?;
                *destination = value;
            }
            _ => {
                return Err(path_failure(
                    "TYPE_MISMATCH",
                    "Set target does not match its parent container.",
                    path,
                ));
            }
        }
    } else {
        *document = value;
    }
    if context.len() > path.len() && context.starts_with(path) {
        *context = path.to_vec();
    }
    Ok(())
}

/// Append a value without shifting any existing array location or working context.
pub fn append_document_value(
    document: &mut DocumentValue,
    path: &[DocumentSegment],
    value: DocumentValue,
) -> Result<(), AuthoringError> {
    match edit_document_parent(document, path)? {
        DocumentValue::Array(values) => {
            values.push(value);
            Ok(())
        }
        _ => Err(path_failure(
            "TYPE_MISMATCH",
            "Append requires an existing array.",
            path,
        )),
    }
}

fn reset_shifted_array_context(
    context: &mut Vec<DocumentSegment>,
    array_path: &[DocumentSegment],
    index: usize,
) {
    if context.starts_with(array_path) {
        if let Some(DocumentSegment::Index { index: focused }) = context.get(array_path.len()) {
            if *focused >= index {
                *context = array_path.to_vec();
            }
        }
    }
}

/// Insert at a valid array boundary and reset context if its positional target could change.
pub fn insert_document_value(
    document: &mut DocumentValue,
    context: &mut Vec<DocumentSegment>,
    path: &[DocumentSegment],
    index: usize,
    value: DocumentValue,
) -> Result<(), AuthoringError> {
    match edit_document_parent(document, path)? {
        DocumentValue::Array(values) if index <= values.len() => {
            values.insert(index, value);
        }
        DocumentValue::Array(_) => {
            return Err(path_failure(
                "INDEX_OUT_OF_BOUNDS",
                "Insert index exceeds the array length.",
                path,
            ));
        }
        _ => {
            return Err(path_failure(
                "TYPE_MISMATCH",
                "Insert requires an existing array.",
                path,
            ));
        }
    }
    reset_shifted_array_context(context, path, index);
    Ok(())
}

/// Delete an existing member or element; deleting root is rejected before mutation.
pub fn delete_document_value(
    document: &mut DocumentValue,
    context: &mut Vec<DocumentSegment>,
    path: &[DocumentSegment],
) -> Result<(), AuthoringError> {
    let (last, parent_path) = path.split_last().ok_or_else(|| {
        path_failure(
            "ROOT_DELETE",
            "Deleting the document root is not supported.",
            path,
        )
    })?;
    let parent = edit_document_parent(document, parent_path)?;
    match (parent, last) {
        (DocumentValue::Object(values), DocumentSegment::Key { key }) => {
            if values.remove(key).is_none() {
                return Err(path_failure(
                    "MISSING_PATH",
                    format!("Delete member missing: {key:?}"),
                    path,
                ));
            }
        }
        (DocumentValue::Array(values), DocumentSegment::Index { index }) => {
            if *index >= values.len() {
                return Err(path_failure(
                    "INDEX_OUT_OF_BOUNDS",
                    "Delete index exceeds the array length.",
                    path,
                ));
            }
            values.remove(*index);
            reset_shifted_array_context(context, parent_path, *index);
        }
        _ => {
            return Err(path_failure(
                "TYPE_MISMATCH",
                "Delete target does not match its parent container.",
                path,
            ));
        }
    }
    if context.starts_with(path) {
        *context = parent_path.to_vec();
    }
    Ok(())
}

fn decode_pointer_token(token: &str) -> Result<String, AuthoringError> {
    let mut output = String::new();
    let mut characters = token.chars();
    while let Some(character) = characters.next() {
        if character != '~' {
            output.push(character);
            continue;
        }
        match characters.next() {
            Some('0') => output.push('~'),
            Some('1') => output.push('/'),
            _ => {
                return Err(AuthoringError::new(
                    "INVALID_PATH",
                    "JSON Pointer escape must be ~0 or ~1.",
                    "Escape literal tildes as ~0 and slashes as ~1.",
                ));
            }
        }
    }
    Ok(output)
}

/// Compile an absolute or relative terminal pointer using each actual parent container.
pub fn parse_terminal_document_path(
    token: &str,
    document: &DocumentValue,
    context: &[DocumentSegment],
) -> Result<DocumentPath, AuthoringError> {
    let (base, suffix) = if token.is_empty() {
        (DocumentPathBase::Root, None)
    } else if token == "." {
        (DocumentPathBase::Context, None)
    } else if let Some(suffix) = token.strip_prefix("./") {
        (DocumentPathBase::Context, Some(suffix))
    } else if let Some(suffix) = token.strip_prefix('/') {
        (DocumentPathBase::Root, Some(suffix))
    } else {
        return Err(AuthoringError::new(
            "INVALID_PATH",
            "Terminal path requires an absolute pointer or explicit relative prefix.",
            "Use /key, ./key, . for context, or a quoted empty string for root.",
        ));
    };
    let mut result = DocumentPath {
        base,
        segments: Vec::new(),
    };
    if let Some(suffix) = suffix {
        for token in suffix.split('/') {
            let key = decode_pointer_token(token)?;
            let absolute = result.absolute_segments(context);
            let parent = read_document_path(document, &absolute)?;
            let segment = match parent {
                DocumentValue::Object(_) => DocumentSegment::Key { key },
                DocumentValue::Array(_) => {
                    let index: usize = key.parse().map_err(|_| {
                        path_failure(
                            "INVALID_PATH",
                            "Array pointer token must be a canonical nonnegative index.",
                            &absolute,
                        )
                    })?;
                    if index.to_string() != key {
                        return Err(path_failure(
                            "INVALID_PATH",
                            "Array pointer token has noncanonical spelling.",
                            &absolute,
                        ));
                    }
                    DocumentSegment::Index { index }
                }
                _ => {
                    return Err(path_failure(
                        "TYPE_MISMATCH",
                        "Terminal pointer cannot descend through a scalar.",
                        &absolute,
                    ));
                }
            };
            result.segments.push(segment);
        }
    }
    Ok(result)
}

/// Encode a context as a JSON Pointer; an empty result is the root, not an empty-key member.
pub fn render_document_pointer(path: &[DocumentSegment]) -> String {
    path.iter()
        .map(|segment| match segment {
            DocumentSegment::Key { key } => {
                format!("/{}", key.replace('~', "~0").replace('/', "~1"))
            }
            DocumentSegment::Index { index } => format!("/{index}"),
        })
        .collect()
}

/// Enumerate immediate typed children in document order for paged discovery and editor completion.
pub fn document_child_locations(
    document: &DocumentValue,
) -> Box<dyn Iterator<Item = (DocumentSegment, &DocumentValue)> + '_> {
    match document {
        DocumentValue::Object(values) => Box::new(
            values
                .iter()
                .map(|(key, value)| (DocumentSegment::Key { key: key.clone() }, value)),
        ),
        DocumentValue::Array(values) => Box::new(
            values
                .iter()
                .enumerate()
                .map(|(index, value)| (DocumentSegment::Index { index }, value)),
        ),
        _ => Box::new(std::iter::empty()),
    }
}

fn visit_document_changes(
    before: Option<&DocumentValue>,
    after: Option<&DocumentValue>,
    path: &mut Vec<DocumentSegment>,
    visit: &mut impl FnMut(
        &[DocumentSegment],
        Option<&DocumentValue>,
        Option<&DocumentValue>,
    ) -> Result<(), AuthoringError>,
) -> Result<(), AuthoringError> {
    if before == after {
        return Ok(());
    }
    if let (Some(DocumentValue::Object(left)), Some(DocumentValue::Object(right))) = (before, after)
    {
        let keys: std::collections::BTreeSet<_> = left.keys().chain(right.keys()).collect();
        for key in keys {
            path.push(DocumentSegment::Key { key: key.clone() });
            visit_document_changes(left.get(key), right.get(key), path, visit)?;
            path.pop();
        }
        Ok(())
    } else {
        visit(path, before, after)
    }
}

/// Describe structural changes in key order, replacing a changed array as one bounded review unit.
pub fn diff_document_values(
    before: &DocumentValue,
    after: &DocumentValue,
) -> Result<Vec<Value>, AuthoringError> {
    fn value_side(value: Option<&DocumentValue>) -> Value {
        match value {
            Some(value) => json!({"exists":true,"json_text":value.compact_document_json()}),
            None => json!({"exists":false}),
        }
    }
    let mut changes = crate::authoring_result::AuthoringResultEntries::default();
    visit_document_changes(
        Some(before),
        Some(after),
        &mut Vec::new(),
        &mut |path, before, after| {
            changes.push_result_entry(json!({"path":DocumentPath::from_absolute(path.to_vec()),"before":value_side(before),"after":value_side(after)}))
        },
    )?;
    Ok(changes.into_result_entries())
}

/// Project the same structural comparison to paths without constructing unused document payloads.
pub fn changed_document_paths(
    before: &DocumentValue,
    after: &DocumentValue,
) -> Result<Vec<Value>, AuthoringError> {
    let mut paths = crate::authoring_result::AuthoringResultEntries::default();
    visit_document_changes(
        Some(before),
        Some(after),
        &mut Vec::new(),
        &mut |path, _, _| {
            paths.push_result_entry(json!(DocumentPath::from_absolute(path.to_vec())))
        },
    )?;
    Ok(paths.into_result_entries())
}
