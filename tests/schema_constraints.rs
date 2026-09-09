mod authoring_fixture;
use authoring_fixture::*;
use programmable_cli::document_value::DocumentValue;
use programmable_cli::kernel_profile::KernelProfile;
use programmable_cli::operation_definition::OperationDefinition;
use programmable_cli::schema_constraint::SchemaConstraint;
use programmable_cli::terminal_completion::AuthoringCompleter;
use programmable_cli::terminal_input::compile_terminal_request;
use reedline::Completer;
use serde_json::{Value, json};
use std::fs;

fn constraint_fixture() -> AuthoringFixture {
    let fixture = AuthoringFixture::new();
    fs::copy(
        repository_path("docs/constraints/acceptance/TASK.md"),
        &fixture.intent,
    )
    .unwrap();
    fixture
}

fn definition(fixture: &AuthoringFixture) -> OperationDefinition {
    OperationDefinition::load_kernel_profile(
        &fixture.declaration,
        KernelProfile::ConstrainedAuthoring,
    )
    .unwrap()
}

fn start(fixture: &AuthoringFixture, terminal: bool, create: bool) -> AuthoringProcess {
    let mut command = fixture.command(terminal, create);
    command.arg("--constraints");
    let process = AuthoringProcess::spawn(command);
    assert_eq!(
        process.header["event"], "session_open",
        "{}",
        process.header
    );
    process
}

fn request(process: &AuthoringProcess, operation: &str, arguments: Value, state: bool) -> Value {
    let mut request = json!({"request_id":uuid::Uuid::new_v4().to_string(),"session_id":process.header["session"]["session_id"],"operation":operation,"arguments":arguments});
    if state {
        request["expected_revision"] = process.header["session"]["revision"].clone();
    }
    request
}

fn ok(response: &Value) {
    assert_eq!(response["status"], "ok", "{response}");
}
fn document(relative: &str) -> DocumentValue {
    DocumentValue::parse_document(&fs::read_to_string(repository_path(relative)).unwrap()).unwrap()
}

fn construct(
    fixture: &AuthoringFixture,
    terminal: bool,
    file: &str,
) -> (AuthoringProcess, Vec<Value>) {
    let mut process = start(fixture, terminal, true);
    let declaration = definition(fixture);
    let mut events = Vec::new();
    for line in fs::read_to_string(repository_path(file)).unwrap().lines() {
        assert!(
            !line.contains(['{', '}', '[', ']']),
            "Raw container in construction: {line}"
        );
        let response = if terminal {
            process.terminal(line)
        } else {
            let request =
                compile_terminal_request(line, &declaration, &fixture.checkpoint_state()).unwrap();
            process.submit(&serde_json::to_value(request).unwrap())
        };
        ok(&response);
        assert!(response["session"]["active_constraint"].is_null());
        assert!(response["session"]["active_interface"].is_null());
        events.push(response);
    }
    (process, events)
}

