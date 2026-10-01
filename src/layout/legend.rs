use super::{
    AREA_CLASSES, LABEL_LINE, LABEL_SIZE, LEGEND_HEIGHT, LEGEND_ROW, PlotArea, count, fit_text,
    series_bar_class, timechart::line_class, title::two_lines,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Element, Polyline, Rect, Text, TextAnchor},
    spec::{ChartSpec, Dataset, LayerRef, Mark},
};

/// Length of the line sample in a time chart's legend.
const LEGEND_LINE: f64 = 24.0;
/// Width of a bar chart's legend swatch with the space before its name.
const LEGEND_SWATCH: f64 = 16.0;

/// One entry of a time chart's legend: the layer it shows, its text, and where it sits.
struct LegendEntry<'a> {
    entry: LayerRef<'a>,
    name: String,
    /// The name on two lines, when it is wider than a whole row: the entry then takes two rows.
    wrapped: Option<(String, String)>,
    x: f64,
    row: usize,
}

/// One legend entry per series name, in order of first appearance, placed in rows: an entry that
/// would reach past `available_width` starts a new row. A modeled series says so in its entry,
/// because the dashing alone is not a legend.
fn legend_entries<'a>(
    spec: &'a ChartSpec,
    left: f64,
    available_width: f64,
    metrics: &impl TextMetrics,
) -> Vec<LegendEntry<'a>> {
    let mut layers: Vec<LayerRef> = Vec::new();
    for entry in spec.data_layers() {
        if !layers
            .iter()
            .any(|known| known.layer.name == entry.layer.name)
        {
            layers.push(entry);
        }
    }
    let words = spec.locale.words();
    let sample = LEGEND_LINE + 8.0;
    let (mut x, mut row) = (left, 0);
    let mut entries = Vec::new();
    for entry in layers {
        let name = entry.layer.name.as_deref().unwrap_or(words.value);
        let name = if entry.layer.modeled {
            format!("{name} ({})", words.modeled)
        } else if entry.layer.mark == Mark::Ohlc {
            format!("{name} ({})", words.candle_key)
        } else {
            name.to_owned()
        };
        let room = available_width - sample;
        let wrapped = two_lines(&name, room, LABEL_SIZE, metrics)
            .map(|(first, second)| (first.to_owned(), second.to_owned()));
        let width = metrics.width(&name, LABEL_SIZE).min(room);
        // A name that needs two lines starts a row of its own and fills it.
        if x > left && (wrapped.is_some() || x + sample + width > left + available_width) {
            x = left;
            row += 1;
        }
        let rows = if wrapped.is_some() { 2 } else { 1 };
        entries.push(LegendEntry {
            entry,
            name,
            wrapped,
            x,
            row,
        });
        if rows == 2 {
            x = left + available_width;
            row += 1;
        } else {
            x += sample + width + 20.0;
        }
    }
    entries
}

/// The number of rows the legend of a time chart or of small multiples takes.
pub(super) fn legend_rows(
    spec: &ChartSpec,
    available_width: f64,
    metrics: &impl TextMetrics,
) -> usize {
    legend_entries(spec, 0.0, available_width, metrics)
        .last()
        .map_or(1, |entry| {
            entry.row + if entry.wrapped.is_some() { 2 } else { 1 }
        })
}

/// Draws the legend of a time chart or of small multiples, see [`legend_entries`]. Every sample is
/// a short piece of the line itself, over a swatch of its fill for an area.
pub(super) fn add_layer_legend(
    spec: &ChartSpec,
    row: f64,
    left: f64,
    available_width: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    for LegendEntry {
        entry,
        name,
        wrapped,
        x,
        row: line,
    } in legend_entries(spec, left, available_width, metrics)
    {
        let y = row + count(line) * LEGEND_HEIGHT;
        let explicit = entry.layer.resolved_color().is_some();
        if entry.layer.mark == Mark::Ohlc {
            crate::ohlc::push_legend_sample(x, y, elements);
        } else if entry.layer.mark == Mark::Area {
            elements.push(Element::Rect(Rect {
                x,
                y: y + 5.0,
                width: LEGEND_LINE,
                height: 8.0,
                class: if explicit {
                    "chartlet-area"
                } else {
                    AREA_CLASSES[spec.palette_index(entry.layer)]
                },
                series_index: None,
                style_index: explicit.then_some(entry.global),
                tooltip: None,
            }));
        }
        // A short piece of the line itself: color, pattern and weight, so that a legend entry
        // never rests on color alone.
        if entry.layer.mark != Mark::Ohlc {
            elements.push(Element::Polyline(Polyline {
                points: vec![(x, y + 5.0), (x + LEGEND_LINE, y + 5.0)],
                class: line_class(spec, entry),
                topic: None,
                series_index: None,
                style_index: explicit.then_some(entry.global),
                tooltip: None,
            }));
        }
        let room = left + available_width - x - LEGEND_LINE - 8.0;
        let path = format!("/panes/{}/layers/{}/name", entry.pane, entry.local);
        let lines = match wrapped {
            Some((first, second)) => vec![first, second],
            None => vec![name],
        };
        for (index, line_text) in lines.iter().enumerate() {
            let label = fit_text(line_text, room, LABEL_SIZE, metrics, warnings, &path);
            elements.push(Element::Text(Text {
                x: x + LEGEND_LINE + 8.0,
                y: y + 9.0 + count(index) * LABEL_LINE,
                class: "chartlet-legend",
                anchor: TextAnchor::Start,
                content: label,
            }));
        }
    }
}

