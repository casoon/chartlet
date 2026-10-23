use super::*;

#[test]
fn renders_accessible_deterministic_svg() {
    let first = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    let second = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert_eq!(first.content, second.content);
    assert!(first.content.contains("role=\"img\""));
    assert!(first.content.contains("<title id="));
    assert!(first.content.contains("<desc id="));
    assert!(first.content.contains("North &lt;East&gt;"));
    assert!(first.content.contains("height=\""));
}

#[test]
fn html_contains_complete_data_table() {
    let output = render_json(SPEC, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(output.content.contains("<figure"));
    assert!(output.content.contains("<details"));
    assert!(output.content.contains("<table>"));
    assert!(output.content.contains("<td>−4</td>"));
}

#[test]
fn html_captions_the_title_instead_of_drawing_it() {
    let svg = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    let html = render_json(SPEC, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(
        svg.content
            .contains("class=\"chartlet-title\">Profit &amp; loss")
    );
    assert!(!html.content.contains("class=\"chartlet-title\""));
    assert!(
        html.content
            .contains("<figcaption>Profit &amp; loss</figcaption>")
    );
    assert!(html.content.contains("-title\">Profit &amp; loss</title>"));
    let ids = |content: &str| {
        content
            .split("id=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .filter(|id| id.starts_with("chartlet-"))
            .map(str::to_owned)
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert!(ids(&svg.content).is_subset(&ids(&html.content)));
}

#[test]
fn show_title_false_leaves_the_title_out_of_any_svg() {
    let spec = SPEC.replacen(
        "\"type\": \"bar\",",
        "\"type\": \"bar\", \"showTitle\": false,",
        1,
    );
    let output = render_json(&spec, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(!output.content.contains("class=\"chartlet-title\""));
    assert!(
        output
            .content
            .contains("-title\">Profit &amp; loss</title>")
    );
}

#[test]
fn html_does_not_shorten_the_title() {
    let title = "A very long title ".repeat(10);
    let spec = SPEC.replacen("Profit & loss", title.trim(), 1);
    let truncated = |format| {
        render_json(&spec, format, &RenderOptions::default())
            .unwrap()
            .warnings
            .iter()
            .any(|warning| warning.code == "text_truncated" && warning.path == "/title")
    };
    assert!(truncated(RenderFormat::Svg));
    assert!(!truncated(RenderFormat::Html));
}

#[test]
fn deserialization_errors_use_json_pointers() {
    for (input, code, path) in [
        (
            SPEC.replace("\"title\"", "\"colour\": 1, \"title\""),
            "invalid_spec",
            "/colour",
        ),
        (
            SPEC.replace("12}", "\"12\"}"),
            "invalid_spec",
            "/data/0/value",
        ),
        (
            SPEC.replace("\"title\": \"Profit & loss\",", ""),
            "invalid_spec",
            "/",
        ),
        ("{\"schemaVersion\": 1,".to_owned(), "invalid_json", "/"),
    ] {
        let error = render_json(&input, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{input}");
    }
}

#[test]
fn rejects_duplicate_labels_with_path() {
    let duplicate = SPEC.replace("South", "North <East>");
    let error = render_json(&duplicate, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "duplicate_label");
    assert_eq!(error.path, "/data/1/label");
}

#[test]
fn explicit_prefix_prevents_duplicate_ids() {
    let first = render_json(
        SPEC,
        RenderFormat::Svg,
        &RenderOptions {
            id_prefix: Some("first".to_owned()),
            ..RenderOptions::default()
        },
    )
    .unwrap();
    let second = render_json(
        SPEC,
        RenderFormat::Svg,
        &RenderOptions {
            id_prefix: Some("second".to_owned()),
            ..RenderOptions::default()
        },
    )
    .unwrap();
    assert!(first.content.contains("first-title"));
    assert!(second.content.contains("second-title"));
}

#[test]
fn preserves_small_values_in_every_text_alternative() {
    let small = SPEC.replace("12}", "0.001}");
    let output = render_json(&small, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(output.content.contains("<td>0.001</td>"));
    assert!(output.content.contains("0.001"));
}

#[test]
fn rejects_values_outside_the_supported_scale_range() {
    let extreme = SPEC.replace("12}", "1e308}");
    let error = render_json(&extreme, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "unsupported_numeric_range");
    assert_eq!(error.path, "/data/0/value");
}

#[test]
fn rejects_xml_control_characters() {
    let invalid = SPEC.replace("Profit & loss", "Bad\\u0000title");
    let error = render_json(&invalid, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "invalid_xml_character");
    assert_eq!(error.path, "/title");
}

#[test]
fn equal_values_do_not_invent_extrema() {
    let tied = SPEC.replace("-4", "12");
    let output = render_json(&tied, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(
        output
            .content
            .contains("Bar chart with 2 equal values: 12 each.")
    );
}

#[test]
fn visible_table_mode_omits_details_disclosure() {
    let output = render_json(
        SPEC,
        RenderFormat::Html,
        &RenderOptions {
            table_mode: super::TableMode::Visible,
            ..RenderOptions::default()
        },
    )
    .unwrap();
    assert!(
        output
            .content
            .contains("<div class=\"chartlet-data\" data-viz-text>")
    );
    assert!(!output.content.contains("<details"));
}

#[test]
fn line_chart_preserves_missing_values_as_gaps() {
    let specification = include_str!("../../examples/monthly-trend.json");
    let output = render_json(specification, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert_eq!(output.content.matches("<polyline").count(), 2);
    assert_eq!(output.content.matches("<circle").count(), 5);
    assert!(output.content.contains("<td>Missing</td>"));
}

#[test]
fn a_value_axis_range_must_be_ordered_and_in_its_place() {
    let reversed = TIME.replace(
        "\"valueAxis\": {\"title\": \"Orders\"}",
        "\"valueAxis\": {\"title\": \"Orders\", \"min\": 10, \"max\": 5}",
    );
    assert_eq!(
        render_err(&reversed),
        ("invalid_axis_range", "/panes/0/valueAxis/max".to_owned())
    );
    let top_level = TIME.replacen('{', "{\"valueAxis\": {\"min\": 0},", 1);
    assert_eq!(
        render_err(&top_level),
        ("option_not_supported", "/valueAxis".to_owned())
    );
}

#[test]
fn too_many_decimals_are_refused() {
    let spec = TIME.replace(
        "\"valueAxis\": {\"title\": \"Orders\"}",
        "\"valueAxis\": {\"title\": \"Orders\", \"decimals\": 9}",
    );
    assert_eq!(
        render_err(&spec),
        ("invalid_decimals", "/panes/0/valueAxis/decimals".to_owned())
    );
}