#[test]
fn authored_schema_guides_an_independent_instance_with_repair_and_source_free_reopen() {
    let mut journeys = Vec::new();
    for terminal in [false, true] {
        let schema_fixture = constraint_fixture();
        let (mut schema_process, construction) = construct(
            &schema_fixture,
            terminal,
            "docs/constraints/acceptance/schema.commands",
        );
        let expected_schema = document("docs/constraints/acceptance/service.schema.json");
        assert_eq!(schema_fixture.checkpoint_state().candidate, expected_schema);
        assert_eq!(schema_fixture.checkpoint_state().accepted, expected_schema);
        let exported = if terminal {
            schema_process.terminal("save service.schema.json")
        } else {
            schema_process.submit(&request(
                &schema_process,
                "save",
                json!({"destination":"service.schema.json"}),
                true,
            ))
        };
        ok(&exported);
        assert!(schema_process.close().success());
        let fixture = constraint_fixture();
        fs::copy(
            schema_fixture.directory.join("service.schema.json"),
            fixture.directory.join("schema.json"),
        )
        .unwrap();
        let mut process = start(&fixture, terminal, true);
        let mut events = Vec::new();
        let cases = [
            (
                "constrain schema.json",
                "constrain",
                json!({"source":"schema.json"}),
                true,
            ),
            ("guide", "guide", json!({}), false),
            ("validate", "validate", json!({}), false),
            (
                "set /name string catalog",
                "set",
                json!({"path":{"base":"root","segments":[{"key":"name"}]},"value":{"kind":"string","value":"catalog"}}),
                true,
            ),
            (
                "set /port string 5432",
                "set",
                json!({"path":{"base":"root","segments":[{"key":"port"}]},"value":{"kind":"string","value":"5432"}}),
                true,
            ),
            (
                "set /mode string production",
                "set",
                json!({"path":{"base":"root","segments":[{"key":"mode"}]},"value":{"kind":"string","value":"production"}}),
                true,
            ),
            ("commit", "commit", json!({}), true),
            ("validate", "validate", json!({}), false),
            (
                "set /port number 5432",
                "set",
                json!({"path":{"base":"root","segments":[{"key":"port"}]},"value":{"kind":"number","value":"5432"}}),
                true,
            ),
            (
                "set /tags array",
                "set",
                json!({"path":{"base":"root","segments":[{"key":"tags"}]},"value":{"kind":"array"}}),
                true,
            ),
            (
                "edit /tags",
                "edit",
                json!({"path":{"base":"root","segments":[{"key":"tags"}]}}),
                true,
            ),
            (
                "append . string public",
                "append",
                json!({"path":{"base":"context","segments":[]},"value":{"kind":"string","value":"public"}}),
                true,
            ),
            ("top", "top", json!({}), true),
            ("validate", "validate", json!({}), false),
            ("commit", "commit", json!({}), true),
        ];
        for (index, (line, operation, arguments, state)) in cases.into_iter().enumerate() {
            let before = fixture.checkpoint_bytes();
            let response = if terminal {
                process.terminal(line)
            } else {
                process.submit(&request(&process, operation, arguments, state))
            };
            if index == 6 {
                check_rejection(&fixture, &before, &response, "SCHEMA_VIOLATION");
                let finding = &response["error"]["validation"]["findings"][0];
                assert_eq!(finding["instance_path"], "/port");
                assert_eq!(finding["schema_path"], "/$defs/port/type");
            } else {
                ok(&response);
            }
            if !state {
                assert_eq!(fixture.checkpoint_bytes(), before);
            }
            if index == 1 {
                assert_eq!(
                    response["result"]["guidance"]["required"],
                    json!(["mode", "name", "port"])
                );
                assert_eq!(
                    response["result"]["guidance"]["children"][2]["types"],
                    json!(["integer"])
                );
            }
            if index == 2 {
                assert_eq!(response["result"]["validation"]["valid"], false);
            }
            if index == 13 {
                assert_eq!(response["result"]["validation"]["valid"], true);
            }
            events.push(normalize_generated_fields(&response, &fixture.directory));
        }
        let expected_instance = document("docs/constraints/acceptance/service.expected.json");
        let saved = fixture.checkpoint_state();
        assert_eq!(saved.candidate, expected_instance);
        assert_eq!(saved.accepted, expected_instance);
        assert_eq!(
            saved.active_constraint.as_ref().unwrap().schema,
            expected_schema
        );
        let before = fixture.checkpoint_bytes();
        let inspection = if terminal {
            process.terminal("schema")
        } else {
            process.submit(&request(&process, "schema", json!({}), false))
        };
        ok(&inspection);
        assert_eq!(
            DocumentValue::parse_document(
                inspection["result"]["schema_json_text"].as_str().unwrap()
            )
            .unwrap(),
            expected_schema
        );
        assert_eq!(fixture.checkpoint_bytes(), before);
        assert!(process.close().success());
        fs::remove_file(fixture.directory.join("schema.json")).unwrap();
        let mut reopened = start(&fixture, false, false);
        assert_eq!(
            reopened.header["session"],
            json!(saved.session_header("durable").unwrap())
        );
        ok(&reopened.submit(&request(&reopened, "validate", json!({}), false)));
        assert_eq!(fixture.checkpoint_bytes(), before);
        assert!(reopened.close().success());
        retain_acceptance_bytes(
            if terminal {
                "constraint-terminal-construction.json"
            } else {
                "constraint-machine-construction.json"
            },
            &serde_json::to_vec_pretty(&construction).unwrap(),
        );
        retain_acceptance_bytes(
            if terminal {
                "constraint-terminal-journey.json"
            } else {
                "constraint-machine-journey.json"
            },
            &serde_json::to_vec_pretty(&events).unwrap(),
        );
        journeys.push(events);
    }
    assert_eq!(journeys[0], journeys[1]);
}

