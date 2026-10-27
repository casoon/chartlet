//! Keeping the frames of groups clear of the steps they do not hold, and apart from each other.

use super::{GROUP_HEAD, GROUP_PAD, measure::Model, model::Diagram};

/// How far the frame of `group` reaches on the cross axis: around its own steps and the frames
/// inside it, with the room a frame takes on either side.
fn frame_range(
    diagram: &Diagram,
    cross: &[f64],
    cross_size: &impl Fn(usize) -> f64,
    group: usize,
    landscape: bool,
) -> (f64, f64) {
    let own = (0..diagram.nodes.len())
        .filter(|node| diagram.nodes[*node].group == Some(group))
        .map(|node| {
            (
                cross[node] - cross_size(node) / 2.0,
                cross[node] + cross_size(node) / 2.0,
            )
        });
    let inner = (0..diagram.groups.len())
        .filter(|child| diagram.groups[*child].parent == Some(group))
        .map(|child| frame_range(diagram, cross, cross_size, child, landscape));
    let (low, high) = own
        .chain(inner)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), (a, b)| {
            (low.min(a), high.max(b))
        });
    let head = if landscape { GROUP_HEAD } else { 0.0 };
    (low - GROUP_PAD - head, high + GROUP_PAD)
}

/// Moves the steps that do not belong to a group out of its frame, with the steps beyond them, to
/// the side they stand on: a frame drawn around a group's steps would otherwise enclose a step of
/// a neighbouring layer that lies between them. Inner frames first; the frames around them then
/// take in what moved.
#[allow(clippy::cast_precision_loss)]
pub(super) fn clear_frames(
    model: &Model,
    cross: &mut [f64],
    cross_size: &impl Fn(usize) -> f64,
    landscape: bool,
) {
    const MARGIN: f64 = 12.0;
    let (graph, diagram) = (&model.graph, model.diagram);
    let before = cross.to_vec();
    let mut order: Vec<usize> = (0..diagram.groups.len()).collect();
    crate::sort::by_key(&mut order, |group| {
        (std::cmp::Reverse(diagram.chain(Some(*group)).len()), *group)
    });
    let item_size = |item: usize| cross_size(item);
    for &group in &order {
        let layers: Vec<usize> = (0..diagram.nodes.len())
            .filter(|node| diagram.inside(*node, group))
            .map(|node| graph.layer[node])
            .collect();
        let first = layers.iter().copied().min().unwrap_or(0);
        let last = layers.iter().copied().max().unwrap_or(0);
        for layer in first..=last {
            let items = &graph.layers[layer];
            for position in 0..items.len() {
                let item = items[position];
                let Some(node) = graph.items[item].node else {
                    continue;
                };
                if diagram.inside(node, group) {
                    continue;
                }
                let (low, high) = frame_range(diagram, cross, cross_size, group, landscape);
                let half = item_size(item) / 2.0;
                let (start, end) = (cross[item] - half - MARGIN, cross[item] + half + MARGIN);
                if end <= low || start >= high {
                    continue;
                }
                let member = |other: &usize| {
                    graph.items[*other]
                        .node
                        .is_some_and(|node| diagram.inside(node, group))
                };
                let before = items[..position].iter().any(member);
                let after = items[position + 1..].iter().any(member);
                let left = if before == after {
                    cross[item] < f64::midpoint(low, high)
                } else {
                    after
                };
                if left {
                    let shift = low - end;
                    for other in &items[..=position] {
                        cross[*other] += shift;
                    }
                } else {
                    let shift = high - start;
                    for other in &items[position..] {
                        cross[*other] += shift;
                    }
                }
            }
        }
    }
    separate_frames(model, cross, cross_size, landscape);
    // Groups that interleave in the layers cannot all be cleared: every step moved out of one frame
    // lands in the next and the steps drift apart without end. Where the content outgrows what the
    // steps could need side by side, the order before stays and `group_overlap` names the rest.
    let budget: f64 = (0..graph.items.len()).map(cross_size).sum::<f64>()
        + diagram.groups.len() as f64 * 2.0 * (GROUP_PAD + GROUP_HEAD + MARGIN);
    let reach = |cross: &[f64]| {
        let (low, high) = frames_reach(model, cross, cross_size, landscape);
        (0..graph.items.len())
            .map(|item| cross[item] + cross_size(item) / 2.0)
            .fold(high, f64::max)
            - (0..graph.items.len())
                .map(|item| cross[item] - cross_size(item) / 2.0)
                .fold(low, f64::min)
    };
    if reach(cross) > budget {
        cross.copy_from_slice(&before);
    }
    let low = (0..graph.items.len())
        .map(|item| cross[item] - cross_size(item) / 2.0)
        .fold(f64::INFINITY, f64::min)
        .min(frames_reach(model, cross, cross_size, landscape).0);
    for position in cross.iter_mut() {
        *position -= low;
    }
}

/// Moves frames of unrelated groups apart where they share a layer and overlap on the cross
/// axis: a frame reaches over every layer of its steps, so two groups that stand side by side in
/// one layer and above each other in another would otherwise be drawn into each other. The frame
/// further along the cross axis moves on, with the steps beside it in the layers it spans.
fn separate_frames(
    model: &Model,
    cross: &mut [f64],
    cross_size: &impl Fn(usize) -> f64,
    landscape: bool,
) {
    const SEPARATION: f64 = 16.0;
    let (graph, diagram) = (&model.graph, model.diagram);
    let groups = diagram.groups.len();
    let span = |group: usize| -> Option<(usize, usize)> {
        let layers = (0..diagram.nodes.len())
            .filter(|node| diagram.inside(*node, group))
            .map(|node| graph.layer[node]);
        layers.clone().min().zip(layers.max())
    };
    let related = |a: usize, b: usize| {
        diagram.chain(Some(a)).contains(&b) || diagram.chain(Some(b)).contains(&a)
    };
    for _ in 0..=groups * groups {
        let mut moved = false;
        for a in 0..groups {
            for b in 0..groups {
                if a == b || related(a, b) {
                    continue;
                }
                let (Some((a_first, a_last)), Some((b_first, b_last))) = (span(a), span(b)) else {
                    continue;
                };
                if a_last < b_first || b_last < a_first {
                    continue;
                }
                let (a_low, a_high) = frame_range(diagram, cross, cross_size, a, landscape);
                let (b_low, b_high) = frame_range(diagram, cross, cross_size, b, landscape);
                // `a` is the frame on the low side of the cross axis.
                if a_low + a_high >= b_low + b_high
                    || a_high + SEPARATION <= b_low
                    || b_high <= a_low
                {
                    continue;
                }
                let shift = a_high + SEPARATION - b_low;
                let anchor = (0..diagram.nodes.len())
                    .filter(|node| diagram.inside(*node, b))
                    .map(|node| cross[node])
                    .fold(f64::INFINITY, f64::min);
                for layer in b_first..=b_last {
                    for &item in &graph.layers[layer] {
                        if cross[item] >= anchor {
                            cross[item] += shift;
                        }
                    }
                }
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
}

/// How far the outermost frames reach on the cross axis: their low and their high end.
pub(super) fn frames_reach(
    model: &Model,
    cross: &[f64],
    cross_size: &impl Fn(usize) -> f64,
    landscape: bool,
) -> (f64, f64) {
    let diagram = model.diagram;
    (0..diagram.groups.len())
        .filter(|group| diagram.groups[*group].parent.is_none())
        .map(|group| frame_range(diagram, cross, cross_size, group, landscape))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), (a, b)| {
            (low.min(a), high.max(b))
        })
}
