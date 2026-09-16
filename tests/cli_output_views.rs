//! Native output acceptance compares an upstream oracle and executes authoring, transfer, and recovery.

mod authoring_fixture;
use authoring_fixture::*;
use programmable_cli::cli_assembly::assemble_cli_document;
use programmable_cli::cli_definition::CliDefinition;
use programmable_cli::cli_output_view::CliOutputView;
use programmable_cli::cli_run_completion::CliRunCompleter;
use programmable_cli::cli_run_routes::CliRunRoutes;
use programmable_cli::document_value::DocumentValue;
use reedline::Completer;
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::process::{Command, Output, Stdio};

const ACCEPTANCE: &str = "docs/output-views/acceptance";

fn output_spec() -> CliDefinition {
    CliDefinition::parse_cli_definition(
        &fs::read(repository_path(&format!("{ACCEPTANCE}/agp.json"))).unwrap(),
    )
    .unwrap()
}
fn precise_value(text: &str) -> DocumentValue {
    DocumentValue::parse_document(text).unwrap()
}
fn input_fixture(name: &str) -> DocumentValue {
    precise_value(
        &fs::read_to_string(repository_path(&format!(
            "{ACCEPTANCE}/fixtures/{name}.json"
        )))
        .unwrap(),
    )
}
fn output_fixture() -> AuthoringFixture {
    let fixture = AuthoringFixture::new();
    fs::copy(
        repository_path(&format!("{ACCEPTANCE}/agp.json")),
        fixture.directory.join("agp.json"),
    )
    .unwrap();
    fs::create_dir(fixture.directory.join("empty-path")).unwrap();
    fixture
}
fn native_cli(fixture: &AuthoringFixture, args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(&fixture.directory)
        .env("PATH", fixture.directory.join("empty-path"))
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
fn expect_exit(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn write_json(fixture: &AuthoringFixture, file: &str, value: &Value) {
    fs::write(
        fixture.directory.join(file),
        serde_json::to_vec(value).unwrap(),
    )
    .unwrap();
}
fn plain_view() -> Value {
    json!({"help":"Inspect supplied rows.","require":{"op":"literal","value":true},"rows":{"op":"root","path":[]},"columns":[{"id":"value","heading":"VALUE","value":{"op":"row","path":[]},"format":{"kind":"plain"}}]})
}

#[test]
fn native_tables_and_display_rows_match_frozen_agp_oracles() {
    let spec = output_spec();
    for name in [
        "connections-cases",
        "connections-empty",
        "routes-cases",
        "routes-empty",
    ] {
        let kind = name.split('-').next().unwrap();
        let result = spec.views.as_ref().unwrap()[kind]
            .present_output_document(kind, &input_fixture(name))
            .unwrap();
        assert_eq!(
            result.table_text,
            fs::read_to_string(repository_path(&format!(
                "{ACCEPTANCE}/expected/{name}.txt"
            )))
            .unwrap(),
            "{name}"
        );
        let rows: Vec<std::collections::BTreeMap<String, String>> = result
            .display_rows
            .iter()
            .map(|row| {
                result
                    .columns
                    .iter()
                    .cloned()
                    .zip(row.iter().cloned())
                    .collect()
            })
            .collect();
        let expected: Value = serde_json::from_slice(
            &fs::read(repository_path(&format!(
                "{ACCEPTANCE}/expected/{name}.rows.json"
            )))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(serde_json::to_value(rows).unwrap(), expected, "{name}");
    }
    let routes = spec.views.as_ref().unwrap()["routes"]
        .present_output_document("routes", &input_fixture("routes-cases"))
        .unwrap();
    let typed: Value = serde_json::from_str(&routes.rows_json_text).unwrap();
    assert_eq!(typed[1]["eligible"], false);
    assert!(typed[1]["path"].is_array());
}

#[test]
fn supplied_durations_and_optional_fields_match_connection_semantics() {
    let spec = output_spec();
    let view = &spec.views.as_ref().unwrap()["connections"];
    let pending = precise_value(
        r#"{"apiVersion":"agp.management/v1","kind":"ConnectionList","items":[{"localSessionId":"pending-1","state":"Active","direction":"outbound","lastTransition":{"event":"RetryExpired"},"timers":[]}]}"#,
    );
    let output = view
        .present_output_document("connections", &pending)
        .unwrap();
    assert_eq!(
        output.display_rows[0],
        [
            "pending-1",
            "-",
            "outbound",
            "Active",
            "-",
            "-",
            "RetryExpired"
        ]
    );
    for (remaining, expected) in [
        ("30000", "30s"),
        ("29000", "29s"),
        ("21000.0001", "22s"),
        ("1e-400", "1s"),
        ("-5", "0s"),
        ("0", "0s"),
    ] {
        let input = format!(
            r#"{{"apiVersion":"agp.management/v1","kind":"ConnectionList","items":[{{"sessionId":"s","state":"Established","establishedDurationMs":176523000,"timers":[{{"name":"hold","state":"disarmed","remainingMs":999}},{{"name":"hold","state":"armed","remainingMs":{remaining}}}]}}]}}"#
        );
        let output = view
            .present_output_document("connections", &precise_value(&input))
            .unwrap();
        assert_eq!(output.display_rows[0][4], "49:02:03");
        assert_eq!(output.display_rows[0][5], expected);
    }
}

#[test]
fn projections_preserve_exact_numbers_nulls_missing_values_and_unicode_width() {
    let view: CliOutputView = serde_json::from_value(plain_view()).unwrap();
    let data = precise_value("[9007199254740993,1.2300,1e400,false,null]");
    let output = view.present_output_document("exact", &data).unwrap();
    assert_eq!(
        output.rows_json_text,
        "[{\"value\":9007199254740993},{\"value\":1.2300},{\"value\":1e400},{\"value\":false},{\"value\":null}]"
    );
    assert_eq!(output.display_rows[0][0], "9007199254740993");
    assert_eq!(
        data.compact_document_json(),
        "[9007199254740993,1.2300,1e400,false,null]"
    );
    let mut object_view = plain_view();
    object_view["columns"][0]["value"]["path"] = json!([{"key":"value"}]);
    let view: CliOutputView = serde_json::from_value(object_view).unwrap();
    let output = view
        .present_output_document("missing", &precise_value("[{}, {\"value\":null}]"))
        .unwrap();
    assert_eq!(output.rows_json_text, "[{},{\"value\":null}]");
    let unicode: CliOutputView = serde_json::from_value(json!({"help":"Display widths.","require":{"op":"literal","value":true},"rows":{"op":"root","path":[]},"columns":[{"id":"name","heading":"N","value":{"op":"row","path":[{"key":"name"}]},"format":{"kind":"plain"}},{"id":"value","heading":"V","value":{"op":"row","path":[{"key":"value"}]},"format":{"kind":"plain"}}]})).unwrap();
    let output = unicode.present_output_document("width", &precise_value(r#"[{"name":"界","value":"x"},{"name":"é","value":"y"},{"name":"a\nb","value":"z\u001b[2J\u202e"}]"#)).unwrap();
    assert_eq!(output.table_text, "N    V\n界   x\né    y\na b  z [2J \n");
    assert!(!output.table_text.contains(['\u{1b}', '\u{202e}']));
}

#[test]
fn contextual_recipe_exports_imports_and_runs_both_views_with_empty_path() {
    let author = output_fixture();
    fs::copy(
        repository_path(&format!("{ACCEPTANCE}/TASK.md")),
        &author.intent,
    )
    .unwrap();
    let mut command = author.command(true, true);
    command
        .arg("--compose")
        .env("PATH", author.directory.join("empty-path"));
    let mut process = AuthoringProcess::spawn(command);
    let recipe =
        fs::read_to_string(repository_path(&format!("{ACCEPTANCE}/agp.commands"))).unwrap();
    // The recipe creates the exported file itself, so remove only the fixture's initial copy.
    fs::remove_file(author.directory.join("agp.json")).unwrap();
    for (index, line) in recipe.lines().enumerate() {
        assert!(
            !line.contains(['{', '}', '[', ']']),
            "Serialized structure in recipe line {}",
            index + 1
        );
        let response = process.terminal(line);
        assert_eq!(
            response["status"],
            "ok",
            "line {}: {line}: {response}",
            index + 1
        );
    }
    assert!(process.close().success());
    assert_eq!(
        fs::read(author.directory.join("agp.json")).unwrap(),
        fs::read(repository_path(&format!("{ACCEPTANCE}/agp.json"))).unwrap()
    );
    let receiver = output_fixture();
    fs::copy(
        author.directory.join("agp-interface.json"),
        receiver.directory.join("portable.json"),
    )
    .unwrap();
    let mut command = receiver.command(true, true);
    command.arg("--compose");
    let mut process = AuthoringProcess::spawn(command);
    for line in [
        "import agp.json",
        "commit",
        "save transferred.json",
        "activate",
        "export-interface received.json",
    ] {
        let response = process.terminal(line);
        assert_eq!(response["status"], "ok", "{line}: {response}");
    }
    assert!(process.close().success());
    for kind in ["connections", "routes"] {
        let input = repository_path(&format!("{ACCEPTANCE}/fixtures/{kind}-cases.json"));
        let expected = fs::read(repository_path(&format!(
            "{ACCEPTANCE}/expected/{kind}-cases.txt"
        )))
        .unwrap();
        for source in ["transferred.json", "portable.json", "received.json"] {
            let output = native_cli(
                &receiver,
                &[
                    "run",
                    "--table",
                    "--grant-json-read",
                    &format!("agp.{kind}"),
                    input.to_str().unwrap(),
                    source,
                    &format!("{kind}.list"),
                ],
                "",
            );
            expect_exit(&output, 0);
            assert_eq!(output.stdout, expected);
            assert!(output.stderr.is_empty());
        }
        let output = native_cli(
            &receiver,
            &["render", "agp.json", kind, input.to_str().unwrap()],
            "",
        );
        expect_exit(&output, 0);
        assert_eq!(output.stdout, expected);
        assert!(String::from_utf8_lossy(&output.stderr).contains("[preview]"));
    }
}

#[test]
fn saved_result_renders_without_sources_grants_or_new_revision() {
    let fixture = output_fixture();
    fs::copy(
        repository_path(&format!("{ACCEPTANCE}/fixtures/connections-cases.json")),
        fixture.directory.join("input.json"),
    )
    .unwrap();
    let output = native_cli(
        &fixture,
        &[
            "run",
            "--json",
            "--table",
            "--session",
            "run.json",
            "--grant-json-read",
            "agp.connections",
            "input.json",
            "agp.json",
            "connections.list",
        ],
        "",
    );
    expect_exit(&output, 0);
    let event: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(event["status"], "ok");
    assert_eq!(event["result"]["invocation"]["binding"], "connected");
    let before = fs::read(fixture.directory.join("run.json")).unwrap();
    let checkpoint: Value = serde_json::from_slice(&before).unwrap();
    assert!(
        checkpoint["active_interface"]["last_invocation"]
            .get("presentation")
            .is_none()
    );
    fs::remove_file(fixture.directory.join("agp.json")).unwrap();
    fs::remove_file(fixture.directory.join("input.json")).unwrap();
    let output = native_cli(
        &fixture,
        &[
            "run",
            "--json",
            "--session",
            "run.json",
            "-",
            ":render",
            "connections",
        ],
        "",
    );
    expect_exit(&output, 0);
    let rendered: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        rendered["result"]["invocation"],
        event["result"]["invocation"]
    );
    assert_eq!(
        rendered["result"]["presentation"],
        event["result"]["presentation"]
    );
    assert_eq!(rendered["result"]["historical"], true);
    assert_eq!(rendered["mutation"], "none");
    assert_eq!(
        fs::read(fixture.directory.join("run.json")).unwrap(),
        before
    );
    let output = native_cli(
        &fixture,
        &[
            "run",
            "--session",
            "run.json",
            "-",
            ":render",
            "connections",
        ],
        "",
    );
    expect_exit(&output, 0);
    assert!(String::from_utf8_lossy(&output.stderr).contains("[historical]"));
    assert_eq!(
        fs::read(fixture.directory.join("run.json")).unwrap(),
        before
    );
}

#[test]
fn failed_presentation_keeps_successful_effect_and_supports_safe_rerender() {
    let fixture = output_fixture();
    let mut spec = serde_json::to_value(output_spec()).unwrap();
    spec["contexts"]["root"]["commands"]["connections.list"]["binding"] = json!({"kind":"simulated","steps":[{"path":[{"key":"changed"}],"value":{"source":"literal","value":true}}],"output":{"source":"literal","value":[{"unexpected":9007199254740993u64}]}});
    spec["views"]["raw"] = plain_view();
    write_json(&fixture, "agp.json", &spec);
    let output = native_cli(
        &fixture,
        &[
            "run",
            "--json",
            "--table",
            "--session",
            "run.json",
            "agp.json",
            "connections.list",
        ],
        "",
    );
    expect_exit(&output, 1);
    let event: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(event["status"], "ok");
    assert_eq!(event["mutation"], "applied");
    assert_eq!(
        event["result"]["presentation"]["error"]["code"],
        "OUTPUT_VIEW_INPUT"
    );
    assert_eq!(event["result"]["invocation"]["effect"], "simulation_only");
    let before = fs::read(fixture.directory.join("run.json")).unwrap();
    let state: Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(state["active_interface"]["mock_state"]["changed"], true);
    let output = native_cli(
        &fixture,
        &["run", "--session", "run.json", "-", ":render", "raw"],
        "",
    );
    expect_exit(&output, 0);
    assert!(String::from_utf8_lossy(&output.stdout).contains("9007199254740993"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("[simulated]"));
    assert_eq!(
        fs::read(fixture.directory.join("run.json")).unwrap(),
        before
    );
    let output = native_cli(
        &fixture,
        &[
            "run",
            "--session",
            "run.json",
            "--view",
            "missing",
            "-",
            "connections.list",
        ],
        "",
    );
    expect_exit(&output, 2);
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read(fixture.directory.join("run.json")).unwrap(),
        before
    );
    let output = native_cli(
        &fixture,
        &[
            "run",
            "--session",
            "run.json",
            "-",
            ":render",
            "connections",
        ],
        "",
    );
    expect_exit(&output, 1);
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invocation outcome is retained"));
    assert_eq!(
        fs::read(fixture.directory.join("run.json")).unwrap(),
        before
    );
}

#[test]
fn invalid_view_definitions_reject_before_creating_run_state() {
    let fixture = output_fixture();
    let original = serde_json::to_value(output_spec()).unwrap();
    for (path, replacement) in [
        ("/views/connections/require/op", json!("exec")),
        ("/views/connections/rows/op", json!("row")),
        ("/views/connections/columns/2/value/op", json!("item")),
        (
            "/views/connections/columns/2/value/path",
            json!([{"key":"x","index":0}]),
        ),
        ("/views/connections/columns/2/id", json!("session_id")),
        (
            "/views/connections/columns/2/format",
            json!({"kind":"plain","run":"anything"}),
        ),
        (
            "/contexts/root/commands/connections.list/view",
            json!("missing"),
        ),
        ("/views", Value::Null),
        ("/format", json!("cli-definition-v1")),
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(path).unwrap() = replacement;
        assert_ne!(changed, original, "INVALID: mutant did not land: {path}");
        write_json(&fixture, "invalid.json", &changed);
        let output = native_cli(
            &fixture,
            &[
                "run",
                "--session",
                "invalid-session.json",
                "invalid.json",
                ":views",
            ],
            "",
        );
        assert_eq!(
            output.status.code(),
            Some(1),
            "Accepted invalid field {path}"
        );
        expect_exit(&output, 1);
        assert!(output.stdout.is_empty());
        assert!(
            !fixture.directory.join("invalid-session.json").exists(),
            "{path}"
        );
    }
    let mut changed = original;
    let mut expression = json!({"op":"literal","value":true});
    for _ in 0..18 {
        expression = json!({"op":"and","values":[expression]});
    }
    changed["views"]["connections"]["require"] = expression;
    assert!(CliDefinition::parse_cli_definition(&serde_json::to_vec(&changed).unwrap()).is_err());
}

#[test]
fn input_work_row_padding_and_decimal_bounds_fail_explicitly() {
    let spec = output_spec();
    let view = &spec.views.as_ref().unwrap()["connections"];
    for input in [
        r#"{"apiVersion":"agp.management/v2","kind":"ConnectionList","items":[]}"#,
        r#"{"apiVersion":"agp.management/v1","kind":"ConnectionList","items":{}}"#,
    ] {
        assert_eq!(
            view.present_output_document("connections", &precise_value(input))
                .unwrap_err()
                .code,
            "OUTPUT_VIEW_INPUT"
        );
    }
    let view: CliOutputView = serde_json::from_value(plain_view()).unwrap();
    assert_eq!(
        view.present_output_document(
            "rows",
            &DocumentValue::Array(vec![DocumentValue::Null; 4097])
        )
        .unwrap_err()
        .code,
        "OUTPUT_VIEW_LIMIT"
    );
    let mut data = vec![DocumentValue::String(String::new()); 100];
    data[0] = DocumentValue::String("x".repeat(50_000));
    let mut padded = plain_view();
    padded["columns"].as_array_mut().unwrap().push(json!({
        "id":"tail", "heading":"TAIL", "value":{"op":"literal","value":""},
        "format":{"kind":"plain"}
    }));
    let padded: CliOutputView = serde_json::from_value(padded).unwrap();
    assert_eq!(
        padded
            .present_output_document("padding", &DocumentValue::Array(data))
            .unwrap_err()
            .code,
        "OUTPUT_VIEW_LIMIT"
    );
    let mut timed = plain_view();
    timed["columns"][0]["format"] = json!({"kind":"duration_hms","fallback":"-"});
    let timed: CliOutputView = serde_json::from_value(timed).unwrap();
    assert_eq!(
        timed
            .present_output_document("digits", &precise_value("[1e5000]"))
            .unwrap_err()
            .code,
        "OUTPUT_VIEW_LIMIT"
    );
    assert_eq!(
        timed
            .present_output_document("negative", &precise_value("[-1]"))
            .unwrap_err()
            .code,
        "OUTPUT_VIEW_INPUT"
    );
    let equivalent = timed
        .present_output_document("exact", &precise_value("[1e4098,0.00001e4103]"))
        .unwrap();
    assert_eq!(equivalent.display_rows[0], equivalent.display_rows[1]);
    let mut ceiling = plain_view();
    ceiling["columns"][0]["format"] = json!({"kind":"seconds_ceil","fallback":"-"});
    let ceiling: CliOutputView = serde_json::from_value(ceiling).unwrap();
    let carry = precise_value(&format!("[{}000.1]", "9".repeat(4096)));
    let carry_error = ceiling
        .present_output_document("carry", &carry)
        .unwrap_err();
    assert_eq!(carry_error.code, "OUTPUT_VIEW_LIMIT");
    assert!(carry_error.message.contains("row 0, column value"));
    let mut expensive = plain_view();
    expensive["columns"][0]["value"] =
        json!({"op":"any","input":{"op":"root","path":[]},"where":{"op":"literal","value":false}});
    let expensive: CliOutputView = serde_json::from_value(expensive).unwrap();
    assert_eq!(
        expensive
            .present_output_document(
                "work",
                &DocumentValue::Array(vec![DocumentValue::Null; 400])
            )
            .unwrap_err()
            .code,
        "OUTPUT_VIEW_LIMIT"
    );
}

#[test]
fn component_assembly_scopes_view_names_and_keeps_result_paths() {
    let fixture = output_fixture();
    let spec = output_spec();
    write_json(
        &fixture,
        "component.json",
        &json!({"format":"cli-component-v1","requires":[],"definition":spec}),
    );
    let manifest = json!({"format":"cli-assembly-v1","id":"assembled-views","description":"Assemble a view consumer.","components":[{"mount":"network","source":fixture.directory.join("component.json")}]});
    let assembled = assemble_cli_document(&precise_value(&manifest.to_string())).unwrap();
    let assembled_spec =
        CliDefinition::parse_cli_definition(assembled.compact_document_json().as_bytes()).unwrap();
    assert_eq!(assembled_spec.format, "cli-definition-v3");
    assert_eq!(
        assembled_spec.contexts["network"].commands["connections.list"]
            .view
            .as_deref(),
        Some("network.connections")
    );
    assert_eq!(
        assembled_spec.views.as_ref().unwrap()["network.connections"],
        spec.views.as_ref().unwrap()["connections"]
    );
    fs::write(
        fixture.directory.join("assembled.json"),
        assembled.compact_document_json(),
    )
    .unwrap();
    let input = repository_path(&format!("{ACCEPTANCE}/fixtures/connections-cases.json"));
    let output = native_cli(
        &fixture,
        &[
            "run",
            "--table",
            "--grant-json-read",
            "agp.connections",
            input.to_str().unwrap(),
            "assembled.json",
            "network",
            "connections.list",
        ],
        "",
    );
    expect_exit(&output, 0);
    assert_eq!(
        output.stdout,
        fs::read(repository_path(&format!(
            "{ACCEPTANCE}/expected/connections-cases.txt"
        )))
        .unwrap()
    );
}

#[test]
fn legacy_output_and_discovery_remain_explicit_and_completion_lists_views() {
    let legacy = repository_path("docs/connected/acceptance/expected-definition.json");
    let bytes = fs::read(&legacy).unwrap();
    let spec = CliDefinition::parse_cli_definition(&bytes).unwrap();
    assert_eq!(
        precise_value(std::str::from_utf8(&bytes).unwrap()),
        precise_value(&serde_json::to_string(&spec).unwrap())
    );
    let fixture = output_fixture();
    let output = native_cli(
        &fixture,
        &[
            "run",
            legacy.to_str().unwrap(),
            "services",
            "quota",
            "1e400",
        ],
        "",
    );
    expect_exit(&output, 0);
    assert_eq!(output.stdout, b"{\"quota\":1e400}\n");
    let output = native_cli(
        &fixture,
        &[
            "run",
            "--table",
            legacy.to_str().unwrap(),
            "services",
            "quota",
            "1",
        ],
        "",
    );
    expect_exit(&output, 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("OUTPUT_VIEW_REQUIRED"));
    let output = native_cli(&fixture, &["run", "--json", "agp.json", ":views"], "");
    expect_exit(&output, 0);
    let views: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(views["result"]["views"].as_object().unwrap().len(), 2);
    let output = native_cli(
        &fixture,
        &["run", "--json", "agp.json", "connections.list", "--help"],
        "",
    );
    expect_exit(&output, 0);
    let help: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(help["result"]["commands"][0]["view"], "connections");
    use programmable_cli::authoring_protocol::SessionRevision;
    use programmable_cli::cli_interface::{ActiveCliInterface, CliInterfaceOrigin};
    let active = ActiveCliInterface::initialize_cli_interface(
        output_spec(),
        CliInterfaceOrigin {
            intent_text: "View completion test.".into(),
            revision: SessionRevision(0),
        },
    );
    let routes = CliRunRoutes::from_cli_definition(&active.definition).unwrap();
    let mut completer = CliRunCompleter::new_run_completer(active, routes);
    let result = completer.complete(":render conn", 12);
    assert_eq!(result.suggestions()[0].value, "connections");
}

#[cfg(unix)]
#[test]
fn table_delivery_failure_preserves_receipt_for_later_rendering() {
    let fixture = output_fixture();
    let input = repository_path(&format!("{ACCEPTANCE}/fixtures/routes-cases.json"));
    let output = Command::new(env!("CARGO_BIN_EXE_cli"))
        .current_dir(&fixture.directory)
        .args([
            "run",
            "--table",
            "--session",
            "run.json",
            "--grant-json-read",
            "agp.routes",
            input.to_str().unwrap(),
            "agp.json",
            "routes.list",
        ])
        .stdout(
            fs::OpenOptions::new()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        )
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    expect_exit(&output, 1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("Run transport stopped"));
    let before = fs::read(fixture.directory.join("run.json")).unwrap();
    let output = native_cli(
        &fixture,
        &["run", "--session", "run.json", "-", ":render", "routes"],
        "",
    );
    expect_exit(&output, 0);
    assert_eq!(
        output.stdout,
        fs::read(repository_path(&format!(
            "{ACCEPTANCE}/expected/routes-cases.txt"
        )))
        .unwrap()
    );
    assert_eq!(
        fs::read(fixture.directory.join("run.json")).unwrap(),
        before
    );
}
