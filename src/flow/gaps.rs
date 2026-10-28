//! The spaces between layers: room for labels and frames, and the tracks on which edges change
//! course.

use super::{
    GRID, GROUP_HEAD, GROUP_PAD, LINE, TRACK, down, graph::Graph, measure::Model, place::Plan,
    ports::Ports, trunks::Trunks, up,
};
use crate::layout::count;

/// One stretch of an edge between two adjacent layers: the cross positions it comes from and
/// goes to, and the track it changes course on.
pub(super) struct Piece {
    pub(super) edge: usize,
    pub(super) layer: usize,
    pub(super) from: f64,
    pub(super) to: f64,
    pub(super) track: Option<usize>,
}

/// The spaces between layers: room for labels where edges leave, tracks for edges that change
/// course, and room for the frames of groups.
pub(super) struct Gaps {
    pub(super) pieces: Vec<Piece>,
    /// Per gap: the room for labels, the number of tracks, and the room a group's frame takes
    /// after the layer before and before the layer after.
    label_room: Vec<f64>,
    /// Per gap: the room for the labels of edges that leave the layer after it against the flow.
    label_room_end: Vec<f64>,
    tracks: Vec<usize>,
    frame_after: Vec<f64>,
    frame_before: Vec<f64>,
    /// The least distance between two layers, and how much the gaps grow at most.
    layer_gap: f64,
    max_stretch: f64,
}

impl Gaps {
    pub(super) fn new(model: &Model, plan: &Plan, ports: &Ports, landscape: bool) -> Self {
        let graph = &model.graph;
        let layers = graph.layers.len();
        let gaps = layers.saturating_sub(1);
        let mut pieces = Vec::new();
        let mut label_room = vec![0.0; gaps];
        let mut label_room_end = vec![0.0; gaps];
        let mut stacks: std::collections::BTreeMap<(usize, bool, bool), f64> =
            std::collections::BTreeMap::new();
        for (edge, chain) in graph.chains.iter().enumerate() {
            let Some(chain) = chain else { continue };
            let items = &chain.items;
            let last = items.len() - 2;
            for (index, pair) in items.windows(2).enumerate() {
                let from = plan.cross[pair[0]] + if index == 0 { ports.leave[edge] } else { 0.0 };
                let to = plan.cross[pair[1]]
                    + if index == last {
                        ports.reach[edge]
                    } else {
                        0.0
                    };
                pieces.push(Piece {
                    edge,
                    layer: graph.layer[pair[0]],
                    from,
                    to,
                    track: None,
                });
            }
            let lines = &model.edge_labels[edge];
            // The label of an edge that meets its step across the flow stands beside the step.
            let beside = if chain.reversed {
                ports.reach_along[edge].is_some()
            } else {
                ports.leave_along[edge].is_some()
            };
            if !lines.is_empty() && !beside {
                // In portrait, labels on the same side of a step stack along the flow.
                let (node, offset) = if chain.reversed {
                    (items[items.len() - 1], ports.reach[edge])
                } else {
                    (items[0], ports.leave[edge])
                };
                let room = if landscape {
                    model.edge_widths[edge] + 26.0
                } else {
                    let stacked = stacks
                        .entry((node, chain.reversed, offset >= 0.0))
                        .or_insert(0.0);
                    *stacked += LINE * count(lines.len()) + 9.0;
                    *stacked + 7.0
                };
                if chain.reversed {
                    let gap = graph.layer[node] - 1;
                    label_room_end[gap] = f64::max(label_room_end[gap], room);
                } else {
                    let gap = graph.layer[node];
                    label_room[gap] = f64::max(label_room[gap], room);
                }
            }
        }
        let mut tracks = vec![0; gaps];
        let trunks = &model.trunks;
        let shared = |piece: &Piece| {
            trunks.leave[piece.edge].is_some() || trunks.reach[piece.edge].is_some()
        };
        for (gap, tracks) in tracks.iter_mut().enumerate() {
            let bends = |piece: &Piece| piece.layer == gap && (piece.from - piece.to).abs() > 0.5;
            let mut bending: Vec<usize> = (0..pieces.len())
                .filter(|index| bends(&pieces[*index]) && !shared(&pieces[*index]))
                .collect();
            crate::sort::by(&mut bending, |a, b| {
                let low = |piece: &Piece| piece.from.min(piece.to);
                low(&pieces[*a])
                    .total_cmp(&low(&pieces[*b]))
                    .then(pieces[*a].edge.cmp(&pieces[*b].edge))
            });
            let mut assigned = assign_tracks(&pieces, &bending);
            assigned.extend(trunk_tracks(&pieces, &assigned, trunks, &bends));
            for (index, track) in &assigned {
                pieces[*index].track = Some(*track);
            }
            *tracks = assigned
                .iter()
                .map(|(_, track)| track + 1)
                .max()
                .unwrap_or(0);
        }
        let (frame_before, frame_after) = frame_room(model, landscape);
        Self {
            pieces,
            label_room,
            label_room_end,
            tracks,
            frame_after,
            frame_before,
            layer_gap: model.spacing.layer_gap,
            max_stretch: model.spacing.max_stretch,
        }
    }

