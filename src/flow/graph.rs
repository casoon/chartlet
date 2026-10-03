//! The layered layout of a flow chart, in the steps of the classic method: break cycles, assign
//! layers, insert placeholders where an edge spans several layers, order every layer to keep
//! crossings few, and place the items of each layer across the flow.
//!
//! Every step is deterministic: ties fall back to the order of the specification.

use std::{cmp::Reverse, collections::BinaryHeap};

use super::Diagram;

/// An item of a layer: a step, or a placeholder that carries an edge through the layer.
pub(super) struct Item {
    pub node: Option<usize>,
    pub lane: Option<usize>,
    /// The groups the item lies in, outermost first; none for a placeholder.
    pub groups: Vec<usize>,
    /// A placeholder of an edge that runs against the flow; it keeps to the outside of its lane,
    /// so that the way back does not cut through the steps.
    pub back: bool,
}

/// The items an edge passes, from where the layout lets it start to where it ends. `reversed`
/// marks an edge the layout turned around to break a cycle; it is drawn back the other way.
pub(super) struct Chain {
    pub items: Vec<usize>,
    pub reversed: bool,
}

/// How an item sorts within its layer: lane, whether it carries an edge back, the means of the
/// groups it lies in level by level ending with its own barycenter, and its earlier position.
type SortKey = (usize, bool, Vec<(f64, usize)>, usize);

pub(super) struct Graph {
    pub items: Vec<Item>,
    /// The items of each layer, in their order across the flow.
    pub layers: Vec<Vec<usize>>,
    /// The layer of each item.
    pub layer: Vec<usize>,
    /// The position of each item within its layer.
    pub position: Vec<usize>,
    /// The chain of every edge; `None` for an edge from a step to itself.
    pub chains: Vec<Option<Chain>>,
    /// Per item, its neighbours in the layer before and the layer after, with a weight: how much
    /// it matters that the link between them runs straight.
    pub before: Vec<Vec<(usize, f64)>>,
    pub after: Vec<Vec<(usize, f64)>>,
}

impl Graph {
    pub(super) fn new(diagram: &Diagram) -> Self {
        let nodes = diagram.nodes.len();
        let ends = &diagram.ends();
        let reversed = feedback_edges(nodes, ends);
        let directed: Vec<Option<(usize, usize)>> = ends
            .iter()
            .zip(&reversed)
            .map(|(&(from, to), &reversed)| match (from == to, reversed) {
                (true, _) => None,
                (false, false) => Some((from, to)),
                (false, true) => Some((to, from)),
            })
            .collect();
        let node_layers = layers(nodes, &directed);
        let mut items: Vec<Item> = (0..nodes)
            .map(|node| Item {
                node: Some(node),
                lane: diagram.nodes[node].lane,
                groups: diagram.chain(diagram.nodes[node].group),
                back: false,
            })
            .collect();
        let mut layer = node_layers.clone();
        let mut chains = Vec::with_capacity(ends.len());
        for (index, edge) in directed.iter().enumerate() {
            let Some((from, to)) = *edge else {
                chains.push(None);
                continue;
            };
            let mut chain = vec![from];
            let (first, last) = (node_layers[from], node_layers[to]);
            for between in first + 1..last {
                // A placeholder keeps to the lane of the step it comes from for the first half of
                // the way, then to the lane of the step it goes to.
                let lane = if (between - first) * 2 <= last - first {
                    items[from].lane
                } else {
                    items[to].lane
                };
                chain.push(items.len());
                items.push(Item {
                    node: None,
                    lane,
                    groups: Vec::new(),
                    back: reversed[index],
                });
                layer.push(between);
            }
            chain.push(to);
            chains.push(Some(Chain {
                items: chain,
                reversed: reversed[index],
            }));
        }
        let count = node_layers.iter().copied().max().map_or(0, |last| last + 1);
        let mut layers = vec![Vec::new(); count];
        for (item, &at) in layer.iter().enumerate() {
            layers[at].push(item);
        }
        let mut before = vec![Vec::new(); items.len()];
        let mut after = vec![Vec::new(); items.len()];
        for (index, chain) in chains.iter().enumerate() {
            let Some(chain) = chain else { continue };
            let main = diagram.edges[index].main;
            for pair in chain.items.windows(2) {
                let placeholders = pair
                    .iter()
                    .filter(|item| items[**item].node.is_none())
                    .count();
                // Long edges and the main path should run straight, other links may bend.
                let weight = match (main, placeholders) {
                    (true, _) => 6.0,
                    (false, 2) => 8.0,
                    (false, 1) => 2.0,
                    (false, _) => 1.0,
                };
                after[pair[0]].push((pair[1], weight));
                before[pair[1]].push((pair[0], weight));
            }
        }
        let mut graph = Self {
            position: vec![0; items.len()],
            items,
            layers,
            layer,
            chains,
            before,
            after,
        };
        graph.order();
        graph
    }

    fn renumber(&mut self) {
        for layer in &self.layers {
            for (position, item) in layer.iter().enumerate() {
                self.position[*item] = position;
            }
        }
    }

