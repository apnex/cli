//! Observe rejection and preservation through the compiled generator in isolated copies.

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct ScaffoldFixture {
    root: PathBuf,
}

impl ScaffoldFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "cli-layer-scaffold-{}-{}",
            std::process::id(),
            FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&root).unwrap();
        let fixture = Self { root };
        fs::create_dir(fixture.root.join("docs")).unwrap();
        let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for relative in ["docs/layers.json", "docs/ARCHITECTURE.md"] {
            fs::copy(source_root.join(relative), fixture.root.join(relative)).unwrap();
        }
        assert_success(fixture.run("--write"));
        fixture
    }

    fn run(&self, mode: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cli-layer-scaffold"))
            .arg(mode)
            .arg("--root")
            .arg(&self.root)
            .current_dir(self.root.parent().unwrap())
            .output()
            .unwrap()
    }

    fn registry(&self) -> Value {
        serde_json::from_slice(&fs::read(self.root.join("docs/layers.json")).unwrap()).unwrap()
    }

    fn write_registry(&self, registry: &Value) {
        let bytes = serde_json::to_vec_pretty(registry).unwrap();
        let path = self.root.join("docs/layers.json");
        fs::write(&path, &bytes).unwrap();
        assert_eq!(
            fs::read(path).unwrap(),
            bytes,
            "The declaration mutation did not land"
        );
    }

    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        fn collect(root: &Path, directory: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    collect(root, &path, output);
                } else if path.extension().is_some_and(|ext| ext == "md") {
                    output.insert(
                        path.strip_prefix(root).unwrap().into(),
                        fs::read(path).unwrap(),
                    );
                }
            }
        }
        let mut snapshot = BTreeMap::new();
        collect(&self.root, &self.root, &mut snapshot);
        snapshot
    }

    fn assert_rejected(&self, mode: &str, expected_error: &str) {
        let before = self.snapshot();
        let output = self.run(mode);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(expected_error), "{error}");
        assert_eq!(self.snapshot(), before, "Rejected input changed a document");
    }
}

impl Drop for ScaffoldFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn assert_success(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn current_scaffold_matches_its_declaration_under_docs() {
    let fixture = ScaffoldFixture::new();
    assert_success(fixture.run("--check"));
    assert!(
        fixture
            .snapshot()
            .keys()
            .all(|path| path.starts_with("docs"))
    );
    assert!(!fixture.root.join("layers").exists());
}

#[test]
fn repeated_generation_preserves_document_bytes() {
    let fixture = ScaffoldFixture::new();
    let before = fixture.snapshot();
    assert_success(fixture.run("--write"));
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn unknown_dependency_is_rejected() {
    let fixture = ScaffoldFixture::new();
    let mut registry = fixture.registry();
    registry["layers"][0]["depends_on"] = json!(["missing-layer"]);
    fixture.write_registry(&registry);
    fixture.assert_rejected("--write", "Layer dependency missing:");
}

#[test]
fn dependency_cycle_is_rejected() {
    let fixture = ScaffoldFixture::new();
    let mut registry = fixture.registry();
    registry["layers"][0]["depends_on"] = json!(["interaction"]);
    fixture.write_registry(&registry);
    fixture.assert_rejected("--check", "Layer dependency cycle:");
}

#[test]
fn directories_outside_documentation_scope_are_rejected() {
    let fixture = ScaffoldFixture::new();
    for directory in ["../outside", "layers/document"] {
        let mut registry = fixture.registry();
        registry["layers"][0]["directory"] = json!(directory);
        fixture.write_registry(&registry);
        fixture.assert_rejected("--write", "Layer registry directory invalid:");
    }
}

#[test]
fn invalid_registry_types_are_rejected() {
    let fixture = ScaffoldFixture::new();
    let original = fixture.registry();
    for version in [json!(true), json!(1.0), json!("1")] {
        let mut registry = original.clone();
        registry["format_version"] = version;
        fixture.write_registry(&registry);
        fixture.assert_rejected("--check", "Layer registry invalid:");
    }
    let mut registry = original;
    registry["layers"][0]["depends_on"] = json!("interaction");
    fixture.write_registry(&registry);
    fixture.assert_rejected("--check", "Layer registry invalid:");
}

#[test]
fn duplicate_identifiers_are_rejected() {
    let fixture = ScaffoldFixture::new();
    let mut registry = fixture.registry();
    let duplicate = registry["layers"][0].clone();
    registry["layers"].as_array_mut().unwrap().push(duplicate);
    fixture.write_registry(&registry);
    fixture.assert_rejected("--check", "Layer registry identifier repeated:");
}

#[test]
fn duplicate_registry_keys_are_rejected() {
    let fixture = ScaffoldFixture::new();
    let path = fixture.root.join("docs/layers.json");
    let original = fs::read_to_string(&path).unwrap();
    let changed = original.replacen(
        "\"format_version\": 1",
        "\"format_version\": 1, \"format_version\": 1",
        1,
    );
    assert_ne!(changed, original, "The duplicate-key mutation did not land");
    fs::write(&path, &changed).unwrap();
    assert_eq!(fs::read_to_string(path).unwrap(), changed);
    fixture.assert_rejected("--write", "duplicate field");
}

#[test]
fn stale_generated_content_is_detected_and_repaired() {
    let fixture = ScaffoldFixture::new();
    let path = fixture.root.join("docs/layers/document/VISION.md");
    let expected = fs::read_to_string(&path).unwrap();
    let changed = format!("{expected}\nA stale statement.\n");
    fs::write(&path, &changed).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), changed);
    fixture.assert_rejected("--check", "Layer scaffold view stale:");
    assert_success(fixture.run("--write"));
    assert_eq!(fs::read_to_string(path).unwrap(), expected);
}

#[test]
fn unowned_content_is_preserved() {
    let fixture = ScaffoldFixture::new();
    let path = fixture.root.join("docs/layers/document/VISION.md");
    let content = "# An independently authored document\n";
    fs::write(&path, content).unwrap();
    assert_eq!(fs::read_to_string(path).unwrap(), content);
    fixture.assert_rejected("--write", "Layer scaffold ownership conflict:");
}

#[test]
fn undeclared_layer_directory_is_detected() {
    let fixture = ScaffoldFixture::new();
    let extra = fixture.root.join("docs/layers/undeclared");
    fs::create_dir(&extra).unwrap();
    assert!(extra.is_dir());
    fixture.assert_rejected("--check", "Layer scaffold directory undeclared:");
}

#[test]
fn generation_preserves_authored_architecture() {
    let fixture = ScaffoldFixture::new();
    let path = fixture.root.join("docs/ARCHITECTURE.md");
    let original = format!(
        "An authored preamble.\n\n{}\nAn authored ending.\n",
        fs::read_to_string(&path).unwrap()
    );
    fs::write(&path, &original).unwrap();
    let mut registry = fixture.registry();
    registry["layers"][0]["title"] = json!("Revised document title");
    fixture.write_registry(&registry);
    assert_success(fixture.run("--write"));
    let actual = fs::read_to_string(path).unwrap();
    let begin = "<!-- BEGIN GENERATED LAYER MAP -->";
    let end = "<!-- END GENERATED LAYER MAP -->";
    assert_eq!(
        actual.split_once(begin).unwrap().0,
        original.split_once(begin).unwrap().0
    );
    assert_eq!(
        actual.split_once(end).unwrap().1,
        original.split_once(end).unwrap().1
    );
    assert!(actual.contains("Revised document title"));
}
