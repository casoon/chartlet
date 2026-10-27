//! Funnels and pyramids. A funnel stacks centered bars as wide as the values and writes the rate
//! between two stages in the gap; a pyramid stacks the levels of a triangle, the widest at the
//! bottom, whose shape ranks the levels and does not measure them. The labels stand left of the
//! shape and the values right of it, so no text sits on a fill; the description and the data table
//! name every stage and rate.

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{LABEL_SIZE, count, fit_text, format_value, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Polyline, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, FunnelSpec, NumberStyle, ValueFormat},
    text,
};

const BAR: &str = "chartlet-funnel-bar";
/// The height of a bar, the gap between two bars of a funnel (the rate stands in it), and the gap
/// between two levels of a pyramid.
const BAR_HEIGHT: (f64, f64) = (24.0, 44.0);
const FUNNEL_GAP: f64 = 20.0;
const PYRAMID_GAP: f64 = 3.0;
/// The narrowest a bar of a funnel is drawn.
const MIN_WIDTH: f64 = 4.0;

fn funnel(spec: &ChartSpec) -> &FunnelSpec {
    spec.funnel
        .as_ref()
        .expect("validated funnels carry a funnel block")
}

/// A share as a percentage, with one decimal.
fn percent(spec: &ChartSpec, share: f64) -> String {
    format_value(
        share,
        NumberStyle {
            format: ValueFormat::Percent,
            decimals: Some(1),
            ..spec.number_style()
        },
    )
}

/// The rate of every stage after the first: its value as a share of the stage before.
fn rates(values: &[f64]) -> Vec<f64> {
    values.windows(2).map(|pair| pair[1] / pair[0]).collect()
}

/// The tooltip of a stage: its label, its value, and for a funnel its shares.
fn tooltip(spec: &ChartSpec, index: usize) -> String {
    let funnel = funnel(spec);
    let label = &funnel.stages[index].label;
    let Some(values) = funnel.values() else {
        return label.clone();
    };
    let value = format_value(values[index], spec.number_style());
    if funnel.is_pyramid() {
        return format!("{label}: {value}");
    }
    if index == 0 {
        format!("{label}: {value}")
    } else {
        format!(
            "{label}: {value} ({} {})",
            percent(spec, values[index] / values[index - 1]),
            text::of_previous_stage(spec.locale)
        )
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let funnel = funnel(spec);
    let values = funnel.values();
    let stages = funnel.stages.len();
    let pyramid = funnel.is_pyramid();
    let compact = spec.width < crate::layout::NARROW;
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let margin = if compact { 12.0 } else { 24.0 };
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        width - 2.0 * margin,
        metrics,
        warnings,
    );
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let inner = width - 2.0 * margin;
    let style = spec.number_style();
    let texts: Vec<String> = values.as_ref().map_or_else(Vec::new, |values| {
        values
            .iter()
            .map(|value| format_value(*value, style))
            .collect()
    });
    let label_width = funnel
        .stages
        .iter()
        .map(|stage| metrics.width(&stage.label, LABEL_SIZE))
        .fold(0.0, f64::max)
        .min(inner * 0.32)
        + 10.0;
    let value_width = texts
        .iter()
        .map(|text| metrics.width(text, LABEL_SIZE))
        .fold(0.0, f64::max)
        + if texts.is_empty() { 0.0 } else { 10.0 };
    let left = margin + label_width;
    let right = width - margin - value_width;
    let area = (right - left).max(40.0);
    let center = left + area / 2.0;
    let gap = if pyramid { PYRAMID_GAP } else { FUNNEL_GAP };
    let note = !pyramid;
    let rows = count(stages);
    let bar = ((height - top - 16.0 - gap * (rows - 1.0) - if note { 28.0 } else { 0.0 }) / rows)
        .clamp(BAR_HEIGHT.0, BAR_HEIGHT.1)
        .floor();
    let reach = values.as_ref().map_or(1.0, |values| {
        values.iter().copied().fold(f64::MIN, f64::max)
    });
    let total = rows * bar + (rows - 1.0) * gap;
    for (index, stage) in funnel.stages.iter().enumerate() {
        let y = top + count(index) * (bar + gap);
        let tip = Some(tooltip(spec, index));
        if let (false, Some(values)) = (pyramid, &values) {
            let bar_width = (area * values[index] / reach).max(MIN_WIDTH);
            elements.push(Element::Rect(Rect {
                x: center - bar_width / 2.0,
                y,
                width: bar_width,
                height: bar,
                class: BAR,
                series_index: None,
                style_index: None,
                tooltip: tip,
            }));
        } else {
            let half = |at: f64| area / 2.0 * (at - top) / total;
            let (above, below) = (half(y), half(y + bar));
            elements.push(Element::Polyline(Polyline {
                points: vec![
                    (center - above, y),
                    (center + above, y),
                    (center + below, y + bar),
                    (center - below, y + bar),
                    (center - above, y),
                ],
                class: BAR,
                series_index: None,
                style_index: None,
                tooltip: tip,
            }));
        }
        elements.push(Element::Text(Text {
            x: left - 8.0,
            y: y + bar / 2.0 + 4.0,
            class: "chartlet-label",
            anchor: TextAnchor::End,
            content: fit_text(
                &stage.label,
                label_width - 9.0,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/funnel/stages/{index}/label"),
            ),
        }));
        if let Some(text) = texts.get(index) {
            elements.push(Element::Text(Text {
                x: width - margin,
                y: y + bar / 2.0 + 4.0,
                class: "chartlet-label",
                anchor: TextAnchor::End,
                content: text.clone(),
            }));
        }
    }
    if let (false, Some(values)) = (pyramid, &values) {
        for (index, rate) in rates(values).into_iter().enumerate() {
            elements.push(Element::Text(Text {
                x: center,
                y: top + count(index + 1) * (bar + gap) - gap / 2.0 + 4.0,
                class: "chartlet-funnel-rate",
                anchor: TextAnchor::Middle,
                content: percent(spec, rate),
            }));
        }
    }
    let mut bottom = top + total + 16.0;
    if note {
        elements.push(Element::Text(Text {
            x: margin,
            y: bottom + 8.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                text::funnel_note(spec.locale),
                inner,
                LABEL_SIZE,
                metrics,
                warnings,
                "/funnel",
            ),
        }));
        bottom += 20.0;
    }
    Scene {
        width: spec.width,
        height: pixels(height.max(bottom + 8.0)),
        elements,
    }
}

