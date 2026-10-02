use std::fmt::Write as _;

use super::{
    AREA_CLASSES, AXIS_GUTTER, LABEL_LINE, LABEL_SIZE, LEGEND_HEIGHT, LEGEND_ROW, PANEL_GUTTER,
    PANEL_MARGIN, PLOT_MARGIN, PlotArea,
    annotation::{push_marker, push_marker_label, push_rule, push_zone},
    axis::{NumericScale, format_value},
    count, fit_text,
    labels::LabelSpace,
    legend::{add_layer_legend, legend_rows},
    panel_plot_pixels, plot_pixels,
    title::{push_title, title_extra},
    tooltip, tooltips_fit,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Circle, Element, Hook, Line, Polyline, Scene, Text, TextAnchor},
    spec::{
        ChartSpec, ChartType, Dash, LayerRef, LegendPlacement, MAX_SERIES, Mark, NumberStyle,
        Stroke, Tooltips,
    },
    time,
    time::{Precision, TimeZone},
};

/// Target distance between two time-axis ticks, in pixels: dates need room for `2026-02-02`, a
/// year label is less than half as wide.
const TIME_TICK_SPACING: u32 = 90;
const YEAR_TICK_SPACING: u32 = 64;

/// The tick spacing that fits the labels an axis of this precision writes.
const fn time_tick_spacing(precision: Precision) -> u32 {
    match precision {
        Precision::Year | Precision::Number(_) => YEAR_TICK_SPACING,
        Precision::Month | Precision::Day | Precision::Minute => TIME_TICK_SPACING,
    }
}

/// Space between the end of the plot and the names at the ends of its lines.
const END_LABEL_GAP: f64 = 8.0;
/// Outer margin of a small-multiples grid, left and right.
const MULTIPLES_MARGIN: f64 = 16.0;
/// Inset that keeps the outermost points and their labels inside the plot.
const TIME_INSET: f64 = 6.0;
/// Above this many observations a layer is drawn as a line only. At the default width the
/// markers and their labels would sit closer together than about eleven pixels, and every marker
/// and tooltip costs bytes in the output; the line and the data table carry the values instead.
const MAX_TIME_MARKERS: usize = 60;
/// The weight classes of a line, by [`Stroke`].
const LINE_WEIGHTS: [&str; 3] = ["", " chartlet-line-thin", " chartlet-line-bold"];
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
    let left = f64::from(AXIS_GUTTER);
    let end_labels = spec.legend == LegendPlacement::End;
    let right = f64::from(PLOT_MARGIN)
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
    let top = 78.0 - head + legend;
    let bottom = if spec.time_axis.title.is_some() {
        56.0
    } else {
        36.0
    };
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: height - top - bottom,
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

