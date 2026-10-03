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
        self, CHIP_REACH, HEAD, arrowhead, chip, cylinder, pixels, rounded, warn_growth,
        with_shadow, wrap,
    },
    error::ChartWarning,
    layout::{count, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Rect, Scene, Text, TextAnchor},
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
    let top = 56.0 + title_extra(spec, width - 2.0 * MARGIN, metrics);
    let model = Model::new(diagram, metrics, warnings);
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
        MARGIN,
        width - 2.0 * MARGIN,
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
    pub dash: Option<Dash>,
    /// Whether the edge belongs to the main path.
    pub main: bool,
    /// Where the edge's label stands in the specification, for warnings about it.
    pub label_path: String,
}

pub(crate) struct Group {
    pub label: String,
    pub path: String,
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
                })
                .collect(),
            orientation: flow.orientation,
        }
    }

    fn ends(&self) -> Vec<(usize, usize)> {
        self.edges.iter().map(|edge| (edge.from, edge.to)).collect()
    }
}

/// What both orientations need: the layered graph, and every text measured and wrapped.
struct Model<'a> {
    diagram: &'a Diagram,
    ends: Vec<(usize, usize)>,
    graph: Graph,
    /// Each step's label on one or two lines, and its box: width and height on the page.
    labels: Vec<Vec<String>>,
    sizes: Vec<(f64, f64)>,
    /// Each edge's label on one or two lines, and the widest of them.
    edge_labels: Vec<Vec<String>>,
    edge_widths: Vec<f64>,
    /// The labels of the edges from each step to itself, joined.
    loops: Vec<Option<String>>,
}

