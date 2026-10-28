//! Emphasis in a diagram: the one or two nodes and the path the reader should look at first.
//! An emphasized node gets the accent outline, heavier than any other, and an accent tint; the
//! edges of the emphasized path get the accent line, heavier than the main path; everything else
//! stays neutral. Weight says it without color. The marks are classes on the groups the diagram
//! draws, put there once the diagram is laid out, so every diagram type shares them; the
//! description and the data table name what is emphasized.

use std::fmt::Write as _;

use crate::{
    DataTable,
    scene::{Circle, Element, Scene},
    spec::{ChartSpec, ChartType, Locale},
};

/// The marks of an emphasized node and edge. The tint is the accent at 14 % opacity over the
/// background, so the text on it keeps the contrast of the text on the background; a node's own
/// fill and its shadow give way to it.
pub(crate) const STYLE: &str = ".chartlet-emph .chartlet-flow-node,.chartlet-emph .chartlet-seq-box{stroke:var(--chartlet-accent);stroke-width:3.5;fill:var(--chartlet-accent);fill-opacity:.14}.chartlet-emph .chartlet-diagram-shadow{fill:none}.chartlet-emph .chartlet-seq-actor,.chartlet-emph .chartlet-seq-actor-head{stroke:var(--chartlet-accent);stroke-width:2.6}.chartlet-emph-ring{fill:none;stroke:var(--chartlet-accent);stroke-width:3}.chartlet-emph-edge .chartlet-flow-edge,.chartlet-emph-edge .chartlet-seq-message{stroke:var(--chartlet-accent);stroke-width:3.2}.chartlet-emph-edge .chartlet-flow-head,.chartlet-emph-edge .chartlet-seq-head{fill:var(--chartlet-accent);stroke:var(--chartlet-accent)}.chartlet-emph-edge .chartlet-flow-hollow{fill:var(--chartlet-background)}.chartlet-emph-edge .chartlet-flow-solid{fill:var(--chartlet-accent)}.chartlet-emph-edge .chartlet-flow-open,.chartlet-emph-edge .chartlet-seq-head-open{stroke:var(--chartlet-accent);stroke-width:2.4}.chartlet-emph-edge .chartlet-diagram-chip-text{font-weight:700}.chartlet-emph-quiet .chartlet-flow-main{stroke:var(--chartlet-zero);stroke-width:1.4}.chartlet-emph-quiet .chartlet-flow-main-head{fill:var(--chartlet-zero);stroke:var(--chartlet-zero)}";

/// How far the ring of an emphasized network node stands from its dot.
const RING_GAP: f64 = 6.5;

/// The index a node group names itself by: its `chartlet-n-<index>` class.
fn own_index(classes: &str) -> Option<usize> {
    classes
        .split(' ')
        .find_map(|class| class.strip_prefix("chartlet-n-")?.parse().ok())
}

/// The nodes an edge group is tied to: its `chartlet-f-<index>` classes.
fn focus_indices(classes: &str) -> Vec<usize> {
    classes
        .split(' ')
        .filter_map(|class| class.strip_prefix("chartlet-f-")?.parse().ok())
        .collect()
}

/// Marks the emphasized nodes and the edges of the emphasized path in a laid-out diagram.
pub(crate) fn apply(spec: &ChartSpec, scene: &mut Scene) {
    let Some(emphasis) = spec.emphasis.as_ref().filter(|_| spec.has_emphasis()) else {
        return;
    };
    // Where each node of the specification stands among the nodes the diagram drew.
    let drawn: Vec<(String, usize)> = scene
        .elements
        .iter()
        .filter_map(|element| match element {
            Element::NodeGroup(classes, id) => Some((id.clone(), own_index(classes)?)),
            _ => None,
        })
        .collect();
    let index_of = |id: &String| {
        drawn
            .iter()
            .find(|(known, _)| known == id)
            .map(|(_, at)| *at)
    };
    let path: Vec<(usize, usize)> = emphasis
        .path
        .windows(2)
        .filter_map(|pair| {
            let (from, to) = (index_of(&pair[0])?, index_of(&pair[1])?);
            Some((from.min(to), from.max(to)))
        })
        .collect();
    let has_path = !emphasis.path.is_empty();
    let mut rings = Vec::new();
    for (position, element) in scene.elements.iter_mut().enumerate() {
        match element {
            Element::NodeGroup(classes, id) if emphasis.has_node(id) => {
                classes.push_str(" chartlet-emph");
                rings.push(position);
            }
            Element::Group(classes) | Element::EdgeGroup(classes, _) => {
                if own_index(classes).is_some() {
                    continue;
                }
                let tied = focus_indices(classes);
                let Some((low, high)) = tied.first().zip(tied.last()) else {
                    continue;
                };
                if path.contains(&(*low, *high)) {
                    classes.push_str(" chartlet-emph-edge");
                } else if has_path {
                    classes.push_str(" chartlet-emph-quiet");
                }
            }
            _ => {}
        }
    }
    if spec.chart_type != ChartType::Network {
        return;
    }
    // A dot is too small for an outline of its own: an emphasized one wears a ring.
    for position in rings.into_iter().rev() {
        let dot = scene.elements[position..]
            .iter()
            .position(|element| matches!(element, Element::Circle(_)))
            .map(|offset| position + offset);
        let Some(dot) = dot else { continue };
        let Element::Circle(circle) = &scene.elements[dot] else {
            continue;
        };
        let ring = Element::Circle(Circle {
            cx: circle.cx,
            cy: circle.cy,
            radius: circle.radius + RING_GAP,
            class: "chartlet-emph-ring",
            series_index: None,
            style_index: None,
            tooltip: None,
        });
        scene.elements.insert(dot + 1, ring);
    }
}

