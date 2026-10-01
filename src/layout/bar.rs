use super::{
    AXIS_GUTTER, LABEL_SIZE, PLOT_MARGIN, PlotArea, SERIES_BAR_CLASSES,
    axis::{
        NumericScale, add_bottom_category_title, format_value, push_category_label, push_side_label,
    },
    base_elements, base_elements_with_title, count,
    labels::{LabelBox, WithReserve, boxes_overlap, label_box},
    legend::{add_legend, legend_space},
    title::{horizontal_title, title_extra},
    tooltip, warn_if_labels_omitted,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    reference,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, Dataset, NumberStyle},
};

pub(super) fn layout_vertical(
    spec: &ChartSpec,
    dataset: &Dataset,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = f64::from(AXIS_GUTTER);
    let right = f64::from(PLOT_MARGIN);
    let plot_width = width - left - right;
    let head = title_extra(spec, plot_width, metrics);
    let top = 78.0 + head + legend_space(dataset, plot_width, metrics);
    let bottom = if spec.category_axis.title.is_some() {
        82.0
    } else {
        62.0
    };
    let plot_height = height - top - bottom;
    let scale = value_scale(spec, dataset);
    let baseline = scale.map(scale.base(), top + plot_height, top);
    let band = plot_width / count(dataset.categories.len());
    let group = Group::new(band, dataset.series.len());
    let plot = PlotArea {
        left,
        top,
        width: plot_width,
        height: plot_height,
        vertical_bars: true,
    };
    let mut elements = base_elements(spec, &scale, plot, warnings, metrics);
    add_legend(dataset, plot, head, &mut elements, warnings, metrics);
    let bars = elements.len();
    let bar_and_label = |index: usize, series_index: usize, value: f64| {
        let center = left + band * (count(index) + 0.5);
        let (offset, thickness) = group.slot(series_index);
        let x = center - group.width / 2.0 + offset;
        let value_y = scale.map(value, top + plot_height, top);
        let content = format_value(value, spec.number_style());
        (
            (
                x,
                baseline.min(value_y),
                thickness,
                (baseline - value_y).abs(),
            ),
            vertical_value_label(value_y <= baseline, x + thickness / 2.0, value_y, content),
        )
    };
    let crowded = crowded_groups(dataset, bar_and_label, metrics);

    for (index, category) in dataset.categories.iter().enumerate() {
        let center = left + band * (count(index) + 0.5);
        for (series_index, series) in dataset.series.iter().enumerate() {
            let Some(value) = series.values[index] else {
                continue;
            };
            let ((x, y, width, height), label) = bar_and_label(index, series_index, value);
            if value != 0.0 {
                elements.push(Element::Rect(Rect {
                    x,
                    y,
                    width,
                    height,
                    class: bar_class(dataset, series_index),
                    series_index: (dataset.series.len() > 1).then_some(series_index),
                    style_index: None,
                    tooltip: Some(tooltip(
                        category,
                        value,
                        spec.number_style(),
                        series.name.as_deref(),
                    )),
                }));
            }

            if spec.show_values && !crowded[index] {
                elements.push(series_text(label, dataset, series_index));
            }
        }

        push_category_label(
            &mut elements,
            category,
            (center, top + plot_height + 24.0),
            (band - 8.0).max(20.0),
            metrics,
            warnings,
            &dataset.category_path(index),
        );
    }
    warn_if_labels_omitted(spec.show_values && crowded.contains(&true), warnings);
    reference::push(spec, scale, plot, bars, &mut elements, warnings, metrics);

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

pub(super) fn layout_horizontal(
    spec: &ChartSpec,
    dataset: &Dataset,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = label_gutter(dataset, metrics);
    let right = 68.0;
    let plot_width = width - left - right;
    let title = horizontal_title(spec, (left, plot_width), metrics);
    let head = title_extra(spec, title.1, metrics);
    let top = 78.0 + head + legend_space(dataset, plot_width, metrics);
    let bottom = if spec.value_axis.title.is_some() {
        56.0
    } else {
        36.0
    };
    let plot_height = height - top - bottom;
    let scale = value_scale(spec, dataset);
    let baseline = scale.map(scale.base(), left, left + plot_width);
    let band = plot_height / count(dataset.categories.len());
    let group = Group::new(band, dataset.series.len());
    let plot = PlotArea {
        left,
        top,
        width: plot_width,
        height: plot_height,
        vertical_bars: false,
    };
    let mut elements = base_elements_with_title(spec, &scale, plot, title, warnings, metrics);
    add_legend(dataset, plot, head, &mut elements, warnings, metrics);
    let bars = elements.len();
    let bar_and_label = |index: usize, series_index: usize, value: f64| {
        let center = top + band * (count(index) + 0.5);
        let (offset, thickness) = group.slot(series_index);
        let y = center - group.width / 2.0 + offset;
        let value_x = scale.map(value, left, left + plot_width);
        (
            (
                baseline.min(value_x),
                y,
                (baseline - value_x).abs(),
                thickness,
            ),
            horizontal_value_label(
                value,
                value_x,
                baseline,
                y + thickness / 2.0,
                spec.number_style(),
                metrics,
            ),
        )
    };
    let crowded = crowded_groups(dataset, bar_and_label, metrics);

    for (index, category) in dataset.categories.iter().enumerate() {
        let center = top + band * (count(index) + 0.5);
        for (series_index, series) in dataset.series.iter().enumerate() {
            let Some(value) = series.values[index] else {
                continue;
            };
            let ((x, y, width, height), label) = bar_and_label(index, series_index, value);
            if value != 0.0 {
                elements.push(Element::Rect(Rect {
                    x,
                    y,
                    width,
                    height,
                    class: bar_class(dataset, series_index),
                    series_index: (dataset.series.len() > 1).then_some(series_index),
                    style_index: None,
                    tooltip: Some(tooltip(
                        category,
                        value,
                        spec.number_style(),
                        series.name.as_deref(),
                    )),
                }));
            }

            if spec.show_values && !crowded[index] {
                elements.push(series_text(label, dataset, series_index));
            }
        }

        push_side_label(
            &mut elements,
            category,
            (left - 12.0, center),
            (left - 28.0, band),
            metrics,
            warnings,
            &dataset.category_path(index),
        );
    }
    warn_if_labels_omitted(spec.show_values && crowded.contains(&true), warnings);
    reference::push(spec, scale, plot, bars, &mut elements, warnings, metrics);

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// Places a value label above a bar that rises from its baseline, below one that hangs from it.
fn vertical_value_label(rising: bool, center_x: f64, value_y: f64, content: String) -> Text {
    Text {
        x: center_x,
        y: if rising {
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
    style: NumberStyle,
    metrics: &impl TextMetrics,
) -> Text {
    let content = format_value(value, style);
    // Inverse text is only readable on the bar itself; a short negative bar gets its label
    // outside, left of the bar end.
    let fits_inside = metrics.width(&content, LABEL_SIZE) + 16.0 <= (baseline - value_x).abs();
    let (x, class, anchor) = if value_x >= baseline {
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

fn series_text(text: Text, dataset: &Dataset, series_index: usize) -> Element {
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
}

impl Group {
    fn new(band: f64, series_count: usize) -> Self {
        if series_count == 1 {
            let width = (band * 0.64).max(2.0);
            return Self {
                width,
                slot: width,
                gap: 0.0,
            };
        }
        let series = count(series_count);
        let width = (band * 0.8).max(2.0 * series);
        let slot = width / series;
        Self {
            width,
            slot,
            gap: (slot * 0.15).min(4.0),
        }
    }

    /// Offset from the group's leading edge and drawn thickness of one series' bar.
    fn slot(self, series_index: usize) -> (f64, f64) {
        (
            count(series_index) * self.slot + self.gap / 2.0,
            self.slot - self.gap,
        )
    }
}

/// Room a value label keeps from other value labels and bars, beyond its measured box.
const VALUE_LABEL_CLEARANCE: f64 = 2.0;

/// Which categories leave out their value labels. `bar_and_label` gives the bar of one series in
/// one category, as left, top, width and height, and its value label. A label is measured with
/// [`FALLBACK_RESERVE`] and [`VALUE_LABEL_CLEARANCE`]; when it overlaps another value label or another bar of its own
/// category or of a neighbouring one, its category shows none of its value labels. All or nothing
/// per category, so that a gap in the labels never reads as a missing value; overlapping labels
/// of two neighbouring categories leave out the labels of both.
fn crowded_groups(
    dataset: &Dataset,
    bar_and_label: impl Fn(usize, usize, f64) -> ((f64, f64, f64, f64), Text),
    metrics: &impl TextMetrics,
) -> Vec<bool> {
    let reserved = WithReserve(metrics);
    let placed: Vec<Vec<(usize, Option<LabelBox>, LabelBox)>> = (0..dataset.categories.len())
        .map(|index| {
            dataset
                .series
                .iter()
                .enumerate()
                .filter_map(|(series_index, series)| {
                    let value = series.values[index]?;
                    let ((x, y, width, height), label) = bar_and_label(index, series_index, value);
                    let bar = (x, y, x + width, y + height);
                    let (left, top, right, bottom) = label_box(&label, &reserved);
                    let label = (
                        left - VALUE_LABEL_CLEARANCE,
                        top - VALUE_LABEL_CLEARANCE,
                        right + VALUE_LABEL_CLEARANCE,
                        bottom + VALUE_LABEL_CLEARANCE,
                    );
                    // A zero value draws no bar.
                    Some((series_index, (value != 0.0).then_some(bar), label))
                })
                .collect()
        })
        .collect();
    (0..placed.len())
        .map(|index| {
            let neighbours = index.saturating_sub(1)..(index + 2).min(placed.len());
            placed[index].iter().any(|&(series_index, _, label)| {
                placed[neighbours.clone()]
                    .iter()
                    .enumerate()
                    .flat_map(|(offset, marks)| {
                        let other = index.saturating_sub(1) + offset;
                        marks.iter().map(move |mark| (other, mark))
                    })
                    .filter(|&(other, &(other_series, _, _))| {
                        (other, other_series) != (index, series_index)
                    })
                    .any(|(_, &(_, bar, other_label))| {
                        boxes_overlap(label, other_label)
                            || bar.is_some_and(|bar| boxes_overlap(label, bar))
                    })
            })
        })
        .collect()
}

fn bar_class(dataset: &Dataset, series_index: usize) -> &'static str {
    if dataset.series.len() == 1 {
        "chartlet-bar"
    } else {
        SERIES_BAR_CLASSES[series_index]
    }
}

/// The left of the plot of horizontal bars: room for the widest category label.
pub(super) fn label_gutter(dataset: &Dataset, metrics: &impl TextMetrics) -> f64 {
    let measured_label = dataset
        .categories
        .iter()
        .map(|category| metrics.width(category, LABEL_SIZE))
        .fold(0.0, f64::max);
    (measured_label + 28.0).clamp(88.0, 210.0)
}

/// The value scale of a bar chart: it starts at zero, or at a power of ten on a logarithmic
/// axis, and reaches every bar, every reference line
/// and the declared range of the value axis.
fn value_scale(spec: &ChartSpec, dataset: &Dataset) -> NumericScale {
    NumericScale::for_axis(
        dataset.values().chain(reference::values(spec)),
        true,
        &spec.value_axis,
    )
}
