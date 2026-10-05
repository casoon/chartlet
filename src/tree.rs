//! Trees: a root and the nodes below it, such as an organization chart, an ownership structure
//! or a hierarchy of norms.
//!
//! Every level of the tree takes a row (a column in landscape); the children of a node sit side
//! by side below it and the node is centered over them. Subtrees are pushed together until they
//! are a gap apart at every level, so that a narrow subtree tucks in under a broad neighbour. The
//! description and the data table list every node with its parent, so that the picture is never
//! the only place the structure is said.

use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::{self, chip, pixels, rounded, warn_growth, wrap},
    error::ChartWarning,
    flow::{Shape, focus_classes, shape},
    layout::{NARROW, count, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Hotspot, Scene, Text, TextAnchor},
    spec::{ChartSpec, DiagramOrientation, TreeNodeKind, TreeSpec},
    text,
};

const MARGIN: f64 = 24.0;
const COMPACT_MARGIN: f64 = 12.0;
const BOTTOM: f64 = 16.0;
const LABEL_SIZE: f64 = 13.0;
const SUBLABEL_SIZE: f64 = 11.0;
/// Distance between the baselines of a node's label lines, and of its sublabel lines.
const NODE_LINE: f64 = 16.0;
const SUBLABEL_LINE: f64 = 14.0;
const PAD: f64 = 12.0;
const MIN_NODE: f64 = 96.0;
const MAX_NODE: f64 = 190.0;
/// The same in a diagram narrower than [`NARROW`], such as a mobile variant.
const COMPACT_MIN_NODE: f64 = 64.0;
const COMPACT_MAX_NODE: f64 = 120.0;
/// Space between two nodes next to each other, and between two subtrees.
const SIBLING_GAP: f64 = 20.0;
const COMPACT_SIBLING_GAP: f64 = 10.0;
/// Space between the two boxes of a couple.
const PARTNER_GAP: f64 = 28.0;
/// Space between two levels, and the same when a link between them carries a label.
const LEVEL_GAP: f64 = 36.0;
const LINKED_LEVEL_GAP: f64 = 58.0;
/// In landscape the label of a link sits on its last, horizontal stretch, which must be long
/// enough to hold it.
const LINKED_COLUMN_GAP: f64 = 112.0;

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let tree = tree(spec);
    let compact = spec.width < NARROW;
    let margin = if compact { COMPACT_MARGIN } else { MARGIN };
    let width = f64::from(spec.width);
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let nodes = boxes(tree, compact, metrics, warnings);
    let landscape = match tree.orientation {
        DiagramOrientation::Portrait => false,
        DiagramOrientation::Landscape => true,
        DiagramOrientation::Auto => {
            let portrait = Plan::new(spec, tree, &nodes, false, (top, margin, compact));
            let landscape = Plan::new(spec, tree, &nodes, true, (top, margin, compact));
            diagram::prefers_landscape(
                spec,
                (portrait.width, portrait.height),
                (landscape.width, landscape.height),
            )
        }
    };
    let plan = Plan::new(spec, tree, &nodes, landscape, (top, margin, compact));
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        width - 2.0 * margin,
        metrics,
        warnings,
    );
    warn_growth(spec, plan.width, plan.height, warnings);
    plan.draw(tree, &nodes, metrics, &mut elements);
    Scene {
        width: pixels(plan.width),
        height: pixels(plan.height),
        elements,
    }
}

fn tree(spec: &ChartSpec) -> &TreeSpec {
    spec.tree
        .as_ref()
        .expect("validated trees carry a tree block")
}

/// A node as drawn: its label and sublabel lines and its box, width and height.
struct NodeBox {
    lines: Vec<String>,
    sublines: Vec<String>,
    width: f64,
    height: f64,
}

