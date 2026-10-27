use super::*;

const BOXES: &str = r#"{"schemaVersion": 1, "type": "boxplot", "title": "Latency", "width": 700, "height": 360,
        "boxes": [
            {"label": "Search", "values": [10, 11, 12, 13, 14, 15, 16, 90]},
            {"label": "Export", "min": 20, "q1": 30, "median": 40, "q3": 50, "max": 70, "outliers": [95]}
        ]}"#;

#[test]
fn boxes_are_computed_from_observations_or_drawn_from_five_numbers() {
    for orientation in ["vertical", "horizontal"] {
        let json = BOXES.replace(
            "\"boxes\"",
            &format!("\"orientation\": \"{orientation}\", \"boxes\""),
        );
        let output = render_ok(&json);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        let svg = &output.content;
        assert_eq!(
            svg.matches("class=\"chartlet-box\"").count(),
            2,
            "{orientation}"
        );
        assert_eq!(svg.matches("class=\"chartlet-box-median\"").count(), 2);
        // One outlier computed (90), one given (95); each box has four whisker strokes.
        assert_eq!(svg.matches("class=\"chartlet-box-outlier\"").count(), 2);
        assert_eq!(svg.matches("class=\"chartlet-box-whisker\"").count(), 8);
        assert!(svg.contains("<title>Search: Median: 13.5 (Q1: 11.75, Q3: 15.25, Minimum: 10, Maximum: 16), Outliers: 90, Observations 8</title>"));
        assert!(svg.contains("<title>Export: Median: 40 (Q1: 30, Q3: 50, Minimum: 20, Maximum: 70), Outliers: 95</title>"));
    }
    let html = render_json(BOXES, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("<th scope=\"col\">Category</th><th scope=\"col\">Minimum</th><th scope=\"col\">Q1</th><th scope=\"col\">Median</th><th scope=\"col\">Q3</th><th scope=\"col\">Maximum</th><th scope=\"col\">Outliers</th>"));
    assert!(html.contains("<th scope=\"row\">Search</th><td>10</td><td>11.75</td><td>13.5</td><td>15.25</td><td>16</td><td>90</td>"));
    assert!(html.contains(
        "Box plot with 2 boxes, each from the first to the third quartile with its median. Highest median: 40 (Export). Lowest median: 13.5 (Search). Outliers beyond the whiskers: Search, Export."
    ));
}

#[test]
fn boxes_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""values": [10, 11, 12, 13, 14, 15, 16, 90]"#,
            r#""values": [10, 11, 12]"#,
            "invalid_values",
            "/boxes/0/values",
        ),
        (
            r#""values": [10, 11, 12, 13, 14, 15, 16, 90]"#,
            r#""min": 1"#,
            "incomplete_box",
            "/boxes/0",
        ),
        (
            r#""values": [10, 11, 12, 13, 14, 15, 16, 90]"#,
            r#""values": [1, 2, 3, 4, 5], "min": 1"#,
            "conflicting_data_shape",
            "/boxes/0/values",
        ),
        (
            r#""median": 40"#,
            r#""median": 25"#,
            "invalid_box",
            "/boxes/1/median",
        ),
        (
            r#""label": "Export""#,
            r#""label": "Search""#,
            "duplicate_label",
            "/boxes/1/label",
        ),
    ] {
        let error = render_json(
            &BOXES.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}

#[test]
fn error_bars_run_through_the_bar_end_and_are_written_in_text() {
    let json = r#"{"schemaVersion": 1, "type": "bar", "title": "Scores", "width": 600, "height": 300,
            "data": [{"label": "A", "value": 4, "lower": 3.5, "upper": 4.5}, {"label": "B", "value": 2, "lower": 1, "upper": 3}]}"#;
    for orientation in ["vertical", "horizontal"] {
        let spec = json.replace(
            "\"type\": \"bar\"",
            &format!("\"type\": \"bar\", \"orientation\": \"{orientation}\""),
        );
        let output = render_ok(&spec);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        // A stroke and two caps for each of the two bars.
        assert_eq!(
            output.content.matches("class=\"chartlet-error\"").count(),
            6,
            "{orientation}"
        );
        assert!(output.content.contains("<title>A: 4 (3.5 to 4.5)</title>"));
    }
    let html = render_json(json, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("<th scope=\"col\">Interval</th>"));
    assert!(html.contains("<th scope=\"row\">B</th><td>2</td><td>1 to 3</td>"));
    assert!(html.contains("Error bars show the interval from lower to upper"));
}

