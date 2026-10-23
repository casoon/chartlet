//! Flow charts: steps joined by arrows, laid out in layers along the direction of the flow —
//! down in portrait, right in landscape — optionally in lanes across it and framed in groups.
//!
//! Positions are worked out along two axes, the main axis of the flow and the cross axis of the
//! layers, and turned into page coordinates only when drawn, so that both orientations share one
//! layout. Edges leave a step on its far side, change course on a track of their own between
//! two layers, and arrive on the near side of the next; an edge the layout turned around to break
//! a cycle is drawn back the other way.

mod blocks;
mod describe;
mod draw;
mod edges;
mod frames;
mod gaps;
mod graph;
mod measure;
mod model;
mod nodes;
pub(crate) mod notes;
mod place;
mod ports;
mod route;
mod shapes;
mod straighten;
mod trunks;

use crate::{
    diagram::{self, pixels, warn_growth},
    error::ChartWarning,
    heat,
    layout::{count, push_title, title_extra},
    metrics::TextMetrics,
    parts, roles,
    scene::Scene,
    spec::{ChartSpec, DiagramOrientation, FlowSpec},
};
pub(crate) use blocks::{Block, Whole, blocks_reachable};
pub(crate) use describe::{data_table, description, heaviest_edge, reading_order};
pub(crate) use draw::{focus_classes, weight_widths_of};
use graph::Graph;
use measure::{Model, Spacing};
pub(crate) use model::{Diagram, Edge, End, Ends, Group, Node, Shape, Sides};
use place::Plan;
pub(crate) use shapes::shape;
use trunks::Trunks;

pub(crate) const MARGIN: f64 = 24.0;
const BOTTOM: f64 = 16.0;
const LABEL_SIZE: f64 = 13.0;
const SUBLABEL_SIZE: f64 = 11.0;
const EDGE_SIZE: f64 = 12.0;
/// Distance between the baselines of an edge label on two lines, and of a step's label.
const LINE: f64 = 15.0;
const NODE_LINE: f64 = 16.0;
/// The size, line height and padding of a member of a class box.
const MEMBER_SIZE: f64 = 12.0;
const MEMBER_LINE: f64 = 15.0;
const MEMBER_PAD: f64 = 6.0;
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
/// Radius of the dot where a shared trunk branches.
const BRANCH: f64 = 3.0;
/// The ball of a provided interface, the socket around it, the gap to its label, and the size
/// and line height of the label.
const BALL: f64 = 6.0;
const SOCKET: f64 = 10.0;
const INTERFACE_GAP: f64 = 6.0;
const INTERFACE_SIZE: f64 = 11.0;
const INTERFACE_LINE: f64 = 13.0;
/// The room around the name of an interface on a chip below its ball, on a narrow canvas.
const INTERFACE_PAD: f64 = 4.0;

/// The number of width steps above the thinnest line of a weighted edge: every step adds half a
/// pixel, from 1 px for no weight to 5 px for the heaviest edge.
pub(crate) const WEIGHT_STEPS: f64 = 8.0;

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let flow = flow(spec);
    let mut diagram = Diagram::from_flow(flow);
    notes::push_notes(&mut diagram, &flow.notes, "flow", |id| {
        notes::Target::Node(flow.node(id).expect("validated"))
    });
    let mut scene = layout_with_trunks(spec, &diagram, flow.shared_trunks, warnings, metrics);
    notes::push_band(&mut scene, spec, (&flow.notes, "flow"), metrics, warnings);
    let values: Vec<Option<f64>> = flow.nodes.iter().map(|node| node.value).collect();
    heat::push_key(
        &mut scene,
        spec,
        (&heat::name(spec, flow.value_label.as_deref()), &values),
        MARGIN,
        metrics,
    );
    let names = parts::names(flow.nodes.iter().map(|node| node.parts.as_slice()));
    parts::push_legend(&mut scene, &names, MARGIN, metrics);
    let used = roles::used(
        flow.nodes.iter().map(|node| node.role),
        flow.edges.iter().map(|edge| edge.role),
    );
    roles::push_legend(
        &mut scene,
        spec.locale,
        &used,
        (MARGIN, roles::push_badge),
        metrics,
    );
    scene
}

/// Lays out and draws `diagram`, whatever specification it comes from.
pub(crate) fn layout_diagram(
    spec: &ChartSpec,
    diagram: &Diagram,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    layout_with_trunks(spec, diagram, false, warnings, metrics)
}

/// [`layout_diagram`], with the edges that leave or reach a step together drawn on shared trunks
/// when `shared_trunks` (see [`trunks`]).
pub(crate) fn layout_with_trunks(
    spec: &ChartSpec,
    diagram: &Diagram,
    shared_trunks: bool,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let spacing = Spacing::new(spec.width, diagram.spacing);
    let margin = spacing.margin;
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let mut model = Model::new(diagram, (spacing, spec.number_style()), metrics, warnings);
    if shared_trunks {
        model.trunks = Trunks::of(diagram, &model.graph);
    }
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
    // The edge labels are placed last and may reach beyond what the layers need. The canvas then
    // grows to hold them, and the layers spread over the wider canvas, which moves the labels
    // again: so the layout is repeated at the larger size until the labels fit. The size it ends
    // at is the size the warning names, and laying the diagram out at that size changes nothing.
    let mut canvas = spec.clone();
    for round in 1..=GROWTH_ROUNDS {
        let plan = Plan::new(&canvas, &model, landscape, top, metrics);
        let mut elements = Vec::new();
        let mut round_warnings = Vec::new();
        push_title(
            &mut elements,
            &canvas,
            margin,
            f64::from(canvas.width) - 2.0 * margin,
            metrics,
            &mut round_warnings,
        );
        let (labels_right, labels_bottom) =
            plan.draw(&model, metrics, &mut round_warnings, &mut elements);
        let (width, height) = (
            plan.width.max((labels_right + 2.0).ceil()),
            plan.height.max((labels_bottom + 2.0).ceil()),
        );
        if width <= plan.width && height <= plan.height || round == GROWTH_ROUNDS {
            warnings.append(&mut round_warnings);
            warn_growth(spec, width, height, warnings);
            return Scene {
                width: pixels(width),
                height: pixels(height),
                elements,
            };
        }
        canvas.width = pixels(width);
        canvas.height = pixels(height);
    }
    unreachable!("the last round returns")
}

/// How often a diagram is laid out again on a canvas grown to hold its edge labels.
const GROWTH_ROUNDS: usize = 24;

fn flow(spec: &ChartSpec) -> &FlowSpec {
    spec.flow
        .as_ref()
        .expect("validated flow charts carry a flow block")
}

#[cfg(test)]
mod tests;
