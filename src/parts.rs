//! Parts of a node: a thin stripe inside the bottom of a box of a flow chart or a tree, split
//! into the shares of its parts, such as the records kept and excluded at a step of a study
//! selection. One legend below the diagram names the parts; the description and the table give
//! every part with its value and share.

use crate::{
    layout::{LABEL_SIZE, count, format_value},
    metrics::TextMetrics,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, NumberStyle, PartSpec, ValueFormat},
    text,
};

/// How much taller a box with parts is drawn, and the stripe in that room.
pub(crate) const ROOM: f64 = 12.0;
const STRIPE: f64 = 6.0;
/// How far the stripe keeps from the sides and the bottom of its box.
const INSET: f64 = 12.0;
const LIFT: f64 = 6.0;
const LEGEND_ROW: f64 = 22.0;

pub(crate) const STYLE: &str = ".chartlet-root{--chartlet-part-5:#0f766e;--chartlet-part-6:#6d28d9;--chartlet-part-7:#b45309;--chartlet-part-8:#be185d}.chartlet-root.chartlet-theme-dark{--chartlet-part-5:#2dd4bf;--chartlet-part-6:#b69cff;--chartlet-part-7:#fbbf24;--chartlet-part-8:#f472b6}.chartlet-part{stroke:none}.chartlet-part-1{fill:var(--chartlet-color-1)}.chartlet-part-2{fill:var(--chartlet-color-2)}.chartlet-part-3{fill:var(--chartlet-color-3)}.chartlet-part-4{fill:var(--chartlet-color-4)}.chartlet-part-5{fill:var(--chartlet-part-5)}.chartlet-part-6{fill:var(--chartlet-part-6)}.chartlet-part-7{fill:var(--chartlet-part-7)}.chartlet-part-8{fill:var(--chartlet-part-8)}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";

/// The stripe of a component whose category a perspective does not state: an outline without
/// a color of its own.
pub(crate) const UNSTATED_STYLE: &str = ".chartlet-part-none{fill:none;stroke:var(--chartlet-muted);stroke-width:1;stroke-dasharray:2 2}";
pub(crate) const UNSTATED: &str = "chartlet-part chartlet-part-none";

pub(crate) const CLASSES: [&str; 8] = [
    "chartlet-part chartlet-part-1",
    "chartlet-part chartlet-part-2",
    "chartlet-part chartlet-part-3",
    "chartlet-part chartlet-part-4",
    "chartlet-part chartlet-part-5",
    "chartlet-part chartlet-part-6",
    "chartlet-part chartlet-part-7",
    "chartlet-part chartlet-part-8",
];

/// The names of the parts of every node, in the order they first appear: each name keeps one
/// color across the diagram.
pub(crate) fn names<'a>(nodes: impl Iterator<Item = &'a [PartSpec]>) -> Vec<&'a str> {
    let mut names: Vec<&str> = Vec::new();
    for part in nodes.flatten() {
        if !names.contains(&part.label.as_str()) {
            names.push(&part.label);
        }
    }
    names
}

fn share(style: NumberStyle, value: f64, total: f64) -> String {
    format_value(
        value / total,
        NumberStyle {
            format: ValueFormat::Percent,
            decimals: Some(0),
            ..style
        },
    )
}

