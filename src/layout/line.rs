use super::{
    AXIS_GUTTER, LABEL_SIZE, PLOT_MARGIN, PlotArea,
    axis::{NumericScale, add_bottom_category_title, format_value},
    base_elements, fit_text,
    title::title_extra,
    tooltip,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Circle, Element, Hook, Polyline, Scene, Text, TextAnchor},
    spec::ChartSpec,
};

pub(super) fn layout_line(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = f64::from(AXIS_GUTTER);
    let right = f64::from(PLOT_MARGIN);
    let plot_width = width - left - right;
    let top = 78.0 + title_extra(spec, plot_width, metrics);
    let bottom = if spec.category_axis.title.is_some() {
        82.0
    } else {
        62.0
    };
    let plot_height = height - top - bottom;
    let scale = NumericScale::from_values(
        spec.data.iter().filter_map(|point| point.value),
        false,
        spec.value_axis.bounds(),
    );
    let mut elements = base_elements(
        spec,
        &scale,
        PlotArea {
            left,
            top,
            width: plot_width,
            height: plot_height,
            vertical_bars: true,
        },
        warnings,
        metrics,
    );
    elements.push(plot_hook(
        spec.data.len(),
        PlotArea {
            left,
            top,
            width: plot_width,
            height: plot_height,
            vertical_bars: true,
        },
        &scale,
    ));
    add_line_data(
        spec,
        PlotArea {
            left,
            top,
            width: plot_width,
            height: plot_height,
            vertical_bars: true,
        },
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

fn add_line_data(
    spec: &ChartSpec,
    plot: PlotArea,
    scale: &NumericScale,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let last_index = spec.data.len().saturating_sub(1);
    let divisor = f64::from(u32::try_from(last_index.max(1)).expect("data count is limited"));
    let spacing = if last_index == 0 {
        plot.width
    } else {
        plot.width / divisor
    };
    let mut segment = Vec::new();

    for (index, point) in spec.data.iter().enumerate() {
        let item_index = f64::from(u32::try_from(index).expect("data count is limited"));
        let x = if last_index == 0 {
            plot.left + plot.width / 2.0
        } else {
            plot.left + 6.0 + item_index / divisor * (plot.width - 12.0)
        };

        if let Some(value) = point.value {
            let y = scale.map(value, plot.top + plot.height, plot.top);
            segment.push((x, y));
            elements.push(Element::Circle(Circle {
                cx: x,
                cy: y,
                radius: 4.0,
                class: "chartlet-point",
                topic: None,
                series_index: None,
                style_index: None,
                tooltip: Some(tooltip(&point.label, value, spec.number_style(), None)),
            }));
            if spec.show_values {
                elements.push(Element::Text(Text {
                    x,
                    y: y - 10.0,
                    class: "chartlet-value",
                    anchor: TextAnchor::Middle,
                    content: format_value(value, spec.number_style()),
                }));
            }
        } else {
            flush_line_segment(elements, &mut segment);
        }

        let label = fit_text(
            &point.label,
            (spacing - 8.0).max(20.0),
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/data/{index}/label"),
        );
        elements.push(Element::Text(Text {
            x,
            y: plot.top + plot.height + 24.0,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content: label,
        }));
    }
    flush_line_segment(elements, &mut segment);
}

/// The plot hook of a line chart: category indices along the axis, as [`add_line_data`] places
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
    })
}

fn flush_line_segment(elements: &mut Vec<Element>, segment: &mut Vec<(f64, f64)>) {
    if segment.len() >= 2 {
        elements.push(Element::Polyline(Polyline {
            points: std::mem::take(segment),
            class: "chartlet-line",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    } else {
        segment.clear();
    }
}
