use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_DATA_POINTS, MAX_SERIES, is_false, validate_number, validate_text};
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
    /// The group the span belongs to, such as a size class: each group has its own color and an
    /// entry in the legend. Either every range names one or none does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl ChartSpec {
    /// The groups of a `rangebar` chart in the order they first appear; empty without groups.
    pub(crate) fn range_groups(&self) -> Vec<&str> {
        let mut groups: Vec<&str> = Vec::new();
        for group in self
            .ranges
            .iter()
            .filter_map(|range| range.group.as_deref())
        {
            if !groups.contains(&group) {
                groups.push(group);
            }
        }
        groups
    }

    /// Either every range names a group or none does, and there are no more groups than
    /// palette colors.
    fn validate_range_groups(&self) -> Result<(), ChartError> {
        let grouped = self.ranges.iter().any(|range| range.group.is_some());
        let mut groups: Vec<&str> = Vec::new();
        for (index, range) in self.ranges.iter().enumerate() {
            let path = format!("/ranges/{index}/group");
            let Some(group) = range.group.as_deref() else {
                if grouped {
                    return Err(ChartError::new(
                        "missing_group",
                        path,
                        "give every range a group, or none",
                    ));
                }
                continue;
            };
            validate_text(group, &path, 60)?;
            if !groups.contains(&group) {
                groups.push(group);
            }
            if groups.len() > MAX_SERIES {
                return Err(ChartError::new(
                    "too_many_series",
                    path,
                    format!("at most {MAX_SERIES} groups are supported, one per palette color"),
                ));
            }
        }
        Ok(())
    }

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
        self.validate_range_groups()?;
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
