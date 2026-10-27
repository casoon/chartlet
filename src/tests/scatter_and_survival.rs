use super::*;

const SCATTER: &str = r#"{"schemaVersion": 1, "type": "scatter", "title": "Plot", "width": 640, "height": 440,
        "scatter": {"xTitle": "Effect", "yTitle": "Significance",
            "lines": [{"axis": "x", "value": 1, "label": "Cutoff"}, {"axis": "y", "value": 2}],
            "points": [
                {"x": -1, "y": 0.5, "group": "Low"}, {"x": 2, "y": 3, "group": "High", "label": "Gene A"},
                {"x": 0.2, "y": 1.1, "group": "Low"}, {"x": 1.5, "y": 2.5, "group": "High"}
            ]}}"#;

#[test]
fn a_scatter_plot_draws_points_lines_and_names() {
    let output = render_ok(SCATTER);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(
        svg.matches("class=\"chartlet-sc-dot chartlet-sc-1\"")
            .count(),
        2
    );
    assert_eq!(
        svg.matches("class=\"chartlet-sc-dot chartlet-sc-2\"")
            .count(),
        2
    );
    assert_eq!(svg.matches("class=\"chartlet-sc-line\"").count(), 2);
    assert!(svg.contains("<title>Gene A: 2, 3</title>"));
    assert!(svg.contains(">Gene A</text>"));
    let html = render_json(SCATTER, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("Scatter plot of 4 points: x from −1 to 2, y from 0.5 to 3. Groups (points): Low 2, High 2. Lines: Cutoff: x = 1, 2 points above it; y = 2, 2 points above it."));
    assert!(html.contains("<th scope=\"row\">Gene A</th><td>High</td><td>2</td><td>3</td>"));
}

