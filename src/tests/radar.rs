use super::*;

const RADAR: &str = r#"{"schemaVersion": 1, "type": "radar", "title": "Skills", "width": 600, "height": 460,
    "radar": {"axes": [
        {"label": "Speed", "max": 10}, {"label": "Power", "max": 10}, {"label": "Range", "max": 10},
        {"label": "Cost", "min": 0, "max": 10}
    ], "series": [
        {"name": "Alpha", "values": [8, 4, 6, 2]},
        {"name": "Beta", "values": [3, 9, 5, 7]}
    ]}}"#;

fn html(spec: &str) -> String {
    render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content
}

#[test]
fn a_radar_chart_draws_rings_spokes_polygons_and_a_marker_for_every_value() {
    let output = render_ok(RADAR);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-radar-ring\"").count(), 4);
    assert_eq!(svg.matches("class=\"chartlet-radar-spoke\"").count(), 4);
    assert_eq!(
        svg.matches("class=\"chartlet-radar-area chartlet-radar-a")
            .count(),
        2
    );
    assert_eq!(
        svg.matches("class=\"chartlet-radar-dot chartlet-radar-d")
            .count(),
        8
    );
    // The values of the rings stand on the first axis, and the axes are named outside.
    for ring in ["2.5", "5", "7.5", "10"] {
        assert!(
            svg.contains(&format!("class=\"chartlet-radar-tick\">{ring}</text>")),
            "{ring}"
        );
    }
    assert!(svg.contains(">Speed</text>"));
    assert!(svg.contains("<title>Beta, Power: 9</title>"));
    // Two series are told apart by the legend's line style as well as by color.
    assert_eq!(svg.matches("class=\"chartlet-radar-key").count(), 2);
}

#[test]
fn the_description_and_the_table_name_every_value() {
    let html = html(RADAR);
    assert!(html.contains("Radar chart of 4 axes (Speed, Power, Range, Cost) with 2 series. Alpha: strongest on Speed (8), weakest on Cost (2). Beta: strongest on Power (9), weakest on Speed (3)."));
    assert!(html.contains("<th scope=\"col\">Axis</th><th scope=\"col\">Range</th><th scope=\"col\">Alpha</th><th scope=\"col\">Beta</th>"));
    assert!(html.contains("<th scope=\"row\">Power</th><td>0–10</td><td>4</td><td>9</td>"));
}

#[test]
fn the_end_of_an_axis_defaults_to_the_next_step_of_the_grid() {
    let json = RADAR.replace(", \"max\": 10", "");
    let html = html(&json);
    // The largest value of Speed is 8: four rings of 2 make 8; of Power 9: rings of 2.5 make 10.
    assert!(html.contains("<th scope=\"row\">Speed</th><td>0–8</td><td>8</td><td>3</td>"));
    assert!(html.contains("<th scope=\"row\">Power</th><td>0–10</td><td>4</td><td>9</td>"));
}

#[test]
fn many_axes_or_series_are_reported_as_hard_to_read() {
    let axes = (0..9)
        .map(|index| format!("{{\"label\": \"A{index}\", \"max\": 10}}"))
        .collect::<Vec<_>>()
        .join(",");
    let values = ["5"; 9].join(",");
    let spec = format!(
        "{{\"schemaVersion\": 1, \"type\": \"radar\", \"title\": \"T\", \"width\": 600, \"height\": 460, \"radar\": {{\"axes\": [{axes}], \"series\": [{{\"name\": \"S\", \"values\": [{values}]}}]}}}}"
    );
    let output = render_ok(&spec);
    assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
    assert_eq!(output.warnings[0].code, "dense_chart");
    assert_eq!(output.warnings[0].path, "/radar/axes");
    let four = RADAR.replace(
        "{\"name\": \"Beta\", \"values\": [3, 9, 5, 7]}",
        "{\"name\": \"Beta\", \"values\": [3, 9, 5, 7]}, {\"name\": \"C\", \"values\": [1, 1, 1, 1]}, {\"name\": \"D\", \"values\": [2, 2, 2, 2]}",
    );
    let output = render_ok(&four);
    assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
    assert_eq!(output.warnings[0].path, "/radar/series");
}

#[test]
fn radar_charts_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            "\"values\": [8, 4, 6, 2]",
            "\"values\": [8, 4, 6]",
            "series_length_mismatch",
            "/radar/series/0/values",
        ),
        (
            "\"values\": [8, 4, 6, 2]",
            "\"values\": [8, 4, 16, 2]",
            "value_outside_axis",
            "/radar/series/0/values/2",
        ),
        (
            "\"values\": [8, 4, 6, 2]",
            "\"values\": [8, -4, 6, 2]",
            "value_outside_axis",
            "/radar/series/0/values/1",
        ),
        (
            "\"label\": \"Power\"",
            "\"label\": \"Speed\"",
            "duplicate_label",
            "/radar/axes/1/label",
        ),
        (
            "\"name\": \"Beta\"",
            "\"name\": \"Alpha\"",
            "duplicate_series",
            "/radar/series/1/name",
        ),
        (
            "\"min\": 0, \"max\": 10",
            "\"min\": 10, \"max\": 10",
            "invalid_axis_range",
            "/radar/axes/3/max",
        ),
    ] {
        let spec = RADAR.replace(from, to);
        assert_eq!(render_err(&spec), (code, path.to_owned()), "{from}");
    }
}

#[test]
fn a_radar_chart_needs_three_axes() {
    let two = r#"{"schemaVersion": 1, "type": "radar", "title": "T", "radar": {"axes": [{"label": "A"}, {"label": "B"}], "series": [{"name": "S", "values": [1, 2]}]}}"#;
    assert_eq!(render_err(two), ("empty_data", "/radar/axes".to_owned()));
}
