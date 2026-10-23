use super::*;

/// The specification with a `mobile` field added at the top level.
fn with_mobile(specification: &str, mobile: &str) -> String {
    specification.replacen('{', &format!("{{\"mobile\": {mobile},"), 1)
}

fn mobile_svg(specification: &str) -> Result<super::RenderOutput, super::ChartError> {
    render_json(
        specification,
        RenderFormat::Svg,
        &RenderOptions {
            variant: Variant::Mobile,
            ..RenderOptions::default()
        },
    )
}

#[test]
fn mobile_sizes_are_validated_with_their_paths() {
    for (mobile, path) in [
        (r#"{"width": 199}"#, "/mobile/width"),
        (r#"{"width": 601}"#, "/mobile/width"),
        (r#"{"width": 360, "height": 159}"#, "/mobile/height"),
        (r#"{"width": 360, "height": 1601}"#, "/mobile/height"),
        (r#"{"width": 360, "breakpoint": 319}"#, "/mobile/breakpoint"),
        (
            r#"{"width": 360, "breakpoint": 1601}"#,
            "/mobile/breakpoint",
        ),
    ] {
        let error = render_json(
            &with_mobile(GROUPED, mobile),
            RenderFormat::Html,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!(
            (error.code, error.path.as_str()),
            ("invalid_dimension", path)
        );
    }
    let missing = render_json(
        &with_mobile(GROUPED, r#"{"height": 360}"#),
        RenderFormat::Html,
        &RenderOptions::default(),
    )
    .unwrap_err();
    assert_eq!(
        (missing.code, missing.path.as_str()),
        ("invalid_spec", "/mobile")
    );
}

#[test]
fn a_missing_mobile_field_keeps_the_default_id() {
    let plain = render_json(GROUPED, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    let spec = ChartSpec::from_json(GROUPED).unwrap();
    assert!(spec.mobile.is_none());
    assert!(!serde_json::to_string(&spec).unwrap().contains("mobile"));
    assert!(plain.content.contains("id=\"chartlet-"));
}

#[test]
fn html_carries_both_variants_behind_a_container_query() {
    let specification = with_mobile(GROUPED, r#"{"width": 360, "breakpoint": 700}"#);
    let html = render_json(
        &specification,
        RenderFormat::Html,
        &RenderOptions {
            id_prefix: Some("chart".to_owned()),
            ..RenderOptions::default()
        },
    )
    .unwrap()
    .content;
    assert!(html.starts_with(
        "<div class=\"chartlet-wrapper chartlet-responsive chartlet-bp-700\"><style>"
    ));
    assert!(html.contains("container-type:inline-size"));
    assert!(html.contains("@container (max-width:699px){.chartlet-bp-700 .chartlet-variant-desktop{display:none}.chartlet-bp-700 .chartlet-variant-mobile{display:block}}"));
    assert!(html.contains(".chartlet-variant-mobile{display:none}"));
    assert!(html.contains("<div class=\"chartlet-variant-desktop\"><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"450\""));
    assert!(html.contains("<div class=\"chartlet-variant-mobile\"><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"360\" height=\"360\""));
    assert!(html.contains("id=\"chart\""));
    assert!(html.contains("id=\"chart-m\""));
    assert!(html.contains("aria-labelledby=\"chart-m-title chart-m-description\""));
    // Every ID is unique across both variants.
    let ids = html
        .split(" id=\"")
        .skip(1)
        .map(|rest| rest.split('"').next().unwrap())
        .collect::<Vec<_>>();
    let unique = ids.iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), unique.len(), "{ids:?}");
    // Caption, source, filter and data table appear once.
    for once in [
        "<figcaption>",
        "<table>",
        "<fieldset class=\"chartlet-filter\">",
    ] {
        assert_eq!(html.matches(once).count(), 1, "{once}");
    }
    // The series filter reaches the bars of both variants.
    assert!(html.contains(".chartlet-wrapper:has(.chartlet-filter input.series-1:not(:checked)) .chartlet-root [data-series=\"1\"]{display:none}"));
    assert_eq!(html.matches("<svg").count(), 2);
}

#[test]
fn the_svg_profile_is_unchanged_by_a_mobile_variant() {
    let options = RenderOptions {
        id_prefix: Some("chart".to_owned()),
        ..RenderOptions::default()
    };
    let plain = render_json(GROUPED, RenderFormat::Svg, &options).unwrap();
    let responsive = render_json(
        &with_mobile(GROUPED, r#"{"width": 360}"#),
        RenderFormat::Svg,
        &options,
    )
    .unwrap();
    assert_eq!(plain.content, responsive.content);
}

#[test]
fn the_mobile_variant_renders_alone_as_svg() {
    let output = mobile_svg(&with_mobile(GROUPED, r#"{"width": 320, "height": 400}"#))
        .unwrap()
        .content;
    assert!(output.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"320\" height=\"400\" viewBox=\"0 0 320 400\" id=\"chartlet-"));
    assert!(output.contains("-m-title\""));

    let error = mobile_svg(GROUPED).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("missing_mobile", "/mobile")
    );

    let error = render_json(
        &with_mobile(GROUPED, r#"{"width": 360}"#),
        RenderFormat::Html,
        &RenderOptions {
            variant: Variant::Mobile,
            ..RenderOptions::default()
        },
    )
    .unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("option_not_supported", "/render/variant")
    );
}

#[test]
fn the_print_variant_draws_the_chart_with_literal_colors() {
    let print = |specification: &str, format| {
        render_json(
            specification,
            format,
            &RenderOptions {
                id_prefix: Some("costs".to_owned()),
                variant: Variant::Print,
                ..RenderOptions::default()
            },
        )
    };
    let desktop = render_json(
        GROUPED,
        RenderFormat::Svg,
        &RenderOptions {
            id_prefix: Some("costs".to_owned()),
            ..RenderOptions::default()
        },
    )
    .unwrap()
    .content;
    let light = print(GROUPED, RenderFormat::Svg).unwrap().content;
    let dark = print(
        &GROUPED.replacen('{', r#"{"theme": "dark","#, 1),
        RenderFormat::Svg,
    )
    .unwrap()
    .content;

    for svg in [&light, &dark] {
        assert!(svg.contains(" id=\"costs-p\""));
        assert!(!svg.contains("var("), "{svg}");
        assert!(!svg.contains(":has("), "{svg}");
        assert!(!svg.contains("currentColor"), "{svg}");
        assert!(!svg.contains("--chartlet"), "{svg}");
        assert!(svg.contains("font-family:Inter,ui-sans-serif,system-ui,sans-serif;"));
    }
    // The same scene as the chart on screen, only the stylesheet differs.
    let body = |svg: &str| svg[svg.find("</style>").unwrap()..].replace("costs-p", "costs");
    assert_eq!(body(&light), body(&desktop));
    assert!(light.contains("<style>#costs-p.chartlet-root{max-width:100%;height:auto;font-family:Inter,ui-sans-serif,system-ui,sans-serif;color:#172033}"));
    assert!(light.contains("#costs-p .chartlet-series-2{fill:#c2410c}"));
    assert!(dark.contains("#costs-p .chartlet-series-2{fill:#ff9c72}"));
    assert!(dark.contains("#costs-p .chartlet-background{fill:#0e131c}"));

    let error = print(GROUPED, RenderFormat::Html).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("option_not_supported", "/render/variant")
    );
}

#[test]
fn the_print_variant_resolves_a_declared_variable_color_to_the_text_color() {
    let svg = render_json(
        &TIME.replacen(
            r#""name": "Orders""#,
            r#""name": "Orders", "color": "var(--brand)""#,
            1,
        ),
        RenderFormat::Svg,
        &RenderOptions {
            variant: Variant::Print,
            ..RenderOptions::default()
        },
    )
    .unwrap();
    assert!(
        svg.content.contains(".chartlet-style-0{stroke:#172033}"),
        "{}",
        svg.content
    );
    assert!(!svg.content.contains("var("), "{}", svg.content);
    assert_eq!(
        svg.warnings
            .iter()
            .map(|warning| (warning.code, warning.path.as_str()))
            .collect::<Vec<_>>(),
        [("color_not_resolved", "/panes/0/layers/0/color")]
    );
}

#[test]
fn zoom_panels_carry_both_variants_under_one_radio_group() {
    let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "title": "Quarterly revenue",
            "mobile": {"width": 360},
            "data": [
                {"label": "Q1", "value": 320},
                {"label": "Q2", "value": 345},
                {"label": "Q3", "value": 380},
                {"label": "Q4", "value": 410}
            ],
            "zoomSteps": [
                {"label": "First half", "from": 0, "to": 1},
                {"label": "All", "from": 0, "to": 3}
            ]
        }"#;
    let html = render_json(
        specification,
        RenderFormat::Html,
        &RenderOptions {
            id_prefix: Some("q".to_owned()),
            ..RenderOptions::default()
        },
    )
    .unwrap()
    .content;
    assert_eq!(
        html.matches("<fieldset class=\"chartlet-zoom\">").count(),
        1
    );
    assert_eq!(html.matches("type=\"radio\"").count(), 2);
    for id in ["q-z0", "q-z0-m", "q-z1", "q-z1-m"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
    }
    assert!(html.contains("<div class=\"chartlet-panel chartlet-panel-1\"><div class=\"chartlet-variant-desktop\"><svg"));
    assert_eq!(html.matches("<table>").count(), 1);
}

#[test]
fn warnings_of_the_mobile_variant_are_marked_and_deduplicated() {
    // A label that is shortened at both sizes is reported once; one that only the mobile
    // variant shortens is reported as coming from it.
    let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "title": "Revenue",
            "mobile": {"width": 280},
            "categories": ["An exceptionally long category label that never fits", "B"],
            "series": [
                {"name": "Direct", "values": [1, 2]},
                {"name": "A partner network with a long name", "values": [2, 1]}
            ]
        }"#;
    let desktop = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
        .unwrap()
        .warnings;
    let html = render_json(specification, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .warnings;
    assert!(html.starts_with(&desktop));
    let added = &html[desktop.len()..];
    assert_ne!(added, []);
    for warning in added {
        assert!(
            warning.message.starts_with("mobile variant: "),
            "{warning:?}"
        );
        assert!(
            !desktop
                .iter()
                .any(|seen| (seen.code, &seen.path) == (warning.code, &warning.path)),
            "{warning:?}"
        );
    }
}

#[test]
fn the_mobile_variant_is_checked_for_density_at_its_own_width() {
    let points = (0..400)
        .map(|index| {
            format!(
                "{{\"time\": {}, \"value\": {index}}}",
                1_772_323_200 + index * 3600
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let specification = format!(
        r#"{{"schemaVersion": 1, "type": "time", "title": "Load", "mobile": {{"width": 280}},
            "panes": [{{"layers": [{{"mark": "line", "points": [{points}]}}]}}]}}"#
    );
    let desktop = render_json(&specification, RenderFormat::Svg, &RenderOptions::default())
        .unwrap()
        .warnings;
    assert!(!desktop.iter().any(|warning| warning.code == "dense_chart"));
    let html = render_json(
        &specification,
        RenderFormat::Html,
        &RenderOptions::default(),
    )
    .unwrap()
    .warnings;
    assert!(
        html.iter().any(|warning| warning.code == "dense_chart"
            && warning.message.starts_with("mobile variant: "))
    );
}

#[test]
fn the_topic_picker_switches_both_variants() {
    let specification = with_mobile(
        include_str!("../../examples/topicmap-sample.json"),
        r#"{"width": 360}"#,
    );
    let html = render_json(
        &specification,
        RenderFormat::Html,
        &RenderOptions::default(),
    )
    .unwrap()
    .content;
    assert_eq!(
        html.matches("<fieldset class=\"chartlet-topic-picker\">")
            .count(),
        1
    );
    assert_eq!(html.matches("<svg").count(), 2);
    // The picker's rules name both variants' root IDs.
    let rule = ":checked) #{root} .chartlet-topic-area.chartlet-topic-0{";
    let desktop = html.split(" id=\"").nth(1).unwrap();
    let desktop = &desktop[..desktop.find('"').unwrap()];
    assert!(html.contains(&rule.replace("{root}", desktop)));
    assert!(html.contains(&rule.replace("{root}", &format!("{desktop}-m"))));
    assert!(html.starts_with("<div class=\"chartlet-wrapper chartlet-responsive"));
}

// --- Collapsed gaps ---
