use super::*;

#[test]
fn grouped_bars_render_legend_bars_and_series_table() {
    let output = render_json(GROUPED, RenderFormat::Html, &RenderOptions::default()).unwrap();
    // Every bar plus one legend swatch per series.
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-bar chartlet-series-1\"")
            .count(),
        4
    );
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-bar chartlet-series-2\"")
            .count(),
        3
    );
    assert!(
        output
            .content
            .contains("<th scope=\"col\">Budget</th><th scope=\"col\">Actual</th>")
    );
    assert!(
        output
            .content
            .contains("<th scope=\"row\">Mar</th><td>140</td><td>Missing</td>")
    );
    assert!(output.content.contains(
        "Bar chart with 3 categories and 2 series (Budget, Actual). Highest: 150 (Budget in Feb). Lowest: 120 (Budget in Jan). 1 value is missing."
    ));
}

#[test]
fn grouped_bars_reject_mismatched_series_length() {
    let short = GROUPED.replace("[130, 145, null]", "[130, 145]");
    let error = render_json(&short, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "series_length_mismatch");
    assert_eq!(error.path, "/series/1/values");
}

#[test]
fn data_and_series_are_mutually_exclusive() {
    let both = GROUPED.replace(
        "\"categories\"",
        "\"data\": [{\"label\": \"Jan\", \"value\": 1}], \"categories\"",
    );
    let error = render_json(&both, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "conflicting_data_shape");
    assert_eq!(error.path, "/data");
}

#[test]
fn line_charts_draw_several_series_apart_by_color_and_pattern() {
    let line = GROUPED.replace("\"bar\"", "\"line\"");
    let svg = render_ok(&line).content;
    assert!(svg.contains("chartlet-line chartlet-line-series-1\""));
    assert!(svg.contains("chartlet-line chartlet-line-series-2 chartlet-line-dashed"));
    assert!(svg.contains("class=\"chartlet-legend\""));
    assert!(!svg.contains("class=\"chartlet-value\""));
}

#[test]
fn patterns_outline_every_other_series() {
    let spec = GROUPED.replacen('{', "{\"patterns\": true,", 1);
    let svg = render_ok(&spec).content;
    assert!(svg.contains("chartlet-bar chartlet-series-2 chartlet-outline"));
    assert!(svg.contains(".chartlet-outline{fill:var(--chartlet-background)"));
    let line = spec.replace("\"bar\"", "\"line\"");
    assert_eq!(
        render_err(&line),
        ("option_not_supported", "/patterns".to_owned())
    );
}

#[test]
fn stacks_add_up_and_a_percent_stack_takes_no_negative_values() {
    let stacked = GROUPED.replacen('{', "{\"stack\": \"normal\",", 1);
    let svg = render_ok(&stacked).content;
    assert!(
        svg.contains("The series are stacked. Highest total:"),
        "{svg}"
    );
    let percent = GROUPED.replacen('{', "{\"stack\": \"percent\",", 1);
    assert!(render_ok(&percent).content.contains("shares of the series"));
    assert_eq!(
        render_err(&percent.replace("120", "-120")),
        ("invalid_value", "/series/0/values/0".to_owned())
    );
    let single = SPEC.replacen('{', "{\"stack\": \"normal\",", 1);
    assert_eq!(
        render_err(&single),
        ("option_not_supported", "/stack".to_owned())
    );
}

#[test]
fn stretched_stripes_fill_the_canvas_and_stretch() {
    let spec = r#"{"schemaVersion": 1, "type": "stripes", "title": "Band",
            "stripes": {"firstYear": 2000, "values": [0.1, -0.2, 0.3, 0.5], "stretch": true}}"#;
    let svg = render_ok(spec).content;
    assert!(svg.contains(" preserveAspectRatio=\"none\""));
    assert!(svg.contains("<rect x=\"0\" y=\"0\""), "{svg}");
    assert!(!svg.contains("class=\"chartlet-title\""));
    assert!(!svg.contains("class=\"chartlet-tick\""));
}

#[test]
fn tooltips_markers_leaves_a_dense_line_without_targets() {
    let points: Vec<String> = (0..120)
        .map(|minute| {
            format!(
                r#"{{"time": "2026-03-01T{:02}:{:02}:00Z", "value": {}}}"#,
                minute / 60,
                minute % 60,
                minute % 7
            )
        })
        .collect();
    let spec = format!(
        r#"{{"schemaVersion": 1, "type": "time", "title": "Visits", "panes": [{{"layers":
            [{{"mark": "line", "name": "Visits", "points": [{}]}}]}}]}}"#,
        points.join(",")
    );
    let every = render_ok(&spec).content;
    assert_eq!(every.matches("class=\"chartlet-hit\"").count(), 120);
    let markers = spec.replacen('{', "{\"tooltips\": \"markers\",", 1);
    let svg = render_ok(&markers).content;
    assert!(!svg.contains("class=\"chartlet-hit\""));
    assert!(svg.len() * 2 < every.len());
    let bar = SPEC.replacen('{', "{\"tooltips\": \"markers\",", 1);
    assert_eq!(
        render_err(&bar),
        ("option_not_supported", "/tooltips".to_owned())
    );
}

#[test]
fn a_point_layer_draws_dots_without_a_line() {
    let spec = include_str!("../../examples/resting-heart-rate.json");
    let svg = render_ok(spec).content;
    // Only the weekly mean is a line; the readings are dots, one in the legend as well.
    assert_eq!(svg.matches("<polyline").count(), 2, "{svg}");
    assert_eq!(
        svg.matches("class=\"chartlet-point chartlet-point-series-1\"")
            .count(),
        28
    );
    let dashed = spec.replacen(
        "\"mark\": \"point\",",
        "\"mark\": \"point\", \"dash\": \"dotted\",",
        1,
    );
    assert_eq!(
        render_err(&dashed),
        ("option_not_supported", "/panes/0/layers/0/dash".to_owned())
    );
}

#[test]
fn a_horizontal_log_axis_writes_only_labels_that_fit_and_keeps_its_end_label() {
    let spec = r#"{"schemaVersion":1,"type":"rangebar","orientation":"horizontal","title":"t",
            "width":480,"height":240,"valueAxis":{"scale":"log"},"ranges":[
            {"label":"A","low":1000000,"high":10000000},{"label":"B","low":10,"high":300}]}"#;
    let svg = render_ok(spec).content;
    // Seven gridlines from 10 to 10,000,000, but only every other tick labelled.
    assert_eq!(svg.matches("class=\"chartlet-grid\"").count(), 7, "{svg}");
    assert_eq!(svg.matches("class=\"chartlet-tick\"").count(), 4, "{svg}");
    let end = svg.find(">1000000 to 10000000<").expect("the end label");
    let x: f64 = svg[..end]
        .rsplit("<text x=\"")
        .next()
        .and_then(|tail| tail.split('"').next())
        .and_then(|x| x.parse().ok())
        .expect("the label's x");
    assert!(
        x + crate::metrics::TextMetrics::width(
            &crate::metrics::BuiltinMetrics,
            "1000000 to 10000000",
            12.0
        ) <= 480.0,
        "{svg}"
    );
}

