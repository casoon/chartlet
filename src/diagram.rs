//! What the diagram types share: arrowheads, rounded corners, data-store cylinders, shadows,
//! labels on up to two lines and on chips, number badges, and a canvas that grows when a diagram
//! does not fit it.

use crate::{
    error::ChartWarning,
    layout::{fit_text, two_lines},
    metrics::TextMetrics,
    scene::{Circle, Element, Polyline, Rect, Text, TextAnchor},
    spec::ChartSpec,
};

/// Length and half width of an arrowhead.
pub(crate) const HEAD: f64 = 8.0;
const HEAD_HALF: f64 = 3.5;
/// Radius of the corners where an edge changes course.
const CORNER: f64 = 6.0;
/// How far a step's shadow falls below it.
const SHADOW: f64 = 2.5;
/// The text of a chip, the distance between its lines, and the space around it.
pub(crate) const CHIP_SIZE: f64 = 12.0;
pub(crate) const CHIP_LINE: f64 = 15.0;
const CHIP_PAD_X: f64 = 5.0;
const CHIP_PAD_Y: f64 = 3.0;
/// How far a chip reaches beyond its text on each side, with a little air.
pub(crate) const CHIP_REACH: f64 = CHIP_PAD_X + 2.0;
/// Radius of a number badge.
pub(crate) const BADGE: f64 = 8.0;

/// A polyline with nothing but its points, its class and an optional tooltip.
pub(crate) fn polyline(
    points: Vec<(f64, f64)>,
    class: &'static str,
    tooltip: Option<String>,
) -> Polyline {
    Polyline {
        points,
        class,
        topic: None,
        series_index: None,
        style_index: None,
        tooltip,
    }
}

/// The arrowhead with its tip at `tip`, pointing along the unit vector `direction`: a closed
/// triangle when `filled`, two strokes otherwise.
pub(crate) fn arrowhead(
    tip: (f64, f64),
    direction: (f64, f64),
    filled: bool,
    class: &'static str,
) -> Polyline {
    let base = (tip.0 - direction.0 * HEAD, tip.1 - direction.1 * HEAD);
    let side = (-direction.1 * HEAD_HALF, direction.0 * HEAD_HALF);
    let one = (base.0 + side.0, base.1 + side.1);
    let other = (base.0 - side.0, base.1 - side.1);
    let points = if filled {
        vec![one, tip, other, one]
    } else {
        vec![one, tip, other]
    };
    polyline(points, class, None)
}

/// `points` with every corner rounded: each turn becomes a short curve, as far as the segments on
/// either side of it leave room.
pub(crate) fn rounded(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut output = vec![points[0]];
    for corner in points.windows(3) {
        let (before, at, after) = (corner[0], corner[1], corner[2]);
        let into = (at.0 - before.0).hypot(at.1 - before.1);
        let out = (after.0 - at.0).hypot(after.1 - at.1);
        let radius = CORNER.min(into / 2.0).min(out / 2.0);
        if radius < 0.5 {
            output.push(at);
            continue;
        }
        let start = (
            at.0 - (at.0 - before.0) / into * radius,
            at.1 - (at.1 - before.1) / into * radius,
        );
        let end = (
            at.0 + (after.0 - at.0) / out * radius,
            at.1 + (after.1 - at.1) / out * radius,
        );
        output.push(start);
        // Points on the quadratic curve from `start` to `end` that `at` controls.
        for t in [0.25, 0.5, 0.75] {
            let u = 1.0 - t;
            output.push((
                u * u * start.0 + 2.0 * u * t * at.0 + t * t * end.0,
                u * u * start.1 + 2.0 * u * t * at.1 + t * t * end.1,
            ));
        }
        output.push(end);
    }
    output.push(points[points.len() - 1]);
    output
}

/// A shape's shadow: the same shape, a little lower, in the shadow color.
fn shadow(element: &Element) -> Option<Element> {
    match element {
        Element::Rect(rect) => Some(Element::Rect(Rect {
            y: rect.y + SHADOW,
            class: "chartlet-diagram-shadow",
            tooltip: None,
            ..rect.clone()
        })),
        Element::Polyline(shape) => Some(Element::Polyline(polyline(
            shape.points.iter().map(|(x, y)| (*x, y + SHADOW)).collect(),
            "chartlet-diagram-shadow",
            None,
        ))),
        _ => None,
    }
}

/// Pushes `shape` with its shadow beneath it.
pub(crate) fn with_shadow(shape: Element, elements: &mut Vec<Element>) {
    if let Some(shadow) = shadow(&shape) {
        elements.push(shadow);
    }
    elements.push(shape);
}

/// The plate of a chip with `lines` whose first baseline is at `y`: left, top, width, height.
pub(crate) fn chip_box(
    lines: &[String],
    (x, y): (f64, f64),
    anchor: TextAnchor,
    metrics: &impl TextMetrics,
) -> (f64, f64, f64, f64) {
    let width = lines
        .iter()
        .map(|line| metrics.width(line, CHIP_SIZE))
        .fold(0.0, f64::max);
    let left = match anchor {
        TextAnchor::Start => x,
        TextAnchor::Middle => x - width / 2.0,
        TextAnchor::End => x - width,
    };
    let rows = crate::layout::count(lines.len());
    (
        left - CHIP_PAD_X,
        y - 11.0 - CHIP_PAD_Y,
        width + 2.0 * CHIP_PAD_X,
        15.0 + CHIP_LINE * (rows - 1.0) + 2.0 * CHIP_PAD_Y,
    )
}

