//! Native applications embed configuration and share the existing run frontend and provider boundary.
use crate::authoring_error::AuthoringError;
use crate::authoring_protocol::SessionRevision;
use crate::cli_composition::parse_cli_interface_source;
use crate::cli_definition::{CliBehaviorBinding, CliCapabilityId, CliConnectedProvider};
use crate::cli_http_get::{
    JsonHttpGetGrants, validate_http_resource_path, validate_http_response_requirement,
};
use crate::cli_interface::CliInterfaceOrigin;
use crate::cli_operator_settings::{CliEndpointSource, CliOperatorProfile, CliOperatorSettings};
use crate::cli_run_frontend::{
    execute_run_tokens, run_cli_stream, run_cli_terminal, run_error_exit,
};
use crate::cli_run_routes::{CliRunRoutes, run_usage_error};
use crate::cli_run_session::CliRunSession;
use crate::cli_view_frontend::CliOutputSelection;
use crate::document_value::{
    DocumentValue, MAX_CHECKPOINT_BYTES, MAX_DOCUMENT_BYTES, MAX_REQUEST_BYTES,
};
use crate::session_storage::read_regular_file_bounded;
use crate::terminal_input::TerminalToken;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliHttpResource {
    pub path: String,
    pub require: DocumentValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliHttpLaunchProfile {
    pub option: String,
    pub environment: String,
    pub resources: BTreeMap<String, CliHttpResource>,
}

impl CliHttpLaunchProfile {
    fn validate_http_launch_profile(&self) -> Result<(), AuthoringError> {
        if self.resources.len() > 32
            || !self.option.starts_with("--")
            || self.option.len() < 3
            || !self.option[2..]
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'-')
            || [
                "--help",
                "--json",
                "--table",
                "--view",
                "--events",
                "--session",
                "--config",
            ]
            .contains(&self.option.as_str())
            || self.environment.is_empty()
            || self.environment.len() > 128
            || !self
                .environment
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(run_usage_error(
                "Invalid application HTTP endpoint option/environment or resource limits.",
            ));
        }
        for (capability, resource) in &self.resources {
            crate::cli_definition::validate_cli_name(capability)?;
            validate_http_resource_path(&resource.path)?;
            validate_http_response_requirement(&resource.require)?;
        }
        Ok(())
    }

    /// Construct the complete validated HTTP grant set for one selected endpoint, without a request.
    pub fn grants_for_endpoint(
        &self,
        endpoint: Option<&str>,
    ) -> Result<JsonHttpGetGrants, AuthoringError> {
        let mut grants = JsonHttpGetGrants::default();
        if let Some(base) = endpoint {
            crate::cli_http_get::validate_loopback_http_base(base)?;
            for (capability, resource) in &self.resources {
                grants.grant_json_http_get(
                    CliCapabilityId(capability.clone()),
                    base,
                    &resource.path,
                    resource.require.clone(),
                )?;
            }
        }
        Ok(grants)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliApplicationOutput {
    Json,
    Table,
}

/// A profile configures process presentation and authority; it supplies no executable code.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliApplicationProfile {
    pub format: String,
    pub default_output: CliApplicationOutput,
    pub control_aliases: BTreeMap<String, String>,
    pub context_help: bool,
    /// HTTP configuration is optional; local mock applications need no endpoint or settings file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http: Option<CliHttpLaunchProfile>,
    pub exit_codes: BTreeMap<String, i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<CliOperatorProfile>,
}

