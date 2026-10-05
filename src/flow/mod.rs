//! Flow charts: steps joined by arrows, laid out in layers along the direction of the flow —
//! down in portrait, right in landscape — optionally in lanes across it and framed in groups.
//!
//! Positions are worked out along two axes, the main axis of the flow and the cross axis of the
//! layers, and turned into page coordinates only when drawn, so that both orientations share one
//! layout. Edges leave a step on its far side, change course on a track of their own between
//! two layers, and arrive on the near side of the next; an edge the layout turned around to break
//! a cycle is drawn back the other way.

mod graph;

use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::{
        self, CHIP_LINE, CHIP_REACH, HEAD, arrowhead, chip, chip_box, cylinder, pixels, rounded,
        warn_growth, with_shadow, wrap,
    },
    error::ChartWarning,
    layout::{NARROW, count, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Circle, Element, Hotspot, Line, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, Dash, DiagramOrientation, FlowSpec, NodeKind},
    text,
};
use graph::{Graph, settle};

const MARGIN: f64 = 24.0;
const BOTTOM: f64 = 16.0;
const LABEL_SIZE: f64 = 13.0;
const SUBLABEL_SIZE: f64 = 11.0;
const EDGE_SIZE: f64 = 12.0;
/// Distance between the baselines of an edge label on two lines, and of a step's label.
const LINE: f64 = 15.0;
const NODE_LINE: f64 = 16.0;
/// Text inset inside a step.
const PAD: f64 = 12.0;
const MIN_NODE: f64 = 104.0;
const MAX_NODE: f64 = 200.0;
/// The widest edge label before it wraps.
const EDGE_LABEL: f64 = 150.0;
/// Space between neighbours in a layer: two steps, a step and a passing edge, and the extra
/// around a group.
const NODE_GAP: f64 = 28.0;
const PASSING_GAP: f64 = 14.0;
const GROUP_GAP: f64 = 30.0;
/// The least distance between two layers, and between two tracks in it.
const GAP: f64 = 28.0;
const TRACK: f64 = 10.0;
/// A group's frame around its steps, and the room above them for its name.
const GROUP_PAD: f64 = 12.0;
const GROUP_HEAD: f64 = 16.0;
/// Inset of the steps from the sides of their lane, and the room for a lane's name.
const LANE_PAD: f64 = 16.0;
const LANE_HEAD: f64 = 26.0;
/// The diameter of a state machine's initial node.
const INITIAL: f64 = 20.0;
/// How far the loop of an edge from a step to itself reaches out.
const LOOP: f64 = 22.0;
/// The spacing of a diagram: roomy on a wide canvas, compact below [`NARROW`] pixels, such as a
/// mobile variant, where steps get narrower and wrap their labels sooner, and the space around
/// and between them shrinks.
#[derive(Debug, Clone, Copy)]
struct Spacing {
    margin: f64,
    min_node: f64,
    max_node: f64,
    node_gap: f64,
    passing_gap: f64,
    group_gap: f64,
    edge_label: f64,
}

impl Spacing {
    const fn for_width(width: u32) -> Self {
        if width < NARROW {
            Self {
                margin: 12.0,
                min_node: 64.0,
                max_node: 120.0,
                node_gap: 16.0,
                passing_gap: 8.0,
                group_gap: 20.0,
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
                edge_label: EDGE_LABEL,
            }
        }
    }
}

/// Growth of the gaps between layers when the canvas leaves room.
const MAX_STRETCH: f64 = 1.6;

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    layout_diagram(spec, &Diagram::from_flow(flow(spec)), warnings, metrics)
}

/// Lays out and draws `diagram`, whatever specification it comes from.
pub(crate) fn layout_diagram(
    spec: &ChartSpec,
    diagram: &Diagram,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let spacing = Spacing::for_width(spec.width);
    let margin = spacing.margin;
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let model = Model::new(diagram, spacing, metrics, warnings);
    let landscape = match diagram.orientation {
        DiagramOrientation::Portrait => false,
        DiagramOrientation::Landscape => true,
        DiagramOrientation::Auto => {
            let portrait = Plan::new(spec, &model, false, top, metrics);
            let landscape = Plan::new(spec, &model, true, top, metrics);
            diagram::prefers_landscape(
                spec,
                (portrait.width, portrait.height),
                (landscape.width, landscape.height),
            )
        }
    };
    let plan = Plan::new(spec, &model, landscape, top, metrics);
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
    plan.draw(&model, metrics, warnings, &mut elements);
    Scene {
        width: pixels(plan.width),
        height: pixels(plan.height),
        elements,
    }
}

fn flow(spec: &ChartSpec) -> &FlowSpec {
    spec.flow
        .as_ref()
        .expect("validated flow charts carry a flow block")
}

/// A diagram for the layered layout, whatever specification it comes from: steps with a shape,
/// edges between them, and the lanes and groups the steps name by index.
pub(crate) struct Diagram {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub lanes: Vec<String>,
    pub groups: Vec<Group>,
    pub orientation: DiagramOrientation,
}

pub(crate) struct Node {
    pub label: String,
    pub sublabel: Option<String>,
    pub shape: Shape,
    pub lane: Option<usize>,
    pub group: Option<usize>,
    /// Where the node stands in the specification, for warnings about its texts.
    pub path: String,
}

pub(crate) struct Edge {
    pub from: usize,
    pub to: usize,
    pub label: Option<String>,
    /// How the connection is made, written in brackets on a line below the label.
    pub technology: Option<String>,
    pub dash: Option<Dash>,
    /// Whether the edge belongs to the main path.
    pub main: bool,
    /// Where the edge's label stands in the specification, for warnings about it.
    pub label_path: String,
}

pub(crate) struct Group {
    pub label: String,
    pub path: String,
    /// The group this one lies in.
    pub parent: Option<usize>,
}

/// How a node is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shape {
    Start,
    End,
    Process,
    Decision,
    Io,
    Subprocess,
    Store,
    External,
    /// Where a state machine begins: a small filled circle without a label inside.
    Initial,
    /// A state: a box with well rounded corners.
    State,
    /// A final state: a state with a double outline.
    Final,
    /// Someone who uses a system: a box with a head on top.
    Person,
    /// What a person sees: a box with a window bar.
    Frontend,
    /// A queue: a box with a stack behind it.
    Queue,
    /// File or object storage: a bucket, wider at the top.
    Bucket,
    /// A cache: a hexagon.
    Cache,
    /// A security component: a shield.
    Shield,
}

impl From<NodeKind> for Shape {
    fn from(kind: NodeKind) -> Self {
        match kind {
            NodeKind::Start => Self::Start,
            NodeKind::End => Self::End,
            NodeKind::Process => Self::Process,
            NodeKind::Decision => Self::Decision,
            NodeKind::Io => Self::Io,
            NodeKind::Subprocess => Self::Subprocess,
            NodeKind::Store => Self::Store,
            NodeKind::External => Self::External,
        }
    }
}

impl Diagram {
    pub(crate) fn from_flow(flow: &FlowSpec) -> Self {
        let ends = flow.ends();
        Self {
            nodes: flow
                .nodes
                .iter()
                .enumerate()
                .map(|(index, node)| Node {
                    label: node.label.clone(),
                    sublabel: node.sublabel.clone(),
                    shape: node.kind.into(),
                    lane: flow.lane_of(index),
                    group: flow.group_of(index),
                    path: format!("/flow/nodes/{index}"),
                })
                .collect(),
            edges: flow
                .edges
                .iter()
                .zip(ends)
                .enumerate()
                .map(|(index, (edge, (from, to)))| Edge {
                    from,
                    to,
                    label: edge.label.clone(),
                    technology: None,
                    dash: edge.dash,
                    main: flow.on_main_path(index),
                    label_path: format!("/flow/edges/{index}/label"),
                })
                .collect(),
            lanes: flow.lanes.iter().map(|lane| lane.label.clone()).collect(),
            groups: flow
                .groups
                .iter()
                .enumerate()
                .map(|(index, group)| Group {
                    label: group.label.clone(),
                    path: format!("/flow/groups/{index}"),
                    parent: None,
                })
                .collect(),
            orientation: flow.orientation,
        }
    }

    /// The groups around `group`, outermost first, ending with `group` itself.
    pub(crate) fn chain(&self, group: Option<usize>) -> Vec<usize> {
        let mut chain = Vec::new();
        let mut at = group;
        while let Some(group) = at {
            chain.push(group);
            at = self.groups[group].parent;
        }
        chain.reverse();
        chain
    }

    /// Whether node `node` lies in group `group`, directly or in a group inside it.
    pub(crate) fn inside(&self, node: usize, group: usize) -> bool {
        self.chain(self.nodes[node].group).contains(&group)
    }

    fn ends(&self) -> Vec<(usize, usize)> {
        self.edges.iter().map(|edge| (edge.from, edge.to)).collect()
    }
}

/// What both orientations need: the layered graph, and every text measured and wrapped.
struct Model<'a> {
    diagram: &'a Diagram,
    spacing: Spacing,
    ends: Vec<(usize, usize)>,
    graph: Graph,
    /// Each step's label on one or two lines, and its box: width and height on the page.
    labels: Vec<Vec<String>>,
    /// Each step's sublabel on one or two lines.
    sublabels: Vec<Vec<String>>,
    sizes: Vec<(f64, f64)>,
    /// Each edge's label on one or two lines, and the widest of them.
    edge_labels: Vec<Vec<String>>,
    edge_widths: Vec<f64>,
    /// The labels of the edges from each step to itself, joined.
    loops: Vec<Option<String>>,
    /// The same, on as many lines as the edge labels take.
    loop_lines: Vec<Vec<String>>,
}

impl<'a> Model<'a> {
    fn new(
        diagram: &'a Diagram,
        spacing: Spacing,
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
        }
    }
}

