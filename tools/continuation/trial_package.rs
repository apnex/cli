//! Prepare a portable unfinished session exclusively through recorded CLI construction commands.

use crate::trial_manifest::{TRIAL_INPUT_FILES, create_trial_manifest};
use crate::trial_process::{
    TrialProcess, TrialResult, require_trial, trial_commands, trial_file, write_trial_json,
};
use programmable_cli::document_value::DocumentValue;
use programmable_cli::kernel_profile::KernelProfile;
use programmable_cli::operation_definition::OperationDefinition;
use programmable_cli::terminal_input::compile_terminal_request;
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

/// Prepare new directories only; historical examples and prior trial outputs remain untouched.
pub fn prepare_continuation(
    root: &Path,
    cli: &Path,
    receiver: &Path,
    destination: &Path,
    terminal: bool,
) -> TrialResult<Value> {
    let started = Instant::now();
    fs::create_dir(destination)?;
    let destination = destination.canonicalize()?;
    let producer = destination.join("producer");
    let package = destination.join("package");
    fs::create_dir(&producer)?;
    fs::create_dir(&package)?;
    fs::copy(
        root.join("docs/authoring/operations.json"),
        producer.join("operations.json"),
    )?;
    fs::copy(
        root.join("docs/composition/acceptance/TASK.md"),
        producer.join("CATALOG-TASK.md"),
    )?;
    fs::copy(
        root.join("docs/reuse/acceptance/TASK.md"),
        producer.join("TASK.md"),
    )?;
    let mut policy = TrialProcess::start(
        cli,
        &producer,
        "policy.session.json",
        "operations.json",
        Some("TASK.md"),
        terminal,
        "logs/policy",
    )?;
    trial_commands(
        &mut policy,
        &fs::read_to_string(root.join("docs/reuse/acceptance/policy.commands"))?,
    )?;
    let expected_policy = DocumentValue::parse_document(&fs::read_to_string(
        root.join("docs/reuse/acceptance/queue-policy.expected.json"),
    )?)?;
    require_trial(
        policy.state()?.accepted == expected_policy,
        "Constructed policy differs from the task oracle",
    )?;
    let policy_metrics = policy.close()?;

    let mut source = TrialProcess::start(
        cli,
        &producer,
        "source.session.json",
        "operations.json",
        Some("CATALOG-TASK.md"),
        terminal,
        "logs/catalog",
    )?;
    trial_commands(
        &mut source,
        &fs::read_to_string(root.join("docs/composition/acceptance/catalog.commands"))?,
    )?;
    let expected_catalog = DocumentValue::parse_document(&fs::read_to_string(
        root.join("docs/composition/acceptance/expected-definition.json"),
    )?)?;
    require_trial(
        source.state()?.candidate == expected_catalog,
        "Reused source differs from the original catalog oracle",
    )?;
    trial_commands(
        &mut source,
        "constrain queue-policy.json\nactivate\nenter services\ninvoke quota 1e400\n",
    )?;
    let source_state = source.state()?;
    let declaration = OperationDefinition::load_kernel_profile(
        &producer.join("operations.json"),
        KernelProfile::ConstrainedComposition,
    )?;
    let abandoned =
        compile_terminal_request("delete /contexts/queues", &declaration, &source_state)?;
    let source_metrics = source.close()?;

    fs::copy(
        producer.join("source.session.json"),
        producer.join("handover.session.json"),
    )?;
    let mut partial = TrialProcess::start(
        cli,
        &producer,
        "handover.session.json",
        "operations.json",
        None,
        terminal,
        "logs/partial",
    )?;
    trial_commands(
        &mut partial,
        &fs::read_to_string(root.join("docs/reuse/acceptance/queue-start.commands"))?,
    )?;
    let handover = partial.state()?;
    require_trial(
        handover.accepted == expected_catalog
            && handover.active_interface == source_state.active_interface,
        "Preparing a draft changed the reused baseline or active interface",
    )?;
    let partial_metrics = partial.close()?;

    for file in [
        "operations.json",
        "CATALOG-TASK.md",
        "TASK.md",
        "queue-policy.json",
        "source.session.json",
        "handover.session.json",
    ] {
        fs::copy(producer.join(file), package.join(file))?;
    }
    fs::copy(
        root.join("docs/reuse/recipient/START.md"),
        package.join("START.md"),
    )?;
    fs::copy(cli, package.join("cli"))?;
    fs::copy(receiver, package.join("receiver"))?;
    let mut incompatible = fs::read(package.join("operations.json"))?;
    incompatible.push(b'\n');
    fs::write(package.join("incompatible-operations.json"), incompatible)?;
    // Machine transport requires one complete request per line.
    let mut request_file = trial_file(&package.join("abandoned-request.json"))?;
    serde_json::to_writer(&mut request_file, &abandoned)?;
    request_file.write_all(b"\n")?;
    create_trial_manifest(
        &package,
        json!({
            "kernel_profile":handover.definition_id, "kernel_sha256":handover.definition_sha256,
            "source_revision":source_state.revision, "handover_revision":handover.revision,
            "active_interface":handover.active_interface.as_ref().map(|value|value.cli_interface_header()),
            "active_constraint":handover.active_constraint,
            "source_task":"CATALOG-TASK.md", "continuation_task":"TASK.md",
        }),
    )?;
    let mut artifact_bytes = serde_json::Map::new();
    for name in TRIAL_INPUT_FILES {
        artifact_bytes.insert(name.into(), json!(fs::metadata(package.join(name))?.len()));
    }
    let evidence = json!({
        "status":"prepared", "directory":destination, "actor":"scripted producer", "fresh_actor_trial":"not-run",
        "elapsed_preparation_ms":started.elapsed().as_millis().to_string(),
        "timing_scope":"Includes source and policy construction, partial extension, file copying, executable hashing, and manifest creation; excludes compilation.",
        "policy_construction":policy_metrics, "source_asset_construction":source_metrics,
        "partial_extension":partial_metrics, "artifact_bytes":artifact_bytes,
        "manifest_bytes":fs::metadata(package.join("manifest.json"))?.len(),
        "reuse_scope":"Complete catalog definition and checkpoint, extended into a second task; no general component resolver.",
        "token_use":null, "targeted_edit_comparison":"not-run",
    });
    write_trial_json(&destination.join("preparation.json"), &evidence)?;
    Ok(evidence)
}