/// The parts of one node in words: `kept 180 (60 %), excluded 120 (40 %)`.
pub(crate) fn listed(style: NumberStyle, parts: &[PartSpec]) -> String {
    let total: f64 = parts.iter().map(|part| part.value).sum();
    parts
        .iter()
        .map(|part| {
            format!(
                "{} {} ({})",
                part.label,
                format_value(part.value, style),
                share(style, part.value, total)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The stripe of a node's parts inside the bottom of its box `(x, y, width, height)`; `lift`
/// raises it above a curved bottom.
pub(crate) fn push_stripe(
    style: NumberStyle,
    (parts, names): (&[PartSpec], &[&str]),
    (x, y, width, height): (f64, f64, f64, f64),
    lift: f64,
    elements: &mut Vec<Element>,
) {
    let total: f64 = parts.iter().map(|part| part.value).sum();
    let room = width - 2.0 * INSET;
    let mut at = x + INSET;
    for part in parts {
        let span = room * part.value / total;
        let color = names
            .iter()
            .position(|name| *name == part.label)
            .expect("every part's name is listed");
        elements.push(Element::Rect(Rect {
            x: at,
            y: y + height - LIFT - lift - STRIPE,
            width: span,
            height: STRIPE,
            class: CLASSES[color],
            series_index: None,
            style_index: None,
            tooltip: Some(format!(
                "{} {} ({})",
                part.label,
                format_value(part.value, style),
                share(style, part.value, total)
            )),
        }));
        at += span;
    }
}

/// A stripe of class `class` across the bottom of a box `(x, y, width, height)`, named by its
/// tooltip; `lift` raises it above a curved bottom.
pub(crate) fn push_band(
    class: &'static str,
    name: &str,
    (x, y, width, height): (f64, f64, f64, f64),
    lift: f64,
    elements: &mut Vec<Element>,
) {
    elements.push(Element::Rect(Rect {
        x: x + INSET,
        y: y + height - LIFT - lift - STRIPE,
        width: width - 2.0 * INSET,
        height: STRIPE,
        class,
        series_index: None,
        style_index: None,
        tooltip: Some(name.to_owned()),
    }));
}

/// The legend of the parts below a diagram, entries in rows that fit; the canvas grows by its
/// height.
pub(crate) fn push_legend(
    scene: &mut Scene,
    names: &[&str],
    margin: f64,
    metrics: &impl TextMetrics,
) {
    let entries: Vec<(&'static str, &str)> = names
        .iter()
        .enumerate()
        .map(|(index, name)| (CLASSES[index], *name))
        .collect();
    push_entries(scene, None, &entries, margin, metrics);
}

/// A legend of stripes below a diagram: a `title` in front, then each entry as the class of its
/// stripe and its name, in rows that fit; the canvas grows by its height.
pub(crate) fn push_entries(
    scene: &mut Scene,
    title: Option<&str>,
    entries: &[(&'static str, &str)],
    margin: f64,
    metrics: &impl TextMetrics,
) {
    if entries.is_empty() {
        return;
    }
    let room = f64::from(scene.width) - 2.0 * margin;
    let top = f64::from(scene.height) + 4.0;
    let (mut x, mut row) = (0.0, 0);
    if let Some(title) = title {
        scene.elements.push(Element::Text(Text {
            x: margin,
            y: top + 14.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: format!("{title}:"),
        }));
        x = metrics.width(title, LABEL_SIZE) + 16.0;
    }
    for &(class, name) in entries {
        let width = 22.0 + metrics.width(name, LABEL_SIZE) + 20.0;
        if x > 0.0 && x + width > room {
            x = 0.0;
            row += 1;
        }
        let y = top + 14.0 + LEGEND_ROW * count(row);
        scene.elements.push(Element::Rect(Rect {
            x: margin + x,
            y: y - 8.0,
            width: 16.0,
            height: STRIPE + 2.0,
            class,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        scene.elements.push(Element::Text(Text {
            x: margin + x + 22.0,
            y,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: (*name).to_owned(),
        }));
        x += width;
    }
    scene.height = crate::diagram::pixels(top + LEGEND_ROW * count(row + 1) + 6.0);
}

/// The sentence with the parts of every node that has some, nodes as `(label, parts)` in reading
/// order.
pub(crate) fn sentence(spec: &ChartSpec, nodes: &[(&str, &[PartSpec])]) -> String {
    let entries: Vec<String> = nodes
        .iter()
        .map(|(label, parts)| format!("{label}: {}", listed(spec.number_style(), parts)))
        .collect();
    text::node_parts(spec.locale, &entries)
}