#[test]
fn every_layer_writes_its_times_as_finely_as_it_needs() {
    // Monthly values beside yearly means placed mid-year: the means say only the year.
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "CO2", "panes": [{"layers": [
            {"mark": "line", "name": "Monthly", "points": [{"time": "2020-01", "value": 1},
                {"time": "2020-02", "value": 2}, {"time": "2020-03", "value": 3}]},
            {"mark": "line", "name": "Yearly", "precision": "year", "points": [
                {"time": "2019-07-01", "value": 1}, {"time": "2020-07-01", "value": 2}]}]}]}"#;
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .expect("renders")
        .content;
    assert!(html.contains("<title>2020 – Yearly: 2</title>"), "{html}");
    assert!(
        html.contains("<title>2020-02 – Monthly: 2</title>"),
        "{html}"
    );
    assert!(html.contains("<th scope=\"row\">2019</th>"), "{html}");

    // On a numeric axis, a finely spaced helper line does not add decimals to the points.
    let numeric = r#"{"schemaVersion": 1, "type": "time", "title": "Quakes",
            "timeAxis": {"kind": "number"}, "panes": [{"layers": [
            {"mark": "point", "name": "Counted", "points": [{"time": 4.5, "value": 3},
                {"time": 5, "value": 2}]},
            {"mark": "line", "name": "Fit", "points": [{"time": 4.45, "value": 3},
                {"time": 5.05, "value": 2}]}]}]}"#;
    let svg = render_ok(numeric).content;
    assert!(svg.contains("<title>4.5 – Counted: 3</title>"), "{svg}");
    assert!(svg.contains("<title>4.45 – Fit: 3</title>"), "{svg}");
    let on_numeric = numeric.replacen(
        "\"name\": \"Fit\",",
        "\"name\": \"Fit\", \"precision\": \"year\",",
        1,
    );
    assert_eq!(
        render_err(&on_numeric),
        (
            "option_not_supported",
            "/panes/0/layers/1/precision".to_owned()
        )
    );
}

#[test]
fn a_line_without_markers_keeps_its_tooltips() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Fit", "showValues": false,
            "panes": [{"layers": [{"mark": "line", "name": "Fit", "markers": false, "points": [
            {"time": "2020", "value": 1}, {"time": "2021", "value": 2}]}]}]}"#;
    let svg = render_ok(spec).content;
    assert!(!svg.contains("class=\"chartlet-point"), "{svg}");
    assert_eq!(svg.matches("class=\"chartlet-hit\"").count(), 2, "{svg}");
    let point = spec.replace("\"mark\": \"line\"", "\"mark\": \"point\"");
    assert_eq!(
        render_err(&point),
        (
            "option_not_supported",
            "/panes/0/layers/0/markers".to_owned()
        )
    );
}

#[test]
fn a_declared_precision_names_times_but_keeps_the_ticks() {
    let points: Vec<String> = (1989..=2022)
        .map(|year| format!(r#"{{"time": "{year}-07-01", "value": {}}}"#, year % 7))
        .collect();
    let spec = format!(
        r#"{{"schemaVersion": 1, "type": "time", "title": "pH", "width": 720, "height": 340,
            "panes": [{{"layers": [{{"mark": "line", "name": "pH", "points": [{}]}}]}}]}}"#,
        points.join(",")
    );
    let declared = spec.replacen('{', "{\"timeAxis\": {\"precision\": \"year\"},", 1);
    let ticks = |svg: &str| svg.matches("class=\"chartlet-tick\"").count();
    assert_eq!(
        ticks(&render_ok(&declared).content),
        ticks(&render_ok(&spec).content)
    );
}

#[test]
fn the_last_step_holds_until_its_end_without_an_observation_there() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Annual means",
            "showValues": false, "panes": [{"layers": [{"mark": "line", "name": "Mean",
            "curve": "step", "stepEnd": "2026", "points": [{"time": "2024", "value": 1},
            {"time": "2025", "value": 2}]}]}]}"#;
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .expect("renders")
        .content;
    // Two observations, two tooltips and two rows; the line runs on to the end of 2025.
    assert_eq!(html.matches("<title>20").count(), 2, "{html}");
    assert!(!html.contains("<th scope=\"row\">2026</th>"), "{html}");
    let line = html
        .split("<polyline points=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("the line");
    assert_eq!(line.split(' ').count(), 4, "{line}");

    let linear = spec.replace("\"curve\": \"step\", ", "");
    assert_eq!(
        render_err(&linear),
        (
            "option_not_supported",
            "/panes/0/layers/0/stepEnd".to_owned()
        )
    );
    let early = spec.replace("\"stepEnd\": \"2026\"", "\"stepEnd\": \"2025\"");
    assert_eq!(
        render_err(&early),
        ("unordered_time", "/panes/0/layers/0/stepEnd".to_owned())
    );
}

#[test]
fn a_compact_chart_keeps_room_for_its_plot() {
    let spec = SPEC.replacen('{', "{\"width\": 240,", 1);
    let svg = render_ok(&spec).content;
    // The plot starts after the compact gutter of 56 pixels.
    assert!(svg.contains("<line x1=\"56\""), "{svg}");
    let too_narrow = SPEC.replacen('{', "{\"width\": 199,", 1);
    assert_eq!(
        render_err(&too_narrow),
        ("invalid_dimension", "/width".to_owned())
    );
}

#[test]
fn panel_titles_wrap_and_notes_sit_below_them() {
    let panel = |title: &str, note: &str| {
        format!(
            r#"{{"title": "{title}", "note": "{note}", "layers": [{{"mark": "line",
                "points": [{{"time": "2000", "value": 1}}, {{"time": "2010", "value": 2}}]}}]}}"#
        )
    };
    let spec = format!(
        r#"{{"schemaVersion": 1, "type": "multiples", "title": "Causes", "width": 720,
            "height": 360, "columns": 3, "panes": [{}, {}]}}"#,
        panel("The sun and its long cycles of activity", "Does not match"),
        panel("Greenhouse gases", "Matches"),
    );
    let output = render_ok(&spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    // The long title takes two lines; the plots of both panels start below the same head.
    assert_eq!(
        svg.matches("class=\"chartlet-panel-title\"").count(),
        3,
        "{svg}"
    );
    assert_eq!(
        svg.matches("class=\"chartlet-panel-note\"").count(),
        2,
        "{svg}"
    );
    assert!(svg.contains(".chartlet-panel-note{"));
    assert!(svg.contains("Notes: The sun and its long cycles of activity: Does not match"));
    let strong = spec.replacen(
        "\"note\": \"Matches\"",
        "\"note\": \"Matches\", \"noteEmphasis\": true",
        1,
    );
    assert!(
        render_ok(&strong)
            .content
            .contains("class=\"chartlet-panel-note chartlet-panel-note-strong\">Matches<")
    );
    let bare = spec.replacen("\"note\": \"Matches\"", "\"noteEmphasis\": true", 1);
    assert_eq!(
        render_err(&bare),
        ("option_not_supported", "/panes/1/noteEmphasis".to_owned())
    );

    let time = SPEC_TIME_FOR_NOTE.to_owned();
    assert_eq!(
        render_err(&time),
        ("option_not_supported", "/panes/0/note".to_owned())
    );
}

