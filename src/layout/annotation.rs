use super::{
    LABEL_SIZE,
    axis::format_value,
    fit_text,
    labels::{LabelSpace, label_box, warn_label_overlap},
    timechart::TimeFrame,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Circle, Element, Polyline, Rect, Text, TextAnchor},
    spec::{ChartSpec, LayerRef, LayerSpec, NumberStyle, Shape},
    time::{Precision, TimeZone},
};

/// Where a zone lies, for its tooltip and the description: its span of time, its range of
/// values, or both.
pub(crate) fn zone_extent(
    spec: &ChartSpec,
    layer: &LayerSpec,
    zone: TimeZone,
    style: NumberStyle,
) -> String {
    let resolve = |time: &Option<crate::time::TimeValue>| {
        time.as_ref().and_then(|time| time.resolve(zone).ok())
    };
    let (from, to) = (resolve(&layer.from), resolve(&layer.to));
    let epochs: Vec<i64> = from.into_iter().chain(to).collect();
    let times = if epochs.is_empty() {
        None
    } else {
        let precision = Precision::of(epochs.into_iter(), zone).at_least(spec.time_precision(zone));
        crate::text::zone_times(
            spec.locale,
            from.map(|epoch| precision.format(epoch, zone)).as_deref(),
            to.map(|epoch| precision.format(epoch, zone)).as_deref(),
        )
    };
    let show = |value: f64| format_value(value, style);
    let values = crate::text::zone_values(
        spec.locale,
        layer.bottom.map(show).as_deref(),
        layer.top.map(show).as_deref(),
    );
    [times, values]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ")
}

/// Where a point marker sits, for its tooltip and the description: its time and its value.
pub(crate) fn marker_position(
    layer: &LayerSpec,
    zone: TimeZone,
    chart: Precision,
    style: NumberStyle,
) -> String {
    let epoch = layer
        .time
        .as_ref()
        .and_then(|time| time.resolve(zone).ok())
        .expect("validated point markers carry a time");
    let value = layer.value.expect("validated point markers carry a value");
    format!(
        "{}, {}",
        Precision::of(std::iter::once(epoch), zone)
            .at_least(chart)
            .format(epoch, zone),
        format_value(value, style)
    )
}

/// A zone: a shaded rectangle behind the data between its edges, each missing edge taken from
/// the plot. Its label sits inside the zone's top left corner when the zone is tall enough, and
/// just outside its upper or lower edge when it is not.
#[allow(clippy::too_many_arguments)]
pub(super) fn push_zone(
    spec: &ChartSpec,
    entry: LayerRef,
    frame: &TimeFrame,
    elements: &mut Vec<Element>,
    labels: &mut Vec<Element>,
    space: &mut LabelSpace,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let layer = entry.layer;
    let label = layer
        .label
        .as_deref()
        .expect("validated zones carry a label");
    let plot = frame.plot;
    let resolve = |time: &Option<crate::time::TimeValue>| {
        time.as_ref().and_then(|time| time.resolve(frame.zone).ok())
    };
    let (start, end) = frame.edges();
    let from = resolve(&layer.from).map_or(start, |epoch| frame.x(epoch));
    let to = resolve(&layer.to).map_or(end, |epoch| frame.x_until(epoch));
    // On an axis that runs from right to left, the zone starts at its right edge.
    let (left, right) = (from.min(to), from.max(to));
    let top = layer.top.map_or(plot.top, |value| frame.y(value));
    let bottom = layer
        .bottom
        .map_or(plot.top + plot.height, |value| frame.y(value));
    elements.push(Element::Rect(Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
        class: "chartlet-zone",
        series_index: None,
        style_index: layer.resolved_color().is_some().then_some(entry.global),
        tooltip: Some(format!(
            "{label}: {}",
            zone_extent(spec, layer, frame.zone, frame.style)
        )),
    }));

    let content = fit_text(
        label,
        plot.width * 0.4,
        LABEL_SIZE,
        metrics,
        warnings,
        &format!("/panes/{}/layers/{}/label", entry.pane, entry.local),
    );
    let width = metrics.width(&content, LABEL_SIZE);
    // A zone at the right edge takes its label on its right end, so the label stays inside.
    let (x, anchor) = if left + 4.0 + width > plot.left + plot.width {
        (right - 4.0, TextAnchor::End)
    } else {
        (left + 4.0, TextAnchor::Start)
    };
    let y = if bottom - top >= 20.0 {
        top + 14.0
    } else if top - 17.0 >= plot.top {
        top - 5.0
    } else {
        bottom + 13.0
    };
    let text = Text {
        x,
        y,
        class: "chartlet-rule-label",
        anchor,
        content,
    };
    space.place(&text, entry, true, metrics, warnings);
    labels.push(Element::Text(text));
}

