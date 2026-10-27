//! Polar charts: one value per category of a cycle, at an angle by the category and a distance
//! from the center by the value, drawn as wedges or as lollipops. Zero is at the center, four rings
//! carry their values along the top, and the category names stand outside the outer ring; a name
//! that would run into the one before it is left out, and the description and the data table name
//! every category and value.

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{LABEL_SIZE, count, fit_text, format_value, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Polyline, Scene, Text, TextAnchor},
    spec::{ChartSpec, PolarSpec, RINGS},
    text,
};

/// The share of the angle of a category that its wedge takes, and the angle of a step of an arc.
const WEDGE: f64 = 0.8;
const ARC_STEP: f64 = 0.05;

fn polar(spec: &ChartSpec) -> &PolarSpec {
    spec.polar
        .as_ref()
        .expect("validated polar charts carry a polar block")
}

/// The angle of a category: the first at the top, the others clockwise.
fn angle(points: usize, index: usize) -> f64 {
    std::f64::consts::TAU * count(index) / count(points) - std::f64::consts::FRAC_PI_2
}

fn at(center: (f64, f64), angle: f64, distance: f64) -> (f64, f64) {
    (
        center.0 + distance * angle.cos(),
        center.1 + distance * angle.sin(),
    )
}

/// What the values stand for: the `valueLabel`, or the word for a value.
fn value_name(spec: &ChartSpec) -> String {
    polar(spec)
        .value_label
        .clone()
        .unwrap_or_else(|| spec.locale.words().value.to_owned())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let polar = polar(spec);
    let points = polar.points.len();
    let end = polar.end();
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
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let room = polar
        .points
        .iter()
        .map(|point| metrics.width(&point.label, LABEL_SIZE))
        .fold(0.0, f64::max)
        .min(width * 0.22)
        + 16.0;
    let area = height - top - 12.0;
    let radius = (width / 2.0 - margin - room)
        .min(area / 2.0 - 20.0)
        .floor()
        .max(30.0);
    let center = (width / 2.0, top + area / 2.0);
    for ring in 1..=RINGS {
        elements.push(Element::Circle(Circle {
            cx: center.0,
            cy: center.1,
            radius: radius * f64::from(ring) / f64::from(RINGS),
            class: "chartlet-polar-ring",
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
    let style = spec.number_style();
    for (index, point) in polar.points.iter().enumerate() {
        let theta = angle(points, index);
        let distance = radius * point.value / end;
        let tooltip = Some(format!(
            "{}: {}",
            point.label,
            format_value(point.value, style)
        ));
        if polar.is_lollipops() {
            let tip = at(center, theta, distance);
            elements.push(Element::Line(Line {
                x1: center.0,
                y1: center.1,
                x2: tip.0,
                y2: tip.1,
                class: "chartlet-polar-stem",
            }));
            elements.push(Element::Circle(Circle {
                cx: tip.0,
                cy: tip.1,
                radius: 4.5,
                class: "chartlet-polar-dot",
                series_index: None,
                style_index: None,
                tooltip,
            }));
        } else if distance > 0.0 {
            let half = std::f64::consts::TAU / count(points) * WEDGE / 2.0;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let steps = ((2.0 * half / ARC_STEP).ceil() as usize).max(2);
            let mut outline = vec![center];
            for step in 0..=steps {
                outline.push(at(
                    center,
                    theta - half + 2.0 * half * count(step) / count(steps),
                    distance,
                ));
            }
            outline.push(center);
            elements.push(Element::Polyline(Polyline {
                points: outline,
                class: "chartlet-polar-bar",
                series_index: None,
                style_index: None,
                tooltip,
            }));
        }
    }
    for ring in 1..=RINGS {
        let fraction = f64::from(ring) / f64::from(RINGS);
        elements.push(Element::Text(Text {
            x: center.0 + 4.0,
            y: center.1 - radius * fraction - 3.0,
            class: "chartlet-polar-tick",
            anchor: TextAnchor::Start,
            content: format_value(end * fraction, style),
        }));
    }
    // The names of the categories outside the outer ring; one that would run into the one before
    // is left out.
    let mut placed: Vec<(f64, f64, f64, f64)> = Vec::new();
    let mut omitted = false;
    for (index, point) in polar.points.iter().enumerate() {
        let theta = angle(points, index);
        let (x, y) = at(center, theta, radius + 10.0);
        let direction = theta.cos();
        let anchor = if direction > 0.3 {
            TextAnchor::Start
        } else if direction < -0.3 {
            TextAnchor::End
        } else {
            TextAnchor::Middle
        };
        let available = match anchor {
            TextAnchor::Start => width - margin - x,
            TextAnchor::End => x - margin,
            TextAnchor::Middle => 2.0 * (x - margin).min(width - margin - x),
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
        let content = fit_text(
            &point.label,
            available,
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/polar/points/{index}/label"),
        );
        let reach = metrics.width(&content, LABEL_SIZE);
        let (from, to) = match anchor {
            TextAnchor::Start => (x, x + reach),
            TextAnchor::End => (x - reach, x),
            TextAnchor::Middle => (x - reach / 2.0, x + reach / 2.0),
        };
        let bounds = (from, y + lift - 11.0, to, y + lift + 3.0);
        if placed
            .iter()
            .any(|b| bounds.0 < b.2 && b.0 < bounds.2 && bounds.1 < b.3 && b.1 < bounds.3)
        {
            omitted = true;
            continue;
        }
        placed.push(bounds);
        elements.push(Element::Text(Text {
            x,
            y: y + lift,
            class: "chartlet-label",
            anchor,
            content,
        }));
    }
    if omitted {
        warnings.push(ChartWarning::new(
            "labels_omitted",
            "/polar/points",
            "some category names do not fit around the chart and were left out; every category remains in the data table",
        ));
    }
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

/// The polar chart in a sentence: its categories, the highest and the lowest, and the outer ring.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let polar = polar(spec);
    let style = spec.number_style();
    let mut highest = 0;
    let mut lowest = 0;
    for (index, point) in polar.points.iter().enumerate() {
        if point.value > polar.points[highest].value {
            highest = index;
        }
        if point.value < polar.points[lowest].value {
            lowest = index;
        }
    }
    let named = |index: usize| {
        (
            polar.points[index].label.as_str(),
            format_value(polar.points[index].value, style),
        )
    };
    let (high, low) = (named(highest), named(lowest));
    let labels: Vec<&str> = polar
        .points
        .iter()
        .map(|point| point.label.as_str())
        .collect();
    text::polar_summary(
        spec.locale,
        (polar.points.len(), &labels.join(", ")),
        ((high.0, high.1.as_str()), (low.0, low.1.as_str())),
        (&value_name(spec), &format_value(polar.end(), style)),
    )
}

/// One row per category: its value.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let words = spec.locale.words();
    let style = spec.number_style();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: vec![words.category.to_owned(), value_name(spec)],
        rows: polar(spec)
            .points
            .iter()
            .map(|point| vec![point.label.clone(), format_value(point.value, style)])
            .collect(),
    }
}