#[test]
fn a_big_scatter_plot_lists_only_its_named_points() {
    let points: Vec<String> = (0..400)
        .map(|i| {
            let label = if i == 7 { r#", "label": "Seven""# } else { "" };
            format!(r#"{{"x": {i}, "y": {}{label}}}"#, i % 13)
        })
        .collect();
    let spec = format!(
        r#"{{"schemaVersion": 1, "type": "scatter", "title": "Big", "scatter": {{"points": [{}]}}}}"#,
        points.join(",")
    );
    let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("1 of 400 points, the named or the highest"));
    assert_eq!(html.matches("<th scope=\"row\">").count(), 1);
    let svg = render_ok(&spec).content;
    assert_eq!(
        svg.matches("<title>").count(),
        1,
        "only the named point has a tooltip"
    );
}

#[test]
fn scatter_plots_are_validated_by_name() {
    let error = render_json(
        &SCATTER.replace(r#""group": "High", "label""#, r#""label""#),
        RenderFormat::Svg,
        &RenderOptions::default(),
    )
    .unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("missing_group", "/scatter/points/1/group")
    );
    let bar = SCATTER.replace(r#""type": "scatter""#, r#""type": "bar""#);
    let error = render_json(&bar, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.path, "/scatter");
    for (from, to) in [
        (r#""axis": "y""#, r#""axis": "z""#),
        (r#""x": -1"#, r#""x": "a""#),
    ] {
        let spec = SCATTER.replace(from, to);
        assert!(render_json(&spec, RenderFormat::Svg, &RenderOptions::default()).is_err());
    }
}

const QUADRANTS: &str = r#"{"schemaVersion": 1, "type": "scatter", "title": "Priorities", "width": 640, "height": 420,
        "scatter": {"xRange": [0, 10], "yRange": [0, 10], "xTitle": "Effort", "yTitle": "Impact",
            "quadrants": {"x": 5, "y": 5, "labels": {"topLeft": "Quick wins", "topRight": "Major projects", "bottomRight": "Thankless"}},
            "points": [{"x": 2, "y": 8, "label": "SSO"}, {"x": 3, "y": 7}, {"x": 7, "y": 8, "label": "Dashboard"},
                {"x": 2, "y": 2, "label": "Import"}, {"x": 5, "y": 3, "label": "Icons"}]}}"#;

#[test]
fn a_scatter_plot_names_its_quadrants_and_says_where_each_point_lies() {
    let output = render_ok(QUADRANTS);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    for name in ["Quick wins", "Major projects", "Thankless"] {
        assert!(svg.contains(&format!(">{name}</text>")), "{name}");
    }
    let html = html_ok(QUADRANTS);
    assert!(
        html.contains(
            "Quadrants divided at x = 5 and y = 5. Top left, Quick wins: 2 points (SSO, and 1 more). Top right, Major projects: 1 point (Dashboard). Bottom left: 1 point (Import). Bottom right, Thankless: no points. On a dividing line: 1 point (Icons)."
        ),
        "{html}"
    );
    assert!(html.contains("<th scope=\"col\">Quadrant</th>"), "{html}");
    for cell in ["Quick wins", "bottom left", "on a dividing line"] {
        assert!(html.contains(&format!("<td>{cell}</td>")), "{cell}");
    }
    let german = QUADRANTS.replace(
        r#""title": "Priorities","#,
        r#""title": "Prioritäten", "locale": "de","#,
    );
    let html = html_ok(&german);
    assert!(
        html.contains("Quadranten geteilt bei x = 5 und y = 5. Oben links, Quick wins: 2 Punkte (SSO und 1 weitere).")
            && html.contains("Unten rechts, Thankless: keine Punkte. Auf einer Trennlinie: 1 Punkt (Icons).")
            && html.contains("<td>unten links</td>"),
        "{html}"
    );
}

#[test]
fn a_quadrant_name_moves_to_a_free_corner_of_its_area() {
    // A point in the top left corner of the plot: the name moves to the top right corner of
    // its area, written to end at the dividing line.
    let crowded = QUADRANTS.replace(r#"{"x": 3, "y": 7}"#, r#"{"x": 0.3, "y": 9.8}"#);
    let svg = render_ok(&crowded).content;
    let start = svg.find(">Quick wins</text>").expect("the name is drawn");
    let text = &svg[svg[..start].rfind("<text").expect("a text element")..start];
    assert!(text.contains("text-anchor=\"end\""), "{text}");
}

#[test]
fn invalid_quadrants_name_the_field() {
    for (from, to, code, path) in [
        (
            r#""x": 5, "y": 5"#,
            r#""x": 10, "y": 5"#,
            "out_of_range",
            "/scatter/quadrants/x",
        ),
        (
            r#""x": 5, "y": 5"#,
            r#""x": 5, "y": 0"#,
            "out_of_range",
            "/scatter/quadrants/y",
        ),
        (
            r#""topLeft": "Quick wins""#,
            r#""topLeft": " ""#,
            "empty_text",
            "/scatter/quadrants/labels/topLeft",
        ),
    ] {
        assert_eq!(
            render_err(&QUADRANTS.replace(from, to)),
            (code, path.to_owned()),
            "{to}"
        );
    }
    let log = QUADRANTS
        .replace(r#""xRange": [0, 10], "#, r#""xScale": "log", "#)
        .replace(r#""x": 5, "y": 5"#, r#""x": -1, "y": 5"#);
    assert_eq!(
        render_err(&log),
        ("invalid_value", "/scatter/quadrants/x".to_owned())
    );
    // Without a fixed range the axis reaches the dividing line; one on its end leaves two
    // areas without room, and says so.
    let edge = QUADRANTS
        .replace(r#""xRange": [0, 10], "#, "")
        .replace(r#""x": 5, "y": 5"#, r#""x": 0, "y": 5"#)
        .replace(
            r#"{"x": 5, "y": 3, "label": "Icons"}"#,
            r#"{"x": 6, "y": 3}"#,
        );
    let output = render_ok(&edge);
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "quadrant_without_room"
                && warning.path == "/scatter/quadrants/x"),
        "{:?}",
        output.warnings
    );
}

#[test]
fn a_scatter_plot_can_name_its_x_ticks_and_fix_its_ranges() {
    let spec = r#"{"schemaVersion": 1, "type": "scatter", "title": "Genome", "width": 640, "height": 400,
            "scatter": {"xRange": [0, 100], "yRange": [0, 10],
                "xTicks": [{"at": 25, "label": "1"}, {"at": 75, "label": "2"}],
                "points": [{"x": 10, "y": 1}, {"x": 60, "y": 9}, {"x": 90, "y": 4}]}}"#;
    let output = render_ok(spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert!(output.content.contains(">1</text>") && output.content.contains(">2</text>"));
    // The ends are exactly the range: no tick label 120 or the like.
    assert!(!output.content.contains(">20</text>") || output.content.contains(">10</text>"));
    let outside = spec.replace(r#""x": 90, "y": 4"#, r#""x": 190, "y": 4"#);
    let error = render_json(&outside, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("out_of_range", "/scatter/points/2/x")
    );
    let backwards = spec.replace(r#""xRange": [0, 100]"#, r#""xRange": [100, 0]"#);
    let error = render_json(&backwards, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("invalid_range", "/scatter/xRange")
    );
}

const SURVIVAL: &str = r#"{"schemaVersion": 1, "type": "survival", "title": "Survival", "width": 640, "height": 440,
        "survival": {"confidence": true, "timeTitle": "Months", "groups": [
            {"label": "A", "observations": [{"time": 2}, {"time": 4}, {"time": 4, "event": false}, {"time": 7}, {"time": 9, "event": false}]},
            {"label": "B", "observations": [{"time": 3}, {"time": 6}, {"time": 8}, {"time": 10}]}
        ]}}"#;

#[test]
fn survival_curves_step_down_at_events_and_mark_the_censored() {
    let output = render_ok(SURVIVAL);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(
        svg.matches("class=\"chartlet-km-mark chartlet-km-1\"")
            .count(),
        2
    );
    assert_eq!(
        svg.matches("class=\"chartlet-km-band chartlet-km-2\"")
            .count(),
        1
    );
    assert!(svg.contains("Number at risk"));
    let html = render_json(SURVIVAL, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    // A: 5 at risk, 1 event at 2 (0.8), at 4 one event with 4 at risk (0.6), a censor at 4,
    // at 7 one event with 2 at risk (0.3).
    assert!(html.contains("A: 5 observations, 3 events, median 7, 30.0% at the end."));
    assert!(html.contains("B: 4 observations, 4 events, median 6, 0.0% at the end."));
}

#[test]
fn survival_curves_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#"{"time": 2}"#,
            r#"{"time": -2}"#,
            "invalid_value",
            "/survival/groups/0/observations/0/time",
        ),
        (
            r#""label": "B""#,
            r#""label": "A""#,
            "duplicate_label",
            "/survival/groups/1/label",
        ),
        (
            r#"[{"time": 3}, {"time": 6}, {"time": 8}, {"time": 10}]"#,
            "[]",
            "invalid_values",
            "/survival/groups/1/observations",
        ),
    ] {
        let error = render_json(
            &SURVIVAL.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}

#[test]
fn a_scatter_plot_fits_a_line_and_takes_log_axes() {
    let line = r#"{"schemaVersion": 1, "type": "scatter", "title": "Fit", "width": 640, "height": 420,
            "scatter": {"regression": true, "points": [{"x": 1, "y": 3}, {"x": 2, "y": 5}, {"x": 3, "y": 7}, {"x": 4, "y": 9}]}}"#;
    let output = render_ok(line);
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-sc-fit chartlet-sc-1\"")
            .count(),
        1
    );
    let html = render_json(line, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(
        html.contains("Line through 4 points: y = 1.000 + 2.000 x, r² = 1.000."),
        "{html}"
    );
    let log = line.replace(
        r#""regression": true"#,
        r#""regression": true, "xScale": "log", "yScale": "log""#,
    );
    let html = render_json(&log, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("fitted to the logarithms"));
    let bad = log.replace(r#""x": 1,"#, r#""x": 0,"#);
    let error = render_json(&bad, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("invalid_value", "/scatter/points/0/x")
    );
}

#[test]
fn survival_curves_test_whether_the_groups_differ() {
    let two = SURVIVAL.replace(
        r#""confidence": true"#,
        r#""confidence": true, "logRank": true"#,
    );
    let html = render_json(&two, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    // Reference values from an independent implementation of the Mantel-Cox test.
    assert!(
        html.contains("Log-rank test: χ² = 0.01, 1 degree of freedom, p = 0.908."),
        "{html}"
    );
    assert!(html.contains("Log-rank p = 0.908"));
    assert!(
        html.contains("Hazard ratio of B against A: 0.91 (95% confidence interval 0.18 to 4.62)."),
        "{html}"
    );
    let three = two.replace(
        r#"{"label": "B","#,
        r#"{"label": "C", "observations": [{"time": 1}, {"time": 5}, {"time": 5, "event": false}, {"time": 11}, {"time": 12}]},
            {"label": "B","#,
    );
    let html = render_json(&three, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("2 degrees of freedom, p = 0.575"), "{html}");
    assert!(
        !html.contains("Hazard ratio"),
        "three groups have no single ratio"
    );
    let one = r#"{"schemaVersion": 1, "type": "survival", "title": "T", "survival": {"logRank": true, "groups": [{"label": "A", "observations": [{"time": 1}, {"time": 2}]}]}}"#;
    let error = render_json(one, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("log_rank_needs_groups", "/survival/logRank")
    );
}

const BUBBLES: &str = r#"{"schemaVersion": 1, "type": "scatter", "title": "Portfolio", "width": 640, "height": 440,
    "scatter": {"sizeTitle": "Revenue", "points": [
        {"x": 1, "y": 2, "size": 400, "label": "Big"},
        {"x": 3, "y": 4, "size": 100, "label": "Quarter"},
        {"x": 5, "y": 1, "size": 4}
    ]}}"#;

#[test]
fn a_bubble_chart_sizes_the_area_of_a_point_by_its_value() {
    let output = render_ok(BUBBLES);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    let radius_of = |title: &str| -> f64 {
        let at = svg.find(title).expect("the bubble is drawn");
        let circle = &svg[svg[..at].rfind("<circle").unwrap()..at];
        let from = circle.find(" r=\"").unwrap() + 4;
        circle[from..from + circle[from..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    // A quarter of the size makes half the radius.
    let (big, quarter) = (
        radius_of("<title>Big: 1, 2, Revenue: 400"),
        radius_of("<title>Quarter: 3, 4, Revenue: 100"),
    );
    assert!((big / quarter - 2.0).abs() < 0.02, "{big} {quarter}");
    assert_eq!(
        svg.matches("class=\"chartlet-sc-dot chartlet-sc-bubble")
            .count(),
        3
    );
    // The size legend: three reference circles next to the name of the size.
    assert_eq!(svg.matches("class=\"chartlet-sc-ref\"").count(), 3);
    assert!(svg.contains(">Revenue</text>"));
    // The value stands in a bubble that has room for it.
    assert!(svg.contains("class=\"chartlet-sc-label\">400</text>"));
}

#[test]
fn the_size_is_in_the_description_and_the_table() {
    let html = render_json(BUBBLES, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("The area of a point shows Revenue, from 4 to 400; the largest is Big."));
    assert!(html.contains("<th scope=\"col\">Revenue</th>"));
    assert!(html.contains("<th scope=\"row\">Big</th><td>1</td><td>2</td><td>400</td>"));
}

#[test]
fn sizes_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            "\"size\": 100",
            "\"size\": 0",
            "invalid_value",
            "/scatter/points/1/size",
        ),
        (
            "\"size\": 4}",
            "\"label\": \"Small\"}",
            "missing_size",
            "/scatter/points/2/size",
        ),
    ] {
        assert_eq!(
            render_err(&BUBBLES.replace(from, to)),
            (code, path.to_owned()),
            "{from}"
        );
    }
    let title_only = r#"{"schemaVersion": 1, "type": "scatter", "title": "T", "scatter": {"sizeTitle": "Revenue", "points": [{"x": 1, "y": 2}]}}"#;
    assert_eq!(
        render_err(title_only),
        ("option_not_supported", "/scatter/sizeTitle".to_owned())
    );
}
