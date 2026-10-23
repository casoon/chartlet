//! The shape of a graph in a few facts, for the descriptions of diagrams with nodes and edges:
//! the most connected nodes, where nothing leads in or out, loops, unconnected parts and the
//! node everything else hangs together through.
//!
//! Graphs here are small (a few dozen nodes), so the methods are the plain ones. Every result
//! follows the order of the specification, so the same input always reads the same.

use std::collections::VecDeque;

use crate::{spec::Locale, text};

/// At most this many nodes are named as the most connected, as sources or as sinks, and at most
/// this many loops.
const MOST_NAMED: usize = 3;
/// A node with fewer connections than this is not worth naming as the most connected.
const HUB_DEGREE: usize = 3;
/// Up to this many cut points are named; more mean the graph is a tree or a chain, not a graph
/// with a bottleneck.
const MOST_CUT_POINTS: usize = 2;

/// Which facts suit a diagram: those about direction (where nothing leads in or out, loops) only
/// where edges have one that matters.
#[derive(Clone, Copy)]
pub(crate) struct Facts {
    pub directed: bool,
}

/// The facts about one graph, as node indices in the order of the specification.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Structure {
    /// The most connected nodes and their number of connections.
    pub hubs: Vec<usize>,
    pub degree: usize,
    pub sources: Vec<usize>,
    pub sinks: Vec<usize>,
    /// One loop per strongly connected part, each from its first node back to it.
    pub loops: Vec<Vec<usize>>,
    /// How many parts the graph falls into when edges are read without direction.
    pub parts: usize,
    pub cut_points: Vec<usize>,
}

impl Structure {
    /// Analyses a graph of `nodes` nodes and the `edges` between them. Nodes in `marked` already
    /// show where the graph starts or ends (start and end steps, the initial and final states)
    /// and are not named again as sources or sinks.
    pub(crate) fn of(
        nodes: usize,
        edges: &[(usize, usize)],
        marked: &[bool],
        facts: Facts,
    ) -> Self {
        // Loops of a node to itself the description names already; they say nothing about shape.
        let edges: Vec<(usize, usize)> = edges.iter().copied().filter(|(a, b)| a != b).collect();
        if nodes < 2 || is_chain(nodes, &edges) {
            return Self::default();
        }
        let mut incoming = vec![0; nodes];
        let mut outgoing = vec![0; nodes];
        for &(from, to) in &edges {
            outgoing[from] += 1;
            incoming[to] += 1;
        }
        let degrees: Vec<usize> = (0..nodes).map(|n| incoming[n] + outgoing[n]).collect();
        let degree = degrees.iter().copied().max().unwrap_or(0);
        let hubs: Vec<usize> = (0..nodes).filter(|&n| degrees[n] == degree).collect();
        let hubs = if degree >= HUB_DEGREE && hubs.len() <= MOST_NAMED {
            hubs
        } else {
            Vec::new()
        };
        let mut structure = Self {
            hubs,
            degree,
            parts: parts(nodes, &edges),
            cut_points: cut_points(nodes, &edges),
            ..Self::default()
        };
        if structure.cut_points.len() > MOST_CUT_POINTS {
            structure.cut_points.clear();
        }
        if facts.directed {
            let unmarked = |n: &usize| !marked.get(*n).copied().unwrap_or(false);
            structure.sources = (0..nodes)
                .filter(|&n| incoming[n] == 0 && outgoing[n] > 0)
                .filter(unmarked)
                .collect();
            structure.sinks = (0..nodes)
                .filter(|&n| outgoing[n] == 0 && incoming[n] > 0)
                .filter(unmarked)
                .collect();
            structure.loops = loops(nodes, &edges);
            // A long list of open ends tells less than the steps themselves do.
            for ends in [&mut structure.sources, &mut structure.sinks] {
                if ends.len() > MOST_NAMED {
                    ends.clear();
                }
            }
        }
        structure
    }