/// Each step's label and sublabel on one or two lines, and its box: width and height.
#[allow(clippy::type_complexity)]
fn node_boxes(
    diagram: &Diagram,
    spacing: &Spacing,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> (Vec<Vec<String>>, Vec<Vec<String>>, Vec<(f64, f64)>) {
    let mut labels = Vec::with_capacity(diagram.nodes.len());
    let mut sublabels = Vec::with_capacity(diagram.nodes.len());
    let mut sizes = Vec::with_capacity(diagram.nodes.len());
    for node in &diagram.nodes {
        if node.shape == Shape::Initial {
            labels.push(Vec::new());
            sublabels.push(Vec::new());
            sizes.push((INITIAL, INITIAL));
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
        let width = (text + 2.0 * PAD)
            .clamp(spacing.min_node, spacing.max_node.max(room + 2.0 * PAD))
            + extra_width;
        let height =
            20.0 + NODE_LINE * count(lines.len()) + 14.0 * count(sublines.len()) + extra_height;
        labels.push(lines);
        sublabels.push(sublines);
        sizes.push((width, height));
    }
    (labels, sublabels, sizes)
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

/// An edge label waiting to be drawn: its lines, the first baseline and how it is anchored.
struct Label {
    lines: Vec<String>,
    /// The focus classes of the edge it labels.
    classes: String,
    at: (f64, f64),
    anchor: TextAnchor,
    /// Where the line it labels runs across the page, when the label stands beside it and may
    /// move to its other side.
    beside: Option<f64>,
}

impl Label {
    /// Moves a label that would run off the side of the page to the other side of its line, or,
    /// beside nothing, back onto the page.
    fn keep_on_page(&mut self, width: f64, metrics: &impl TextMetrics) {
        let (left, _, chip_width, _) = chip_box(&self.lines, self.at, self.anchor, metrics);
        let reach = CHIP_REACH + 2.0;
        let over = left + chip_width - (width - 2.0);
        if over > 0.0 {
            match (self.beside, self.anchor) {
                (Some(line), TextAnchor::Start) => {
                    self.at.0 = line - reach;
                    self.anchor = TextAnchor::End;
                }
                _ => self.at.0 -= over,
            }
        } else if left < 2.0 {
            match (self.beside, self.anchor) {
                (Some(line), TextAnchor::End) => {
                    self.at.0 = line + reach;
                    self.anchor = TextAnchor::Start;
                }
                _ => self.at.0 += 2.0 - left,
            }
        }
    }
}

impl Model<'_> {
    /// The focus classes of a step: the step itself and every step it shares an edge with.
    fn node_classes(&self, node: usize) -> String {
        let mut related: Vec<usize> = self
            .ends
            .iter()
            .filter_map(|&(from, to)| match (from == node, to == node) {
                (true, _) => Some(to),
                (_, true) => Some(from),
                _ => None,
            })
            .chain(std::iter::once(node))
            .collect();
        related.sort_unstable();
        related.dedup();
        focus_classes(Some(node), &related)
    }

    /// The focus classes of an edge: the steps at both its ends.
    fn edge_classes(&self, edge: usize) -> String {
        let (from, to) = self.ends[edge];
        focus_classes(None, &[from.min(to), from.max(to)])
    }
}

/// The classes that tie a drawn part to the nodes whose focus brings it forward, and to its own
/// node when it is one: `chartlet-f chartlet-n-3 chartlet-f-1 chartlet-f-3`.
pub(crate) fn focus_classes(own: Option<usize>, related: &[usize]) -> String {
    let mut classes = String::from("chartlet-f");
    if let Some(own) = own {
        write!(classes, " chartlet-n-{own}").expect("writing to String cannot fail");
    }
    let mut last = None;
    for node in related {
        if last != Some(*node) {
            write!(classes, " chartlet-f-{node}").expect("writing to String cannot fail");
        }
        last = Some(*node);
    }
    classes
}

/// An edge's way on the main and cross axes, and where its label goes.
struct Route {
    points: Vec<(f64, f64)>,
    /// Where the label is written: where the edge leaves the step it comes from.
    label_at: (f64, f64),
    /// Whether the edge leaves that step against the direction of the flow: an edge the layout
    /// turned around, whose label is then written before the step instead of after it.
    against: bool,
    /// Whether the label goes on the side of larger cross positions: the side of its port away
    /// from the middle of the step, so that it keeps clear of the step's other edges.
    after: bool,
    /// How far the label moves further to that side, past the other edges that meet the same
    /// side of the step there.
    clear: f64,
}

/// One orientation of the layout, in main and cross coordinates.
struct Plan {
    landscape: bool,
    /// Each step's box on the page: width and height.
    sizes: Vec<(f64, f64)>,
    /// The cross position of every item.
    cross: Vec<f64>,
    /// The start and end of every layer on the main axis.
    layers: Vec<(f64, f64)>,
    /// The lanes on the cross axis.
    bands: Vec<(f64, f64)>,
    /// One route per edge; `None` for an edge from a step to itself.
    routes: Vec<Option<Route>>,
    /// How far the layers reach on the main axis, and the content on the cross axis.
    main_length: f64,
    cross_length: f64,
    /// Where the main and cross axes begin on the page.
    main_origin: f64,
    cross_origin: f64,
    width: f64,
    height: f64,
}

impl Plan {
    fn new(
        spec: &ChartSpec,
        model: &Model,
        landscape: bool,
        top: f64,
        metrics: &impl TextMetrics,
    ) -> Self {
        let graph = &model.graph;
        let diagram = model.diagram;
        let sizes = step_sizes(model, landscape);
        // A step's size across the layers and along the flow.
        let across = |node: usize| {
            if landscape {
                sizes[node].1
            } else {
                sizes[node].0
            }
        };
        let cross_size = |item: usize| graph.items[item].node.map_or(0.0, across);
        let main_size = |node: usize| {
            if landscape {
                sizes[node].0
            } else {
                sizes[node].1
            }
        };
        let loop_room = loop_room(model, landscape, metrics);
        let after = |item: usize| graph.items[item].node.map_or(0.0, |node| loop_room[node]);
        let gap = |a: usize, b: usize| {
            let (first, second) = (&graph.items[a], &graph.items[b]);
            let base = if first.node.is_some() && second.node.is_some() {
                model.spacing.node_gap
            } else {
                model.spacing.passing_gap
            };
            // Room for every frame between the two.
            let frames = count(first.frames_between(second));
            cross_size(a) / 2.0
                + cross_size(b) / 2.0
                + base
                + after(a)
                + model.spacing.group_gap * frames
        };
        let lane_head = if landscape { LANE_HEAD } else { 0.0 };
        let bands = lane_bands(
            diagram,
            graph,
            &cross_size,
            &gap,
            &after,
            lane_head,
            metrics,
        );
        let cross = place(graph, &cross_size, &gap, &bands, lane_head);
        let cross_length = if bands.is_empty() {
            content_length(graph, &cross, &cross_size, &after)
        } else {
            bands.last().map_or(0.0, |band| band.1)
        };
        let (main_room, cross_room) = rooms(spec, landscape, top, model.spacing.margin);
        let mut plan = Self {
            landscape,
            sizes: sizes.clone(),
            cross,
            layers: Vec::new(),
            bands,
            routes: Vec::new(),
            main_length: 0.0,
            cross_length,
            main_origin: 0.0,
            cross_origin: 0.0,
            width: 0.0,
            height: 0.0,
        };
        let cross_needed = plan.cross_length;
        if !plan.bands.is_empty() && cross_room > plan.cross_length {
            plan.widen_lanes(graph, cross_room);
        }
        straighten(
            model,
            &mut plan.cross,
            &cross_size,
            &gap,
            &plan.bands,
            lane_head,
        );
        let cross_needed = if plan.bands.is_empty() && !diagram.groups.is_empty() {
            plan.keep_frames_clear(model, &cross_size, &gap, &after)
        } else {
            cross_needed
        };
        let ports = Ports::new(graph, &plan.cross, &cross_size, model, landscape);
        let ends = lane_ends(diagram, landscape);
        let gaps = Gaps::new(model, &plan, &ports, landscape);
        let natural = gaps.length(&main_size, graph, ends);
        let stretch = gaps.stretch(natural, main_room);
        plan.main_length = gaps.place(&mut plan, &main_size, graph, ends, stretch);
        plan.routes = route(model, &plan, &ports, &gaps, stretch);
        let rooms = (main_room, cross_room);
        plan.finish(
            spec,
            (top, model.spacing.margin),
            rooms,
            (natural, cross_needed),
        );
        plan
    }

    /// Places the layout on the page, in the middle of the room the canvas leaves, and works out
    /// the size of the canvas: as given, or larger where the diagram needs it.
    fn finish(
        &mut self,
        spec: &ChartSpec,
        (top, margin): (f64, f64),
        (main_room, cross_room): (f64, f64),
        (main_needed, cross_needed): (f64, f64),
    ) {
        let (page_width, page_height) = (f64::from(spec.width), f64::from(spec.height));
        let landscape = self.landscape;
        self.main_origin = (if landscape { margin } else { top })
            + ((main_room - self.main_length) / 2.0).max(0.0);
        self.cross_origin = (if landscape { top } else { margin })
            + if self.bands.is_empty() {
                ((cross_room - self.cross_length) / 2.0).max(0.0)
            } else {
                0.0
            };
        (self.width, self.height) = if landscape {
            (
                page_width.max(2.0 * margin + main_needed),
                page_height.max(top + cross_needed + BOTTOM),
            )
        } else {
            (
                page_width.max(2.0 * margin + cross_needed),
                page_height.max(top + main_needed + BOTTOM),
            )
        };
    }

    /// Keeps steps out of the frames they do not belong to, the main path straight, and returns
    /// how far the content then reaches on the cross axis. Clearing a frame can bend the main path
    /// again, and straightening it can move a step back into a frame: clear, straighten, clear.
    fn keep_frames_clear(
        &mut self,
        model: &Model,
        cross_size: &impl Fn(usize) -> f64,
        gap: &impl Fn(usize, usize) -> f64,
        after: &impl Fn(usize) -> f64,
    ) -> f64 {
        let landscape = self.landscape;
        clear_frames(model, &mut self.cross, cross_size, landscape);
        straighten(model, &mut self.cross, cross_size, gap, &self.bands, 0.0);
        clear_frames(model, &mut self.cross, cross_size, landscape);
        self.cross_length = content_length(&model.graph, &self.cross, cross_size, after)
            .max(frames_reach(model, &self.cross, cross_size, landscape).1);
        self.cross_length
    }

    /// Spreads the lanes over `room` on the cross axis, each its share wider, its steps and
    /// passing edges moved to its middle.
    fn widen_lanes(&mut self, graph: &Graph, room: f64) {
        let extra = (room - self.cross_length) / count(self.bands.len());
        for (item, cross) in self.cross.iter_mut().enumerate() {
            let lane = graph.items[item].lane.unwrap_or(0);
            *cross += extra * (count(lane) + 0.5);
        }
        for (lane, band) in self.bands.iter_mut().enumerate() {
            band.0 += extra * count(lane);
            band.1 += extra * count(lane + 1);
        }
        self.cross_length = room;
    }

    /// A main and a cross position as a point on the page.
    fn page(&self, (main, cross): (f64, f64)) -> (f64, f64) {
        if self.landscape {
            (self.main_origin + main, self.cross_origin + cross)
        } else {
            (self.cross_origin + cross, self.main_origin + main)
        }
    }

    /// The box of step `node` on the page: left, top, width, height.
    fn node_box(&self, model: &Model, node: usize) -> (f64, f64, f64, f64) {
        let layer = self.layers[model.graph.layer[node]];
        let middle = f64::midpoint(layer.0, layer.1);
        let (x, y) = self.page((middle, self.cross[node]));
        let (width, height) = self.sizes[node];
        (x - width / 2.0, y - height / 2.0, width, height)
    }
}

/// How far the content reaches on the cross axis, loops beside steps included.
fn content_length(
    graph: &Graph,
    cross: &[f64],
    cross_size: &impl Fn(usize) -> f64,
    after: &impl Fn(usize) -> f64,
) -> f64 {
    (0..graph.items.len())
        .map(|item| cross[item] + cross_size(item) / 2.0 + after(item))
        .fold(0.0, f64::max)
}

/// The room the canvas leaves the layout on the main and the cross axis.
fn rooms(spec: &ChartSpec, landscape: bool, top: f64, margin: f64) -> (f64, f64) {
    let across = f64::from(spec.width) - 2.0 * margin;
    let down = f64::from(spec.height) - top - BOTTOM;
    if landscape {
        (across, down)
    } else {
        (down, across)
    }
}

/// Each step's box on the page. In landscape, labelled edges leave a step one below the other,
/// and the step grows to keep their labels apart.
fn step_sizes(model: &Model, landscape: bool) -> Vec<(f64, f64)> {
    let mut sizes = model.sizes.clone();
    if landscape {
        for (node, size) in sizes.iter_mut().enumerate() {
            let labelled = model
                .graph
                .chains
                .iter()
                .zip(&model.edge_labels)
                .filter(|(chain, lines)| {
                    !lines.is_empty() && chain.as_ref().is_some_and(|chain| chain.items[0] == node)
                })
                .count();
            size.1 = size.1.max(18.0 * count(labelled) + 14.0);
        }
    }
    sizes
}

/// What a loop from a step to itself takes beside the step on the cross axis.
fn loop_room(model: &Model, landscape: bool, metrics: &impl TextMetrics) -> Vec<f64> {
    model
        .loops
        .iter()
        .zip(&model.loop_lines)
        .map(|(label, lines)| match label {
            None => 0.0,
            Some(_) if landscape => LOOP + 8.0 + CHIP_LINE * count(lines.len().max(1)),
            Some(_) => {
                LOOP + 2.0 * CHIP_REACH
                    + lines
                        .iter()
                        .map(|line| metrics.width(line, EDGE_SIZE))
                        .fold(0.0, f64::max)
            }
        })
        .collect()
}

/// The room lanes take on the main axis before the first layer and after the last: a little
/// air, and in portrait the strip with their names ahead of it.
fn lane_ends(diagram: &Diagram, landscape: bool) -> (f64, f64) {
    if diagram.lanes.is_empty() {
        (0.0, 0.0)
    } else if landscape {
        (LANE_PAD, LANE_PAD)
    } else {
        (LANE_HEAD + LANE_PAD, LANE_PAD)
    }
}

/// The lanes on the cross axis: each as wide as its widest layer needs, and as its name.
fn lane_bands(
    diagram: &Diagram,
    graph: &Graph,
    cross_size: &impl Fn(usize) -> f64,
    gap: &impl Fn(usize, usize) -> f64,
    after: &impl Fn(usize) -> f64,
    lane_head: f64,
    metrics: &impl TextMetrics,
) -> Vec<(f64, f64)> {
    let mut widths: Vec<f64> = diagram
        .lanes
        .iter()
        .map(|lane| {
            if lane_head > 0.0 {
                0.0
            } else {
                metrics.width(lane, SUBLABEL_SIZE) + 2.0 * LANE_PAD
            }
        })
        .collect();
    if widths.is_empty() {
        return Vec::new();
    }
    for layer in &graph.layers {
        for segment in segments(graph, layer) {
            let span = cross_size(segment[0]) / 2.0
                + segment
                    .windows(2)
                    .map(|pair| gap(pair[0], pair[1]))
                    .sum::<f64>()
                + cross_size(segment[segment.len() - 1]) / 2.0
                + after(segment[segment.len() - 1]);
            let lane = graph.items[segment[0]].lane.unwrap_or(0);
            widths[lane] = widths[lane].max(span + 2.0 * LANE_PAD + lane_head);
        }
    }
    let mut start = 0.0;
    widths
        .into_iter()
        .map(|width| {
            let band = (start, start + width);
            start += width;
            band
        })
        .collect()
}

/// The runs of a layer's items that share a lane.
fn segments<'a>(graph: &'a Graph, layer: &'a [usize]) -> impl Iterator<Item = &'a [usize]> {
    layer.chunk_by(move |a, b| graph.items[*a].lane == graph.items[*b].lane)
}

/// How often the layers are settled against their neighbours, alternately down and up.
const SWEEPS: usize = 10;

/// The cross position of every item: each layer settled against its neighbours, sweeping down
/// and up the layers, inside its lanes. Without lanes, the content starts at 0.
fn place(
    graph: &Graph,
    cross_size: &impl Fn(usize) -> f64,
    gap: &impl Fn(usize, usize) -> f64,
    bands: &[(f64, f64)],
    lane_head: f64,
) -> Vec<f64> {
    let mut cross = vec![0.0; graph.items.len()];
    let bounds = |segment: &[usize]| {
        graph.items[segment[0]].lane.map(|lane| {
            let band = bands[lane];
            (
                band.0 + LANE_PAD + lane_head + cross_size(segment[0]) / 2.0,
                band.1 - LANE_PAD - cross_size(segment[segment.len() - 1]) / 2.0,
            )
        })
    };
    let gaps = |segment: &[usize]| -> Vec<f64> {
        std::iter::once(0.0)
            .chain(segment.windows(2).map(|pair| gap(pair[0], pair[1])))
            .collect()
    };
    // Start packed in the middle of each lane, or around 0.
    for layer in &graph.layers {
        for segment in segments(graph, layer) {
            let gaps = gaps(segment);
            let span: f64 = gaps.iter().sum();
            let middle = bounds(segment).map_or(0.0, |(low, high)| f64::midpoint(low, high));
            let mut at = middle - span / 2.0;
            for (item, gap) in segment.iter().zip(&gaps) {
                at += gap;
                cross[*item] = at;
            }
        }
    }
    for sweep in 0..SWEEPS {
        let (down, both) = (sweep % 2 == 0, sweep == SWEEPS - 1);
        let order: Vec<usize> = if down {
            (0..graph.layers.len()).collect()
        } else {
            (0..graph.layers.len()).rev().collect()
        };
        for layer in order {
            for segment in segments(graph, &graph.layers[layer]) {
                let (targets, weights): (Vec<f64>, Vec<f64>) = segment
                    .iter()
                    .map(|item| {
                        let neighbours: Vec<&(usize, f64)> = match (both, down) {
                            (true, _) => graph.before[*item]
                                .iter()
                                .chain(&graph.after[*item])
                                .collect(),
                            (false, true) => graph.before[*item].iter().collect(),
                            (false, false) => graph.after[*item].iter().collect(),
                        };
                        let total: f64 = neighbours.iter().map(|(_, weight)| weight).sum();
                        if total == 0.0 {
                            (cross[*item], 0.25)
                        } else {
                            let sum: f64 = neighbours
                                .iter()
                                .map(|(other, weight)| cross[*other] * weight)
                                .sum();
                            (sum / total, total)
                        }
                    })
                    .unzip();
                let settled = settle(&targets, &weights, &gaps(segment), bounds(segment));
                for (item, position) in segment.iter().zip(settled) {
                    cross[*item] = position;
                }
            }
        }
    }
    if bands.is_empty() {
        let low = (0..graph.items.len())
            .map(|item| cross[item] - cross_size(item) / 2.0)
            .fold(f64::INFINITY, f64::min);
        for position in &mut cross {
            *position -= low;
        }
    }
    cross
}

/// Lines the main path up where its neighbours leave room: each item of each of its edges moves
/// to the cross position of the item before it, unless that would crowd a neighbour or leave the
/// lane.
fn straighten(
    model: &Model,
    cross: &mut [f64],
    cross_size: &impl Fn(usize) -> f64,
    gap: &impl Fn(usize, usize) -> f64,
    bands: &[(f64, f64)],
    lane_head: f64,
) {
    let graph = &model.graph;
    let edges: Vec<usize> = (0..model.diagram.edges.len())
        .filter(|edge| model.diagram.edges[*edge].main)
        .collect();
    // Moving one item may make room for, or take room from, another; a few rounds settle it.
    for _ in 0..3 {
        for &edge in &edges {
            let Some(chain) = &graph.chains[edge] else {
                continue;
            };
            for pair in chain.items.windows(2) {
                if (cross[pair[0]] - cross[pair[1]]).abs() < 0.01 {
                    continue;
                }
                // The later item follows the earlier one, or else the earlier one the later.
                let _ = align(
                    graph,
                    cross,
                    pair[1],
                    cross[pair[0]],
                    cross_size,
                    gap,
                    bands,
                    lane_head,
                    true,
                ) || align(
                    graph,
                    cross,
                    pair[0],
                    cross[pair[1]],
                    cross_size,
                    gap,
                    bands,
                    lane_head,
                    true,
                );
            }
        }
    }
    // Then the small steps of every other edge, where an item off the main path can move.
    let on_main: Vec<bool> = (0..graph.items.len())
        .map(|item| {
            edges.iter().any(|edge| {
                graph.chains[*edge]
                    .as_ref()
                    .is_some_and(|chain| chain.items.contains(&item))
            })
        })
        .collect();
    for chain in graph.chains.iter().flatten() {
        for pair in chain.items.windows(2) {
            let jog = (cross[pair[0]] - cross[pair[1]]).abs();
            if jog < 0.01 || jog > 24.0 {
                continue;
            }
            let _ = (!on_main[pair[1]]
                && align(
                    graph,
                    cross,
                    pair[1],
                    cross[pair[0]],
                    cross_size,
                    gap,
                    bands,
                    lane_head,
                    false,
                ))
                || (!on_main[pair[0]]
                    && align(
                        graph,
                        cross,
                        pair[0],
                        cross[pair[1]],
                        cross_size,
                        gap,
                        bands,
                        lane_head,
                        false,
                    ));
        }
    }
    if bands.is_empty() {
        let low = (0..graph.items.len())
            .map(|item| cross[item] - cross_size(item) / 2.0)
            .fold(f64::INFINITY, f64::min);
        for position in cross.iter_mut() {
            *position -= low;
        }
    }
}

/// How far the frame of `group` reaches on the cross axis: around its own steps and the frames
/// inside it, with the room a frame takes on either side.
fn frame_range(
    diagram: &Diagram,
    cross: &[f64],
    cross_size: &impl Fn(usize) -> f64,
    group: usize,
    landscape: bool,
) -> (f64, f64) {
    let own = (0..diagram.nodes.len())
        .filter(|node| diagram.nodes[*node].group == Some(group))
        .map(|node| {
            (
                cross[node] - cross_size(node) / 2.0,
                cross[node] + cross_size(node) / 2.0,
            )
        });
    let inner = (0..diagram.groups.len())
        .filter(|child| diagram.groups[*child].parent == Some(group))
        .map(|child| frame_range(diagram, cross, cross_size, child, landscape));
    let (low, high) = own
        .chain(inner)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), (a, b)| {
            (low.min(a), high.max(b))
        });
    let head = if landscape { GROUP_HEAD } else { 0.0 };
    (low - GROUP_PAD - head, high + GROUP_PAD)
}

