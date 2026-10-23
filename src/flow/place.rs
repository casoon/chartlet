//! One orientation of the layout: the steps placed across the flow, layer by layer and inside
//! their lanes, and the layers along it.

use super::{
    BALL, BOTTOM, EDGE_SIZE, INTERFACE_GAP, INTERFACE_LINE, LANE_HEAD, LANE_PAD, LOOP,
    SUBLABEL_SIZE,
    frames::{clear_frames, frames_reach},
    gaps::Gaps,
    graph::{Graph, settle},
    measure::Model,
    model::{Diagram, Shape},
    ports::Ports,
    route::{Route, route},
    straighten::straighten,
};
use crate::{
    diagram::{CHIP_LINE, CHIP_REACH},
    layout::count,
    metrics::TextMetrics,
    spec::ChartSpec,
};

/// One orientation of the layout, in main and cross coordinates.
pub(super) struct Plan {
    pub(super) landscape: bool,
    /// Each step's box on the page: width and height.
    pub(super) sizes: Vec<(f64, f64)>,
    /// The cross position of every item.
    pub(super) cross: Vec<f64>,
    /// The start and end of every layer on the main axis.
    pub(super) layers: Vec<(f64, f64)>,
    /// The lanes on the cross axis.
    pub(super) bands: Vec<(f64, f64)>,
    /// One route per edge; `None` for an edge from a step to itself.
    pub(super) routes: Vec<Option<Route>>,
    /// How far the layers reach on the main axis, and the content on the cross axis.
    pub(super) main_length: f64,
    cross_length: f64,
    /// Where the main and cross axes begin on the page.
    main_origin: f64,
    cross_origin: f64,
    pub(super) width: f64,
    pub(super) height: f64,
}

impl Plan {
    #[allow(clippy::too_many_lines)]
    pub(super) fn new(
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
        // Beside a step, the room of its loop and of the edges that meet it across the flow.
        let after = |item: usize| {
            graph.items[item]
                .node
                .map_or(0.0, |node| loop_room[node] + model.side_room[node].1)
        };
        let before = |item: usize| {
            graph.items[item]
                .node
                .map_or(0.0, |node| model.side_room[node].0)
        };
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
                + before(b)
                + model.spacing.group_gap * frames
        };
        let lane_head = if landscape { LANE_HEAD } else { 0.0 };
        let bands = lane_bands(
            diagram,
            graph,
            &cross_size,
            &gap,
            (&before, &after),
            lane_head,
            metrics,
        );
        let cross = place(graph, &cross_size, &gap, &before, &bands, lane_head);
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
        let mut cross_needed = if plan.bands.is_empty() && !diagram.groups.is_empty() {
            plan.keep_frames_clear(model, &cross_size, &gap, &after)
        } else {
            cross_needed
        };
        if plan.bands.is_empty() && model.side_room.iter().any(|room| room.0 > 0.0) {
            cross_needed += plan.clear_low_sides(graph, &cross_size, &before);
        }
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

    /// Moves the content on along the cross axis where edges that meet the first steps across the
    /// flow, on their low side, reach before 0; returns how far.
    fn clear_low_sides(
        &mut self,
        graph: &Graph,
        cross_size: &impl Fn(usize) -> f64,
        before: &impl Fn(usize) -> f64,
    ) -> f64 {
        let low = (0..graph.items.len())
            .map(|item| self.cross[item] - cross_size(item) / 2.0 - before(item))
            .fold(0.0, f64::min);
        for position in &mut self.cross {
            *position -= low;
        }
        self.cross_length -= low;
        -low
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
    pub(super) fn page(&self, (main, cross): (f64, f64)) -> (f64, f64) {
        if self.landscape {
            (self.main_origin + main, self.cross_origin + cross)
        } else {
            (self.cross_origin + cross, self.main_origin + main)
        }
    }

    /// The box of step `node` on the page: left, top, width, height.
    pub(super) fn node_box(&self, model: &Model, node: usize) -> (f64, f64, f64, f64) {
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
            // The name of an interface stands below its ball rather than beside it.
            if model.diagram.nodes[node].shape == Shape::Interface {
                if model.spacing.is_compact() {
                    continue;
                }
                let text = size.0 / 2.0 - BALL - INTERFACE_GAP;
                let lines = count(model.labels[node].len());
                *size = (
                    text.max(2.0 * BALL),
                    2.0 * (BALL + INTERFACE_GAP + INTERFACE_LINE * lines),
                );
                continue;
            }
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
    (before, after): (&impl Fn(usize) -> f64, &impl Fn(usize) -> f64),
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
                metrics.width(&lane.label, SUBLABEL_SIZE) + 2.0 * LANE_PAD
            }
        })
        .collect();
    if widths.is_empty() {
        return Vec::new();
    }
    for layer in &graph.layers {
        for segment in segments(graph, layer) {
            let span = before(segment[0])
                + cross_size(segment[0]) / 2.0
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
pub(super) fn place(
    graph: &Graph,
    cross_size: &impl Fn(usize) -> f64,
    gap: &impl Fn(usize, usize) -> f64,
    before: &impl Fn(usize) -> f64,
    bands: &[(f64, f64)],
    lane_head: f64,
) -> Vec<f64> {
    let mut cross = vec![0.0; graph.items.len()];
    let bounds = |segment: &[usize]| {
        graph.items[segment[0]].lane.map(|lane| {
            let band = bands[lane];
            (
                band.0 + LANE_PAD + lane_head + before(segment[0]) + cross_size(segment[0]) / 2.0,
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
