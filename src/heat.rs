//! Node colors from values: the boxes of flow charts and trees filled from a sequential scale of
//! six blues, light for the smallest value and dark for the largest, with a key below the
//! diagram. The colors are custom properties, the same in the light and the dark theme, and every
//! step fixes its text color so that labels keep at least 4.5:1 against their fill.

use crate::{
    layout::{LABEL_SIZE, count, format_value},
    metrics::TextMetrics,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::ChartSpec,
    text,
};

pub(crate) const STEPS: usize = 6;

pub(crate) const STYLE: &str = ".chartlet-root{--chartlet-heat-0:#cde0f5;--chartlet-heat-1:#a7c8ea;--chartlet-heat-2:#81afdf;--chartlet-heat-3:#5b97d3;--chartlet-heat-4:#0f66ac;--chartlet-heat-5:#064f89}.chartlet-heat-0 .chartlet-flow-node,.chartlet-heat-key-0{fill:var(--chartlet-heat-0)}.chartlet-heat-1 .chartlet-flow-node,.chartlet-heat-key-1{fill:var(--chartlet-heat-1)}.chartlet-heat-2 .chartlet-flow-node,.chartlet-heat-key-2{fill:var(--chartlet-heat-2)}.chartlet-heat-3 .chartlet-flow-node,.chartlet-heat-key-3{fill:var(--chartlet-heat-3)}.chartlet-heat-4 .chartlet-flow-node,.chartlet-heat-key-4{fill:var(--chartlet-heat-4)}.chartlet-heat-5 .chartlet-flow-node,.chartlet-heat-key-5{fill:var(--chartlet-heat-5)}.chartlet-heat-0 text,.chartlet-heat-1 text,.chartlet-heat-2 text,.chartlet-heat-3 text{fill:#172033}.chartlet-heat-4 text,.chartlet-heat-5 text{fill:#fff}.chartlet-heat-key{stroke:var(--chartlet-grid);stroke-width:1}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";

const KEY_CLASSES: [&str; STEPS] = [
    "chartlet-heat-key chartlet-heat-key-0",
    "chartlet-heat-key chartlet-heat-key-1",
    "chartlet-heat-key chartlet-heat-key-2",
    "chartlet-heat-key chartlet-heat-key-3",
    "chartlet-heat-key chartlet-heat-key-4",
    "chartlet-heat-key chartlet-heat-key-5",
];
const SWATCH: (f64, f64) = (28.0, 12.0);
/// The height the key adds below a diagram.
const KEY_HEIGHT: f64 = 52.0;

/// The step of every node with a value, in proportion to where its value lies between the
/// smallest and the largest; the middle step when all values are equal.
pub(crate) fn steps(values: &[Option<f64>]) -> Vec<Option<usize>> {
    let Some((low, high)) = range(values) else {
        return vec![None; values.len()];
    };
    values
        .iter()
        .map(|value| {
            value.map(|value| {
                if high > low {
                    let share = (value - low) / (high - low);
                    // A share of 0 to 1 times the last step, rounded: a small whole number.
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let step = (share * count(STEPS - 1)).round() as usize;
                    step
                } else {
                    STEPS / 2
                }
            })
        })
        .collect()
}

/// The smallest and the largest of the values that are there.
pub(crate) fn range(values: &[Option<f64>]) -> Option<(f64, f64)> {
    let mut present = values.iter().flatten().copied();
    let first = present.next()?;
    Some(present.fold((first, first), |(low, high), value| {
        (low.min(value), high.max(value))
    }))
}

/// The class a node's group gains for its step.
pub(crate) fn class(step: usize) -> String {
    format!(" chartlet-heat-{step}")
}

/// What the values stand for: the diagram's `valueLabel`, or the word for a value.
pub(crate) fn name(spec: &ChartSpec, label: Option<&str>) -> String {
    label.map_or_else(|| spec.locale.words().value.to_owned(), str::to_owned)
}

/// The key below a diagram: its name, then the six steps from the smallest value to the largest
/// with both ends written under them. The canvas grows by its height.
pub(crate) fn push_key(
    scene: &mut Scene,
    spec: &ChartSpec,
    (label, values): (&str, &[Option<f64>]),
    margin: f64,
    metrics: &impl TextMetrics,
) {
    let Some((low, high)) = range(values) else {
        return;
    };
    let top = f64::from(scene.height) + 4.0;
    let style = spec.number_style();
    let name_width = metrics.width(label, LABEL_SIZE);
    scene.elements.push(Element::Text(Text {
        x: margin,
        y: top + 14.0,
        class: "chartlet-legend",
        anchor: TextAnchor::Start,
        content: label.to_owned(),
    }));
    let left = margin + name_width + 12.0;
    for (step, class) in KEY_CLASSES.iter().enumerate() {
        scene.elements.push(Element::Rect(Rect {
            x: left + SWATCH.0 * count(step),
            y: top + 4.0,
            width: SWATCH.0,
            height: SWATCH.1,
            class,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
    for (x, value, anchor) in [
        (left, low, TextAnchor::Start),
        (left + SWATCH.0 * count(STEPS), high, TextAnchor::End),
    ] {
        scene.elements.push(Element::Text(Text {
            x,
            y: top + 32.0,
            class: "chartlet-legend",
            anchor,
            content: format_value(value, style),
        }));
    }
    let right = left + SWATCH.0 * count(STEPS) + margin;
    scene.width = scene.width.max(crate::diagram::pixels(right));
    scene.height = crate::diagram::pixels(f64::from(scene.height) + KEY_HEIGHT);
}

/// The sentence that says what the colors show and gives every value, nodes in reading order as
/// `(label, value)`.
pub(crate) fn sentence(spec: &ChartSpec, name: &str, nodes: &[(&str, f64)]) -> String {
    let style = spec.number_style();
    let values: Vec<Option<f64>> = nodes.iter().map(|(_, value)| Some(*value)).collect();
    let (low, high) = range(&values).unwrap_or((0.0, 0.0));
    let listed: Vec<String> = nodes
        .iter()
        .map(|(label, value)| format!("{label} {}", format_value(*value, style)))
        .collect();
    text::node_values(
        spec.locale,
        name,
        (&format_value(low, style), &format_value(high, style)),
        &listed,
    )
}

#[cfg(test)]
mod tests {
    use super::steps;

    #[test]
    fn values_spread_over_the_steps_in_proportion() {
        assert_eq!(
            steps(&[Some(0.0), Some(50.0), None, Some(100.0)]),
            [Some(0), Some(3), None, Some(5)]
        );
        assert_eq!(steps(&[Some(7.0), Some(7.0)]), [Some(3), Some(3)]);
        assert_eq!(steps(&[None]), [None]);
    }
}
