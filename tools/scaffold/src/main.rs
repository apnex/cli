//! Render and check the documentation owned by the provisional layer registry.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const LAYER_MAP_BEGIN: &str = "<!-- BEGIN GENERATED LAYER MAP -->";
const LAYER_MAP_END: &str = "<!-- END GENERATED LAYER MAP -->";
const LAYER_FILE_MARKER: &str =
    "<!-- Generated from docs/layers.json; edit that declaration instead. -->";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerScaffoldRegistry {
    format_version: u32,
    status: String,
    layers: Vec<LayerScaffoldDefinition>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerScaffoldDefinition {
    id: String,
    title: String,
    directory: String,
    vision: String,
    duty: String,
    rationale: String,
    consumes: Vec<String>,
    exposes: Vec<String>,
    depends_on: Vec<String>,
    principles: Vec<String>,
    non_goals: Vec<String>,
    success_criteria: Vec<String>,
    open_questions: Vec<String>,
}

fn read_layer_registry(root: &Path) -> Result<LayerScaffoldRegistry, String> {
    let source = fs::read_to_string(root.join("docs/layers.json")).map_err(|e| e.to_string())?;
    let registry: LayerScaffoldRegistry =
        serde_json::from_str(&source).map_err(|e| format!("Layer registry invalid: {e}"))?;
    if registry.format_version != 1
        || registry.status != "provisional"
        || registry.layers.is_empty()
    {
        return Err(
            "Layer registry invalid: expected version 1 and nonempty provisional layers".into(),
        );
    }
    let mut identifiers = BTreeSet::new();
    for layer in &registry.layers {
        if !layer.id.starts_with(|c: char| c.is_ascii_lowercase())
            || !layer
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(format!("Layer registry identifier invalid: {}", layer.id));
        }
        if !identifiers.insert(layer.id.as_str()) {
            return Err(format!("Layer registry identifier repeated: {}", layer.id));
        }
        if layer.directory != format!("docs/layers/{}", layer.id) {
            return Err(format!(
                "Layer registry directory invalid: {} must live under docs/layers",
                layer.id
            ));
        }
        for (field, value) in [
            ("title", &layer.title),
            ("vision", &layer.vision),
            ("duty", &layer.duty),
            ("rationale", &layer.rationale),
        ] {
            if value.trim().is_empty() || value.contains('\n') {
                return Err(format!("Layer registry text invalid: {}.{field}", layer.id));
            }
        }
        for (field, values) in [
            ("consumes", &layer.consumes),
            ("exposes", &layer.exposes),
            ("depends_on", &layer.depends_on),
            ("principles", &layer.principles),
            ("non_goals", &layer.non_goals),
            ("success_criteria", &layer.success_criteria),
            ("open_questions", &layer.open_questions),
        ] {
            let unique: BTreeSet<_> = values.iter().collect();
            if (field != "depends_on" && values.is_empty())
                || unique.len() != values.len()
                || values
                    .iter()
                    .any(|v| v.trim().is_empty() || v.contains('\n'))
            {
                return Err(format!("Layer registry list invalid: {}.{field}", layer.id));
            }
        }
    }
    let dependencies: BTreeMap<_, _> = registry
        .layers
        .iter()
        .map(|layer| (layer.id.as_str(), layer.depends_on.as_slice()))
        .collect();
    for layer in &registry.layers {
        for dependency in &layer.depends_on {
            if !identifiers.contains(dependency.as_str()) {
                return Err(format!(
                    "Layer dependency missing: {} -> {dependency}",
                    layer.id
                ));
            }
        }
    }
    let mut visited = BTreeSet::new();
    for identifier in &identifiers {
        check_layer_dependency_path(identifier, &dependencies, &mut Vec::new(), &mut visited)?;
    }
    let layer_root = root.join("docs/layers");
    if layer_root.exists() {
        for entry in fs::read_dir(layer_root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().map_err(|e| e.to_string())?.is_dir()
                && !identifiers.contains(name.as_str())
            {
                return Err(format!("Layer scaffold directory undeclared: {name}"));
            }
        }
    }
    Ok(registry)
}

