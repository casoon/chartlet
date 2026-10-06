//! Waterfalls: a running total that rises and falls step by step. A delta is a bar between the
//! total before and the total after it, a start and a total are bars from zero; a dashed line
//! carries the total from one bar to the next. The description and the data table list every
//! step with its change and the total after it.

use crate::{
    DataTable,
    error::ChartWarning,
    layout::{
        LABEL_SIZE, NumericScale, PlotArea, WithReserve, add_bottom_category_title, base_elements,
        base_elements_with_title, count, format_value, horizontal_title, push_category_label,
        push_side_label, title_extra, warn_if_labels_omitted,
    },
    metrics::TextMetrics,
    scene::{Element, Line, Rect, Scene, Text, TextAnchor},
    spec::WaterfallBar,
    spec::{ChartSpec, Orientation, StepKind, WaterfallSpec},
    text,
};

fn waterfall(spec: &ChartSpec) -> &WaterfallSpec {
    spec.waterfall
        .as_ref()
        .expect("validated waterfalls carry a waterfall block")
}

/// A change with its sign: `+12`, `−5`; a start and a total as they are.
fn written(spec: &ChartSpec, bar: &WaterfallBar) -> String {
    let text = format_value(bar.value, spec.number_style());
    if bar.kind == StepKind::Delta && bar.value > 0.0 {
        format!("+{text}")
    } else {
        text
    }
}

fn class(bar: &WaterfallBar) -> &'static str {
    match bar.kind {
        StepKind::Delta if bar.value < 0.0 => "chartlet-wf-down",
        StepKind::Delta => "chartlet-wf-up",
        StepKind::Start | StepKind::Total => "chartlet-wf-total",
    }
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let bars = waterfall(spec).bars();
    let values = bars.iter().flat_map(|bar| [bar.from, bar.to]);
    let scale = NumericScale::for_axis(values, true, &spec.value_axis);
    let elements = match spec.orientation {
        Orientation::Horizontal => layout_horizontal(spec, &bars, &scale, warnings, metrics),
        Orientation::Vertical => layout_vertical(spec, &bars, &scale, warnings, metrics),
    };
    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// One bar with its tooltip. `along` maps a value onto the value axis; `across` is the leading
/// edge and the thickness of the bar.
fn push_bar(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (index, bar): (usize, &WaterfallBar),
    orientation: Orientation,
    along: &impl Fn(f64) -> f64,
    across: (f64, f64),
) {
    let (low, high) = (along(bar.from), along(bar.to));
    let (start, extent) = (low.min(high), (high - low).abs().max(1.5));
    let (x, y, width, height) = match orientation {
        Orientation::Horizontal => (start, across.0, extent, across.1),
        Orientation::Vertical => (across.0, start, across.1, extent),
    };
    let label = &waterfall(spec).steps[index].label;
    elements.push(Element::Rect(Rect {
        x,
        y,
        width,
        height,
        class: class(bar),
        series_index: None,
        style_index: None,
        tooltip: Some(format!(
            "{label}: {}, {} {}",
            written(spec, bar),
            spec.locale.words().total.to_lowercase(),
            format_value(bar.running, spec.number_style())
        )),
    }));
}

/// The dashed line that carries the running total from one bar to the next.
fn push_link(
    elements: &mut Vec<Element>,
    orientation: Orientation,
    level: f64,
    (from, to): (f64, f64),
) {
    elements.push(Element::Line(match orientation {
        Orientation::Horizontal => Line {
            x1: level,
            y1: from,
            x2: level,
            y2: to,
            class: "chartlet-wf-link",
        },
        Orientation::Vertical => Line {
            x1: from,
            y1: level,
            x2: to,
            y2: level,
            class: "chartlet-wf-link",
        },
    }));
}

