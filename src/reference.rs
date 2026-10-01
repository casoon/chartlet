//! Reference lines across a bar chart: a value every bar is read against, such as an average or
//! a target. A line runs across the whole plot at its value and carries its label; its value
//! widens the value axis, so that it never falls outside the plot.

use crate::{
    error::ChartWarning,
    layout::{LABEL_SIZE, NumericScale, PlotArea, fit_text, format_value},
    metrics::TextMetrics,
    scene::{Element, Polyline, Text, TextAnchor},
    spec::ChartSpec,
};

/// The values the value axis has to reach for the reference lines.
pub(crate) fn values(spec: &ChartSpec) -> impl Iterator<Item = f64> + '_ {
    spec.references.iter().map(|reference| reference.value)
}

/// Draws every reference line at `under`, the place of the first bar in `elements`, so that bars
/// and their value labels stay readable on top of it; its label goes on top of everything: above
/// the line at its right end on vertical bars, beside the line just above the plot on horizontal
/// bars.
pub(crate) fn push(
    spec: &ChartSpec,
    scale: NumericScale,
    plot: PlotArea,
    under: usize,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let right = plot.left + plot.width;
    let bottom = plot.top + plot.height;
    let mut lines = Vec::new();
    for (index, reference) in spec.references.iter().enumerate() {
        let path = format!("/references/{index}/label");
        let (points, x, y, anchor, room) = if plot.vertical_bars {
            let y = scale.map(reference.value, bottom, plot.top);
            (
                vec![(plot.left, y), (right, y)],
                right - 4.0,
                y - 6.0,
                TextAnchor::End,
                plot.width / 2.0,
            )
        } else {
            let x = scale.map(reference.value, plot.left, right);
            // A line in the right part of the plot takes its label on its left, so the label
            // stays inside the chart.
            let (label_x, anchor) = if x > plot.left + plot.width * 0.7 {
                (x - 4.0, TextAnchor::End)
            } else {
                (x + 4.0, TextAnchor::Start)
            };
            (
                vec![(x, plot.top), (x, bottom)],
                label_x,
                plot.top - 6.0,
                anchor,
                plot.width * 0.4,
            )
        };
        lines.push(Element::Polyline(Polyline {
            points,
            class: "chartlet-rule",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: Some(format!(
                "{}: {}",
                reference.label,
                format_value(reference.value, spec.number_style())
            )),
        }));
        elements.push(Element::Text(Text {
            x,
            y,
            class: "chartlet-rule-label",
            anchor,
            content: fit_text(&reference.label, room, LABEL_SIZE, metrics, warnings, &path),
        }));
    }
    elements.splice(under..under, lines);
}
