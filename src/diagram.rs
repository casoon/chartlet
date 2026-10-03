//! What the diagram types share: arrowheads, data-store cylinders, labels on up to two lines, and
//! a canvas that grows when a diagram does not fit it.

use crate::{
    error::ChartWarning,
    layout::{fit_text, two_lines},
    metrics::TextMetrics,
    scene::{Element, Polyline},
    spec::ChartSpec,
};

/// Length and half width of an arrowhead.
pub(crate) const HEAD: f64 = 9.0;
const HEAD_HALF: f64 = 4.5;

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
    elements.push(Element::Polyline(polyline(outline, body, tooltip)));
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

/// Whole pixels, rounded up so that nothing is cut off.
pub(crate) fn pixels(value: f64) -> u32 {
    // Canvas sizes stay far below the range of u32 and are never negative.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let pixels = value.ceil() as u32;
    pixels
}
