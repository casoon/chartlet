//! Where the nodes and bands of a Sankey diagram stand.

// Node counts are at most 40, far below what an `f64` holds exactly.
#![allow(clippy::cast_precision_loss)]

use crate::spec::SankeyGraph;

/// The points along the top edge of a band, and back along the bottom edge.
const SAMPLES: usize = 16;
/// The room between two nodes of a column at most.
const GAP: f64 = 10.0;
/// The sweeps that move nodes towards the average place of their neighbours.
const SWEEPS: usize = 6;

pub(super) struct Plan<'a> {
    graph: &'a SankeyGraph,
    pub node_width: f64,
    pub columns: usize,
    pub spacing: f64,
    pub rank: Vec<usize>,
    /// The palette color, from 1, of every node.
    pub color: Vec<usize>,
    x: Vec<f64>,
    y: Vec<f64>,
    height: Vec<f64>,
    /// Where each link leaves its source and enters its target: the top edge.
    slots: Vec<(f64, f64)>,
    scale: f64,
}

impl<'a> Plan<'a> {
    /// The places of everything in a box `(x, y, width, height)`.
    pub(super) fn new(
        graph: &'a SankeyGraph,
        bounds: (f64, f64, f64, f64),
        node_width: f64,
    ) -> Self {
        let rank = graph
            .ranks
            .clone()
            .expect("validated Sankey diagrams have no cycle");
        let columns = rank.iter().max().map_or(1, |most| most + 1);
        let mut order = initial_order(&rank, columns);
        for sweep in 0..SWEEPS {
            sweep_columns(graph, &mut order, sweep % 2 == 0);
        }
        let spacing = if columns > 1 {
            (bounds.2 - node_width) / (columns - 1) as f64
        } else {
            0.0
        };
        let mut plan = Self {
            graph,
            node_width,
            columns,
            spacing,
            color: colors(graph, &rank),
            x: rank
                .iter()
                .map(|r| bounds.0 + spacing * *r as f64)
                .collect(),
            y: vec![0.0; rank.len()],
            height: vec![0.0; rank.len()],
            slots: vec![(0.0, 0.0); graph.links.len()],
            scale: 0.0,
            rank,
        };
        plan.place_nodes(&order, bounds.1, bounds.3);
        plan.place_slots();
        plan
    }

    fn place_nodes(&mut self, order: &[Vec<usize>], top: f64, available: f64) {
        let most = order.iter().map(Vec::len).max().unwrap_or(1);
        let gap = if most > 1 {
            GAP.min(available * 0.25 / (most - 1) as f64)
        } else {
            GAP
        };
        self.scale = order
            .iter()
            .map(|column| {
                let sum: f64 = column.iter().map(|node| self.graph.size(*node)).sum();
                (available - gap * (column.len() - 1) as f64) / sum
            })
            .fold(f64::MAX, f64::min);
        for column in order {
            let used: f64 = column
                .iter()
                .map(|node| (self.graph.size(*node) * self.scale).max(2.0))
                .sum::<f64>()
                + gap * (column.len() - 1) as f64;
            let mut at = top + (available - used) / 2.0;
            for node in column {
                self.height[*node] = (self.graph.size(*node) * self.scale).max(2.0);
                self.y[*node] = at;
                at += self.height[*node] + gap;
            }
        }
    }

    /// Stacks the links along the nodes they leave and enter, by the place of the other end.
    fn place_slots(&mut self) {
        for node in 0..self.graph.labels.len() {
            for outgoing in [true, false] {
                let mut mine: Vec<usize> = (0..self.graph.links.len())
                    .filter(|link| {
                        let (from, to, _) = self.graph.links[*link];
                        if outgoing { from == node } else { to == node }
                    })
                    .collect();
                let other = |link: &usize| {
                    let (from, to, _) = self.graph.links[*link];
                    let end = if outgoing { to } else { from };
                    self.y[end] + self.height[end] / 2.0
                };
                crate::sort::by(&mut mine, |a, b| other(a).total_cmp(&other(b)));
                let mut at = self.y[node];
                for link in mine {
                    let slot = &mut self.slots[link];
                    if outgoing {
                        slot.0 = at;
                    } else {
                        slot.1 = at;
                    }
                    at += self.graph.links[link].2 * self.scale;
                }
            }
        }
    }

