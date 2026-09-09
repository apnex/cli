//! Serial dispatch publishes one checkpoint and receipt before acknowledging a state change.

use crate::authoring_error::{AuthoringError, authoring_limit_error};
use crate::authoring_operations::HandlerOutcome;
use crate::authoring_protocol::{
    AuthoringReceipt, AuthoringRequest, AuthoringResponse, RequestIdentity, SessionCheckpoint,
    SessionHeader, SessionRevision,
};
use crate::document_value::{
    DocumentValue, MAX_CHECKPOINT_BYTES, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES,
    decode_authoring_json,
};
use crate::operation_definition::OperationDefinition;
use crate::session_storage::SessionStorage;
use crate::storage_faults::StorageFaultControl;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;

/// The runtime owns one serial authoring session; presentation cannot publish or mutate its tree.
pub struct AuthoringRuntime {
    definition: OperationDefinition,
    state: SessionCheckpoint,
    storage: SessionStorage,
    uncertain: bool,
    faults: StorageFaultControl,
}

fn serialize_checkpoint(state: &SessionCheckpoint) -> Result<Vec<u8>, AuthoringError> {
    let mut bytes = serde_json::to_vec(state).expect("Validated checkpoint serializes");
    bytes.push(b'\n');
    if bytes.len() > MAX_CHECKPOINT_BYTES {
        return Err(authoring_limit_error(
            "Checkpoint exceeds 8 MiB including its receipt.",
        ));
    }
    Ok(bytes)
}

/// Check a complete output envelope before publishing state or writing any portion of its response.
pub fn checked_response_bytes(value: &impl serde::Serialize) -> Result<Vec<u8>, AuthoringError> {
    let mut bytes = serde_json::to_vec(value).expect("Authoring response serializes");
    bytes.push(b'\n');
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(authoring_limit_error(
            "Response envelope exceeds 8 MiB; request a smaller explicit view.",
        ));
    }
    Ok(bytes)
}

impl AuthoringRuntime {
    /// Open an existing checkpoint, or create a new session only when original task text is supplied.
    pub fn open_authoring_session(
        path: &Path,
        definition: OperationDefinition,
        create_intent: Option<String>,
        faults: StorageFaultControl,
    ) -> Result<Self, AuthoringError> {
        Self::open_initialized_session(path, definition, create_intent, None, faults)
    }

    /// Seed an already validated interface in the first checkpoint publication of a new run.
    pub fn create_cli_run_session(
        path: &Path,
        definition: OperationDefinition,
        intent: String,
        interface: crate::cli_interface::ActiveCliInterface,
        faults: StorageFaultControl,
    ) -> Result<Self, AuthoringError> {
        Self::open_initialized_session(path, definition, Some(intent), Some(interface), faults)
    }

