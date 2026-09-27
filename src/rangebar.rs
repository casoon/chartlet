//! Range bars: one span per category from its low to its high value, with an optional central
//! estimate drawn across it. A modeled span is hatched, so that the difference between measured
//! and modeled does not rest on color.

use std::fmt::Write as _;

use crate::{
    error::ChartWarning,
    layout::{
        LABEL_SIZE, NumericScale, PlotArea, add_bottom_category_title, base_elements, count,
        fit_text, format_value, warn_if_labels_omitted,
    },
    metrics::TextMetrics,
    scene::{Element, Line, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, Orientation, RangeSpec},
};

/// Extra top margin for the legend that explains the hatching.
const LEGEND_HEIGHT: f64 = 24.0;
/// A span of zero width is still drawn this wide, so that it does not vanish.
const MIN_EXTENT: f64 = 2.0;
/// How far the central mark reaches beyond the bar on either side.
const MID_OVERHANG: f64 = 3.0;

/// The label of one span: its central value and its bounds, or only the bounds.
fn span_label(spec: &ChartSpec, range: &RangeSpec) -> String {
    let show = |value| format_value(value, spec.value_format());
    match range.mid {
        Some(mid) => format!(
            "{} ({} to {})",
            show(mid),
            show(range.low),
            show(range.high)
        ),
        None => format!("{} to {}", show(range.low), show(range.high)),
    }
}

