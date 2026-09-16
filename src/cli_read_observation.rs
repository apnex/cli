//! Provider-neutral observation identity; no endpoint or grant is retained.
use crate::cli_definition::{CliCapabilityId, CliConnectedProvider};
use crate::document_value::MAX_DOCUMENT_BYTES;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Connected read observations retain byte identity and exact output identity, never a target path or grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliReadObservation {
    pub provider: CliConnectedProvider,
    pub capability: CliCapabilityId,
    pub source_sha256: String,
    pub source_bytes: usize,
    pub output_sha256: String,
}

impl CliReadObservation {
    /// Validate source-free observation consistency; historical source bytes are not remeasured.
    pub fn validate_file_observation(&self, capability: &CliCapabilityId, output: &str) -> bool {
        self.validate_read_observation(&CliConnectedProvider::JsonFileRead, capability, output)
    }

    /// Match the declared provider while preserving legacy file receipt bytes.
    pub fn validate_read_observation(
        &self,
        provider: &CliConnectedProvider,
        capability: &CliCapabilityId,
        output: &str,
    ) -> bool {
        self.provider == *provider
            && self.capability == *capability
            && self.source_bytes > 0
            && self.source_bytes <= MAX_DOCUMENT_BYTES
            && self.source_sha256.len() == 64
            && self
                .source_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            && self.output_sha256 == format!("{:x}", Sha256::digest(output.as_bytes()))
    }
}
