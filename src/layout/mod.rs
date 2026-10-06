mod annotation;
mod atlas;
mod axis;
mod bar;
mod labels;
mod legend;
mod line;
mod panelbars;
mod stack;
mod timechart;
mod title;
mod topicmap;

use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Element, Line, Scene, Text, TextAnchor},
    spec::{ChartSpec, ChartType, MAX_SERIES, NumberStyle, Orientation},
};
pub(crate) use annotation::{marker_position, zone_extent};
pub(crate) use atlas::REALM_CLASSES;
use atlas::layout_atlas;
pub(crate) use axis::{
    NumericScale, add_bottom_category_title, format_value, push_category_label, push_side_label,
};
use bar::{layout_horizontal, layout_vertical};
pub(crate) use labels::WithReserve;
use line::layout_line;
use panelbars::layout_panel_bars;
pub(crate) use timechart::{TimeFrame, tooltip_name};
use timechart::{layout_multiples, layout_time};
pub(crate) use title::{
    CONTENT_LEFT, NARROW, horizontal_title, push_title, title_extra, two_lines,
};
use topicmap::layout_topicmap;

pub(crate) const LABEL_SIZE: f64 = 12.0;
/// Distance between the baselines of a category label wrapped onto two lines.
const LABEL_LINE: f64 = 14.0;
/// Distance between the baselines of a title wrapped onto two lines; what a wrapped title adds to
/// the height above the plot.
const TITLE_LINE: f64 = 28.0;
/// Extra top margin that makes room for the legend of a multi-series chart.
const LEGEND_HEIGHT: f64 = 24.0;
/// Top of the legend row below a drawn title.
const LEGEND_ROW: f64 = 46.0;
/// Gutter left of the plot for the value-axis ticks, and the margin right of it.
pub(crate) const AXIS_GUTTER: u32 = 72;
pub(crate) const PLOT_MARGIN: u32 = 24;
/// Below this width a chart is compact, such as a panel in a grid of columns: a narrower gutter
/// and margin leave its plot room, at the same text size.
pub(crate) const COMPACT: u32 = 320;

/// The gutter left of the plot of a chart `width` wide.
pub(crate) const fn axis_gutter(width: u32) -> u32 {
    if width < COMPACT { 56 } else { AXIS_GUTTER }
}

/// The margin right of the plot of a chart `width` wide.
pub(crate) const fn plot_margin(width: u32) -> u32 {
    if width < COMPACT { 12 } else { PLOT_MARGIN }
}

/// The horizontal pixels a plot keeps out of a chart of `width`: the gutter for the value-axis
/// ticks and the margin on the right come off. Measured in whole pixels, so the density check in
/// the specification and the tick count use the same number.
/// Below this median distance between neighbouring marks, in pixels, a mark gets no tooltip: the
/// targets would be too small to point at, and every tooltip costs bytes. The values stay in the
/// data table and the description. One rule for the markers of lines and for candles.
pub(crate) const MIN_TOOLTIP_SPACING: f64 = 4.0;

/// Whether marks at the horizontal positions `xs` stand far enough apart for a tooltip each.
pub(crate) fn tooltips_fit(xs: &[f64]) -> bool {
    let mut gaps: Vec<f64> = xs
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .collect();
    if gaps.is_empty() {
        return true;
    }
    crate::sort::by(&mut gaps, f64::total_cmp);
    gaps[gaps.len() / 2] >= MIN_TOOLTIP_SPACING
}

