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
    scene::{Circle, Element, Line, Rect, Scene, Text, TextAnchor},
    spec::{BoxSummary, ChartSpec, Orientation},
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
    let values = summaries.iter().flat_map(|summary| {
        [summary.min, summary.max]
            .into_iter()
            .chain(summary.outliers.iter().copied())
    });
    let scale = NumericScale::for_axis(values, false, &spec.value_axis);
    let elements = match spec.orientation {
        Orientation::Horizontal => layout_horizontal(spec, &summaries, &scale, warnings, metrics),
        Orientation::Vertical => layout_vertical(spec, &summaries, &scale, warnings, metrics),
    };
    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
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
            "chartlet-box-whisker",
        ));
        elements.push(line(
            (along(end), middle - cap),
            (along(end), middle + cap),
            "chartlet-box-whisker",
        ));
    }
    elements.push(Element::Rect(Rect {
        x,
        y,
        width,
        height,
        class: "chartlet-box",
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
            class: "chartlet-box-outlier",
            topic: None,
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

/// The furthest value a box reaches, with its outliers.
fn reach(summary: &BoxSummary) -> (f64, f64) {
    summary
        .outliers
        .iter()
        .fold((summary.min, summary.max), |(low, high), value| {
            (low.min(*value), high.max(*value))
        })
}

fn layout_horizontal(
    spec: &ChartSpec,
    summaries: &[BoxSummary],
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
    let top = 78.0 + title_extra(spec, title.1, metrics);
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: crate::layout::plot_height(height - top - bottom, warnings),
        vertical_bars: false,
    };
    let mut elements = base_elements_with_title(spec, scale, plot, title, warnings, metrics);
    let band = plot.height / count(spec.boxes.len());
    let thickness = (band * 0.5).clamp(4.0, 32.0);
    let along = |value| scale.map(value, plot.left, plot.left + plot.width);
    for (index, summary) in summaries.iter().enumerate() {
        let center = plot.top + band * (count(index) + 0.5);
        push_box(
            &mut elements,
            spec,
            (index, summary),
            Orientation::Horizontal,
            along,
            (center - thickness / 2.0, thickness),
        );
        if spec.show_values {
            elements.push(Element::Text(Text {
                x: along(reach(summary).1) + 8.0,
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
    summaries: &[BoxSummary],
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
    let top = 78.0 + title_extra(spec, width - left - right, metrics);
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: crate::layout::plot_height(height - top - bottom, warnings),
        vertical_bars: true,
    };
    let mut elements = base_elements(spec, scale, plot, warnings, metrics);
    let band = plot.width / count(spec.boxes.len());
    let thickness = (band * 0.5).clamp(6.0, 48.0);
    let along = |value| scale.map(value, plot.top + plot.height, plot.top);
    let mut omitted = false;
    for (index, summary) in summaries.iter().enumerate() {
        let center = plot.left + band * (count(index) + 0.5);
        push_box(
            &mut elements,
            spec,
            (index, summary),
            Orientation::Vertical,
            along,
            (center - thickness / 2.0, thickness),
        );
        if spec.show_values {
            let content = format_value(summary.median, spec.number_style());
            if metrics.width(&content, LABEL_SIZE) <= band - 4.0 {
                elements.push(Element::Text(Text {
                    x: center,
                    y: along(reach(summary).1) - 8.0,
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
    text::boxplot_summary(
        spec.locale,
        summaries.len(),
        (&show(summaries[highest].median), &spec.boxes[highest].label),
        (&show(summaries[lowest].median), &spec.boxes[lowest].label),
        &outliers,
    )
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
