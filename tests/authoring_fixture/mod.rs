//! Application fixtures launch real processes and compare complete acknowledged checkpoint state.

#![allow(dead_code)]

use programmable_cli::authoring_protocol::SessionCheckpoint;
use programmable_cli::document_value::{DocumentValue, MAX_RESPONSE_BYTES};
use programmable_cli::operation_definition::OperationDefinition;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

pub fn repository_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}
pub fn fixture_corpus() -> Value {
    serde_json::from_slice(
        &fs::read(repository_path(
            "docs/authoring/acceptance/authoring-cases.json",
        ))
        .unwrap(),
    )
    .unwrap()
}
pub fn fixture_definition() -> OperationDefinition {
    OperationDefinition::load_operation_definition(&repository_path(
        "docs/authoring/operations.json",
    ))
    .unwrap()
}

pub struct AuthoringFixture {
    pub directory: PathBuf,
    pub checkpoint: PathBuf,
    pub declaration: PathBuf,
    pub intent: PathBuf,
}

impl AuthoringFixture {
    pub fn new() -> Self {
        let directory =
            std::env::temp_dir().join(format!("cli-acceptance-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&directory).unwrap();
        let checkpoint = directory.join("session.json");
        let declaration = directory.join("operations.json");
        let intent = directory.join("task.txt");
        fs::copy(
            repository_path("docs/authoring/operations.json"),
            &declaration,
        )
        .unwrap();
        fs::copy(
            repository_path("docs/authoring/acceptance/SERVICE-CATALOG.md"),
            &intent,
        )
        .unwrap();
        Self {
            directory,
            checkpoint,
            declaration,
            intent,
        }
    }

    pub fn seed(&self, initial: &Value) {
        let definition = fixture_definition();
        let mut state = SessionCheckpoint::new_authoring_session(
            fs::read_to_string(&self.intent).unwrap(),
            definition.definition_id.clone(),
            definition.sha256.clone(),
        )
        .unwrap();
        state.candidate =
            DocumentValue::parse_document(initial["candidate_json"].as_str().unwrap_or("{}"))
                .unwrap();
        state.accepted =
            DocumentValue::parse_document(initial["accepted_json"].as_str().unwrap_or("{}"))
                .unwrap();
        if let Some(context) = initial.get("context") {
            state.context = serde_json::from_value(context.clone()).unwrap();
        }
        if let Some(revision) = initial.get("revision") {
            state.revision = serde_json::from_value(revision.clone()).unwrap();
        }
        if let Some(revision) = initial.get("accepted_revision") {
            state.accepted_revision = serde_json::from_value(revision.clone()).unwrap();
        }
        definition.validate_checkpoint_receipt(&state).unwrap();
        self.write_checkpoint(&state);
    }

    pub fn write_checkpoint(&self, state: &SessionCheckpoint) {
        let mut bytes = serde_json::to_vec(state).unwrap();
        bytes.push(b'\n');
        fs::write(&self.checkpoint, bytes).unwrap();
    }
    pub fn checkpoint_bytes(&self) -> Vec<u8> {
        fs::read(&self.checkpoint).unwrap()
    }
    pub fn checkpoint_state(&self) -> SessionCheckpoint {
        serde_json::from_slice(&self.checkpoint_bytes()).unwrap()
    }
    pub fn command(&self, terminal: bool, create: bool) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cli"));
        command
            .current_dir(&self.directory)
            .arg("--definition")
            .arg(&self.declaration)
            .arg("--session")
            .arg(&self.checkpoint)
            .arg(if terminal { "--commands" } else { "--machine" });
        if create {
            command
                .arg("--create")
                .arg("--intent-file")
                .arg(&self.intent);
        }
        command
    }
    pub fn start(&self, terminal: bool, create: bool) -> AuthoringProcess {
        AuthoringProcess::spawn(self.command(terminal, create))
    }
    #[cfg(feature = "fault-injection")]
    pub fn start_fault(
        &self,
        terminal: bool,
        point: &str,
        action: &str,
    ) -> (AuthoringProcess, PathBuf) {
        let marker = self
            .directory
            .join(format!("fault-{}.json", uuid::Uuid::new_v4()));
        let mut command = self.command(terminal, false);
        command
            .env("CLI_TEST_FAULT_POINT", point)
            .env("CLI_TEST_FAULT_ACTION", action)
            .env("CLI_TEST_FAULT_MARKER", &marker);
        (AuthoringProcess::spawn(command), marker)
    }
    pub fn reject_open(&self, code: &str) {
        let before = self.checkpoint_bytes();
        let mut process = self.start(false, false);
        assert_eq!(process.header["error"]["code"], code, "{}", process.header);
        assert_eq!(process.header["status"], "error");
        assert!(process.header["session"].is_null());
        assert!(!process.close().success());
        assert_eq!(self.checkpoint_bytes(), before);
    }
}
impl Drop for AuthoringFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

pub struct AuthoringProcess {
    pub child: Child,
    input: Option<ChildStdin>,
    output: Receiver<Vec<u8>>,
    pub header: Value,
}
impl AuthoringProcess {
    pub fn spawn(mut command: Command) -> Self {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (sender, output) = channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut bytes = Vec::new();
                if reader.read_until(b'\n', &mut bytes).unwrap() == 0 {
                    break;
                }
                if sender.send(bytes).is_err() {
                    break;
                }
            }
        });
        let mut process = Self {
            child,
            input,
            output,
            header: Value::Null,
        };
        process.header = process.receive();
        process
    }
    pub fn receive_bytes(&mut self) -> Vec<u8> {
        let bytes = self
            .output
            .recv_timeout(Duration::from_secs(20))
            .expect("Application response within 20 seconds");
        assert!(
            bytes.len() <= MAX_RESPONSE_BYTES,
            "Application returned an oversized response: {}",
            bytes.len()
        );
        bytes
    }
    pub fn receive(&mut self) -> Value {
        let bytes = self.receive_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        if value.get("session").is_some() {
            self.header = value.clone();
        }
        value
    }
    pub fn send_bytes(&mut self, bytes: &[u8]) {
        let input = self.input.as_mut().unwrap();
        input.write_all(bytes).unwrap();
        input.flush().unwrap();
    }
    pub fn raw_line(&mut self, bytes: &[u8]) -> Value {
        self.send_bytes(bytes);
        self.receive()
    }
    pub fn canonical_request(&self, fragment: &Value) -> Value {
        let mut request = fragment.clone();
        let object = request.as_object_mut().unwrap();
        object
            .entry("request_id")
            .or_insert_with(|| json!(uuid::Uuid::new_v4().to_string()));
        object
            .entry("session_id")
            .or_insert_with(|| self.header["session"]["session_id"].clone());
        let effect = fixture_definition()
            .operations
            .get(object["operation"].as_str().unwrap())
            .map(|operation| operation.effect.clone());
        if effect.is_some_and(|effect| effect != "read") {
            object
                .entry("expected_revision")
                .or_insert_with(|| self.header["session"]["revision"].clone());
        }
        request
    }
    pub fn submit(&mut self, request: &Value) -> Value {
        let mut bytes = serde_json::to_vec(request).unwrap();
        bytes.push(b'\n');
        self.raw_line(&bytes)
    }
    pub fn request(&mut self, fragment: &Value) -> Value {
        let request = self.canonical_request(fragment);
        self.submit(&request)
    }
    pub fn terminal(&mut self, line: &str) -> Value {
        self.raw_line(format!("{line}\n").as_bytes())
    }
    pub fn close(&mut self) -> std::process::ExitStatus {
        self.input.take();
        self.child.wait().unwrap()
    }
    pub fn kill(&mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
    }
}
impl Drop for AuthoringProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn wait_for_fault(marker: &Path, point: &str, child: &AuthoringProcess) {
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(bytes) = fs::read(marker)
            && let Ok(value) = serde_json::from_slice::<Value>(&bytes)
        {
            assert_eq!(value["point"], point, "INVALID: unexpected fault boundary");
            assert_eq!(
                value["pid"],
                child.child.id(),
                "INVALID: marker belongs to another process"
            );
            println!("FAULT_LANDED {value}");
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "INVALID: fault injection did not land at {point}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

pub fn check_rejection(fixture: &AuthoringFixture, before: &[u8], response: &Value, code: &str) {
    assert_eq!(response["status"], "error", "{response}");
    assert_eq!(response["error"]["code"], code, "{response}");
    assert_eq!(response["mutation"], "none", "{response}");
    assert_eq!(
        fixture.checkpoint_bytes(),
        before,
        "Rejected request altered checkpoint bytes"
    );
    assert_eq!(
        response["session"],
        json!(
            fixture
                .checkpoint_state()
                .session_header("durable")
                .unwrap()
        )
    );
}

pub fn report_pass(id: &str) {
    println!("PASS {id}");
}

pub fn retain_acceptance_bytes(name: &str, bytes: &[u8]) {
    if let Some(directory) = std::env::var_os("CLI_ACCEPTANCE_EVIDENCE_DIR") {
        let directory = PathBuf::from(directory);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join(name), bytes).unwrap();
    }
}

pub fn normalize_generated_fields(value: &Value, directory: &Path) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| {
                    let value = if ["request_id", "session_id"].contains(&key.as_str())
                        && !value.is_null()
                    {
                        json!(format!("<generated-{key}>"))
                    } else {
                        normalize_generated_fields(value, directory)
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| normalize_generated_fields(value, directory))
                .collect(),
        ),
        Value::String(text) => json!(text.replace(directory.to_str().unwrap(), "<fixture>")),
        _ => value.clone(),
    }
}

