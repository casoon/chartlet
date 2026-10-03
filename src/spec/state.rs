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
    /// The `id` of the composite state this state lies in.
    #[serde(default, rename = "in", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
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
    /// A state made of states: a frame around the states that name it with `in`.
    Composite,
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

/// How deeply composite states may nest inside each other.
pub(crate) const MAX_COMPOSITE_DEPTH: usize = 4;

impl StateSpec {
    /// Whether state `index` is a composite state, a frame rather than a state of its own.
    pub(crate) fn is_composite(&self, index: usize) -> bool {
        self.states[index].kind == StateKind::Composite
    }

    /// The composite states around state `index`, outermost first; `None` when they lie in each
    /// other in a circle.
    pub(crate) fn composites_around(&self, index: usize) -> Option<Vec<usize>> {
        let mut chain = Vec::new();
        let mut at = index;
        while let Some(parent) = self.states[at].parent.as_deref() {
            at = self.state(parent)?;
            if chain.contains(&at) || at == index {
                return None;
            }
            chain.push(at);
        }
        chain.reverse();
        Some(chain)
    }

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
                if machine.is_composite(index) {
                    continue;
                }
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
        self.validate_composites()
    }

    /// Composite states: they lie in composite states only, without a circle and at most four
    /// deep, are never final, and hold at least one state of their own.
    fn validate_composites(&self) -> Result<(), ChartError> {
        for (index, state) in self.states.iter().enumerate() {
            let path = format!("/state/states/{index}");
            if let Some(parent) = &state.parent {
                let Some(outer) = self.state(parent) else {
                    return Err(ChartError::new(
                        "unknown_state",
                        format!("{path}/in"),
                        format!("no state has the id \"{parent}\""),
                    ));
                };
                if !self.is_composite(outer) {
                    return Err(ChartError::new(
                        "not_composite",
                        format!("{path}/in"),
                        format!(
                            "\"{parent}\" is no composite state; give it \"kind\": \"composite\""
                        ),
                    ));
                }
            }
            let Some(chain) = self.composites_around(index) else {
                return Err(ChartError::new(
                    "state_cycle",
                    format!("{path}/in"),
                    "composite states lie in each other in a circle",
                ));
            };
            if chain.len() >= MAX_COMPOSITE_DEPTH && self.is_composite(index) {
                return Err(ChartError::new(
                    "states_too_deep",
                    format!("{path}/in"),
                    format!("composite states nest at most {MAX_COMPOSITE_DEPTH} deep"),
                ));
            }
            if self.is_composite(index) {
                if state.is_final {
                    return Err(ChartError::new(
                        "option_not_supported",
                        format!("{path}/final"),
                        "a composite state is a frame and cannot be final; make a state inside it final",
                    ));
                }
                let holds = (0..self.states.len()).any(|inner| {
                    !self.is_composite(inner)
                        && self
                            .composites_around(inner)
                            .is_some_and(|chain| chain.contains(&index))
                });
                if !holds {
                    return Err(ChartError::new(
                        "empty_composite",
                        path,
                        format!("no state lies in \"{}\"", state.id),
                    ));
                }
            }
        }
        Ok(())
    }

    /// An error when `id` names a composite state, which transitions cannot reach.
    fn reject_composite(&self, id: &str, path: String) -> Result<(), ChartError> {
        match self.state(id) {
            Some(index) if self.is_composite(index) => Err(ChartError::new(
                "composite_not_allowed",
                path,
                format!("\"{id}\" is a composite state, a frame; name a state inside it"),
            )),
            _ => Ok(()),
        }
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
                self.reject_composite(id, format!("{path}/{field}"))?;
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
        if let Some(initial) = &self.initial {
            self.reject_composite(initial, "/state/initial".to_owned())?;
        }
        for (index, id) in self.main_path.iter().enumerate() {
            if self.state(id).is_none() {
                return Err(ChartError::new(
                    "unknown_state",
                    format!("/state/mainPath/{index}"),
                    format!("no state has the id \"{id}\""),
                ));
            }
            self.reject_composite(id, format!("/state/mainPath/{index}"))?;
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
