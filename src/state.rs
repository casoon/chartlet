//! State diagrams: states joined by transitions, laid out by the layered layout of flow charts.
//! The initial state gets a dot with an arrow into it, final states a double outline, choices a
//! diamond. Transitions are written `event [guard] / action`; the description and the data table
//! list every one of them, the table with its parts in columns of their own.

use std::fmt::Write as _;

use crate::{
    DataTable,
    error::ChartWarning,
    flow::{Diagram, Edge, Node, Shape, layout_diagram, reading_order},
    metrics::TextMetrics,
    scene::Scene,
    spec::{ChartSpec, StateKind, StateSpec},
    text,
};

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    layout_diagram(spec, &diagram(spec), warnings, metrics)
}

fn machine(spec: &ChartSpec) -> &StateSpec {
    spec.state
        .as_ref()
        .expect("validated state diagrams carry a state block")
}

/// The diagram the layered layout draws: the initial dot first, when there is one, then every
/// state, then the transitions after the arrow into the initial state.
fn diagram(spec: &ChartSpec) -> Diagram {
    let machine = machine(spec);
    let words = spec.locale.words();
    let offset = usize::from(machine.initial.is_some());
    let mut nodes = Vec::with_capacity(machine.states.len() + offset);
    let mut edges = Vec::with_capacity(machine.transitions.len() + offset);
    if let Some(initial) = &machine.initial {
        nodes.push(Node {
            label: words.start.to_owned(),
            sublabel: None,
            shape: Shape::Initial,
            lane: None,
            group: None,
            path: "/state/initial".to_owned(),
        });
        edges.push(Edge {
            from: 0,
            to: offset + machine.state(initial).expect("validated"),
            label: None,
            dash: None,
            main: machine.main_path.first() == Some(initial),
            label_path: "/state/initial".to_owned(),
        });
    }
    for (index, state) in machine.states.iter().enumerate() {
        nodes.push(Node {
            label: state.label.clone(),
            sublabel: state.sublabel.clone(),
            shape: match (state.kind, state.is_final) {
                (StateKind::Choice, _) => Shape::Decision,
                (StateKind::State, true) => Shape::Final,
                (StateKind::State, false) => Shape::State,
            },
            lane: None,
            group: None,
            path: format!("/state/states/{index}"),
        });
    }
    for (index, transition) in machine.transitions.iter().enumerate() {
        edges.push(Edge {
            from: offset + machine.state(&transition.from).expect("validated"),
            to: offset + machine.state(&transition.to).expect("validated"),
            label: transition.label(),
            dash: transition.dash,
            main: machine.on_main_path(index),
            label_path: format!("/state/transitions/{index}"),
        });
    }
    Diagram {
        nodes,
        edges,
        lanes: Vec::new(),
        groups: Vec::new(),
        orientation: machine.orientation,
    }
}

/// The states in reading order, as indices into the specification's states.
fn states_in_order(spec: &ChartSpec) -> Vec<usize> {
    let offset = usize::from(machine(spec).initial.is_some());
    reading_order(&diagram(spec))
        .into_iter()
        .filter_map(|node| node.checked_sub(offset))
        .collect()
}

/// The transitions in reading order: by the state they leave, in the order of the
/// specification.
fn transitions_in_order(spec: &ChartSpec) -> Vec<usize> {
    let machine = machine(spec);
    states_in_order(spec)
        .into_iter()
        .flat_map(|state| {
            let id = &machine.states[state].id;
            machine
                .transitions
                .iter()
                .enumerate()
                .filter(move |(_, transition)| &transition.from == id)
                .map(|(index, _)| index)
        })
        .collect()
}

/// The diagram in sentences: its states, where it starts and ends, the main path, and every
/// transition in reading order.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let machine = machine(spec);
    let locale = spec.locale;
    let label = |id: &str| {
        machine.states[machine.state(id).expect("validated")]
            .label
            .as_str()
    };
    let initial = machine.initial.as_deref().map(label);
    let finals: Vec<String> = machine
        .states
        .iter()
        .filter(|state| state.is_final)
        .map(|state| state.label.clone())
        .collect();
    let mut description = text::state_opening(
        locale,
        machine.states.len(),
        machine.transitions.len(),
        initial,
        &finals,
    );
    if !machine.main_path.is_empty() {
        let path: Vec<&str> = machine.main_path.iter().map(|id| label(id)).collect();
        description.push(' ');
        description.push_str(&text::main_path(locale, &path));
    }
    for index in transitions_in_order(spec) {
        let transition = &machine.transitions[index];
        let to = (transition.from != transition.to).then(|| label(&transition.to));
        write!(
            description,
            " {}",
            text::transition(
                locale,
                label(&transition.from),
                to,
                transition.label().as_deref()
            )
        )
        .expect("writing to String cannot fail");
    }
    let choices: Vec<&str> = machine
        .states
        .iter()
        .filter(|state| state.kind == StateKind::Choice)
        .map(|state| state.label.as_str())
        .collect();
    if !choices.is_empty() {
        description.push(' ');
        description.push_str(&text::choices(locale, &choices));
    }
    description
}

