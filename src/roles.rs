//! Roles of the nodes and edges of a diagram: what something means beyond its kind, such as
//! critical, planned or deprecated, drawn alike in every diagram. A role keeps the shape and the
//! fill of its kind and changes the outline or the line: its color, taken from the role colors of
//! the theme, and a sign that does not depend on color, a stronger or patterned line and a badge
//! on the corner of a box. One legend below the diagram names the roles it uses; the description
//! and the table name the role of everything that has one.

use crate::{
    diagram::BADGE,
    layout::{LABEL_SIZE, count},
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Scene, Text, TextAnchor},
    spec::{ChartType, DiagramRole, Locale},
    text,
};

/// The roles drawn on the outlines of boxes, on lines and their heads, and on the badges. They
/// come after the main path, so that a role recolors a main edge, and before the widths of
/// weighted edges, so that a weight keeps its width.
pub(crate) const STYLE: &str = ".chartlet-flow-role-critical .chartlet-flow-node{stroke:var(--chartlet-role-red);stroke-width:3}.chartlet-flow-role-planned .chartlet-flow-node{stroke:var(--chartlet-role-violet);stroke-dasharray:6 3}.chartlet-flow-role-deprecated .chartlet-flow-node{stroke:var(--chartlet-role-gray);stroke-dasharray:1.5 3}.chartlet-flow-role-critical .chartlet-flow-edge{stroke:var(--chartlet-role-red);stroke-width:2.6}.chartlet-flow-role-planned .chartlet-flow-edge{stroke:var(--chartlet-role-violet);stroke-dasharray:6 4}.chartlet-flow-role-deprecated .chartlet-flow-edge{stroke:var(--chartlet-role-gray);stroke-dasharray:1 4;stroke-linecap:round}.chartlet-flow-role-critical .chartlet-flow-head,.chartlet-flow-role-critical .chartlet-diagram-badge{fill:var(--chartlet-role-red);stroke:var(--chartlet-role-red)}.chartlet-flow-role-planned .chartlet-flow-head,.chartlet-flow-role-planned .chartlet-diagram-badge{fill:var(--chartlet-role-violet);stroke:var(--chartlet-role-violet)}.chartlet-flow-role-deprecated .chartlet-flow-head,.chartlet-flow-role-deprecated .chartlet-diagram-badge{fill:var(--chartlet-role-gray);stroke:var(--chartlet-role-gray)}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";

/// In class diagrams, the hollow marks of a relation stay hollow under a role, and the open
/// arrowhead of a dependency or an association takes the role's color like a filled head.
const CLASSES_STYLE: &str = ".chartlet-flow-role-critical .chartlet-flow-hollow,.chartlet-flow-role-planned .chartlet-flow-hollow,.chartlet-flow-role-deprecated .chartlet-flow-hollow{fill:var(--chartlet-background)}.chartlet-flow-role-critical .chartlet-flow-open{stroke:var(--chartlet-role-red)}.chartlet-flow-role-planned .chartlet-flow-open{stroke:var(--chartlet-role-violet)}.chartlet-flow-role-deprecated .chartlet-flow-open{stroke:var(--chartlet-role-gray)}";

/// In networks, the ring a role draws around a dot, which keeps the color of its group.
const NETWORK_STYLE: &str = ".chartlet-net-role{fill:none}";

/// The style of the roles of a diagram of `chart_type`.
pub(crate) fn style(chart_type: ChartType) -> String {
    match chart_type {
        ChartType::Classes => [STYLE, CLASSES_STYLE].concat(),
        ChartType::Network => [STYLE, NETWORK_STYLE].concat(),
        _ => STYLE.to_owned(),
    }
}

/// The groups of the shared stylesheet for the roles, each with the chart types it serves.
pub(crate) const STYLE_GROUPS: [(&str, &[ChartType]); 3] = [
    (
        STYLE,
        &[
            ChartType::Flow,
            ChartType::Architecture,
            ChartType::State,
            ChartType::Network,
            ChartType::Cycle,
            ChartType::Tree,
            ChartType::Classes,
        ],
    ),
    (CLASSES_STYLE, &[ChartType::Classes]),
    (NETWORK_STYLE, &[ChartType::Network]),
];

/// How far the ring of a role stands from the dot of a network node, which wears a halo of the
/// background on its edge.
pub(crate) const RING_GAP: f64 = 2.5;

const LEGEND_ROW: f64 = 22.0;
/// The length of the line that stands for an edge in the legend.
const SAMPLE: f64 = 22.0;

/// The class a node's or an edge's group gains for its role.
pub(crate) const fn class(role: DiagramRole) -> &'static str {
    match role {
        DiagramRole::Critical => " chartlet-flow-role-critical",
        DiagramRole::Planned => " chartlet-flow-role-planned",
        DiagramRole::Deprecated => " chartlet-flow-role-deprecated",
    }
}

/// The sign on a role's badge.
const fn sign(role: DiagramRole) -> &'static str {
    match role {
        DiagramRole::Critical => "!",
        DiagramRole::Planned => "+",
        DiagramRole::Deprecated => "−",
    }
}