    fn open_initialized_session(
        path: &Path,
        definition: OperationDefinition,
        create_intent: Option<String>,
        initial_interface: Option<crate::cli_interface::ActiveCliInterface>,
        faults: StorageFaultControl,
    ) -> Result<Self, AuthoringError> {
        let mut storage = SessionStorage::acquire_session_storage(path, faults.clone())?;
        let state = match create_intent {
            Some(intent) => {
                let mut state = SessionCheckpoint::new_authoring_session(
                    intent,
                    definition.definition_id.clone(),
                    definition.sha256.clone(),
                )?;
                state.active_interface = initial_interface;
                definition.validate_checkpoint_receipt(&state)?;
                let bytes = serialize_checkpoint(&state)?;
                storage.publish_session_checkpoint(&bytes, true)?;
                state
            }
            None => {
                let bytes = storage.read_session_checkpoint()?;
                let envelope: Value =
                    decode_authoring_json(&bytes, MAX_CHECKPOINT_BYTES, "INVALID_SESSION")?;
                if let Some(version) = envelope.get("format_version").and_then(Value::as_u64) {
                    if version != u64::from(definition.kernel_profile().checkpoint_version()) {
                        return Err(AuthoringError::new(
                            "UNSUPPORTED_FORMAT",
                            "Checkpoint format version is unsupported.",
                            "Use a compatible version without rewriting the checkpoint.",
                        ));
                    }
                }
                if !definition.kernel_profile().has_composition()
                    && (envelope.get("active_interface").is_some()
                        || envelope
                            .pointer("/last_receipt/response/session/active_interface")
                            .is_some())
                {
                    return Err(AuthoringError::new(
                        "INVALID_SESSION",
                        "Checkpoint profile cannot contain an interface extension.",
                        "Preserve the file and select the profile matching its original format.",
                    ));
                }
                if !definition.kernel_profile().has_constraints()
                    && (envelope.get("active_constraint").is_some()
                        || envelope
                            .pointer("/last_receipt/response/session/active_constraint")
                            .is_some())
                {
                    return Err(AuthoringError::new(
                        "INVALID_SESSION",
                        "Checkpoint profile cannot contain a constraint extension.",
                        "Preserve the checkpoint and select its original profile.",
                    ));
                }
                #[derive(Deserialize)]
                struct CheckpointDocuments {
                    candidate: Box<serde_json::value::RawValue>,
                    accepted: Box<serde_json::value::RawValue>,
                }
                let documents: CheckpointDocuments =
                    serde_json::from_slice(&bytes).map_err(|_| {
                        AuthoringError::new(
                            "INVALID_SESSION",
                            "Checkpoint document fields are missing or malformed.",
                            "Preserve the file and select a valid checkpoint.",
                        )
                    })?;
                for document in [documents.candidate, documents.accepted] {
                    DocumentValue::parse_document(document.get()).map_err(|error| {
                        if error.code == "LIMIT_EXCEEDED" {
                            error
                        } else {
                            error.with_error_code("INVALID_SESSION")
                        }
                    })?;
                }
                let state: SessionCheckpoint =
                    decode_authoring_json(&bytes, MAX_CHECKPOINT_BYTES, "INVALID_SESSION")?;
                definition.validate_checkpoint_receipt(&state)?;
                storage.synchronize_reopened_checkpoint()?;
                state
            }
        };
        let runtime = Self {
            definition,
            state,
            storage,
            uncertain: false,
            faults,
        };
        checked_response_bytes(&runtime.session_open_event())?;
        Ok(runtime)
    }

    /// Expose the loaded declaration to discovery and terminal input compilation.
    pub fn operation_definition(&self) -> &OperationDefinition {
        &self.definition
    }

    /// Read the acknowledged checkpoint without permitting frontend mutation.
    pub fn session_checkpoint(&self) -> &SessionCheckpoint {
        &self.state
    }

    /// Report unresolved checkpoint publication as a write stop until the session is reopened.
    pub fn session_is_uncertain(&self) -> bool {
        self.uncertain
    }

    /// Return the same state fields to both presentations, including explicit uncertainty.
    pub fn current_session_header(&self) -> SessionHeader {
        self.state
            .session_header(if self.uncertain {
                "uncertain"
            } else {
                "durable"
            })
            .expect("Runtime owns a validated context")
    }

    /// Supply task and state before the first request, including the last persisted receipt.
    pub fn session_open_event(&self) -> Value {
        json!({"event":"session_open","session":self.current_session_header(),"intent_text":self.state.intent_text,"last_receipt":self.state.last_receipt})
    }

    /// Emit a structured terminal parse error using the same session and outcome contract.
    pub fn rejected_terminal_input(
        &self,
        operation: Option<String>,
        error: AuthoringError,
    ) -> AuthoringResponse {
        AuthoringResponse::operation_failure(
            None,
            operation,
            Some(self.current_session_header()),
            error,
        )
    }

