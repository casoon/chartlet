//! The marks of a pane: areas, bands, lines and points, with their markers, labels and
//! tooltips.

use std::fmt::Write as _;

use super::{
    BAND_CLASSES, LINE_CLASSES, LINE_DASHES, LINE_WEIGHTS, MAX_TIME_MARKERS, POINT_CLASSES,
    frame::{Detail, TimeFrame},
};
use crate::{
    error::ChartWarning,
    layout::{
        AREA_CLASSES, LABEL_SIZE,
        annotation::{push_marker, push_marker_label, push_rule, push_zone},
        axis::format_value,
        labels::LabelSpace,
        tooltip, tooltips_fit,
    },
    metrics::TextMetrics,
    scene::{Circle, Element, Hook, Polyline, Text, TextAnchor},
    spec::{ChartSpec, ChartType, Dash, LayerRef, LayerSpec, MAX_SERIES, Mark, Stroke, Tooltips},
    time::Precision,
};

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
pub(super) fn draw_pane(
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
        push_line(spec, *entry, frame, detail, elements, warnings, metrics);
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
        extend_last_step(entry.layer, frame, last, &mut outline);
        let end = outline.last().map_or(frame.x(last), |point| point.0);
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
            outline.push((end, frame.y(0.0)));
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
    if let Some(unit) = &frame.unit {
        write!(text, " {unit}").expect("writing to String cannot fail");
    }
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
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
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
    let markers = !spark && (!dense || is_point) && layer.markers != Some(false);
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
    // Where the last value label of the layer ends: the next one stays off it.
    let mut labels_end = f64::NEG_INFINITY;
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
            let content = format_value(*value, frame.style);
            let half = metrics.width(&content, LABEL_SIZE) / 2.0;
            // Two neighbours that would run into each other: the later one is left out.
            if x - half < labels_end + 4.0 {
                crate::layout::warn_if_labels_omitted(true, warnings);
            } else {
                labels_end = x + half;
                elements.push(Element::Text(Text {
                    x,
                    y: y - 10.0,
                    class: "chartlet-value",
                    anchor: TextAnchor::Middle,
                    content,
                }));
            }
        }
    }
}

/// Holds the last value of a step layer until its `stepEnd`, if `last` is the layer's last
/// observation.
fn extend_last_step(layer: &LayerSpec, frame: &TimeFrame, last: i64, points: &mut Vec<(f64, f64)>) {
    let Some(end) = layer
        .step_end
        .as_ref()
        .and_then(|time| time.resolve(frame.zone).ok())
    else {
        return;
    };
    // Only a layer whose last observation has a value holds it on; after a trailing gap the
    // line stays broken.
    let last_point = layer.points.last();
    if last_point.is_some_and(|point| point.value.is_some())
        && last_point.and_then(|point| point.time.resolve(frame.zone).ok()) == Some(last)
        && let Some(&(_, y)) = points.last()
    {
        points.push((frame.x(end), y));
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
        let mut points = layer.curve.points(
            segment
                .iter()
                .map(|(epoch, value)| (frame.x(*epoch), frame.y(*value)))
                .collect(),
        );
        extend_last_step(layer, frame, segment[segment.len() - 1].0, &mut points);
        elements.push(Element::Polyline(Polyline {
            points,
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
pub(in crate::layout) fn line_class(spec: &ChartSpec, entry: LayerRef) -> &'static str {
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
        Stroke::Medium => 3,
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