/// Where each series' entry of a bar chart's legend goes, as its offset from the left of the
/// legend and its row: an entry that would reach past `available_width` starts a new row, as in
/// [`legend_entries`]. Empty for a single series, which has no legend.
fn bar_legend_entries(
    dataset: &Dataset,
    available_width: f64,
    metrics: &impl TextMetrics,
) -> Vec<SeriesEntry> {
    series_legend_entries(dataset, available_width, LEGEND_SWATCH, metrics)
}

/// One entry of a series legend: its offset from the left of the legend, its row, and its name
/// on one line or, when it is wider than a whole row, on two, taking two rows.
struct SeriesEntry {
    offset: f64,
    row: usize,
    lines: Vec<String>,
}

/// Where the entries of a series legend sit when each takes `swatch` before its name.
fn series_legend_entries(
    dataset: &Dataset,
    available_width: f64,
    swatch: f64,
    metrics: &impl TextMetrics,
) -> Vec<SeriesEntry> {
    if dataset.series.len() < 2 {
        return Vec::new();
    }
    let room = available_width - swatch;
    let (mut x, mut row) = (0.0, 0);
    let mut entries = Vec::new();
    for series in &dataset.series {
        let name = series
            .name
            .as_deref()
            .expect("multi-series charts name every series");
        let lines = match two_lines(name, room, LABEL_SIZE, metrics) {
            Some((first, second)) => vec![first.to_owned(), second.to_owned()],
            None => vec![name.to_owned()],
        };
        let width = metrics.width(name, LABEL_SIZE).min(room);
        // A name that needs two lines starts a row of its own and fills it.
        if x > 0.0 && (lines.len() > 1 || x + swatch + width > available_width) {
            x = 0.0;
            row += 1;
        }
        let rows = lines.len();
        entries.push(SeriesEntry {
            offset: x,
            row,
            lines,
        });
        if rows > 1 {
            x = available_width;
            row += rows - 1;
        } else {
            x += swatch + width + 20.0;
        }
    }
    entries
}

/// The height of a series legend: one row height per row its entries take.
fn series_legend_height(entries: &[SeriesEntry]) -> f64 {
    entries.last().map_or(0.0, |entry| {
        count(entry.row + entry.lines.len()) * LEGEND_HEIGHT
    })
}

/// The height the legend of a multi-series bar chart takes above the plot.
pub(super) fn legend_space(
    dataset: &Dataset,
    available_width: f64,
    metrics: &impl TextMetrics,
) -> f64 {
    series_legend_height(&bar_legend_entries(dataset, available_width, metrics))
}

/// The height the legend of a categorical line chart with several series takes above the plot.
pub(super) fn line_legend_space(
    dataset: &Dataset,
    available_width: f64,
    metrics: &impl TextMetrics,
) -> f64 {
    series_legend_height(&series_legend_entries(
        dataset,
        available_width,
        LEGEND_LINE + 6.0,
        metrics,
    ))
}

/// Writes the name of series `index` at `(x, y)`, line by line, each shortened only if it does
/// not fit `room`.
fn push_series_name(
    elements: &mut Vec<Element>,
    (x, y): (f64, f64),
    lines: &[String],
    room: f64,
    index: usize,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    for (line, text) in lines.iter().enumerate() {
        let label = fit_text(
            text,
            room,
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/series/{index}/name"),
        );
        elements.push(Element::Text(Text {
            x,
            y: y + 9.0 + count(line) * LABEL_LINE,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: label,
        }));
    }
}

/// Draws the legend of a categorical line chart with several series above `plot`: a sample of
/// each line, in its color and pattern, before its name.
pub(super) fn add_line_legend(
    dataset: &Dataset,
    classes: &[&'static str],
    plot: PlotArea,
    head: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let swatch = LEGEND_LINE + 6.0;
    let (left, top, available_width) = (plot.left, LEGEND_ROW + head, plot.width);
    let entries = series_legend_entries(dataset, available_width, swatch, metrics);
    for (index, entry) in entries.iter().enumerate() {
        let (x, y) = (left + entry.offset, top + count(entry.row) * LEGEND_HEIGHT);
        elements.push(Element::Polyline(Polyline {
            points: vec![(x, y + 5.0), (x + LEGEND_LINE, y + 5.0)],
            class: classes[index],
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        push_series_name(
            elements,
            (x + swatch, y),
            &entry.lines,
            available_width - swatch,
            index,
            warnings,
            metrics,
        );
    }
}

/// Draws the legend of a multi-series bar chart above `plot`, below a title that takes `head`
/// more than one line.
pub(super) fn add_legend(
    spec: &ChartSpec,
    dataset: &Dataset,
    plot: PlotArea,
    head: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let (left, top, available_width) = (plot.left, LEGEND_ROW + head, plot.width);
    let entries = bar_legend_entries(dataset, available_width, metrics);
    for (index, entry) in entries.iter().enumerate() {
        let (x, y) = (left + entry.offset, top + count(entry.row) * LEGEND_HEIGHT);
        elements.push(Element::Rect(Rect {
            x,
            y,
            width: 10.0,
            height: 10.0,
            class: series_bar_class(spec, index),
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        push_series_name(
            elements,
            (x + LEGEND_SWATCH, y),
            &entry.lines,
            available_width - LEGEND_SWATCH,
            index,
            warnings,
            metrics,
        );
    }
}
