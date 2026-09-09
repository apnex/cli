mod authoring_fixture;

use authoring_fixture::*;
use programmable_cli::authoring_frontend::render_authoring_event;
use serde_json::json;
use std::fs;

#[test]
fn constructed_cli_prints_a_derived_verb_tree_without_mutating_the_session() {
    let fixture = AuthoringFixture::new();
    fs::copy(
        repository_path("docs/composition/examples/platform/TASK.md"),
        &fixture.intent,
    )
    .unwrap();
    let mut command = fixture.command(true, true);
    command.arg("--compose");
    let mut process = AuthoringProcess::spawn(command);
    assert_eq!(process.header["event"], "session_open");
    let before = fixture.checkpoint_bytes();
    assert_eq!(
        process.terminal("tree")["error"]["code"],
        "NO_ACTIVE_INTERFACE"
    );
    assert_eq!(fixture.checkpoint_bytes(), before);
    let trace = fs::read_to_string(repository_path(
        "docs/composition/examples/platform/platform.commands",
    ))
    .unwrap();
    for line in trace.lines() {
        assert!(!line.contains(['{', '}', '[', ']']));
        let response = process.terminal(line);
        assert_eq!(response["status"], "ok", "{line}: {response}");
        if line != "activate" {
            assert!(response["session"]["active_interface"].is_null());
        }
    }

    let expected = fs::read_to_string(repository_path(
        "docs/composition/examples/platform/expected-verb-tree.txt",
    ))
    .unwrap();
    let before = fixture.checkpoint_bytes();
    let discovered = process.terminal("tree");
    assert_eq!(discovered["status"], "ok");
    assert_eq!(discovered["result"]["verb_tree_text"], expected);
    assert_eq!(discovered["mutation"], "none");
    assert_eq!(fixture.checkpoint_bytes(), before);
    let readable = render_authoring_event(&discovered);
    assert_eq!(readable, expected);
    let details = process.terminal("discover");
    assert!(details["result"].get("verb_tree_text").is_none());
    assert_eq!(
        details["result"]["context"]["commands"]["version"]["id"],
        "platform.version"
    );
    assert!(render_authoring_event(&details).contains("mock_state_json_text"));
    assert_eq!(fixture.checkpoint_bytes(), before);

    assert_eq!(process.terminal("enter releases")["status"], "ok");
    let nested = process.terminal("tree");
    assert_eq!(nested["result"]["verb_tree_text"], expected);
    assert_eq!(nested["result"]["interface"]["context"], "releases");
    assert_eq!(
        process.terminal("discover")["result"]["context"]["commands"]["rollback"]["id"],
        "releases.rollback"
    );

    for line in [
        "delete /contexts/services/commands/restart",
        "set /contexts/releases/parent string missing",
    ] {
        assert_eq!(process.terminal(line)["status"], "ok");
    }
    assert_eq!(process.terminal("activate")["status"], "error");
    assert_eq!(
        process.terminal("tree")["result"]["verb_tree_text"],
        expected
    );
    for line in [
        "set /contexts/releases/parent string deployments",
        "set /contexts/releases/commands/rollback/parameters/0/type string boolean",
        "append /contexts/deployments/related string services",
        "set /contexts/empty object",
        "set /contexts/empty/parent string releases",
        "set /contexts/empty/help string Empty",
        "set /contexts/empty/related array",
        "set /contexts/empty/commands object",
        "set /contexts/deployments/commands/releases object",
        "set /contexts/deployments/commands/releases/id string deployments.releases",
        "set /contexts/deployments/commands/releases/help string Placeholder",
        "set /contexts/deployments/commands/releases/parameters array",
        "set /contexts/deployments/commands/releases/binding object",
        "set /contexts/deployments/commands/releases/binding/kind string unbound",
        "set /contexts/deployments/commands/releases/binding/reason string Unimplemented",
    ] {
        assert_eq!(process.terminal(line)["status"], "ok", "{line}");
    }
    assert_eq!(
        process.terminal("tree")["result"]["verb_tree_text"],
        expected
    );
    assert_eq!(process.terminal("activate")["status"], "ok");
    let updated = process.terminal("tree");
    let tree = updated["result"]["verb_tree_text"].as_str().unwrap();
    assert!(!tree.contains("invoke restart"));
    assert!(tree.contains("invoke rollback <version:boolean> [unbound]"));
    assert!(tree.contains("|   |-- invoke releases [unbound]"));
    assert!(tree.contains("|   `-- releases/"));
    assert!(tree.contains("|       `-- empty/"));
    assert_eq!(tree.matches("services/").count(), 1);
    assert!(process.close().success());

    let before = fixture.checkpoint_bytes();
    let mut command = fixture.command(false, false);
    command.arg("--compose");
    let mut machine = AuthoringProcess::spawn(command);
    let request = json!({
        "request_id": "verb-tree-machine",
        "session_id": machine.header["session"]["session_id"],
        "operation": "tree",
        "arguments": {}
    });
    let discovered = machine.submit(&request);
    assert_eq!(discovered["status"], "ok");
    assert_eq!(discovered["result"], updated["result"]);
    assert_eq!(fixture.checkpoint_bytes(), before);
    assert!(machine.close().success());
}
