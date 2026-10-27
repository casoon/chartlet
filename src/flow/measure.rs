//! What both orientations of a diagram share: its spacing, the layered graph, and every text
//! measured and wrapped into the boxes of the steps and the labels of the edges.

use super::{
    BALL, EDGE_LABEL, EDGE_SIZE, GAP, GROUP_GAP, INITIAL, INTERFACE_GAP, INTERFACE_LINE,
    INTERFACE_PAD, INTERFACE_SIZE, LABEL_SIZE, MARGIN, MAX_NODE, MEMBER_LINE, MEMBER_PAD,
    MEMBER_SIZE, MIN_NODE, NODE_GAP, NODE_LINE, PAD, PASSING_GAP, SUBLABEL_SIZE,
    graph::Graph,
    model::{Diagram, Node, Shape},
    ports::{Across, across_sides, side_room},
    trunks::Trunks,
};
use crate::{
    diagram::wrap,
    error::ChartWarning,
    layout::{NARROW, count},
    metrics::TextMetrics,
    parts,
    spec::{DiagramSpacing, NumberStyle},
    symbols,
};

/// The spacing of a diagram: roomy on a wide canvas, compact below [`NARROW`] pixels, such as a
/// mobile variant, where steps get narrower and wrap their labels sooner, and the space around
/// and between them shrinks.
#[derive(Debug, Clone, Copy)]
pub(super) struct Spacing {
    pub(super) margin: f64,
    min_node: f64,
    max_node: f64,
    pub(super) node_gap: f64,
    pub(super) passing_gap: f64,
    pub(super) group_gap: f64,
    /// The least distance between two layers, and how much it grows when the canvas leaves room.
    pub(super) layer_gap: f64,
    pub(super) max_stretch: f64,
    edge_label: f64,
}

impl Spacing {
    /// Whether this is the compact spacing of a narrow canvas.
    pub(super) fn is_compact(&self) -> bool {
        self.margin < MARGIN
    }

    pub(super) fn new(width: u32, profile: DiagramSpacing) -> Self {
        let spacing = if width < NARROW {
            Self {
                margin: 12.0,
                min_node: 64.0,
                max_node: 120.0,
                node_gap: 16.0,
                passing_gap: 8.0,
                group_gap: 20.0,
                layer_gap: GAP,
                max_stretch: MAX_STRETCH,
                edge_label: 96.0,
            }
        } else {
            Self {
                margin: MARGIN,
                min_node: MIN_NODE,
                max_node: MAX_NODE,
                node_gap: NODE_GAP,
                passing_gap: PASSING_GAP,
                group_gap: GROUP_GAP,
                layer_gap: GAP,
                max_stretch: MAX_STRETCH,
                edge_label: EDGE_LABEL,
            }
        };
        spacing.scaled(profile)
    }

    /// The gaps between layers and between neighbours in a layer, scaled by `profile`: never the
    /// size of steps or texts, the margins or the distance between two tracks, and a compact gap
    /// never below the gaps of a narrow canvas between steps, or 20 between layers. A compact
    /// diagram keeps its gaps on a canvas that leaves room, instead of spreading its layers over
    /// it.
    fn scaled(self, profile: DiagramSpacing) -> Self {
        let (factor, floor) = match profile {
            DiagramSpacing::Normal => return self,
            DiagramSpacing::Compact => (COMPACT, true),
            DiagramSpacing::Roomy => (ROOMY, false),
        };
        let scale = |gap: f64, least: f64| {
            let gap = gap * factor;
            if floor { gap.max(least) } else { gap }
        };
        Self {
            node_gap: scale(self.node_gap, 16.0),
            passing_gap: scale(self.passing_gap, 8.0),
            group_gap: scale(self.group_gap, 20.0),
            layer_gap: scale(self.layer_gap, 20.0),
            max_stretch: if floor { 1.0 } else { self.max_stretch },
            ..self
        }
    }
}

/// How much the compact and the roomy spacing scale the gaps of a diagram.
const COMPACT: f64 = 0.7;
const ROOMY: f64 = 1.5;

/// Growth of the gaps between layers when the canvas leaves room.
const MAX_STRETCH: f64 = 1.6;

