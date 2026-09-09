//! Compare known authoring recipes while retaining full-continuation gaps and all input surfaces.

use crate::trial_manifest::{TRIAL_INPUT_FILES, trial_file_digest, verify_trial_manifest};
use crate::trial_oracle::{evaluate_worker_artifacts, measure_trial_logs};
use crate::trial_package::prepare_continuation;
use crate::trial_process::{
    TrialProcess, TrialResult, require_trial, trial_commands, trial_file, write_trial_json,
};
use crate::trial_rehearsal::begin_continuation_trial;
use crate::trial_text_edit::apply_trial_text_edits;
use programmable_cli::document_value::DocumentValue;
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Copy)]
enum ComparisonMethod {
    ContinueCli,
    EditJson,
    RebuildCli,
    ImportEditedJson,
}

impl ComparisonMethod {
    fn directory_name(self) -> &'static str {
        match self {
            Self::ContinueCli => "cli-continuation",
            Self::EditJson => "targeted-json-editing",
            Self::RebuildCli => "cli-reconstruction",
            Self::ImportEditedJson => "targeted-json-import",
        }
    }
}

fn copy_comparison_package(source: &Path, destination: &Path) -> TrialResult<Value> {
    let started = Instant::now();
    verify_trial_manifest(source)?;
    fs::create_dir(destination)?;
    let mut bytes = 0;
    for name in TRIAL_INPUT_FILES.into_iter().chain(["manifest.json"]) {
        bytes += fs::copy(source.join(name), destination.join(name))?;
    }
    verify_trial_manifest(destination)?;
    require_trial(
        fs::read(source.join("manifest.json"))? == fs::read(destination.join("manifest.json"))?,
        "Comparison copy changed the input manifest",
    )?;
    Ok(
        json!({"bytes_copied":bytes,"elapsed_copy_ms":started.elapsed().as_millis().to_string(),
        "scope":"Full immutable package copy and both manifest checks; included in method wall time."}),
    )
}

fn validate_comparison_file(
    process: &TrialProcess,
    file: &Path,
    evidence: &Path,
) -> TrialResult<Value> {
    let started = Instant::now();
    let bytes = fs::read(file)?;
    let document = DocumentValue::parse_document(std::str::from_utf8(&bytes)?)?;
    let state = process.state()?;
    let constraint = state
        .active_constraint
        .as_ref()
        .ok_or("Comparison policy is missing")?;
    let validation = constraint.validate_schema_instance(&document)?;
    let report = json!({"validation":validation,"document_bytes_read":bytes.len(),
        "elapsed_validation_ms":started.elapsed().as_millis().to_string(),
        "validator":"Existing Rust schema checker called by trial tooling on the actual external file; not a CLI commit.",
        "timing_overlap":"Included in method and open-process wall time."});
    write_trial_json(evidence, &report)?;
    Ok(report)
}

fn run_comparison_method(
    recipe_directory: &Path,
    package: &Path,
    method: ComparisonMethod,
) -> TrialResult<Value> {
    let mut process = begin_continuation_trial(package, true, "logs/comparison")?;
    let controls = measure_trial_logs(package)?;
    let recipe = fs::read_to_string(recipe_directory.join("queue-finish.commands"))?;
    require_trial(
        recipe.matches("\ncommit\n").count() == 1,
        "Comparison completion boundary is ambiguous",
    )?;
    let (construction, finish) = recipe.split_once("\ncommit\n").unwrap();
    let mut external = Value::Null;
    match method {
        ComparisonMethod::ContinueCli => trial_commands(&mut process, construction)?,
        ComparisonMethod::RebuildCli => {
            let rebuild = fs::read_to_string(recipe_directory.join("rebuild.commands"))?;
            trial_file(&package.join("rebuild.commands"))?.write_all(rebuild.as_bytes())?;
            trial_commands(&mut process, &rebuild)?;
        }
        ComparisonMethod::EditJson | ComparisonMethod::ImportEditedJson => {
            let (export, file_name) = match method {
                ComparisonMethod::ImportEditedJson => {
                    ("save edited.definition.json\n", "edited.definition.json")
                }
                _ => (
                    "save completed.definition.json\n",
                    "completed.definition.json",
                ),
            };
            trial_commands(&mut process, export)?;
            let file = package.join(file_name);
            let before = validate_comparison_file(
                &process,
                &file,
                &package.join("external-validation-before.json"),
            )?;
            require_trial(
                before["validation"]["valid"] == false,
                "Supplied file unexpectedly passed schema validation",
            )?;
            let edits = apply_trial_text_edits(
                &file,
                &recipe_directory.join("targeted-edits.json"),
                &package.join("external-edit"),
            )?;
            let after = validate_comparison_file(
                &process,
                &file,
                &package.join("external-validation-after.json"),
            )?;
            require_trial(
                after["validation"]["valid"] == true,
                "Edited file failed schema validation",
            )?;
            external = json!({"editing":edits,"validation_before":before,"validation_after":after});
            if matches!(method, ComparisonMethod::ImportEditedJson) {
                trial_commands(
                    &mut process,
                    "import edited.definition.json\nedit /contexts/queues/commands\n",
                )?;
            }
        }
    }
    let construction_done = measure_trial_logs(package)?;
    if !matches!(method, ComparisonMethod::EditJson) {
        trial_commands(&mut process, "commit\n")?;
    }
    for line in finish.lines() {
        let command = match (method, line) {
            (ComparisonMethod::EditJson, "activate") => "activate completed.definition.json",
            (ComparisonMethod::EditJson, "save completed.definition.json") => continue,
            (_, line) => line,
        };
        trial_commands(&mut process, command)?;
    }
    require_trial(
        process.close()?["exit_code"] == 0,
        "Comparison process did not close cleanly",
    )?;
    let interaction = measure_trial_logs(package)?;
    let phase = |start: &Value, end: &Value| {
        json!({
            "input_lines":end["input_lines"].as_u64().unwrap()-start["input_lines"].as_u64().unwrap(),
            "input_bytes":end["input_bytes"].as_u64().unwrap()-start["input_bytes"].as_u64().unwrap(),
            "output_bytes":end["output_bytes"].as_u64().unwrap()-start["output_bytes"].as_u64().unwrap()
        })
    };
    let workflow = json!({"interaction":interaction,"external_work":external,
        "phases":{"shared_controls":controls,"construction":phase(&controls,&construction_done),
            "completion":phase(&construction_done,&interaction)},
        "phase_note":"Phase counters use retained wire records; intermediate process timers are incomplete until close. External edit/validation costs are separate, not zero.",
        "candidate_import_used":matches!(method, ComparisonMethod::ImportEditedJson)});
    write_trial_json(&package.join("workflow.json"), &workflow)?;
    Ok(workflow)
}