#[test]
fn openapi_description_is_constructed_without_serialized_containers_or_activation() {
    for terminal in [false, true] {
        let fixture = constraint_fixture();
        let (mut process, events) = construct(
            &fixture,
            terminal,
            "docs/constraints/acceptance/openapi.commands",
        );
        let expected = document("docs/constraints/acceptance/openapi.expected.json");
        assert_eq!(fixture.checkpoint_state().candidate, expected);
        assert_eq!(fixture.checkpoint_state().accepted, expected);
        let response = if terminal {
            process.terminal("show /paths/~1services/get/responses/200")
        } else {
            process.submit(&request(&process, "show", json!({"path":{"base":"root","segments":[{"key":"paths"},{"key":"/services"},{"key":"get"},{"key":"responses"},{"key":"200"}]}}), false))
        };
        ok(&response);
        assert!(
            response["result"]["json_text"]
                .as_str()
                .unwrap()
                .contains("Service list")
        );
        assert!(process.close().success());
        retain_acceptance_bytes(
            if terminal {
                "openapi-terminal-journey.json"
            } else {
                "openapi-machine-journey.json"
            },
            &serde_json::to_vec_pretty(&events).unwrap(),
        );
    }
}

#[test]
fn retained_upstream_positive_and_negative_instances_match_the_selected_dialect() {
    let mut positive = 0;
    let mut negative = 0;
    for name in [
        "type",
        "required",
        "minimum",
        "multipleOf",
        "const",
        "properties",
        "additionalProperties",
        "prefixItems",
        "allOf",
        "anyOf",
        "oneOf",
    ] {
        let groups: Value = serde_json::from_slice(
            &fs::read(repository_path(&format!(
                "docs/constraints/acceptance/upstream/{name}.json"
            )))
            .unwrap(),
        )
        .unwrap();
        for group in groups.as_array().unwrap() {
            let constraint = SchemaConstraint::from_schema_document(
                DocumentValue::parse_document(&group["schema"].to_string()).unwrap(),
            )
            .unwrap_or_else(|error| panic!("{name}: {}: {error}", group["description"]));
            for case in group["tests"].as_array().unwrap() {
                let actual = constraint
                    .validate_schema_instance(
                        &DocumentValue::parse_document(&case["data"].to_string()).unwrap(),
                    )
                    .unwrap();
                assert_eq!(
                    actual.valid,
                    case["valid"].as_bool().unwrap(),
                    "{name}: {} / {}",
                    group["description"],
                    case["description"]
                );
                if actual.valid {
                    positive += 1;
                } else {
                    negative += 1;
                }
            }
        }
    }
    assert!(positive > 0 && negative > 0);
    println!("UPSTREAM_CASES positive={positive} negative={negative}");
}