impl CliApplicationProfile {
    pub fn parse_application_profile(source: &[u8]) -> Result<Self, AuthoringError> {
        if source.len() > MAX_DOCUMENT_BYTES {
            return Err(run_usage_error("Application profile exceeds one MiB."));
        }
        let text = std::str::from_utf8(source)
            .map_err(|_| run_usage_error("Application profile must be UTF-8."))?;
        DocumentValue::parse_document(text)?;
        let profile: Self = serde_json::from_str(text)
            .map_err(|error| run_usage_error(format!("Invalid application profile: {error}")))?;
        if profile.format != "cli-application-v1"
            || profile.exit_codes.len() > 64
            || profile.exit_codes.iter().any(|(key, code)| {
                key.is_empty()
                    || key.len() > 64
                    || !key.bytes().all(|b| b.is_ascii_uppercase() || b == b'_')
                    || !(1..=125).contains(code)
            })
        {
            return Err(run_usage_error(
                "Invalid application format or error status mapping.",
            ));
        }
        if let Some(http) = &profile.http {
            http.validate_http_launch_profile()?;
        }
        Ok(profile)
    }

    pub fn validate_application_definition(&self, source: &[u8]) -> Result<(), AuthoringError> {
        let active = parse_cli_interface_source(
            source,
            None,
            CliInterfaceOrigin {
                intent_text: "Validate configured application.".into(),
                revision: SessionRevision(0),
            },
        )?;
        let mut routes = CliRunRoutes::from_cli_definition(&active.definition)?;
        routes.configure_operator_routes(
            &active.definition,
            self.operator.clone(),
            self.http.is_some(),
        )?;
        routes.configure_control_aliases(&active.definition, self.control_aliases.clone())?;
        let required: BTreeSet<_> = active
            .definition
            .contexts
            .values()
            .flat_map(|context| context.commands.values())
            .filter_map(|command| {
                if let CliBehaviorBinding::Connected {
                    provider: CliConnectedProvider::JsonHttpGet,
                    capability,
                } = &command.binding
                {
                    Some(capability.0.clone())
                } else {
                    None
                }
            })
            .collect();
        if required
            != self
                .http
                .iter()
                .flat_map(|http| http.resources.keys().cloned())
                .collect()
        {
            return Err(run_usage_error(
                "Application HTTP resources must match the definition's HTTP capabilities exactly.",
            ));
        }
        Ok(())
    }
}

