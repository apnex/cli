//! Requests, checkpoints, and response headers preserve session identity across transport boundaries.

use crate::authoring_error::AuthoringError;
use crate::document_path::{DocumentSegment, read_document_path};
use crate::document_value::{DocumentValue, MAX_SCALAR_BYTES};
use crate::kernel_profile::KernelProfile;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;

fn decode_opaque_identity<'de, D: Deserializer<'de>>(input: D) -> Result<String, D::Error> {
    let value = String::deserialize(input)?;
    if value.is_empty() || value.len() > 128 {
        return Err(serde::de::Error::custom(
            "Authoring identity must contain 1 through 128 UTF-8 bytes",
        ));
    }
    Ok(value)
}

/// An opaque session identity cannot be confused with a request identity in Rust calls.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionIdentity(#[serde(deserialize_with = "decode_opaque_identity")] String);

/// A request identity identifies one submitted request, not a document revision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequestIdentity(#[serde(deserialize_with = "decode_opaque_identity")] String);

impl RequestIdentity {
    /// Generate a fresh request identity for terminal input before canonical dispatch.
    pub fn new_request_identity() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

/// A session revision serializes as a canonical decimal string to avoid numeric transport loss.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionRevision(pub u64);

impl Serialize for SessionRevision {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> Deserialize<'de> for SessionRevision {
    fn deserialize<D: Deserializer<'de>>(input: D) -> Result<Self, D::Error> {
        let text = String::deserialize(input)?;
        let number: u64 = text.parse().map_err(serde::de::Error::custom)?;
        if number.to_string() != text {
            return Err(serde::de::Error::custom(
                "Session revision requires canonical unsigned decimal spelling",
            ));
        }
        Ok(Self(number))
    }
}

/// A complete machine request retains supplied arguments exactly for last-receipt comparison.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoringRequest {
    pub request_id: RequestIdentity,
    pub session_id: SessionIdentity,
    pub operation: String,
    pub arguments: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_revision: Option<SessionRevision>,
}

/// The shared header exposes context, exact revision, and durability independently of a prompt.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionHeader {
    pub session_id: SessionIdentity,
    pub revision: SessionRevision,
    pub accepted_revision: SessionRevision,
    pub context: Vec<DocumentSegment>,
    pub node_kind: String,
    pub dirty: bool,
    pub definition_id: String,
    pub definition_sha256: String,
    pub constraint_mode: String,
    pub durability: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_interface: Option<crate::cli_interface::CliInterfaceHeader>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_constraint: Option<crate::schema_constraint::SchemaConstraintHeader>,
}

/// A response distinguishes authoritative mutation from export effects and unresolved publication.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoringResponse {
    pub request_id: Option<RequestIdentity>,
    pub event: String,
    pub operation: Option<String>,
    pub status: String,
    pub mutation: String,
    pub session: Option<SessionHeader>,
    pub result: Option<Value>,
    pub error: Option<AuthoringError>,
    pub replayed: bool,
}

impl AuthoringResponse {
    /// Return a successful operation with an explicitly supplied mutation outcome.
    pub fn operation_success(
        request: &AuthoringRequest,
        session: SessionHeader,
        result: Value,
        mutation: &str,
    ) -> Self {
        Self {
            request_id: Some(request.request_id.clone()),
            event: "response".into(),
            operation: Some(request.operation.clone()),
            status: "ok".into(),
            mutation: mutation.into(),
            session: Some(session),
            result: Some(result),
            error: None,
            replayed: false,
        }
    }

    /// Return an actionable rejection or uncertain publication without inventing a successful result.
    pub fn operation_failure(
        request_id: Option<RequestIdentity>,
        operation: Option<String>,
        session: Option<SessionHeader>,
        error: AuthoringError,
    ) -> Self {
        let uncertain = error.publication_uncertain();
        let mutation = if error.code == "PERSISTENCE_UNCERTAIN" {
            "unknown"
        } else {
            "none"
        };
        Self {
            request_id,
            event: "response".into(),
            operation,
            status: if uncertain { "uncertain" } else { "error" }.into(),
            mutation: mutation.into(),
            session,
            result: None,
            error: Some(error),
            replayed: false,
        }
    }
}

/// A persisted receipt makes the last successful state change recoverable after response loss.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoringReceipt {
    pub request: AuthoringRequest,
    pub response: AuthoringResponse,
}

/// One checkpoint owns the unfinished candidate, accepted baseline, task, context, and receipt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionCheckpoint {
    pub format_version: u32,
    pub session_id: SessionIdentity,
    pub revision: SessionRevision,
    pub accepted_revision: SessionRevision,
    pub candidate: DocumentValue,
    pub accepted: DocumentValue,
    pub context: Vec<DocumentSegment>,
    pub intent_text: String,
    pub definition_id: String,
    pub definition_sha256: String,
    pub constraint_mode: String,
    pub last_receipt: Option<AuthoringReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_interface: Option<crate::cli_interface::ActiveCliInterface>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_constraint: Option<crate::schema_constraint::SchemaConstraint>,
}

