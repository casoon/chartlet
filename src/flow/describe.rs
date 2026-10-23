//! A flow chart in words: its description and its data table.

use std::fmt::Write as _;

use super::{flow, graph::Graph, model::Diagram, notes};
use crate::{
    DataTable, heat,
    layout::format_value,
    parts, roles,
    spec::{ChartSpec, DiagramRole, FlowSpec, NodeKind, PartSpec},
    structure::{Facts, Structure},
    text,
};

/// The nodes of `diagram` in reading order: layer by layer, and across each layer in its order.
pub(crate) fn reading_order(diagram: &Diagram) -> Vec<usize> {
    let graph = Graph::new(diagram);
    graph
        .layers
        .iter()
        .flatten()
        .filter_map(|item| graph.items[*item].node)
        .collect()
}

/// Where each step leads, as `label` or `label (edge label, weight)`, in the order of the edges.
fn next_steps(
    spec: &ChartSpec,
    ends: &[(usize, usize)],
    node: usize,
    with_roles: bool,
) -> Vec<String> {
    let flow = flow(spec);
    flow.edges
        .iter()
        .zip(ends)
        .filter(|(_, (from, _))| *from == node)
        .map(|(edge, (_, to))| {
            let notes: Vec<String> = edge
                .label
                .iter()
                .cloned()
                .chain(edge.weight.map(|weight| {
                    text::edge_weight(spec.locale, &format_value(weight, spec.number_style()))
                }))
                .chain(
                    edge.role
                        .filter(|_| with_roles)
                        .map(|role| text::role(spec.locale, role).to_owned()),
                )
                .collect();
            let target = &flow.nodes[*to].label;
            if notes.is_empty() {
                target.clone()
            } else {
                format!("{target} ({})", notes.join(", "))
            }
        })
        .collect()
}

/// The edge with the largest weight, as a sentence, when at least two edges have one; the first
/// at a tie. `ends` writes every edge with the labels at both ends, as `A → B`.
pub(crate) fn heaviest_edge(
    spec: &ChartSpec,
    weights: &[Option<f64>],
    ends: impl Fn(usize) -> String,
) -> Option<String> {
    if weights.iter().flatten().count() < 2 {
        return None;
    }
    let (edge, weight) = weights
        .iter()
        .enumerate()
        .filter_map(|(edge, weight)| weight.map(|weight| (edge, weight)))
        .fold(
            None,
            |best: Option<(usize, f64)>, (edge, weight)| match best {
                Some((_, most)) if most >= weight => best,
                _ => Some((edge, weight)),
            },
        )?;
    Some(text::heaviest_edge(
        spec.locale,
        &ends(edge),
        &format_value(weight, spec.number_style()),
    ))
}

/// The chart in sentences: its size and lanes, the main path, every step in reading order with
/// where it leads, and the groups.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let flow = flow(spec);
    let locale = spec.locale;
    let ends = flow.ends();
    let lanes: Vec<String> = flow.lanes.iter().map(|lane| lane.label.clone()).collect();
    let mut description = text::flow_opening(locale, flow.nodes.len(), flow.edges.len(), &lanes);
    let main_path = flow.main_path_steps();
    if !main_path.is_empty() {
        let path: Vec<&str> = main_path
            .iter()
            .map(|id| flow.nodes[flow.node(id).expect("validated")].label.as_str())
            .collect();
        description.push(' ');
        description.push_str(&text::main_path(locale, &path));
    }
    let marked: Vec<bool> = flow
        .nodes
        .iter()
        .map(|node| matches!(node.kind, NodeKind::Start | NodeKind::End))
        .collect();
    let structure = Structure::of(flow.nodes.len(), &ends, &marked, Facts { directed: true })
        .sentences(locale, |node| flow.nodes[node].label.clone());
    if !structure.is_empty() {
        description.push(' ');
        description.push_str(&structure);
    }
    let weights: Vec<Option<f64>> = flow.edges.iter().map(|edge| edge.weight).collect();
    if let Some(heaviest) = heaviest_edge(spec, &weights, |edge| {
        let (from, to) = ends[edge];
        format!("{} → {}", flow.nodes[from].label, flow.nodes[to].label)
    }) {
        description.push(' ');
        description.push_str(&heaviest);
    }
    for node in reading_order(&Diagram::from_flow(flow)) {
        let step = &flow.nodes[node];
        let lane = flow
            .lane_of(node)
            .map(|lane| flow.lanes[lane].label.as_str());
        write!(
            description,
            " {}",
            text::flow_step(
                locale,
                &step.label,
                step.kind,
                lane,
                &next_steps(spec, &ends, node, false)
            )
        )
        .expect("writing to String cannot fail");
    }
    let valued: Vec<(&str, f64)> = reading_order(&Diagram::from_flow(flow))
        .into_iter()
        .filter_map(|node| {
            let step = &flow.nodes[node];
            step.value.map(|value| (step.label.as_str(), value))
        })
        .collect();
    if !valued.is_empty() {
        description.push(' ');
        description.push_str(&heat::sentence(
            spec,
            &heat::name(spec, flow.value_label.as_deref()),
            &valued,
        ));
    }
    let with_parts: Vec<(&str, &[PartSpec])> = reading_order(&Diagram::from_flow(flow))
        .into_iter()
        .map(|node| &flow.nodes[node])
        .filter(|step| !step.parts.is_empty())
        .map(|step| (step.label.as_str(), step.parts.as_slice()))
        .collect();
    if !with_parts.is_empty() {
        description.push(' ');
        description.push_str(&parts::sentence(spec, &with_parts));
    }
    for group in &flow.groups {
        let members: Vec<&str> = group
            .nodes
            .iter()
            .map(|id| flow.nodes[flow.node(id).expect("validated")].label.as_str())
            .collect();
        description.push(' ');
        description.push_str(&text::flow_group(locale, &group.label, &members));
    }
    if let Some(sentence) = roles::sentence(locale, &with_roles(flow, &ends)) {
        description.push(' ');
        description.push_str(&sentence);
    }
    description.push_str(&notes::sentences(locale, &flow.notes, |id| {
        &flow.nodes[flow.node(id).expect("validated")].label
    }));
    description
}

