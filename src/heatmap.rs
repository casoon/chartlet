//! Heatmaps: values, or categories, in a grid of rows and columns. A value is a step of a
//! sequential or a diverging scale, written into its cell where it fits, with a text color that
//! keeps at least 4.5:1 against the fill. A category has a fill and a symbol, so that color is
//! never the only cue. A cell that is not listed is drawn as an empty, dashed box. The key, the
//! description and the data table name every value: the grid is for the eye, not the only place a
//! value is said.

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{LABEL_SIZE, count, fit_text, format_value, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Circle, Element, Polyline, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, HeatmapScale, HeatmapSpec},
    text,
};

const SEQUENTIAL_CELLS: [&str; 6] = [
    "chartlet-hm-cell chartlet-hm-s0",
    "chartlet-hm-cell chartlet-hm-s1",
    "chartlet-hm-cell chartlet-hm-s2",
    "chartlet-hm-cell chartlet-hm-s3",
    "chartlet-hm-cell chartlet-hm-s4",
    "chartlet-hm-cell chartlet-hm-s5",
];
const SEQUENTIAL_KEYS: [&str; 6] = [
    "chartlet-hm-key chartlet-hm-s0",
    "chartlet-hm-key chartlet-hm-s1",
    "chartlet-hm-key chartlet-hm-s2",
    "chartlet-hm-key chartlet-hm-s3",
    "chartlet-hm-key chartlet-hm-s4",
    "chartlet-hm-key chartlet-hm-s5",
];
const DIVERGING_CELLS: [&str; 7] = [
    "chartlet-hm-cell chartlet-hm-d0",
    "chartlet-hm-cell chartlet-hm-d1",
    "chartlet-hm-cell chartlet-hm-d2",
    "chartlet-hm-cell chartlet-hm-d3",
    "chartlet-hm-cell chartlet-hm-d4",
    "chartlet-hm-cell chartlet-hm-d5",
    "chartlet-hm-cell chartlet-hm-d6",
];
const DIVERGING_KEYS: [&str; 7] = [
    "chartlet-hm-key chartlet-hm-d0",
    "chartlet-hm-key chartlet-hm-d1",
    "chartlet-hm-key chartlet-hm-d2",
    "chartlet-hm-key chartlet-hm-d3",
    "chartlet-hm-key chartlet-hm-d4",
    "chartlet-hm-key chartlet-hm-d5",
    "chartlet-hm-key chartlet-hm-d6",
];
const CATEGORY_CELLS: [&str; crate::spec::MAX_SERIES] = [
    "chartlet-hm-cell chartlet-hm-c1",
    "chartlet-hm-cell chartlet-hm-c2",
    "chartlet-hm-cell chartlet-hm-c3",
    "chartlet-hm-cell chartlet-hm-c4",
];
const CATEGORY_KEYS: [&str; crate::spec::MAX_SERIES] = [
    "chartlet-hm-key chartlet-hm-c1",
    "chartlet-hm-key chartlet-hm-c2",
    "chartlet-hm-key chartlet-hm-c3",
    "chartlet-hm-key chartlet-hm-c4",
];
const ON_LIGHT: &str = "chartlet-hm-value chartlet-hm-on-light";
const ON_DARK: &str = "chartlet-hm-value chartlet-hm-on-dark";
/// The steps of the diverging scale and the middle one, where the value is the midpoint.
const DIVERGING_STEPS: usize = 7;
const MIDDLE: usize = DIVERGING_STEPS / 2;
/// The widest a cell gets, and the range of its height.
const MAX_CELL_WIDTH: f64 = 96.0;
const CELL_HEIGHT: (f64, f64) = (20.0, 36.0);
/// The height of the column labels, the swatch of the key, and the height of a legend line.
const HEADER: f64 = 22.0;
const SWATCH: (f64, f64) = (28.0, 12.0);
const ENTRY: f64 = 22.0;

fn heatmap(spec: &ChartSpec) -> &HeatmapSpec {
    spec.heatmap
        .as_ref()
        .expect("validated heatmaps carry a heatmap block")
}

/// What a cell shows.
enum Content {
    /// No value: the cell is not listed.
    Missing,
    /// A value, and the step of the scale it falls on.
    Value(f64, usize),
    /// A category, by its index.
    Category(usize),
}

