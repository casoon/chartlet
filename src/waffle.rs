//! Waffles: a grid of squares that each stand for the same share of a whole, filled by the parts
//! in reading order. The legend names every part with its value and share, and so do the
//! description and the data table: the grid is for the eye, never the only place a share is said.

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{LABEL_SIZE, count, fit_text, format_value, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, NumberStyle, ValueFormat, WaffleSpec},
    text,
};

const CELL_CLASSES: [&str; crate::spec::MAX_SERIES + 1] = [
    "chartlet-waffle-cell chartlet-waffle-1",
    "chartlet-waffle-cell chartlet-waffle-2",
    "chartlet-waffle-cell chartlet-waffle-3",
    "chartlet-waffle-cell chartlet-waffle-4",
    "chartlet-waffle-cell chartlet-waffle-rest",
];
/// The gap between two squares, the largest a square gets, and the height of a legend entry.
const GAP: f64 = 2.0;
const MAX_CELL: f64 = 32.0;
const ENTRY: f64 = 22.0;

fn waffle(spec: &ChartSpec) -> &WaffleSpec {
    spec.waffle
        .as_ref()
        .expect("validated waffles carry a waffle block")
}

/// A share of the whole as a percentage, with one decimal.
fn share(spec: &ChartSpec, value: f64, whole: f64) -> String {
    format_value(
        value / whole,
        NumberStyle {
            format: ValueFormat::Percent,
            decimals: Some(1),
            ..spec.number_style()
        },
    )
}

/// Every entry of the waffle: the parts, then the rest if the parts do not make up the whole. The
/// label, the value, the squares it takes and the class of its squares.
fn entries(spec: &ChartSpec) -> Vec<(String, f64, u32, &'static str)> {
    let waffle = waffle(spec);
    let squares = waffle.squares();
    let sum: f64 = waffle.parts.iter().map(|part| part.value).sum();
    let mut entries: Vec<_> = waffle
        .parts
        .iter()
        .enumerate()
        .map(|(index, part)| {
            (
                part.label.clone(),
                part.value,
                squares[index],
                CELL_CLASSES[index],
            )
        })
        .collect();
    if let Some(rest) = squares.get(waffle.parts.len()) {
        entries.push((
            spec.locale.words().rest.to_owned(),
            waffle.whole() - sum,
            *rest,
            CELL_CLASSES[crate::spec::MAX_SERIES],
        ));
    }
    entries
}

/// The legend: a swatch and the label, value and share of every entry, one under the other.
fn push_legend(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (entries, whole): (&[(String, f64, u32, &'static str)], f64),
    (left, top, width): (f64, f64, f64),
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) {
    for (position, (label, value, _, class)) in entries.iter().enumerate() {
        let y = top + ENTRY * count(position);
        elements.push(Element::Rect(Rect {
            x: left,
            y,
            width: 12.0,
            height: 12.0,
            class,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: left + 20.0,
            y: y + 10.5,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                &format!(
                    "{label} – {} ({})",
                    format_value(*value, spec.number_style()),
                    share(spec, *value, whole)
                ),
                width - 20.0,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/waffle/parts/{position}/label"),
            ),
        }));
    }
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let waffle = waffle(spec);
    let entries = entries(spec);
    let whole = waffle.whole();
    let compact = spec.width < crate::layout::NARROW;
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let margin = if compact { 12.0 } else { 24.0 };
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        width - 2.0 * margin,
        metrics,
        warnings,
    );
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let (columns, cells) = (waffle.columns(), waffle.cells());
    let rows = cells.div_ceil(columns);
    // The legend stands right of the grid on a wide canvas, below it on a narrow one.
    let beside = width >= 520.0;
    let legend_width = if beside {
        (width * 0.4).min(320.0)
    } else {
        width - 2.0 * margin
    };
    let legend_height = if beside {
        0.0
    } else {
        ENTRY * count(entries.len()) + 12.0
    };
    let grid_width = width - 2.0 * margin - if beside { legend_width + 24.0 } else { 0.0 };
    let grid_height = height - top - 16.0 - legend_height;
    let cell = (grid_width / f64::from(columns))
        .min(grid_height / f64::from(rows))
        .min(MAX_CELL)
        .floor()
        .max(6.0);
    let side = cell - GAP;
    let mut index = 0_u32;
    for (label, value, squares, class) in &entries {
        let tooltip = format!(
            "{label}: {} ({})",
            format_value(*value, spec.number_style()),
            share(spec, *value, whole)
        );
        for _ in 0..*squares {
            let (row, column) = (index / columns, index % columns);
            elements.push(Element::Rect(Rect {
                x: margin + f64::from(column) * cell,
                y: top + f64::from(row) * cell,
                width: side,
                height: side,
                class,
                series_index: None,
                style_index: None,
                tooltip: Some(tooltip.clone()),
            }));
            index += 1;
        }
    }
    let grid_bottom = top + f64::from(rows) * cell;
    let (legend_left, legend_top) = if beside {
        (width - margin - legend_width, top)
    } else {
        (margin, grid_bottom + 16.0)
    };
    push_legend(
        &mut elements,
        spec,
        (&entries, whole),
        (legend_left, legend_top, legend_width),
        (warnings, metrics),
    );
    let needed = if beside {
        grid_bottom.max(legend_top + ENTRY * count(entries.len())) + 16.0
    } else {
        legend_top + ENTRY * count(entries.len()) + 8.0
    };
    Scene {
        width: spec.width,
        height: pixels(height.max(needed)),
        elements,
    }
}

/// The waffle in a sentence: how many squares it has, what one stands for, and every part.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let waffle = waffle(spec);
    let whole = waffle.whole();
    let parts: Vec<String> = entries(spec)
        .iter()
        .map(|(label, value, squares, _)| {
            format!(
                "{label} {} ({}, {squares})",
                format_value(*value, spec.number_style()),
                share(spec, *value, whole)
            )
        })
        .collect();
    text::waffle_summary(
        spec.locale,
        (
            waffle.cells(),
            &format_value(whole / f64::from(waffle.cells()), spec.number_style()),
        ),
        &parts.join("; "),
    )
}

/// One row per part: its value, its share of the whole and the squares it takes.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let words = spec.locale.words();
    let whole = waffle(spec).whole();
    let rows = entries(spec)
        .iter()
        .map(|(label, value, squares, _)| {
            vec![
                label.clone(),
                format_value(*value, spec.number_style()),
                share(spec, *value, whole),
                squares.to_string(),
            ]
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: vec![
            words.part.to_owned(),
            words.value.to_owned(),
            words.share.to_owned(),
            words.squares.to_owned(),
        ],
        rows,
    }
}