const SPEC_TIME_FOR_NOTE: &str = r#"{"schemaVersion": 1, "type": "time", "title": "t",
        "panes": [{"note": "n", "layers": [{"mark": "line", "points": [{"time": "2000",
        "value": 1}, {"time": "2010", "value": 2}]}]}]}"#;

#[test]
fn a_declared_time_step_places_the_ticks() {
    let points: Vec<String> = (1850..=2019)
        .map(|year| format!(r#"{{"time": "{year}", "value": {}}}"#, year % 3))
        .collect();
    let spec = format!(
        r#"{{"schemaVersion": 1, "type": "time", "title": "Forcing", "width": 260,
            "height": 240, "showValues": false, "timeAxis": {{"step": 50}},
            "panes": [{{"layers": [{{"mark": "line", "points": [{}]}}]}}]}}"#,
        points.join(",")
    );
    let svg = render_ok(&spec).content;
    for year in ["1850", "1900", "1950", "2000"] {
        assert!(svg.contains(&format!(">{year}</text>")), "{year}: {svg}");
    }
    // Ticks every five years would run into each other: only the labels that fit are written.
    let yearly = spec.replace("\"step\": 50", "\"step\": 5");
    let svg = render_ok(&yearly).content;
    assert!(svg.matches("class=\"chartlet-tick\"").count() < 20, "{svg}");
    let fraction = spec.replace("\"step\": 50", "\"step\": 2.5");
    assert_eq!(
        render_err(&fraction),
        ("invalid_axis_range", "/timeAxis/step".to_owned())
    );
}

#[test]
fn a_declared_end_extends_the_time_axis() {
    let points: Vec<String> = (1991..=2025)
        .map(|year| format!(r#"{{"time": "{year}", "value": {}}}"#, year % 5))
        .collect();
    let spec = format!(
        r#"{{"schemaVersion": 1, "type": "time", "title": "O2", "width": 380, "height": 190,
            "showValues": false, "timeAxis": {{"step": 10, "min": "1990"}},
            "panes": [{{"layers": [{{"mark": "line", "points": [{}]}}]}}]}}"#,
        points.join(",")
    );
    let output = render_ok(&spec);
    assert!(
        output.content.contains(">1990</text>"),
        "{}",
        output.content
    );
    // The axis end is no observation: the description still starts in 1991.
    assert!(output.content.contains("1991"), "{}", output.content);
    let reversed = spec.replace("\"min\": \"1990\"", "\"min\": \"1990\", \"max\": \"1980\"");
    assert_eq!(
        render_err(&reversed),
        ("invalid_axis_range", "/timeAxis/max".to_owned())
    );
    let collapsed = spec.replace("\"step\": 10,", "\"step\": 10, \"gaps\": \"collapse\",");
    assert_eq!(
        render_err(&collapsed),
        ("option_not_supported", "/timeAxis/min".to_owned())
    );
}

#[test]
fn a_medium_line_and_a_bare_panel_suit_small_charts() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Panel", "showTitle": false,
            "width": 260, "height": 180, "showValues": false, "panes": [{"layers": [
            {"mark": "line", "stroke": "medium", "markers": false, "points": [
            {"time": "2000", "value": 1}, {"time": "2010", "value": 2}]}]}]}"#;
    let svg = render_ok(spec).content;
    assert!(
        svg.contains("class=\"chartlet-line chartlet-line-medium "),
        "{svg}"
    );
    assert!(
        svg.contains(".chartlet-line-medium{stroke-width:2}"),
        "{svg}"
    );
    // No title, legend or value axis title: the plot starts 14 pixels from the top.
    assert!(svg.contains(" y1=\"14\""), "{svg}");
    // Short value labels on a compact chart: a gutter narrower than the compact 56 pixels.
    let left: f64 = svg
        .split("<line x1=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .and_then(|x| x.parse().ok())
        .expect("the first gridline");
    assert!(left < 56.0, "{left}");
}

#[test]
fn a_unit_follows_the_top_tick_and_names_the_columns() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Sun", "showTitle": false,
            "width": 340, "height": 190, "showValues": false, "panes": [{"valueAxis": {"min": -2,
            "max": 4, "exact": true, "step": 1, "unit": "W/m²"}, "layers": [{"mark": "line",
            "name": "Sun", "points": [{"time": "2000", "value": 1}, {"time": "2010", "value": 2}]}]}]}"#;
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .expect("renders")
        .content;
    assert!(html.contains(">4 W/m²</text>"), "{html}");
    assert_eq!(html.matches("W/m²</text>").count(), 1, "{html}");
    assert!(html.contains(">Sun (W/m²)</th>"), "{html}");
    // No title above the plot: the plot starts near the top.
    assert!(html.contains(" y1=\"14\""), "{html}");
    let bar = SPEC.replacen('{', "{\"valueAxis\": {\"unit\": \"€\"},", 1);
    assert_eq!(
        render_err(&bar),
        ("option_not_supported", "/valueAxis/unit".to_owned())
    );
}

#[test]
fn a_short_negative_bar_keeps_its_label_out_of_the_category_labels() {
    let spec = r#"{"schemaVersion": 1, "type": "bar", "title": "Change", "width": 360,
            "height": 360, "orientation": "horizontal", "valueAxis": {"format": "percent"},
            "data": [{"label": "First quarter", "value": 0.12},
            {"label": "Second quarter", "value": -0.04}, {"label": "Fourth quarter", "value": 0.15}]}"#;
    let output = render_ok(spec);
    assert!(
        !output.content.contains(">\u{2212}4%<"),
        "{}",
        output.content
    );
    assert!(output.content.contains(">12%<"));
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "value_labels_omitted")
    );
}

#[test]
fn a_low_pane_labels_only_the_value_ticks_that_fit() {
    let points: Vec<String> = (1..=28)
        .map(|day| format!(r#"{{"time": "2026-02-{day:02}", "value": {}}}"#, day % 9))
        .collect();
    let spec = format!(
        r#"{{"schemaVersion": 1, "type": "time", "title": "Volume", "width": 360,
            "height": 200, "showValues": false, "panes": [{{"layers": [{{"mark": "line",
            "points": [{}]}}]}}]}}"#,
        points.join(",")
    );
    let svg = render_ok(&spec).content;
    let grid = svg.matches("class=\"chartlet-grid\"").count()
        + svg.matches("class=\"chartlet-zero\"").count();
    let ys: Vec<f64> = svg
        .split("text-anchor=\"end\" class=\"chartlet-tick\"")
        .filter_map(|head| head.rsplit(" y=\"").next()?.split('"').next()?.parse().ok())
        .collect();
    // Labelled value ticks stand at least a label's height apart.
    assert!(
        ys.windows(2).all(|pair| (pair[0] - pair[1]).abs() >= 12.0),
        "{ys:?}"
    );
    assert!(grid > ys.len(), "{svg}");
}