    /// The facts as sentences, or nothing when the graph has none worth telling.
    pub(crate) fn sentences(&self, locale: Locale, label: impl Fn(usize) -> String) -> String {
        let names = |nodes: &[usize]| nodes.iter().map(|&n| label(n)).collect::<Vec<_>>();
        let mut sentences = Vec::new();
        if !self.hubs.is_empty() {
            sentences.push(text::most_connected(
                locale,
                &names(&self.hubs),
                self.degree,
            ));
        }
        if !self.sources.is_empty() {
            sentences.push(text::sources(locale, &names(&self.sources)));
        }
        if !self.sinks.is_empty() {
            sentences.push(text::sinks(locale, &names(&self.sinks)));
        }
        if !self.loops.is_empty() {
            let loops: Vec<String> = self
                .loops
                .iter()
                .map(|cycle| {
                    let mut steps = names(cycle);
                    steps.push(label(cycle[0]));
                    steps.join(" → ")
                })
                .collect();
            sentences.push(text::loops(locale, &loops));
        }
        if self.parts > 1 {
            sentences.push(text::unconnected_parts(locale, self.parts));
        }
        if !self.cut_points.is_empty() {
            sentences.push(text::bottlenecks(locale, &names(&self.cut_points)));
        }
        sentences.join(" ")
    }
}

/// Whether the graph is one chain without branches, read in the direction of its edges: then
/// there is nothing to tell beyond the steps themselves.
fn is_chain(nodes: usize, edges: &[(usize, usize)]) -> bool {
    if edges.len() != nodes - 1 {
        return false;
    }
    let mut incoming = vec![0; nodes];
    let mut outgoing = vec![0; nodes];
    for &(from, to) in edges {
        outgoing[from] += 1;
        incoming[to] += 1;
    }
    incoming.iter().all(|&count| count <= 1)
        && outgoing.iter().all(|&count| count <= 1)
        && parts(nodes, edges) == 1
}

/// Each node's neighbours, read without direction, in the order of the edges.
pub(crate) fn neighbours(nodes: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut neighbours = vec![Vec::new(); nodes];
    for &(from, to) in edges {
        neighbours[from].push(to);
        neighbours[to].push(from);
    }
    neighbours
}

/// How many parts the graph falls into when edges are read without direction.
fn parts(nodes: usize, edges: &[(usize, usize)]) -> usize {
    let neighbours = neighbours(nodes, edges);
    let mut seen = vec![false; nodes];
    let mut count = 0;
    for start in 0..nodes {
        if seen[start] {
            continue;
        }
        count += 1;
        seen[start] = true;
        let mut stack = vec![start];
        while let Some(node) = stack.pop() {
            for &next in &neighbours[node] {
                if !seen[next] {
                    seen[next] = true;
                    stack.push(next);
                }
            }
        }
    }
    count
}

/// The nodes without which a part of the graph falls apart (read without direction) into at
/// least two pieces of more than one node each; a node that only cuts off a single end step is
/// no bottleneck.
fn cut_points(nodes: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let neighbours = neighbours(nodes, edges);
    (0..nodes)
        .filter(|&cut| {
            let mut seen = vec![false; nodes];
            seen[cut] = true;
            let mut large = 0;
            for &start in &neighbours[cut] {
                if seen[start] {
                    continue;
                }
                seen[start] = true;
                let mut size = 1;
                let mut stack = vec![start];
                while let Some(node) = stack.pop() {
                    for &next in &neighbours[node] {
                        if !seen[next] {
                            seen[next] = true;
                            size += 1;
                            stack.push(next);
                        }
                    }
                }
                if size > 1 {
                    large += 1;
                }
            }
            large >= 2
        })
        .collect()
}