/// Compare independent scripted methods, retaining the activation-only baseline beside candidate import.
pub fn compare_authoring_methods(
    root: &Path,
    cli: &Path,
    receiver: &Path,
    destination: &Path,
) -> TrialResult<Value> {
    let started = Instant::now();
    fs::create_dir(destination)?;
    let destination = destination.canonicalize()?;
    let recipe_directory = destination.join("recipes");
    fs::create_dir(&recipe_directory)?;
    let mut recipe_inputs = serde_json::Map::new();
    for (name, source) in [
        (
            "queue-finish.commands",
            "docs/reuse/acceptance/queue-finish.commands",
        ),
        ("rebuild.commands", "docs/reuse/comparison/rebuild.commands"),
        (
            "targeted-edits.json",
            "docs/reuse/comparison/targeted-edits.json",
        ),
    ] {
        let copied = recipe_directory.join(name);
        let bytes = fs::copy(root.join(source), &copied)?;
        recipe_inputs.insert(
            name.into(),
            json!({"source":source,"bytes":bytes,"sha256":trial_file_digest(&copied)?}),
        );
    }
    let preparation = prepare_continuation(root, cli, receiver, &destination.join("shared"), true)?;
    let seed = destination.join("shared/package");
    let mut methods = Vec::new();
    for method in [
        ComparisonMethod::ContinueCli,
        ComparisonMethod::EditJson,
        ComparisonMethod::RebuildCli,
        ComparisonMethod::ImportEditedJson,
    ] {
        let method_started = Instant::now();
        let directory = destination.join(method.directory_name());
        fs::create_dir(&directory)?;
        let package = directory.join("package");
        let mut record = json!({"method":method.directory_name(),"actor":"scripted known recipe","token_use":null,
            "actor_effort":null,"comparison_to_fresh_agent_timing":"not-comparable"});
        let result = (|| -> TrialResult<()> {
            record["package_copy"] = copy_comparison_package(&seed, &package)?;
            record["workflow"] = run_comparison_method(&recipe_directory, &package, method)?;
            record["elapsed_workflow_wall_ms"] =
                json!(method_started.elapsed().as_millis().to_string());
            record["evaluation"] =
                evaluate_worker_artifacts(root, &package, &directory.join("evaluation"))?;
            record["checkpoint_bytes"] =
                json!(fs::metadata(package.join("work.session.json"))?.len());
            record["submitted_definition_sha256"] = json!(trial_file_digest(
                &package.join("completed.definition.json")
            )?);
            Ok(())
        })();
        record["status"] = json!(if result.is_ok() { "pass" } else { "fail" });
        if let Err(error) = result {
            record["error"] = json!(error.to_string());
            record["available_interaction"] = measure_trial_logs(&package)
                .unwrap_or_else(|error| json!({"unavailable":error.to_string()}));
        }
        record["elapsed_method_and_evaluation_ms"] =
            json!(method_started.elapsed().as_millis().to_string());
        write_trial_json(&directory.join("result.json"), &record)?;
        methods.push(record);
    }
    let passed = methods.iter().all(|method| method["status"] == "pass");
    let report = json!({"status":if passed {"pass"} else {"fail"},
        "scope":"Controlled artifact and runtime comparison; consult each full_staged_continuation verdict for workflow parity.",
        "method_order":["cli-continuation","targeted-json-editing","cli-reconstruction","targeted-json-import"],
        "shared_preparation":preparation,"recipe_inputs":recipe_inputs,"methods":methods,
        "elapsed_experiment_ms":started.elapsed().as_millis().to_string(),
        "timing_scope":"Experiment includes preparation, copies, methods, and evaluation; nested timers overlap. Compilation and recipe authoring are excluded.",
        "agent_or_human_advantage":"not-claimed", "full_workflow_economic_advantage":"not-claimed",
        "sample_scope":"One execution per known method; no randomized or population timing inference."});
    write_trial_json(&destination.join("comparison.json"), &report)?;
    require_trial(
        passed,
        "Authoring comparison failed; retained comparison.json contains every method result",
    )?;
    Ok(report)
}