#[test]
fn qa_0_7_second_review_findings_stay_fixed() {
    let line = |extra: &str, points: &str| {
        format!(
            r#"{{"schemaVersion": 1, "type": "time", "title": "T", "showValues": false {extra},
                "panes": [{{"layers": [{{"mark": "line", "points": [{points}]}}]}}]}}"#
        )
    };
    let unit = r#"{"schemaVersion": 1, "type": "time", "title": "S", "panes": [{"valueAxis":
            {"unit": "kg"}, "layers": [{"mark": "line", "points": [{"time": "2020", "value": 1},
            {"time": "2021", "value": 2}]}]}]}"#;
    // A stacked bar chart takes a step across its totals; a percent stack across 0 to 1.
    let stacked = r#"{"schemaVersion": 1, "type": "bar", "title": "S", "stack": "normal",
            "valueAxis": {"step": 50}, "categories": ["a", "b"], "series": [
            {"name": "x", "values": [100, 100]}, {"name": "y", "values": [100, 100]},
            {"name": "z", "values": [100, 100]}, {"name": "w", "values": [100, 100]}]}"#;
    assert!(render_ok(stacked).content.contains(">350</text>"));
    let percent = stacked
        .replace("\"normal\"", "\"percent\"")
        .replace("\"step\": 50", "\"step\": 0.25");
    assert!(render_ok(&percent).content.contains(">75%</text>"));

    // Stacks grow from zero as drawn, whatever `reverse` says.
    let reversed = stacked.replace("\"step\": 50", "\"reverse\": true");
    let options = |id: &str| RenderOptions {
        id_prefix: Some(id.to_owned()),
        ..RenderOptions::default()
    };
    let plain = stacked.replace("\"valueAxis\": {\"step\": 50}, ", "");
    assert_eq!(
        render_json(&reversed, RenderFormat::Svg, &options("r")).map(|out| out.content),
        render_json(&plain, RenderFormat::Svg, &options("r")).map(|out| out.content)
    );

    // Tooltips and the description write the unit after the value.
    let svg = render_ok(unit).content;
    assert!(svg.contains("<title>2021: 2 kg</title>"), "{svg}");
    assert!(svg.contains("Highest: 2 kg"), "{svg}");
    assert!(!svg.contains("(kg)"), "{svg}");

    // A declared step that places a single year tick is accepted.
    let decade = line(
        r#", "timeAxis": {"step": 10}"#,
        r#"{"time": "2003", "value": 1}, {"time": "2012", "value": 2}"#,
    );
    assert!(render_ok(&decade).content.contains(">2010</text>"));
}

#[test]
fn qa_0_7_findings_stay_fixed() {
    let line = |extra: &str, points: &str| {
        format!(
            r#"{{"schemaVersion": 1, "type": "time", "title": "T", "showValues": false {extra},
                "panes": [{{"layers": [{{"mark": "line", "curve": "step", "stepEnd": "2025",
                "points": [{points}]}}]}}]}}"#
        )
    };
    let years = r#"{"time": "2020", "value": 1}, {"time": "2021", "value": 2},
            {"time": "2022", "value": null}, {"time": "2023", "value": null}"#;
    // A trailing gap stays open: the last step does not run on to stepEnd.
    let svg = render_ok(&line("", years)).content;
    let polyline = svg
        .split("<polyline points=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("the line");
    assert_eq!(polyline.split(' ').count(), 3, "{polyline}");

    // A zoom window shows its own span, not the declared ends of the whole axis.
    let zoomed = line(
        r#", "timeAxis": {"min": "2010", "max": "2030"},
            "zoomSteps": [{"label": "All", "from": "2020", "to": "2023"},
            {"label": "Start", "from": "2020", "to": "2021"}]"#,
        r#"{"time": "2020", "value": 1}, {"time": "2021", "value": 2},
            {"time": "2022", "value": 3}, {"time": "2023", "value": 2}"#,
    );
    let html = render_json(&zoomed, RenderFormat::Html, &RenderOptions::default())
        .expect("renders")
        .content;
    let window = html.split("-z1\"").nth(1).expect("the second window");
    assert!(
        !window.contains(">2030<") && !window.contains(">2010<"),
        "{window}"
    );

    // A sparkline has no axis for a unit; an unnamed layer's column still names it.
    let spark = r#"{"schemaVersion": 1, "type": "time", "title": "S", "sparkline": true,
            "width": 120, "height": 32, "panes": [{"valueAxis": {"unit": "kg"}, "layers": [
            {"mark": "line", "points": [{"time": "2020", "value": 1}, {"time": "2021", "value": 2}]}]}]}"#;
    assert_eq!(
        render_err(spark),
        ("option_not_supported", "/panes/0/valueAxis/unit".to_owned())
    );
    let unit = spark
        .replace("\"sparkline\": true,", "")
        .replace("\"width\": 120, \"height\": 32,", "");
    let html = render_json(&unit, RenderFormat::Html, &RenderOptions::default())
        .expect("renders")
        .content;
    assert!(html.contains(">Value (kg)</th>"), "{html}");

    // A range bar legend that does not fit on one row wraps; every group keeps its key.
    let ranges: String = [
        "Small companies",
        "Medium companies",
        "Large companies",
        "Public sector bodies",
    ]
    .iter()
    .enumerate()
    .map(|(index, group)| {
        format!(r#"{{"label": "{index}", "low": 1, "high": 2, "group": "{group}"}}"#)
    })
    .collect::<Vec<_>>()
    .join(",");
    let groups = format!(
        r#"{{"schemaVersion": 1, "type": "rangebar", "title": "R", "width": 320,
            "height": 420, "ranges": [{ranges}]}}"#
    );
    let output = render_ok(&groups);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let rows: std::collections::BTreeSet<&str> = output
        .content
        .split("class=\"chartlet-legend\"")
        .filter_map(|head| head.rsplit(" y=\"").next()?.split('"').next())
        .collect();
    assert!(rows.len() > 2, "{rows:?}");

    // The smallest chart keeps a plot of its own height and says it is too small.
    let small = r#"{"schemaVersion": 1, "type": "line", "title": "Staff by month",
            "width": 200, "height": 160, "categoryAxis": {"title": "Month"},
            "valueAxis": {"title": "Employees"}, "data": [{"label": "Jan", "value": 1},
            {"label": "Feb", "value": 5}, {"label": "Mar", "value": 3}]}"#;
    let output = render_ok(small);
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "dense_chart")
    );
}

