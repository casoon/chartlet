//! Inspections of an architecture model for gaps in its documentation: connections without a
//! label, components without a second line, components no view shows. They are asked for on
//! their own, apart from rendering; a specification that renders keeps rendering, and nothing
//! here becomes a warning of a render.
//!
//! Every finding is limited to what the model states. An isolated component or a view that adds
//! nothing is a hint to review, never a judgement of the architecture.

use crate::{
    ChartError, ChartSpec, ChartType, ComponentKind,
    spec::{ArchitectureSpec, ViewSpec},
    structure,
};

/// The codes of every inspection, in the order findings are reported.
pub const INSPECTIONS: [&str; 6] = [
    UNLABELLED_CONNECTION,
    CONNECTION_WITHOUT_TECHNOLOGY,
    COMPONENT_WITHOUT_SUBLABEL,
    ISOLATED_COMPONENT,
    COMPONENT_IN_NO_VIEW,
    VIEW_ADDS_NOTHING,
];

const UNLABELLED_CONNECTION: &str = "unlabelled_connection";
const CONNECTION_WITHOUT_TECHNOLOGY: &str = "connection_without_technology";
const COMPONENT_WITHOUT_SUBLABEL: &str = "component_without_sublabel";
const ISOLATED_COMPONENT: &str = "isolated_component";
const COMPONENT_IN_NO_VIEW: &str = "component_in_no_view";
const VIEW_ADDS_NOTHING: &str = "view_adds_nothing";

/// How much a finding weighs: a `Gap` is missing documentation the specification has a field
/// for; a `Hint` is something to look at, which may well be intended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Gap,
    Hint,
}

impl Severity {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Gap => "gap",
            Self::Hint => "hint",
        }
    }
}

/// One finding: a stable code, its weight, the JSON Pointer of the part of the model it is
/// about, what was found and how to improve it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    pub code: &'static str,
    pub severity: Severity,
    pub path: String,
    pub message: String,
    pub hint: String,
}

/// Inspects the architecture model `input`, leaving out the inspections whose codes are in
/// `disabled`. Findings follow the order of [`INSPECTIONS`], and within one inspection the order
/// of the specification.
///
/// # Errors
///
/// Returns the structured error of an invalid specification, as rendering would, and
/// `inspections_unavailable` for a specification that is not an architecture diagram.
pub fn inspect_json(input: &str, disabled: &[&str]) -> Result<Vec<Inspection>, ChartError> {
    let spec = ChartSpec::from_json(input)?;
    spec.validate()?;
    let (ChartType::Architecture, Some(model)) = (spec.chart_type, &spec.architecture) else {
        return Err(ChartError::new(
            "inspections_unavailable",
            "/type",
            "inspections are available for architecture diagrams",
        ));
    };
    let mut findings = Vec::new();
    for code in INSPECTIONS {
        if disabled.contains(&code) {
            continue;
        }
        match code {
            UNLABELLED_CONNECTION => unlabelled_connections(model, &mut findings),
            CONNECTION_WITHOUT_TECHNOLOGY => connections_without_technology(model, &mut findings),
            COMPONENT_WITHOUT_SUBLABEL => components_without_sublabel(model, &mut findings),
            ISOLATED_COMPONENT => isolated_components(model, &mut findings),
            COMPONENT_IN_NO_VIEW => components_in_no_view(model, &mut findings),
            _ => views_adding_nothing(model, &mut findings),
        }
    }
    Ok(findings)
}

fn component_label<'a>(model: &'a ArchitectureSpec, id: &'a str) -> &'a str {
    model
        .component(id)
        .map_or(id, |index| model.components[index].label.as_str())
}

fn connection_name(model: &ArchitectureSpec, index: usize) -> String {
    let connection = &model.connections[index];
    format!(
        "the connection from \"{}\" to \"{}\"",
        component_label(model, &connection.from),
        component_label(model, &connection.to)
    )
}

