use programmable_cli::authoring_error::AuthoringError;
use programmable_cli::document_path::DocumentSegment;
use serde_json::json;

#[test]
fn authoring_error_wire_fields_preserve_paths_and_optional_details() {
    let rejected = AuthoringError::new("INVALID_ARGUMENT", "Invalid value.", "Repair the value.");
    let required = json!({
        "code": "INVALID_ARGUMENT",
        "message": "Invalid value.",
        "recovery": "Repair the value."
    });
    assert_eq!(serde_json::to_value(&rejected).unwrap(), required);
    let path = [
        DocumentSegment::Key { key: "0/~".into() },
        DocumentSegment::Index { index: 0 },
    ];
    let located = rejected.clone().at_document_path(path).at_batch_index(0);
    let mut expected = required.clone();
    expected["path"] = json!([{"key": "0/~"}, {"index": 0}]);
    expected["failed_operation_index"] = json!(0);
    assert_eq!(serde_json::to_value(&located).unwrap(), expected);
    assert_eq!(
        serde_json::from_value::<AuthoringError>(expected).unwrap(),
        located
    );

    let root = rejected
        .clone()
        .at_document_path(Vec::<DocumentSegment>::new());
    assert_eq!(serde_json::to_value(root).unwrap()["path"], json!([]));
    let pointer = rejected.at_document_path("/0~1~0");
    assert_eq!(serde_json::to_value(pointer).unwrap()["path"], "/0~1~0");
    println!(
        "AUTHORING_ERROR_INLINE_BYTES={}",
        std::mem::size_of::<AuthoringError>()
    );
}

#[test]
fn authoring_error_decodes_existing_validation_details_and_rejects_unknown_fields() {
    let expected = json!({
        "code": "SCHEMA_VIOLATION",
        "message": "Candidate does not satisfy the attached schema.",
        "recovery": "Use validate to inspect findings, edit the preserved draft, then commit again.",
        "path": "/port",
        "validation": {
            "valid": false,
            "findings": [{
                "instance_path": "/port",
                "schema_path": "/properties/port/type",
                "keyword": "type",
                "message": "expected integer",
                "message_truncated": false
            }],
            "truncated": true
        }
    });
    let decoded: AuthoringError = serde_json::from_value(expected.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded.clone()).unwrap(), expected);
    assert!(!decoded.publication_uncertain());
    for code in ["PERSISTENCE_UNCERTAIN", "EXPORT_UNCERTAIN"] {
        let uncertain = decoded.clone().with_error_code(code);
        assert!(uncertain.publication_uncertain());
        let mut classified = expected.clone();
        classified["code"] = json!(code);
        assert_eq!(serde_json::to_value(uncertain).unwrap(), classified);
    }
    let mut unknown_error_field = expected.clone();
    unknown_error_field["unexpected"] = json!(true);
    let mut unknown_report_field = expected.clone();
    unknown_report_field["validation"]["unexpected"] = json!(true);
    let mut unknown_finding_field = expected;
    unknown_finding_field["validation"]["findings"][0]["unexpected"] = json!(true);
    for malformed in [
        unknown_error_field,
        unknown_report_field,
        unknown_finding_field,
    ] {
        assert!(serde_json::from_value::<AuthoringError>(malformed).is_err());
    }
}
