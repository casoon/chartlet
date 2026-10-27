use serde::{Deserialize, Serialize};

use super::{ChartSpec, validate_optional_text};
use crate::error::{ChartError, ChartWarning};

/// How many stages a journey may have, and how long the text of a cell may be.
const MAX_STAGES: usize = 8;
const MAX_CELL: usize = 160;

/// A user journey: stages as columns with what the customer does, where, thinks and feels in each;
/// as a service blueprint it adds what the service does in front of and behind the customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JourneySpec {
    pub stages: Vec<JourneyStageSpec>,
    /// `journey` (default) or `blueprint`, which adds the rows of the service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<JourneyDisplay>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JourneyDisplay {
    /// The customer's rows: actions, touchpoints, thoughts and sentiment.
    #[default]
    Journey,
    /// A service blueprint: the customer's rows, the line of interaction, the frontstage, the
    /// line of visibility, the backstage and the support.
    Blueprint,
}

/// One stage of the journey. Every row is optional; a row that no stage fills is left out.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JourneyStageSpec {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub touchpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,
    /// How the customer feels, from -2 (very negative) to 2 (very positive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentiment: Option<i8>,
    /// What the service does where the customer sees it (blueprint only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontstage: Option<String>,
    /// What the service does out of the customer's sight (blueprint only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backstage: Option<String>,
    /// The systems and processes that carry the service (blueprint only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support: Option<String>,
}

impl JourneySpec {
    pub(crate) fn is_blueprint(&self) -> bool {
        self.display == Some(JourneyDisplay::Blueprint)
    }
}

impl JourneyStageSpec {
    /// The text of a row of the stage.
    pub(crate) fn text(&self, row: JourneyRow) -> Option<&str> {
        match row {
            JourneyRow::Action => &self.action,
            JourneyRow::Touchpoint => &self.touchpoint,
            JourneyRow::Thought => &self.thought,
            JourneyRow::Frontstage => &self.frontstage,
            JourneyRow::Backstage => &self.backstage,
            JourneyRow::Support => &self.support,
        }
        .as_deref()
    }
}

/// The rows of text of a journey, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JourneyRow {
    Action,
    Touchpoint,
    Thought,
    Frontstage,
    Backstage,
    Support,
}

impl ChartSpec {
    /// Two to eight stages with unique labels, sentiments from -2 to 2, and the rows of a service
    /// blueprint only on a blueprint.
    pub(super) fn validate_journey(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_chart_fields("a journey is drawn from its stages")?;
        let Some(journey) = &self.journey else {
            return Err(ChartError::new(
                "missing_journey",
                "/journey",
                "a journey requires a journey block",
            ));
        };
        if journey.stages.len() < 2 {
            return Err(ChartError::new(
                "empty_data",
                "/journey/stages",
                "provide at least two stages",
            ));
        }
        if journey.stages.len() > MAX_STAGES {
            return Err(ChartError::new(
                "too_many_nodes",
                "/journey/stages",
                format!("at most {MAX_STAGES} stages are supported; merge stages"),
            ));
        }
        let mut seen: Vec<&str> = Vec::new();
        for (index, stage) in journey.stages.iter().enumerate() {
            let path = format!("/journey/stages/{index}");
            super::validate_text(&stage.label, &format!("{path}/label"), 60)?;
            if seen.contains(&stage.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "stage labels must be unique",
                ));
            }
            seen.push(&stage.label);
            for (name, value) in [
                ("action", &stage.action),
                ("touchpoint", &stage.touchpoint),
                ("thought", &stage.thought),
                ("frontstage", &stage.frontstage),
                ("backstage", &stage.backstage),
                ("support", &stage.support),
            ] {
                validate_optional_text(value.as_ref(), &format!("{path}/{name}"), MAX_CELL)?;
                let service_row = matches!(name, "frontstage" | "backstage" | "support");
                if service_row && value.is_some() && !journey.is_blueprint() {
                    return Err(ChartError::new(
                        "option_not_supported",
                        format!("{path}/{name}"),
                        "the rows of the service belong to a blueprint; set \"display\": \"blueprint\" or remove this field",
                    ));
                }
            }
            if let Some(sentiment) = stage.sentiment
                && !(-2..=2).contains(&sentiment)
            {
                return Err(ChartError::new(
                    "invalid_value",
                    format!("{path}/sentiment"),
                    "sentiment is a whole number from -2 to 2",
                ));
            }
        }
        Ok(Vec::new())
    }
}
