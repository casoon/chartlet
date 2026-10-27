//! Inspections of a model for gaps in its documentation: in an architecture model connections
//! without a label, components without a second line, components no view shows; in a flow chart
//! unlabelled branches of a decision and steps no start leads to; in a class diagram classes
//! without members. They are asked for on their own, apart from rendering; a specification that
//! renders keeps rendering, and nothing here becomes a warning of a render.
//!
//! Every finding is limited to what the model states. An isolated component or a view that adds
//! nothing is a hint to review, never a judgement of the architecture.

use crate::{
    ChartError, ChartSpec, ChartType, ComponentKind, NodeKind,
    readability::{self, DIAGRAM_OVER_BUDGET},
    spec::{ArchitectureSpec, ClassesSpec, FlowSpec, ViewSpec},
    structure,
};

/// The codes of every inspection, in the order findings are reported: those of architecture
/// models, of flow charts, then of class diagrams.
pub const INSPECTIONS: [&str; 12] = [
    UNLABELLED_CONNECTION,
    CONNECTION_WITHOUT_TECHNOLOGY,
    COMPONENT_WITHOUT_SUBLABEL,
    ISOLATED_COMPONENT,
    COMPONENT_IN_NO_VIEW,
    VIEW_ADDS_NOTHING,
    UNLABELLED_DECISION_BRANCH,
    UNREACHABLE_STEP,
    FLOW_WITHOUT_END,
    CLASS_WITHOUT_MEMBERS,
    ISOLATED_CLASS,
    DIAGRAM_OVER_BUDGET,
];

const UNLABELLED_CONNECTION: &str = "unlabelled_connection";
const CONNECTION_WITHOUT_TECHNOLOGY: &str = "connection_without_technology";
const COMPONENT_WITHOUT_SUBLABEL: &str = "component_without_sublabel";
const ISOLATED_COMPONENT: &str = "isolated_component";
const COMPONENT_IN_NO_VIEW: &str = "component_in_no_view";
const VIEW_ADDS_NOTHING: &str = "view_adds_nothing";
const UNLABELLED_DECISION_BRANCH: &str = "unlabelled_decision_branch";
const UNREACHABLE_STEP: &str = "unreachable_step";
const FLOW_WITHOUT_END: &str = "flow_without_end";
const CLASS_WITHOUT_MEMBERS: &str = "class_without_members";
const ISOLATED_CLASS: &str = "isolated_class";

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