/// The gridlines of the value axis with their tick labels: left of a vertical axis, below a
/// horizontal one.
fn push_value_ticks(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    scale: &NumericScale,
    plot: PlotArea,
    metrics: &impl TextMetrics,
) {
    let label_step = horizontal_tick_step(spec, scale, plot, metrics);
    for (index, value) in scale.ticks().enumerate() {
        if plot.vertical_bars {
            let y = scale.map(value, plot.top + plot.height, plot.top);
            elements.push(Element::Line(Line {
                x1: plot.left,
                y1: y,
                x2: plot.left + plot.width,
                y2: y,
                class: if scale.is_zero(value) {
                    "chartlet-zero"
                } else {
                    "chartlet-grid"
                },
            }));
            elements.push(Element::Text(Text {
                x: plot.left - 10.0,
                y: y + 4.0,
                class: "chartlet-tick",
                anchor: TextAnchor::End,
                content: scale.tick_label(value, spec.axis_style()),
            }));
        } else {
            let x = scale.map(value, plot.left, plot.left + plot.width);
            elements.push(Element::Line(Line {
                x1: x,
                y1: plot.top,
                x2: x,
                y2: plot.top + plot.height,
                class: if scale.is_zero(value) {
                    "chartlet-zero"
                } else {
                    "chartlet-grid"
                },
            }));
            if index.is_multiple_of(label_step) {
                elements.push(Element::Text(Text {
                    x,
                    y: plot.top + plot.height + 22.0,
                    class: "chartlet-tick",
                    anchor: TextAnchor::Middle,
                    content: scale.tick_label(value, spec.axis_style()),
                }));
            }
        }
    }
}

/// Which tick labels a horizontal value axis writes: every one, or every `step`-th where wide
/// labels such as those of a logarithmic axis would otherwise run into each other. The gridlines
/// stay at every tick. A vertical axis writes every label.
fn horizontal_tick_step(
    spec: &ChartSpec,
    scale: &NumericScale,
    plot: PlotArea,
    metrics: &impl TextMetrics,
) -> usize {
    if plot.vertical_bars {
        return 1;
    }
    let ticks: Vec<(f64, f64)> = scale
        .ticks()
        .map(|value| {
            let x = scale.map(value, plot.left, plot.left + plot.width);
            let width =
                WithReserve(metrics).width(&scale.tick_label(value, spec.axis_style()), LABEL_SIZE);
            (x, width)
        })
        .collect();
    (1..ticks.len().max(2))
        .find(|step| {
            ticks
                .iter()
                .step_by(*step)
                .collect::<Vec<_>>()
                .windows(2)
                .all(|pair| pair[1].0 - pair[0].0 >= f64::midpoint(pair[0].1, pair[1].1) + 6.0)
        })
        .unwrap_or(ticks.len().max(1))
}

/// The height a plot keeps at least: below it an axis cannot show its values, and a smaller or
/// negative height would turn the plot upside down.
const MIN_PLOT_HEIGHT: f64 = 40.0;

/// The height of a plot from what the chart leaves it, at least [`MIN_PLOT_HEIGHT`]; a plot
/// that has to be raised to it is reported, since it then reaches below the chart's content.
pub(crate) fn plot_height(available: f64, warnings: &mut Vec<ChartWarning>) -> f64 {
    if available < MIN_PLOT_HEIGHT {
        warnings.push(ChartWarning::new(
            "dense_chart",
            "/height",
            format!(
                "the plot is less than {MIN_PLOT_HEIGHT} pixels tall at this size; raise height or leave out the title, legend or axis titles"
            ),
        ));
        return MIN_PLOT_HEIGHT;
    }
    available
}

pub(crate) const fn plot_pixels(width: u32) -> u32 {
    width.saturating_sub(axis_gutter(width) + plot_margin(width))
}

/// Gutter left of each panel's plot for its value ticks, and the margin right of it.
const PANEL_GUTTER: u32 = 44;
const PANEL_MARGIN: u32 = 12;

/// The horizontal pixels of one small-multiples panel's plot, by the same rule as
/// [`plot_pixels`]: what is left of a grid cell once the panel's gutter and margin come off.
pub(crate) const fn panel_plot_pixels(width: u32, columns: u32) -> u32 {
    let cell = width.saturating_sub(2 * 16) / if columns == 0 { 1 } else { columns };
    cell.saturating_sub(PANEL_GUTTER + PANEL_MARGIN)
}

const SERIES_BAR_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-bar chartlet-series-1",
    "chartlet-bar chartlet-series-2",
    "chartlet-bar chartlet-series-3",
    "chartlet-bar chartlet-series-4",
];
/// The same bars drawn as outlines: with `patterns`, every other series, so that series differ in
/// form as well as in color.
const OUTLINED_BAR_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-bar chartlet-series-1",
    "chartlet-bar chartlet-series-2 chartlet-outline",
    "chartlet-bar chartlet-series-3",
    "chartlet-bar chartlet-series-4 chartlet-outline",
];

