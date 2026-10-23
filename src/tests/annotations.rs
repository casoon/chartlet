use super::*;

#[test]
fn a_reference_line_crosses_the_bars_and_is_described() {
    let spec = SPEC.replacen(
        '{',
        "{\"references\": [{\"value\": 20, \"label\": \"Target\"}],",
        1,
    );
    let output = render_json(&spec, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(output.content.contains("class=\"chartlet-rule\""));
    assert!(output.content.contains(">Target</text>"));
    assert!(output.content.contains("Reference lines: Target at 20."));
    let line = SPEC
        .replace("\"type\": \"bar\"", "\"type\": \"line\"")
        .replacen(
            '{',
            "{\"references\": [{\"value\": 1, \"label\": \"T\"}],",
            1,
        );
    assert_eq!(
        render_err(&line),
        ("option_not_supported", "/references".to_owned())
    );
}

#[test]
fn reference_lines_are_drawn_labelled_and_described() {
    let spec = with_rules(
        r#"{"mark": "annotation", "value": 30, "label": "Target"},
               {"mark": "annotation", "time": "2026-03-02", "label": "Launch"}"#,
    );
    let output = render_ok(&spec);
    let svg = &output.content;
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert_eq!(svg.matches("class=\"chartlet-rule\"").count(), 2);
    assert!(svg.contains(">Target</text>"));
    assert!(svg.contains(">Launch</text>"));
    assert!(svg.contains("<title>Target: 30</title>"));
    assert!(svg.contains("<title>Launch: 2026-03-02</title>"));
    // 30 lies above every observation, so the axis reaches it.
    assert!(svg.contains(">30<"));
    assert!(svg.contains("Reference lines: Target at 30; Launch at 2026-03-02."));
    // Reference lines are neither series in the legend nor columns in the table.
    assert!(!svg.contains("class=\"chartlet-legend\""));
    let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(
        html.content
            .contains("<th scope=\"col\">Time</th><th scope=\"col\">Orders</th></tr>")
    );
}

#[test]
fn reference_lines_are_validated_by_name() {
    for (rule, code, path) in [
        (
            r#"{"mark": "annotation", "value": 1}"#,
            "missing_label",
            "/panes/0/layers/0/label",
        ),
        (
            r#"{"mark": "annotation", "label": "Nowhere"}"#,
            "missing_position",
            "/panes/0/layers/0",
        ),
        (
            r#"{"mark": "annotation", "value": 1, "label": "L", "shape": "circle"}"#,
            "option_not_supported",
            "/panes/0/layers/0/shape",
        ),
        (
            r#"{"mark": "annotation", "value": 1, "label": "L", "name": "N"}"#,
            "option_not_supported",
            "/panes/0/layers/0/name",
        ),
        (
            r#"{"mark": "annotation", "time": "1600-01-01", "label": "L"}"#,
            "invalid_time",
            "/panes/0/layers/0/time",
        ),
    ] {
        assert_eq!(
            render_err(&with_rules(rule)),
            (code, path.to_owned()),
            "{rule}"
        );
    }
    let only_rules = r#"{"schemaVersion": 1, "type": "time", "title": "T", "panes": [
            {"layers": [{"mark": "annotation", "value": 1, "label": "L"}]}]}"#;
    assert_eq!(
        render_err(only_rules),
        ("empty_data", "/panes/0/layers".to_owned())
    );
    let seven = (0..7)
        .map(|index| format!(r#"{{"mark": "annotation", "value": {index}, "label": "L{index}"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        render_err(&with_rules(&seven)),
        ("too_many_annotations", "/panes/0/layers".to_owned())
    );
}

#[test]
fn zones_are_drawn_behind_the_data_labelled_and_described() {
    let spec = with_rules(
        r##"{"mark": "band", "label": "Quiet", "from": "2026-03-01", "to": "2026-03-02"},
               {"mark": "band", "label": "Target", "bottom": 25, "top": 40, "color": "#16a34a"}"##,
    );
    let output = render_ok(&spec);
    let svg = &output.content;
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert_eq!(svg.matches("class=\"chartlet-zone").count(), 2);
    assert!(svg.contains("class=\"chartlet-zone chartlet-style-1-swatch\""));
    assert!(svg.contains(".chartlet-style-1-swatch{fill:#16a34a}"));
    // Behind the data: every zone comes before the line.
    assert!(svg.rfind("chartlet-zone\"").unwrap() < svg.find("class=\"chartlet-line").unwrap());
    assert!(svg.contains(">Quiet</text>"));
    assert!(svg.contains(">Target</text>"));
    assert!(svg.contains("<title>Quiet: 2026-03-01 to 2026-03-02</title>"));
    assert!(svg.contains("<title>Target: values 25 to 40</title>"));
    // 40 lies above every observation, so the axis reaches it.
    assert!(svg.contains(">40<"));
    assert!(svg.contains("Zones: Quiet, 2026-03-01 to 2026-03-02; Target, values 25 to 40."));
    // Zones are neither series in the legend nor columns in the table.
    assert!(!svg.contains("class=\"chartlet-legend\""));
    let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(
        html.content
            .contains("<th scope=\"col\">Time</th><th scope=\"col\">Orders</th></tr>")
    );
}

