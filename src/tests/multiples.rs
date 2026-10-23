use super::*;

const PANEL_BARS: &str = r#"{"schemaVersion": 1, "type": "multiples", "title": "Benchmarks", "width": 900, "height": 400,
        "categories": ["A", "B", "C"],
        "panes": [
            {"title": "Speed", "values": [10, 20, 30], "valueAxis": {"unit": "req/s"}},
            {"title": "Memory", "values": [5, null, 3], "valueAxis": {"unit": "MB"}}
        ]}"#;

#[test]
fn small_multiples_of_bars_have_a_value_axis_per_panel_and_say_so_in_text() {
    let output = render_ok(PANEL_BARS);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    // Two panels of two and three bars; the missing value has none.
    assert_eq!(svg.matches("class=\"chartlet-bar\"").count(), 5);
    assert!(svg.contains(">req/s</text>") && svg.contains(">MB</text>"));
    assert!(svg.contains("<title>A – Speed: 10</title>"));
    assert!(svg.contains(
        "Bar chart in 2 panels over 3 categories, each with its own value axis. Speed: Highest 30 req/s (C), Lowest 10 req/s (A). Memory: Highest 5 MB (A), Lowest 3 MB (C)."
    ));
    let html = render_json(PANEL_BARS, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(html.content.contains(
        "<th scope=\"col\">Category</th><th scope=\"col\">Speed (req/s)</th><th scope=\"col\">Memory (MB)</th>"
    ));
    assert!(
        html.content
            .contains("<th scope=\"row\">B</th><td>20</td><td>Missing</td>")
    );
}

#[test]
fn small_multiples_of_bars_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""values": [10, 20, 30]"#,
            r#""values": [10, 20]"#,
            "series_length_mismatch",
            "/panes/0/values",
        ),
        (
            r#""values": [5, null, 3]"#,
            r#""values": [null, null, null]"#,
            "empty_data",
            "/panes/1/values",
        ),
        (
            r#""title": "Memory""#,
            r#""title": "Speed""#,
            "duplicate_title",
            "/panes/1/title",
        ),
        (
            r#""categories": ["A", "B", "C"]"#,
            r#""categories": ["A", "A", "C"]"#,
            "duplicate_label",
            "/categories/1",
        ),
        (
            r#""title": "Benchmarks""#,
            r#""title": "Benchmarks", "valueAxis": {"unit": "ms"}"#,
            "option_not_supported",
            "/valueAxis",
        ),
        (
            r#""title": "Speed", "values": [10, 20, 30]"#,
            r#""title": "Speed", "values": [10, 20, 30], "layers": [{"mark": "line", "points": []}]"#,
            "option_not_supported",
            "/panes/0/layers",
        ),
    ] {
        let json = PANEL_BARS.replace(from, to);
        let error = render_json(&json, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
    // Values belong to bars: a time chart's panels take layers.
    let time = r#"{"schemaVersion": 1, "type": "multiples", "title": "T", "width": 600, "height": 300,
            "panes": [{"title": "A", "values": [1], "layers": []}, {"title": "B", "layers": []}]}"#;
    let error = render_json(time, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "option_not_supported");
    assert_eq!(error.path, "/panes/0/values");
}

#[test]
fn small_multiples_share_one_scale_and_one_legend() {
    let spec = multiples(
        &[
            panel("Energy", 2.0),
            panel("Industry", 1.0),
            panel("Transport", 0.5),
        ]
        .join(","),
    );
    let output = render_ok(&spec);
    let svg = &output.content;
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert!(svg.contains("class=\"chartlet-root chartlet-multiples\""));
    for title in ["Energy", "Industry", "Transport"] {
        assert!(svg.contains(&format!(">{title}</text>")));
    }
    // One shared scale: the top tick of the largest panel appears in every panel.
    assert_eq!(svg.matches(">30</text>").count(), 3);
    // One legend entry per name, not per panel; the same name keeps its color everywhere.
    assert_eq!(svg.matches("class=\"chartlet-legend\"").count(), 2);
    assert!(svg.contains(">Pathway (modelled)<"));
    assert_eq!(
        svg.matches("chartlet-line chartlet-line-modeled chartlet-line-series-2")
            .count(),
        // Three panels and the legend sample.
        4
    );
    assert!(svg.contains("<title>2050 – Industry · Pathway: 0 (range 0 to 2)</title>"));
    assert!(svg.contains(
        "Time chart in 3 panels (Energy, Industry, Transport) with a shared value axis, each from 2000 to 2050, showing History, Pathway (modelled)."
    ));
    let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(
        html.content
            .contains("<th scope=\"col\">Energy · History</th>")
    );
    assert!(
        html.content
            .contains("<th scope=\"col\">Transport · Pathway (upper)</th>")
    );
}

#[test]
fn small_multiples_are_validated_by_name() {
    let two = [panel("A", 1.0), panel("B", 1.0)].join(",");
    assert_eq!(
        render_err(&multiples(&panel("A", 1.0))),
        ("not_enough_panes", "/panes".to_owned())
    );
    assert_eq!(
        render_err(&multiples(&two.replacen("\"title\": \"A\", ", "", 1))),
        ("missing_title", "/panes/0/title".to_owned())
    );
    assert_eq!(
        render_err(&multiples(
            &two.replace("\"title\": \"B\"", "\"title\": \"A\"")
        )),
        ("duplicate_title", "/panes/1/title".to_owned())
    );
    assert_eq!(
        render_err(&multiples(&two.replacen(
            "\"title\": \"A\",",
            "\"title\": \"A\", \"valueAxis\": {\"title\": \"Own\"},",
            1
        ))),
        ("option_not_supported", "/panes/0/valueAxis".to_owned())
    );
    let columns = multiples(&two).replace(
        "\"type\": \"multiples\",",
        "\"type\": \"multiples\", \"columns\": 7,",
    );
    assert_eq!(
        render_err(&columns),
        ("invalid_columns", "/columns".to_owned())
    );
    let thirteen = (0..13)
        .map(|index| panel(&format!("P{index}"), 1.0))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        render_err(&multiples(&thirteen)),
        ("too_many_panes", "/panes".to_owned())
    );
    let titled = TIME.replace(
        "\"valueAxis\": {\"title\": \"Orders\"},",
        "\"title\": \"Pane\",",
    );
    assert_eq!(
        render_err(&titled),
        ("option_not_supported", "/panes/0/title".to_owned())
    );
}
