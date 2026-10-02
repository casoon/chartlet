use super::{
    AXIS_GUTTER, LABEL_SIZE, PLOT_MARGIN, PlotArea,
    axis::{NumericScale, add_bottom_category_title, format_value},
    base_elements, fit_text,
    legend::{add_line_legend, line_legend_space},
    title::title_extra,
    tooltip,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Circle, Element, Hook, Polyline, Scene, Text, TextAnchor},
    spec::{ChartSpec, Dataset, MAX_SERIES},
};

/// The lines of a categorical line chart with several series: each its own color and, so that
/// color is not the only difference, its own pattern or weight.
const SERIES_LINE_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-line chartlet-line-series-1",
    "chartlet-line chartlet-line-series-2 chartlet-line-dashed",
    "chartlet-line chartlet-line-series-3 chartlet-line-dotted",
    "chartlet-line chartlet-line-series-4 chartlet-line-thin",
];
/// The markers of those lines, in the colors of their lines.
const SERIES_POINT_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-point chartlet-point-series-1",
    "chartlet-point chartlet-point-series-2",
    "chartlet-point chartlet-point-series-3",
    "chartlet-point chartlet-point-series-4",
];

pub(super) fn layout_line(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let dataset = spec.dataset();
    let several = dataset.series.len() > 1;
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = f64::from(AXIS_GUTTER);
    let right = f64::from(PLOT_MARGIN);
    let plot_width = width - left - right;
    let head = title_extra(spec, plot_width, metrics);
    let legend = if several {
        line_legend_space(&dataset, plot_width, metrics)
    } else {
        0.0
    };
    let top = 78.0 + head + legend;
    let bottom = if spec.category_axis.title.is_some() {
        82.0
    } else {
        62.0
    };
    let plot = PlotArea {
        left,
        top,
        width: plot_width,
        height: height - top - bottom,
        vertical_bars: true,
    };
    let scale = NumericScale::for_axis(dataset.values(), false, &spec.value_axis);
    let mut elements = base_elements(spec, &scale, plot, warnings, metrics);
    elements.push(plot_hook(dataset.categories.len(), plot, &scale));
    if several {
        add_line_legend(
            &dataset,
            &SERIES_LINE_CLASSES,
            plot,
            head,
            &mut elements,
            warnings,
            metrics,
        );
    }
    add_line_data(
        spec,
        &dataset,
        plot,
        &scale,
        &mut elements,
        warnings,
        metrics,
    );

    add_bottom_category_title(
        spec,
        left,
        plot_width,
        height,
        &mut elements,
        warnings,
        metrics,
    );

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// The horizontal position of category `index` of `categories`, evenly spread across the plot.
fn category_x(plot: PlotArea, index: usize, categories: usize) -> f64 {
    let last_index = categories.saturating_sub(1);
    if last_index == 0 {
        return plot.left + plot.width / 2.0;
    }
    let divisor = f64::from(u32::try_from(last_index).expect("data count is limited"));
    let item_index = f64::from(u32::try_from(index).expect("data count is limited"));
    plot.left + 6.0 + item_index / divisor * (plot.width - 12.0)
}

/// The lines with their markers, then the category labels. Value labels are drawn for a single
/// line only; with several, they would collide, and the tooltips and the data table carry the
/// values.
fn add_line_data(
    spec: &ChartSpec,
    dataset: &Dataset,
    plot: PlotArea,
    scale: &NumericScale,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let categories = dataset.categories.len();
    let several = dataset.series.len() > 1;
    let spacing = if categories > 1 {
        plot.width / f64::from(u32::try_from(categories - 1).expect("data count is limited"))
    } else {
        plot.width
    };
    let mut category_label = |elements: &mut Vec<Element>, index: usize| {
        let label = fit_text(
            &dataset.categories[index],
            (spacing - 8.0).max(20.0),
            LABEL_SIZE,
            metrics,
            warnings,
            &dataset.category_path(index),
        );
        elements.push(Element::Text(Text {
            x: category_x(plot, index, categories),
            y: plot.top + plot.height + 24.0,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content: label,
        }));
    };
    for (series_index, series) in dataset.series.iter().enumerate() {
        let (line_class, point_class) = if several {
            (
                SERIES_LINE_CLASSES[series_index],
                SERIES_POINT_CLASSES[series_index],
            )
        } else {
            ("chartlet-line", "chartlet-point")
        };
        let series_mark = several.then_some(series_index);
        let mut segment = Vec::new();
        for (index, value) in series.values.iter().enumerate() {
            let x = category_x(plot, index, categories);
            let Some(value) = *value else {
                flush_line_segment(elements, &mut segment, line_class, series_mark);
                // A single line keeps each category label beside its point.
                if !several {
                    category_label(elements, index);
                }
                continue;
            };
            let y = scale.map(value, plot.top + plot.height, plot.top);
            segment.push((x, y));
            elements.push(Element::Circle(Circle {
                cx: x,
                cy: y,
                radius: 4.0,
                class: point_class,
                topic: None,
                series_index: series_mark,
                style_index: None,
                tooltip: Some(tooltip(
                    &dataset.categories[index],
                    value,
                    spec.number_style(),
                    series.name.as_deref(),
                )),
            }));
            if spec.show_values && !several {
                elements.push(Element::Text(Text {
                    x,
                    y: y - 10.0,
                    class: "chartlet-value",
                    anchor: TextAnchor::Middle,
                    content: format_value(value, spec.number_style()),
                }));
            }
            if !several {
                category_label(elements, index);
            }
        }
        flush_line_segment(elements, &mut segment, line_class, series_mark);
    }
    if several {
        for index in 0..categories {
            category_label(elements, index);
        }
    }
}

/// The plot hook of a line chart: category indices along the axis, as [`category_x`] places
/// them, and both ends of the value scale.
fn plot_hook(count: usize, plot: PlotArea, scale: &NumericScale) -> Element {
    let last = f64::from(u32::try_from(count.saturating_sub(1)).expect("data count is limited"));
    let x = if count <= 1 {
        vec![(0.0, plot.left + plot.width / 2.0)]
    } else {
        vec![(0.0, plot.left + 6.0), (last, plot.left + plot.width - 6.0)]
    };
    let (min, max) = scale.ends();
    let y = |value| scale.map(value, plot.top + plot.height, plot.top);
    Element::Hook(Hook::Plot {
        pane: 0,
        x,
        y: [(min, y(min)), (max, y(max))],
        log: scale.is_log(),
    })
}

fn flush_line_segment(
    elements: &mut Vec<Element>,
    segment: &mut Vec<(f64, f64)>,
    class: &'static str,
    series_index: Option<usize>,
) {
    if segment.len() >= 2 {
        elements.push(Element::Polyline(Polyline {
            points: std::mem::take(segment),
            class,
            topic: None,
            series_index,
            style_index: None,
            tooltip: None,
        }));
    } else {
        segment.clear();
    }
}
