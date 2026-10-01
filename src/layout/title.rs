use super::{PLOT_MARGIN, TITLE_LINE, count, fit_text};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Element, Text, TextAnchor},
    spec::ChartSpec,
};

const TITLE_SIZE: f64 = 22.0;
/// Height of the drawn title with its spacing; a chart without a drawn title gains it.
const TITLE_BLOCK: f64 = 44.0;

/// Breaks `text` that is wider than `max_width` into two lines at a space: the longest run of
/// whole words that fits, and the rest, which may still be too wide. `None` when the text fits as
/// it is, or when not even its first word fits; such a text stays on one line.
pub(super) fn two_lines<'a>(
    text: &'a str,
    max_width: f64,
    font_size: f64,
    metrics: &impl TextMetrics,
) -> Option<(&'a str, &'a str)> {
    if metrics.width(text, font_size) <= max_width {
        return None;
    }
    text.match_indices(' ')
        .map(|(index, _)| (text[..index].trim_end(), text[index + 1..].trim_start()))
        .filter(|(first, rest)| !first.is_empty() && !rest.is_empty())
        .take_while(|(first, _)| metrics.width(first, font_size) <= max_width)
        .last()
}

/// How far whatever sits below the title moves down: by a second line when the title wraps, see
/// [`push_title`], and up into its place when the title is not drawn.
pub(crate) fn title_extra(spec: &ChartSpec, max_width: f64, metrics: &impl TextMetrics) -> f64 {
    if !spec.show_title {
        -TITLE_BLOCK
    } else if two_lines(&spec.title, max_width, TITLE_SIZE, metrics).is_some() {
        TITLE_LINE
    } else {
        0.0
    }
}

/// Left edge of the chart's content: where the category labels left of a horizontal plot may
/// start.
pub(crate) const CONTENT_LEFT: f64 = 16.0;

/// Where the title of a horizontal chart goes, as its left and its width: above the plot, unless
/// it would have to be shortened there, as it may after a wide gutter of category labels on a
/// narrow chart; it then spans the chart from the left edge of the content to the right margin.
pub(crate) fn horizontal_title(
    spec: &ChartSpec,
    (left, plot_width): (f64, f64),
    metrics: &impl TextMetrics,
) -> (f64, f64) {
    if title_fits(spec, plot_width, metrics) {
        (left, plot_width)
    } else {
        (
            CONTENT_LEFT,
            f64::from(spec.width) - CONTENT_LEFT - f64::from(PLOT_MARGIN),
        )
    }
}

/// Whether the title fits `max_width` on one line or two without being shortened.
pub(crate) fn title_fits(spec: &ChartSpec, max_width: f64, metrics: &impl TextMetrics) -> bool {
    let title = spec.title.as_str();
    metrics.width(title, TITLE_SIZE) <= max_width
        || two_lines(title, max_width, TITLE_SIZE, metrics)
            .is_some_and(|(_, rest)| metrics.width(rest, TITLE_SIZE) <= max_width)
}

/// Draws the chart title at `x`: on one line if it fits `max_width`, otherwise on two, broken at
/// a space. Only a second line that is still too wide, or a first word wider than the chart, is
/// shortened. Whatever sits below the title moves down by [`title_extra`]. Draws nothing when the
/// specification leaves the title out.
pub(crate) fn push_title(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    x: f64,
    max_width: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    if !spec.show_title {
        return;
    }
    let title = spec.title.as_str();
    let lines = match two_lines(title, max_width, TITLE_SIZE, metrics) {
        Some((first, rest)) => vec![
            first.to_owned(),
            fit_text(rest, max_width, TITLE_SIZE, metrics, warnings, "/title"),
        ],
        None => vec![fit_text(
            title, max_width, TITLE_SIZE, metrics, warnings, "/title",
        )],
    };
    for (index, content) in lines.into_iter().enumerate() {
        elements.push(Element::Text(Text {
            x,
            y: 30.0 + count(index) * TITLE_LINE,
            class: "chartlet-title",
            anchor: TextAnchor::Start,
            content,
        }));
    }
}
