use super::*;

const PNF: &str = r#"{"schemaVersion": 1, "type": "pointfigure", "title": "Count", "width": 640, "height": 400,
        "valueAxis": {"title": "Price"},
        "pointfigure": {"boxSize": 1, "columns": [
            {"direction": "down", "low": 100, "high": 104},
            {"direction": "up", "low": 101, "high": 104},
            {"direction": "down", "low": 101, "high": 103},
            {"direction": "up", "low": 102, "high": 108}
        ], "highlights": [{"from": 1, "to": 3, "label": "Counted: 3 columns"}]},
        "references": [{"value": 104, "label": "Count line 104"}]}"#;

#[test]
fn a_point_and_figure_chart_draws_one_mark_per_box() {
    let output = render_ok(PNF);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    // O: 5 + 3 boxes; X: 4 + 7 boxes, two strokes each.
    assert_eq!(svg.matches("class=\"chartlet-pnf-o\"").count(), 8);
    assert_eq!(svg.matches("class=\"chartlet-pnf-x\"").count(), 22);
    assert_eq!(svg.matches("class=\"chartlet-pnf-highlight\"").count(), 1);
    assert!(svg.contains(">Counted: 3 columns</text>"));
    assert!(svg.contains(">Count line 104</text>"));
    assert!(svg.contains("<title>Column 4: X (rising), 102 to 108 (Boxes: 7)</title>"));
    assert!(svg.contains("--chartlet-rise:") && svg.contains("--chartlet-fall:"));
    assert!(svg.contains(
        "Point-and-figure chart of 4 columns, box size 1: 2 columns of X (rising) and 2 columns of O (falling), between 100 and 108. The last column is X from 102 to 108 (7 boxes). Highlighted: columns 1 to 3 (Counted: 3 columns). Reference lines: Count line 104 at 104."
    ));
}

#[test]
fn a_point_and_figure_table_lists_every_column() {
    let html = render_json(PNF, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("<th scope=\"col\">Column</th><th scope=\"col\">Direction</th><th scope=\"col\">Lowest box</th><th scope=\"col\">Highest box</th><th scope=\"col\">Boxes</th>"));
    assert!(html.contains(
        "<th scope=\"row\">1</th><td>O (falling)</td><td>100</td><td>104</td><td>5</td>"
    ));
    let german = render_json(
        &PNF.replace("\"type\"", "\"locale\": \"de\", \"type\""),
        RenderFormat::Html,
        &RenderOptions::default(),
    )
    .unwrap()
    .content;
    assert!(german.contains(
        "<th scope=\"row\">4</th><td>X (steigend)</td><td>102</td><td>108</td><td>7</td>"
    ));
    assert!(german.contains("Point-and-Figure-Diagramm aus 4 Spalten, Kästchengröße 1"));
    assert!(german.contains("Hervorgehoben: Spalten 1 bis 3 (Counted: 3 columns)."));
}

#[test]
fn point_and_figure_boxes_may_be_fractions() {
    let spec = PNF
        .replace("\"boxSize\": 1", "\"boxSize\": 0.1")
        .replace("\"low\": 100, \"high\": 104", "\"low\": 10, \"high\": 10.4")
        .replace(
            "\"low\": 101, \"high\": 104",
            "\"low\": 10.1, \"high\": 10.4",
        )
        .replace(
            "\"low\": 101, \"high\": 103",
            "\"low\": 10.1, \"high\": 10.3",
        )
        .replace(
            "\"low\": 102, \"high\": 108",
            "\"low\": 10.2, \"high\": 10.8",
        )
        .replace("\"value\": 104", "\"value\": 10.4");
    let output = render_ok(&spec);
    assert_eq!(
        output.content.matches("class=\"chartlet-pnf-o\"").count(),
        8
    );
    assert_eq!(
        output.content.matches("class=\"chartlet-pnf-x\"").count(),
        22
    );
}