impl<'a> Model<'a> {
    fn new(
        diagram: &'a Diagram,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
    ) -> Self {
        let ends = diagram.ends();
        let graph = Graph::new(diagram);
        let mut labels = Vec::with_capacity(diagram.nodes.len());
        let mut sizes = Vec::with_capacity(diagram.nodes.len());
        for node in &diagram.nodes {
            if node.shape == Shape::Initial {
                labels.push(Vec::new());
                sizes.push((INITIAL, INITIAL));
                continue;
            }
            let (extra_width, extra_height) = match node.shape {
                Shape::Decision => (48.0, 24.0),
                Shape::Io => (20.0, 0.0),
                Shape::Subprocess => (16.0, 0.0),
                Shape::Store => (0.0, 10.0),
                Shape::Final => (8.0, 8.0),
                _ => (0.0, 0.0),
            };
            let room = MAX_NODE - 2.0 * PAD;
            let path = format!("{}/label", node.path);
            let lines = wrap(&node.label, room, LABEL_SIZE, metrics, warnings, &path);
            let text = lines
                .iter()
                .map(|line| metrics.width(line, LABEL_SIZE))
                .chain(
                    node.sublabel
                        .iter()
                        .map(|sublabel| metrics.width(sublabel, SUBLABEL_SIZE)),
                )
                .fold(0.0, f64::max);
            let width = (text + 2.0 * PAD).clamp(MIN_NODE, MAX_NODE) + extra_width;
            let height = 20.0
                + NODE_LINE * count(lines.len())
                + if node.sublabel.is_some() { 14.0 } else { 0.0 }
                + extra_height;
            labels.push(lines);
            sizes.push((width, height));
        }
        let mut edge_labels = Vec::with_capacity(diagram.edges.len());
        let mut edge_widths = Vec::with_capacity(diagram.edges.len());
        for edge in &diagram.edges {
            let lines = edge.label.as_ref().map_or_else(Vec::new, |label| {
                wrap(
                    label,
                    EDGE_LABEL,
                    EDGE_SIZE,
                    metrics,
                    warnings,
                    &edge.label_path,
                )
            });
            edge_widths.push(
                lines
                    .iter()
                    .map(|line| metrics.width(line, EDGE_SIZE))
                    .fold(0.0, f64::max),
            );
            edge_labels.push(lines);
        }
        let loops = (0..diagram.nodes.len())
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
        Self {
            diagram,
            ends,
            graph,
            labels,
            sizes,
            edge_labels,
            edge_widths,
            loops,
        }
    }
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
                NODE_GAP
            } else {
                PASSING_GAP
            };
            let grouped =
                first.group != second.group && (first.group.is_some() || second.group.is_some());
            cross_size(a) / 2.0
                + cross_size(b) / 2.0
                + base
                + after(a)
                + if grouped { GROUP_GAP } else { 0.0 }
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
            graph
                .items
                .iter()
                .enumerate()
                .map(|(item, _)| cross[item] + cross_size(item) / 2.0 + after(item))
                .fold(0.0, f64::max)
        } else {
            bands.last().map_or(0.0, |band| band.1)
        };
        let (main_room, cross_room) = rooms(spec, landscape, top);
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
        let ports = Ports::new(graph, &plan.cross, &cross_size, model, landscape);
        let lead = if !landscape && !diagram.lanes.is_empty() {
            LANE_HEAD
        } else {
            0.0
        };
        let gaps = Gaps::new(model, &plan, &ports, landscape);
        let natural = gaps.length(&main_size, graph, lead);
        let stretch = if natural < main_room && gaps.total() > 0.0 {
            (1.0 + (main_room - natural) / gaps.total()).min(MAX_STRETCH)
        } else {
            1.0
        };
        plan.main_length = gaps.place(&mut plan, &main_size, graph, lead, stretch);
        plan.routes = route(model, &plan, &ports, &gaps, stretch);
        plan.finish(spec, top, (main_room, cross_room), (natural, cross_needed));
        plan
    }

    /// Places the layout on the page, in the middle of the room the canvas leaves, and works out
    /// the size of the canvas: as given, or larger where the diagram needs it.
    fn finish(
        &mut self,
        spec: &ChartSpec,
        top: f64,
        (main_room, cross_room): (f64, f64),
        (main_needed, cross_needed): (f64, f64),
    ) {
        let (page_width, page_height) = (f64::from(spec.width), f64::from(spec.height));
        let landscape = self.landscape;
        self.main_origin = (if landscape { MARGIN } else { top })
            + ((main_room - self.main_length) / 2.0).max(0.0);
        self.cross_origin = (if landscape { top } else { MARGIN })
            + if self.bands.is_empty() {
                ((cross_room - self.cross_length) / 2.0).max(0.0)
            } else {
                0.0
            };
        (self.width, self.height) = if landscape {
            (
                page_width.max(2.0 * MARGIN + main_needed),
                page_height.max(top + cross_needed + BOTTOM),
            )
        } else {
            (
                page_width.max(2.0 * MARGIN + cross_needed),
                page_height.max(top + main_needed + BOTTOM),
            )
        };
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

/// The room the canvas leaves the layout on the main and the cross axis.
fn rooms(spec: &ChartSpec, landscape: bool, top: f64) -> (f64, f64) {
    let across = f64::from(spec.width) - 2.0 * MARGIN;
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
        .map(|label| match label {
            None => 0.0,
            Some(_) if landscape => LOOP + 18.0,
            Some(label) => LOOP + 8.0 + metrics.width(label, EDGE_SIZE).min(EDGE_LABEL),
        })
        .collect()
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
                ) || align(
                    graph,
                    cross,
                    pair[0],
                    cross[pair[1]],
                    cross_size,
                    gap,
                    bands,
                    lane_head,
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
                edges.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
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
                let room = if landscape {
                    model.edge_widths[edge] + 26.0
                } else {
                    LINE * count(lines.len()) + 16.0
                };
                if chain.reversed {
                    let gap = graph.layer[items[items.len() - 1]] - 1;
                    label_room_end[gap] = f64::max(label_room_end[gap], room);
                } else {
                    let gap = graph.layer[items[0]];
                    label_room[gap] = f64::max(label_room[gap], room);
                }
            }
        }
        let mut tracks = vec![0; gaps];
        for (gap, tracks) in tracks.iter_mut().enumerate() {
            let mut ends: Vec<f64> = Vec::new();
            let mut bending: Vec<usize> = (0..pieces.len())
                .filter(|index| {
                    pieces[*index].layer == gap
                        && (pieces[*index].from - pieces[*index].to).abs() > 0.5
                })
                .collect();
            bending.sort_by(|a, b| {
                let low = |piece: &Piece| piece.from.min(piece.to);
                low(&pieces[*a])
                    .total_cmp(&low(&pieces[*b]))
                    .then(pieces[*a].edge.cmp(&pieces[*b].edge))
            });
            for index in bending {
                let piece = &mut pieces[index];
                let (low, high) = (piece.from.min(piece.to), piece.from.max(piece.to));
                let track = ends
                    .iter()
                    .position(|end| *end + 10.0 < low)
                    .unwrap_or_else(|| {
                        ends.push(f64::NEG_INFINITY);
                        ends.len() - 1
                    });
                ends[track] = high;
                piece.track = Some(track);
            }
            *tracks = ends.len();
        }
        let mut frame_after = vec![0.0; layers];
        let mut frame_before = vec![0.0; layers];
        for group in 0..model.diagram.groups.len() {
            let members: Vec<usize> = (0..model.diagram.nodes.len())
                .filter(|node| model.diagram.nodes[*node].group == Some(group))
                .map(|node| graph.layer[node])
                .collect();
            let (first, last) = (
                members.iter().copied().min().unwrap_or(0),
                members.iter().copied().max().unwrap_or(0),
            );
            let head = if landscape {
                GROUP_PAD
            } else {
                GROUP_PAD + GROUP_HEAD
            };
            frame_before[first] = f64::max(frame_before[first], head);
            frame_after[last] = f64::max(frame_after[last], GROUP_PAD);
        }
        Self {
            pieces,
            label_room,
            label_room_end,
            tracks,
            frame_after,
            frame_before,
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
    fn length(&self, main_size: &impl Fn(usize) -> f64, graph: &Graph, lead: f64) -> f64 {
        let depths: f64 = graph
            .layers
            .iter()
            .map(|layer| depth(graph, layer, main_size))
            .sum();
        lead + self.frame_before.first().copied().unwrap_or(0.0)
            + depths
            + self.total()
            + self.frame_after.last().copied().unwrap_or(0.0)
    }

    /// Places the layers on the main axis and returns their length.
    fn place(
        &self,
        plan: &mut Plan,
        main_size: &impl Fn(usize) -> f64,
        graph: &Graph,
        lead: f64,
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
        at + self.frame_after.last().copied().unwrap_or(0.0)
    }

    /// The main position of track `track` in gap `gap`, which begins at `start`.
    fn track(&self, gap: usize, track: usize, start: f64, stretch: f64) -> f64 {
        start
            + (self.frame_after[gap] + self.label_room[gap] + GAP / 2.0 + count(track) * TRACK)
                * stretch
    }
}

