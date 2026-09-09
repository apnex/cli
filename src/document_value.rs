//! JSON authoring preserves number tokens and decoded keys without floating-point conversion.

use crate::authoring_error::{AuthoringError, authoring_limit_error};
use serde::de::{DeserializeOwned, MapAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;
use std::collections::BTreeMap;
use std::fmt;

pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;
pub const MAX_DOCUMENT_DEPTH: usize = 64;
pub const MAX_SCALAR_BYTES: usize = 64 * 1024;
pub const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_CHECKPOINT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_BATCH_EDITS: usize = 256;

/// An authored JSON value preserves the exact validated spelling of each number.
#[derive(Clone, Debug)]
pub enum DocumentValue {
    Object(BTreeMap<String, DocumentValue>),
    Array(Vec<DocumentValue>),
    String(String),
    Number(Box<RawValue>),
    Boolean(bool),
    Null,
}

impl PartialEq for DocumentValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Object(left), Self::Object(right)) => left == right,
            (Self::Array(left), Self::Array(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left.get() == right.get(),
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Null, Self::Null) => true,
            _ => false,
        }
    }
}
impl Eq for DocumentValue {}

impl Serialize for DocumentValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Object(values) => {
                let mut map = serializer.serialize_map(Some(values.len()))?;
                for (key, value) in values {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            Self::Array(values) => {
                let mut sequence = serializer.serialize_seq(Some(values.len()))?;
                for value in values {
                    sequence.serialize_element(value)?;
                }
                sequence.end()
            }
            Self::String(value) => serializer.serialize_str(value),
            Self::Number(value) => value.serialize(serializer),
            Self::Boolean(value) => serializer.serialize_bool(*value),
            Self::Null => serializer.serialize_unit(),
        }
    }
}

impl<'de> Deserialize<'de> for DocumentValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        Self::parse_document(raw.get()).map_err(serde::de::Error::custom)
    }
}

struct UniqueObjectEntries(Vec<(String, Box<RawValue>)>);

impl<'de> Deserialize<'de> for UniqueObjectEntries {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueObjectVisitor;
        impl<'de> Visitor<'de> for UniqueObjectVisitor {
            type Value = UniqueObjectEntries;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an object with unique decoded keys")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut entries = Vec::new();
                let mut keys = std::collections::BTreeSet::new();
                while let Some((key, value)) = map.next_entry::<String, Box<RawValue>>()? {
                    if !keys.insert(key.clone()) {
                        return Err(serde::de::Error::custom(format!(
                            "Duplicate decoded JSON key: {key:?}"
                        )));
                    }
                    entries.push((key, value));
                }
                Ok(UniqueObjectEntries(entries))
            }
        }
        deserializer.deserialize_map(UniqueObjectVisitor)
    }
}

fn document_syntax_error(error: impl fmt::Display) -> AuthoringError {
    AuthoringError::new(
        "INVALID_VALUE",
        format!("JSON value rejected: {error}"),
        "Supply valid JSON with unique decoded keys and valid Unicode.",
    )
}

impl DocumentValue {
    /// Parse a complete document with exact numbers and recursive duplicate-key rejection.
    pub fn parse_document(text: &str) -> Result<Self, AuthoringError> {
        let raw: Box<RawValue> = serde_json::from_str(text).map_err(document_syntax_error)?;
        let document = Self::parse_raw_value(raw, 0, MAX_DOCUMENT_DEPTH)?;
        document.validate_document_limits()?;
        Ok(document)
    }

    fn parse_raw_value(
        raw: Box<RawValue>,
        depth: usize,
        max_depth: usize,
    ) -> Result<Self, AuthoringError> {
        if depth > max_depth {
            return Err(authoring_limit_error(
                "JSON nesting depth exceeds the supported boundary.",
            ));
        }
        let text = raw.get();
        match text.as_bytes().first().copied() {
            Some(b'{') => {
                let entries: UniqueObjectEntries =
                    serde_json::from_str(text).map_err(document_syntax_error)?;
                let mut values = BTreeMap::new();
                for (key, value) in entries.0 {
                    values.insert(key, Self::parse_raw_value(value, depth + 1, max_depth)?);
                }
                Ok(Self::Object(values))
            }
            Some(b'[') => {
                let entries: Vec<Box<RawValue>> =
                    serde_json::from_str(text).map_err(document_syntax_error)?;
                Ok(Self::Array(
                    entries
                        .into_iter()
                        .map(|value| Self::parse_raw_value(value, depth + 1, max_depth))
                        .collect::<Result<_, _>>()?,
                ))
            }
            Some(b'"') => Ok(Self::String(
                serde_json::from_str(text).map_err(document_syntax_error)?,
            )),
            Some(b't' | b'f') => Ok(Self::Boolean(
                serde_json::from_str(text).map_err(document_syntax_error)?,
            )),
            Some(b'n') => Ok(Self::Null),
            Some(b'-' | b'0'..=b'9') => Ok(Self::Number(raw)),
            _ => Err(document_syntax_error("unexpected JSON token")),
        }
    }