#[test]
fn error_bars_and_groups_exclude_each_other() {
    let json = r#"{"schemaVersion": 1, "type": "bar", "title": "S", "width": 600, "height": 300,
            "data": [{"label": "A", "value": 4, "group": "x", "lower": 3, "upper": 5}, {"label": "B", "value": 2, "group": "y", "lower": 1, "upper": 3}]}"#;
    let error = render_json(json, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("option_not_supported", "/data/0/group")
    );
}

#[test]
fn error_bars_are_validated_by_name() {
    let base = r#"{"schemaVersion": 1, "type": "bar", "title": "S", "width": 600, "height": 300,
            "data": [{"label": "A", "value": 4, "lower": 3.5, "upper": 4.5}, {"label": "B", "value": 2, "lower": 1, "upper": 3}]}"#;
    for (from, to, code, path) in [
        (
            r#""lower": 1, "upper": 3"#,
            r#""lower": 1"#,
            "missing_bounds",
            "/data/1",
        ),
        (
            r#""lower": 1, "upper": 3"#,
            r#""lower": 2.5, "upper": 3"#,
            "invalid_bounds",
            "/data/1/lower",
        ),
        // A line chart leaves a gap for a missing value; bounds around the gap are refused.
        (
            r#""type": "bar", "title": "S", "width": 600, "height": 300,
            "data": [{"label": "A", "value": 4, "lower": 3.5, "upper": 4.5}, {"label": "B", "value": 2,"#,
            r#""type": "line", "title": "S", "width": 600, "height": 300,
            "data": [{"label": "A", "value": 4, "lower": 3.5, "upper": 4.5}, {"label": "B","#,
            "invalid_bounds",
            "/data/1/value",
        ),
    ] {
        let error = render_json(
            &base.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}

const FOREST: &str = r#"{"schemaVersion": 1, "type": "rangebar", "orientation": "horizontal", "title": "Effects", "width": 700, "height": 320,
        "references": [{"label": "No effect", "value": 0}],
        "ranges": [
            {"label": "Study A", "low": -4, "high": 1, "mid": -1.5, "weight": 30},
            {"label": "Study B", "low": -6, "high": -1, "mid": -3.5, "weight": 10},
            {"label": "Overall", "low": -3.5, "high": -1, "mid": -2.2, "summary": true}
        ]}"#;

#[test]
fn a_forest_plot_draws_squares_by_weight_a_diamond_and_the_line_of_no_effect() {
    let output = render_ok(FOREST);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-range-weight\"").count(), 2);
    assert_eq!(svg.matches("class=\"chartlet-range-summary\"").count(), 1);
    assert_eq!(svg.matches("class=\"chartlet-rule\"").count(), 1);
    assert!(svg.contains(">No effect</text>"));
    assert!(svg.contains("<title>Study A: −1.5 (−4 to 1), weight 30</title>"));
    // The square of the lighter study is smaller: sqrt(10 / 30) of 14 pixels.
    let sides: Vec<f64> = svg
        .match_indices("class=\"chartlet-range-weight\"")
        .map(|(at, _)| {
            let rect = &svg[svg[..at].rfind("<rect").unwrap()..at];
            let value = rect
                .split("width=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap();
            value.parse().unwrap()
        })
        .collect();
    assert!(
        (sides[0] - 14.0).abs() < 0.01 && (sides[1] - 14.0 * (1.0_f64 / 3.0).sqrt()).abs() < 0.01,
        "{sides:?}"
    );
    let html = render_json(FOREST, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("<th scope=\"col\">Weight</th>"));
    assert!(html.contains("<th scope=\"row\">Overall (summary)</th>"));
    assert!(html.contains("Overall result, drawn as a diamond: Overall (−2.2 (−3.5 to −1))."));
    assert!(html.contains("Reference lines: No effect at 0."));
}

#[test]
fn weights_and_summaries_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""mid": -1.5, "weight": 30"#,
            r#""weight": 30"#,
            "invalid_weight",
            "/ranges/0/weight",
        ),
        (
            r#""weight": 10"#,
            r#""weight": 0"#,
            "invalid_weight",
            "/ranges/1/weight",
        ),
        (
            r#""mid": -2.2, "summary": true"#,
            r#""summary": true"#,
            "summary_without_mid",
            "/ranges/2/summary",
        ),
    ] {
        let error = render_json(
            &FOREST.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}

#[test]
fn violins_can_share_an_area_scale() {
    let spec = |scale: &str| {
        format!(
            r#"{{"schemaVersion": 1, "type": "boxplot", "boxDisplay": "violin", "violinScale": "{scale}", "title": "T",
                "boxes": [{{"label": "Tight", "values": [5, 5.1, 5.2, 5.3, 5.4, 5.5]}}, {{"label": "Wide", "values": [1, 3, 5, 7, 9, 11]}}]}}"#
        )
    };
    let width = render_ok(&spec("width")).content;
    let area = render_ok(&spec("area")).content;
    let count = render_ok(&spec("count")).content;
    assert_ne!(width, area);
    // The same number of observations: count and area agree on the shapes.
    let polylines = |svg: &str| {
        svg.split("<polyline")
            .skip(1)
            .map(|p| {
                p.split("points=\"")
                    .nth(1)
                    .unwrap()
                    .split('"')
                    .next()
                    .unwrap()
                    .to_owned()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(polylines(&area), polylines(&count));
    let plain = spec("area").replace(r#""boxDisplay": "violin""#, r#""boxDisplay": "strip""#);
    let error = render_json(&plain, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("option_not_supported", "/violinScale")
    );
}

#[test]
fn box_plots_color_their_boxes_by_group() {
    let spec = r#"{"schemaVersion": 1, "type": "boxplot", "title": "T",
            "boxes": [
                {"label": "A EU", "group": "EU", "values": [1, 2, 3, 4, 5]},
                {"label": "A Asia", "group": "Asia", "values": [2, 3, 4, 5, 9]},
                {"label": "B EU", "group": "EU", "values": [3, 4, 5, 6, 7]}
            ]}"#;
    let output = render_ok(spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    // Two EU boxes and the legend swatch; one Asia box and its swatch.
    assert_eq!(
        svg.matches("class=\"chartlet-box chartlet-box-g1\"")
            .count(),
        3
    );
    assert_eq!(
        svg.matches("class=\"chartlet-box chartlet-box-g2\"")
            .count(),
        2
    );
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("<th scope=\"col\">Group</th>"));
    let missing = spec.replace(r#""group": "Asia", "#, "");
    let error = render_json(&missing, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("missing_group", "/boxes/1/group")
    );
}

#[test]
fn box_plots_can_mark_the_mean() {
    let spec = r#"{"schemaVersion": 1, "type": "boxplot", "showMean": true, "title": "T",
            "boxes": [{"label": "A", "values": [1, 2, 3, 4, 10]}]}"#;
    let output = render_ok(spec);
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-box-mean\"")
            .count(),
        1
    );
    assert!(output.content.contains("Mean: 4"));
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("<th scope=\"col\">Mean</th>"));
    let five = spec.replace(
        r#""values": [1, 2, 3, 4, 10]"#,
        r#""min": 1, "q1": 2, "median": 3, "q3": 4, "max": 5"#,
    );
    let error = render_json(&five, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("values_required", "/boxes/0/values")
    );
}

