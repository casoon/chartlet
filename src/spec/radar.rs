use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_SERIES, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many axes a radar chart takes, and the number above which it is reported as hard to read;
/// the same for the series.
pub(crate) const MIN_AXES: usize = 3;
pub(crate) const MAX_AXES: usize = 16;
pub(crate) const READABLE_AXES: usize = 8;
pub(crate) const READABLE_SERIES: usize = 3;
/// The grid rings a radar chart draws between the center and the end of the axes.
pub(crate) const RINGS: u32 = 4;

/// A radar chart: one axis per measure, drawn from the center like the spokes of a wheel, and one
/// closed polygon per series through its values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RadarSpec {
    pub axes: Vec<RadarAxisSpec>,
    pub series: Vec<RadarSeriesSpec>,
}

/// One axis: its label and, optionally, the values at its center and at its end.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RadarAxisSpec {
    pub label: String,
    /// The value at the center; 0 by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// The value at the end of the axis; by default the next step of the grid above the largest
    /// value of the axis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RadarSeriesSpec {
    pub name: String,
    /// One value for every axis, in the order of `axes`.
    pub values: Vec<f64>,
}

impl RadarSpec {
    /// The value at the center of an axis.
    pub(crate) fn min(&self, axis: usize) -> f64 {
        self.axes[axis].min.unwrap_or(0.0)
    }

    /// The value at the end of an axis: the one named, or the end of the grid above the largest
    /// value, see [`grid_end`].
    pub(crate) fn max(&self, axis: usize) -> f64 {
        self.axes[axis].max.unwrap_or_else(|| {
            let min = self.min(axis);
            let largest = self
                .series
                .iter()
                .map(|series| series.values[axis])
                .fold(min, f64::max);
            grid_end(min, largest)
        })
    }

    /// Where a value lies between the center and the end of its axis, from 0 to 1.
    pub(crate) fn fraction(&self, axis: usize, value: f64) -> f64 {
        let (min, max) = (self.min(axis), self.max(axis));
        (value - min) / (max - min)
    }
}

/// The end of a grid of four rings that starts at `min` and reaches `largest`: the smallest round
/// step (1, 2, 2.5, 4, 5, 6 or 8 times a power of ten) that makes four rings reach it.
pub(crate) fn grid_end(min: f64, largest: f64) -> f64 {
    let reach = if largest > min {
        (largest - min) / f64::from(RINGS)
    } else {
        1.0
    };
    #[allow(clippy::cast_possible_truncation)]
    let power = 10_f64.powi(reach.log10().floor() as i32);
    let step = [1.0, 2.0, 2.5, 4.0, 5.0, 6.0, 8.0, 10.0]
        .into_iter()
        .map(|mantissa| mantissa * power)
        .find(|step| *step >= reach * (1.0 - 1e-9))
        .unwrap_or(10.0 * power);
    min + step * f64::from(RINGS)
}

impl ChartSpec {
    /// Three to sixteen axes with unique labels and a range that holds the values, and one to four
    /// series that name a value for every axis; a chart of more than 8 axes or 3 series is
    /// reported as hard to read.
    #[allow(clippy::too_many_lines)]
    pub(super) fn validate_radar(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
                    "a radar chart is drawn from axes and series; remove this field",
                ));
            }
        }
        let Some(radar) = &self.radar else {
            return Err(ChartError::new(
                "missing_radar",
                "/radar",
                "a radar chart requires a radar block",
            ));
        };
        if radar.axes.len() < MIN_AXES {
            return Err(ChartError::new(
                "empty_data",
                "/radar/axes",
                format!("provide at least {MIN_AXES} axes"),
            ));
        }
        if radar.axes.len() > MAX_AXES {
            return Err(ChartError::new(
                "too_many_data_points",
                "/radar/axes",
                format!("at most {MAX_AXES} axes are supported"),
            ));
        }
        if radar.series.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/radar/series",
                "provide at least one series",
            ));
        }
        if radar.series.len() > MAX_SERIES {
            return Err(ChartError::new(
                "too_many_series",
                "/radar/series",
                format!("at most {MAX_SERIES} series are supported, one per palette color"),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, axis) in radar.axes.iter().enumerate() {
            let path = format!("/radar/axes/{index}");
            validate_text(&axis.label, &format!("{path}/label"), 100)?;
            if !labels.insert(axis.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "axis labels must be unique",
                ));
            }
            for (field, value) in [("min", axis.min), ("max", axis.max)] {
                if let Some(value) = value {
                    validate_number(value, &format!("{path}/{field}"))?;
                }
            }
            if let (Some(min), Some(max)) = (axis.min, axis.max)
                && min >= max
            {
                return Err(ChartError::new(
                    "invalid_axis_range",
                    format!("{path}/max"),
                    "`max` must be greater than `min`",
                ));
            }
        }
        let mut names = BTreeSet::new();
        for (index, series) in radar.series.iter().enumerate() {
            let path = format!("/radar/series/{index}");
            validate_text(&series.name, &format!("{path}/name"), 100)?;
            if !names.insert(series.name.as_str()) {
                return Err(ChartError::new(
                    "duplicate_series",
                    format!("{path}/name"),
                    "series names must be unique",
                ));
            }
            if series.values.len() != radar.axes.len() {
                return Err(ChartError::new(
                    "series_length_mismatch",
                    format!("{path}/values"),
                    format!("give one value for each of the {} axes", radar.axes.len()),
                ));
            }
            for (axis, value) in series.values.iter().enumerate() {
                validate_number(*value, &format!("{path}/values/{axis}"))?;
                let a = &radar.axes[axis];
                if *value < radar.min(axis) || a.max.is_some_and(|max| *value > max) {
                    return Err(ChartError::new(
                        "value_outside_axis",
                        format!("{path}/values/{axis}"),
                        "the value lies outside the range of its axis; widen `min` and `max` of the axis (`min` is 0 by default)",
                    ));
                }
            }
        }
        let mut warnings = Vec::new();
        if radar.axes.len() > READABLE_AXES {
            warnings.push(ChartWarning::new(
                "dense_chart",
                "/radar/axes",
                format!(
                    "more than {READABLE_AXES} axes are hard to compare on a radar chart; consider bars"
                ),
            ));
        }
        if radar.series.len() > READABLE_SERIES {
            warnings.push(ChartWarning::new(
                "dense_chart",
                "/radar/series",
                format!(
                    "more than {READABLE_SERIES} polygons overlap and are hard to tell apart; consider fewer series"
                ),
            ));
        }
        Ok(warnings)
    }
}
