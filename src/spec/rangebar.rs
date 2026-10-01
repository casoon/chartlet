use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_DATA_POINTS, is_false, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// One span of a `rangebar` chart: a category with a low and a high value and, optionally, a
/// central estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RangeSpec {
    pub label: String,
    pub low: f64,
    pub high: f64,
    /// A central estimate, drawn as a mark across the bar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mid: Option<f64>,
    /// The span comes from a model rather than a measurement; it is hatched.
    #[serde(default, skip_serializing_if = "is_false")]
    pub modeled: bool,
}

impl ChartSpec {
    /// Spans per category: a low no higher than its high, and a central value between them.
    pub(super) fn validate_rangebar(&self) -> Result<Vec<ChartWarning>, ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
            ("/timeAxis", !self.time_axis.is_default()),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    "a rangebar chart is drawn from ranges; remove this field",
                ));
            }
        }
        if self.ranges.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/ranges",
                "provide at least one range",
            ));
        }
        if self.ranges.len() > MAX_DATA_POINTS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/ranges",
                format!("at most {MAX_DATA_POINTS} ranges are supported"),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, range) in self.ranges.iter().enumerate() {
            let path = format!("/ranges/{index}");
            validate_text(&range.label, &format!("{path}/label"), 200)?;
            if !labels.insert(range.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "range labels must be unique",
                ));
            }
            validate_number(range.low, &format!("{path}/low"))?;
            validate_number(range.high, &format!("{path}/high"))?;
            if range.low > range.high {
                return Err(ChartError::new(
                    "invalid_range",
                    format!("{path}/high"),
                    "high must not be below low",
                ));
            }
            if let Some(mid) = range.mid {
                validate_number(mid, &format!("{path}/mid"))?;
                if !(range.low..=range.high).contains(&mid) {
                    return Err(ChartError::new(
                        "mid_outside_range",
                        format!("{path}/mid"),
                        "mid must lie between low and high",
                    ));
                }
            }
        }
        let mut warnings = Vec::new();
        if self.ranges.len() > 16 {
            warnings.push(ChartWarning::new(
                "dense_chart",
                "/ranges",
                "more than 16 categories can be difficult to read at the configured size",
            ));
        }
        Ok(warnings)
    }
}
