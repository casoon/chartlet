//! The way of every edge from step to step, and where its label goes.

use super::{
    BALL, SOCKET,
    gaps::Gaps,
    measure::Model,
    model::{End, Shape},
    place::Plan,
    ports::Ports,
};

/// An edge's way on the main and cross axes, and where its label goes.
pub(super) struct Route {
    pub(super) points: Vec<(f64, f64)>,
    /// Where the label is written: where the edge leaves the step it comes from.
    pub(super) label_at: (f64, f64),
    /// Whether the edge leaves that step against the direction of the flow: an edge the layout
    /// turned around, whose label is then written before the step instead of after it.
    pub(super) against: bool,
    /// Whether the label goes on the side of larger cross positions: the side of its port away
    /// from the middle of the step, so that it keeps clear of the step's other edges.
    pub(super) after: bool,
    /// How far the label moves further to that side, past the other edges that meet the same
    /// side of the step there.
    pub(super) clear: f64,
}

/// How far edge `edge` of step `node` ends from its middle on the main axis, at `offset` across:
/// half its depth, less towards the corners of a diamond; at an interface, its socket or its
/// ball.
fn edge_depth(plan: &Plan, model: &Model, (node, edge): (usize, usize), offset: f64) -> f64 {
    if model.diagram.nodes[node].shape == Shape::Interface {
        return if model.diagram.edges[edge].ends.head == End::Socket {
            SOCKET
        } else {
            BALL
        };
    }
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

/// Where an edge meets step `node`, whose middle lies at `middle` on the main axis, at `offset`
/// across: on its far side when `leaving`, on its near side otherwise. An edge that meets the step
/// across the flow, at `along`, meets the side of the step and runs along the end of its stub: the
/// two points in the order the edge passes them.
fn port_points(
    plan: &Plan,
    model: &Model,
    (node, edge, middle): (usize, usize, f64),
    (offset, along, into): (f64, Option<f64>, Option<f64>),
    leaving: bool,
) -> Vec<(f64, f64)> {
    let cross = plan.cross[node];
    let Some(along) = along else {
        let depth = edge_depth(plan, model, (node, edge), offset);
        let main = if leaving {
            middle + depth
        } else {
            middle - depth
        };
        // Into a block, the edge runs on to the side of the node it joins there.
        return match (into, leaving) {
            (None, _) => vec![(main, cross + offset)],
            (Some(into), true) => vec![(middle + into, cross + offset), (main, cross + offset)],
            (Some(into), false) => vec![(main, cross + offset), (middle + into, cross + offset)],
        };
    };
    let side = side_depth(plan, model, node, along).copysign(offset);
    let main = middle + along;
    let mut points = vec![(main, cross + side), (main, cross + offset)];
    if !leaving {
        points.reverse();
    }
    points
}

/// How far the side of step `node` lies from its middle on the cross axis, at `along` on the main
/// axis: half its width across the flow, less towards the corners of a diamond.
fn side_depth(plan: &Plan, model: &Model, node: usize, along: f64) -> f64 {
    let (width, height) = plan.sizes[node];
    let (main, cross) = if plan.landscape {
        (width, height)
    } else {
        (height, width)
    };
    if model.diagram.nodes[node].shape == Shape::Decision {
        cross / 2.0 * (1.0 - 2.0 * along.abs() / main).max(0.0)
    } else {
        cross / 2.0
    }
}

/// The way of every edge: out of its first step, along each piece and its track, into its last
/// step; an edge the layout turned around runs back the other way.
pub(super) fn route(
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
            // An edge that meets its step across the flow leaves the side of the step, and runs
            // along the flow on its stub beside it; its label goes beside the stub.
            let (leave, reach) = (ports.leave[edge], ports.reach[edge]);
            let leaving = (leave, ports.leave_along[edge], ports.leave_into[edge]);
            let mut points = port_points(plan, model, (first, edge, middle(first)), leaving, true);
            let start = points[points.len() - 1];
            for piece in gaps.pieces.iter().filter(|piece| piece.edge == edge) {
                if let Some(track) = piece.track {
                    let main = gaps.track(piece.layer, track, plan.layers[piece.layer].1, stretch);
                    points.push((main, piece.from));
                    points.push((main, piece.to));
                }
            }
            let reaching = (reach, ports.reach_along[edge], ports.reach_into[edge]);
            let arrival = port_points(plan, model, (last, edge, middle(last)), reaching, false);
            let end = arrival[0];
            points.extend(arrival);
            let (label_at, offset) = if chain.reversed {
                (end, reach)
            } else {
                (start, leave)
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