#[test]
fn violins_and_strips_draw_the_observations_and_need_them() {
    let boxes = r#""boxes": [{"label": "A", "values": [1, 2, 3, 4, 5, 6, 7, 8]}, {"label": "B", "values": [2, 3, 3, 4, 4, 4, 5, 9]}]"#;
    let spec = |display: &str| {
        format!(
            r#"{{"schemaVersion": 1, "type": "boxplot", "boxDisplay": "{display}", "title": "T", {boxes}}}"#
        )
    };
    let violin = render_ok(&spec("violin"));
    assert!(violin.warnings.is_empty(), "{:?}", violin.warnings);
    assert_eq!(
        violin.content.matches("class=\"chartlet-violin\"").count(),
        2
    );
    assert_eq!(
        violin
            .content
            .matches("class=\"chartlet-violin-median\"")
            .count(),
        2
    );
    let strip = render_ok(&spec("strip"));
    assert_eq!(
        strip
            .content
            .matches("class=\"chartlet-box-point\"")
            .count(),
        16
    );
    let again = render_ok(&spec("strip"));
    assert_eq!(strip.content, again.content);
    let five = spec("violin").replace(
        r#"{"label": "B", "values": [2, 3, 3, 4, 4, 4, 5, 9]}"#,
        r#"{"label": "B", "min": 1, "q1": 2, "median": 3, "q3": 4, "max": 5}"#,
    );
    let error = render_json(&five, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("values_required", "/boxes/1/values")
    );
    let bar = r#"{"schemaVersion": 1, "type": "bar", "boxDisplay": "violin", "title": "T", "data": [{"label": "a", "value": 1}]}"#;
    let error = render_json(bar, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("option_not_supported", "/boxDisplay")
    );
}