/// The step of every value on the scale: six on a sequential one, from the smallest value to the
/// largest; seven on a diverging one, parted at the midpoint, the outermost as far from it as the
/// farthest value.
fn steps(heatmap: &HeatmapSpec) -> Vec<(usize, Option<usize>)> {
    let values: Vec<Option<f64>> = heatmap.cells.iter().map(|cell| cell.value).collect();
    if heatmap.scale == Some(HeatmapScale::Diverging) {
        let midpoint = heatmap.midpoint();
        let reach = values
            .iter()
            .flatten()
            .map(|value| (value - midpoint).abs())
            .fold(0.0, f64::max);
        values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let step = value.map(|value| {
                    if reach > 0.0 {
                        // A share of -1 to 1 times three steps, rounded: a small whole number.
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let offset = ((value - midpoint) / reach * 3.0 + 3.0).round() as usize;
                        offset.min(DIVERGING_STEPS - 1)
                    } else {
                        MIDDLE
                    }
                });
                (index, step)
            })
            .collect()
    } else {
        crate::heat::steps(&values)
            .into_iter()
            .enumerate()
            .collect()
    }
}

/// The content of the cell at a row and a column, by index.
fn content(
    heatmap: &HeatmapSpec,
    steps: &[(usize, Option<usize>)],
    row: usize,
    column: usize,
) -> Content {
    let Some(position) = heatmap
        .cells
        .iter()
        .position(|cell| cell.row == heatmap.rows[row] && cell.column == heatmap.columns[column])
    else {
        return Content::Missing;
    };
    let cell = &heatmap.cells[position];
    match (&cell.category, cell.value, steps[position].1) {
        (Some(category), _, _) => Content::Category(
            heatmap
                .categories
                .iter()
                .position(|label| label == category)
                .expect("validated cells name a category"),
        ),
        (None, Some(value), Some(step)) => Content::Value(value, step),
        _ => Content::Missing,
    }
}

/// Whether the text on a step is white: on the darkest steps.
const fn on_dark(diverging: bool, step: usize) -> bool {
    if diverging {
        step == 0 || step == DIVERGING_STEPS - 1
    } else {
        step >= 4
    }
}

