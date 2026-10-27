//! The axes of a time chart: the time axis title, the value grid, the time ticks and the
//! value axis of every pane.

use super::{frame::TimeFrame, time_tick_spacing};
use crate::{
    error::ChartWarning,
    layout::{LABEL_SIZE, PlotArea, fit_text},
    metrics::TextMetrics,
    scene::{Element, Line, Text, TextAnchor},
    spec::{ChartSpec, Stack},
    time,
};

/// The title of the time axis, centred below the plot.
pub(super) fn push_time_axis_title(
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

/// The horizontal grid lines and value ticks of one plot.
/// The unit after the top value tick label of a pane, shortened where it does not fit between
/// `left_edge` and the plot next to the widest tick label.
pub(super) fn fitted_unit(
    unit: Option<&str>,
    frame: &TimeFrame,
    left_edge: f64,
    path: &str,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Option<String> {
    let unit = unit?;
    let widest = frame
        .scale
        .ticks()
        .map(|value| metrics.width(&frame.scale.tick_label(value, frame.style), LABEL_SIZE))
        .fold(0.0, f64::max);
    let room = frame.plot.left - 10.0 - left_edge - widest - 4.0;
    let fitted = fit_text(unit, room.max(0.0), LABEL_SIZE, metrics, warnings, path);
    // A unit shortened to nothing but the ellipsis says nothing; it stays in the table.
    (fitted != "…").then_some(fitted)
}

pub(super) fn push_value_grid(frame: &TimeFrame, unit: Option<&str>, elements: &mut Vec<Element>) {
    let plot = frame.plot;
    let ticks: Vec<(f64, f64)> = frame
        .scale
        .ticks()
        .map(|value| (value, frame.y(value)))
        .collect();
    // In a low pane the ticks can stand closer than a label is tall: then every `step`-th tick is
    // labelled, the gridlines stay.
    let spacing = ticks
        .windows(2)
        .map(|pair| (pair[1].1 - pair[0].1).abs())
        .fold(f64::INFINITY, f64::min);
    // Bounded by the tick count, which is small.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let step = if spacing.is_finite() && spacing < LABEL_SIZE + 2.0 {
        ((LABEL_SIZE + 2.0) / spacing.max(1.0)).ceil() as usize
    } else {
        1
    };
    // The unit follows the label nearest the top of the plot.
    let top = ticks
        .iter()
        .enumerate()
        .filter(|(index, _)| index % step == 0)
        .min_by(|(_, a), (_, b)| a.1.total_cmp(&b.1))
        .map(|(index, _)| index);
    for (index, &(value, y)) in ticks.iter().enumerate() {
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
        if index % step != 0 {
            continue;
        }
        let label = frame.scale.tick_label(value, frame.style);
        elements.push(Element::Text(Text {
            x: plot.left - 10.0,
            y: y + 4.0,
            class: "chartlet-tick",
            anchor: TextAnchor::End,
            content: match unit {
                Some(unit) if Some(index) == top => format!("{label} {unit}"),
                _ => label,
            },
        }));
    }
}

/// The value axis of a stream: a bar as tall as one step of the scale, centered on the middle of
/// the plot and labelled with the value it stands for, since position on a stream means nothing,
/// only thickness does.
fn push_scale_bar(
    frame: &TimeFrame,
    unit: Option<&str>,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let plot = frame.plot;
    let step = frame.scale.step;
    let (top, bottom) = (frame.y(step / 2.0), frame.y(-step / 2.0));
    let x = plot.left - 10.0;
    for (x1, y1, x2, y2) in [
        (x, top, x, bottom),
        (x - 4.0, top, x + 4.0, top),
        (x - 4.0, bottom, x + 4.0, bottom),
    ] {
        elements.push(Element::Line(Line {
            x1,
            y1,
            x2,
            y2,
            class: "chartlet-zero",
        }));
    }
    let label = frame.scale.tick_label(step, frame.style);
    let content = match unit {
        Some(unit) => format!("{label} {unit}"),
        None => label,
    };
    let room = (x - 8.0 - 4.0).max(0.0);
    elements.push(Element::Text(Text {
        x: x - 8.0,
        y: f64::midpoint(top, bottom) + 4.0,
        class: "chartlet-tick",
        anchor: TextAnchor::End,
        content: fit_text(&content, room, LABEL_SIZE, metrics, warnings, "/panes"),
    }));
}

/// The vertical grid lines and time ticks of one plot, the tick labels only with `labels`. A label
/// that would reach past an edge of the chart moves inward until it fits, since a tick can sit at
/// either end of the span and the gutter left of the plot may be narrow.
pub(super) fn push_time_ticks(
    frame: &TimeFrame,
    max_ticks: usize,
    labels: bool,
    canvas_width: f64,
    metrics: &impl TextMetrics,
    elements: &mut Vec<Element>,
) {
    let plot = frame.plot;
    let sub_day = frame.sub_day;
    let ticks = match &frame.slots {
        Some(slots) => {
            time::collapsed_ticks(slots, frame.zone, max_ticks, sub_day, frame.tick_step)
        }
        None => time::ticks(
            frame.span.0,
            frame.span.1,
            frame.zone,
            max_ticks,
            sub_day,
            frame.tick_step,
        ),
    };
    // A label that would run into the one before it is left out; its gridline stays.
    let mut last_end = f64::NEG_INFINITY;
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
        let x = x.min(canvas_width - 4.0 - half).max(4.0 + half);
        let (start, end) = if frame.reversed {
            (-(x + half), -(x - half))
        } else {
            (x - half, x + half)
        };
        if start < last_end + 6.0 {
            continue;
        }
        last_end = end;
        elements.push(Element::Text(Text {
            x,
            y: plot.top + plot.height + 24.0,
            class: "chartlet-tick",
            anchor: TextAnchor::Middle,
            content: tick.label,
        }));
    }
}

/// The value grid, the vertical time grid and the value axis title of one pane of a time chart;
/// the time tick labels only below the bottom pane, since all panes share one time axis.
pub(super) fn push_pane_axes(
    spec: &ChartSpec,
    pane_index: usize,
    frame: &TimeFrame,
    bottom_pane: bool,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let plot = frame.plot;
    let unit = fitted_unit(
        spec.axis_unit(pane_index),
        frame,
        4.0,
        &format!("/panes/{pane_index}/valueAxis/unit"),
        warnings,
        metrics,
    );
    if spec.panes[pane_index].stack == Some(Stack::Stream) {
        push_scale_bar(frame, unit.as_deref(), elements, warnings, metrics);
    } else {
        push_value_grid(frame, unit.as_deref(), elements);
    }
    // The plot's own width, which the measured gutter leaves it, decides how many ticks fit.
    // Plot widths are far below u32::MAX.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let plot_width = frame.plot.width.max(0.0) as u32;
    let max_ticks = usize::try_from(
        (plot_width / time_tick_spacing(spec.observed_precision(frame.zone))).max(2),
    )
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