/// What is inspected: an architecture model, a flow chart or a class diagram.
#[derive(Clone, Copy)]
enum Model<'a> {
    Architecture(&'a ArchitectureSpec),
    Flow(&'a FlowSpec),
    Classes(&'a ClassesSpec),
}

/// Inspects the architecture model, flow chart or class diagram `input`, leaving out the
/// inspections whose codes are in `disabled`. Findings follow the order of [`INSPECTIONS`], and
/// within one inspection the order of the specification.
///
/// # Errors
///
/// Returns the structured error of an invalid specification, as rendering would, and
/// `inspections_unavailable` for a specification of another type.
pub fn inspect_json(input: &str, disabled: &[&str]) -> Result<Vec<Inspection>, ChartError> {
    let spec = ChartSpec::from_json(input)?;
    spec.validate()?;
    let model = match (
        spec.chart_type,
        &spec.architecture,
        &spec.flow,
        &spec.classes,
    ) {
        (ChartType::Architecture, Some(model), _, _) => Model::Architecture(model),
        (ChartType::Flow, _, Some(flow), _) => Model::Flow(flow),
        (ChartType::Classes, _, _, Some(diagram)) => Model::Classes(diagram),
        _ => {
            return Err(ChartError::new(
                "inspections_unavailable",
                "/type",
                "inspections are available for architecture, flow and class diagrams",
            ));
        }
    };
    let mut findings = Vec::new();
    for code in INSPECTIONS {
        if disabled.contains(&code) {
            continue;
        }
        let found = &mut findings;
        match (code, model) {
            (UNLABELLED_CONNECTION, Model::Architecture(model)) => {
                unlabelled_connections(model, found);
            }
            (CONNECTION_WITHOUT_TECHNOLOGY, Model::Architecture(model)) => {
                connections_without_technology(model, found);
            }
            (COMPONENT_WITHOUT_SUBLABEL, Model::Architecture(model)) => {
                components_without_sublabel(model, found);
            }
            (ISOLATED_COMPONENT, Model::Architecture(model)) => isolated_components(model, found),
            (COMPONENT_IN_NO_VIEW, Model::Architecture(model)) => {
                components_in_no_view(model, found);
            }
            (VIEW_ADDS_NOTHING, Model::Architecture(model)) => views_adding_nothing(model, found),
            (UNLABELLED_DECISION_BRANCH, Model::Flow(flow)) => {
                unlabelled_decision_branches(flow, found);
            }
            (UNREACHABLE_STEP, Model::Flow(flow)) => unreachable_steps(flow, found),
            (FLOW_WITHOUT_END, Model::Flow(flow)) => flow_without_end(flow, found),
            (CLASS_WITHOUT_MEMBERS, Model::Classes(diagram)) => {
                classes_without_members(diagram, found);
            }
            (ISOLATED_CLASS, Model::Classes(diagram)) => isolated_classes(diagram, found),
            (DIAGRAM_OVER_BUDGET, _) => {
                if let Some(finding) = readability::finding(&spec) {
                    found.push(Inspection {
                        code: DIAGRAM_OVER_BUDGET,
                        severity: Severity::Hint,
                        path: finding.path.to_owned(),
                        message: finding.message,
                        hint: finding.hint,
                    });
                }
            }
            // An inspection of another type of model.
            _ => {}
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
/// never reported: the order of its steps is what it adds; nor is a deployment view, which adds
/// the instances of an environment.
fn views_adding_nothing(model: &ArchitectureSpec, findings: &mut Vec<Inspection>) {
    let is_dynamic = |view: &ViewSpec| !view.steps.is_empty() || view.environment.is_some();
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

fn step_label<'a>(flow: &'a FlowSpec, id: &'a str) -> &'a str {
    flow.nodes
        .iter()
        .find(|node| node.id == id)
        .map_or(id, |node| node.label.as_str())
}

/// An edge out of a decision is one of its answers; without a label the reader cannot tell which.
/// A decision with a single way on has no branches to tell apart, and is not reported.
fn unlabelled_decision_branches(flow: &FlowSpec, findings: &mut Vec<Inspection>) {
    for decision in flow
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Decision)
    {
        let branches: Vec<usize> = (0..flow.edges.len())
            .filter(|&index| flow.edges[index].from == decision.id)
            .collect();
        if branches.len() < 2 {
            continue;
        }
        for index in branches {
            let edge = &flow.edges[index];
            if edge.label.is_none() {
                findings.push(Inspection {
                    code: UNLABELLED_DECISION_BRANCH,
                    severity: Severity::Gap,
                    path: format!("/flow/edges/{index}"),
                    message: format!(
                        "the branch from \"{}\" to \"{}\" has no label",
                        decision.label,
                        step_label(flow, &edge.to)
                    ),
                    hint: "label it with the answer it stands for, such as \"yes\" or \"no\""
                        .to_owned(),
                });
            }
        }
    }
}

/// A step that no edge leads to from a start step. Only a flow with a start step says where it
/// begins; data stores and external steps feed into a flow from outside, and are not reported.
fn unreachable_steps(flow: &FlowSpec, findings: &mut Vec<Inspection>) {
    let mut reached: Vec<bool> = flow
        .nodes
        .iter()
        .map(|node| node.kind == NodeKind::Start)
        .collect();
    let mut queue: Vec<usize> = (0..reached.len()).filter(|&index| reached[index]).collect();
    if queue.is_empty() {
        return;
    }
    while let Some(node) = queue.pop() {
        for edge in flow
            .edges
            .iter()
            .filter(|edge| edge.from == flow.nodes[node].id)
        {
            if let Some(next) = flow.nodes.iter().position(|node| node.id == edge.to)
                && !reached[next]
            {
                reached[next] = true;
                queue.push(next);
            }
        }
    }
    for (index, node) in flow.nodes.iter().enumerate() {
        if !reached[index] && !matches!(node.kind, NodeKind::Store | NodeKind::External) {
            findings.push(Inspection {
                code: UNREACHABLE_STEP,
                severity: Severity::Hint,
                path: format!("/flow/nodes/{index}"),
                message: format!("no start step leads to \"{}\"", node.label),
                hint: "add the edge that leads to it, or keep it if it is a way in of its own"
                    .to_owned(),
            });
        }
    }
}

/// A flow that marks where it begins but not where it ends. A flow with neither start nor end
/// step is a part of a larger one, and is not reported.
fn flow_without_end(flow: &FlowSpec, findings: &mut Vec<Inspection>) {
    let has = |kind: NodeKind| flow.nodes.iter().any(|node| node.kind == kind);
    if has(NodeKind::Start) && !has(NodeKind::End) {
        findings.push(Inspection {
            code: FLOW_WITHOUT_END,
            severity: Severity::Hint,
            path: "/flow/nodes".to_owned(),
            message: "the flow has a start step but no end step".to_owned(),
            hint: "add a step of kind \"end\" where the flow finishes".to_owned(),
        });
    }
}

/// A class without attributes or methods shows only its name. A diagram in which no class has
/// members is an overview of names on purpose, and is not reported.
fn classes_without_members(diagram: &ClassesSpec, findings: &mut Vec<Inspection>) {
    let empty: Vec<bool> = diagram
        .classes
        .iter()
        .map(|class| class.attributes.is_empty() && class.methods.is_empty())
        .collect();
    if empty.iter().all(|&empty| empty) {
        return;
    }
    for (index, class) in diagram.classes.iter().enumerate() {
        if empty[index] {
            findings.push(Inspection {
                code: CLASS_WITHOUT_MEMBERS,
                severity: Severity::Gap,
                path: format!("/classes/classes/{index}"),
                message: format!("\"{}\" has no attributes or methods", class.label),
                hint: "add the attributes or methods that matter here".to_owned(),
            });
        }
    }
}

/// A class without a relation to another class, in a diagram with others it could be related
/// to. Notes are no relations, nor is a relation of a class to itself.
fn isolated_classes(diagram: &ClassesSpec, findings: &mut Vec<Inspection>) {
    if diagram.classes.len() < 2 {
        return;
    }
    for (index, class) in diagram.classes.iter().enumerate() {
        let related = diagram.relations.iter().any(|relation| {
            relation.from != relation.to && (relation.from == class.id || relation.to == class.id)
        });
        if !related {
            findings.push(Inspection {
                code: ISOLATED_CLASS,
                severity: Severity::Hint,
                path: format!("/classes/classes/{index}"),
                message: format!("\"{}\" has no relation to any other class", class.label),
                hint: "relate it, or keep it if it stands apart on purpose".to_owned(),
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
        assert!(
            found(&documented(), "unlabelled_connection").is_empty(),
            "{:?}",
            found(&documented(), "unlabelled_connection")
        );
    }

    #[test]
    fn a_disabled_inspection_reports_nothing() {
        let mut architecture = documented();
        architecture["connections"][1]
            .as_object_mut()
            .unwrap()
            .remove("label");
        let findings = inspect_json(&model(&architecture), &["unlabelled_connection"]).unwrap();
        assert!(findings.is_empty(), "{findings:?}");
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
        assert!(
            found(&documented(), "connection_without_technology").is_empty(),
            "{:?}",
            found(&documented(), "connection_without_technology")
        );
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
        assert!(
            found(&documented(), "component_without_sublabel").is_empty(),
            "{:?}",
            found(&documented(), "component_without_sublabel")
        );
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
        assert!(
            found(&documented(), "isolated_component").is_empty(),
            "{:?}",
            found(&documented(), "isolated_component")
        );
        let alone = json!({ "components": [{ "id": "web", "label": "Web shop" }] });
        assert!(
            found(&alone, "isolated_component").is_empty(),
            "{:?}",
            found(&alone, "isolated_component")
        );
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
        assert!(
            found(&all, "component_in_no_view").is_empty(),
            "{:?}",
            found(&all, "component_in_no_view")
        );
        assert!(
            found(&documented(), "component_in_no_view").is_empty(),
            "{:?}",
            found(&documented(), "component_in_no_view")
        );
    }

    #[test]
    fn a_dynamic_view_shows_the_ends_of_its_steps() {
        let architecture = with_views(&json!([
            { "key": "front", "title": "Front", "components": ["user", "web"] },
            { "key": "order", "title": "Order", "steps": [{ "connection": "store" }] }
        ]));
        assert!(
            found(&architecture, "component_in_no_view").is_empty(),
            "{:?}",
            found(&architecture, "component_in_no_view")
        );
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
        assert!(
            found(&apart, "view_adds_nothing").is_empty(),
            "{:?}",
            found(&apart, "view_adds_nothing")
        );
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

    /// The findings of `code` alone in a specification of `chart_type` with `body` under the
    /// field of that name, as their paths.
    fn found_in(chart_type: &str, body: &Value, code: &str) -> Vec<String> {
        let spec =
            json!({ "schemaVersion": 1, "type": chart_type, "title": "Order", chart_type: body });
        let disabled: Vec<&str> = INSPECTIONS.into_iter().filter(|&c| c != code).collect();
        inspect_json(&spec.to_string(), &disabled)
            .expect("a valid specification")
            .into_iter()
            .map(|finding| {
                assert_eq!(finding.code, code);
                finding.path
            })
            .collect()
    }

    fn order_flow() -> Value {
        json!({
            "nodes": [
                { "id": "start", "label": "Order received", "kind": "start" },
                { "id": "paid", "label": "Paid?", "kind": "decision" },
                { "id": "ship", "label": "Ship" },
                { "id": "remind", "label": "Send reminder" },
                { "id": "done", "label": "Done", "kind": "end" },
                { "id": "stock", "label": "Stock", "kind": "store" }
            ],
            "edges": [
                { "from": "start", "to": "paid" },
                { "from": "paid", "to": "ship", "label": "yes" },
                { "from": "paid", "to": "remind", "label": "no" },
                { "from": "stock", "to": "ship" },
                { "from": "ship", "to": "done" },
                { "from": "remind", "to": "done" }
            ]
        })
    }

    #[test]
    fn a_documented_flow_has_no_findings() {
        let spec =
            json!({ "schemaVersion": 1, "type": "flow", "title": "Order", "flow": order_flow() });
        assert_eq!(inspect_json(&spec.to_string(), &[]), Ok(Vec::new()));
    }

    #[test]
    fn an_unlabelled_branch_of_a_decision_is_a_gap_unless_it_is_the_only_way_on() {
        let mut flow = order_flow();
        flow["edges"][2].as_object_mut().unwrap().remove("label");
        assert_eq!(
            found_in("flow", &flow, "unlabelled_decision_branch"),
            ["/flow/edges/2"]
        );
        // A decision with one way on has nothing to tell apart.
        flow["edges"].as_array_mut().unwrap().remove(1);
        assert!(
            found_in("flow", &flow, "unlabelled_decision_branch").is_empty(),
            "{:?}",
            found_in("flow", &flow, "unlabelled_decision_branch")
        );
        assert!(
            found_in("flow", &order_flow(), "unlabelled_decision_branch").is_empty(),
            "{:?}",
            found_in("flow", &order_flow(), "unlabelled_decision_branch")
        );
    }

    #[test]
    fn a_step_no_start_leads_to_is_a_hint_except_a_store_or_without_a_start() {
        let mut flow = order_flow();
        flow["edges"][0]["to"] = json!("ship");
        // Neither the decision nor the reminder can be reached now; the store never could.
        assert_eq!(
            found_in("flow", &flow, "unreachable_step"),
            ["/flow/nodes/1", "/flow/nodes/3"]
        );
        assert!(
            found_in("flow", &order_flow(), "unreachable_step").is_empty(),
            "{:?}",
            found_in("flow", &order_flow(), "unreachable_step")
        );
        flow["nodes"][0]["kind"] = json!("process");
        assert!(
            found_in("flow", &flow, "unreachable_step").is_empty(),
            "{:?}",
            found_in("flow", &flow, "unreachable_step")
        );
    }

    #[test]
    fn a_flow_with_a_start_but_no_end_is_a_hint() {
        let mut flow = order_flow();
        flow["nodes"][4]["kind"] = json!("process");
        assert_eq!(found_in("flow", &flow, "flow_without_end"), ["/flow/nodes"]);
        assert!(
            found_in("flow", &order_flow(), "flow_without_end").is_empty(),
            "{:?}",
            found_in("flow", &order_flow(), "flow_without_end")
        );
        // Without a start either, the flow is a part of a larger one.
        flow["nodes"][0]["kind"] = json!("process");
        assert!(
            found_in("flow", &flow, "flow_without_end").is_empty(),
            "{:?}",
            found_in("flow", &flow, "flow_without_end")
        );
    }

    fn shop_classes() -> Value {
        json!({
            "classes": [
                { "id": "order", "label": "Order", "attributes": ["id: Int"] },
                { "id": "line", "label": "Line", "attributes": ["quantity: Int"] },
                { "id": "product", "label": "Product", "methods": ["price(): Money"] }
            ],
            "relations": [
                { "from": "order", "to": "line", "kind": "composition" },
                { "from": "line", "to": "product" }
            ]
        })
    }

    #[test]
    fn a_documented_class_diagram_has_no_findings() {
        let spec = json!({ "schemaVersion": 1, "type": "classes", "title": "Shop", "classes": shop_classes() });
        assert_eq!(inspect_json(&spec.to_string(), &[]), Ok(Vec::new()));
    }

    #[test]
    fn a_class_without_members_is_a_gap_unless_no_class_has_any() {
        let mut classes = shop_classes();
        classes["classes"][2]
            .as_object_mut()
            .unwrap()
            .remove("methods");
        assert_eq!(
            found_in("classes", &classes, "class_without_members"),
            ["/classes/classes/2"]
        );
        assert!(
            found_in("classes", &shop_classes(), "class_without_members").is_empty(),
            "{:?}",
            found_in("classes", &shop_classes(), "class_without_members")
        );
        for class in classes["classes"].as_array_mut().unwrap() {
            class.as_object_mut().unwrap().remove("attributes");
        }
        assert!(
            found_in("classes", &classes, "class_without_members").is_empty(),
            "{:?}",
            found_in("classes", &classes, "class_without_members")
        );
    }

    #[test]
    fn an_isolated_class_is_a_hint_in_a_diagram_with_others() {
        let mut classes = shop_classes();
        classes["relations"][1]["to"] = json!("line");
        // A relation of a class to itself relates it to no other.
        assert_eq!(
            found_in("classes", &classes, "isolated_class"),
            ["/classes/classes/2"]
        );
        assert!(
            found_in("classes", &shop_classes(), "isolated_class").is_empty(),
            "{:?}",
            found_in("classes", &shop_classes(), "isolated_class")
        );
        let alone = json!({ "classes": [{ "id": "order", "label": "Order" }] });
        assert!(
            found_in("classes", &alone, "isolated_class").is_empty(),
            "{:?}",
            found_in("classes", &alone, "isolated_class")
        );
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
