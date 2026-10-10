use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_SERIES, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// The most observations one group holds.
const MAX_OBSERVATIONS: usize = 2000;

/// Kaplan-Meier curves: the share of each group still without the event as time passes, from
/// observed times of which some ended in the event and some were cut off (censored).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurvivalSpec {
    pub groups: Vec<SurvivalGroupSpec>,
    /// Draws the 95 % confidence band of every curve (log-log transformed, from Greenwood's
    /// variance).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub confidence: bool,
    /// Tests whether the curves differ (log-rank, Mantel-Cox) and writes the result in the plot
    /// and the description. Needs two or more groups.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub log_rank: bool,
    /// Writes the number still at risk under the time axis, for every group. On by default.
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub at_risk: bool,
    /// The title of the time axis, such as "Months".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_title: Option<String>,
}

const fn yes() -> bool {
    true
}

// serde hands this function a reference, so the signature follows serde's shape.
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_yes(value: &bool) -> bool {
    *value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurvivalGroupSpec {
    pub label: String,
    pub observations: Vec<SurvivalObservationSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurvivalObservationSpec {
    /// The time until the event, or until the subject left the study.
    pub time: f64,
    /// Whether the event happened at that time; `false` cuts the observation off (censored).
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub event: bool,
}

/// The curve after one time at which events happened.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct KmStep {
    pub time: f64,
    pub survival: f64,
    pub lower: f64,
    pub upper: f64,
}

/// The Kaplan-Meier estimate of one group.
#[derive(Debug, Clone)]
pub(crate) struct KmCurve {
    /// From `(0, 1)`, one step for every time with events.
    pub steps: Vec<KmStep>,
    /// Where observations were cut off: their time and the survival there.
    pub censored: Vec<(f64, f64)>,
    /// The last time observed.
    pub end: f64,
    pub events: usize,
    /// The first time the survival falls to a half or below.
    pub median: Option<f64>,
}

impl KmCurve {
    /// The step in force at `time`: the last one at or before it.
    pub(crate) fn at(&self, time: f64) -> KmStep {
        self.steps
            .iter()
            .rev()
            .find(|step| step.time <= time)
            .copied()
            .expect("a curve starts at time zero")
    }
}

/// The width of the confidence band, in standard errors.
const Z: f64 = 1.96;

impl SurvivalGroupSpec {
    /// The Kaplan-Meier estimate with the Greenwood variance, log-log confidence limits and the
    /// censoring marks.
    pub(crate) fn curve(&self) -> KmCurve {
        let mut sorted: Vec<(f64, bool)> = self
            .observations
            .iter()
            .map(|observation| (observation.time, observation.event))
            .collect();
        sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut steps = vec![KmStep {
            time: 0.0,
            survival: 1.0,
            lower: 1.0,
            upper: 1.0,
        }];
        let (mut censored, mut survival, mut variance, mut events) = (Vec::new(), 1.0, 0.0, 0);
        let mut index = 0;
        while index < sorted.len() {
            let time = sorted[index].0;
            let at_risk = to_f64(sorted.len() - index);
            let same = sorted[index..]
                .iter()
                .take_while(|entry| entry.0.total_cmp(&time).is_eq());
            let (mut died, mut left) = (0, 0);
            for entry in same {
                if entry.1 {
                    died += 1;
                } else {
                    left += 1;
                }
            }
            if died > 0 {
                let died_f = to_f64(died);
                survival *= 1.0 - died_f / at_risk;
                if at_risk > died_f {
                    variance += died_f / (at_risk * (at_risk - died_f));
                }
                events += died;
                steps.push(limits(time, survival, variance));
            }
            censored.extend(std::iter::repeat_n((time, survival), left));
            index += died + left;
        }
        let median = steps
            .iter()
            .find(|step| step.survival <= 0.5)
            .map(|step| step.time);
        KmCurve {
            steps,
            censored,
            end: sorted.last().map_or(0.0, |entry| entry.0),
            events,
            median,
        }
    }

    /// How many observations reach `time` or beyond.
    pub(crate) fn at_risk(&self, time: f64) -> usize {
        self.observations
            .iter()
            .filter(|observation| observation.time >= time)
            .count()
    }
}

/// The step with its log-log confidence limits: the survival raised to a power that grows with
/// the Greenwood sum, so that the limits stay between zero and one.
fn limits(time: f64, survival: f64, variance: f64) -> KmStep {
    if survival <= 0.0 || survival >= 1.0 {
        return KmStep {
            time,
            survival,
            lower: survival,
            upper: survival,
        };
    }
    let error = variance.sqrt() / survival.ln().abs();
    KmStep {
        time,
        survival,
        lower: survival.powf((Z * error).exp()),
        upper: survival.powf((-Z * error).exp()),
    }
}

#[allow(clippy::cast_precision_loss)]
const fn to_f64(count: usize) -> f64 {
    count as f64
}

/// The log-rank test of two or more groups: the statistic, its degrees of freedom and p.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LogRank {
    pub chi_squared: f64,
    pub degrees: usize,
    pub p: f64,
}

