//! Kaplan-Meier curves: for every group the share still without the event, as a step curve that
//! drops at each time with events, a mark where an observation was cut off, optionally the 95 %
//! confidence band, and under the time axis the number still at risk. The description gives
//! every group's size, events and median; the data table the survival and the number at risk at
//! every tick of the time axis.

use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{
        LABEL_SIZE, NumericScale, fit_text, format_value, plot_height, push_title, title_extra,
    },
    metrics::TextMetrics,
    scene::{Element, Line, Polyline, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, KmCurve, NumberStyle, SurvivalSpec, ValueAxisSpec, ValueFormat},
    text,
};

fn survival(spec: &ChartSpec) -> &SurvivalSpec {
    spec.survival
        .as_ref()
        .expect("validated survival charts carry a survival block")
}

const LINES: [&str; 4] = [
    "chartlet-km-line chartlet-km-1",
    "chartlet-km-line chartlet-km-2",
    "chartlet-km-line chartlet-km-3",
    "chartlet-km-line chartlet-km-4",
];
const BANDS: [&str; 4] = [
    "chartlet-km-band chartlet-km-1",
    "chartlet-km-band chartlet-km-2",
    "chartlet-km-band chartlet-km-3",
    "chartlet-km-band chartlet-km-4",
];
const MARKS: [&str; 4] = [
    "chartlet-km-mark chartlet-km-1",
    "chartlet-km-mark chartlet-km-2",
    "chartlet-km-mark chartlet-km-3",
    "chartlet-km-mark chartlet-km-4",
];
const SWATCHES: [&str; 4] = [
    "chartlet-km-swatch chartlet-km-1",
    "chartlet-km-swatch chartlet-km-2",
    "chartlet-km-swatch chartlet-km-3",
    "chartlet-km-swatch chartlet-km-4",
];

/// The half length of a censoring mark.
const MARK: f64 = 4.0;
/// The height of a row of the number at risk.
const ROW: f64 = 18.0;

fn percent(spec: &ChartSpec, value: f64, decimals: u8) -> String {
    format_value(
        value,
        NumberStyle {
            format: ValueFormat::Percent,
            decimals: Some(decimals),
            ..spec.number_style()
        },
    )
}

/// The scale of the time axis, from zero over the longest observation, and its ticks.
fn time_scale(spec: &ChartSpec) -> (NumericScale, Vec<f64>) {
    let longest = survival(spec)
        .groups
        .iter()
        .flat_map(|group| {
            group
                .observations
                .iter()
                .map(|observation| observation.time)
        })
        .fold(0.0, f64::max);
    let scale = NumericScale::for_axis([0.0, longest].into_iter(), true, &ValueAxisSpec::default());
    let ticks = scale.ticks().collect();
    (scale, ticks)
}