fn check_layer_dependency_path<'a>(
    identifier: &'a str,
    dependencies: &BTreeMap<&'a str, &'a [String]>,
    ancestors: &mut Vec<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Result<(), String> {
    if ancestors.contains(&identifier) {
        ancestors.push(identifier);
        return Err(format!(
            "Layer dependency cycle: {}",
            ancestors.join(" -> ")
        ));
    }
    if visited.contains(identifier) {
        return Ok(());
    }
    ancestors.push(identifier);
    for dependency in dependencies[identifier] {
        check_layer_dependency_path(dependency, dependencies, ancestors, visited)?;
    }
    ancestors.pop();
    visited.insert(identifier);
    Ok(())
}

fn render_registry_bullets(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("- {value}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_layer_map(layers: &[LayerScaffoldDefinition]) -> String {
    let mut output = String::from(
        "| Responsibility | Duty | Consumes | Exposes | Depends on | Local record |\n\
         |---|---|---|---|---|---|\n",
    );
    for layer in layers {
        let dependencies = if layer.depends_on.is_empty() {
            "None".into()
        } else {
            layer
                .depends_on
                .iter()
                .map(|id| format!("`{id}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        output.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | [Vision](layers/{}/VISION.md), [responsibility](layers/{}/contract.md) |\n",
            layer.id, layer.duty.replace('|', "\\|"),
            layer.consumes.join("; ").replace('|', "\\|"),
            layer.exposes.join("; ").replace('|', "\\|"), dependencies, layer.id, layer.id,
        ));
    }
    output.push_str(
        "\nArrows point from a consumer to its declared dependency.\n\n```mermaid\nflowchart TD\n",
    );
    for layer in layers {
        output.push_str(&format!(
            "    {}[\"{}\"]\n",
            layer.id,
            layer.title.replace('"', "&quot;")
        ));
    }
    for layer in layers {
        for dependency in &layer.depends_on {
            output.push_str(&format!("    {} --> {dependency}\n", layer.id));
        }
    }
    output.push_str("```\n");
    output
}

fn render_layer_vision(layer: &LayerScaffoldDefinition) -> String {
    separate_document_sections(&format!(
        r#"{LAYER_FILE_MARKER}
# {title} vision

**Status: draft scope within the [project vision](../../../VISION.md).**
The declaration in the [layer registry](../../layers.json) owns this generated view.

## North star

> {vision}

## Purpose and boundaries

{rationale}

This scope excludes:

{non_goals}

## Success dimensions

These criteria are assessed separately and do not collapse into a score:

{success_criteria}

## Authority

The project owner holds this purpose through the parent vision.
Holding this draft does not approve an API, certify behavior, or grant execution authority.
Changes retain their rationale in the registry and the project record.

## Mechanics, rationale, and consequence

### Mechanics

The [responsibility record](contract.md) states the proposed concern, dependencies, and unresolved questions.

### Rationale

This local vision keeps the purpose discoverable from the component's own directory.

### Consequence of violation

An actor that treats a provisional scope as implemented behavior would rely on guarantees this document does not establish.
"#,
        title = layer.title,
        vision = layer.vision,
        rationale = layer.rationale,
        non_goals = render_registry_bullets(&layer.non_goals),
        success_criteria = render_registry_bullets(&layer.success_criteria),
    ))
}

fn render_layer_contract(layer: &LayerScaffoldDefinition) -> String {
    let dependencies = if layer.depends_on.is_empty() {
        "No other proposed layer is required by this responsibility.".into()
    } else {
        layer
            .depends_on
            .iter()
            .map(|id| format!("- [`{id}`](../{id}/contract.md)"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    separate_document_sections(&format!(
        r#"{LAYER_FILE_MARKER}
# {title} responsibility

**Maturity: provisional responsibility sketch; public APIs are not yet designed.**
This record elaborates the [local vision](VISION.md) at the altitude of the [target architecture](../../ARCHITECTURE.md).

## Duty

{duty}

## Consumes

{consumes}

## Exposes

{exposes}

These are semantic obligations, not implemented type signatures.

## Dependencies

{dependencies}

## Boundary and evidence

The following concerns are excluded:

{non_goals}

The following observations are required to substantiate this responsibility:

{success_criteria}

## Open questions

{open_questions}

## Mechanics, rationale, and consequence

### Mechanics

This view is generated from the layer registry.
The declared dependency graph is a proposal and is checked independently of any runtime implementation.

### Rationale

{rationale}
The architectural anchors for this responsibility are {principles}.

### Consequence of violation

Hidden ownership or dependencies prevent a new actor from reasoning locally about this concern.
"#,
        title = layer.title,
        duty = layer.duty,
        rationale = layer.rationale,
        consumes = render_registry_bullets(&layer.consumes),
        exposes = render_registry_bullets(&layer.exposes),
        non_goals = render_registry_bullets(&layer.non_goals),
        success_criteria = render_registry_bullets(&layer.success_criteria),
        open_questions = render_registry_bullets(&layer.open_questions),
        principles = layer
            .principles
            .iter()
            .map(|p| format!("`{p}`"))
            .collect::<Vec<_>>()
            .join(", "),
    ))
}

fn separate_document_sections(content: &str) -> String {
    if content
        .lines()
        .filter(|line| line.starts_with("## "))
        .count()
        < 5
    {
        return content.into();
    }
    let mut output = String::new();
    let mut seen_section = false;
    for line in content.lines() {
        if line.starts_with("## ") {
            if seen_section {
                output.push_str("---\n\n");
            }
            seen_section = true;
        }
        output.push_str(line);
        output.push('\n');
    }
    output
}

fn render_layer_scaffold(root: &Path, write: bool) -> Result<(), String> {
    let registry = read_layer_registry(root)?;
    let architecture_path = root.join("docs/ARCHITECTURE.md");
    let architecture = fs::read_to_string(&architecture_path).map_err(|e| e.to_string())?;
    if architecture.matches(LAYER_MAP_BEGIN).count() != 1
        || architecture.matches(LAYER_MAP_END).count() != 1
    {
        return Err(
            "Layer map markers invalid: architecture requires exactly one ordered marker pair"
                .into(),
        );
    }
    let (before, remainder) = architecture.split_once(LAYER_MAP_BEGIN).unwrap();
    let (_, after) = remainder
        .split_once(LAYER_MAP_END)
        .ok_or("Layer map markers invalid: marker order reversed")?;
    let mut outputs = vec![(
        architecture_path.clone(),
        format!(
            "{before}{LAYER_MAP_BEGIN}\n{}{LAYER_MAP_END}{after}",
            render_layer_map(&registry.layers),
        ),
    )];
    for layer in &registry.layers {
        outputs.push((
            root.join(&layer.directory).join("VISION.md"),
            render_layer_vision(layer),
        ));
        outputs.push((
            root.join(&layer.directory).join("contract.md"),
            render_layer_contract(layer),
        ));
    }
    let mut differences = Vec::new();
    for (path, expected) in &outputs {
        let current = match fs::read_to_string(path) {
            Ok(content) => Some(content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.to_string()),
        };
        if current.as_ref() == Some(expected) {
            continue;
        }
        if let Some(content) = &current {
            if path != &architecture_path && !content.contains(LAYER_FILE_MARKER) {
                return Err(format!(
                    "Layer scaffold ownership conflict: {} is not marked generated",
                    path.display()
                ));
            }
        }
        differences.push((path, expected));
    }
    if !write && !differences.is_empty() {
        return Err(differences
            .iter()
            .map(|(path, _)| {
                format!(
                    "Layer scaffold view stale: {}",
                    path.strip_prefix(root).unwrap().display(),
                )
            })
            .collect::<Vec<_>>()
            .join("\n"));
    }
    if write {
        for (path, expected) in &differences {
            fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            fs::write(path, expected).map_err(|e| e.to_string())?;
        }
        println!(
            "Layer scaffold rendered: {} files updated",
            differences.len()
        );
    } else {
        println!(
            "Layer scaffold verified: {} declared layers, {} synchronized views",
            registry.layers.len(),
            outputs.len()
        );
    }
    Ok(())
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let mode = args.next();
    let write = match mode.as_deref().and_then(|value| value.to_str()) {
        Some("--write") => true,
        Some("--check") => false,
        _ => {
            eprintln!("Usage: cli-layer-scaffold (--check | --write) [--root PATH]");
            std::process::exit(64);
        }
    };
    let root = match (args.next(), args.next(), args.next()) {
        (None, None, None) => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        (Some(flag), Some(path), None) if flag == "--root" => PathBuf::from(path),
        _ => {
            eprintln!("Usage: cli-layer-scaffold (--check | --write) [--root PATH]");
            std::process::exit(64);
        }
    };
    let result = root
        .canonicalize()
        .map_err(|e| e.to_string())
        .and_then(|root| render_layer_scaffold(&root, write));
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
