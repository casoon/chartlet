use super::*;

const WAFFLE: &str = r#"{"schemaVersion": 1, "type": "waffle", "title": "Mix", "width": 600, "height": 360,
        "waffle": {"parts": [{"label": "A", "value": 60}, {"label": "B", "value": 30}, {"label": "C", "value": 0.4}]}}"#;

#[test]
fn a_waffle_fills_every_square_and_gives_each_part_at_least_one() {
    let output = render_ok(WAFFLE);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    // 100 squares, and one swatch for each of three parts.
    assert_eq!(
        svg.matches("chartlet-waffle-cell chartlet-waffle-").count(),
        100 + 3
    );
    let spec = ChartSpec::from_json(WAFFLE).unwrap();
    let squares = spec.waffle.as_ref().unwrap().squares();
    assert_eq!(squares.iter().sum::<u32>(), 100);
    assert_eq!(squares, vec![66, 33, 1]);
    let html = render_json(WAFFLE, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("<th scope=\"col\">Part</th><th scope=\"col\">Value</th><th scope=\"col\">Share</th><th scope=\"col\">Squares</th>"));
    assert!(html.contains("<th scope=\"row\">C</th><td>0.4</td><td>0.4%</td><td>1</td>"));
    // A total above the sum of the parts leaves squares for the rest.
    let with_rest = WAFFLE.replace("\"parts\"", "\"total\": 200, \"parts\"");
    let spec = ChartSpec::from_json(&with_rest).unwrap();
    assert_eq!(spec.waffle.as_ref().unwrap().squares(), vec![30, 15, 1, 54]);
}

#[test]
fn waffles_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""value": 30"#,
            r#""value": 0"#,
            "invalid_value",
            "/waffle/parts/1/value",
        ),
        (
            r#""label": "B""#,
            r#""label": "A""#,
            "duplicate_label",
            "/waffle/parts/1/label",
        ),
        (
            r#""parts""#,
            r#""cells": 5, "parts""#,
            "invalid_cells",
            "/waffle/cells",
        ),
        (
            r#""parts""#,
            r#""total": 10, "parts""#,
            "invalid_total",
            "/waffle/total",
        ),
    ] {
        let error = render_json(
            &WAFFLE.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}

#[test]
fn a_nested_treemap_frames_the_parts_of_an_item() {
    let spec = r#"{"schemaVersion": 1, "type": "treemap", "title": "Budget", "width": 640, "height": 400,
            "treemap": {"items": [
                {"label": "Society", "children": [{"label": "Schools", "value": 60}, {"label": "Parks", "value": 10}]},
                {"label": "Roads", "children": [{"label": "Rail", "value": 30}]}
            ]}}"#;
    let output = render_ok(spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(
        svg.matches("class=\"chartlet-tm-frame chartlet-tm-1\"")
            .count(),
        1
    );
    assert_eq!(
        svg.matches("class=\"chartlet-tm-cell chartlet-tm-1\"")
            .count(),
        2 + 1
    );
    assert!(svg.contains("<title>Society › Schools: 60 (60.0%)</title>"));
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("Treemap of 3 items, together 100;"), "{html}");
    assert!(html.contains(
        "<th scope=\"row\">Society › Parks</th><td>10</td><td>10.0%</td><td>Society</td>"
    ));
    for (from, to, code, path) in [
        (
            r#"{"label": "Society", "children""#,
            r#"{"label": "Society", "value": 5, "children""#,
            "value_with_children",
            "/treemap/items/0/value",
        ),
        (
            r#"{"label": "Parks", "value": 10}"#,
            r#"{"label": "Parks", "value": 10, "group": "g"}"#,
            "group_with_children",
            "/treemap/items/0/children/1/group",
        ),
        (
            r#"{"label": "Rail", "value": 30}"#,
            r#"{"label": "Rail", "children": [{"label": "A", "children": [{"label": "B", "children": [{"label": "C", "value": 1}]}]}]}"#,
            "too_deep",
            "/treemap/items/1/children/0/children/0/children",
        ),
    ] {
        let error = render_json(
            &spec.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}

const TREEMAP: &str = r#"{"schemaVersion": 1, "type": "treemap", "title": "Budget", "width": 640, "height": 400,
        "treemap": {"items": [
            {"label": "Schools", "value": 60, "group": "Society"},
            {"label": "Roads", "value": 30, "group": "Infrastructure"},
            {"label": "Parks", "value": 10, "group": "Society"}
        ]}}"#;

#[test]
fn a_treemap_gives_every_item_an_area_by_its_value_and_names_it_everywhere() {
    let output = render_ok(TREEMAP);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(
        svg.matches("class=\"chartlet-tm-cell chartlet-tm-1\"")
            .count(),
        3
    );
    assert_eq!(
        svg.matches("class=\"chartlet-tm-cell chartlet-tm-2\"")
            .count(),
        2
    );
    assert!(svg.contains("<title>Schools: 60 (60.0%)</title>"));
    let html = render_json(TREEMAP, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("Treemap of 3 items, together 100; the rectangle of each has the area of its value. Largest: Schools (60.0%), Roads (30.0%), Parks (10.0%). Groups: Society 70.0%, Infrastructure 30.0%."));
    assert!(
        html.contains(
            "<th scope=\"row\">Roads</th><td>30</td><td>30.0%</td><td>Infrastructure</td>"
        )
    );
}

#[test]
fn treemaps_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""value": 10"#,
            r#""value": 0"#,
            "invalid_value",
            "/treemap/items/2/value",
        ),
        (
            r#""label": "Roads""#,
            r#""label": "Parks""#,
            "duplicate_label",
            "/treemap/items/2/label",
        ),
        (
            r#", "group": "Society"}
        ]"#,
            r"}
        ]",
            "missing_group",
            "/treemap/items/2/group",
        ),
    ] {
        let error = render_json(
            &TREEMAP.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}

const PARLIAMENT: &str = r#"{"schemaVersion": 1, "type": "parliament", "title": "Seats", "width": 640, "height": 460,
        "parliament": {"parties": [
            {"label": "Red", "seats": 40}, {"label": "Green", "seats": 25}, {"label": "Blue", "seats": 35}
        ], "majority": true, "coalition": ["Red", "Green"]}}"#;

