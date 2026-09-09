//! Structured failures preserve the distinction between rejection and uncertain publication.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An authoring failure carries a stable code and an actionable recovery instruction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoringError {
    pub code: String,
    pub message: String,
    pub recovery: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_operation_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<crate::schema_constraint::SchemaValidationReport>,
}

impl AuthoringError {
    /// Construct a rejected operation; publication uncertainty is identified by its code.
    pub fn new(code: &str, message: impl Into<String>, recovery: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recovery: recovery.into(),
            path: None,
            failed_operation_index: None,
            validation: None,
        }
    }

    /// Attach the absolute document location implicated by this failure.
    pub fn at_document_path(mut self, path: impl Serialize) -> Self {
        self.path = Some(serde_json::to_value(path).expect("Document paths serialize"));
        self
    }

    /// Identify the first failed primitive edit within an atomic batch.
    pub fn at_batch_index(mut self, index: usize) -> Self {
        self.failed_operation_index = Some(index);
        self
    }

    /// Preserve the original message while classifying failure at a transport boundary.
    pub fn with_error_code(mut self, code: &str) -> Self {
        self.code = code.into();
        self
    }

    /// Identify an uncertain publication that must never be reported as an ordinary rejection.
    pub fn publication_uncertain(&self) -> bool {
        matches!(
            self.code.as_str(),
            "PERSISTENCE_UNCERTAIN" | "EXPORT_UNCERTAIN"
        )
    }
}

impl std::fmt::Display for AuthoringError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}: {} Recovery: {}",
            self.code, self.message, self.recovery
        )
    }
}

impl std::error::Error for AuthoringError {}

/// Reject a resource boundary before any authoritative state is changed.
pub fn authoring_limit_error(message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        "LIMIT_EXCEEDED",
        message,
        "Reduce the input to the documented limit and retry.",
    )
}