#[test]
fn failed_attachments_preserve_drafts_and_reference_policy_is_explicit() {
    let fixture = constraint_fixture();
    let mut process = start(&fixture, true, true);
    for command in ["schema", "guide", "validate"] {
        let before = fixture.checkpoint_bytes();
        check_rejection(
            &fixture,
            &before,
            &process.terminal(command),
            "NO_ACTIVE_SCHEMA",
        );
    }
    for line in ["set /type string nonsense", "edit /type"] {
        ok(&process.terminal(line));
    }
    let before = fixture.checkpoint_bytes();
    check_rejection(
        &fixture,
        &before,
        &process.terminal("constrain"),
        "INVALID_SCHEMA",
    );
    ok(&process.terminal("set . string object"));
    ok(&process.terminal("constrain"));
    let attached = fixture.checkpoint_state().active_constraint.clone();
    for (schema, code) in [
        (
            r#"{"$schema":"http://json-schema.org/draft-07/schema#"}"#,
            "UNSUPPORTED_SCHEMA",
        ),
        (
            r#"{"$ref":"https://example.invalid/schema"}"#,
            "UNSUPPORTED_SCHEMA",
        ),
        (
            r#"{"$ref":"file:///tmp/schema.json"}"#,
            "UNSUPPORTED_SCHEMA",
        ),
        (r##"{"$ref":"#/$defs/missing"}"##, "INVALID_SCHEMA"),
        (r##"{"$ref":"#"}"##, "UNSUPPORTED_SCHEMA"),
        (
            r##"{"$defs":{"a":{"$ref":"#/$defs/b"},"b":{"$ref":"#/$defs/a"}}}"##,
            "UNSUPPORTED_SCHEMA",
        ),
        (
            r##"{"default":{"type":"string"},"$ref":"#/default"}"##,
            "INVALID_SCHEMA",
        ),
        (r##"{"$dynamicRef":"#node"}"##, "UNSUPPORTED_SCHEMA"),
        (r#"{"type":"object","propertiez":{}}"#, "UNSUPPORTED_SCHEMA"),
        (
            r#"{"type":"string","pattern":"(?<=a)b"}"#,
            "UNSUPPORTED_SCHEMA",
        ),
        (r#"{"type":"string","required":true}"#, "INVALID_SCHEMA"),
    ] {
        fs::write(fixture.directory.join("invalid.json"), schema).unwrap();
        let before = fixture.checkpoint_bytes();
        check_rejection(
            &fixture,
            &before,
            &process.terminal("constrain invalid.json"),
            code,
        );
        assert_eq!(fixture.checkpoint_state().active_constraint, attached);
    }
    let annotation_schema = r#"{"type":"string","format":"email","default":{"$ref":"https://example.invalid","unknown":true},"examples":[{"$id":"data"}]}"#;
    fs::write(fixture.directory.join("annotation.json"), annotation_schema).unwrap();
    ok(&process.terminal("constrain annotation.json"));
    ok(&process.terminal("set \"\" string not-an-email"));
    ok(&process.terminal("commit"));
    assert!(process.close().success());
}

#[test]
fn schema_guidance_and_completion_share_missing_fields_types_literals_and_limits() {
    let fixture = constraint_fixture();
    let mut process = start(&fixture, true, true);
    fs::copy(
        repository_path("docs/constraints/acceptance/service.schema.json"),
        fixture.directory.join("schema.json"),
    )
    .unwrap();
    ok(&process.terminal("constrain schema.json"));
    let before = fixture.checkpoint_bytes();
    let guide = process.terminal("guide");
    let complete = process.terminal("complete");
    assert_eq!(guide["result"]["guidance"], complete["result"]["guidance"]);
    let mut completer = AuthoringCompleter::new_authoring_completer(
        definition(&fixture),
        fixture.checkpoint_state(),
    );
    for (line, expected) in [
        ("set /", vec!["/mode", "/name", "/port", "/tags"]),
        ("set /port ", vec!["number"]),
        ("set /mode string ", vec!["development", "production"]),
    ] {
        let actual: Vec<_> = completer
            .complete(line, line.len())
            .suggestions()
            .iter()
            .map(|suggestion| suggestion.value.clone())
            .collect();
        assert_eq!(actual, expected, "{line}");
    }
    assert_eq!(fixture.checkpoint_bytes(), before);
    let port = process.terminal("guide /port");
    assert_eq!(port["result"]["guidance"]["types"], json!(["integer"]));
    assert!(
        port["result"]["guidance"]["schema_paths"]
            .as_array()
            .unwrap()
            .contains(&json!("/$defs/port"))
    );
    fs::write(fixture.directory.join("branches.json"), r#"{"type":"object","oneOf":[{"required":["a"],"properties":{"a":{"type":"integer"}}},{"required":["b"],"properties":{"b":{"type":"string"}}}]}"#).unwrap();
    ok(&process.terminal("constrain branches.json"));
    let partial = process.terminal("guide");
    assert_eq!(partial["result"]["guidance"]["complete"], false);
    assert!(
        partial["result"]["guidance"]["notes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|note| note.as_str().unwrap().contains("oneOf"))
    );
    assert_eq!(
        process.terminal("validate")["result"]["validation"]["valid"],
        false
    );
    assert!(process.close().success());
}

#[test]
fn constraint_numbers_are_exact_and_resource_failures_do_not_certify_instances() {
    let fixture = constraint_fixture();
    let mut process = start(&fixture, true, true);
    for (schema, number, expected) in [
        (r#"{"const":1e400}"#, "10e399", true),
        (
            r#"{"minimum":18446744073709551616}"#,
            "18446744073709551615",
            false,
        ),
        (
            r#"{"minimum":18446744073709551616}"#,
            "18446744073709551617",
            true,
        ),
        (r#"{"multipleOf":0.1}"#, "0.3", true),
        (r#"{"multipleOf":0.1}"#, "0.31", false),
    ] {
        fs::write(fixture.directory.join("number.json"), schema).unwrap();
        ok(&process.terminal("constrain number.json"));
        ok(&process.terminal(&format!("set \"\" number {number}")));
        let report = process.terminal("validate");
        assert_eq!(report["result"]["validation"]["valid"], expected);
        let before = fixture.checkpoint_bytes();
        let committed = process.terminal("commit");
        if expected {
            ok(&committed);
        } else {
            check_rejection(&fixture, &before, &committed, "SCHEMA_VIOLATION");
        }
        assert_eq!(
            fixture.checkpoint_state().candidate.compact_document_json(),
            number
        );
    }
    ok(&process.terminal("set \"\" number 1e4097"));
    let before = fixture.checkpoint_bytes();
    check_rejection(
        &fixture,
        &before,
        &process.terminal("validate"),
        "LIMIT_EXCEEDED",
    );
    check_rejection(
        &fixture,
        &before,
        &process.terminal("commit"),
        "LIMIT_EXCEEDED",
    );
    fs::write(
        fixture.directory.join("array.json"),
        r#"{"type":"array","items":{"type":"integer"}}"#,
    )
    .unwrap();
    ok(&process.terminal("constrain array.json"));
    ok(&process.terminal("set \"\" array"));
    for _ in 0..102 {
        ok(&process.terminal("append . string bad"));
    }
    let report = process.terminal("validate");
    assert_eq!(report["result"]["validation"]["valid"], false);
    assert_eq!(report["result"]["validation"]["truncated"], true);
    assert_eq!(
        report["result"]["validation"]["findings"]
            .as_array()
            .unwrap()
            .len(),
        100
    );
    assert!(process.close().success());
}

#[test]
fn attachment_replay_detachment_discard_and_checkpoint_profiles_remain_distinct() {
    let fixture = constraint_fixture();
    let mut process = start(&fixture, false, true);
    fs::write(fixture.directory.join("false.json"), "false").unwrap();
    let attach = request(&process, "constrain", json!({"source":"false.json"}), true);
    let response = process.submit(&attach);
    ok(&response);
    let before = fixture.checkpoint_bytes();
    assert_eq!(process.submit(&attach)["replayed"], true);
    assert_eq!(fixture.checkpoint_bytes(), before);
    let mut stale = attach.clone();
    stale["request_id"] = json!("stale");
    check_rejection(
        &fixture,
        &before,
        &process.submit(&stale),
        "REVISION_CONFLICT",
    );
    let discard = request(&process, "discard", json!({}), true);
    ok(&process.submit(&discard));
    assert!(fixture.checkpoint_state().active_constraint.is_some());
    let validate = request(&process, "validate", json!({"view":"accepted"}), false);
    assert_eq!(
        process.submit(&validate)["result"]["validation"]["valid"],
        false
    );
    let detach = request(&process, "unconstrain", json!({}), true);
    ok(&process.submit(&detach));
    assert!(fixture.checkpoint_state().active_constraint.is_none());
    assert_eq!(fixture.checkpoint_state().constraint_mode, "unconstrained");
    ok(&process.submit(&request(
        &process,
        "constrain",
        json!({"source":"false.json"}),
        true,
    )));
    assert!(process.close().success());
    let original = fixture.checkpoint_bytes();
    for field in ["schema_sha256", "policy", "dialect"] {
        let mut malformed: Value = serde_json::from_slice(&original).unwrap();
        malformed["active_constraint"][field] = json!("wrong");
        fs::write(&fixture.checkpoint, serde_json::to_vec(&malformed).unwrap()).unwrap();
        let before = fixture.checkpoint_bytes();
        let mut command = fixture.command(false, false);
        command.arg("--constraints");
        let mut rejected = AuthoringProcess::spawn(command);
        assert_eq!(rejected.header["error"]["code"], "INVALID_SESSION");
        assert!(!rejected.close().success());
        assert_eq!(fixture.checkpoint_bytes(), before);
    }
    fs::write(&fixture.checkpoint, &original).unwrap();
    fixture.reject_open("UNSUPPORTED_FORMAT");
    let mut process = start(&fixture, true, false);
    ok(&process.terminal("unconstrain"));
    ok(&process.terminal("commit"));
    assert!(process.close().success());
}

#[test]
fn constraints_compose_with_mock_interfaces_without_sharing_document_roles() {
    let fixture = constraint_fixture();
    let mut command = fixture.command(true, true);
    command.args(["--constraints", "--compose"]);
    let mut process = AuthoringProcess::spawn(command);
    assert_eq!(process.header["event"], "session_open");
    fs::write(fixture.directory.join("allow.json"), "true").unwrap();
    ok(&process.terminal("constrain allow.json"));
    for line in fs::read_to_string(repository_path(
        "docs/composition/examples/platform/platform.commands",
    ))
    .unwrap()
    .lines()
    {
        ok(&process.terminal(line));
    }
    let state = fixture.checkpoint_state();
    assert_eq!(state.format_version, 4);
    assert!(state.active_constraint.is_some());
    assert!(state.active_interface.is_some());
    let before = fixture.checkpoint_bytes();
    assert!(
        process.terminal("tree")["result"]["verb_tree_text"]
            .as_str()
            .unwrap()
            .contains("invoke scale")
    );
    assert_eq!(fixture.checkpoint_bytes(), before);
    ok(&process.terminal("enter services"));
    ok(&process.terminal("invoke scale 4"));
    assert_eq!(fixture.checkpoint_state().candidate, state.candidate);
    assert_eq!(
        fixture.checkpoint_state().active_constraint,
        state.active_constraint
    );
    assert!(process.close().success());
    let before = fixture.checkpoint_bytes();
    let mut command = fixture.command(true, false);
    command.args(["--constraints", "--compose"]);
    let mut reopened = AuthoringProcess::spawn(command);
    assert_eq!(reopened.header["event"], "session_open");
    assert_eq!(fixture.checkpoint_bytes(), before);
    assert!(reopened.close().success());
}

#[cfg(feature = "fault-injection")]
#[test]
fn schema_attachment_publication_faults_and_lost_response_recover_through_saved_receipts() {
    for (point, action) in [
        ("before_checkpoint_write", "error"),
        ("after_checkpoint_rename", "error"),
        ("before_response_delivery", "pause"),
    ] {
        let fixture = constraint_fixture();
        let mut initial = start(&fixture, false, true);
        assert!(initial.close().success());
        fs::write(
            fixture.directory.join("schema.json"),
            r#"{"type":"object"}"#,
        )
        .unwrap();
        let marker = fixture.directory.join("fault.json");
        let mut command = fixture.command(false, false);
        command
            .arg("--constraints")
            .env("CLI_TEST_FAULT_POINT", point)
            .env("CLI_TEST_FAULT_ACTION", action)
            .env("CLI_TEST_FAULT_MARKER", &marker);
        let mut process = AuthoringProcess::spawn(command);
        assert_eq!(process.header["event"], "session_open");
        let attach = request(&process, "constrain", json!({"source":"schema.json"}), true);
        let before = fixture.checkpoint_bytes();
        process.send_bytes(format!("{attach}\n").as_bytes());
        wait_for_fault(&marker, point, &process);
        if action == "pause" {
            process.kill();
        } else {
            let response = process.receive();
            if point == "before_checkpoint_write" {
                check_rejection(&fixture, &before, &response, "PERSISTENCE_FAILED");
                assert!(process.close().success());
                continue;
            }
            assert_eq!(response["status"], "uncertain");
            assert_eq!(response["mutation"], "unknown");
            assert!(!process.close().success());
        }
        let after = fixture.checkpoint_bytes();
        assert_ne!(before, after);
        assert!(fixture.checkpoint_state().active_constraint.is_some());
        fs::remove_file(fixture.directory.join("schema.json")).unwrap();
        let mut reopened = start(&fixture, false, false);
        assert_eq!(reopened.header["last_receipt"]["request"], attach);
        assert_eq!(reopened.submit(&attach)["replayed"], true);
        assert_eq!(fixture.checkpoint_bytes(), after);
        assert!(reopened.close().success());
    }
}

#[test]
fn schema_reference_siblings_conditionals_and_admission_boundaries_are_observable() {
    let intersected = SchemaConstraint::from_schema_document(
        DocumentValue::parse_document(
            r##"{"$defs":{"choice":{"enum":["a","b"]}},"$ref":"#/$defs/choice","enum":["b","c"]}"##,
        )
        .unwrap(),
    )
    .unwrap();
    let guidance =
        programmable_cli::schema_guidance::describe_schema_path(&intersected, &[]).unwrap();
    assert!(!guidance.complete);
    assert!(
        guidance
            .notes
            .iter()
            .any(|note| note.contains("intersection"))
    );
    for (instance, expected) in [("\"a\"", false), ("\"b\"", true), ("\"c\"", false)] {
        assert_eq!(
            intersected
                .validate_schema_instance(&DocumentValue::parse_document(instance).unwrap())
                .unwrap()
                .valid,
            expected
        );
    }
    for (schema, instance, valid) in [
        (
            r##"{"$defs":{"n":{"type":"integer"}},"$ref":"#/$defs/n","minimum":5}"##,
            "4",
            false,
        ),
        (
            r##"{"$defs":{"n":{"type":"integer"}},"$ref":"#/$defs/n","minimum":5}"##,
            "5",
            true,
        ),
        (
            r#"{"if":{"type":"string"},"then":{"minLength":2},"else":{"const":42}}"#,
            "\"a\"",
            false,
        ),
        (
            r#"{"if":{"type":"string"},"then":{"minLength":2},"else":{"const":42}}"#,
            "42",
            true,
        ),
        (
            r#"{"type":"object","allOf":[{"properties":{"a":{"type":"integer"}}}],"unevaluatedProperties":false}"#,
            "{\"a\":1}",
            true,
        ),
        (
            r#"{"type":"object","allOf":[{"properties":{"a":{"type":"integer"}}}],"unevaluatedProperties":false}"#,
            "{\"b\":1}",
            false,
        ),
        (r#"{"uniqueItems":true}"#, "[1,1.0]", false),
        (
            r##"{"$defs":{"a/b~":{"type":"boolean"}},"$ref":"#/$defs/a~1b~0"}"##,
            "true",
            true,
        ),
        ("false", "null", false),
        ("true", "null", true),
    ] {
        let constraint =
            SchemaConstraint::from_schema_document(DocumentValue::parse_document(schema).unwrap())
                .unwrap();
        assert_eq!(
            constraint
                .validate_schema_instance(&DocumentValue::parse_document(instance).unwrap())
                .unwrap()
                .valid,
            valid,
            "{schema} / {instance}"
        );
    }
    let fixture = constraint_fixture();
    let mut process = start(&fixture, false, true);
    fs::write(fixture.directory.join("allow.json"), "true").unwrap();
    ok(&process.submit(&request(
        &process,
        "constrain",
        json!({"source":"allow.json"}),
        true,
    )));
    let before = fixture.checkpoint_bytes();
    for (path, code) in [
        (
            json!({"base":"root","segments":[{"index":0}]}),
            "TYPE_MISMATCH",
        ),
        (
            json!({"base":"root","segments":[{"key":"absent"},{"key":"nested"}]}),
            "MISSING_PATH",
        ),
    ] {
        let response = process.submit(&request(&process, "guide", json!({"path":path}), false));
        check_rejection(&fixture, &before, &response, code);
    }
    let many = json!({"properties":(0..256).map(|n|(n.to_string(),json!(true))).collect::<serde_json::Map<_,_>>()});
    fs::write(
        fixture.directory.join("large.json"),
        serde_json::to_vec(&many).unwrap(),
    )
    .unwrap();
    check_rejection(
        &fixture,
        &before,
        &process.submit(&request(
            &process,
            "constrain",
            json!({"source":"large.json"}),
            true,
        )),
        "LIMIT_EXCEEDED",
    );
    let mut definitions = serde_json::Map::new();
    definitions.insert("0".into(), json!(true));
    for n in 1..13 {
        definitions.insert(n.to_string(), json!({"allOf":[{"$ref":format!("#/$defs/{}",n-1)},{"$ref":format!("#/$defs/{}",n-1)}]}));
    }
    fs::write(
        fixture.directory.join("expansion.json"),
        serde_json::to_vec(&json!({"$defs":definitions,"$ref":"#/$defs/12"})).unwrap(),
    )
    .unwrap();
    check_rejection(
        &fixture,
        &before,
        &process.submit(&request(
            &process,
            "constrain",
            json!({"source":"expansion.json"}),
            true,
        )),
        "LIMIT_EXCEEDED",
    );
    assert!(process.close().success());
    for profile in [KernelProfile::JsonAuthoring, KernelProfile::CliComposition] {
        let old = constraint_fixture();
        let mut command = old.command(false, true);
        if profile.has_composition() {
            command.arg("--compose");
        }
        let mut process = AuthoringProcess::spawn(command);
        assert_eq!(process.header["event"], "session_open");
        assert!(process.close().success());
        let mut malformed: Value = serde_json::from_slice(&old.checkpoint_bytes()).unwrap();
        malformed["active_constraint"] = Value::Null;
        fs::write(&old.checkpoint, serde_json::to_vec(&malformed).unwrap()).unwrap();
        let before = old.checkpoint_bytes();
        let mut command = old.command(false, false);
        if profile.has_composition() {
            command.arg("--compose");
        }
        let mut process = AuthoringProcess::spawn(command);
        assert_eq!(process.header["error"]["code"], "INVALID_SESSION");
        assert!(!process.close().success());
        assert_eq!(old.checkpoint_bytes(), before);
    }
}