/// The list of the stages with their values and rates.
fn stage_list(spec: &ChartSpec) -> String {
    let funnel = funnel(spec);
    let style = spec.number_style();
    let values = funnel.values();
    funnel
        .stages
        .iter()
        .enumerate()
        .map(|(index, stage)| match (&values, funnel.is_pyramid()) {
            (None, _) => stage.label.clone(),
            (Some(values), true) => {
                format!("{} {}", stage.label, format_value(values[index], style))
            }
            (Some(values), false) if index == 0 => {
                format!("{} {}", stage.label, format_value(values[index], style))
            }
            (Some(values), false) => format!(
                "{} {} ({})",
                stage.label,
                format_value(values[index], style),
                percent(spec, values[index] / values[index - 1])
            ),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// The funnel or pyramid in a sentence: its stages, and for a funnel the share that is left at the
/// end and the biggest drop.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let funnel = funnel(spec);
    let list = stage_list(spec);
    let stages = funnel.stages.len();
    let Some(values) = funnel.values().filter(|_| !funnel.is_pyramid()) else {
        return text::pyramid_summary(spec.locale, stages, &list);
    };
    let style = spec.number_style();
    let end = stages - 1;
    let drop = rates(&values)
        .into_iter()
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .filter(|(_, rate)| *rate < 1.0)
        .map(|(index, rate)| {
            (
                funnel.stages[index].label.as_str(),
                funnel.stages[index + 1].label.as_str(),
                percent(spec, rate),
            )
        });
    text::funnel_summary(
        spec.locale,
        stages,
        (
            (
                funnel.stages[0].label.as_str(),
                &format_value(values[0], style),
            ),
            (
                funnel.stages[end].label.as_str(),
                &format_value(values[end], style),
            ),
        ),
        &percent(spec, values[end] / values[0]),
        &list,
        drop.as_ref()
            .map(|(from, to, rate)| (*from, *to, rate.as_str())),
    )
}

/// One row per stage: its value, its share of the first stage and its rate from the stage before;
/// a pyramid has the position from the top and the value, if it has one.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let words = spec.locale.words();
    let funnel = funnel(spec);
    let style = spec.number_style();
    let values = funnel.values();
    let columns = if funnel.is_pyramid() {
        let mut columns = vec![words.level, words.position];
        if values.is_some() {
            columns.push(words.value);
        }
        columns
    } else {
        vec![words.stage, words.value, words.share, words.rate]
    };
    let rows = funnel
        .stages
        .iter()
        .enumerate()
        .map(|(index, stage)| {
            let mut row = vec![stage.label.clone()];
            match (&values, funnel.is_pyramid()) {
                (Some(values), false) => {
                    row.push(format_value(values[index], style));
                    row.push(percent(spec, values[index] / values[0]));
                    row.push(if index == 0 {
                        String::new()
                    } else {
                        percent(spec, values[index] / values[index - 1])
                    });
                }
                (values, _) => {
                    row.push((index + 1).to_string());
                    if let Some(values) = values {
                        row.push(format_value(values[index], style));
                    }
                }
            }
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: columns.into_iter().map(str::to_owned).collect(),
        rows,
    }
}