/// The class of a series' bars and legend swatch: filled, or an outline for every other series
/// when the chart asks for `patterns`.
pub(super) fn series_bar_class(spec: &ChartSpec, series_index: usize) -> &'static str {
    if spec.patterns {
        OUTLINED_BAR_CLASSES[series_index]
    } else {
        SERIES_BAR_CLASSES[series_index]
    }
}

/// Fill of an area layer, in the palette color of its line.
const AREA_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-area chartlet-band-series-1",
    "chartlet-area chartlet-band-series-2",
    "chartlet-area chartlet-band-series-3",
    "chartlet-area chartlet-band-series-4",
];

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let dataset = spec.dataset();
    match (spec.chart_type, spec.orientation) {
        (ChartType::Bar, _) if spec.stack.is_some() => {
            stack::layout(spec, &dataset, warnings, metrics)
        }
        (ChartType::Bar, _) if spec.group_view().is_some() => {
            let view = spec.group_view().expect("checked by the guard");
            stack::layout(&view, &view.dataset(), warnings, metrics)
        }
        (ChartType::Bar, Orientation::Vertical) => {
            layout_vertical(spec, &dataset, warnings, metrics)
        }
        (ChartType::Bar, Orientation::Horizontal) => {
            layout_horizontal(spec, &dataset, warnings, metrics)
        }
        (ChartType::Line, _) => layout_line(spec, warnings, metrics),
        (ChartType::Time, _) => layout_time(spec, warnings, metrics),
        (ChartType::Multiples, _) if spec.is_bar_multiples() => {
            layout_panel_bars(spec, warnings, metrics)
        }
        (ChartType::Multiples, _) => layout_multiples(spec, warnings, metrics),
        (ChartType::Stripes, _) => crate::stripes::layout(spec, warnings, metrics),
        (ChartType::Calendar, _) => crate::calendar::layout(spec, warnings, metrics),
        (ChartType::Rangebar, _) => crate::rangebar::layout(spec, warnings, metrics),
        (ChartType::Boxplot, _) => crate::boxplot::layout(spec, warnings, metrics),
        (ChartType::Topicmap, _) => layout_topicmap(spec, warnings, metrics),
        (ChartType::Atlas, _) => layout_atlas(spec, warnings, metrics),
        (ChartType::Sequence, _) => crate::sequence::layout(spec, warnings, metrics),
        (ChartType::Flow, _) => crate::flow::layout(spec, warnings, metrics),
        (ChartType::State, _) => crate::state::layout(spec, warnings, metrics),
        (ChartType::Architecture, _) => crate::architecture::layout(spec, warnings, metrics),
        (ChartType::Tree, _) => crate::tree::layout(spec, warnings, metrics),
        (ChartType::Timeline, _) => crate::timeline::layout(spec, warnings, metrics),
        (ChartType::Sankey, _) => crate::sankey::layout(spec, warnings, metrics),
        (ChartType::Treemap, _) => crate::treemap::layout(spec, warnings, metrics),
        (ChartType::Parliament, _) => crate::parliament::layout(spec, warnings, metrics),
        (ChartType::Waffle, _) => crate::waffle::layout(spec, warnings, metrics),
        (ChartType::Waterfall, _) => crate::waterfall::layout(spec, warnings, metrics),
    }
}

pub(crate) fn base_elements(
    spec: &ChartSpec,
    scale: &NumericScale,
    plot: PlotArea,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    base_elements_with_title(
        spec,
        scale,
        plot,
        (plot.left, plot.width),
        warnings,
        metrics,
    )
}

