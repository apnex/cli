//! Output expressions read supplied documents with explicit scope and bounded work; they have no IO.

use crate::authoring_error::AuthoringError;
use crate::cli_output_view::output_view_error;
use crate::document_path::DocumentSegment;
use crate::document_value::DocumentValue;
use std::collections::BTreeMap;

pub(crate) enum OutputExpression {
    Path(String, Vec<DocumentSegment>),
    Literal(DocumentValue),
    Coalesce(Vec<Self>, bool),
    Equal(Box<Self>, Box<Self>),
    And(Vec<Self>),
    Is(Box<Self>, String),
    If(Box<Self>, Box<Self>, Box<Self>),
    First(Box<Self>, Box<Self>, Vec<DocumentSegment>),
    Any(Box<Self>, Box<Self>),
    Concat(Vec<Self>),
}

fn expression_error(message: impl Into<String>) -> AuthoringError {
    output_view_error("INVALID_OUTPUT_VIEW", message)
}

fn expression_field<'a>(
    fields: &'a BTreeMap<String, DocumentValue>,
    name: &str,
) -> Result<&'a DocumentValue, AuthoringError> {
    fields
        .get(name)
        .ok_or_else(|| expression_error(format!("Output expression requires {name}.")))
}

fn expression_text(value: &DocumentValue) -> Result<&str, AuthoringError> {
    match value {
        DocumentValue::String(text) => Ok(text),
        _ => Err(expression_error(
            "Output expression field requires a string.",
        )),
    }
}

fn expression_path(value: &DocumentValue) -> Result<Vec<DocumentSegment>, AuthoringError> {
    let path: Vec<DocumentSegment> = serde_json::from_str(&value.compact_document_json())
        .map_err(|_| expression_error("Output path requires typed key/index segments."))?;
    if path.len() > 64 {
        return Err(expression_error(
            "Output expression path exceeds 64 segments.",
        ));
    }
    Ok(path)
}

