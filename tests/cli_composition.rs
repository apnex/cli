mod authoring_fixture;
use authoring_fixture::*;
#[cfg(feature = "fault-injection")]
use programmable_cli::authoring_protocol::SessionCheckpoint;
use programmable_cli::document_value::DocumentValue;
use programmable_cli::operation_definition::OperationDefinition;
use programmable_cli::terminal_input::compile_terminal_request;
use serde_json::{Value, json};
use std::fs;

fn composition_fixture() -> AuthoringFixture {
    let fixture = AuthoringFixture::new();
    fs::copy(
        repository_path("docs/composition/acceptance/TASK.md"),
        &fixture.intent,
    )
    .unwrap();
    fixture
}

fn composition_definition(fixture: &AuthoringFixture) -> OperationDefinition {
    OperationDefinition::load_composition_definition(&fixture.declaration).unwrap()
}

fn start_composition(fixture: &AuthoringFixture, terminal: bool, create: bool) -> AuthoringProcess {
    let mut command = fixture.command(terminal, create);
    command.arg("--compose");
    let process = AuthoringProcess::spawn(command);
    assert_eq!(
        process.header["event"], "session_open",
        "{}",
        process.header
    );
    process
}

fn composition_request(process: &AuthoringProcess, operation: &str, arguments: Value) -> Value {
    let mut request = json!({"request_id":uuid::Uuid::new_v4().to_string(),"session_id":process.header["session"]["session_id"],"operation":operation,"arguments":arguments});
    if !["discover", "status", "show", "help", "complete", "diff"].contains(&operation) {
        request["expected_revision"] = process.header["session"]["revision"].clone();
    }
    request
}

fn submit_composition(process: &mut AuthoringProcess, operation: &str, arguments: Value) -> Value {
    let request = composition_request(process, operation, arguments);
    process.submit(&request)
}

fn assert_ok(response: &Value) {
    assert_eq!(response["status"], "ok", "{response}");
}

