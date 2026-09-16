//! Authored output views project typed rows and display cells independently of invocation effects.

use crate::authoring_error::AuthoringError;
use crate::cli_definition::validate_cli_name;
use crate::cli_table_render::{clean_output_cell, format_output_cell, render_output_table};
use crate::cli_view_expression::{
    OutputEvaluationBudget, OutputExpression, OutputScope, compile_output_expression,
};
use crate::document_value::DocumentValue;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_OUTPUT_VIEW_BYTES: usize = 4 * 1024 * 1024;

/// Cell formatting changes display only; fallback text applies to absent or incompatible value kinds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CliCellFormat {
    Plain {},
    DurationHms { fallback: String },
    SecondsCeil { fallback: String },
    Join { separator: String, fallback: String },
}

/// Stable column IDs address typed projections; headings and formats describe their table display.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliOutputColumn {
    pub id: String,
    pub heading: String,
    pub value: DocumentValue,
    pub format: CliCellFormat,
}

/// Output view expressions have no authority beyond their supplied root document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliOutputView {
    pub help: String,
    pub require: DocumentValue,
    pub rows: DocumentValue,
    pub columns: Vec<CliOutputColumn>,
}

/// Typed projected rows use exact JSON text; display rows and table text are separate derived values.
#[derive(Clone, Debug, Serialize)]
pub struct CliOutputPresentation {
    pub view: String,
    pub columns: Vec<String>,
    pub rows_json_text: String,
    pub display_rows: Vec<Vec<String>>,
    pub table_text: String,
}

pub fn output_view_error(code: &str, message: impl Into<String>) -> AuthoringError {
    AuthoringError::new(
        code,
        message,
        "Inspect the view and original result; repair or select a compatible view, then render the retained result without invoking its command again.",
    )
}

struct CompiledOutputView {
    require: OutputExpression,
    rows: OutputExpression,
    columns: Vec<OutputExpression>,
}

impl CliOutputView {
    fn compile_output_view(&self) -> Result<CompiledOutputView, AuthoringError> {
        if self.help.trim().is_empty() || self.columns.is_empty() || self.columns.len() > 32 {
            return Err(output_view_error(
                "INVALID_OUTPUT_VIEW",
                "Output view requires help and 1 through 32 columns.",
            ));
        }
        let mut remaining = 2048;
        let require = compile_output_expression(&self.require, false, false, 0, &mut remaining)?;
        let rows = compile_output_expression(&self.rows, false, false, 0, &mut remaining)?;
        let mut identifiers = BTreeSet::new();
        let mut columns = Vec::new();
        for column in &self.columns {
            validate_cli_name(&column.id)?;
            if !identifiers.insert(&column.id) || column.heading.trim().is_empty() {
                return Err(output_view_error(
                    "INVALID_OUTPUT_VIEW",
                    "Output columns require unique IDs and nonempty headings.",
                ));
            }
            columns.push(compile_output_expression(
                &column.value,
                true,
                false,
                0,
                &mut remaining,
            )?);
        }
        Ok(CompiledOutputView {
            require,
            rows,
            columns,
        })
    }

    /// Reject invalid expression shapes and references before a definition becomes active.
    pub fn validate_output_view(&self) -> Result<(), AuthoringError> {
        self.compile_output_view().map(|_| ())
    }

    /// Evaluate and render one supplied document without reading clocks, files, processes, or runtime state.
    pub fn present_output_document(
        &self,
        id: &str,
        document: &DocumentValue,
    ) -> Result<CliOutputPresentation, AuthoringError> {
        document.validate_document_limits()?;
        let compiled = self.compile_output_view()?;
        let mut budget = OutputEvaluationBudget::default();
        let scope = OutputScope {
            root: document,
            row: None,
            item: None,
        };
        if !matches!(
            compiled.require.evaluate(scope, &mut budget)?,
            Some(DocumentValue::Boolean(true))
        ) {
            return Err(output_view_error(
                "OUTPUT_VIEW_INPUT",
                "Document does not satisfy the output view's input requirement.",
            ));
        }
        let Some(DocumentValue::Array(input_rows)) = compiled.rows.evaluate(scope, &mut budget)?
        else {
            return Err(output_view_error(
                "OUTPUT_VIEW_INPUT",
                "Output view rows require an array.",
            ));
        };
        if input_rows.len() > 4096 {
            return Err(output_view_error(
                "OUTPUT_VIEW_LIMIT",
                "Output view exceeds 4096 rows.",
            ));
        }
        let mut projected = Vec::new();
        let mut display_rows = Vec::new();
        let mut bytes = 0usize;
        for (row_index, row) in input_rows.iter().enumerate() {
            let mut projected_row = BTreeMap::new();
            let mut display_row = Vec::new();
            for (column, expression) in self.columns.iter().zip(&compiled.columns) {
                let contextualize = |mut error: AuthoringError| {
                    error.message = format!(
                        "View {id}, row {row_index}, column {}: {}",
                        column.id, error.message
                    );
                    error
                };
                let value = expression
                    .evaluate(
                        OutputScope {
                            row: Some(row),
                            ..scope
                        },
                        &mut budget,
                    )
                    .map_err(contextualize)?;
                let display =
                    format_output_cell(&column.format, value.as_ref()).map_err(contextualize)?;
                bytes = bytes.saturating_add(display.len());
                if let Some(value) = value {
                    bytes = bytes
                        .saturating_add(value.compact_document_json().len())
                        .saturating_add(column.id.len());
                    projected_row.insert(column.id.clone(), value);
                }
                if bytes > MAX_OUTPUT_VIEW_BYTES {
                    return Err(output_view_error(
                        "OUTPUT_VIEW_LIMIT",
                        "Output projection exceeds 4 MiB.",
                    ));
                }
                display_row.push(display);
            }
            projected.push(DocumentValue::Object(projected_row));
            display_rows.push(display_row);
        }
        let headings: Vec<_> = self
            .columns
            .iter()
            .map(|column| clean_output_cell(&column.heading))
            .collect();
        let table_text = render_output_table(&headings, &display_rows)?;
        let result = CliOutputPresentation {
            view: id.into(),
            columns: self
                .columns
                .iter()
                .map(|column| column.id.clone())
                .collect(),
            rows_json_text: DocumentValue::Array(projected).compact_document_json(),
            display_rows,
            table_text,
        };
        if serde_json::to_vec(&result)
            .expect("Output presentation serializes")
            .len()
            > MAX_OUTPUT_VIEW_BYTES
        {
            return Err(output_view_error(
                "OUTPUT_VIEW_LIMIT",
                "Complete output presentation exceeds 4 MiB.",
            ));
        }
        Ok(result)
    }
}
