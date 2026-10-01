//! The social variant: the chart on a 1200 × 630 canvas for link previews such as Open Graph
//! images, with the title drawn large above it and the source below.

use std::fmt::Write;

use crate::{
    error::ChartWarning, layout::fit_text, metrics::TextMetrics, render::escape, spec::ChartSpec,
};

/// The size of the canvas, the common size of Open Graph images.
pub(crate) const WIDTH: u32 = 1200;
pub(crate) const HEIGHT: u32 = 630;
/// Left edge of the title and the source, and the space above the title.
const MARGIN: f64 = 56.0;
const TITLE_SIZE: f64 = 44.0;
const TITLE_LINE: f64 = 54.0;
const SOURCE_SIZE: f64 = 20.0;
/// The chart is laid out at a third of the canvas less and drawn scaled up, so that its 12-pixel
/// labels stay readable where a preview shows the image at half its size.
const SCALE: f64 = 1.5;
/// Left edge of the chart on the canvas. Its own left margin, scaled, brings its first labels in
/// line with the title.
const CHART_LEFT: f64 = 30.0;

/// The canvas around the chart: the title in at most two lines, the source line, and the place
/// and size of the chart.
pub(crate) struct Frame {
    title: Vec<String>,
    source: Option<String>,
    chart_top: f64,
    /// The size at which the chart is laid out, before it is scaled onto the canvas.
    pub(crate) chart_width: u32,
    pub(crate) chart_height: u32,
}

impl Frame {
    pub(crate) fn new(
        spec: &ChartSpec,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
    ) -> Self {
        let max_width = f64::from(WIDTH) - 2.0 * MARGIN;
        let title = title_lines(&spec.title, max_width, metrics, warnings);
        let source = spec.source.as_ref().map(|source| {
            fit_text(
                &format!("{}: {source}", spec.locale.words().source),
                max_width,
                SOURCE_SIZE,
                metrics,
                warnings,
                "/source",
            )
        });
        let last_baseline = MARGIN + TITLE_SIZE * 0.8 + TITLE_LINE * count(title.len() - 1);
        let chart_top = last_baseline + 24.0;
        let chart_bottom = f64::from(HEIGHT) - if source.is_some() { 76.0 } else { 40.0 };
        Self {
            title,
            source,
            chart_top,
            chart_width: pixels((f64::from(WIDTH) - 2.0 * CHART_LEFT) / SCALE),
            chart_height: pixels((chart_bottom - chart_top) / SCALE),
        }
    }

    /// Places `chart`, the print SVG of the chart laid out at [`Frame::chart_width`] ×
    /// [`Frame::chart_height`], on the canvas. Its root, title, description and stylesheet become
    /// those of the canvas; the tooltips of its marks are left out, an image shows none.
    pub(crate) fn compose(&self, chart: &str) -> String {
        let (width, height) = (self.chart_width, self.chart_height);
        let chart = chart.replacen(
            &format!("width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\""),
            &format!("width=\"{WIDTH}\" height=\"{HEIGHT}\" viewBox=\"0 0 {WIDTH} {HEIGHT}\""),
            1,
        );
        let head_end = chart
            .find("</style>")
            .expect("a print SVG carries its stylesheet")
            + 8;
        let body = chart[head_end..]
            .strip_suffix("</svg>")
            .expect("an SVG ends with its root");
        let mut output = String::with_capacity(chart.len() + 512);
        output.push_str(&chart[..head_end]);
        write!(
            output,
            "<rect class=\"chartlet-background\" x=\"0\" y=\"0\" width=\"{WIDTH}\" height=\"{HEIGHT}\"/>"
        )
        .expect("writing to String cannot fail");
        for (index, line) in self.title.iter().enumerate() {
            write!(
                output,
                "<text x=\"{MARGIN}\" y=\"{}\" class=\"chartlet-title\" style=\"font-size:{TITLE_SIZE}px\">{}</text>",
                MARGIN + TITLE_SIZE * 0.8 + TITLE_LINE * count(index),
                escape(line)
            )
            .expect("writing to String cannot fail");
        }
        if let Some(source) = &self.source {
            write!(
                output,
                "<text x=\"{MARGIN}\" y=\"{}\" class=\"chartlet-label\" style=\"font-size:{SOURCE_SIZE}px\">{}</text>",
                f64::from(HEIGHT) - 40.0,
                escape(source)
            )
            .expect("writing to String cannot fail");
        }
        write!(
            output,
            "<g transform=\"translate({CHART_LEFT} {}) scale({SCALE})\">{}</g></svg>",
            self.chart_top,
            without_tooltips(body)
        )
        .expect("writing to String cannot fail");
        output
    }
}

/// The title on one line, or on two broken at a space; a second line that is still too wide is
/// shortened.
fn title_lines(
    title: &str,
    max_width: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<String> {
    if metrics.width(title, TITLE_SIZE) <= max_width {
        return vec![title.to_owned()];
    }
    let split = title
        .match_indices(' ')
        .map(|(index, _)| (title[..index].trim_end(), title[index + 1..].trim_start()))
        .filter(|(first, rest)| !first.is_empty() && !rest.is_empty())
        .take_while(|(first, _)| metrics.width(first, TITLE_SIZE) <= max_width)
        .last();
    match split {
        Some((first, rest)) => vec![
            first.to_owned(),
            fit_text(rest, max_width, TITLE_SIZE, metrics, warnings, "/title"),
        ],
        None => vec![fit_text(
            title, max_width, TITLE_SIZE, metrics, warnings, "/title",
        )],
    }
}

/// The marks' `<title>` children, which a browser shows as tooltips, removed. The root's title
/// carries an ID and stays.
fn without_tooltips(body: &str) -> String {
    let mut output = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find("<title>") {
        output.push_str(&rest[..start]);
        let end = start + rest[start..].find("</title>").expect("a title is closed") + 8;
        rest = &rest[end..];
    }
    output.push_str(rest);
    output
}

fn count(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("a title has at most two lines"))
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn pixels(value: f64) -> u32 {
    value.floor() as u32
}
