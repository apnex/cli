//! Run and document-preview surfaces share native output views and keep presentation outside receipts.

use crate::authoring_error::AuthoringError;
use crate::authoring_protocol::{AuthoringResponse, SessionRevision};
use crate::authoring_runtime::checked_response_bytes;
use crate::cli_composition::parse_cli_interface_source;
use crate::cli_definition::{CliCommand, CliDefinition};
use crate::cli_interface::CliInterfaceOrigin;
use crate::cli_output_view::{CliOutputPresentation, output_view_error};
use crate::cli_run_frontend::escape_run_metadata;
use crate::cli_run_routes::run_usage_error;
use crate::document_value::{DocumentValue, MAX_CHECKPOINT_BYTES, MAX_DOCUMENT_BYTES};
use crate::session_storage::read_regular_file_bounded;
use serde_json::{Value, json};
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;

/// Output selection is a process preference; changing it cannot change a persisted definition.
#[derive(Default)]
pub enum CliOutputSelection {
    #[default]
    Json,
    CommandView,
    NamedView(String),
}

/// Resolve table preferences before invoking a command, so selection errors cannot produce effects.
pub fn select_command_output_view(
    definition: &CliDefinition,
    command: &CliCommand,
    selection: &CliOutputSelection,
) -> Result<Option<String>, AuthoringError> {
    let name = match selection {
        CliOutputSelection::Json => return Ok(None),
        CliOutputSelection::CommandView => command.view.as_ref().ok_or_else(|| {
            output_view_error(
                "OUTPUT_VIEW_REQUIRED",
                "This command has no declared output view; choose --view or use JSON output.",
            )
        })?,
        CliOutputSelection::NamedView(name) => name,
    };
    require_output_view(definition, name)?;
    Ok(Some(name.clone()))
}

pub fn require_output_view<'a>(
    definition: &'a CliDefinition,
    name: &str,
) -> Result<&'a crate::cli_output_view::CliOutputView, AuthoringError> {
    definition
        .views
        .as_ref()
        .and_then(|views| views.get(name))
        .ok_or_else(|| {
            output_view_error(
                "UNKNOWN_OUTPUT_VIEW",
                format!("Output view does not exist: {name}"),
            )
        })
}

pub fn present_invocation_output(
    definition: &CliDefinition,
    name: &str,
    text: &str,
) -> Result<CliOutputPresentation, AuthoringError> {
    let document = DocumentValue::parse_document(text)?;
    require_output_view(definition, name)?.present_output_document(name, &document)
}

/// Add derived presentation only to the transport response; errors keep the successful invocation intact.
pub fn attach_output_presentation(
    definition: &CliDefinition,
    name: &str,
    response: &mut AuthoringResponse,
) {
    if response.status != "ok" {
        return;
    }
    let result = response.result.as_mut().expect("Successful result");
    let Some(text) = result["invocation"]["output_json_text"].as_str() else {
        return;
    };
    result["presentation"] = match present_invocation_output(definition, name, text) {
        Ok(presentation) => {
            let mut value =
                serde_json::to_value(presentation).expect("Output presentation serializes");
            value["status"] = json!("ok");
            value
        }
        Err(error) => json!({"status":"error", "view":name, "error":error}),
    };
    if checked_response_bytes(response).is_err() {
        response.result.as_mut().unwrap()["presentation"] = json!({"status":"error", "view":name, "error":output_view_error("OUTPUT_VIEW_LIMIT", "Presentation and invocation exceed the response size limit.")});
    }
}

/// Expose authored view definitions without fetching result data or invoking a provider.
pub fn discover_output_views(definition: &CliDefinition) -> Value {
    let mut text = String::new();
    if let Some(views) = &definition.views {
        for (id, view) in views {
            text.push_str(&format!("{}  {}\n", id, escape_run_metadata(&view.help)));
        }
    }
    if text.is_empty() {
        text.push_str("No output views are declared.\n");
    }
    json!({"help_text":text, "views":definition.views})
}

fn preview_output_document(
    arguments: impl Iterator<Item = OsString>,
    structured: &mut bool,
) -> Result<i32, AuthoringError> {
    let mut arguments: Vec<_> = arguments.collect();
    if arguments.first().is_some_and(|arg| arg == "--help") && arguments.len() == 1 {
        writeln!(io::stdout(), "cli render [--json] <spec.json> <view> <document.json>\nPreview an authored output view with native Rust projection and rendering; no binding is invoked.").map_err(|error| output_view_error("RUN_DELIVERY_FAILED", error.to_string()))?;
        return Ok(0);
    }
    if arguments.first().is_some_and(|arg| arg == "--json") {
        *structured = true;
        arguments.remove(0);
    }
    if arguments.len() != 3 {
        return Err(run_usage_error(
            "Use cli render [--json] <spec.json> <view> <document.json>.",
        ));
    }
    let view = arguments[1]
        .to_str()
        .ok_or_else(|| run_usage_error("View identifier must be UTF-8."))?;
    let definition = read_regular_file_bounded(
        Path::new(&arguments[0]),
        MAX_CHECKPOINT_BYTES,
        "INVALID_CLI_DEFINITION",
    )?;
    let active = parse_cli_interface_source(
        &definition,
        None,
        CliInterfaceOrigin {
            intent_text: "Document output preview; no binding invocation.".into(),
            revision: SessionRevision(0),
        },
    )?;
    let bytes = read_regular_file_bounded(
        Path::new(&arguments[2]),
        MAX_DOCUMENT_BYTES,
        "OUTPUT_VIEW_INPUT",
    )?;
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        output_view_error("OUTPUT_VIEW_INPUT", "Preview document must be UTF-8 JSON.")
    })?;
    let presentation = present_invocation_output(&active.definition, view, text)?;
    let delivery = if *structured {
        let response = json!({"source":"document_preview", "definition_sha256":active.definition_sha256, "presentation":presentation});
        writeln!(io::stdout(), "{response}")
    } else {
        writeln!(io::stderr(), "[preview] document; no binding invoked")
            .and_then(|()| io::stdout().write_all(presentation.table_text.as_bytes()))
    };
    delivery.map_err(|error| output_view_error("RUN_DELIVERY_FAILED", error.to_string()))?;
    Ok(0)
}

/// Launch an explicit document preview; errors are separate from successful table stdout.
pub fn launch_cli_render(arguments: impl Iterator<Item = OsString>) -> i32 {
    let mut structured = false;
    match preview_output_document(arguments, &mut structured) {
        Ok(code) => code,
        Err(error) => {
            if structured {
                let _ = writeln!(
                    io::stderr(),
                    "{}",
                    json!({"status":"error", "source":"document_preview", "error":error})
                );
            } else {
                let _ = writeln!(io::stderr(), "{}", escape_run_metadata(&error.to_string()));
            }
            crate::cli_run_frontend::run_error_exit(&error)
        }
    }
}