fn boxes(
    tree: &TreeSpec,
    compact: bool,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<NodeBox> {
    let (min, max) = if compact {
        (COMPACT_MIN_NODE, COMPACT_MAX_NODE)
    } else {
        (MIN_NODE, MAX_NODE)
    };
    tree.nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            // Never narrower than the longest word, which cannot wrap.
            let word = node
                .label
                .split(' ')
                .map(|word| metrics.width(word, LABEL_SIZE))
                .fold(0.0, f64::max);
            let room = (max - 2.0 * PAD).max(word);
            let lines = wrap(
                &node.label,
                room,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/tree/nodes/{index}/label"),
            );
            let sublines = node.sublabel.as_ref().map_or_else(Vec::new, |sublabel| {
                wrap(
                    sublabel,
                    room,
                    SUBLABEL_SIZE,
                    metrics,
                    warnings,
                    &format!("/tree/nodes/{index}/sublabel"),
                )
            });
            let text = lines
                .iter()
                .map(|line| metrics.width(line, LABEL_SIZE))
                .chain(
                    sublines
                        .iter()
                        .map(|line| metrics.width(line, SUBLABEL_SIZE)),
                )
                .fold(0.0, f64::max);
            // A person's head takes room on top of the box.
            let head = if node.kind == TreeNodeKind::Person {
                18.0
            } else {
                0.0
            };
            let height = 20.0
                + NODE_LINE * count(lines.len())
                + SUBLABEL_LINE * count(sublines.len())
                + head;
            NodeBox {
                width: (text + 2.0 * PAD)
                    .clamp(min, max.max(room + 2.0 * PAD))
                    .ceil(),
                height,
                lines,
                sublines,
            }
        })
        .collect()
}

/// One orientation of the layout. Positions are centers, on the page.
struct Plan {
    landscape: bool,
    center: Vec<(f64, f64)>,
    /// Each level's row on the main axis, on the page: where it begins and ends.
    rows: Vec<(f64, f64)>,
    /// The level of every node.
    levels: Vec<usize>,
    width: f64,
    height: f64,
}

/// The cross position of every node, the root at 0.
///
/// Subtrees go from the leaves up: each node's offset to its parent, and the reach of its subtree
/// on either side at every level, measured from the node's own center. The next child of a node
/// is pushed as far as it must so that it keeps `gap` from everything placed before it at every
/// level, and the node then stands over the middle of its first and last child.
///
/// A couple counts as one node whose reach on the two sides is `extent`, from the middle of the
/// line between its partners; a partner takes no position of its own here.
fn cross_positions(
    tree: &TreeSpec,
    count_of_nodes: usize,
    extent: impl Fn(usize) -> (f64, f64),
    gap: f64,
) -> Vec<f64> {
    let order = tree.reading_order();
    let mut offset = vec![0.0; count_of_nodes];
    let mut reach: Vec<Vec<(f64, f64)>> = vec![Vec::new(); count_of_nodes];
    for &node in order
        .iter()
        .rev()
        .filter(|node| tree.head_of(**node).is_none())
    {
        let own = extent(node);
        let children = tree.children(node);
        let mut placed: Vec<(f64, f64)> = Vec::new();
        let mut at = Vec::with_capacity(children.len());
        for &child in &children {
            let shift = reach[child]
                .iter()
                .zip(&placed)
                .map(|(new, old)| old.1 + gap - new.0)
                .fold(f64::NEG_INFINITY, f64::max);
            let shift = if placed.is_empty() { 0.0 } else { shift };
            for (level, extent) in reach[child].iter().enumerate() {
                let moved = (extent.0 + shift, extent.1 + shift);
                match placed.get_mut(level) {
                    Some(old) => *old = (old.0.min(moved.0), old.1.max(moved.1)),
                    None => placed.push(moved),
                }
            }
            at.push(shift);
        }
        // The node stands over the middle of its first and last child.
        let middle = match (at.first(), at.last()) {
            (Some(first), Some(last)) => f64::midpoint(*first, *last),
            _ => 0.0,
        };
        for (&child, shift) in children.iter().zip(&at) {
            offset[child] = shift - middle;
        }
        let mut extents = vec![own];
        extents.extend(
            placed
                .iter()
                .map(|(low, high)| (low - middle, high - middle)),
        );
        reach[node] = extents;
    }
    // Positions across: the root at 0, every other node from its parent.
    let mut cross = vec![0.0; count_of_nodes];
    for &node in &order {
        if let Some(parent) = tree.parent_of(node) {
            cross[node] = cross[parent] + offset[node];
        }
    }
    cross
}