/// A connection without a label says that two components talk, not what about. A dynamic view
/// that labels every one of its steps on the connection does not make up for it: the model and
/// its other views still draw it without one.
fn unlabelled_connections(model: &ArchitectureSpec, findings: &mut Vec<Inspection>) {
    for (index, connection) in model.connections.iter().enumerate() {
        if connection.label.is_none() {
            findings.push(Inspection {
                code: UNLABELLED_CONNECTION,
                severity: Severity::Gap,
                path: format!("/architecture/connections/{index}"),
                message: format!("{} has no label", connection_name(model, index)),
                hint: "add a label that says what it does, such as \"reads orders\"".to_owned(),
            });
        }
    }
}

/// A connection without a technology leaves open how it is made. Only a hint: many models
/// leave the protocol to their components.
fn connections_without_technology(model: &ArchitectureSpec, findings: &mut Vec<Inspection>) {
    for (index, connection) in model.connections.iter().enumerate() {
        if connection.technology.is_none() {
            findings.push(Inspection {
                code: CONNECTION_WITHOUT_TECHNOLOGY,
                severity: Severity::Hint,
                path: format!("/architecture/connections/{index}"),
                message: format!("{} names no technology", connection_name(model, index)),
                hint: "add a technology, such as \"HTTPS\" or \"SQL/TLS\"".to_owned(),
            });
        }
    }
}

/// A component without a second line names no technology or host. A person has neither, so it
/// is not asked for one.
fn components_without_sublabel(model: &ArchitectureSpec, findings: &mut Vec<Inspection>) {
    for (index, component) in model.components.iter().enumerate() {
        if component.sublabel.is_none() && component.kind != ComponentKind::Person {
            findings.push(Inspection {
                code: COMPONENT_WITHOUT_SUBLABEL,
                severity: Severity::Gap,
                path: format!("/architecture/components/{index}"),
                message: format!("\"{}\" has no sublabel", component.label),
                hint: "add a sublabel naming its technology or host".to_owned(),
            });
        }
    }
}

/// A component without any connection, in a model with others it could be connected to.
fn isolated_components(model: &ArchitectureSpec, findings: &mut Vec<Inspection>) {
    let nodes = model.components.len();
    if nodes < 2 {
        return;
    }
    let edges: Vec<(usize, usize)> = model
        .connections
        .iter()
        .filter_map(|connection| {
            Some((
                model.component(&connection.from)?,
                model.component(&connection.to)?,
            ))
        })
        .collect();
    for (index, neighbours) in structure::neighbours(nodes, &edges).iter().enumerate() {
        if neighbours.is_empty() {
            findings.push(Inspection {
                code: ISOLATED_COMPONENT,
                severity: Severity::Hint,
                path: format!("/architecture/components/{index}"),
                message: format!(
                    "\"{}\" has no connection to any other component",
                    model.components[index].label
                ),
                hint: "connect it, or keep it if it stands apart on purpose".to_owned(),
            });
        }
    }
}

/// A component that no view shows is never drawn, since a model with views renders only them.
/// A model without views shows every component and has none to report.
fn components_in_no_view(model: &ArchitectureSpec, findings: &mut Vec<Inspection>) {
    let selections: Vec<Vec<usize>> = model
        .views
        .iter()
        .map(|view| model.selection(view).0)
        .collect();
    if selections.is_empty() {
        return;
    }
    for (index, component) in model.components.iter().enumerate() {
        if !selections.iter().any(|shown| shown.contains(&index)) {
            findings.push(Inspection {
                code: COMPONENT_IN_NO_VIEW,
                severity: Severity::Gap,
                path: format!("/architecture/components/{index}"),
                message: format!("no view shows \"{}\"", component.label),
                hint: "add it to a view, or remove it from the model".to_owned(),
            });
        }
    }
}

