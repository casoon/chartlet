use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many stages a funnel or a pyramid may have.
pub(crate) const MIN_STAGES: usize = 2;
pub(crate) const MAX_STAGES: usize = 12;

/// A funnel: stages from the first to the last, each narrower than the one before as the value
/// falls; or a pyramid of ranked levels, the first at the top.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FunnelSpec {
    pub stages: Vec<FunnelStageSpec>,
    /// `funnel` (default): centered bars as wide as the values, with the rate between two stages.
    /// `pyramid`: levels of a triangle, the widest at the bottom; the values are optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<FunnelDisplay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FunnelStageSpec {
    pub label: String,
    /// The value of the stage; positive. A funnel needs it on every stage, a pyramid on all or none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FunnelDisplay {
    /// Bars as wide as the values, centered, with the rate between two stages.
    #[default]
    Funnel,
    /// Levels of a triangle, the widest at the bottom.
    Pyramid,
}

impl FunnelSpec {
    pub(crate) fn is_pyramid(&self) -> bool {
        self.display == Some(FunnelDisplay::Pyramid)
    }

    /// The value of every stage, if the stages have values.
    pub(crate) fn values(&self) -> Option<Vec<f64>> {
        self.stages.iter().map(|stage| stage.value).collect()
    }
}

impl ChartSpec {
    /// Two to twelve stages with unique labels and positive values: on every stage of a funnel,
    /// on all or none of a pyramid.
    pub(super) fn validate_funnel(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
                    "a funnel is drawn from stages; remove this field",
                ));
            }
        }
        let Some(funnel) = &self.funnel else {
            return Err(ChartError::new(
                "missing_funnel",
                "/funnel",
                "a funnel requires a funnel block",
            ));
        };
        if funnel.stages.len() < MIN_STAGES {
            return Err(ChartError::new(
                "empty_data",
                "/funnel/stages",
                format!("provide at least {MIN_STAGES} stages"),
            ));
        }
        if funnel.stages.len() > MAX_STAGES {
            return Err(ChartError::new(
                "too_many_data_points",
                "/funnel/stages",
                format!("at most {MAX_STAGES} stages are supported"),
            ));
        }
        let with_values = funnel
            .stages
            .iter()
            .filter(|stage| stage.value.is_some())
            .count();
        let mut labels = BTreeSet::new();
        for (index, stage) in funnel.stages.iter().enumerate() {
            let path = format!("/funnel/stages/{index}");
            validate_text(&stage.label, &format!("{path}/label"), 100)?;
            if !labels.insert(stage.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "stage labels must be unique",
                ));
            }
            match stage.value {
                Some(value) => {
                    validate_number(value, &format!("{path}/value"))?;
                    if value <= 0.0 {
                        return Err(ChartError::new(
                            "invalid_value",
                            format!("{path}/value"),
                            "a stage has a value above zero",
                        ));
                    }
                }
                None if !funnel.is_pyramid() || with_values > 0 => {
                    return Err(ChartError::new(
                        "invalid_value",
                        format!("{path}/value"),
                        "a funnel needs a value on every stage; a pyramid takes values on all levels or on none",
                    ));
                }
                None => {}
            }
        }
        let mut warnings = Vec::new();
        if let (false, Some(values)) = (funnel.is_pyramid(), funnel.values()) {
            for (index, pair) in values.windows(2).enumerate() {
                if pair[1] > pair[0] {
                    warnings.push(ChartWarning::new(
                        "stage_exceeds_previous",
                        format!("/funnel/stages/{}/value", index + 1),
                        "a stage is larger than the one before, so the funnel widens; check the order or use a pyramid",
                    ));
                }
            }
        }
        Ok(warnings)
    }
}