impl SessionCheckpoint {
    /// Create an empty draft with the original bounded task and loaded definition identity.
    pub fn new_authoring_session(
        intent_text: String,
        definition_id: String,
        definition_sha256: String,
    ) -> Result<Self, AuthoringError> {
        if intent_text.len() > MAX_SCALAR_BYTES {
            return Err(crate::authoring_error::authoring_limit_error(
                "Original task text exceeds 64 KiB.",
            ));
        }
        Ok(Self {
            format_version: KernelProfile::from_definition_id(&definition_id)
                .ok_or_else(|| {
                    AuthoringError::new(
                        "DEFINITION_MISMATCH",
                        "Unknown kernel profile.",
                        "Select a supported operation definition.",
                    )
                })?
                .checkpoint_version(),
            session_id: SessionIdentity(uuid::Uuid::new_v4().to_string()),
            revision: SessionRevision(0),
            accepted_revision: SessionRevision(0),
            candidate: DocumentValue::Object(BTreeMap::new()),
            accepted: DocumentValue::Object(BTreeMap::new()),
            context: Vec::new(),
            intent_text,
            definition_id,
            definition_sha256,
            constraint_mode: "unconstrained".into(),
            last_receipt: None,
            active_interface: None,
            active_constraint: None,
        })
    }

    /// Compare document values rather than revisions when reporting unfinished edits.
    pub fn candidate_is_dirty(&self) -> bool {
        self.candidate != self.accepted
    }

    /// Construct a state header only when the saved context resolves in the candidate.
    pub fn session_header(&self, durability: &str) -> Result<SessionHeader, AuthoringError> {
        Ok(SessionHeader {
            session_id: self.session_id.clone(),
            revision: self.revision,
            accepted_revision: self.accepted_revision,
            context: self.context.clone(),
            node_kind: read_document_path(&self.candidate, &self.context)?
                .document_kind()
                .into(),
            dirty: self.candidate_is_dirty(),
            definition_id: self.definition_id.clone(),
            definition_sha256: self.definition_sha256.clone(),
            constraint_mode: self.constraint_mode.clone(),
            durability: durability.into(),
            active_interface: self
                .active_interface
                .as_ref()
                .map(|active| active.cli_interface_header()),
            active_constraint: self
                .active_constraint
                .as_ref()
                .map(|constraint| constraint.constraint_header()),
        })
    }

    /// Reject impossible checkpoint relationships before exposing any saved state for mutation.
    pub fn validate_checkpoint_state(&self) -> Result<(), AuthoringError> {
        let invalid = |message: &str| {
            AuthoringError::new(
                "INVALID_SESSION",
                message,
                "Preserve the checkpoint and reopen a compatible, internally consistent session.",
            )
        };
        if ![1, 2, 3, 4].contains(&self.format_version) {
            return Err(AuthoringError::new(
                "UNSUPPORTED_FORMAT",
                "Checkpoint format version is unsupported.",
                "Use a compatible application version without rewriting this file.",
            ));
        }
        if self.intent_text.len() > MAX_SCALAR_BYTES {
            return Err(crate::authoring_error::authoring_limit_error(
                "Original task text exceeds 64 KiB.",
            ));
        }
        let expected_mode = if self.active_constraint.is_some() {
            "json-schema-2020-12"
        } else {
            "unconstrained"
        };
        if self.accepted_revision > self.revision || self.constraint_mode != expected_mode {
            return Err(invalid("Checkpoint state fields have inconsistent values."));
        }
        let profile = KernelProfile::from_definition_id(&self.definition_id)
            .ok_or_else(|| invalid("Checkpoint definition identity is unsupported."))?;
        if self.format_version != profile.checkpoint_version()
            || self.definition_sha256.len() != 64
            || !self
                .definition_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(invalid("Checkpoint definition identity is malformed."));
        }
        if !profile.has_composition() && self.active_interface.is_some() {
            return Err(invalid(
                "This authoring profile cannot carry an active CLI interface.",
            ));
        }
        if let Some(active) = &self.active_interface {
            active
                .validate_cli_interface()
                .map_err(|error| error.with_error_code("INVALID_SESSION"))?;
        }
        if self.active_constraint.is_some() && !profile.has_constraints() {
            return Err(invalid("This profile cannot carry schema constraints."));
        }
        if let Some(constraint) = &self.active_constraint {
            constraint.validate_constraint_snapshot()?;
        }
        self.candidate.validate_document_limits()?;
        self.accepted.validate_document_limits()?;
        let header = self
            .session_header("durable")
            .map_err(|_| invalid("Checkpoint context does not resolve into the candidate."))?;
        match (&self.last_receipt, self.revision.0) {
            (None, 0) => {}
            (Some(receipt), revision) if revision > 0 => {
                if receipt.request.session_id != self.session_id
                    || receipt.request.expected_revision != Some(SessionRevision(revision - 1))
                    || receipt.response.request_id.as_ref() != Some(&receipt.request.request_id)
                    || receipt.response.operation.as_ref() != Some(&receipt.request.operation)
                    || receipt.response.event != "response"
                    || receipt.response.status != "ok"
                    || receipt.response.mutation != "applied"
                    || receipt.response.error.is_some()
                    || receipt.response.result.is_none()
                    || receipt.response.replayed
                    || receipt.response.session.as_ref() != Some(&header)
                {
                    return Err(invalid(
                        "Checkpoint receipt disagrees with the saved session state.",
                    ));
                }
            }
            _ => {
                return Err(invalid(
                    "Checkpoint revision and receipt presence disagree.",
                ));
            }
        }
        Ok(())
    }
}