fn layout_horizontal(
    spec: &ChartSpec,
    bars: &[WaterfallBar],
    scale: &NumericScale,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let steps = &waterfall(spec).steps;
    let widest = steps
        .iter()
        .map(|step| metrics.width(&step.label, LABEL_SIZE))
        .fold(0.0, f64::max);
    let left = (widest + 29.0).clamp(88.0, 210.0);
    let right = bars
        .iter()
        .map(|bar| WithReserve(metrics).width(&written(spec, bar), LABEL_SIZE))
        .fold(0.0, f64::max)
        .clamp(8.0, 120.0)
        + 24.0;
    let bottom = if spec.value_axis.title.is_some() {
        56.0
    } else {
        36.0
    };
    let title = horizontal_title(spec, (left, width - left - right), metrics);
    let top = 78.0 + title_extra(spec, title.1, metrics);
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: crate::layout::plot_height(height - top - bottom, warnings),
        vertical_bars: false,
    };
    let mut elements = base_elements_with_title(spec, scale, plot, title, warnings, metrics);
    let band = plot.height / count(bars.len());
    let thickness = (band * 0.6).clamp(4.0, 32.0);
    let along = |value| scale.map(value, plot.left, plot.left + plot.width);
    for (index, bar) in bars.iter().enumerate() {
        let center = plot.top + band * (count(index) + 0.5);
        if let Some(next) = bars.get(index + 1) {
            push_link(
                &mut elements,
                Orientation::Horizontal,
                along(bar.running),
                (center + thickness / 2.0, center + band - thickness / 2.0),
            );
            let _ = next;
        }
        push_bar(
            &mut elements,
            spec,
            (index, bar),
            Orientation::Horizontal,
            &along,
            (center - thickness / 2.0, thickness),
        );
        let (reach, anchor) = if bar.to >= bar.from {
            (along(bar.to) + 8.0, TextAnchor::Start)
        } else {
            (along(bar.from) + 8.0, TextAnchor::Start)
        };
        elements.push(Element::Text(Text {
            x: reach.max(along(bar.from.max(bar.to)) + 8.0),
            y: center + 4.0,
            class: "chartlet-value",
            anchor,
            content: written(spec, bar),
        }));
        push_side_label(
            &mut elements,
            &steps[index].label,
            (plot.left - 12.0, center),
            (plot.left - 28.0, band),
            metrics,
            warnings,
            &format!("/waterfall/steps/{index}/label"),
        );
    }
    elements
}

fn layout_vertical(
    spec: &ChartSpec,
    bars: &[WaterfallBar],
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
    let top = 78.0 + title_extra(spec, width - left - right, metrics);
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: crate::layout::plot_height(height - top - bottom, warnings),
        vertical_bars: true,
    };
    let mut elements = base_elements(spec, scale, plot, warnings, metrics);
    let band = plot.width / count(bars.len());
    let thickness = (band * 0.6).clamp(6.0, 56.0);
    let along = |value| scale.map(value, plot.top + plot.height, plot.top);
    let steps = &waterfall(spec).steps;
    let mut omitted = false;
    for (index, bar) in bars.iter().enumerate() {
        let center = plot.left + band * (count(index) + 0.5);
        if bars.get(index + 1).is_some() {
            push_link(
                &mut elements,
                Orientation::Vertical,
                along(bar.running),
                (center + thickness / 2.0, center + band - thickness / 2.0),
            );
        }
        push_bar(
            &mut elements,
            spec,
            (index, bar),
            Orientation::Vertical,
            &along,
            (center - thickness / 2.0, thickness),
        );
        let content = written(spec, bar);
        if metrics.width(&content, LABEL_SIZE) <= band - 4.0 {
            // Beyond the end of the bar in the direction of its change.
            let falling = bar.to < bar.from;
            elements.push(Element::Text(Text {
                x: center,
                y: if falling {
                    along(bar.to) + 16.0
                } else {
                    along(bar.to) - 8.0
                },
                class: "chartlet-value",
                anchor: TextAnchor::Middle,
                content,
            }));
        } else {
            omitted = true;
        }
        push_category_label(
            &mut elements,
            &steps[index].label,
            (center, plot.top + plot.height + 24.0),
            (band - 8.0).max(20.0),
            metrics,
            warnings,
            &format!("/waterfall/steps/{index}/label"),
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
    elements
}

/// The waterfall in sentences: its start and end, its biggest rise and fall, and its totals.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let waterfall = waterfall(spec);
    let bars = waterfall.bars();
    let show = |value| format_value(value, spec.number_style());
    let deltas: Vec<usize> = (0..bars.len())
        .filter(|index| bars[*index].kind == StepKind::Delta)
        .collect();
    let rise = deltas
        .iter()
        .copied()
        .filter(|index| bars[*index].value > 0.0)
        .max_by(|a, b| bars[*a].value.total_cmp(&bars[*b].value));
    let fall = deltas
        .iter()
        .copied()
        .filter(|index| bars[*index].value < 0.0)
        .min_by(|a, b| bars[*a].value.total_cmp(&bars[*b].value));
    let label = |index: usize| waterfall.steps[index].label.as_str();
    text::waterfall_summary(
        spec.locale,
        (bars.len(), &show(bars[bars.len() - 1].running)),
        rise.map(|index| (label(index), written(spec, &bars[index]))),
        fall.map(|index| (label(index), written(spec, &bars[index]))),
    )
}

/// One row per step: its kind, its change and the running total after it.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let waterfall = waterfall(spec);
    let words = spec.locale.words();
    let rows = waterfall
        .bars()
        .iter()
        .enumerate()
        .map(|(index, bar)| {
            let kind = match bar.kind {
                StepKind::Delta => words.step_delta,
                StepKind::Start => words.step_start,
                StepKind::Total => words.total,
            };
            vec![
                waterfall.steps[index].label.clone(),
                kind.to_owned(),
                written(spec, bar),
                format_value(bar.running, spec.number_style()),
            ]
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: vec![
            words.step.to_owned(),
            words.message_kind.to_owned(),
            words.value.to_owned(),
            words.total.to_owned(),
        ],
        rows,
    }
}