/// The symbol of a point marker, in its declared shape. Shape and label carry its meaning, so a
/// marker never depends on its color alone.
pub(super) fn push_marker(
    entry: LayerRef,
    frame: &TimeFrame,
    radius: f64,
    elements: &mut Vec<Element>,
    space: &mut LabelSpace,
) {
    let layer = entry.layer;
    let (x, y) = marker_point(entry, frame);
    let label = layer
        .label
        .as_deref()
        .expect("validated point markers carry a label");
    let style_index = layer.resolved_color().is_some().then_some(entry.global);
    let tooltip = Some(format!(
        "{label}: {}",
        marker_position(layer, frame.zone, frame.precision, frame.style)
    ));
    let reach = radius * 1.25;
    space
        .symbols
        .push((x - reach, y - reach, x + reach, y + reach));
    let outline = |corners: &[(f64, f64)]| -> Vec<(f64, f64)> {
        let mut points: Vec<(f64, f64)> = corners
            .iter()
            .map(|(dx, dy)| (x + dx * radius, y + dy * radius))
            .collect();
        points.push(points[0]);
        points
    };
    let points = match layer.marker_shape() {
        Shape::Circle => {
            elements.push(Element::Circle(Circle {
                cx: x,
                cy: y,
                radius,
                class: "chartlet-marker",
                topic: None,
                series_index: None,
                style_index,
                tooltip,
            }));
            return;
        }
        Shape::Square => outline(&[(-0.9, -0.9), (0.9, -0.9), (0.9, 0.9), (-0.9, 0.9)]),
        Shape::Diamond => outline(&[(0.0, -1.25), (1.25, 0.0), (0.0, 1.25), (-1.25, 0.0)]),
        Shape::TriangleUp => outline(&[(0.0, -1.25), (1.15, 0.8), (-1.15, 0.8)]),
        Shape::TriangleDown => outline(&[(0.0, 1.25), (1.15, -0.8), (-1.15, -0.8)]),
    };
    elements.push(Element::Polyline(Polyline {
        points,
        class: "chartlet-marker",
        topic: None,
        series_index: None,
        style_index,
        tooltip,
    }));
}

/// Where a point marker sits on the plot.
fn marker_point(entry: LayerRef, frame: &TimeFrame) -> (f64, f64) {
    let layer = entry.layer;
    let epoch = layer
        .time
        .as_ref()
        .and_then(|time| time.resolve(frame.zone).ok())
        .expect("validated point markers carry a time");
    let value = layer.value.expect("validated point markers carry a value");
    (frame.x(epoch), frame.y(value))
}

