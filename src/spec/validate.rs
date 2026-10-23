use serde_path_to_error::Segment;

use super::{AxisScale, ChartSpec, Orientation, ValueAxisSpec, ValueFormat};
use crate::error::ChartError;

impl ChartSpec {
    /// Rejects the fields that only a bar, line, or time chart has, naming the field and the fix.
    /// Neither map has axes, series or panes. `noun` and `source` name the type in the message,
    /// so the error says what to do rather than only what is wrong.
    pub(super) fn reject_map_options(&self, noun: &str, source: &str) -> Result<(), ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    format!("{noun} is drawn from {source}; remove this field"),
                ));
            }
        }
        if self.orientation != Orientation::Vertical {
            return Err(ChartError::new(
                "option_not_supported",
                "/orientation",
                "orientation is only available for bar charts",
            ));
        }
        if self.category_axis.title.is_some() {
            return Err(ChartError::new(
                "option_not_supported",
                "/categoryAxis/title",
                format!("{noun} has no category axis"),
            ));
        }
        if self.value_axis.title.is_some()
            || self.value_axis.format != ValueFormat::Number
            || self.value_axis.min.is_some()
            || self.value_axis.max.is_some()
            || self.value_axis.thousands_separator
            || self.value_axis.scale != AxisScale::Linear
            || self.value_axis.reverse
            || self.value_axis.step.is_some()
            || self.value_axis.exact
        {
            return Err(ChartError::new(
                "option_not_supported",
                "/valueAxis",
                format!("{noun} has no value axis"),
            ));
        }
        if !self.time_axis.is_default() {
            return Err(ChartError::new(
                "option_not_supported",
                "/timeAxis",
                format!("{noun} has no time axis"),
            ));
        }
        Ok(())
    }
}

/// Converts a deserialization path such as `data[0].value` into the JSON Pointer
/// `/data/0/value` that validation errors use. The document root is `/`.
pub(super) fn json_pointer(path: &serde_path_to_error::Path) -> String {
    let pointer: String = path
        .iter()
        .filter_map(|segment| match segment {
            Segment::Seq { index } => Some(format!("/{index}")),
            Segment::Map { key } => Some(format!("/{}", key.replace('~', "~0").replace('/', "~1"))),
            Segment::Enum { variant } => Some(format!("/{variant}")),
            Segment::Unknown => None,
        })
        .collect();
    if pointer.is_empty() {
        "/".to_owned()
    } else {
        pointer
    }
}

/// A value on a logarithmic axis: above zero.
pub(super) fn positive(value: f64, path: String) -> Result<(), ChartError> {
    if value > 0.0 {
        Ok(())
    } else {
        Err(ChartError::new(
            "invalid_value",
            path,
            "a logarithmic axis takes values above zero only",
        ))
    }
}

/// The declared range of a logarithmic axis: above zero.
pub(super) fn positive_bounds(axis: &ValueAxisSpec, path: &str) -> Result<(), ChartError> {
    for (name, bound) in [("min", axis.min), ("max", axis.max)] {
        if let Some(bound) = bound {
            positive(bound, format!("{path}/{name}"))?;
        }
    }
    Ok(())
}

pub(crate) fn validate_number(value: f64, path: &str) -> Result<(), ChartError> {
    if !value.is_finite() {
        return Err(ChartError::new(
            "non_finite_value",
            path,
            "value must be finite",
        ));
    }
    let magnitude = value.abs();
    if magnitude > 1e100 || (magnitude > 0.0 && magnitude < 1e-100) {
        return Err(ChartError::new(
            "unsupported_numeric_range",
            path,
            "value magnitude must be zero or between 1e-100 and 1e100",
        ));
    }
    Ok(())
}

pub(super) fn validate_optional_text(
    value: Option<&String>,
    path: &str,
    max_length: usize,
) -> Result<(), ChartError> {
    if let Some(value) = value {
        validate_text(value, path, max_length)?;
    }
    Ok(())
}

pub(super) fn validate_text(value: &str, path: &str, max_length: usize) -> Result<(), ChartError> {
    let length = value.chars().count();
    if value.trim().is_empty() {
        return Err(ChartError::new(
            "empty_text",
            path,
            "value must contain visible text",
        ));
    }
    if length > max_length {
        return Err(ChartError::new(
            "text_too_long",
            path,
            format!("value must not exceed {max_length} characters"),
        ));
    }
    if value.chars().any(|character| !is_xml_character(character)) {
        return Err(ChartError::new(
            "invalid_xml_character",
            path,
            "text contains a control character that XML 1.0 cannot represent",
        ));
    }
    Ok(())
}

fn is_xml_character(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
        || ('\u{20}'..='\u{D7FF}').contains(&character)
        || ('\u{E000}'..='\u{FFFD}').contains(&character)
        || ('\u{10000}'..='\u{10FFFF}').contains(&character)
}
