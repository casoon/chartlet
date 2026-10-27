mod advice;
mod annotations;
mod bars_and_lines;
mod basics;
mod collapsed_gaps;
mod diagrams;
mod distributions;
mod fishbone;
mod heatmap;
mod limits;
mod locale;
mod manifest;
mod multiples;
mod outlines;
mod panes;
mod part_of_whole;
mod ranges;
mod scatter_and_survival;
mod slope;
mod stripes_and_calendar;
mod time;
mod timeline;
mod variants;

use super::{
    BuiltinMetrics, ChartError, ChartSpec, RenderFormat, RenderOptions, RenderOutput, TableMode,
    TextMetrics, Variant, render_json, render_with_metrics, sha256, text_alternative,
};

const SPEC: &str = r#"{
        "schemaVersion": 1,
        "type": "bar",
        "title": "Profit & loss",
        "data": [
            {"label": "North <East>", "value": 12},
            {"label": "South", "value": -4}
        ]
    }"#;

const GROUPED: &str = r#"{
        "schemaVersion": 1,
        "type": "bar",
        "title": "Budget and actual",
        "categories": ["Jan", "Feb", "Mar"],
        "series": [
            {"name": "Budget", "values": [120, 150, 140]},
            {"name": "Actual", "values": [130, 145, null]}
        ]
    }"#;

const TIME: &str = r#"{
        "schemaVersion": 1,
        "type": "time",
        "title": "Daily orders",
        "timeAxis": {"timezone": "UTC", "title": "Day"},
        "panes": [
            {
                "valueAxis": {"title": "Orders"},
                "layers": [
                    {
                        "mark": "line",
                        "name": "Orders",
                        "points": [
                            {"time": "2026-03-01", "value": 10},
                            {"time": "2026-03-02", "value": 15},
                            {"time": "2026-03-03", "value": 20}
                        ]
                    }
                ]
            }
        ]
    }"#;

/// Adds `"locale": "de"` to a specification of the given type.
fn german(spec: &str, chart_type: &str) -> String {
    spec.replace(
        &format!("\"type\": \"{chart_type}\","),
        &format!("\"type\": \"{chart_type}\", \"locale\": \"de\","),
    )
}

fn html_ok(spec: &str) -> String {
    render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content
}

fn render_ok(spec: &str) -> super::RenderOutput {
    render_json(spec, RenderFormat::Svg, &RenderOptions::default())
        .unwrap_or_else(|error| panic!("should render: {error}"))
}

fn render_err(spec: &str) -> (&'static str, String) {
    let error = render_json(spec, RenderFormat::Svg, &RenderOptions::default())
        .expect_err("should be refused");
    (error.code, error.path)
}

fn with_rules(rules: &str) -> String {
    TIME.replace("\"layers\": [", &format!("\"layers\": [{rules},"))
}

const STRIPES: &str = r#"{
        "schemaVersion": 1,
        "type": "stripes",
        "title": "Stripes",
        "stripes": {"firstYear": 1850, "values": [-1, -0.5, null, 0, 0.5, 2], "min": -1, "max": 1}
    }"#;

fn calendar(layout: &str, days: &str) -> String {
    format!(
        r#"{{
                "schemaVersion": 1,
                "type": "calendar",
                "title": "Calendar",
                "calendar": {{"year": 2024, "layout": "{layout}", "days": [{days}]}}
            }}"#
    )
}

const DAYS: &str = r#"{"date": "2024-01-01", "value": -2}, {"date": "2024-02-29", "value": 4}, {"date": "2024-12-31", "value": null}"#;

const RANGES: &str = r#"{
        "schemaVersion": 1,
        "type": "rangebar",
        "orientation": "horizontal",
        "title": "Contributions",
        "ranges": [
            {"label": "Observed", "low": 0.9, "high": 1.2, "mid": 1.05},
            {"label": "Natural", "low": -0.1, "high": 0.1, "modeled": true}
        ]
    }"#;

fn multiples(panes: &str) -> String {
    format!(
        r#"{{
                "schemaVersion": 1,
                "type": "multiples",
                "title": "Sectors",
                "valueAxis": {{"title": "Emissions"}},
                "panes": [{panes}]
            }}"#
    )
}

fn panel(title: &str, scale: f64) -> String {
    format!(
        r#"{{"title": "{title}", "layers": [
                {{"mark": "line", "name": "History", "points": [{{"time": "2000", "value": {}}}, {{"time": "2010", "value": {}}}]}},
                {{"mark": "line", "name": "Pathway", "modeled": true, "points": [{{"time": "2010", "value": {}, "lower": 0, "upper": {}}}, {{"time": "2050", "value": 0, "lower": 0, "upper": {}}}]}},
                {{"mark": "annotation", "value": 0, "label": "Net zero"}}
            ]}}"#,
        10.0 * scale,
        12.0 * scale,
        12.0 * scale,
        14.0 * scale,
        2.0 * scale
    )
}

/// Position and content of every text of `class`, in document order.
fn placed_texts(svg: &str, class: &str) -> Vec<(f64, f64, String)> {
    svg.split("<text ")
        .skip(1)
        .filter(|chunk| chunk.contains(&format!("class=\"{class}\"")))
        .map(|chunk| {
            let read = |name: &str| -> f64 {
                chunk
                    .split(&format!(" {name}=\""))
                    .nth(1)
                    .or_else(|| chunk.strip_prefix(&format!("{name}=\"")))
                    .and_then(|rest| rest.split('"').next())
                    .expect("a coordinate")
                    .parse()
                    .expect("a number")
            };
            let content = chunk
                .split_once('>')
                .and_then(|(_, rest)| rest.split_once("</text>"))
                .map(|(content, _)| content.to_owned())
                .expect("a text element has content");
            (read("x"), read("y"), content)
        })
        .collect()
}
