//! Radar charts: one axis per measure from the center like the spokes of a wheel, grid rings with
//! their values on the first axis, the axis names outside, and one closed polygon per series with a
//! marker on every value. A series is told apart by its dash pattern as well as its color. The
//! description names the strongest and the weakest axis of every series, and the data table has
//! every value with the range of its axis.

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{LABEL_SIZE, count, fit_text, format_value, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Polyline, Scene, Text, TextAnchor},
    spec::{ChartSpec, MAX_SERIES, RINGS, RadarSpec},
    text,
};

const AREAS: [&str; MAX_SERIES] = [
    "chartlet-radar-area chartlet-radar-a1",
    "chartlet-radar-area chartlet-radar-a2",
    "chartlet-radar-area chartlet-radar-a3",
    "chartlet-radar-area chartlet-radar-a4",
];
const KEYS: [&str; MAX_SERIES] = [
    "chartlet-radar-key chartlet-radar-a1",
    "chartlet-radar-key chartlet-radar-a2",
    "chartlet-radar-key chartlet-radar-a3",
    "chartlet-radar-key chartlet-radar-a4",
];
const DOTS: [&str; MAX_SERIES] = [
    "chartlet-radar-dot chartlet-radar-d1",
    "chartlet-radar-dot chartlet-radar-d2",
    "chartlet-radar-dot chartlet-radar-d3",
    "chartlet-radar-dot chartlet-radar-d4",
];
/// The length of the line in the legend, and the height of a legend row.
const KEY: f64 = 24.0;
const ROW: f64 = 20.0;

fn radar(spec: &ChartSpec) -> &RadarSpec {
    spec.radar
        .as_ref()
        .expect("validated radar charts carry a radar block")
}

/// The point at a distance from the center, on the axis at an index of `axes`; the first axis
/// points up and the others follow clockwise.
fn point(center: (f64, f64), count_of_axes: usize, axis: usize, distance: f64) -> (f64, f64) {
    let angle =
        std::f64::consts::TAU * count(axis) / count(count_of_axes) - std::f64::consts::FRAC_PI_2;
    (
        center.0 + distance * angle.cos(),
        center.1 + distance * angle.sin(),
    )
}