    /// How much the gaps grow to fill `room` on the main axis, up to `max_stretch`.
    pub(super) fn stretch(&self, natural: f64, room: f64) -> f64 {
        if natural < room && self.total() > 0.0 {
            (1.0 + (room - natural) / self.total()).min(self.max_stretch)
        } else {
            1.0
        }
    }

    /// The width of gap `gap` at scale 1.
    fn size(&self, gap: usize) -> f64 {
        up(self.raw_size(gap), GRID)
    }

    fn raw_size(&self, gap: usize) -> f64 {
        self.layer_gap
            + self.frame_after[gap]
            + self.label_room[gap]
            + count(self.tracks[gap]) * TRACK
            + self.label_room_end[gap]
            + self.frame_before[gap + 1]
    }

    fn total(&self) -> f64 {
        (0..self.tracks.len()).map(|gap| self.size(gap)).sum()
    }

    /// The length of the layers along the main axis with the gaps at their least.
    pub(super) fn length(
        &self,
        main_size: &impl Fn(usize) -> f64,
        graph: &Graph,
        (lead, trail): (f64, f64),
    ) -> f64 {
        let depths: f64 = graph
            .layers
            .iter()
            .map(|layer| depth(graph, layer, main_size))
            .sum();
        lead + self.frame_before.first().copied().unwrap_or(0.0)
            + depths
            + self.total()
            + self.frame_after.last().copied().unwrap_or(0.0)
            + trail
    }

    /// Places the layers on the main axis and returns their length.
    pub(super) fn place(
        &self,
        plan: &mut Plan,
        main_size: &impl Fn(usize) -> f64,
        graph: &Graph,
        (lead, trail): (f64, f64),
        stretch: f64,
    ) -> f64 {
        let mut at = lead + self.frame_before.first().copied().unwrap_or(0.0);
        plan.layers.clear();
        for (index, layer) in graph.layers.iter().enumerate() {
            let depth = depth(graph, layer, main_size);
            plan.layers.push((at, at + depth));
            at += depth;
            if index < self.tracks.len() {
                at += down(self.size(index) * stretch, GRID).max(self.size(index));
            }
        }
        at + self.frame_after.last().copied().unwrap_or(0.0) + trail
    }

    /// The main position of track `track` in gap `gap`, which begins at `start`.
    pub(super) fn track(&self, gap: usize, track: usize, start: f64, stretch: f64) -> f64 {
        start
            + (self.frame_after[gap]
                + self.label_room[gap]
                + self.layer_gap / 2.0
                + count(track) * TRACK)
                * stretch
    }
}

/// Per layer, the room the frames of groups take before it on the main axis, where they open,
/// and after it, where they close.
fn frame_room(model: &Model, landscape: bool) -> (Vec<f64>, Vec<f64>) {
    let graph = &model.graph;
    let layers = graph.layers.len();
    let mut frame_after = vec![0.0; layers];
    let mut frame_before = vec![0.0; layers];
    // The layers each group spans, through the groups inside it.
    let diagram = model.diagram;
    let spans: Vec<(usize, usize)> = (0..diagram.groups.len())
        .map(|group| {
            let layers: Vec<usize> = (0..diagram.nodes.len())
                .filter(|node| diagram.inside(*node, group))
                .map(|node| graph.layer[node])
                .collect();
            (
                layers.iter().copied().min().unwrap_or(0),
                layers.iter().copied().max().unwrap_or(0),
            )
        })
        .collect();
    let head = if landscape {
        GROUP_PAD
    } else {
        GROUP_PAD + GROUP_HEAD
    };
    // A step needs room for every frame that opens before it or closes after it.
    for node in 0..diagram.nodes.len() {
        let layer = graph.layer[node];
        let chain = diagram.chain(diagram.nodes[node].group);
        let opening = chain
            .iter()
            .filter(|group| spans[**group].0 == layer)
            .count();
        let closing = chain
            .iter()
            .filter(|group| spans[**group].1 == layer)
            .count();
        frame_before[layer] = f64::max(frame_before[layer], head * count(opening));
        frame_after[layer] = f64::max(frame_after[layer], GROUP_PAD * count(closing));
    }
    (frame_before, frame_after)
}

/// How deep a layer is on the main axis: as deep as its deepest step.
fn depth(graph: &Graph, layer: &[usize], main_size: &impl Fn(usize) -> f64) -> f64 {
    up(
        layer
            .iter()
            .filter_map(|item| graph.items[*item].node)
            .map(main_size)
            .fold(0.0, f64::max),
        2.0 * GRID,
    )
}