/// The title, the legend and the shared value axis title above a small-multiples grid; returns
/// where the grid starts.
fn multiples_header(
    spec: &ChartSpec,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> f64 {
    let width = f64::from(spec.width);
    push_title(
        elements,
        spec,
        MULTIPLES_MARGIN,
        width - 2.0 * MULTIPLES_MARGIN,
        metrics,
        warnings,
    );

    let head = title_extra(spec, width - 2.0 * MULTIPLES_MARGIN, metrics);
    let mut cursor = LEGEND_ROW + head;
    if spec.series_names().len() > 1 {
        add_layer_legend(
            spec,
            LEGEND_ROW + head,
            MULTIPLES_MARGIN,
            width - 2.0 * MULTIPLES_MARGIN,
            elements,
            warnings,
            metrics,
        );
        cursor += count(legend_rows(spec, width - 2.0 * MULTIPLES_MARGIN, metrics)) * LEGEND_HEIGHT;
    }
    if let Some(axis_title) = &spec.value_axis.title {
        elements.push(Element::Text(Text {
            x: MULTIPLES_MARGIN,
            y: cursor + 10.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Start,
            content: fit_text(
                axis_title,
                width - 2.0 * MULTIPLES_MARGIN,
                LABEL_SIZE,
                metrics,
                warnings,
                "/valueAxis/title",
            ),
        }));
        cursor += 20.0;
    }
    cursor + 8.0
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

/// The title of the time axis, centred below the plot.
fn push_time_axis_title(
    spec: &ChartSpec,
    plot: PlotArea,
    height: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    if let Some(title) = &spec.time_axis.title {
        let title = fit_text(
            title,
            plot.width,
            LABEL_SIZE,
            metrics,
            warnings,
            "/timeAxis/title",
        );
        elements.push(Element::Text(Text {
            x: plot.left + plot.width / 2.0,
            y: height - 14.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: title,
        }));
    }
}

/// The widest name a line takes at its end, with the gap before it, at most a third of the chart.
fn end_label_room(spec: &ChartSpec, width: f64, metrics: &impl TextMetrics) -> f64 {
    let widest = spec
        .data_layers()
        .map(|entry| metrics.width(&end_label(spec, entry), LABEL_SIZE))
        .fold(0.0, f64::max);
    (widest + END_LABEL_GAP).min(width / 3.0)
}

/// The name at the end of a line: its legend entry, so a modeled series says so.
fn end_label(spec: &ChartSpec, entry: LayerRef) -> String {
    let words = spec.locale.words();
    let name = entry.layer.name.as_deref().unwrap_or(words.value);
    if entry.layer.modeled {
        format!("{name} ({})", words.modeled)
    } else {
        name.to_owned()
    }
}

/// Names every line and area at its last observation, right of the plot. Names that would overlap
/// move apart, at least a line of text from each other, and stay inside their pane.
fn push_end_labels(
    spec: &ChartSpec,
    frames: &[TimeFrame],
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let width = f64::from(spec.width);
    for (pane_index, frame) in frames.iter().enumerate() {
        let mut labels: Vec<(f64, String, String)> = spec
            .data_layers()
            .filter(|entry| entry.pane == pane_index)
            .filter_map(|entry| {
                let (_, value) = *spec.drawn_points(entry, frame.zone).last()?;
                let path = format!("/panes/{}/layers/{}/name", entry.pane, entry.local);
                Some((frame.y(value) + 4.0, end_label(spec, entry), path))
            })
            .collect();
        labels.sort_by(|a, b| a.0.total_cmp(&b.0));
        let plot = frame.plot;
        let mut floor = plot.top + LABEL_SIZE;
        for label in &mut labels {
            label.0 = label.0.max(floor);
            floor = label.0 + LABEL_LINE;
        }
        // Pushed past the bottom of the pane, the lowest names move back up together.
        let overflow = labels
            .last()
            .map_or(0.0, |last| (last.0 - (plot.top + plot.height)).max(0.0));
        let x = plot.left + plot.width + END_LABEL_GAP;
        for (y, name, path) in labels {
            elements.push(Element::Text(Text {
                x,
                y: y - overflow,
                class: "chartlet-legend",
                anchor: TextAnchor::Start,
                content: fit_text(&name, width - x - 4.0, LABEL_SIZE, metrics, warnings, &path),
            }));
        }
    }
}

/// The title above one panel of small multiples, in the panel's cell starting at `cell_top`.
fn panel_title(
    pane: &crate::spec::PaneSpec,
    pane_index: usize,
    plot: PlotArea,
    cell_top: f64,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Element {
    Element::Text(Text {
        x: plot.left,
        y: cell_top + 18.0,
        class: "chartlet-panel-title",
        anchor: TextAnchor::Start,
        content: fit_text(
            pane.title.as_deref().unwrap_or_default(),
            plot.width,
            13.0,
            metrics,
            warnings,
            &format!("/panes/{pane_index}/title"),
        ),
    })
}

pub(super) fn layout_multiples(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let zone = spec.time_zone().unwrap_or_default();
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let columns = spec.multiples_columns();
    let panels = spec.panes.len();
    let rows = panels.div_ceil(usize::try_from(columns).expect("columns are limited"));

    let mut elements = Vec::new();
    let grid_top = multiples_header(spec, &mut elements, warnings, metrics);
    let grid_bottom = height
        - if spec.time_axis.title.is_some() {
            36.0
        } else {
            12.0
        };
    let cell_width = (width - 2.0 * MULTIPLES_MARGIN) / f64::from(columns);
    let cell_height = (grid_bottom - grid_top) / count(rows);

    let (span, slots) = time_axis(spec, zone);
    let precision = spec.time_precision(zone);
    let shared = time_scale(spec, zone, None);
    let max_ticks = usize::try_from(
        (panel_plot_pixels(spec.width, columns) / time_tick_spacing(precision)).max(2),
    )
    .expect("a usize is at least 32 bits wide");

    for (pane_index, pane) in spec.panes.iter().enumerate() {
        let column = pane_index % usize::try_from(columns).expect("columns are limited");
        let row = pane_index / usize::try_from(columns).expect("columns are limited");
        let cell_left = MULTIPLES_MARGIN + cell_width * count(column);
        let cell_top = grid_top + cell_height * count(row);
        let plot = PlotArea {
            left: cell_left + f64::from(PANEL_GUTTER),
            top: cell_top + 30.0,
            width: cell_width - f64::from(PANEL_GUTTER + PANEL_MARGIN),
            height: cell_height - 30.0 - 28.0,
            vertical_bars: true,
        };
        let frame = TimeFrame {
            plot,
            span,
            slots: slots.clone(),
            zone,
            precision,
            scale: if spec.independent_axes {
                time_scale(spec, zone, Some(pane_index))
            } else {
                shared
            },
            style: spec.number_style(),
            reversed: spec.time_axis.reverse,
        };

        elements.push(panel_title(
            pane, pane_index, plot, cell_top, warnings, metrics,
        ));
        push_value_grid(&frame, &mut elements);
        push_time_ticks(&frame, max_ticks, true, width, metrics, &mut elements);
        elements.push(frame.hook(pane_index));
        draw_pane(
            spec,
            pane_index,
            &frame,
            Detail::Compact,
            &mut elements,
            warnings,
            metrics,
        );
    }

    if cell_height - 58.0 < 60.0 {
        warnings.push(ChartWarning::new(
            "dense_chart",
            "/height",
            "the panels are less than 60 pixels tall; raise height or use more columns",
        ));
    }

    if let Some(axis_title) = &spec.time_axis.title {
        elements.push(Element::Text(Text {
            x: width / 2.0,
            y: height - 14.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: fit_text(
                axis_title,
                width - 2.0 * MULTIPLES_MARGIN,
                LABEL_SIZE,
                metrics,
                warnings,
                "/timeAxis/title",
            ),
        }));
    }

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// Title, value grid, value ticks and time ticks, shared by every pane.
/// Draws one polyline per layer, plus markers and value labels while the observations stay far
/// enough apart for them to be readable.
/// Everything a pane needs to map an observation onto its plot.
pub(crate) struct TimeFrame {
    pub(super) plot: PlotArea,
    span: (i64, i64),
    /// With gaps collapsed, the observed timestamps in order, each placed at the same distance
    /// from the next; `None` while the axis keeps the distances in time.
    slots: Option<Vec<i64>>,
    pub(crate) zone: TimeZone,
    pub(crate) precision: Precision,
    scale: NumericScale,
    /// How the pane writes its values: its own value axis on a time chart, the shared one in
    /// small multiples.
    pub(crate) style: NumberStyle,
    /// The axis runs from right to left: the largest value, such as the oldest age in millions of
    /// years, at the left.
    reversed: bool,
}

impl TimeFrame {
    /// The position of a timestamp. With gaps collapsed, a time between two observations takes
    /// the slot of the next one.
    pub(crate) fn x(&self, epoch: i64) -> f64 {
        self.oriented(match &self.slots {
            Some(slots) => slot_x(
                slots.partition_point(|slot| *slot < epoch),
                slots.len(),
                self.plot,
            ),
            None => time_x(epoch, self.span, self.plot),
        })
    }

    /// A position mirrored across the plot when the axis runs from right to left.
    fn oriented(&self, x: f64) -> f64 {
        if self.reversed {
            2.0 * self.plot.left + self.plot.width - x
        } else {
            x
        }
    }

    /// The edge of the plot where the axis starts, and the one where it ends.
    pub(super) fn edges(&self) -> (f64, f64) {
        let (left, right) = (self.plot.left, self.plot.left + self.plot.width);
        if self.reversed {
            (right, left)
        } else {
            (left, right)
        }
    }

    /// The position of the end of a span of time. With gaps collapsed, a time between two
    /// observations takes the slot of the previous one, so a zone covers only the observations
    /// between its edges.
    pub(super) fn x_until(&self, epoch: i64) -> f64 {
        self.oriented(match &self.slots {
            Some(slots) => slot_x(
                slots
                    .partition_point(|slot| *slot <= epoch)
                    .saturating_sub(1),
                slots.len(),
                self.plot,
            ),
            None => time_x(epoch, self.span, self.plot),
        })
    }

    pub(crate) fn y(&self, value: f64) -> f64 {
        self.scale
            .map(value, self.plot.top + self.plot.height, self.plot.top)
    }

    /// The plot hook of this pane: every slot with collapsed gaps, otherwise both ends of the
    /// span, and both ends of the value scale.
    fn hook(&self, pane: usize) -> Element {
        #[allow(clippy::cast_precision_loss)]
        let x = match &self.slots {
            Some(slots) => slots
                .iter()
                .map(|slot| (*slot as f64, self.x(*slot)))
                .collect(),
            None => vec![
                (self.span.0 as f64, self.x(self.span.0)),
                (self.span.1 as f64, self.x(self.span.1)),
            ],
        };
        let (min, max) = self.scale.ends();
        Element::Hook(Hook::Plot {
            pane,
            x,
            y: [(min, self.y(min)), (max, self.y(max))],
            log: self.scale.is_log(),
        })
    }
}

/// How much a pane draws besides its lines: a time chart labels its values, a small-multiples
/// panel is too small for that and keeps smaller markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Detail {
    Full,
    Compact,
    /// A sparkline: no markers, only a dot at the last observation of each line.
    Spark,
}

/// The value scale of one pane of a time chart, or of all small multiples when `pane` is `None`:
/// every observation, every candle's high and low, every band edge, every horizontal reference
/// line, every point marker and every zone edge, so that none of them falls outside the plot. An
/// area is filled down to zero, so a pane with one always shows zero.
fn time_scale(spec: &ChartSpec, zone: TimeZone, pane: Option<usize>) -> NumericScale {
    let inside = |pane_index: usize| pane.is_none_or(|pane| pane == pane_index);
    let layers = || {
        spec.indexed_layers()
            .filter(|entry| inside(entry.pane))
            .map(|entry| entry.layer)
    };
    let mut values = Vec::new();
    for entry in spec.data_layers().filter(|entry| inside(entry.pane)) {
        values.extend(
            spec.drawn_points(entry, zone)
                .into_iter()
                .map(|(_, value)| value),
        );
        for (_, lower, upper) in entry.layer.resolved_band(zone) {
            values.push(lower);
            values.push(upper);
        }
        for (_, [_, high, low, _]) in entry.layer.resolved_candles(zone) {
            values.push(high);
            values.push(low);
        }
    }
    values.extend(
        layers()
            .filter(|layer| layer.mark == Mark::Annotation)
            .filter_map(|layer| layer.value),
    );
    values.extend(
        layers()
            .filter(|layer| layer.mark == Mark::Band)
            .flat_map(|layer| [layer.bottom, layer.top])
            .flatten(),
    );
    let area = layers().any(|layer| layer.mark == Mark::Area);
    // Small multiples share the top-level value axis, even when every panel gets its own scale.
    let axis = match pane {
        Some(pane) if spec.chart_type != ChartType::Multiples => &spec.panes[pane].value_axis,
        _ => &spec.value_axis,
    };
    NumericScale::for_axis(values.into_iter(), area, axis)
}

/// The name a tooltip gives a layer: in small multiples the panel title comes first.
pub(crate) fn tooltip_name(spec: &ChartSpec, entry: LayerRef) -> Option<String> {
    let title = if spec.chart_type == ChartType::Multiples {
        spec.panes[entry.pane].title.clone()
    } else {
        None
    };
    match (title, &entry.layer.name) {
        (Some(title), Some(name)) => Some(format!("{title} · {name}")),
        (Some(title), None) => Some(title),
        (None, name) => name.clone(),
    }
}

/// Draws the layers of one pane: zones first, filled areas and uncertainty bands over them,
/// reference lines over those, then the lines with their markers, the point markers, and the
/// labels of every annotation last so nothing covers them.
fn draw_pane(
    spec: &ChartSpec,
    pane: usize,
    frame: &TimeFrame,
    detail: Detail,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let entries: Vec<LayerRef> = spec
        .indexed_layers()
        .filter(|entry| entry.pane == pane)
        .collect();
    let mut space = LabelSpace::new(frame, &entries);
    let mut rule_labels = Vec::new();

    for entry in entries
        .iter()
        .filter(|entry| entry.layer.mark == Mark::Band)
    {
        elements.push(Element::Hook(Hook::Layer(entry.global)));
        push_zone(
            spec,
            *entry,
            frame,
            elements,
            &mut rule_labels,
            &mut space,
            warnings,
            metrics,
        );
        elements.push(Element::Hook(Hook::End));
    }
    for entry in entries
        .iter()
        .filter(|entry| entry.layer.mark == Mark::Area)
    {
        elements.push(Element::Hook(Hook::Layer(entry.global)));
        push_area(spec, *entry, frame, elements);
        elements.push(Element::Hook(Hook::End));
    }
    for entry in entries
        .iter()
        .filter(|entry| entry.layer.is_data() && entry.layer.has_band())
    {
        elements.push(Element::Hook(Hook::Layer(entry.global)));
        push_band(spec, *entry, frame, elements);
        elements.push(Element::Hook(Hook::End));
    }

    for entry in entries.iter().filter(|entry| entry.layer.is_rule()) {
        elements.push(Element::Hook(Hook::Layer(entry.global)));
        push_rule(
            *entry,
            frame,
            elements,
            &mut rule_labels,
            &mut space,
            warnings,
            metrics,
        );
        elements.push(Element::Hook(Hook::End));
    }

    // Candles go below the lines, so that a moving average stays readable across them.
    for entry in entries
        .iter()
        .filter(|entry| entry.layer.mark == Mark::Ohlc)
    {
        elements.push(Element::Hook(Hook::Layer(entry.global)));
        crate::ohlc::push_candles(spec, *entry, frame, elements);
        elements.push(Element::Hook(Hook::End));
    }
    for entry in entries
        .iter()
        .filter(|entry| entry.layer.is_data() && entry.layer.mark != Mark::Ohlc)
    {
        elements.push(Element::Hook(Hook::Layer(entry.global)));
        push_line(spec, *entry, frame, detail, elements);
        elements.push(Element::Hook(Hook::End));
    }

    let markers: Vec<LayerRef> = entries
        .iter()
        .filter(|entry| entry.layer.is_marker())
        .copied()
        .collect();
    let radius = if detail == Detail::Full { 6.0 } else { 4.5 };
    for entry in &markers {
        elements.push(Element::Hook(Hook::Layer(entry.global)));
        push_marker(*entry, frame, radius, elements, &mut space);
        elements.push(Element::Hook(Hook::End));
    }
    for entry in &markers {
        push_marker_label(
            *entry,
            frame,
            radius,
            &mut rule_labels,
            &mut space,
            warnings,
            metrics,
        );
    }
    elements.extend(rule_labels);
}

/// The region between an area layer's line and zero, one closed outline per run of values: along
/// the line and back along the zero line. A stacked area goes back along the top of the area
/// below it instead, which has the same times and no gaps. It takes the layer's color at the
/// band's low opacity; the line itself is drawn on top with the other lines.
fn push_area(spec: &ChartSpec, entry: LayerRef, frame: &TimeFrame, elements: &mut Vec<Element>) {
    let explicit = entry.layer.resolved_color().is_some();
    let palette = spec.palette_index(entry.layer);
    let base = spec.stack_base(entry, frame.zone);
    let segments = if base.is_some() {
        vec![spec.drawn_points(entry, frame.zone)]
    } else {
        entry.layer.resolved_segments(frame.zone)
    };
    for segment in segments {
        if segment.len() < 2 {
            continue;
        }
        let (first, last) = (segment[0].0, segment[segment.len() - 1].0);
        let mut outline = entry.layer.curve.points(
            segment
                .iter()
                .map(|(epoch, value)| (frame.x(*epoch), frame.y(*value)))
                .collect(),
        );
        if let Some(base) = &base {
            let below = entry.layer.curve.points(
                segment
                    .iter()
                    .zip(base)
                    .map(|((epoch, _), below)| (frame.x(*epoch), frame.y(*below)))
                    .collect(),
            );
            outline.extend(below.into_iter().rev());
        } else {
            outline.push((frame.x(last), frame.y(0.0)));
            outline.push((frame.x(first), frame.y(0.0)));
        }
        elements.push(Element::Polyline(Polyline {
            points: outline,
            class: if explicit {
                "chartlet-area"
            } else {
                AREA_CLASSES[palette]
            },
            topic: None,
            series_index: None,
            style_index: explicit.then_some(entry.global),
            tooltip: None,
        }));
    }
}

/// The band of one layer as closed outlines, one per run of values between missing ones: along
/// the upper edge and back along the lower one. A modeled band gets a hatched copy on top, so
/// that it reads as modeled without relying on color.
fn push_band(spec: &ChartSpec, entry: LayerRef, frame: &TimeFrame, elements: &mut Vec<Element>) {
    let mut runs = vec![Vec::new()];
    for point in &entry.layer.points {
        let Ok(epoch) = point.time.resolve(frame.zone) else {
            continue;
        };
        match (point.lower, point.upper) {
            (Some(lower), Some(upper)) if point.value.is_some() => runs
                .last_mut()
                .expect("there is always a current run")
                .push((epoch, lower, upper)),
            _ => runs.push(Vec::new()),
        }
    }
    for band in runs.into_iter().filter(|run| !run.is_empty()) {
        push_band_run(spec, entry, frame, &band, elements);
    }
}

/// One closed band outline, see [`push_band`].
fn push_band_run(
    spec: &ChartSpec,
    entry: LayerRef,
    frame: &TimeFrame,
    band: &[(i64, f64, f64)],
    elements: &mut Vec<Element>,
) {
    let mut outline: Vec<(f64, f64)> = band
        .iter()
        .map(|(epoch, _, upper)| (frame.x(*epoch), frame.y(*upper)))
        .collect();
    outline.extend(
        band.iter()
            .rev()
            .map(|(epoch, lower, _)| (frame.x(*epoch), frame.y(*lower))),
    );
    let explicit = entry.layer.resolved_color().is_some();
    let palette = spec.palette_index(entry.layer);
    if entry.layer.modeled {
        elements.push(Element::Polyline(Polyline {
            points: outline.clone(),
            class: if explicit {
                "chartlet-band"
            } else {
                BAND_CLASSES[palette]
            },
            topic: None,
            series_index: None,
            style_index: explicit.then_some(entry.global),
            tooltip: None,
        }));
        elements.push(Element::Polyline(Polyline {
            points: outline,
            class: "chartlet-hatch",
            topic: None,
            series_index: None,
            style_index: Some(entry.global),
            tooltip: None,
        }));
    } else {
        elements.push(Element::Polyline(Polyline {
            points: outline,
            class: if explicit {
                "chartlet-band"
            } else {
                BAND_CLASSES[palette]
            },
            topic: None,
            series_index: None,
            style_index: explicit.then_some(entry.global),
            tooltip: None,
        }));
    }
}

/// The tooltip of one observation: its time, value and layer, and its band if it has one.
fn observation_tooltip(
    spec: &ChartSpec,
    frame: &TimeFrame,
    (epoch, value): (i64, f64),
    band: Option<(f64, f64)>,
    name: Option<&str>,
    precision: Precision,
) -> String {
    let mut text = tooltip(
        &precision.format(epoch, frame.zone),
        value,
        frame.style,
        name,
    );
    if let Some((lower, upper)) = band {
        write!(
            text,
            " ({} {} {} {})",
            spec.locale.words().range,
            format_value(lower, frame.style),
            spec.locale.words().to,
            format_value(upper, frame.style)
        )
        .expect("writing to String cannot fail");
    }
    text
}

/// One line, broken at every missing value, with its markers and, in a full-size chart, its
/// value labels. A point layer draws the markers alone, however many there are.
fn push_line(
    spec: &ChartSpec,
    entry: LayerRef,
    frame: &TimeFrame,
    detail: Detail,
    elements: &mut Vec<Element>,
) {
    let layer = entry.layer;
    let points = layer.resolved_points(frame.zone);
    let explicit = layer.resolved_color().is_some();
    let palette = spec.palette_index(layer);
    let is_point = layer.mark == Mark::Point;
    // A point layer draws no line at all.
    if !is_point {
        push_line_segments(spec, entry, frame, elements);
    }

    // Markers and their labels are only drawn while the observations stay far enough apart
    // for them to be readable. Beyond that, an invisible target keeps the tooltip of each
    // observation, as long as the targets stay far enough apart to point at and the chart does
    // not limit tooltips to markers.
    let spark = detail == Detail::Spark;
    let dense = points.len() > MAX_TIME_MARKERS;
    let markers = !spark && (!dense || is_point);
    let fits = || {
        let xs: Vec<f64> = points.iter().map(|(epoch, _)| frame.x(*epoch)).collect();
        spec.tooltips == Tooltips::Observations && tooltips_fit(&xs)
    };
    let hits = !markers && fits();
    // Dense points are drawn smaller, and carry tooltips under the same rule as targets.
    let marker_tooltips = !dense || fits();
    if !markers && !hits && !spark {
        return;
    }
    let last = points.len().saturating_sub(1);
    let band = layer.resolved_band(frame.zone);
    let name = tooltip_name(spec, entry);
    let precision = spec.layer_precision(layer, frame.zone);
    // A stacked area's markers sit on top of the stack, but tell the layer's own value.
    let drawn = spec.drawn_points(entry, frame.zone);
    for (index, (epoch, value)) in points.iter().enumerate() {
        let x = frame.x(*epoch);
        let y = frame.y(drawn[index].1);
        let text = observation_tooltip(
            spec,
            frame,
            (*epoch, *value),
            band.get(index).map(|(_, lower, upper)| (*lower, *upper)),
            name.as_deref(),
            precision,
        );
        let point = |radius: f64, tooltip: Option<String>| {
            Element::Circle(Circle {
                cx: x,
                cy: y,
                radius,
                class: if explicit {
                    "chartlet-point"
                } else {
                    POINT_CLASSES[palette]
                },
                topic: None,
                series_index: None,
                style_index: explicit.then_some(entry.global),
                tooltip,
            })
        };
        if !markers {
            // A sparkline marks where its line ends.
            if spark && index == last {
                elements.push(point(2.5, (!hits).then(|| text.clone())));
            }
            if hits {
                elements.push(Element::Circle(Circle {
                    cx: x,
                    cy: y,
                    radius: 4.0,
                    class: "chartlet-hit",
                    topic: None,
                    series_index: None,
                    style_index: None,
                    tooltip: Some(text),
                }));
            }
            continue;
        }
        elements.push(point(
            if detail == Detail::Full && !dense {
                4.0
            } else {
                2.5
            },
            marker_tooltips.then_some(text),
        ));
        if detail == Detail::Full && spec.show_values && !dense {
            elements.push(Element::Text(Text {
                x,
                y: y - 10.0,
                class: "chartlet-value",
                anchor: TextAnchor::Middle,
                content: format_value(*value, frame.style),
            }));
        }
    }
}

/// The line of a layer, broken at every missing value; a lone value between two gaps keeps only
/// its marker.
fn push_line_segments(
    spec: &ChartSpec,
    entry: LayerRef,
    frame: &TimeFrame,
    elements: &mut Vec<Element>,
) {
    let layer = entry.layer;
    let class = line_class(spec, entry);
    let explicit = layer.resolved_color().is_some();
    // A stacked area has no gaps; its line runs along the top of the stack.
    let segments = if spec.stack_base(entry, frame.zone).is_some() {
        vec![spec.drawn_points(entry, frame.zone)]
    } else {
        layer.resolved_segments(frame.zone)
    };
    for segment in segments {
        if segment.len() < 2 {
            continue;
        }
        elements.push(Element::Polyline(Polyline {
            points: layer.curve.points(
                segment
                    .iter()
                    .map(|(epoch, value)| (frame.x(*epoch), frame.y(*value)))
                    .collect(),
            ),
            class,
            topic: None,
            series_index: None,
            style_index: explicit.then_some(entry.global),
            tooltip: None,
        }));
    }
}

/// The classes of a line: palette or declared color, its pattern and its weight. The legend
/// draws its sample with the same classes, so it looks exactly like the line.
pub(super) fn line_class(spec: &ChartSpec, entry: LayerRef) -> &'static str {
    let layer = entry.layer;
    let color = if layer.resolved_color().is_some() {
        0
    } else {
        spec.palette_index(layer) + 1
    };
    let dash = match (layer.effective_dash(), layer.modeled) {
        (Dash::Solid, _) => 0,
        (Dash::Dashed, true) => 1,
        (Dash::Dashed, false) => 2,
        (Dash::Dotted, _) => 3,
    };
    let weight = match layer.stroke {
        Stroke::Regular => 0,
        Stroke::Thin => 1,
        Stroke::Bold => 2,
    };
    let classes = LINE_CLASSES.get_or_init(|| {
        let mut classes = Vec::new();
        for color in 0..=MAX_SERIES {
            let color = if color == 0 {
                String::new()
            } else {
                format!(" chartlet-line-series-{color}")
            };
            for dash in LINE_DASHES {
                for weight in LINE_WEIGHTS {
                    classes.push(format!("chartlet-line{weight}{dash}{color}"));
                }
            }
        }
        classes
    });
    &classes[(color * LINE_DASHES.len() + dash) * LINE_WEIGHTS.len() + weight]
}

/// The horizontal grid lines and value ticks of one plot.
fn push_value_grid(frame: &TimeFrame, elements: &mut Vec<Element>) {
    let plot = frame.plot;
    for value in frame.scale.ticks() {
        let y = frame.y(value);
        elements.push(Element::Line(Line {
            x1: plot.left,
            y1: y,
            x2: plot.left + plot.width,
            y2: y,
            class: if frame.scale.is_zero(value) {
                "chartlet-zero"
            } else {
                "chartlet-grid"
            },
        }));
        elements.push(Element::Text(Text {
            x: plot.left - 10.0,
            y: y + 4.0,
            class: "chartlet-tick",
            anchor: TextAnchor::End,
            content: frame.scale.tick_label(value, frame.style),
        }));
    }
}

/// The vertical grid lines and time ticks of one plot, the tick labels only with `labels`. A label
/// that would reach past the right edge of the chart moves left until it fits, since a tick can
/// sit at the very end of the span.
fn push_time_ticks(
    frame: &TimeFrame,
    max_ticks: usize,
    labels: bool,
    canvas_width: f64,
    metrics: &impl TextMetrics,
    elements: &mut Vec<Element>,
) {
    let plot = frame.plot;
    let sub_day = frame.precision == Precision::Minute;
    let ticks = match &frame.slots {
        Some(slots) => time::collapsed_ticks(slots, frame.zone, max_ticks, sub_day),
        None => time::ticks(frame.span.0, frame.span.1, frame.zone, max_ticks, sub_day),
    };
    for tick in ticks {
        let x = frame.x(tick.epoch);
        elements.push(Element::Line(Line {
            x1: x,
            y1: plot.top,
            x2: x,
            y2: plot.top + plot.height,
            class: "chartlet-grid",
        }));
        if !labels {
            continue;
        }
        let half = metrics.width(&tick.label, LABEL_SIZE) / 2.0;
        elements.push(Element::Text(Text {
            x: x.min(canvas_width - 4.0 - half),
            y: plot.top + plot.height + 24.0,
            class: "chartlet-tick",
            anchor: TextAnchor::Middle,
            content: tick.label,
        }));
    }
}

/// Space between two stacked panes; it holds the value axis title of the lower one.
const PANE_GAP: f64 = 36.0;
/// A pane lower than this cannot show its value axis in a readable way.
const MIN_PANE_HEIGHT: f64 = 40.0;

/// One frame per pane of a time chart, stacked from top to bottom within `plot`: every pane
/// shares the time span and takes a share of the height by its `heightRatio`, and scales its
/// values on its own.
fn pane_frames(
    spec: &ChartSpec,
    zone: TimeZone,
    plot: PlotArea,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<TimeFrame> {
    let (span, slots) = time_axis(spec, zone);
    let precision = spec.time_precision(zone);
    let panes = spec.panes.len();
    let ratios: u32 = spec.panes.iter().map(|pane| pane.height_ratio).sum();
    let available = plot.height - PANE_GAP * count(panes - 1);
    let mut top = plot.top;
    let mut frames = Vec::new();
    for (pane_index, pane) in spec.panes.iter().enumerate() {
        let height = if panes == 1 {
            plot.height
        } else {
            available * f64::from(pane.height_ratio) / f64::from(ratios)
        };
        if panes > 1 && height < MIN_PANE_HEIGHT {
            warnings.push(ChartWarning::new(
                "dense_chart",
                format!("/panes/{pane_index}/heightRatio"),
                "the pane is less than 40 pixels tall; raise height or its heightRatio",
            ));
        }
        frames.push(TimeFrame {
            plot: PlotArea {
                top,
                height,
                ..plot
            },
            span,
            slots: slots.clone(),
            zone,
            precision,
            scale: time_scale(spec, zone, Some(pane_index)),
            style: spec.pane_style(pane_index),
            reversed: spec.time_axis.reverse,
        });
        top += height + PANE_GAP;
    }
    frames
}

/// The value grid, the vertical time grid and the value axis title of one pane of a time chart;
/// the time tick labels only below the bottom pane, since all panes share one time axis.
fn push_pane_axes(
    spec: &ChartSpec,
    pane_index: usize,
    frame: &TimeFrame,
    bottom_pane: bool,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let plot = frame.plot;
    push_value_grid(frame, elements);
    let max_ticks =
        usize::try_from((plot_pixels(spec.width) / time_tick_spacing(frame.precision)).max(2))
            .expect("a usize is at least 32 bits wide");
    push_time_ticks(
        frame,
        max_ticks,
        bottom_pane,
        f64::from(spec.width),
        metrics,
        elements,
    );

    if let Some(title) = spec.panes[pane_index].value_axis.title.as_deref() {
        elements.push(Element::Text(Text {
            x: plot.left,
            y: plot.top - 20.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Start,
            content: fit_text(
                title,
                plot.width,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/panes/{pane_index}/valueAxis/title"),
            ),
        }));
    }
}

/// The first and last timestamp any data layer, vertical reference line, point marker or zone
/// edge uses, a missing value included. Validation guarantees every data layer holds at least two observations that
/// increase, and a zoom window at least two of one layer, so the span is never empty.
fn time_span(spec: &ChartSpec, zone: TimeZone) -> (i64, i64) {
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    let data = spec
        .data_layers()
        .flat_map(|entry| entry.layer.resolved_times(zone));
    let rules = spec
        .layers()
        .flat_map(|layer| [&layer.time, &layer.from, &layer.to])
        .filter_map(Option::as_ref)
        .filter_map(|time| time.resolve(zone).ok());
    for epoch in data.chain(rules) {
        min = min.min(epoch);
        max = max.max(epoch);
    }
    (min, max)
}

/// The span of the time axis and, with gaps collapsed, its slots. A collapsed axis runs from the
/// first observation to the last, since nothing else has a place on it.
fn time_axis(spec: &ChartSpec, zone: TimeZone) -> ((i64, i64), Option<Vec<i64>>) {
    let slots = spec.time_slots(zone);
    let span = match slots.as_deref() {
        Some([first, .., last]) => (*first, *last),
        _ => time_span(spec, zone),
    };
    (span, slots)
}

/// Maps the slot at `index` of `total` evenly spaced slots onto the time axis.
fn slot_x(index: usize, total: usize, plot: PlotArea) -> f64 {
    let last = total.saturating_sub(1);
    let ratio = count(index.min(last)) / count(last.max(1));
    plot.left + TIME_INSET + ratio * (plot.width - 2.0 * TIME_INSET)
}

/// Maps a timestamp onto the time axis.
fn time_x(epoch: i64, span: (i64, i64), plot: PlotArea) -> f64 {
    // The specification allows 1700 to 2200, so a span stays below 2^53 seconds and the ratio
    // keeps full precision even in 64-bit seconds.
    #[allow(clippy::cast_precision_loss)]
    let width = (span.1 - span.0).max(1) as f64;
    #[allow(clippy::cast_precision_loss)]
    let ratio = (epoch - span.0) as f64 / width;
    plot.left + TIME_INSET + ratio * (plot.width - 2.0 * TIME_INSET)
}