/// One loop for each of the first strongly connected parts with more than one node: the
/// shortest way from its first node back to it.
fn loops(nodes: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut forward = vec![Vec::new(); nodes];
    for &(from, to) in edges {
        forward[from].push(to);
    }
    let reach = |start: usize, within: &dyn Fn(usize) -> bool| -> Vec<Option<usize>> {
        // The node each one was first reached from, by breadth-first search.
        let mut from = vec![None; nodes];
        let mut queue = VecDeque::from([start]);
        let mut seen = vec![false; nodes];
        seen[start] = true;
        while let Some(node) = queue.pop_front() {
            for &next in &forward[node] {
                if within(next) && !seen[next] {
                    seen[next] = true;
                    from[next] = Some(node);
                    queue.push_back(next);
                }
            }
        }
        from
    };
    let mut backward = vec![Vec::new(); nodes];
    for &(from, to) in edges {
        backward[to].push(from);
    }
    let mut part = vec![usize::MAX; nodes];
    let mut found = Vec::new();
    for start in 0..nodes {
        if part[start] != usize::MAX || found.len() == MOST_NAMED {
            continue;
        }
        // The strongly connected part of `start`: what it reaches and what reaches it.
        let ahead = reach(start, &|_| true);
        let mut behind = vec![false; nodes];
        behind[start] = true;
        let mut stack = vec![start];
        while let Some(node) = stack.pop() {
            for &previous in &backward[node] {
                if !behind[previous] {
                    behind[previous] = true;
                    stack.push(previous);
                }
            }
        }
        let members: Vec<usize> = (0..nodes)
            .filter(|&n| behind[n] && (n == start || ahead[n].is_some()))
            .collect();
        for &member in &members {
            part[member] = start;
        }
        if members.len() < 2 {
            continue;
        }
        // The shortest way back to `start`: from each of its predecessors inside the part.
        let inside = |n: usize| part[n] == start;
        let way = reach(start, &inside);
        let closing = edges
            .iter()
            .filter(|(from, to)| *to == start && inside(*from))
            .map(|(from, _)| *from)
            .min_by_key(|&last| {
                let mut length = 0;
                let mut node = last;
                while let Some(previous) = way[node] {
                    length += 1;
                    node = previous;
                }
                (length, last)
            })
            .expect("a part of two nodes has an edge back to its first");
        let mut cycle = vec![closing];
        while let Some(previous) = way[*cycle.last().expect("not empty")] {
            cycle.push(previous);
        }
        cycle.reverse();
        found.push(cycle);
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{Facts, Structure};

    const DIRECTED: Facts = Facts { directed: true };

    fn of(nodes: usize, edges: &[(usize, usize)]) -> Structure {
        Structure::of(nodes, edges, &[], DIRECTED)
    }

    #[test]
    fn a_chain_has_nothing_to_tell() {
        assert_eq!(of(4, &[(0, 1), (1, 2), (2, 3)]), Structure::default());
    }

    #[test]
    fn a_loop_runs_from_its_first_node_back_to_it() {
        let structure = of(4, &[(0, 1), (1, 2), (2, 3), (3, 1)]);
        assert_eq!(structure.loops, vec![vec![1, 2, 3]]);
        assert_eq!(structure.sources, vec![0]);
        assert!(structure.sinks.is_empty(), "{:?}", structure.sinks);
    }

    #[test]
    fn the_shortest_way_back_wins() {
        // 0 → 1 → 2 → 0 and 0 → 3 → 0: the second is shorter.
        let structure = of(4, &[(0, 1), (1, 2), (2, 0), (0, 3), (3, 0)]);
        assert_eq!(structure.loops, vec![vec![0, 3]]);
    }

    #[test]
    fn unconnected_parts_are_counted() {
        let structure = of(5, &[(0, 1), (0, 2), (3, 4)]);
        assert_eq!(structure.parts, 2);
    }

    #[test]
    fn hubs_tie_in_the_order_of_the_specification() {
        // 1 and 2 have three connections each.
        let structure = of(5, &[(0, 1), (1, 2), (1, 3), (2, 3), (2, 4)]);
        assert_eq!(structure.hubs, vec![1, 2]);
        assert_eq!(structure.degree, 3);
    }

    #[test]
    fn a_bottleneck_joins_two_halves() {
        // Two triangles joined through node 2.
        let edges = [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)];
        assert_eq!(of(5, &edges).cut_points, vec![2]);
    }

    #[test]
    fn marked_ends_are_not_named_again() {
        let structure = Structure::of(
            4,
            &[(0, 1), (1, 2), (1, 3)],
            &[true, false, false, false],
            DIRECTED,
        );
        assert!(structure.sources.is_empty(), "{:?}", structure.sources);
        assert_eq!(structure.sinks, vec![2, 3]);
    }

    #[test]
    fn undirected_facts_leave_out_direction() {
        let structure = Structure::of(3, &[(0, 1), (1, 2), (2, 0)], &[], Facts { directed: false });
        assert!(structure.loops.is_empty() && structure.sources.is_empty());
    }
}