#[test]
fn series_of_a_chart_can_carry_error_bars() {
    let spec = r#"{"schemaVersion": 1, "type": "bar", "title": "T", "width": 640, "height": 360,
            "categories": ["a", "b"],
            "series": [
                {"name": "One", "values": [2, 3], "lower": [1, 2], "upper": [3, 4]},
                {"name": "Two", "values": [4, null], "lower": [3, null], "upper": [5, null]}
            ]}"#;
    let output = render_ok(spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    // Three values with bounds: a stroke and two caps each.
    assert_eq!(
        output.content.matches("class=\"chartlet-error\"").count(),
        9
    );
    assert!(output.content.contains("a – Two: 4 (3 to 5)"));
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("<th scope=\"col\">One – Interval</th>"));
    assert!(html.contains("<th scope=\"col\">Two – Interval</th>"));
    for (from, to, code, path) in [
        (
            r#""upper": [3, 4]"#,
            r#""upper": [3]"#,
            "series_length_mismatch",
            "/series/0/upper",
        ),
        (
            r#""lower": [1, 2]"#,
            r#""lower": [1, 3.5]"#,
            "invalid_bounds",
            "/series/0/lower/1",
        ),
        (
            r#""lower": [3, null]"#,
            r#""lower": [null, null]"#,
            "missing_bounds",
            "/series/1/lower/0",
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

#[test]
fn a_line_chart_draws_error_bars_through_its_points() {
    let spec = r#"{"schemaVersion": 1, "type": "line", "title": "T", "width": 640, "height": 360,
            "data": [{"label": "a", "value": 2, "lower": 1, "upper": 3}, {"label": "b", "value": 3, "lower": 2, "upper": 5}]}"#;
    let output = render_ok(spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert_eq!(
        output.content.matches("class=\"chartlet-error\"").count(),
        6
    );
    assert!(output.content.contains("<title>b: 3 (2 to 5)</title>"));
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("the table gives it for every point."));
    let area = spec.replace(r#""type": "line""#, r#""type": "rangebar""#);
    assert!(render_json(&area, RenderFormat::Svg, &RenderOptions::default()).is_err());
}

#[test]
fn beeswarms_move_points_sideways_until_none_overlap() {
    for orientation in ["vertical", "horizontal"] {
        let spec = |display: &str, extra: &str| {
            format!(
                r#"{{"schemaVersion": 1, "type": "boxplot", "boxDisplay": "{display}", {extra} "orientation": "{orientation}", "title": "T", "width": 700, "height": 360,
                "boxes": [
                    {{"label": "Search", "values": [10, 10, 10, 10, 10, 11, 11, 11, 12, 12, 13, 14, 15, 16, 30]}},
                    {{"label": "Export", "values": [20, 30, 40, 50, 60, 70]}}
                ]}}"#
            )
        };
        let output = render_ok(&spec("beeswarm", ""));
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        let svg = &output.content;
        assert_eq!(
            svg.matches("class=\"chartlet-box-point\"").count(),
            15 + 6,
            "{orientation}"
        );
        assert_eq!(svg.matches("class=\"chartlet-box-median\"").count(), 2);
        assert!(!svg.contains("class=\"chartlet-box\""));
        assert!(svg.contains("<title>Search: 30</title>"));
        // Same data, same picture.
        assert_eq!(svg, &render_ok(&spec("beeswarm", "")).content);
        // The box lies under the points on request.
        let boxed = render_ok(&spec("beeswarm", "\"showBox\": true,")).content;
        assert_eq!(boxed.matches("class=\"chartlet-box\"").count(), 2);
        assert!(boxed.find("class=\"chartlet-box\"") < boxed.find("class=\"chartlet-box-point\""));
        assert!(svg.contains(
            "Every observation is drawn as a dot, moved sideways so that no two dots overlap."
        ));
    }
}

