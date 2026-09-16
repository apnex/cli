//! Operator endpoint selection stays separate from portable interfaces and is saved only explicitly.

use crate::authoring_error::AuthoringError;
use crate::cli_application::CliHttpLaunchProfile;
use crate::cli_definition::CliDefinition;
use crate::cli_http_get::{JsonHttpGetGrants, validate_loopback_http_base};
use crate::cli_run_routes::run_usage_error;
use crate::session_storage::{SessionStorage, read_regular_file_bounded};
use crate::storage_faults::StorageFaultControl;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) const CLI_ENDPOINT_ACTIONS: [(&str, &str); 5] = [
    ("show", "Show current and saved endpoints"),
    ("set", "Select an endpoint URL"),
    ("clear", "Clear this session's endpoint"),
    ("save", "Save the current selection"),
    ("load", "Load the saved selection"),
];

/// Compact operator presentation explicitly identifies equivalent commands and listing fallbacks.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliOperatorProfile {
    pub command_aliases: BTreeMap<String, String>,
    pub context_listing: Vec<String>,
}

impl CliOperatorProfile {
    /// Reject misleading aliases before their commands can be collapsed in human discovery.
    pub fn validate_operator_profile(
        &self,
        definition: &CliDefinition,
    ) -> Result<(), AuthoringError> {
        if matches!(definition.id.as_str(), "." | "..") {
            return Err(run_usage_error(
                "Operator application identity must be a distinct configuration directory name.",
            ));
        }
        let commands: BTreeMap<_, _> = definition
            .contexts
            .values()
            .flat_map(|context| context.commands.values())
            .map(|command| (command.id.as_str(), command))
            .collect();
        if self.command_aliases.len() > 256
            || self.context_listing.len() > 16
            || self.context_listing.iter().collect::<BTreeSet<_>>().len()
                != self.context_listing.len()
        {
            return Err(run_usage_error(
                "Operator aliases or listing words exceed their limits or repeat.",
            ));
        }
        for word in &self.context_listing {
            crate::cli_definition::validate_cli_name(word)?;
        }
        for (alias, target) in &self.command_aliases {
            let pair = commands
                .get(alias.as_str())
                .zip(commands.get(target.as_str()));
            let Some((source, target_command)) = pair else {
                return Err(run_usage_error(
                    "Operator aliases must reference declared command IDs.",
                ));
            };
            if alias == target
                || self.command_aliases.contains_key(target)
                || source.view != target_command.view
                || serde_json::to_value(&source.binding).unwrap()
                    != serde_json::to_value(&target_command.binding).unwrap()
                || serde_json::to_value(&source.parameters).unwrap()
                    != serde_json::to_value(&target_command.parameters).unwrap()
            {
                return Err(run_usage_error(
                    "Operator aliases require identical behavior, parameters, and output view without alias chains.",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedManagementSettings {
    format: String,
    application: String,
    endpoint: Option<String>,
}

/// Endpoint origin describes selection, never observed server reachability.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliEndpointSource {
    Unconfigured,
    Option,
    Environment,
    Saved,
    Session,
}

/// Local management settings are process-owned; only an explicit save updates the user's file.
pub struct CliOperatorSettings {
    pub profile: CliOperatorProfile,
    pub endpoint: Option<String>,
    pub source: CliEndpointSource,
    pub http: CliHttpLaunchProfile,
    application: String,
    path: Option<PathBuf>,
    observed_bytes: Option<Vec<u8>>,
    saved_endpoint: Option<String>,
}

fn settings_error(message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        "INVALID_MANAGEMENT_SETTINGS",
        message,
        "Inspect the management settings file, or select a separate file with --config <file>.",
    )
}

fn default_settings_path(application: &str) -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".config"))
        })
        .map(|root| {
            root.join("programmable-cli")
                .join(application)
                .join("management.json")
        })
}

fn read_settings_bytes(path: &Path) -> Result<Option<Vec<u8>>, AuthoringError> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        _ => read_regular_file_bounded(path, 4096, "INVALID_MANAGEMENT_SETTINGS").map(Some),
    }
}

fn decode_settings_endpoint(
    application: &str,
    bytes: Option<&[u8]>,
) -> Result<Option<String>, AuthoringError> {
    let Some(bytes) = bytes else {
        return Ok(None);
    };
    let document: SavedManagementSettings = serde_json::from_slice(bytes)
        .map_err(|error| settings_error(format!("Invalid management settings: {error}")))?;
    if document.format != "cli-management-v1" || document.application != application {
        return Err(settings_error(
            "Management settings format or application identity does not match.",
        ));
    }
    document
        .endpoint
        .as_deref()
        .map(|url| validate_loopback_http_base(url).map(str::to_owned))
        .transpose()
}

