//! Schema admission and validation own the explicit, offline Draft 2020-12 interpretation.

use crate::authoring_error::{AuthoringError, authoring_limit_error};
use crate::document_value::DocumentValue;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";
pub const SCHEMA_POLICY: &str = "local-2020-12-v1";
pub const MAX_SCHEMA_BYTES: usize = 64 * 1024;

const SCHEMA_KEYWORDS: &[&str] = &[
    "$schema",
    "$defs",
    "$ref",
    "type",
    "enum",
    "const",
    "properties",
    "required",
    "additionalProperties",
    "patternProperties",
    "propertyNames",
    "dependentRequired",
    "dependentSchemas",
    "items",
    "prefixItems",
    "contains",
    "minContains",
    "maxContains",
    "minItems",
    "maxItems",
    "uniqueItems",
    "minProperties",
    "maxProperties",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "multipleOf",
    "minLength",
    "maxLength",
    "pattern",
    "allOf",
    "anyOf",
    "oneOf",
    "not",
    "if",
    "then",
    "else",
    "unevaluatedProperties",
    "unevaluatedItems",
    "title",
    "description",
    "default",
    "examples",
    "deprecated",
    "readOnly",
    "writeOnly",
    "$comment",
    "format",
];
const SCHEMA_MAPS: &[&str] = &[
    "$defs",
    "properties",
    "patternProperties",
    "dependentSchemas",
];
const SCHEMA_ARRAYS: &[&str] = &["prefixItems", "allOf", "anyOf", "oneOf"];
const SCHEMA_SINGLES: &[&str] = &[
    "additionalProperties",
    "propertyNames",
    "items",
    "contains",
    "not",
    "if",
    "then",
    "else",
    "unevaluatedProperties",
    "unevaluatedItems",
];

/// The header makes active interpretation visible without copying the whole schema into every reply.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaConstraintHeader {
    pub dialect: String,
    pub policy: String,
    pub schema_sha256: String,
}

/// A checkpoint owns an immutable schema snapshot independent of its former source document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaConstraint {
    pub dialect: String,
    pub policy: String,
    pub schema_sha256: String,
    pub schema: DocumentValue,
}

/// A validation finding identifies both the evaluated instance and the violated schema rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaFinding {
    pub instance_path: String,
    pub schema_path: String,
    pub keyword: String,
    pub message: String,
    pub message_truncated: bool,
}

/// An invalid report remains invalid when additional findings exceed the bounded result window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaValidationReport {
    pub valid: bool,
    pub findings: Vec<SchemaFinding>,
    pub truncated: bool,
}

fn schema_error(code: &str, message: impl Into<String>, path: &str) -> AuthoringError {
    AuthoringError::new(code, message, "Inspect the schema path and the local-2020-12-v1 policy; repair the schema as data and attach it again.")
        .at_document_path(path)
}

