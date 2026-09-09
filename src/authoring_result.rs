//! Result collections stop at their wire budget while being assembled, before publication.

use crate::authoring_error::{AuthoringError, authoring_limit_error};
use crate::document_value::MAX_RESPONSE_BYTES;
use serde_json::Value;

/// A bounded result array prevents repeated long paths from growing beyond the response envelope.
pub struct AuthoringResultEntries {
    values: Vec<Value>,
    serialized_bytes: usize,
}

impl Default for AuthoringResultEntries {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            serialized_bytes: 2,
        }
    }
}

impl AuthoringResultEntries {
    /// Account for the exact serialized entry before retaining it in the response collection.
    pub fn push_result_entry(&mut self, entry: Value) -> Result<(), AuthoringError> {
        let size = serde_json::to_vec(&entry)
            .expect("Authoring result entry serializes")
            .len()
            + usize::from(!self.values.is_empty());
        if self.serialized_bytes.saturating_add(size) > MAX_RESPONSE_BYTES {
            return Err(authoring_limit_error(
                "Response payload exceeds 8 MiB; request a smaller explicit view.",
            ));
        }
        self.serialized_bytes += size;
        self.values.push(entry);
        Ok(())
    }

    /// Return the complete bounded collection; no entries are silently discarded.
    pub fn into_result_entries(self) -> Vec<Value> {
        self.values
    }
}
