//! Shared trunks: edges that leave one step for steps of the next layer, or reach one step from
//! steps of the layer before, drawn as one line out of the step that branches on a track of its
//! own, with a dot at every branch. Only edges that read the same on the shared part share it:
//! the usual arrowhead, no label, the same pattern, weight and role, off the main path, and with
//! their other ends in the same lane and frame, so that they cross the same frames. The branches
//! of a decision keep a line each. Every edge is still
//! drawn, from end to end, with its tooltip; where an edge would be ambiguous on the shared line,
//! such as a second edge between the same two steps, it keeps a line of its own.

use super::{Diagram, Ends, Graph, Shape};
use crate::spec::{Dash, DiagramRole};

/// What the edges of a trunk share: the step, the lane and the frame of their other ends, their
/// pattern, their weight and their role.
type Key = (
    usize,
    Option<usize>,
    Option<usize>,
    Option<Dash>,
    Option<u64>,
    Option<DiagramRole>,
);

/// The edges drawn on a shared trunk.
pub(super) struct Trunks {
    /// Per edge, the trunk it leaves its first step on, and the trunk it reaches its last step on.
    pub leave: Vec<Option<usize>>,
    pub reach: Vec<Option<usize>>,
    pub shared: Vec<Trunk>,
}

/// Edges that share their way out of a step or into it.
pub(super) struct Trunk {
    /// Whether the edges leave that step; otherwise they reach it.
    pub leaving: bool,
    pub edges: Vec<usize>,
}

impl Trunks {
    /// Every edge with a line of its own.
    pub(super) fn none(edges: usize) -> Self {
        Self {
            leave: vec![None; edges],
            reach: vec![None; edges],
            shared: Vec::new(),
        }
    }

    /// The trunks of `diagram`: first the edges that leave one step together, then, of the rest,
    /// those that reach one step together, at least two each.
    pub(super) fn of(diagram: &Diagram, graph: &Graph) -> Self {
        let edges = diagram.edges.len();
        let mut trunks = Self::none(edges);
        let shareable: Vec<bool> = (0..edges)
            .map(|edge| {
                let spec = &diagram.edges[edge];
                graph.chains[edge]
                    .as_ref()
                    .is_some_and(|chain| chain.items.len() == 2 && !chain.reversed)
                    && spec.label.is_none()
                    && spec.technology.is_none()
                    && !spec.main
                    && spec.ends == Ends::arrow()
            })
            .collect();
        for leaving in [true, false] {
            let mut candidates: Vec<(Key, Vec<usize>)> = Vec::new();
            for edge in (0..edges).filter(|&edge| shareable[edge] && trunks.leave[edge].is_none()) {
                let spec = &diagram.edges[edge];
                let (node, other) = if leaving {
                    (spec.from, spec.to)
                } else {
                    (spec.to, spec.from)
                };
                if leaving && diagram.nodes[node].shape == Shape::Decision {
                    continue;
                }
                let key = (
                    node,
                    diagram.nodes[other].lane,
                    diagram.nodes[other].group,
                    spec.dash,
                    spec.weight.map(f64::to_bits),
                    spec.role,
                );
                match candidates
                    .iter_mut()
                    .find(|(other_key, _)| *other_key == key)
                {
                    Some((_, members)) => members.push(edge),
                    None => candidates.push((key, vec![edge])),
                }
            }
            for (_, members) in candidates {
                let other = |edge: usize| {
                    let spec = &diagram.edges[edge];
                    if leaving { spec.to } else { spec.from }
                };
                let distinct = members.iter().enumerate().all(|(index, &edge)| {
                    members[..index].iter().all(|&e| other(e) != other(edge))
                });
                if members.len() < 2 || !distinct {
                    continue;
                }
                let trunk = trunks.shared.len();
                for &edge in &members {
                    if leaving {
                        trunks.leave[edge] = Some(trunk);
                    } else {
                        trunks.reach[edge] = Some(trunk);
                    }
                }
                trunks.shared.push(Trunk {
                    leaving,
                    edges: members,
                });
            }
        }
        trunks
    }

    /// The trunk edge `edge` shares at its first step when `leaving`, at its last otherwise.
    pub(super) fn at(&self, edge: usize, leaving: bool) -> Option<usize> {
        if leaving {
            self.leave[edge]
        } else {
            self.reach[edge]
        }
    }
}

/// Where the edges of a trunk branch, on the main and cross axes: `shared` is the cross position
/// of the trunk, `bus` the main position of the track it branches on, and `others` the cross
/// position at which each edge leaves the track (the trunk's own for an edge that runs straight
/// on). A branch is where the line goes on in more than one way: on the trunk where edges part to
/// more than one side, and at every edge that leaves the track before its farthest one on the same
/// side.
pub(super) fn branches(shared: f64, bus: f64, others: &[f64]) -> Vec<(f64, f64)> {
    let side = |sign: f64| {
        let mut side: Vec<f64> = others
            .iter()
            .copied()
            .filter(|other| (other - shared) * sign > 0.5)
            .collect();
        crate::sort::by(&mut side, |a, b| {
            ((a - shared) * sign).total_cmp(&((b - shared) * sign))
        });
        side
    };
    let (before, after) = (side(-1.0), side(1.0));
    let straight = others.iter().any(|other| (other - shared).abs() <= 0.5);
    let ways =
        usize::from(!before.is_empty()) + usize::from(!after.is_empty()) + usize::from(straight);
    let mut points = Vec::new();
    if ways >= 2 {
        points.push((bus, shared));
    }
    for side in [before, after] {
        if let Some((_, inner)) = side.split_last() {
            points.extend(inner.iter().map(|&other| (bus, other)));
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::branches;

    #[test]
    fn a_trunk_branches_where_its_line_goes_on_in_more_than_one_way() {
        // All to one side: the trunk only turns; every edge but the farthest leaves the track.
        assert_eq!(
            branches(0.0, 5.0, &[-30.0, -10.0, -20.0]),
            [(5.0, -10.0), (5.0, -20.0)]
        );
        // To both sides and straight on: the trunk branches too.
        assert_eq!(
            branches(0.0, 5.0, &[-10.0, 0.0, 10.0, 20.0]),
            [(5.0, 0.0), (5.0, 10.0)]
        );
        assert!(branches(0.0, 5.0, &[10.0]).is_empty());
    }
}