/// Escape one decoded object key for a schema or instance JSON Pointer.
pub fn escape_schema_component(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

fn bounded_message(message: String) -> (String, bool) {
    if message.len() <= 2048 {
        return (message, false);
    }
    let mut end = 2048;
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    (message[..end].to_owned(), true)
}

fn schema_finding(error: &jsonschema::ValidationError<'_>) -> SchemaFinding {
    let schema_path = error.schema_path().to_string();
    let (message, message_truncated) = bounded_message(error.to_string());
    SchemaFinding {
        instance_path: error.instance_path().to_string(),
        keyword: error.kind().keyword().into(),
        schema_path,
        message,
        message_truncated,
    }
}

fn validate_number_bounds(value: &DocumentValue, path: &str) -> Result<(), AuthoringError> {
    match value {
        DocumentValue::Number(number) => {
            let text = number.get();
            let exponent = text
                .split_once(['e', 'E'])
                .map(|(_, exponent)| exponent.parse::<i32>());
            if text.len() > 1024
                || exponent
                    .is_some_and(|exponent| exponent.map_or(true, |n| n.unsigned_abs() > 4096))
            {
                return Err(authoring_limit_error("Constraint evaluation requires number tokens at most 1,024 bytes and exponent magnitude at most 4,096; the authored value is unchanged.").at_document_path(path));
            }
        }
        DocumentValue::Object(values) => {
            for (key, child) in values {
                validate_number_bounds(child, &format!("{path}/{}", escape_schema_component(key)))?;
            }
        }
        DocumentValue::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                validate_number_bounds(child, &format!("{path}/{index}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn collect_schema_nodes(
    schema: &Value,
    path: &str,
    edges: &mut BTreeMap<String, Vec<String>>,
    references: &mut Vec<(String, String)>,
) -> Result<(), AuthoringError> {
    if edges.len() == 256 {
        return Err(authoring_limit_error(
            "Schema contains more than 256 schema locations.",
        ));
    }
    edges.insert(path.into(), Vec::new());
    let Some(object) = schema.as_object() else {
        return Ok(());
    };
    for (keyword, value) in object {
        let location = format!("{path}/{}", escape_schema_component(keyword));
        if !SCHEMA_KEYWORDS.contains(&keyword.as_str()) {
            return Err(schema_error(
                "UNSUPPORTED_SCHEMA",
                format!("Schema keyword {keyword:?} is outside local-2020-12-v1."),
                &location,
            ));
        }
        if keyword == "$schema"
            && value.as_str().map(|s| s.strip_suffix('#').unwrap_or(s)) != Some(SCHEMA_DIALECT)
        {
            return Err(schema_error(
                "UNSUPPORTED_SCHEMA",
                "Only Draft 2020-12 is admitted.",
                &location,
            ));
        }
        if keyword == "$ref" {
            let target = value.as_str().unwrap_or("");
            if !(target == "#" || target.starts_with("#/"))
                || target.contains('%')
                || target
                    .as_bytes()
                    .windows(2)
                    .any(|pair| pair[0] == b'~' && !b"01".contains(&pair[1]))
                || target.ends_with('~')
            {
                return Err(schema_error(
                    "UNSUPPORTED_SCHEMA",
                    "References require an in-document JSON Pointer without URI percent escapes.",
                    &location,
                ));
            }
            references.push((path.into(), target[1..].into()));
        }
        let children: Vec<(String, &Value)> = if SCHEMA_MAPS.contains(&keyword.as_str()) {
            value
                .as_object()
                .into_iter()
                .flat_map(|map| map.iter())
                .map(|(key, child)| {
                    (
                        format!("{location}/{}", escape_schema_component(key)),
                        child,
                    )
                })
                .collect()
        } else if SCHEMA_ARRAYS.contains(&keyword.as_str()) {
            value
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
                .map(|(index, child)| (format!("{location}/{index}"), child))
                .collect()
        } else if SCHEMA_SINGLES.contains(&keyword.as_str()) {
            vec![(location, value)]
        } else {
            Vec::new()
        };
        for (child_path, child) in children {
            edges.get_mut(path).unwrap().push(child_path.clone());
            collect_schema_nodes(child, &child_path, edges, references)?;
        }
    }
    Ok(())
}

fn check_schema_expansion(
    path: &str,
    edges: &BTreeMap<String, Vec<String>>,
    ancestors: &mut BTreeSet<String>,
    visits: &mut usize,
) -> Result<(), AuthoringError> {
    if !ancestors.insert(path.into()) {
        return Err(schema_error(
            "UNSUPPORTED_SCHEMA",
            "Cyclic schema references are outside this policy.",
            path,
        ));
    }
    *visits += 1;
    if *visits > 4096 || ancestors.len() > 64 {
        return Err(authoring_limit_error(
            "Schema expansion exceeds 4,096 visits or depth 64.",
        ));
    }
    for child in &edges[path] {
        check_schema_expansion(child, edges, ancestors, visits)?;
    }
    ancestors.remove(path);
    Ok(())
}

/// Compile an admitted schema with one exact-number, offline validator and explicit feature policy.
pub fn compile_constraint_schema(
    document: &DocumentValue,
) -> Result<jsonschema::Validator, AuthoringError> {
    document.validate_document_limits()?;
    let text = document.compact_document_json();
    if text.len() > MAX_SCHEMA_BYTES {
        return Err(authoring_limit_error("Schema exceeds 64 KiB."));
    }
    validate_number_bounds(document, "")?;
    let schema: Value = serde_json::from_str(&text).expect("Document JSON is valid");
    // Admission precedes meta-validation so alternate dialects never select a remote meta-schema.
    let mut edges = BTreeMap::new();
    let mut references = Vec::new();
    collect_schema_nodes(&schema, "", &mut edges, &mut references)?;
    jsonschema::draft202012::meta::validate(&schema).map_err(|error| {
        schema_error(
            "INVALID_SCHEMA",
            bounded_message(error.to_string()).0,
            &error.instance_path().to_string(),
        )
    })?;
    for (source, target) in references {
        if !edges.contains_key(&target) {
            return Err(schema_error(
                "INVALID_SCHEMA",
                "Reference does not resolve to an admitted schema location.",
                &format!("{source}/$ref"),
            ));
        }
        edges.get_mut(&source).unwrap().push(target);
    }
    check_schema_expansion("", &edges, &mut BTreeSet::new(), &mut 0)?;
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .offline()
        .should_validate_formats(false)
        .with_pattern_options(jsonschema::PatternOptions::regex())
        .build(&schema)
        .map_err(|error| {
            schema_error(
                "UNSUPPORTED_SCHEMA",
                bounded_message(error.to_string()).0,
                &error.schema_path().to_string(),
            )
        })
}

impl SchemaConstraint {
    /// Admit and snapshot authored data without changing the document being edited.
    pub fn from_schema_document(schema: DocumentValue) -> Result<Self, AuthoringError> {
        compile_constraint_schema(&schema)?;
        Ok(Self {
            dialect: SCHEMA_DIALECT.into(),
            policy: SCHEMA_POLICY.into(),
            schema_sha256: format!(
                "{:x}",
                Sha256::digest(schema.compact_document_json().as_bytes())
            ),
            schema,
        })
    }

    /// Expose the stable attachment identity and its interpretation policy.
    pub fn constraint_header(&self) -> SchemaConstraintHeader {
        SchemaConstraintHeader {
            dialect: self.dialect.clone(),
            policy: self.policy.clone(),
            schema_sha256: self.schema_sha256.clone(),
        }
    }

    /// Reject malformed or unsupported saved attachments before reopening the session.
    pub fn validate_constraint_snapshot(&self) -> Result<(), AuthoringError> {
        if self.dialect != SCHEMA_DIALECT
            || self.policy != SCHEMA_POLICY
            || self.schema_sha256
                != format!(
                    "{:x}",
                    Sha256::digest(self.schema.compact_document_json().as_bytes())
                )
        {
            return Err(schema_error(
                "INVALID_SESSION",
                "Saved schema identity or interpretation disagrees with its content.",
                "",
            ));
        }
        compile_constraint_schema(&self.schema)
            .map(|_| ())
            .map_err(|error| error.with_error_code("INVALID_SESSION"))
    }

    /// Evaluate a complete instance; a bounded invalid report never becomes a success by truncation.
    pub fn validate_schema_instance(
        &self,
        document: &DocumentValue,
    ) -> Result<SchemaValidationReport, AuthoringError> {
        validate_number_bounds(document, "")?;
        let validator = compile_constraint_schema(&self.schema)?;
        let instance: Value = serde_json::from_str(&document.compact_document_json())
            .expect("Document JSON is valid");
        let mut findings: Vec<_> = validator
            .iter_errors(&instance)
            .take(101)
            .map(|error| schema_finding(&error))
            .collect();
        let truncated = findings.len() > 100;
        findings.truncate(100);
        Ok(SchemaValidationReport {
            valid: findings.is_empty(),
            findings,
            truncated,
        })
    }

    /// Reject acceptance without consuming a revision or replacing an editable draft.
    pub fn require_valid_instance(&self, document: &DocumentValue) -> Result<(), AuthoringError> {
        let report = self.validate_schema_instance(document)?;
        if report.valid {
            return Ok(());
        }
        let mut error = AuthoringError::new(
            "SCHEMA_VIOLATION",
            "Candidate does not satisfy the attached schema.",
            "Use validate to inspect findings, edit the preserved draft, then commit again.",
        )
        .at_document_path(&report.findings[0].instance_path);
        error.validation = Some(Box::new(report));
        Err(error)
    }
}