fn run_application(
    source: &[u8],
    profile: &CliApplicationProfile,
    arguments: impl Iterator<Item = OsString>,
) -> Result<i32, AuthoringError> {
    profile.validate_application_definition(source)?;
    let mut arguments = arguments.peekable();
    let mut endpoint = None;
    let mut checkpoint = None;
    let mut config = None;
    let mut structured = false;
    let mut output = None;
    let mut words = Vec::new();
    let mut size = 0usize;
    let mut literal = false;
    while let Some(argument) = arguments.next() {
        let text = argument
            .into_string()
            .map_err(|_| run_usage_error("Application arguments must be UTF-8."))?;
        size = size.saturating_add(text.len() + 1);
        if size > MAX_REQUEST_BYTES {
            return Err(run_usage_error("Application arguments exceed two MiB."));
        }
        let mut value = || {
            arguments
                .next()
                .and_then(|value| value.into_string().ok())
                .ok_or_else(|| run_usage_error(format!("{text} requires a UTF-8 value.")))
        };
        if !literal {
            if profile
                .http
                .as_ref()
                .is_some_and(|http| text == http.option)
            {
                if endpoint.is_some() {
                    return Err(run_usage_error("Duplicate endpoint option."));
                }
                endpoint = Some(value()?);
                continue;
            }
            match text.as_str() {
                "--json" | "--table" | "--view" => {
                    if output.is_some() {
                        return Err(run_usage_error("Choose one output mode."));
                    }
                    output = Some(match text.as_str() {
                        "--json" => CliOutputSelection::Json,
                        "--table" => CliOutputSelection::CommandView,
                        _ => CliOutputSelection::NamedView(value()?),
                    });
                    continue;
                }
                "--events" => {
                    if structured {
                        return Err(run_usage_error("Duplicate --events."));
                    }
                    structured = true;
                    continue;
                }
                "--session" => {
                    if checkpoint.is_some() {
                        return Err(run_usage_error("Duplicate --session."));
                    }
                    checkpoint = Some(PathBuf::from(value()?));
                    continue;
                }
                "--config" => {
                    if config.is_some() || profile.operator.is_none() || profile.http.is_none() {
                        return Err(run_usage_error(
                            "--config requires an HTTP-enabled operator profile and may appear only once.",
                        ));
                    }
                    let path = value()?;
                    if path.is_empty() {
                        return Err(run_usage_error("--config requires a nonempty file path."));
                    }
                    config = Some(PathBuf::from(path));
                    continue;
                }
                "--" => literal = true,
                "-h" => {
                    words.push(TerminalToken {
                        start: 0,
                        end: 6,
                        text: "--help".into(),
                    });
                    continue;
                }
                _ => {}
            }
        }
        words.push(TerminalToken {
            start: 0,
            end: text.len(),
            text,
        });
    }
    let explicit_endpoint = endpoint.is_some();
    let endpoint = match endpoint {
        Some(endpoint) => Some(endpoint),
        None => match &profile.http {
            Some(http) => std::env::var(&http.environment)
                .map(|value| if value.is_empty() { None } else { Some(value) })
                .or_else(|error| match error {
                    std::env::VarError::NotPresent => Ok(None),
                    _ => Err(run_usage_error("Endpoint environment must be UTF-8.")),
                })?,
            None => None,
        },
    };
    let grants = profile
        .http
        .as_ref()
        .map(|http| http.grants_for_endpoint(endpoint.as_deref()))
        .transpose()?
        .unwrap_or_default();
    let mut session = CliRunSession::open_embedded_cli_run(source, checkpoint.as_deref(), grants)?;
    session.routes.configure_operator_routes(
        &session.active_interface().definition.clone(),
        profile.operator.clone(),
        profile.http.is_some(),
    )?;
    session.routes.configure_control_aliases(
        &session.active_interface().definition.clone(),
        profile.control_aliases.clone(),
    )?;
    if profile.operator.is_some()
        && let Some(http) = &profile.http
    {
        let settings = CliOperatorSettings::open_operator_settings(
            &session.active_interface().definition.id,
            http.clone(),
            config,
            endpoint.map(|url| {
                (
                    url,
                    if explicit_endpoint {
                        CliEndpointSource::Option
                    } else {
                        CliEndpointSource::Environment
                    },
                )
            }),
        )?;
        session.runtime.replace_runtime_http_grants(
            settings
                .http
                .grants_for_endpoint(settings.endpoint.as_deref())?,
        );
        session.endpoint_settings = Some(settings);
    }
    session.context_help = profile.context_help;
    session.capability_help = profile.http.as_ref().map(|http| {
        format!(
            "Endpoint: {} <loopback-url> or {}",
            http.option, http.environment
        )
    });
    let endpoint_option = profile
        .http
        .as_ref()
        .map(|http| format!("{} <loopback-url>, ", http.option))
        .unwrap_or_default();
    session.launch_help = Some(format!(
        "Options: {endpoint_option}--table, --json (raw result), --view <name>, --events (runtime events), --session <file>. Use -- before literal command arguments."
    ));
    if session.endpoint_settings.is_some() {
        session
            .launch_help
            .as_mut()
            .unwrap()
            .push_str(" --config <file> selects saved management settings.");
    }
    session.exit_codes = profile.exit_codes.clone();
    session.output_selection = output.unwrap_or(match profile.default_output {
        CliApplicationOutput::Json => CliOutputSelection::Json,
        CliApplicationOutput::Table => CliOutputSelection::CommandView,
    });
    if let CliOutputSelection::NamedView(name) = &session.output_selection {
        crate::cli_view_frontend::require_output_view(
            &session.active_interface().definition,
            name,
        )?;
    }
    let result = if !words.is_empty() {
        execute_run_tokens(
            &mut session,
            &words,
            true,
            structured,
            &mut io::stdout().lock(),
            &mut io::stderr().lock(),
        )
        .map(|(code, _)| code)
    } else if io::stdin().is_terminal() && io::stdout().is_terminal() {
        run_cli_terminal(&mut session, structured)
    } else {
        run_cli_stream(
            &mut session,
            structured,
            &mut io::stdin().lock(),
            &mut io::stdout().lock(),
            &mut io::stderr().lock(),
        )
    };
    result.map_err(|error| {
        AuthoringError::new(
            "RUN_DELIVERY_FAILED",
            error.to_string(),
            "Inspect :status before retrying; a read receipt may already be retained.",
        )
    })
}