/// The level of every node, where each level's row begins along the flow and how deep it is: as
/// deep as its deepest node, with more room before a level whose links carry labels.
fn level_rows(
    tree: &TreeSpec,
    count_of_nodes: usize,
    along: impl Fn(usize) -> f64,
    landscape: bool,
) -> (Vec<usize>, Vec<f64>, Vec<f64>) {
    let depths: Vec<usize> = (0..count_of_nodes).map(|node| tree.depth(node)).collect();
    let levels = depths.iter().copied().max().unwrap_or(0) + 1;
    let mut thickness = vec![0.0_f64; levels];
    let mut linked = vec![false; levels];
    for node in 0..count_of_nodes {
        thickness[depths[node]] = thickness[depths[node]].max(along(node));
        linked[depths[node]] |= tree.nodes[node].link.is_some();
    }
    let mut level_start = vec![0.0; levels];
    for level in 1..levels {
        let space = if linked[level] && landscape {
            LINKED_COLUMN_GAP
        } else if linked[level] {
            LINKED_LEVEL_GAP
        } else {
            LEVEL_GAP
        };
        level_start[level] = level_start[level - 1] + thickness[level - 1] + space;
    }
    (depths, level_start, thickness)
}

impl Plan {
    fn new(
        spec: &ChartSpec,
        tree: &TreeSpec,
        nodes: &[NodeBox],
        landscape: bool,
        (top, margin, compact): (f64, f64, bool),
    ) -> Self {
        let gap = if compact {
            COMPACT_SIBLING_GAP
        } else {
            SIBLING_GAP
        };
        // Across the levels (the cross axis) and along them (the main axis).
        let across = |node: usize| {
            if landscape {
                nodes[node].height
            } else {
                nodes[node].width
            }
        };
        let along = |node: usize| {
            if landscape {
                nodes[node].width
            } else {
                nodes[node].height
            }
        };
        // A couple is as wide as its two boxes and the room between them, and the middle of that
        // room is where its children's lines start.
        let extent = |node: usize| match tree.partner_of(node) {
            Some(partner) => (
                -(across(node) + PARTNER_GAP / 2.0),
                across(partner) + PARTNER_GAP / 2.0,
            ),
            None => (-across(node) / 2.0, across(node) / 2.0),
        };
        let mut cross = cross_positions(tree, nodes.len(), extent, gap);
        for node in 0..nodes.len() {
            if let Some(partner) = tree.partner_of(node) {
                let middle = cross[node];
                cross[node] = middle - PARTNER_GAP / 2.0 - across(node) / 2.0;
                cross[partner] = middle + PARTNER_GAP / 2.0 + across(partner) / 2.0;
            }
        }
        let (depths, level_start, thickness) = level_rows(tree, nodes.len(), along, landscape);
        let levels = thickness.len();
        let length = level_start[levels - 1] + thickness[levels - 1];
        let (low, high) = cross.iter().enumerate().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(low, high), (node, at)| {
                (
                    low.min(at - across(node) / 2.0),
                    high.max(at + across(node) / 2.0),
                )
            },
        );
        let breadth = high - low;
        let (width, height) = (f64::from(spec.width), f64::from(spec.height));
        let (page_width, page_height) = if landscape {
            (
                width.max(2.0 * margin + length),
                height.max(top + breadth + BOTTOM),
            )
        } else {
            (
                width.max(2.0 * margin + breadth),
                height.max(top + length + BOTTOM),
            )
        };
        // The tree stands in the middle of the room the canvas leaves across, and begins at the
        // top (at the left in landscape).
        let center = (0..nodes.len())
            .map(|node| {
                // Nodes of a level start where the level starts.
                let along_at = level_start[depths[node]] + along(node) / 2.0;
                if landscape {
                    let room = page_height - top - BOTTOM;
                    (
                        margin + along_at,
                        top + (room - breadth) / 2.0 + cross[node] - low,
                    )
                } else {
                    let room = page_width - 2.0 * margin;
                    (
                        margin + (room - breadth) / 2.0 + cross[node] - low,
                        top + along_at,
                    )
                }
            })
            .collect();
        let origin = if landscape { margin } else { top };
        let rows = (0..levels)
            .map(|level| {
                (
                    origin + level_start[level],
                    origin + level_start[level] + thickness[level],
                )
            })
            .collect();
        Self {
            landscape,
            center,
            rows,
            levels: depths,
            width: page_width,
            height: page_height,
        }
    }

    /// A node's box on the page: left, top, width and height.
    fn node_box(&self, nodes: &[NodeBox], node: usize) -> (f64, f64, f64, f64) {
        let (x, y) = self.center[node];
        let (width, height) = (nodes[node].width, nodes[node].height);
        (x - width / 2.0, y - height / 2.0, width, height)
    }

    fn draw(
        &self,
        tree: &TreeSpec,
        nodes: &[NodeBox],
        metrics: &impl TextMetrics,
        elements: &mut Vec<Element>,
    ) {
        let related = |node: usize| {
            let mut related: Vec<usize> = tree
                .children(node)
                .into_iter()
                .chain(tree.parent_of(node))
                .chain(std::iter::once(node))
                .collect();
            related.sort_unstable();
            related
        };
        for node in 0..nodes.len() {
            let Some(partner) = tree.partner_of(node) else {
                continue;
            };
            elements.push(Element::Group(focus_classes(
                None,
                &[node.min(partner), node.max(partner)],
            )));
            let (from, to) = self.couple_line(nodes, node, partner);
            elements.push(Element::Polyline(diagram::polyline(
                vec![from, to],
                "chartlet-flow-edge",
                Some(format!(
                    "{} + {}",
                    tree.nodes[node].label, tree.nodes[partner].label
                )),
            )));
            elements.push(Element::GroupEnd);
        }
        let mut labels = Vec::new();
        for node in tree.reading_order() {
            let Some(parent) = tree.parent_of(node) else {
                continue;
            };
            elements.push(Element::Group(focus_classes(
                None,
                &[parent.min(node), parent.max(node)],
            )));
            self.draw_link(tree, nodes, parent, node, elements, &mut labels);
            elements.push(Element::GroupEnd);
        }
        for (lines, at, classes) in labels {
            elements.push(Element::Group(classes));
            chip(&lines, at, TextAnchor::Middle, metrics, elements);
            elements.push(Element::GroupEnd);
        }
        for node in 0..nodes.len() {
            let spec = &tree.nodes[node];
            elements.push(Element::Group(focus_classes(Some(node), &related(node))));
            let (x, y, width, height) = self.node_box(nodes, node);
            let tooltip = Some(match &spec.sublabel {
                Some(sublabel) => format!("{} – {sublabel}", spec.label),
                None => spec.label.clone(),
            });
            let kind = match spec.kind {
                TreeNodeKind::Unit => Shape::Process,
                TreeNodeKind::Person => Shape::Person,
                TreeNodeKind::External => Shape::External,
            };
            shape(kind, (x, y, width, height), tooltip, elements);
            Self::node_text(
                &nodes[node],
                spec.kind == TreeNodeKind::Person,
                (x, y, width, height),
                elements,
            );
            elements.push(Element::GroupEnd);
            elements.push(Element::Hotspot(Hotspot {
                node,
                label: spec.label.clone(),
                x,
                y,
                width,
                height,
            }));
        }
    }

    /// The short line between the two boxes of a couple: from the head to its partner.
    fn couple_line(
        &self,
        nodes: &[NodeBox],
        head: usize,
        partner: usize,
    ) -> ((f64, f64), (f64, f64)) {
        let (ax, ay, aw, ah) = self.node_box(nodes, head);
        let (bx, by, bw, bh) = self.node_box(nodes, partner);
        if self.landscape {
            // Stacked: the line runs down where the two boxes overlap across.
            let x = f64::midpoint(ax.max(bx), (ax + aw).min(bx + bw));
            ((x, ay + ah), (x, by))
        } else {
            let y = f64::midpoint(ay.max(by), (ay + ah).min(by + bh));
            ((ax + aw, y), (bx, y))
        }
    }

    /// Where the lines to the children of `parent` start: on its bottom edge, or, for a couple,
    /// in the middle of the line between the partners.
    fn start_of(&self, nodes: &[NodeBox], parent: usize, partner: Option<usize>) -> (f64, f64) {
        if let Some(partner) = partner {
            let (from, to) = self.couple_line(nodes, parent, partner);
            return (f64::midpoint(from.0, to.0), f64::midpoint(from.1, to.1));
        }
        let (x, y, width, height) = self.node_box(nodes, parent);
        if self.landscape {
            (x + width, y + height / 2.0)
        } else {
            (x + width / 2.0, y + height)
        }
    }

    /// The line from a parent to a child: down from the parent, across, and down into the child
    /// (right, across and right in landscape), with the child's link on its last stretch.
    fn draw_link(
        &self,
        tree: &TreeSpec,
        nodes: &[NodeBox],
        parent: usize,
        child: usize,
        elements: &mut Vec<Element>,
        labels: &mut Vec<(Vec<String>, (f64, f64), String)>,
    ) {
        let (cx, cy, cw, ch) = self.node_box(nodes, child);
        // The line turns halfway between the row of the parent and the row of the child, the
        // same for all children of a level.
        let (_, parent_end) = self.rows[self.levels[parent]];
        let (child_start, _) = self.rows[self.levels[child]];
        let middle = f64::midpoint(parent_end, child_start);
        let from = self.start_of(nodes, parent, tree.partner_of(parent));
        let points = if self.landscape {
            let end = cy + ch / 2.0;
            vec![from, (middle, from.1), (middle, end), (cx, end)]
        } else {
            let end = cx + cw / 2.0;
            vec![from, (from.0, middle), (end, middle), (end, cy)]
        };
        let link = tree.nodes[child].link.as_deref();
        let tooltip = match link {
            Some(link) => format!(
                "{} → {}: {link}",
                tree.nodes[parent].label, tree.nodes[child].label
            ),
            None => format!("{} → {}", tree.nodes[parent].label, tree.nodes[child].label),
        };
        elements.push(Element::Polyline(diagram::polyline(
            rounded(&points),
            "chartlet-flow-edge",
            Some(tooltip),
        )));
        if let Some(link) = link {
            let last = points[points.len() - 1];
            // Between the turn and the row of the child, on the line.
            let at = if self.landscape {
                (f64::midpoint(middle, child_start), last.1 + 4.5)
            } else {
                (last.0, f64::midpoint(middle, child_start) + 4.5)
            };
            labels.push((
                vec![link.to_owned()],
                at,
                focus_classes(None, &[parent.min(child), parent.max(child)]),
            ));
        }
    }

    /// A node's label, centered on one or two lines, and its sublabel below.
    fn node_text(
        node: &NodeBox,
        person: bool,
        (x, y, width, height): (f64, f64, f64, f64),
        elements: &mut Vec<Element>,
    ) {
        let (middle_x, middle_y) = (x + width / 2.0, y + height / 2.0);
        let block =
            NODE_LINE * count(node.lines.len() - 1) + SUBLABEL_LINE * count(node.sublines.len());
        // The text clears the head of a person.
        let first = middle_y - block / 2.0 + 4.5 + if person { 8.0 } else { 0.0 };
        for (index, line) in node.lines.iter().enumerate() {
            elements.push(Element::Text(Text {
                x: middle_x,
                y: first + NODE_LINE * count(index),
                class: "chartlet-flow-label",
                anchor: TextAnchor::Middle,
                content: line.clone(),
            }));
        }
        for (index, line) in node.sublines.iter().enumerate() {
            elements.push(Element::Text(Text {
                x: middle_x,
                y: first
                    + NODE_LINE * count(node.lines.len() - 1)
                    + SUBLABEL_LINE * count(index + 1),
                class: "chartlet-flow-sublabel",
                anchor: TextAnchor::Middle,
                content: line.clone(),
            }));
        }
    }
}

