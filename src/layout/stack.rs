//! Stacked bars: the series of a category stacked into one bar, either by value (`"stack":
//! "normal"`) or as shares of the category's total (`"stack": "percent"`). Positive values stack
//! away from zero in one direction and negative ones in the other, so that no segment hides
//! another. A segment carries its value label inside it when it fits; a normal stack also writes
//! its total beyond its end.

use std::fmt::Write as _;

use super::{
    AXIS_GUTTER, LABEL_SIZE, PLOT_MARGIN, PlotArea,
    axis::{
        NumericScale, add_bottom_category_title, format_value, label_step, push_side_label,
        push_stepped_category_label, warn_if_labels_thinned,
    },
    bar::label_gutter,
    base_elements_with_title, count,
    legend::{add_legend, legend_space},
    series_bar_class,
    title::{horizontal_title, title_extra},
    tooltip, warn_if_labels_omitted,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    reference,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, Dataset, Orientation, Stack},
};

/// One segment of a stack: where it starts and ends on the value axis, and the value it stands
/// for.
#[derive(Debug, Clone, Copy)]
struct Segment {
    start: f64,
    end: f64,
    value: f64,
}

/// The segments of every category, series by series; a missing value has none. In a percent
/// stack the extents are shares of the category's total, from 0 to 1.
fn segments(spec: &ChartSpec, dataset: &Dataset) -> Vec<Vec<Option<Segment>>> {
    (0..dataset.categories.len())
        .map(|index| {
            let total: f64 = dataset
                .series
                .iter()
                .filter_map(|series| series.values[index])
                .sum();
            let (mut positive, mut negative) = (0.0, 0.0);
            dataset
                .series
                .iter()
                .map(|series| {
                    let value = series.values[index]?;
                    let extent = match spec.stack {
                        Some(Stack::Percent) if total > 0.0 => value / total,
                        Some(Stack::Percent) => 0.0,
                        _ => value,
                    };
                    let cursor = if extent >= 0.0 {
                        &mut positive
                    } else {
                        &mut negative
                    };
                    let start = *cursor;
                    *cursor += extent;
                    Some(Segment {
                        start,
                        end: *cursor,
                        value,
                    })
                })
                .collect()
        })
        .collect()
}

/// The total of a category, and where its stack ends in the direction of the total.
fn total(stack: &[Option<Segment>]) -> (f64, f64) {
    let total: f64 = stack.iter().flatten().map(|segment| segment.value).sum();
    let end = stack
        .iter()
        .flatten()
        .map(|segment| segment.end)
        .fold(0.0, |end: f64, value| {
            if total >= 0.0 {
                end.max(value)
            } else {
                end.min(value)
            }
        });
    (total, end)
}

/// Where the stacks of a chart sit: the plot, the band of each category and the scale along
/// which the stacks grow.
struct Frame {
    vertical: bool,
    plot: PlotArea,
    band: f64,
    thickness: f64,
    scale: NumericScale,
}

impl Frame {
    /// The pixel position of a value along the stacks.
    fn along(&self, value: f64) -> f64 {
        let plot = self.plot;
        if self.vertical {
            self.scale.map(value, plot.top + plot.height, plot.top)
        } else {
            self.scale.map(value, plot.left, plot.left + plot.width)
        }
    }

    /// The middle of category `index`, across the stacks.
    fn center(&self, index: usize) -> f64 {
        let start = if self.vertical {
            self.plot.left
        } else {
            self.plot.top
        };
        start + self.band * (count(index) + 0.5)
    }

    /// The rectangle of a segment from `start` to `end` in category `index`.
    fn rect(&self, index: usize, start: f64, end: f64) -> (f64, f64, f64, f64) {
        let (from, to) = (self.along(start), self.along(end));
        let across = self.center(index) - self.thickness / 2.0;
        let length = (to - from).abs();
        if self.vertical {
            (across, from.min(to), self.thickness, length)
        } else {
            (from.min(to), across, length, self.thickness)
        }
    }
}

/// The plot of a stacked chart and where its title goes: the same margins as side-by-side bars.
fn plot_area(
    spec: &ChartSpec,
    dataset: &Dataset,
    metrics: &impl TextMetrics,
) -> (PlotArea, (f64, f64), f64) {
    let vertical = spec.orientation == Orientation::Vertical;
    let (left, right) = if vertical {
        (f64::from(AXIS_GUTTER), f64::from(PLOT_MARGIN))
    } else {
        (label_gutter(dataset, metrics), 68.0)
    };
    let width = f64::from(spec.width) - left - right;
    let title = if vertical {
        (left, width)
    } else {
        horizontal_title(spec, (left, width), metrics)
    };
    let head = title_extra(spec, title.1, metrics);
    let top = 78.0 + head + legend_space(dataset, width, metrics);
    let bottom = match (
        vertical,
        spec.category_axis.title.is_some(),
        spec.value_axis.title.is_some(),
    ) {
        (true, true, _) => 82.0,
        (true, false, _) => 62.0,
        (false, _, true) => 56.0,
        (false, _, false) => 36.0,
    };
    let plot = PlotArea {
        left,
        top,
        width,
        height: f64::from(spec.height) - top - bottom,
        vertical_bars: vertical,
    };
    (plot, title, head)
}