#[test]
fn range_groups_take_palette_colors_and_a_legend() {
    let spec = include_str!("../../examples/soil-animals.json");
    let svg = render_ok(spec).content;
    // Three groups: a span in each color, and a swatch per group in the legend.
    for (series, spans) in [(1, 2), (2, 3), (3, 3)] {
        assert_eq!(
            svg.matches(&format!(
                "class=\"chartlet-range chartlet-range-series-{series}\""
            ))
            .count(),
            spans + 1,
            "{svg}"
        );
    }
    assert!(svg.contains(">Mesofauna</text>"));
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .expect("renders")
        .content;
    assert!(html.contains("Nematodes (Microfauna)"), "{html}");
    let partial = spec.replacen(", \"group\": \"Microfauna\"", "", 1);
    assert_eq!(
        render_err(&partial),
        ("missing_group", "/ranges/0/group".to_owned())
    );
    let five = spec
        .replacen("\"Microfauna\"", "\"A\"", 1)
        .replacen("\"Mesofauna\"", "\"B\"", 1);
    assert_eq!(
        render_err(&five),
        ("too_many_series", "/ranges/5/group".to_owned())
    );
}

#[test]
fn stacked_areas_rest_on_each_other() {
    let spec = include_str!("../../examples/generation-mix.json");
    let options = RenderOptions {
        hooks: true,
        ..RenderOptions::default()
    };
    let output = render_json(spec, RenderFormat::Html, &options).expect("renders");
    // The scale reaches the total of the highest stack, 2,920 TWh in 2016.
    assert!(
        output.content.contains(">3,000</text>"),
        "{}",
        output.content
    );
    assert_eq!(output.content.matches(" data-stacked=\"\"").count(), 4);
    assert!(
        output
            .content
            .contains("Stacked areas, from the bottom up: Fossil, Nuclear")
    );
    // A tooltip tells the layer's own value, not the top of the stack.
    assert!(
        output.content.contains("Wind and solar: 760"),
        "{}",
        output.content
    );

    // A stacked pane takes eight areas; beyond the palette they bring their own colors.
    let mut extra = String::new();
    for (index, color) in ["#0a0", "#a0a", "#aa0", "#0aa"].iter().enumerate() {
        let points = [2000, 2004, 2008, 2012, 2016, 2020, 2024]
            .map(|year| format!(r#"{{"time": "{year}", "value": 10}}"#))
            .join(",");
        std::fmt::Write::write_fmt(
            &mut extra,
            format_args!(r#",{{"mark": "area", "name": "Extra {index}", "color": "{color}", "points": [{points}]}}"#),
        )
        .expect("writing to String cannot fail");
    }
    let eight = spec.replacen(
        "\n      ]\n    }\n  ]",
        &format!("{extra}\n      ]\n    }}\n  ]"),
        1,
    );
    assert_eq!(
        render_ok(&eight)
            .content
            .matches("class=\"chartlet-legend\"")
            .count(),
        8
    );
    let nine = eight.replacen(",{\"mark\": \"area\", \"name\": \"Extra 0\"", ",{\"mark\": \"area\", \"name\": \"Extra 9\", \"color\": \"#123\", \"points\": [{\"time\": \"2000\", \"value\": 1}, {\"time\": \"2024\", \"value\": 1}]},{\"mark\": \"area\", \"name\": \"Extra 0\"", 1);
    assert_eq!(
        render_err(&nine),
        ("too_many_layers", "/panes/0/layers".to_owned())
    );

    let percent = spec.replace("\"stack\": \"normal\"", "\"stack\": \"percent\"");
    assert_eq!(
        render_err(&percent),
        ("option_not_supported", "/panes/0/stack".to_owned())
    );
    let gap = spec.replacen("\"value\": 1520", "\"value\": null", 1);
    assert_eq!(
        render_err(&gap),
        (
            "unaligned_stack",
            "/panes/0/layers/0/points/1/value".to_owned()
        )
    );
    let negative = spec.replacen("\"value\": 1520", "\"value\": -1", 1);
    assert_eq!(
        render_err(&negative),
        (
            "negative_in_stack",
            "/panes/0/layers/0/points/1/value".to_owned()
        )
    );
    let shifted = spec.replacen(
        "\"time\": \"2024\", \"value\": 620",
        "\"time\": \"2023\", \"value\": 620",
        1,
    );
    assert_eq!(
        render_err(&shifted),
        ("unaligned_stack", "/panes/0/layers/1/points".to_owned())
    );
}

#[test]
fn a_sparkline_draws_only_its_line_at_a_small_size() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Visitors", "sparkline": true,
            "width": 120, "height": 32, "panes": [{"layers": [{"mark": "line", "name": "Visitors",
            "points": [{"time": "2026-03-01", "value": 3}, {"time": "2026-03-02", "value": 5},
                       {"time": "2026-03-03", "value": 4}]}]}]}"#;
    let svg = render_ok(spec).content;
    assert!(svg.contains("width=\"120\" height=\"32\""));
    assert!(svg.contains("<title id="));
    assert!(!svg.contains("class=\"chartlet-tick\""));
    assert!(!svg.contains("class=\"chartlet-grid\""));
    // One visible dot where the line ends; every observation keeps an invisible tooltip.
    assert_eq!(
        svg.matches("class=\"chartlet-point chartlet-point-series-1\"")
            .count(),
        1,
        "{svg}"
    );
    assert_eq!(svg.matches("class=\"chartlet-hit\"").count(), 3, "{svg}");
    let wide = spec.replace("\"width\": 120", "\"width\": 800");
    assert_eq!(
        render_err(&wide),
        ("invalid_dimension", "/width".to_owned())
    );
    let bar = SPEC.replacen('{', "{\"sparkline\": true,", 1);
    assert_eq!(render_err(&bar).0, "option_not_supported");
}

#[test]
fn names_at_the_end_of_the_lines_replace_the_legend() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Two lines", "legend": "end",
            "panes": [{"layers": [
            {"mark": "line", "name": "North", "points": [{"time": "2020", "value": 1}, {"time": "2021", "value": 3}]},
            {"mark": "line", "name": "South", "points": [{"time": "2020", "value": 2}, {"time": "2021", "value": 3}]}]}]}"#;
    let svg = render_ok(spec).content;
    let names: Vec<&str> = svg
        .split("<text ")
        .filter(|text| text.contains("class=\"chartlet-legend\""))
        .collect();
    assert_eq!(names.len(), 2, "{svg}");
    // Both end at 3, so the names move apart.
    let ys: Vec<f64> = names
        .iter()
        .filter_map(|text| text.split("y=\"").nth(1)?.split('"').next()?.parse().ok())
        .collect();
    assert!((ys[0] - ys[1]).abs() >= 14.0, "{ys:?}");
    let bar = SPEC.replacen('{', "{\"legend\": \"end\",", 1);
    assert_eq!(
        render_err(&bar),
        ("option_not_supported", "/legend".to_owned())
    );
}