/// The outline of a curve as steps from `(0, from_start)` to its end: horizontal to the time of
/// a step, then straight to the new level.
fn stepped(curve: &KmCurve, level: impl Fn(&crate::spec::KmStep) -> f64) -> Vec<(f64, f64)> {
    let mut points = vec![(0.0, level(&curve.steps[0]))];
    for step in &curve.steps[1..] {
        let before = points.last().map_or(1.0, |point| point.1);
        points.push((step.time, before));
        points.push((step.time, level(step)));
    }
    let last = points.last().map_or(1.0, |point| point.1);
    points.push((curve.end, last));
    points
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let survival = survival(spec);
    let compact = spec.width < crate::layout::NARROW;
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let right = if compact { 12.0 } else { 24.0 };
    let widest = survival
        .groups
        .iter()
        .map(|group| metrics.width(&group.label, LABEL_SIZE))
        .fold(0.0, f64::max);
    let left = if survival.at_risk {
        (widest + 26.0).clamp(
            f64::from(crate::layout::axis_gutter(spec.width)),
            width * 0.3,
        )
    } else {
        f64::from(crate::layout::axis_gutter(spec.width))
    };
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
    top = push_legend(&mut elements, warnings, spec, (left, top - 10.0), metrics) + 8.0;
    let table = if survival.at_risk {
        ROW * (crate::layout::count(survival.groups.len()) + 1.0) + 8.0
    } else {
        0.0
    };
    let title = if survival.time_title.is_some() {
        22.0
    } else {
        0.0
    };
    let plot = (
        left,
        top,
        width - left - right,
        plot_height(height - top - 36.0 - title - table, warnings),
    );
    push_axes(&mut elements, spec, plot, metrics, warnings);
    let (scale, ticks) = time_scale(spec);
    let x = |time: f64| scale.map(time, plot.0, plot.0 + plot.2);
    let y = |level: f64| plot.1 + plot.3 * (1.0 - level);
    for (index, group) in survival.groups.iter().enumerate() {
        let curve = group.curve();
        if survival.confidence {
            let mut outline = stepped(&curve, |step| step.upper);
            outline.extend(stepped(&curve, |step| step.lower).into_iter().rev());
            elements.push(Element::Polyline(Polyline {
                points: outline.into_iter().map(|(t, s)| (x(t), y(s))).collect(),
                class: BANDS[index],
                topic: None,
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
        elements.push(Element::Polyline(Polyline {
            points: stepped(&curve, |step| step.survival)
                .into_iter()
                .map(|(t, s)| (x(t), y(s)))
                .collect(),
            class: LINES[index],
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: Some(group_sentence(spec, index, &curve)),
        }));
        for (time, level) in &curve.censored {
            elements.push(Element::Line(Line {
                x1: x(*time),
                y1: y(*level) - MARK,
                x2: x(*time),
                y2: y(*level) + MARK,
                class: MARKS[index],
            }));
        }
    }
    if survival.at_risk {
        push_at_risk(&mut elements, spec, (plot, &ticks), &x, metrics, warnings);
    }
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

/// The gridlines and labels of both axes, and the title of the time axis.
fn push_axes(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (left, top, width, height): (f64, f64, f64, f64),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    let levels = NumericScale::for_axis([0.0, 1.0].into_iter(), true, &ValueAxisSpec::default());
    let style = NumberStyle {
        format: ValueFormat::Percent,
        decimals: Some(0),
        ..spec.axis_style()
    };
    for level in levels.ticks() {
        let y = top + height * (1.0 - level);
        elements.push(Element::Line(Line {
            x1: left,
            y1: y,
            x2: left + width,
            y2: y,
            class: if levels.is_zero(level) {
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
            content: levels.tick_label(level, style),
        }));
    }
    let (scale, ticks) = time_scale(spec);
    for tick in ticks {
        let x = scale.map(tick, left, left + width);
        elements.push(Element::Text(Text {
            x,
            y: top + height + 22.0,
            class: "chartlet-tick",
            anchor: TextAnchor::Middle,
            content: scale.tick_label(tick, spec.axis_style()),
        }));
    }
    if let Some(title) = &survival(spec).time_title {
        elements.push(Element::Text(Text {
            x: left + width / 2.0,
            y: top + height + 42.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: fit_text(
                title,
                width,
                LABEL_SIZE,
                metrics,
                warnings,
                "/survival/timeTitle",
            ),
        }));
    }
}

/// The number at risk of every group at every tick, under the axis, with the name of the group
/// at the left.
fn push_at_risk(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    ((left, top, _, height), ticks): ((f64, f64, f64, f64), &[f64]),
    x: &impl Fn(f64) -> f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    let survival = survival(spec);
    let title = if survival.time_title.is_some() {
        22.0
    } else {
        0.0
    };
    let first = top + height + 44.0 + title;
    elements.push(Element::Text(Text {
        x: left,
        y: first,
        class: "chartlet-axis-title",
        anchor: TextAnchor::Start,
        content: spec.locale.words().number_at_risk.to_owned(),
    }));
    for (index, group) in survival.groups.iter().enumerate() {
        let y = first + ROW * (crate::layout::count(index) + 1.0);
        elements.push(Element::Text(Text {
            x: left - 22.0,
            y,
            class: "chartlet-tick",
            anchor: TextAnchor::End,
            content: fit_text(
                &group.label,
                left - 26.0,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/survival/groups/{index}/label"),
            ),
        }));
        for tick in ticks {
            elements.push(Element::Text(Text {
                x: x(*tick),
                y,
                class: "chartlet-tick",
                anchor: TextAnchor::Middle,
                content: group.at_risk(*tick).to_string(),
            }));
        }
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
    let right = f64::from(spec.width) - 12.0;
    let (mut x, mut y) = (left, top);
    for (index, group) in survival(spec).groups.iter().enumerate() {
        let reach = 24.0 + metrics.width(&group.label, LABEL_SIZE);
        if x > left && x + reach > right {
            x = left;
            y += 20.0;
        }
        elements.push(Element::Rect(Rect {
            x,
            y: y + 3.0,
            width: 14.0,
            height: 3.0,
            class: SWATCHES[index],
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: x + 20.0,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                &group.label,
                (right - x - 20.0).max(0.0),
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/survival/groups/{index}/label"),
            ),
        }));
        x += reach + 16.0;
    }
    y + 16.0
}

/// A group in a sentence: its size, its events, and its median and last survival.
fn group_sentence(spec: &ChartSpec, index: usize, curve: &KmCurve) -> String {
    let group = &survival(spec).groups[index];
    let last = curve.steps.last().map_or(1.0, |step| step.survival);
    text::survival_group(
        spec.locale,
        (&group.label, group.observations.len(), curve.events),
        curve
            .median
            .map(|median| format_value(median, spec.number_style()))
            .as_deref(),
        &percent(spec, last, 1),
    )
}

/// The curves in sentences: one for every group.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let survival = survival(spec);
    let mut groups = String::new();
    for (index, group) in survival.groups.iter().enumerate() {
        write!(groups, " {}", group_sentence(spec, index, &group.curve())).expect("write");
    }
    text::survival_summary(
        spec.locale,
        survival.groups.len(),
        survival.confidence,
        &groups,
    )
}

/// One row for every group at every tick of the time axis: its survival, the confidence limits
/// if they are drawn, and the number at risk.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let survival = survival(spec);
    let words = spec.locale.words();
    let (scale, ticks) = time_scale(spec);
    let mut columns = vec![
        words.group.to_owned(),
        words.time.to_owned(),
        words.survival.to_owned(),
    ];
    if survival.confidence {
        columns.push(words.lower.to_owned());
        columns.push(words.upper.to_owned());
    }
    columns.push(words.at_risk.to_owned());
    let mut rows = Vec::new();
    for group in &survival.groups {
        let curve = group.curve();
        for tick in &ticks {
            let step = curve.at(tick.min(curve.end));
            let mut row = vec![
                group.label.clone(),
                scale.tick_label(*tick, spec.axis_style()),
                percent(spec, step.survival, 1),
            ];
            if survival.confidence {
                row.push(percent(spec, step.lower, 1));
                row.push(percent(spec, step.upper, 1));
            }
            row.push(group.at_risk(*tick).to_string());
            rows.push(row);
        }
    }
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}
