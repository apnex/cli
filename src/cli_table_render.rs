//! Table formatting preserves source values and computes duration rounding without floating point.

use crate::authoring_error::AuthoringError;
use crate::cli_output_view::{CliCellFormat, MAX_OUTPUT_VIEW_BYTES, output_view_error};
use crate::cli_view_expression::output_value_text;
use crate::document_value::DocumentValue;
use num_bigint::BigUint;
use unicode_width::UnicodeWidthStr;

/// Escape display controls and backslashes without changing the source document.
pub(crate) fn clean_output_cell(text: &str) -> String {
    text.chars()
        .flat_map(|character| {
            if character.is_control()
                || matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
            {
                " ".chars().collect::<Vec<_>>()
            } else if character == '\\' {
                vec!['\\', '\\']
            } else {
                vec![character]
            }
        })
        .collect()
}

fn duration_seconds(token: &str, ceil: bool) -> Result<BigUint, AuthoringError> {
    let negative = token.starts_with('-');
    let unsigned = token.trim_start_matches('-');
    let (mantissa, exponent) = unsigned.split_once(['e', 'E']).unwrap_or((unsigned, "0"));
    let coefficient: String = mantissa.chars().filter(|c| *c != '.').collect();
    let digits = coefficient.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(BigUint::from(0u32));
    }
    if negative {
        if ceil {
            return Ok(BigUint::from(0u32));
        }
        return Err(output_view_error(
            "OUTPUT_VIEW_INPUT",
            "Elapsed duration cannot be negative.",
        ));
    }
    let fractional = mantissa.split_once('.').map_or(0, |(_, tail)| tail.len()) as i64;
    let exponent = exponent.parse::<i64>().unwrap_or_else(|_| {
        if exponent.starts_with('-') {
            i64::MIN
        } else {
            i64::MAX
        }
    });
    let position = (digits.len() as i64)
        .saturating_add(exponent)
        .saturating_sub(fractional)
        .saturating_sub(3);
    if position <= 0 {
        return Ok(BigUint::from(u32::from(ceil)));
    }
    if position > 4096 {
        return Err(output_view_error(
            "OUTPUT_VIEW_LIMIT",
            "Duration formatting exceeds 4096 integer digits.",
        ));
    }
    let position = position as usize;
    let (integer, remainder) = if position < digits.len() {
        (
            digits[..position].to_owned(),
            digits[position..].bytes().any(|c| c != b'0'),
        )
    } else {
        let mut integer = digits.to_owned();
        integer.extend(std::iter::repeat_n('0', position - digits.len()));
        (integer, false)
    };
    let mut seconds =
        BigUint::parse_bytes(integer.as_bytes(), 10).expect("Validated JSON number digits");
    if ceil && remainder {
        seconds += 1u32;
        if seconds.to_string().len() > 4096 {
            return Err(output_view_error(
                "OUTPUT_VIEW_LIMIT",
                "Rounded duration exceeds 4096 integer digits.",
            ));
        }
    }
    Ok(seconds)
}

pub(crate) fn format_output_cell(
    format: &CliCellFormat,
    value: Option<&DocumentValue>,
) -> Result<String, AuthoringError> {
    let text = match format {
        CliCellFormat::Plain {} => output_value_text(value),
        CliCellFormat::DurationHms { fallback } | CliCellFormat::SecondsCeil { fallback } => {
            if let Some(DocumentValue::Number(number)) = value {
                let ceil = matches!(format, CliCellFormat::SecondsCeil { .. });
                let seconds = duration_seconds(number.get(), ceil)?;
                if ceil {
                    format!("{seconds}s")
                } else {
                    format!(
                        "{:0>2}:{:0>2}:{:0>2}",
                        (&seconds / 3600u32).to_string(),
                        ((&seconds / 60u32) % 60u32).to_string(),
                        (&seconds % 60u32).to_string()
                    )
                }
            } else {
                fallback.clone()
            }
        }
        CliCellFormat::Join {
            separator,
            fallback,
        } => {
            if let Some(DocumentValue::Array(values)) = value {
                let mut text = String::new();
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        text.push_str(separator);
                    }
                    text.push_str(&output_value_text(Some(value)));
                    if text.len() > MAX_OUTPUT_VIEW_BYTES {
                        return Err(output_view_error(
                            "OUTPUT_VIEW_LIMIT",
                            "Joined cell exceeds output limit.",
                        ));
                    }
                }
                text
            } else {
                fallback.clone()
            }
        }
    };
    Ok(clean_output_cell(&text))
}

/// Size padding before allocation; every row keeps its full cells and fixed column order.
pub(crate) fn render_output_table(
    headings: &[String],
    rows: &[Vec<String>],
) -> Result<String, AuthoringError> {
    let mut widths: Vec<_> = headings.iter().map(|text| text.width()).collect();
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.width());
        }
    }
    let mut bytes = 0usize;
    for row in std::iter::once(headings).chain(rows.iter().map(Vec::as_slice)) {
        for (index, cell) in row.iter().enumerate() {
            bytes = bytes.saturating_add(cell.len());
            if index + 1 != widths.len() {
                bytes = bytes.saturating_add(widths[index] - cell.width() + 2);
            }
        }
        bytes = bytes.saturating_add(1);
    }
    if bytes > MAX_OUTPUT_VIEW_BYTES {
        return Err(output_view_error(
            "OUTPUT_VIEW_LIMIT",
            "Padded table exceeds 4 MiB.",
        ));
    }
    let mut text = String::with_capacity(bytes);
    for row in std::iter::once(headings).chain(rows.iter().map(Vec::as_slice)) {
        for (index, cell) in row.iter().enumerate() {
            text.push_str(cell);
            if index + 1 != widths.len() {
                text.extend(std::iter::repeat_n(' ', widths[index] - cell.width() + 2));
            }
        }
        text.push('\n');
    }
    Ok(text)
}
