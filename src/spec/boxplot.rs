use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_DATA_POINTS, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// The most observations one box may be computed from.
pub(crate) const MAX_OBSERVATIONS: usize = 1000;
/// The fewest observations a box is computed from.
pub(crate) const MIN_OBSERVATIONS: usize = 5;

/// How the observations of a `boxplot` chart are drawn.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BoxDisplay {
    /// A box with whiskers and the points beyond them.
    #[default]
    Box,
    /// The estimated density of the observations as a mirrored outline, with the box inside.
    Violin,
    /// Every observation as a point, spread across the width, with the median.
    Strip,
}

impl BoxDisplay {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(super) const fn is_box(&self) -> bool {
        matches!(self, Self::Box)
    }
}

/// One box of a `boxplot` chart: a category with either its observations, from which chartlet
/// computes the box, or the five numbers of a box that was computed elsewhere.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoxSpec {
    pub label: String,
    /// The observations: at least five. The box runs from the first to the third quartile, the
    /// whiskers to the most extreme observations within 1.5 times that distance, and every
    /// observation beyond them is drawn as a point.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<f64>,
    /// The lower end of the lower whisker. With `q1`, `median`, `q3` and `max`, instead of
    /// `values`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub q1: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub q3: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Points beyond the whiskers of a box given by its five numbers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outliers: Vec<f64>,
}

/// What a box shows: five numbers, the points beyond its whiskers and, if it was computed from
/// observations, how many there were.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BoxSummary {
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub outliers: Vec<f64>,
    pub count: Option<usize>,
}

/// The value `quarters` quarters into the sorted observations, by linear interpolation between
/// the two nearest ones (the default of R and `NumPy`).
fn quantile(sorted: &[f64], quarters: usize) -> f64 {
    let steps = quarters * (sorted.len() - 1);
    let (index, rest) = (steps / 4, steps % 4);
    match sorted.get(index + 1) {
        Some(next) if rest > 0 => sorted[index] + (next - sorted[index]) * count_as_f64(rest) / 4.0,
        _ => sorted[index],
    }
}

fn count_as_f64(count: usize) -> f64 {
    f64::from(u32::try_from(count).expect("observations are limited"))
}

impl BoxSpec {
    /// The five numbers of the box and its outliers.
    pub(crate) fn summary(&self) -> BoxSummary {
        if self.values.is_empty() {
            return BoxSummary {
                min: self.min.expect("validated"),
                q1: self.q1.expect("validated"),
                median: self.median.expect("validated"),
                q3: self.q3.expect("validated"),
                max: self.max.expect("validated"),
                outliers: self.outliers.clone(),
                count: None,
            };
        }
        let mut sorted = self.values.clone();
        sorted.sort_by(f64::total_cmp);
        let (q1, median, q3) = (
            quantile(&sorted, 1),
            quantile(&sorted, 2),
            quantile(&sorted, 3),
        );
        let reach = 1.5 * (q3 - q1);
        let (low, high) = (q1 - reach, q3 + reach);
        let inside = || sorted.iter().filter(|value| (low..=high).contains(*value));
        BoxSummary {
            min: inside().copied().next().unwrap_or(q1),
            q1,
            median,
            q3,
            max: inside().copied().last().unwrap_or(q3),
            outliers: sorted
                .iter()
                .copied()
                .filter(|value| !(low..=high).contains(value))
                .collect(),
            count: Some(sorted.len()),
        }
    }
}

impl ChartSpec {
    /// The summary of every box, in order.
    pub(crate) fn box_summaries(&self) -> Vec<BoxSummary> {
        self.boxes.iter().map(BoxSpec::summary).collect()
    }

