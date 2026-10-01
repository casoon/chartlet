use super::{
    AREA_CLASSES, LABEL_SIZE, LEGEND_HEIGHT, LEGEND_ROW, PlotArea, count, fit_text,
    series_bar_class, timechart::line_class,
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
        let width = metrics
            .width(&name, LABEL_SIZE)
            .min(available_width - sample);
        if x > left && x + sample + width > left + available_width {
            x = left;
            row += 1;
        }
        entries.push(LegendEntry {
            entry,
            name,
            x,
            row,
        });
        x += sample + width + 20.0;
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
        .map_or(1, |entry| entry.row + 1)
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
        let label = fit_text(
            &name,
            left + available_width - x - LEGEND_LINE - 8.0,
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/panes/{}/layers/{}/name", entry.pane, entry.local),
        );
        elements.push(Element::Text(Text {
            x: x + LEGEND_LINE + 8.0,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: label,
        }));
    }
}

/// Where each series' entry of a bar chart's legend goes, as its offset from the left of the
/// legend and its row: an entry that would reach past `available_width` starts a new row, as in
/// [`legend_entries`]. Empty for a single series, which has no legend.
fn bar_legend_entries(
    dataset: &Dataset,
    available_width: f64,
    metrics: &impl TextMetrics,
) -> Vec<(f64, usize)> {
    series_legend_entries(dataset, available_width, LEGEND_SWATCH, metrics)
}

/// Where the entries of a series legend sit, as offset and row, when each takes `swatch` before
/// its name.
fn series_legend_entries(
    dataset: &Dataset,
    available_width: f64,
    swatch: f64,
    metrics: &impl TextMetrics,
) -> Vec<(f64, usize)> {
    if dataset.series.len() < 2 {
        return Vec::new();
    }
    let (mut x, mut row) = (0.0, 0);
    let mut entries = Vec::new();
    for series in &dataset.series {
        let name = series
            .name
            .as_deref()
            .expect("multi-series charts name every series");
        let width = metrics
            .width(name, LABEL_SIZE)
            .min(available_width - swatch);
        if x > 0.0 && x + swatch + width > available_width {
            x = 0.0;
            row += 1;
        }
        entries.push((x, row));
        x += swatch + width + 20.0;
    }
    entries
}

/// The height the legend of a multi-series bar chart takes above the plot.
pub(super) fn legend_space(
    dataset: &Dataset,
    available_width: f64,
    metrics: &impl TextMetrics,
) -> f64 {
    bar_legend_entries(dataset, available_width, metrics)
        .last()
        .map_or(0.0, |(_, row)| count(row + 1) * LEGEND_HEIGHT)
}

/// The height the legend of a categorical line chart with several series takes above the plot.
pub(super) fn line_legend_space(
    dataset: &Dataset,
    available_width: f64,
    metrics: &impl TextMetrics,
) -> f64 {
    series_legend_entries(dataset, available_width, LEGEND_LINE + 6.0, metrics)
        .last()
        .map_or(0.0, |(_, row)| count(row + 1) * LEGEND_HEIGHT)
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
    for (index, (series, (offset, row))) in dataset.series.iter().zip(entries).enumerate() {
        let name = series
            .name
            .as_deref()
            .expect("multi-series charts name every series");
        let (x, y) = (left + offset, top + count(row) * LEGEND_HEIGHT);
        elements.push(Element::Polyline(Polyline {
            points: vec![(x, y + 5.0), (x + LEGEND_LINE, y + 5.0)],
            class: classes[index],
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        let label = fit_text(
            name,
            available_width - swatch,
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/series/{index}/name"),
        );
        elements.push(Element::Text(Text {
            x: x + swatch,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: label,
        }));
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
    for (index, (series, (offset, row))) in dataset.series.iter().zip(entries).enumerate() {
        let name = series
            .name
            .as_deref()
            .expect("multi-series charts name every series");
        let (x, y) = (left + offset, top + count(row) * LEGEND_HEIGHT);
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
        let label = fit_text(
            name,
            available_width - LEGEND_SWATCH,
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/series/{index}/name"),
        );
        elements.push(Element::Text(Text {
            x: x + LEGEND_SWATCH,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: label,
        }));
    }
}