impl CliOperatorSettings {
    /// Load saved preferences without performing a request; explicit launch selection takes precedence.
    pub fn open_operator_settings(
        application: &str,
        profile: CliOperatorProfile,
        http: CliHttpLaunchProfile,
        path: Option<PathBuf>,
        selected: Option<(String, CliEndpointSource)>,
    ) -> Result<Self, AuthoringError> {
        crate::cli_definition::validate_cli_name(application)?;
        if matches!(application, "." | "..") {
            return Err(settings_error(
                "Management application identity cannot be a directory traversal component.",
            ));
        }
        let path = path.or_else(|| default_settings_path(application));
        if path
            .as_ref()
            .is_some_and(|path| path.as_os_str().is_empty() || path.file_name().is_none())
        {
            return Err(settings_error("Management settings require a file path."));
        }
        let observed_bytes = path
            .as_deref()
            .map(read_settings_bytes)
            .transpose()?
            .flatten();
        let saved_endpoint = decode_settings_endpoint(application, observed_bytes.as_deref())?;
        let (endpoint, source) = match selected {
            Some((url, origin)) => (Some(validate_loopback_http_base(&url)?.to_owned()), origin),
            None if saved_endpoint.is_some() => (saved_endpoint.clone(), CliEndpointSource::Saved),
            None => (None, CliEndpointSource::Unconfigured),
        };
        Ok(Self {
            profile,
            endpoint,
            source,
            http,
            application: application.to_owned(),
            path,
            observed_bytes,
            saved_endpoint,
        })
    }

    /// Validate the whole endpoint capability set before replacing any process authority.
    pub fn select_operator_endpoint(
        &mut self,
        endpoint: Option<&str>,
    ) -> Result<JsonHttpGetGrants, AuthoringError> {
        let endpoint = endpoint
            .map(|url| validate_loopback_http_base(url).map(str::to_owned))
            .transpose()?;
        let grants = self.http.grants_for_endpoint(endpoint.as_deref())?;
        self.endpoint = endpoint;
        self.source = CliEndpointSource::Session;
        Ok(grants)
    }

    /// A one-shot selection is acknowledged only after its explicit save succeeds.
    pub(crate) fn select_persistent_endpoint(
        &mut self,
        endpoint: Option<&str>,
    ) -> Result<JsonHttpGetGrants, AuthoringError> {
        let previous = (self.endpoint.clone(), self.source);
        let grants = self.select_operator_endpoint(endpoint)?;
        if let Err(error) = self.save_operator_settings() {
            self.endpoint = previous.0;
            self.source = previous.1;
            return Err(error);
        }
        Ok(grants)
    }

    /// Explicit reload replaces the selection only after validating the saved document and grants.
    pub fn reload_operator_settings(&mut self) -> Result<JsonHttpGetGrants, AuthoringError> {
        let path = self.settings_path()?;
        let bytes = read_settings_bytes(path)?;
        let endpoint = decode_settings_endpoint(&self.application, bytes.as_deref())?;
        let grants = self.http.grants_for_endpoint(endpoint.as_deref())?;
        self.observed_bytes = bytes;
        self.saved_endpoint = endpoint.clone();
        self.endpoint = endpoint;
        self.source = if self.endpoint.is_some() {
            CliEndpointSource::Saved
        } else {
            CliEndpointSource::Unconfigured
        };
        Ok(grants)
    }

    fn settings_path(&self) -> Result<&Path, AuthoringError> {
        self.path.as_deref().ok_or_else(|| {
            settings_error(
                "No user configuration directory is available; launch with --config <file>.",
            )
        })
    }

    /// Atomically save under a stable lock, refusing another process's changes since load.
    pub fn save_operator_settings(&mut self) -> Result<(), AuthoringError> {
        let path = self.settings_path()?.to_owned();
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent).map_err(|error| {
            settings_error(format!(
                "Cannot create management settings directory: {error}"
            ))
        })?;
        let mut storage =
            SessionStorage::acquire_session_storage(&path, StorageFaultControl::default())?;
        let current = if read_settings_bytes(&path)?.is_some() {
            Some(storage.read_session_checkpoint()?)
        } else {
            None
        };
        if current != self.observed_bytes {
            return Err(AuthoringError::new(
                "MANAGEMENT_SETTINGS_CHANGED",
                "Saved management settings changed in another process.",
                "Reload saved settings with the endpoint control before changing and saving them again.",
            ));
        }
        let create = current.is_none();
        let mut bytes = serde_json::to_vec(&SavedManagementSettings {
            format: "cli-management-v1".into(),
            application: self.application.clone(),
            endpoint: self.endpoint.clone(),
        })
        .unwrap();
        bytes.push(b'\n');
        storage.publish_session_checkpoint(&bytes, create)?;
        self.observed_bytes = Some(bytes);
        self.saved_endpoint = self.endpoint.clone();
        Ok(())
    }

    /// Machine-readable local configuration explicitly distinguishes current and saved selections.
    pub fn operator_settings_status(&self) -> Value {
        json!({"endpoint":self.endpoint,"source":self.source,"saved_endpoint":self.saved_endpoint,
            "config_file":self.path.as_ref().map(|path| path.to_string_lossy()),
            "unsaved_changes":self.endpoint != self.saved_endpoint})
    }
}
