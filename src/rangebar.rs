//! Range bars: one span per category from its low to its high value, with an optional central
//! estimate drawn across it. A modeled span is hatched, so that the difference between measured
//! and modeled does not rest on color. Spans may belong to groups, each in a palette color with
//! an entry in the legend.

use crate::{
    error::ChartWarning,
    layout::{
        CONTENT_LEFT, LABEL_SIZE, NumericScale, PlotArea, WithReserve, add_bottom_category_title,
        base_elements, base_elements_with_title, count, fit_text, format_value, horizontal_title,
        push_category_label, push_side_label, title_extra, warn_if_labels_omitted,
    },
    metrics::TextMetrics,
    scene::{Element, Line, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, Orientation, RangeSpec},
    text,
};

/// The span of each group, in the palette colors.
const GROUP_CLASSES: [&str; crate::spec::MAX_SERIES] = [
    "chartlet-range chartlet-range-series-1",
    "chartlet-range chartlet-range-series-2",
    "chartlet-range chartlet-range-series-3",
    "chartlet-range chartlet-range-series-4",
];
/// Gap between two legend entries.
const LEGEND_GAP: f64 = 16.0;

/// Extra top margin for the legend that explains the groups and the hatching.
const LEGEND_HEIGHT: f64 = 24.0;
/// Width of the legend swatch with the space before its text.
const LEGEND_SWATCH: f64 = 16.0;
/// A span of zero width is still drawn this wide, so that it does not vanish.
const MIN_EXTENT: f64 = 2.0;
/// How far the central mark reaches beyond the bar on either side.
const MID_OVERHANG: f64 = 3.0;

/// The label of one span: its central value and its bounds, or only the bounds.
fn span_label(spec: &ChartSpec, range: &RangeSpec) -> String {
    let show = |value| format_value(value, spec.number_style());
    let to = spec.locale.words().to;
    // A span of one value is that value, not "3 to 3".
    if range.low.total_cmp(&range.high).is_eq()
        && range
            .mid
            .is_none_or(|mid| mid.total_cmp(&range.low).is_eq())
    {
        return show(range.low);
    }
    match range.mid {
        Some(mid) => format!(
            "{} ({} {to} {})",
            show(mid),
            show(range.low),
            show(range.high)
        ),
        None => format!("{} {to} {}", show(range.low), show(range.high)),
    }
}

fn tooltip(spec: &ChartSpec, range: &RangeSpec) -> String {
    let modeled = if range.modeled {
        format!(", {}", spec.locale.words().modeled)
    } else {
        String::new()
    };
    let group = range
        .group
        .as_ref()
        .map(|group| format!(" ({group})"))
        .unwrap_or_default();
    format!(
        "{}{group}: {}{modeled}",
        range.label,
        span_label(spec, range)
    )
}

/// The class of a span: its group's palette color, or the accent color without groups.
fn range_class(spec: &ChartSpec, range: &RangeSpec) -> &'static str {
    range.group.as_deref().map_or("chartlet-range", |group| {
        let index = spec
            .range_groups()
            .iter()
            .position(|known| *known == group)
            .expect("every group is among the chart's groups");
        GROUP_CLASSES[index]
    })
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
    let scale = NumericScale::for_axis(values, false, &spec.value_axis);
    let elements = match spec.orientation {
        Orientation::Horizontal => layout_horizontal(spec, &scale, warnings, metrics),
        Orientation::Vertical => layout_vertical(spec, &scale, warnings, metrics),
    };
    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// Extra top margin for each further row of the legend.
const LEGEND_ROW: f64 = 20.0;

/// The top of the plot: below the title, which may take two lines of up to `title_width`, and
/// below the legend of the groups and of the hatching, if there is one.
fn plot_top(spec: &ChartSpec, title_width: f64, left: f64, metrics: &impl TextMetrics) -> f64 {
    let legend = match legend_layout(spec, left, metrics)
        .iter()
        .map(|(_, _, row)| *row)
        .max()
    {
        Some(rows) => LEGEND_HEIGHT + LEGEND_ROW * count(rows),
        None => 0.0,
    };
    78.0 + title_extra(spec, title_width, metrics) + legend
}

/// The legend entries: one per group, then one that says what the hatching means if any span is
/// modeled; each with its swatch classes, its text and the path its text comes from.
fn legend_entries(spec: &ChartSpec) -> Vec<(&'static [&'static str], String, String)> {
    let mut entries: Vec<(&'static [&'static str], String, String)> = spec
        .range_groups()
        .into_iter()
        .enumerate()
        .map(|(index, group)| {
            let first = spec
                .ranges
                .iter()
                .position(|range| range.group.as_deref() == Some(group))
                .expect("a group comes from a range");
            (
                std::slice::from_ref(&GROUP_CLASSES[index]),
                group.to_owned(),
                format!("/ranges/{first}/group"),
            )
        })
        .collect();
    if spec.ranges.iter().any(|range| range.modeled) {
        entries.push((
            &["chartlet-range", "chartlet-range-hatch"],
            spec.locale.words().hatched_modeled.to_owned(),
            "/locale".to_owned(),
        ));
    }
    entries
}

