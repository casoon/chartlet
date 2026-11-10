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
    let rules = |count: usize| {
        (0..count)
            .map(|index| {
                format!(r#"{{"mark": "annotation", "value": {index}, "label": "L{index}"}}"#)
            })
            .collect::<Vec<_>>()
            .join(",")
    };
    render_ok(&with_rules(&rules(12)));
    assert_eq!(
        render_err(&with_rules(&rules(13))),
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
    let thirteen = (0..13)
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
        render_err(&with_rules(&thirteen)),
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

/// The labels of a rendering's annotations as position and content, in document order.
fn rule_labels(svg: &str) -> Vec<(f64, f64, String)> {
    placed_texts(svg, "chartlet-rule-label")
}

/// A line that zigzags between 0 and 20 so tightly that no label near the middle escapes it.
fn zigzag() -> String {
    let points = (0..60)
        .map(|index| {
            format!(
                r#"{{"time": {}, "value": {}}}"#,
                1_772_323_200 + index * 86_400,
                (index % 2) * 20
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{"schemaVersion": 1, "type": "time", "title": "Zigzag", "showValues": false,
            "panes": [{{"layers": [
                {{"mark": "annotation", "value": 10, "label": "Limit"}},
                {{"mark": "line", "name": "Zigzag", "markers": false, "points": [{points}]}}
            ]}}]}}"#
    )
}

#[test]
fn labels_of_reference_lines_at_one_value_stand_side_by_side() {
    let spec = with_rules(
        r#"{"mark": "annotation", "value": 12, "label": "First"},
               {"mark": "annotation", "value": 12, "label": "Second"}"#,
    );
    let output = render_ok(&spec);
    assert_eq!(output.warnings, []);
    let labels = rule_labels(&output.content);
    assert_eq!(labels[0].2, "First");
    assert_eq!(labels[1].2, "Second");
    // On the same line of text, the second after the first.
    assert!((labels[0].1 - labels[1].1).abs() < 1e-9, "{labels:?}");
    assert!(labels[1].0 > labels[0].0 + 30.0, "{labels:?}");
}

#[test]
fn labels_of_neighbouring_vertical_lines_step_down_like_a_staircase() {
    let days = ["03-01", "03-01T01:00", "03-01T02:00", "03-01T03:00"];
    let rules = days
        .iter()
        .enumerate()
        .map(|(index, day)| {
            format!(r#"{{"mark": "annotation", "time": "2026-{day}", "label": "Phase {index}"}}"#)
        })
        .collect::<Vec<_>>()
        .join(",");
    let output = render_ok(&with_rules(&rules));
    assert_eq!(output.warnings, []);
    let labels = rule_labels(&output.content);
    // Each label right of its own line, a line of text below the one before it.
    for pair in labels.windows(2) {
        assert!(pair[1].0 > pair[0].0, "{labels:?}");
        assert!((pair[1].1 - pair[0].1 - 14.0).abs() < 1e-9, "{labels:?}");
    }
}

#[test]
fn labels_without_a_free_place_are_reported_not_dropped() {
    // Four lines of text on either side of the line; the ninth label finds none free.
    let rules = (0..9)
        .map(|index| {
            format!(r#"{{"mark": "annotation", "time": "2026-03-02", "label": "Event {index}"}}"#)
        })
        .collect::<Vec<_>>()
        .join(",");
    let output = render_ok(&with_rules(&rules));
    assert_eq!(rule_labels(&output.content).len(), 9);
    let paths: Vec<&str> = output
        .warnings
        .iter()
        .map(|warning| {
            assert_eq!(warning.code, "label_overlap");
            warning.path.as_str()
        })
        .collect();
    assert_eq!(paths, ["/panes/0/layers/8"]);
}

#[test]
fn a_reference_line_label_moves_off_a_data_line() {
    // The line runs along the rule on the left and falls away on the right: the label takes
    // the right end.
    let spec = TIME
        .replace("\"value\": 10}", "\"value\": 10.4}")
        .replace("\"value\": 15}", "\"value\": 10.4}")
        .replace("\"value\": 20}", "\"value\": 0}")
        .replace(
            "\"layers\": [",
            r#""layers": [{"mark": "annotation", "value": 10, "label": "Limit"},"#,
        );
    let output = render_ok(&spec);
    assert_eq!(output.warnings, []);
    assert!(
        output
            .content
            .contains("text-anchor=\"end\" class=\"chartlet-rule-label\">Limit</text>")
    );
}

#[test]
fn a_reference_line_label_across_a_data_line_is_reported() {
    let output = render_ok(&zigzag());
    assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
    assert_eq!(output.warnings[0].code, "label_overlap");
    assert_eq!(output.warnings[0].path, "/panes/0/layers/0");
    assert!(output.warnings[0].message.contains("data line"));
    // It stays where a label of a horizontal line stands first: above its left end.
    assert!(
        output
            .content
            .contains("text-anchor=\"start\" class=\"chartlet-rule-label\">Limit</text>")
    );
}

#[test]
fn a_label_that_would_leave_the_plot_goes_below_its_line() {
    // An area starts the axis at zero without padding, so 20 is the top edge of the plot and
    // a label above a reference line there would sit outside it.
    let spec = TIME
        .replace("\"mark\": \"line\"", "\"mark\": \"area\"")
        .replace(
            "\"layers\": [",
            r#""layers": [{"mark": "annotation", "value": 20, "label": "Ceiling"},"#,
        );
    let output = render_ok(&spec);
    assert_eq!(output.warnings, []);
    let svg = &output.content;
    let line = svg
        .split("class=\"chartlet-rule\"")
        .next()
        .and_then(|head| head.rsplit("<polyline points=\"").next())
        .and_then(|points| points.split([',', ' ']).nth(1))
        .and_then(|y| y.parse::<f64>().ok())
        .expect("the reference line");
    assert!(
        (rule_labels(svg)[0].1 - (line + 14.0)).abs() < 1e-9,
        "{svg}"
    );
}

#[test]
fn a_zone_label_moves_to_the_bottom_where_the_data_crosses_its_top() {
    // The line starts just below the top of the plot, where the label of the zone would stand.
    let spec = TIME
        .replace("\"value\": 10}", "\"value\": 19.5}")
        .replace("\"value\": 15}", "\"value\": 19.5}")
        .replace(
            r#"{"title": "Orders"}"#,
            r#"{"title": "Orders", "min": 0, "max": 20, "exact": true}"#,
        )
        .replace(
            "\"layers\": [",
            r#""layers": [{"mark": "band", "label": "First day", "from": "2026-03-01", "to": "2026-03-02"},"#,
        );
    let output = render_ok(&spec);
    assert_eq!(output.warnings, []);
    let label = &rule_labels(&output.content)[0];
    assert_eq!(label.2, "First day");
    // The plot of this chart ends at 394; the label stands 6 pixels above its bottom edge.
    assert!(label.1 > 300.0, "{label:?}");
}

#[test]
fn label_collisions_are_measured_with_the_injected_metrics() {
    // Wide metrics stretch the label of the horizontal line into the place of the vertical
    // one's, which steps down a line.
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
    assert!(wide.warnings.is_empty(), "{:?}", wide.warnings);
    let deploy = |svg: &str| {
        rule_labels(svg)
            .into_iter()
            .find(|(_, _, content)| content == "Deploy")
            .map(|(_, y, _)| y)
            .expect("the label of the vertical line")
    };
    assert!((deploy(&wide.content) - deploy(&narrow.content) - 14.0).abs() < 1e-9);
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