    /// A node as `(x, y, height)`.
    pub(super) fn node(&self, node: usize) -> (f64, f64, f64) {
        (self.x[node], self.y[node], self.height[node])
    }

    /// The outline of a link: along the top edge from its source to its target, back along the
    /// bottom edge, the edges easing from one level to the other.
    pub(super) fn band(&self, link: usize) -> Vec<(f64, f64)> {
        let (from, to, value) = self.graph.links[link];
        let (start, end) = self.slots[link];
        let thickness = value * self.scale;
        let x0 = self.x[from] + self.node_width;
        let x1 = self.x[to];
        let at = |step: usize| {
            let t = step as f64 / SAMPLES as f64;
            let ease = t * t * (3.0 - 2.0 * t);
            (x0 + (x1 - x0) * t, start + (end - start) * ease)
        };
        (0..=SAMPLES)
            .map(at)
            .chain((0..=SAMPLES).rev().map(|step| {
                let (x, y) = at(step);
                (x, y + thickness)
            }))
            .collect()
    }
}

impl SankeyGraph {
    /// Whether a node stands in the last column (`last`) or the first.
    pub(crate) fn is_edge(&self, node: usize, last: bool) -> bool {
        let ranks = self.ranks.as_ref().expect("validated");
        let most = ranks.iter().max().copied().unwrap_or(0);
        if last {
            ranks[node] == most
        } else {
            ranks[node] == 0
        }
    }
}

fn initial_order(rank: &[usize], columns: usize) -> Vec<Vec<usize>> {
    let mut order = vec![Vec::new(); columns];
    for (node, column) in rank.iter().enumerate() {
        order[*column].push(node);
    }
    order
}

/// Orders each column by the weighted average place of its neighbours in the column before it
/// (`forward`) or after it.
fn sweep_columns(graph: &SankeyGraph, order: &mut [Vec<usize>], forward: bool) {
    let columns: Vec<usize> = if forward {
        (1..order.len()).collect()
    } else {
        (0..order.len().saturating_sub(1)).rev().collect()
    };
    for column in columns {
        let places: Vec<f64> = {
            let mut places = vec![0.0; graph.labels.len()];
            for members in order.iter() {
                for (position, node) in members.iter().enumerate() {
                    places[*node] = (position as f64 + 0.5) / members.len() as f64;
                }
            }
            places
        };
        let key = |node: usize| {
            let (sum, weight) =
                graph
                    .links
                    .iter()
                    .fold((0.0, 0.0), |(sum, weight), &(from, to, value)| {
                        let other = if forward {
                            (to == node).then_some(from)
                        } else {
                            (from == node).then_some(to)
                        };
                        other.map_or((sum, weight), |other| {
                            (sum + places[other] * value, weight + value)
                        })
                    });
            if weight > 0.0 {
                sum / weight
            } else {
                places[node]
            }
        };
        let keys: Vec<(usize, f64)> = order[column]
            .iter()
            .map(|node| (*node, key(*node)))
            .collect();
        let mut keyed = keys;
        crate::sort::by(&mut keyed, |a, b| a.1.total_cmp(&b.1));
        order[column] = keyed.into_iter().map(|(node, _)| node).collect();
    }
}

/// The nodes without incoming links take the palette colors in turn; every other node the color
/// of the node its biggest incoming link comes from.
fn colors(graph: &SankeyGraph, rank: &[usize]) -> Vec<usize> {
    let mut color = vec![0; rank.len()];
    let mut sources = 0;
    let mut by_rank: Vec<usize> = (0..rank.len()).collect();
    crate::sort::by(&mut by_rank, |a, b| rank[*a].cmp(&rank[*b]));
    for node in by_rank {
        let biggest = graph.links.iter().filter(|link| link.1 == node).fold(
            None,
            |best: Option<(usize, f64)>, &(from, _, value)| {
                best.filter(|(_, most)| *most >= value)
                    .or(Some((from, value)))
            },
        );
        color[node] = if let Some((from, _)) = biggest {
            color[from]
        } else {
            sources += 1;
            (sources - 1) % 4 + 1
        };
    }
    color
}