/// The tracks of the pieces that change course in one gap, so that no edge crosses another where
/// that can be avoided. A piece leaves its layer at `from`, runs along its track and arrives at
/// `to`: it crosses a piece whose track lies before its own when that one's start lies under it,
/// and one whose track lies after its own when that one's end does. So a piece whose start lies
/// under another takes the earlier track, and one whose end lies under another the later.
/// Pieces that cannot both be satisfied, and pieces left over, follow the order given.
fn assign_tracks(pieces: &[Piece], order: &[usize]) -> Vec<(usize, usize)> {
    let range = |piece: &Piece| (piece.from.min(piece.to), piece.from.max(piece.to));
    let under = |x: f64, (low, high): (f64, f64)| low + 0.5 < x && x < high - 0.5;
    // `before[v]` lists the pieces that must have an earlier track than `v`.
    let mut before: Vec<Vec<usize>> = vec![Vec::new(); order.len()];
    for (a, &first) in order.iter().enumerate() {
        for (b, &second) in order.iter().enumerate() {
            if a == b {
                continue;
            }
            let (one, other) = (&pieces[first], &pieces[second]);
            // `a`'s start under `b`'s run: `a` goes first. `a`'s end under it: `a` goes last.
            if under(one.from, range(other)) {
                before[b].push(a);
            }
            if under(one.to, range(other)) {
                before[a].push(b);
            }
        }
    }
    let mut tracks: Vec<Option<usize>> = vec![None; order.len()];
    let mut on_track: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut result = Vec::with_capacity(order.len());
    for _ in 0..order.len() {
        // The next piece is the first whose predecessors all have tracks; with a cycle, the first
        // left.
        let next = (0..order.len())
            .find(|&v| tracks[v].is_none() && before[v].iter().all(|&u| tracks[u].is_some()))
            .or_else(|| (0..order.len()).find(|&v| tracks[v].is_none()))
            .expect("a piece is left");
        let (low, high) = range(&pieces[order[next]]);
        let earliest = before[next]
            .iter()
            .filter_map(|&u| tracks[u])
            .map(|track| track + 1)
            .max()
            .unwrap_or(0);
        let track = (earliest..on_track.len())
            .find(|&track| {
                on_track.get(track).is_none_or(|ranges| {
                    ranges
                        .iter()
                        .all(|&(l, h)| h + 10.0 < low || high + 10.0 < l)
                })
            })
            .unwrap_or(earliest.max(on_track.len()));
        if on_track.len() <= track {
            on_track.resize(track + 1, Vec::new());
        }
        on_track[track].push((low, high));
        tracks[next] = Some(track);
        result.push((order[next], track));
    }
    result
}

/// The tracks of the shared trunks in one gap, given the tracks `assigned` to the other pieces
/// there: the pieces of a trunk that `bend` there take one track together, the first whose pieces
/// keep clear of the trunk's whole run.
fn trunk_tracks(
    pieces: &[Piece],
    assigned: &[(usize, usize)],
    trunks: &Trunks,
    bends: &impl Fn(&Piece) -> bool,
) -> Vec<(usize, usize)> {
    let range = |piece: &Piece| (piece.from.min(piece.to), piece.from.max(piece.to));
    let mut on_track: Vec<Vec<(f64, f64)>> = Vec::new();
    for &(index, track) in assigned {
        if on_track.len() <= track {
            on_track.resize(track + 1, Vec::new());
        }
        on_track[track].push(range(&pieces[index]));
    }
    let mut result = Vec::new();
    for trunk in &trunks.shared {
        let members: Vec<usize> = (0..pieces.len())
            .filter(|&index| bends(&pieces[index]) && trunk.edges.contains(&pieces[index].edge))
            .collect();
        let Some((low, high)) = members
            .iter()
            .map(|&index| range(&pieces[index]))
            .reduce(|(low, high), (l, h)| (low.min(l), high.max(h)))
        else {
            continue;
        };
        let track = (0..=on_track.len())
            .find(|&track| {
                on_track.get(track).is_none_or(|ranges| {
                    ranges
                        .iter()
                        .all(|&(l, h)| h + 10.0 < low || high + 10.0 < l)
                })
            })
            .expect("a track past the last is free");
        if on_track.len() <= track {
            on_track.resize(track + 1, Vec::new());
        }
        on_track[track].push((low, high));
        result.extend(members.into_iter().map(|index| (index, track)));
    }
    result
}

#[cfg(test)]
mod track_tests {
    use super::{Piece, assign_tracks};

    fn piece(edge: usize, from: f64, to: f64) -> Piece {
        Piece {
            edge,
            layer: 0,
            from,
            to,
            track: None,
        }
    }

    #[test]
    fn edges_turning_the_same_way_take_tracks_that_do_not_cross() {
        // The left edge leaves at 561 and turns right at its track; the right edge leaves at 584,
        // under the left one's run, so it must turn first: its start would cross the left run
        // otherwise.
        let pieces = [piece(0, 561.0, 703.0), piece(1, 584.0, 860.0)];
        let tracks = assign_tracks(&pieces, &[0, 1]);
        let track = |index: usize| tracks.iter().find(|(piece, _)| *piece == index).unwrap().1;
        assert!(track(1) < track(0), "{tracks:?}");
    }

    #[test]
    fn pieces_that_do_not_overlap_share_a_track() {
        let pieces = [piece(0, 10.0, 50.0), piece(1, 100.0, 140.0)];
        let tracks = assign_tracks(&pieces, &[0, 1]);
        assert!(tracks.iter().all(|(_, track)| *track == 0), "{tracks:?}");
    }
}