    /// Boxes with unique labels, each from observations or from five ordered numbers.
    pub(super) fn validate_boxplot(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
                    "a boxplot is drawn from boxes; remove this field",
                ));
            }
        }
        if self.boxes.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/boxes",
                "provide at least one box",
            ));
        }
        if self.boxes.len() > MAX_DATA_POINTS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/boxes",
                format!("at most {MAX_DATA_POINTS} boxes are supported"),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, boxed) in self.boxes.iter().enumerate() {
            let path = format!("/boxes/{index}");
            if self.box_display != BoxDisplay::Box && boxed.values.is_empty() {
                return Err(ChartError::new(
                    "values_required",
                    format!("{path}/values"),
                    "a violin or a strip is drawn from observations; give values instead of the five numbers",
                ));
            }
            validate_text(&boxed.label, &format!("{path}/label"), 200)?;
            if !labels.insert(boxed.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "box labels must be unique",
                ));
            }
            boxed.validate(&path)?;
        }
        let mut warnings = Vec::new();
        if self.boxes.len() > 16 {
            warnings.push(ChartWarning::new(
                "dense_chart",
                "/boxes",
                "more than 16 boxes can be difficult to read at the configured size",
            ));
        }
        Ok(warnings)
    }
}

impl BoxSpec {
    fn validate(&self, path: &str) -> Result<(), ChartError> {
        let numbers = [self.min, self.q1, self.median, self.q3, self.max];
        let given = numbers.iter().flatten().count();
        if !self.values.is_empty() {
            if given > 0 || !self.outliers.is_empty() {
                return Err(ChartError::new(
                    "conflicting_data_shape",
                    format!("{path}/values"),
                    "use either values, or min, q1, median, q3, max and outliers, not both",
                ));
            }
            if self.values.len() < MIN_OBSERVATIONS || self.values.len() > MAX_OBSERVATIONS {
                return Err(ChartError::new(
                    "invalid_values",
                    format!("{path}/values"),
                    format!(
                        "give {MIN_OBSERVATIONS} to {MAX_OBSERVATIONS} values, or the five numbers of the box"
                    ),
                ));
            }
            for (index, value) in self.values.iter().enumerate() {
                validate_number(*value, &format!("{path}/values/{index}"))?;
            }
            return Ok(());
        }
        if given < 5 {
            return Err(ChartError::new(
                "incomplete_box",
                path.to_owned(),
                "give values, or all of min, q1, median, q3 and max",
            ));
        }
        let names = ["min", "q1", "median", "q3", "max"];
        let mut previous = f64::NEG_INFINITY;
        for (name, value) in names.iter().zip(numbers) {
            let value = value.expect("all five are given");
            validate_number(value, &format!("{path}/{name}"))?;
            if value < previous {
                return Err(ChartError::new(
                    "invalid_box",
                    format!("{path}/{name}"),
                    "min, q1, median, q3 and max must not decrease",
                ));
            }
            previous = value;
        }
        for (index, value) in self.outliers.iter().enumerate() {
            validate_number(*value, &format!("{path}/outliers/{index}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BoxSpec, quantile};

    #[test]
    fn quartiles_interpolate_like_r_and_numpy() {
        let sorted = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        assert!((quantile(&sorted, 2) - 3.5).abs() < 1e-12);
        assert!((quantile(&sorted, 1) - 2.25).abs() < 1e-12);
        assert!((quantile(&sorted, 3) - 4.75).abs() < 1e-12);
        assert!((quantile(&[7.0], 3) - 7.0).abs() < 1e-12);
    }

    #[test]
    fn whiskers_stop_at_the_last_value_within_one_and_a_half_boxes() {
        let boxed = BoxSpec {
            label: "A".to_owned(),
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 40.0, -30.0],
            min: None,
            q1: None,
            median: None,
            q3: None,
            max: None,
            outliers: Vec::new(),
        };
        let summary = boxed.summary();
        assert_eq!((summary.min, summary.max), (1.0, 6.0));
        assert_eq!(summary.outliers, vec![-30.0, 40.0]);
        assert_eq!(summary.count, Some(8));
    }
}
