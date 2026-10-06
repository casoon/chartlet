//! Scatter plots: points on two numeric axes in up to four groups, threshold lines across the
//! plot, and names beside the points that matter (a volcano plot, a Manhattan plot). A plot of
//! many points draws them small and without tooltips; the data table then lists only the named
//! points, or the highest ones, so that it stays readable.

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{
        LABEL_SIZE, NumericScale, fit_text, format_value, plot_height, push_title, title_extra,
        warn_if_labels_omitted,
    },
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, ScatterAxis, ScatterPointSpec, ScatterSpec, ValueAxisSpec},
    text,
};

fn scatter(spec: &ChartSpec) -> &ScatterSpec {
    spec.scatter
        .as_ref()
        .expect("validated scatter plots carry a scatter block")
}

const DOTS: [&str; 4] = [
    "chartlet-sc-dot chartlet-sc-1",
    "chartlet-sc-dot chartlet-sc-2",
    "chartlet-sc-dot chartlet-sc-3",
    "chartlet-sc-dot chartlet-sc-4",
];
const SWATCHES: [&str; 4] = [
    "chartlet-sc-swatch chartlet-sc-1",
    "chartlet-sc-swatch chartlet-sc-2",
    "chartlet-sc-swatch chartlet-sc-3",
    "chartlet-sc-swatch chartlet-sc-4",
];

/// Up to this many points get a radius and a tooltip each; more are drawn small and plain.
const FEW: usize = 300;
/// The points a big plot lists in its table when none is named, and the most any plot lists.
const LISTED: usize = 20;
const TABLE_ALL: usize = 100;

/// The points the data table lists: all of a small plot, else the named ones, else the highest.
fn listed(scatter: &ScatterSpec) -> Vec<usize> {
    let all: Vec<usize> = (0..scatter.points.len()).collect();
    if scatter.points.len() <= TABLE_ALL {
        return all;
    }
    let named: Vec<usize> = all
        .iter()
        .copied()
        .filter(|index| scatter.points[*index].label.is_some())
        .collect();
    if !named.is_empty() {
        return named;
    }
    let mut highest = all;
    crate::sort::by(&mut highest, |a, b| {
        scatter.points[*b].y.total_cmp(&scatter.points[*a].y)
    });
    highest.truncate(LISTED);
    highest
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let scatter = scatter(spec);
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let left = f64::from(crate::layout::axis_gutter(spec.width));
    let right = f64::from(crate::layout::plot_margin(spec.width));
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        left,
        width - left - right,
        metrics,
        warnings,
    );
    let mut top = 56.0 + title_extra(spec, width - left - right, metrics);
    top = push_legend(&mut elements, warnings, spec, (left, top - 10.0), metrics);
    let bottom = if scatter.x_title.is_some() {
        62.0
    } else {
        36.0
    };
    let plot = (
        left,
        top + 8.0,
        width - left - right,
        plot_height(height - top - 8.0 - bottom, warnings),
    );
    let along = |axis: ScatterAxis| {
        let values = scatter
            .points
            .iter()
            .map(|point| {
                if axis == ScatterAxis::X {
                    point.x
                } else {
                    point.y
                }
            })
            .chain(
                scatter
                    .lines
                    .iter()
                    .filter(|line| line.axis == axis)
                    .map(|line| line.value),
            );
        NumericScale::for_axis(values, false, &ValueAxisSpec::default())
    };
    let (xs, ys) = (along(ScatterAxis::X), along(ScatterAxis::Y));
    let x = |value: f64| xs.map(value, plot.0, plot.0 + plot.2);
    let y = |value: f64| ys.map(value, plot.1 + plot.3, plot.1);
    push_axes(&mut elements, spec, (plot, &xs, &ys), metrics, warnings);
    for line in &scatter.lines {
        push_line(&mut elements, line, plot, (&x, &y), metrics, warnings);
    }
    let few = scatter.points.len() <= FEW;
    for point in &scatter.points {
        elements.push(Element::Circle(Circle {
            cx: x(point.x),
            cy: y(point.y),
            radius: if few { 3.5 } else { 2.0 },
            class: DOTS[scatter.color(point) - 1],
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: (few || point.label.is_some()).then(|| tooltip(spec, point)),
        }));
    }
    let omitted = push_names(&mut elements, spec, plot, (&x, &y), metrics);
    warn_if_labels_omitted(omitted, warnings);
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