/// Compile expression data once, rejecting unknown fields and unavailable row/item scopes.
pub(crate) fn compile_output_expression(
    value: &DocumentValue,
    row_scope: bool,
    item_scope: bool,
    depth: usize,
    remaining: &mut usize,
) -> Result<OutputExpression, AuthoringError> {
    if depth > 16 || *remaining == 0 {
        return Err(expression_error(
            "Output expressions exceed depth 16 or 2048 nodes per view.",
        ));
    }
    *remaining -= 1;
    let DocumentValue::Object(fields) = value else {
        return Err(expression_error("Output expression requires an object."));
    };
    let op = expression_text(expression_field(fields, "op")?)?;
    let allowed: &[&str] = match op {
        "root" | "row" | "item" => &["op", "path"],
        "literal" => &["op", "value"],
        "coalesce" => &["op", "values", "skip_false"],
        "eq" => &["op", "left", "right"],
        "and" | "concat" => &["op", "values"],
        "is" => &["op", "value", "type"],
        "if" => &["op", "condition", "then", "else"],
        "first" => &["op", "input", "where", "path"],
        "any" => &["op", "input", "where"],
        _ => {
            return Err(expression_error(format!(
                "Unsupported output expression operator: {op}"
            )));
        }
    };
    if fields.len() != allowed.len() || fields.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(expression_error(format!(
            "Output expression {op} requires exactly {}.",
            allowed.join(", ")
        )));
    }
    let compile = |name: &str, remaining: &mut usize| {
        compile_output_expression(
            expression_field(fields, name)?,
            row_scope,
            item_scope,
            depth + 1,
            remaining,
        )
    };
    let list = |remaining: &mut usize| -> Result<Vec<OutputExpression>, AuthoringError> {
        let DocumentValue::Array(values) = expression_field(fields, "values")? else {
            return Err(expression_error(
                "Output expression values require an array.",
            ));
        };
        if values.is_empty() || values.len() > 32 {
            return Err(expression_error(
                "Output expression requires 1 through 32 operands.",
            ));
        }
        values
            .iter()
            .map(|value| {
                compile_output_expression(value, row_scope, item_scope, depth + 1, remaining)
            })
            .collect()
    };
    Ok(match op {
        "root" | "row" | "item" => {
            if (op == "row" && !row_scope) || (op == "item" && !item_scope) {
                return Err(expression_error(format!(
                    "Output expression scope {op} is unavailable here."
                )));
            }
            OutputExpression::Path(
                op.into(),
                expression_path(expression_field(fields, "path")?)?,
            )
        }
        "literal" => OutputExpression::Literal(expression_field(fields, "value")?.clone()),
        "coalesce" => {
            let DocumentValue::Boolean(skip_false) = expression_field(fields, "skip_false")? else {
                return Err(expression_error(
                    "Output coalesce skip_false requires a boolean.",
                ));
            };
            OutputExpression::Coalesce(list(remaining)?, *skip_false)
        }
        "eq" => OutputExpression::Equal(
            Box::new(compile("left", remaining)?),
            Box::new(compile("right", remaining)?),
        ),
        "and" => OutputExpression::And(list(remaining)?),
        "is" => {
            let kind = expression_text(expression_field(fields, "type")?)?;
            if ![
                "object", "array", "string", "number", "boolean", "null", "missing",
            ]
            .contains(&kind)
            {
                return Err(expression_error(
                    "Output type predicate names an unsupported type.",
                ));
            }
            OutputExpression::Is(Box::new(compile("value", remaining)?), kind.into())
        }
        "if" => OutputExpression::If(
            Box::new(compile("condition", remaining)?),
            Box::new(compile("then", remaining)?),
            Box::new(compile("else", remaining)?),
        ),
        "first" | "any" => {
            let input = Box::new(compile("input", remaining)?);
            let predicate = Box::new(compile_output_expression(
                expression_field(fields, "where")?,
                row_scope,
                true,
                depth + 1,
                remaining,
            )?);
            if op == "first" {
                OutputExpression::First(
                    input,
                    predicate,
                    expression_path(expression_field(fields, "path")?)?,
                )
            } else {
                OutputExpression::Any(input, predicate)
            }
        }
        "concat" => OutputExpression::Concat(list(remaining)?),
        _ => unreachable!("Operators checked above"),
    })
}

pub(crate) struct OutputEvaluationBudget {
    steps: usize,
    bytes: usize,
}

impl Default for OutputEvaluationBudget {
    fn default() -> Self {
        Self {
            steps: 100_000,
            bytes: 16 * 1024 * 1024,
        }
    }
}

impl OutputEvaluationBudget {
    pub(crate) fn step(&mut self) -> Result<(), AuthoringError> {
        self.steps = self.steps.checked_sub(1).ok_or_else(|| {
            output_view_error(
                "OUTPUT_VIEW_LIMIT",
                "Output evaluation exceeds 100000 steps.",
            )
        })?;
        Ok(())
    }
    fn copy(
        &mut self,
        value: Option<&DocumentValue>,
    ) -> Result<Option<DocumentValue>, AuthoringError> {
        let Some(value) = value else {
            return Ok(None);
        };
        let bytes = value.compact_document_json().len();
        self.bytes = self.bytes.checked_sub(bytes).ok_or_else(|| {
            output_view_error(
                "OUTPUT_VIEW_LIMIT",
                "Output evaluation exceeds 16 MiB of cumulative value copying.",
            )
        })?;
        Ok(Some(value.clone()))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct OutputScope<'a> {
    pub root: &'a DocumentValue,
    pub row: Option<&'a DocumentValue>,
    pub item: Option<&'a DocumentValue>,
}

fn output_path<'a>(
    mut value: Option<&'a DocumentValue>,
    path: &[DocumentSegment],
    budget: &mut OutputEvaluationBudget,
) -> Result<Option<&'a DocumentValue>, AuthoringError> {
    for segment in path {
        budget.step()?;
        value = match (value, segment) {
            (Some(DocumentValue::Object(values)), DocumentSegment::Key { key }) => values.get(key),
            (Some(DocumentValue::Array(values)), DocumentSegment::Index { index }) => {
                values.get(*index)
            }
            _ => None,
        };
    }
    Ok(value)
}

