//! Direct CLI acceptance measures real stdout, stderr, exit status, and persistent interface state.

mod authoring_fixture;
use authoring_fixture::*;
use std::process::Command;

#[test]
fn exported_cli_runs_direct_verbs_without_authoring_setup() {
    let output = Command::new(env!("CARGO_BIN_EXE_cli"))
        .arg("run")
        .arg(repository_path(
            "docs/connected/acceptance/expected-definition.json",
        ))
        .args(["services", "quota", "1e400"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"{\"quota\":1e400}\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("[simulated]"));
}

use programmable_cli::authoring_protocol::SessionRevision;
use programmable_cli::cli_composition::parse_cli_interface_source;
use programmable_cli::cli_interface::CliInterfaceOrigin;
use programmable_cli::cli_run_completion::CliRunCompleter;
use programmable_cli::cli_run_routes::CliRunRoutes;
use programmable_cli::document_value::DocumentValue;
use reedline::Completer;
use serde_json::{Value, json};
use std::fs;
use std::io::{Read, Write};
use std::process::{Output, Stdio};
use std::time::{Duration, Instant};

fn run_command(fixture: &AuthoringFixture) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cli"));
    command.arg("run").current_dir(&fixture.directory);
    command
}
fn run_output(fixture: &AuthoringFixture, args: &[&str], input: &str) -> Output {
    let mut child = run_command(fixture)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
fn require_exit(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn catalog_fixture() -> AuthoringFixture {
    let fixture = AuthoringFixture::new();
    fs::copy(
        repository_path("docs/connected/acceptance/expected-definition.json"),
        fixture.directory.join("spec.json"),
    )
    .unwrap();
    fs::copy(
        repository_path("docs/connected/acceptance/catalog-initial.json"),
        fixture.directory.join("catalog.json"),
    )
    .unwrap();
    fixture
}
fn precise_document(bytes: &[u8]) -> DocumentValue {
    DocumentValue::parse_document(std::str::from_utf8(bytes).unwrap()).unwrap()
}
fn read_event(output: &Output) -> Value {
    require_exit(output, 0);
    serde_json::from_slice(&output.stdout).unwrap()
}
fn spec_value(fixture: &AuthoringFixture) -> Value {
    serde_json::from_slice(&fs::read(fixture.directory.join("spec.json")).unwrap()).unwrap()
}
fn write_spec(fixture: &AuthoringFixture, value: &Value) {
    fs::write(
        fixture.directory.join("spec.json"),
        serde_json::to_vec(value).unwrap(),
    )
    .unwrap();
}
fn echo_command(id: &str, kind: &str) -> Value {
    json!({"id":id,"help":"Echo declared input.","parameters":[{"name":"value","type":kind,"help":"Exact input."}],"binding":{"kind":"simulated","steps":[],"output":{"source":"argument","name":"value"}}})
}
fn completion_for(fixture: &AuthoringFixture) -> CliRunCompleter {
    let active = parse_cli_interface_source(
        &fs::read(fixture.directory.join("spec.json")).unwrap(),
        None,
        CliInterfaceOrigin {
            intent_text: "Acceptance".into(),
            revision: SessionRevision(0),
        },
    )
    .unwrap();
    let routes = CliRunRoutes::from_cli_definition(&active.definition).unwrap();
    CliRunCompleter::new_run_completer(active, routes)
}

#[test]
fn command_authored_export_runs_and_agrees_with_canonical_invocation() {
    let fixture = catalog_fixture();
    let mut command = fixture.command(true, true);
    command.arg("--compose");
    let mut author = AuthoringProcess::spawn(command);
    let recipe = fs::read_to_string(repository_path(
        "docs/connected/acceptance/catalog.commands",
    ))
    .unwrap();
    let mut events = Vec::new();
    for line in recipe.lines() {
        assert!(!line.contains(['{', '}', '[', ']']));
        let response = author.terminal(line);
        assert_eq!(response["status"], "ok", "{response}");
        events.push(response);
    }
    assert_eq!(author.terminal("activate")["status"], "ok");
    assert_eq!(
        author.terminal("export-interface portable.json")["status"],
        "ok"
    );
    let canonical = author.terminal("invoke services/quota 1e400");
    assert_eq!(canonical["status"], "ok", "{canonical}");
    assert_eq!(canonical["session"]["active_interface"]["context"], "root");
    assert!(author.close().success());
    for source in ["connected-cli.json", "portable.json"] {
        let output = run_output(
            &fixture,
            &["--json", source, "services", "quota", "1e400"],
            "",
        );
        let direct = read_event(&output);
        assert_eq!(
            direct["result"]["invocation"],
            canonical["result"]["invocation"]
        );
        assert!(output.stderr.is_empty());
        events.push(direct);
    }
    retain_acceptance_bytes(
        "run-authored-and-canonical.json",
        &serde_json::to_vec_pretty(&events).unwrap(),
    );
    retain_acceptance_bytes(
        "run-command-authored-spec.json",
        &fs::read(fixture.directory.join("connected-cli.json")).unwrap(),
    );
}

#[test]
fn direct_connected_read_labels_history_and_requires_fresh_grants() {
    let fixture = catalog_fixture();
    let granted = [
        "--session",
        "run.json",
        "--grant-json-read",
        "services.catalog",
        "catalog.json",
        "spec.json",
        "services",
        "inspect",
    ];
    let output = run_output(&fixture, &granted, "");
    require_exit(&output, 0);
    assert_eq!(
        output.stdout,
        b"{\"enabled\":true,\"name\":\"api\",\"quota\":1.2300}\n"
    );
    assert!(output.stderr.is_empty());
    let before = fs::read(fixture.directory.join("run.json")).unwrap();
    let denied = run_output(
        &fixture,
        &["--session", "run.json", "spec.json", "services", "inspect"],
        "",
    );
    require_exit(&denied, 1);
    assert!(denied.stdout.is_empty());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("CAPABILITY_NOT_GRANTED"));
    assert_eq!(
        fs::read(fixture.directory.join("run.json")).unwrap(),
        before
    );
    fs::remove_file(fixture.directory.join("catalog.json")).unwrap();
    let status = read_event(&run_output(
        &fixture,
        &["--json", "--session", "run.json", "-", ":status"],
        "",
    ));
    assert_eq!(status["result"]["last_invocation"]["binding"], "connected");
    require_exit(
        &run_output(
            &fixture,
            &["--session", "run.json", "-", ":export", "history.json"],
            "",
        ),
        0,
    );
    let imported = read_event(&run_output(
        &fixture,
        &["--json", "history.json", ":status"],
        "",
    ));
    assert_eq!(
        imported["result"]["last_invocation"],
        status["result"]["last_invocation"]
    );
    let denied = run_output(&fixture, &["history.json", "services", "inspect"], "");
    require_exit(&denied, 1);
    retain_acceptance_bytes(
        "run-connected-status.json",
        &serde_json::to_vec_pretty(&status).unwrap(),
    );
    retain_acceptance_bytes(
        "run-history-export.json",
        &fs::read(fixture.directory.join("history.json")).unwrap(),
    );
}

#[test]
fn persistent_qualified_invocation_preserves_cursor_and_failures_preserve_all_bytes() {
    let fixture = catalog_fixture();
    let output = run_output(
        &fixture,
        &["--session", "run.json", "spec.json"],
        "services\nquota 1.2300\n",
    );
    require_exit(&output, 0);
    assert_eq!(output.stdout, b"{\"quota\":1.2300}\n");
    let original = fs::read(fixture.directory.join("run.json")).unwrap();
    let initial: Value = serde_json::from_slice(&original).unwrap();
    assert_eq!(initial["active_interface"]["context"], "services");
    assert_eq!(initial["revision"], "2");
    for (words, code) in [
        (vec!["services", "restart"], 1),
        (vec!["services", "quota"], 2),
        (vec!["services", "quota", "abc"], 2),
        (vec!["services", "quota", "7", "extra"], 2),
        (vec!["services", "missing"], 2),
        (vec!["set", "x", "7"], 2),
    ] {
        let mut args = vec!["--session", "run.json", "spec.json"];
        args.extend(words);
        let failed = run_output(&fixture, &args, "");
        require_exit(&failed, code);
        assert!(failed.stdout.is_empty());
        let error = String::from_utf8_lossy(&failed.stderr);
        if error.contains("INVALID_CLI_ARGUMENTS") {
            assert!(error.contains("Use :help"), "{error}");
        }
        assert_eq!(
            fs::read(fixture.directory.join("run.json")).unwrap(),
            original
        );
    }
    let event = read_event(&run_output(
        &fixture,
        &[
            "--json",
            "--session",
            "run.json",
            "spec.json",
            "services",
            "quota",
            "1e400",
        ],
        "",
    ));
    assert_eq!(event["session"]["revision"], "3");
    assert_eq!(event["session"]["active_interface"]["context"], "services");
    assert_eq!(
        event["result"]["invocation"]["output_json_text"],
        "{\"quota\":1e400}"
    );
    let before = fs::read(fixture.directory.join("run.json")).unwrap();
    let mut changed = spec_value(&fixture);
    changed["description"] = json!("Changed definition");
    write_spec(&fixture, &changed);
    let mismatch = run_output(
        &fixture,
        &["--session", "run.json", "spec.json", ":status"],
        "",
    );
    require_exit(&mismatch, 1);
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("RUN_SPEC_MISMATCH"));
    assert_eq!(
        fs::read(fixture.directory.join("run.json")).unwrap(),
        before
    );
    fs::remove_file(fixture.directory.join("spec.json")).unwrap();
    let reopened = read_event(&run_output(
        &fixture,
        &["--json", "--session", "run.json", "-", ":status"],
        "",
    ));
    assert_eq!(
        reopened["result"]["last_invocation"],
        event["result"]["invocation"]
    );
    // A qualified target at root does not conceal an implicit navigation transition.
    require_exit(
        &run_output(&fixture, &["--session", "run.json", "-", ":top"], ""),
        0,
    );
    let event = read_event(&run_output(
        &fixture,
        &[
            "--json",
            "--session",
            "run.json",
            "-",
            "services",
            "quota",
            "9",
        ],
        "",
    ));
    assert_eq!(event["session"]["active_interface"]["context"], "root");
    assert_eq!(event["session"]["revision"], "5");
    retain_acceptance_bytes(
        "run-persistent-checkpoint.json",
        &fs::read(fixture.directory.join("run.json")).unwrap(),
    );
}

#[test]
fn generated_help_tree_and_completion_describe_callable_routes() {
    let fixture = catalog_fixture();
    let mut spec = spec_value(&fixture);
    spec["contexts"]["services"]["commands"]["enable"] = echo_command("catalog.enable", "boolean");
    spec["contexts"]["services"]["help"] = json!("Text\u{1b}[31m\nwith controls");
    write_spec(&fixture, &spec);
    let help_output = run_output(&fixture, &["spec.json", "services", "--help"], "");
    require_exit(&help_output, 0);
    assert!(!help_output.stdout.contains(&27));
    let help = read_event(&run_output(
        &fixture,
        &["--json", "spec.json", "services", "--help"],
        "",
    ));
    let commands: Vec<_> = help["result"]["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|command| command["word"].as_str().unwrap())
        .collect();
    assert_eq!(commands, vec!["enable", "inspect", "quota", "restart"]);
    let inspect = read_event(&run_output(
        &fixture,
        &["--json", "spec.json", "services", "inspect", "--help"],
        "",
    ));
    assert_eq!(
        inspect["result"]["commands"][0]["requirement"]["capability"],
        "services.catalog"
    );
    assert_eq!(
        inspect["result"]["commands"][0]["requirement"]["granted"],
        false
    );
    let tree = run_output(&fixture, &["spec.json", ":tree"], "");
    require_exit(&tree, 0);
    let tree = String::from_utf8(tree.stdout).unwrap();
    assert!(tree.contains("|-- enable <value:boolean> [simulated]"));
    assert!(!tree.contains("invoke "));
    let mut completion = completion_for(&fixture);
    for (line, expected, start) in [
        ("ser", "services", 0),
        ("services qu", "quota", 9),
        ("services enable f", "false", 16),
        ("services enable \"t", "true", 16),
        (":help services qu", "quota", 15),
    ] {
        let result = completion.complete(line, line.len());
        let suggestions = result.suggestions();
        assert_eq!(suggestions.len(), 1, "{line}: {suggestions:?}");
        assert_eq!(suggestions[0].value, expected);
        assert_eq!(suggestions[0].span.start, start);
        assert_eq!(suggestions[0].span.end, line.len());
    }
    assert!(completion.complete("é", 1).suggestions().is_empty());
    assert!(
        completion
            .complete("services quota 2 ", 17)
            .suggestions()
            .is_empty()
    );
    retain_acceptance_bytes("run-tree.txt", tree.as_bytes());
    retain_acceptance_bytes("run-help.json", &serde_json::to_vec_pretty(&help).unwrap());
}

#[test]
fn argv_and_interactive_strings_are_literal_and_control_like_domain_words_remain_callable() {
    let fixture = catalog_fixture();
    let mut spec = spec_value(&fixture);
    for word in ["set", "help", "tree", "exit", "invoke", "--help", "--json"] {
        spec["contexts"]["root"]["commands"][word] = echo_command(word, "string");
    }
    write_spec(&fixture, &spec);
    let input = "spaces \"quotes\" \\ newline\n雪 $HOME `touch should-not-exist` $(echo surprise)";
    for word in ["set", "help", "tree", "exit", "invoke", "--help", "--json"] {
        let args = if word.starts_with('-') {
            vec!["spec.json", "--", word, input]
        } else {
            vec!["spec.json", word, input]
        };
        let output = run_output(&fixture, &args, "");
        require_exit(&output, 0);
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            json!(input)
        );
    }
    let output = run_output(
        &fixture,
        &["spec.json"],
        &format!("set {}\n:exit\n", serde_json::to_string(input).unwrap()),
    );
    require_exit(&output, 0);
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!(input)
    );
    let output = run_output(&fixture, &["spec.json", "set", "--", "--help"], "");
    require_exit(&output, 0);
    assert_eq!(output.stdout, b"\"--help\"\n");
    let output = run_output(&fixture, &["spec.json", "set", ""], "");
    require_exit(&output, 0);
    assert_eq!(output.stdout, b"\"\"\n");
    assert!(!fixture.directory.join("should-not-exist").exists());
    let output = run_output(
        &fixture,
        &["spec.json"],
        "missing\nservices quota 8\n:exit\n",
    );
    require_exit(&output, 2);
    assert_eq!(output.stdout, b"{\"quota\":8}\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("INVALID_RUN_COMMAND"));
}

#[test]
fn assembled_nested_routes_use_local_words_and_keep_component_state_separate() {
    let fixture = catalog_fixture();
    // Assembly embeds this nested source; run cannot depend on its source paths after export.
    let mut services: Value = serde_json::from_slice(
        &fs::read(repository_path(
            "docs/components/acceptance/services.expected.json",
        ))
        .unwrap(),
    )
    .unwrap();
    services["definition"]["contexts"]["services"] = json!({"parent":"root","help":"Nested service context","related":[],"commands":{"echo":echo_command("echo","string")}});
    fs::write(
        fixture.directory.join("services.component.json"),
        serde_json::to_vec(&services).unwrap(),
    )
    .unwrap();
    fs::copy(
        repository_path("docs/components/acceptance/queues.expected.json"),
        fixture.directory.join("queues.component.json"),
    )
    .unwrap();
    let manifest = json!({"format":"cli-assembly-v1","id":"assembled-run","description":"Run assembly","components":[{"mount":"primary","source":"services.component.json"},{"mount":"queues","source":"queues.component.json"}]});
    fs::write(
        fixture.directory.join("assembly.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let mut command = fixture.command(true, true);
    command.arg("--compose");
    let mut author = AuthoringProcess::spawn(command);
    for line in [
        "import assembly.json",
        "assemble",
        "commit",
        "activate",
        "export-interface assembled.json",
    ] {
        let response = author.terminal(line);
        assert_eq!(response["status"], "ok", "{line}: {response}");
    }
    assert!(author.close().success());
    fs::remove_file(fixture.directory.join("services.component.json")).unwrap();
    fs::remove_file(fixture.directory.join("queues.component.json")).unwrap();
    let output = run_output(
        &fixture,
        &["assembled.json", "primary", "services", "echo", "hello"],
        "",
    );
    require_exit(&output, 0);
    assert_eq!(output.stdout, b"\"hello\"\n");
    let tree = run_output(&fixture, &["assembled.json", ":tree"], "");
    require_exit(&tree, 0);
    let text = String::from_utf8_lossy(&tree.stdout);
    assert!(text.contains("services/"));
    assert!(!text.contains("primary.services/"));
    let output = run_output(
        &fixture,
        &["--session", "run.json", "assembled.json"],
        "primary\nservices\necho \"nested snow 雪\"\n:up\nquota 99\n:top\nqueues pause true\n",
    );
    require_exit(&output, 0);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "\"nested snow 雪\"\n99\ntrue\n"
    );
    let state: programmable_cli::authoring_protocol::SessionCheckpoint =
        serde_json::from_slice(&fs::read(fixture.directory.join("run.json")).unwrap()).unwrap();
    assert_eq!(state.active_interface.unwrap().mock_state,precise_document(b"{\"primary\":{\"name\":\"api\",\"quota\":99},\"queues\":{\"name\":\"jobs\",\"paused\":true}}"));
    // Completion also uses the assembled local word, not the stored global identity.
    fs::copy(
        fixture.directory.join("assembled.json"),
        fixture.directory.join("spec.json"),
    )
    .unwrap();
    let mut completion = completion_for(&fixture);
    let result = completion.complete("primary ser", 11);
    assert_eq!(result.suggestions()[0].value, "services");
    retain_acceptance_bytes("run-assembled-tree.txt", &tree.stdout);
    retain_acceptance_bytes(
        "run-assembled-export.json",
        &fs::read(fixture.directory.join("assembled.json")).unwrap(),
    );
}

#[test]
fn ambiguous_routes_and_invalid_sources_reject_before_checkpoint_creation() {
    let fixture = catalog_fixture();
    let original = spec_value(&fixture);
    for (label, mut variant) in [
        ("collision", original.clone()),
        ("invalid-parent", original.clone()),
        ("invalid-parameter", original.clone()),
    ] {
        match label {
            "collision" => {
                variant["contexts"]["root"]["commands"]["services"] =
                    echo_command("collision", "string")
            }
            "invalid-parent" => variant["contexts"]["services"]["parent"] = json!("missing"),
            _ => {
                variant["contexts"]["services"]["commands"]["quota"]["parameters"][0]["type"] =
                    json!("anything")
            }
        }
        assert_ne!(variant, original, "Mutant must apply");
        write_spec(&fixture, &variant);
        let output = run_output(
            &fixture,
            &["--session", "run.json", "spec.json", ":help"],
            "",
        );
        require_exit(&output, 1);
        assert!(!fixture.directory.join("run.json").exists());
        if label == "collision" {
            assert!(String::from_utf8_lossy(&output.stderr).contains("AMBIGUOUS_RUN_ROUTE"));
        }
        println!("RUN_REJECTION_LANDED {label}");
    }
    for args in [
        vec![],
        vec!["--session"],
        vec!["--bogus"],
        vec!["missing.json"],
    ] {
        let output = run_output(&fixture, &args, "");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
    let output = run_output(&fixture, &["--help"], "");
    require_exit(&output, 0);
    assert!(String::from_utf8_lossy(&output.stdout).contains("cli run"));
    #[cfg(target_os = "linux")]
    {
        let output = run_command(&fixture)
            .arg("--help")
            .stdout(
                fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/full")
                    .unwrap(),
            )
            .output()
            .unwrap();
        require_exit(&output, 1);
        assert!(String::from_utf8_lossy(&output.stderr).contains("RUN_DELIVERY_FAILED"));
    }
}

#[cfg(feature = "fault-injection")]
#[test]
fn run_publication_faults_preserve_recovery_and_never_acknowledge_uncertain_success() {
    for (point, code, saved) in [
        ("before_checkpoint_write", 1, false),
        ("after_checkpoint_rename", 1, true),
        ("before_response_delivery", 1, true),
    ] {
        let fixture = catalog_fixture();
        require_exit(
            &run_output(
                &fixture,
                &["--session", "run.json", "spec.json", ":help"],
                "",
            ),
            0,
        );
        let before = fs::read(fixture.directory.join("run.json")).unwrap();
        let marker = fixture.directory.join("fault.json");
        let output = run_command(&fixture)
            .args([
                "--session",
                "run.json",
                "spec.json",
                "services",
                "quota",
                "42",
            ])
            .env("CLI_TEST_FAULT_POINT", point)
            .env("CLI_TEST_FAULT_ACTION", "error")
            .env("CLI_TEST_FAULT_MARKER", &marker)
            .output()
            .unwrap();
        require_exit(&output, code);
        assert!(output.stdout.is_empty());
        let fault: Value = serde_json::from_slice(&fs::read(&marker).unwrap()).unwrap();
        assert_eq!(fault["point"], point);
        assert!(fault["pid"].as_u64().unwrap() > 0);
        println!("FAULT_LANDED {fault}");
        let after = fs::read(fixture.directory.join("run.json")).unwrap();
        if saved {
            assert_ne!(after, before);
            let state: Value = serde_json::from_slice(&after).unwrap();
            assert_eq!(state["revision"], "1");
            assert_eq!(state["active_interface"]["context"], "root");
            assert_eq!(
                state["last_receipt"]["response"]["result"]["invocation"]["output_json_text"],
                "{\"quota\":42}"
            );
        } else {
            assert_eq!(after, before);
        }
        let reopened = read_event(&run_output(
            &fixture,
            &["--json", "--session", "run.json", "-", ":status"],
            "",
        ));
        assert_eq!(
            reopened["session"]["revision"],
            if saved { "1" } else { "0" }
        );
        retain_acceptance_bytes(&format!("run-fault-{point}.stderr"), &output.stderr);
        retain_acceptance_bytes(&format!("run-fault-{point}.json"), &after);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn actual_terminal_completes_context_and_executes_direct_commands() {
    use std::sync::mpsc;
    let fixture = catalog_fixture();
    let quote = |value: &str| format!("'{}'", value.replace('\'', "'\\''"));
    let shell = format!(
        "stty cols 120 rows 30; exec {} run --session tty.json spec.json",
        quote(env!("CARGO_BIN_EXE_cli"))
    );
    let mut child = Command::new("script")
        .args(["-qefc", &shell, "/dev/null"])
        .env("TERM", "xterm-256color")
        .current_dir(&fixture.directory)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = child.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = [0; 4096];
        loop {
            match output.read(&mut bytes) {
                Ok(0) | Err(_) => break,
                Ok(count) => {
                    if sender.send(bytes[..count].to_vec()).is_err() {
                        break;
                    }
                }
            }
        }
    });
    let actions = [
        ("service-catalog-connected [/]", "ser"),
        ("mser\x1b", "\t"),
        ("Read the granted catalog or change simulated quota.", "\r"),
        ("mservices \x1b", "\r"),
        ("service-catalog-connected [services]", "quota 1e400"),
        ("mquota 1e400\x1b", "\r"),
        ("{\"quota\":1e400}", ""),
        ("service-catalog-connected [services]", ":top"),
        ("m:top\x1b", "\r"),
        ("service-catalog-connected [/]", ":exit"),
        ("m:exit\x1b", "\r"),
    ];
    let mut transcript = Vec::new();
    let mut pending = Vec::new();
    let mut stage = 0;
    let mut cpr = 0;
    let deadline = Instant::now() + Duration::from_secs(25);
    while Instant::now() < deadline {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(bytes) => {
                transcript.extend(&bytes);
                pending.extend(bytes);
                let queries = transcript
                    .windows(4)
                    .filter(|bytes| *bytes == b"\x1b[6n")
                    .count();
                while cpr < queries {
                    input.write_all(b"\x1b[1;1R").unwrap();
                    input.flush().unwrap();
                    cpr += 1;
                }
                while stage < actions.len() {
                    let Some(position) = pending
                        .windows(actions[stage].0.len())
                        .position(|bytes| bytes == actions[stage].0.as_bytes())
                    else {
                        break;
                    };
                    pending.drain(..position + actions[stage].0.len());
                    input.write_all(actions[stage].1.as_bytes()).unwrap();
                    input.flush().unwrap();
                    stage += 1;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
    drop(input);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            break child.wait().unwrap();
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    retain_acceptance_bytes("run-terminal.raw", &transcript);
    assert_eq!(
        stage,
        actions.len(),
        "Terminal transcript: {}",
        String::from_utf8_lossy(&transcript)
    );
    assert!(
        status.success(),
        "Terminal status {status}: {}",
        String::from_utf8_lossy(&transcript)
    );
    assert!(String::from_utf8_lossy(&transcript).contains("[simulated]"));
    let state: Value =
        serde_json::from_slice(&fs::read(fixture.directory.join("tty.json")).unwrap()).unwrap();
    assert_eq!(state["revision"], "3");
    assert_eq!(state["active_interface"]["context"], "root");
    assert_eq!(
        state["active_interface"]["last_invocation"]["output_json_text"],
        "{\"quota\":1e400}"
    );
}
