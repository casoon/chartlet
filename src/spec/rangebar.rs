use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_DATA_POINTS, MAX_SERIES, is_false, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How the spans of a `rangebar` chart are drawn.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RangeDisplay {
    /// A bar from low to high.
    #[default]
    Bar,
    /// An open dot at the low value, a filled dot at the high value and a line between them.
    Dumbbell,
}

impl RangeDisplay {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(super) const fn is_bar(&self) -> bool {
        matches!(self, Self::Bar)
    }
}

/// The order of the spans of a `rangebar` chart.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RangeSort {
    /// As listed.
    #[default]
    None,
    /// The widest span first: the biggest change between low and high.
    Change,
}

impl RangeSort {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(super) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

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
    /// How much the span counts, such as the weight of a study: the central mark becomes a square
    /// whose area follows it. Needs `mid`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// The span is an overall result, such as a pooled effect: it is drawn as a diamond from low
    /// to high, widest at `mid`, which it needs.
    #[serde(default, skip_serializing_if = "is_false")]
    pub summary: bool,
}

impl ChartSpec {
    /// The indices of the spans in the order they are shown: as listed, or the widest span first.
    pub(crate) fn range_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.ranges.len()).collect();
        if self.sort == RangeSort::Change {
            let change = |index: usize| self.ranges[index].high - self.ranges[index].low;
            crate::sort::by(&mut order, |a, b| change(*b).total_cmp(&change(*a)));
        }
        order
    }

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
    #[allow(clippy::too_many_lines)]
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
            if self.display == RangeDisplay::Dumbbell {
                for (field, present) in [
                    ("mid", range.mid.is_some()),
                    ("weight", range.weight.is_some()),
                    ("summary", range.summary),
                    ("modeled", range.modeled),
                ] {
                    if present {
                        return Err(ChartError::new(
                            "option_not_supported",
                            format!("{path}/{field}"),
                            "a dumbbell connects the low and the high value of a span; remove this field or draw bars",
                        ));
                    }
                }
            }
            if let Some(weight) = range.weight {
                validate_number(weight, &format!("{path}/weight"))?;
                if weight <= 0.0 || range.mid.is_none() {
                    return Err(ChartError::new(
                        "invalid_weight",
                        format!("{path}/weight"),
                        "a weight is above zero and goes with a mid, where the square sits",
                    ));
                }
            }
            if range.summary && range.mid.is_none() {
                return Err(ChartError::new(
                    "summary_without_mid",
                    format!("{path}/summary"),
                    "a summary diamond is widest at mid; give the range one",
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
        self.validate_references()?;
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