/// [`base_elements`] with the title at `title_left` and up to `title_width` wide instead of
/// above the plot.
pub(crate) fn base_elements_with_title(
    spec: &ChartSpec,
    scale: &NumericScale,
    plot: PlotArea,
    (title_left, title_width): (f64, f64),
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        title_left,
        title_width,
        metrics,
        warnings,
    );

    push_value_ticks(&mut elements, spec, scale, plot, metrics);

    if let Some(title) = &spec.value_axis.title {
        let title = fit_text(
            title,
            plot.width,
            LABEL_SIZE,
            metrics,
            warnings,
            "/valueAxis/title",
        );
        if plot.vertical_bars {
            elements.push(Element::Text(Text {
                x: plot.left,
                y: plot.top - 20.0,
                class: "chartlet-axis-title",
                anchor: TextAnchor::Start,
                content: title.clone(),
            }));
        } else {
            elements.push(Element::Text(Text {
                x: plot.left + plot.width / 2.0,
                y: plot.top + plot.height + 44.0,
                class: "chartlet-axis-title",
                anchor: TextAnchor::Middle,
                content: title,
            }));
        }
    }

    if !plot.vertical_bars
        && let Some(title) = &spec.category_axis.title
    {
        let title = fit_text(
            title,
            plot.left - 24.0,
            LABEL_SIZE,
            metrics,
            warnings,
            "/categoryAxis/title",
        );
        elements.push(Element::Text(Text {
            x: plot.left - 12.0,
            y: plot.top - 20.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::End,
            content: title,
        }));
    }

    elements
}

pub(crate) fn tooltip(
    category: &str,
    value: f64,
    style: NumberStyle,
    series_name: Option<&str>,
) -> String {
    let formatted = format_value(value, style);
    match series_name {
        Some(name) => format!("{category} – {name}: {formatted}"),
        None => format!("{category}: {formatted}"),
    }
}

pub(crate) fn warn_if_labels_omitted(omitted: bool, warnings: &mut Vec<ChartWarning>) {
    if omitted
        && !warnings
            .iter()
            .any(|warning| warning.code == "value_labels_omitted")
    {
        warnings.push(ChartWarning::new(
            "value_labels_omitted",
            "/showValues",
            "some value labels do not fit next to their bars or points and were left out; every value remains in the HTML data alternative",
        ));
    }
}

pub(crate) fn count(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("counts are limited by validation"))
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotArea {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub vertical_bars: bool,
}

