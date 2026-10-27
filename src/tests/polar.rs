use super::*;

const POLAR: &str = r#"{"schemaVersion": 1, "type": "polar", "title": "Wind", "width": 600, "height": 460,
    "polar": {"valueLabel": "Hours", "points": [
        {"label": "N", "value": 10}, {"label": "E", "value": 40},
        {"label": "S", "value": 25}, {"label": "W", "value": 0}
    ]}}"#;

fn html(spec: &str) -> String {
    render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content
}

#[test]
fn a_polar_chart_draws_rings_and_a_wedge_for_every_nonzero_value() {
    let output = render_ok(POLAR);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-polar-ring\"").count(), 4);
    // Zero has no wedge: it is the center.
    assert_eq!(svg.matches("class=\"chartlet-polar-bar\"").count(), 3);
    assert!(svg.contains("<title>E: 40</title>"));
    // The largest value is 40: four rings of 10.
    for ring in ["10", "20", "30", "40"] {
        assert!(
            svg.contains(&format!("class=\"chartlet-polar-tick\">{ring}</text>")),
            "{ring}"
        );
    }
    for name in ["N", "E", "S", "W"] {
        assert!(svg.contains(&format!(">{name}</text>")), "{name}");
    }
}

#[test]
fn lollipops_draw_a_stem_and_a_dot() {
    let svg = render_ok(&POLAR.replace(
        "\"valueLabel\"",
        "\"display\": \"lollipops\", \"valueLabel\"",
    ))
    .content;
    assert_eq!(svg.matches("class=\"chartlet-polar-stem\"").count(), 4);
    assert_eq!(svg.matches("class=\"chartlet-polar-dot\"").count(), 4);
    assert_eq!(svg.matches("class=\"chartlet-polar-bar\"").count(), 0);
}

#[test]
fn the_description_and_the_table_name_every_category() {
    let html = html(POLAR);
    assert!(html.contains("Polar chart of 4 categories, from the top clockwise: N, E, S, W. Highest: E (40); lowest: W (0). Hours runs from 0 at the center to 40 at the outer ring."));
    assert!(html.contains("<th scope=\"col\">Category</th><th scope=\"col\">Hours</th>"));
    assert!(html.contains("<th scope=\"row\">S</th><td>25</td>"));
}

#[test]
fn a_crowded_cycle_leaves_out_names_that_would_overlap() {
    let points = (0..60)
        .map(|index| {
            format!(
                "{{\"label\": \"Category {index}\", \"value\": {}}}",
                index % 7 + 1
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let spec = format!(
        "{{\"schemaVersion\": 1, \"type\": \"polar\", \"title\": \"T\", \"width\": 480, \"height\": 420, \"polar\": {{\"points\": [{points}]}}}}"
    );
    let output = render_ok(&spec);
    assert!(
        output.warnings.iter().any(|w| w.code == "labels_omitted"),
        "{:?}",
        output.warnings
    );
    // Every category stays in the table.
    assert_eq!(
        html(&spec).matches("<th scope=\"row\">Category").count(),
        60
    );
}

#[test]
fn polar_charts_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            "\"value\": 40",
            "\"value\": -4",
            "invalid_value",
            "/polar/points/1/value",
        ),
        (
            "\"label\": \"S\"",
            "\"label\": \"N\"",
            "duplicate_label",
            "/polar/points/2/label",
        ),
        (
            "\"valueLabel\"",
            "\"max\": 30, \"valueLabel\"",
            "value_outside_axis",
            "/polar/points/1/value",
        ),
        (
            "\"valueLabel\"",
            "\"max\": 0, \"valueLabel\"",
            "invalid_axis_range",
            "/polar/max",
        ),
    ] {
        let spec = POLAR.replace(from, to);
        assert_eq!(render_err(&spec), (code, path.to_owned()), "{from}");
    }
}

#[test]
fn a_polar_chart_needs_three_points() {
    let two = r#"{"schemaVersion": 1, "type": "polar", "title": "T", "polar": {"points": [{"label": "A", "value": 1}, {"label": "B", "value": 2}]}}"#;
    assert_eq!(render_err(two), ("empty_data", "/polar/points".to_owned()));
}
