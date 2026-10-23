use super::*;

#[test]
fn range_bars_draw_spans_marks_and_hatching() {
    for orientation in ["horizontal", "vertical"] {
        let spec = RANGES.replace("\"horizontal\"", &format!("\"{orientation}\""));
        let output = render_ok(&spec);
        let svg = &output.content;
        assert_eq!(
            svg.matches("class=\"chartlet-range\"").count(),
            3,
            "{orientation}"
        );
        assert_eq!(svg.matches("class=\"chartlet-range-hatch\"").count(), 2);
        assert_eq!(svg.matches("class=\"chartlet-range-mid\"").count(), 1);
        assert!(svg.contains("-hatch-range\" width=\"6\""));
        assert!(svg.contains("<title>Observed: 1.05 (0.9 to 1.2)</title>"));
        assert!(svg.contains("<title>Natural: −0.1 to 0.1, modelled</title>"));
        assert!(svg.contains(">Hatched: modelled<"));
        // The legend text needs its own color; without it a dark chart draws it black.
        assert!(svg.contains(".chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}"));
        assert!(svg.contains(
            "Range chart with 2 categories, each a span from low to high and a central value. Lowest low: −0.1 (Natural). Highest high: 1.2 (Observed). Modelled, drawn hatched: Natural."
        ));
    }
    let html = render_json(RANGES, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(html.content.contains(
        "<th scope=\"col\">Low</th><th scope=\"col\">Mid</th><th scope=\"col\">High</th>"
    ));
    assert!(html.content.contains(
        "<th scope=\"row\">Natural (modelled)</th><td>−0.1</td><td>Missing</td><td>0.1</td>"
    ));
}

#[test]
fn a_span_of_one_value_is_labeled_with_that_value() {
    let json = r#"{"schemaVersion": 1, "type": "rangebar", "title": "Share", "width": 500, "height": 240,
            "ranges": [
                {"label": "Single", "low": 3, "high": 3},
                {"label": "Single mid", "low": 3, "high": 3, "mid": 3},
                {"label": "Span", "low": 2, "high": 4, "mid": 3}
            ]}"#;
    let svg = render_ok(json).content;
    assert!(svg.contains("<title>Single: 3</title>"));
    assert!(svg.contains("<title>Single mid: 3</title>"));
    assert!(svg.contains("<title>Span: 3 (2 to 4)</title>"));
    assert!(!svg.contains("3 to 3"));
}

#[test]
fn ranges_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            "\"low\": 0.9, \"high\": 1.2",
            "\"low\": 1.3, \"high\": 1.2",
            "invalid_range",
            "/ranges/0/high",
        ),
        (
            "\"mid\": 1.05",
            "\"mid\": 2",
            "mid_outside_range",
            "/ranges/0/mid",
        ),
        (
            "\"label\": \"Natural\"",
            "\"label\": \"Observed\"",
            "duplicate_label",
            "/ranges/1/label",
        ),
        (
            "\"orientation\": \"horizontal\",",
            "\"orientation\": \"horizontal\", \"data\": [{\"label\": \"A\", \"value\": 1}],",
            "option_not_supported",
            "/data",
        ),
    ] {
        assert_eq!(
            render_err(&RANGES.replace(from, to)),
            (code, path.to_owned()),
            "{to}"
        );
    }
}
