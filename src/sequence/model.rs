//! What both orientations need from the specification: every participant's box and label,
//! every message's label, the depth of each fragment, and the activation bars.

use super::{
    COMPACT_BOX, COMPACT_MARGIN, COMPACT_TEXT, LABEL_SIZE, LINE, MARGIN, MAX_BOX, MIN_BOX,
    SUBLABEL_SIZE, pad,
};
use crate::{
    diagram::wrap,
    flow::focus_classes,
    metrics::TextMetrics,
    spec::{FragmentSpec, MessageKind, ParticipantKind, SequenceSpec},
};

/// A stretch of one participant's lifeline during which it is busy: from the call that reaches
/// it to the reply it sends, or else to the last message it takes part in.
pub(super) struct Activation {
    pub(super) participant: usize,
    pub(super) from: usize,
    pub(super) to: usize,
    /// How many of the participant's activations are still open around this one.
    pub(super) level: usize,
}

/// What both orientations need from the specification.
pub(super) struct Model<'a> {
    pub(super) sequence: &'a SequenceSpec,
    pub(super) ends: Vec<(usize, usize)>,
    /// Each message's label as drawn, with its number when the diagram is numbered.
    pub(super) labels: Vec<String>,
    pub(super) activations: Vec<Activation>,
    /// How many fragments enclose each fragment.
    pub(super) depths: Vec<usize>,
    /// The width a participant's box needs for its label and sublabel.
    pub(super) box_widths: Vec<f64>,
    /// Height of the band that holds the participants' names.
    pub(super) band: f64,
    /// Whether the diagram is narrower than [`COMPACT_BELOW`](super::COMPACT_BELOW): participant names wrap, margins shrink.
    pub(super) compact: bool,
    pub(super) margin: f64,
    pub(super) has_actor: bool,
}

impl<'a> Model<'a> {
    pub(super) fn new(
        sequence: &'a SequenceSpec,
        compact: bool,
        metrics: &impl TextMetrics,
    ) -> Self {
        let ends = sequence.ends();
        let labels = sequence
            .messages
            .iter()
            .map(|message| message.label.clone())
            .collect();
        let mut two_lines = false;
        let box_widths = sequence
            .participants
            .iter()
            .map(|participant| {
                let mut label = metrics.width(&participant.label, LABEL_SIZE);
                if compact && label > COMPACT_TEXT {
                    // Never narrower than the longest word, which cannot wrap.
                    let word = participant
                        .label
                        .split(' ')
                        .map(|word| metrics.width(word, LABEL_SIZE))
                        .fold(0.0, f64::max);
                    let mut ignored = Vec::new();
                    let lines = wrap(
                        &participant.label,
                        COMPACT_TEXT.max(word),
                        LABEL_SIZE,
                        metrics,
                        &mut ignored,
                        "",
                    );
                    two_lines |= lines.len() > 1;
                    label = lines
                        .iter()
                        .map(|line| metrics.width(line, LABEL_SIZE))
                        .fold(0.0, f64::max);
                }
                let sublabel = participant
                    .sublabel
                    .as_ref()
                    .map_or(0.0, |sublabel| metrics.width(sublabel, SUBLABEL_SIZE));
                // A compact box is as wide as its widest line, even a long word.
                let low = if compact { COMPACT_BOX } else { MIN_BOX };
                // Rounded up: a width that is the text plus padding exactly can fall a hair short of
                // the text again after the padding is taken off, and the name would be shortened.
                (label.max(sublabel) + 2.0 * pad(compact))
                    .ceil()
                    .clamp(low, MAX_BOX)
            })
            .collect();
        let has_sublabel = sequence
            .participants
            .iter()
            .any(|participant| participant.sublabel.is_some());
        Self {
            activations: activations(sequence, &ends),
            depths: (0..sequence.fragments.len())
                .map(|index| sequence.depth(index))
                .collect(),
            ends,
            labels,
            sequence,
            box_widths,
            band: if has_sublabel { 48.0 } else { 34.0 } + if two_lines { LINE } else { 0.0 },
            compact,
            margin: if compact { COMPACT_MARGIN } else { MARGIN },
            has_actor: sequence
                .participants
                .iter()
                .any(|participant| participant.kind == ParticipantKind::Actor),
        }
    }

    pub(super) fn participants(&self) -> usize {
        self.sequence.participants.len()
    }

    /// The widest box any participant needs.
    pub(super) fn widest_box(&self) -> f64 {
        let floor = if self.compact { COMPACT_BOX } else { MIN_BOX };
        self.box_widths.iter().copied().fold(floor, f64::max)
    }

    /// Whether `participant` is busy while message `index` is sent.
    pub(super) fn active(&self, participant: usize, index: usize) -> bool {
        self.activations.iter().any(|activation| {
            activation.participant == participant
                && activation.from <= index
                && index <= activation.to
        })
    }

    /// The participants that send or receive a message inside `fragment`.
    pub(super) fn span(&self, fragment: &FragmentSpec) -> (usize, usize) {
        self.ends[fragment.from..=fragment.to]
            .iter()
            .flat_map(|(from, to)| [*from, *to])
            .fold((usize::MAX, 0), |(low, high), participant| {
                (low.min(participant), high.max(participant))
            })
    }
}

impl Model<'_> {
    /// The focus classes of a participant: itself and everyone it exchanges a message with.
    pub(super) fn participant_classes(&self, participant: usize) -> String {
        let mut related: Vec<usize> = self
            .ends
            .iter()
            .filter_map(
                |&(from, to)| match (from == participant, to == participant) {
                    (true, _) => Some(to),
                    (_, true) => Some(from),
                    _ => None,
                },
            )
            .chain(std::iter::once(participant))
            .collect();
        related.sort_unstable();
        related.dedup();
        focus_classes(Some(participant), &related)
    }

    /// The focus classes of a message: the participants at both its ends.
    pub(super) fn message_classes(&self, index: usize) -> String {
        let (from, to) = self.ends[index];
        focus_classes(None, &[from.min(to), from.max(to)])
    }
}

/// The activations of a sequence: a call opens one on its receiver, a reply closes the last one
/// its sender has open, and one left open ends with the last message its participant takes part
/// in. A message to oneself and an asynchronous message open none.
pub(super) fn activations(sequence: &SequenceSpec, ends: &[(usize, usize)]) -> Vec<Activation> {
    let mut activations: Vec<Activation> = Vec::new();
    let mut open: Vec<Vec<usize>> = vec![Vec::new(); sequence.participants.len()];
    let mut closed = Vec::new();
    for (index, (message, (from, to))) in sequence.messages.iter().zip(ends).enumerate() {
        match message.kind {
            MessageKind::Call if from != to => {
                open[*to].push(activations.len());
                activations.push(Activation {
                    participant: *to,
                    from: index,
                    to: index,
                    level: open[*to].len() - 1,
                });
            }
            MessageKind::Reply => {
                if let Some(activation) = open[*from].pop() {
                    activations[activation].to = index;
                    closed.push(activation);
                }
            }
            MessageKind::Call | MessageKind::Async => {}
        }
    }
    for (index, activation) in activations.iter_mut().enumerate() {
        if closed.contains(&index) {
            continue;
        }
        activation.to = ends
            .iter()
            .rposition(|(from, to)| {
                *from == activation.participant || *to == activation.participant
            })
            .expect("the call that opened it involves the participant")
            .max(activation.from);
    }
    activations
}