    /// Orders every layer by the mean position of each item's neighbours, sweeping down and up
    /// the layers, and keeps the order with the fewest crossings. Within a layer, items keep to
    /// their lanes and the members of a group stay together.
    fn order(&mut self) {
        self.renumber();
        let mut best = (self.crossings(), self.layers.clone());
        for sweep in 0..16 {
            let down = sweep % 2 == 0;
            let order: Vec<usize> = if down {
                (1..self.layers.len()).collect()
            } else {
                (0..self.layers.len().saturating_sub(1)).rev().collect()
            };
            for layer in order {
                self.sort_layer(layer, down);
            }
            let crossings = self.crossings();
            if crossings < best.0 {
                best = (crossings, self.layers.clone());
            }
        }
        self.layers = best.1;
        self.renumber();
    }

    fn sort_layer(&mut self, layer: usize, down: bool) {
        let barycenter = |item: usize| {
            let neighbours = if down {
                &self.before[item]
            } else {
                &self.after[item]
            };
            if neighbours.is_empty() {
                return crate::layout::count(self.position[item]);
            }
            let total: f64 = neighbours.iter().map(|(_, weight)| weight).sum();
            neighbours
                .iter()
                .map(|(other, weight)| crate::layout::count(self.position[*other]) * weight)
                .sum::<f64>()
                / total
        };
        let items = &self.layers[layer];
        let own: Vec<f64> = items.iter().map(|item| barycenter(*item)).collect();
        // The members of a group share the mean of their barycenters, level by level, so that a
        // group stays together and so do the groups inside it.
        let keys: Vec<SortKey> = items
            .iter()
            .zip(&own)
            .map(|(item, key)| {
                let mut path: Vec<(f64, usize)> = self.items[*item]
                    .groups
                    .iter()
                    .map(|group| {
                        let members: Vec<f64> = items
                            .iter()
                            .zip(&own)
                            .filter(|(other, _)| self.items[**other].groups.contains(group))
                            .map(|(_, key)| *key)
                            .collect();
                        let mean =
                            members.iter().sum::<f64>() / crate::layout::count(members.len());
                        (mean, *group)
                    })
                    .collect();
                path.push((*key, usize::MAX));
                (
                    self.items[*item].lane.unwrap_or(0),
                    self.items[*item].back,
                    path,
                    self.position[*item],
                )
            })
            .collect();
        let mut order: Vec<usize> = (0..items.len()).collect();
        order.sort_by(|a, b| {
            let (a, b) = (&keys[*a], &keys[*b]);
            let path =
                a.2.iter()
                    .zip(&b.2)
                    .map(|(a, b)| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
                    .find(|order| order.is_ne())
                    .unwrap_or(std::cmp::Ordering::Equal);
            a.0.cmp(&b.0)
                .then(a.1.cmp(&b.1))
                .then(path)
                .then(a.3.cmp(&b.3))
        });
        let sorted: Vec<usize> = order.iter().map(|index| items[*index]).collect();
        self.layers[layer] = sorted;
        for (position, item) in self.layers[layer].iter().enumerate() {
            self.position[*item] = position;
        }
    }

    /// The crossings between the links of every pair of adjacent layers.
    fn crossings(&self) -> usize {
        let mut total = 0;
        for layer in &self.layers {
            let links: Vec<(usize, usize)> = layer
                .iter()
                .flat_map(|item| {
                    self.after[*item]
                        .iter()
                        .map(|(next, _)| (self.position[*item], self.position[*next]))
                })
                .collect();
            for (index, a) in links.iter().enumerate() {
                total += links[index + 1..]
                    .iter()
                    .filter(|b| (a.0 < b.0 && a.1 > b.1) || (a.0 > b.0 && a.1 < b.1))
                    .count();
            }
        }
        total
    }
}

/// The edges a depth-first search from every step in turn finds leading back to a step still on
/// its path: turning them around leaves no cycle.
fn feedback_edges(nodes: usize, ends: &[(usize, usize)]) -> Vec<bool> {
    let mut outgoing = vec![Vec::new(); nodes];
    for (index, (from, to)) in ends.iter().enumerate() {
        if from != to {
            outgoing[*from].push(index);
        }
    }
    // 0: not yet visited, 1: on the current path, 2: done.
    let mut state = vec![0_u8; nodes];
    let mut reversed = vec![false; ends.len()];
    for root in 0..nodes {
        if state[root] != 0 {
            continue;
        }
        state[root] = 1;
        let mut stack = vec![(root, 0)];
        while let Some((node, next)) = stack.last_mut() {
            let node = *node;
            if let Some(&edge) = outgoing[node].get(*next) {
                *next += 1;
                let to = ends[edge].1;
                match state[to] {
                    0 => {
                        state[to] = 1;
                        stack.push((to, 0));
                    }
                    1 => reversed[edge] = true,
                    _ => {}
                }
            } else {
                state[node] = 2;
                stack.pop();
            }
        }
    }
    reversed
}

/// The layer of every step: one after the latest of the steps before it, and a step that only
/// leads on placed just before the earliest step it leads to.
fn layers(nodes: usize, edges: &[Option<(usize, usize)>]) -> Vec<usize> {
    let mut incoming = vec![0; nodes];
    let mut outgoing = vec![Vec::new(); nodes];
    for (from, to) in edges.iter().flatten() {
        incoming[*to] += 1;
        outgoing[*from].push(*to);
    }
    let mut layer = vec![0; nodes];
    let mut waiting = incoming.clone();
    let mut ready: BinaryHeap<Reverse<usize>> = (0..nodes)
        .filter(|node| incoming[*node] == 0)
        .map(Reverse)
        .collect();
    let mut sorted = Vec::with_capacity(nodes);
    while let Some(Reverse(node)) = ready.pop() {
        sorted.push(node);
        for &next in &outgoing[node] {
            layer[next] = layer[next].max(layer[node] + 1);
            waiting[next] -= 1;
            if waiting[next] == 0 {
                ready.push(Reverse(next));
            }
        }
    }
    for &node in sorted.iter().rev() {
        if incoming[node] == 0 && !outgoing[node].is_empty() {
            layer[node] = outgoing[node]
                .iter()
                .map(|next| layer[*next])
                .min()
                .expect("a step that leads on has a next step")
                - 1;
        }
    }
    layer
}

/// Positions along one segment of a layer, as close to `targets` as `weights` ask while every
/// item keeps at least `gaps[i]` from the one before it (`gaps[0]` is unused) and, with `bounds`,
/// the segment stays between them. This is weighted isotonic regression: shifting each target by
/// the sum of the gaps before it turns the gaps into a plain order, which pooling adjacent
/// violators solves exactly.
pub(super) fn settle(
    targets: &[f64],
    weights: &[f64],
    gaps: &[f64],
    bounds: Option<(f64, f64)>,
) -> Vec<f64> {
    let mut offsets = Vec::with_capacity(targets.len());
    let mut offset = 0.0;
    for (index, gap) in gaps.iter().enumerate() {
        if index > 0 {
            offset += gap;
        }
        offsets.push(offset);
    }
    // Blocks of pooled values: weighted sum, weight, and how many items they hold.
    let mut blocks: Vec<(f64, f64, usize)> = Vec::new();
    for ((target, weight), offset) in targets.iter().zip(weights).zip(&offsets) {
        blocks.push(((target - offset) * weight, *weight, 1));
        while blocks.len() > 1 {
            let last = blocks[blocks.len() - 1];
            let previous = blocks[blocks.len() - 2];
            if previous.0 / previous.1 <= last.0 / last.1 {
                break;
            }
            blocks.pop();
            let merged = blocks.last_mut().expect("two blocks were there");
            merged.0 += last.0;
            merged.1 += last.1;
            merged.2 += last.2;
        }
    }
    let mut positions = Vec::with_capacity(targets.len());
    for (sum, weight, count) in blocks {
        let value = sum / weight;
        let value = match bounds {
            Some((low, high)) => value.max(low).min(high - offset),
            None => value,
        };
        for _ in 0..count {
            let index = positions.len();
            positions.push(value + offsets[index]);
        }
    }
    positions
}

#[cfg(test)]
mod tests {
    use super::{feedback_edges, layers, settle};

