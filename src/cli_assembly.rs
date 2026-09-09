//! Component assembly derives one executable CLI from bounded local sources and owned state scopes.

use crate::authoring_error::{AuthoringError, authoring_limit_error};
use crate::cli_definition::{
    CliBehaviorBinding, CliContext, CliContextParent, CliDefinition, CliMockExpression,
    validate_cli_name,
};
use crate::document_path::DocumentSegment;
use crate::document_value::{
    DocumentValue, MAX_CHECKPOINT_BYTES, MAX_DOCUMENT_BYTES, decode_authoring_json,
};
use crate::session_storage::read_regular_file_bounded;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// A component mount is a namespace identity, distinct from a source filename or component identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CliComponentMount(String);

/// A reusable component carries a plain definition and explicit required component identities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliComponentDocument {
    pub format: String,
    pub requires: Vec<String>,
    pub definition: CliDefinition,
}

/// A component snapshot retains exact document values and observed source identity, not authority.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliComponentSnapshot {
    pub source: String,
    pub source_sha256: String,
    pub component_sha256: String,
    pub component: CliComponentDocument,
}

/// Assembly sources are embedded so activation and recovery never depend on the original files.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliAssemblySources {
    pub components: BTreeMap<CliComponentMount, CliComponentSnapshot>,
}

/// Decode present assembly sources strictly; only an absent field denotes a plain definition.
pub(crate) fn deserialize_assembly_sources<'de, D: serde::Deserializer<'de>>(
    input: D,
) -> Result<Option<CliAssemblySources>, D::Error> {
    CliAssemblySources::deserialize(input).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CliAssemblyEntry {
    mount: CliComponentMount,
    source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CliAssemblyManifest {
    format: String,
    id: String,
    description: String,
    components: Vec<CliAssemblyEntry>,
}

fn cli_assembly_error(message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        "INVALID_CLI_ASSEMBLY",
        message,
        "Repair the component documents or assembly manifest, then assemble explicitly.",
    )
}

fn component_collision(message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        "CLI_COMPONENT_COLLISION",
        message,
        "Choose unique component identities, mounts, and derived context and operation identities.",
    )
}

fn component_dependency(message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        "CLI_COMPONENT_DEPENDENCY",
        message,
        "Supply each required component identity exactly once and remove cyclic or duplicate requirements.",
    )
}

fn validate_component_count(count: usize) -> Result<(), AuthoringError> {
    if !(1..=32).contains(&count) {
        return Err(authoring_limit_error(
            "CLI assembly requires 1 through 32 components.",
        ));
    }
    Ok(())
}

fn canonical_component_digest(component: &CliComponentDocument) -> Result<String, AuthoringError> {
    let document = DocumentValue::parse_document(&serde_json::to_string(component).unwrap())?;
    Ok(format!(
        "{:x}",
        Sha256::digest(document.compact_document_json().as_bytes())
    ))
}

fn valid_source_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn validate_component_document(component: &CliComponentDocument) -> Result<(), AuthoringError> {
    if component.format != "cli-component-v1"
        || component.definition.format != "cli-definition-v1"
        || component.definition.assembly.is_some()
    {
        return Err(cli_assembly_error(
            "CLI component requires one plain definition; nested assembly is unsupported.",
        ));
    }
    component.definition.validate_cli_definition()?;
    let mut requirements = BTreeSet::new();
    for required in &component.requires {
        validate_cli_name(required)?;
        if required == &component.definition.id || !requirements.insert(required) {
            return Err(component_dependency(format!(
                "CLI component {} has a self or duplicate requirement: {required}",
                component.definition.id
            )));
        }
    }
    if component.requires.len() > 31 {
        return Err(authoring_limit_error(
            "CLI component exceeds 31 dependencies.",
        ));
    }
    Ok(())
}