    /// Decode one bounded JSON line and retain process usability after malformed input.
    pub fn process_machine_line(&mut self, bytes: &[u8]) -> AuthoringResponse {
        let envelope: Value =
            match decode_authoring_json(bytes, MAX_REQUEST_BYTES, "INVALID_REQUEST") {
                Ok(value) => value,
                Err(error) => {
                    return AuthoringResponse::operation_failure(
                        None,
                        None,
                        Some(self.current_session_header()),
                        error,
                    );
                }
            };
        let request_id: Option<RequestIdentity> = envelope
            .get("request_id")
            .and_then(|id| serde_json::from_value(id.clone()).ok());
        let operation = envelope
            .get("operation")
            .and_then(Value::as_str)
            .filter(|name| self.definition.operations.contains_key(*name))
            .map(str::to_owned);
        if envelope
            .get("expected_revision")
            .is_some_and(|value| !value.is_string())
        {
            return AuthoringResponse::operation_failure(
                request_id,
                operation,
                Some(self.current_session_header()),
                AuthoringError::new(
                    "INVALID_REQUEST",
                    "Expected revision must be a canonical decimal string when supplied.",
                    "Omit it for reads and supply the observed revision for state changes and exports.",
                ),
            );
        }
        let request: AuthoringRequest = match serde_json::from_value(envelope) {
            Ok(request) => request,
            Err(error) => {
                return AuthoringResponse::operation_failure(
                    request_id,
                    operation,
                    Some(self.current_session_header()),
                    AuthoringError::new(
                        "INVALID_REQUEST",
                        format!("Request envelope rejected: {error}"),
                        "Supply exactly request_id, session_id, operation, arguments, and the required expected_revision.",
                    ),
                );
            }
        };
        self.execute_authoring_request(request)
    }

    /// Dispatch one canonical request and persist each successful state transition with its receipt.
    pub fn execute_authoring_request(&mut self, request: AuthoringRequest) -> AuthoringResponse {
        let recognized = self
            .definition
            .operations
            .contains_key(&request.operation)
            .then(|| request.operation.clone());
        let outcome = self.try_authoring_request(&request);
        match outcome {
            Ok(response) => response,
            Err(error) => {
                if error.code == "PERSISTENCE_UNCERTAIN" {
                    self.uncertain = true;
                }
                AuthoringResponse::operation_failure(
                    Some(request.request_id),
                    recognized,
                    Some(self.current_session_header()),
                    error,
                )
            }
        }
    }

