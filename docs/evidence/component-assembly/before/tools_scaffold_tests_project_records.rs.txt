//! Check the real documentation graph and fixture structure without emulating the application.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn collect_project_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            if entry.file_name() != ".git" && entry.file_name() != "target" {
                collect_project_files(&entry.path(), files);
            }
        } else {
            files.push(entry.path());
        }
    }
}

fn markdown_anchors(text: &str) -> BTreeSet<String> {
    let mut anchors = BTreeSet::new();
    let mut occurrences = BTreeMap::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced || !line.starts_with('#') {
            continue;
        }
        let title = line.trim_start_matches('#').trim().to_lowercase();
        let anchor: String = title
            .chars()
            .filter_map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    Some(c)
                } else if c == ' ' {
                    Some('-')
                } else {
                    None
                }
            })
            .collect();
        let occurrence = occurrences.entry(anchor.clone()).or_insert(0);
        anchors.insert(if *occurrence == 0 {
            anchor
        } else {
            format!("{anchor}-{occurrence}")
        });
        *occurrence += 1;
    }
    anchors
}

fn check_local_reference(source: &Path, target: &str) -> bool {
    if target.contains("://") || target.starts_with("mailto:") {
        return false;
    }
    let (relative, fragment) = target.split_once('#').unwrap_or((target, ""));
    let path = if relative.is_empty() {
        source.to_path_buf()
    } else {
        source.parent().unwrap().join(relative)
    };
    assert!(
        path.exists(),
        "{} has a missing local target: {target}",
        source.display()
    );
    if !fragment.is_empty() {
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            markdown_anchors(&content).contains(fragment),
            "{} has a missing anchor: {target}",
            source.display()
        );
    }
    true
}

fn board_record_id(cell: &str) -> &str {
    cell.split_once("CLI-").unwrap().1.get(..3).unwrap()
}

fn table_cells(line: &str) -> Vec<&str> {
    line.trim_matches('|').split('|').map(str::trim).collect()
}

fn check_board_path<'a>(
    id: &'a str,
    dependencies: &'a BTreeMap<String, Vec<String>>,
    ancestors: &mut Vec<&'a str>,
) {
    assert!(
        !ancestors.contains(&id),
        "Board dependency cycle at CLI-{id}"
    );
    ancestors.push(id);
    if let Some(items) = dependencies.get(id) {
        for dependency in items {
            check_board_path(dependency, dependencies, ancestors);
        }
    }
    ancestors.pop();
}