/// A symbol for a category, centered on a point: a circle, a square, a triangle or a diamond.
fn push_symbol(elements: &mut Vec<Element>, category: usize, (x, y): (f64, f64), radius: f64) {
    const CLASS: &str = "chartlet-hm-symbol";
    match category {
        0 => elements.push(Element::Circle(Circle {
            cx: x,
            cy: y,
            radius,
            class: CLASS,
            series_index: None,
            style_index: None,
            tooltip: None,
        })),
        1 => elements.push(Element::Rect(Rect {
            x: x - radius,
            y: y - radius,
            width: 2.0 * radius,
            height: 2.0 * radius,
            class: CLASS,
            series_index: None,
            style_index: None,
            tooltip: None,
        })),
        _ => {
            let points = if category == 2 {
                vec![
                    (x, y - radius),
                    (x + radius, y + radius),
                    (x - radius, y + radius),
                    (x, y - radius),
                ]
            } else {
                vec![
                    (x, y - radius),
                    (x + radius, y),
                    (x, y + radius),
                    (x - radius, y),
                    (x, y - radius),
                ]
            };
            elements.push(Element::Polyline(Polyline {
                points,
                class: CLASS,
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
    }
}

/// The words for a cell's content, in its tooltip, its description and its table.
fn words(spec: &ChartSpec, content: &Content) -> String {
    let heatmap = heatmap(spec);
    match content {
        Content::Missing => spec.locale.words().no_value.to_owned(),
        Content::Value(value, _) => format_value(*value, spec.number_style()),
        Content::Category(category) => heatmap.categories[*category].clone(),
    }
}

/// What the numbers stand for: the heatmap's `valueLabel`, or the word for a value.
fn value_name(spec: &ChartSpec) -> String {
    heatmap(spec)
        .value_label
        .clone()
        .unwrap_or_else(|| spec.locale.words().value.to_owned())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let heatmap = heatmap(spec);
    let (rows, columns) = heatmap.order();
    let steps = steps(heatmap);
    let diverging = heatmap.scale == Some(HeatmapScale::Diverging);
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
    let inner = width - 2.0 * margin;
    let label_width = rows
        .iter()
        .map(|row| metrics.width(&heatmap.rows[*row], LABEL_SIZE))
        .fold(0.0, f64::max)
        .min(inner * 0.35)
        + 10.0;
    let cell_width = ((inner - label_width) / count(columns.len()))
        .min(MAX_CELL_WIDTH)
        .floor()
        .max(8.0);
    let missing = rows
        .iter()
        .flat_map(|row| columns.iter().map(move |column| (*row, *column)))
        .filter(|(row, column)| matches!(content(heatmap, &steps, *row, *column), Content::Missing))
        .count();
    let legend_lines = if heatmap.is_categorical() {
        heatmap.categories.len() + usize::from(missing > 0)
    } else {
        1 + usize::from(missing > 0)
    };
    let legend_height = 24.0 + ENTRY * count(legend_lines);
    let grid_top = top + HEADER;
    let cell_height = ((height - grid_top - legend_height - 16.0) / count(rows.len()))
        .clamp(CELL_HEIGHT.0, CELL_HEIGHT.1)
        .floor();
    let left = margin + label_width;
    for (position, column) in columns.iter().enumerate() {
        elements.push(Element::Text(Text {
            x: left + (count(position) + 0.5) * cell_width,
            y: top + 14.0,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content: fit_text(
                &heatmap.columns[*column],
                cell_width - 4.0,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/heatmap/columns/{column}"),
            ),
        }));
    }
    for (line, row) in rows.iter().enumerate() {
        let y = grid_top + count(line) * cell_height;
        elements.push(Element::Text(Text {
            x: left - 8.0,
            y: y + cell_height / 2.0 + 4.0,
            class: "chartlet-label",
            anchor: TextAnchor::End,
            content: fit_text(
                &heatmap.rows[*row],
                label_width - 10.0,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/heatmap/rows/{row}"),
            ),
        }));
        for (position, column) in columns.iter().enumerate() {
            let x = left + count(position) * cell_width;
            let cell = content(heatmap, &steps, *row, *column);
            let class = match &cell {
                Content::Missing => "chartlet-hm-missing",
                Content::Value(_, step) if diverging => DIVERGING_CELLS[*step],
                Content::Value(_, step) => SEQUENTIAL_CELLS[*step],
                Content::Category(category) => CATEGORY_CELLS[*category],
            };
            elements.push(Element::Rect(Rect {
                x,
                y,
                width: cell_width,
                height: cell_height,
                class,
                series_index: None,
                style_index: None,
                tooltip: Some(format!(
                    "{}, {}: {}",
                    heatmap.rows[*row],
                    heatmap.columns[*column],
                    words(spec, &cell)
                )),
            }));
            let center = (x + cell_width / 2.0, y + cell_height / 2.0);
            match &cell {
                Content::Value(value, step) => {
                    let text = format_value(*value, spec.number_style());
                    if metrics.width(&text, LABEL_SIZE) <= cell_width - 6.0 {
                        elements.push(Element::Text(Text {
                            x: center.0,
                            y: center.1 + 4.0,
                            class: if on_dark(diverging, *step) {
                                ON_DARK
                            } else {
                                ON_LIGHT
                            },
                            anchor: TextAnchor::Middle,
                            content: text,
                        }));
                    }
                }
                Content::Category(category) => push_symbol(&mut elements, *category, center, 5.0),
                Content::Missing => {}
            }
        }
    }
    let grid_bottom = grid_top + count(rows.len()) * cell_height;
    let mut y = grid_bottom + 16.0;
    if heatmap.is_categorical() {
        for (index, label) in heatmap.categories.iter().enumerate() {
            elements.push(Element::Rect(Rect {
                x: margin,
                y,
                width: SWATCH.0 / 2.0,
                height: SWATCH.1,
                class: CATEGORY_KEYS[index],
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
            push_symbol(
                &mut elements,
                index,
                (margin + SWATCH.0 / 4.0, y + SWATCH.1 / 2.0),
                4.0,
            );
            elements.push(Element::Text(Text {
                x: margin + SWATCH.0 / 2.0 + 8.0,
                y: y + 10.5,
                class: "chartlet-legend",
                anchor: TextAnchor::Start,
                content: fit_text(
                    label,
                    inner - SWATCH.0,
                    LABEL_SIZE,
                    metrics,
                    warnings,
                    &format!("/heatmap/categories/{index}"),
                ),
            }));
            y += ENTRY;
        }
    } else {
        y = push_key(
            &mut elements,
            spec,
            steps_range(heatmap),
            y,
            margin,
            metrics,
        );
    }
    if missing > 0 {
        elements.push(Element::Rect(Rect {
            x: margin,
            y,
            width: SWATCH.0 / 2.0,
            height: SWATCH.1,
            class: "chartlet-hm-missing",
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: margin + SWATCH.0 / 2.0 + 8.0,
            y: y + 10.5,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: spec.locale.words().no_value.to_owned(),
        }));
        y += ENTRY;
    }
    Scene {
        width: spec.width,
        height: pixels(height.max(y + 8.0)),
        elements,
    }
}

/// The smallest and the largest value, and where a diverging scale parts.
fn steps_range(heatmap: &HeatmapSpec) -> (f64, f64) {
    let values: Vec<Option<f64>> = heatmap.cells.iter().map(|cell| cell.value).collect();
    crate::heat::range(&values).unwrap_or((0.0, 0.0))
}

/// The key of a numeric heatmap: what the values are, then the steps of the scale with the ends
/// written under them, and the midpoint under the middle of a diverging one. Returns where the
/// next line of the legend starts.
fn push_key(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (low, high): (f64, f64),
    top: f64,
    margin: f64,
    metrics: &impl TextMetrics,
) -> f64 {
    let heatmap = heatmap(spec);
    let diverging = heatmap.scale == Some(HeatmapScale::Diverging);
    let keys: &[&'static str] = if diverging {
        &DIVERGING_KEYS
    } else {
        &SEQUENTIAL_KEYS
    };
    let name = value_name(spec);
    let style = spec.number_style();
    elements.push(Element::Text(Text {
        x: margin,
        y: top + 10.5,
        class: "chartlet-legend",
        anchor: TextAnchor::Start,
        content: name.clone(),
    }));
    let left = margin + metrics.width(&name, LABEL_SIZE) + 12.0;
    for (step, class) in keys.iter().enumerate() {
        elements.push(Element::Rect(Rect {
            x: left + SWATCH.0 * count(step),
            y: top,
            width: SWATCH.0,
            height: SWATCH.1,
            class,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
    let right = left + SWATCH.0 * count(keys.len());
    let mut ends = vec![
        (left, low, TextAnchor::Start),
        (right, high, TextAnchor::End),
    ];
    if diverging {
        ends.push((
            f64::midpoint(left, right),
            heatmap.midpoint(),
            TextAnchor::Middle,
        ));
    }
    for (x, value, anchor) in ends {
        elements.push(Element::Text(Text {
            x,
            y: top + 28.0,
            class: "chartlet-legend",
            anchor,
            content: format_value(value, style),
        }));
    }
    top + ENTRY + 8.0
}

/// The heatmap in a sentence: its size, what the colors or the categories show, the highest and
/// the lowest cell, and how many cells have no value. The data table has every cell.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let heatmap = heatmap(spec);
    let (rows, columns) = heatmap.order();
    let steps = steps(heatmap);
    let size = (heatmap.rows.len(), heatmap.columns.len());
    let missing = heatmap.rows.len() * heatmap.columns.len() - heatmap.cells.len();
    if heatmap.is_categorical() {
        let counts: Vec<String> = heatmap
            .categories
            .iter()
            .map(|category| {
                let cells = heatmap
                    .cells
                    .iter()
                    .filter(|cell| cell.category.as_ref() == Some(category))
                    .count();
                format!("{category} {cells}")
            })
            .collect();
        return text::heatmap_categories(spec.locale, size, &counts.join(", "), missing);
    }
    let style = spec.number_style();
    let (low, high) = steps_range(heatmap);
    let named = |pick: fn(f64, f64) -> bool| {
        let mut found: Option<(usize, usize, f64)> = None;
        for row in &rows {
            for column in &columns {
                if let Content::Value(value, _) = content(heatmap, &steps, *row, *column)
                    && found.is_none_or(|(_, _, best)| pick(value, best))
                {
                    found = Some((*row, *column, value));
                }
            }
        }
        found.map_or_else(String::new, |(row, column, value)| {
            format!(
                "{}, {} ({})",
                heatmap.rows[row],
                heatmap.columns[column],
                format_value(value, style)
            )
        })
    };
    text::heatmap_values(
        spec.locale,
        size,
        &value_name(spec),
        (
            (
                &format_value(low, style),
                &named(|value, best| value < best),
            ),
            (
                &format_value(high, style),
                &named(|value, best| value > best),
            ),
        ),
        missing,
    )
}

/// One row per cell, in the order drawn: its row, its column, and its value or category, or the
/// words for no value.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let words_of = spec.locale.words();
    let heatmap = heatmap(spec);
    let (rows, columns) = heatmap.order();
    let steps = steps(heatmap);
    let mut table_rows = Vec::new();
    for row in &rows {
        for column in &columns {
            let cell = content(heatmap, &steps, *row, *column);
            table_rows.push(vec![
                heatmap.rows[*row].clone(),
                heatmap.columns[*column].clone(),
                words(spec, &cell),
            ]);
        }
    }
    DataTable {
        caption: format!("{} {}", words_of.data_for, spec.title),
        columns: vec![
            words_of.row.to_owned(),
            words_of.column.to_owned(),
            if heatmap.is_categorical() {
                words_of.category.to_owned()
            } else {
                value_name(spec)
            },
        ],
        rows: table_rows,
    }
}