/// Moves the steps that do not belong to a group out of its frame, with the steps beyond them, to
/// the side they stand on: a frame drawn around a group's steps would otherwise enclose a step of
/// a neighbouring layer that lies between them. Inner frames first; the frames around them then
/// take in what moved.
fn clear_frames(
    model: &Model,
    cross: &mut [f64],
    cross_size: &impl Fn(usize) -> f64,
    landscape: bool,
) {
    const MARGIN: f64 = 12.0;
    let (graph, diagram) = (&model.graph, model.diagram);
    let mut order: Vec<usize> = (0..diagram.groups.len()).collect();
    crate::sort::by_key(&mut order, |group| {
        (std::cmp::Reverse(diagram.chain(Some(*group)).len()), *group)
    });
    let item_size = |item: usize| cross_size(item);
    for &group in &order {
        let layers: Vec<usize> = (0..diagram.nodes.len())
            .filter(|node| diagram.inside(*node, group))
            .map(|node| graph.layer[node])
            .collect();
        let first = layers.iter().copied().min().unwrap_or(0);
        let last = layers.iter().copied().max().unwrap_or(0);
        for layer in first..=last {
            let items = &graph.layers[layer];
            for position in 0..items.len() {
                let item = items[position];
                let Some(node) = graph.items[item].node else {
                    continue;
                };
                if diagram.inside(node, group) {
                    continue;
                }
                let (low, high) = frame_range(diagram, cross, cross_size, group, landscape);
                let half = item_size(item) / 2.0;
                let (start, end) = (cross[item] - half - MARGIN, cross[item] + half + MARGIN);
                if end <= low || start >= high {
                    continue;
                }
                let member = |other: &usize| {
                    graph.items[*other]
                        .node
                        .is_some_and(|node| diagram.inside(node, group))
                };
                let before = items[..position].iter().any(member);
                let after = items[position + 1..].iter().any(member);
                let left = if before == after {
                    cross[item] < f64::midpoint(low, high)
                } else {
                    after
                };
                if left {
                    let shift = low - end;
                    for other in &items[..=position] {
                        cross[*other] += shift;
                    }
                } else {
                    let shift = high - start;
                    for other in &items[position..] {
                        cross[*other] += shift;
                    }
                }
            }
        }
    }
    separate_frames(model, cross, cross_size, landscape);
    let low = (0..graph.items.len())
        .map(|item| cross[item] - cross_size(item) / 2.0)
        .fold(f64::INFINITY, f64::min)
        .min(frames_reach(model, cross, cross_size, landscape).0);
    for position in cross.iter_mut() {
        *position -= low;
    }
}

