mod authoring_fixture;
use authoring_fixture::*;
use programmable_cli::terminal_input::compile_terminal_request;
use serde_json::{Value, json};
use std::fs;

#[test]
fn service_catalog_journey_through_both_process_presentations() {
    let corpus = fixture_corpus();
    let mut observations = Vec::new();
    let mut exports = Vec::new();
    for terminal in [false, true] {
        let fixture = AuthoringFixture::new();
        let mut process = fixture.start(terminal, true);
        assert_eq!(
            process.header["event"], "session_open",
            "{}",
            process.header
        );
        assert_eq!(
            process.header["intent_text"],
            fs::read_to_string(&fixture.intent).unwrap()
        );
        let mut events = vec![normalize_generated_fields(
            &process.header,
            &fixture.directory,
        )];
        for (index, step) in corpus["journey"]["steps"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let before = fixture.checkpoint_bytes();
            let previous = process.header.clone();
            let response = if step["harness_action"] == "close_and_reopen" {
                assert!(process.close().success());
                process = fixture.start(terminal, false);
                process.header.clone()
            } else if terminal {
                let compiled = compile_terminal_request(
                    step["terminal"].as_str().unwrap(),
                    &fixture_definition(),
                    &fixture.checkpoint_state(),
                )
                .unwrap();
                assert_eq!(
                    json!(compiled.arguments),
                    step["request"]["arguments"],
                    "Terminal lowering at journey step {index}"
                );
                assert_eq!(
                    compiled.operation,
                    step["request"]["operation"].as_str().unwrap()
                );
                process.terminal(step["terminal"].as_str().unwrap())
            } else {
                process.request(&step["request"])
            };
            check_corpus_expectations(
                &fixture,
                &response,
                &step["expect"],
                &before,
                &previous,
                step.get("request"),
            );
            events.push(normalize_generated_fields(&response, &fixture.directory));
            report_pass(&format!(
                "service-catalog:{}:{}",
                if terminal { "terminal" } else { "machine" },
                index + 1
            ));
        }
        assert!(process.close().success());
        let presentation = if terminal { "terminal" } else { "machine" };
        retain_acceptance_bytes(
            &format!("{presentation}-journey.json"),
            &serde_json::to_vec_pretty(&events).unwrap(),
        );
        retain_acceptance_bytes(
            &format!("{presentation}-catalog.json"),
            &fs::read(fixture.directory.join("catalog.json")).unwrap(),
        );
        retain_acceptance_bytes(
            &format!("{presentation}-draft.json"),
            &fs::read(fixture.directory.join("draft.json")).unwrap(),
        );
        exports.push((
            fs::read(fixture.directory.join("draft.json")).unwrap(),
            fs::read(fixture.directory.join("catalog.json")).unwrap(),
        ));
        observations.push(events);
    }
    assert_eq!(
        observations[0], observations[1],
        "Human and machine semantic outcomes diverged"
    );
    assert_eq!(exports[0], exports[1]);
    report_pass("terminal-machine-parity");
}

#[test]
fn recorded_operation_cases_execute_against_application() {
    let corpus = fixture_corpus();
    for case in corpus["operation_cases"].as_array().unwrap() {
        let modes: Vec<bool> = match (case["request"].is_null(), case["terminal"].is_string()) {
            (true, _) => vec![true],
            (false, true) => vec![false, true],
            _ => vec![false],
        };
        for terminal in modes {
            let fixture = AuthoringFixture::new();
            fixture.seed(&case["initial"]);
            let before = fixture.checkpoint_bytes();
            let mut process = fixture.start(terminal, false);
            assert_eq!(
                process.header["event"], "session_open",
                "{}",
                process.header
            );
            let previous = process.header.clone();
            let response = if terminal {
                process.terminal(case["terminal"].as_str().unwrap())
            } else {
                process.request(&case["request"])
            };
            check_corpus_expectations(
                &fixture,
                &response,
                &case["expect"],
                &before,
                &previous,
                if case["request"].is_null() {
                    None
                } else {
                    Some(&case["request"])
                },
            );
            assert!(process.close().success());
        }
        report_pass(case["id"].as_str().unwrap());
    }
}

#[test]
fn terminal_batch_edits_share_atomic_dispatch_and_context_rules() {
    let fixture = AuthoringFixture::new();
    fixture.seed(&json!({}));
    let mut process = fixture.start(true, false);
    let before = fixture.checkpoint_bytes();
    assert_eq!(process.terminal("batch")["state"], "unsent");
    for command in [
        "set /services array",
        "append /services object",
        "set /services/0/name string api",
    ] {
        assert_eq!(process.terminal(command)["state"], "unsent");
        assert_eq!(fixture.checkpoint_bytes(), before);
    }
    let response = process.terminal("end");
    assert_eq!(response["status"], "ok", "{response}");
    assert_eq!(response["session"]["revision"], "1");
    assert_eq!(
        fixture.checkpoint_state().candidate.compact_document_json(),
        r#"{"services":[{"name":"api"}]}"#
    );
    assert!(process.close().success());
}

#[test]
fn duplicate_and_unknown_request_shapes_reject_without_losing_next_line() {
    let fixture = AuthoringFixture::new();
    fixture.seed(&json!({}));
    let mut process = fixture.start(false, false);
    let before = fixture.checkpoint_bytes();
    for line in [
        b"{broken\n".as_slice(),
        b"{\"operation\":\"status\",\"op\\u0065ration\":\"status\"}\n",
        b"{\"x\":\"\\uD800\"}\n",
        b"\xff\n",
    ] {
        let response = process.raw_line(line);
        check_rejection(&fixture, &before, &response, "INVALID_REQUEST");
        let response = process.request(&json!({"operation":"status","arguments":{}}));
        assert_eq!(response["status"], "ok");
    }
    let mut request = process.canonical_request(&json!({"operation":"top","arguments":{}}));
    request.as_object_mut().unwrap().remove("expected_revision");
    check_rejection(
        &fixture,
        &before,
        &process.submit(&request),
        "INVALID_REQUEST",
    );
    for invalid in [
        Value::Null,
        json!(0),
        json!("00"),
        json!("18446744073709551616"),
    ] {
        request["expected_revision"] = invalid;
        check_rejection(
            &fixture,
            &before,
            &process.submit(&request),
            "INVALID_REQUEST",
        );
    }
    assert!(process.close().success());
    report_pass("malformed-machine-line");
    report_pass("missing-revision");
}