#[test]
fn point_and_figure_styles_go_to_the_shared_stylesheet() {
    let svg = render_json(
        PNF,
        RenderFormat::Svg,
        &RenderOptions {
            styles: crate::Styles::External,
            ..RenderOptions::default()
        },
    )
    .unwrap()
    .content;
    assert!(!svg.contains("<style>"), "{svg}");
    let shared = crate::stylesheet_for(&[crate::ChartType::PointFigure]);
    assert!(
        shared.contains(".chartlet-type-pointfigure .chartlet-pnf-x{stroke:var(--chartlet-rise)")
    );
    assert!(shared.contains(".chartlet-type-pointfigure .chartlet-rule{"));
}

#[test]
fn point_and_figure_charts_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""boxSize": 1"#,
            r#""boxSize": 0"#,
            "invalid_box_size",
            "/pointfigure/boxSize",
        ),
        (
            r#""low": 101, "high": 103"#,
            r#""low": 104, "high": 103"#,
            "invalid_range",
            "/pointfigure/columns/2/high",
        ),
        (
            r#""low": 101, "high": 103"#,
            r#""low": 101.5, "high": 103.5"#,
            "off_grid",
            "/pointfigure/columns/2/low",
        ),
        (
            r#""low": 101, "high": 103"#,
            r#""low": 101, "high": 103.5"#,
            "off_grid",
            "/pointfigure/columns/2/high",
        ),
        (
            r#""from": 1"#,
            r#""from": 0"#,
            "invalid_highlight",
            "/pointfigure/highlights/0/from",
        ),
        (
            r#""to": 3"#,
            r#""to": 5"#,
            "invalid_highlight",
            "/pointfigure/highlights/0/to",
        ),
        (
            r#""from": 1, "to": 3"#,
            r#""from": 3, "to": 2"#,
            "invalid_highlight",
            "/pointfigure/highlights/0/to",
        ),
        (
            r#""valueAxis": {"title": "Price"}"#,
            r#""valueAxis": {"scale": "log"}"#,
            "option_not_supported",
            "/valueAxis/scale",
        ),
        (
            r#""valueAxis": {"title": "Price"}"#,
            r#""valueAxis": {"min": 100, "max": 105, "exact": true}"#,
            "value_outside_axis",
            "/pointfigure/columns/3/high",
        ),
        (
            r#""valueAxis": {"title": "Price"}"#,
            r#""timeAxis": {"kind": "number"}"#,
            "option_not_supported",
            "/timeAxis",
        ),
    ] {
        let spec = PNF.replace(from, to);
        assert_ne!(spec, PNF, "{from}");
        assert_eq!(render_err(&spec), (code, path.to_owned()), "{to}");
    }
    let missing = r#"{"schemaVersion": 1, "type": "pointfigure", "title": "Count"}"#;
    assert_eq!(
        render_err(missing),
        ("missing_pointfigure", "/pointfigure".to_owned())
    );
    let foreign = r#"{"schemaVersion": 1, "type": "bar", "title": "Count", "data": [{"label": "A", "value": 1}],
        "pointfigure": {"boxSize": 1, "columns": [{"direction": "up", "low": 1, "high": 2}]}}"#;
    assert_eq!(
        render_err(foreign),
        ("option_not_supported", "/pointfigure".to_owned())
    );
}

#[test]
fn a_crowded_point_and_figure_chart_is_reported() {
    let columns: Vec<String> = (0..200)
        .map(|index| {
            let direction = if index % 2 == 0 { "up" } else { "down" };
            format!(r#"{{"direction": "{direction}", "low": 100, "high": 102}}"#)
        })
        .collect();
    let spec = format!(
        r#"{{"schemaVersion": 1, "type": "pointfigure", "title": "Dense", "width": 480, "height": 300,
            "pointfigure": {{"boxSize": 1, "columns": [{}]}}}}"#,
        columns.join(",")
    );
    let output = render_ok(&spec);
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "dense_chart" && warning.path == "/pointfigure/columns"),
        "{:?}",
        output.warnings
    );
    // Only every 20th column is numbered, so the numbers stay apart.
    assert!(output.content.contains(">20</text>"));
    assert!(!output.content.contains(">19</text>"));
}
