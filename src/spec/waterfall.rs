use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_DATA_POINTS, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// A waterfall: a running total that moves up and down step by step, from a start to a total.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WaterfallSpec {
    pub steps: Vec<StepSpec>,
}

/// What a step does to the running total.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepKind {
    /// Adds its value to the running total, or takes it away when it is negative.
    #[default]
    Delta,
    /// Starts the total at its value, as a bar from zero.
    Start,
    /// Shows the running total so far, as a bar from zero: a subtotal, or the end result. It has
    /// no value of its own; if it names one, the value must be the running total.
    Total,
}

impl StepKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_delta(&self) -> bool {
        matches!(self, Self::Delta)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StepSpec {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(default, skip_serializing_if = "StepKind::is_delta")]
    pub kind: StepKind,
}

/// One bar of a waterfall: where it starts and ends along the value axis, the change it stands
/// for and the running total after it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WaterfallBar {
    pub from: f64,
    pub to: f64,
    /// The change of a delta, the value of a start, the total of a total.
    pub value: f64,
    pub kind: StepKind,
    pub running: f64,
}

/// The most steps one waterfall draws.
const MAX_STEPS: usize = 40;

impl WaterfallSpec {
    /// The bars of the waterfall, with the running total carried from step to step.
    pub(crate) fn bars(&self) -> Vec<WaterfallBar> {
        let mut running = 0.0;
        self.steps
            .iter()
            .map(|step| match step.kind {
                StepKind::Delta => {
                    let change = step.value.expect("validated");
                    let from = running;
                    running += change;
                    WaterfallBar {
                        from,
                        to: running,
                        value: change,
                        kind: step.kind,
                        running,
                    }
                }
                StepKind::Start => {
                    running = step.value.expect("validated");
                    WaterfallBar {
                        from: 0.0,
                        to: running,
                        value: running,
                        kind: step.kind,
                        running,
                    }
                }
                StepKind::Total => WaterfallBar {
                    from: 0.0,
                    to: running,
                    value: running,
                    kind: step.kind,
                    running,
                },
            })
            .collect()
    }
}

impl ChartSpec {
    /// Steps with unique labels: a delta or a start has a value, a total none, or the running
    /// total, which it must then match.
    pub(super) fn validate_waterfall(&self) -> Result<Vec<ChartWarning>, ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
            ("/references", !self.references.is_empty()),
            ("/timeAxis", !self.time_axis.is_default()),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    "a waterfall is drawn from steps; remove this field",
                ));
            }
        }
        let Some(waterfall) = &self.waterfall else {
            return Err(ChartError::new(
                "missing_waterfall",
                "/waterfall",
                "a waterfall requires a waterfall block",
            ));
        };
        if waterfall.steps.len() < 2 {
            return Err(ChartError::new(
                "empty_data",
                "/waterfall/steps",
                "provide at least two steps",
            ));
        }
        if waterfall.steps.len() > MAX_STEPS.min(MAX_DATA_POINTS) {
            return Err(ChartError::new(
                "too_many_data_points",
                "/waterfall/steps",
                format!("at most {MAX_STEPS} steps are supported"),
            ));
        }
        let mut labels = BTreeSet::new();
        let mut running = 0.0;
        for (index, step) in waterfall.steps.iter().enumerate() {
            let path = format!("/waterfall/steps/{index}");
            validate_text(&step.label, &format!("{path}/label"), 100)?;
            if !labels.insert(step.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "step labels must be unique",
                ));
            }
            if let Some(value) = step.value {
                validate_number(value, &format!("{path}/value"))?;
            }
            match (step.kind, step.value) {
                (StepKind::Delta, Some(change)) => running += change,
                (StepKind::Start, Some(start)) => running = start,
                (StepKind::Delta | StepKind::Start, None) => {
                    return Err(ChartError::new(
                        "missing_value",
                        format!("{path}/value"),
                        "a delta and a start need a value",
                    ));
                }
                (StepKind::Total, Some(total)) => {
                    if (total - running).abs() > 1e-9 * running.abs().max(1.0) {
                        return Err(ChartError::new(
                            "total_mismatch",
                            format!("{path}/value"),
                            "a total is the sum of the steps before it; leave the value out or correct it",
                        ));
                    }
                }
                (StepKind::Total, None) => {}
            }
        }
        Ok(Vec::new())
    }
}
