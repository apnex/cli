//! Exact-endpoint grants provide bounded native HTTP JSON reads on literal loopback.
use crate::authoring_error::AuthoringError;
use crate::cli_definition::{CliCapabilityId, CliConnectedProvider, validate_cli_name};
use crate::cli_read_observation::CliReadObservation;
use crate::cli_view_expression::{OutputEvaluationBudget, OutputScope, compile_output_expression};
use crate::document_value::{DocumentValue, MAX_DOCUMENT_BYTES};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::time::Duration;

fn http_error(code: &str, message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        code,
        message,
        "Check the explicitly granted loopback endpoint and response contract, then issue a new read.",
    )
}

/// Restrict authority before URL parsing; never resolve names or normalize hosts.
pub fn validate_loopback_http_base(base: &str) -> Result<&str, AuthoringError> {
    let base = base.strip_suffix('/').unwrap_or(base);
    let port = base
        .strip_prefix("http://127.0.0.1:")
        .or_else(|| base.strip_prefix("http://[::1]:"));
    if !port.is_some_and(|port| {
        !port.is_empty()
            && port.bytes().all(|b| b.is_ascii_digit())
            && port.parse::<u16>().is_ok_and(|port| port > 0)
    }) {
        return Err(http_error(
            "INVALID_HTTP_GRANT",
            "Expected http://127.0.0.1:PORT or http://[::1]:PORT with port 1 through 65535.",
        ));
    }
    Ok(base)
}

pub fn validate_http_resource_path(path: &str) -> Result<(), AuthoringError> {
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.len() > 2048
        || !path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/-_.".contains(&b))
        || path.split('/').any(|part| part == "." || part == "..")
    {
        return Err(http_error(
            "INVALID_HTTP_GRANT",
            "Expected a fixed absolute resource path without traversal, query, or fragment.",
        ));
    }
    Ok(())
}

#[derive(Clone)]
struct JsonHttpGetGrant {
    url: String,
    require: DocumentValue,
}

/// Runtime authority is not serializable and never follows an interface export.
#[derive(Clone, Default)]
pub struct JsonHttpGetGrants {
    endpoints: BTreeMap<CliCapabilityId, JsonHttpGetGrant>,
}

pub fn validate_http_response_requirement(require: &DocumentValue) -> Result<(), AuthoringError> {
    require.validate_document_limits()?;
    compile_output_expression(require, false, false, 0, &mut 2048).map(|_| ())
}

impl JsonHttpGetGrants {
    pub fn grant_json_http_get(
        &mut self,
        capability: CliCapabilityId,
        base: &str,
        path: &str,
        require: DocumentValue,
    ) -> Result<(), AuthoringError> {
        validate_cli_name(&capability.0)?;
        let base = validate_loopback_http_base(base)?;
        validate_http_resource_path(path)?;
        validate_http_response_requirement(&require)?;
        if self.endpoints.len() >= 32 || self.endpoints.contains_key(&capability) {
            return Err(http_error(
                "INVALID_HTTP_GRANT",
                "HTTP grant is duplicated or exceeds 32 endpoints.",
            ));
        }
        self.endpoints.insert(
            capability,
            JsonHttpGetGrant {
                url: format!("{base}{path}"),
                require,
            },
        );
        Ok(())
    }

    pub fn has_json_http_get_grant(&self, capability: &CliCapabilityId) -> bool {
        self.endpoints.contains_key(capability)
    }

    pub fn read_granted_json_http(
        &self,
        capability: &CliCapabilityId,
    ) -> Result<(DocumentValue, CliReadObservation), AuthoringError> {
        let grant = self.endpoints.get(capability).ok_or_else(|| {
            http_error(
                "CAPABILITY_NOT_GRANTED",
                format!("HTTP capability is not granted: {}", capability.0),
            )
        })?;
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .proxy(None)
            .max_redirects(0)
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(2)))
            .timeout_global(Some(Duration::from_secs(7)))
            .build()
            .into();
        let mut response = agent
            .get(&grant.url)
            .header("Accept", "application/json")
            .call()
            .map_err(|_| {
                http_error(
                    "HTTP_TRANSPORT_FAILED",
                    format!("Cannot read HTTP capability {}.", capability.0),
                )
            })?;
        if response.status().as_u16() != 200 {
            return Err(http_error(
                "HTTP_STATUS_FAILED",
                format!(
                    "HTTP capability {} returned status {}.",
                    capability.0,
                    response.status().as_u16()
                ),
            ));
        }
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(MAX_DOCUMENT_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| {
                http_error(
                    "HTTP_TRANSPORT_FAILED",
                    "HTTP body could not be read within its deadline.",
                )
            })?;
        if bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(http_error(
                "INVALID_HTTP_DOCUMENT",
                "HTTP response exceeds one MiB.",
            ));
        }
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| http_error("INVALID_HTTP_DOCUMENT", "HTTP response is not UTF-8 JSON."))?;
        let output = DocumentValue::parse_document(text)
            .map_err(|error| error.with_error_code("INVALID_HTTP_DOCUMENT"))?;
        let require = compile_output_expression(&grant.require, false, false, 0, &mut 2048)?;
        let accepted = require
            .evaluate(
                OutputScope {
                    root: &output,
                    row: None,
                    item: None,
                },
                &mut OutputEvaluationBudget::default(),
            )
            .map_err(|error| error.with_error_code("INVALID_HTTP_DOCUMENT"))?;
        if !matches!(accepted, Some(DocumentValue::Boolean(true))) {
            return Err(http_error(
                "INVALID_HTTP_DOCUMENT",
                "HTTP response does not satisfy its declared requirement.",
            ));
        }
        let observation = CliReadObservation {
            provider: CliConnectedProvider::JsonHttpGet,
            capability: capability.clone(),
            source_sha256: format!("{:x}", Sha256::digest(&bytes)),
            source_bytes: bytes.len(),
            output_sha256: format!(
                "{:x}",
                Sha256::digest(output.compact_document_json().as_bytes())
            ),
        };
        Ok((output, observation))
    }
}
