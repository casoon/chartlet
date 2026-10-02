//! Warming stripes: one colored stripe per year, on the diverging scale around a reference value.
//! The stripes carry no axis; the years at both ends, the tooltips, the description and the data
//! table carry the numbers.

use std::fmt::Write as _;

use crate::{
    diverging,
    error::ChartWarning,
    layout::{LABEL_SIZE, count, format_value, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::ChartSpec,
    text,
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
    // Stretched, the stripes fill the canvas and nothing else is drawn.
    let (margin, top, bottom) = if stripes.stretch {
        (0.0, 0.0, 0.0)
    } else {
        (
            MARGIN,
            56.0 + title_extra(spec, width - 2.0 * MARGIN, metrics),
            if stripes.year_labels { 40.0 } else { 24.0 },
        )
    };
    let plot_width = width - 2.0 * margin;
    let plot_height = crate::layout::plot_height(height - top - bottom, warnings);
    let stripe = plot_width / count(stripes.values.len());
    let scale = stripes.diverging();

    let mut elements = Vec::new();
    if !stripes.stretch {
        push_title(&mut elements, spec, MARGIN, plot_width, metrics, warnings);
    }

    let last = stripes.values.len() - 1;
    for (index, (year, value)) in stripes.years().zip(&stripes.values).enumerate() {
        let Some(value) = value else {
            continue;
        };
        elements.push(Element::Rect(Rect {
            x: margin + stripe * count(index),
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
                format_value(*value, spec.number_style())
            )),
        }));
    }

    if stripes.year_labels && !stripes.stretch {
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

    if stripe < 1.0 && !stripes.stretch {
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
    let show = |value| format_value(value, spec.number_style());
    let words = spec.locale.words();
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
    let scale = text::diverging_scale(
        spec.locale,
        &show(scale.reference),
        &show(scale.min),
        &show(scale.max),
    );
    let mut description = text::stripes_opening(
        spec.locale,
        stripes.first_year,
        stripes.years().last().unwrap_or(stripes.first_year),
        &scale,
    );
    write!(
        description,
        " {}: {} ({}). {}: {} ({}).",
        words.lowest,
        show(lowest.1),
        lowest.0,
        words.highest,
        show(highest.1),
        highest.0,
    )
    .expect("writing to String cannot fail");
    description.push_str(&text::years_without_value(
        spec.locale,
        stripes.values.len() - years.len(),
    ));
    description
}
