//! Box plots: one box per category, from its first to its third quartile with the median across
//! it, whiskers to the most extreme values within reach and a point for every value beyond. The
//! boxes are computed from the observations of a category, or drawn from five numbers that were
//! computed elsewhere.

use std::fmt::Write as _;

use crate::{
    error::ChartWarning,
    layout::{
        LABEL_SIZE, NumericScale, PlotArea, WithReserve, add_bottom_category_title, base_elements,
        base_elements_with_title, count, format_value, horizontal_title, push_category_label,
        push_side_label, title_extra, warn_if_labels_omitted,
    },
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Polyline, Rect, Scene, Text, TextAnchor},
    spec::{BoxDisplay, BoxSummary, ChartSpec, Orientation, ViolinScale},
    text,
};

/// How far a whisker's cap reaches on either side of the whisker, as a share of the box.
const CAP: f64 = 0.5;
/// Radius of a point beyond the whiskers.
const OUTLIER_RADIUS: f64 = 3.5;
/// A box of no height is still drawn this thick.
const MIN_EXTENT: f64 = 2.0;

/// The summary of the box at `index`, written in a sentence for tooltips: median first.
fn summary_label(spec: &ChartSpec, summary: &BoxSummary) -> String {
    let show = |value| format_value(value, spec.number_style());
    let words = spec.locale.words();
    let mut label = format!(
        "{}: {} ({}: {}, {}: {}, {}: {}, {}: {})",
        words.median,
        show(summary.median),
        words.quartile_1,
        show(summary.q1),
        words.quartile_3,
        show(summary.q3),
        words.minimum,
        show(summary.min),
        words.maximum,
        show(summary.max),
    );
    if let Some(mean) = summary.mean.filter(|_| spec.show_mean) {
        write!(label, ", {}: {}", words.mean, show(mean)).expect("writing to String cannot fail");
    }
    if !summary.outliers.is_empty() {
        let outliers: Vec<String> = summary.outliers.iter().map(|value| show(*value)).collect();
        write!(label, ", {}: {}", words.outliers, outliers.join(", "))
            .expect("writing to String cannot fail");
    }
    label
}