/// The label of a point marker. It tries the right of the symbol first, then the left, above and
/// below, and takes the first place that stays inside the plot and clear of other labels, data
/// lines and markers; failing that, the first one that at least stays inside and clear of other
/// labels. Only when no place works is the label kept on the right and reported.
pub(super) fn push_marker_label(
    entry: LayerRef,
    frame: &TimeFrame,
    radius: f64,
    labels: &mut Vec<Element>,
    space: &mut LabelSpace,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let (x, y) = marker_point(entry, frame);
    let content = fit_text(
        entry
            .layer
            .label
            .as_deref()
            .expect("validated point markers carry a label"),
        frame.plot.width * 0.4,
        LABEL_SIZE,
        metrics,
        warnings,
        &format!("/panes/{}/layers/{}/label", entry.pane, entry.local),
    );
    let gap = radius * 1.25 + 4.0;
    let candidates = [
        (x + gap, y + 4.0, TextAnchor::Start),
        (x - gap, y + 4.0, TextAnchor::End),
        (x, y - gap - 3.0, TextAnchor::Middle),
        (x, y + gap + 10.0, TextAnchor::Middle),
    ]
    .map(|(x, y, anchor)| Text {
        x,
        y,
        class: "chartlet-rule-label",
        anchor,
        content: content.clone(),
    });
    let boxes = candidates.each_ref().map(|text| label_box(text, metrics));
    let clear = boxes
        .iter()
        .position(|label| space.collision(*label, true).is_none() && !space.hits_symbol(*label));
    let readable = || {
        boxes
            .iter()
            .position(|label| space.collision(*label, false).is_none())
    };
    let chosen = clear.or_else(readable);
    let index = chosen.unwrap_or(0);
    if chosen.is_none()
        && let Some(collision) = space.collision(boxes[0], false)
    {
        warn_label_overlap(entry, collision, warnings);
    }
    space.labels.push(boxes[index]);
    let text = candidates
        .into_iter()
        .nth(index)
        .expect("the index comes from the candidates");
    labels.push(Element::Text(text));
}

/// A reference line across the plot: horizontal at a value, vertical at a time. Its label is
/// collected separately and drawn after the data; it stays where it is, and a collision with the
/// plot's edges, another label or a data line is reported.
pub(super) fn push_rule(
    entry: LayerRef,
    frame: &TimeFrame,
    elements: &mut Vec<Element>,
    labels: &mut Vec<Element>,
    space: &mut LabelSpace,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let layer = entry.layer;
    let label = layer
        .label
        .as_deref()
        .expect("validated annotations carry a label");
    let path = format!("/panes/{}/layers/{}/label", entry.pane, entry.local);
    let plot = frame.plot;
    let bottom = plot.top + plot.height;
    let (points, text, tooltip) = if let Some(value) = layer.value {
        let y = frame.y(value);
        (
            vec![(plot.left, y), (plot.left + plot.width, y)],
            Text {
                x: plot.left + 4.0,
                y: y - 6.0,
                class: "chartlet-rule-label",
                anchor: TextAnchor::Start,
                content: fit_text(
                    label,
                    plot.width / 2.0,
                    LABEL_SIZE,
                    metrics,
                    warnings,
                    &path,
                ),
            },
            format!("{label}: {}", format_value(value, frame.style)),
        )
    } else {
        let epoch = layer
            .time
            .as_ref()
            .and_then(|time| time.resolve(frame.zone).ok())
            .expect("validated annotations carry a value or a time");
        let x = frame.x(epoch);
        // A line in the right part of the plot takes its label on its left, so the label stays
        // inside the chart.
        let (label_x, anchor) = if x > plot.left + plot.width * 0.7 {
            (x - 4.0, TextAnchor::End)
        } else {
            (x + 4.0, TextAnchor::Start)
        };
        let precision = Precision::of(std::iter::once(epoch), frame.zone).at_least(frame.precision);
        (
            vec![(x, plot.top), (x, bottom)],
            Text {
                x: label_x,
                y: plot.top + 12.0,
                class: "chartlet-rule-label",
                anchor,
                content: fit_text(
                    label,
                    plot.width * 0.4,
                    LABEL_SIZE,
                    metrics,
                    warnings,
                    &path,
                ),
            },
            format!("{label}: {}", precision.format(epoch, frame.zone)),
        )
    };
    elements.push(Element::Polyline(Polyline {
        points,
        class: "chartlet-rule",
        topic: None,
        series_index: None,
        style_index: layer.resolved_color().is_some().then_some(entry.global),
        tooltip: Some(tooltip),
    }));
    space.place(&text, entry, true, metrics, warnings);
    labels.push(Element::Text(text));
}
