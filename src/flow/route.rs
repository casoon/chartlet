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
    let mut routes: Vec<Option<Route>> = graph
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
        .collect();
    separate_parallels(model, plan, &mut routes);
    routes
}

/// Runs along the flow of different edges that lie closer than this are moved apart.
const APART: f64 = 12.0;

/// Whether a straight run of `edge` over `span` at `fixed` lies side by side with a run of
/// another edge at less than [`APART`]: along the flow when `along` (fixed is a cross position),
/// otherwise across it (fixed is a main position).
fn crowded(
    routes: &[Option<Route>],
    edge: usize,
    along: bool,
    (low, high): (f64, f64),
    fixed: f64,
) -> bool {
    let on = |p: &(f64, f64)| if along { p.1 } else { p.0 };
    let over = |p: &(f64, f64)| if along { p.0 } else { p.1 };
    routes
        .iter()
        .enumerate()
        .filter(|(other, _)| *other != edge)
        .filter_map(|(_, route)| route.as_ref())
        .any(|route| {
            route.points.windows(2).any(|pair| {
                let distance = (on(&pair[0]) - fixed).abs();
                (on(&pair[0]) - on(&pair[1])).abs() < 1e-6
                    && distance > 0.01
                    && distance < APART - 1e-6
                    && over(&pair[0]).max(over(&pair[1])).min(high)
                        - over(&pair[0]).min(over(&pair[1])).max(low)
                        > 2.0
            })
        })
}

/// Whether two straight pieces of a way cross.
#[allow(clippy::many_single_char_names)]
fn crossing((a, b): ((f64, f64), (f64, f64)), (c, d): ((f64, f64), (f64, f64))) -> bool {
    let side = (b.0 - a.0) * (d.1 - c.1) - (b.1 - a.1) * (d.0 - c.0);
    if side.abs() < 1e-9 {
        return false;
    }
    let t = ((c.0 - a.0) * (d.1 - c.1) - (c.1 - a.1) * (d.0 - c.0)) / side;
    let u = ((c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)) / side;
    (0.001..0.999).contains(&t) && (0.001..0.999).contains(&u)
}

/// How often the way of `edge` crosses the ways of the other edges.
fn crossings(routes: &[Option<Route>], edge: usize) -> usize {
    let Some(route) = &routes[edge] else {
        return 0;
    };
    let mut total = 0;
    for (other, route_2) in routes.iter().enumerate() {
        let Some(route_2) = route_2.as_ref().filter(|_| other != edge) else {
            continue;
        };
        for first in route.points.windows(2) {
            for second in route_2.points.windows(2) {
                total += usize::from(crossing((first[0], first[1]), (second[0], second[1])));
            }
        }
    }
    total
}

/// Moves runs along the flow, between the first and the last point of an edge's way, away from
/// the runs of other edges that lie closer than [`APART`], by the least distance that does not
/// lead the edge through a step or across more edges than before.
#[allow(clippy::too_many_lines)]
fn separate_parallels(model: &Model, plan: &Plan, routes: &mut [Option<Route>]) {
    let graph = &model.graph;
    let boxes: Vec<[f64; 4]> = (0..model.diagram.nodes.len())
        .map(|node| {
            let layer = plan.layers[graph.layer[node]];
            let middle = f64::midpoint(layer.0, layer.1);
            let (width, height) = plan.sizes[node];
            let (main, cross) = if plan.landscape {
                (width, height)
            } else {
                (height, width)
            };
            [
                middle - main / 2.0,
                middle + main / 2.0,
                plan.cross[node] - cross / 2.0,
                plan.cross[node] + cross / 2.0,
            ]
        })
        .collect();
    let hits = |a: (f64, f64), b: (f64, f64)| {
        boxes.iter().any(|node| {
            a.0.max(b.0) > node[0] + 1.0
                && a.0.min(b.0) < node[1] - 1.0
                && a.1.max(b.1) > node[2] + 1.0
                && a.1.min(b.1) < node[3] - 1.0
        })
    };
    for _ in 0..3 {
        for edge in 0..routes.len() {
            let Some(length) = routes[edge].as_ref().map(|route| route.points.len()) else {
                continue;
            };
            for at in 1..length.saturating_sub(2) {
                let points = routes[edge].as_ref().expect("a route").points.clone();
                let (p, q) = (points[at], points[at + 1]);
                if (p.1 - q.1).abs() > 1e-6 || (p.0 - q.0).abs() < 2.0 {
                    continue;
                }
                let span = (p.0.min(q.0), p.0.max(q.0));
                if !crowded(routes, edge, true, span, p.1) {
                    continue;
                }
                let before = crossings(routes, edge);
                for step in 1..=6 {
                    for sign in [1.0, -1.0] {
                        let cross = p.1 + sign * 4.0 * f64::from(step);
                        let (a, b) = ((p.0, cross), (q.0, cross));
                        if crowded(routes, edge, true, span, cross)
                            || hits(a, b)
                            || hits(points[at - 1], a)
                            || hits(b, points[at + 2])
                        {
                            continue;
                        }
                        let route = routes[edge].as_mut().expect("a route");
                        route.points[at] = a;
                        route.points[at + 1] = b;
                        if crossings(routes, edge) > before {
                            let route = routes[edge].as_mut().expect("a route");
                            route.points[at] = p;
                            route.points[at + 1] = q;
                            continue;
                        }
                        break;
                    }
                    let moved = routes[edge].as_ref().expect("a route").points[at].1;
                    if (moved - p.1).abs() > 1e-6 {
                        break;
                    }
                }
            }
            // A change of course beside a run of another edge moves along the flow, out of its
            // way.
            for at in 1..length.saturating_sub(2) {
                let points = routes[edge].as_ref().expect("a route").points.clone();
                let (p, q) = (points[at], points[at + 1]);
                if (p.0 - q.0).abs() > 1e-6 {
                    continue;
                }
                let runs_crowded = |main: f64, routes: &[Option<Route>]| {
                    // The run that ends at the change of course now ends at `main`.
                    let (b0, b1) = if points[at - 1].0 < p.0 {
                        (points[at - 1].0, main)
                    } else {
                        (main, points[at - 1].0)
                    };
                    let (a0, a1) = if points[at + 2].0 < q.0 {
                        (points[at + 2].0, main)
                    } else {
                        (main, points[at + 2].0)
                    };
                    crowded(routes, edge, true, (b0, b1), points[at - 1].1)
                        || crowded(routes, edge, true, (a0, a1), points[at + 2].1)
                };
                if !runs_crowded(p.0, routes) {
                    continue;
                }
                let before = crossings(routes, edge);
                'search: for step in 1..=6 {
                    for sign in [1.0, -1.0] {
                        let main = p.0 + sign * 4.0 * f64::from(step);
                        let (a, b) = ((main, p.1), (main, q.1));
                        if runs_crowded(main, routes)
                            || crowded(routes, edge, false, (p.1.min(q.1), p.1.max(q.1)), main)
                            || hits(points[at - 1], a)
                            || hits(a, b)
                            || hits(b, points[at + 2])
                        {
                            continue;
                        }
                        let route = routes[edge].as_mut().expect("a route");
                        route.points[at] = a;
                        route.points[at + 1] = b;
                        if crossings(routes, edge) > before {
                            let route = routes[edge].as_mut().expect("a route");
                            route.points[at] = p;
                            route.points[at + 1] = q;
                            continue;
                        }
                        break 'search;
                    }
                }
            }
        }
    }
}
