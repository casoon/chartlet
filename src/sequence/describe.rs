//! The text alternative and the data table: every message in order, with the fragments it is
//! in.

use std::fmt::Write as _;

use super::sequence;
use crate::{
    DataTable,
    spec::{ChartSpec, FragmentSpec, Locale, SequenceSpec},
    text,
};

/// The diagram in sentences: its participants, then every message in order, then the fragments.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let sequence = sequence(spec);
    let locale = spec.locale;
    let participants: Vec<String> = sequence
        .participants
        .iter()
        .map(|participant| text::participant(locale, &participant.label, participant.kind))
        .collect();
    let mut description = text::sequence_opening(locale, &participants, sequence.messages.len());
    for (index, (message, (from, to))) in sequence.messages.iter().zip(sequence.ends()).enumerate()
    {
        let sender = &sequence.participants[from].label;
        let receiver = (from != to).then(|| sequence.participants[to].label.as_str());
        write!(
            description,
            " {}",
            text::sequence_message(
                locale,
                index + 1,
                sender,
                receiver,
                &message.label,
                message.kind
            )
        )
        .expect("writing to String cannot fail");
    }
    for fragment in &sequence.fragments {
        description.push(' ');
        description.push_str(&text::fragment(locale, fragment));
    }
    description
}

/// One row per message: its number, sender, receiver, label and kind, and the fragments around
/// it when the diagram has any.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let sequence = sequence(spec);
    let words = spec.locale.words();
    let mut columns: Vec<String> = [
        words.number,
        words.sender,
        words.receiver,
        words.message,
        words.message_kind,
    ]
    .iter()
    .map(|word| (*word).to_owned())
    .collect();
    let framed = !sequence.fragments.is_empty();
    if framed {
        columns.push(words.fragment.to_owned());
    }
    let rows = sequence
        .messages
        .iter()
        .zip(sequence.ends())
        .enumerate()
        .map(|(index, (message, (from, to)))| {
            let mut row = vec![
                (index + 1).to_string(),
                sequence.participants[from].label.clone(),
                sequence.participants[to].label.clone(),
                message.label.clone(),
                text::message_kind(spec.locale, message.kind).to_owned(),
            ];
            if framed {
                row.push(fragments_at(spec.locale, sequence, index));
            }
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}

/// The fragments around message `index`, outermost first, each as its kind and the label of the
/// branch the message is in.
fn fragments_at(locale: Locale, sequence: &SequenceSpec, index: usize) -> String {
    let mut around: Vec<(usize, &FragmentSpec)> = sequence
        .fragments
        .iter()
        .enumerate()
        .filter(|(_, fragment)| fragment.from <= index && index <= fragment.to)
        .map(|(position, fragment)| (sequence.depth(position), fragment))
        .collect();
    crate::sort::by_key(&mut around, |(depth, _)| *depth);
    around
        .iter()
        .map(|(_, fragment)| {
            let label = fragment
                .branches
                .iter()
                .rev()
                .find(|branch| branch.from <= index)
                .map_or(fragment.label.as_deref(), |branch| {
                    Some(branch.label.as_deref().unwrap_or(text::otherwise(locale)))
                });
            match label {
                Some(label) => format!("{} [{label}]", fragment.kind.keyword()),
                None => fragment.kind.keyword().to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