fn validate_component_graph(sources: &CliAssemblySources) -> Result<(), AuthoringError> {
    validate_component_count(sources.components.len())?;
    let mut components = BTreeMap::new();
    for (mount, snapshot) in &sources.components {
        validate_cli_name(&mount.0)?;
        if mount.0 == "root" || mount.0 == ".." {
            return Err(component_collision(format!(
                "CLI component mount is reserved: {}",
                mount.0
            )));
        }
        validate_component_document(&snapshot.component)?;
        if snapshot.source.is_empty()
            || !valid_source_digest(&snapshot.source_sha256)
            || snapshot.component_sha256 != canonical_component_digest(&snapshot.component)?
        {
            return Err(cli_assembly_error(
                "CLI component snapshot source or content identity is invalid.",
            ));
        }
        let id = snapshot.component.definition.id.as_str();
        if components.insert(id, &snapshot.component).is_some() {
            return Err(component_collision(format!(
                "CLI component identity is duplicated: {id}"
            )));
        }
    }
    fn visit<'a>(
        id: &'a str,
        components: &BTreeMap<&'a str, &'a CliComponentDocument>,
        visiting: &mut BTreeSet<&'a str>,
        visited: &mut BTreeSet<&'a str>,
    ) -> Result<(), AuthoringError> {
        if visited.contains(id) {
            return Ok(());
        }
        let component = components.get(id).ok_or_else(|| {
            component_dependency(format!("CLI required component is missing: {id}"))
        })?;
        if !visiting.insert(id) {
            return Err(component_dependency(format!(
                "CLI component dependency cycle reaches: {id}"
            )));
        }
        for required in &component.requires {
            visit(required, components, visiting, visited)?;
        }
        visiting.remove(id);
        visited.insert(id);
        Ok(())
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in components.keys() {
        visit(id, &components, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn scope_component_path(path: &mut Vec<DocumentSegment>, mount: &CliComponentMount) {
    path.insert(
        0,
        DocumentSegment::Key {
            key: mount.0.clone(),
        },
    );
}

fn scope_component_expression(expression: &mut CliMockExpression, mount: &CliComponentMount) {
    if let CliMockExpression::State { path } = expression {
        scope_component_path(path, mount);
    }
}

fn expand_component_sources(
    id: &str,
    description: &str,
    sources: &CliAssemblySources,
) -> Result<CliDefinition, AuthoringError> {
    validate_component_graph(sources)?;
    let mut contexts = BTreeMap::from([(
        "root".into(),
        CliContext {
            parent: CliContextParent::Root(()),
            help: description.into(),
            related: Vec::new(),
            commands: BTreeMap::new(),
        },
    )]);
    let mut state = BTreeMap::new();
    let mut operations = BTreeSet::new();
    for (mount, snapshot) in &sources.components {
        let definition = &snapshot.component.definition;
        let names: BTreeMap<_, _> = definition
            .contexts
            .keys()
            .map(|local| {
                let name = if local == "root" {
                    mount.0.clone()
                } else {
                    format!("{}.{}", mount.0, local)
                };
                (local.clone(), name)
            })
            .collect();
        state.insert(mount.0.clone(), definition.mock_state.clone());
        for (local, original) in &definition.contexts {
            let mut context = original.clone();
            context.parent = match &original.parent {
                CliContextParent::Root(()) => CliContextParent::Context("root".into()),
                CliContextParent::Context(parent) => {
                    CliContextParent::Context(names[parent].clone())
                }
            };
            context.related = original
                .related
                .iter()
                .map(|related| names[related].clone())
                .collect();
            for command in context.commands.values_mut() {
                command.id = format!("{}.{}", mount.0, command.id);
                if !operations.insert(command.id.clone()) {
                    return Err(component_collision(format!(
                        "CLI assembled operation identity collides: {}",
                        command.id
                    )));
                }
                if let CliBehaviorBinding::Simulated { steps, output } = &mut command.binding {
                    for step in steps {
                        scope_component_path(&mut step.path, mount);
                        scope_component_expression(&mut step.value, mount);
                    }
                    scope_component_expression(output, mount);
                }
            }
            if contexts.insert(names[local].clone(), context).is_some() {
                return Err(component_collision(format!(
                    "CLI assembled context identity collides: {}",
                    names[local]
                )));
            }
        }
    }
    // Validate the ordinary executable model first, without recursive assembly validation.
    let mut definition = CliDefinition {
        format: "cli-definition-v1".into(),
        id: id.into(),
        description: description.into(),
        contexts,
        mock_state: DocumentValue::Object(state),
        assembly: None,
    };
    definition.validate_cli_definition()?;
    definition.format = "cli-definition-v2".into();
    definition.assembly = Some(sources.clone());
    DocumentValue::parse_document(&serde_json::to_string(&definition).unwrap())?;
    Ok(definition)
}

/// Resolve a candidate assembly manifest once; the caller owns candidate publication and receipt replay.
pub fn assemble_cli_document(candidate: &DocumentValue) -> Result<DocumentValue, AuthoringError> {
    let manifest: CliAssemblyManifest = decode_authoring_json(
        candidate.compact_document_json().as_bytes(),
        MAX_DOCUMENT_BYTES,
        "INVALID_CLI_ASSEMBLY",
    )?;
    if manifest.format != "cli-assembly-v1" {
        return Err(cli_assembly_error(
            "CLI assembly manifest format is unsupported.",
        ));
    }
    validate_cli_name(&manifest.id)?;
    validate_component_count(manifest.components.len())?;
    let mut sources = CliAssemblySources {
        components: BTreeMap::new(),
    };
    let mut source_bytes = 0usize;
    for entry in manifest.components {
        validate_cli_name(&entry.mount.0)?;
        if entry.source.is_empty() {
            return Err(cli_assembly_error("CLI component source path is empty."));
        }
        if sources.components.contains_key(&entry.mount) {
            return Err(component_collision(format!(
                "CLI component mount is duplicated: {}",
                entry.mount.0
            )));
        }
        let bytes = read_regular_file_bounded(
            Path::new(&entry.source),
            MAX_DOCUMENT_BYTES,
            "CLI_COMPONENT_SOURCE",
        )?;
        source_bytes += bytes.len();
        if source_bytes > MAX_CHECKPOINT_BYTES {
            return Err(authoring_limit_error("CLI assembly sources exceed 8 MiB."));
        }
        let component: CliComponentDocument =
            decode_authoring_json(&bytes, MAX_DOCUMENT_BYTES, "INVALID_CLI_ASSEMBLY")?;
        validate_component_document(&component)?;
        let component_sha256 = canonical_component_digest(&component)?;
        sources.components.insert(
            entry.mount,
            CliComponentSnapshot {
                source: entry.source,
                source_sha256: format!("{:x}", Sha256::digest(&bytes)),
                component_sha256,
                component,
            },
        );
    }
    let definition = expand_component_sources(&manifest.id, &manifest.description, &sources)?;
    DocumentValue::parse_document(&serde_json::to_string(&definition).unwrap())
}

/// Validate an assembled definition against embedded sources without opening their provenance paths.
pub fn validate_cli_assembly(definition: &CliDefinition) -> Result<(), AuthoringError> {
    let sources = definition.assembly.as_ref().ok_or_else(|| {
        cli_assembly_error("CLI assembled definition has no embedded components.")
    })?;
    let expected = expand_component_sources(&definition.id, &definition.description, sources)?;
    if definition != &expected {
        return Err(cli_assembly_error(
            "CLI assembled executable content disagrees with its embedded components.",
        ));
    }
    Ok(())
}

/// Validate component state ownership on recovery while allowing each component any JSON root value.
pub fn validate_assembled_state(
    definition: &CliDefinition,
    state: &DocumentValue,
) -> Result<(), AuthoringError> {
    if let Some(sources) = &definition.assembly {
        let DocumentValue::Object(values) = state else {
            return Err(cli_assembly_error(
                "CLI assembled state requires an object of component scopes.",
            ));
        };
        if !values
            .keys()
            .map(String::as_str)
            .eq(sources.components.keys().map(|mount| mount.0.as_str()))
        {
            return Err(cli_assembly_error(
                "CLI assembled state scopes disagree with component mounts.",
            ));
        }
    }
    Ok(())
}