/// One row per transition in reading order: the state it leaves, its event, guard and action,
/// and the state it reaches; first the start, when there is one.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let machine = machine(spec);
    let words = spec.locale.words();
    let label = |id: &str| {
        machine.states[machine.state(id).expect("validated")]
            .label
            .clone()
    };
    let columns = [
        words.sender,
        words.event,
        words.guard,
        words.action,
        words.next_state,
    ]
    .iter()
    .map(|word| (*word).to_owned())
    .collect();
    let mut rows = Vec::new();
    if let Some(initial) = &machine.initial {
        rows.push(vec![
            words.start.to_owned(),
            String::new(),
            String::new(),
            String::new(),
            label(initial),
        ]);
    }
    for index in transitions_in_order(spec) {
        let transition = &machine.transitions[index];
        rows.push(vec![
            label(&transition.from),
            transition.event.clone().unwrap_or_default(),
            transition.guard.clone().unwrap_or_default(),
            transition.action.clone().unwrap_or_default(),
            label(&transition.to),
        ]);
    }
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use crate::{ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative};

    const SPEC: &str = r#"{
        "schemaVersion": 1,
        "type": "state",
        "title": "Door",
        "state": {
            "initial": "closed",
            "states": [
                {"id": "closed", "label": "Closed"},
                {"id": "open", "label": "Open"},
                {"id": "locked", "label": "Locked"},
                {"id": "gone", "label": "Removed", "final": true}
            ],
            "transitions": [
                {"from": "closed", "to": "open", "event": "push", "guard": "unlocked"},
                {"from": "open", "to": "closed", "event": "release"},
                {"from": "closed", "to": "locked", "event": "lock", "action": "beep"},
                {"from": "locked", "to": "closed", "event": "unlock"},
                {"from": "open", "to": "open", "event": "wind"},
                {"from": "locked", "to": "gone", "event": "dismantle"}
            ],
            "mainPath": ["closed", "locked", "gone"]
        }
    }"#;

    fn svg(json: &str) -> crate::RenderOutput {
        render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect("the test specification renders")
    }

    fn error(json: &str) -> (&'static str, String) {
        let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("the test specification is invalid");
        (error.code, error.path)
    }

    #[test]
    fn states_transitions_and_the_initial_dot_are_drawn() {
        let output = svg(SPEC);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert!(
            output
                .content
                .contains("class=\"chartlet-state-initial\"><title>Start</title>")
        );
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-state-inner\"")
                .count(),
            1
        );
        assert!(
            output
                .content
                .contains("<title>Closed → Open: push [unlocked]</title>")
        );
        assert!(output.content.contains("<title>Open → Open: wind</title>"));
        assert!(output.content.contains(">lock / beep</text>"));
    }

    #[test]
    fn the_text_alternative_lists_every_transition_with_its_parts() {
        let alternative =
            text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
        assert!(alternative.description.starts_with(
            "State diagram with 4 states and 6 transitions. It starts in Closed. Final state: Removed. Main path: Closed → Locked → Removed. Closed to Open on push [unlocked]."
        ));
        assert!(alternative.description.contains("Open to itself on wind."));
        assert_eq!(
            alternative.table.columns,
            ["From", "Event", "Guard", "Action", "To"]
        );
        assert_eq!(alternative.table.rows[0], ["Start", "", "", "", "Closed"]);
        assert_eq!(alternative.table.rows.len(), 7);
        assert!(alternative.table.rows.contains(&vec![
            "Closed".to_owned(),
            "lock".to_owned(),
            String::new(),
            "beep".to_owned(),
            "Locked".to_owned(),
        ]));
    }

    #[test]
    fn german_describes_states_and_names_its_columns() {
        let json = SPEC.replace(
            "\"schemaVersion\": 1,",
            "\"schemaVersion\": 1, \"locale\": \"de\",",
        );
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(alternative.description.starts_with(
            "Zustandsdiagramm mit 4 Zuständen und 6 Übergängen. Es beginnt in Closed. Endzustand: Removed."
        ));
        assert!(
            alternative
                .description
                .contains("Open zu sich selbst bei wind.")
        );
        assert_eq!(
            alternative.table.columns,
            ["Von", "Ereignis", "Bedingung", "Aktion", "Nach"]
        );
    }

    #[test]
    fn a_state_nothing_leads_to_is_reported() {
        let json = SPEC
            .replace(
                r#"{"from": "locked", "to": "gone", "event": "dismantle"}"#,
                r#"{"from": "gone", "to": "locked", "event": "rebuild"}"#,
            )
            .replace(r#"["closed", "locked", "gone"]"#, r#"["closed", "locked"]"#);
        let warnings = svg(&json).warnings;
        assert_eq!(warnings[0].code, "unreachable_state");
        assert_eq!(warnings[0].path, "/state/states/3");
    }

    #[test]
    fn invalid_state_diagrams_name_the_field() {
        for (from, to, code, path) in [
            (
                r#""to": "open", "event": "push""#,
                r#""to": "ajar", "event": "push""#,
                "unknown_state",
                "/state/transitions/0/to",
            ),
            (
                r#""initial": "closed""#,
                r#""initial": "shut""#,
                "unknown_state",
                "/state/initial",
            ),
            (
                r#"{"id": "open""#,
                r#"{"id": "closed""#,
                "duplicate_id",
                "/state/states/1/id",
            ),
            (
                r#"["closed", "locked", "gone"]"#,
                r#"["closed", "gone"]"#,
                "main_path_gap",
                "/state/mainPath/1",
            ),
            (
                r#""label": "Removed", "final": true"#,
                r#""label": "Removed", "final": true, "kind": "choice""#,
                "option_not_supported",
                "/state/states/3/final",
            ),
            (
                r#""type": "state""#,
                r#""type": "flow""#,
                "option_not_supported",
                "/state",
            ),
        ] {
            assert!(SPEC.contains(from), "{from}");
            assert_eq!(
                error(&SPEC.replace(from, to)),
                (code, path.to_owned()),
                "{to}"
            );
        }
        let missing = r#"{"schemaVersion": 1, "type": "state", "title": "Empty"}"#;
        assert_eq!(error(missing), ("missing_state", "/state".to_owned()));
    }
}