#[test]
fn an_exact_axis_ends_where_it_says_and_ticks_at_multiples_of_its_step() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Forcing", "panes": [{
            "valueAxis": {"min": -2.5, "max": 3.5, "exact": true, "step": 1},
            "layers": [{"mark": "line", "name": "V", "points": [
                {"time": "2020", "value": -1}, {"time": "2021", "value": 2.2}]}]}]}"#;
    let svg = render_ok(spec).content;
    let ticks: Vec<&str> = svg
        .split("class=\"chartlet-tick\">")
        .skip(1)
        .filter_map(|rest| rest.split('<').next())
        // The time axis writes years; the value ticks are the short ones.
        .filter(|tick| tick.chars().count() <= 2)
        .collect();
    // The ends stay at −2.5 and 3.5; the ticks sit on multiples of the step, zero included.
    assert_eq!(ticks, ["\u{2212}2", "\u{2212}1", "0", "1", "2", "3"]);
    assert!(svg.contains("class=\"chartlet-zero\""));
    let outside = spec.replace("\"value\": 2.2", "\"value\": 4");
    assert_eq!(
        render_err(&outside),
        (
            "value_outside_axis",
            "/panes/0/layers/0/points/1/value".to_owned()
        )
    );
    let half = spec.replace("\"max\": 3.5, ", "");
    assert_eq!(render_err(&half).0, "invalid_axis_range");
    let bars = SPEC.replacen(
        '{',
        "{\"valueAxis\": {\"min\": 5, \"max\": 20, \"exact\": true},",
        1,
    );
    assert_eq!(render_err(&bars).0, "invalid_axis_range");
}

#[test]
fn a_dense_axis_labels_every_few_categories() {
    let data: Vec<String> = (0..43)
        .map(|index| format!("{{\"label\": \"Country {index}\", \"value\": {index}}}"))
        .collect();
    let spec = format!(
        "{{\"schemaVersion\": 1, \"type\": \"bar\", \"title\": \"Dense\", \"width\": 720, \"data\": [{}]}}",
        data.join(",")
    );
    let output = render_ok(&spec);
    let labels = output.content.matches("class=\"chartlet-label\"").count();
    assert!(labels < 43 && labels > 5, "{labels} labels");
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "labels_thinned")
    );
    assert!(
        !output
            .warnings
            .iter()
            .any(|warning| warning.code == "text_truncated")
    );
}

#[test]
fn months_read_as_months_and_the_precision_can_be_set() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Monthly", "panes": [{"layers": [
            {"mark": "line", "name": "V", "points": [{"time": "2026-01", "value": 1},
             {"time": "2026-02", "value": 2}, {"time": "2026-03", "value": 3}]}]}]}"#;
    let svg = render_ok(spec).content;
    assert!(svg.contains("<title>2026-02 – V: 2</title>"), "{svg}");
    let german = render_ok(&spec.replace(
        "\"type\": \"time\",",
        "\"type\": \"time\", \"locale\": \"de\",",
    ));
    assert!(
        german.content.contains("<title>02.2026 – V: 2</title>"),
        "{}",
        german.content
    );
    let mid_year = r#"{"schemaVersion": 1, "type": "time", "title": "Annual",
            "timeAxis": {"precision": "year"}, "panes": [{"layers": [
            {"mark": "line", "name": "V", "points": [{"time": "1950-07-01", "value": 1},
             {"time": "1951-07-01", "value": 2}]}]}]}"#;
    assert!(
        render_ok(mid_year)
            .content
            .contains("<title>1950 – V: 1</title>")
    );
}

#[test]
fn a_step_line_holds_each_value_until_the_next() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Steps", "panes": [{"layers": [
            {"mark": "line", "name": "Mean", "curve": "step",
             "points": [{"time": "2020", "value": 1}, {"time": "2021", "value": 3},
                        {"time": "2022", "value": 2}]}]}]}"#;
    let svg = render_ok(spec).content;
    let points = svg
        .split("<polyline points=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or_default();
    // Three observations and a corner before each of the two jumps.
    assert_eq!(points.split(' ').count(), 5, "{points}");
    let annotation = spec.replace(
        r#"{"mark": "line", "name": "Mean", "curve": "step","#,
        r#"{"mark": "annotation", "label": "A", "value": 1, "curve": "step"}, {"mark": "line", "name": "Mean","#,
    );
    assert_eq!(render_err(&annotation).0, "option_not_supported");
}

#[test]
fn a_numeric_axis_reads_plain_numbers_and_can_run_backwards() {
    let spec = r#"{"schemaVersion": 1, "type": "time", "title": "Profile",
            "timeAxis": {"kind": "number", "reverse": true, "title": "Distance (km)"},
            "panes": [{"layers": [{"mark": "line", "name": "Height",
                "points": [{"time": 0, "value": 10}, {"time": 2.5, "value": 30},
                           {"time": 10, "value": 20}]}]}]}"#;
    let html = html_ok(spec);
    assert!(
        html.contains("Line chart with 3 points from 0.0 to 10.0."),
        "{html}"
    );
    assert!(html.contains("<th scope=\"col\">Distance (km)</th>"));
    let svg = render_ok(spec).content;
    // Reversed: the first position, 0, sits at the right end of the plot.
    let first = svg.find("<title>0.0 – ").map(|_| ()).is_some();
    assert!(first, "{svg}");
    assert_eq!(
        render_err(&spec.replace(
            r#""kind": "number""#,
            r#""kind": "number", "timezone": "+02:00""#
        )),
        ("option_not_supported", "/timeAxis/timezone".to_owned())
    );
    assert_eq!(
        render_err(&spec.replace(r#""time": 2.5"#, r#""time": "2026-01-01""#)).0,
        "invalid_time"
    );
}

#[test]
fn the_mobile_variant_of_small_multiples_takes_its_own_columns() {
    let spec = r#"{"schemaVersion": 1, "type": "multiples", "title": "Panels", "columns": 2,
            "mobile": {"width": 360, "height": 600, "columns": 1}, "panes": [
            {"title": "A", "layers": [{"mark": "line", "name": "V",
                "points": [{"time": "2020", "value": 1}, {"time": "2021", "value": 2}]}]},
            {"title": "B", "layers": [{"mark": "line", "name": "V",
                "points": [{"time": "2020", "value": 3}, {"time": "2021", "value": 4}]}]}]}"#;
    let mobile = render_json(
        spec,
        RenderFormat::Svg,
        &RenderOptions {
            variant: Variant::Mobile,
            ..RenderOptions::default()
        },
    )
    .unwrap()
    .content;
    // One column: both panel titles start at the same x.
    let xs: Vec<&str> = mobile
        .split("<text ")
        .filter(|text| text.contains("class=\"chartlet-panel-title\""))
        .filter_map(|text| text.split('"').nth(1))
        .collect();
    assert_eq!(xs.len(), 2, "{mobile}");
    assert_eq!(xs[0], xs[1]);
    let bar = SPEC.replacen('{', "{\"mobile\": {\"width\": 360, \"columns\": 1},", 1);
    assert_eq!(
        render_err(&bar),
        ("option_not_supported", "/mobile/columns".to_owned())
    );
}