fn output_boolean(value: Option<DocumentValue>) -> Result<bool, AuthoringError> {
    match value {
        Some(DocumentValue::Boolean(value)) => Ok(value),
        _ => Err(output_view_error(
            "OUTPUT_VIEW_INPUT",
            "Output condition requires a boolean.",
        )),
    }
}

pub(crate) fn output_value_text(value: Option<&DocumentValue>) -> String {
    match value {
        None | Some(DocumentValue::Null) => String::new(),
        Some(DocumentValue::String(text)) => text.clone(),
        Some(value) => value.compact_document_json(),
    }
}

impl OutputExpression {
    pub(crate) fn evaluate(
        &self,
        scope: OutputScope<'_>,
        budget: &mut OutputEvaluationBudget,
    ) -> Result<Option<DocumentValue>, AuthoringError> {
        budget.step()?;
        Ok(match self {
            Self::Path(source, path) => {
                let base = match source.as_str() {
                    "root" => Some(scope.root),
                    "row" => scope.row,
                    _ => scope.item,
                };
                let value = output_path(base, path, budget)?;
                budget.copy(value)?
            }
            Self::Literal(value) => budget.copy(Some(value))?,
            Self::Coalesce(values, skip_false) => {
                let mut result = None;
                for value in values {
                    let value = value.evaluate(scope, budget)?;
                    if value.as_ref().is_some_and(|value| {
                        !matches!(value, DocumentValue::Null)
                            && !(*skip_false && matches!(value, DocumentValue::Boolean(false)))
                    }) {
                        result = value;
                        break;
                    }
                }
                result
            }
            Self::Equal(left, right) => Some(DocumentValue::Boolean(
                left.evaluate(scope, budget)? == right.evaluate(scope, budget)?,
            )),
            Self::And(values) => {
                let mut result = true;
                for value in values {
                    if !output_boolean(value.evaluate(scope, budget)?)? {
                        result = false;
                        break;
                    }
                }
                Some(DocumentValue::Boolean(result))
            }
            Self::Is(value, kind) => Some(DocumentValue::Boolean(
                value
                    .evaluate(scope, budget)?
                    .as_ref()
                    .map_or("missing", DocumentValue::document_kind)
                    == kind,
            )),
            Self::If(condition, yes, no) => {
                if output_boolean(condition.evaluate(scope, budget)?)? {
                    yes.evaluate(scope, budget)?
                } else {
                    no.evaluate(scope, budget)?
                }
            }
            Self::First(input, predicate, path) => {
                let mut result = None;
                if let Some(DocumentValue::Array(items)) = input.evaluate(scope, budget)? {
                    for item in &items {
                        budget.step()?;
                        if output_boolean(predicate.evaluate(
                            OutputScope {
                                item: Some(item),
                                ..scope
                            },
                            budget,
                        )?)? {
                            let value = output_path(Some(item), path, budget)?;
                            result = budget.copy(value)?;
                            break;
                        }
                    }
                }
                result
            }
            Self::Any(input, predicate) => {
                let Some(DocumentValue::Array(items)) = input.evaluate(scope, budget)? else {
                    return Err(output_view_error(
                        "OUTPUT_VIEW_INPUT",
                        "Output any input requires an array.",
                    ));
                };
                let mut found = false;
                for item in &items {
                    budget.step()?;
                    if output_boolean(predicate.evaluate(
                        OutputScope {
                            item: Some(item),
                            ..scope
                        },
                        budget,
                    )?)? {
                        found = true;
                        break;
                    }
                }
                Some(DocumentValue::Boolean(found))
            }
            Self::Concat(values) => {
                let mut text = String::new();
                for value in values {
                    text.push_str(&output_value_text(value.evaluate(scope, budget)?.as_ref()));
                    if text.len() > crate::document_value::MAX_SCALAR_BYTES {
                        return Err(output_view_error(
                            "OUTPUT_VIEW_LIMIT",
                            "Output concatenation exceeds 64 KiB.",
                        ));
                    }
                }
                Some(DocumentValue::String(text))
            }
        })
    }
}
