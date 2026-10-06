use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_SERIES, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// The most points one scatter plot draws, and the most lines.
pub(crate) const MAX_POINTS: usize = 5000;
const MAX_LINES: usize = 8;

/// A scatter plot: points on two numeric axes, in up to four groups, with threshold lines and
/// names on the points that matter: a volcano plot, a Manhattan plot, a regression residual.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScatterSpec {
    pub points: Vec<ScatterPointSpec>,
    /// Lines across the plot at a value of x or y, such as a threshold.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<ScatterLineSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScatterPointSpec {
    pub x: f64,
    pub y: f64,
    /// Points of one group share a palette color and an entry in the legend: on every point or
    /// none, at most four groups.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Writes a name beside the point, and lists the point in the data table of a big plot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScatterAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScatterLineSpec {
    /// The axis the value is on: `x` draws a vertical line, `y` a horizontal one.
    pub axis: ScatterAxis,
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl ScatterSpec {
    /// The groups in the order they first appear.
    pub(crate) fn groups(&self) -> Vec<&str> {
        let mut groups: Vec<&str> = Vec::new();
        for group in self
            .points
            .iter()
            .filter_map(|point| point.group.as_deref())
        {
            if !groups.contains(&group) {
                groups.push(group);
            }
        }
        groups
    }

    /// The palette color, from 1, of a point.
    pub(crate) fn color(&self, point: &ScatterPointSpec) -> usize {
        point.group.as_deref().map_or(1, |group| {
            self.groups()
                .iter()
                .position(|known| *known == group)
                .map_or(1, |position| position + 1)
        })
    }
}

impl ChartSpec {
    /// Points with finite numbers, groups on all of them or none, and a few named lines.
    pub(super) fn validate_scatter(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
                    "a scatter plot is drawn from points; remove this field, and name the axes with xTitle and yTitle",
                ));
            }
        }
        let Some(scatter) = &self.scatter else {
            return Err(ChartError::new(
                "missing_scatter",
                "/scatter",
                "a scatter plot requires a scatter block",
            ));
        };
        if scatter.points.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/scatter/points",
                "provide at least one point",
            ));
        }
        if scatter.points.len() > MAX_POINTS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/scatter/points",
                format!("at most {MAX_POINTS} points are supported"),
            ));
        }
        for (path, title) in [("xTitle", &scatter.x_title), ("yTitle", &scatter.y_title)] {
            if let Some(title) = title {
                validate_text(title, &format!("/scatter/{path}"), 100)?;
            }
        }
        validate_points(scatter)?;
        if scatter.lines.len() > MAX_LINES {
            return Err(ChartError::new(
                "too_many_references",
                "/scatter/lines",
                format!("at most {MAX_LINES} lines are supported"),
            ));
        }
        for (index, line) in scatter.lines.iter().enumerate() {
            let path = format!("/scatter/lines/{index}");
            validate_number(line.value, &format!("{path}/value"))?;
            if let Some(label) = &line.label {
                validate_text(label, &format!("{path}/label"), 100)?;
            }
        }
        Ok(Vec::new())
    }
}

fn validate_points(scatter: &ScatterSpec) -> Result<(), ChartError> {
    let grouped = scatter.points.iter().any(|point| point.group.is_some());
    let mut groups = BTreeSet::new();
    for (index, point) in scatter.points.iter().enumerate() {
        let path = format!("/scatter/points/{index}");
        validate_number(point.x, &format!("{path}/x"))?;
        validate_number(point.y, &format!("{path}/y"))?;
        if let Some(label) = &point.label {
            validate_text(label, &format!("{path}/label"), 60)?;
        }
        match (&point.group, grouped) {
            (Some(group), _) => {
                validate_text(group, &format!("{path}/group"), 60)?;
                groups.insert(group.as_str());
                if groups.len() > MAX_SERIES {
                    return Err(ChartError::new(
                        "too_many_series",
                        format!("{path}/group"),
                        format!("at most {MAX_SERIES} groups are supported, one per palette color"),
                    ));
                }
            }
            (None, true) => {
                return Err(ChartError::new(
                    "missing_group",
                    format!("{path}/group"),
                    "give every point a group, or none",
                ));
            }
            (None, false) => {}
        }
    }
    Ok(())
}