/// How deep a layer is on the main axis: as deep as its deepest step.
fn depth(graph: &Graph, layer: &[usize], main_size: &impl Fn(usize) -> f64) -> f64 {
    layer
        .iter()
        .filter_map(|item| graph.items[*item].node)
        .map(main_size)
        .fold(0.0, f64::max)
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
        for edge in 0..model.diagram.edges.len() {
            self.draw_edge(model, edge, metrics, elements);
        }
        for node in 0..model.diagram.nodes.len() {
            self.draw_loop(model, node, metrics, elements);
        }
        for node in 0..model.diagram.nodes.len() {
            self.draw_node(model, node, metrics, warnings, elements);
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

    fn draw_groups(
        &self,
        model: &Model,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
        elements: &mut Vec<Element>,
    ) {
        let diagram = model.diagram;
        for (index, group) in diagram.groups.iter().enumerate() {
            let members: Vec<usize> = (0..diagram.nodes.len())
                .filter(|node| diagram.nodes[*node].group == Some(index))
                .collect();
            let boxes: Vec<(f64, f64, f64, f64)> = members
                .iter()
                .map(|node| self.node_box(model, *node))
                .collect();
            let left = boxes.iter().map(|b| b.0).fold(f64::INFINITY, f64::min) - GROUP_PAD;
            let top =
                boxes.iter().map(|b| b.1).fold(f64::INFINITY, f64::min) - GROUP_PAD - GROUP_HEAD;
            let right = boxes
                .iter()
                .map(|b| b.0 + b.2)
                .fold(f64::NEG_INFINITY, f64::max)
                + GROUP_PAD;
            let bottom = boxes
                .iter()
                .map(|b| b.1 + b.3)
                .fold(f64::NEG_INFINITY, f64::max)
                + GROUP_PAD;
            let intruder = (0..diagram.nodes.len())
                .filter(|node| !members.contains(node))
                .map(|node| self.node_box(model, node))
                .any(|(x, y, width, height)| {
                    x < right && x + width > left && y < bottom && y + height > top
                });
            if intruder {
                warnings.push(ChartWarning::new(
                    "group_overlap",
                    group.path.clone(),
                    "a step outside the group lies inside its frame; move it to another lane or regroup the steps",
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
            let path = format!("{}/label", group.path);
            elements.push(Element::Text(Text {
                x: left + 8.0,
                y: top + 13.0,
                class: "chartlet-flow-group-label",
                anchor: TextAnchor::Start,
                content: crate::layout::fit_text(
                    &group.label,
                    right - left - 16.0,
                    SUBLABEL_SIZE,
                    metrics,
                    warnings,
                    &path,
                ),
            }));
        }
    }

    fn draw_edge(
        &self,
        model: &Model,
        edge: usize,
        metrics: &impl TextMetrics,
        elements: &mut Vec<Element>,
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
        let tooltip = match &spec.label {
            Some(label) => format!(
                "{} → {}: {label}",
                diagram.nodes[from].label, diagram.nodes[to].label
            ),
            None => format!(
                "{} → {}",
                diagram.nodes[from].label, diagram.nodes[to].label
            ),
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
        chip(lines, (x, y), anchor, metrics, elements);
    }

    /// An edge from a step to itself: a loop beside the step, right in portrait and below in
    /// landscape, with the labels of all such edges joined.
    fn draw_loop(
        &self,
        model: &Model,
        node: usize,
        metrics: &impl TextMetrics,
        elements: &mut Vec<Element>,
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
        if !label.is_empty() {
            chip(
                std::slice::from_ref(label),
                (text.0, text.1),
                text.2,
                metrics,
                elements,
            );
        }
    }

    fn draw_node(
        &self,
        model: &Model,
        node: usize,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
        elements: &mut Vec<Element>,
    ) {
        let spec = &model.diagram.nodes[node];
        let (x, y, width, height) = self.node_box(model, node);
        let tooltip = Some(match &spec.sublabel {
            Some(sublabel) => format!("{} – {sublabel}", spec.label),
            None => spec.label.clone(),
        });
        shape(spec.shape, (x, y, width, height), tooltip, elements);
        Self::node_text(
            model,
            node,
            (x, y, width, height),
            metrics,
            warnings,
            elements,
        );
    }

    /// A step's label, centered on one or two lines, and its sublabel below.
    fn node_text(
        model: &Model,
        node: usize,
        (x, y, width, height): (f64, f64, f64, f64),
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
        elements: &mut Vec<Element>,
    ) {
        let spec = &model.diagram.nodes[node];
        // The initial dot carries no text; its name is its tooltip.
        if spec.shape == Shape::Initial {
            return;
        }
        let (middle_x, middle_y) = (x + width / 2.0, y + height / 2.0);
        let lines = &model.labels[node];
        let sublabel = spec.sublabel.is_some();
        let block = NODE_LINE * count(lines.len() - 1) + if sublabel { 14.0 } else { 0.0 };
        let shift = if spec.shape == Shape::Store { 3.0 } else { 0.0 };
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
        if let Some(sublabel) = &spec.sublabel {
            let room = width - 2.0 * PAD;
            let path = format!("{}/sublabel", spec.path);
            elements.push(Element::Text(Text {
                x: middle_x,
                y: first + NODE_LINE * count(lines.len() - 1) + 14.0,
                class: "chartlet-flow-sublabel",
                anchor: TextAnchor::Middle,
                content: crate::layout::fit_text(
                    sublabel,
                    room,
                    SUBLABEL_SIZE,
                    metrics,
                    warnings,
                    &path,
                ),
            }));
        }
    }
}

/// The shape of a step of `kind` in the box at `x`, `y`, with its shadow: every kind has its own,
/// and its role color only repeats what the shape says.
fn shape(
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