/// What both orientations need: the layered graph, and every text measured and wrapped.
pub(super) struct Model<'a> {
    pub(super) diagram: &'a Diagram,
    pub(super) spacing: Spacing,
    pub(super) ends: Vec<(usize, usize)>,
    pub(super) graph: Graph,
    /// Each step's label on one or two lines, and its box: width and height on the page.
    pub(super) labels: Vec<Vec<String>>,
    /// Each step's sublabel on one or two lines.
    pub(super) sublabels: Vec<Vec<String>>,
    pub(super) sizes: Vec<(f64, f64)>,
    /// Each edge's label on one or two lines, and the widest of them.
    pub(super) edge_labels: Vec<Vec<String>>,
    pub(super) edge_widths: Vec<f64>,
    /// The labels of the edges from each step to itself, joined.
    pub(super) loops: Vec<Option<String>>,
    /// The same, on as many lines as the edge labels take.
    pub(super) loop_lines: Vec<Vec<String>>,
    /// How the values of the parts of a step are written.
    pub(super) number_style: NumberStyle,
    /// The edges drawn on shared trunks.
    pub(super) trunks: Trunks,
    /// Per edge, the side across the flow it leaves its first step on and reaches its last on.
    pub(super) across: Vec<(Option<Across>, Option<Across>)>,
    /// Per step, the room the edges that meet it across the flow take on its low and high side.
    pub(super) side_room: Vec<(f64, f64)>,
    /// Per group, the width its frame needs for its name, which sits in the top left corner.
    pub(super) group_names: Vec<f64>,
}