#[test]
fn a_zone_with_one_open_end_reaches_the_plot_edge_and_says_so() {
    let spec = with_rules(
        r#"{"mark": "band", "label": "Later", "from": "2026-03-02", "bottom": 0, "top": 5}"#,
    );
    let svg = render_ok(&spec).content;
    assert!(svg.contains("<title>Later: from 2026-03-02, values 0 to 5</title>"));
}

#[test]
fn zones_are_validated_by_name() {
    for (zone, code, path) in [
        (
            r#"{"mark": "band", "bottom": 0, "top": 1}"#,
            "missing_label",
            "/panes/0/layers/0/label",
        ),
        (
            r#"{"mark": "band", "label": "Z"}"#,
            "missing_position",
            "/panes/0/layers/0",
        ),
        (
            r#"{"mark": "band", "label": "Z", "bottom": 2, "top": 2}"#,
            "invalid_band",
            "/panes/0/layers/0/top",
        ),
        (
            r#"{"mark": "band", "label": "Z", "from": "2026-03-02", "to": "2026-03-01"}"#,
            "invalid_band",
            "/panes/0/layers/0/to",
        ),
        (
            r#"{"mark": "band", "label": "Z", "from": "1600-01-01", "to": "2026-03-01"}"#,
            "invalid_time",
            "/panes/0/layers/0/from",
        ),
        (
            r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1, "name": "N"}"#,
            "option_not_supported",
            "/panes/0/layers/0/name",
        ),
        (
            r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1, "value": 1}"#,
            "option_not_supported",
            "/panes/0/layers/0/value",
        ),
        (
            r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1, "shape": "circle"}"#,
            "option_not_supported",
            "/panes/0/layers/0/shape",
        ),
        (
            r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1, "dash": "dotted"}"#,
            "option_not_supported",
            "/panes/0/layers/0/dash",
        ),
    ] {
        assert_eq!(
            render_err(&with_rules(zone)),
            (code, path.to_owned()),
            "{zone}"
        );
    }
    // Zones, reference lines and point markers share one limit.
    let seven = (0..7)
        .map(|index| match index % 3 {
            0 => format!(r#"{{"mark": "band", "bottom": {index}, "top": 50, "label": "Z{index}"}}"#),
            1 => format!(r#"{{"mark": "annotation", "value": {index}, "label": "R{index}"}}"#),
            _ => format!(
                r#"{{"mark": "annotation", "time": "2026-03-02", "value": {index}, "label": "M{index}"}}"#
            ),
        })
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        render_err(&with_rules(&seven)),
        ("too_many_annotations", "/panes/0/layers".to_owned())
    );
}

#[test]
fn point_markers_are_drawn_in_their_shape_labelled_and_described() {
    let spec = with_rules(
        r##"{"mark": "annotation", "time": "2026-03-02", "value": 50, "label": "Launch", "shape": "triangle-up"},
               {"mark": "annotation", "time": "2026-03-01", "value": 12, "label": "Start"},
               {"mark": "annotation", "time": "2026-03-02", "value": 5, "label": "Dip", "shape": "diamond", "color": "#16a34a"}"##,
    );
    let output = render_ok(&spec);
    let svg = &output.content;
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    // Shapes other than the circle are closed outlines; the default is a circle.
    assert_eq!(svg.matches("class=\"chartlet-marker").count(), 3);
    assert!(svg.contains("<circle cx="));
    assert!(svg.contains("class=\"chartlet-marker\"><title>Start: 2026-03-01, 12</title>"));
    assert!(svg.contains("<title>Launch: 2026-03-02, 50</title>"));
    assert!(
        svg.contains(
            "class=\"chartlet-marker chartlet-style-2\"><title>Dip: 2026-03-02, 5</title>"
        )
    );
    assert!(svg.contains(
        ".chartlet-marker.chartlet-style-2{fill:#16a34a;stroke:var(--chartlet-background)}"
    ));
    assert!(svg.contains(">Launch</text>"));
    // 50 lies above every observation, so the axis reaches it.
    assert!(svg.contains(">50<"));
    assert!(svg.contains(
        "Markers: Launch at 2026-03-02, 50; Start at 2026-03-01, 12; Dip at 2026-03-02, 5."
    ));
    assert!(!svg.contains("Reference lines"));
    assert!(!svg.contains("class=\"chartlet-legend\""));
}