/// The legend: a line sample in the style of the polygon and the name of every series, in rows
/// under the title. Returns where it ends.
fn push_legend(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (left, top): (f64, f64),
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) -> f64 {
    let right = f64::from(spec.width) - left;
    let (mut x, mut y) = (left, top);
    for (index, series) in radar(spec).series.iter().enumerate() {
        let reach = KEY + 8.0 + metrics.width(&series.name, LABEL_SIZE);
        if x > left && x + reach > right {
            x = left;
            y += ROW;
        }
        elements.push(Element::Line(Line {
            x1: x,
            y1: y + 5.0,
            x2: x + KEY,
            y2: y + 5.0,
            class: KEYS[index],
        }));
        elements.push(Element::Text(Text {
            x: x + KEY + 8.0,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                &series.name,
                (right - x - KEY - 8.0).max(0.0),
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/radar/series/{index}/name"),
            ),
        }));
        x += reach + 20.0;
    }
    y + ROW + 6.0
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let radar = radar(spec);
    let axes = radar.axes.len();
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
    // The names at the sides take room from the width; those above and below do not.
    let room = radar
        .axes
        .iter()
        .enumerate()
        .filter(|(axis, _)| point((0.0, 0.0), axes, *axis, 1.0).0.abs() > 0.3)
        .map(|(_, axis)| metrics.width(&axis.label, LABEL_SIZE))
        .fold(0.0, f64::max)
        .min(width * 0.28)
        + 16.0;
    let area = height - top - 12.0;
    let radius = (width / 2.0 - margin - room)
        .min(area / 2.0 - 20.0)
        .floor()
        .max(30.0);
    let center = (width / 2.0, top + area / 2.0);
    // The grid: rings at equal steps, and a spoke for every axis.
    for ring in 1..=RINGS {
        let distance = radius * f64::from(ring) / f64::from(RINGS);
        let mut points: Vec<(f64, f64)> = (0..axes)
            .map(|axis| point(center, axes, axis, distance))
            .collect();
        points.push(points[0]);
        elements.push(Element::Polyline(Polyline {
            points,
            class: "chartlet-radar-ring",
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
    for axis in 0..axes {
        let end = point(center, axes, axis, radius);
        elements.push(Element::Line(Line {
            x1: center.0,
            y1: center.1,
            x2: end.0,
            y2: end.1,
            class: "chartlet-radar-spoke",
        }));
    }
    // The values of the rings, on the first axis.
    for ring in 1..=RINGS {
        let fraction = f64::from(ring) / f64::from(RINGS);
        let value = radar.min(0) + (radar.max(0) - radar.min(0)) * fraction;
        elements.push(Element::Text(Text {
            x: center.0 + 4.0,
            y: center.1 - radius * fraction - 3.0,
            class: "chartlet-radar-tick",
            anchor: TextAnchor::Start,
            content: format_value(value, spec.number_style()),
        }));
    }
    // The names of the axes, outside the outer ring.
    for (axis, item) in radar.axes.iter().enumerate() {
        let (x, y) = point(center, axes, axis, radius + 10.0);
        let direction = (x - center.0) / (radius + 10.0);
        let (anchor, available) = if direction > 0.3 {
            (TextAnchor::Start, width - margin - x)
        } else if direction < -0.3 {
            (TextAnchor::End, x - margin)
        } else {
            (
                TextAnchor::Middle,
                2.0 * (x - margin).min(width - margin - x),
            )
        };
        // Above the chart the name clears the value of the outer ring.
        let y = if direction.abs() <= 0.3 && y < center.1 {
            y - 6.0
        } else {
            y
        };
        let lift = if y > center.1 + 4.0 {
            10.0
        } else if y < center.1 - 4.0 {
            -2.0
        } else {
            4.0
        };
        elements.push(Element::Text(Text {
            x,
            y: y + lift,
            class: "chartlet-label",
            anchor,
            content: fit_text(
                &item.label,
                available,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/radar/axes/{axis}/label"),
            ),
        }));
    }
    // The polygons, and a marker on every value.
    for (index, series) in radar.series.iter().enumerate() {
        let spots: Vec<(f64, f64)> = series
            .values
            .iter()
            .enumerate()
            .map(|(axis, value)| {
                point(
                    center,
                    axes,
                    axis,
                    radius * radar.fraction(axis, *value).clamp(0.0, 1.0),
                )
            })
            .collect();
        let mut points = spots.clone();
        points.push(spots[0]);
        elements.push(Element::Polyline(Polyline {
            points,
            class: AREAS[index],
            series_index: Some(index),
            style_index: None,
            tooltip: Some(series.name.clone()),
        }));
        for (axis, (x, y)) in spots.into_iter().enumerate() {
            elements.push(Element::Circle(Circle {
                cx: x,
                cy: y,
                radius: 3.5,
                class: DOTS[index],
                series_index: Some(index),
                style_index: None,
                tooltip: Some(format!(
                    "{}, {}: {}",
                    series.name,
                    radar.axes[axis].label,
                    format_value(series.values[axis], spec.number_style())
                )),
            }));
        }
    }
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

/// The axis where a series stands highest and the one where it stands lowest, as the label and
/// the value, by the place of the value between the center and the end of its axis; the first of
/// equals wins.
fn extremes(spec: &ChartSpec, series: usize) -> ((&str, String), (&str, String)) {
    let radar = radar(spec);
    let values = &radar.series[series].values;
    let fractions: Vec<f64> = values
        .iter()
        .enumerate()
        .map(|(axis, value)| radar.fraction(axis, *value))
        .collect();
    let (mut strongest, mut weakest) = (0, 0);
    for (axis, fraction) in fractions.iter().enumerate() {
        if *fraction > fractions[strongest] {
            strongest = axis;
        }
        if *fraction < fractions[weakest] {
            weakest = axis;
        }
    }
    let name = |axis: usize| {
        (
            radar.axes[axis].label.as_str(),
            format_value(values[axis], spec.number_style()),
        )
    };
    (name(strongest), name(weakest))
}

/// The radar chart in sentences: its axes, and for every series its strongest and weakest axis.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let radar = radar(spec);
    let axes: Vec<&str> = radar.axes.iter().map(|axis| axis.label.as_str()).collect();
    let mut description = text::radar_summary(
        spec.locale,
        (radar.axes.len(), radar.series.len()),
        &axes.join(", "),
    );
    for (index, series) in radar.series.iter().enumerate() {
        let (strongest, weakest) = extremes(spec, index);
        description.push(' ');
        description.push_str(&text::radar_series(
            spec.locale,
            &series.name,
            (
                (strongest.0, strongest.1.as_str()),
                (weakest.0, weakest.1.as_str()),
            ),
        ));
    }
    description
}

/// One row per axis: its range and the value of every series.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let words = spec.locale.words();
    let radar = radar(spec);
    let style = spec.number_style();
    let mut columns = vec![words.axis.to_owned(), words.axis_range.to_owned()];
    columns.extend(radar.series.iter().map(|series| series.name.clone()));
    let rows = radar
        .axes
        .iter()
        .enumerate()
        .map(|(axis, item)| {
            let mut row = vec![
                item.label.clone(),
                format!(
                    "{}–{}",
                    format_value(radar.min(axis), style),
                    format_value(radar.max(axis), style)
                ),
            ];
            row.extend(
                radar
                    .series
                    .iter()
                    .map(|series| format_value(series.values[axis], style)),
            );
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}
