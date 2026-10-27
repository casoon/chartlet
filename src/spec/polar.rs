use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, radar::grid_end, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many categories a polar chart takes.
pub(crate) const MIN_POINTS: usize = 3;
pub(crate) const MAX_POINTS: usize = 60;

/// A polar chart: one value for every category of a cycle (months, hours, directions), drawn at an
/// angle by the category and a distance from the center by the value, which is zero at the center.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolarSpec {
    /// The categories in the order of the cycle: the first at the top, the others clockwise.
    pub points: Vec<PolarPointSpec>,
    /// `bars` (default): a wedge from the center to the value. `lollipops`: a line and a dot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<PolarDisplay>,
    /// The value at the outer ring; by default the next step of four rings above the largest value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// What the values stand for, named in the table; by default the word for a value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolarPointSpec {
    pub label: String,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PolarDisplay {
    /// A wedge from the center to the value.
    #[default]
    Bars,
    /// A line from the center to the value, with a dot at its end.
    Lollipops,
}

impl PolarSpec {
    pub(crate) fn is_lollipops(&self) -> bool {
        self.display == Some(PolarDisplay::Lollipops)
    }

    /// The value at the outer ring.
    pub(crate) fn end(&self) -> f64 {
        self.max.unwrap_or_else(|| {
            let largest = self
                .points
                .iter()
                .map(|point| point.value)
                .fold(0.0, f64::max);
            grid_end(0.0, largest)
        })
    }
}

impl ChartSpec {
    /// Three to sixty categories with unique labels and values of zero or more, and an outer ring
    /// that holds them.
    pub(super) fn validate_polar(&self) -> Result<Vec<ChartWarning>, ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
            ("/references", !self.references.is_empty()),
            ("/timeAxis", !self.time_axis.is_default()),
            (
                "/valueAxis",
                self.value_axis != super::ValueAxisSpec::default(),
            ),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    "a polar chart is drawn from points; remove this field",
                ));
            }
        }
        let Some(polar) = &self.polar else {
            return Err(ChartError::new(
                "missing_polar",
                "/polar",
                "a polar chart requires a polar block",
            ));
        };
        if polar.points.len() < MIN_POINTS {
            return Err(ChartError::new(
                "empty_data",
                "/polar/points",
                format!("provide at least {MIN_POINTS} points"),
            ));
        }
        if polar.points.len() > MAX_POINTS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/polar/points",
                format!("at most {MAX_POINTS} points are supported"),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, point) in polar.points.iter().enumerate() {
            let path = format!("/polar/points/{index}");
            validate_text(&point.label, &format!("{path}/label"), 100)?;
            if !labels.insert(point.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "point labels must be unique",
                ));
            }
            validate_number(point.value, &format!("{path}/value"))?;
            if point.value < 0.0 {
                return Err(ChartError::new(
                    "invalid_value",
                    format!("{path}/value"),
                    "a polar chart starts at zero in the center; a value is zero or more",
                ));
            }
        }
        if let Some(max) = polar.max {
            validate_number(max, "/polar/max")?;
            if max <= 0.0 {
                return Err(ChartError::new(
                    "invalid_axis_range",
                    "/polar/max",
                    "`max` must be above zero",
                ));
            }
            if let Some(index) = polar.points.iter().position(|point| point.value > max) {
                return Err(ChartError::new(
                    "value_outside_axis",
                    format!("/polar/points/{index}/value"),
                    "the value lies beyond `max`; raise `max` or leave it out",
                ));
            }
        }
        if let Some(label) = &polar.value_label {
            validate_text(label, "/polar/valueLabel", 100)?;
        }
        Ok(Vec::new())
    }
}
