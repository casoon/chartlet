//! Small multiples: the panel heads, the grid of panels and the gutter they share.

use super::{
    MULTIPLES_MARGIN,
    axes::{fitted_unit, push_time_ticks, push_value_grid},
    frame::{Detail, TimeFrame, time_axis, time_scale},
    marks::draw_pane,
    time_tick_spacing,
};
use crate::{
    error::ChartWarning,
    layout::{
        LABEL_SIZE, LEGEND_HEIGHT, LEGEND_ROW, PANEL_GUTTER, PANEL_MARGIN, PlotArea, count,
        fit_text,
        legend::{add_layer_legend, legend_rows},
        panel_plot_pixels,
        title::{push_title, title_extra},
    },
    metrics::TextMetrics,
    scene::{Element, Scene, Text, TextAnchor},
    spec::ChartSpec,
    time::{Precision, TimeZone},
};

/// The title, the legend and the shared value axis title above a small-multiples grid; returns
/// where the grid starts.
pub(in crate::layout) fn multiples_header(
    spec: &ChartSpec,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> f64 {
    let width = f64::from(spec.width);
    push_title(
        elements,
        spec,
        MULTIPLES_MARGIN,
        width - 2.0 * MULTIPLES_MARGIN,
        metrics,
        warnings,
    );

    let head = title_extra(spec, width - 2.0 * MULTIPLES_MARGIN, metrics);
    let mut cursor = LEGEND_ROW + head;
    if spec.series_names().len() > 1 {
        add_layer_legend(
            spec,
            LEGEND_ROW + head,
            MULTIPLES_MARGIN,
            width - 2.0 * MULTIPLES_MARGIN,
            elements,
            warnings,
            metrics,
        );
        cursor += count(legend_rows(spec, width - 2.0 * MULTIPLES_MARGIN, metrics)) * LEGEND_HEIGHT;
    }
    if let Some(axis_title) = &spec.value_axis.title {
        elements.push(Element::Text(Text {
            x: MULTIPLES_MARGIN,
            y: cursor + 10.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Start,
            content: fit_text(
                axis_title,
                width - 2.0 * MULTIPLES_MARGIN,
                LABEL_SIZE,
                metrics,
                warnings,
                "/valueAxis/title",
            ),
        }));
        cursor += 20.0;
    }
    cursor + 8.0
}

/// A text of a panel's heading on up to two lines of `width`; only the second is shortened if
/// it still does not fit.
fn panel_lines(
    text: &str,
    width: f64,
    size: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) -> Vec<String> {
    match super::title::two_lines(text, width, size, metrics) {
        Some((first, rest)) => vec![
            first.to_owned(),
            fit_text(rest, width, size, metrics, warnings, path),
        ],
        None => vec![fit_text(text, width, size, metrics, warnings, path)],
    }
}

/// The lines of a panel's title and of its note, and the class of the note.
pub(in crate::layout) type PanelHead = (Vec<String>, Vec<String>, &'static str);

/// Height of a line of a panel title and of a panel note.
const PANEL_TITLE_LINE: f64 = 16.0;
const PANEL_NOTE_LINE: f64 = 15.0;

/// The heading of every panel: its title and its note, each on up to two lines. Every panel
/// reserves as many lines as the longest heading needs, so that the plots stay aligned.
pub(in crate::layout) fn panel_heads(
    spec: &ChartSpec,
    width: f64,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> (Vec<PanelHead>, f64, usize) {
    let heads: Vec<PanelHead> = spec
        .panes
        .iter()
        .enumerate()
        .map(|(index, pane)| {
            let title = panel_lines(
                pane.title.as_deref().unwrap_or_default(),
                width,
                13.0,
                metrics,
                warnings,
                &format!("/panes/{index}/title"),
            );
            let note = pane.note.as_deref().map_or_else(Vec::new, |note| {
                panel_lines(
                    note,
                    width,
                    LABEL_SIZE,
                    metrics,
                    warnings,
                    &format!("/panes/{index}/note"),
                )
            });
            let class = if pane.note_emphasis {
                "chartlet-panel-note chartlet-panel-note-strong"
            } else {
                "chartlet-panel-note"
            };
            (title, note, class)
        })
        .collect();
    let title_lines = heads
        .iter()
        .map(|(title, ..)| title.len())
        .max()
        .unwrap_or(1);
    let note_lines = heads
        .iter()
        .map(|(_, note, _)| note.len())
        .max()
        .unwrap_or(0);
    let height = 14.0 + PANEL_TITLE_LINE * count(title_lines) + PANEL_NOTE_LINE * count(note_lines);
    (heads, height, title_lines)
}

/// The heading of one panel above its plot, starting at `cell_top`.
pub(in crate::layout) fn push_panel_head(
    (title, note, note_class): &PanelHead,
    title_lines: usize,
    plot: PlotArea,
    cell_top: f64,
    elements: &mut Vec<Element>,
) {
    for (index, line) in title.iter().enumerate() {
        elements.push(Element::Text(Text {
            x: plot.left,
            y: cell_top + 18.0 + PANEL_TITLE_LINE * count(index),
            class: "chartlet-panel-title",
            anchor: TextAnchor::Start,
            content: line.clone(),
        }));
    }
    let below = cell_top + 18.0 + PANEL_TITLE_LINE * count(title_lines - 1);
    for (index, line) in note.iter().enumerate() {
        elements.push(Element::Text(Text {
            x: plot.left,
            y: below + PANEL_NOTE_LINE * count(index + 1),
            class: note_class,
            anchor: TextAnchor::Start,
            content: line.clone(),
        }));
    }
}

/// The title of the shared time axis below the grid of small multiples.
fn push_multiples_time_title(
    spec: &ChartSpec,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    if let Some(axis_title) = &spec.time_axis.title {
        elements.push(Element::Text(Text {
            x: width / 2.0,
            y: height - 14.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: fit_text(
                axis_title,
                width - 2.0 * MULTIPLES_MARGIN,
                LABEL_SIZE,
                metrics,
                warnings,
                "/timeAxis/title",
            ),
        }));
    }
}

pub(in crate::layout) fn layout_multiples(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let zone = spec.time_zone().unwrap_or_default();
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let columns = spec.multiples_columns();
    let panels = spec.panes.len();
    let rows = panels.div_ceil(usize::try_from(columns).expect("columns are limited"));

    let mut elements = Vec::new();
    let grid_top = multiples_header(spec, &mut elements, warnings, metrics);
    let grid_bottom = height
        - if spec.time_axis.title.is_some() {
            36.0
        } else {
            12.0
        };
    let cell_width = (width - 2.0 * MULTIPLES_MARGIN) / f64::from(columns);
    let cell_height = (grid_bottom - grid_top) / count(rows);
    let panel_gutter = multiples_gutter(spec, zone, warnings, metrics);
    let (heads, head_height, title_lines) = panel_heads(
        spec,
        cell_width - panel_gutter - f64::from(PANEL_MARGIN),
        warnings,
        metrics,
    );

    let (span, slots) = time_axis(spec, zone);
    let precision = spec.time_precision(zone);
    let shared = time_scale(spec, zone, None);
    let max_ticks = usize::try_from(
        (panel_plot_pixels(spec.width, columns) / time_tick_spacing(spec.observed_precision(zone)))
            .max(2),
    )
    .expect("a usize is at least 32 bits wide");

    for (pane_index, head) in heads.iter().enumerate() {
        let column = pane_index % usize::try_from(columns).expect("columns are limited");
        let row = pane_index / usize::try_from(columns).expect("columns are limited");
        let cell_left = MULTIPLES_MARGIN + cell_width * count(column);
        let cell_top = grid_top + cell_height * count(row);
        let plot = PlotArea {
            left: cell_left + panel_gutter,
            top: cell_top + head_height,
            width: cell_width - panel_gutter - f64::from(PANEL_MARGIN),
            height: cell_height - head_height - 28.0,
            vertical_bars: true,
        };
        let frame = TimeFrame {
            plot,
            span,
            slots: slots.clone(),
            zone,
            precision,
            tick_step: spec.time_axis.step,
            sub_day: spec.observed_precision(zone) == Precision::Minute,
            unit: spec.axis_unit(pane_index).map(str::to_owned),
            scale: if spec.independent_axes {
                time_scale(spec, zone, Some(pane_index))
            } else {
                shared
            },
            style: spec.number_style(),
            reversed: spec.time_axis.reverse,
        };

        push_panel_head(head, title_lines, plot, cell_top, &mut elements);
        let unit = fitted_unit(
            spec.axis_unit(pane_index),
            &frame,
            cell_left,
            "/valueAxis/unit",
            warnings,
            metrics,
        );
        push_value_grid(&frame, unit.as_deref(), &mut elements);
        push_time_ticks(&frame, max_ticks, true, width, metrics, &mut elements);
        elements.push(frame.hook(pane_index));
        draw_pane(
            spec,
            pane_index,
            &frame,
            Detail::Compact,
            &mut elements,
            warnings,
            metrics,
        );
    }

    if cell_height - head_height - 28.0 < 60.0 {
        warnings.push(ChartWarning::new(
            "dense_chart",
            "/height",
            "the panels are less than 60 pixels tall; raise height or use more columns",
        ));
    }

    push_multiples_time_title(spec, &mut elements, warnings, metrics);

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// The gutter left of each panel of small multiples: the regular panel gutter, wider where the
/// tick labels or the unit after the top one need it, at most 40 % of a panel.
fn multiples_gutter(
    spec: &ChartSpec,
    zone: TimeZone,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> f64 {
    let gutter = f64::from(PANEL_GUTTER);
    let style = spec.number_style();
    // Panels on axes of their own each have their ticks; otherwise they share one scale.
    let panes: Vec<Option<usize>> = if spec.independent_axes {
        (0..spec.panes.len()).map(Some).collect()
    } else {
        vec![None]
    };
    let labels: Vec<String> = panes
        .into_iter()
        .flat_map(|pane| {
            let scale = time_scale(spec, zone, pane);
            scale
                .ticks()
                .map(move |value| scale.tick_label(value, style))
                .collect::<Vec<_>>()
        })
        .collect();
    let width = |text: &str| super::WithReserve(metrics).width(text, LABEL_SIZE);
    let plain = labels.iter().map(|label| width(label)).fold(0.0, f64::max);
    let with_unit = spec.value_axis.unit.as_deref().map_or(plain, |unit| {
        labels
            .iter()
            .map(|label| width(&format!("{label} {unit}")))
            .fold(0.0, f64::max)
    });
    // At most 40 % of a panel, so that its plot keeps its width; the unit gives way first.
    let cell =
        (f64::from(spec.width) - 2.0 * MULTIPLES_MARGIN) / f64::from(spec.multiples_columns());
    let most = (cell * 0.4).max(gutter);
    if plain + 16.0 > most {
        warnings.push(ChartWarning::new(
            "dense_chart",
            "/columns",
            "the value tick labels need more room than the panels leave them; use fewer columns or a wider chart",
        ));
    }
    gutter.max(with_unit + 16.0).min(most)
}