    fn try_authoring_request(
        &mut self,
        request: &AuthoringRequest,
    ) -> Result<AuthoringResponse, AuthoringError> {
        if serde_json::to_vec(request)
            .expect("Request serializes")
            .len()
            + 1
            > MAX_REQUEST_BYTES
        {
            return Err(authoring_limit_error(
                "Canonical request exceeds the 2 MiB line limit.",
            ));
        }
        let declared = self.definition.operations.get(&request.operation);
        if request.operation.is_empty()
            || declared.is_some_and(|operation| {
                (operation.effect != "read") != request.expected_revision.is_some()
            })
        {
            return Err(AuthoringError::new(
                "INVALID_REQUEST",
                "Expected revision must be present for state changes and exports, and absent for reads.",
                "Supply the request shape declared for this operation.",
            ));
        }
        if request.session_id != self.state.session_id {
            return Err(AuthoringError::new(
                "SESSION_MISMATCH",
                "Request names a different authoring session.",
                "Use the session identity from the startup event or status response.",
            ));
        }
        if let Some(receipt) = &self.state.last_receipt {
            if receipt.request.request_id == request.request_id {
                if receipt.request != *request {
                    return Err(AuthoringError::new(
                        "REQUEST_ID_REUSE",
                        "Last receipt identifier was reused with different input.",
                        "Inspect the saved receipt and use a fresh identifier for a different request.",
                    ));
                }
                if self.uncertain {
                    return Err(AuthoringError::new(
                        "PERSISTENCE_UNCERTAIN",
                        "Receipt replay is unavailable until uncertain checkpoint publication is resolved.",
                        "Close and reopen the session, then inspect and retry the original request.",
                    ));
                }
                let mut response = receipt.response.clone();
                response.replayed = true;
                checked_response_bytes(&response)?;
                return Ok(response);
            }
        }
        let operation = declared.ok_or_else(|| {
            AuthoringError::new(
                "UNKNOWN_OPERATION",
                "Request operation is not present in the loaded declaration.",
                "Inspect help for available operation names.",
            )
        })?;
        if request
            .expected_revision
            .is_some_and(|revision| revision != self.state.revision)
        {
            return Err(AuthoringError::new(
                "REVISION_CONFLICT",
                "Request expected revision differs from the acknowledged session revision.",
                "Inspect status and the saved receipt before deciding whether to retry.",
            ));
        }
        if self.uncertain && operation.effect != "read" {
            return Err(AuthoringError::new(
                "PERSISTENCE_UNCERTAIN",
                "Authoring writes are stopped after uncertain checkpoint publication.",
                "Close and reopen the session before any state change or export.",
            ));
        }
        let arguments = self
            .definition
            .validate_operation_arguments(operation, &request.arguments)?;
        let mut working = self.state.clone();
        if operation.effect == "state" {
            working.revision =
                SessionRevision(working.revision.0.checked_add(1).ok_or_else(|| {
                    authoring_limit_error("Session revision counter is exhausted.")
                })?);
        }
        let outcome =
            self.definition
                .invoke_authoring_handler(operation, &mut working, &arguments)?;
        if operation.effect != "state" && working != self.state {
            return Err(AuthoringError::new(
                "DEFINITION_MISMATCH",
                "A non-state handler attempted to alter the authoring checkpoint.",
                "Correct the registered handler before using this declaration.",
            ));
        }
        let outcome = match outcome {
            HandlerOutcome::Export(destination) => {
                let mut bytes = self.state.candidate.compact_document_json().into_bytes();
                bytes.push(b'\n');
                HandlerOutcome::ExportDocument { destination, bytes }
            }
            other => other,
        };
        match (operation.effect.as_str(), outcome) {
            ("state", HandlerOutcome::Data(result)) => {
                working.candidate.validate_document_limits()?;
                let response = AuthoringResponse::operation_success(
                    request,
                    working.session_header("durable")?,
                    result,
                    "applied",
                );
                checked_response_bytes(&response)?;
                working.last_receipt = Some(AuthoringReceipt {
                    request: request.clone(),
                    response: response.clone(),
                });
                self.definition.validate_checkpoint_receipt(&working)?;
                let bytes = serialize_checkpoint(&working)?;
                self.storage.publish_session_checkpoint(&bytes, false)?;
                self.state = working;
                Ok(response)
            }
            ("read", HandlerOutcome::Data(result)) => {
                let response = AuthoringResponse::operation_success(
                    request,
                    self.current_session_header(),
                    result,
                    "none",
                );
                checked_response_bytes(&response)?;
                Ok(response)
            }
            ("export", HandlerOutcome::ExportDocument { destination, bytes }) => {
                let result = self.storage.export_document_bytes(
                    Path::new(&destination),
                    &bytes,
                    &self.state.revision.0.to_string(),
                )?;
                let response = AuthoringResponse::operation_success(
                    request,
                    self.current_session_header(),
                    result,
                    "none",
                );
                checked_response_bytes(&response)?;
                Ok(response)
            }
            _ => Err(AuthoringError::new(
                "DEFINITION_MISMATCH",
                "Registered handler returned an outcome incompatible with its declared effect.",
                "Correct the handler binding before retrying.",
            )),
        }
    }

    /// Expose the post-persistence, pre-delivery boundary to explicitly enabled crash tests.
    pub fn before_response_delivery(&self) -> std::io::Result<()> {
        self.faults.hit_storage_boundary("before_response_delivery")
    }
}