/// A view whose components and connections another view all shows as well. A dynamic view is
/// never reported: the order of its steps is what it adds.
fn views_adding_nothing(model: &ArchitectureSpec, findings: &mut Vec<Inspection>) {
    let is_dynamic = |view: &ViewSpec| !view.steps.is_empty();
    let selections: Vec<(Vec<usize>, Vec<usize>)> = model
        .views
        .iter()
        .map(|view| {
            let (components, connections, _) = model.selection(view);
            (components, connections)
        })
        .collect();
    for (index, view) in model.views.iter().enumerate() {
        if is_dynamic(view) {
            continue;
        }
        let (components, connections) = &selections[index];
        // Of two views that show the same, only the later one is reported.
        let covering = model.views.iter().enumerate().find(|&(other, other_view)| {
            let (other_components, other_connections) = &selections[other];
            let covers = other != index
                && !is_dynamic(other_view)
                && components.iter().all(|c| other_components.contains(c))
                && connections.iter().all(|c| other_connections.contains(c));
            let same = other_components.len() == components.len()
                && other_connections.len() == connections.len();
            covers && (!same || other < index)
        });
        if let Some((_, other)) = covering {
            findings.push(Inspection {
                code: VIEW_ADDS_NOTHING,
                severity: Severity::Hint,
                path: format!("/architecture/views/{index}"),
                message: format!(
                    "the view \"{}\" shows nothing that the view \"{}\" does not",
                    view.key, other.key
                ),
                hint: "keep it if it tells the same from another angle, or remove it".to_owned(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{INSPECTIONS, Inspection, Severity, inspect_json};

    fn model(architecture: &Value) -> String {
        json!({ "schemaVersion": 1, "type": "architecture", "title": "Shop", "architecture": architecture })
            .to_string()
    }

    /// The findings of `code` alone, as their paths.
    fn found(architecture: &Value, code: &str) -> Vec<String> {
        let disabled: Vec<&str> = INSPECTIONS.into_iter().filter(|&c| c != code).collect();
        inspect_json(&model(architecture), &disabled)
            .expect("a valid model")
            .into_iter()
            .map(|finding: Inspection| {
                assert_eq!(finding.code, code);
                finding.path
            })
            .collect()
    }

    fn documented() -> Value {
        json!({
            "components": [
                { "id": "user", "label": "Customer", "kind": "person" },
                { "id": "web", "label": "Web shop", "sublabel": "Astro" },
                { "id": "db", "label": "Orders", "sublabel": "PostgreSQL", "kind": "database" }
            ],
            "connections": [
                { "id": "browse", "from": "user", "to": "web", "label": "browses", "technology": "HTTPS" },
                { "id": "store", "from": "web", "to": "db", "label": "stores orders", "technology": "SQL" }
            ]
        })
    }

    #[test]
    fn a_documented_model_has_no_findings() {
        assert_eq!(inspect_json(&model(&documented()), &[]), Ok(Vec::new()));
    }

    #[test]
    fn a_connection_without_a_label_is_a_gap() {
        let mut architecture = documented();
        architecture["connections"][1]
            .as_object_mut()
            .unwrap()
            .remove("label");
        assert_eq!(
            found(&architecture, "unlabelled_connection"),
            ["/architecture/connections/1"]
        );
        let findings = inspect_json(&model(&architecture), &[]).unwrap();
        assert_eq!(findings[0].severity, Severity::Gap);
        assert_eq!(
            findings[0].message,
            "the connection from \"Web shop\" to \"Orders\" has no label"
        );
        assert!(found(&documented(), "unlabelled_connection").is_empty());
    }

    #[test]
    fn a_disabled_inspection_reports_nothing() {
        let mut architecture = documented();
        architecture["connections"][1]
            .as_object_mut()
            .unwrap()
            .remove("label");
        let findings = inspect_json(&model(&architecture), &["unlabelled_connection"]).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn a_connection_without_a_technology_is_a_hint() {
        let mut architecture = documented();
        architecture["connections"][0]
            .as_object_mut()
            .unwrap()
            .remove("technology");
        assert_eq!(
            found(&architecture, "connection_without_technology"),
            ["/architecture/connections/0"]
        );
        let findings = inspect_json(&model(&architecture), &[]).unwrap();
        assert_eq!(findings[0].severity, Severity::Hint);
        assert!(found(&documented(), "connection_without_technology").is_empty());
    }

    #[test]
    fn a_component_without_a_sublabel_is_a_gap_except_a_person() {
        let mut architecture = documented();
        architecture["components"][2]
            .as_object_mut()
            .unwrap()
            .remove("sublabel");
        assert_eq!(
            found(&architecture, "component_without_sublabel"),
            ["/architecture/components/2"]
        );
        // The person of `documented` has no sublabel either, and is not reported.
        assert!(found(&documented(), "component_without_sublabel").is_empty());
    }

    #[test]
    fn an_isolated_component_is_a_hint_in_a_model_with_others() {
        let mut architecture = documented();
        architecture["components"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "id": "audit", "label": "Audit log", "sublabel": "S3" }));
        assert_eq!(
            found(&architecture, "isolated_component"),
            ["/architecture/components/3"]
        );
        assert!(found(&documented(), "isolated_component").is_empty());
        let alone = json!({ "components": [{ "id": "web", "label": "Web shop" }] });
        assert!(found(&alone, "isolated_component").is_empty());
    }

    fn with_views(views: &Value) -> Value {
        let mut architecture = documented();
        architecture["views"] = views.clone();
        architecture
    }

    #[test]
    fn a_component_in_no_view_is_a_gap_only_in_a_model_with_views() {
        let architecture = with_views(&json!([
            { "key": "front", "title": "Front", "components": ["user", "web"] }
        ]));
        assert_eq!(
            found(&architecture, "component_in_no_view"),
            ["/architecture/components/2"]
        );
        let all = with_views(&json!([
            { "key": "front", "title": "Front", "components": ["user", "web"] },
            { "key": "back", "title": "Back", "components": ["web", "db"] }
        ]));
        assert!(found(&all, "component_in_no_view").is_empty());
        assert!(found(&documented(), "component_in_no_view").is_empty());
    }

    #[test]
    fn a_dynamic_view_shows_the_ends_of_its_steps() {
        let architecture = with_views(&json!([
            { "key": "front", "title": "Front", "components": ["user", "web"] },
            { "key": "order", "title": "Order", "steps": [{ "connection": "store" }] }
        ]));
        assert!(found(&architecture, "component_in_no_view").is_empty());
    }

    #[test]
    fn a_view_covered_by_another_is_a_hint_except_a_dynamic_one() {
        let architecture = with_views(&json!([
            { "key": "all", "title": "All", "components": ["user", "web", "db"] },
            { "key": "front", "title": "Front", "components": ["user", "web"] },
            { "key": "order", "title": "Order", "steps": [{ "connection": "browse" }] }
        ]));
        assert_eq!(
            found(&architecture, "view_adds_nothing"),
            ["/architecture/views/1"]
        );
        let apart = with_views(&json!([
            { "key": "front", "title": "Front", "components": ["user", "web"] },
            { "key": "back", "title": "Back", "components": ["web", "db"] }
        ]));
        assert!(found(&apart, "view_adds_nothing").is_empty());
    }

    #[test]
    fn of_two_equal_views_the_later_is_reported() {
        let architecture = with_views(&json!([
            { "key": "front", "title": "Front", "components": ["user", "web"] },
            { "key": "again", "title": "Again", "components": ["web", "user"] }
        ]));
        assert_eq!(
            found(&architecture, "view_adds_nothing"),
            ["/architecture/views/1"]
        );
    }

    #[test]
    fn specification_errors_stay_errors() {
        let mut architecture = documented();
        architecture["connections"][0]["to"] = json!("nowhere");
        let error = inspect_json(&model(&architecture), &[]).unwrap_err();
        assert_ne!(error.code, "inspections_unavailable");
    }

    #[test]
    fn other_chart_types_have_no_inspections() {
        let bar = json!({
            "schemaVersion": 1, "type": "bar", "title": "Sales", "categories": ["A"],
            "series": [{ "name": "2026", "values": [1] }]
        });
        let error = inspect_json(&bar.to_string(), &[]).unwrap_err();
        assert_eq!(error.code, "inspections_unavailable");
    }
}