pub(super) fn layout(
    spec: &ChartSpec,
    dataset: &Dataset,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let percent = spec.stack == Some(Stack::Percent);
    let (plot, title, head) = plot_area(spec, dataset, metrics);
    let stacks = segments(spec, dataset);
    let ends = stacks
        .iter()
        .flatten()
        .flatten()
        .map(|segment| segment.end)
        .chain(percent.then_some(1.0));
    let scale = NumericScale::from_values(
        ends.chain(reference::values(spec)),
        true,
        spec.value_axis.bounds(),
    );
    let mut elements = base_elements_with_title(spec, &scale, plot, title, warnings, metrics);
    add_legend(spec, dataset, plot, head, &mut elements, warnings, metrics);
    let bars = elements.len();
    let length = if plot.vertical_bars {
        plot.width
    } else {
        plot.height
    };
    let band = length / count(dataset.categories.len());
    let frame = Frame {
        vertical: plot.vertical_bars,
        plot,
        band,
        thickness: (band * 0.64).max(2.0),
        scale,
    };
    let mut labels_omitted = false;
    let step = if frame.vertical {
        label_step(&dataset.categories, band, metrics)
    } else {
        1
    };
    for (index, category) in dataset.categories.iter().enumerate() {
        for (series_index, segment) in stacks[index].iter().enumerate() {
            if let Some(segment) = segment {
                labels_omitted |= push_segment(
                    spec,
                    dataset,
                    &frame,
                    (index, series_index),
                    *segment,
                    &mut elements,
                    metrics,
                );
            }
        }
        if spec.show_values && !percent && stacks[index].iter().flatten().count() > 0 {
            elements.push(Element::Text(total_label(
                spec,
                &frame,
                index,
                &stacks[index],
            )));
        }
        let path = dataset.category_path(index);
        if frame.vertical {
            let at = (frame.center(index), plot.top + plot.height + 24.0);
            let place = (index, step, band);
            push_stepped_category_label(
                &mut elements,
                category,
                at,
                place,
                metrics,
                warnings,
                &path,
            );
        } else {
            let at = (plot.left - 12.0, frame.center(index));
            let room = (plot.left - 28.0, band);
            push_side_label(&mut elements, category, at, room, metrics, warnings, &path);
        }
    }
    warn_if_labels_omitted(labels_omitted, warnings);
    warn_if_labels_thinned(step, warnings);
    reference::push(spec, scale, plot, bars, &mut elements, warnings, metrics);
    if frame.vertical {
        add_bottom_category_title(
            spec,
            plot.left,
            plot.width,
            f64::from(spec.height),
            &mut elements,
            warnings,
            metrics,
        );
    }
    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// Draws one segment with its tooltip and, when it fits inside, its value or share; returns
/// whether a label had to be left out.
fn push_segment(
    spec: &ChartSpec,
    dataset: &Dataset,
    frame: &Frame,
    (index, series_index): (usize, usize),
    segment: Segment,
    elements: &mut Vec<Element>,
    metrics: &impl TextMetrics,
) -> bool {
    let (x, y, width, height) = frame.rect(index, segment.start, segment.end);
    if width == 0.0 || height == 0.0 {
        return false;
    }
    let percent = spec.stack == Some(Stack::Percent);
    let style = spec.number_style();
    let share = format_value(segment.end - segment.start, spec.axis_style());
    let mut text = tooltip(
        &dataset.categories[index],
        segment.value,
        style,
        dataset.series[series_index].name.as_deref(),
    );
    if percent {
        write!(text, " ({share})").expect("writing to String cannot fail");
    }
    elements.push(Element::Rect(Rect {
        x,
        y,
        width,
        height,
        class: series_bar_class(spec, series_index),
        series_index: None,
        style_index: None,
        tooltip: Some(text),
    }));
    if !spec.show_values {
        return false;
    }
    let content = if percent {
        share
    } else {
        format_value(segment.value, style)
    };
    let text_width = metrics.width(&content, LABEL_SIZE);
    let fits = if frame.vertical {
        height >= LABEL_SIZE + 6.0 && width >= text_width + 6.0
    } else {
        width >= text_width + 12.0 && height >= LABEL_SIZE + 4.0
    };
    if fits {
        elements.push(Element::Text(Text {
            x: x + width / 2.0,
            y: y + height / 2.0 + 4.0,
            // An outlined segment shows the background, so its label takes the text color.
            class: if series_bar_class(spec, series_index).ends_with("outline") {
                "chartlet-value"
            } else {
                "chartlet-value-inverse"
            },
            anchor: TextAnchor::Middle,
            content,
        }));
    }
    !fits
}

/// The total of a stack by value, beyond the end of the stack in the direction of the total.
fn total_label(spec: &ChartSpec, frame: &Frame, index: usize, stack: &[Option<Segment>]) -> Text {
    let (total, end) = total(stack);
    let end = frame.along(end);
    let center = frame.center(index);
    let content = format_value(total, spec.number_style());
    let forward = total >= 0.0;
    if frame.vertical {
        Text {
            x: center,
            y: if forward { end - 8.0 } else { end + 16.0 },
            class: "chartlet-value",
            anchor: TextAnchor::Middle,
            content,
        }
    } else {
        Text {
            x: if forward { end + 8.0 } else { end - 8.0 },
            y: center + 4.0,
            class: "chartlet-value",
            anchor: if forward {
                TextAnchor::Start
            } else {
                TextAnchor::End
            },
            content,
        }
    }
}