    #[test]
    fn a_cycle_loses_the_edge_that_closes_it() {
        assert_eq!(
            feedback_edges(3, &[(0, 1), (1, 2), (2, 0)]),
            [false, false, true]
        );
        assert_eq!(feedback_edges(2, &[(0, 1), (1, 1)]), [false, false]);
    }

    #[test]
    fn a_step_lies_one_layer_after_its_latest_predecessor() {
        let edges = [Some((0, 1)), Some((1, 2)), Some((0, 2)), Some((3, 2))];
        // Step 3 only leads on, to step 2 in layer 2, so it sits in layer 1.
        assert_eq!(layers(4, &edges), [0, 1, 2, 1]);
    }

    #[test]
    fn settling_keeps_the_gaps_and_stays_as_close_as_it_can() {
        // Two items that both want 0 split the difference around it.
        assert_eq!(
            settle(&[0.0, 0.0], &[1.0, 1.0], &[0.0, 10.0], None),
            [-5.0, 5.0]
        );
        // A heavier item gets its way.
        assert_eq!(
            settle(&[0.0, 0.0], &[3.0, 1.0], &[0.0, 8.0], None),
            [-2.0, 6.0]
        );
        // Items already apart stay where they are; bounds pull them inside.
        assert_eq!(
            settle(&[0.0, 50.0], &[1.0, 1.0], &[0.0, 10.0], None),
            [0.0, 50.0]
        );
        assert_eq!(
            settle(&[0.0, 50.0], &[1.0, 1.0], &[0.0, 10.0], Some((5.0, 40.0))),
            [5.0, 40.0]
        );
    }
}
