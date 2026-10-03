use serde::{Deserialize, Serialize};

use super::{
    ChartSpec, Dash, DiagramOrientation, is_false, sequence::is_identifier, validate_text,
};
use crate::error::{ChartError, ChartWarning};

/// States of one diagram, and transitions between them; beyond this a layered layout stops being
/// readable at a glance.
pub(crate) const MAX_STATES: usize = 40;
pub(crate) const MAX_TRANSITIONS: usize = 80;

/// A state diagram: states joined by transitions, laid out like a flow chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StateSpec {
    pub states: Vec<StateNodeSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<TransitionSpec>,
    /// The `id` of the state the machine starts in; it gets the initial dot and its arrow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial: Option<String>,
    /// The usual way through the states, as ids joined by transitions: kept in line and drawn
    /// stronger.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub main_path: Vec<String>,
    #[serde(default, skip_serializing_if = "DiagramOrientation::is_auto")]
    pub orientation: DiagramOrientation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StateNodeSpec {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
    #[serde(default, skip_serializing_if = "StateKind::is_state")]
    pub kind: StateKind,
    /// A state the machine ends in, drawn with a double outline.
    #[serde(default, rename = "final", skip_serializing_if = "is_false")]
    pub is_final: bool,
}

/// What a state is.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StateKind {
    /// A state the machine rests in: a box with well rounded corners.
    #[default]
    State,
    /// A choice the machine passes through at once, by its guards: a diamond.
    Choice,
}

impl StateKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_state(&self) -> bool {
        matches!(self, Self::State)
    }
}

/// A transition from one state to another, or to itself, written `event [guard] / action`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionSpec {
    pub from: String,
    pub to: String,
    /// What triggers the transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// The condition under which it is taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    /// What happens on the way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash: Option<Dash>,
}

impl TransitionSpec {
    /// The transition's label in the usual notation: `event [guard] / action`, each part only
    /// where it is given; `None` without any.
    pub(crate) fn label(&self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(event) = &self.event {
            parts.push(event.clone());
        }
        if let Some(guard) = &self.guard {
            parts.push(format!("[{guard}]"));
        }
        if let Some(action) = &self.action {
            parts.push(format!("/ {action}"));
        }
        (!parts.is_empty()).then(|| parts.join(" "))
    }
}

impl StateSpec {
    /// The index of the state with `id`.
    pub(crate) fn state(&self, id: &str) -> Option<usize> {
        self.states.iter().position(|state| state.id == id)
    }

    /// Whether transition `index` joins two consecutive states of the main path.
    pub(crate) fn on_main_path(&self, index: usize) -> bool {
        let transition = &self.transitions[index];
        self.main_path
            .windows(2)
            .any(|pair| pair[0] == transition.from && pair[1] == transition.to)
    }
}

impl ChartSpec {
    /// A state diagram: states with unique identifiers, transitions between them, an initial
    /// state and a main path along transitions. A state nothing leads to, other than the initial
    /// one, is reported as unreachable.
    pub(super) fn validate_state(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a state diagram", "state.states")?;
        let Some(machine) = &self.state else {
            return Err(ChartError::new(
                "missing_state",
                "/state",
                "a state diagram requires a state block",
            ));
        };
        machine.validate_states()?;
        machine.validate_transitions()?;
        machine.validate_initial_and_path()?;
        let mut warnings = Vec::new();
        if let Some(initial) = &machine.initial {
            for (index, state) in machine.states.iter().enumerate() {
                let reached = machine
                    .transitions
                    .iter()
                    .any(|transition| transition.to == state.id && transition.from != state.id);
                if !reached && &state.id != initial {
                    warnings.push(ChartWarning::new(
                        "unreachable_state",
                        format!("/state/states/{index}"),
                        format!(
                            "no transition leads to \"{}\" and it is not the initial state",
                            state.id
                        ),
                    ));
                }
            }
        }
        Ok(warnings)
    }
}

impl StateSpec {
    fn validate_states(&self) -> Result<(), ChartError> {
        if self.states.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/state/states",
                "provide at least one state",
            ));
        }
        if self.states.len() > MAX_STATES {
            return Err(ChartError::new(
                "too_many_states",
                "/state/states",
                format!("at most {MAX_STATES} states are supported"),
            ));
        }
        for (index, state) in self.states.iter().enumerate() {
            let path = format!("/state/states/{index}");
            if !is_identifier(&state.id) {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/id"),
                    "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            if self.states[..index]
                .iter()
                .any(|earlier| earlier.id == state.id)
            {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    format!(
                        "the id \"{}\" is already taken by an earlier state",
                        state.id
                    ),
                ));
            }
            validate_text(&state.label, &format!("{path}/label"), 60)?;
            if let Some(sublabel) = &state.sublabel {
                validate_text(sublabel, &format!("{path}/sublabel"), 60)?;
            }
            if state.is_final && state.kind == StateKind::Choice {
                return Err(ChartError::new(
                    "option_not_supported",
                    format!("{path}/final"),
                    "a choice is passed through at once and cannot be final",
                ));
            }
        }
        Ok(())
    }

    fn validate_transitions(&self) -> Result<(), ChartError> {
        if self.transitions.len() > MAX_TRANSITIONS {
            return Err(ChartError::new(
                "too_many_transitions",
                "/state/transitions",
                format!("at most {MAX_TRANSITIONS} transitions are supported"),
            ));
        }
        for (index, transition) in self.transitions.iter().enumerate() {
            let path = format!("/state/transitions/{index}");
            for (field, id) in [("from", &transition.from), ("to", &transition.to)] {
                if self.state(id).is_none() {
                    return Err(ChartError::new(
                        "unknown_state",
                        format!("{path}/{field}"),
                        format!("no state has the id \"{id}\""),
                    ));
                }
            }
            for (field, text) in [
                ("event", &transition.event),
                ("guard", &transition.guard),
                ("action", &transition.action),
            ] {
                if let Some(text) = text {
                    validate_text(text, &format!("{path}/{field}"), 40)?;
                }
            }
        }
        Ok(())
    }

    fn validate_initial_and_path(&self) -> Result<(), ChartError> {
        if let Some(initial) = &self.initial
            && self.state(initial).is_none()
        {
            return Err(ChartError::new(
                "unknown_state",
                "/state/initial",
                format!("no state has the id \"{initial}\""),
            ));
        }
        for (index, id) in self.main_path.iter().enumerate() {
            if self.state(id).is_none() {
                return Err(ChartError::new(
                    "unknown_state",
                    format!("/state/mainPath/{index}"),
                    format!("no state has the id \"{id}\""),
                ));
            }
        }
        for (index, pair) in self.main_path.windows(2).enumerate() {
            let joined = self
                .transitions
                .iter()
                .any(|transition| transition.from == pair[0] && transition.to == pair[1]);
            if !joined {
                return Err(ChartError::new(
                    "main_path_gap",
                    format!("/state/mainPath/{}", index + 1),
                    format!(
                        "no transition leads from \"{}\" to \"{}\"; the main path follows transitions",
                        pair[0], pair[1]
                    ),
                ));
            }
        }
        Ok(())
    }
}