#[test]
fn a_parliament_draws_every_seat_in_blocks_and_names_the_majority_and_the_coalition() {
    let output = render_ok(PARLIAMENT);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(
        svg.matches("class=\"chartlet-parl-seat chartlet-parl-1\"")
            .count(),
        40 + 1
    );
    assert_eq!(
        svg.matches("class=\"chartlet-parl-seat chartlet-parl-2\"")
            .count(),
        25 + 1
    );
    assert_eq!(
        svg.matches("class=\"chartlet-parl-seat chartlet-parl-3\"")
            .count(),
        35 + 1
    );
    assert_eq!(svg.matches("class=\"chartlet-parl-coalition\"").count(), 65);
    assert_eq!(svg.matches("class=\"chartlet-parl-majority\"").count(), 1);
    assert!(svg.contains("<title>Red: seat 1 / 40</title>"));
    let html = render_json(PARLIAMENT, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("Parliament with 100 seats and 3 parties: Red 40 (40.0%), Green 25 (25.0%), Blue 35 (35.0%). A majority takes 51 seats. Coalition Red + Green: 65 seats, a majority."));
    assert!(html.contains("<th scope=\"row\">Blue</th><td>35</td><td>35.0%</td><td></td>"));
}

#[test]
fn parliaments_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""seats": 25"#,
            r#""seats": 0"#,
            "invalid_value",
            "/parliament/parties/1/seats",
        ),
        (
            r#""label": "Green""#,
            r#""label": "Red""#,
            "duplicate_label",
            "/parliament/parties/1/label",
        ),
        (
            r#""coalition": ["Red", "Green"]"#,
            r#""coalition": ["Red", "Pink"]"#,
            "unknown_node",
            "/parliament/coalition/1",
        ),
        (
            r#""seats": 40"#,
            r#""seats": 900"#,
            "too_many_data_points",
            "/parliament/parties",
        ),
    ] {
        let error = render_json(
            &PARLIAMENT.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}
