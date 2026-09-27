//! Warming stripes: one colored stripe per year, on the diverging scale around a reference value.
//! The stripes carry no axis; the years at both ends, the tooltips, the description and the data
//! table carry the numbers.

use std::fmt::Write as _;

use crate::{
    diverging,
    error::ChartWarning,
    layout::{LABEL_SIZE, count, fit_text, format_value},
    metrics::TextMetrics,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, ValueFormat},
};

/// Margin around the stripes, left and right.
const MARGIN: f64 = 24.0;
/// Overlap of neighbouring stripes, so that anti-aliasing leaves no hairline between them.
const OVERLAP: f64 = 0.5;

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let stripes = spec
        .stripes
        .as_ref()
        .expect("validated stripes charts carry a stripes block");
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let top = 56.0;
    let bottom = if stripes.year_labels { 40.0 } else { 24.0 };
    let plot_width = width - 2.0 * MARGIN;
    let plot_height = height - top - bottom;
    let stripe = plot_width / count(stripes.values.len());
    let scale = stripes.diverging();

    let mut elements = vec![Element::Text(Text {
        x: MARGIN,
        y: 30.0,
        class: "chartlet-title",
        anchor: TextAnchor::Start,
        content: fit_text(&spec.title, plot_width, 22.0, metrics, warnings, "/title"),
    })];

    let last = stripes.values.len() - 1;
    for (index, (year, value)) in stripes.years().zip(&stripes.values).enumerate() {
        let Some(value) = value else {
            continue;
        };
        elements.push(Element::Rect(Rect {
            x: MARGIN + stripe * count(index),
            y: top,
            width: if index == last {
                stripe
            } else {
                stripe + OVERLAP
            },
            height: plot_height,
            class: diverging::CLASSES[scale.step(*value)],
            series_index: None,
            style_index: None,
            tooltip: Some(format!(
                "{year}: {}",
                format_value(*value, ValueFormat::Number)
            )),
        }));
    }

    if stripes.year_labels {
        let first = stripes.first_year;
        let last_year = stripes.years().last().unwrap_or(first);
        for (x, anchor, year) in [
            (MARGIN, TextAnchor::Start, first),
            (MARGIN + plot_width, TextAnchor::End, last_year),
        ] {
            elements.push(Element::Text(Text {
                x,
                y: top + plot_height + LABEL_SIZE + 10.0,
                class: "chartlet-tick",
                anchor,
                content: year.to_string(),
            }));
        }
    }

    if stripe < 1.0 {
        warnings.push(ChartWarning::new(
            "dense_chart",
            "/stripes/values",
            "the stripes are narrower than one pixel at this width; raise width or shorten the series",
        ));
    }

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// What the stripes show in one sentence: the years, the scale, and the extremes.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let stripes = spec
        .stripes
        .as_ref()
        .expect("validated stripes charts carry a stripes block");
    let scale = stripes.diverging();
    let show = |value| format_value(value, ValueFormat::Number);
    let years: Vec<(i32, f64)> = stripes
        .years()
        .zip(&stripes.values)
        .filter_map(|(year, value)| value.map(|value| (year, value)))
        .collect();
    let lowest = years
        .iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("validated stripes hold a value");
    let highest = years
        .iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("validated stripes hold a value");
    let mut description = format!(
        "Warming stripes from {} to {}, one stripe per year, on a diverging color scale around {} with its outermost steps at {} and {}. Lowest: {} ({}). Highest: {} ({}).",
        stripes.first_year,
        stripes.years().last().unwrap_or(stripes.first_year),
        show(scale.reference),
        show(scale.min),
        show(scale.max),
        show(lowest.1),
        lowest.0,
        show(highest.1),
        highest.0,
    );
    match stripes.values.len() - years.len() {
        0 => {}
        1 => description.push_str(" 1 year has no value."),
        missing => write!(description, " {missing} years have no value.")
            .expect("writing to String cannot fail"),
    }
    description
}
