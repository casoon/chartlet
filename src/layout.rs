use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Polyline, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, ChartType, Dataset, MAX_SERIES, Orientation, ValueFormat},
};

const LABEL_SIZE: f64 = 12.0;
/// Extra top margin that makes room for the legend of a multi-series chart.
const LEGEND_HEIGHT: f64 = 24.0;
const SERIES_BAR_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-bar chartlet-series-1",
    "chartlet-bar chartlet-series-2",
    "chartlet-bar chartlet-series-3",
    "chartlet-bar chartlet-series-4",
];

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let dataset = spec.dataset();
    match (spec.chart_type, spec.orientation) {
        (ChartType::Bar, Orientation::Vertical) => {
            layout_vertical(spec, &dataset, warnings, metrics)
        }
        (ChartType::Bar, Orientation::Horizontal) => {
            layout_horizontal(spec, &dataset, warnings, metrics)
        }
        (ChartType::Line, _) => layout_line(spec, warnings, metrics),
    }
}

fn layout_vertical(
    spec: &ChartSpec,
    dataset: &Dataset<'_>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = 72.0;
    let right = 24.0;
    let top = 78.0 + legend_space(dataset);
    let bottom = if spec.category_axis.title.is_some() {
        82.0
    } else {
        62.0
    };
    let plot_width = width - left - right;
    let plot_height = height - top - bottom;
    let scale = NumericScale::from_values(dataset.values(), true);
    let baseline = scale.map(0.0, top + plot_height, top);
    let band = plot_width / count(dataset.categories.len());
    let group = Group::new(band, dataset.series.len());
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
    add_legend(dataset, left, plot_width, &mut elements, warnings, metrics);
    let mut labels_omitted = false;

    for (index, category) in dataset.categories.iter().enumerate() {
        let center = left + band * (count(index) + 0.5);
        for (series_index, series) in dataset.series.iter().enumerate() {
            let Some(value) = series.values[index] else {
                continue;
            };
            let (offset, thickness) = group.slot(series_index);
            let x = center - group.width / 2.0 + offset;
            let value_y = scale.map(value, top + plot_height, top);
            if value != 0.0 {
                elements.push(Element::Rect(Rect {
                    x,
                    y: baseline.min(value_y),
                    width: thickness,
                    height: (baseline - value_y).abs(),
                    class: bar_class(dataset, series_index),
                    series_index: (dataset.series.len() > 1).then_some(series_index),
                    tooltip: Some(tooltip(
                        category,
                        value,
                        spec.value_axis.format,
                        series.name,
                    )),
                }));
            }

            if spec.show_values {
                let content = format_value(value, spec.value_axis.format);
                if group.fits(metrics.width(&content, LABEL_SIZE)) {
                    let label = vertical_value_label(value, x + thickness / 2.0, value_y, content);
                    elements.push(series_text(label, dataset, series_index));
                } else {
                    labels_omitted = true;
                }
            }
        }

        let max_label_width = (band - 8.0).max(20.0);
        let label = fit_text(
            category,
            max_label_width,
            LABEL_SIZE,
            metrics,
            warnings,
            &dataset.category_path(index),
        );
        elements.push(Element::Text(Text {
            x: center,
            y: top + plot_height + 24.0,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content: label,
        }));
    }
    warn_if_labels_omitted(labels_omitted, warnings);

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

fn layout_horizontal(
    spec: &ChartSpec,
    dataset: &Dataset<'_>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let measured_label = dataset
        .categories
        .iter()
        .map(|category| metrics.width(category, LABEL_SIZE))
        .fold(0.0, f64::max);
    let left = (measured_label + 28.0).clamp(88.0, 210.0);
    let right = 68.0;
    let top = 78.0 + legend_space(dataset);
    let bottom = if spec.value_axis.title.is_some() {
        56.0
    } else {
        36.0
    };
    let plot_width = width - left - right;
    let plot_height = height - top - bottom;
    let scale = NumericScale::from_values(dataset.values(), true);
    let baseline = scale.map(0.0, left, left + plot_width);
    let band = plot_height / count(dataset.categories.len());
    let group = Group::new(band, dataset.series.len());
    let mut elements = base_elements(
        spec,
        &scale,
        PlotArea {
            left,
            top,
            width: plot_width,
            height: plot_height,
            vertical_bars: false,
        },
        warnings,
        metrics,
    );
    add_legend(dataset, left, plot_width, &mut elements, warnings, metrics);
    let mut labels_omitted = false;

    for (index, category) in dataset.categories.iter().enumerate() {
        let center = top + band * (count(index) + 0.5);
        for (series_index, series) in dataset.series.iter().enumerate() {
            let Some(value) = series.values[index] else {
                continue;
            };
            let (offset, thickness) = group.slot(series_index);
            let y = center - group.width / 2.0 + offset;
            let value_x = scale.map(value, left, left + plot_width);
            if value != 0.0 {
                elements.push(Element::Rect(Rect {
                    x: baseline.min(value_x),
                    y,
                    width: (baseline - value_x).abs(),
                    height: thickness,
                    class: bar_class(dataset, series_index),
                    series_index: (dataset.series.len() > 1).then_some(series_index),
                    tooltip: Some(tooltip(
                        category,
                        value,
                        spec.value_axis.format,
                        series.name,
                    )),
                }));
            }

            if spec.show_values {
                if group.fits(LABEL_SIZE + 2.0) {
                    let label = horizontal_value_label(
                        value,
                        value_x,
                        baseline,
                        y + thickness / 2.0,
                        spec.value_axis.format,
                        metrics,
                    );
                    elements.push(series_text(label, dataset, series_index));
                } else {
                    labels_omitted = true;
                }
            }
        }

        let label = fit_text(
            category,
            left - 28.0,
            LABEL_SIZE,
            metrics,
            warnings,
            &dataset.category_path(index),
        );
        elements.push(Element::Text(Text {
            x: left - 12.0,
            y: center + 4.0,
            class: "chartlet-label",
            anchor: TextAnchor::End,
            content: label,
        }));
    }
    warn_if_labels_omitted(labels_omitted, warnings);

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

fn layout_line(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = 72.0;
    let right = 24.0;
    let top = 78.0;
    let bottom = if spec.category_axis.title.is_some() {
        82.0
    } else {
        62.0
    };
    let plot_width = width - left - right;
    let plot_height = height - top - bottom;
    let scale = NumericScale::from_values(spec.data.iter().filter_map(|point| point.value), false);
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
                series_index: None,
                tooltip: Some(tooltip(&point.label, value, spec.value_axis.format, None)),
            }));
            if spec.show_values {
                elements.push(Element::Text(Text {
                    x,
                    y: y - 10.0,
                    class: "chartlet-value",
                    anchor: TextAnchor::Middle,
                    content: format_value(value, spec.value_axis.format),
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

fn flush_line_segment(elements: &mut Vec<Element>, segment: &mut Vec<(f64, f64)>) {
    if segment.len() >= 2 {
        elements.push(Element::Polyline(Polyline {
            points: std::mem::take(segment),
            class: "chartlet-line",
            series_index: None,
            tooltip: None,
        }));
    } else {
        segment.clear();
    }
}

fn base_elements(
    spec: &ChartSpec,
    scale: &NumericScale,
    plot: PlotArea,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    let title = fit_text(&spec.title, plot.width, 22.0, metrics, warnings, "/title");
    let mut elements = vec![Element::Text(Text {
        x: plot.left,
        y: 30.0,
        class: "chartlet-title",
        anchor: TextAnchor::Start,
        content: title,
    })];

    for value in scale.ticks() {
        if plot.vertical_bars {
            let y = scale.map(value, plot.top + plot.height, plot.top);
            elements.push(Element::Line(Line {
                x1: plot.left,
                y1: y,
                x2: plot.left + plot.width,
                y2: y,
                class: if value.abs() < scale.step / 100.0 {
                    "chartlet-zero"
                } else {
                    "chartlet-grid"
                },
            }));
            elements.push(Element::Text(Text {
                x: plot.left - 10.0,
                y: y + 4.0,
                class: "chartlet-tick",
                anchor: TextAnchor::End,
                content: format_value(value, spec.value_axis.format),
            }));
        } else {
            let x = scale.map(value, plot.left, plot.left + plot.width);
            elements.push(Element::Line(Line {
                x1: x,
                y1: plot.top,
                x2: x,
                y2: plot.top + plot.height,
                class: if value.abs() < scale.step / 100.0 {
                    "chartlet-zero"
                } else {
                    "chartlet-grid"
                },
            }));
            elements.push(Element::Text(Text {
                x,
                y: plot.top + plot.height + 22.0,
                class: "chartlet-tick",
                anchor: TextAnchor::Middle,
                content: format_value(value, spec.value_axis.format),
            }));
        }
    }

    if let Some(title) = &spec.value_axis.title {
        let title = fit_text(
            title,
            plot.width,
            LABEL_SIZE,
            metrics,
            warnings,
            "/valueAxis/title",
        );
        if plot.vertical_bars {
            elements.push(Element::Text(Text {
                x: plot.left,
                y: plot.top - 20.0,
                class: "chartlet-axis-title",
                anchor: TextAnchor::Start,
                content: title.clone(),
            }));
        } else {
            elements.push(Element::Text(Text {
                x: plot.left + plot.width / 2.0,
                y: plot.top + plot.height + 44.0,
                class: "chartlet-axis-title",
                anchor: TextAnchor::Middle,
                content: title,
            }));
        }
    }

    if !plot.vertical_bars
        && let Some(title) = &spec.category_axis.title
    {
        let title = fit_text(
            title,
            plot.left - 24.0,
            LABEL_SIZE,
            metrics,
            warnings,
            "/categoryAxis/title",
        );
        elements.push(Element::Text(Text {
            x: plot.left - 12.0,
            y: plot.top - 20.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::End,
            content: title,
        }));
    }

    elements
}

/// Centers the category axis title below a plot whose categories run horizontally.
fn add_bottom_category_title(
    spec: &ChartSpec,
    left: f64,
    plot_width: f64,
    chart_height: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    if let Some(title) = &spec.category_axis.title {
        let title = fit_text(
            title,
            plot_width,
            LABEL_SIZE,
            metrics,
            warnings,
            "/categoryAxis/title",
        );
        elements.push(Element::Text(Text {
            x: left + plot_width / 2.0,
            y: chart_height - 14.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: title,
        }));
    }
}

/// Places a value label above a positive bar or below a negative one.
fn vertical_value_label(value: f64, center_x: f64, value_y: f64, content: String) -> Text {
    Text {
        x: center_x,
        y: if value >= 0.0 {
            value_y - 8.0
        } else {
            value_y + 16.0
        },
        class: "chartlet-value",
        anchor: TextAnchor::Middle,
        content,
    }
}

fn horizontal_value_label(
    value: f64,
    value_x: f64,
    baseline: f64,
    center_y: f64,
    format: ValueFormat,
    metrics: &impl TextMetrics,
) -> Text {
    let content = format_value(value, format);
    // Inverse text is only readable on the bar itself; a short negative bar gets its label
    // outside, left of the bar end.
    let fits_inside = metrics.width(&content, LABEL_SIZE) + 16.0 <= (baseline - value_x).abs();
    let (x, class, anchor) = if value >= 0.0 {
        (value_x + 8.0, "chartlet-value", TextAnchor::Start)
    } else if fits_inside {
        (value_x + 8.0, "chartlet-value-inverse", TextAnchor::Start)
    } else {
        (value_x - 8.0, "chartlet-value", TextAnchor::End)
    };
    Text {
        x,
        y: center_y + 4.0,
        class,
        anchor,
        content,
    }
}

fn series_text(text: Text, dataset: &Dataset<'_>, series_index: usize) -> Element {
    if dataset.series.len() > 1 {
        Element::SeriesText(text, series_index)
    } else {
        Element::Text(text)
    }
}

/// The bars of one category: a single bar, or one slot per series side by side.
#[derive(Debug, Clone, Copy)]
struct Group {
    width: f64,
    slot: f64,
    gap: f64,
    single: bool,
}

impl Group {
    fn new(band: f64, series_count: usize) -> Self {
        if series_count == 1 {
            let width = (band * 0.64).max(2.0);
            return Self {
                width,
                slot: width,
                gap: 0.0,
                single: true,
            };
        }
        let series = count(series_count);
        let width = (band * 0.8).max(2.0 * series);
        let slot = width / series;
        Self {
            width,
            slot,
            gap: (slot * 0.15).min(4.0),
            single: false,
        }
    }

    /// Offset from the group's leading edge and drawn thickness of one series' bar.
    fn slot(self, series_index: usize) -> (f64, f64) {
        (
            count(series_index) * self.slot + self.gap / 2.0,
            self.slot - self.gap,
        )
    }

    /// Whether a value label of the given extent fits without reaching into the neighbouring
    /// series. A single bar always keeps its label.
    fn fits(self, extent: f64) -> bool {
        self.single || extent <= self.slot
    }
}

fn legend_space(dataset: &Dataset<'_>) -> f64 {
    if dataset.series.len() > 1 {
        LEGEND_HEIGHT
    } else {
        0.0
    }
}

fn add_legend(
    dataset: &Dataset<'_>,
    left: f64,
    available_width: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    if dataset.series.len() < 2 {
        return;
    }
    let entry_width = available_width / count(dataset.series.len());
    let mut x = left;
    for (index, series) in dataset.series.iter().enumerate() {
        let name = series.name.expect("multi-series charts name every series");
        elements.push(Element::Rect(Rect {
            x,
            y: 46.0,
            width: 10.0,
            height: 10.0,
            class: SERIES_BAR_CLASSES[index],
            series_index: None,
            tooltip: None,
        }));
        let label = fit_text(
            name,
            entry_width - 36.0,
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/series/{index}/name"),
        );
        let label_width = metrics.width(&label, LABEL_SIZE);
        elements.push(Element::Text(Text {
            x: x + 16.0,
            y: 55.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: label,
        }));
        x += 16.0 + label_width + 20.0;
    }
}

fn bar_class(dataset: &Dataset<'_>, series_index: usize) -> &'static str {
    if dataset.series.len() == 1 {
        "chartlet-bar"
    } else {
        SERIES_BAR_CLASSES[series_index]
    }
}

fn tooltip(category: &str, value: f64, format: ValueFormat, series_name: Option<&str>) -> String {
    let formatted = format_value(value, format);
    match series_name {
        Some(name) => format!("{category} – {name}: {formatted}"),
        None => format!("{category}: {formatted}"),
    }
}

fn warn_if_labels_omitted(omitted: bool, warnings: &mut Vec<ChartWarning>) {
    if omitted {
        warnings.push(ChartWarning::new(
            "value_labels_omitted",
            "/showValues",
            "some value labels do not fit next to their bars and were left out; every value remains in the HTML data alternative",
        ));
    }
}

fn count(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("counts are limited by validation"))
}

#[derive(Debug, Clone, Copy)]
struct PlotArea {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    vertical_bars: bool,
}

fn fit_text(
    text: &str,
    max_width: f64,
    font_size: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) -> String {
    if metrics.width(text, font_size) <= max_width {
        return text.to_owned();
    }

    let ellipsis_width = metrics.width("…", font_size);
    let mut output = String::new();
    for character in text.chars() {
        let candidate = format!("{output}{character}");
        if metrics.width(&candidate, font_size) + ellipsis_width > max_width {
            break;
        }
        output.push(character);
    }
    output.push('…');

    if !warnings
        .iter()
        .any(|warning| warning.code == "text_truncated" && warning.path == path)
    {
        warnings.push(ChartWarning::new(
            "text_truncated",
            path,
            "text was shortened in the visual chart; the full value remains in the specification and HTML data alternative",
        ));
    }
    output
}

pub(crate) fn format_value(value: f64, format: ValueFormat) -> String {
    let value = match format {
        ValueFormat::Number => value,
        ValueFormat::Percent => value * 100.0,
    };
    let suffix = if format == ValueFormat::Percent {
        "%"
    } else {
        ""
    };
    format!("{}{suffix}", tidy(value))
}

/// Removes binary floating-point noise (`0.07 * 100`, `0.1 + 0.2`) by keeping 12 significant
/// digits, and normalizes `-0.0`. Rust's float formatting does not depend on the platform, so
/// the result stays deterministic.
fn tidy(value: f64) -> f64 {
    let rounded: f64 = format!("{value:.11e}")
        .parse()
        .expect("a formatted float parses back");
    if rounded == 0.0 { 0.0 } else { rounded }
}

#[derive(Debug, Clone, Copy)]
struct NumericScale {
    min: f64,
    max: f64,
    step: f64,
}

impl NumericScale {
    fn from_values(mut values: impl Iterator<Item = f64>, include_zero: bool) -> Self {
        let first = values.next().expect("validated charts contain values");
        let mut min = if include_zero { first.min(0.0) } else { first };
        let mut max = if include_zero { first.max(0.0) } else { first };
        for value in values {
            min = min.min(value);
            max = max.max(value);
        }
        if (max - min).abs() < f64::EPSILON {
            if max.abs() < f64::EPSILON {
                min = -1.0;
                max = 1.0;
            } else {
                let padding = max.abs() * 0.2;
                min -= padding;
                max += padding;
            }
        } else if !include_zero {
            let padding = (max - min) * 0.05;
            min -= padding;
            max += padding;
        }
        let raw_step = (max - min) / 5.0;
        let magnitude = 10.0_f64.powf(raw_step.log10().floor());
        let fraction = raw_step / magnitude;
        let nice_fraction = if fraction <= 1.0 {
            1.0
        } else if fraction <= 2.0 {
            2.0
        } else if fraction <= 5.0 {
            5.0
        } else {
            10.0
        };
        let step = nice_fraction * magnitude;
        Self {
            min: tidy(tidy(min / step).floor() * step),
            max: tidy(tidy(max / step).ceil() * step),
            step,
        }
    }

    fn map(self, value: f64, output_min: f64, output_max: f64) -> f64 {
        let ratio = (value - self.min) / (self.max - self.min);
        output_min + ratio * (output_max - output_min)
    }

    /// Ticks are computed from their index instead of by repeated addition, so rounding errors
    /// do not accumulate along the axis.
    fn ticks(self) -> impl Iterator<Item = f64> {
        std::iter::successors(Some(0.0_f64), |index| Some(index + 1.0))
            .map(move |index| tidy(self.min + index * self.step))
            .take_while(move |tick| *tick <= self.max + self.step / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{NumericScale, format_value};
    use crate::spec::ValueFormat;

    #[test]
    fn scale_includes_zero_and_uses_nice_ticks() {
        let scale = NumericScale::from_values([12.0, 18.0, 15.0].into_iter(), true);
        assert!(scale.min.abs() < f64::EPSILON);
        assert!((scale.max - 20.0).abs() < f64::EPSILON);
        assert_eq!(
            scale.ticks().collect::<Vec<_>>(),
            vec![0.0, 5.0, 10.0, 15.0, 20.0]
        );
    }

    #[test]
    fn formats_percent_without_duplicate_suffix() {
        assert_eq!(format_value(0.125, ValueFormat::Percent), "12.5%");
        assert_eq!(format_value(1.0, ValueFormat::Percent), "100%");
    }

    #[test]
    fn removes_floating_point_noise_from_labels() {
        assert_eq!(format_value(0.07, ValueFormat::Percent), "7%");
        assert_eq!(format_value(0.1 + 0.2, ValueFormat::Number), "0.3");
        assert_eq!(format_value(-0.0, ValueFormat::Number), "0");
    }

    #[test]
    fn ticks_do_not_accumulate_rounding_errors() {
        let scale = NumericScale::from_values([0.12, -0.04, 0.15].into_iter(), true);
        assert_eq!(
            scale.ticks().collect::<Vec<_>>(),
            vec![-0.05, 0.0, 0.05, 0.1, 0.15]
        );
        let scale = NumericScale::from_values([0.07, -0.005, 0.3].into_iter(), true);
        assert_eq!(
            scale
                .ticks()
                .map(|tick| format_value(tick, ValueFormat::Percent))
                .collect::<Vec<_>>(),
            vec!["-10%", "0%", "10%", "20%", "30%"]
        );
    }
}
