use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many bones, causes per bone and sub-causes per cause a fishbone diagram may have.
pub(crate) const MAX_BONES: usize = 6;
const MAX_CAUSES: usize = 8;
const MAX_SUBCAUSES: usize = 4;

/// A fishbone (cause and effect) diagram: one effect at the head and, on bones that alternate above
/// and below the spine, the categories of its causes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FishboneSpec {
    pub effect: String,
    pub bones: Vec<FishboneBoneSpec>,
}

/// A category of causes: a bone with its causes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FishboneBoneSpec {
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causes: Vec<FishboneCauseSpec>,
}

/// A cause on a bone, with the sub-causes that lead to it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FishboneCauseSpec {
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causes: Vec<FishboneSubcauseSpec>,
}

/// A sub-cause: the second and last level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FishboneSubcauseSpec {
    pub label: String,
}

impl FishboneBoneSpec {
    /// How many causes and sub-causes the bone has.
    pub(crate) fn cause_count(&self) -> usize {
        self.causes.iter().map(|cause| 1 + cause.causes.len()).sum()
    }
}

impl ChartSpec {
    /// Refuses the fields of charts with axes on a type that is drawn from a block of its own;
    /// `noun` says what the type is drawn from.
    pub(super) fn reject_chart_fields(&self, noun: &str) -> Result<(), ChartError> {
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
                    format!("{noun}; remove this field"),
                ));
            }
        }
        Ok(())
    }

    /// An effect, one to six bones with unique labels, and at most two levels of causes.
    pub(super) fn validate_fishbone(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_chart_fields("a fishbone diagram is drawn from its effect and bones")?;
        let Some(fishbone) = &self.fishbone else {
            return Err(ChartError::new(
                "missing_fishbone",
                "/fishbone",
                "a fishbone diagram requires a fishbone block",
            ));
        };
        validate_text(&fishbone.effect, "/fishbone/effect", 100)?;
        if fishbone.bones.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/fishbone/bones",
                "provide at least one bone, a category of causes",
            ));
        }
        if fishbone.bones.len() > MAX_BONES {
            return Err(ChartError::new(
                "too_many_nodes",
                "/fishbone/bones",
                format!("at most {MAX_BONES} bones are supported; merge categories"),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, bone) in fishbone.bones.iter().enumerate() {
            let path = format!("/fishbone/bones/{index}");
            validate_text(&bone.label, &format!("{path}/label"), 100)?;
            if !labels.insert(bone.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "bone labels must be unique",
                ));
            }
            if bone.causes.len() > MAX_CAUSES {
                return Err(ChartError::new(
                    "too_many_nodes",
                    format!("{path}/causes"),
                    format!("at most {MAX_CAUSES} causes per bone are supported; merge causes"),
                ));
            }
            for (cause_index, cause) in bone.causes.iter().enumerate() {
                let cause_path = format!("{path}/causes/{cause_index}");
                validate_text(&cause.label, &format!("{cause_path}/label"), 100)?;
                if cause.causes.len() > MAX_SUBCAUSES {
                    return Err(ChartError::new(
                        "too_many_nodes",
                        format!("{cause_path}/causes"),
                        format!(
                            "at most {MAX_SUBCAUSES} sub-causes per cause are supported; merge sub-causes"
                        ),
                    ));
                }
                for (sub_index, sub) in cause.causes.iter().enumerate() {
                    validate_text(
                        &sub.label,
                        &format!("{cause_path}/causes/{sub_index}/label"),
                        100,
                    )?;
                }
            }
        }
        Ok(Vec::new())
    }
}
