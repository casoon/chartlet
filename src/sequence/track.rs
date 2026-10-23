//! The time axis: the extent of every slot, and where each arrow, fragment frame and branch
//! stands along it.

use super::{MAX_STRETCH, TAG_SIZE, model::Model};
use crate::{
    metrics::TextMetrics,
    spec::{FragmentSpec, SequenceSpec},
};

/// Lengthens the heads of fragments and branches in landscape, where a fragment's label and the
/// labels of its branches sit side by side along the top of the frame: each gets as far as the
/// next branch, or the end of the frame.
pub(super) fn make_room_for_guards(model: &Model, slots: &mut Slots, metrics: &impl TextMetrics) {
    let track = Track::new(model, slots, 1.0);
    let guard = |label: Option<&str>| {
        label.map_or(0.0, |label| {
            metrics.width(&format!("[{label}]"), TAG_SIZE) + 12.0
        })
    };
    for (index, fragment) in model.sequence.fragments.iter().enumerate() {
        let end = track.frames[index].1;
        let starts = &track.branches[index];
        let needed = tab_width(fragment, metrics) + 6.0 + guard(fragment.label.as_deref());
        let room = starts.first().copied().unwrap_or(end) - track.frames[index].0;
        slots.head[index] += (needed - room).max(0.0);
        for (number, branch) in fragment.branches.iter().enumerate() {
            let next = starts.get(number + 1).copied().unwrap_or(end);
            let needed = 6.0 + guard(Some(branch.label.as_deref().unwrap_or("")));
            slots.branch[index][number] += (needed - (next - starts[number])).max(0.0);
        }
    }
}

/// The width of the tab that names a fragment's kind.
pub(super) fn tab_width(fragment: &FragmentSpec, metrics: &impl TextMetrics) -> f64 {
    metrics.width(fragment.kind.keyword(), TAG_SIZE) + 12.0
}

/// Positions along the time axis, relative to where the messages begin.
pub(super) struct Track {
    /// Where each message's arrow runs.
    pub(super) arrows: Vec<f64>,
    /// Where each fragment's frame begins and ends.
    pub(super) frames: Vec<(f64, f64)>,
    /// Where each further branch of each fragment begins.
    pub(super) branches: Vec<Vec<f64>>,
    pub(super) length: f64,
}

/// The extent of the slots on the time axis: before each arrow, the whole slot, each fragment's
/// head, the head of each of its branches, and a fragment's foot.
pub(super) struct Slots {
    pub(super) arrow: Vec<f64>,
    pub(super) slot: Vec<f64>,
    pub(super) head: Vec<f64>,
    pub(super) branch: Vec<Vec<f64>>,
    pub(super) foot: f64,
}

impl Slots {
    /// The heads of every fragment and branch at the same extent.
    pub(super) fn uniform(
        sequence: &SequenceSpec,
        head: f64,
        branch: f64,
    ) -> (Vec<f64>, Vec<Vec<f64>>) {
        (
            vec![head; sequence.fragments.len()],
            sequence
                .fragments
                .iter()
                .map(|fragment| vec![branch; fragment.branches.len()])
                .collect(),
        )
    }
}

impl Track {
    fn new(model: &Model, slots: &Slots, scale: f64) -> Self {
        let sequence = model.sequence;
        let fragments = &sequence.fragments;
        let mut frames = vec![(0.0, 0.0); fragments.len()];
        let mut branches: Vec<Vec<f64>> = fragments
            .iter()
            .map(|fragment| vec![0.0; fragment.branches.len()])
            .collect();
        let mut arrows = Vec::with_capacity(sequence.messages.len());
        let mut at = 0.0;
        let mut by_depth: Vec<usize> = (0..fragments.len()).collect();
        crate::sort::by_key(&mut by_depth, |index| (model.depths[*index], *index));
        for index in 0..sequence.messages.len() {
            for &fragment in &by_depth {
                if fragments[fragment].from == index {
                    frames[fragment].0 = at;
                    at += slots.head[fragment] * scale;
                }
                if let Some(branch) = fragments[fragment]
                    .branches
                    .iter()
                    .position(|branch| branch.from == index)
                {
                    branches[fragment][branch] = at;
                    at += slots.branch[fragment][branch] * scale;
                }
            }
            arrows.push(at + slots.arrow[index] * scale);
            at += slots.slot[index] * scale;
            for &fragment in by_depth.iter().rev() {
                if fragments[fragment].to == index {
                    frames[fragment].1 = at;
                    at += slots.foot * scale;
                }
            }
        }
        Self {
            arrows,
            frames,
            branches,
            length: at,
        }
    }

    /// The track at the scale that fills `room` without growing slots by more than
    /// [`MAX_STRETCH`]; at scale 1 when it does not fit.
    pub(super) fn fitted(model: &Model, slots: &Slots, room: f64) -> Self {
        let natural = Self::new(model, slots, 1.0);
        if natural.length >= room || natural.length <= 0.0 {
            return natural;
        }
        Self::new(model, slots, (room / natural.length).min(MAX_STRETCH))
    }
}