fn tooltip(spec: &ChartSpec, point: &ScatterPointSpec) -> String {
    let at = format!(
        "{}, {}",
        format_value(point.x, spec.number_style()),
        format_value(point.y, spec.number_style())
    );
    point
        .label
        .as_ref()
        .map_or(at.clone(), |label| format!("{label}: {at}"))
}

/// The names of the points beside them, in the order of the list, leaving out a name that would
/// run over one already written; returns whether any was left out.
fn push_names(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    plot: (f64, f64, f64, f64),
    (x, y): (&impl Fn(f64) -> f64, &impl Fn(f64) -> f64),
    metrics: &impl TextMetrics,
) -> bool {
    let mut taken: Vec<(f64, f64, f64, f64)> = Vec::new();
    let mut omitted = false;
    for point in &scatter(spec).points {
        let Some(label) = &point.label else { continue };
        let reach = metrics.width(label, LABEL_SIZE);
        let (cx, cy) = (x(point.x), y(point.y));
        let beside_right = cx + 7.0 + reach <= plot.0 + plot.2;
        let start = if beside_right {
            cx + 7.0
        } else {
            cx - 7.0 - reach
        };
        let rect = (start, cy - 9.0, start + reach, cy + 5.0);
        let clash = taken
            .iter()
            .any(|t| rect.0 < t.2 && t.0 < rect.2 && rect.1 < t.3 && t.1 < rect.3);
        if clash || start < plot.0 {
            omitted = true;
            continue;
        }
        taken.push(rect);
        elements.push(Element::Text(Text {
            x: if beside_right { start } else { start + reach },
            y: cy + 4.0,
            class: "chartlet-sc-label",
            anchor: if beside_right {
                TextAnchor::Start
            } else {
                TextAnchor::End
            },
            content: label.clone(),
        }));
    }
    omitted
}

/// A line across the plot at a value of one axis, with its label at the end.
fn push_line(
    elements: &mut Vec<Element>,
    line: &crate::spec::ScatterLineSpec,
    plot: (f64, f64, f64, f64),
    (x, y): (&impl Fn(f64) -> f64, &impl Fn(f64) -> f64),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    let (left, top, width, height) = plot;
    let (x1, y1, x2, y2) = match line.axis {
        ScatterAxis::X => (x(line.value), top, x(line.value), top + height),
        ScatterAxis::Y => (left, y(line.value), left + width, y(line.value)),
    };
    elements.push(Element::Line(Line {
        x1,
        y1,
        x2,
        y2,
        class: "chartlet-sc-line",
    }));
    if let Some(label) = &line.label {
        let (at_x, at_y, anchor) = match line.axis {
            ScatterAxis::X => (x1 + 4.0, top + 12.0, TextAnchor::Start),
            ScatterAxis::Y => (left + width - 4.0, y1 - 5.0, TextAnchor::End),
        };
        let room = match line.axis {
            ScatterAxis::X => left + width - at_x,
            ScatterAxis::Y => width,
        };
        elements.push(Element::Text(Text {
            x: at_x,
            y: at_y,
            class: "chartlet-sc-label",
            anchor,
            content: fit_text(label, room, LABEL_SIZE, metrics, warnings, "/scatter/lines"),
        }));
    }
}

/// The gridlines and labels of both axes, and their titles.
fn push_axes(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    ((left, top, width, height), xs, ys): ((f64, f64, f64, f64), &NumericScale, &NumericScale),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    for tick in ys.ticks() {
        let y = ys.map(tick, top + height, top);
        elements.push(Element::Line(Line {
            x1: left,
            y1: y,
            x2: left + width,
            y2: y,
            class: if ys.is_zero(tick) {
                "chartlet-zero"
            } else {
                "chartlet-grid"
            },
        }));
        elements.push(Element::Text(Text {
            x: left - 10.0,
            y: y + 4.0,
            class: "chartlet-tick",
            anchor: TextAnchor::End,
            content: ys.tick_label(tick, spec.axis_style()),
        }));
    }
    for tick in xs.ticks() {
        let x = xs.map(tick, left, left + width);
        elements.push(Element::Line(Line {
            x1: x,
            y1: top,
            x2: x,
            y2: top + height,
            class: if xs.is_zero(tick) {
                "chartlet-zero"
            } else {
                "chartlet-grid"
            },
        }));
        elements.push(Element::Text(Text {
            x,
            y: top + height + 22.0,
            class: "chartlet-tick",
            anchor: TextAnchor::Middle,
            content: xs.tick_label(tick, spec.axis_style()),
        }));
    }
    let scatter = scatter(spec);
    if let Some(title) = &scatter.x_title {
        elements.push(Element::Text(Text {
            x: left + width / 2.0,
            y: top + height + 44.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: fit_text(
                title,
                width,
                LABEL_SIZE,
                metrics,
                warnings,
                "/scatter/xTitle",
            ),
        }));
    }
    if let Some(title) = &scatter.y_title {
        elements.push(Element::Text(Text {
            x: left,
            y: top - 10.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Start,
            content: fit_text(
                title,
                width,
                LABEL_SIZE,
                metrics,
                warnings,
                "/scatter/yTitle",
            ),
        }));
    }
}

