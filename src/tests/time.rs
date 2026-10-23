use super::*;

/// A time chart with `points` hourly observations in one layer, for the density rules.
fn hourly(points: usize, extra: &str) -> String {
    let sample = (0..points)
        .map(|index| {
            let offset = i64::try_from(index).expect("test sizes fit in an i64") * 3_600;
            format!(
                "{{\"time\": {}, \"value\": {}}}",
                1_770_000_000 + offset,
                index % 5 + 1
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Hourly load",
                "timeAxis": {{"timezone": "UTC"}},
                "panes": [
                    {{
                        "valueAxis": {{"title": "Load"}},
                        "layers": [{{"mark": "line", "name": "Load"{extra}, "points": [{sample}]}}]
                    }}
                ]
            }}"#
    )
}

#[test]
fn time_chart_renders_a_line_marks_and_a_time_axis() {
    let svg = render_json(TIME, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(svg.content.contains("<polyline"));
    assert_eq!(
        svg.content
            .matches("class=\"chartlet-point chartlet-point-series-1\"")
            .count(),
        3
    );
    assert!(
        svg.content
            .contains("<title>2026-03-01 – Orders: 10</title>")
    );
    assert!(svg.content.contains(">2026-03-01<"));
    assert!(svg.content.contains(">Day<"));
    assert!(svg.content.contains(">Orders<"));
    assert!(svg.warnings.is_empty(), "{:?}", svg.warnings);

    let html = render_json(TIME, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(
        html.content
            .contains("<th scope=\"col\">Time</th><th scope=\"col\">Orders</th>")
    );
    assert!(
        html.content
            .contains("<th scope=\"row\">2026-03-03</th><td>20</td>")
    );
}

#[test]
fn time_chart_without_a_description_gets_a_generated_one() {
    let output = render_json(TIME, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(output.content.contains(
        "Time chart with 3 points from 2026-03-01 to 2026-03-03. Highest: 20 (2026-03-03). Lowest: 10 (2026-03-01)."
    ));
}

#[test]
fn the_timezone_moves_the_wall_clock_labels() {
    // A bare date is wall-clock midnight in the configured zone, so the offset does not move
    // it: +02:00 keeps the date labels.
    let bare = TIME.replace("\"timezone\": \"UTC\"", "\"timezone\": \"+02:00\"");
    let bare = render_json(&bare, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(bare.content.contains(">2026-03-01<"));
    assert!(!bare.content.contains(">2026-03-01 02:00<"));

    // Absolute seconds are read in the zone, so midnight in UTC is two in the morning
    // in +02:00 and the labels carry a time of day.
    let seconds = TIME
        .replace("\"time\": \"2026-03-01\"", "\"time\": 1772323200")
        .replace("\"time\": \"2026-03-02\"", "\"time\": 1772409600")
        .replace("\"time\": \"2026-03-03\"", "\"time\": 1772496000");
    let utc = render_json(&seconds, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(utc.content.contains(">2026-03-01<"));
    assert!(!utc.content.contains("00:00<"));

    let shifted = seconds.replace("\"timezone\": \"UTC\"", "\"timezone\": \"+02:00\"");
    let shifted = render_json(&shifted, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(shifted.content.contains(">2026-03-02 00:00<"));
    assert!(shifted.content.contains(">2026-03-01 12:00<"));

    // The data table names the observation in the zone as well.
    let zone = seconds.replace("\"timezone\": \"UTC\"", "\"timezone\": \"+02:00\"");
    let html = render_json(&zone, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(
        html.content
            .contains("<th scope=\"row\">2026-03-01 02:00</th>")
    );
}

#[test]
fn timestamps_that_do_not_increase_are_rejected() {
    let reversed = TIME.replace("\"value\": 10", "\"value\": 99").replace(
        "\"time\": \"2026-03-01\", \"value\": 99",
        "\"time\": \"2026-03-03\", \"value\": 99",
    );
    let error = render_json(&reversed, RenderFormat::Svg, &RenderOptions::default())
        .expect_err("a backwards series has no direction");
    assert_eq!(error.code, "unordered_time");
    assert_eq!(error.path, "/panes/0/layers/0/points/1/time");
}

#[test]
fn fractional_and_out_of_range_timestamps_are_rejected() {
    for (time, message) in [
        ("12.5", "Unix seconds must be a whole number"),
        (
            "8000000000",
            "timestamps must lie between 1700-01-01 and 2200-01-01",
        ),
        (
            "\"March 1st\"",
            "expected an ISO 8601 date such as 2026-03-01 or 2026-03-01T12:00:00Z, a month such as 2026-03, or a year such as 1850",
        ),
    ] {
        let broken = TIME.replace("\"time\": \"2026-03-01\"", &format!("\"time\": {time}"));
        let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("a timestamp outside the contract is refused");
        assert_eq!(
            (error.code, error.path.as_str()),
            ("invalid_time", "/panes/0/layers/0/points/0/time")
        );
        assert_eq!(error.message, message, "{time}");
    }
}

#[test]
fn a_line_needs_two_observations() {
    let error = render_json(&hourly(1, ""), RenderFormat::Svg, &RenderOptions::default())
        .expect_err("a single observation is not a line");
    assert_eq!(error.code, "empty_series");
    assert_eq!(error.path, "/panes/0/layers/0/points");
}

#[test]
fn dense_layers_are_drawn_as_a_line_without_markers() {
    let sparse = render_json(
        &hourly(60, ""),
        RenderFormat::Svg,
        &RenderOptions::default(),
    )
    .expect("60 observations stay readable");
    assert_eq!(
        sparse
            .content
            .matches("class=\"chartlet-point chartlet-point-series-1\"")
            .count(),
        60
    );

    // One more observation and the markers would sit closer than eleven pixels apart.
    let dense = render_json(
        &hourly(61, ""),
        RenderFormat::Svg,
        &RenderOptions::default(),
    )
    .unwrap();
    assert!(dense.content.contains("<polyline"));
    assert_eq!(
        dense
            .content
            .matches("class=\"chartlet-point chartlet-point-series-1\"")
            .count(),
        0
    );
}

#[test]
fn dense_layers_are_reported_once_they_outnumber_the_plot_pixels() {
    // The default width is 800, of which 704 pixels are plot.
    let output = render_json(
        &hourly(1_000, ""),
        RenderFormat::Svg,
        &RenderOptions::default(),
    )
    .unwrap();
    assert_eq!(output.warnings.len(), 1);
    assert_eq!(output.warnings[0].code, "dense_chart");
    assert_eq!(output.warnings[0].path, "/panes/0/layers/0/points");
    assert!(
        output.warnings[0]
            .message
            .contains("1000 observations exceed the 704 horizontal pixels"),
        "{}",
        output.warnings[0].message
    );
}

#[test]
fn the_shared_stylesheet_is_the_common_part_and_one_part_per_type() {
    let common = crate::stylesheet_common();
    assert!(!common.contains(".chartlet-type-"));
    let (bar, time) = (
        crate::stylesheet_types(&[crate::ChartType::Bar]),
        crate::stylesheet_types(&[crate::ChartType::Time]),
    );
    assert_eq!(
        crate::stylesheet_for(&[crate::ChartType::Time, crate::ChartType::Bar]),
        format!("{common}{time}{bar}")
    );
    let every_type: String = crate::ChartType::ALL
        .iter()
        .map(|chart_type| crate::stylesheet_types(&[*chart_type]))
        .collect();
    assert_eq!(crate::stylesheet(), format!("{common}{every_type}"));
}

#[test]
fn declared_layer_colors_reach_the_stylesheet() {
    let specification = include_str!("../../examples/revenue-vs-forecast.json");
    let svg = render_json(specification, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(svg.content.contains(".chartlet-style-0{stroke:#7ea6ff}"));
    // A bare var() would drop the series on a page that never defines the variable.
    assert!(
        svg.content
            .contains(".chartlet-style-1{stroke:var(--chart-forecast,currentColor)}")
    );
    assert_eq!(svg.content.matches("chartlet-theme-dark").count(), 2);
    assert!(svg.content.contains("class=\"chartlet-background\""));
    assert!(svg.content.contains("class=\"chartlet-legend\""));
}

#[test]
fn declared_layer_colors_stay_inside_their_chart() {
    let specification = include_str!("../../examples/revenue-vs-forecast.json");
    let svg = render_json(
        specification,
        RenderFormat::Svg,
        &RenderOptions {
            id_prefix: Some("first".to_owned()),
            ..RenderOptions::default()
        },
    )
    .unwrap();
    // Inline SVG stylesheets apply to the whole page; an unscoped rule would recolor the
    // layers of every other chart on it.
    assert!(svg.content.contains("id=\"first\" role=\"img\""));
    assert!(
        svg.content
            .contains("#first .chartlet-style-0{stroke:#7ea6ff}")
    );
    assert!(!svg.content.contains("}.chartlet-style-0{"));
    // Every other rule is scoped too, so a later chart's `.chartlet-line` cannot restyle
    // this one; the dark palette outranks a later light chart's defaults.
    assert!(svg.content.contains("#first .chartlet-title{"));
    assert!(
        svg.content
            .contains(".chartlet-root.chartlet-theme-dark{--chartlet-text:")
    );
    assert!(
        svg.content
            .contains("<style>.chartlet-root{--chartlet-text:")
    );
}

#[test]
fn an_invalid_color_is_reported_and_replaced_by_the_neutral_tone() {
    let declared = hourly(3, ", \"color\": \"red\"");
    let output = render_json(&declared, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert_eq!(output.warnings.len(), 1);
    assert_eq!(output.warnings[0].code, "color_not_supported");
    assert_eq!(output.warnings[0].path, "/panes/0/layers/0/color");
    assert!(output.content.contains(".chartlet-style-0{stroke:#667085}"));
}

#[test]
fn fields_that_belong_to_another_mark_are_rejected_by_name() {
    let annotated = TIME.replace(
        "\"name\": \"Orders\"",
        "\"name\": \"Orders\", \"label\": \"Peak\"",
    );
    let error = render_json(&annotated, RenderFormat::Svg, &RenderOptions::default())
        .expect_err("a label belongs to an annotation");
    assert_eq!(error.code, "option_not_supported");
    assert_eq!(error.path, "/panes/0/layers/0/label");
    assert!(error.message.contains("annotation"), "{}", error.message);
}

#[test]
fn time_charts_reject_the_bar_and_line_fields() {
    for (field, path) in [
        ("\"data\": [{\"label\": \"Jan\", \"value\": 1}],", "/data"),
        ("\"categories\": [\"Jan\"],", "/categories"),
    ] {
        let mixed = TIME.replace(
            "\"schemaVersion\": 1,",
            &format!("\"schemaVersion\": 1, {field}"),
        );
        let error = render_json(&mixed, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("a time chart is drawn from panes and layers");
        assert_eq!(error.code, "option_not_supported", "{field}");
        assert_eq!(error.path, path, "{field}");
    }
}

#[test]
fn layer_names_are_required_and_unique_within_a_pane() {
    let second = "{\"mark\": \"line\", \"name\": \"Orders\", \"points\": [{\"time\": \"2026-03-01\", \"value\": 1}, {\"time\": \"2026-03-02\", \"value\": 2}]}";
    let duplicated = TIME.replace("\"layers\": [", &format!("\"layers\": [{second},"));
    let error = render_json(&duplicated, RenderFormat::Svg, &RenderOptions::default())
        .expect_err("two layers with one name cannot be told apart");
    assert_eq!(error.code, "duplicate_series");
    assert_eq!(error.path, "/panes/0/layers/1/name");

    let anonymous = TIME.replace("\"name\": \"Orders\", ", "");
    let error = render_json(&anonymous, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert_eq!(error.warnings, []);
}

// --- Uncertainty bands, reference lines, stripes, calendars, range bars, small multiples ---

/// A time chart with one measured and one modeled layer; `band` is spliced into every point
/// of the modeled layer and `extra` into its layer object.
fn projection(band: &str, extra: &str) -> String {
    format!(
        r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Projection",
                "showValues": false,
                "panes": [{{
                    "layers": [
                        {{"mark": "line", "name": "Measured", "points": [
                            {{"time": "2000", "value": 0.4}},
                            {{"time": "2010", "value": 0.6}},
                            {{"time": "2020", "value": 1.0}}
                        ]}},
                        {{"mark": "line", "name": "Scenario", "modeled": true{extra}, "points": [
                            {{"time": "2020", "value": 1.0{band}}},
                            {{"time": "2050", "value": 1.8{band}}}
                        ]}}
                    ]
                }}]
            }}"#
    )
}

const BAND: &str = r#", "lower": 0.5, "upper": 2.5"#;

#[test]
fn a_modeled_band_is_hatched_dashed_and_named_modeled() {
    let output = render_ok(&projection(BAND, ""));
    let svg = &output.content;
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    // The band, its hatched copy, and the pattern it refers to by the chart's own prefix.
    assert!(svg.contains("class=\"chartlet-band chartlet-band-series-2\""));
    assert!(svg.contains("<pattern id=\""));
    assert!(svg.contains("-hatch-1\" width=\"6\""));
    assert!(svg.contains("fill=\"url(#chartlet-"));
    assert!(svg.contains("class=\"chartlet-hatch chartlet-style-1\""));
    assert!(svg.contains("chartlet-line chartlet-line-modeled chartlet-line-series-2"));
    assert!(svg.contains(">Scenario (modelled)<"));
    assert!(svg.contains("<title>2050 – Scenario: 1.8 (range 0.5 to 2.5)</title>"));
    assert!(svg.contains(
        "One line has a shaded band from its lower to its upper bound, hatched where modelled"
    ));
    // Annual data reads as years, not as January 1st.
    assert!(svg.contains("from 2000 to 2050"));

    let html = render_json(
        &projection(BAND, ""),
        RenderFormat::Html,
        &RenderOptions::default(),
    )
    .unwrap();
    assert!(html.content.contains(
        "<th scope=\"col\">Scenario</th><th scope=\"col\">Scenario (lower)</th><th scope=\"col\">Scenario (upper)</th>"
    ));
    assert!(html.content.contains(
        "<th scope=\"row\">2050</th><td>Missing</td><td>1.8</td><td>0.5</td><td>2.5</td>"
    ));
}

#[test]
fn a_band_widens_the_value_scale() {
    let svg = render_ok(&projection(BAND, "")).content;
    // The upper edge of 2.5 lies above every value, so the axis has to reach it.
    assert!(svg.contains(">2.5<"), "{svg}");
}

#[test]
fn a_band_needs_both_edges_on_every_point() {
    let half = projection(r#", "lower": 0.5"#, "");
    assert_eq!(
        render_err(&half),
        (
            "incomplete_band",
            "/panes/0/layers/1/points/0/upper".to_owned()
        )
    );
    let mixed = projection(BAND, "").replacen(
        r#""value": 1.8, "lower": 0.5, "upper": 2.5"#,
        r#""value": 1.8"#,
        1,
    );
    assert_eq!(
        render_err(&mixed),
        (
            "incomplete_band",
            "/panes/0/layers/1/points/1/lower".to_owned()
        )
    );
    let inverted = projection(r#", "lower": 2.5, "upper": 0.5"#, "");
    assert_eq!(
        render_err(&inverted),
        (
            "invalid_band",
            "/panes/0/layers/1/points/0/upper".to_owned()
        )
    );
}

#[test]
fn a_value_outside_its_band_is_reported() {
    let output = render_ok(&projection(r#", "lower": 0.1, "upper": 0.2"#, ""));
    assert_eq!(output.warnings.len(), 2);
    assert_eq!(output.warnings[0].code, "value_outside_band");
    assert_eq!(output.warnings[0].path, "/panes/0/layers/1/points/0/value");
}

#[test]
fn a_declared_color_paints_line_band_and_hatch() {
    let svg = render_ok(&projection(BAND, r##", "color": "#7c3aed""##)).content;
    assert!(svg.contains(".chartlet-band.chartlet-style-1{fill:#7c3aed;stroke:none}"));
    assert!(svg.contains("class=\"chartlet-hatch-line chartlet-style-1\""));
}

#[test]
fn the_band_mark_points_to_the_line_band() {
    let error = render_json(
        &with_rules(
            r#"{"mark": "band", "label": "Z", "bottom": 0, "top": 1,
                    "points": [{"time": "2026-03-01", "value": 1}]}"#,
        ),
        RenderFormat::Svg,
        &RenderOptions::default(),
    )
    .expect_err("a zone has no points");
    assert_eq!(error.code, "option_not_supported");
    assert_eq!(error.path, "/panes/0/layers/0/points");
    assert!(
        error.message.contains("lower and upper"),
        "{}",
        error.message
    );
}

/// A time chart over five days whose pane holds `layers`; `extra` goes into the top level.
fn week(layers: &str, extra: &str) -> String {
    format!(
        r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Sensors",
                "showValues": false{extra},
                "panes": [{{"layers": [{layers}]}}]
            }}"#
    )
}

/// A line layer over the five days of [`week`]; `fields` goes into the layer object and
/// `values` are the five values, `null` included.
fn sensor(name: &str, fields: &str, values: [&str; 5]) -> String {
    let points = values
        .iter()
        .enumerate()
        .map(|(day, value)| format!(r#"{{"time": "2026-03-0{}", "value": {value}}}"#, day + 1))
        .collect::<Vec<_>>()
        .join(", ");
    format!(r#"{{"mark": "line", "name": "{name}"{fields}, "points": [{points}]}}"#)
}

const GAPPED: [&str; 5] = ["10", "12", "null", "14", "15"];

#[test]
fn a_null_value_breaks_the_line_and_counts_as_missing() {
    let spec = week(&sensor("Load", "", GAPPED), "");
    let output = render_ok(&spec);
    let svg = &output.content;
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    // Two pieces of line around the gap, and a marker for each of the four values.
    assert_eq!(svg.matches("<polyline").count(), 2);
    assert_eq!(
        svg.matches("class=\"chartlet-point chartlet-point-series-1\"")
            .count(),
        4
    );
    assert!(svg.contains("1 value is missing."), "{svg}");

    let html = html_ok(&spec);
    assert!(html.contains("<th scope=\"row\">2026-03-03</th><td>Missing</td>"));

    let german = html_ok(&german(&spec, "time"));
    assert!(german.contains("1 Wert fehlt."));
    assert!(german.contains("<th scope=\"row\">03.03.2026</th><td>fehlt</td>"));

    // A lone value between two gaps keeps its marker but draws no line.
    let lone = week(&sensor("Load", "", ["10", "null", "12", "null", "15"]), "");
    let svg = render_ok(&lone).content;
    assert!(!svg.contains("<polyline"), "{svg}");
    assert_eq!(
        svg.matches("class=\"chartlet-point chartlet-point-series-1\"")
            .count(),
        3
    );
    assert!(svg.contains("2 values are missing."));

    // Small multiples break their lines the same way.
    let gapped = panel("B", 1.0).replacen(
        r#"{"time": "2010", "value": 12}"#,
        r#"{"time": "2005", "value": null}, {"time": "2010", "value": 12}"#,
        1,
    );
    let spec = multiples(&[panel("A", 1.0), gapped].join(","));
    assert!(render_ok(&spec).content.contains("1 value is missing."));
}

#[test]
fn a_band_breaks_at_a_missing_value_and_a_missing_value_has_no_band() {
    let banded = |gap: &str| {
        let points = [
            r#"{"time": "2026-03-01", "value": 1, "lower": 0, "upper": 2}"#,
            r#"{"time": "2026-03-02", "value": 2, "lower": 1, "upper": 3}"#,
            gap,
            r#"{"time": "2026-03-04", "value": 2, "lower": 1, "upper": 3}"#,
            r#"{"time": "2026-03-05", "value": 3, "lower": 2, "upper": 4}"#,
        ]
        .join(", ");
        week(
            &format!(r#"{{"mark": "line", "name": "Load", "points": [{points}]}}"#),
            "",
        )
    };
    let svg = render_ok(&banded(r#"{"time": "2026-03-03", "value": null}"#)).content;
    assert_eq!(
        svg.matches("class=\"chartlet-band chartlet-band-series-1\"")
            .count(),
        2
    );
    assert_eq!(
        render_err(&banded(
            r#"{"time": "2026-03-03", "value": null, "lower": 1, "upper": 3}"#
        )),
        (
            "invalid_band",
            "/panes/0/layers/0/points/2/lower".to_owned()
        )
    );
}

#[test]
fn a_line_takes_its_declared_pattern_and_weight_in_plot_and_legend() {
    let spec = week(
        &[
            sensor("Dotted", r#", "dash": "dotted""#, GAPPED),
            sensor("Bold", r#", "stroke": "bold""#, GAPPED),
            sensor("Dashed", r#", "dash": "dashed""#, GAPPED),
            sensor(
                "Solid model",
                r#", "modeled": true, "dash": "solid""#,
                GAPPED,
            ),
        ]
        .join(", "),
        "",
    );
    let svg = render_ok(&spec).content;
    assert!(svg.contains(".chartlet-line-dotted{stroke-dasharray:0 7}"));
    assert!(svg.contains(".chartlet-line-bold{stroke-width:4.5}"));
    // Two pieces of line around the gap, and the legend sample.
    for class in [
        "chartlet-line chartlet-line-dotted chartlet-line-series-1",
        "chartlet-line chartlet-line-bold chartlet-line-series-2",
        "chartlet-line chartlet-line-dashed chartlet-line-series-3",
        "chartlet-line chartlet-line-series-4",
    ] {
        assert_eq!(
            svg.matches(&format!("class=\"{class}\"")).count(),
            3,
            "{class}"
        );
    }
    // An explicit pattern replaces the modeled dash, but the legend still says modeled.
    assert!(svg.contains(">Solid model (modelled)<"));
    assert!(!svg.contains("class=\"chartlet-line chartlet-line-modeled"));

    // Without the new options a chart carries none of their rules.
    let plain = render_ok(&week(&sensor("Load", "", GAPPED), "")).content;
    assert!(!plain.contains("chartlet-line-dotted"));

    let rule = week(
        &format!(
            r#"{}, {{"mark": "annotation", "value": 1, "label": "Limit", "dash": "dotted"}}"#,
            sensor("Load", "", GAPPED)
        ),
        "",
    );
    assert_eq!(
        render_err(&rule),
        ("option_not_supported", "/panes/0/layers/1/dash".to_owned())
    );
}

#[test]
fn an_area_is_filled_down_to_zero_and_described() {
    let area = sensor("Load", "", ["10", "12", "null", "14", "15"])
        .replace("\"mark\": \"line\"", "\"mark\": \"area\"");
    let spec = week(
        &[
            area.clone(),
            sensor("Base", "", ["8", "9", "9", "10", "11"]),
        ]
        .join(", "),
        "",
    );
    let output = render_ok(&spec);
    let svg = &output.content;
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert!(svg.contains(".chartlet-area{stroke:none;fill-opacity:.18}"));
    // One filled outline on either side of the gap, and a swatch under the legend sample.
    assert_eq!(
        svg.matches("<polyline points=\"").count(),
        // Two area outlines, two line pieces, one line for the second layer, two samples.
        7
    );
    assert_eq!(
        svg.matches("class=\"chartlet-area chartlet-band-series-1\"")
            .count(),
        3
    );
    // The value axis reaches zero although every value lies well above it.
    assert!(svg.contains("class=\"chartlet-tick\">0</text>"), "{svg}");
    assert!(svg.contains("The area below Load is filled down to zero."));
    assert!(
        html_ok(&german(&spec, "time")).contains("Die Fläche unter Load ist bis null gefüllt.")
    );

    let colored = week(
        &area.replace(
            "\"name\": \"Load\"",
            "\"name\": \"Load\", \"color\": \"#0f766e\"",
        ),
        "",
    );
    let svg = render_ok(&colored).content;
    assert!(svg.contains(".chartlet-area.chartlet-style-0{fill:#0f766e;stroke:none}"));
    assert!(svg.contains("The area below Load is filled down to zero."));
}

#[test]
fn a_time_chart_zooms_into_windows_of_time() {
    let zoom = r#", "zoomSteps": [
            {"label": "All", "from": "2026-03-01", "to": "2026-03-05"},
            {"label": "End", "from": "2026-03-04", "to": 1772668800}
        ]"#;
    let spec = week(&sensor("Load", "", GAPPED), zoom);
    let html = html_ok(&spec);
    assert!(html.contains("<fieldset class=\"chartlet-zoom\">"));
    assert_eq!(html.matches("<svg").count(), 2);
    let end = html
        .split("chartlet-panel chartlet-panel-1")
        .nth(1)
        .and_then(|panel| panel.split("</div>").next())
        .expect("the second window has a panel");
    assert!(end.contains("Time chart with 2 points from 2026-03-04 to 2026-03-05."));
    assert!(!end.contains("2026-03-01"));
    // The table keeps every observation.
    assert!(html.contains("<th scope=\"row\">2026-03-01</th>"));
    // The SVG profile stays one static chart.
    assert_eq!(render_ok(&spec).content.matches("<svg").count(), 1);

    for (steps, code, path) in [
        (
            r#"[{"label": "A", "from": "2026-03-04", "to": "2026-03-04"}, {"label": "B", "from": "2026-03-01", "to": "2026-03-05"}]"#,
            "invalid_zoom_step",
            "/zoomSteps/0/from",
        ),
        (
            r#"[{"label": "A", "from": "2026-03-01", "to": "2026-03-05"}, {"label": "B", "from": "2026-03-02", "to": "2026-03-03"}]"#,
            "zoom_out_of_range",
            "/zoomSteps/1",
        ),
        (
            r#"[{"label": "A", "from": "March", "to": "2026-03-05"}, {"label": "B", "from": "2026-03-01", "to": "2026-03-05"}]"#,
            "invalid_time",
            "/zoomSteps/0/from",
        ),
        (
            r#"[{"label": "A", "from": "2026-03-01", "to": "2026-03-05"}]"#,
            "not_enough_zoom_steps",
            "/zoomSteps",
        ),
    ] {
        let spec = week(
            &sensor("Load", "", GAPPED),
            &format!(r#", "zoomSteps": {steps}"#),
        );
        assert_eq!(render_err(&spec), (code, path.to_owned()), "{steps}");
    }

    // A category chart still counts categories, and a timestamp is no category.
    let dated = SPEC.replace(
        "\"schemaVersion\": 1,",
        r#""schemaVersion": 1, "zoomSteps": [{"label": "A", "from": "2026-03-01", "to": 1}, {"label": "B", "from": 0, "to": 1}],"#,
    );
    assert_eq!(
        render_err(&dated),
        ("invalid_spec", "/zoomSteps/0/from".to_owned())
    );
    let panels = [panel("A", 1.0), panel("B", 1.0)].join(",");
    let zoomed = multiples(&panels).replace(
        "\"schemaVersion\": 1,",
        r#""schemaVersion": 1, "zoomSteps": [{"label": "A", "from": "2000", "to": "2010"}, {"label": "B", "from": "2000", "to": "2050"}],"#,
    );
    assert_eq!(
        render_err(&zoomed),
        ("option_not_supported", "/zoomSteps".to_owned())
    );
}

#[test]
fn a_pane_takes_six_layers_when_two_bring_their_own_color() {
    let colored = r##", "color": "#0f766e""##;
    let layers = |names: &[(&str, &str)]| {
        names
            .iter()
            .map(|(name, fields)| sensor(name, fields, GAPPED))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let six = [
        ("Own", colored),
        ("A", ""),
        ("B", ""),
        ("C", ""),
        ("Also own", colored),
        ("D", ""),
    ];
    let output = render_ok(&week(&layers(&six), ""));
    let svg = &output.content;
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    // The fifth data layer takes the first palette color no layer before it holds.
    assert!(svg.contains("class=\"chartlet-line chartlet-line-series-1\""));
    for series in 2..=4 {
        assert!(svg.contains(&format!("chartlet-line-series-{series}")));
    }
    assert_eq!(svg.matches("class=\"chartlet-legend\"").count(), 6);

    let seven = [six.as_slice(), &[("E", colored)]].concat();
    assert_eq!(
        render_err(&week(&layers(&seven), "")),
        ("too_many_layers", "/panes/0/layers".to_owned())
    );
    let five_in_palette = [("A", ""), ("B", ""), ("C", ""), ("D", ""), ("E", "")];
    assert_eq!(
        render_err(&week(&layers(&five_in_palette), "")),
        ("too_many_layers", "/panes/0/layers/4/color".to_owned())
    );
}

#[test]
fn a_long_legend_wraps_into_rows_without_overlap() {
    let names = [
        "Server hall north",
        "Server hall south",
        "Cooling plant",
        "Office floors",
        "Lighting",
        "Charging",
    ];
    let layers = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let fields = if index < 4 {
                String::new()
            } else {
                format!(r##", "color": "#12345{index}""##)
            };
            sensor(name, &fields, GAPPED)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let svg = render_ok(&week(&layers, r#", "width": 480"#)).content;
    let legend = placed_texts(&svg, "chartlet-legend");
    let written: Vec<&str> = legend
        .iter()
        .map(|(_, _, content)| content.as_str())
        .collect();
    assert_eq!(written, names, "every name is written in full");
    let rows: std::collections::BTreeSet<u64> =
        legend.iter().map(|(_, y, _)| y.to_bits()).collect();
    assert!(rows.len() > 1, "{legend:?}");
    let width = |content: &str| BuiltinMetrics.width(content, 12.0);
    for (x, y, content) in &legend {
        assert!(x + width(content) <= 480.0, "{content} leaves the chart");
        for (other_x, other_y, other) in &legend {
            if other_y.to_bits() == y.to_bits() && other_x > x {
                assert!(
                    x + width(content) < *other_x - 24.0,
                    "{content} runs into {other}"
                );
            }
        }
    }
    // The plot starts below the last legend row.
    let lowest = legend.iter().map(|(_, y, _)| *y).fold(0.0, f64::max);
    let ticks = placed_texts(&svg, "chartlet-tick");
    assert!(ticks.iter().all(|(_, y, _)| *y > lowest), "{ticks:?}");
}

// --- Candlesticks and stacked panes ---
