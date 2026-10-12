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
    spec::{
        AxisScale, ChartSpec, NumberStyle, Quadrant, ScatterAxis, ScatterPointSpec,
        ScatterQuadrantsSpec, ScatterSpec, ValueAxisSpec,
    },
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

/// The scale of one axis: over the points, the lines and the named ticks along it, or exactly
/// over its range.
fn axis_scale(scatter: &ScatterSpec, axis: ScatterAxis) -> NumericScale {
    let along_x = axis == ScatterAxis::X;
    let values = scatter
        .points
        .iter()
        .map(|point| if along_x { point.x } else { point.y })
        .chain(
            scatter
                .lines
                .iter()
                .filter(|line| line.axis == axis)
                .map(|line| line.value),
        )
        .chain(
            scatter
                .x_ticks
                .iter()
                .filter(|_| along_x)
                .map(|tick| tick.at),
        )
        .chain(
            scatter
                .quadrants
                .iter()
                .map(|quadrants| if along_x { quadrants.x } else { quadrants.y }),
        );
    let (scale, range) = if along_x {
        (scatter.x_scale, scatter.x_range)
    } else {
        (scatter.y_scale, scatter.y_range)
    };
    NumericScale::for_axis(
        values,
        false,
        &ValueAxisSpec {
            min: range.map(|range| range[0]),
            max: range.map(|range| range[1]),
            exact: range.is_some(),
            scale,
            ..ValueAxisSpec::default()
        },
    )
}

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
    let (xs, ys) = (
        axis_scale(scatter, ScatterAxis::X),
        axis_scale(scatter, ScatterAxis::Y),
    );
    let x = |value: f64| xs.map(value, plot.0, plot.0 + plot.2);
    let y = |value: f64| ys.map(value, plot.1 + plot.3, plot.1);
    push_axes(&mut elements, spec, (plot, &xs, &ys), metrics, warnings);
    let (names, omitted) = place_names(spec, plot, (&x, &y), metrics);
    if let Some(quadrants) = &scatter.quadrants {
        let taken: Vec<Bounds> = names.iter().map(|(rect, _)| *rect).collect();
        push_quadrants(
            &mut elements,
            spec,
            quadrants,
            (plot, (&x, &y)),
            &taken,
            metrics,
            warnings,
        );
    }
    for line in &scatter.lines {
        push_line(&mut elements, line, plot, (&x, &y), metrics, warnings);
    }
    push_fits(&mut elements, scatter, (&x, &y));
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
    elements.extend(names.into_iter().map(|(_, text)| Element::Text(text)));
    warn_if_labels_omitted(omitted, warnings);
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

/// The least-squares lines, under the points, in the color of their group.
fn push_fits(
    elements: &mut Vec<Element>,
    scatter: &ScatterSpec,
    (x, y): (&impl Fn(f64) -> f64, &impl Fn(f64) -> f64),
) {
    if !scatter.regression {
        return;
    }
    let groups = scatter.groups();
    let back = |scale: AxisScale, value: f64| {
        if scale == AxisScale::Log {
            value.exp()
        } else {
            value
        }
    };
    for (group, fit) in scatter.fits() {
        let at = |along: f64| fit.intercept + fit.slope * along;
        let color = group.map_or(1, |name| {
            groups
                .iter()
                .position(|known| *known == name)
                .map_or(1, |p| p + 1)
        });
        elements.push(Element::Line(Line {
            x1: x(back(scatter.x_scale, fit.from)),
            y1: y(back(scatter.y_scale, at(fit.from))),
            x2: x(back(scatter.x_scale, fit.to)),
            y2: y(back(scatter.y_scale, at(fit.to))),
            class: FITS[color - 1],
        }));
    }
}

const FITS: [&str; 4] = [
    "chartlet-sc-fit chartlet-sc-1",
    "chartlet-sc-fit chartlet-sc-2",
    "chartlet-sc-fit chartlet-sc-3",
    "chartlet-sc-fit chartlet-sc-4",
];