/// A consumer's main function only supplies its embedded data and process arguments.
pub fn launch_configured_cli(
    source: &[u8],
    profile_bytes: &[u8],
    arguments: impl Iterator<Item = OsString>,
) -> i32 {
    let arguments: Vec<_> = arguments.collect();
    let structured = arguments
        .iter()
        .take_while(|argument| *argument != "--")
        .any(|argument| argument == "--events");
    let profile = match CliApplicationProfile::parse_application_profile(profile_bytes) {
        Ok(profile) => profile,
        Err(error) => {
            write_application_launch_error(&error, structured);
            return run_error_exit(&error);
        }
    };
    match run_application(source, &profile, arguments.into_iter()) {
        Ok(code) => code,
        Err(error) => {
            write_application_launch_error(&error, structured);
            profile
                .exit_codes
                .get(&error.code)
                .copied()
                .unwrap_or_else(|| run_error_exit(&error))
        }
    }
}

fn write_application_launch_error(error: &AuthoringError, structured: bool) {
    if structured {
        let response = crate::authoring_protocol::AuthoringResponse::operation_failure(
            None,
            None,
            None,
            error.clone(),
        );
        if let Ok(bytes) = crate::authoring_runtime::checked_response_bytes(&response) {
            let _ = io::stderr().write_all(&bytes);
        }
    } else {
        let _ = writeln!(
            io::stderr(),
            "{}",
            crate::cli_run_frontend::escape_run_metadata(&error.to_string())
        );
    }
}

/// Run the same application profile from transferred files without compiling a named executable.
pub fn launch_cli_application_files(mut arguments: impl Iterator<Item = OsString>) -> i32 {
    let arguments: Vec<_> = arguments.by_ref().collect();
    if arguments.len() == 1 && arguments[0] == "--help" {
        let _ = writeln!(
            io::stdout(),
            "cli app <definition.json> <application.json> [context ... command arguments ...]\nRun a configured native application from transferred data. Omit domain words for a contextual shell.\nApplication options: --json, --table, --view <name>, --events, --session <file>, and the profile's endpoint option."
        );
        return 0;
    }
    let structured = arguments
        .iter()
        .skip(2)
        .take_while(|argument| *argument != "--")
        .any(|argument| argument == "--events");
    let mut arguments = arguments.into_iter();
    let sources = (|| {
        let definition = arguments.next().ok_or_else(|| {
            run_usage_error("Use cli app <definition.json> <application.json> [arguments ...].")
        })?;
        let profile = arguments
            .next()
            .ok_or_else(|| run_usage_error("Application profile path is required."))?;
        Ok::<_, AuthoringError>((
            read_regular_file_bounded(
                Path::new(&definition),
                MAX_CHECKPOINT_BYTES,
                "INVALID_CLI_DEFINITION",
            )?,
            read_regular_file_bounded(Path::new(&profile), MAX_DOCUMENT_BYTES, "INVALID_REQUEST")?,
        ))
    })();
    match sources {
        Ok((source, profile)) => launch_configured_cli(&source, &profile, arguments),
        Err(error) => {
            write_application_launch_error(&error, structured);
            run_error_exit(&error)
        }
    }
}