/// Moves frames of unrelated groups apart where they share a layer and overlap on the cross
/// axis: a frame reaches over every layer of its steps, so two groups that stand side by side in
/// one layer and above each other in another would otherwise be drawn into each other. The frame
/// further along the cross axis moves on, with the steps beside it in the layers it spans.
fn separate_frames(
    model: &Model,
    cross: &mut [f64],
    cross_size: &impl Fn(usize) -> f64,
    landscape: bool,
) {
    const SEPARATION: f64 = 16.0;
    let (graph, diagram) = (&model.graph, model.diagram);
    let groups = diagram.groups.len();
    let span = |group: usize| -> Option<(usize, usize)> {
        let layers = (0..diagram.nodes.len())
            .filter(|node| diagram.inside(*node, group))
            .map(|node| graph.layer[node]);
        layers.clone().min().zip(layers.max())
    };
    let related = |a: usize, b: usize| {
        diagram.chain(Some(a)).contains(&b) || diagram.chain(Some(b)).contains(&a)
    };
    for _ in 0..=groups * groups {
        let mut moved = false;
        for a in 0..groups {
            for b in 0..groups {
                if a == b || related(a, b) {
                    continue;
                }
                let (Some((a_first, a_last)), Some((b_first, b_last))) = (span(a), span(b)) else {
                    continue;
                };
                if a_last < b_first || b_last < a_first {
                    continue;
                }
                let (a_low, a_high) = frame_range(diagram, cross, cross_size, a, landscape);
                let (b_low, b_high) = frame_range(diagram, cross, cross_size, b, landscape);
                // `a` is the frame on the low side of the cross axis.
                if a_low + a_high >= b_low + b_high
                    || a_high + SEPARATION <= b_low
                    || b_high <= a_low
                {
                    continue;
                }
                let shift = a_high + SEPARATION - b_low;
                let anchor = (0..diagram.nodes.len())
                    .filter(|node| diagram.inside(*node, b))
                    .map(|node| cross[node])
                    .fold(f64::INFINITY, f64::min);
                for layer in b_first..=b_last {
                    for &item in &graph.layers[layer] {
                        if cross[item] >= anchor {
                            cross[item] += shift;
                        }
                    }
                }
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
}

/// How far the outermost frames reach on the cross axis: their low and their high end.
fn frames_reach(
    model: &Model,
    cross: &[f64],
    cross_size: &impl Fn(usize) -> f64,
    landscape: bool,
) -> (f64, f64) {
    let diagram = model.diagram;
    (0..diagram.groups.len())
        .filter(|group| diagram.groups[*group].parent.is_none())
        .map(|group| frame_range(diagram, cross, cross_size, group, landscape))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), (a, b)| {
            (low.min(a), high.max(b))
        })
}

/// Moves `item` to the cross position `target`: alone if its neighbours leave room, else with
/// the whole run of its lane in its layer if the lane does. Returns whether it moved.
#[allow(clippy::too_many_arguments)]
fn align(
    graph: &Graph,
    cross: &mut [f64],
    item: usize,
    target: f64,
    cross_size: &impl Fn(usize) -> f64,
    gap: &impl Fn(usize, usize) -> f64,
    bands: &[(f64, f64)],
    lane_head: f64,
    whole_run: bool,
) -> bool {
    let layer = &graph.layers[graph.layer[item]];
    let position = graph.position[item];
    let lane = graph.items[item].lane;
    let inside = |low_edge: f64, high_edge: f64| {
        lane.is_none_or(|lane| {
            let band = bands[lane];
            low_edge >= band.0 + LANE_PAD + lane_head - 0.01
                && high_edge <= band.1 - LANE_PAD + 0.01
        })
    };
    let fits_left = position == 0 || {
        let left = layer[position - 1];
        graph.items[left].lane != lane || cross[left] + gap(left, item) <= target + 0.01
    };
    let fits_right = layer.get(position + 1).is_none_or(|&right| {
        graph.items[right].lane != lane || target + gap(item, right) <= cross[right] + 0.01
    });
    let half = cross_size(item) / 2.0;
    if fits_left && fits_right && inside(target - half, target + half) {
        cross[item] = target;
        return true;
    }
    if !whole_run {
        return false;
    }
    let segment: Vec<usize> = layer
        .iter()
        .copied()
        .filter(|other| graph.items[*other].lane == lane)
        .collect();
    let shift = target - cross[item];
    let (first, last) = (segment[0], segment[segment.len() - 1]);
    if inside(
        cross[first] + shift - cross_size(first) / 2.0,
        cross[last] + shift + cross_size(last) / 2.0,
    ) {
        for other in segment {
            cross[other] += shift;
        }
        return true;
    }
    false
}

/// Where edges leave and reach their steps on the cross axis, as offsets from the step's middle.
struct Ports {
    /// Per edge: the offset where it leaves its first step and where it reaches its last.
    leave: Vec<f64>,
    reach: Vec<f64>,
}

impl Ports {
    fn new(
        graph: &Graph,
        cross: &[f64],
        cross_size: &impl Fn(usize) -> f64,
        model: &Model,
        landscape: bool,
    ) -> Self {
        let edges = graph.chains.len();
        let mut leave = vec![0.0; edges];
        let mut reach = vec![0.0; edges];
        for node in 0..model.diagram.nodes.len() {
            for (outgoing, offsets) in [(true, &mut leave), (false, &mut reach)] {
                let mut edges: Vec<(f64, usize)> = graph
                    .chains
                    .iter()
                    .enumerate()
                    .filter_map(|(edge, chain)| {
                        let chain = chain.as_ref()?;
                        let items = &chain.items;
                        let (end, next) = if outgoing {
                            (items[0], items[1])
                        } else {
                            (items[items.len() - 1], items[items.len() - 2])
                        };
                        (end == node).then_some((cross[next], edge))
                    })
                    .collect();
                crate::sort::by(&mut edges, |a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                let ports = count(edges.len());
                if edges.len() < 2 {
                    continue;
                }
                let size = cross_size(node);
                let share = if model.diagram.nodes[node].shape == Shape::Decision {
                    0.3
                } else {
                    0.6
                };
                let mut step = (size * share / (ports - 1.0)).min(24.0);
                if landscape && outgoing {
                    step = step.max(18.0).min(size / ports);
                }
                // An edge of the main path keeps the middle, so that the path runs straight; the
                // others keep their order on either side of it.
                let main = edges
                    .iter()
                    .position(|(_, edge)| model.diagram.edges[*edge].main)
                    .map(count);
                let middle = main.unwrap_or((ports - 1.0) / 2.0);
                let reach = (middle).max(ports - 1.0 - middle);
                let step = step.min((size / 2.0 - 8.0) / reach);
                for (index, (_, edge)) in edges.iter().enumerate() {
                    offsets[*edge] = (count(index) - middle) * step;
                }
            }
        }
        Self { leave, reach }
    }
}

/// How far the edge of step `node` lies from its middle on the main axis, at `offset` across:
/// half its depth, less towards the corners of a diamond.
fn edge_depth(plan: &Plan, model: &Model, node: usize, offset: f64) -> f64 {
    let (width, height) = plan.sizes[node];
    let (main, cross) = if plan.landscape {
        (width, height)
    } else {
        (height, width)
    };
    if model.diagram.nodes[node].shape == Shape::Decision {
        main / 2.0 * (1.0 - 2.0 * offset.abs() / cross).max(0.0)
    } else {
        main / 2.0
    }
}

/// One stretch of an edge between two adjacent layers: the cross positions it comes from and
/// goes to, and the track it changes course on.
struct Piece {
    edge: usize,
    layer: usize,
    from: f64,
    to: f64,
    track: Option<usize>,
}

/// The spaces between layers: room for labels where edges leave, tracks for edges that change
/// course, and room for the frames of groups.
struct Gaps {
    pieces: Vec<Piece>,
    /// Per gap: the room for labels, the number of tracks, and the room a group's frame takes
    /// after the layer before and before the layer after.
    label_room: Vec<f64>,
    /// Per gap: the room for the labels of edges that leave the layer after it against the flow.
    label_room_end: Vec<f64>,
    tracks: Vec<usize>,
    frame_after: Vec<f64>,
    frame_before: Vec<f64>,
}

impl Gaps {
    fn new(model: &Model, plan: &Plan, ports: &Ports, landscape: bool) -> Self {
        let graph = &model.graph;
        let layers = graph.layers.len();
        let gaps = layers.saturating_sub(1);
        let mut pieces = Vec::new();
        let mut label_room = vec![0.0; gaps];
        let mut label_room_end = vec![0.0; gaps];
        let mut stacks: std::collections::BTreeMap<(usize, bool, bool), f64> =
            std::collections::BTreeMap::new();
        for (edge, chain) in graph.chains.iter().enumerate() {
            let Some(chain) = chain else { continue };
            let items = &chain.items;
            let last = items.len() - 2;
            for (index, pair) in items.windows(2).enumerate() {
                let from = plan.cross[pair[0]] + if index == 0 { ports.leave[edge] } else { 0.0 };
                let to = plan.cross[pair[1]]
                    + if index == last {
                        ports.reach[edge]
                    } else {
                        0.0
                    };
                pieces.push(Piece {
                    edge,
                    layer: graph.layer[pair[0]],
                    from,
                    to,
                    track: None,
                });
            }
            let lines = &model.edge_labels[edge];
            if !lines.is_empty() {
                // In portrait, labels on the same side of a step stack along the flow.
                let (node, offset) = if chain.reversed {
                    (items[items.len() - 1], ports.reach[edge])
                } else {
                    (items[0], ports.leave[edge])
                };
                let room = if landscape {
                    model.edge_widths[edge] + 26.0
                } else {
                    let stacked = stacks
                        .entry((node, chain.reversed, offset >= 0.0))
                        .or_insert(0.0);
                    *stacked += LINE * count(lines.len()) + 9.0;
                    *stacked + 7.0
                };
                if chain.reversed {
                    let gap = graph.layer[node] - 1;
                    label_room_end[gap] = f64::max(label_room_end[gap], room);
                } else {
                    let gap = graph.layer[node];
                    label_room[gap] = f64::max(label_room[gap], room);
                }
            }
        }
        let mut tracks = vec![0; gaps];
        for (gap, tracks) in tracks.iter_mut().enumerate() {
            let mut bending: Vec<usize> = (0..pieces.len())
                .filter(|index| {
                    pieces[*index].layer == gap
                        && (pieces[*index].from - pieces[*index].to).abs() > 0.5
                })
                .collect();
            crate::sort::by(&mut bending, |a, b| {
                let low = |piece: &Piece| piece.from.min(piece.to);
                low(&pieces[*a])
                    .total_cmp(&low(&pieces[*b]))
                    .then(pieces[*a].edge.cmp(&pieces[*b].edge))
            });
            let assigned = assign_tracks(&pieces, &bending);
            for (index, track) in &assigned {
                pieces[*index].track = Some(*track);
            }
            *tracks = assigned
                .iter()
                .map(|(_, track)| track + 1)
                .max()
                .unwrap_or(0);
        }
        let (frame_before, frame_after) = frame_room(model, landscape);
        Self {
            pieces,
            label_room,
            label_room_end,
            tracks,
            frame_after,
            frame_before,
        }
    }

    /// How much the gaps grow to fill `room` on the main axis, up to [`MAX_STRETCH`].
    fn stretch(&self, natural: f64, room: f64) -> f64 {
        if natural < room && self.total() > 0.0 {
            (1.0 + (room - natural) / self.total()).min(MAX_STRETCH)
        } else {
            1.0
        }
    }

    /// The width of gap `gap` at scale 1.
    fn size(&self, gap: usize) -> f64 {
        GAP + self.frame_after[gap]
            + self.label_room[gap]
            + count(self.tracks[gap]) * TRACK
            + self.label_room_end[gap]
            + self.frame_before[gap + 1]
    }

    fn total(&self) -> f64 {
        (0..self.tracks.len()).map(|gap| self.size(gap)).sum()
    }

    /// The length of the layers along the main axis with the gaps at their least.
    fn length(
        &self,
        main_size: &impl Fn(usize) -> f64,
        graph: &Graph,
        (lead, trail): (f64, f64),
    ) -> f64 {
        let depths: f64 = graph
            .layers
            .iter()
            .map(|layer| depth(graph, layer, main_size))
            .sum();
        lead + self.frame_before.first().copied().unwrap_or(0.0)
            + depths
            + self.total()
            + self.frame_after.last().copied().unwrap_or(0.0)
            + trail
    }

    /// Places the layers on the main axis and returns their length.
    fn place(
        &self,
        plan: &mut Plan,
        main_size: &impl Fn(usize) -> f64,
        graph: &Graph,
        (lead, trail): (f64, f64),
        stretch: f64,
    ) -> f64 {
        let mut at = lead + self.frame_before.first().copied().unwrap_or(0.0);
        plan.layers.clear();
        for (index, layer) in graph.layers.iter().enumerate() {
            let depth = depth(graph, layer, main_size);
            plan.layers.push((at, at + depth));
            at += depth;
            if index < self.tracks.len() {
                at += self.size(index) * stretch;
            }
        }
        at + self.frame_after.last().copied().unwrap_or(0.0) + trail
    }

    /// The main position of track `track` in gap `gap`, which begins at `start`.
    fn track(&self, gap: usize, track: usize, start: f64, stretch: f64) -> f64 {
        start
            + (self.frame_after[gap] + self.label_room[gap] + GAP / 2.0 + count(track) * TRACK)
                * stretch
    }
}

/// Per layer, the room the frames of groups take before it on the main axis, where they open,
/// and after it, where they close.
fn frame_room(model: &Model, landscape: bool) -> (Vec<f64>, Vec<f64>) {
    let graph = &model.graph;
    let layers = graph.layers.len();
    let mut frame_after = vec![0.0; layers];
    let mut frame_before = vec![0.0; layers];
    // The layers each group spans, through the groups inside it.
    let diagram = model.diagram;
    let spans: Vec<(usize, usize)> = (0..diagram.groups.len())
        .map(|group| {
            let layers: Vec<usize> = (0..diagram.nodes.len())
                .filter(|node| diagram.inside(*node, group))
                .map(|node| graph.layer[node])
                .collect();
            (
                layers.iter().copied().min().unwrap_or(0),
                layers.iter().copied().max().unwrap_or(0),
            )
        })
        .collect();
    let head = if landscape {
        GROUP_PAD
    } else {
        GROUP_PAD + GROUP_HEAD
    };
    // A step needs room for every frame that opens before it or closes after it.
    for node in 0..diagram.nodes.len() {
        let layer = graph.layer[node];
        let chain = diagram.chain(diagram.nodes[node].group);
        let opening = chain
            .iter()
            .filter(|group| spans[**group].0 == layer)
            .count();
        let closing = chain
            .iter()
            .filter(|group| spans[**group].1 == layer)
            .count();
        frame_before[layer] = f64::max(frame_before[layer], head * count(opening));
        frame_after[layer] = f64::max(frame_after[layer], GROUP_PAD * count(closing));
    }
    (frame_before, frame_after)
}

/// How deep a layer is on the main axis: as deep as its deepest step.
fn depth(graph: &Graph, layer: &[usize], main_size: &impl Fn(usize) -> f64) -> f64 {
    layer
        .iter()
        .filter_map(|item| graph.items[*item].node)
        .map(main_size)
        .fold(0.0, f64::max)
}

/// The tracks of the pieces that change course in one gap, so that no edge crosses another where
/// that can be avoided. A piece leaves its layer at `from`, runs along its track and arrives at
/// `to`: it crosses a piece whose track lies before its own when that one's start lies under it,
/// and one whose track lies after its own when that one's end does. So a piece whose start lies
/// under another takes the earlier track, and one whose end lies under another the later.
/// Pieces that cannot both be satisfied, and pieces left over, follow the order given.
fn assign_tracks(pieces: &[Piece], order: &[usize]) -> Vec<(usize, usize)> {
    let range = |piece: &Piece| (piece.from.min(piece.to), piece.from.max(piece.to));
    let under = |x: f64, (low, high): (f64, f64)| low + 0.5 < x && x < high - 0.5;
    // `before[v]` lists the pieces that must have an earlier track than `v`.
    let mut before: Vec<Vec<usize>> = vec![Vec::new(); order.len()];
    for (a, &first) in order.iter().enumerate() {
        for (b, &second) in order.iter().enumerate() {
            if a == b {
                continue;
            }
            let (one, other) = (&pieces[first], &pieces[second]);
            // `a`'s start under `b`'s run: `a` goes first. `a`'s end under it: `a` goes last.
            if under(one.from, range(other)) {
                before[b].push(a);
            }
            if under(one.to, range(other)) {
                before[a].push(b);
            }
        }
    }
    let mut tracks: Vec<Option<usize>> = vec![None; order.len()];
    let mut on_track: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut result = Vec::with_capacity(order.len());
    for _ in 0..order.len() {
        // The next piece is the first whose predecessors all have tracks; with a cycle, the first
        // left.
        let next = (0..order.len())
            .find(|&v| tracks[v].is_none() && before[v].iter().all(|&u| tracks[u].is_some()))
            .or_else(|| (0..order.len()).find(|&v| tracks[v].is_none()))
            .expect("a piece is left");
        let (low, high) = range(&pieces[order[next]]);
        let earliest = before[next]
            .iter()
            .filter_map(|&u| tracks[u])
            .map(|track| track + 1)
            .max()
            .unwrap_or(0);
        let track = (earliest..on_track.len())
            .find(|&track| {
                on_track.get(track).is_none_or(|ranges| {
                    ranges
                        .iter()
                        .all(|&(l, h)| h + 10.0 < low || high + 10.0 < l)
                })
            })
            .unwrap_or(earliest.max(on_track.len()));
        if on_track.len() <= track {
            on_track.resize(track + 1, Vec::new());
        }
        on_track[track].push((low, high));
        tracks[next] = Some(track);
        result.push((order[next], track));
    }
    result
}

/// The way of every edge: out of its first step, along each piece and its track, into its last
/// step; an edge the layout turned around runs back the other way.
fn route(
    model: &Model,
    plan: &Plan,
    ports: &Ports,
    gaps: &Gaps,
    stretch: f64,
) -> Vec<Option<Route>> {
    let graph = &model.graph;
    let middle = |item: usize| {
        let layer = plan.layers[graph.layer[item]];
        f64::midpoint(layer.0, layer.1)
    };
    graph
        .chains
        .iter()
        .enumerate()
        .map(|(edge, chain)| {
            let chain = chain.as_ref()?;
            let items = &chain.items;
            let first = items[0];
            let last = items[items.len() - 1];
            let start = (
                middle(first) + edge_depth(plan, model, first, ports.leave[edge]),
                plan.cross[first] + ports.leave[edge],
            );
            let mut points = vec![start];
            for piece in gaps.pieces.iter().filter(|piece| piece.edge == edge) {
                if let Some(track) = piece.track {
                    let main = gaps.track(piece.layer, track, plan.layers[piece.layer].1, stretch);
                    points.push((main, piece.from));
                    points.push((main, piece.to));
                }
            }
            points.push((
                middle(last) - edge_depth(plan, model, last, ports.reach[edge]),
                plan.cross[last] + ports.reach[edge],
            ));
            let end = points[points.len() - 1];
            let (label_at, offset) = if chain.reversed {
                (end, ports.reach[edge])
            } else {
                (start, ports.leave[edge])
            };
            // A label in the middle goes to the side where no other edge meets the same side of
            // the step: right in portrait, above in landscape, unless that side is taken.
            let siblings: Vec<f64> = graph
                .chains
                .iter()
                .enumerate()
                .filter_map(|(other, sibling)| {
                    let items = &sibling.as_ref()?.items;
                    let meets = if chain.reversed {
                        items[items.len() - 1] == last
                    } else {
                        items[0] == first
                    };
                    let offset = if chain.reversed {
                        ports.reach[other]
                    } else {
                        ports.leave[other]
                    };
                    (other != edge && meets).then_some(offset)
                })
                .collect();
            let taken_after = siblings.iter().any(|offset| *offset > 0.5);
            let taken_before = siblings.iter().any(|offset| *offset < -0.5);
            let after = if offset > 0.5 {
                true
            } else if offset < -0.5 {
                false
            } else if plan.landscape {
                taken_before && !taken_after
            } else {
                !taken_after || taken_before
            };
            let clear = siblings
                .iter()
                .map(|sibling| {
                    if after {
                        sibling - offset
                    } else {
                        offset - sibling
                    }
                })
                .filter(|distance| *distance > 0.5)
                .fold(0.0, f64::max);
            if chain.reversed {
                points.reverse();
            }
            Some(Route {
                points,
                label_at,
                against: chain.reversed,
                after,
                clear,
            })
        })
        .collect()
}

impl Plan {
    fn draw(
        &self,
        model: &Model,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
        elements: &mut Vec<Element>,
    ) {
        self.draw_lanes(model, elements);
        self.draw_groups(model, metrics, warnings, elements);
        let mut labels = Vec::new();
        // Every edge, loop and step sits in a group whose classes name the steps it belongs to,
        // so that focusing a step can bring them forward.
        for edge in 0..model.diagram.edges.len() {
            elements.push(Element::Group(model.edge_classes(edge)));
            self.draw_edge(model, edge, elements, &mut labels);
            elements.push(Element::GroupEnd);
        }
        for node in 0..model.diagram.nodes.len() {
            elements.push(Element::Group(model.node_classes(node)));
            self.draw_loop(model, node, elements, &mut labels);
            elements.push(Element::GroupEnd);
        }
        self.draw_labels(model, labels, metrics, elements);
        for node in 0..model.diagram.nodes.len() {
            elements.push(Element::Group(model.node_classes(node)));
            self.draw_node(model, node, elements);
            elements.push(Element::GroupEnd);
            let spec = &model.diagram.nodes[node];
            if spec.shape != Shape::Initial {
                let (x, y, width, height) = self.node_box(model, node);
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
    }

    fn draw_lanes(&self, model: &Model, elements: &mut Vec<Element>) {
        let lanes = &model.diagram.lanes;
        for (index, (lane, band)) in lanes.iter().zip(&self.bands).enumerate() {
            let (x1, y1) = self.page((0.0, band.0));
            let (x2, y2) = self.page((self.main_length, band.1));
            elements.push(Element::Rect(Rect {
                x: x1.min(x2),
                y: y1.min(y2),
                width: (x2 - x1).abs(),
                height: (y2 - y1).abs(),
                class: if index % 2 == 0 {
                    "chartlet-flow-lane"
                } else {
                    "chartlet-flow-lane chartlet-flow-lane-alt"
                },
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
            // A strip along the lane's start holds its name.
            let (h1, h2) = if self.landscape {
                (
                    self.page((0.0, band.0)),
                    self.page((self.main_length, band.0 + LANE_HEAD)),
                )
            } else {
                (self.page((0.0, band.0)), self.page((LANE_HEAD, band.1)))
            };
            elements.push(Element::Rect(Rect {
                x: h1.0.min(h2.0) + 0.5,
                y: h1.1.min(h2.1) + 0.5,
                width: (h2.0 - h1.0).abs() - 1.0,
                height: (h2.1 - h1.1).abs() - 1.0,
                class: "chartlet-flow-lane-head",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
            let (x, y, anchor) = if self.landscape {
                (x1 + 10.0, y1.min(y2) + 17.0, TextAnchor::Start)
            } else {
                (f64::midpoint(x1, x2), y1.min(y2) + 17.0, TextAnchor::Middle)
            };
            elements.push(Element::Text(Text {
                x,
                y,
                class: "chartlet-flow-lane-label",
                anchor,
                content: lane.clone(),
            }));
        }
    }

    /// The frame of every group: around its own steps and the frames inside it.
    fn group_frames(&self, model: &Model) -> Vec<(f64, f64, f64, f64)> {
        let diagram = model.diagram;
        let groups = diagram.groups.len();
        let depth = |group: usize| diagram.chain(Some(group)).len();
        // Frames from the innermost out.
        let mut frames: Vec<(f64, f64, f64, f64)> = vec![(0.0, 0.0, 0.0, 0.0); groups];
        let mut inner_first: Vec<usize> = (0..groups).collect();
        crate::sort::by_key(&mut inner_first, |group| {
            (std::cmp::Reverse(depth(*group)), *group)
        });
        for &group in &inner_first {
            let boxes = (0..diagram.nodes.len())
                .filter(|node| diagram.nodes[*node].group == Some(group))
                .map(|node| {
                    let (x, y, width, height) = self.node_box(model, node);
                    (x, y, x + width, y + height)
                })
                .chain(
                    (0..groups)
                        .filter(|child| diagram.groups[*child].parent == Some(group))
                        .map(|child| frames[child]),
                );
            let (left, top, right, bottom) = boxes.fold(
                (
                    f64::INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                ),
                |(left, top, right, bottom), b| {
                    (left.min(b.0), top.min(b.1), right.max(b.2), bottom.max(b.3))
                },
            );
            frames[group] = (
                left - GROUP_PAD,
                top - GROUP_PAD - GROUP_HEAD,
                right + GROUP_PAD,
                bottom + GROUP_PAD,
            );
        }
        frames
    }

    fn draw_groups(
        &self,
        model: &Model,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
        elements: &mut Vec<Element>,
    ) {
        let diagram = model.diagram;
        let groups = diagram.groups.len();
        let frames = self.group_frames(model);
        let depth = |group: usize| diagram.chain(Some(group)).len();
        let mut inner_first: Vec<usize> = (0..groups).collect();
        crate::sort::by_key(&mut inner_first, |group| {
            (std::cmp::Reverse(depth(*group)), *group)
        });
        for &group in inner_first.iter().rev() {
            let (left, top, right, bottom) = frames[group];
            let intruder = (0..diagram.nodes.len())
                .filter(|node| !diagram.inside(*node, group))
                .map(|node| self.node_box(model, node))
                .any(|(x, y, width, height)| {
                    x < right && x + width > left && y < bottom && y + height > top
                });
            if intruder {
                warnings.push(ChartWarning::new(
                    "group_overlap",
                    diagram.groups[group].path.clone(),
                    "a step outside the frame lies inside it; move it elsewhere or regroup the steps",
                ));
            }
            elements.push(Element::Rect(Rect {
                x: left,
                y: top,
                width: right - left,
                height: bottom - top,
                class: "chartlet-flow-group",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
            let path = format!("{}/label", diagram.groups[group].path);
            elements.push(Element::Text(Text {
                x: left + 8.0,
                y: top + 13.0,
                class: "chartlet-flow-group-label",
                anchor: TextAnchor::Start,
                content: crate::layout::fit_text(
                    &diagram.groups[group].label,
                    right - left - 16.0,
                    SUBLABEL_SIZE,
                    metrics,
                    warnings,
                    &path,
                ),
            }));
        }
    }

    /// The label of an edge that leaves a step beside others that go the same way. Beyond their
    /// lines it would read as theirs, so in portrait it goes under the longest stretch the edge
    /// runs across instead.
    fn run_label(
        &self,
        model: &Model,
        edge: usize,
        route: &Route,
        lines: &[String],
    ) -> Option<Label> {
        if self.landscape || route.clear <= 0.5 {
            return None;
        }
        let corners: Vec<(f64, f64)> = route.points.iter().map(|p| self.page(*p)).collect();
        let run = corners
            .windows(2)
            .filter(|pair| (pair[0].1 - pair[1].1).abs() < 0.5)
            .max_by(|a, b| (a[0].0 - a[1].0).abs().total_cmp(&(b[0].0 - b[1].0).abs()))?;
        Some(Label {
            lines: lines.to_vec(),
            classes: model.edge_classes(edge),
            at: (f64::midpoint(run[0].0, run[1].0), run[0].1 + 18.0),
            anchor: TextAnchor::Middle,
            beside: None,
        })
    }

    fn draw_edge(
        &self,
        model: &Model,
        edge: usize,
        elements: &mut Vec<Element>,
        labels: &mut Vec<Label>,
    ) {
        let Some(route) = &self.routes[edge] else {
            return;
        };
        let diagram = model.diagram;
        let spec = &diagram.edges[edge];
        let (from, to) = model.ends[edge];
        let mut points: Vec<(f64, f64)> =
            route.points.iter().map(|point| self.page(*point)).collect();
        let tip = points[points.len() - 1];
        let before = points[points.len() - 2];
        let length = (tip.0 - before.0).hypot(tip.1 - before.1);
        let direction = if length > 0.0 {
            ((tip.0 - before.0) / length, (tip.1 - before.1) / length)
        } else {
            (0.0, 1.0)
        };
        let last = points.len() - 1;
        points[last] = (tip.0 - direction.0 * HEAD, tip.1 - direction.1 * HEAD);
        let main = spec.main;
        let class = match (main, spec.dash) {
            (true, _) => "chartlet-flow-edge chartlet-flow-main",
            (false, Some(Dash::Dashed)) => "chartlet-flow-edge chartlet-flow-dashed",
            (false, Some(Dash::Dotted)) => "chartlet-flow-edge chartlet-flow-dotted",
            (false, _) => "chartlet-flow-edge",
        };
        let note = match (&spec.label, &spec.technology) {
            (Some(label), Some(technology)) => Some(format!("{label} [{technology}]")),
            (Some(label), None) => Some(label.clone()),
            (None, Some(technology)) => Some(format!("[{technology}]")),
            (None, None) => None,
        };
        let (from_label, to_label) = (&diagram.nodes[from].label, &diagram.nodes[to].label);
        let tooltip = match note {
            Some(note) => format!("{from_label} → {to_label}: {note}"),
            None => format!("{from_label} → {to_label}"),
        };
        elements.push(Element::Polyline(diagram::polyline(
            points,
            class,
            Some(tooltip),
        )));
        elements.push(Element::Polyline(arrowhead(
            tip,
            direction,
            true,
            if main {
                "chartlet-flow-head chartlet-flow-main-head"
            } else {
                "chartlet-flow-head"
            },
        )));
        let lines = &model.edge_labels[edge];
        if lines.is_empty() {
            return;
        }
        if let Some(label) = self.run_label(model, edge, route, lines) {
            labels.push(label);
            return;
        }
        let (x, y) = self.page(route.label_at);
        let last = count(lines.len() - 1);
        let reach = CHIP_REACH + 2.0 + route.clear;
        let side = |x: f64| {
            if route.after {
                (x + reach, TextAnchor::Start)
            } else {
                (x - reach, TextAnchor::End)
            }
        };
        // In landscape a label sits above its edge, or below it on the lower side of the step.
        let above = |y: f64| {
            if route.after {
                y + 18.0 + route.clear
            } else {
                y - 11.0 - LINE * last - route.clear
            }
        };
        let along = CHIP_REACH + 2.0;
        let (x, y, anchor) = match (self.landscape, route.against) {
            (true, false) => (x + along, above(y), TextAnchor::Start),
            (true, true) => (x - along, above(y), TextAnchor::End),
            (false, false) => {
                let (x, anchor) = side(x);
                (x, y + 19.0, anchor)
            }
            (false, true) => {
                let (x, anchor) = side(x);
                (x, y - 12.0 - LINE * last, anchor)
            }
        };
        let (line, _) = self.page(route.label_at);
        labels.push(Label {
            lines: lines.clone(),
            classes: model.edge_classes(edge),
            at: (x, y),
            anchor,
            beside: (!self.landscape).then_some(line),
        });
    }

    /// Draws the edge labels over all edges, each moved along the flow until it no longer covers
    /// one drawn before it.
    fn draw_labels(
        &self,
        model: &Model,
        labels: Vec<Label>,
        metrics: &impl TextMetrics,
        elements: &mut Vec<Element>,
    ) {
        const AIR: f64 = 3.0;
        // Labels keep clear of the steps as well as of each other.
        let mut placed: Vec<(f64, f64, f64, f64)> = (0..model.diagram.nodes.len())
            .map(|node| self.node_box(model, node))
            .collect();
        // In portrait the borders of the frames are kept clear too, as thin boxes.
        let borders: Vec<(f64, f64, f64, f64)> = self
            .group_frames(model)
            .into_iter()
            .flat_map(|(left, top, right, bottom)| {
                if self.landscape {
                    [
                        (left, top, 0.0, bottom - top),
                        (right, top, 0.0, bottom - top),
                    ]
                } else {
                    [
                        (left, top, right - left, 0.0),
                        (left, bottom, right - left, 0.0),
                    ]
                }
            })
            .collect();
        for mut label in labels {
            label.keep_on_page(self.width, metrics);
            for _ in 0..12 {
                let (x, y, width, height) = chip_box(&label.lines, label.at, label.anchor, metrics);
                let hits = |other: &&(f64, f64, f64, f64)| {
                    x < other.0 + other.2 + AIR
                        && other.0 < x + width + AIR
                        && y < other.1 + other.3 + AIR
                        && other.1 < y + height + AIR
                };
                if let Some(border) = borders.iter().find(hits) {
                    label.at.1 += border.1 + border.3 + AIR - y;
                    continue;
                }
                let Some(other) = placed.iter().find(|other| {
                    x < other.0 + other.2 + AIR
                        && other.0 < x + width + AIR
                        && y < other.1 + other.3 + AIR
                        && other.1 < y + height + AIR
                }) else {
                    break;
                };
                if self.landscape {
                    label.at.0 += other.0 + other.2 + AIR - x;
                } else {
                    label.at.1 += other.1 + other.3 + AIR - y;
                }
            }
            placed.push(chip_box(&label.lines, label.at, label.anchor, metrics));
            elements.push(Element::Group(label.classes.clone()));
            chip(&label.lines, label.at, label.anchor, metrics, elements);
            elements.push(Element::GroupEnd);
        }
    }

    /// An edge from a step to itself: a loop beside the step, right in portrait and below in
    /// landscape, with the labels of all such edges joined.
    fn draw_loop(
        &self,
        model: &Model,
        node: usize,
        elements: &mut Vec<Element>,
        labels: &mut Vec<Label>,
    ) {
        let Some(label) = &model.loops[node] else {
            return;
        };
        let (x, y, width, height) = self.node_box(model, node);
        let (points, tip, direction, text) = if self.landscape {
            let (middle, bottom) = (x + width / 2.0, y + height);
            (
                vec![
                    (middle - 8.0, bottom),
                    (middle - 8.0, bottom + LOOP),
                    (middle + 8.0, bottom + LOOP),
                    (middle + 8.0, bottom + HEAD),
                ],
                (middle + 8.0, bottom),
                (0.0, -1.0),
                (middle, bottom + LOOP + 18.0, TextAnchor::Middle),
            )
        } else {
            let (right, middle) = (x + width, y + height / 2.0);
            (
                vec![
                    (right, middle - 8.0),
                    (right + LOOP, middle - 8.0),
                    (right + LOOP, middle + 8.0),
                    (right + HEAD, middle + 8.0),
                ],
                (right, middle + 8.0),
                (-1.0, 0.0),
                (
                    right + LOOP + CHIP_REACH + 2.0,
                    middle + 4.0,
                    TextAnchor::Start,
                ),
            )
        };
        let name = &model.diagram.nodes[node].label;
        let tooltip = if label.is_empty() {
            format!("{name} → {name}")
        } else {
            format!("{name} → {name}: {label}")
        };
        elements.push(Element::Polyline(diagram::polyline(
            rounded(&points),
            "chartlet-flow-edge",
            Some(tooltip),
        )));
        elements.push(Element::Polyline(arrowhead(
            tip,
            direction,
            true,
            "chartlet-flow-head",
        )));
        let lines = &model.loop_lines[node];
        if !lines.is_empty() {
            // Beside the loop, the lines centred on it; below it, they run down.
            let rise = if self.landscape {
                0.0
            } else {
                CHIP_LINE * count(lines.len() - 1) / 2.0
            };
            labels.push(Label {
                lines: lines.clone(),
                classes: model.node_classes(node),
                at: (text.0, text.1 - rise),
                anchor: text.2,
                beside: None,
            });
        }
    }

    fn draw_node(&self, model: &Model, node: usize, elements: &mut Vec<Element>) {
        let spec = &model.diagram.nodes[node];
        let (x, y, width, height) = self.node_box(model, node);
        let tooltip = Some(match &spec.sublabel {
            Some(sublabel) => format!("{} – {sublabel}", spec.label),
            None => spec.label.clone(),
        });
        shape(spec.shape, (x, y, width, height), tooltip, elements);
        Self::node_text(model, node, (x, y, width, height), elements);
    }

    /// A step's label, centered on one or two lines, and its sublabel below.
    fn node_text(
        model: &Model,
        node: usize,
        (x, y, width, height): (f64, f64, f64, f64),
        elements: &mut Vec<Element>,
    ) {
        let spec = &model.diagram.nodes[node];
        // The initial dot carries no text; its name is its tooltip.
        if spec.shape == Shape::Initial {
            return;
        }
        let (middle_x, middle_y) = (x + width / 2.0, y + height / 2.0);
        let lines = &model.labels[node];
        let sublines = &model.sublabels[node];
        let block = NODE_LINE * count(lines.len() - 1) + 14.0 * count(sublines.len());
        // Text clears what a shape draws at its top.
        let shift = match spec.shape {
            Shape::Store => 3.0,
            Shape::Person => 8.0,
            Shape::Frontend => 6.0,
            Shape::Shield => -6.0,
            _ => 0.0,
        };
        let first = middle_y - block / 2.0 + 4.5 + shift;
        for (index, line) in lines.iter().enumerate() {
            elements.push(Element::Text(Text {
                x: middle_x,
                y: first + NODE_LINE * count(index),
                class: "chartlet-flow-label",
                anchor: TextAnchor::Middle,
                content: line.clone(),
            }));
        }
        for (index, line) in sublines.iter().enumerate() {
            elements.push(Element::Text(Text {
                x: middle_x,
                y: first + NODE_LINE * count(lines.len() - 1) + 14.0 * count(index + 1),
                class: "chartlet-flow-sublabel",
                anchor: TextAnchor::Middle,
                content: line.clone(),
            }));
        }
    }
}

/// The shape of a step of `kind` in the box at `x`, `y`, with its shadow: every kind has its own,
/// and its role color only repeats what the shape says.
pub(crate) fn shape(
    kind: Shape,
    (x, y, width, height): (f64, f64, f64, f64),
    tooltip: Option<String>,
    elements: &mut Vec<Element>,
) {
    let shape = |points: Vec<(f64, f64)>, class: &'static str, tooltip: Option<String>| {
        Element::Polyline(diagram::polyline(points, class, tooltip))
    };
    let rect = |class: &'static str, tooltip: Option<String>| {
        Element::Rect(Rect {
            x,
            y,
            width,
            height,
            class,
            series_index: None,
            style_index: None,
            tooltip,
        })
    };
    let (middle_x, middle_y) = (x + width / 2.0, y + height / 2.0);
    // Every kind has its own shape; its role color only repeats what the shape says.
    match kind {
        Shape::Process => with_shadow(
            rect("chartlet-flow-node chartlet-role-blue", tooltip),
            elements,
        ),
        Shape::External => with_shadow(
            rect(
                "chartlet-flow-node chartlet-flow-external chartlet-role-gray",
                tooltip,
            ),
            elements,
        ),
        Shape::Subprocess => {
            with_shadow(
                rect("chartlet-flow-node chartlet-role-blue", tooltip),
                elements,
            );
            for side in [x + 7.0, x + width - 7.0] {
                elements.push(Element::Line(Line {
                    x1: side,
                    y1: y,
                    x2: side,
                    y2: y + height,
                    class: "chartlet-flow-inner",
                }));
            }
        }
        Shape::Start | Shape::End => {
            let class = if kind == Shape::Start {
                "chartlet-flow-node chartlet-flow-start chartlet-role-green"
            } else {
                "chartlet-flow-node chartlet-flow-end chartlet-role-green"
            };
            with_shadow(shape(pill(x, y, width, height), class, tooltip), elements);
        }
        Shape::Decision => with_shadow(
            shape(
                vec![
                    (middle_x, y),
                    (x + width, middle_y),
                    (middle_x, y + height),
                    (x, middle_y),
                    (middle_x, y),
                ],
                "chartlet-flow-node chartlet-role-amber",
                tooltip,
            ),
            elements,
        ),
        Shape::Io => with_shadow(
            shape(
                vec![
                    (x + 10.0, y),
                    (x + width, y),
                    (x + width - 10.0, y + height),
                    (x, y + height),
                    (x + 10.0, y),
                ],
                "chartlet-flow-node chartlet-role-violet",
                tooltip,
            ),
            elements,
        ),
        Shape::Store => cylinder(
            (x, y, width, height),
            (
                "chartlet-flow-node chartlet-role-teal",
                "chartlet-diagram-rim",
            ),
            tooltip,
            elements,
        ),
        Shape::State | Shape::Final | Shape::Initial => {
            state_shape(kind, (x, y, width, height), tooltip, elements);
        }
        Shape::Person
        | Shape::Frontend
        | Shape::Queue
        | Shape::Bucket
        | Shape::Cache
        | Shape::Shield => {
            component_shape(kind, (x, y, width, height), tooltip, elements);
        }
    }
}

/// The shapes of a state diagram: a state, a final state with its double outline, and the
/// initial dot.
fn state_shape(
    kind: Shape,
    (x, y, width, height): (f64, f64, f64, f64),
    tooltip: Option<String>,
    elements: &mut Vec<Element>,
) {
    let rect = |class: &'static str, tooltip: Option<String>| {
        Element::Rect(Rect {
            x,
            y,
            width,
            height,
            class,
            series_index: None,
            style_index: None,
            tooltip,
        })
    };
    match kind {
        Shape::State => with_shadow(
            rect(
                "chartlet-flow-node chartlet-state chartlet-role-blue",
                tooltip,
            ),
            elements,
        ),
        Shape::Final => {
            with_shadow(
                rect(
                    "chartlet-flow-node chartlet-state chartlet-role-green",
                    tooltip,
                ),
                elements,
            );
            elements.push(Element::Rect(Rect {
                x: x + 4.0,
                y: y + 4.0,
                width: width - 8.0,
                height: height - 8.0,
                class: "chartlet-state-inner",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
        Shape::Initial => elements.push(Element::Circle(Circle {
            cx: x + width / 2.0,
            cy: y + height / 2.0,
            radius: width / 2.0,
            class: "chartlet-state-initial",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip,
        })),
        _ => unreachable!("only the shapes of a state diagram come here"),
    }
}

/// The shapes of an architecture diagram beyond those of a flow chart.
fn component_shape(
    kind: Shape,
    (x, y, width, height): (f64, f64, f64, f64),
    tooltip: Option<String>,
    elements: &mut Vec<Element>,
) {
    let rect = |x: f64, y: f64, class: &'static str, tooltip: Option<String>| {
        Element::Rect(Rect {
            x,
            y,
            width,
            height,
            class,
            series_index: None,
            style_index: None,
            tooltip,
        })
    };
    let polygon = |points: Vec<(f64, f64)>, class: &'static str, tooltip: Option<String>| {
        Element::Polyline(diagram::polyline(points, class, tooltip))
    };
    let middle = y + height / 2.0;
    match kind {
        Shape::Person => {
            with_shadow(
                rect(x, y, "chartlet-flow-node chartlet-role-violet", tooltip),
                elements,
            );
            head(x + width / 2.0, y + 11.0, elements);
        }
        Shape::Frontend => {
            with_shadow(
                rect(x, y, "chartlet-flow-node chartlet-role-blue", tooltip),
                elements,
            );
            window_bar(x, y, width, elements);
        }
        Shape::Queue => {
            with_shadow(
                rect(
                    x + 4.0,
                    y - 4.0,
                    "chartlet-flow-node chartlet-role-violet",
                    None,
                ),
                elements,
            );
            with_shadow(
                rect(x, y, "chartlet-flow-node chartlet-role-violet", tooltip),
                elements,
            );
        }
        Shape::Bucket => with_shadow(
            polygon(
                vec![
                    (x, y),
                    (x + width, y),
                    (x + width - 8.0, y + height),
                    (x + 8.0, y + height),
                    (x, y),
                ],
                "chartlet-flow-node chartlet-role-teal",
                tooltip,
            ),
            elements,
        ),
        Shape::Cache => with_shadow(
            polygon(
                vec![
                    (x + 12.0, y),
                    (x + width - 12.0, y),
                    (x + width, middle),
                    (x + width - 12.0, y + height),
                    (x + 12.0, y + height),
                    (x, middle),
                    (x + 12.0, y),
                ],
                "chartlet-flow-node chartlet-role-amber",
                tooltip,
            ),
            elements,
        ),
        Shape::Shield => with_shadow(
            polygon(
                vec![
                    (x, y),
                    (x + width, y),
                    (x + width, y + height * 0.6),
                    (x + width / 2.0, y + height),
                    (x, y + height * 0.6),
                    (x, y),
                ],
                "chartlet-flow-node chartlet-role-red",
                tooltip,
            ),
            elements,
        ),
        _ => unreachable!("only the shapes of an architecture diagram come here"),
    }
}

/// The head on top of a person.
fn head(x: f64, y: f64, elements: &mut Vec<Element>) {
    elements.push(Element::Circle(Circle {
        cx: x,
        cy: y,
        radius: 6.0,
        class: "chartlet-arch-head",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
}

/// The window bar along the top of a frontend: a line and three dots.
fn window_bar(x: f64, y: f64, width: f64, elements: &mut Vec<Element>) {
    elements.push(Element::Line(Line {
        x1: x,
        y1: y + 12.0,
        x2: x + width,
        y2: y + 12.0,
        class: "chartlet-flow-inner",
    }));
    for dot in 0..3 {
        elements.push(Element::Circle(Circle {
            cx: x + 8.0 + 6.0 * f64::from(dot),
            cy: y + 6.0,
            radius: 1.8,
            class: "chartlet-arch-dot",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
}

/// A box with fully rounded ends.
fn pill(x: f64, y: f64, width: f64, height: f64) -> Vec<(f64, f64)> {
    const STEPS: u32 = 8;
    let radius = height / 2.0;
    let arc = |center: f64, from: f64| -> Vec<(f64, f64)> {
        (0..=STEPS)
            .map(|step| {
                let angle = from + std::f64::consts::PI * f64::from(step) / f64::from(STEPS);
                (
                    center + radius * angle.cos(),
                    y + radius + radius * angle.sin(),
                )
            })
            .collect()
    };
    let half = std::f64::consts::FRAC_PI_2;
    let mut points = arc(x + width - radius, -half);
    points.extend(arc(x + radius, half));
    points.push(points[0]);
    points
}

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

/// Where each step leads, as `label` or `label (edge label)`, in the order of the edges.
fn next_steps(flow: &FlowSpec, ends: &[(usize, usize)], node: usize) -> Vec<String> {
    flow.edges
        .iter()
        .zip(ends)
        .filter(|(_, (from, _))| *from == node)
        .map(|(edge, (_, to))| match &edge.label {
            Some(label) => format!("{} ({label})", flow.nodes[*to].label),
            None => flow.nodes[*to].label.clone(),
        })
        .collect()
}

/// The chart in sentences: its size and lanes, the main path, every step in reading order with
/// where it leads, and the groups.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let flow = flow(spec);
    let locale = spec.locale;
    let ends = flow.ends();
    let lanes: Vec<String> = flow.lanes.iter().map(|lane| lane.label.clone()).collect();
    let mut description = text::flow_opening(locale, flow.nodes.len(), flow.edges.len(), &lanes);
    if !flow.main_path.is_empty() {
        let path: Vec<&str> = flow
            .main_path
            .iter()
            .map(|id| flow.nodes[flow.node(id).expect("validated")].label.as_str())
            .collect();
        description.push(' ');
        description.push_str(&text::main_path(locale, &path));
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
                &next_steps(flow, &ends, node)
            )
        )
        .expect("writing to String cannot fail");
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
    description
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
    columns.push(words.leads_to.to_owned());
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
            row.push(next_steps(flow, &ends, node).join(", "));
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
mod tests;

#[cfg(test)]
mod track_tests {
    use super::{Piece, assign_tracks};

    fn piece(edge: usize, from: f64, to: f64) -> Piece {
        Piece {
            edge,
            layer: 0,
            from,
            to,
            track: None,
        }
    }

    #[test]
    fn edges_turning_the_same_way_take_tracks_that_do_not_cross() {
        // The left edge leaves at 561 and turns right at its track; the right edge leaves at 584,
        // under the left one's run, so it must turn first: its start would cross the left run
        // otherwise.
        let pieces = [piece(0, 561.0, 703.0), piece(1, 584.0, 860.0)];
        let tracks = assign_tracks(&pieces, &[0, 1]);
        let track = |index: usize| tracks.iter().find(|(piece, _)| *piece == index).unwrap().1;
        assert!(track(1) < track(0), "{tracks:?}");
    }

    #[test]
    fn pieces_that_do_not_overlap_share_a_track() {
        let pieces = [piece(0, 10.0, 50.0), piece(1, 100.0, 140.0)];
        let tracks = assign_tracks(&pieces, &[0, 1]);
        assert!(tracks.iter().all(|(_, track)| *track == 0), "{tracks:?}");
    }
}