/// The fitted lines in words: slope, intercept and the share of variance they explain.
fn fit_sentences(spec: &ChartSpec) -> String {
    let scatter = scatter(spec);
    let number = |value: f64| {
        format_value(
            value,
            NumberStyle {
                decimals: Some(3),
                ..spec.number_style()
            },
        )
    };
    scatter
        .fits()
        .into_iter()
        .map(|(group, fit)| {
            text::scatter_fit(
                spec.locale,
                group,
                (
                    number(fit.slope),
                    number(fit.intercept),
                    number(fit.r_squared),
                ),
                (
                    scatter.x_scale == AxisScale::Log || scatter.y_scale == AxisScale::Log,
                    fit.count,
                ),
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
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

/// Left, top, right and bottom.
type Bounds = (f64, f64, f64, f64);

/// The names of the points beside them, in the order of the list, leaving out a name that would
/// run over one already written: each with the box it takes, and whether any was left out.
fn place_names(
    spec: &ChartSpec,
    plot: (f64, f64, f64, f64),
    (x, y): (&impl Fn(f64) -> f64, &impl Fn(f64) -> f64),
    metrics: &impl TextMetrics,
) -> (Vec<(Bounds, Text)>, bool) {
    let mut placed: Vec<(Bounds, Text)> = Vec::new();
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
        let clash = placed.iter().any(|(t, _)| overlap(rect, *t));
        if clash || start < plot.0 {
            omitted = true;
            continue;
        }
        placed.push((
            rect,
            Text {
                x: if beside_right { start } else { start + reach },
                y: cy + 4.0,
                class: "chartlet-sc-label",
                anchor: if beside_right {
                    TextAnchor::Start
                } else {
                    TextAnchor::End
                },
                content: label.clone(),
            },
        ));
    }
    (placed, omitted)
}

/// Whether two boxes, as left, top, right and bottom, overlap.
fn overlap(a: Bounds, b: Bounds) -> bool {
    a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
}

/// Space between a quadrant's name and the edges of its area, and the height of the name.
const QUADRANT_PAD: f64 = 6.0;
const QUADRANT_TEXT: f64 = 14.0;

/// The box of a quadrant's name `reach` wide in a corner of its area (left, top, right, bottom),
/// named by its sides: on the left edge of the area or not, on its top edge or not; and whether it
/// starts on the left.
fn corner_box(
    (x0, y0, x1, y1): Bounds,
    reach: f64,
    (on_left, on_top): (bool, bool),
) -> (Bounds, bool) {
    let start = if on_left {
        x0 + QUADRANT_PAD
    } else {
        x1 - QUADRANT_PAD - reach
    };
    let upper = if on_top {
        y0 + QUADRANT_PAD
    } else {
        y1 - QUADRANT_PAD - QUADRANT_TEXT
    };
    (
        (start, upper, start + reach, upper + QUADRANT_TEXT),
        on_left,
    )
}

/// The first corner of a quadrant's area, from the outer one, where its name covers no point and
/// no point's name.
fn free_corner(
    quadrant: Quadrant,
    area: Bounds,
    reach: f64,
    (dots, radius, names): (&[(f64, f64)], f64, &[Bounds]),
) -> Option<(Bounds, bool)> {
    let (left, top) = (quadrant.is_left(), quadrant.is_top());
    [(left, top), (!left, top), (left, !top), (!left, !top)]
        .into_iter()
        .map(|sides| corner_box(area, reach, sides))
        .find(|(rect, _)| {
            let grown = (
                rect.0 - radius,
                rect.1 - radius,
                rect.2 + radius,
                rect.3 + radius,
            );
            !dots
                .iter()
                .any(|(cx, cy)| *cx > grown.0 && *cx < grown.2 && *cy > grown.1 && *cy < grown.3)
                && !names.iter().any(|name| overlap(*rect, *name))
        })
}

/// The two dividing lines of the quadrants and the name of each area in one of its corners: the
/// outer corner, or the first other corner where it covers no point and no point's name.
fn push_quadrants(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    quadrants: &ScatterQuadrantsSpec,
    ((left, top, width, height), (x, y)): (
        (f64, f64, f64, f64),
        (&impl Fn(f64) -> f64, &impl Fn(f64) -> f64),
    ),
    names: &[Bounds],
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    let scatter = scatter(spec);
    let (right, bottom) = (left + width, top + height);
    let (at_x, at_y) = (x(quadrants.x), y(quadrants.y));
    elements.push(Element::Line(Line {
        x1: at_x,
        y1: top,
        x2: at_x,
        y2: bottom,
        class: "chartlet-zero",
    }));
    elements.push(Element::Line(Line {
        x1: left,
        y1: at_y,
        x2: right,
        y2: at_y,
        class: "chartlet-zero",
    }));
    for (name, at, low, high) in [("x", at_x, left, right), ("y", at_y, top, bottom)] {
        if at - low < 1.0 || high - at < 1.0 {
            warnings.push(ChartWarning::new(
                "quadrant_without_room",
                format!("/scatter/quadrants/{name}"),
                format!("the dividing line falls on the end of the axis, so two areas have no room; set {name}Range so that it lies inside"),
            ));
        }
    }
    let radius = if scatter.points.len() <= FEW {
        3.5
    } else {
        2.0
    } + 1.0;
    let dots: Vec<(f64, f64)> = scatter
        .points
        .iter()
        .map(|point| (x(point.x), y(point.y)))
        .collect();
    for quadrant in Quadrant::ALL {
        let Some(label) = quadrants.label(quadrant) else {
            continue;
        };
        let path = format!("/scatter/quadrants/labels/{}", quadrant.field());
        let (x0, x1) = if quadrant.is_left() {
            (left, at_x)
        } else {
            (at_x, right)
        };
        let (y0, y1) = if quadrant.is_top() {
            (top, at_y)
        } else {
            (at_y, bottom)
        };
        let room = x1 - x0 - 2.0 * QUADRANT_PAD;
        if room < 2.0 * QUADRANT_TEXT || y1 - y0 < QUADRANT_TEXT + 2.0 * QUADRANT_PAD {
            if x1 - x0 >= 1.0 && y1 - y0 >= 1.0 {
                warnings.push(ChartWarning::new(
                    "quadrant_without_room",
                    path,
                    "this area is too small for its name, which is left out of the drawing; the description and the table still name it",
                ));
            }
            continue;
        }
        let content = fit_text(label, room, LABEL_SIZE, metrics, warnings, &path);
        let reach = metrics.width(&content, LABEL_SIZE);
        let spot = free_corner(quadrant, (x0, y0, x1, y1), reach, (&dots, radius, names));
        let (rect, on_left) = spot.unwrap_or_else(|| {
            warnings.push(ChartWarning::new(
                "label_overlap",
                path.clone(),
                "every corner of this area holds points or their names, so its name is drawn over them",
            ));
            corner_box(
                (x0, y0, x1, y1),
                reach,
                (quadrant.is_left(), quadrant.is_top()),
            )
        });
        elements.push(Element::Text(Text {
            x: if on_left { rect.0 } else { rect.2 },
            y: rect.3 - 3.0,
            class: "chartlet-axis-title",
            anchor: if on_left {
                TextAnchor::Start
            } else {
                TextAnchor::End
            },
            content,
        }));
    }
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
    if !scatter(spec).x_ticks.is_empty() {
        push_named_ticks(elements, spec, ((left, top, width, height), xs), metrics);
    }
    for tick in xs.ticks().filter(|_| scatter(spec).x_ticks.is_empty()) {
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

/// The named labels of the x axis, each at its place, leaving out one that would run into the one
/// before it.
fn push_named_ticks(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    ((left, top, width, height), xs): ((f64, f64, f64, f64), &NumericScale),
    metrics: &impl TextMetrics,
) {
    let mut ticks: Vec<(f64, &str)> = scatter(spec)
        .x_ticks
        .iter()
        .map(|tick| (xs.map(tick.at, left, left + width), tick.label.as_str()))
        .collect();
    crate::sort::by(&mut ticks, |a, b| a.0.total_cmp(&b.0));
    let mut end = f64::MIN;
    for (x, label) in ticks {
        let reach = metrics.width(label, LABEL_SIZE);
        if x - reach / 2.0 < end + 4.0 {
            continue;
        }
        end = x + reach / 2.0;
        elements.push(Element::Line(Line {
            x1: x,
            y1: top + height,
            x2: x,
            y2: top + height + 5.0,
            class: "chartlet-zero",
        }));
        elements.push(Element::Text(Text {
            x,
            y: top + height + 22.0,
            class: "chartlet-tick",
            anchor: TextAnchor::Middle,
            content: label.to_owned(),
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
        text::value_span(spec.locale, &show(low), &show(high))
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
    let mut description = text::scatter_summary(
        spec.locale,
        scatter.points.len(),
        (&span(|point| point.x), &span(|point| point.y)),
        (&groups.join(", "), &lines.join("; ")),
        &fit_sentences(spec),
    );
    if let Some(quadrants) = &scatter.quadrants {
        description.push(' ');
        description.push_str(&quadrant_sentences(spec, quadrants));
    }
    description
}

/// The quadrants in sentences: where they are divided, then every area in reading order with
/// its points and the names of the first of them, then the points on a dividing line, if any.
fn quadrant_sentences(spec: &ChartSpec, quadrants: &ScatterQuadrantsSpec) -> String {
    let scatter = scatter(spec);
    let locale = spec.locale;
    let show = |value| format_value(value, spec.number_style());
    let mut sentences = vec![text::quadrants_opening(
        locale,
        &show(quadrants.x),
        &show(quadrants.y),
    )];
    let points_in = |area: Option<Quadrant>| {
        let points: Vec<&ScatterPointSpec> = scatter
            .points
            .iter()
            .filter(|point| quadrants.of(point) == area)
            .collect();
        let names: Vec<&str> = points
            .iter()
            .filter_map(|point| point.label.as_deref())
            .collect();
        (points.len(), names)
    };
    for quadrant in Quadrant::ALL {
        let (count, names) = points_in(Some(quadrant));
        let position = text::quadrant_position(locale, quadrant, true);
        sentences.push(text::quadrant_points(
            locale,
            Some((&position, quadrants.label(quadrant))),
            count,
            &names,
        ));
    }
    let (count, names) = points_in(None);
    if count > 0 {
        sentences.push(text::quadrant_points(locale, None, count, &names));
    }
    sentences.join(" ")
}

/// The name of the area a point lies in, for the table: its label, or where it lies.
fn quadrant_name(
    spec: &ChartSpec,
    quadrants: &ScatterQuadrantsSpec,
    point: &ScatterPointSpec,
) -> String {
    quadrants.of(point).map_or_else(
        || text::on_dividing_line(spec.locale).to_owned(),
        |quadrant| {
            quadrants.label(quadrant).map_or_else(
                || text::quadrant_position(spec.locale, quadrant, false),
                str::to_owned,
            )
        },
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
    if scatter.quadrants.is_some() {
        columns.push(words.quadrant.to_owned());
    }
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
            if let Some(quadrants) = &scatter.quadrants {
                row.push(quadrant_name(spec, quadrants, point));
            }
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
