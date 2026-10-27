use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, DiagramRole, sequence::is_identifier, validate_text};
use crate::error::{ChartError, ChartWarning};

pub(crate) const MIN_STEPS: usize = 3;
pub(crate) const MAX_STEPS: usize = 8;

/// A cycle: steps on a ring where the last step feeds the first, optionally with a hub in the
/// middle that the steps feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CycleSpec {
    /// The steps clockwise from twelve o'clock; 3 to 8.
    pub steps: Vec<CycleStepSpec>,
    /// What the steps feed, drawn in the middle with a link from every step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hub: Option<CycleHubSpec>,
    /// Where the labels stand: outside the ring beside numbered dots, or inside boxes on it.
    #[serde(default, skip_serializing_if = "CycleLabels::is_outside")]
    pub labels: CycleLabels,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CycleStepSpec {
    pub id: String,
    pub label: String,
    /// A second, smaller line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// The label of this step's link to the hub; needs a hub.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hub_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<DiagramRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CycleHubSpec {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Where the steps of a cycle carry their labels.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CycleLabels {
    /// Numbered dots on the ring, the labels outside it.
    #[default]
    Outside,
    /// Boxes on the ring that hold the labels.
    Inside,
}

impl CycleLabels {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_outside(&self) -> bool {
        matches!(self, Self::Outside)
    }
}

impl ChartSpec {
    /// Three to eight steps with unique ids and texts, and hub labels only where there is a hub.
    pub(super) fn validate_cycle(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a cycle", "cycle.steps")?;
        let Some(cycle) = &self.cycle else {
            return Err(ChartError::new(
                "missing_cycle",
                "/cycle",
                "a cycle requires a cycle block",
            ));
        };
        if cycle.steps.len() < MIN_STEPS {
            return Err(ChartError::new(
                "too_few_steps",
                "/cycle/steps",
                format!(
                    "a cycle needs at least {MIN_STEPS} steps; fewer read better as a sentence or a flow chart"
                ),
            ));
        }
        if cycle.steps.len() > MAX_STEPS {
            return Err(ChartError::new(
                "too_many_steps",
                "/cycle/steps",
                format!("at most {MAX_STEPS} steps fit a ring; group steps or use a flow chart"),
            ));
        }
        let mut ids = BTreeSet::new();
        for (index, step) in cycle.steps.iter().enumerate() {
            let path = format!("/cycle/steps/{index}");
            if !is_identifier(&step.id) {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/id"),
                    "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            if !ids.insert(step.id.as_str()) {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    format!("the id \"{}\" is already taken by an earlier step", step.id),
                ));
            }
            validate_text(&step.label, &format!("{path}/label"), 60)?;
            if let Some(note) = &step.note {
                validate_text(note, &format!("{path}/note"), 80)?;
            }
            if let Some(label) = &step.hub_label {
                validate_text(label, &format!("{path}/hubLabel"), 24)?;
                if cycle.hub.is_none() {
                    return Err(ChartError::new(
                        "option_not_supported",
                        format!("{path}/hubLabel"),
                        "a hub label names the link to the hub; add a hub or remove hubLabel",
                    ));
                }
            }
        }
        if let Some(hub) = &cycle.hub {
            validate_text(&hub.label, "/cycle/hub/label", 60)?;
            if let Some(note) = &hub.note {
                validate_text(note, "/cycle/hub/note", 80)?;
            }
        }
        Ok(Vec::new())
    }
}