pub fn check_corpus_expectations(
    fixture: &AuthoringFixture,
    response: &Value,
    expected: &Value,
    before: &[u8],
    previous_event: &Value,
    fragment: Option<&Value>,
) {
    let state = fixture.checkpoint_state();
    if response["event"] == "response" {
        assert_eq!(
            response["status"],
            expected.get("status").cloned().unwrap_or(json!("ok")),
            "{response}"
        );
        assert_eq!(response["session"]["durability"], "durable");
        if response["status"] == "error" {
            assert_eq!(fixture.checkpoint_bytes(), before);
        } else if fragment.is_some_and(|fragment| {
            fixture_definition().operations[fragment["operation"].as_str().unwrap()].effect
                != "state"
        }) {
            assert_eq!(fixture.checkpoint_bytes(), before);
            assert_eq!(response["mutation"], "none");
        } else {
            assert_eq!(response["mutation"], "applied");
        }
    }
    assert_eq!(
        response["session"],
        json!(state.session_header("durable").unwrap())
    );
    for (key, wanted) in expected.as_object().unwrap() {
        let actual = match key.as_str() {
            "status" | "mutation" => response[key].clone(),
            "revision" | "accepted_revision" | "context" | "dirty" => {
                response["session"][key].clone()
            }
            "code" | "failed_operation_index" => response["error"][key].clone(),
            "changed_paths" | "json_text" | "next_offset" | "value_kinds" => {
                response["result"][key].clone()
            }
            "operation" => response["result"]["operations"][0]["name"].clone(),
            "argument_names" => json!(
                response["result"]["operations"][0]["arguments"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|argument| argument["name"].clone())
                    .collect::<Vec<_>>()
            ),
            "keys" => json!(
                response["result"]["entries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|entry| entry["segment"]["key"].clone())
                    .collect::<Vec<_>>()
            ),
            "changed_root_keys" => json!(
                response["result"]["changes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|change| change["path"]["segments"][0]["key"].clone())
                    .collect::<Vec<_>>()
            ),
            "checkpoint_bytes" => {
                assert_eq!(wanted, "unchanged");
                assert_eq!(fixture.checkpoint_bytes(), before);
                continue;
            }
            "candidate_json" | "accepted_json" => {
                let document = if key == "candidate_json" {
                    &state.candidate
                } else {
                    &state.accepted
                };
                assert_eq!(
                    document,
                    &DocumentValue::parse_document(wanted.as_str().unwrap()).unwrap()
                );
                continue;
            }
            "candidate_document" | "accepted_document" | "export_document" => {
                let wanted = DocumentValue::parse_document(
                    &fs::read_to_string(repository_path(&format!(
                        "docs/authoring/acceptance/{}",
                        wanted.as_str().unwrap()
                    )))
                    .unwrap(),
                )
                .unwrap();
                let actual = if key == "candidate_document" {
                    state.candidate.clone()
                } else if key == "accepted_document" {
                    state.accepted.clone()
                } else {
                    let exported = fs::read_to_string(
                        fixture.directory.join(
                            fragment.unwrap()["arguments"]["destination"]
                                .as_str()
                                .unwrap(),
                        ),
                    )
                    .unwrap();
                    DocumentValue::parse_document(&exported).unwrap()
                };
                assert_eq!(actual, wanted);
                continue;
            }
            "same_session_id" => {
                json!(response["session"]["session_id"] == previous_event["session"]["session_id"])
            }
            "same_intent_text" => {
                json!(response["intent_text"] == fs::read_to_string(&fixture.intent).unwrap())
            }
            "startup_event" => response["event"].clone(),
            _ => panic!("Corpus expectation has no assertion: {key}"),
        };
        assert_eq!(&actual, wanted, "Expectation {key}: {response}");
    }
}