/// The legend of the groups in a row under the title; returns where it ends.
fn push_legend(
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    spec: &ChartSpec,
    (left, top): (f64, f64),
    metrics: &impl TextMetrics,
) -> f64 {
    let groups = scatter(spec).groups();
    if groups.is_empty() {
        return top + 10.0;
    }
    let right = f64::from(spec.width) - 12.0;
    let (mut x, mut y) = (left, top);
    for (index, group) in groups.iter().enumerate() {
        let reach = 16.0 + metrics.width(group, LABEL_SIZE);
        if x > left && x + reach > right {
            x = left;
            y += 20.0;
        }
        elements.push(Element::Rect(Rect {
            x,
            y,
            width: 10.0,
            height: 10.0,
            class: SWATCHES[index],
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: x + 16.0,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                group,
                (right - x - 16.0).max(0.0),
                LABEL_SIZE,
                metrics,
                warnings,
                "/scatter/points",
            ),
        }));
        x += reach + 16.0;
    }
    y + 26.0
}

/// The plot in sentences: how many points, where they lie, the groups and the lines.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let scatter = scatter(spec);
    let show = |value| format_value(value, spec.number_style());
    let span = |pick: fn(&ScatterPointSpec) -> f64| {
        let (low, high) = scatter
            .points
            .iter()
            .map(pick)
            .fold((f64::MAX, f64::MIN), |(low, high), value| {
                (low.min(value), high.max(value))
            });
        format!("{} to {}", show(low), show(high))
    };
    let groups: Vec<String> = scatter
        .groups()
        .into_iter()
        .map(|group| {
            let points = scatter
                .points
                .iter()
                .filter(|point| point.group.as_deref() == Some(group))
                .count();
            format!("{group} {points}")
        })
        .collect();
    let lines: Vec<String> = scatter
        .lines
        .iter()
        .map(|line| {
            let axis = if line.axis == ScatterAxis::X {
                "x"
            } else {
                "y"
            };
            let beyond = scatter
                .points
                .iter()
                .filter(|point| {
                    let value = if line.axis == ScatterAxis::X {
                        point.x
                    } else {
                        point.y
                    };
                    value > line.value
                })
                .count();
            let name = line
                .label
                .as_ref()
                .map_or(String::new(), |label| format!("{label}: "));
            format!(
                "{name}{axis} = {}, {beyond} points above it",
                show(line.value)
            )
        })
        .collect();
    text::scatter_summary(
        spec.locale,
        scatter.points.len(),
        (&span(|point| point.x), &span(|point| point.y)),
        (&groups.join(", "), &lines.join("; ")),
    )
}

/// One row for every listed point: its name, group and both numbers.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let scatter = scatter(spec);
    let words = spec.locale.words();
    let rows_of = listed(scatter);
    let grouped = !scatter.groups().is_empty();
    let mut columns = vec![words.point.to_owned()];
    if grouped {
        columns.push(words.group.to_owned());
    }
    columns.push(scatter.x_title.clone().unwrap_or_else(|| "x".to_owned()));
    columns.push(scatter.y_title.clone().unwrap_or_else(|| "y".to_owned()));
    let rows = rows_of
        .iter()
        .map(|index| {
            let point = &scatter.points[*index];
            let mut row = vec![
                point
                    .label
                    .clone()
                    .unwrap_or_else(|| (index + 1).to_string()),
            ];
            if grouped {
                row.push(point.group.clone().unwrap_or_default());
            }
            row.push(format_value(point.x, spec.number_style()));
            row.push(format_value(point.y, spec.number_style()));
            row
        })
        .collect();
    let caption = if rows_of.len() == scatter.points.len() {
        format!("{} {}", words.data_for, spec.title)
    } else {
        text::scatter_caption(
            spec.locale,
            &format!("{} {}", words.data_for, spec.title),
            (rows_of.len(), scatter.points.len()),
        )
    };
    DataTable {
        caption,
        columns,
        rows,
    }
}
