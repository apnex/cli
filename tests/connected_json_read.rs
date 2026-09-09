//! Real file observations are measured separately from mock state and receiving-process authority.

mod authoring_fixture;
use authoring_fixture::*;
use programmable_cli::document_value::DocumentValue;
use programmable_cli::document_value::{MAX_DOCUMENT_BYTES, MAX_SCALAR_BYTES};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

const FIRST_CATALOG: &[u8] = b"{\"name\":\"api\",\"quota\":1.2300,\"enabled\":true}\n";
const LATER_CATALOG: &[u8] = b"{\"name\":\"worker\",\"quota\":1e400,\"enabled\":false}\n";

fn connected_start(
    fixture: &AuthoringFixture,
    terminal: bool,
    create: bool,
    granted: bool,
) -> AuthoringProcess {
    let mut command = fixture.command(terminal, create);
    command.args(["--compose", "--constraints"]);
    if granted {
        command.args(["--grant-json-read", "services.catalog", "catalog.json"]);
    }
    let process = AuthoringProcess::spawn(command);
    assert_eq!(
        process.header["event"], "session_open",
        "{}",
        process.header
    );
    process
}

fn connected_request(process: &AuthoringProcess, operation: &str, arguments: Value) -> Value {
    let mut request = json!({"request_id":uuid::Uuid::new_v4().to_string(),"session_id":process.header["session"]["session_id"],"operation":operation,"arguments":arguments});
    if !["tree", "discover", "show", "diff", "status", "help"].contains(&operation) {
        request["expected_revision"] = process.header["session"]["revision"].clone();
    }
    request
}

fn connected_step(
    process: &mut AuthoringProcess,
    terminal: bool,
    line: &str,
    operation: &str,
    arguments: Value,
) -> Value {
    if terminal {
        process.terminal(line)
    } else {
        process.submit(&connected_request(process, operation, arguments))
    }
}

fn connected_ok(
    process: &mut AuthoringProcess,
    terminal: bool,
    line: &str,
    operation: &str,
    arguments: Value,
) -> Value {
    let response = connected_step(process, terminal, line, operation, arguments);
    assert_eq!(response["status"], "ok", "{line}: {response}");
    response
}

fn prepare_connected_definition(fixture: &AuthoringFixture) {
    fs::copy(
        repository_path("docs/connected/acceptance/TASK.md"),
        &fixture.intent,
    )
    .unwrap();
    fs::copy(
        repository_path("docs/connected/acceptance/expected-definition.json"),
        fixture.directory.join("spec.json"),
    )
    .unwrap();
    fs::write(fixture.directory.join("catalog.json"), FIRST_CATALOG).unwrap();
}

fn activate_connected_definition(process: &mut AuthoringProcess, terminal: bool) {
    for (line, operation, arguments) in [
        ("import spec.json", "import", json!({"source":"spec.json"})),
        ("commit", "commit", json!({})),
        ("activate", "activate", json!({})),
        ("enter services", "enter", json!({"context":"services"})),
    ] {
        connected_ok(process, terminal, line, operation, arguments);
    }
}

fn assert_catalog_observation(response: &Value, source: &[u8]) {
    let outcome = &response["result"]["invocation"];
    assert_eq!(outcome["binding"], "connected");
    assert_eq!(outcome["effect"], "external_read");
    assert_eq!(outcome["simulated_steps"], 0);
    let expected = DocumentValue::parse_document(std::str::from_utf8(source).unwrap())
        .unwrap()
        .compact_document_json();
    assert_eq!(outcome["output_json_text"], expected);
    assert_eq!(
        outcome["observation"],
        json!({"provider":"json-file-read-v1","capability":"services.catalog","source_bytes":source.len(),"source_sha256":format!("{:x}",Sha256::digest(source)),"output_sha256":format!("{:x}",Sha256::digest(expected.as_bytes()))})
    );
}

