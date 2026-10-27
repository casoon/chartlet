//! The frame a pane maps its observations onto: the time and value scales, the gutter the
//! value labels need and the share of the plot every pane takes.

use super::TIME_INSET;
use crate::{
    error::ChartWarning,
    layout::{LABEL_SIZE, PlotArea, axis::NumericScale, count},
    metrics::TextMetrics,
    scene::{Element, Hook},
    spec::{ChartSpec, ChartType, Mark, NumberStyle},
    time::{Precision, TimeZone},
};

/// Title, value grid, value ticks and time ticks, shared by every pane.
/// Draws one polyline per layer, plus markers and value labels while the observations stay far
/// enough apart for them to be readable.
/// Everything a pane needs to map an observation onto its plot.
pub(crate) struct TimeFrame {
    pub(in crate::layout) plot: PlotArea,
    pub(super) span: (i64, i64),
    /// With gaps collapsed, the observed timestamps in order, each placed at the same distance
    /// from the next; `None` while the axis keeps the distances in time.
    pub(super) slots: Option<Vec<i64>>,
    pub(crate) zone: TimeZone,
    pub(crate) precision: Precision,
    /// The declared distance between time ticks, if any.
    pub(super) tick_step: Option<f64>,
    /// Whether the observations carry a time of day, so that ticks may fall within a day; taken
    /// from the data, not from a declared precision.
    pub(super) sub_day: bool,
    /// The unit of the pane's value axis, which tooltips write after a value.
    pub(crate) unit: Option<String>,
    pub(super) scale: NumericScale,
    /// How the pane writes its values: its own value axis on a time chart, the shared one in
    /// small multiples.
    pub(crate) style: NumberStyle,
    /// The axis runs from right to left: the largest value, such as the oldest age in millions of
    /// years, at the left.
    pub(super) reversed: bool,
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
    pub(in crate::layout) fn edges(&self) -> (f64, f64) {
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
    pub(in crate::layout) fn x_until(&self, epoch: i64) -> f64 {
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
    pub(super) fn hook(&self, pane: usize) -> Element {
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
pub(super) enum Detail {
    Full,
    Compact,
    /// A sparkline: no markers, only a dot at the last observation of each line.
    Spark,
}

/// The gutter left of a time chart's plot: as wide as its widest value tick label needs.
pub(super) fn time_gutter(
    spec: &ChartSpec,
    zone: TimeZone,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> f64 {
    let gutter = f64::from(super::axis_gutter(spec.width));
    let widest = (0..spec.panes.len())
        .flat_map(|pane| {
            let scale = time_scale(spec, zone, Some(pane));
            let style = spec.pane_style(pane);
            // Measured as if every label carried the unit: one of them does, the top one.
            let unit = spec
                .axis_unit(pane)
                .map(|unit| format!(" {unit}"))
                .unwrap_or_default();
            scale
                .ticks()
                .map(move |value| format!("{}{unit}", scale.tick_label(value, style)))
                .collect::<Vec<_>>()
        })
        .map(|label| super::WithReserve(metrics).width(&label, LABEL_SIZE))
        .fold(0.0, f64::max);
    // The labels end 10 pixels left of the plot and keep 12 from the edge of the chart. Short
    // labels take less than the regular gutter; long ones, such as a value with its unit, more,
    // up to 40 % of the chart.
    let needed = widest + 22.0;
    let most = f64::from(spec.width) * 0.4;
    if needed > most.max(gutter) {
        warnings.push(ChartWarning::new(
            "dense_chart",
            "/width",
            "the value tick labels are wider than the chart leaves them and are cut at its left edge; raise width or use fewer digits",
        ));
    }
    if needed <= gutter {
        needed.max(28.0)
    } else {
        needed.min(most).max(gutter)
    }
}

/// The value scale of one pane of a time chart, or of all small multiples when `pane` is `None`:
/// every observation, every candle's high and low, every band edge, every horizontal reference
/// line, every point marker and every zone edge, so that none of them falls outside the plot. An
/// area is filled down to zero, so a pane with one always shows zero.
pub(super) fn time_scale(spec: &ChartSpec, zone: TimeZone, pane: Option<usize>) -> NumericScale {
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
        // A stream reaches below zero by half its total.
        if spec.panes[entry.pane].stack == Some(crate::spec::Stack::Stream) {
            values.extend(spec.stack_base(entry, zone).into_iter().flatten());
        }
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

/// Space between two stacked panes; it holds the value axis title of the lower one.
const PANE_GAP: f64 = 36.0;
/// A pane lower than this cannot show its value axis in a readable way.
const MIN_PANE_HEIGHT: f64 = 40.0;

/// One frame per pane of a time chart, stacked from top to bottom within `plot`: every pane
/// shares the time span and takes a share of the height by its `heightRatio`, and scales its
/// values on its own.
pub(super) fn pane_frames(
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
            tick_step: spec.time_axis.step,
            sub_day: spec.observed_precision(zone) == Precision::Minute,
            unit: spec.axis_unit(pane_index).map(str::to_owned),
            scale: time_scale(spec, zone, Some(pane_index)),
            style: spec.pane_style(pane_index),
            reversed: spec.time_axis.reverse,
        });
        top += height + PANE_GAP;
    }
    frames
}

/// The first and last timestamp any data layer, vertical reference line, point marker, zone
/// edge, step end or declared end of the time axis uses, a missing value included. Validation guarantees every data layer holds at least two observations that
/// increase, and a zoom window at least two of one layer, so the span is never empty.
fn time_span(spec: &ChartSpec, zone: TimeZone) -> (i64, i64) {
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    let data = spec
        .data_layers()
        .flat_map(|entry| entry.layer.resolved_times(zone));
    let rules = spec
        .layers()
        .flat_map(|layer| [&layer.time, &layer.from, &layer.to, &layer.step_end])
        .chain([&spec.time_axis.min, &spec.time_axis.max])
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
pub(super) fn time_axis(spec: &ChartSpec, zone: TimeZone) -> ((i64, i64), Option<Vec<i64>>) {
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