pub(crate) fn fit_text(
    text: &str,
    max_width: f64,
    font_size: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) -> String {
    if metrics.width(text, font_size) <= max_width {
        return text.to_owned();
    }

    let ellipsis_width = metrics.width("…", font_size);
    let mut output = String::new();
    for character in text.chars() {
        let candidate = format!("{output}{character}");
        if metrics.width(&candidate, font_size) + ellipsis_width > max_width {
            break;
        }
        output.push(character);
    }
    output.push('…');

    if !warnings
        .iter()
        .any(|warning| warning.code == "text_truncated" && warning.path == path)
    {
        warnings.push(ChartWarning::new(
            "text_truncated",
            path,
            "text was shortened in the visual chart; the full value remains in the specification and HTML data alternative",
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{LABEL_LINE, LEGEND_HEIGHT, count};

    /// Every character is half as wide as the font is high: easy widths to reason about.
    struct HalfEm;

    impl crate::metrics::TextMetrics for HalfEm {
        fn width(&self, text: &str, font_size: f64) -> f64 {
            count(text.chars().count()) * font_size / 2.0
        }
    }

    /// The texts of a class, as (y, content), in the order they are drawn.
    fn texts(scene: &crate::scene::Scene, class: &str) -> Vec<(f64, String)> {
        scene
            .elements
            .iter()
            .filter_map(|element| match element {
                crate::scene::Element::Text(text)
                    if text.class.split(' ').any(|name| name == class) =>
                {
                    Some((text.y, text.content.clone()))
                }
                _ => None,
            })
            .collect()
    }

    fn lay_out(json: &str) -> (crate::scene::Scene, Vec<crate::error::ChartWarning>) {
        let spec = crate::spec::ChartSpec::from_json(json).expect("valid specification");
        let mut warnings = Vec::new();
        let scene = super::layout(&spec, &mut warnings, &HalfEm);
        (scene, warnings)
    }

    /// The top of the plot, as the highest value grid line or zero line.
    fn plot_top(scene: &crate::scene::Scene) -> f64 {
        scene
            .elements
            .iter()
            .filter_map(|element| match element {
                crate::scene::Element::Line(line) => Some(line.y1),
                _ => None,
            })
            .fold(f64::INFINITY, f64::min)
    }

    fn bar_chart(title: &str, width: u32) -> String {
        format!(
            r#"{{"schemaVersion": 1, "type": "bar", "title": "{title}", "width": {width},
                "data": [{{"label": "A", "value": 1}}, {{"label": "B", "value": 2}}]}}"#
        )
    }

    #[test]
    fn a_title_too_wide_for_one_line_wraps_onto_two_and_moves_the_plot_down() {
        // The plot of a 360-pixel chart is 264 pixels wide: 24 characters of the title.
        let (short, _) = lay_out(&bar_chart("Revenue by channel", 360));
        let (long, warnings) = lay_out(&bar_chart("Monthly revenue by sales channel", 360));

        assert_eq!(
            texts(&short, "chartlet-title"),
            [(30.0, "Revenue by channel".into())]
        );
        assert_eq!(
            texts(&long, "chartlet-title"),
            [
                (30.0, "Monthly revenue by sales".into()),
                (58.0, "channel".into())
            ]
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!((plot_top(&long) - plot_top(&short) - super::TITLE_LINE).abs() < 1e-9);
    }

    #[test]
    fn a_title_too_long_for_two_lines_shortens_the_second() {
        let title = "Monthly revenue by sales channel and region compared with the previous year";
        let (scene, warnings) = lay_out(&bar_chart(title, 360));
        let lines = texts(&scene, "chartlet-title");

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].1, "Monthly revenue by sales");
        assert!(lines[1].1.ends_with('…'));
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            (warnings[0].code, warnings[0].path.as_str()),
            ("text_truncated", "/title")
        );
    }

    #[test]
    fn a_title_word_wider_than_the_chart_stays_on_one_shortened_line() {
        let (scene, warnings) = lay_out(&bar_chart(&"W".repeat(40), 360));
        let lines = texts(&scene, "chartlet-title");

        assert_eq!(lines.len(), 1);
        assert!(lines[0].1.ends_with('…'));
        assert_eq!(warnings[0].code, "text_truncated");
    }

    #[test]
    fn a_bar_legend_wraps_into_rows_instead_of_shortening_names() {
        let json = |width: u32| {
            format!(
                r#"{{"schemaVersion": 1, "type": "bar", "title": "Costs", "width": {width},
                    "categories": ["A", "B"],
                    "series": [
                        {{"name": "Budget for the year", "values": [1, 2]}},
                        {{"name": "Actual spending", "values": [2, 1]}},
                        {{"name": "Forecast from planning", "values": [1, 1]}}
                    ]}}"#
            )
        };
        let (wide, _) = lay_out(&json(800));
        let (narrow, warnings) = lay_out(&json(360));

        let rows = |scene: &crate::scene::Scene| {
            texts(scene, "chartlet-legend")
                .into_iter()
                .map(|(y, _)| y)
                .collect::<Vec<_>>()
        };
        assert_eq!(rows(&wide), [55.0, 55.0, 55.0]);
        assert_eq!(rows(&narrow), [55.0, 55.0, 79.0]);
        assert!(
            texts(&narrow, "chartlet-legend")
                .iter()
                .all(|(_, name)| !name.ends_with('…'))
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!((plot_top(&narrow) - plot_top(&wide) - LEGEND_HEIGHT).abs() < 1e-9);
    }

    #[test]
    fn a_category_label_wraps_onto_two_lines_before_it_is_shortened() {
        // Four categories on a 360-pixel chart: 66-pixel bands, 58 pixels or 9 characters of
        // label.
        let json = r#"{"schemaVersion": 1, "type": "bar", "title": "Costs", "width": 360,
            "data": [
                {"label": "Sales team", "value": 1},
                {"label": "Product development", "value": 2},
                {"label": "Logistics", "value": 3},
                {"label": "Administration", "value": 4}
            ]}"#;
        let (scene, warnings) = lay_out(json);
        let labels = texts(&scene, "chartlet-label");
        let contents: Vec<&str> = labels.iter().map(|(_, text)| text.as_str()).collect();

        assert_eq!(
            contents,
            [
                "Sales",
                "team",
                "Product",
                "developm…",
                "Logistics",
                "Administ…"
            ]
        );
        assert!((labels[1].0 - labels[0].0 - LABEL_LINE).abs() < 1e-9);
        let truncated: Vec<&str> = warnings
            .iter()
            .map(|warning| warning.path.as_str())
            .collect();
        assert_eq!(truncated, ["/data/1/label", "/data/3/label"]);
    }

    #[test]
    fn a_side_label_wraps_onto_two_lines_centered_on_its_band() {
        let json = r#"{"schemaVersion": 1, "type": "bar", "orientation": "horizontal",
            "title": "Change", "data": [
                {"label": "First", "value": 1},
                {"label": "Third quarter with a deliberately long label", "value": 2}
            ]}"#;
        let (scene, warnings) = lay_out(json);
        let labels = texts(&scene, "chartlet-label");

        assert_eq!(labels.len(), 3);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!((labels[2].0 - labels[1].0 - LABEL_LINE).abs() < 1e-9);
    }

    /// The value labels of a chart, as (x, content), in the order they are drawn.
    fn value_labels(scene: &crate::scene::Scene) -> Vec<(f64, String)> {
        scene
            .elements
            .iter()
            .filter_map(|element| match element {
                crate::scene::Element::Text(text) | crate::scene::Element::SeriesText(text, _)
                    if text.class.starts_with("chartlet-value") =>
                {
                    Some((text.x, text.content.clone()))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_category_whose_value_labels_collide_shows_none_of_them() {
        // Three categories of three series on a 360-pixel chart: 23-pixel slots. "1000" is 24
        // pixels wide, 30 with reserve and clearance.
        let json = |width: u32| {
            format!(
                r#"{{"schemaVersion": 1, "type": "bar", "title": "Costs", "width": {width},
                    "categories": ["A", "B", "C"],
                    "series": [
                        {{"name": "X", "values": [1000, 5, 1000]}},
                        {{"name": "Y", "values": [1000, 5, 5]}},
                        {{"name": "Z", "values": [1000, 5, 5]}}
                    ]}}"#
            )
        };
        let (wide, wide_warnings) = lay_out(&json(800));
        let (narrow, warnings) = lay_out(&json(360));

        assert_eq!(value_labels(&wide).len(), 9);
        assert!(wide_warnings.is_empty(), "{wide_warnings:?}");
        // A's labels stand side by side at one height and overlap: A shows none. C's "1000" is
        // wider than its slot, but the bars beside it end far below it: C keeps all three.
        let contents: Vec<String> = value_labels(&narrow)
            .into_iter()
            .map(|(_, content)| content)
            .collect();
        assert_eq!(contents, ["5", "5", "5", "1000", "5", "5"]);
        let codes: Vec<&str> = warnings.iter().map(|warning| warning.code).collect();
        assert_eq!(codes, ["value_labels_omitted"]);
    }

    #[test]
    fn a_value_label_reaching_over_a_taller_neighbouring_bar_drops_its_category() {
        // B's "1000" on the short bar reaches over the taller bar beside it.
        let json = r#"{"schemaVersion": 1, "type": "bar", "title": "Costs", "width": 360,
            "categories": ["A", "B", "C"],
            "series": [
                {"name": "X", "values": [5, 1000, 5]},
                {"name": "Y", "values": [5, 9000, 5]},
                {"name": "Z", "values": [5, 5, 5]}
            ]}"#;
        let (scene, warnings) = lay_out(json);

        assert_eq!(value_labels(&scene).len(), 6);
        assert!(
            value_labels(&scene)
                .iter()
                .all(|(_, content)| content == "5")
        );
        assert_eq!(warnings[0].code, "value_labels_omitted");
    }

    #[test]
    fn a_rangebar_hatching_legend_moves_left_when_it_would_overflow_the_chart() {
        // With HalfEm, "Schraffiert: modelliert" is 138 pixels wide, 149 with the reserve; the
        // gutter for "Other human drivers" is 143 pixels.
        let json = |width: u32| {
            format!(
                r#"{{"schemaVersion": 1, "type": "rangebar", "orientation": "horizontal",
                    "locale": "de", "title": "Beiträge", "width": {width},
                    "ranges": [
                        {{"label": "Observed", "low": 0.9, "high": 1.2}},
                        {{"label": "Other human drivers", "low": -0.8, "high": 0.0,
                          "modeled": true}}
                    ]}}"#
            )
        };
        let legend = |scene: &crate::scene::Scene| {
            scene
                .elements
                .iter()
                .find_map(|element| match element {
                    crate::scene::Element::Text(text) if text.class == "chartlet-legend" => {
                        Some((text.x, text.content.clone()))
                    }
                    _ => None,
                })
                .expect("a modeled span has a legend")
        };
        let (wide, _) = lay_out(&json(800));
        let (narrow, warnings) = lay_out(&json(320));

        assert_eq!(
            legend(&wide),
            (143.0 + 16.0, "Schraffiert: modelliert".into())
        );
        assert_eq!(
            legend(&narrow),
            (16.0 + 16.0, "Schraffiert: modelliert".into())
        );
        assert!(
            warnings.iter().all(|warning| warning.path != "/locale"),
            "{warnings:?}"
        );
    }

    #[test]
    fn a_horizontal_rangebar_title_spans_the_chart_when_the_gutter_leaves_too_little_room() {
        // With HalfEm the gutter for "Other human drivers" is 143 pixels; on a 320-pixel chart
        // the plot is 153 pixels wide, too narrow for the title even on two lines of the small title.
        let json = |width: u32| {
            format!(
                r#"{{"schemaVersion": 1, "type": "rangebar", "orientation": "horizontal",
                    "title": "Contributions to global warming since 1850", "width": {width},
                    "showValues": false,
                    "ranges": [
                        {{"label": "Observed", "low": 0.9, "high": 1.2}},
                        {{"label": "Other human drivers", "low": -0.8, "high": 0.0}}
                    ]}}"#
            )
        };
        let title = |scene: &crate::scene::Scene| {
            scene
                .elements
                .iter()
                .filter_map(|element| match element {
                    crate::scene::Element::Text(text)
                        if text.class.starts_with("chartlet-title") =>
                    {
                        Some((text.x, text.content.clone()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let (wide, _) = lay_out(&json(800));
        let (narrow, warnings) = lay_out(&json(320));

        assert_eq!(
            title(&wide),
            [(143.0, "Contributions to global warming since 1850".into())]
        );
        assert_eq!(
            title(&narrow),
            [
                (16.0, "Contributions to global warming".into()),
                (16.0, "since 1850".into())
            ]
        );
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn a_horizontal_bar_title_spans_the_chart_when_the_gutter_leaves_too_little_room() {
        // With HalfEm the gutter for "Other human drivers" is 142 pixels; on a 320-pixel chart
        // the plot is 110 pixels wide, too narrow for the title even on two lines.
        let json = |width: u32| {
            format!(
                r#"{{"schemaVersion": 1, "type": "bar", "orientation": "horizontal",
                    "title": "Contributions to global warming", "width": {width},
                    "categories": ["Observed", "Other human drivers"],
                    "series": [{{"name": "Warming", "values": [1.2, 0.4]}}]}}"#
            )
        };
        let title = |scene: &crate::scene::Scene| {
            scene
                .elements
                .iter()
                .filter_map(|element| match element {
                    crate::scene::Element::Text(text)
                        if text.class.starts_with("chartlet-title") =>
                    {
                        Some((text.x, text.content.clone()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let (wide, _) = lay_out(&json(800));
        let (narrow, warnings) = lay_out(&json(320));

        assert_eq!(
            title(&wide),
            [(142.0, "Contributions to global warming".into())]
        );
        assert_eq!(
            title(&narrow),
            [(16.0, "Contributions to global warming".into())]
        );
        assert!(warnings.is_empty(), "{warnings:?}");
    }
}
