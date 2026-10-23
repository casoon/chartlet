//! Where edges meet their steps: the sides across the flow they are asked to leave and reach,
//! and their ports on the cross axis.

use super::{
    TRACK,
    graph::Graph,
    measure::Model,
    model::{Diagram, Shape},
};
use crate::{error::ChartWarning, layout::count, spec::PortSide};

/// A side of a step across the flow: towards smaller cross positions, left in portrait and top in
/// landscape, or towards larger ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Across {
    Low,
    High,
}

/// Per edge, the side across the flow it is asked to leave its first step on and reach its last
/// on, in the order the layout runs it: an edge the layout turned around leaves its `to` and
/// reaches its `from`. A side along the flow is where the edge meets its step anyway, or else
/// would turn the edge back over that step and its other edges: then it is reported and the edge
/// meets the step where the flow leads.
pub(super) fn across_sides(
    diagram: &Diagram,
    graph: &Graph,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<(Option<Across>, Option<Across>)> {
    diagram
        .edges
        .iter()
        .zip(&graph.chains)
        .map(|(edge, chain)| {
            let Some(chain) = chain else {
                return (None, None);
            };
            let mut side = |side: Option<PortSide>, field: &str, along: PortSide| match side {
                Some(PortSide::Left) => Some(Across::Low),
                Some(PortSide::Right) => Some(Across::High),
                Some(side) if side != along => {
                    warnings.push(ChartWarning::new(
                        "port_side_ignored",
                        format!("{}/{field}", edge.sides.path),
                        "on this side the edge would turn back over its node; it meets the node \
                         where the flow leads, use left or right to leave the flow",
                    ));
                    None
                }
                _ => None,
            };
            let (from_along, to_along) = if chain.reversed {
                (PortSide::Start, PortSide::End)
            } else {
                (PortSide::End, PortSide::Start)
            };
            let from = side(edge.sides.from, "fromSide", from_along);
            let to = side(edge.sides.to, "toSide", to_along);
            if chain.reversed {
                (to, from)
            } else {
                (from, to)
            }
        })
        .collect()
}

/// Per step, the room the edges that meet it across the flow take beside it on its low and its
/// high side: a stub out of the step, and a track more for every further edge on one side.
pub(super) fn side_room(
    nodes: usize,
    graph: &Graph,
    across: &[(Option<Across>, Option<Across>)],
    stub: f64,
) -> Vec<(f64, f64)> {
    (0..nodes)
        .map(|node| {
            let room = |side: Across| {
                let (mut leaving, mut reaching) = (0, 0);
                for (chain, (leave, reach)) in graph.chains.iter().zip(across) {
                    let Some(chain) = chain else { continue };
                    if chain.items[0] == node && *leave == Some(side) {
                        leaving += 1;
                    }
                    if chain.items[chain.items.len() - 1] == node && *reach == Some(side) {
                        reaching += 1;
                    }
                }
                let most = usize::max(leaving, reaching);
                if most == 0 {
                    0.0
                } else {
                    stub + TRACK * count(most - 1)
                }
            };
            (room(Across::Low), room(Across::High))
        })
        .collect()
}

/// Where edges leave and reach their steps on the cross axis, as offsets from the step's middle.
pub(super) struct Ports {
    /// Per edge: the offset where it leaves its first step and where it reaches its last. For an
    /// edge that meets its step across the flow, the offset of the stub beside the step it runs
    /// along.
    pub(super) leave: Vec<f64>,
    pub(super) reach: Vec<f64>,
    /// Per edge that leaves or reaches its step across the flow: where on the main axis it meets
    /// the side of the step, as an offset from the step's middle.
    pub(super) leave_along: Vec<Option<f64>>,
    pub(super) reach_along: Vec<Option<f64>>,
    /// Per edge that leaves or reaches a block: where on the main axis, from the block's middle,
    /// it meets the node inside the block it joins.
    pub(super) leave_into: Vec<Option<f64>>,
    pub(super) reach_into: Vec<Option<f64>>,
}

impl Ports {
    pub(super) fn new(
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
            // The edges of a block meet it where the nodes they join lie inside.
            if model.diagram.nodes[node].shape == Shape::Block {
                continue;
            }
            for (outgoing, offsets) in [(true, &mut leave), (false, &mut reach)] {
                let edges: Vec<(f64, usize)> = graph
                    .chains
                    .iter()
                    .enumerate()
                    .filter_map(|(edge, chain)| {
                        let chain = chain.as_ref()?;
                        let items = &chain.items;
                        let (end, next, side) = if outgoing {
                            (items[0], items[1], model.across[edge].0)
                        } else {
                            (
                                items[items.len() - 1],
                                items[items.len() - 2],
                                model.across[edge].1,
                            )
                        };
                        (end == node && side.is_none()).then_some((cross[next], edge))
                    })
                    .collect();
                // The edges of a shared trunk take one port, where they lie on average.
                let mut shared: Vec<(f64, Vec<usize>)> = Vec::new();
                for (at, edge) in edges {
                    let trunk = model.trunks.at(edge, outgoing);
                    match shared.iter_mut().find(|(_, members)| {
                        trunk.is_some() && model.trunks.at(members[0], outgoing) == trunk
                    }) {
                        Some((sum, members)) => {
                            *sum += at;
                            members.push(edge);
                        }
                        None => shared.push((at, vec![edge])),
                    }
                }
                let mut edges: Vec<(f64, Vec<usize>)> = shared
                    .into_iter()
                    .map(|(sum, members)| (sum / count(members.len()), members))
                    .collect();
                crate::sort::by(&mut edges, |a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                let ports = count(edges.len());
                if edges.len() < 2 || model.diagram.nodes[node].shape == Shape::Interface {
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
                    .position(|(_, members)| model.diagram.edges[members[0]].main)
                    .map(count);
                let middle = main.unwrap_or((ports - 1.0) / 2.0);
                let reach = (middle).max(ports - 1.0 - middle);
                let step = step.min((size / 2.0 - 8.0) / reach);
                for (index, (_, members)) in edges.iter().enumerate() {
                    for edge in members {
                        offsets[*edge] = (count(index) - middle) * step;
                    }
                }
            }
        }
        let mut ports = Self {
            leave,
            reach,
            leave_along: vec![None; edges],
            reach_along: vec![None; edges],
            leave_into: vec![None; edges],
            reach_into: vec![None; edges],
        };
        ports.across(graph, cross, cross_size, model, landscape);
        ports.blocks(graph, model, landscape);
        ports
    }

    /// The ports of the edges that meet their steps across the flow. On each side of a step, the
    /// edges that reach it come first along the flow and those that leave it after them, spread
    /// over the step's depth; each runs along a stub of its own beside the step, the edge that
    /// runs further along the step outside the others, so that none crosses another there.
    fn across(
        &mut self,
        graph: &Graph,
        cross: &[f64],
        cross_size: &impl Fn(usize) -> f64,
        model: &Model,
        landscape: bool,
    ) {
        let stub = model.spacing.passing_gap;
        let edges = graph.chains.len();
        let (mut leave_alone, mut reach_alone) = (vec![false; edges], vec![false; edges]);
        for node in 0..model.diagram.nodes.len() {
            let depth = if landscape {
                model.sizes[node].0
            } else {
                model.sizes[node].1
            };
            for side in [Across::Low, Across::High] {
                let meeting = |leaving: bool| -> Vec<usize> {
                    (0..graph.chains.len())
                        .filter(|&edge| {
                            graph.chains[edge].as_ref().is_some_and(|chain| {
                                let items = &chain.items;
                                if leaving {
                                    items[0] == node && model.across[edge].0 == Some(side)
                                } else {
                                    items[items.len() - 1] == node
                                        && model.across[edge].1 == Some(side)
                                }
                            })
                        })
                        .collect()
                };
                let (reaching, leaving) = (meeting(false), meeting(true));
                let ports = reaching.len() + leaving.len();
                if ports == 0 {
                    continue;
                }
                let step = if ports > 1 {
                    (depth * 0.6 / count(ports - 1)).min(16.0)
                } else {
                    0.0
                };
                let sign = if side == Across::High { 1.0 } else { -1.0 };
                let middle = count(ports - 1) / 2.0;
                for (index, &edge) in reaching.iter().enumerate() {
                    self.reach_along[edge] = Some((count(index) - middle) * step);
                    self.reach[edge] =
                        sign * (cross_size(node) / 2.0 + stub + TRACK * count(index));
                    reach_alone[edge] = ports == 1;
                }
                for (index, &edge) in leaving.iter().enumerate() {
                    let outside = leaving.len() - 1 - index;
                    self.leave_along[edge] = Some((count(reaching.len() + index) - middle) * step);
                    self.leave[edge] =
                        sign * (cross_size(node) / 2.0 + stub + TRACK * count(outside));
                    leave_alone[edge] = ports == 1;
                }
            }
        }
        // An edge alone on its side of a step runs on in line with what it leads to beside the
        // step, where the step's layer leaves room: the item that carries it on, or its stub at
        // the other end on the same side. It then needs no track to change course.
        let room = |node: usize, offset: f64| {
            let layer = &graph.layers[graph.layer[node]];
            let position = graph.position[node];
            let neighbour = if offset > 0.0 {
                layer.get(position + 1)
            } else {
                position.checked_sub(1).map(|before| &layer[before])
            };
            neighbour.map_or(f64::INFINITY, |&other| {
                (cross[other] - cross[node]).abs() - cross_size(other) / 2.0 - stub
            })
        };
        let outer = |offset: f64, a: f64, b: f64| if offset > 0.0 { a.max(b) } else { a.min(b) };
        for (edge, chain) in graph.chains.iter().enumerate() {
            let Some(chain) = chain else { continue };
            let items = &chain.items;
            let (first, last) = (items[0], items[items.len() - 1]);
            let (next, before) = (items[1], items[items.len() - 2]);
            let (leave, reach) = (self.leave[edge], self.reach[edge]);
            let fits =
                |node: usize, at: f64| (at - cross[node]).abs() <= room(node, at - cross[node]);
            if leave_alone[edge]
                && reach_alone[edge]
                && items.len() == 2
                && model.across[edge].0 == model.across[edge].1
            {
                let at = outer(leave, cross[first] + leave, cross[last] + reach);
                if fits(first, at) && fits(last, at) {
                    self.leave[edge] = at - cross[first];
                    self.reach[edge] = at - cross[last];
                }
                continue;
            }
            if leave_alone[edge] && graph.items[next].node.is_none() {
                let at = outer(leave, cross[first] + leave, cross[next]);
                if fits(first, at) {
                    self.leave[edge] = at - cross[first];
                }
            }
            if reach_alone[edge] && graph.items[before].node.is_none() {
                let at = outer(reach, cross[last] + reach, cross[before]);
                if fits(last, at) {
                    self.reach[edge] = at - cross[last];
                }
            }
        }
    }
}
