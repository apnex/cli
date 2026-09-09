//! Schema guidance projects local structural hints without substituting for whole-document validation.

use crate::authoring_error::AuthoringError;
use crate::document_path::{DocumentPath, DocumentSegment, render_document_pointer};
use crate::document_value::DocumentValue;
use crate::schema_constraint::{SchemaConstraint, escape_schema_component};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// A declared child remains discoverable before the user creates its instance value.
#[derive(Clone, Debug, Serialize)]
pub struct SchemaChildGuidance {
    pub key: String,
    pub path: DocumentPath,
    pub terminal_path: String,
    pub required: bool,
    pub types: Vec<String>,
}

/// Guidance exposes its provenance and completeness; suggested values are explicitly typed constructors.
#[derive(Clone, Debug, Serialize)]
pub struct SchemaPathGuidance {
    pub path: DocumentPath,
    pub terminal_path: String,
    pub schema_paths: Vec<String>,
    pub types: Vec<String>,
    pub values: Vec<Value>,
    pub required: Vec<String>,
    pub children: Vec<SchemaChildGuidance>,
    pub rules: Vec<Value>,
    pub complete: bool,
    pub notes: Vec<String>,
}

fn schema_at<'a>(root: &'a DocumentValue, pointer: &str) -> Option<&'a DocumentValue> {
    if pointer.is_empty() {
        return Some(root);
    }
    let mut value = root;
    for raw in pointer.strip_prefix('/')?.split('/') {
        let key = raw.replace("~1", "/").replace("~0", "~");
        value = match value {
            DocumentValue::Object(map) => map.get(&key)?,
            DocumentValue::Array(array) => array.get(key.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(value)
}

fn expanded_locations(
    root: &DocumentValue,
    starts: Vec<String>,
    notes: &mut BTreeSet<String>,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut pending = starts;
    while let Some(path) = pending.pop() {
        if !seen.insert(path.clone()) {
            continue;
        }
        let Some(DocumentValue::Object(object)) = schema_at(root, &path) else {
            continue;
        };
        if let Some(DocumentValue::String(target)) = object.get("$ref") {
            pending.push(target[1..].into());
            if object.keys().any(|keyword| {
                ![
                    "$ref",
                    "$schema",
                    "$defs",
                    "$comment",
                    "title",
                    "description",
                    "default",
                    "examples",
                    "deprecated",
                    "readOnly",
                    "writeOnly",
                    "format",
                ]
                .contains(&keyword.as_str())
            }) {
                notes.insert("Reference sibling rules also apply; structural suggestions do not solve their intersection.".into());
            }
        }
        if let Some(DocumentValue::Array(branches)) = object.get("allOf") {
            for index in 0..branches.len() {
                pending.push(format!("{path}/allOf/{index}"));
            }
            notes.insert("Conjunctive rules may further restrict structural suggestions; validate the complete instance.".into());
        }
        for keyword in [
            "anyOf",
            "oneOf",
            "not",
            "if",
            "then",
            "else",
            "dependentSchemas",
            "dependentRequired",
            "patternProperties",
            "propertyNames",
            "unevaluatedProperties",
            "unevaluatedItems",
            "contains",
        ] {
            if object.contains_key(keyword) {
                notes.insert(format!(
                    "{keyword} is validated but is not fully enumerated by guidance."
                ));
            }
        }
    }
    seen.into_iter().collect()
}

fn child_locations(
    root: &DocumentValue,
    locations: &[String],
    segment: &DocumentSegment,
    notes: &mut BTreeSet<String>,
) -> Vec<String> {
    let mut children = Vec::new();
    for path in locations {
        match schema_at(root, path) {
            Some(DocumentValue::Object(object)) => {
                match segment {
                    DocumentSegment::Key { key } => {
                        let target = format!("{path}/properties/{}", escape_schema_component(key));
                        if schema_at(root, &target).is_some() {
                            children.push(target);
                        } else if object.contains_key("additionalProperties")
                            && !object.contains_key("patternProperties")
                        {
                            children.push(format!("{path}/additionalProperties"));
                        }
                    }
                    DocumentSegment::Index { index } => {
                        let target = format!("{path}/prefixItems/{index}");
                        if schema_at(root, &target).is_some() {
                            children.push(target);
                        } else if object.contains_key("items") {
                            children.push(format!("{path}/items"));
                        }
                    }
                }
                if object.contains_key("enum") || object.contains_key("const") {
                    notes.insert("An ancestor enum or const constrains this value; local suggestions are incomplete.".into());
                }
            }
            Some(DocumentValue::Boolean(false)) => {
                notes.insert(
                    "A false ancestor schema forbids every instance at this location.".into(),
                );
                children.push(path.clone());
            }
            _ => {}
        }
    }
    expanded_locations(root, children, notes)
}

fn declared_schema_types(root: &DocumentValue, locations: &[String]) -> Vec<String> {
    let mut types = BTreeSet::new();
    for path in locations {
        if let Some(value) = schema_at(root, &format!("{path}/type")) {
            match value {
                DocumentValue::String(kind) => {
                    types.insert(kind.clone());
                }
                DocumentValue::Array(kinds) => {
                    for kind in kinds {
                        if let DocumentValue::String(kind) = kind {
                            types.insert(kind.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    types.into_iter().collect()
}

fn literal_constructor(value: &DocumentValue) -> Option<Value> {
    Some(match value {
        DocumentValue::String(value) => json!({"kind":"string","value":value}),
        DocumentValue::Number(value) => json!({"kind":"number","value":value.get()}),
        DocumentValue::Boolean(value) => json!({"kind":"boolean","value":value}),
        DocumentValue::Null => json!({"kind":"null"}),
        _ => return None,
    })
}

/// Describe direct properties, array positions, and reference targets with explicit projection limits.
pub fn describe_schema_path(
    constraint: &SchemaConstraint,
    path: &[DocumentSegment],
) -> Result<SchemaPathGuidance, AuthoringError> {
    let root = &constraint.schema;
    let mut notes = BTreeSet::new();
    let mut locations = expanded_locations(root, vec![String::new()], &mut notes);
    for segment in path {
        locations = child_locations(root, &locations, segment, &mut notes);
    }
    let types = declared_schema_types(root, &locations);
    let mut required = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let mut values = Vec::new();
    let mut rules = Vec::new();
    for location in &locations {
        let Some(schema) = schema_at(root, location) else {
            continue;
        };
        rules.push(
            json!({"schema_path":location,"schema_json_text":schema.compact_document_json()}),
        );
        if let DocumentValue::Object(object) = schema {
            if let Some(DocumentValue::Array(names)) = object.get("required") {
                for name in names {
                    if let DocumentValue::String(name) = name {
                        required.insert(name.clone());
                    }
                }
            }
            if let Some(DocumentValue::Object(properties)) = object.get("properties") {
                keys.extend(properties.keys().cloned());
            }
            for literal in object.get("const").into_iter().chain(
                object
                    .get("enum")
                    .and_then(|value| {
                        if let DocumentValue::Array(values) = value {
                            Some(values.iter())
                        } else {
                            None
                        }
                    })
                    .into_iter()
                    .flatten(),
            ) {
                if let Some(value) = literal_constructor(literal) {
                    if !values.contains(&value) {
                        values.push(value);
                    }
                } else {
                    notes.insert("Container enum/const values are shown in rules; construct them incrementally.".into());
                }
            }
            if object.get("additionalProperties") != Some(&DocumentValue::Boolean(false))
                && (object.contains_key("properties")
                    || object.get("type") == Some(&DocumentValue::String("object".into())))
            {
                notes.insert("Additional object keys may be allowed; the declared children are not an exhaustive key set.".into());
            }
        }
    }
    if locations.is_empty() || (types.is_empty() && values.is_empty()) {
        notes.insert("No finite set of types or values is established here; inspect the applicable rules and validate.".into());
    }
    let children = keys
        .into_iter()
        .map(|key| {
            let segment = DocumentSegment::Key { key: key.clone() };
            let mut child_path = path.to_vec();
            child_path.push(segment.clone());
            let child_locations = child_locations(root, &locations, &segment, &mut BTreeSet::new());
            SchemaChildGuidance {
                required: required.contains(&key),
                key,
                terminal_path: render_document_pointer(&child_path),
                path: DocumentPath::from_absolute(child_path),
                types: declared_schema_types(root, &child_locations),
            }
        })
        .collect();
    Ok(SchemaPathGuidance {
        path: DocumentPath::from_absolute(path.to_vec()),
        terminal_path: render_document_pointer(path),
        schema_paths: locations,
        types,
        values,
        required: required.into_iter().collect(),
        children,
        rules,
        complete: notes.is_empty(),
        notes: notes.into_iter().collect(),
    })
}