/// The name of the emphasized nodes and of the path, as the description's sentences, each
/// preceded by a space; empty without emphasis.
pub(crate) fn sentences(spec: &ChartSpec) -> String {
    let Some(emphasis) = spec.emphasis.as_ref().filter(|_| spec.has_emphasis()) else {
        return String::new();
    };
    let labels = spec.emphasis_labels();
    let label = |id: &String| {
        labels
            .iter()
            .find(|(known, _)| known == id)
            .map_or_else(|| id.clone(), |(_, label)| (*label).to_owned())
    };
    let mut sentences = String::new();
    if !emphasis.nodes.is_empty() {
        let names: Vec<String> = emphasis.nodes.iter().map(label).collect();
        let lead = match spec.locale {
            Locale::En => "Emphasized",
            Locale::De => "Hervorgehoben",
        };
        write!(sentences, " {lead}: {}.", names.join(", ")).expect("writing to String cannot fail");
    }
    if !emphasis.path.is_empty() {
        let names: Vec<String> = emphasis.path.iter().map(label).collect();
        let lead = match spec.locale {
            Locale::En => "Emphasized path",
            Locale::De => "Hervorgehobener Weg",
        };
        write!(sentences, " {lead}: {}.", names.join(" → "))
            .expect("writing to String cannot fail");
    }
    sentences
}

/// The description of a diagram with the sentences of its emphasis and its legend appended.
pub(crate) fn describe(spec: &ChartSpec, mut description: String) -> String {
    if !spec.takes_emphasis() {
        return description;
    }
    description.push_str(&sentences(spec));
    if let Some(legend) = crate::legend::sentence(spec) {
        description.push(' ');
        description.push_str(&legend);
    }
    description
}

/// Adds a column that says which nodes are emphasized and which lie on the emphasized path.
pub(crate) fn extend_table(spec: &ChartSpec, table: &mut DataTable) {
    let Some(emphasis) = spec.emphasis.as_ref().filter(|_| spec.has_emphasis()) else {
        return;
    };
    let locale = spec.locale;
    let labels = spec.emphasis_labels();
    let id_of = |label: &str| {
        labels
            .iter()
            .find(|(_, known)| *known == label)
            .map(|(id, _)| (*id).to_owned())
    };
    let on_path = |id: &str| emphasis.path.iter().any(|step| step == id);
    let node_cell = |label: &str| {
        let Some(id) = id_of(label) else {
            return String::new();
        };
        cell(locale, emphasis.has_node(&id), on_path(&id))
    };
    let column = match locale {
        Locale::En => "Emphasis",
        Locale::De => "Hervorhebung",
    };
    let rows: Vec<String> = if spec.chart_type == ChartType::Sequence {
        // A message row names its sender and receiver; it is emphasized when either is, and on
        // the path when both are consecutive steps of it.
        table
            .rows
            .iter()
            .map(|row| {
                let ends: Vec<String> = [row.get(1), row.get(2)]
                    .into_iter()
                    .flatten()
                    .filter_map(|label| id_of(label))
                    .collect();
                let joined = ends.len() == 2
                    && emphasis.path.windows(2).any(|pair| {
                        (pair[0] == ends[0] && pair[1] == ends[1])
                            || (pair[0] == ends[1] && pair[1] == ends[0])
                    });
                cell(locale, false, joined)
            })
            .collect()
    } else {
        table
            .rows
            .iter()
            .map(|row| {
                row.first()
                    .map_or_else(String::new, |label| node_cell(label))
            })
            .collect()
    };
    table.columns.push(column.to_owned());
    for (row, value) in table.rows.iter_mut().zip(rows) {
        row.push(value);
    }
}

/// The cell of a node or a message: emphasized, on the emphasized path, both or neither.
fn cell(locale: Locale, emphasized: bool, on_path: bool) -> String {
    let mut parts = Vec::new();
    if emphasized {
        parts.push(match locale {
            Locale::En => "emphasized",
            Locale::De => "hervorgehoben",
        });
    }
    if on_path {
        parts.push(match locale {
            Locale::En => "on emphasized path",
            Locale::De => "auf hervorgehobenem Weg",
        });
    }
    parts.join(", ")
}