/// A label on a chip: its lines on a small rounded plate in the background color, so that it
/// reads clearly where it crosses or sits beside lines. `y` is the baseline of the first line.
pub(crate) fn chip(
    lines: &[String],
    (x, y): (f64, f64),
    anchor: TextAnchor,
    metrics: &impl TextMetrics,
    elements: &mut Vec<Element>,
) {
    let (left, top, width, height) = chip_box(lines, (x, y), anchor, metrics);
    elements.push(Element::Rect(Rect {
        x: left,
        y: top,
        width,
        height,
        class: "chartlet-diagram-chip",
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    for (index, line) in lines.iter().enumerate() {
        elements.push(Element::Text(Text {
            x,
            y: y + CHIP_LINE * crate::layout::count(index),
            class: "chartlet-diagram-chip-text",
            anchor,
            content: line.clone(),
        }));
    }
}

/// A round badge with a number, centred on `center`.
pub(crate) fn badge(number: usize, center: (f64, f64), elements: &mut Vec<Element>) {
    elements.push(Element::Circle(Circle {
        cx: center.0,
        cy: center.1,
        radius: BADGE,
        class: "chartlet-diagram-badge",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    elements.push(Element::Text(Text {
        x: center.0,
        y: center.1 + 3.5,
        class: "chartlet-diagram-badge-text",
        anchor: TextAnchor::Middle,
        content: number.to_string(),
    }));
}

/// A data store: a cylinder in the box at `x`, `y`, whose top shows its front rim. `classes`
/// style the body and the rim.
pub(crate) fn cylinder(
    (x, y, width, height): (f64, f64, f64, f64),
    (body, rim): (&'static str, &'static str),
    tooltip: Option<String>,
    elements: &mut Vec<Element>,
) {
    const RIM: f64 = 5.0;
    const STEPS: u32 = 12;
    let center = x + width / 2.0;
    let arc = |middle: f64, sign: f64, forward: bool| -> Vec<(f64, f64)> {
        (0..=STEPS)
            .map(|step| {
                let angle = std::f64::consts::PI * f64::from(step) / f64::from(STEPS);
                let cos = if forward { -angle.cos() } else { angle.cos() };
                (
                    center + width / 2.0 * cos,
                    middle + sign * RIM * angle.sin(),
                )
            })
            .collect()
    };
    let mut outline = vec![(x, y + RIM)];
    outline.extend(arc(y + height - RIM, 1.0, true));
    outline.extend(arc(y + RIM, -1.0, false));
    outline.push((x, y + RIM));
    with_shadow(
        Element::Polyline(polyline(outline, body, tooltip)),
        elements,
    );
    elements.push(Element::Polyline(polyline(
        arc(y + RIM, 1.0, true),
        rim,
        None,
    )));
}

/// A label on one or two lines that fit `max_width`; a longer second line, or a first word wider
/// than that, is shortened.
pub(crate) fn wrap(
    label: &str,
    max_width: f64,
    size: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) -> Vec<String> {
    match two_lines(label, max_width, size, metrics) {
        Some((first, rest)) => vec![
            first.to_owned(),
            fit_text(rest, max_width, size, metrics, warnings, path),
        ],
        None => vec![fit_text(label, max_width, size, metrics, warnings, path)],
    }
}

/// Which orientation `auto` takes, from the size each needs: portrait where it fits the canvas,
/// landscape where only that fits, and otherwise the one that has to grow the canvas less.
pub(crate) fn prefers_landscape(
    spec: &ChartSpec,
    portrait: (f64, f64),
    landscape: (f64, f64),
) -> bool {
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let growth = |(needed_width, needed_height): (f64, f64)| {
        (needed_width / width).max(needed_height / height)
    };
    growth(portrait) > 1.0 && growth(landscape) < growth(portrait)
}

/// Warns that the diagram did not fit the canvas and was drawn larger, naming the size it needs.
pub(crate) fn warn_growth(
    spec: &ChartSpec,
    width: f64,
    height: f64,
    warnings: &mut Vec<ChartWarning>,
) {
    for (path, needed, given) in [
        ("/width", pixels(width), spec.width),
        ("/height", pixels(height), spec.height),
    ] {
        if needed > given {
            warnings.push(ChartWarning::new(
                "canvas_too_small",
                path,
                format!(
                    "the diagram needs {needed} pixels here and was drawn that large; raise {} to {needed}, shorten labels, or try the other orientation",
                    &path[1..]
                ),
            ));
        }
    }
}

/// Whole pixels, rounded up so that nothing is cut off. A layout that fills the canvas exactly
/// may come out a hair above it in floating point; that is no reason to grow it by a pixel.
pub(crate) fn pixels(value: f64) -> u32 {
    // Canvas sizes stay far below the range of u32 and are never negative.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let pixels = (value - 1e-6).ceil() as u32;
    pixels
}