    /// Serialize compact JSON in decoded-key order without changing number spelling.
    pub fn compact_document_json(&self) -> String {
        serde_json::to_string(self).expect("Validated document values serialize")
    }

    /// Describe the current node without dumping its complete subtree.
    pub fn document_kind(&self) -> &'static str {
        match self {
            Self::Object(_) => "object",
            Self::Array(_) => "array",
            Self::String(_) => "string",
            Self::Number(_) => "number",
            Self::Boolean(_) => "boolean",
            Self::Null => "null",
        }
    }

    /// Enforce the document, nesting, and scalar limits on a proposed private tree.
    pub fn validate_document_limits(&self) -> Result<(), AuthoringError> {
        fn walk(value: &DocumentValue, depth: usize) -> Result<(), AuthoringError> {
            if depth > MAX_DOCUMENT_DEPTH {
                return Err(authoring_limit_error("Document depth exceeds 64."));
            }
            match value {
                DocumentValue::Object(values) => {
                    for (key, value) in values {
                        if key.len() > MAX_SCALAR_BYTES {
                            return Err(authoring_limit_error("Object key exceeds 64 KiB."));
                        }
                        walk(value, depth + 1)?;
                    }
                }
                DocumentValue::Array(values) => {
                    for value in values {
                        walk(value, depth + 1)?;
                    }
                }
                DocumentValue::String(value) if value.len() > MAX_SCALAR_BYTES => {
                    return Err(authoring_limit_error("String value exceeds 64 KiB."));
                }
                DocumentValue::Number(value) if value.get().len() > MAX_SCALAR_BYTES => {
                    return Err(authoring_limit_error("Number token exceeds 64 KiB."));
                }
                _ => {}
            }
            Ok(())
        }
        walk(self, 0)?;
        if self.compact_document_json().len() > MAX_DOCUMENT_BYTES {
            return Err(authoring_limit_error("Compact document exceeds 1 MiB."));
        }
        Ok(())
    }
}

/// Decode an envelope only after checking UTF-8, duplicate keys, and bounded input size.
pub fn decode_authoring_json<T: DeserializeOwned>(
    bytes: &[u8],
    maximum: usize,
    code: &str,
) -> Result<T, AuthoringError> {
    if bytes.len() > maximum {
        return Err(authoring_limit_error(
            "JSON envelope exceeds its byte limit.",
        ));
    }
    let text =
        std::str::from_utf8(bytes).map_err(|e| document_syntax_error(e).with_error_code(code))?;
    let raw: Box<RawValue> =
        serde_json::from_str(text).map_err(|e| document_syntax_error(e).with_error_code(code))?;
    DocumentValue::parse_raw_value(raw, 0, 128).map_err(|e| e.with_error_code(code))?;
    serde_json::from_str(text).map_err(|e| document_syntax_error(e).with_error_code(code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_numbers_and_strings_survive_without_coercion() {
        let source = r#"{"large":9007199254740993,"exponent":1e400,"zero":-0,"quota":1.2300,"text":"true","unicode":"\u0061"}"#;
        let value = DocumentValue::parse_document(source).unwrap();
        let encoded = value.compact_document_json();
        for token in ["9007199254740993", "1e400", "-0", "1.2300", "\"true\""] {
            assert!(encoded.contains(token));
        }
        assert_eq!(DocumentValue::parse_document(&encoded).unwrap(), value);
        assert_ne!(
            DocumentValue::parse_document("1").unwrap(),
            DocumentValue::parse_document("1.0").unwrap()
        );
    }

    #[test]
    fn invalid_numbers_duplicates_and_unicode_are_rejected_recursively() {
        for source in [
            "NaN",
            "Infinity",
            "01",
            "+1",
            "1e",
            "1.",
            "true false",
            r#""\uD800""#,
            r#"{"outer":{"name":0,"n\u0061me":1}}"#,
            r#"[{"":0,"":1}]"#,
        ] {
            assert!(
                DocumentValue::parse_document(source).is_err(),
                "accepted {source}"
            );
        }
        let invalid_utf8 = [b'"', 0xff, b'"'];
        assert!(
            decode_authoring_json::<serde_json::Value>(&invalid_utf8, 10, "INVALID_REQUEST")
                .is_err()
        );
    }
}