#[test]
fn a_point_marker_needs_a_label_and_only_it_takes_a_shape() {
    assert_eq!(
        render_err(&with_rules(
            r#"{"mark": "annotation", "time": "2026-03-02", "value": 5}"#
        )),
        ("missing_label", "/panes/0/layers/0/label".to_owned())
    );
    let error = render_json(
        &with_rules(
            r#"{"mark": "annotation", "time": "2026-03-02", "label": "L", "shape": "square"}"#,
        ),
        RenderFormat::Svg,
        &RenderOptions::default(),
    )
    .expect_err("a reference line has no shape");
    assert_eq!(error.code, "option_not_supported");
    assert_eq!(error.path, "/panes/0/layers/0/shape");
    assert!(error.message.contains("point marker"), "{}", error.message);
}

#[test]
fn a_marker_label_moves_to_the_left_at_the_right_edge() {
    let spec =
        with_rules(r#"{"mark": "annotation", "time": "2026-03-03", "value": 20, "label": "Peak"}"#);
    let output = render_ok(&spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert!(
        output
            .content
            .contains("text-anchor=\"end\" class=\"chartlet-rule-label\">Peak</text>")
    );
}

#[test]
fn overlapping_annotation_labels_are_reported_not_dropped() {
    let spec = with_rules(
        r#"{"mark": "annotation", "value": 12, "label": "First"},
               {"mark": "annotation", "value": 12, "label": "Second"}"#,
    );
    let output = render_ok(&spec);
    assert!(output.content.contains(">First</text>"));
    assert!(output.content.contains(">Second</text>"));
    assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
    assert_eq!(output.warnings[0].code, "label_overlap");
    assert_eq!(output.warnings[0].path, "/panes/0/layers/1");
}

#[test]
fn a_reference_line_label_across_a_data_line_is_reported() {
    let spec = TIME
        .replace("\"value\": 10}", "\"value\": 10.4}")
        .replace("\"value\": 15}", "\"value\": 10.4}")
        .replace("\"value\": 20}", "\"value\": 0}")
        .replace(
            "\"layers\": [",
            r#""layers": [{"mark": "annotation", "value": 10, "label": "Limit"},"#,
        );
    let output = render_ok(&spec);
    assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
    assert_eq!(output.warnings[0].code, "label_overlap");
    assert_eq!(output.warnings[0].path, "/panes/0/layers/0");
    assert!(output.warnings[0].message.contains("data line"));
}

#[test]
fn a_label_outside_the_plot_is_reported() {
    // An area starts the axis at zero without padding, so 20 is the top edge of the plot and
    // the label of a reference line there sits above it.
    let spec = TIME
        .replace("\"mark\": \"line\"", "\"mark\": \"area\"")
        .replace(
            "\"layers\": [",
            r#""layers": [{"mark": "annotation", "value": 20, "label": "Ceiling"},"#,
        );
    let output = render_ok(&spec);
    assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
    assert_eq!(output.warnings[0].code, "label_overlap");
    assert_eq!(output.warnings[0].path, "/panes/0/layers/0");
    assert!(output.warnings[0].message.contains("outside"));
}

#[test]
fn label_collisions_are_measured_with_the_injected_metrics() {
    // Wide metrics stretch the label of the horizontal line into that of the vertical one.
    struct Wide;
    impl TextMetrics for Wide {
        fn width(&self, text: &str, font_size: f64) -> f64 {
            BuiltinMetrics.width(text, font_size) * 4.0
        }
    }
    let spec = ChartSpec::from_json(&with_rules(
        r#"{"mark": "annotation", "value": 24, "label": "Ceiling"},
               {"mark": "annotation", "time": "2026-03-01T06:00:00Z", "label": "Deploy"}"#,
    ))
    .unwrap();
    let narrow = render_ok(&serde_json::to_string(&spec).unwrap());
    assert!(narrow.warnings.is_empty(), "{:?}", narrow.warnings);
    let wide =
        render_with_metrics(&spec, RenderFormat::Svg, &RenderOptions::default(), &Wide).unwrap();
    assert!(
        wide.warnings
            .iter()
            .any(|warning| warning.code == "label_overlap"),
        "{:?}",
        wide.warnings
    );
}

#[test]
fn zones_and_markers_are_described_in_german() {
    let spec = with_rules(
        r#"{"mark": "band", "label": "Ruhe", "from": "2026-03-01", "to": "2026-03-02"},
               {"mark": "band", "label": "Ziel", "bottom": 2.5},
               {"mark": "annotation", "time": "2026-03-02", "value": 12.5, "label": "Start"}"#,
    )
    .replace(
        "\"schemaVersion\": 1,",
        "\"schemaVersion\": 1, \"locale\": \"de\",",
    );
    // A zone with one edge covers everything above it, up to the edge of the plot.
    let open = render_ok(&spec).content;
    assert!(open.contains("Ziel, Werte über 2,5."), "{open}");
    let bare = spec.replace(r#", "bottom": 2.5}"#, "}");
    assert_eq!(
        render_err(&bare),
        ("missing_position", "/panes/0/layers/1".to_owned())
    );
    let spec = spec.replace(
        r#""bottom": 2.5}"#,
        r#""bottom": 2.5, "from": "2026-03-02", "to": "2026-03-03"}"#,
    );
    let svg = render_ok(&spec).content;
    assert!(svg.contains(
        "Bereiche: Ruhe, 01.03.2026 bis 02.03.2026; Ziel, 02.03.2026 bis 03.03.2026, Werte über 2,5."
    ), "{svg}");
    assert!(svg.contains("Markierungen: Start bei 02.03.2026, 12,5."));
    assert!(svg.contains("<title>Start: 02.03.2026, 12,5</title>"));
}

#[test]
fn a_zoom_window_cuts_zones_and_leaves_out_those_it_misses() {
    let spec = with_rules(
        r#"{"mark": "band", "label": "Early", "from": "2026-03-01", "to": "2026-03-02"},
               {"mark": "band", "label": "Wide", "from": "2026-02-20", "to": "2026-03-10"}"#,
    )
    .replace(
        "\"timeAxis\"",
        r#""zoomSteps": [{"label": "All", "from": "2026-03-01", "to": "2026-03-03"},
                             {"label": "Late", "from": "2026-03-02", "to": "2026-03-03"}],
               "timeAxis""#,
    );
    let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    // Both zones in the first window, only the wide one in the second.
    assert_eq!(html.matches("class=\"chartlet-zone\"").count(), 3);
    assert!(html.contains("<title>Wide: 2026-03-02 to 2026-03-03</title>"));
}

#[test]
fn value_labels_keep_off_reference_lines_and_off_each_other() {
    let bars = r#"{"schemaVersion": 1, "type": "bar", "orientation": "horizontal", "title": "T", "width": 700, "height": 300,
            "data": [{"label": "A", "value": 68}, {"label": "B", "value": 40}],
            "references": [{"label": "Target", "value": 70}]}"#;
    let svg = render_ok(bars).content;
    let line = svg
        .find("class=\"chartlet-rule\"")
        .expect("a reference line");
    let rule_x: f64 = {
        let points = &svg[..line];
        let at = points.rfind("points=\"").expect("points") + 8;
        points[at..].split(',').next().unwrap().parse().unwrap()
    };
    let label = svg.find(">68</text>").expect("the label");
    let x: f64 = {
        let head = &svg[..label];
        let at = head.rfind("x=\"").expect("x") + 3;
        head[at..].split('"').next().unwrap().parse().unwrap()
    };
    assert!(x > rule_x, "{x} should be right of {rule_x}");
    let time = r#"{"schemaVersion": 1, "type": "time", "title": "T", "width": 800, "height": 400, "showValues": true,
            "panes": [{"layers": [{"mark": "line", "name": "A", "points": [
                {"time": "2024-01-01", "value": 62.5}, {"time": "2024-01-02", "value": 63.5}, {"time": "2024-06-01", "value": 70}]}]}]}"#;
    let output = render_ok(time);
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "value_labels_omitted")
    );
    assert!(output.content.contains(">62.5</text>") && !output.content.contains(">63.5</text>"));
}
