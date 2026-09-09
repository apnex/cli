mod authoring_fixture;
use authoring_fixture::*;
use programmable_cli::authoring_frontend::render_authoring_event;
use programmable_cli::terminal_completion::AuthoringCompleter;
use programmable_cli::terminal_input::{compile_terminal_request, tokenize_terminal_input};
use reedline::Completer;
use serde_json::{Value, json};
use std::fs;

#[test]
fn changed_declared_words_and_positions_drive_help_completion_and_dispatch() {
    let fixture = AuthoringFixture::new();
    let mut declaration: Value =
        serde_json::from_slice(&fs::read(&fixture.declaration).unwrap()).unwrap();
    let set = declaration["operations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|operation| operation["name"] == "set")
        .unwrap();
    set["name"] = json!("put");
    set["terminal"]["command"] = json!("write");
    set["terminal"]["positional"] = json!(["value", "path"]);
    fs::write(
        &fixture.declaration,
        serde_json::to_vec(&declaration).unwrap(),
    )
    .unwrap();
    let mut process = fixture.start(true, true);
    assert_eq!(
        process.header["event"], "session_open",
        "{}",
        process.header
    );
    let response = process.terminal("write number 1.2300 /quota");
    assert_eq!(response["status"], "ok", "{response}");
    assert_eq!(response["operation"], "put");
    assert_eq!(
        fixture.checkpoint_state().candidate.compact_document_json(),
        "{\"quota\":1.2300}"
    );
    let help = process.terminal("help put");
    assert_eq!(
        help["result"]["operations"][0]["terminal"]["command"],
        "write"
    );
    let definition =
        programmable_cli::operation_definition::OperationDefinition::load_operation_definition(
            &fixture.declaration,
        )
        .unwrap();
    let mut completer =
        AuthoringCompleter::new_authoring_completer(definition, fixture.checkpoint_state());
    assert_eq!(completer.complete("wr", 2).suggestions()[0].value, "write");
    assert!(completer.complete("se", 2).suggestions().is_empty());
    let before = fixture.checkpoint_bytes();
    check_rejection(
        &fixture,
        &before,
        &process.terminal("set /quota null"),
        "UNKNOWN_OPERATION",
    );
    assert!(process.close().success());
}

#[test]
fn completion_escapes_arbitrary_keys_and_uses_current_context_and_constructor_metadata() {
    let fixture = AuthoringFixture::new();
    fixture.seed(&json!({"candidate_json":r#"{"folder":{"":null,"a/b":1,"~name":true,"release note":"text","line\nkey":[],"é":{}}}"#, "context":[{"key":"folder"}]}));
    let state = fixture.checkpoint_state();
    let definition = fixture_definition();
    let mut completer =
        AuthoringCompleter::new_authoring_completer(definition.clone(), state.clone());
    let result = completer.complete("show ./", 7);
    let values: Vec<_> = result
        .suggestions()
        .iter()
        .map(|suggestion| suggestion.value.clone())
        .collect();
    for expected in [
        "./",
        "./a~1b",
        "./~0name",
        "\"./release note\"",
        "\"./line\\nkey\"",
        "./é",
    ] {
        assert!(
            values.contains(&expected.to_owned()),
            "Missing {expected}: {values:?}"
        );
    }
    for suggestion in result.suggestions() {
        let line = format!("show {}", suggestion.value);
        let request = compile_terminal_request(&line, &definition, &state).unwrap();
        assert_eq!(request.operation, "show");
        assert!(
            suggestion
                .value
                .chars()
                .all(|character| !character.is_control())
        );
    }
    let partial = "show \"./release";
    assert_eq!(
        completer.complete(partial, partial.len()).suggestions()[0].value,
        "\"./release note\""
    );
    let kinds = completer.complete("set /new ", 9);
    assert_eq!(kinds.suggestions().len(), 6);
    let booleans = completer.complete("set /new boolean ", 17);
    assert_eq!(
        booleans
            .suggestions()
            .iter()
            .map(|suggestion| suggestion.value.as_str())
            .collect::<Vec<_>>(),
        ["true", "false"]
    );
    let mut root_state = state.clone();
    root_state.context.clear();
    completer.refresh_completion_context(&root_state);
    assert_eq!(
        completer
            .complete("show ./", 7)
            .suggestions()
            .iter()
            .map(|suggestion| suggestion.value.as_str())
            .collect::<Vec<_>>(),
        ["./folder"]
    );
    assert!(tokenize_terminal_input("set /x string \"unfinished").is_err());
    assert_eq!(
        tokenize_terminal_input("set /x string $(literal)").unwrap()[3].text,
        "$(literal)"
    );
}

#[test]
fn readable_terminal_errors_include_path_mutation_and_recovery() {
    let fixture = AuthoringFixture::new();
    fixture.seed(&json!({"candidate_json":"[1]"}));
    let mut process = fixture.start(false, false);
    let response=process.request(&json!({"operation":"set","arguments":{"path":{"base":"root","segments":[{"index":9}]},"value":{"kind":"null"}}}));
    let rendered = render_authoring_event(&response);
    for field in [
        "INDEX_OUT_OF_BOUNDS",
        "mutation",
        "none",
        "path",
        "index",
        "9",
        "recovery",
    ] {
        assert!(
            rendered.contains(field),
            "Readable error omitted {field}: {rendered}"
        );
    }
    assert!(process.close().success());
}

#[test]
fn completion_reports_partial_menus_and_follows_unsent_batch_preview() {
    let fixture = AuthoringFixture::new();
    let object: serde_json::Map<String, Value> = (0..101)
        .map(|index| (format!("key{index:03}"), Value::Null))
        .collect();
    fixture.seed(&json!({"candidate_json":Value::Object(object).to_string()}));
    let definition = fixture_definition();
    let state = fixture.checkpoint_state();
    let mut completer =
        AuthoringCompleter::new_authoring_completer(definition.clone(), state.clone());
    let partial = completer.complete("show ./", 7);
    assert_eq!(partial.suggestions().len(), 101);
    assert!(
        partial
            .suggestions()
            .last()
            .unwrap()
            .description
            .as_ref()
            .unwrap()
            .contains("More matching paths")
    );
    let focused = completer.complete("show ./key100", 13);
    assert_eq!(focused.suggestions().len(), 1);
    assert_eq!(focused.suggestions()[0].value, "./key100");
    let mut input = programmable_cli::terminal_input::TerminalInputBuffer::default();
    input.accept_terminal_line("batch", &definition, &state);
    input.accept_terminal_line("set /created object", &definition, &state);
    completer.refresh_completion_context(input.completion_checkpoint(&state));
    completer.set_batch_completion(true);
    assert_eq!(
        completer.complete("set /cre", 8).suggestions()[0].value,
        "/created"
    );
    assert_eq!(completer.complete("en", 2).suggestions()[0].value, "end");
    assert!(completer.complete("comm", 4).suggestions().is_empty());
    assert_eq!(fixture.checkpoint_state(), state);
}

#[cfg(feature = "contract-probes")]
#[test]
fn separately_built_native_registry_drift_is_rejected_before_session_creation() {
    for (binary, change) in [
        (env!("CARGO_BIN_EXE_probe-missing-handler"), "missing"),
        (env!("CARGO_BIN_EXE_probe-extra-handler"), "extra"),
        (env!("CARGO_BIN_EXE_probe-changed-argument"), "argument"),
    ] {
        let fixture = AuthoringFixture::new();
        let output = std::process::Command::new(binary)
            .arg(&fixture.declaration)
            .arg(&fixture.checkpoint)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("LANDED {change}\n"),
            "INVALID: actual registration mutation did not land"
        );
        assert_eq!(output.stdout, b"DEFINITION_MISMATCH\n");
        assert!(!fixture.checkpoint.exists());
    }
    report_pass("declaration-handler-drift");
}