/// A node named with its sublabel and link in a parenthesis: `Holding GmbH (holding, 60 %)`; a
/// couple names both of its partners.
fn mention(tree: &TreeSpec, node: usize) -> String {
    let named = |node: usize, with_link: bool| {
        let spec = &tree.nodes[node];
        let notes: Vec<&str> = [
            spec.sublabel.as_deref(),
            spec.link.as_deref().filter(|_| with_link),
        ]
        .into_iter()
        .flatten()
        .collect();
        if notes.is_empty() {
            spec.label.clone()
        } else {
            format!("{} ({})", spec.label, notes.join(", "))
        }
    };
    match tree.partner_of(node) {
        Some(partner) => format!("{} + {}", named(node, true), named(partner, false)),
        None => named(node, true),
    }
}

/// The tree in sentences: its size, its root, and what hangs below every node that has anything
/// below it, in reading order.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let tree = tree(spec);
    let locale = spec.locale;
    let levels = (0..tree.nodes.len())
        .map(|node| tree.depth(node))
        .max()
        .unwrap_or(0)
        + 1;
    let mut description = text::tree_opening(locale, tree.nodes.len(), levels);
    description.push(' ');
    description.push_str(&text::tree_root(locale, &mention(tree, tree.root())));
    for node in tree.reading_order() {
        let children: Vec<String> = tree
            .children(node)
            .into_iter()
            .map(|child| mention(tree, child))
            .collect();
        if !children.is_empty() {
            let name = tree.partner_of(node).map_or_else(
                || tree.nodes[node].label.clone(),
                |partner| format!("{} + {}", tree.nodes[node].label, tree.nodes[partner].label),
            );
            write!(description, " {name}: {}.", children.join(", "))
                .expect("writing to String cannot fail");
        }
    }
    description
}

