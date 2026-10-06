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
