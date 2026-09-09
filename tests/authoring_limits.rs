mod authoring_fixture;
use authoring_fixture::*;
use programmable_cli::authoring_runtime::checked_response_bytes;
use programmable_cli::document_value::{
    DocumentValue, MAX_BATCH_EDITS, MAX_CHECKPOINT_BYTES, MAX_DOCUMENT_BYTES, MAX_DOCUMENT_DEPTH,
    MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, MAX_SCALAR_BYTES,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;

fn set_constructor(path: Value, value: Value) -> Value {
    json!({"operation":"set","arguments":{"path":{"base":"root","segments":path},"value":value}})
}

fn exact_size_document(size: usize) -> DocumentValue {
    let last_size = size - 49 - 15 * MAX_SCALAR_BYTES;
    let mut values = vec![DocumentValue::String("x".repeat(MAX_SCALAR_BYTES)); 15];
    values.push(DocumentValue::String("x".repeat(last_size)));
    let document = DocumentValue::Array(values);
    assert_eq!(document.compact_document_json().len(), size);
    document
}

#[test]
fn declared_scalar_depth_document_request_checkpoint_and_batch_boundaries() {
    for kind in ["string", "number", "key"] {
        for size in [MAX_SCALAR_BYTES - 1, MAX_SCALAR_BYTES, MAX_SCALAR_BYTES + 1] {
            let fixture = AuthoringFixture::new();
            fixture.seed(&json!({}));
            let before = fixture.checkpoint_bytes();
            let mut process = fixture.start(false, false);
            let value = if kind == "number" {
                "1".repeat(size)
            } else {
                "x".repeat(size)
            };
            let request = if kind == "key" {
                set_constructor(json!([{"key":value}]), json!({"kind":"null"}))
            } else {
                set_constructor(json!([]), json!({"kind":kind,"value":value}))
            };
            let response = process.request(&request);
            if size > MAX_SCALAR_BYTES {
                check_rejection(&fixture, &before, &response, "LIMIT_EXCEEDED");
            } else {
                assert_eq!(response["status"], "ok", "{kind} size {size}: {response}");
                let state = fixture.checkpoint_state();
                match kind {
                    "string" => assert_eq!(state.candidate, DocumentValue::String(value)),
                    "number" => assert_eq!(state.candidate.compact_document_json(), value),
                    _ => assert_eq!(
                        state.candidate,
                        DocumentValue::Object(BTreeMap::from([(value, DocumentValue::Null)]))
                    ),
                }
            }
            assert!(process.close().success());
        }
    }
    for size in [MAX_SCALAR_BYTES - 1, MAX_SCALAR_BYTES, MAX_SCALAR_BYTES + 1] {
        let fixture = AuthoringFixture::new();
        fs::write(&fixture.intent, "i".repeat(size)).unwrap();
        let mut process = fixture.start(false, true);
        if size > MAX_SCALAR_BYTES {
            assert_eq!(process.header["error"]["code"], "LIMIT_EXCEEDED");
            assert!(!fixture.checkpoint.exists());
            assert!(!process.close().success());
        } else {
            assert_eq!(process.header["intent_text"], "i".repeat(size));
            assert!(process.close().success());
        }
    }
    for depth in [
        MAX_DOCUMENT_DEPTH - 1,
        MAX_DOCUMENT_DEPTH,
        MAX_DOCUMENT_DEPTH + 1,
    ] {
        let initial = format!("{}{{}}{}", "[".repeat(depth - 1), "]".repeat(depth - 1));
        let fixture = AuthoringFixture::new();
        fixture.seed(&json!({"candidate_json":initial}));
        let before = fixture.checkpoint_bytes();
        let mut process = fixture.start(false, false);
        let mut segments = vec![json!({"index":0}); depth - 1];
        segments.push(json!({"key":"x"}));
        let response = process.request(&set_constructor(json!(segments), json!({"kind":"null"})));
        if depth > MAX_DOCUMENT_DEPTH {
            check_rejection(&fixture, &before, &response, "LIMIT_EXCEEDED");
        } else {
            assert_eq!(response["status"], "ok", "depth {depth}: {response}");
            fixture
                .checkpoint_state()
                .candidate
                .validate_document_limits()
                .unwrap();
        }
        assert!(process.close().success());
    }
    for size in [
        MAX_DOCUMENT_BYTES - 1,
        MAX_DOCUMENT_BYTES,
        MAX_DOCUMENT_BYTES + 1,
    ] {
        let fixture = AuthoringFixture::new();
        fixture.seed(&json!({"candidate_json":exact_size_document(MAX_DOCUMENT_BYTES-2).compact_document_json()}));
        let before = fixture.checkpoint_bytes();
        let mut process = fixture.start(false, false);
        let new_string = "x".repeat(size - 49 - 15 * MAX_SCALAR_BYTES);
        let response = process.request(&set_constructor(
            json!([{"index":15}]),
            json!({"kind":"string","value":new_string}),
        ));
        if size > MAX_DOCUMENT_BYTES {
            check_rejection(&fixture, &before, &response, "LIMIT_EXCEEDED");
        } else {
            assert_eq!(
                response["status"], "ok",
                "document bytes {size}: {response}"
            );
            assert_eq!(
                fixture
                    .checkpoint_state()
                    .candidate
                    .compact_document_json()
                    .len(),
                size
            );
        }
        assert!(process.close().success());
    }
    for size in [
        MAX_REQUEST_BYTES - 1,
        MAX_REQUEST_BYTES,
        MAX_REQUEST_BYTES + 1,
    ] {
        let fixture = AuthoringFixture::new();
        fixture.seed(&json!({}));
        let before = fixture.checkpoint_bytes();
        let mut process = fixture.start(false, false);
        let request = process.canonical_request(&json!({"operation":"status","arguments":{}}));
        let mut line = serde_json::to_vec(&request).unwrap();
        line.resize(size - 1, b' ');
        line.push(b'\n');
        assert_eq!(line.len(), size);
        let response = process.raw_line(&line);
        if size > MAX_REQUEST_BYTES {
            check_rejection(&fixture, &before, &response, "LIMIT_EXCEEDED");
        } else {
            assert_eq!(response["status"], "ok");
            assert_eq!(fixture.checkpoint_bytes(), before);
        }
        assert_eq!(
            process.request(&json!({"operation":"status","arguments":{}}))["status"],
            "ok"
        );
        assert!(process.close().success());
    }
    for size in [
        MAX_CHECKPOINT_BYTES - 1,
        MAX_CHECKPOINT_BYTES,
        MAX_CHECKPOINT_BYTES + 1,
    ] {
        let fixture = AuthoringFixture::new();
        fixture.seed(&json!({}));
        let mut checkpoint = fixture.checkpoint_bytes();
        checkpoint.resize(size, b' ');
        fs::write(&fixture.checkpoint, &checkpoint).unwrap();
        assert_eq!(fixture.checkpoint_bytes().len(), size);
        if size > MAX_CHECKPOINT_BYTES {
            fixture.reject_open("LIMIT_EXCEEDED");
        } else {
            let mut process = fixture.start(false, false);
            assert_eq!(
                process.header["event"], "session_open",
                "{}",
                process.header
            );
            assert!(process.close().success());
            assert_eq!(fixture.checkpoint_bytes(), checkpoint);
        }
    }
    for count in [
        0,
        1,
        2,
        MAX_BATCH_EDITS - 1,
        MAX_BATCH_EDITS,
        MAX_BATCH_EDITS + 1,
    ] {
        let fixture = AuthoringFixture::new();
        fixture.seed(&json!({}));
        let before = fixture.checkpoint_bytes();
        let mut process = fixture.start(false, false);
        let operations = vec![set_constructor(json!([{"key":"x"}]), json!({"kind":"null"})); count];
        let response =
            process.request(&json!({"operation":"batch","arguments":{"operations":operations}}));
        if count == 0 || count > MAX_BATCH_EDITS {
            check_rejection(&fixture, &before, &response, "LIMIT_EXCEEDED");
        } else {
            assert_eq!(response["status"], "ok");
            assert_eq!(response["session"]["revision"], "1");
            assert_eq!(
                fixture.checkpoint_state().candidate.compact_document_json(),
                "{\"x\":null}"
            );
        }
        assert!(process.close().success());
    }
    report_pass("declared-limits:scalar-depth-document-request-checkpoint-batch");
}

#[test]
fn receipt_overhead_and_response_bound_reject_before_publication() {
    let long_key = "k".repeat(MAX_SCALAR_BYTES);
    for children in [125, 126, 128, 10_000] {
        let fixture = AuthoringFixture::new();
        let nested = DocumentValue::Object(
            (0..children)
                .map(|index| (format!("child{index:03}"), DocumentValue::Null))
                .collect(),
        );
        let document = DocumentValue::Object(BTreeMap::from([(long_key.clone(), nested)]));
        fixture.seed(&json!({"candidate_json":document.compact_document_json()}));
        let before = fixture.checkpoint_bytes();
        let mut process = fixture.start(false, false);
        let response = process.request(&set_constructor(
            json!([{"key":long_key}]),
            json!({"kind":"object"}),
        ));
        if children == 125 {
            assert_eq!(response["status"], "ok", "{response}");
            assert_eq!(
                response["result"]["changed_paths"]
                    .as_array()
                    .unwrap()
                    .len(),
                children
            );
            assert!(checked_response_bytes(&response).unwrap().len() < MAX_RESPONSE_BYTES);
            let checkpoint = fixture.checkpoint_bytes();
            assert!(checkpoint.len() <= MAX_CHECKPOINT_BYTES);
            assert!(process.close().success());
            let mut reopened = fixture.start(false, false);
            assert_eq!(
                reopened.header["event"], "session_open",
                "{}",
                reopened.header
            );
            assert_eq!(reopened.header["last_receipt"]["response"], response);
            let status = reopened.request(&json!({"operation":"status","arguments":{}}));
            assert_eq!(status["result"]["last_receipt"]["response"], response);
            let shown = reopened.request(&json!({"operation":"show","arguments":{}}));
            assert_eq!(
                shown["result"]["json_text"],
                fixture.checkpoint_state().candidate.compact_document_json()
            );
            assert_eq!(fixture.checkpoint_bytes(), checkpoint);
            assert!(reopened.close().success());
        } else {
            check_rejection(&fixture, &before, &response, "LIMIT_EXCEEDED");
            let message = response["error"]["message"].as_str().unwrap();
            assert!(
                message.contains(if children == 126 {
                    "Checkpoint"
                } else {
                    "Response"
                }),
                "Unexpected limiting envelope: {message}"
            );
            assert!(process.close().success());
        }
    }
    for size in [
        MAX_RESPONSE_BYTES - 1,
        MAX_RESPONSE_BYTES,
        MAX_RESPONSE_BYTES + 1,
    ] {
        let value = json!("x".repeat(size - 3));
        assert_eq!(serde_json::to_vec(&value).unwrap().len() + 1, size);
        let bytes = checked_response_bytes(&value);
        if size > MAX_RESPONSE_BYTES {
            assert_eq!(bytes.unwrap_err().code, "LIMIT_EXCEEDED");
        } else {
            assert_eq!(bytes.unwrap().len(), size);
        }
    }
    report_pass("declared-limits:receipt-overhead");
    report_pass("bounded-result-delivery");
}

#[test]
fn maximal_explicit_show_and_task_text_are_delivered_completely() {
    let fixture = AuthoringFixture::new();
    fs::write(&fixture.intent, "t".repeat(MAX_SCALAR_BYTES)).unwrap();
    let document = exact_size_document(MAX_DOCUMENT_BYTES);
    fixture.seed(&json!({"candidate_json":document.compact_document_json()}));
    let mut process = fixture.start(false, false);
    assert_eq!(
        process.header["intent_text"].as_str().unwrap().len(),
        MAX_SCALAR_BYTES
    );
    let response = process.request(&json!({"operation":"show","arguments":{}}));
    assert_eq!(
        response["result"]["json_text"],
        document.compact_document_json()
    );
    assert!(checked_response_bytes(&response).unwrap().len() <= MAX_RESPONSE_BYTES);
    assert!(process.close().success());
}

#[test]
fn maximal_supported_context_path_survives_navigation_receipt_and_reopen() {
    let fixture = AuthoringFixture::new();
    fs::write(&fixture.intent, "t".repeat(MAX_SCALAR_BYTES)).unwrap();
    let mut keys = vec!["k".repeat(MAX_SCALAR_BYTES); 15];
    keys.push("k".repeat(MAX_DOCUMENT_BYTES - 5 * MAX_DOCUMENT_DEPTH - 4 - 15 * MAX_SCALAR_BYTES));
    keys.resize(MAX_DOCUMENT_DEPTH, String::new());
    let mut document = DocumentValue::Null;
    for key in keys.iter().rev() {
        document = DocumentValue::Object(BTreeMap::from([(key.clone(), document)]));
    }
    assert_eq!(document.compact_document_json().len(), MAX_DOCUMENT_BYTES);
    let context: Vec<_> = keys.into_iter().map(|key| json!({"key":key})).collect();
    fixture.seed(&json!({"candidate_json":document.compact_document_json(),"context":context}));
    let mut process = fixture.start(false, false);
    assert_eq!(process.header["session"]["context"], json!(context));
    let response = process.request(
        &json!({"operation":"edit","arguments":{"path":{"base":"context","segments":[]}}}),
    );
    assert_eq!(response["status"], "ok", "Large context navigation failed");
    assert_eq!(response["session"]["context"], json!(context));
    assert!(process.close().success());
    let mut reopened = fixture.start(false, false);
    assert_eq!(reopened.header["last_receipt"]["response"], response);
    let shown = reopened
        .request(&json!({"operation":"show","arguments":{"path":{"base":"root","segments":[]}}}));
    assert_eq!(
        shown["result"]["json_text"],
        document.compact_document_json()
    );
    assert!(reopened.close().success());
}

#[test]
fn invalid_framing_inside_terminal_batch_cannot_publish_valid_prefix() {
    for malformed in [b"\xff\n".to_vec(), {
        let mut bytes = vec![b'x'; MAX_REQUEST_BYTES];
        bytes.push(b'\n');
        bytes
    }] {
        let fixture = AuthoringFixture::new();
        fixture.seed(&json!({}));
        let before = fixture.checkpoint_bytes();
        let mut process = fixture.start(true, false);
        process.terminal("batch");
        process.terminal("set /x null");
        let first = process.raw_line(&malformed);
        assert_eq!(first["status"], "error");
        let end = process.terminal("end");
        assert_eq!(end["status"], "error");
        assert_eq!(end["error"]["failed_operation_index"], 1);
        assert_eq!(fixture.checkpoint_bytes(), before);
        assert!(process.close().success());
    }
}
