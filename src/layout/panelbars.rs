//! Small multiples of bars: one panel per measure over the same categories, each with a value
//! axis of its own, so that measures in different units, such as requests per second and
//! megabytes, stand side by side.

use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Element, Rect, Text, TextAnchor},
    spec::ChartSpec,
};

use super::{
    LABEL_SIZE, NumericScale, PANEL_MARGIN, PlotArea, count, push_side_label, push_value_ticks,
    timechart::{MULTIPLES_MARGIN, multiples_header, panel_heads, push_panel_head},
    tooltip, warn_if_labels_omitted,
};
use crate::scene::Scene;

/// Room below a panel's bars for the ticks of its value axis and its unit.
const TICKS_BELOW: f64 = 52.0;
/// The widest the bars of a panel get, and the share of its row they take.
const MAX_BAR: f64 = 30.0;
const BAR_SHARE: f64 = 0.66;

/// The bars of one panel with the category names at its left and, when asked for, the values at
/// their ends; whether a value label had to be left out.
#[allow(clippy::too_many_arguments)]
fn push_bars(
    spec: &ChartSpec,
    pane_index: usize,
    scale: &NumericScale,
    plot: PlotArea,
    (cell_right, gutter): (f64, f64),
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> bool {
    let pane = &spec.panes[pane_index];
    let categories = &spec.categories;
    let mut omitted = false;
    let style = spec.pane_style(pane_index);
    let band = plot.height / count(categories.len());
    let thickness = (band * BAR_SHARE).min(MAX_BAR);
    let baseline = scale.map(scale.base(), plot.left, plot.left + plot.width);
    for (index, category) in categories.iter().enumerate() {
        let center = plot.top + band * (count(index) + 0.5);
        push_side_label(
            elements,
            category,
            (plot.left - 8.0, center),
            (gutter - 12.0, band),
            metrics,
            warnings,
            &format!("/categories/{index}"),
        );
        let Some(value) = pane.values[index] else {
            continue;
        };
        let value_x = scale.map(value, plot.left, plot.left + plot.width);
        elements.push(Element::Rect(Rect {
            x: baseline.min(value_x),
            y: center - thickness / 2.0,
            width: (baseline - value_x).abs(),
            height: thickness,
            class: "chartlet-bar",
            series_index: None,
            style_index: None,
            tooltip: Some(tooltip(category, value, style, pane.title.as_deref())),
        }));
        if spec.show_values {
            let content = super::format_value(value, style);
            let reach = metrics.width(&content, LABEL_SIZE);
            let (x, anchor) = if value >= 0.0 {
                (value_x + 6.0, TextAnchor::Start)
            } else {
                (value_x - 6.0, TextAnchor::End)
            };
            let fits = match anchor {
                TextAnchor::End => x - reach >= plot.left - 4.0,
                _ => x + reach <= cell_right - 2.0,
            };
            if fits {
                elements.push(Element::Text(Text {
                    x,
                    y: center + 4.0,
                    class: "chartlet-value",
                    anchor,
                    content,
                }));
            } else {
                omitted = true;
            }
        }
    }
    omitted
}

pub(super) fn layout_panel_bars(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let columns = usize::try_from(spec.multiples_columns()).expect("columns are limited");
    let panels = spec.panes.len();
    let rows = panels.div_ceil(columns);

    let mut elements = Vec::new();
    let grid_top = multiples_header(spec, &mut elements, warnings, metrics);
    let grid_bottom = height - 12.0;
    let cell_width = (width - 2.0 * MULTIPLES_MARGIN) / count(columns);
    let cell_height = (grid_bottom - grid_top) / count(rows);
    let dataset = spec.bar_multiples_dataset();
    // Every panel names its categories at its left: as wide as the longest name, within reason.
    let widest = dataset
        .categories
        .iter()
        .map(|category| metrics.width(category, LABEL_SIZE))
        .fold(0.0, f64::max);
    let gutter = (widest + 16.0)
        .clamp(48.0, (cell_width * 0.4).max(48.0))
        .ceil();
    // The longest value label of any panel fits beyond the longest bar.
    let label_room = if spec.show_values {
        spec.panes
            .iter()
            .enumerate()
            .flat_map(|(index, pane)| {
                let style = spec.pane_style(index);
                pane.values
                    .iter()
                    .flatten()
                    .map(move |value| super::format_value(*value, style))
            })
            .map(|label| metrics.width(&label, LABEL_SIZE) + 10.0)
            .fold(0.0, f64::max)
    } else {
        0.0
    };
    let plot_width = cell_width - gutter - f64::from(PANEL_MARGIN) - label_room;
    let (heads, head_height, title_lines) = panel_heads(spec, plot_width, warnings, metrics);
    let plot_height = cell_height - head_height - TICKS_BELOW;
    let mut omitted = false;

    for (pane_index, head) in heads.iter().enumerate() {
        let pane = &spec.panes[pane_index];
        let cell_left = MULTIPLES_MARGIN + cell_width * count(pane_index % columns);
        let cell_top = grid_top + cell_height * count(pane_index / columns);
        let plot = PlotArea {
            left: cell_left + gutter,
            top: cell_top + head_height,
            width: plot_width,
            height: plot_height,
            vertical_bars: false,
        };
        // The panel writes its ticks and values by its own value axis.
        let mut panel = spec.clone();
        panel.value_axis = pane.value_axis.clone();
        let scale = NumericScale::for_axis(
            pane.values.iter().flatten().copied(),
            true,
            &pane.value_axis,
        );
        push_panel_head(head, title_lines, plot, cell_top, &mut elements);
        if let Some(unit) = &pane.value_axis.unit {
            elements.push(Element::Text(Text {
                x: plot.left + plot.width / 2.0,
                y: plot.top + plot.height + 44.0,
                class: "chartlet-axis-title",
                anchor: TextAnchor::Middle,
                content: unit.clone(),
            }));
        }
        push_value_ticks(&mut elements, &panel, &scale, plot, metrics);

        omitted |= push_bars(
            spec,
            pane_index,
            &scale,
            plot,
            (cell_left + cell_width, gutter),
            &mut elements,
            warnings,
            metrics,
        );
    }
    warn_if_labels_omitted(omitted, warnings);
    if plot_height / count(dataset.categories.len()) < 14.0 {
        warnings.push(ChartWarning::new(
            "dense_chart",
            "/height",
            "the bars of the panels are less than 14 pixels apart; raise height or use more columns",
        ));
    }

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}
