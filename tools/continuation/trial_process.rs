//! Capture real CLI process input, output, and timing for a bounded continuation trial.

use programmable_cli::authoring_protocol::SessionCheckpoint;
use programmable_cli::kernel_profile::KernelProfile;
use programmable_cli::operation_definition::OperationDefinition;
use programmable_cli::terminal_input::compile_terminal_request;
use serde_json::{Value, json};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub type TrialResult<T> = Result<T, Box<dyn std::error::Error>>;

/// The process transport retains wire bytes before interpreting a result or computing counters.
pub struct TrialProcess {
    child: Child,
    input: Option<ChildStdin>,
    output: Receiver<std::io::Result<Vec<u8>>>,
    input_log: File,
    output_log: File,
    metrics_path: PathBuf,
    checkpoint: PathBuf,
    declaration: PathBuf,
    terminal: bool,
    started: Instant,
    started_unix_ms: u128,
    input_bytes: u64,
    output_bytes: u64,
    requests: u64,
    failures: u64,
    applied: u64,
    pub header: Value,
}

/// A failed trial predicate is an error, never an inferred successful result.
pub fn require_trial(condition: bool, message: &str) -> TrialResult<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

/// Create an evidence file exclusively so another trial cannot silently replace its history.
pub fn trial_file(path: &Path) -> TrialResult<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(OpenOptions::new().create_new(true).write(true).open(path)?)
}

/// Serialize an experiment record without editing any product-authored JSON document.
pub fn write_trial_json(path: &Path, value: &Value) -> TrialResult<()> {
    let mut file = trial_file(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}

impl TrialProcess {
    /// Start one CLI process with an explicit checkpoint, declaration, presentation, and trace prefix.
    pub fn start(
        cli: &Path,
        directory: &Path,
        checkpoint_name: &str,
        declaration_name: &str,
        create_intent: Option<&str>,
        terminal: bool,
        trace: &str,
    ) -> TrialResult<Self> {
        let mut command = Command::new(cli);
        command.current_dir(directory).args([
            "--definition",
            declaration_name,
            "--session",
            checkpoint_name,
            "--compose",
            "--constraints",
            if terminal { "--commands" } else { "--machine" },
        ]);
        if let Some(intent) = create_intent {
            command.args(["--create", "--intent-file", intent]);
        }
        let prefix = directory.join(trace);
        let input_log = trial_file(&prefix.with_extension("input.txt"))?;
        let output_log = trial_file(&prefix.with_extension("output.jsonl"))?;
        let stderr = trial_file(&prefix.with_extension("stderr.txt"))?;
        let started = Instant::now();
        let started_unix_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .spawn()?;
        let input = child.stdin.take();
        let stdout = child.stdout.take().ok_or("Trial stdout unavailable")?;
        let (sender, output) = channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut bytes = Vec::new();
                match reader.read_until(b'\n', &mut bytes) {
                    Ok(0) => break,
                    Ok(_) => {
                        if sender.send(Ok(bytes)).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = sender.send(Err(error));
                        break;
                    }
                }
            }
        });
        let mut process = Self {
            child,
            input,
            output,
            input_log,
            output_log,
            metrics_path: prefix.with_extension("metrics.json"),
            checkpoint: directory.join(checkpoint_name),
            declaration: directory.join(declaration_name),
            terminal,
            started,
            started_unix_ms,
            input_bytes: 0,
            output_bytes: 0,
            requests: 0,
            failures: 0,
            applied: 0,
            header: Value::Null,
        };
        process.header = process.receive()?;
        Ok(process)
    }

    fn receive(&mut self) -> TrialResult<Value> {
        let bytes = self.output.recv_timeout(Duration::from_secs(20))??;
        self.output_log.write_all(&bytes)?;
        self.output_log.flush()?;
        self.output_bytes += bytes.len() as u64;
        let value: Value = serde_json::from_slice(&bytes)?;
        if value["status"] == "error" || value["status"] == "uncertain" {
            self.failures += 1;
        }
        if value["mutation"] == "applied" {
            self.applied += 1;
        }
        self.header = value.clone();
        Ok(value)
    }

    /// Send a retained wire request; callers use this for the unchanged abandoned request.
    pub fn send_wire(&mut self, bytes: &[u8]) -> TrialResult<Value> {
        self.input_log.write_all(bytes)?;
        self.input_log.flush()?;
        self.input_bytes += bytes.len() as u64;
        self.requests += 1;
        let input = self.input.as_mut().ok_or("Trial process is closed")?;
        input.write_all(bytes)?;
        input.flush()?;
        self.receive()
    }

    /// Use the product compiler for paired machine input, or submit literal terminal commands.
    pub fn command(&mut self, line: &str) -> TrialResult<Value> {
        let mut bytes = if self.terminal {
            line.as_bytes().to_vec()
        } else {
            let definition = OperationDefinition::load_kernel_profile(
                &self.declaration,
                KernelProfile::ConstrainedComposition,
            )?;
            let request = compile_terminal_request(line, &definition, &self.state()?)?;
            serde_json::to_vec(&request)?
        };
        bytes.push(b'\n');
        self.send_wire(&bytes)
    }

    /// Read the actual checkpoint acknowledged by this process, never a constructed state proxy.
    pub fn state(&self) -> TrialResult<SessionCheckpoint> {
        Ok(serde_json::from_slice(&fs::read(&self.checkpoint)?)?)
    }

    /// Retain normal completion or timeout; process time excludes actor reasoning between invocations.
    pub fn close(mut self) -> TrialResult<Value> {
        self.input.take();
        let deadline = Instant::now() + Duration::from_secs(20);
        let status = loop {
            if let Some(status) = self.child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                return Err("Trial process did not exit within 20 seconds".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let metrics = json!({
            "presentation":if self.terminal {"terminal"} else {"machine"},
            "started_unix_ms":self.started_unix_ms.to_string(),
            "elapsed_process_ms":self.started.elapsed().as_millis().to_string(),
            "requests":self.requests, "input_bytes":self.input_bytes,
            "output_bytes":self.output_bytes, "failures":self.failures,
            "applied":self.applied, "exit_code":status.code(),
            "token_use":null, "actor_reasoning_time":null,
        });
        write_trial_json(&self.metrics_path, &metrics)?;
        Ok(metrics)
    }
}

impl Drop for TrialProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Execute a literal construction recipe and stop on its first unexpected result.
#[allow(dead_code)] // The recipient binary records actor input without producer recipes.
pub fn trial_commands(process: &mut TrialProcess, text: &str) -> TrialResult<()> {
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        require_trial(
            !line.contains(['{', '}', '[', ']']),
            "Construction recipe contains raw JSON containers",
        )?;
        let response = process.command(line)?;
        require_trial(
            response["status"] == "ok",
            &format!("Trial command failed: {line}: {response}"),
        )?;
    }
    Ok(())
}