#[test]
fn panels_on_axes_of_their_own_are_not_compared() {
    let spec = r#"{"schemaVersion": 1, "type": "multiples", "title": "Two units",
            "independentAxes": true, "panes": [
            {"title": "Small", "layers": [{"mark": "line", "name": "V",
                "points": [{"time": "2020", "value": 1}, {"time": "2021", "value": 2}]}]},
            {"title": "Large", "layers": [{"mark": "line", "name": "V",
                "points": [{"time": "2020", "value": 1000}, {"time": "2021", "value": 2000}]}]}]}"#;
    let svg = render_ok(spec).content;
    assert!(svg.contains("with a value axis of their own"), "{svg}");
    assert!(!svg.contains("Highest:"));
    assert!(
        svg.contains(">2.0</text>") || svg.contains(">2</text>"),
        "{svg}"
    );
}

#[test]
fn a_logarithmic_axis_takes_positive_values_only() {
    let log = SPEC.replacen('{', "{\"valueAxis\": {\"scale\": \"log\"},", 1);
    // SPEC has a negative value.
    assert_eq!(render_err(&log).0, "invalid_value");
    let positive = log.replace("-4", "4");
    let hooked = render_json(
        &positive.replace("\"type\": \"bar\"", "\"type\": \"line\""),
        RenderFormat::Svg,
        &RenderOptions {
            hooks: true,
            ..RenderOptions::default()
        },
    )
    .unwrap();
    assert!(hooked.content.contains(" data-y-scale=\"log\""));
    let svg = render_ok(&positive).content;
    assert!(
        svg.contains(">10</text>") || svg.contains(">1</text>"),
        "{svg}"
    );
}

#[test]
fn single_series_output_carries_no_series_styles() {
    let output = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(!output.content.contains("chartlet-series-"));
}

#[test]
fn short_negative_bar_keeps_its_label_readable() {
    let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "orientation": "horizontal",
            "title": "Change",
            "valueAxis": {"format": "percent"},
            "data": [
                {"label": "A", "value": 0.3},
                {"label": "B", "value": -0.005}
            ]
        }"#;
    let output = render_json(specification, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(
        output
            .content
            .contains("text-anchor=\"end\" class=\"chartlet-value\">−0.5%</text>")
    );
    assert!(!output.content.contains("class=\"chartlet-value-inverse\""));
}

#[test]
fn bar_chart_rejects_missing_values() {
    let missing = SPEC.replace("12}", "null}");
    let error = render_json(&missing, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "missing_bar_value");
    assert_eq!(error.path, "/data/0/value");
}

