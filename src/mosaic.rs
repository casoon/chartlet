//! Mosaic (Marimekko) charts: columns as wide as their share of the whole, each split by the share
//! of its segments. Segments of one label share a color across the columns, are written with their
//! share where they fit, and a legend names them; the column names and totals stand under the
//! columns. The description and the data table have both levels: every column with its share of
//! the whole and every segment with its share of its column.

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{LABEL_SIZE, count, fit_text, format_value, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, MAX_SERIES, MosaicSpec, NumberStyle, ValueFormat},
    text,
};

const CELLS: [&str; MAX_SERIES] = [
    "chartlet-mosaic-cell chartlet-mosaic-1",
    "chartlet-mosaic-cell chartlet-mosaic-2",
    "chartlet-mosaic-cell chartlet-mosaic-3",
    "chartlet-mosaic-cell chartlet-mosaic-4",
];
const SWATCHES: [&str; MAX_SERIES] = [
    "chartlet-mosaic-swatch chartlet-mosaic-1",
    "chartlet-mosaic-swatch chartlet-mosaic-2",
    "chartlet-mosaic-swatch chartlet-mosaic-3",
    "chartlet-mosaic-swatch chartlet-mosaic-4",
];
/// The gap between two columns, the room under the columns for their names, and the height of a
/// legend row.
const GAP: f64 = 3.0;
const FOOTER: f64 = 40.0;
const ROW: f64 = 20.0;

fn mosaic(spec: &ChartSpec) -> &MosaicSpec {
    spec.mosaic
        .as_ref()
        .expect("validated mosaic charts carry a mosaic block")
}

/// A share as a percentage, with one decimal.
fn percent(spec: &ChartSpec, share: f64) -> String {
    format_value(
        share,
        NumberStyle {
            format: ValueFormat::Percent,
            decimals: Some(1),
            ..spec.number_style()
        },
    )
}

/// The legend of the segments in rows under the title. Returns where it ends.
fn push_legend(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (left, top): (f64, f64),
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) -> f64 {
    let right = f64::from(spec.width) - left;
    let (mut x, mut y) = (left, top);
    for (index, label) in mosaic(spec).segments().into_iter().enumerate() {
        let reach = 16.0 + metrics.width(label, LABEL_SIZE);
        if x > left && x + reach > right {
            x = left;
            y += ROW;
        }
        elements.push(Element::Rect(Rect {
            x,
            y,
            width: 10.0,
            height: 10.0,
            class: SWATCHES[index],
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: x + 16.0,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                label,
                (right - x - 16.0).max(0.0),
                LABEL_SIZE,
                metrics,
                warnings,
                "/mosaic/columns",
            ),
        }));
        x += reach + 16.0;
    }
    y + ROW + 6.0
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let mosaic = mosaic(spec);
    let segments = mosaic.segments();
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
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics) - 10.0;
    let top = push_legend(&mut elements, spec, (margin, top), (warnings, metrics));
    let plot_height = (height - top - FOOTER - 16.0).max(60.0);
    let columns = mosaic.columns.len();
    let usable = width - 2.0 * margin - GAP * count(columns - 1);
    let whole = mosaic.total();
    let style = spec.number_style();
    let mut x = margin;
    for (index, column) in mosaic.columns.iter().enumerate() {
        let total = mosaic.column_total(index);
        let column_width = usable * total / whole;
        let mut y = top;
        for label in &segments {
            let Some(segment) = column.segments.iter().find(|s| s.label == *label) else {
                continue;
            };
            let color = segments.iter().position(|l| l == label).unwrap_or(0);
            let cell_height = plot_height * segment.value / total;
            let share = percent(spec, segment.value / total);
            elements.push(Element::Rect(Rect {
                x,
                y,
                width: column_width,
                height: cell_height,
                class: CELLS[color],
                series_index: None,
                style_index: None,
                tooltip: Some(text::mosaic_tooltip(
                    spec.locale,
                    (
                        &column.label,
                        &segment.label,
                        &format_value(segment.value, style),
                    ),
                    (&share, &percent(spec, segment.value / whole)),
                )),
            }));
            if cell_height >= 18.0 && column_width >= metrics.width(&share, LABEL_SIZE) + 8.0 {
                elements.push(Element::Text(Text {
                    x: x + column_width / 2.0,
                    y: y + cell_height / 2.0 + 4.0,
                    class: "chartlet-mosaic-value",
                    anchor: TextAnchor::Middle,
                    content: share,
                }));
            }
            y += cell_height;
        }
        // The name of the column and its total with its share of the whole, under the column.
        let center = x + column_width / 2.0;
        let reach = column_width + GAP;
        elements.push(Element::Text(Text {
            x: center,
            y: top + plot_height + 16.0,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content: fit_text(
                &column.label,
                reach,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/mosaic/columns/{index}/label"),
            ),
        }));
        let summary = format!(
            "{} ({})",
            format_value(total, style),
            percent(spec, total / whole)
        );
        let short = percent(spec, total / whole);
        let fitting = [summary, short]
            .into_iter()
            .find(|text| metrics.width(text, LABEL_SIZE) <= reach);
        if let Some(content) = fitting {
            elements.push(Element::Text(Text {
                x: center,
                y: top + plot_height + 32.0,
                class: "chartlet-legend",
                anchor: TextAnchor::Middle,
                content,
            }));
        }
        x += column_width + GAP;
    }
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

/// The mosaic chart in sentences: its size, and every column with its share of the whole and the
/// share of each of its segments.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let mosaic = mosaic(spec);
    let style = spec.number_style();
    let whole = mosaic.total();
    let columns: Vec<String> = mosaic
        .columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let total = mosaic.column_total(index);
            let parts: Vec<String> = column
                .segments
                .iter()
                .map(|segment| {
                    format!("{} {}", segment.label, percent(spec, segment.value / total))
                })
                .collect();
            format!(
                "{} {} ({}): {}",
                column.label,
                percent(spec, total / whole),
                format_value(total, style),
                parts.join(", ")
            )
        })
        .collect();
    text::mosaic_summary(
        spec.locale,
        (
            mosaic.columns.len(),
            mosaic.segments().len(),
            &format_value(whole, style),
        ),
        &columns.join("; "),
    )
}

/// Both levels: for every column its segments with value, share of the column and share of the
/// whole, then the column itself as a total.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let words = spec.locale.words();
    let mosaic = mosaic(spec);
    let style = spec.number_style();
    let whole = mosaic.total();
    let mut rows = Vec::new();
    for (index, column) in mosaic.columns.iter().enumerate() {
        let total = mosaic.column_total(index);
        for segment in &column.segments {
            rows.push(vec![
                column.label.clone(),
                segment.label.clone(),
                format_value(segment.value, style),
                percent(spec, segment.value / total),
                percent(spec, segment.value / whole),
            ]);
        }
        rows.push(vec![
            column.label.clone(),
            words.total.to_owned(),
            format_value(total, style),
            percent(spec, 1.0),
            percent(spec, total / whole),
        ]);
    }
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: vec![
            words.column.to_owned(),
            words.segment.to_owned(),
            words.value.to_owned(),
            words.share_of_column.to_owned(),
            words.share_of_total.to_owned(),
        ],
        rows,
    }
}