/// The badge of a role centred on `(cx, cy)`: on a box, its top left corner, away from the labels
/// of the edges that leave it on its right or bottom; in the legend, beside the role's name.
pub(crate) fn push_badge(role: DiagramRole, (cx, cy): (f64, f64), elements: &mut Vec<Element>) {
    elements.push(Element::Circle(Circle {
        cx,
        cy,
        radius: BADGE,
        class: "chartlet-diagram-badge",
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    elements.push(Element::Text(Text {
        x: cx,
        y: cy + 3.5,
        class: "chartlet-diagram-badge-text",
        anchor: TextAnchor::Middle,
        content: sign(role).to_owned(),
    }));
}

/// The ring of a role around the dot of a network node centred on `(cx, cy)`: its pattern says the
/// role without color, as the outline of a box does; a dot is too small for a badge.
pub(crate) fn push_ring((cx, cy): (f64, f64), radius: f64, elements: &mut Vec<Element>) {
    elements.push(Element::Circle(Circle {
        cx,
        cy,
        radius,
        class: "chartlet-flow-node chartlet-net-role",
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
}

/// The roles a diagram uses, in their fixed order, each with whether a node and whether an edge
/// has it.
pub(crate) fn used(
    nodes: impl Iterator<Item = Option<DiagramRole>>,
    edges: impl Iterator<Item = Option<DiagramRole>>,
) -> Vec<(DiagramRole, bool, bool)> {
    let (nodes, edges): (Vec<DiagramRole>, Vec<DiagramRole>) =
        (nodes.flatten().collect(), edges.flatten().collect());
    [
        DiagramRole::Critical,
        DiagramRole::Planned,
        DiagramRole::Deprecated,
    ]
    .into_iter()
    .map(|role| (role, nodes.contains(&role), edges.contains(&role)))
    .filter(|(_, on_nodes, on_edges)| *on_nodes || *on_edges)
    .collect()
}

/// What draws the sign of a node with a role centred on a point: a badge, or a ring in networks.
pub(crate) type NodeSign = fn(DiagramRole, (f64, f64), &mut Vec<Element>);

/// The legend of the roles below a diagram: for every role used, the sign of a node with it
/// where a node has it (drawn by `node_sign` centred on a point), a piece of line where an edge
/// has it, and its name, in rows that fit. The canvas grows by its height.
pub(crate) fn push_legend(
    scene: &mut Scene,
    locale: Locale,
    used: &[(DiagramRole, bool, bool)],
    (margin, node_sign): (f64, NodeSign),
    metrics: &impl TextMetrics,
) {
    if used.is_empty() {
        return;
    }
    let room = f64::from(scene.width) - 2.0 * margin;
    let top = f64::from(scene.height) + 4.0;
    let (mut x, mut row) = (0.0, 0);
    for &(role, on_nodes, on_edges) in used {
        let name = capitalized(text::role(locale, role));
        let samples = if on_nodes { 2.0 * BADGE + 6.0 } else { 0.0 }
            + if on_edges { SAMPLE + 6.0 } else { 0.0 };
        let width = samples + metrics.width(&name, LABEL_SIZE) + 20.0;
        if x > 0.0 && x + width > room {
            x = 0.0;
            row += 1;
        }
        let y = top + 14.0 + LEGEND_ROW * count(row);
        let mut at = margin + x;
        scene
            .elements
            .push(Element::Group(class(role).trim_start().to_owned()));
        if on_nodes {
            node_sign(role, (at + BADGE, y - 4.0), &mut scene.elements);
            at += 2.0 * BADGE + 6.0;
        }
        if on_edges {
            scene.elements.push(Element::Line(Line {
                x1: at,
                y1: y - 4.0,
                x2: at + SAMPLE,
                y2: y - 4.0,
                class: "chartlet-flow-edge",
            }));
            at += SAMPLE + 6.0;
        }
        scene.elements.push(Element::GroupEnd);
        scene.elements.push(Element::Text(Text {
            x: at,
            y,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: name,
        }));
        x += width;
    }
    scene.height = crate::diagram::pixels(top + LEGEND_ROW * count(row + 1) + 6.0);
}

fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// The sentence with what has each role, as `(role, name)` pairs of nodes and edges, in their
/// order in the description; `None` when nothing has a role.
pub(crate) fn sentence(locale: Locale, named: &[(DiagramRole, String)]) -> Option<String> {
    let mut roles: Vec<DiagramRole> = named.iter().map(|(role, _)| *role).collect();
    roles.sort_unstable();
    roles.dedup();
    if roles.is_empty() {
        return None;
    }
    let entries: Vec<String> = roles
        .into_iter()
        .map(|role| {
            let members: Vec<&str> = named
                .iter()
                .filter(|(other, _)| *other == role)
                .map(|(_, name)| name.as_str())
                .collect();
            format!("{}: {}", text::role(locale, role), members.join(", "))
        })
        .collect();
    Some(text::roles(locale, &entries))
}