impl SurvivalSpec {
    /// The log-rank (Mantel-Cox) test of the groups; `None` for one group, no events or a
    /// covariance that cannot be inverted.
    pub(crate) fn log_rank_test(&self) -> Option<LogRank> {
        let k = self.groups.len();
        if k < 2 {
            return None;
        }
        let mut times: Vec<f64> = self
            .groups
            .iter()
            .flat_map(|group| {
                group
                    .observations
                    .iter()
                    .filter(|o| o.event)
                    .map(|o| o.time)
            })
            .collect();
        times.sort_by(f64::total_cmp);
        times.dedup_by(|a, b| a.total_cmp(b).is_eq());
        let mut gap = vec![0.0; k];
        let mut variance = vec![vec![0.0; k]; k];
        for time in times {
            let at_risk: Vec<f64> = self
                .groups
                .iter()
                .map(|group| to_f64(group.observations.iter().filter(|o| o.time >= time).count()))
                .collect();
            let died: Vec<f64> = self
                .groups
                .iter()
                .map(|group| {
                    to_f64(
                        group
                            .observations
                            .iter()
                            .filter(|o| o.event && o.time.total_cmp(&time).is_eq())
                            .count(),
                    )
                })
                .collect();
            let (n, d) = (at_risk.iter().sum::<f64>(), died.iter().sum::<f64>());
            for j in 0..k {
                gap[j] += died[j] - d * at_risk[j] / n;
                if n > 1.0 {
                    for l in 0..k {
                        let kronecker = if j == l { 1.0 } else { 0.0 };
                        variance[j][l] +=
                            d * (at_risk[j] / n) * (kronecker - at_risk[l] / n) * (n - d)
                                / (n - 1.0);
                    }
                }
            }
        }
        let degrees = k - 1;
        let chi_squared = quadratic(&variance, &gap, degrees)?;
        Some(LogRank {
            chi_squared,
            degrees,
            p: chi_squared_tail(chi_squared, degrees),
        })
    }
}

/// `gapᵀ · variance⁻¹ · gap` over the first `size` groups, by Gaussian elimination; `None` if
/// the matrix is singular.
fn quadratic(variance: &[Vec<f64>], gap: &[f64], size: usize) -> Option<f64> {
    let mut rows: Vec<Vec<f64>> = (0..size)
        .map(|i| {
            let mut row: Vec<f64> = variance[i][..size].to_vec();
            row.push(gap[i]);
            row
        })
        .collect();
    for column in 0..size {
        let pivot = (column..size)
            .max_by(|a, b| rows[*a][column].abs().total_cmp(&rows[*b][column].abs()))?;
        if rows[pivot][column].abs() < 1e-12 {
            return None;
        }
        rows.swap(column, pivot);
        for row in column + 1..size {
            let factor = rows[row][column] / rows[column][column];
            let above = rows[column].clone();
            for (cell, top) in rows[row].iter_mut().zip(above).skip(column) {
                *cell -= factor * top;
            }
        }
    }
    let mut solution = vec![0.0; size];
    for row in (0..size).rev() {
        let known: f64 = (row + 1..size).map(|at| rows[row][at] * solution[at]).sum();
        solution[row] = (rows[row][size] - known) / rows[row][row];
    }
    Some((0..size).map(|i| gap[i] * solution[i]).sum())
}

