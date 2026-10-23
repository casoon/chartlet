mod axes;
mod end_labels;
mod frame;
mod marks;
mod multiples;

use super::{
    LEGEND_HEIGHT, LEGEND_ROW, PlotArea, WithReserve, axis_gutter, count,
    legend::{add_layer_legend, legend_rows},
    title,
    title::{push_title, title_extra},
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::Scene,
    spec::{ChartSpec, LegendPlacement, MAX_SERIES, Mark},
    time::Precision,
};
use axes::{push_pane_axes, push_time_axis_title};
use end_labels::{end_label_room, push_end_labels};
pub(crate) use frame::TimeFrame;
use frame::{Detail, pane_frames, time_gutter};
use marks::draw_pane;
pub(super) use marks::line_class;
pub(crate) use marks::tooltip_name;
pub(super) use multiples::{layout_multiples, multiples_header, panel_heads, push_panel_head};

/// Target distance between two time-axis ticks, in pixels: dates need room for `2026-02-02`, a
/// year label is less than half as wide, a date with a time of day such as `2026-03-01 12:00`
/// needs more.
const TIME_TICK_SPACING: u32 = 90;
const YEAR_TICK_SPACING: u32 = 64;
const MINUTE_TICK_SPACING: u32 = 120;

/// The tick spacing that fits the labels an axis of this precision writes.
const fn time_tick_spacing(precision: Precision) -> u32 {
    match precision {
        Precision::Year | Precision::Number(_) => YEAR_TICK_SPACING,
        Precision::Month | Precision::Day => TIME_TICK_SPACING,
        Precision::Minute => MINUTE_TICK_SPACING,
    }
}

/// Space between the end of the plot and the names at the ends of its lines.
const END_LABEL_GAP: f64 = 8.0;
/// Outer margin of a small-multiples grid, left and right.
pub(super) const MULTIPLES_MARGIN: f64 = 16.0;
/// Inset that keeps the outermost points and their labels inside the plot.
const TIME_INSET: f64 = 6.0;
/// Above this many observations a layer is drawn as a line only. At the default width the
/// markers and their labels would sit closer together than about eleven pixels, and every marker
/// and tooltip costs bytes in the output; the line and the data table carry the values instead.
const MAX_TIME_MARKERS: usize = 60;
/// The weight classes of a line, by [`Stroke`].
const LINE_WEIGHTS: [&str; 4] = [
    "",
    " chartlet-line-thin",
    " chartlet-line-bold",
    " chartlet-line-medium",
];
/// The pattern classes of a line: solid, dashed because modeled, dashed, dotted. A modeled line
/// keeps its own class, which the stylesheet dashes.
const LINE_DASHES: [&str; 4] = [
    "",
    " chartlet-line-modeled",
    " chartlet-line-dashed",
    " chartlet-line-dotted",
];
/// Every combination of color, pattern and weight a time line can take, built once: a declared
/// color first, then the palette colors.
static LINE_CLASSES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
/// Fill of an uncertainty band, in the palette color of its line.
const BAND_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-band chartlet-band-series-1",
    "chartlet-band chartlet-band-series-2",
    "chartlet-band chartlet-band-series-3",
    "chartlet-band chartlet-band-series-4",
];
/// Markers of a layer that takes a palette color other than the first.
pub(super) const POINT_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-point chartlet-point-series-1",
    "chartlet-point chartlet-point-series-2",
    "chartlet-point chartlet-point-series-3",
    "chartlet-point chartlet-point-series-4",
];

/// Lays out a time chart: one shared time axis and one value axis per pane, with the layers of
/// every pane drawn inside it.
pub(super) fn layout_time(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    if spec.sparkline {
        return layout_sparkline(spec, warnings, metrics);
    }
    let zone = spec.time_zone().unwrap_or_default();
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = time_gutter(spec, zone, warnings, metrics);
    let end_labels = spec.legend == LegendPlacement::End;
    let right = f64::from(super::plot_margin(spec.width))
        + if end_labels {
            end_label_room(spec, width, metrics)
        } else {
            0.0
        };
    // Candles always take a legend entry: it says which body is rising and which is falling.
    let layered = !end_labels
        && (spec.series_names().len() > 1 || spec.layers().any(|layer| layer.mark == Mark::Ohlc));
    // Without a drawn title the legend and the plot move up into its place; a title on two lines
    // pushes them down.
    let head = -title_extra(spec, width - left - right, metrics);
    let legend = if layered {
        count(legend_rows(spec, width - left - right, metrics)) * LEGEND_HEIGHT
    } else {
        0.0
    };
    // With neither title, legend nor a value axis title above the first pane, nothing sits above
    // the plot but the half of its top tick label.
    let bare = !spec.show_title
        && legend == 0.0
        && spec
            .panes
            .first()
            .is_none_or(|pane| pane.value_axis.title.is_none());
    let top = if bare { 14.0 } else { 78.0 - head + legend };
    let bottom = if spec.time_axis.title.is_some() {
        56.0
    } else {
        36.0
    };
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: super::plot_height(height - top - bottom, warnings),
        vertical_bars: true,
    };

    let frames = pane_frames(spec, zone, plot, warnings);
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        plot.left,
        plot.width,
        metrics,
        warnings,
    );
    for (pane_index, frame) in frames.iter().enumerate() {
        let bottom_pane = pane_index + 1 == frames.len();
        push_pane_axes(
            spec,
            pane_index,
            frame,
            bottom_pane,
            &mut elements,
            warnings,
            metrics,
        );
    }
    if layered {
        add_layer_legend(
            spec,
            LEGEND_ROW - head,
            plot.left,
            plot.width,
            &mut elements,
            warnings,
            metrics,
        );
    }

    for (pane_index, frame) in frames.iter().enumerate() {
        elements.push(frame.hook(pane_index));
        draw_pane(
            spec,
            pane_index,
            frame,
            Detail::Full,
            &mut elements,
            warnings,
            metrics,
        );
    }
    if end_labels {
        push_end_labels(spec, &frames, &mut elements, warnings, metrics);
    }

    push_time_axis_title(spec, plot, height, &mut elements, warnings, metrics);

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// Lays out small multiples: a grid of small time plots that share the value scale and the time
/// span, so that the panels can be compared by position alone.
/// A sparkline: the lines and areas of the one pane across the whole canvas, inset by the dot at
/// the end of each line.
fn layout_sparkline(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let zone = spec.time_zone().unwrap_or_default();
    let inset = 4.0;
    let plot = PlotArea {
        left: inset - TIME_INSET,
        top: inset,
        width: f64::from(spec.width) - 2.0 * (inset - TIME_INSET),
        height: f64::from(spec.height) - 2.0 * inset,
        vertical_bars: true,
    };
    let frames = pane_frames(spec, zone, plot, warnings);
    let mut elements = Vec::new();
    for (pane_index, frame) in frames.iter().enumerate() {
        elements.push(frame.hook(pane_index));
        draw_pane(
            spec,
            pane_index,
            frame,
            Detail::Spark,
            &mut elements,
            warnings,
            metrics,
        );
    }
    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}
