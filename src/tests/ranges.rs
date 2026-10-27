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

const DUMBBELLS: &str = r#"{"schemaVersion": 1, "type": "rangebar", "display": "dumbbell", "orientation": "horizontal",
        "title": "Before and after", "width": 700, "height": 320, "valueAxis": {"min": 0, "max": 100},
        "ranges": [
            {"label": "Billing", "low": 49, "high": 64},
            {"label": "Mobile app", "low": 66, "high": 86},
            {"label": "Live chat", "low": 72, "high": 79}
        ]}"#;

#[test]
fn dumbbells_draw_two_dots_a_line_and_both_values() {
    for orientation in ["horizontal", "vertical"] {
        let spec = DUMBBELLS.replace("\"horizontal\"", &format!("\"{orientation}\""));
        let output = render_ok(&spec);
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert_eq!(
            svg.matches("class=\"chartlet-dumbbell-line\"").count(),
            3,
            "{orientation}"
        );
        assert_eq!(svg.matches("class=\"chartlet-dumbbell-low\"").count(), 3);
        assert_eq!(svg.matches("class=\"chartlet-dumbbell-high\"").count(), 3);
        assert!(svg.contains("<title>Mobile app: 66 to 86</title>"));
        assert!(svg.contains(".chartlet-dumbbell-low{fill:var(--chartlet-background)"));
        // The value of every end stands beside its dot; none is combined.
        for value in ["49", "64", "66", "86", "72", "79"] {
            assert!(
                svg.contains(&format!("class=\"chartlet-value\">{value}</text>")),
                "{orientation}: {value}"
            );
        }
        assert!(!svg.contains(">49 to 64<"));
        assert!(svg.contains(
            "Each row connects an open dot at the low value with a filled dot at the high value. Widest change: 20 (Mobile app)."
        ));
    }
}

#[test]
fn a_dumbbell_falls_back_to_one_label_where_the_low_value_has_no_room() {
    let spec = DUMBBELLS.replace(
        "\"min\": 0, \"max\": 100",
        "\"min\": 49, \"max\": 100, \"exact\": true",
    );
    let svg = render_ok(&spec).content;
    assert!(svg.contains(">49 to 64<"), "{svg}");
    assert!(!svg.contains(">49</text>"));
}

#[test]
fn sorting_by_change_puts_the_widest_span_first_everywhere() {
    let spec = DUMBBELLS.replace(
        "\"display\": \"dumbbell\",",
        "\"display\": \"dumbbell\", \"sort\": \"change\",",
    );
    let svg = render_ok(&spec).content;
    assert!(svg.find("<title>Mobile app").unwrap() < svg.find("<title>Billing").unwrap());
    assert!(svg.find("<title>Billing").unwrap() < svg.find("<title>Live chat").unwrap());
    let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    let row = |label: &str| html.find(&format!("<th scope=\"row\">{label}")).unwrap();
    assert!(row("Mobile app") < row("Billing") && row("Billing") < row("Live chat"));
    // Bars sort too; without the option the order is as listed.
    let bars = spec.replace("\"display\": \"dumbbell\",", "");
    let svg = render_ok(&bars).content;
    assert!(svg.find("<title>Mobile app").unwrap() < svg.find("<title>Billing").unwrap());
    let listed = DUMBBELLS.replace("\"display\": \"dumbbell\",", "");
    let svg = render_ok(&listed).content;
    assert!(svg.find("<title>Billing").unwrap() < svg.find("<title>Mobile app").unwrap());
}

#[test]
fn dumbbells_and_sorting_are_validated_by_name() {
    for (from, to, path) in [
        (
            "\"low\": 66, \"high\": 86",
            "\"low\": 66, \"high\": 86, \"mid\": 70",
            "/ranges/1/mid",
        ),
        (
            "\"low\": 66, \"high\": 86",
            "\"low\": 66, \"high\": 86, \"weight\": 2, \"mid\": 70",
            "/ranges/1/mid",
        ),
        (
            "\"low\": 66, \"high\": 86",
            "\"low\": 66, \"high\": 86, \"modeled\": true",
            "/ranges/1/modeled",
        ),
    ] {
        assert_eq!(
            render_err(&DUMBBELLS.replace(from, to)),
            ("option_not_supported", path.to_owned()),
            "{to}"
        );
    }
    for field in ["display\": \"dumbbell", "sort\": \"change"] {
        let spec = format!(
            r#"{{"schemaVersion": 1, "type": "bar", "{field}", "title": "T", "data": [{{"label": "A", "value": 1}}]}}"#
        );
        let path = format!("/{}", field.split('"').next().unwrap());
        assert_eq!(render_err(&spec), ("option_not_supported", path), "{field}");
    }
}
