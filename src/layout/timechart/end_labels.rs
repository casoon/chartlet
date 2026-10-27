//! The names at the ends of the lines, in place of a legend.

use super::{END_LABEL_GAP, frame::TimeFrame};
use crate::{
    error::ChartWarning,
    layout::{LABEL_LINE, LABEL_SIZE, fit_text},
    metrics::TextMetrics,
    scene::{Element, Text, TextAnchor},
    spec::{ChartSpec, LayerRef, Stack},
};

/// The widest name a line takes at its end, with the gap before it, at most a third of the chart.
pub(super) fn end_label_room(spec: &ChartSpec, width: f64, metrics: &impl TextMetrics) -> f64 {
    let widest = spec
        .data_layers()
        .map(|entry| metrics.width(&end_label(spec, entry), LABEL_SIZE))
        .fold(0.0, f64::max);
    (widest + END_LABEL_GAP).min(width / 3.0)
}

/// The name at the end of a line: its legend entry, so a modeled series says so.
fn end_label(spec: &ChartSpec, entry: LayerRef) -> String {
    let words = spec.locale.words();
    let name = entry.layer.name.as_deref().unwrap_or(words.value);
    if entry.layer.modeled {
        format!("{name} ({})", words.modeled)
    } else {
        name.to_owned()
    }
}

/// Names every line and area at its last observation, right of the plot. Names that would overlap
/// move apart, at least a line of text from each other, and stay inside their pane.
pub(super) fn push_end_labels(
    spec: &ChartSpec,
    frames: &[TimeFrame],
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let width = f64::from(spec.width);
    for (pane_index, frame) in frames.iter().enumerate() {
        let mut labels: Vec<(f64, String, String)> = spec
            .data_layers()
            .filter(|entry| entry.pane == pane_index)
            .filter_map(|entry| {
                let (_, value) = *spec.drawn_points(entry, frame.zone).last()?;
                let path = format!("/panes/{}/layers/{}/name", entry.pane, entry.local);
                // A stream is named in the middle of its band, not at its top edge.
                let at = match spec.stack_base(entry, frame.zone) {
                    Some(base) if spec.panes[entry.pane].stack == Some(Stack::Stream) => {
                        f64::midpoint(frame.y(value), frame.y(*base.last()?))
                    }
                    _ => frame.y(value),
                };
                Some((at + 4.0, end_label(spec, entry), path))
            })
            .collect();
        crate::sort::by(&mut labels, |a, b| a.0.total_cmp(&b.0));
        let plot = frame.plot;
        let mut floor = plot.top + LABEL_SIZE;
        for label in &mut labels {
            label.0 = label.0.max(floor);
            floor = label.0 + LABEL_LINE;
        }
        // Pushed past the bottom of the pane, the lowest names move back up together.
        let overflow = labels
            .last()
            .map_or(0.0, |last| (last.0 - (plot.top + plot.height)).max(0.0));
        let x = plot.left + plot.width + END_LABEL_GAP;
        for (y, name, path) in labels {
            elements.push(Element::Text(Text {
                x,
                y: y - overflow,
                class: "chartlet-legend",
                anchor: TextAnchor::Start,
                content: fit_text(&name, width - x - 4.0, LABEL_SIZE, metrics, warnings, &path),
            }));
        }
    }
}
