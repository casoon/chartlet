//! Notes of a diagram, wherever they stand: beside an element, joined to it by a dotted line;
//! inside a frame, at its head, without a line; or on the diagram as a whole, in a band below it.
//! None of them adds an edge, and the description and table name every one.

use super::{
    Diagram, MEMBER_LINE, Node, PAD, Shape, count,
    measure::{NOTE_PAD, Spacing, note_box},
    shape,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Element, Scene, Text, TextAnchor},
    spec::{ChartSpec, DiagramSpacing, Locale, NoteSpec},
    text,
};

/// What a note explains.
pub(crate) enum Target {
    /// The node with this index.
    Node(usize),
    /// The group with this index.
    Frame(usize),
}

/// Adds the notes on elements and frames to `diagram`, `block` naming them in paths; `target`
/// finds what each `on` names. A note on the whole diagram is no node: [`push_band`] draws it.
pub(crate) fn push_notes(
    diagram: &mut Diagram,
    notes: &[NoteSpec],
    block: &str,
    target: impl Fn(&str) -> Target,
) {
    for (index, note) in notes.iter().enumerate() {
        let path = format!("/{block}/notes/{index}");
        match note.on.as_deref().map(&target) {
            Some(Target::Node(node)) => diagram.push_note(node, &note.text, path),
            Some(Target::Frame(group)) => diagram.push_frame_note(group, &note.text, path),
            None => {}
        }
    }
}

impl Diagram {
    /// Adds a note with `text` on group `group`: a node of its own inside the group, without a
    /// line; the layout stands it in the first layer of the group's steps.
    fn push_frame_note(&mut self, group: usize, text: &str, path: String) {
        self.nodes.push(Node {
            label: text.to_owned(),
            sublabel: None,
            shape: Shape::Note,
            lane: None,
            group: Some(group),
            path,
            compartments: Vec::new(),
            value: None,
            parts: Vec::new(),
            id: None,
            symbol: None,
            tag: None,
            role: None,
            category: None,
        });
    }
}

/// Draws the notes on the whole diagram in a band below it, left to right, wrapping onto as many
/// rows as the canvas needs; nothing without them.
pub(crate) fn push_band(
    scene: &mut Scene,
    spec: &ChartSpec,
    (notes, block): (&[NoteSpec], &str),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    let editorial = spec.look == crate::spec::Look::Editorial;
    let spacing = Spacing::new(spec.width, DiagramSpacing::Normal, (editorial, spec.tags));
    let face = crate::metrics::Face::new(metrics, editorial);
    let metrics = &face;
    let boxes: Vec<(&str, Vec<String>, (f64, f64))> = notes
        .iter()
        .enumerate()
        .filter(|(_, note)| note.on.is_none())
        .map(|(index, note)| {
            let path = format!("/{block}/notes/{index}");
            let (lines, size) = note_box(&note.text, &path, &spacing, metrics, warnings);
            (note.text.as_str(), lines, size)
        })
        .collect();
    if boxes.is_empty() {
        return;
    }
    let right = f64::from(scene.width) - spacing.margin;
    let (mut x, mut y, mut row_height) = (spacing.margin, f64::from(scene.height), 0.0_f64);
    for (note, lines, (width, height)) in boxes {
        if x > spacing.margin && x + width > right {
            x = spacing.margin;
            y += row_height + spacing.node_gap / 2.0;
            row_height = 0.0;
        }
        shape(
            Shape::Note,
            (x, y, width, height),
            Some(note.to_owned()),
            &mut scene.elements,
        );
        for (index, line) in lines.into_iter().enumerate() {
            scene.elements.push(Element::Text(Text {
                x: x + PAD,
                y: y + NOTE_PAD + 11.0 + MEMBER_LINE * count(index),
                class: "chartlet-diagram-chip-text",
                anchor: TextAnchor::Start,
                content: line,
            }));
        }
        x += width + spacing.node_gap;
        row_height = row_height.max(height);
    }
    scene.height = crate::diagram::pixels(y + row_height + super::BOTTOM);
}

/// Every note as a sentence, after a space, with the label of what it explains; `label` gives
/// that for an `on`. Nothing without notes.
pub(crate) fn sentences<'a>(
    locale: Locale,
    notes: &'a [NoteSpec],
    label: impl Fn(&str) -> &'a str,
) -> String {
    if notes.is_empty() {
        return String::new();
    }
    let notes: Vec<(Option<&str>, &str)> = notes
        .iter()
        .map(|note| (note.on.as_deref().map(&label), note.text.as_str()))
        .collect();
    format!(" {}", text::notes(locale, &notes))
}

/// The caption of a diagram's table, followed by the notes no row carries: those on the whole
/// diagram and, where `in_row` says so, on frames; `label` names what a note explains.
pub(crate) fn caption<'a>(
    caption: String,
    locale: Locale,
    notes: &'a [NoteSpec],
    in_row: impl Fn(&str) -> bool,
    label: impl Fn(&str) -> &'a str,
) -> String {
    let rowless: Vec<(Option<&str>, &str)> = notes
        .iter()
        .filter(|note| note.on.as_deref().is_none_or(|on| !in_row(on)))
        .map(|note| (note.on.as_deref().map(&label), note.text.as_str()))
        .collect();
    if rowless.is_empty() {
        caption
    } else {
        format!("{caption}. {}", text::notes(locale, &rowless))
    }
}

/// The notes on the element `id` for its row, separated by semicolons.
pub(crate) fn of(notes: &[NoteSpec], id: &str) -> String {
    notes
        .iter()
        .filter(|note| note.on.as_deref() == Some(id))
        .map(|note| note.text.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}