/// Every step with a role, in reading order, then every edge with one, as `From → To`.
fn with_roles(flow: &FlowSpec, ends: &[(usize, usize)]) -> Vec<(DiagramRole, String)> {
    reading_order(&Diagram::from_flow(flow))
        .into_iter()
        .filter_map(|node| {
            let step = &flow.nodes[node];
            step.role.map(|role| (role, step.label.clone()))
        })
        .chain(
            flow.edges
                .iter()
                .zip(ends)
                .filter_map(|(edge, (from, to))| {
                    edge.role.map(|role| {
                        (
                            role,
                            format!("{} → {}", flow.nodes[*from].label, flow.nodes[*to].label),
                        )
                    })
                }),
        )
        .collect()
}

/// One row per step in reading order: its label, kind, lane and group where the chart has them,
/// and where it leads.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let flow = flow(spec);
    let words = spec.locale.words();
    let ends = flow.ends();
    let (lanes, groups) = (!flow.lanes.is_empty(), !flow.groups.is_empty());
    let mut columns = vec![words.step.to_owned(), words.message_kind.to_owned()];
    if lanes {
        columns.push(words.lane.to_owned());
    }
    if groups {
        columns.push(words.group.to_owned());
    }
    let values = flow.nodes.iter().any(|node| node.value.is_some());
    if values {
        columns.push(heat::name(spec, flow.value_label.as_deref()));
    }
    let with_parts = flow.nodes.iter().any(|node| !node.parts.is_empty());
    if with_parts {
        columns.push(words.parts.to_owned());
    }
    let with_roles = flow.nodes.iter().any(|node| node.role.is_some());
    if with_roles {
        columns.push(words.role.to_owned());
    }
    columns.push(words.leads_to.to_owned());
    let with_notes = flow.notes.iter().any(|note| note.on.is_some());
    if with_notes {
        columns.push(words.notes.to_owned());
    }
    let rows = reading_order(&Diagram::from_flow(flow))
        .into_iter()
        .map(|node| {
            let step = &flow.nodes[node];
            let mut row = vec![
                step.label.clone(),
                text::node_kind(spec.locale, step.kind).to_owned(),
            ];
            if lanes {
                row.push(
                    flow.lane_of(node)
                        .map_or_else(String::new, |lane| flow.lanes[lane].label.clone()),
                );
            }
            if groups {
                row.push(
                    flow.group_of(node)
                        .map_or_else(String::new, |group| flow.groups[group].label.clone()),
                );
            }
            if values {
                row.push(step.value.map_or_else(String::new, |value| {
                    format_value(value, spec.number_style())
                }));
            }
            if with_parts {
                row.push(parts::listed(spec.number_style(), &step.parts));
            }
            if with_roles {
                row.push(
                    step.role
                        .map_or_else(String::new, |role| text::role(spec.locale, role).to_owned()),
                );
            }
            row.push(next_steps(spec, &ends, node, true).join(", "));
            if with_notes {
                row.push(notes::of(&flow.notes, &step.id));
            }
            row
        })
        .collect();
    DataTable {
        caption: notes::caption(
            format!("{} {}", words.data_for, spec.title),
            spec.locale,
            &flow.notes,
            |_| true,
            |id| &flow.nodes[flow.node(id).expect("validated")].label,
        ),
        columns,
        rows,
    }
}