/// One row per node in reading order: its label, level, parent, link and children.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let tree = tree(spec);
    let words = spec.locale.words();
    let linked = tree.nodes.iter().any(|node| node.link.is_some());
    let coupled = tree.nodes.iter().any(|node| node.partner.is_some());
    let mut columns = vec![
        words.node.to_owned(),
        words.level.to_owned(),
        words.parent.to_owned(),
    ];
    if coupled {
        columns.push(words.partner.to_owned());
    }
    if linked {
        columns.push(words.link.to_owned());
    }
    columns.push(words.children.to_owned());
    let rows = tree
        .reading_order()
        .into_iter()
        .map(|node| {
            let mut row = vec![
                tree.nodes[node].label.clone(),
                (tree.depth(node) + 1).to_string(),
                tree.parent_of(node)
                    .map_or_else(String::new, |parent| tree.nodes[parent].label.clone()),
            ];
            if coupled {
                row.push(
                    tree.partner_of(node)
                        .or_else(|| tree.head_of(node))
                        .map_or_else(String::new, |other| tree.nodes[other].label.clone()),
                );
            }
            if linked {
                row.push(tree.nodes[node].link.clone().unwrap_or_default());
            }
            row.push(
                tree.children(node)
                    .into_iter()
                    .map(|child| tree.nodes[child].label.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BuiltinMetrics, ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative,
    };

    use super::{Plan, boxes, tree};

    const SPEC: &str = r#"{
        "schemaVersion": 1,
        "type": "tree",
        "title": "Company",
        "width": 900,
        "height": 500,
        "tree": {
            "nodes": [
                {"id": "ceo", "label": "Chief executive", "sublabel": "board"},
                {"id": "tech", "label": "Technology", "parent": "ceo", "link": "reports"},
                {"id": "ops", "label": "Operations", "parent": "ceo"},
                {"id": "platform", "label": "Platform", "parent": "tech"},
                {"id": "apps", "label": "Applications", "parent": "tech"},
                {"id": "web", "label": "Web", "parent": "apps"},
                {"id": "mobile", "label": "Mobile", "parent": "apps"},
                {"id": "support", "label": "Support", "parent": "ops"}
            ]
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

    fn plan(json: &str, landscape: bool) -> (Plan, Vec<(f64, f64, f64, f64)>) {
        let spec: ChartSpec = serde_json::from_str(json).expect("a specification");
        let tree = tree(&spec);
        let nodes = boxes(tree, false, &BuiltinMetrics, &mut Vec::new());
        let plan = Plan::new(&spec, tree, &nodes, landscape, (56.0, 24.0, false));
        let rectangles = (0..nodes.len())
            .map(|node| plan.node_box(&nodes, node))
            .collect();
        (plan, rectangles)
    }

    #[test]
    fn no_two_nodes_overlap_in_either_orientation() {
        for landscape in [false, true] {
            let (_, boxes) = plan(SPEC, landscape);
            for (index, a) in boxes.iter().enumerate() {
                for b in &boxes[index + 1..] {
                    let apart = a.0 + a.2 <= b.0
                        || b.0 + b.2 <= a.0
                        || a.1 + a.3 <= b.1
                        || b.1 + b.3 <= a.1;
                    assert!(apart, "{a:?} overlaps {b:?} (landscape: {landscape})");
                }
            }
        }
    }

    #[test]
    fn a_parent_stands_over_the_middle_of_its_children() {
        let (plan, _) = plan(SPEC, false);
        // ids by position: tech (1) has platform (3) and apps (4); apps has web (5) and mobile (6).
        for (parent, first, last) in [(1, 3, 4), (4, 5, 6)] {
            let middle = f64::midpoint(plan.center[first].0, plan.center[last].0);
            assert!((plan.center[parent].0 - middle).abs() < 0.01, "{parent}");
        }
    }

    #[test]
    fn a_narrow_subtree_tucks_in_under_a_broad_neighbour() {
        // `ops` has one child; its subtree may stand closer than the width of the whole of
        // `tech`'s subtree.
        let (plan, _) = plan(SPEC, false);
        assert!(plan.center[2].0 - plan.center[1].0 < 4.0 * 190.0);
        assert!(plan.center[7].1 > plan.center[2].1);
    }

    #[test]
    fn a_tree_renders_without_warnings_and_twice_to_the_same_bytes() {
        let first = svg(SPEC);
        assert!(first.warnings.is_empty(), "{:?}", first.warnings);
        assert_eq!(first.content, svg(SPEC).content);
        assert!(first.content.contains(">Chief executive</text>"));
        assert!(first.content.contains(">reports</text>"));
    }

    #[test]
    fn the_text_alternative_names_the_root_and_what_hangs_below_every_node() {
        let alternative =
            text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
        let description = alternative.description;
        assert!(
            description
                .starts_with("Tree with 8 nodes in 4 levels. Root: Chief executive (board).")
        );
        assert!(description.contains("Chief executive: Technology (reports), Operations."));
        assert!(description.contains("Applications: Web, Mobile."));
    }

    #[test]
    fn the_html_table_lists_every_node_with_level_and_parent() {
        let html = render_json(SPEC, RenderFormat::Html, &RenderOptions::default())
            .expect("renders")
            .content;
        for needle in [">Level<", ">Parent<", ">Link<", "Platform"] {
            assert!(html.contains(needle), "{needle}");
        }
    }

    #[test]
    fn german_writes_the_description_and_the_table_in_german() {
        let json = SPEC.replace(
            "\"title\": \"Company\",",
            "\"title\": \"Firma\", \"locale\": \"de\",",
        );
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(
            alternative
                .description
                .starts_with("Baum mit 8 Knoten in 4 Ebenen. Wurzel:")
        );
        let html = render_json(&json, RenderFormat::Html, &RenderOptions::default())
            .expect("renders")
            .content;
        assert!(html.contains(">Knoten<") && html.contains(">Übergeordnet<"));
    }

    #[test]
    fn invalid_trees_name_the_field() {
        for (from, to, code, path) in [
            (
                r#""id": "ops", "label": "Operations", "parent": "ceo""#,
                r#""id": "ops", "label": "Operations""#,
                "invalid_root",
                "/tree/nodes/2",
            ),
            (
                r#""id": "web", "label": "Web", "parent": "apps""#,
                r#""id": "web", "label": "Web", "parent": "nowhere""#,
                "unknown_node",
                "/tree/nodes/5/parent",
            ),
            (
                r#""id": "ops""#,
                r#""id": "tech""#,
                "duplicate_id",
                "/tree/nodes/2/id",
            ),
            (
                r#""id": "ceo", "label": "Chief executive", "sublabel": "board""#,
                r#""id": "ceo", "label": "Chief executive", "link": "100 %""#,
                "option_not_supported",
                "/tree/nodes/0/link",
            ),
            (
                r#""id": "ceo", "label": "Chief executive", "sublabel": "board""#,
                r#""id": "ceo", "label": "Chief executive", "parent": "web""#,
                "invalid_root",
                "/tree/nodes",
            ),
        ] {
            let (got_code, got_path) = error(&SPEC.replace(from, to));
            assert_eq!((got_code, got_path.as_str()), (code, path), "{from}");
        }
    }

    #[test]
    fn a_circle_of_parents_and_a_missing_block_are_refused() {
        let circle = r#"{"schemaVersion": 1, "type": "tree", "title": "Loop", "width": 600, "height": 300,
            "tree": {"nodes": [
                {"id": "a", "label": "A"},
                {"id": "b", "label": "B", "parent": "c"},
                {"id": "c", "label": "C", "parent": "b"}
            ]}}"#;
        assert_eq!(
            error(circle),
            ("circular_parent", "/tree/nodes/1/parent".to_owned())
        );
        let missing =
            r#"{"schemaVersion": 1, "type": "tree", "title": "None", "width": 600, "height": 300}"#;
        assert_eq!(error(missing), ("missing_tree", "/tree".to_owned()));
    }

    const COUPLES: &str = r#"{
        "schemaVersion": 1,
        "type": "tree",
        "title": "Family",
        "width": 900,
        "height": 500,
        "tree": {
            "nodes": [
                {"id": "a", "label": "Ana", "kind": "person"},
                {"id": "t", "label": "Tomas", "kind": "person", "partner": "a"},
                {"id": "l", "label": "Lena", "kind": "person", "parent": "a"},
                {"id": "m", "label": "Marc", "kind": "person", "partner": "l"},
                {"id": "p", "label": "Paul", "kind": "person", "parent": "t"},
                {"id": "s", "label": "Sofia", "parent": "l"},
                {"id": "f", "label": "Foundation", "kind": "external", "parent": "p"}
            ]
        }
    }"#;

    #[test]
    fn a_couple_stands_side_by_side_and_its_children_hang_from_the_middle() {
        for landscape in [false, true] {
            let (plan, boxes) = plan(COUPLES, landscape);
            // ids by position: a 0, t 1, l 2, m 3, p 4, s 5, f 6. Ana and Tomas share a row.
            let (a, t) = (boxes[0], boxes[1]);
            if landscape {
                assert!((a.0 - t.0).abs() < 0.01 && t.1 > a.1 + a.3);
            } else {
                assert!((a.1 - t.1).abs() < 0.01 && t.0 > a.0 + a.2);
            }
            // Lena (a child of Ana) and Paul (a child of Tomas) are both below the couple, and
            // Sofia is below Lena and Marc.
            let level = |node: usize| {
                if landscape {
                    plan.center[node].0
                } else {
                    plan.center[node].1
                }
            };
            assert!(level(2) > level(0) && (level(2) - level(4)).abs() < 0.01);
            assert!(level(5) > level(2) && (level(3) - level(2)).abs() < 0.01);
        }
    }

    #[test]
    fn couples_and_kinds_are_drawn_and_described() {
        let output = svg(COUPLES);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert!(output.content.contains("chartlet-arch-head"));
        assert!(output.content.contains("chartlet-flow-external"));
        let alternative =
            text_alternative(&ChartSpec::from_json(COUPLES).expect("parses")).expect("valid");
        assert!(alternative.description.contains("Root: Ana + Tomas."));
        assert!(
            alternative
                .description
                .contains("Ana + Tomas: Lena + Marc, Paul.")
        );
        assert_eq!(alternative.table.columns[3], "Partner");
    }

    #[test]
    fn invalid_partners_name_the_field() {
        for (from, to, code, path) in [
            (
                r#""partner": "a""#,
                r#""partner": "nobody""#,
                "unknown_node",
                "/tree/nodes/1/partner",
            ),
            (
                r#""partner": "a""#,
                r#""partner": "t""#,
                "invalid_partner",
                "/tree/nodes/1/partner",
            ),
            (
                r#""partner": "l""#,
                r#""partner": "t""#,
                "invalid_partner",
                "/tree/nodes/3/partner",
            ),
            (
                r#""kind": "person", "partner": "a""#,
                r#""kind": "person", "parent": "a", "partner": "a""#,
                "invalid_partner",
                "/tree/nodes/1/partner",
            ),
        ] {
            let (got_code, got_path) = error(&COUPLES.replace(from, to));
            assert_eq!((got_code, got_path.as_str()), (code, path), "{from}");
        }
    }

    #[test]
    fn a_broad_tree_of_long_names_keeps_its_nodes_apart_and_its_text() {
        let mut nodes =
            vec![r#"{"id": "root", "label": "Root of the whole organization"}"#.to_owned()];
        for branch in 0..9 {
            nodes.push(format!(
                r#"{{"id": "b{branch}", "label": "Branch number {branch} of the organization", "parent": "root"}}"#
            ));
            for leaf in 0..14 {
                nodes.push(format!(
                    r#"{{"id": "l{branch}x{leaf}", "label": "Team {leaf} of branch {branch}", "sublabel": "responsible for something long", "parent": "b{branch}"}}"#
                ));
            }
        }
        let json = format!(
            r#"{{"schemaVersion": 1, "type": "tree", "title": "Big", "width": 800, "height": 400, "tree": {{"nodes": [{}]}}}}"#,
            nodes.join(",")
        );
        assert_eq!(nodes.len(), 136);
        for landscape in [false, true] {
            let (_, boxes) = plan(&json, landscape);
            for (index, a) in boxes.iter().enumerate() {
                for b in &boxes[index + 1..] {
                    let apart = a.0 + a.2 <= b.0
                        || b.0 + b.2 <= a.0
                        || a.1 + a.3 <= b.1
                        || b.1 + b.3 <= a.1;
                    assert!(apart, "{a:?} overlaps {b:?}");
                }
            }
        }
        let output = svg(&json);
        assert!(
            output
                .warnings
                .iter()
                .all(|warning| warning.code != "text_truncated"),
            "{:?}",
            output.warnings
        );
    }
}