impl<'a> Model<'a> {
    pub(super) fn new(
        diagram: &'a Diagram,
        (spacing, number_style): (Spacing, NumberStyle),
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
    ) -> Self {
        let ends = diagram.ends();
        let graph = Graph::new(diagram);
        let (labels, sublabels, sizes) = node_boxes(diagram, &spacing, metrics, warnings);
        let (edge_labels, edge_widths) =
            edge_labels(diagram, spacing.edge_label, metrics, warnings);
        let loops: Vec<Option<String>> = (0..diagram.nodes.len())
            .map(|node| {
                let labels: Vec<&str> = diagram
                    .edges
                    .iter()
                    .zip(&ends)
                    .filter(|(_, (from, to))| *from == node && *to == node)
                    .map(|(edge, _)| edge.label.as_deref().unwrap_or(""))
                    .collect();
                (!labels.is_empty()).then(|| {
                    labels
                        .into_iter()
                        .filter(|label| !label.is_empty())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
            })
            .collect();
        let loop_lines = loops
            .iter()
            .enumerate()
            .map(|(node, label): (usize, &Option<String>)| {
                let Some(label) = label.as_ref().filter(|label| !label.is_empty()) else {
                    return Vec::new();
                };
                let path = diagram
                    .edges
                    .iter()
                    .find(|edge| edge.from == node && edge.to == node)
                    .map_or("", |edge| edge.label_path.as_str());
                wrap(
                    label,
                    spacing.edge_label,
                    EDGE_SIZE,
                    metrics,
                    warnings,
                    path,
                )
            })
            .collect();
        let across = across_sides(diagram, &graph, warnings);
        let side_room = side_room(diagram.nodes.len(), &graph, &across, spacing.passing_gap);
        let group_names = diagram
            .groups
            .iter()
            .map(|group| (metrics.width(&group.label, SUBLABEL_SIZE) + 16.0).ceil())
            .collect();
        Self {
            diagram,
            spacing,
            ends,
            graph,
            labels,
            sublabels,
            sizes,
            edge_labels,
            edge_widths,
            loops,
            loop_lines,
            number_style,
            trunks: Trunks::none(diagram.edges.len()),
            across,
            side_room,
            group_names,
        }
    }
}

/// Each step's label and sublabel on one or two lines, and its box: width and height.
#[allow(clippy::type_complexity, clippy::too_many_lines)]
fn node_boxes(
    diagram: &Diagram,
    spacing: &Spacing,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> (Vec<Vec<String>>, Vec<Vec<String>>, Vec<(f64, f64)>) {
    let mut labels = Vec::with_capacity(diagram.nodes.len());
    let mut sublabels = Vec::with_capacity(diagram.nodes.len());
    let mut sizes = Vec::with_capacity(diagram.nodes.len());
    for (index, node) in diagram.nodes.iter().enumerate() {
        if node.shape == Shape::Block {
            let block = diagram
                .blocks
                .iter()
                .find(|block| block.node == index)
                .expect("a block node has its block");
            labels.push(Vec::new());
            sublabels.push(Vec::new());
            sizes.push(block.size);
            continue;
        }
        if node.shape == Shape::Initial {
            labels.push(Vec::new());
            sublabels.push(Vec::new());
            sizes.push((INITIAL, INITIAL));
            continue;
        }
        if matches!(node.shape, Shape::Note | Shape::Interface) {
            let (lines, size) = if node.shape == Shape::Note {
                note_box(&node.label, &node.path, spacing, metrics, warnings)
            } else {
                interface_box(node, spacing, metrics, warnings)
            };
            labels.push(lines);
            sublabels.push(Vec::new());
            sizes.push(size);
            continue;
        }
        let (extra_width, extra_height) = match node.shape {
            Shape::Decision => (48.0, 24.0),
            Shape::Io => (20.0, 0.0),
            Shape::Subprocess => (16.0, 0.0),
            Shape::Store => (0.0, 10.0),
            Shape::Final => (8.0, 8.0),
            Shape::Frontend => (0.0, 12.0),
            Shape::Bucket => (16.0, 4.0),
            Shape::Cache => (24.0, 0.0),
            Shape::Shield | Shape::Person => (0.0, 18.0),
            Shape::Node => (8.0, 6.0),
            Shape::Artifact => (8.0, 0.0),
            _ => (0.0, 0.0),
        };
        // Never narrower than the longest word, which cannot wrap.
        let word = node
            .label
            .split(' ')
            .map(|word| metrics.width(word, LABEL_SIZE))
            .fold(0.0, f64::max);
        let room = (spacing.max_node - 2.0 * PAD).max(word);
        let path = format!("{}/label", node.path);
        let lines = wrap(&node.label, room, LABEL_SIZE, metrics, warnings, &path);
        let sublines = node.sublabel.as_ref().map_or_else(Vec::new, |sublabel| {
            let path = format!("{}/sublabel", node.path);
            wrap(sublabel, room, SUBLABEL_SIZE, metrics, warnings, &path)
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
        if node.shape == Shape::Class {
            // Members are never wrapped: the box is as wide as its widest line.
            let members = node
                .compartments
                .iter()
                .flatten()
                .map(|line| metrics.width(line, MEMBER_SIZE))
                .fold(0.0, f64::max);
            let width = (text.max(members) + 2.0 * PAD).max(spacing.min_node);
            let height = class_header(lines.len(), sublines.len())
                + class_compartments(&node.compartments).iter().sum::<f64>();
            labels.push(lines);
            sublabels.push(sublines);
            sizes.push((width, height));
            continue;
        }
        let width = (text + 2.0 * PAD)
            .clamp(spacing.min_node, spacing.max_node.max(room + 2.0 * PAD))
            + extra_width;
        let stripe = if node.has_stripe() { parts::ROOM } else { 0.0 };
        let symbol = if node.symbol.is_some() {
            symbols::ROOM
        } else {
            0.0
        };
        let height = 20.0
            + NODE_LINE * count(lines.len())
            + 14.0 * count(sublines.len())
            + extra_height
            + stripe
            + symbol;
        labels.push(lines);
        sublabels.push(sublines);
        sizes.push((width, height));
    }
    (labels, sublabels, sizes)
}

/// The name of an interface on one or two lines, and its box in portrait: the ball in the
/// middle, the name on one side and as much room on the other.
fn interface_box(
    node: &Node,
    spacing: &Spacing,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> (Vec<String>, (f64, f64)) {
    let path = format!("{}/label", node.path);
    let lines = wrap(
        &node.label,
        spacing.min_node,
        INTERFACE_SIZE,
        metrics,
        warnings,
        &path,
    );
    let text = lines
        .iter()
        .map(|line| metrics.width(line, INTERFACE_SIZE))
        .fold(0.0, f64::max);
    let size = if spacing.is_compact() {
        // On a narrow canvas the name stands below the ball, on a chip over the stem.
        (
            (text + 2.0 * INTERFACE_PAD).max(2.0 * BALL),
            2.0 * (BALL + INTERFACE_PAD + INTERFACE_LINE * count(lines.len())),
        )
    } else {
        (
            2.0 * (BALL + INTERFACE_GAP + text),
            (2.0 * BALL).max(INTERFACE_LINE * count(lines.len())),
        )
    };
    (lines, size)
}

/// The text of a note, its padding above and below, and the most lines it takes before the rest
/// is shortened.
const NOTE_SIZE: f64 = 12.0;
pub(super) const NOTE_PAD: f64 = 8.0;
const MAX_NOTE_LINES: usize = 10;

/// The lines of a note and its box: width and height.
pub(super) fn note_box(
    text: &str,
    path: &str,
    spacing: &Spacing,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> (Vec<String>, (f64, f64)) {
    let path = format!("{path}/text");
    let lines = note_lines(text, spacing.max_node - 2.0 * PAD, metrics, warnings, &path);
    let text = lines
        .iter()
        .map(|line| metrics.width(line, NOTE_SIZE))
        .fold(0.0, f64::max);
    let width = (text + 2.0 * PAD).max(spacing.min_node / 2.0);
    let height = 2.0 * NOTE_PAD + MEMBER_LINE * count(lines.len());
    (lines, (width, height))
}

/// The text of a note on as many lines of at most `room` as it needs, word by word; past
/// [`MAX_NOTE_LINES`] the last line is shortened, as is a word wider than a line, with a warning.
fn note_lines(
    text: &str,
    room: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let longer = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if line.is_empty() || metrics.width(&longer, NOTE_SIZE) <= room {
            line = longer;
        } else {
            lines.push(std::mem::replace(&mut line, word.to_owned()));
        }
    }
    lines.push(line);
    if lines.len() > MAX_NOTE_LINES {
        let rest = lines.split_off(MAX_NOTE_LINES - 1).join(" ");
        lines.push(rest);
    }
    lines
        .iter()
        .map(|line| crate::layout::fit_text(line, room, NOTE_SIZE, metrics, warnings, path))
        .collect()
}

/// The height of the top of a class box: its stereotype lines above its name lines.
pub(super) fn class_header(name_lines: usize, stereotype_lines: usize) -> f64 {
    16.0 + NODE_LINE * count(name_lines) + 14.0 * count(stereotype_lines)
}

/// The height of every compartment of a class box below its name; one without lines is left out.
fn class_compartments(compartments: &[Vec<String>]) -> Vec<f64> {
    compartments
        .iter()
        .filter(|lines| !lines.is_empty())
        .map(|lines| 2.0 * MEMBER_PAD + MEMBER_LINE * count(lines.len()))
        .collect()
}

/// Each edge's label on one or two lines with its technology in brackets below, and the widest
/// line of each.
fn edge_labels(
    diagram: &Diagram,
    widest: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> (Vec<Vec<String>>, Vec<f64>) {
    let mut edge_labels = Vec::with_capacity(diagram.edges.len());
    let mut edge_widths = Vec::with_capacity(diagram.edges.len());
    for edge in &diagram.edges {
        let mut lines = edge.label.as_ref().map_or_else(Vec::new, |label| {
            let word = label
                .split(' ')
                .map(|word| metrics.width(word, EDGE_SIZE))
                .fold(0.0, f64::max);
            wrap(
                label,
                widest.max(word),
                EDGE_SIZE,
                metrics,
                warnings,
                &edge.label_path,
            )
        });
        if let Some(technology) = &edge.technology {
            // A technology cannot wrap: it may run as wide as a label does on a wide canvas.
            lines.push(crate::layout::fit_text(
                &format!("[{technology}]"),
                widest.max(EDGE_LABEL),
                EDGE_SIZE,
                metrics,
                warnings,
                &edge.label_path.replace("/label", "/technology"),
            ));
        }
        edge_widths.push(
            lines
                .iter()
                .map(|line| metrics.width(line, EDGE_SIZE))
                .fold(0.0, f64::max),
        );
        edge_labels.push(lines);
    }
    (edge_labels, edge_widths)
}