/// The chance of a chi-squared value at least this large: the regularized upper incomplete gamma
/// function of `degrees / 2` and `value / 2`.
fn chi_squared_tail(value: f64, degrees: usize) -> f64 {
    let (shape, half) = (to_f64(degrees) / 2.0, value / 2.0);
    if half <= 0.0 {
        return 1.0;
    }
    let log_gamma = ln_gamma(shape);
    if half < shape + 1.0 {
        // The series of the lower function.
        let (mut term, mut sum) = (1.0 / shape, 1.0 / shape);
        for n in 1..500 {
            term *= half / (shape + f64::from(n));
            sum += term;
            if term.abs() < sum.abs() * 1e-15 {
                break;
            }
        }
        (1.0 - sum * (-half + shape * half.ln() - log_gamma).exp()).clamp(0.0, 1.0)
    } else {
        // The continued fraction of the upper function (modified Lentz).
        let tiny = 1e-300;
        let mut offset = half + 1.0 - shape;
        let mut c = 1.0 / tiny;
        let mut d = 1.0 / offset;
        let mut h = d;
        for n in 1..500 {
            let an = -f64::from(n) * (f64::from(n) - shape);
            offset += 2.0;
            d = an * d + offset;
            if d.abs() < tiny {
                d = tiny;
            }
            c = offset + an / c;
            if c.abs() < tiny {
                c = tiny;
            }
            d = 1.0 / d;
            let delta = d * c;
            h *= delta;
            if (delta - 1.0).abs() < 1e-15 {
                break;
            }
        }
        ((-half + shape * half.ln() - log_gamma).exp() * h).clamp(0.0, 1.0)
    }
}

/// The logarithm of the gamma function, by the Lanczos approximation.
fn ln_gamma(x: f64) -> f64 {
    const COEFFICIENTS: [f64; 6] = [
        76.180_091_729_471_46,
        -86.505_320_329_416_77,
        24.014_098_240_830_91,
        -1.231_739_572_450_155,
        0.001_208_650_973_866_179,
        -0.000_005_395_239_384_953,
    ];
    let mut series = 1.000_000_000_190_015;
    for (offset, coefficient) in COEFFICIENTS.iter().enumerate() {
        series += coefficient / (x + 1.0 + to_f64(offset));
    }
    let shifted = x + 5.5;
    (x + 0.5) * shifted.ln() - shifted + (2.506_628_274_631_000_5 * series / x).ln()
}

impl ChartSpec {
    /// One to four groups with unique labels, each with observed times of at least zero.
    pub(super) fn validate_survival(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
                    "survival curves are drawn from groups; remove this field",
                ));
            }
        }
        let Some(survival) = &self.survival else {
            return Err(ChartError::new(
                "missing_survival",
                "/survival",
                "survival curves require a survival block",
            ));
        };
        if survival.groups.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/survival/groups",
                "provide at least one group",
            ));
        }
        if survival.groups.len() > MAX_SERIES {
            return Err(ChartError::new(
                "too_many_series",
                "/survival/groups",
                format!("at most {MAX_SERIES} groups are supported, one per palette color"),
            ));
        }
        if survival.log_rank && survival.groups.len() < 2 {
            return Err(ChartError::new(
                "log_rank_needs_groups",
                "/survival/logRank",
                "the log-rank test compares two or more groups",
            ));
        }
        if let Some(title) = &survival.time_title {
            validate_text(title, "/survival/timeTitle", 100)?;
        }
        let mut labels = BTreeSet::new();
        for (index, group) in survival.groups.iter().enumerate() {
            let path = format!("/survival/groups/{index}");
            validate_text(&group.label, &format!("{path}/label"), 100)?;
            if !labels.insert(group.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "group labels must be unique",
                ));
            }
            if group.observations.len() < 2 || group.observations.len() > MAX_OBSERVATIONS {
                return Err(ChartError::new(
                    "invalid_values",
                    format!("{path}/observations"),
                    format!("give 2 to {MAX_OBSERVATIONS} observations for every group"),
                ));
            }
            for (at, observation) in group.observations.iter().enumerate() {
                let time_path = format!("{path}/observations/{at}/time");
                validate_number(observation.time, &time_path)?;
                if observation.time < 0.0 {
                    return Err(ChartError::new(
                        "invalid_value",
                        time_path,
                        "a time must be zero or more",
                    ));
                }
            }
        }
        Ok(Vec::new())
    }
}