#[test]
fn project_records_resolve_and_keep_design_and_application_evidence_distinct() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let mut files = Vec::new();
    collect_project_files(&root, &mut files);
    files.sort();
    let markdown: Vec<_> = files
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    let mut local_links = 0;
    for path in &markdown {
        let relative = path.strip_prefix(&root).unwrap();
        assert!(
            relative.starts_with("docs")
                || ["README.md", "VISION.md", "AGENTS.md"]
                    .iter()
                    .any(|p| relative == Path::new(p)),
            "Project documentation outside docs: {}",
            relative.display()
        );
        let content = fs::read_to_string(path).unwrap();
        assert!(
            content.is_ascii(),
            "Non-ASCII authored Markdown: {}",
            path.display()
        );
        for after in content.split("](").skip(1) {
            let target = after.split_once(')').expect("Unterminated inline link").0;
            if check_local_reference(path, target) {
                local_links += 1;
            }
        }
    }
    assert!(
        !files
            .iter()
            .any(|p| p.extension().is_some_and(|e| e == "py")),
        "Python repository tooling remains"
    );

    let board = fs::read_to_string(root.join("docs/BOARD.md")).unwrap();
    let backlog = fs::read_to_string(root.join("docs/BACKLOG.md")).unwrap();
    let mut record_states = BTreeMap::new();
    for section in backlog.split("### CLI-").skip(1) {
        let (id, body) = section.split_once('\n').unwrap();
        let state_line = body
            .lines()
            .find(|line| line.starts_with("| State |"))
            .unwrap();
        let state = table_cells(state_line)[1]
            .trim_end_matches('.')
            .split_whitespace()
            .next()
            .unwrap();
        assert!(
            record_states
                .insert(id.trim().to_owned(), state.to_owned())
                .is_none(),
            "Duplicate backlog ID"
        );
    }
    let mut board_states = BTreeMap::new();
    for line in board.lines().filter(|line| line.starts_with("| [CLI-")) {
        let cells = table_cells(line);
        if cells.len() != 6 {
            continue;
        }
        let id = board_record_id(cells[0]);
        let record_state = &record_states[id];
        let expected = match cells[4] {
            "Done" => "Closed",
            "Held" => "Parked",
            "Ready" | "Waiting" | "Active" => "Open",
            _ => panic!("Unknown board state"),
        };
        assert_eq!(record_state, expected, "Board/backlog mismatch: CLI-{id}");
        assert!(
            board_states
                .insert(id.to_owned(), cells[4].to_owned())
                .is_none()
        );
    }
    assert_eq!(
        board_states.keys().collect::<Vec<_>>(),
        record_states.keys().collect::<Vec<_>>()
    );
    let mut dependencies = BTreeMap::new();
    for line in board
        .lines()
        .filter(|line| line.starts_with("| M") && line.contains("[CLI-"))
    {
        let cells = table_cells(line);
        let id = board_record_id(cells[1]);
        assert_eq!(cells[2], board_states[id]);
        let required: Vec<String> = cells[3]
            .split("CLI-")
            .skip(1)
            .map(|part| part[..3].to_owned())
            .collect();
        for dependency in &required {
            assert!(record_states.contains_key(dependency), "Unknown dependency");
            if ["Ready", "Active", "Done"].contains(&cells[2]) {
                assert_eq!(
                    record_states[dependency], "Closed",
                    "Eligible or completed work has an unmet prerequisite"
                );
            }
        }
        dependencies.insert(id.to_owned(), required);
    }
    for id in dependencies.keys() {
        check_board_path(id, &dependencies, &mut Vec::new());
    }
    assert_eq!(board_states["001"], "Done");
    if board_states["002"] == "Done" {
        let evidence: Value = serde_json::from_slice(
            &fs::read(root.join("docs/evidence/cli-002/verification.json"))
                .expect("Completed authoring needs its separate execution record"),
        )
        .unwrap();
        assert_eq!(evidence["application_acceptance"]["status"], "pass");
        assert_eq!(
            evidence["acceptance_corpus_sha256"],
            format!(
                "{:x}",
                Sha256::digest(
                    fs::read(root.join("docs/authoring/acceptance/authoring-cases.json")).unwrap()
                )
            )
        );
    }
    if board_states["003"] == "Done" {
        let evidence: Value = serde_json::from_slice(
            &fs::read(root.join("docs/evidence/cli-003/verification.json"))
                .expect("Completed composition needs its separate execution record"),
        )
        .unwrap();
        assert_eq!(evidence["application_acceptance"]["status"], "pass");
        for relative in [
            "docs/composition/acceptance/TASK.md",
            "docs/composition/acceptance/catalog.commands",
            "docs/composition/acceptance/expected-definition.json",
        ] {
            assert_eq!(
                evidence["acceptance_sources_sha256"][relative],
                format!(
                    "{:x}",
                    Sha256::digest(fs::read(root.join(relative)).unwrap())
                )
            );
        }
    }
    if board_states["004"] == "Done" {
        let evidence: Value = serde_json::from_slice(
            &fs::read(root.join("docs/evidence/cli-004/verification.json"))
                .expect("Completed constraints need their execution record"),
        )
        .unwrap();
        assert_eq!(evidence["application_acceptance"]["status"], "pass");
        assert_eq!(evidence["walkthrough"]["status"], "pass");
        assert!(evidence["upstream_cases"]["positive"].as_u64().unwrap() > 0);
        assert!(evidence["upstream_cases"]["negative"].as_u64().unwrap() > 0);
        for relative in [
            "docs/constraints/acceptance/TASK.md",
            "docs/constraints/acceptance/schema.commands",
            "docs/constraints/acceptance/instance.commands",
            "docs/constraints/acceptance/openapi.commands",
            "docs/constraints/acceptance/service.schema.json",
            "docs/constraints/acceptance/service.expected.json",
            "docs/constraints/acceptance/openapi.expected.json",
        ] {
            assert_eq!(
                evidence["acceptance_sources_sha256"][relative],
                format!(
                    "{:x}",
                    Sha256::digest(fs::read(root.join(relative)).unwrap())
                )
            );
        }
    }
    for relative in [
        "docs/ARCHITECTURE.md",
        "docs/BOARD.md",
        "docs/BACKLOG.md",
        "docs/resume-work.md",
    ] {
        let content = fs::read_to_string(root.join(relative)).unwrap();
        assert!(
            content.contains(
                "authoring/CLI-001.md#decision-0001-establish-the-bounded-authoring-contract"
            ),
            "Decision absorption reference missing: {relative}"
        );
    }

    let declaration_path = root.join("docs/authoring/operations.json");
    let declaration: Value = serde_json::from_slice(&fs::read(&declaration_path).unwrap()).unwrap();
    let operations = declaration["operations"].as_array().unwrap();
    let mut declared_names = BTreeSet::new();
    let mut handlers = BTreeSet::new();
    let mut effects = BTreeMap::new();
    let custom_types = declaration["types"].as_object().unwrap();
    for definition in custom_types.values() {
        assert!(!definition["codec"].as_str().unwrap().is_empty());
        check_local_reference(&declaration_path, definition["contract"].as_str().unwrap());
    }
    for operation in operations {
        let name = operation["name"].as_str().unwrap();
        assert!(declared_names.insert(name));
        assert!(handlers.insert(operation["handler"].as_str().unwrap()));
        assert_eq!(operation["unknown_arguments"], "reject");
        assert!(!operation["help"].as_str().unwrap().is_empty());
        let effect = operation["effect"].as_str().unwrap();
        assert!(["read", "state", "export"].contains(&effect));
        effects.insert(name, effect);
        let arguments = operation["arguments"].as_array().unwrap();
        let names: BTreeSet<_> = arguments
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();
        assert_eq!(names.len(), arguments.len());
        for argument in arguments {
            let kind = argument["type"].as_str().unwrap();
            assert!(
                ["string", "integer", "enum"].contains(&kind) || custom_types.contains_key(kind)
            );
        }
        if operation["terminal"]["form"] == "tokens" {
            assert_eq!(operation["terminal"]["command"], name);
            for positional in operation["terminal"]["positional"].as_array().unwrap() {
                assert!(names.contains(positional.as_str().unwrap()));
            }
        } else {
            assert_eq!(name, "batch");
        }
        assert_eq!(
            operation["batchable"],
            json!(["set", "append", "insert", "delete"].contains(&name))
        );
    }
    let corpus_path = root.join("docs/authoring/acceptance/authoring-cases.json");
    let extension_path = root.join("docs/composition/operations.json");
    let extension: Value = serde_json::from_slice(&fs::read(&extension_path).unwrap()).unwrap();
    for definition in extension["types"].as_object().unwrap().values() {
        check_local_reference(&extension_path, definition["contract"].as_str().unwrap());
    }
    let corpus: Value = serde_json::from_slice(&fs::read(&corpus_path).unwrap()).unwrap();
    assert_eq!(corpus["execution_status"], "not-run-against-application");
    for field in ["contract", "operation_declaration"] {
        check_local_reference(&corpus_path, corpus[field].as_str().unwrap());
    }
    let steps = corpus["journey"]["steps"].as_array().unwrap();
    let operation_cases = corpus["operation_cases"].as_array().unwrap();
    let scenarios = corpus["system_scenarios"].as_array().unwrap();
    let mut case_ids = BTreeSet::new();
    for case in operation_cases.iter().chain(scenarios) {
        assert!(
            case_ids.insert(case["id"].as_str().unwrap()),
            "Duplicate acceptance ID"
        );
        assert!(!case["expect"].as_object().unwrap().is_empty());
    }
    let mut covered = BTreeSet::new();
    let mut revision = 0_u64;
    for step in steps {
        let request = &step["request"];
        if let Some(operation) = request["operation"].as_str() {
            assert!(declared_names.contains(operation));
            covered.insert(operation);
            let terminal = step["terminal"].as_str().unwrap();
            assert!(
                !terminal.contains(|c| matches!(c, '{' | '}' | '[' | ']')),
                "Journey imports serialized containers"
            );
            if effects[operation] == "state" && step["expect"]["status"] != "error" {
                revision += 1;
            }
        } else {
            assert_eq!(step["harness_action"], "close_and_reopen");
        }
        assert_eq!(
            step["expect"]["revision"],
            revision.to_string(),
            "Inconsistent declared journey revision"
        );
        for field in ["candidate_document", "accepted_document", "export_document"] {
            if let Some(reference) = step["expect"][field].as_str() {
                check_local_reference(&corpus_path, reference);
            }
        }
    }
    for case in operation_cases {
        if let Some(operation) = case["request"]["operation"].as_str() {
            if declared_names.contains(operation) {
                covered.insert(operation);
            } else {
                assert_eq!(case["expect"]["code"], "UNKNOWN_OPERATION");
            }
        }
        for field in ["candidate_json", "accepted_json"] {
            let document = case["initial"][field].as_str().unwrap();
            serde_json::from_str::<Value>(document).expect("Invalid initial fixture JSON");
        }
        if let Some(document) = case["expect"]["candidate_json"].as_str() {
            serde_json::from_str::<Value>(document).expect("Invalid expected fixture JSON");
        }
    }
    // The import corpus extends coverage without rewriting the original authoring task or oracle.
    let import_path = root.join("docs/authoring/acceptance/import-cases.json");
    let import_corpus: Value = serde_json::from_slice(&fs::read(&import_path).unwrap()).unwrap();
    check_local_reference(&import_path, import_corpus["contract"].as_str().unwrap());
    let import_operation = import_corpus["operation"].as_str().unwrap();
    assert!(declared_names.contains(import_operation));
    for group in ["valid", "invalid"] {
        let cases = import_corpus[group].as_array().unwrap();
        assert!(
            !cases.is_empty(),
            "Import corpus needs positive and negative cases"
        );
        for case in cases {
            assert!(
                case_ids.insert(case["id"].as_str().unwrap()),
                "Duplicate import acceptance ID"
            );
            assert!(case["source"].is_string());
            assert!(
                case[if group == "valid" {
                    "expected_json"
                } else {
                    "code"
                }]
                .is_string()
            );
        }
    }
    covered.insert(import_operation);
    assert_eq!(
        covered, declared_names,
        "An operation has no acceptance example"
    );
    for name in [
        "service-catalog.expected.json",
        "service-catalog.draft.json",
    ] {
        let document = fs::read_to_string(corpus_path.parent().unwrap().join(name)).unwrap();
        serde_json::from_str::<Value>(&document).expect("Invalid oracle JSON");
        assert!(document.contains("9007199254740993") && document.contains("1.2300"));
    }
    let hashes: BTreeMap<_, _> = files
        .iter()
        .filter(|path| {
            let relative = path.strip_prefix(&root).unwrap();
            relative != Path::new("docs/evidence/cli-001/verification.json")
                && relative != Path::new("docs/evidence/cli-002/verification.json")
                && relative != Path::new("docs/evidence/cli-003/verification.json")
                && path.extension().is_some_and(|ext| {
                    ["md", "json", "rs", "toml", "lock"]
                        .iter()
                        .any(|e| ext == *e)
                })
        })
        .map(|path| {
            (
                path.strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                format!("{:x}", Sha256::digest(fs::read(path).unwrap())),
            )
        })
        .collect();
    let evidence = json!({
        "status": "pass", "scope": "Static project-record checks; no application requests executed",
        "markdown_documents": markdown.len(), "local_inline_links": local_links,
        "documentation_layout_violations": 0, "unresolved_local_inline_links": 0,
        "board_records": record_states.len(), "milestones": dependencies.len(),
        "board_state_mismatches": 0, "dependency_cycles": 0, "unmet_ready_prerequisites": 0,
        "declared_operations": operations.len(), "uncovered_operations": [],
        "journey_steps": steps.len(), "operation_cases": operation_cases.len(), "system_scenarios": scenarios.len(),
        "application_acceptance_in_this_check": "not-executed", "design_corpus_status_at_creation": corpus["execution_status"], "source_files_sha256": hashes,
    });
    println!("\nPROJECT_RECORD_EVIDENCE: {evidence}");
}