#[test]
fn grouped_bars_render_series_filter_checkboxes() {
    let output = render_json(GROUPED, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(
        output
            .content
            .contains("<fieldset class=\"chartlet-filter\">")
    );
    assert!(output.content.contains("class=\"series-0\" checked"));
    assert!(output.content.contains("data-series=\"0\""));
    assert!(output.content.contains("<text data-series=\"0\""));
    assert!(output.content.contains("<title>Jan – Budget: 120</title>"));
}

#[test]
fn single_series_bars_carry_native_tooltips() {
    let output = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(
        output
            .content
            .contains("<title>North &lt;East&gt;: 12</title>")
    );
    assert!(!output.content.contains("data-series"));
}

#[test]
fn zoom_steps_render_radio_selectable_panels() {
    let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "title": "Quarterly revenue",
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
    let output = render_json(specification, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(
        output
            .content
            .contains("<fieldset class=\"chartlet-zoom\">")
    );
    assert!(
        output
            .content
            .contains("class=\"chartlet-panel chartlet-panel-0\"")
    );
    assert!(
        output
            .content
            .contains("class=\"chartlet-panel chartlet-panel-1\"")
    );
    assert!(output.content.contains("class=\"zoom-0\" checked"));
    // Every panel gets its own non-colliding accessibility IDs.
    assert_eq!(output.content.matches("<title id=").count(), 2);
}

#[test]
fn a_single_zoom_step_is_rejected() {
    let specification = r#"{
            "schemaVersion": 1,
            "type": "bar",
            "title": "Quarterly revenue",
            "data": [
                {"label": "Q1", "value": 320},
                {"label": "Q2", "value": 345}
            ],
            "zoomSteps": [
                {"label": "All", "from": 0, "to": 1}
            ]
        }"#;
    let error = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
        .expect_err("one zoom step cannot provide a choice");
    assert_eq!(error.code, "not_enough_zoom_steps");
    assert_eq!(error.path, "/zoomSteps");
}

#[test]
fn a_thin_line_is_drawn_thin_in_the_plot_and_in_the_legend() {
    let spec = TIME.replace(
        "\"name\": \"Orders\",",
        "\"name\": \"Orders\", \"stroke\": \"thin\",",
    ).replace(
        "]\n            }\n        ]",
        ", {\"mark\": \"line\", \"name\": \"Mean\", \"points\": [{\"time\": \"2026-03-01\", \"value\": 12}, {\"time\": \"2026-03-03\", \"value\": 18}]}]\n            }\n        ]",
    );
    let svg = render_ok(&spec).content;
    assert!(svg.contains(".chartlet-line-thin{stroke-width:1}"));
    // The plotted line and its legend sample.
    assert_eq!(
        svg.matches("class=\"chartlet-line chartlet-line-thin chartlet-line-series-1\"")
            .count(),
        2
    );
    assert_eq!(
        svg.matches("class=\"chartlet-line chartlet-line-series-2\"")
            .count(),
        2
    );
    let rule = TIME.replace(
        "\"name\": \"Orders\",",
        "\"name\": \"Orders\", \"points\": [{\"time\": \"2026-03-01\", \"value\": 1}, {\"time\": \"2026-03-02\", \"value\": 2}]}, {\"mark\": \"annotation\", \"value\": 1, \"label\": \"Limit\", \"stroke\": \"thin\"}, {\"mark\": \"line\", \"name\": \"Other\",",
    );
    assert_eq!(
        render_err(&rule),
        (
            "option_not_supported",
            "/panes/0/layers/1/stroke".to_owned()
        )
    );
}

#[test]
fn bars_named_by_group_get_one_color_and_legend_entry_per_group() {
    let json = r#"{"schemaVersion": 1, "type": "bar", "title": "Benefit and harm", "width": 600, "height": 300,
            "data": [{"label": "A", "value": 5, "group": "Benefit"}, {"label": "B", "value": 4, "group": "Harm"},
                     {"label": "C", "value": 3, "group": "Benefit"}]}"#;
    let output = render_ok(json);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert!(svg.contains("chartlet-series-1") && svg.contains("chartlet-series-2"));
    assert!(svg.contains(">Benefit</text>") && svg.contains(">Harm</text>"));
    assert!(svg.contains("Colors show groups: Benefit: A, C; Harm: B."));
    // The data table keeps the chart as written: one value per category, and its group.
    let html = render_json(json, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(html.content.contains(
        "<th scope=\"col\">Category</th><th scope=\"col\">Value</th><th scope=\"col\">Group</th>"
    ));
    assert!(
        html.content
            .contains("<th scope=\"row\">B</th><td>4</td><td>Harm</td>")
    );
    assert!(!html.content.contains("Missing"));
}

#[test]
fn groups_are_all_or_none_at_most_four_and_for_bars() {
    let base = |data: &str, kind: &str| {
        format!(
            r#"{{"schemaVersion": 1, "type": "{kind}", "title": "T", "width": 600, "height": 300, "data": [{data}]}}"#
        )
    };
    for (data, kind, code, path) in [
        (
            r#"{"label": "A", "value": 1, "group": "x"}, {"label": "B", "value": 2}"#,
            "bar",
            "missing_group",
            "/data/1/group",
        ),
        (
            r#"{"label": "A", "value": 1, "group": "a"}, {"label": "B", "value": 2, "group": "b"}, {"label": "C", "value": 3, "group": "c"}, {"label": "D", "value": 4, "group": "d"}, {"label": "E", "value": 5, "group": "e"}"#,
            "bar",
            "too_many_series",
            "/data/4/group",
        ),
        (
            r#"{"label": "A", "value": 1, "group": "x"}, {"label": "B", "value": 2, "group": "y"}"#,
            "line",
            "option_not_supported",
            "/data/0/group",
        ),
    ] {
        let error = render_json(
            &base(data, kind),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{data}");
    }
}

const WATERFALL: &str = r#"{"schemaVersion": 1, "type": "waterfall", "title": "Profit", "width": 700, "height": 360,
        "waterfall": {"steps": [
            {"label": "Revenue", "value": 100, "kind": "start"},
            {"label": "Costs", "value": -40},
            {"label": "Other", "value": 15},
            {"label": "Profit", "kind": "total"}
        ]}}"#;

#[test]
fn a_waterfall_carries_the_running_total_from_bar_to_bar() {
    for orientation in ["vertical", "horizontal"] {
        let json = WATERFALL.replace(
            "\"waterfall\": {",
            &format!("\"orientation\": \"{orientation}\", \"waterfall\": {{"),
        );
        let output = render_ok(&json);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        let svg = &output.content;
        assert_eq!(
            svg.matches("class=\"chartlet-wf-total\"").count(),
            2,
            "{orientation}"
        );
        assert_eq!(svg.matches("class=\"chartlet-wf-down\"").count(), 1);
        assert_eq!(svg.matches("class=\"chartlet-wf-up\"").count(), 1);
        assert_eq!(svg.matches("class=\"chartlet-wf-link\"").count(), 3);
        assert!(svg.contains("<title>Costs: −40, total 60</title>"));
        assert!(svg.contains(">+15</text>"));
    }
    let html = render_json(WATERFALL, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("Waterfall with 4 steps, ending at 75. Biggest rise: Other (+15). Biggest fall: Costs (−40)."));
    assert!(html.contains("<th scope=\"row\">Profit</th><td>Total</td><td>75</td><td>75</td>"));
}

#[test]
fn waterfall_steps_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""value": -40"#,
            r#""kind": "delta""#,
            "missing_value",
            "/waterfall/steps/1/value",
        ),
        (
            r#""label": "Profit", "kind": "total""#,
            r#""label": "Profit", "kind": "total", "value": 80"#,
            "total_mismatch",
            "/waterfall/steps/3/value",
        ),
        (
            r#""label": "Costs""#,
            r#""label": "Revenue""#,
            "duplicate_label",
            "/waterfall/steps/1/label",
        ),
    ] {
        let error = render_json(
            &WATERFALL.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}

#[test]
fn a_stream_centers_its_areas_and_shows_thickness() {
    let spec = include_str!("../../examples/generation-mix.json")
        .replace("\"stack\": \"normal\"", "\"stack\": \"stream\"")
        .replace("\"legend\": \"end\",", "");
    let options = RenderOptions {
        hooks: true,
        ..RenderOptions::default()
    };
    let output = render_json(&spec, RenderFormat::Html, &options).expect("renders");
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let html = &output.content;
    assert!(html.contains("Streamgraph, from the bottom up: Fossil, Nuclear"));
    assert!(!html.contains("the top edge shows their total"));
    // The hooks tell the interactive module that the stack is centered; the table keeps each
    // layer's own value.
    assert_eq!(html.matches(" data-stream=\"\"").count(), 4);
    assert!(html.contains("Wind and solar: 760"));
    // The areas are named at their ends without asking for it, and the value axis is a scale
    // bar: three strokes and one label instead of a grid of ticks.
    let svg = render_ok(&spec).content;
    assert_eq!(svg.matches("class=\"chartlet-legend\"").count(), 4);
    assert_eq!(svg.matches("class=\"chartlet-zero\"").count(), 3);

    // The stack is centered: at every time its lower edge lies as far below zero as its upper
    // edge lies above it.
    let parsed: ChartSpec = serde_json::from_str(&spec).expect("parses");
    let zone = parsed.time_zone().unwrap_or_default();
    let entries: Vec<_> = parsed.data_layers().collect();
    let bottom = parsed.stack_base(entries[0], zone).expect("stacked");
    let top = parsed.drawn_points(*entries.last().unwrap(), zone);
    for (low, (_, high)) in bottom.iter().zip(&top) {
        assert!((low + high).abs() < 1e-9, "{low} and {high}");
    }
    assert!((top[4].1 - 2920.0 / 2.0).abs() < 1e-9);
}

#[test]
fn a_stream_is_validated_by_name() {
    let stream = include_str!("../../examples/generation-mix.json")
        .replace("\"stack\": \"normal\"", "\"stack\": \"stream\"");
    for (from, to, path) in [
        (
            "\"thousandsSeparator\": true",
            "\"thousandsSeparator\": true, \"min\": -100",
            "/panes/0/valueAxis/min",
        ),
        (
            "\"thousandsSeparator\": true",
            "\"thousandsSeparator\": true, \"reverse\": true",
            "/panes/0/valueAxis/reverse",
        ),
        (
            "\"mark\": \"area\",\n          \"name\": \"Nuclear\"",
            "\"mark\": \"line\",\n          \"name\": \"Nuclear\"",
            "/panes/0/layers/1/mark",
        ),
    ] {
        assert_eq!(
            render_err(&stream.replacen(from, to, 1)),
            ("option_not_supported", path.to_owned()),
            "{to}"
        );
    }
    let bar = r#"{"schemaVersion": 1, "type": "bar", "title": "T", "stack": "stream",
        "categories": ["a", "b"], "series": [{"name": "x", "values": [1, 2]}, {"name": "y", "values": [3, 4]}]}"#;
    assert_eq!(
        render_err(bar),
        ("option_not_supported", "/stack".to_owned())
    );
}