#[test]
fn a_swarm_keeps_its_points_apart() {
    let points: Vec<f64> = (0..40).map(|index| f64::from(index % 5) * 0.4).collect();
    let offsets = crate::boxplot::swarm_offsets(&points, 3.0);
    for first in 0..points.len() {
        for second in first + 1..points.len() {
            let apart = ((points[first] - points[second]).powi(2)
                + (offsets[first] - offsets[second]).powi(2))
            .sqrt();
            assert!(
                apart >= 6.0 - 1e-6,
                "{first} and {second} are {apart} apart"
            );
        }
    }
}

#[test]
fn ridgelines_stack_one_density_curve_per_category() {
    for orientation in ["vertical", "horizontal"] {
        let spec = format!(
            r#"{{"schemaVersion": 1, "type": "boxplot", "boxDisplay": "ridgeline", "showMean": true, "orientation": "{orientation}", "title": "T", "width": 700, "height": 360,
            "boxes": [
                {{"label": "A", "values": [1, 2, 2, 3, 3, 3, 4, 4, 5, 9]}},
                {{"label": "B", "values": [4, 5, 5, 6, 6, 6, 7, 7, 8, 12]}},
                {{"label": "C", "values": [2, 3, 3, 4, 4, 4, 5, 5, 6, 7]}}
            ]}}"#
        );
        let output = render_ok(&spec);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        let svg = &output.content;
        assert_eq!(
            svg.matches("class=\"chartlet-violin\"").count(),
            3,
            "{orientation}"
        );
        assert_eq!(svg.matches("class=\"chartlet-box-median\"").count(), 3);
        assert_eq!(svg.matches("class=\"chartlet-box-mean\"").count(), 3);
        assert!(svg.contains(
            "Each category is a density curve of its observations on a shared value axis, the curves stacked with a slight overlap."
        ));
    }
}

#[test]
fn beeswarms_and_ridgelines_are_validated_by_name() {
    let spec = |display: &str, extra: &str, box_json: &str| {
        format!(
            r#"{{"schemaVersion": 1, "type": "boxplot", "boxDisplay": "{display}", {extra} "title": "T", "boxes": [{box_json}]}}"#
        )
    };
    let five = r#"{"label": "A", "min": 1, "q1": 2, "median": 3, "q3": 4, "max": 5}"#;
    for display in ["beeswarm", "ridgeline"] {
        assert_eq!(
            render_err(&spec(display, "", five)),
            ("values_required", "/boxes/0/values".to_owned()),
            "{display}"
        );
    }
    let observed = r#"{"label": "A", "values": [1, 2, 3, 4, 5]}"#;
    for display in ["box", "violin", "strip", "ridgeline"] {
        assert_eq!(
            render_err(&spec(display, "\"showBox\": true,", observed)),
            ("option_not_supported", "/showBox".to_owned()),
            "{display}"
        );
    }
    assert_eq!(
        render_err(&spec("ridgeline", "\"violinScale\": \"area\",", observed)),
        ("option_not_supported", "/violinScale".to_owned())
    );
    let bar = r#"{"schemaVersion": 1, "type": "bar", "showBox": true, "title": "T", "data": [{"label": "a", "value": 1}]}"#;
    assert_eq!(
        render_err(bar),
        ("option_not_supported", "/showBox".to_owned())
    );
}