fn tooltip(spec: &ChartSpec, range: &RangeSpec) -> String {
    let modeled = if range.modeled { ", modeled" } else { "" };
    format!("{}: {}{modeled}", range.label, span_label(spec, range))
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let values = spec
        .ranges
        .iter()
        .flat_map(|range| [Some(range.low), Some(range.high), range.mid])
        .flatten();
    let scale = NumericScale::from_values(values, false);
    let legend = spec.ranges.iter().any(|range| range.modeled);
    let top = 78.0 + if legend { LEGEND_HEIGHT } else { 0.0 };
    let mut elements = match spec.orientation {
        Orientation::Horizontal => layout_horizontal(spec, &scale, top, warnings, metrics),
        Orientation::Vertical => layout_vertical(spec, &scale, top, warnings, metrics),
    };
    if legend {
        push_legend(&mut elements, spec.orientation, spec, metrics);
    }
    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// One entry that says what the hatching means.
fn push_legend(
    elements: &mut Vec<Element>,
    orientation: Orientation,
    spec: &ChartSpec,
    metrics: &impl TextMetrics,
) {
    let x = match orientation {
        Orientation::Vertical => f64::from(crate::layout::AXIS_GUTTER),
        Orientation::Horizontal => horizontal_gutter(spec, metrics),
    };
    for class in ["chartlet-range", "chartlet-range-hatch"] {
        elements.push(Element::Rect(Rect {
            x,
            y: 46.0,
            width: 10.0,
            height: 10.0,
            class,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
    elements.push(Element::Text(Text {
        x: x + 16.0,
        y: 55.0,
        class: "chartlet-legend",
        anchor: TextAnchor::Start,
        content: "Hatched: modeled".to_owned(),
    }));
}

/// The bar of one span, its hatching if modeled, and its central mark. `along` maps a value onto
/// the value axis; `across` is the leading edge and thickness of the bar.
fn push_span(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    range: &RangeSpec,
    orientation: Orientation,
    along: impl Fn(f64) -> f64,
    across: (f64, f64),
) {
    let (low, high) = (along(range.low), along(range.high));
    let (start, extent) = (low.min(high), (high - low).abs().max(MIN_EXTENT));
    let rect = |class| {
        let (x, y, width, height) = match orientation {
            Orientation::Horizontal => (start, across.0, extent, across.1),
            Orientation::Vertical => (across.0, start, across.1, extent),
        };
        Element::Rect(Rect {
            x,
            y,
            width,
            height,
            class,
            series_index: None,
            style_index: None,
            tooltip: Some(tooltip(spec, range)),
        })
    };
    elements.push(rect("chartlet-range"));
    if range.modeled {
        elements.push(rect("chartlet-range-hatch"));
    }
    if let Some(mid) = range.mid {
        let position = along(mid);
        let (from, to) = (across.0 - MID_OVERHANG, across.0 + across.1 + MID_OVERHANG);
        elements.push(Element::Line(match orientation {
            Orientation::Horizontal => Line {
                x1: position,
                y1: from,
                x2: position,
                y2: to,
                class: "chartlet-range-mid",
            },
            Orientation::Vertical => Line {
                x1: from,
                y1: position,
                x2: to,
                y2: position,
                class: "chartlet-range-mid",
            },
        }));
    }
}

/// Room left of a horizontal plot for the longest category label.
fn horizontal_gutter(spec: &ChartSpec, metrics: &impl TextMetrics) -> f64 {
    let widest = spec
        .ranges
        .iter()
        .map(|range| metrics.width(&range.label, LABEL_SIZE))
        .fold(0.0, f64::max);
    (widest + 29.0).clamp(88.0, 210.0)
}

fn layout_horizontal(
    spec: &ChartSpec,
    scale: &NumericScale,
    top: f64,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = horizontal_gutter(spec, metrics);
    let right = if spec.show_values {
        spec.ranges
            .iter()
            .map(|range| metrics.width(&span_label(spec, range), LABEL_SIZE))
            .fold(0.0, f64::max)
            .clamp(8.0, 200.0)
            + 16.0
    } else {
        24.0
    };
    let bottom = if spec.value_axis.title.is_some() {
        56.0
    } else {
        36.0
    };
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: height - top - bottom,
        vertical_bars: false,
    };
    let mut elements = base_elements(spec, scale, plot, warnings, metrics);
    let band = plot.height / count(spec.ranges.len());
    let thickness = (band * 0.5).clamp(2.0, 28.0);
    let along = |value| scale.map(value, plot.left, plot.left + plot.width);

    for (index, range) in spec.ranges.iter().enumerate() {
        let center = plot.top + band * (count(index) + 0.5);
        push_span(
            &mut elements,
            spec,
            range,
            Orientation::Horizontal,
            along,
            (center - thickness / 2.0, thickness),
        );
        if spec.show_values {
            elements.push(Element::Text(Text {
                x: along(range.high).max(along(range.low)) + 8.0,
                y: center + 4.0,
                class: "chartlet-value",
                anchor: TextAnchor::Start,
                content: span_label(spec, range),
            }));
        }
        elements.push(Element::Text(Text {
            x: plot.left - 12.0,
            y: center + 4.0,
            class: "chartlet-label",
            anchor: TextAnchor::End,
            content: fit_text(
                &range.label,
                plot.left - 28.0,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/ranges/{index}/label"),
            ),
        }));
    }
    elements
}

fn layout_vertical(
    spec: &ChartSpec,
    scale: &NumericScale,
    top: f64,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = f64::from(crate::layout::AXIS_GUTTER);
    let right = f64::from(crate::layout::PLOT_MARGIN);
    let bottom = if spec.category_axis.title.is_some() {
        82.0
    } else {
        62.0
    };
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: height - top - bottom,
        vertical_bars: true,
    };
    let mut elements = base_elements(spec, scale, plot, warnings, metrics);
    let band = plot.width / count(spec.ranges.len());
    let thickness = (band * 0.5).clamp(2.0, 36.0);
    let along = |value| scale.map(value, plot.top + plot.height, plot.top);
    let mut omitted = false;

    for (index, range) in spec.ranges.iter().enumerate() {
        let center = plot.left + band * (count(index) + 0.5);
        push_span(
            &mut elements,
            spec,
            range,
            Orientation::Vertical,
            along,
            (center - thickness / 2.0, thickness),
        );
        if spec.show_values {
            let content = span_label(spec, range);
            if metrics.width(&content, LABEL_SIZE) <= band - 4.0 {
                elements.push(Element::Text(Text {
                    x: center,
                    y: along(range.high).min(along(range.low)) - 8.0,
                    class: "chartlet-value",
                    anchor: TextAnchor::Middle,
                    content,
                }));
            } else {
                omitted = true;
            }
        }
        elements.push(Element::Text(Text {
            x: center,
            y: plot.top + plot.height + 24.0,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content: fit_text(
                &range.label,
                (band - 8.0).max(20.0),
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/ranges/{index}/label"),
            ),
        }));
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

/// What the spans show in one sentence: how many, their overall reach, and which are modeled.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let show = |value| format_value(value, spec.value_format());
    let lowest = spec
        .ranges
        .iter()
        .min_by(|a, b| a.low.total_cmp(&b.low))
        .expect("validated rangebar charts have a range");
    let highest = spec
        .ranges
        .iter()
        .max_by(|a, b| a.high.total_cmp(&b.high))
        .expect("validated rangebar charts have a range");
    let categories = spec.ranges.len();
    let mid = if spec.ranges.iter().any(|range| range.mid.is_some()) {
        " and a central value"
    } else {
        ""
    };
    let mut description = format!(
        "Range chart with {categories} {}, each a span from low to high{mid}. Lowest low: {} ({}). Highest high: {} ({}).",
        if categories == 1 {
            "category"
        } else {
            "categories"
        },
        show(lowest.low),
        lowest.label,
        show(highest.high),
        highest.label,
    );
    let modeled: Vec<&str> = spec
        .ranges
        .iter()
        .filter(|range| range.modeled)
        .map(|range| range.label.as_str())
        .collect();
    if !modeled.is_empty() {
        write!(
            description,
            " Modeled, drawn hatched: {}.",
            modeled.join(", ")
        )
        .expect("writing to String cannot fail");
    }
    description
}