/// Where each legend entry goes, as its index, its left edge and its row. The legend starts at
/// `left`, the left of the plot, unless its text, measured with the fallback reserve, would then
/// reach into the chart's right margin, as it does after a wide gutter of category labels on a
/// narrow chart; it then starts at the left edge of the content, and an entry that does not fit
/// beside the one before it starts a new row.
fn legend_layout(
    spec: &ChartSpec,
    left: f64,
    metrics: &impl TextMetrics,
) -> Vec<(usize, f64, usize)> {
    let entries = legend_entries(spec);
    let right = legend_right(spec);
    let widths: Vec<f64> = entries
        .iter()
        .map(|(_, text, _)| LEGEND_SWATCH + WithReserve(metrics).width(text, LABEL_SIZE))
        .collect();
    let total = widths.iter().sum::<f64>() + LEGEND_GAP * count(widths.len().saturating_sub(1));
    let start = if left + total <= right {
        left
    } else {
        CONTENT_LEFT
    };
    let (mut x, mut row) = (start, 0);
    let mut placed = Vec::new();
    for (index, width) in widths.into_iter().enumerate() {
        if x > start && x + width > right {
            x = start;
            row += 1;
        }
        placed.push((index, x, row));
        x += width + LEGEND_GAP;
    }
    placed
}

/// The right edge the legend keeps to: the chart's right margin.
fn legend_right(spec: &ChartSpec) -> f64 {
    f64::from(spec.width) - f64::from(crate::layout::plot_margin(spec.width))
}

/// The legend below a title up to `title_width` wide, as [`legend_layout`] places it. An entry is
/// shortened only if it does not fit even on a row of its own.
fn push_legend(
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    spec: &ChartSpec,
    (left, title_width): (f64, f64),
    metrics: &impl TextMetrics,
) {
    let entries = legend_entries(spec);
    let right = legend_right(spec);
    let top = 46.0 + title_extra(spec, title_width, metrics);
    for (index, x, row) in legend_layout(spec, left, metrics) {
        let (classes, text, path) = &entries[index];
        let y = top + LEGEND_ROW * count(row);
        let content = fit_text(
            text,
            (right - x - LEGEND_SWATCH).max(0.0),
            LABEL_SIZE,
            metrics,
            warnings,
            path,
        );
        for class in *classes {
            elements.push(Element::Rect(Rect {
                x,
                y,
                width: 10.0,
                height: 10.0,
                class,
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
        elements.push(Element::Text(Text {
            x: x + LEGEND_SWATCH,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content,
        }));
    }
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
    elements.push(rect(range_class(spec, range)));
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
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = horizontal_gutter(spec, metrics);
    let right = if spec.show_values {
        spec.ranges
            .iter()
            .map(|range| WithReserve(metrics).width(&span_label(spec, range), LABEL_SIZE))
            .fold(0.0, f64::max)
            .clamp(8.0, 200.0)
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
    let top = plot_top(spec, title.1, left, metrics);
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: crate::layout::plot_height(height - top - bottom, warnings),
        vertical_bars: false,
    };
    let mut elements = base_elements_with_title(spec, scale, plot, title, warnings, metrics);
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
        push_side_label(
            &mut elements,
            &range.label,
            (plot.left - 12.0, center),
            (plot.left - 28.0, band),
            metrics,
            warnings,
            &format!("/ranges/{index}/label"),
        );
    }
    push_legend(&mut elements, warnings, spec, (plot.left, title.1), metrics);
    elements
}

fn layout_vertical(
    spec: &ChartSpec,
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
    let top = plot_top(spec, width - left - right, left, metrics);
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: crate::layout::plot_height(height - top - bottom, warnings),
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
        push_category_label(
            &mut elements,
            &range.label,
            (center, plot.top + plot.height + 24.0),
            (band - 8.0).max(20.0),
            metrics,
            warnings,
            &format!("/ranges/{index}/label"),
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
    push_legend(
        &mut elements,
        warnings,
        spec,
        (plot.left, plot.width),
        metrics,
    );
    elements
}

/// What the spans show in one sentence: how many, their overall reach, and which are modeled.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let show = |value| format_value(value, spec.number_style());
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
    let mut description = text::rangebar_opening(
        spec.locale,
        spec.ranges.len(),
        spec.ranges.iter().any(|range| range.mid.is_some()),
        (&show(lowest.low), &lowest.label),
        (&show(highest.high), &highest.label),
    );
    let modeled: Vec<&str> = spec
        .ranges
        .iter()
        .filter(|range| range.modeled)
        .map(|range| range.label.as_str())
        .collect();
    let groups: Vec<String> = spec
        .range_groups()
        .into_iter()
        .map(|group| {
            let labels: Vec<&str> = spec
                .ranges
                .iter()
                .filter(|range| range.group.as_deref() == Some(group))
                .map(|range| range.label.as_str())
                .collect();
            format!("{group}: {}", labels.join(", "))
        })
        .collect();
    if !groups.is_empty() {
        description.push_str(&text::range_groups(spec.locale, &groups.join("; ")));
    }
    if !modeled.is_empty() {
        description.push_str(&text::modeled_ranges(spec.locale, &modeled.join(", ")));
    }
    description
}