#[test]
fn connected_definition_is_authored_with_typed_commands_and_activates_without_authority() {
    let fixture = AuthoringFixture::new();
    let mut command = fixture.command(true, true);
    command.arg("--compose");
    let mut process = AuthoringProcess::spawn(command);
    assert_eq!(
        process.header["event"], "session_open",
        "{}",
        process.header
    );
    let recipe = fs::read_to_string(repository_path(
        "docs/connected/acceptance/catalog.commands",
    ))
    .unwrap();
    let mut events = vec![process.header.clone()];
    for line in recipe.lines() {
        assert!(
            !line.contains(['{', '}', '[', ']']),
            "Raw JSON container in recipe: {line}"
        );
        let response = process.terminal(line);
        assert_eq!(response["status"], "ok", "{line}: {response}");
        events.push(response);
    }
    let expected = DocumentValue::parse_document(
        &fs::read_to_string(repository_path(
            "docs/connected/acceptance/expected-definition.json",
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(fixture.checkpoint_state().candidate, expected);
    assert_eq!(fixture.checkpoint_state().accepted, expected);
    let activation = process.terminal("activate");
    assert_eq!(activation["status"], "ok", "{activation}");
    events.push(activation);
    let tree = process.terminal("tree");
    assert!(
        tree["result"]["verb_tree_text"]
            .as_str()
            .unwrap()
            .contains("invoke inspect [connected:json-file-read-v1]")
    );
    assert_eq!(process.terminal("enter services")["status"], "ok");
    let discovery = process.terminal("discover");
    assert_eq!(
        discovery["result"]["context"]["commands"]["inspect"]["binding"]["granted"],
        false
    );
    let before = fixture.checkpoint_bytes();
    check_rejection(
        &fixture,
        &before,
        &process.terminal("invoke inspect"),
        "CAPABILITY_NOT_GRANTED",
    );
    retain_acceptance_bytes(
        "connected-author.json",
        &serde_json::to_vec_pretty(&events).unwrap(),
    );
    retain_acceptance_bytes(
        "connected-authored-cli.json",
        &fs::read(fixture.directory.join("connected-cli.json")).unwrap(),
    );
    assert!(process.close().success());
}

#[test]
fn connected_reads_observe_real_files_in_both_presentations_without_mutating_authored_or_mock_state()
 {
    let mut observations = Vec::new();
    for terminal in [false, true] {
        let fixture = AuthoringFixture::new();
        prepare_connected_definition(&fixture);
        fs::write(
            fixture.directory.join("policy.json"),
            b"{\"type\":\"object\"}",
        )
        .unwrap();
        let mut process = connected_start(&fixture, terminal, true, true);
        activate_connected_definition(&mut process, terminal);
        connected_ok(
            &mut process,
            terminal,
            "constrain policy.json",
            "constrain",
            json!({"source":"policy.json"}),
        );
        connected_ok(
            &mut process,
            terminal,
            "edit /contexts/services",
            "edit",
            json!({"path":{"base":"root","segments":[{"key":"contexts"},{"key":"services"}]}}),
        );
        let mut events = Vec::new();
        for source in [
            FIRST_CATALOG,
            LATER_CATALOG,
            b"{\"binding\":\"simulated\",\"effect\":\"success\",\"observation\":null}".as_slice(),
        ] {
            fs::write(fixture.directory.join("replacement.json"), source).unwrap();
            fs::rename(
                fixture.directory.join("replacement.json"),
                fixture.directory.join("catalog.json"),
            )
            .unwrap();
            let before = fixture.checkpoint_state();
            let response = connected_ok(
                &mut process,
                terminal,
                "invoke inspect",
                "invoke",
                json!({"command":"inspect","values":{}}),
            );
            assert_catalog_observation(&response, source);
            let mut after = fixture.checkpoint_state();
            assert_eq!(after.revision.0, before.revision.0 + 1);
            assert_eq!(
                after.last_receipt.as_ref().unwrap().response.result,
                Some(response["result"].clone())
            );
            after.revision = before.revision;
            after.last_receipt = before.last_receipt.clone();
            after.active_interface.as_mut().unwrap().last_invocation = before
                .active_interface
                .as_ref()
                .unwrap()
                .last_invocation
                .clone();
            assert_eq!(
                after, before,
                "A connected observation changed unrelated session state"
            );
            assert_eq!(
                fs::read(fixture.directory.join("catalog.json")).unwrap(),
                source
            );
            events.push(response);
        }
        let discovery = connected_ok(&mut process, terminal, "discover", "discover", json!({}));
        assert_eq!(
            discovery["result"]["context"]["commands"]["inspect"]["binding"]["granted"],
            true
        );
        assert_eq!(
            discovery["result"]["interface"]["binding_scope"],
            "includes_connected_read"
        );
        let target_before = fs::read(fixture.directory.join("catalog.json")).unwrap();
        let simulated = connected_ok(
            &mut process,
            terminal,
            "invoke quota 1e400",
            "invoke",
            json!({"command":"quota","values":{"value":{"kind":"number","value":"1e400"}}}),
        );
        assert_eq!(
            simulated["result"]["invocation"]["effect"],
            "simulation_only"
        );
        assert!(
            simulated["result"]["invocation"]
                .get("observation")
                .is_none()
        );
        assert_eq!(
            simulated["result"]["invocation"]["output_json_text"],
            r#"{"quota":1e400}"#
        );
        assert_eq!(
            fs::read(fixture.directory.join("catalog.json")).unwrap(),
            target_before
        );
        let before = fixture.checkpoint_bytes();
        check_rejection(
            &fixture,
            &before,
            &connected_step(
                &mut process,
                terminal,
                "invoke restart",
                "invoke",
                json!({"command":"restart","values":{}}),
            ),
            "UNBOUND_OPERATION",
        );
        observations.push(
            events
                .iter()
                .map(|value| normalize_generated_fields(value, &fixture.directory))
                .collect::<Vec<_>>(),
        );
        retain_acceptance_bytes(
            &format!("connected-observations-{terminal}.json"),
            &serde_json::to_vec_pretty(&events).unwrap(),
        );
        retain_acceptance_bytes(
            &format!("connected-state-{terminal}.json"),
            &fixture.checkpoint_bytes(),
        );
        assert!(process.close().success());
    }
    assert_eq!(observations[0], observations[1]);
}

#[test]
fn connected_read_replay_and_handover_preserve_history_without_transferring_grants() {
    let fixture = AuthoringFixture::new();
    prepare_connected_definition(&fixture);
    let mut process = connected_start(&fixture, false, true, true);
    activate_connected_definition(&mut process, false);
    let request = connected_request(&process, "invoke", json!({"command":"inspect","values":{}}));
    let original = process.submit(&request);
    assert_eq!(original["status"], "ok", "{original}");
    let mut replay = original.clone();
    replay["replayed"] = json!(true);
    let checkpoint = fixture.checkpoint_bytes();
    fs::write(fixture.directory.join("catalog.json"), LATER_CATALOG).unwrap();
    assert_eq!(process.submit(&request), replay);
    fs::remove_file(fixture.directory.join("catalog.json")).unwrap();
    assert_eq!(process.submit(&request), replay);
    assert_eq!(fixture.checkpoint_bytes(), checkpoint);
    assert!(process.close().success());
    let mut reopened = connected_start(&fixture, false, false, false);
    assert_eq!(reopened.header["last_receipt"]["response"], original);
    assert_eq!(reopened.submit(&request), replay);
    check_rejection(
        &fixture,
        &checkpoint,
        &connected_step(
            &mut reopened,
            false,
            "invoke inspect",
            "invoke",
            json!({"command":"inspect","values":{}}),
        ),
        "CAPABILITY_NOT_GRANTED",
    );
    connected_ok(
        &mut reopened,
        false,
        "export-interface handover.json",
        "export-interface",
        json!({"destination":"handover.json"}),
    );
    let transferred = fs::read(fixture.directory.join("handover.json")).unwrap();
    assert!(!String::from_utf8_lossy(&transferred).contains("catalog.json"));
    assert!(reopened.close().success());
    for terminal in [false, true] {
        let recipient = AuthoringFixture::new();
        fs::write(recipient.directory.join("handover.json"), &transferred).unwrap();
        let mut receiving = connected_start(&recipient, terminal, true, false);
        connected_ok(
            &mut receiving,
            terminal,
            "activate handover.json",
            "activate",
            json!({"source":"handover.json"}),
        );
        let view = connected_ok(&mut receiving, terminal, "discover", "discover", json!({}));
        assert_eq!(
            view["result"]["last_invocation"],
            original["result"]["invocation"]
        );
        assert_eq!(
            view["result"]["context"]["commands"]["inspect"]["binding"]["granted"],
            false
        );
        let before = recipient.checkpoint_bytes();
        check_rejection(
            &recipient,
            &before,
            &connected_step(
                &mut receiving,
                terminal,
                "invoke inspect",
                "invoke",
                json!({"command":"inspect","values":{}}),
            ),
            "CAPABILITY_NOT_GRANTED",
        );
        assert!(receiving.close().success());
        fs::remove_file(recipient.directory.join("handover.json")).unwrap();
        fs::write(recipient.directory.join("catalog.json"), LATER_CATALOG).unwrap();
        let mut regranted = connected_start(&recipient, terminal, false, true);
        let result = connected_ok(
            &mut regranted,
            terminal,
            "invoke inspect",
            "invoke",
            json!({"command":"inspect","values":{}}),
        );
        assert_catalog_observation(&result, LATER_CATALOG);
        retain_acceptance_bytes(
            &format!("connected-handover-{terminal}.json"),
            &serde_json::to_vec_pretty(&json!({"without_grant":view,"new_observation":result}))
                .unwrap(),
        );
        assert!(regranted.close().success());
    }
    retain_acceptance_bytes("connected-historical-interface.json", &transferred);
}

#[test]
fn connected_read_rejects_bad_targets_documents_signatures_and_stale_requests_atomically() {
    for terminal in [false, true] {
        let fixture = AuthoringFixture::new();
        prepare_connected_definition(&fixture);
        let mut process = connected_start(&fixture, terminal, true, true);
        activate_connected_definition(&mut process, terminal);
        connected_ok(
            &mut process,
            terminal,
            "invoke inspect",
            "invoke",
            json!({"command":"inspect","values":{}}),
        );
        let before = fixture.checkpoint_bytes();
        let mut events = Vec::new();
        for (source, code) in [
            (b"{bad".to_vec(), "INVALID_CONNECTED_DOCUMENT"),
            (
                br#"{"a":1,"\u0061":2}"#.to_vec(),
                "INVALID_CONNECTED_DOCUMENT",
            ),
            (vec![b'"', 0xff, b'"'], "INVALID_CONNECTED_DOCUMENT"),
            (Vec::new(), "INVALID_CONNECTED_DOCUMENT"),
            (vec![b' '; MAX_DOCUMENT_BYTES + 1], "LIMIT_EXCEEDED"),
            (
                format!("\"{}\"", "x".repeat(MAX_SCALAR_BYTES + 1)).into_bytes(),
                "LIMIT_EXCEEDED",
            ),
            (
                format!("{}0{}", "[".repeat(65), "]".repeat(65)).into_bytes(),
                "LIMIT_EXCEEDED",
            ),
        ] {
            fs::write(fixture.directory.join("catalog.json"), &source).unwrap();
            assert_eq!(
                fs::read(fixture.directory.join("catalog.json")).unwrap(),
                source
            );
            let result = connected_step(
                &mut process,
                terminal,
                "invoke inspect",
                "invoke",
                json!({"command":"inspect","values":{}}),
            );
            check_rejection(&fixture, &before, &result, code);
            events.push(result);
        }
        fs::remove_file(fixture.directory.join("catalog.json")).unwrap();
        for kind in ["missing", "symlink", "directory", "fifo"] {
            let target = fixture.directory.join("catalog.json");
            match kind {
                "symlink" => std::os::unix::fs::symlink("spec.json", &target).unwrap(),
                "directory" => fs::create_dir(&target).unwrap(),
                "fifo" => assert!(
                    std::process::Command::new("mkfifo")
                        .arg(&target)
                        .status()
                        .unwrap()
                        .success()
                ),
                _ => (),
            }
            let result = connected_step(
                &mut process,
                terminal,
                "invoke inspect",
                "invoke",
                json!({"command":"inspect","values":{}}),
            );
            check_rejection(&fixture, &before, &result, "CONNECTED_READ_FAILED");
            events.push(result);
            match kind {
                "directory" => fs::remove_dir(&target).unwrap(),
                "missing" => (),
                _ => fs::remove_file(&target).unwrap(),
            }
        }
        check_rejection(
            &fixture,
            &before,
            &connected_step(
                &mut process,
                terminal,
                "invoke inspect unexpected",
                "invoke",
                json!({"command":"inspect","values":{"target":{"kind":"string","value":"spec.json"}}}),
            ),
            "INVALID_CLI_ARGUMENTS",
        );
        assert!(process.close().success());
        let mut machine = connected_start(&fixture, false, false, true);
        let mut stale =
            connected_request(&machine, "invoke", json!({"command":"inspect","values":{}}));
        stale["expected_revision"] = json!("9999");
        check_rejection(
            &fixture,
            &before,
            &machine.submit(&stale),
            "REVISION_CONFLICT",
        );
        retain_acceptance_bytes(
            &format!("connected-rejections-{terminal}.json"),
            &serde_json::to_vec_pretty(&events).unwrap(),
        );
        assert!(machine.close().success());
    }
}

#[test]
fn connected_grants_pin_parent_directories_and_launcher_rejects_invalid_authority() {
    let fixture = AuthoringFixture::new();
    prepare_connected_definition(&fixture);
    fs::create_dir(fixture.directory.join("original")).unwrap();
    fs::write(
        fixture.directory.join("original/catalog.json"),
        FIRST_CATALOG,
    )
    .unwrap();
    let mut command = fixture.command(false, true);
    command.args([
        "--compose",
        "--grant-json-read",
        "services.catalog",
        "original/catalog.json",
    ]);
    let mut process = AuthoringProcess::spawn(command);
    assert_eq!(process.header["event"], "session_open");
    activate_connected_definition(&mut process, false);
    fs::rename(
        fixture.directory.join("original"),
        fixture.directory.join("pinned"),
    )
    .unwrap();
    fs::create_dir(fixture.directory.join("redirect")).unwrap();
    fs::write(
        fixture.directory.join("redirect/catalog.json"),
        LATER_CATALOG,
    )
    .unwrap();
    std::os::unix::fs::symlink("redirect", fixture.directory.join("original")).unwrap();
    let result = connected_ok(
        &mut process,
        false,
        "invoke inspect",
        "invoke",
        json!({"command":"inspect","values":{}}),
    );
    assert_catalog_observation(&result, FIRST_CATALOG);
    fs::write(fixture.directory.join("pinned/catalog.json"), b"42").unwrap();
    let result = connected_ok(
        &mut process,
        false,
        "invoke inspect",
        "invoke",
        json!({"command":"inspect","values":{}}),
    );
    assert_catalog_observation(&result, b"42");
    assert!(process.close().success());
    for arguments in [
        vec!["--grant-json-read", "services.catalog", "catalog.json"],
        vec![
            "--compose",
            "--grant-json-read",
            "invalid/id",
            "catalog.json",
        ],
        vec![
            "--compose",
            "--grant-json-read",
            "services.catalog",
            "catalog.json",
            "--grant-json-read",
            "services.catalog",
            "different.json",
        ],
        vec![
            "--compose",
            "--grant-json-read",
            "services.catalog",
            "missing-parent/catalog.json",
        ],
        vec!["--compose", "--grant-json-read", "services.catalog"],
    ] {
        let rejected = AuthoringFixture::new();
        let mut command = rejected.command(false, true);
        command.args(arguments);
        let mut process = AuthoringProcess::spawn(command);
        assert_eq!(
            process.header["error"]["code"], "INVALID_CAPABILITY_GRANT",
            "{}",
            process.header
        );
        assert!(!rejected.checkpoint.exists());
        assert!(!process.close().success());
    }
    for count in [32, 33] {
        let bounded = AuthoringFixture::new();
        let mut command = bounded.command(false, true);
        command.arg("--compose");
        for index in 0..count {
            command.args([
                "--grant-json-read",
                &format!("catalog{index}"),
                "absent.json",
            ]);
        }
        let mut process = AuthoringProcess::spawn(command);
        if count == 32 {
            assert_eq!(process.header["event"], "session_open");
            assert!(process.close().success());
        } else {
            assert_eq!(process.header["error"]["code"], "INVALID_CAPABILITY_GRANT");
            assert!(!bounded.checkpoint.exists());
            assert!(!process.close().success());
        }
    }
}

#[test]
fn connected_definitions_and_saved_observations_reject_applied_inconsistencies() {
    for terminal in [false, true] {
        let fixture = AuthoringFixture::new();
        prepare_connected_definition(&fixture);
        let mut process = connected_start(&fixture, terminal, true, true);
        activate_connected_definition(&mut process, terminal);
        connected_ok(
            &mut process,
            terminal,
            "invoke inspect",
            "invoke",
            json!({"command":"inspect","values":{}}),
        );
        connected_ok(
            &mut process,
            terminal,
            "export-interface original.json",
            "export-interface",
            json!({"destination":"original.json"}),
        );
        let before = fixture.checkpoint_bytes();
        let baseline: Value =
            serde_json::from_slice(&fs::read(fixture.directory.join("spec.json")).unwrap())
                .unwrap();
        let exported: Value =
            serde_json::from_slice(&fs::read(fixture.directory.join("original.json")).unwrap())
                .unwrap();
        let mut events = Vec::new();
        for (pointer, replacement) in [
            (
                "/contexts/services/commands/inspect/binding/provider",
                json!("exec"),
            ),
            (
                "/contexts/services/commands/inspect/binding/kind",
                json!("unknown"),
            ),
            (
                "/contexts/services/commands/inspect/binding/capability",
                json!("../catalog"),
            ),
            (
                "/contexts/services/commands/inspect/binding/capability",
                Value::Null,
            ),
            (
                "/contexts/services/commands/inspect/binding",
                json!({"kind":"connected","provider":"json-file-read-v1","capability":"services.catalog","granted":true}),
            ),
            (
                "/contexts/services/commands/inspect/parameters",
                json!([{"name":"file","type":"string","help":"Attempt to redirect read"}]),
            ),
        ] {
            let mut mutant = baseline.clone();
            *mutant.pointer_mut(pointer).unwrap() = replacement;
            assert_ne!(mutant, baseline, "INVALID: definition mutant did not apply");
            let bytes = serde_json::to_vec(&mutant).unwrap();
            fs::write(fixture.directory.join("mutant.json"), &bytes).unwrap();
            assert_eq!(
                fs::read(fixture.directory.join("mutant.json")).unwrap(),
                bytes
            );
            let response = connected_step(
                &mut process,
                terminal,
                "activate mutant.json",
                "activate",
                json!({"source":"mutant.json"}),
            );
            check_rejection(&fixture, &before, &response, "INVALID_CLI_DEFINITION");
            println!("CONNECTED_MUTANT_LANDED definition {pointer}");
            events.push(response);
        }
        for (field, replacement) in [
            ("binding", json!("simulated")),
            ("effect", json!("simulation_only")),
            ("simulated_steps", json!(1)),
            ("output_json_text", json!("null")),
            ("observation", Value::Null),
            ("observation/capability", json!("different.catalog")),
            ("observation/source_bytes", json!(0)),
            ("observation/source_bytes", json!(MAX_DOCUMENT_BYTES + 1)),
            ("observation/source_sha256", json!("not-a-digest")),
            ("observation/output_sha256", json!("0".repeat(64))),
        ] {
            let mut mutant = exported.clone();
            *mutant
                .pointer_mut(&format!("/interface/last_invocation/{field}"))
                .unwrap() = replacement;
            assert_ne!(
                mutant, exported,
                "INVALID: observation mutant did not apply"
            );
            let bytes = serde_json::to_vec(&mutant).unwrap();
            fs::write(fixture.directory.join("mutant.json"), &bytes).unwrap();
            assert_eq!(
                fs::read(fixture.directory.join("mutant.json")).unwrap(),
                bytes
            );
            let response = connected_step(
                &mut process,
                terminal,
                "activate mutant.json",
                "activate",
                json!({"source":"mutant.json"}),
            );
            check_rejection(&fixture, &before, &response, "INVALID_CLI_DEFINITION");
            println!("CONNECTED_MUTANT_LANDED observation {field}");
            events.push(response);
        }
        retain_acceptance_bytes(
            &format!("connected-mutants-{terminal}.json"),
            &serde_json::to_vec_pretty(&events).unwrap(),
        );
        assert!(process.close().success());
        let mut checkpoint: Value = serde_json::from_slice(&before).unwrap();
        checkpoint["active_interface"]["last_invocation"]["observation"]["capability"] =
            json!("different.catalog");
        checkpoint["last_receipt"]["response"]["result"]["invocation"]["observation"]["capability"] =
            json!("different.catalog");
        let mutant = serde_json::to_vec(&checkpoint).unwrap();
        assert_ne!(mutant, before);
        fs::write(&fixture.checkpoint, &mutant).unwrap();
        assert_eq!(fixture.checkpoint_bytes(), mutant);
        let mut command = fixture.command(false, false);
        command.args(["--compose", "--constraints"]);
        let mut rejected = AuthoringProcess::spawn(command);
        assert_eq!(
            rejected.header["error"]["code"], "INVALID_SESSION",
            "{}",
            rejected.header
        );
        assert_eq!(fixture.checkpoint_bytes(), mutant);
        assert!(!rejected.close().success());
        println!("CONNECTED_MUTANT_LANDED checkpoint and receipt");
    }
}

#[test]
fn assembled_connected_components_preserve_requirements_and_share_only_explicit_read_authority() {
    for terminal in [false, true] {
        let fixture = AuthoringFixture::new();
        prepare_connected_definition(&fixture);
        let baseline: Value =
            serde_json::from_slice(&fs::read(fixture.directory.join("spec.json")).unwrap())
                .unwrap();
        for mount in ["primary", "secondary"] {
            let mut definition = baseline.clone();
            definition["id"] = json!(mount);
            fs::write(
                fixture.directory.join(format!("{mount}.json")),
                serde_json::to_vec(
                    &json!({"format":"cli-component-v1","requires":[],"definition":definition}),
                )
                .unwrap(),
            )
            .unwrap();
        }
        fs::write(fixture.directory.join("assembly.json"), serde_json::to_vec(&json!({"format":"cli-assembly-v1","id":"catalog-pair","description":"Two independent CLI components sharing an explicitly granted read.","components":[{"mount":"primary","source":"primary.json"},{"mount":"secondary","source":"secondary.json"}]})).unwrap()).unwrap();
        let mut process = connected_start(&fixture, terminal, true, true);
        for (line, operation, arguments) in [
            (
                "import assembly.json",
                "import",
                json!({"source":"assembly.json"}),
            ),
            ("assemble", "assemble", json!({})),
            ("commit", "commit", json!({})),
            ("activate", "activate", json!({})),
            (
                "enter primary.services",
                "enter",
                json!({"context":"primary.services"}),
            ),
        ] {
            connected_ok(&mut process, terminal, line, operation, arguments);
        }
        let tree = connected_ok(&mut process, terminal, "tree", "tree", json!({}));
        assert_eq!(
            tree["result"]["verb_tree_text"]
                .as_str()
                .unwrap()
                .matches("[connected:json-file-read-v1]")
                .count(),
            2
        );
        connected_ok(
            &mut process,
            terminal,
            "invoke quota 9",
            "invoke",
            json!({"command":"quota","values":{"value":{"kind":"number","value":"9"}}}),
        );
        let mut events = vec![tree];
        for mount in ["primary", "secondary"] {
            connected_ok(
                &mut process,
                terminal,
                &format!("enter {mount}.services"),
                "enter",
                json!({"context":format!("{mount}.services")}),
            );
            let response = connected_ok(
                &mut process,
                terminal,
                "invoke inspect",
                "invoke",
                json!({"command":"inspect","values":{}}),
            );
            assert_catalog_observation(&response, FIRST_CATALOG);
            assert_eq!(
                response["result"]["invocation"]["operation_id"],
                format!("{mount}.catalog.inspect")
            );
            events.push(response);
        }
        assert_eq!(
            fixture
                .checkpoint_state()
                .active_interface
                .as_ref()
                .unwrap()
                .mock_state
                .compact_document_json(),
            r#"{"primary":{"quota":9},"secondary":{"quota":7}}"#
        );
        connected_ok(
            &mut process,
            terminal,
            "export-interface assembled.json",
            "export-interface",
            json!({"destination":"assembled.json"}),
        );
        let export = fs::read(fixture.directory.join("assembled.json")).unwrap();
        assert!(process.close().success());
        for name in [
            "primary.json",
            "secondary.json",
            "assembly.json",
            "catalog.json",
            "spec.json",
        ] {
            fs::remove_file(fixture.directory.join(name)).unwrap();
        }
        let mut resumed = connected_start(&fixture, terminal, false, false);
        let view = connected_ok(&mut resumed, terminal, "discover", "discover", json!({}));
        assert_eq!(
            view["result"]["context"]["commands"]["inspect"]["binding"]["capability"],
            "services.catalog"
        );
        assert_eq!(
            view["result"]["context"]["commands"]["inspect"]["binding"]["granted"],
            false
        );
        assert!(resumed.close().success());
        let recipient = AuthoringFixture::new();
        fs::write(recipient.directory.join("assembled.json"), &export).unwrap();
        let mut receiving = connected_start(&recipient, terminal, true, false);
        connected_ok(
            &mut receiving,
            terminal,
            "activate assembled.json",
            "activate",
            json!({"source":"assembled.json"}),
        );
        let before = recipient.checkpoint_bytes();
        check_rejection(
            &recipient,
            &before,
            &connected_step(
                &mut receiving,
                terminal,
                "invoke inspect",
                "invoke",
                json!({"command":"inspect","values":{}}),
            ),
            "CAPABILITY_NOT_GRANTED",
        );
        assert!(receiving.close().success());
        retain_acceptance_bytes(
            &format!("connected-assembled-{terminal}.json"),
            &serde_json::to_vec_pretty(&events).unwrap(),
        );
        retain_acceptance_bytes(
            &format!("connected-assembled-{terminal}.interface.json"),
            &export,
        );
    }
}

#[cfg(feature = "fault-injection")]
#[test]
fn connected_read_publication_failures_preserve_recoverable_observations_without_claiming_no_read()
{
    for terminal in [false, true] {
        for point in ["after_checkpoint_write", "after_checkpoint_rename"] {
            let fixture = AuthoringFixture::new();
            prepare_connected_definition(&fixture);
            let mut seed = connected_start(&fixture, false, true, false);
            activate_connected_definition(&mut seed, false);
            assert!(seed.close().success());
            let before = fixture.checkpoint_bytes();
            let marker = fixture.directory.join("connected-fault.json");
            let mut command = fixture.command(terminal, false);
            command
                .args([
                    "--compose",
                    "--constraints",
                    "--grant-json-read",
                    "services.catalog",
                    "catalog.json",
                ])
                .env("CLI_TEST_FAULT_POINT", point)
                .env("CLI_TEST_FAULT_ACTION", "error")
                .env("CLI_TEST_FAULT_MARKER", &marker);
            let mut failing = AuthoringProcess::spawn(command);
            assert_eq!(
                failing.header["event"], "session_open",
                "{}",
                failing.header
            );
            let response = connected_step(
                &mut failing,
                terminal,
                "invoke inspect",
                "invoke",
                json!({"command":"inspect","values":{}}),
            );
            wait_for_fault(&marker, point, &failing);
            assert_eq!(
                fs::read(fixture.directory.join("catalog.json")).unwrap(),
                FIRST_CATALOG
            );
            if point == "after_checkpoint_write" {
                check_rejection(&fixture, &before, &response, "PERSISTENCE_FAILED");
                assert!(failing.close().success());
                fs::write(fixture.directory.join("catalog.json"), LATER_CATALOG).unwrap();
                let mut retry = connected_start(&fixture, terminal, false, true);
                assert_eq!(fixture.checkpoint_bytes(), before);
                let observed = connected_ok(
                    &mut retry,
                    terminal,
                    "invoke inspect",
                    "invoke",
                    json!({"command":"inspect","values":{}}),
                );
                assert_catalog_observation(&observed, LATER_CATALOG);
                assert!(retry.close().success());
            } else {
                assert_eq!(response["status"], "uncertain", "{response}");
                assert_eq!(response["error"]["code"], "PERSISTENCE_UNCERTAIN");
                let published = fixture.checkpoint_bytes();
                let checkpoint = fixture.checkpoint_state();
                let receipt = checkpoint.last_receipt.as_ref().unwrap();
                let request = serde_json::to_value(&receipt.request).unwrap();
                let saved = serde_json::to_value(&receipt.response).unwrap();
                assert_catalog_observation(&saved, FIRST_CATALOG);
                assert!(!failing.close().success());
                fs::remove_file(fixture.directory.join("catalog.json")).unwrap();
                let mut resumed = connected_start(&fixture, false, false, false);
                let replay = resumed.submit(&request);
                assert_eq!(replay["replayed"], true);
                assert_catalog_observation(&replay, FIRST_CATALOG);
                assert_eq!(fixture.checkpoint_bytes(), published);
                assert!(resumed.close().success());
            }
            retain_acceptance_bytes(
                &format!("connected-fault-{terminal}-{point}.json"),
                &serde_json::to_vec_pretty(&response).unwrap(),
            );
        }
    }
}
