//! Lining up the main path, and the runs of edges beside the steps they meet across the flow,
//! where the neighbours leave room.

use super::{LANE_PAD, graph::Graph, measure::Model, ports::Across};

/// Lines the main path up where its neighbours leave room: each item of each of its edges moves
/// to the cross position of the item before it, unless that would crowd a neighbour or leave the
/// lane.
pub(super) fn straighten(
    model: &Model,
    cross: &mut [f64],
    cross_size: &impl Fn(usize) -> f64,
    gap: &impl Fn(usize, usize) -> f64,
    bands: &[(f64, f64)],
    lane_head: f64,
) {
    let graph = &model.graph;
    // An edge that meets a step across the flow cannot run straight.
    let edges: Vec<usize> = (0..model.diagram.edges.len())
        .filter(|edge| model.diagram.edges[*edge].main && model.across[*edge] == (None, None))
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
    line_up_side_runs(model, cross, cross_size, gap, bands, lane_head);
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

/// Lines up the items that carry an edge on beside a step it meets across the flow with the
/// outermost stub on that side, where their neighbours leave room.
fn line_up_side_runs(
    model: &Model,
    cross: &mut [f64],
    cross_size: &impl Fn(usize) -> f64,
    gap: &impl Fn(usize, usize) -> f64,
    bands: &[(f64, f64)],
    lane_head: f64,
) {
    let graph = &model.graph;
    for (edge, chain) in graph.chains.iter().enumerate() {
        let Some(chain) = chain else { continue };
        let inner = &chain.items[1..chain.items.len() - 1];
        let stub = |node: usize, side: Across| {
            let (low, high) = model.side_room[node];
            match side {
                Across::Low => cross[node] - cross_size(node) / 2.0 - low,
                Across::High => cross[node] + cross_size(node) / 2.0 + high,
            }
        };
        let (first, last) = (chain.items[0], chain.items[chain.items.len() - 1]);
        // From the end that meets its step across the flow; from both, outside both stubs.
        let (target, backwards) = match model.across[edge] {
            (None, None) => continue,
            (Some(Across::Low), Some(Across::Low)) => (
                f64::min(stub(first, Across::Low), stub(last, Across::Low)),
                false,
            ),
            (Some(Across::High), Some(Across::High)) => (
                f64::max(stub(first, Across::High), stub(last, Across::High)),
                false,
            ),
            (Some(side), _) => (stub(first, side), false),
            (None, Some(side)) => (stub(last, side), true),
        };
        let order: Vec<usize> = if backwards {
            inner.iter().rev().copied().collect()
        } else {
            inner.to_vec()
        };
        for item in order {
            if !align(
                graph, cross, item, target, cross_size, gap, bands, lane_head, false,
            ) {
                break;
            }
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