fn tooltip(spec: &ChartSpec, index: usize, summary: &BoxSummary) -> String {
    let count = summary
        .count
        .map(|count| format!(", {} {count}", spec.locale.words().observations))
        .unwrap_or_default();
    format!(
        "{}: {}{count}",
        spec.boxes[index].label,
        summary_label(spec, summary)
    )
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let summaries = spec.box_summaries();
    let values: Vec<f64> = (0..summaries.len())
        .flat_map(|index| {
            let (low, high) = reach(spec, index, &summaries[index]);
            [low, high]
        })
        .collect();
    let scale = NumericScale::for_axis(values.into_iter(), false, &spec.value_axis);
    let peak = violin_peak(spec, &summaries);
    let elements = match spec.orientation {
        Orientation::Horizontal => {
            layout_horizontal(spec, (&summaries, peak), &scale, warnings, metrics)
        }
        Orientation::Vertical => {
            layout_vertical(spec, (&summaries, peak), &scale, warnings, metrics)
        }
    };
    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

const BOX: [&str; 5] = [
    "chartlet-box",
    "chartlet-box chartlet-box-g1",
    "chartlet-box chartlet-box-g2",
    "chartlet-box chartlet-box-g3",
    "chartlet-box chartlet-box-g4",
];
const WHISKER: [&str; 5] = [
    "chartlet-box-whisker",
    "chartlet-box-whisker chartlet-box-g1",
    "chartlet-box-whisker chartlet-box-g2",
    "chartlet-box-whisker chartlet-box-g3",
    "chartlet-box-whisker chartlet-box-g4",
];
const OUTLIER: [&str; 5] = [
    "chartlet-box-outlier",
    "chartlet-box-outlier chartlet-box-g1",
    "chartlet-box-outlier chartlet-box-g2",
    "chartlet-box-outlier chartlet-box-g3",
    "chartlet-box-outlier chartlet-box-g4",
];
const VIOLIN: [&str; 5] = [
    "chartlet-violin",
    "chartlet-violin chartlet-box-g1",
    "chartlet-violin chartlet-box-g2",
    "chartlet-violin chartlet-box-g3",
    "chartlet-violin chartlet-box-g4",
];
const POINT: [&str; 5] = [
    "chartlet-box-point",
    "chartlet-box-point chartlet-box-g1",
    "chartlet-box-point chartlet-box-g2",
    "chartlet-box-point chartlet-box-g3",
    "chartlet-box-point chartlet-box-g4",
];

/// The room the legend of the groups takes above the plot.
fn legend_space(spec: &ChartSpec) -> f64 {
    if spec.box_groups().is_empty() {
        0.0
    } else {
        22.0
    }
}

/// The legend of the groups in a row under the title.
fn push_legend(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (left, top): (f64, f64),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    let right = f64::from(spec.width) - 12.0;
    let (mut x, mut y) = (left, top);
    for (index, group) in spec.box_groups().iter().enumerate() {
        let reach = 16.0 + metrics.width(group, LABEL_SIZE);
        if x > left && x + reach > right {
            x = left;
            y += 18.0;
        }
        elements.push(Element::Rect(Rect {
            x,
            y,
            width: 10.0,
            height: 10.0,
            class: BOX[index + 1],
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: x + 16.0,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: crate::layout::fit_text(
                group,
                (right - x - 16.0).max(0.0),
                LABEL_SIZE,
                metrics,
                warnings,
                "/boxes",
            ),
        }));
        x += reach + 16.0;
    }
}

/// Draws one category as a box, a violin or a strip, as the specification says.
fn push_mark(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    entry: (usize, &BoxSummary),
    orientation: Orientation,
    along: impl Fn(f64) -> f64,
    across: (f64, f64),
    peak: f64,
) {
    let (lead, thick) = across;
    let mean = entry.1.mean.filter(|_| spec.show_mean).map(&along);
    match spec.box_display {
        BoxDisplay::Box => push_box(elements, spec, entry, orientation, &along, across),
        BoxDisplay::Violin => {
            push_violin(elements, spec, entry, orientation, &along, (across, peak));
        }
        BoxDisplay::Strip => push_strip(elements, spec, entry, orientation, &along, across),
        BoxDisplay::Beeswarm => push_beeswarm(elements, spec, entry, orientation, &along, across),
        BoxDisplay::Ridgeline => {
            push_ridgeline(elements, spec, entry, orientation, &along, (across, peak));
        }
    }
    if let Some(at) = mean {
        if spec.box_display == BoxDisplay::Ridgeline {
            push_mean(elements, orientation, at, lead, thick.abs());
        } else {
            push_mean(elements, orientation, at, lead + thick / 2.0, thick);
        }
    }
}

/// The mean as a diamond on the middle of the box.
fn push_mean(
    elements: &mut Vec<Element>,
    orientation: Orientation,
    at: f64,
    middle: f64,
    thick: f64,
) {
    let half = (thick * 0.18).clamp(4.0, 7.0);
    let points = [(-half, 0.0), (0.0, -half), (half, 0.0), (0.0, half)]
        .into_iter()
        .map(|(a, b)| page(orientation, at + a, middle + b))
        .collect();
    elements.push(Element::Polyline(Polyline {
        points,
        class: "chartlet-box-mean",
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
}

/// A point of the page from its place along the value axis and across it.
fn page(orientation: Orientation, along: f64, across: f64) -> (f64, f64) {
    match orientation {
        Orientation::Horizontal => (along, across),
        Orientation::Vertical => (across, along),
    }
}

/// The points along the value axis at which a violin is estimated.
const VIOLIN_STEPS: usize = 48;

/// The density of the observations at `steps + 1` evenly spaced values from the lowest to the
/// highest, by a Gaussian kernel with Silverman's bandwidth, as `(value, density)`.
fn density(values: &[f64], summary: &BoxSummary) -> Vec<(f64, f64)> {
    let n = count(values.len());
    let mean = values.iter().sum::<f64>() / n;
    let spread = (values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / n)
        .sqrt();
    let reach = (summary.q3 - summary.q1) / 1.34;
    let (low, high) = values
        .iter()
        .fold((f64::MAX, f64::MIN), |(low, high), value| {
            (low.min(*value), high.max(*value))
        });
    let width = if spread > 0.0 && reach > 0.0 {
        spread.min(reach)
    } else {
        spread.max(reach)
    };
    let bandwidth = if width > 0.0 {
        0.9 * width * n.powf(-0.2)
    } else {
        ((high - low) / 10.0).max(1.0)
    };
    (0..=VIOLIN_STEPS)
        .map(|step| {
            let at = low + (high - low) * count(step) / count(VIOLIN_STEPS);
            let sum: f64 = values
                .iter()
                .map(|value| (-0.5 * ((at - value) / bandwidth).powi(2)).exp())
                .sum();
            (
                at,
                sum / (n * bandwidth * (2.0 * std::f64::consts::PI).sqrt()),
            )
        })
        .collect()
}

/// The estimated density of the observations as an outline mirrored about the middle, as wide as
/// the box would be at its widest, with a narrow box and the median inside.
fn push_violin(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (index, summary): (usize, &BoxSummary),
    orientation: Orientation,
    along: impl Fn(f64) -> f64,
    ((lead, thick), peak): ((f64, f64), f64),
) {
    let middle = lead + thick / 2.0;
    let curve = density(&spec.boxes[index].values, summary);
    let most = curve.iter().map(|point| point.1).fold(0.0, f64::max);
    let observations = count(spec.boxes[index].values.len());
    let half = |density: f64| match spec.violin_scale {
        ViolinScale::Width => thick / 2.0 * density / most,
        ViolinScale::Area => thick / 2.0 * density / peak,
        ViolinScale::Count => thick / 2.0 * density * observations / peak,
    };
    let mut points: Vec<(f64, f64)> = curve
        .iter()
        .map(|(at, d)| page(orientation, along(*at), middle - half(*d)))
        .collect();
    points.extend(
        curve
            .iter()
            .rev()
            .map(|(at, d)| page(orientation, along(*at), middle + half(*d))),
    );
    elements.push(Element::Polyline(Polyline {
        points,
        class: VIOLIN[spec.box_tone(index)],
        series_index: None,
        style_index: None,
        tooltip: Some(tooltip(spec, index, summary)),
    }));
    let inner = thick.min(40.0) / 8.0;
    let (low, high) = (along(summary.q1), along(summary.q3));
    let (x, y, width, height) = match orientation {
        Orientation::Horizontal => (
            low.min(high),
            middle - inner / 2.0,
            (high - low).abs(),
            inner,
        ),
        Orientation::Vertical => (
            middle - inner / 2.0,
            low.min(high),
            inner,
            (high - low).abs(),
        ),
    };
    elements.push(Element::Rect(Rect {
        x,
        y,
        width,
        height,
        class: "chartlet-violin-box",
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    let (cx, cy) = page(orientation, along(summary.median), middle);
    elements.push(Element::Circle(Circle {
        cx,
        cy,
        radius: 3.0,
        class: "chartlet-violin-median",
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
}

/// Every observation as a point, spread over the width by a fixed sequence so that the same data
/// always gives the same picture, and the median across.
fn push_strip(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (index, summary): (usize, &BoxSummary),
    orientation: Orientation,
    along: impl Fn(f64) -> f64,
    (lead, thick): (f64, f64),
) {
    let middle = lead + thick / 2.0;
    for (position, value) in spec.boxes[index].values.iter().enumerate() {
        let spread = (count(position) * 0.618_033_988_75).fract() - 0.5;
        let (cx, cy) = page(orientation, along(*value), middle + spread * thick * 0.8);
        elements.push(Element::Circle(Circle {
            cx,
            cy,
            radius: 2.5,
            class: POINT[spec.box_tone(index)],
            series_index: None,
            style_index: None,
            tooltip: Some(format!(
                "{}: {}",
                spec.boxes[index].label,
                format_value(*value, spec.number_style())
            )),
        }));
    }
    let (x1, y1) = page(orientation, along(summary.median), lead);
    let (x2, y2) = page(orientation, along(summary.median), lead + thick);
    elements.push(Element::Line(Line {
        x1,
        y1,
        x2,
        y2,
        class: "chartlet-box-median",
    }));
}

/// How far from the middle each point of a swarm stands, in the order of `positions`, along the
/// value axis in pixels: points are placed from the lowest on, each as near the middle as it can
/// be without touching a point placed before it, so the same data always gives the same picture.
pub(crate) fn swarm_offsets(positions: &[f64], radius: f64) -> Vec<f64> {
    let mut order: Vec<usize> = (0..positions.len()).collect();
    crate::sort::by(&mut order, |a, b| positions[*a].total_cmp(&positions[*b]));
    let clearance = 2.0 * radius + 0.5;
    let mut placed: Vec<(f64, f64)> = Vec::with_capacity(order.len());
    let mut offsets = vec![0.0; positions.len()];
    for index in order {
        let at = positions[index];
        // The points placed before this one that it could touch.
        let near: Vec<(f64, f64)> = placed
            .iter()
            .rev()
            .take_while(|(other, _)| at - other < clearance)
            .copied()
            .collect();
        let mut candidates = vec![0.0];
        for (other, offset) in &near {
            let reach = (clearance.powi(2) - (at - other).powi(2)).max(0.0).sqrt();
            candidates.push(offset + reach);
            candidates.push(offset - reach);
        }
        crate::sort::by(&mut candidates, |a, b| {
            a.abs().total_cmp(&b.abs()).then(b.total_cmp(a))
        });
        let offset = candidates
            .into_iter()
            .find(|candidate| {
                near.iter().all(|(other, offset)| {
                    (candidate - offset).powi(2) + (at - other).powi(2) >= clearance.powi(2) - 1e-6
                })
            })
            .expect("a candidate beside the nearest point is always free");
        placed.push((at, offset));
        offsets[index] = offset;
    }
    offsets
}

/// The radii a swarm tries, the largest first, and the sideways room it may use.
const SWARM_RADII: [f64; 4] = [3.0, 2.5, 2.0, 1.5];

/// Every observation as a point, moved sideways just far enough that no two overlap, with the
/// median across; a smaller point is tried before the swarm outgrows its band. The box and its
/// whiskers lie under the points if `showBox` asks for them.
fn push_beeswarm(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (index, summary): (usize, &BoxSummary),
    orientation: Orientation,
    along: impl Fn(f64) -> f64,
    (lead, thick): (f64, f64),
) {
    if spec.show_box {
        // The points beyond the whiskers are among the dots already.
        let inside = BoxSummary {
            outliers: Vec::new(),
            ..summary.clone()
        };
        push_box(
            elements,
            spec,
            (index, &inside),
            orientation,
            &along,
            (lead + thick * 0.3, thick * 0.4),
        );
    }
    let middle = lead + thick / 2.0;
    let values = &spec.boxes[index].values;
    let positions: Vec<f64> = values.iter().map(|value| along(*value)).collect();
    let room = thick / 2.0;
    let mut radius = SWARM_RADII[0];
    let mut offsets = swarm_offsets(&positions, radius);
    for candidate in SWARM_RADII {
        radius = candidate;
        offsets = swarm_offsets(&positions, radius);
        if offsets.iter().all(|offset| offset.abs() + radius <= room) {
            break;
        }
    }
    let limit = (room - radius).max(0.0);
    for ((value, position), offset) in values.iter().zip(&positions).zip(&offsets) {
        let (cx, cy) = page(orientation, *position, middle + offset.clamp(-limit, limit));
        elements.push(Element::Circle(Circle {
            cx,
            cy,
            radius,
            class: POINT[spec.box_tone(index)],
            series_index: None,
            style_index: None,
            tooltip: Some(format!(
                "{}: {}",
                spec.boxes[index].label,
                format_value(*value, spec.number_style())
            )),
        }));
    }
    // With the box, its own median is already drawn, under the dots.
    if spec.show_box {
        return;
    }
    let (x1, y1) = page(orientation, along(summary.median), lead);
    let (x2, y2) = page(orientation, along(summary.median), lead + thick);
    elements.push(Element::Line(Line {
        x1,
        y1,
        x2,
        y2,
        class: "chartlet-box-median",
    }));
}

/// The estimated density of the observations as a filled curve on a baseline at `lead`, rising
/// by `thick` (negative where it rises toward lower page coordinates) where the density is `peak`,
/// the densest point of any curve of the chart; with the median as a line up to the curve.
fn push_ridgeline(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (index, summary): (usize, &BoxSummary),
    orientation: Orientation,
    along: impl Fn(f64) -> f64,
    ((lead, thick), peak): ((f64, f64), f64),
) {
    let curve = density(&spec.boxes[index].values, summary);
    let rise = |density: f64| lead + thick * density / peak;
    let mut points: Vec<(f64, f64)> = curve
        .iter()
        .map(|(at, d)| page(orientation, along(*at), rise(*d)))
        .collect();
    if let (Some(first), Some(last)) = (curve.first(), curve.last()) {
        points.push(page(orientation, along(last.0), lead));
        points.push(page(orientation, along(first.0), lead));
    }
    elements.push(Element::Polyline(Polyline {
        points,
        class: VIOLIN[spec.box_tone(index)],
        series_index: None,
        style_index: None,
        tooltip: Some(tooltip(spec, index, summary)),
    }));
    // The density under the median, between the two points of the curve around it.
    let height = curve
        .windows(2)
        .find(|pair| pair[1].0 >= summary.median)
        .map_or(0.0, |pair| {
            let span = pair[1].0 - pair[0].0;
            let share = if span > 0.0 {
                (summary.median - pair[0].0) / span
            } else {
                0.0
            };
            pair[0].1 + (pair[1].1 - pair[0].1) * share
        });
    let (x1, y1) = page(orientation, along(summary.median), lead);
    let (x2, y2) = page(orientation, along(summary.median), rise(height));
    elements.push(Element::Line(Line {
        x1,
        y1,
        x2,
        y2,
        class: "chartlet-box-median",
    }));
}

/// The densest point of any violin in the scale of the violins: the widest violin of the chart
/// is as wide as a box. Zero unless the violins share an area scale.
fn violin_peak(spec: &ChartSpec, summaries: &[BoxSummary]) -> f64 {
    let ridges = spec.box_display == BoxDisplay::Ridgeline;
    if !ridges
        && (spec.box_display != BoxDisplay::Violin || spec.violin_scale == ViolinScale::Width)
    {
        return 0.0;
    }
    spec.boxes
        .iter()
        .zip(summaries)
        .map(|(boxed, summary)| {
            let most = density(&boxed.values, summary)
                .iter()
                .map(|point| point.1)
                .fold(0.0, f64::max);
            if !ridges && spec.violin_scale == ViolinScale::Count {
                most * count(boxed.values.len())
            } else {
                most
            }
        })
        .fold(0.0, f64::max)
}

/// The box, whiskers, median and outliers of one category. `along` maps a value onto the value
/// axis; `across` is the leading edge and the thickness of the box.
fn push_box(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (index, summary): (usize, &BoxSummary),
    orientation: Orientation,
    along: impl Fn(f64) -> f64,
    across: (f64, f64),
) {
    let (lead, thick) = across;
    let tone = spec.box_tone(index);
    let middle = lead + thick / 2.0;
    let line = |from: (f64, f64), to: (f64, f64), class| {
        // `(along, across)` pairs, turned into page coordinates.
        let (x1, y1, x2, y2) = match orientation {
            Orientation::Horizontal => (from.0, from.1, to.0, to.1),
            Orientation::Vertical => (from.1, from.0, to.1, to.0),
        };
        Element::Line(Line {
            x1,
            y1,
            x2,
            y2,
            class,
        })
    };
    let (low, high) = (along(summary.q1), along(summary.q3));
    let (start, extent) = (low.min(high), (high - low).abs().max(MIN_EXTENT));
    let (x, y, width, height) = match orientation {
        Orientation::Horizontal => (start, lead, extent, thick),
        Orientation::Vertical => (lead, start, thick, extent),
    };
    let cap = thick * CAP / 2.0;
    // Whiskers first, so that the box lies over their ends.
    for (end, edge) in [(summary.min, summary.q1), (summary.max, summary.q3)] {
        elements.push(line(
            (along(edge), middle),
            (along(end), middle),
            WHISKER[tone],
        ));
        elements.push(line(
            (along(end), middle - cap),
            (along(end), middle + cap),
            WHISKER[tone],
        ));
    }
    elements.push(Element::Rect(Rect {
        x,
        y,
        width,
        height,
        class: BOX[tone],
        series_index: None,
        style_index: None,
        tooltip: Some(tooltip(spec, index, summary)),
    }));
    elements.push(line(
        (along(summary.median), lead),
        (along(summary.median), lead + thick),
        "chartlet-box-median",
    ));
    for value in &summary.outliers {
        let (cx, cy) = match orientation {
            Orientation::Horizontal => (along(*value), middle),
            Orientation::Vertical => (middle, along(*value)),
        };
        elements.push(Element::Circle(Circle {
            cx,
            cy,
            radius: OUTLIER_RADIUS,
            class: OUTLIER[tone],
            series_index: None,
            style_index: None,
            tooltip: Some(format!(
                "{}: {}",
                spec.boxes[index].label,
                format_value(*value, spec.number_style())
            )),
        }));
    }
}

/// How far the ridges of a ridgeline are apart and how high the densest curve rises, as
/// `(step, rise)`, if the chart is one: the rise is `RIDGE_OVERLAP` times the step, so that every
/// curve reaches into the one before it, and the last curve ends at the end of `extent`.
fn ridge_geometry(spec: &ChartSpec, extent: f64) -> Option<(f64, f64)> {
    if spec.box_display != BoxDisplay::Ridgeline {
        return None;
    }
    let step = extent / (count(spec.boxes.len() - 1) + RIDGE_OVERLAP);
    Some((step, step * RIDGE_OVERLAP))
}

/// How many steps between two ridges the densest curve rises.
const RIDGE_OVERLAP: f64 = 1.6;

/// The furthest values a box reaches, with its outliers; a violin or a strip reaches every
/// observation.
fn reach(spec: &ChartSpec, index: usize, summary: &BoxSummary) -> (f64, f64) {
    if spec.box_display != BoxDisplay::Box {
        return spec.boxes[index]
            .values
            .iter()
            .fold((f64::MAX, f64::MIN), |(low, high), value| {
                (low.min(*value), high.max(*value))
            });
    }
    summary
        .outliers
        .iter()
        .fold((summary.min, summary.max), |(low, high), value| {
            (low.min(*value), high.max(*value))
        })
}

fn layout_horizontal(
    spec: &ChartSpec,
    (summaries, peak): (&[BoxSummary], f64),
    scale: &NumericScale,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let widest = spec
        .boxes
        .iter()
        .map(|boxed| metrics.width(&boxed.label, LABEL_SIZE))
        .fold(0.0, f64::max);
    let left = (widest + 29.0).clamp(88.0, 210.0);
    let right = if spec.show_values {
        summaries
            .iter()
            .map(|summary| {
                WithReserve(metrics).width(
                    &format_value(summary.median, spec.number_style()),
                    LABEL_SIZE,
                )
            })
            .fold(0.0, f64::max)
            .clamp(8.0, 120.0)
            + 16.0
    } else {
        // The last tick label is centered on the end of the axis; half of it must fit.
        let last = scale.ticks().last().map_or(0.0, |value| {
            metrics.width(&scale.tick_label(value, spec.axis_style()), LABEL_SIZE)
        });
        f64::max(24.0, last / 2.0 + 4.0)
    };
    let bottom = if spec.value_axis.title.is_some() {
        56.0
    } else {
        36.0
    };
    let title = horizontal_title(spec, (left, width - left - right), metrics);
    let top = 78.0 + title_extra(spec, title.1, metrics) + legend_space(spec);
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: crate::layout::plot_height(height - top - bottom, warnings),
        vertical_bars: false,
    };
    let mut elements = base_elements_with_title(spec, scale, plot, title, warnings, metrics);
    push_legend(
        &mut elements,
        spec,
        (plot.left, plot.top - 56.0),
        metrics,
        warnings,
    );
    let band = plot.height / count(spec.boxes.len());
    let thickness = if spec.box_display == BoxDisplay::Box {
        (band * 0.5).clamp(4.0, 32.0)
    } else {
        (band * 0.75).clamp(8.0, 56.0)
    };
    let ridges = ridge_geometry(spec, plot.height);
    let along = |value| scale.map(value, plot.left, plot.left + plot.width);
    for (index, summary) in summaries.iter().enumerate() {
        // A ridge rises from its baseline, the first one at the top of the plot.
        let (center, across) = if let Some((step, rise)) = ridges {
            let baseline = plot.top + rise + step * count(index);
            (baseline - step * 0.3, (baseline, -rise))
        } else {
            let center = plot.top + band * (count(index) + 0.5);
            (center, (center - thickness / 2.0, thickness))
        };
        push_mark(
            &mut elements,
            spec,
            (index, summary),
            Orientation::Horizontal,
            along,
            across,
            peak,
        );
        if spec.show_values {
            elements.push(Element::Text(Text {
                x: along(reach(spec, index, summary).1) + 8.0,
                y: center + 4.0,
                class: "chartlet-value",
                anchor: TextAnchor::Start,
                content: format_value(summary.median, spec.number_style()),
            }));
        }
        push_side_label(
            &mut elements,
            &spec.boxes[index].label,
            (plot.left - 12.0, center),
            (plot.left - 28.0, band),
            metrics,
            warnings,
            &format!("/boxes/{index}/label"),
        );
    }
    elements
}

fn layout_vertical(
    spec: &ChartSpec,
    (summaries, peak): (&[BoxSummary], f64),
    scale: &NumericScale,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = f64::from(crate::layout::axis_gutter(spec.width));
    let right = f64::from(crate::layout::plot_margin(spec.width));
    let bottom = if spec.category_axis.title.is_some() {
        82.0
    } else {
        62.0
    };
    let top = 78.0 + title_extra(spec, width - left - right, metrics) + legend_space(spec);
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: crate::layout::plot_height(height - top - bottom, warnings),
        vertical_bars: true,
    };
    let mut elements = base_elements(spec, scale, plot, warnings, metrics);
    push_legend(
        &mut elements,
        spec,
        (plot.left, plot.top - 56.0),
        metrics,
        warnings,
    );
    let band = plot.width / count(spec.boxes.len());
    let thickness = if spec.box_display == BoxDisplay::Box {
        (band * 0.5).clamp(6.0, 48.0)
    } else {
        (band * 0.75).clamp(10.0, 130.0)
    };
    let ridges = ridge_geometry(spec, plot.width);
    let along = |value| scale.map(value, plot.top + plot.height, plot.top);
    let mut omitted = false;
    for (index, summary) in summaries.iter().enumerate() {
        // A ridge extends to the right of its baseline, the last one up to the plot's edge.
        let (center, across) = if let Some((step, rise)) = ridges {
            let baseline = plot.left + step * count(index);
            (baseline + step * 0.5, (baseline, rise))
        } else {
            let center = plot.left + band * (count(index) + 0.5);
            (center, (center - thickness / 2.0, thickness))
        };
        push_mark(
            &mut elements,
            spec,
            (index, summary),
            Orientation::Vertical,
            along,
            across,
            peak,
        );
        if spec.show_values {
            let content = format_value(summary.median, spec.number_style());
            if metrics.width(&content, LABEL_SIZE) <= band - 4.0 {
                elements.push(Element::Text(Text {
                    x: center,
                    y: along(reach(spec, index, summary).1) - 8.0,
                    class: "chartlet-value",
                    anchor: TextAnchor::Middle,
                    content,
                }));
            } else {
                omitted = true;
            }
        }
        push_category_label(
            &mut elements,
            &spec.boxes[index].label,
            (center, plot.top + plot.height + 24.0),
            (band - 8.0).max(20.0),
            metrics,
            warnings,
            &format!("/boxes/{index}/label"),
        );
    }
    warn_if_labels_omitted(omitted, warnings);
    add_bottom_category_title(
        spec,
        plot.left,
        plot.width,
        height,
        &mut elements,
        warnings,
        metrics,
    );
    elements
}

/// What the boxes show in sentences: how many and where their medians lie, and the outliers.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let show = |value| format_value(value, spec.number_style());
    let summaries = spec.box_summaries();
    let highest = (0..summaries.len())
        .max_by(|a, b| summaries[*a].median.total_cmp(&summaries[*b].median))
        .expect("validated boxplots have a box");
    let lowest = (0..summaries.len())
        .min_by(|a, b| summaries[*a].median.total_cmp(&summaries[*b].median))
        .expect("validated boxplots have a box");
    let outliers: Vec<&str> = (0..summaries.len())
        .filter(|index| !summaries[*index].outliers.is_empty())
        .map(|index| spec.boxes[index].label.as_str())
        .collect();
    let mut description = text::boxplot_summary(
        spec.locale,
        summaries.len(),
        (&show(summaries[highest].median), &spec.boxes[highest].label),
        (&show(summaries[lowest].median), &spec.boxes[lowest].label),
        &outliers,
    );
    match spec.box_display {
        BoxDisplay::Beeswarm | BoxDisplay::Ridgeline => {
            description.push_str(&text::box_display_note(
                spec.locale,
                spec.box_display == BoxDisplay::Ridgeline,
            ));
        }
        _ => {}
    }
    description
}

/// The outliers of every box as the table writes them: one cell per box.
pub(crate) fn outlier_cells(spec: &ChartSpec) -> Vec<String> {
    spec.box_summaries()
        .iter()
        .map(|summary| {
            summary
                .outliers
                .iter()
                .map(|value| format_value(*value, spec.number_style()))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .collect()
}