fn construct_catalog(fixture: &AuthoringFixture, terminal: bool) -> (AuthoringProcess, Vec<Value>) {
    let mut process = start_composition(fixture, terminal, true);
    let definition = composition_definition(fixture);
    let mut events = vec![process.header.clone()];
    let trace = fs::read_to_string(repository_path(
        "docs/composition/acceptance/catalog.commands",
    ))
    .unwrap();
    for (index, line) in trace.lines().enumerate() {
        assert!(
            !line.contains(['{', '}', '[', ']']),
            "Construction must use typed operations"
        );
        let state = fixture.checkpoint_state();
        let request = compile_terminal_request(line, &definition, &state).unwrap();
        let response = if terminal {
            process.terminal(line)
        } else {
            process.submit(&serde_json::to_value(&request).unwrap())
        };
        assert_eq!(
            response["status"], "ok",
            "Construction step {index} {line}: {response}"
        );
        if response["mutation"] == "applied" {
            let receipt = fixture.checkpoint_state().last_receipt.unwrap();
            assert_eq!(receipt.request.operation, request.operation);
            assert_eq!(receipt.request.arguments, request.arguments);
        }
        assert!(
            response["session"]["active_interface"].is_null(),
            "Draft editing activated an interface"
        );
        events.push(response);
    }
    let expected = DocumentValue::parse_document(
        &fs::read_to_string(repository_path(
            "docs/composition/acceptance/expected-definition.json",
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(fixture.checkpoint_state().candidate, expected);
    assert_eq!(fixture.checkpoint_state().accepted, expected);
    assert_eq!(
        DocumentValue::parse_document(
            &fs::read_to_string(fixture.directory.join("catalog-definition.json")).unwrap()
        )
        .unwrap(),
        expected
    );
    (process, events)
}

#[test]
fn constructed_cli_activates_and_runs_through_both_presentations_and_transfers() {
    let mut journeys = Vec::new();
    let mut exports = Vec::new();
    for terminal in [false, true] {
        let fixture = composition_fixture();
        let (mut process, mut events) = construct_catalog(&fixture, terminal);
        let cases = [
            (
                "discover",
                "discover",
                json!({}),
                Some("NO_ACTIVE_INTERFACE"),
            ),
            ("activate", "activate", json!({}), None),
            ("discover", "discover", json!({}), None),
            (
                "invoke about",
                "invoke",
                json!({"command":"about","values":{}}),
                None,
            ),
            (
                "enter services",
                "enter",
                json!({"context":"services"}),
                None,
            ),
            ("discover", "discover", json!({}), None),
            (
                "invoke quota 1e400",
                "invoke",
                json!({"command":"quota","values":{"value":{"kind":"number","value":"1e400"}}}),
                None,
            ),
            (
                "invoke inspect",
                "invoke",
                json!({"command":"inspect","values":{}}),
                None,
            ),
            (
                "invoke restart",
                "invoke",
                json!({"command":"restart","values":{}}),
                Some("UNBOUND_OPERATION"),
            ),
            (
                "enter deployments",
                "enter",
                json!({"context":"deployments"}),
                None,
            ),
            (
                "invoke plan",
                "invoke",
                json!({"command":"plan","values":{}}),
                None,
            ),
            ("enter ..", "enter", json!({"context":".."}), None),
            (
                "enter services",
                "enter",
                json!({"context":"services"}),
                None,
            ),
            (
                "export-interface interface.json",
                "export-interface",
                json!({"destination":"interface.json"}),
                None,
            ),
        ];
        for (index, (line, operation, arguments, error)) in cases.into_iter().enumerate() {
            let before = fixture.checkpoint_bytes();
            let request = composition_request(&process, operation, arguments);
            let response = if terminal {
                process.terminal(line)
            } else {
                process.submit(&request)
            };
            if let Some(code) = error {
                check_rejection(&fixture, &before, &response, code);
            } else {
                assert_ok(&response);
            }
            match index {
                2 => {
                    assert_eq!(
                        response["result"]["children"]
                            .as_object()
                            .unwrap()
                            .keys()
                            .cloned()
                            .collect::<Vec<_>>(),
                        ["deployments", "services"]
                    );
                    assert_eq!(
                        response["result"]["context"]["commands"]["about"]["id"],
                        "catalog.about"
                    );
                }
                5 => {
                    assert_eq!(
                        response["result"]["context"]["related"],
                        json!(["deployments"])
                    );
                    assert_eq!(
                        response["result"]["context"]["commands"]["quota"]["parameters"][0]["type"],
                        "number"
                    );
                    assert_eq!(
                        response["result"]["context"]["commands"]["restart"]["binding"]["kind"],
                        "unbound"
                    );
                }
                6 => assert_eq!(
                    response["result"]["invocation"]["output_json_text"],
                    "1e400"
                ),
                7 => assert_eq!(
                    response["result"]["invocation"]["output_json_text"],
                    r#"{"enabled":true,"name":"api","quota":1e400}"#
                ),
                10 => assert_eq!(
                    response["result"]["invocation"]["output_json_text"],
                    r#"{"status":"planned"}"#
                ),
                _ => {}
            }
            if response["status"] == "ok" && operation == "invoke" {
                assert_eq!(response["result"]["invocation"]["binding"], "simulated");
                assert_eq!(
                    response["result"]["invocation"]["effect"],
                    "simulation_only"
                );
                assert_eq!(response["mutation"], "applied");
            }
            if response["mutation"] == "applied" {
                let receipt = fixture.checkpoint_state().last_receipt.unwrap();
                assert_eq!(
                    serde_json::to_value(receipt.request.arguments).unwrap(),
                    request["arguments"]
                );
            }
            events.push(response);
        }
        let expected_interface = fixture.checkpoint_state().active_interface.unwrap();
        assert!(process.close().success());
        let mut reopened = start_composition(&fixture, false, false);
        assert_eq!(
            reopened.header["session"]["active_interface"]["context"],
            "services"
        );
        let response = submit_composition(&mut reopened, "discover", json!({}));
        assert_eq!(
            response["result"]["last_invocation"]["context"],
            "deployments"
        );
        assert!(reopened.close().success());

        let export = fs::read(fixture.directory.join("interface.json")).unwrap();
        let destination = composition_fixture();
        fs::write(
            &destination.intent,
            "Inspect this imported prototype; no external actions authorized.",
        )
        .unwrap();
        let mut receiver = start_composition(&destination, false, true);
        assert_ok(&submit_composition(
            &mut receiver,
            "activate",
            json!({"source":fixture.directory.join("interface.json")}),
        ));
        assert_eq!(
            destination.checkpoint_state().active_interface.unwrap(),
            expected_interface
        );
        assert_eq!(
            destination
                .checkpoint_state()
                .candidate
                .compact_document_json(),
            "{}"
        );
        assert_eq!(
            destination.checkpoint_state().intent_text,
            "Inspect this imported prototype; no external actions authorized."
        );
        let before = destination.checkpoint_bytes();
        check_rejection(
            &destination,
            &before,
            &submit_composition(
                &mut receiver,
                "invoke",
                json!({"command":"restart","values":{}}),
            ),
            "UNBOUND_OPERATION",
        );
        assert!(receiver.close().success());
        let normalized = normalize_generated_fields(&json!(events), &fixture.directory);
        let name = if terminal { "terminal" } else { "machine" };
        retain_acceptance_bytes(
            &format!("composition-{name}-journey.json"),
            &serde_json::to_vec_pretty(&normalized).unwrap(),
        );
        retain_acceptance_bytes(&format!("composition-{name}-interface.json"), &export);
        retain_acceptance_bytes(
            "authored-catalog-definition.json",
            &fs::read(fixture.directory.join("catalog-definition.json")).unwrap(),
        );
        journeys.push(normalized);
        exports.push(export);
    }
    assert_eq!(
        journeys[0], journeys[1],
        "Terminal and machine composition diverged"
    );
    assert_eq!(
        exports[0], exports[1],
        "Portable output depends on presentation"
    );
    report_pass("cli-003-construction-parity-transfer");
}

#[test]
fn definition_edits_and_failed_activation_preserve_the_prior_active_interface() {
    let fixture = composition_fixture();
    let (mut process, _) = construct_catalog(&fixture, true);
    assert_ok(&process.terminal("activate"));
    assert_ok(&process.terminal("enter services"));
    assert_ok(&process.terminal("invoke quota 1e400"));
    let active = fixture.checkpoint_state().active_interface.unwrap();
    assert_ok(&process.terminal("set /contexts/services/parent string missing"));
    assert_eq!(
        fixture.checkpoint_state().active_interface.as_ref(),
        Some(&active)
    );
    let before = fixture.checkpoint_bytes();
    check_rejection(
        &fixture,
        &before,
        &process.terminal("activate"),
        "INVALID_CLI_DEFINITION",
    );
    assert_eq!(
        fixture.checkpoint_state().active_interface.as_ref(),
        Some(&active)
    );
    assert_ok(&process.terminal("discard"));
    assert_ok(&process.terminal("activate"));
    assert_eq!(
        fixture.checkpoint_state().active_interface.as_ref(),
        Some(&active),
        "Identical activation reset simulation"
    );
    assert_ok(&process.terminal("set /description string \"Revised prototype\""));
    assert_eq!(
        fixture.checkpoint_state().active_interface.as_ref(),
        Some(&active)
    );
    assert_ok(&process.terminal("activate"));
    let replacement = fixture.checkpoint_state().active_interface.unwrap();
    assert_ne!(replacement.definition_sha256, active.definition_sha256);
    assert_eq!(replacement.context, "root");
    assert_eq!(
        replacement.mock_state.compact_document_json(),
        r#"{"enabled":true,"name":"api","quota":1.2300}"#
    );
    assert!(replacement.last_invocation.is_none());
    assert!(process.close().success());
    report_pass("cli-003-explicit-replacement");
}

#[test]
fn invalid_definitions_and_mock_failures_never_publish_partial_state() {
    let fixture = composition_fixture();
    let (mut process, _) = construct_catalog(&fixture, false);
    assert_ok(&submit_composition(&mut process, "activate", json!({})));
    assert_ok(&submit_composition(
        &mut process,
        "enter",
        json!({"context":"services"}),
    ));
    let original: Value = serde_json::from_slice(
        &fs::read(repository_path(
            "docs/composition/acceptance/expected-definition.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let invalid_cases = [
        ("/format", json!("cli-definition-v99")),
        ("/contexts/services/parent", json!("services")),
        ("/contexts/root/parent", json!("services")),
        ("/contexts/services/related", json!(["missing"])),
        (
            "/contexts/services/commands/restart/id",
            json!("catalog.quota"),
        ),
        (
            "/contexts/services/commands/restart/binding",
            json!({"kind":"connected","executable":"touch never-created"}),
        ),
        (
            "/contexts/services/commands/quota/parameters/0/type",
            json!("integer"),
        ),
        (
            "/contexts/services/commands/quota/binding/steps/0/value/name",
            json!("missing"),
        ),
        (
            "/contexts/services/commands/quota/binding/steps/0/path",
            json!([{"key":"quota","index":0}]),
        ),
    ];
    for (pointer, value) in invalid_cases {
        let mut invalid = original.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert_ne!(
            invalid, original,
            "INVALID: malformed definition did not land"
        );
        fs::write(
            fixture.directory.join("invalid.json"),
            serde_json::to_vec(&invalid).unwrap(),
        )
        .unwrap();
        let before = fixture.checkpoint_bytes();
        check_rejection(
            &fixture,
            &before,
            &submit_composition(&mut process, "activate", json!({"source":"invalid.json"})),
            "INVALID_CLI_DEFINITION",
        );
    }
    for values in [
        json!({"value":{"kind":"string","value":"1e400"}}),
        json!({}),
        json!({"value":{"kind":"number","value":"3"},"extra":{"kind":"boolean","value":true}}),
        json!({"value":3}),
    ] {
        let before = fixture.checkpoint_bytes();
        check_rejection(
            &fixture,
            &before,
            &submit_composition(
                &mut process,
                "invoke",
                json!({"command":"quota","values":values}),
            ),
            "INVALID_CLI_ARGUMENTS",
        );
    }
    let mut failure = original.clone();
    failure["contexts"]["services"]["commands"]["quota"]["binding"]["steps"].as_array_mut().unwrap().push(json!({"path":[{"key":"missing"},{"key":"child"}],"value":{"source":"literal","value":false}}));
    fs::write(
        fixture.directory.join("failure.json"),
        serde_json::to_vec(&failure).unwrap(),
    )
    .unwrap();
    assert_ok(&submit_composition(
        &mut process,
        "activate",
        json!({"source":"failure.json"}),
    ));
    assert_ok(&submit_composition(
        &mut process,
        "enter",
        json!({"context":"services"}),
    ));
    let before = fixture.checkpoint_bytes();
    let response = submit_composition(
        &mut process,
        "invoke",
        json!({"command":"quota","values":{"value":{"kind":"number","value":"99"}}}),
    );
    assert_eq!(response["status"], "error");
    assert_eq!(response["error"]["failed_operation_index"], 1);
    assert_eq!(
        fixture.checkpoint_bytes(),
        before,
        "A failed second step published the first assignment"
    );
    assert!(!fixture.directory.join("never-created").exists());
    assert!(process.close().success());
    report_pass("cli-003-invalid-definition-and-atomic-mock");
}

#[test]
fn configured_names_drive_discovery_completion_and_invocation() {
    use programmable_cli::terminal_completion::AuthoringCompleter;
    use reedline::Completer;
    let fixture = composition_fixture();
    let (mut process, _) = construct_catalog(&fixture, true);
    assert_ok(
        &process.terminal("set /contexts/services/commands/quota/parameters/0/name string amount"),
    );
    assert_ok(&process.terminal(
        "set /contexts/services/commands/quota/binding/steps/0/value/name string amount",
    ));
    assert_ok(&process.terminal("activate"));
    assert_ok(&process.terminal("enter services"));
    let definition = composition_definition(&fixture);
    let state = fixture.checkpoint_state();
    let mut completer =
        AuthoringCompleter::new_authoring_completer(definition.clone(), state.clone());
    let values: Vec<_> = completer
        .complete("invoke q", 8)
        .suggestions()
        .iter()
        .map(|entry| entry.value.clone())
        .collect();
    assert_eq!(values, ["quota"]);
    let request = compile_terminal_request("invoke quota 1.2300", &definition, &state).unwrap();
    assert_eq!(
        serde_json::to_string(&request.arguments["values"]).unwrap(),
        r#"{"amount":{"kind":"number","value":"1.2300"}}"#
    );
    let response = process.terminal("invoke quota 1.2300");
    assert_ok(&response);
    assert_eq!(
        response["result"]["invocation"]["output_json_text"],
        "1.2300"
    );
    let context_choices: Vec<_> = completer
        .complete("enter d", 7)
        .suggestions()
        .iter()
        .map(|entry| entry.value.clone())
        .collect();
    assert_eq!(context_choices, ["deployments"]);
    assert_ok(&process.terminal("enter deployments"));
    completer.refresh_completion_context(&fixture.checkpoint_state());
    assert!(completer.complete("invoke q", 8).suggestions().is_empty());
    assert!(process.close().success());
    report_pass("cli-003-configured-signature-discovery");
}

#[test]
fn runtime_simulation_labels_survive_authored_claims_and_corrupt_transfer_is_rejected() {
    let fixture = composition_fixture();
    let (mut process, _) = construct_catalog(&fixture, true);
    for line in [
        "set /contexts/root/commands/about/binding/output/value object",
        "set /contexts/root/commands/about/binding/output/value/effect string real",
        "activate",
        "invoke about",
        "export-interface labeled.json",
    ] {
        assert_ok(&process.terminal(line));
    }
    let active = fixture.checkpoint_state().active_interface.unwrap();
    let outcome = active.last_invocation.unwrap();
    assert_eq!(outcome.output_json_text, r#"{"effect":"real"}"#);
    assert_eq!(outcome.effect, "simulation_only");
    let export: Value =
        serde_json::from_slice(&fs::read(fixture.directory.join("labeled.json")).unwrap()).unwrap();
    for (pointer, value) in [
        ("/interface/last_invocation/effect", json!("real")),
        ("/interface/last_invocation/binding", json!("connected")),
        ("/interface/definition_sha256", json!("wrong")),
        ("/interface/context", json!("missing")),
        (
            "/interface/last_invocation/operation_id",
            json!("catalog.restart"),
        ),
    ] {
        let mut corrupt = export.clone();
        *corrupt.pointer_mut(pointer).unwrap() = value;
        assert_ne!(corrupt, export, "INVALID: export corruption did not land");
        fs::write(
            fixture.directory.join("corrupt.json"),
            serde_json::to_vec(&corrupt).unwrap(),
        )
        .unwrap();
        let before = fixture.checkpoint_bytes();
        check_rejection(
            &fixture,
            &before,
            &process.terminal("activate corrupt.json"),
            "INVALID_CLI_DEFINITION",
        );
    }
    assert!(process.close().success());
    report_pass("cli-003-simulation-provenance");
}

#[test]
fn composition_checkpoint_corruption_and_wrong_profile_are_rejected() {
    let fixture = composition_fixture();
    let (mut process, _) = construct_catalog(&fixture, true);
    for line in ["activate", "enter services", "invoke quota 9"] {
        assert_ok(&process.terminal(line));
    }
    assert!(process.close().success());
    let original: Value = serde_json::from_slice(&fixture.checkpoint_bytes()).unwrap();
    for (pointer, value) in [
        ("/active_interface/definition_sha256", json!("wrong")),
        ("/active_interface/last_invocation/effect", json!("real")),
        (
            "/last_receipt/response/session/active_interface/context",
            json!("root"),
        ),
        ("/last_receipt/request/arguments/values/value", json!("9")),
    ] {
        let mut corrupt = original.clone();
        *corrupt.pointer_mut(pointer).unwrap() = value;
        assert_ne!(
            corrupt, original,
            "INVALID: checkpoint corruption did not land"
        );
        fs::write(&fixture.checkpoint, serde_json::to_vec(&corrupt).unwrap()).unwrap();
        let before = fixture.checkpoint_bytes();
        let mut command = fixture.command(false, false);
        command.arg("--compose");
        let mut reopened = AuthoringProcess::spawn(command);
        assert_eq!(
            reopened.header["error"]["code"], "INVALID_SESSION",
            "{}",
            reopened.header
        );
        assert!(!reopened.close().success());
        assert_eq!(fixture.checkpoint_bytes(), before);
    }
    fs::write(&fixture.checkpoint, serde_json::to_vec(&original).unwrap()).unwrap();
    fixture.reject_open("UNSUPPORTED_FORMAT");
    let mut reopened = start_composition(&fixture, false, false);
    assert!(reopened.close().success());
    report_pass("cli-003-checkpoint-validation");
}

#[test]
fn composition_registry_checks_actual_handler_and_argument_drift() {
    use programmable_cli::cli_composition::registered_composition_handlers;
    let mut manifest: Value = serde_json::from_slice(
        &fs::read(repository_path("docs/authoring/operations.json")).unwrap(),
    )
    .unwrap();
    let extension: Value = serde_json::from_slice(
        &fs::read(repository_path("docs/composition/operations.json")).unwrap(),
    )
    .unwrap();
    manifest["definition_id"] = json!("bootstrap-cli-composition-v1");
    manifest["types"]
        .as_object_mut()
        .unwrap()
        .extend(extension["types"].as_object().unwrap().clone());
    manifest["operations"]
        .as_array_mut()
        .unwrap()
        .extend(extension["operations"].as_array().unwrap().clone());
    let bytes = serde_json::to_vec(&manifest).unwrap();
    assert!(
        OperationDefinition::from_definition_bytes(&bytes, registered_composition_handlers())
            .is_ok()
    );
    for mutation in ["missing", "extra", "argument"] {
        let mut handlers = registered_composition_handlers();
        match mutation {
            "missing" => {
                assert!(handlers.remove("composition.invoke").is_some());
            }
            "extra" => {
                assert!(
                    handlers
                        .insert(
                            "composition.invented".into(),
                            handlers["composition.invoke"].clone()
                        )
                        .is_none()
                );
            }
            _ => {
                handlers.get_mut("composition.invoke").unwrap().parameters[0].name = "wrong";
            }
        }
        let result = OperationDefinition::from_definition_bytes(&bytes, handlers);
        assert!(result.is_err(), "Live registry drift accepted: {mutation}");
    }
    let invoke = manifest["operations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|op| op["name"] == "invoke")
        .unwrap();
    invoke["terminal"]["positional"] = json!(["values", "command"]);
    assert!(
        OperationDefinition::from_definition_bytes(
            &serde_json::to_vec(&manifest).unwrap(),
            registered_composition_handlers()
        )
        .is_err()
    );
    report_pass("cli-003-live-registry-drift");
}

#[test]
fn exact_numbers_survive_arguments_literals_discovery_and_reopen() {
    let fixture = composition_fixture();
    let (mut process, _) = construct_catalog(&fixture, false);
    assert_ok(&submit_composition(&mut process, "activate", json!({})));
    assert_ok(&submit_composition(
        &mut process,
        "enter",
        json!({"context":"services"}),
    ));
    for token in ["1e400", "1E-400", "-0", "1.2300", "9007199254740993"] {
        let response = submit_composition(
            &mut process,
            "invoke",
            json!({"command":"quota","values":{"value":{"kind":"number","value":token}}}),
        );
        assert_ok(&response);
        assert_eq!(response["result"]["invocation"]["output_json_text"], token);
        assert!(
            fixture
                .checkpoint_state()
                .active_interface
                .unwrap()
                .mock_state
                .compact_document_json()
                .contains(token)
        );
    }
    assert!(process.close().success());
    let mut process = start_composition(&fixture, true, false);
    for token in ["1e400", "1E-400", "-0", "1.2300", "9007199254740993"] {
        assert_eq!(
            process.terminal(&format!("invoke quota {token}"))["result"]["invocation"]["output_json_text"],
            token
        );
    }
    for line in [
        "set /contexts/root/commands/about/binding/output/value number 1e400",
        "activate",
        "invoke about",
    ] {
        assert_ok(&process.terminal(line));
    }
    let discovered = process.terminal("discover");
    let program =
        discovered["result"]["context"]["commands"]["about"]["binding"]["program_json_text"]
            .as_str()
            .unwrap();
    assert!(program.contains("1e400"));
    assert!(!program.contains("1e+400"));
    assert_ok(&process.terminal("export-interface exact.json"));
    assert!(
        String::from_utf8(fs::read(fixture.directory.join("exact.json")).unwrap())
            .unwrap()
            .contains("1e400")
    );
    assert!(process.close().success());
    let mut reopened = start_composition(&fixture, true, false);
    assert_eq!(
        reopened.terminal("invoke about")["result"]["invocation"]["output_json_text"],
        "1e400"
    );
    assert!(reopened.close().success());
    report_pass("cli-003-exact-number-boundaries");
}

#[test]
fn definition_limits_unknown_fields_and_duplicate_keys_reject_before_activation() {
    let fixture = composition_fixture();
    let (mut process, _) = construct_catalog(&fixture, false);
    assert_ok(&submit_composition(&mut process, "activate", json!({})));
    let original: Value = serde_json::from_slice(
        &fs::read(repository_path(
            "docs/composition/acceptance/expected-definition.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let mut cases = Vec::new();
    let mut missing_parent = original.clone();
    missing_parent["contexts"]["root"]
        .as_object_mut()
        .unwrap()
        .remove("parent");
    cases.push((missing_parent, "INVALID_CLI_DEFINITION"));
    let mut unknown = original.clone();
    unknown["contexts"]["root"]["commands"]["about"]["binding"]["extra"] = json!(true);
    cases.push((unknown, "INVALID_CLI_DEFINITION"));
    let mut reserved = original.clone();
    reserved["contexts"][".."] = original["contexts"]["services"].clone();
    cases.push((reserved, "INVALID_CLI_DEFINITION"));
    for (dimension, count) in [
        ("contexts", 129),
        ("commands", 1025),
        ("parameters", 65),
        ("steps", 257),
    ] {
        let mut too_many = original.clone();
        match dimension {
            "contexts" => {
                let contexts = too_many["contexts"].as_object_mut().unwrap();
                while contexts.len() < count {
                    contexts.insert(format!("context{}",contexts.len()),json!({"parent":"root","help":"Example context","related":[],"commands":{}}));
                }
            }
            "commands" => {
                too_many["contexts"] =
                    json!({"root":{"parent":null,"help":"Root","related":[],"commands":{}}});
                for index in 0..count {
                    too_many["contexts"]["root"]["commands"][format!("verb{index}")] = json!({"id":format!("operation{index}"),"help":"Unbound command","parameters":[],"binding":{"kind":"unbound","reason":"Unimplemented"}});
                }
            }
            "parameters" => {
                too_many["contexts"]["services"]["commands"]["quota"]["parameters"] = json!(
                (0..count)
                    .map(
                        |index| json!({"name":format!("arg{index}"),"type":"number","help":"Value"})
                    )
                    .collect::<Vec<_>>()
            )
            }
            _ => {
                too_many["contexts"]["services"]["commands"]["quota"]["binding"]["steps"] =
                    json!(vec![
                        json!({"path":[],"value":{"source":"literal","value":null}});
                        count
                    ])
            }
        }
        cases.push((too_many, "LIMIT_EXCEEDED"));
    }
    for (document, code) in cases {
        assert_ne!(
            document, original,
            "INVALID: definition mutation did not land"
        );
        fs::write(
            fixture.directory.join("bad.json"),
            serde_json::to_vec(&document).unwrap(),
        )
        .unwrap();
        let before = fixture.checkpoint_bytes();
        check_rejection(
            &fixture,
            &before,
            &submit_composition(&mut process, "activate", json!({"source":"bad.json"})),
            code,
        );
    }
    let text = serde_json::to_string(&original).unwrap();
    assert!(text.contains("\"kind\":\"simulated\""));
    fs::write(
        fixture.directory.join("duplicate.json"),
        text.replacen(
            "\"kind\":\"simulated\"",
            "\"kind\":\"simulated\",\"kind\":\"unbound\"",
            1,
        ),
    )
    .unwrap();
    let before = fixture.checkpoint_bytes();
    check_rejection(
        &fixture,
        &before,
        &submit_composition(&mut process, "activate", json!({"source":"duplicate.json"})),
        "INVALID_CLI_DEFINITION",
    );
    assert!(process.close().success());
    report_pass("cli-003-definition-input-boundaries");
}

#[test]
fn domain_word_collisions_and_scalar_types_are_configured_without_new_handlers() {
    use programmable_cli::terminal_completion::AuthoringCompleter;
    use reedline::Completer;
    let fixture = composition_fixture();
    let (mut process, _) = construct_catalog(&fixture, true);
    let mut definition: Value = serde_json::from_slice(
        &fs::read(fixture.directory.join("catalog-definition.json")).unwrap(),
    )
    .unwrap();
    definition["contexts"]["services"]["commands"]["set"] = json!({"id":"catalog.set","help":"Set a simulated name and enabled state","parameters":[{"name":"name","type":"string","help":"Name"},{"name":"enabled","type":"boolean","help":"Enabled"}],"binding":{"kind":"simulated","steps":[{"path":[{"key":"name"}],"value":{"source":"argument","name":"name"}},{"path":[{"key":"enabled"}],"value":{"source":"argument","name":"enabled"}}],"output":{"source":"state","path":[]}}});
    fs::write(
        fixture.directory.join("changed.json"),
        serde_json::to_vec(&definition).unwrap(),
    )
    .unwrap();
    for line in ["activate changed.json", "enter services"] {
        assert_ok(&process.terminal(line));
    }
    let mut completer = AuthoringCompleter::new_authoring_completer(
        composition_definition(&fixture),
        fixture.checkpoint_state(),
    );
    assert_eq!(
        completer.complete("invoke se", 9).suggestions()[0].value,
        "set"
    );
    let choices = completer.complete("invoke set name ", 16);
    assert_eq!(
        choices
            .suggestions()
            .iter()
            .map(|entry| entry.value.as_str())
            .collect::<Vec<_>>(),
        ["true", "false"]
    );
    let response = process.terminal("invoke set \"true name\" false");
    assert_ok(&response);
    assert_eq!(
        response["result"]["invocation"]["output_json_text"],
        r#"{"enabled":false,"name":"true name","quota":1.2300}"#
    );
    assert!(process.close().success());
    let mut machine = start_composition(&fixture, false, false);
    let response = submit_composition(
        &mut machine,
        "invoke",
        json!({"command":"set","values":{"name":{"kind":"string","value":"true name"},"enabled":{"kind":"boolean","value":false}}}),
    );
    assert_ok(&response);
    assert_eq!(
        response["result"]["invocation"]["output_json_text"],
        r#"{"enabled":false,"name":"true name","quota":1.2300}"#
    );
    assert!(machine.close().success());
    report_pass("cli-003-configured-domain-words-and-types");
}

#[cfg(feature = "fault-injection")]
#[test]
fn activation_publication_failures_and_uncertainty_use_the_existing_checkpoint_boundary() {
    for point in ["before_checkpoint_write", "after_checkpoint_rename"] {
        let fixture = composition_fixture();
        let (mut initial, _) = construct_catalog(&fixture, false);
        assert!(initial.close().success());
        let before = fixture.checkpoint_bytes();
        let marker = fixture.directory.join("activation-fault.json");
        let mut command = fixture.command(false, false);
        command
            .arg("--compose")
            .env("CLI_TEST_FAULT_POINT", point)
            .env("CLI_TEST_FAULT_ACTION", "error")
            .env("CLI_TEST_FAULT_MARKER", &marker);
        let mut process = AuthoringProcess::spawn(command);
        let request = composition_request(&process, "activate", json!({}));
        let response = process.submit(&request);
        wait_for_fault(&marker, point, &process);
        if point == "before_checkpoint_write" {
            check_rejection(&fixture, &before, &response, "PERSISTENCE_FAILED");
            assert!(process.close().success());
        } else {
            assert_eq!(response["status"], "uncertain");
            assert_eq!(response["mutation"], "unknown");
            assert!(response["session"]["active_interface"].is_null());
            assert_eq!(
                submit_composition(&mut process, "activate", json!({}))["error"]["code"],
                "PERSISTENCE_UNCERTAIN"
            );
            assert!(!process.close().success());
            let after = fixture.checkpoint_bytes();
            assert_ne!(after, before);
            let mut reopened = start_composition(&fixture, false, false);
            let replay = reopened.submit(&request);
            assert_ok(&replay);
            assert_eq!(replay["replayed"], true);
            assert_eq!(fixture.checkpoint_bytes(), after);
            assert!(reopened.close().success());
        }
    }
    report_pass("cli-003-activation-publication-faults");
}

#[cfg(feature = "fault-injection")]
#[test]
fn lost_mock_response_replays_after_process_death_without_another_transition() {
    let fixture = composition_fixture();
    let (mut initial, _) = construct_catalog(&fixture, true);
    assert_ok(&initial.terminal("activate"));
    assert_ok(&initial.terminal("enter services"));
    assert!(initial.close().success());
    let marker = fixture.directory.join("invocation-fault.json");
    let mut command = fixture.command(false, false);
    command
        .arg("--compose")
        .env("CLI_TEST_FAULT_POINT", "before_response_delivery")
        .env("CLI_TEST_FAULT_ACTION", "pause")
        .env("CLI_TEST_FAULT_MARKER", &marker);
    let mut process = AuthoringProcess::spawn(command);
    let request = composition_request(
        &process,
        "invoke",
        json!({"command":"quota","values":{"value":{"kind":"number","value":"1e400"}}}),
    );
    process.send_bytes(format!("{request}\n").as_bytes());
    wait_for_fault(&marker, "before_response_delivery", &process);
    process.kill();
    let after = fixture.checkpoint_bytes();
    let state: SessionCheckpoint = serde_json::from_slice(&after).unwrap();
    assert_eq!(
        state
            .active_interface
            .unwrap()
            .last_invocation
            .unwrap()
            .output_json_text,
        "1e400"
    );
    let mut reopened = start_composition(&fixture, false, false);
    assert_eq!(reopened.header["last_receipt"]["request"], request);
    let replay = reopened.submit(&request);
    assert_ok(&replay);
    assert_eq!(replay["replayed"], true);
    assert_eq!(fixture.checkpoint_bytes(), after);
    assert!(reopened.close().success());
    report_pass("cli-003-lost-mock-response");
}
