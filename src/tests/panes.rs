use super::*;

/// Three candles: rising, falling, and one that opens and closes alike.
const CANDLES: &str = r#"[
        {"time": "2026-03-02", "open": 10, "high": 12, "low": 9, "close": 11},
        {"time": "2026-03-03", "open": 11, "high": 11.5, "low": 8, "close": 9},
        {"time": "2026-03-04", "open": 9, "high": 10, "low": 8.5, "close": 9}
    ]"#;

/// A time chart whose panes are given as JSON; `extra` goes into the top level.
fn stacked(panes: &str, extra: &str) -> String {
    format!(
        r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Share price",
                "timeAxis": {{"title": "Day"}}{extra},
                "panes": [{panes}]
            }}"#
    )
}

fn candle_pane(extra_layers: &str) -> String {
    format!(
        r#"{{"heightRatio": 3, "valueAxis": {{"title": "Price", "decimals": 2}}, "layers": [{{"mark": "ohlc", "name": "Price", "data": {CANDLES}}}{extra_layers}]}}"#
    )
}

const VOLUME_PANE: &str = r#"{"valueAxis": {"title": "Volume", "format": "percent"}, "layers": [{"mark": "area", "name": "Volume", "points": [{"time": "2026-03-02", "value": 0.12}, {"time": "2026-03-03", "value": 0.2}, {"time": "2026-03-04", "value": 0.08}]}]}"#;

#[test]
fn candles_differ_in_shape_and_color_by_direction() {
    let output = render_ok(&stacked(&candle_pane(""), ""));
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = output.content;
    // A rising candle is hollow, a falling one filled; the legend sample shows both.
    assert_eq!(
        svg.matches("class=\"chartlet-candle chartlet-candle-rise\"")
            .count(),
        3,
        "two rising or level candles and the legend sample"
    );
    assert_eq!(
        svg.matches("class=\"chartlet-candle chartlet-candle-fall\"")
            .count(),
        2
    );
    assert!(svg.contains(
        ".chartlet-candle-rise{fill:var(--chartlet-background);stroke:var(--chartlet-rise)}"
    ));
    assert!(svg.contains(".chartlet-theme-dark{--chartlet-rise:"));
    assert!(svg.contains("Price (hollow: rising, filled: falling)</text>"));
    assert!(svg.contains(
        "<title>2026-03-02 – Price: open 10.00, high 12.00, low 9.00, close 11.00</title>"
    ));
    // A level candle keeps a body one pixel high.
    assert!(svg.contains(" height=\"1\" class=\"chartlet-candle chartlet-candle-rise\""));
    assert!(svg.contains(
        "opened at 10.00 on 2026-03-02 and closed at 9.00 on 2026-03-04, a change of \u{2212}1.00 (\u{2212}10.0%); high 12.00 on 2026-03-02, low 8.00 on 2026-03-03."
    ), "{svg}");
    let html = html_ok(&stacked(&candle_pane(""), ""));
    assert!(html.contains("<th scope=\"col\">Price (open)</th><th scope=\"col\">Price (high)</th><th scope=\"col\">Price (low)</th><th scope=\"col\">Price (close)</th>"));
    assert!(html.contains("<td>11.00</td><td>11.50</td><td>8.00</td><td>9.00</td>"));
    // Charts without candles carry none of their styles.
    assert!(!render_ok(TIME).content.contains("chartlet-rise"));
}

#[test]
fn dense_candles_are_drawn_as_wicks_only() {
    // 704 plot pixels at the default width leave room for 234 candles.
    let data = (0..240)
        .map(|index| {
            format!(
                r#"{{"time": {}, "open": 10, "high": 12, "low": 9, "close": 11}}"#,
                1_770_000_000 + index * 86_400
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let output = render_ok(&stacked(
        &format!(r#"{{"layers": [{{"mark": "ohlc", "data": [{data}]}}]}}"#),
        "",
    ));
    assert_eq!(output.warnings.len(), 1, "{:?}", output.warnings);
    assert_eq!(output.warnings[0].code, "dense_chart");
    assert_eq!(output.warnings[0].path, "/panes/0/layers/0/data");
    let svg = output.content;
    assert_eq!(
        svg.matches("class=\"chartlet-wick chartlet-wick-rise\"")
            .count(),
        241
    );
    // Only the legend sample keeps its bodies.
    assert_eq!(
        svg.matches("class=\"chartlet-candle chartlet-candle-rise\"")
            .count(),
        1
    );
}

/// The left edge and the width of every zone of a rendering, in document order.
fn zone_spans(svg: &str) -> Vec<(f64, f64)> {
    svg.split("<rect ")
        .skip(1)
        .filter_map(|rest| rest.split('>').next())
        .filter(|tag| tag.contains("class=\"chartlet-zone"))
        .map(|tag| {
            let attribute = |name: &str| -> f64 {
                tag.split(&format!("{name}=\""))
                    .nth(1)
                    .and_then(|rest| rest.split('"').next())
                    .and_then(|value| value.parse().ok())
                    .expect("a numeric attribute")
            };
            (attribute("x"), attribute("width"))
        })
        .collect()
}

/// The horizontal position of every wick in the plot, in order; the legend sample comes first
/// in the document and is left out.
fn wick_xs(svg: &str) -> Vec<f64> {
    svg.split("<line x1=\"")
        .skip(1)
        .filter(|rest| {
            rest.split('>')
                .next()
                .is_some_and(|tag| tag.contains("chartlet-wick"))
        })
        .map(|rest| rest[..rest.find('"').unwrap()].parse().unwrap())
        .skip(2)
        .collect()
}

#[test]
fn a_zone_covers_the_candles_at_its_edges_in_full() {
    let zones = r#", {"mark": "band", "label": "Hammer", "from": "2026-03-03", "to": "2026-03-03"},
        {"mark": "band", "label": "Two days", "from": "2026-03-03", "to": "2026-03-04"}"#;
    let output = render_ok(&stacked(&candle_pane(zones), ""));
    let svg = &output.content;
    let xs = wick_xs(svg);
    assert_eq!(xs.len(), 3);
    let step = xs[1] - xs[0];
    let spans = zone_spans(svg);
    // One candle: half a candle's room on either side of its middle.
    assert!(
        (spans[0].0 - (xs[1] - step / 2.0)).abs() < 0.01,
        "{spans:?}"
    );
    assert!((spans[0].1 - step).abs() < 0.01, "{spans:?}");
    // Two candles, the second at the end of the axis: the zone stops at the edge of the plot,
    // which lies 6 pixels beyond the last candle.
    assert!(
        (spans[1].0 - (xs[1] - step / 2.0)).abs() < 0.01,
        "{spans:?}"
    );
    assert!(
        (spans[1].0 + spans[1].1 - (xs[2] + 6.0)).abs() < 0.01,
        "{spans:?}"
    );
    // Tooltip and description keep the times as given: one time for one candle.
    assert!(svg.contains("<title>Hammer: 2026-03-03</title>"));
    assert!(svg.contains("<title>Two days: 2026-03-03 to 2026-03-04</title>"));
    assert!(svg.contains("Zones: Hammer, 2026-03-03; Two days, 2026-03-03 to 2026-03-04."));
}

#[test]
fn a_zone_follows_the_candles_of_the_chart_in_every_pane() {
    // The volume pane has no candles of its own; its zone still lines up with the one above.
    let zone = r#"{"mark": "band", "label": "Day", "from": "2026-03-03", "to": "2026-03-03"}"#;
    let volume = VOLUME_PANE.replace("\"layers\": [", &format!("\"layers\": [{zone}, "));
    let svg = render_ok(&stacked(
        &format!("{}, {volume}", candle_pane(&format!(", {zone}"))),
        "",
    ))
    .content;
    let spans = zone_spans(&svg);
    assert_eq!(spans.len(), 2);
    assert!((spans[0].0 - spans[1].0).abs() < 0.01, "{spans:?}");
    assert!((spans[0].1 - spans[1].1).abs() < 0.01, "{spans:?}");
    assert!(spans[0].1 > 1.0, "{spans:?}");
}

#[test]
fn a_zone_over_one_candle_keeps_its_slot_with_collapsed_gaps() {
    // Friday, then Monday and Tuesday: the weekend takes no room.
    let data = r#"[
        {"time": "2026-03-06", "open": 10, "high": 12, "low": 9, "close": 11},
        {"time": "2026-03-09", "open": 11, "high": 11.5, "low": 8, "close": 9},
        {"time": "2026-03-10", "open": 9, "high": 10, "low": 8.5, "close": 9.5}
    ]"#;
    let pane = format!(
        r#"{{"layers": [{{"mark": "ohlc", "data": {data}}}, {{"mark": "band", "label": "Monday", "from": "2026-03-09", "to": "2026-03-09"}}]}}"#
    );
    let spec = stacked(&pane, "").replace(
        r#""timeAxis": {"title": "Day"}"#,
        r#""timeAxis": {"gaps": "collapse"}"#,
    );
    let svg = render_ok(&spec).content;
    let xs = wick_xs(&svg);
    let step = xs[1] - xs[0];
    assert!((xs[2] - xs[1] - step).abs() < 0.01);
    let spans = zone_spans(&svg);
    assert!(
        (spans[0].0 - (xs[1] - step / 2.0)).abs() < 0.01,
        "{spans:?}"
    );
    assert!((spans[0].1 - step).abs() < 0.01, "{spans:?}");
    // A zone that encloses no candle still has no slot.
    assert_eq!(
        render_err(&spec.replace(
            r#""from": "2026-03-09", "to": "2026-03-09""#,
            r#""from": "2026-03-07", "to": "2026-03-07""#
        )),
        ("time_out_of_range", "/panes/0/layers/1".to_owned())
    );
}

#[test]
fn a_zone_at_one_time_needs_candles() {
    let zone = r#"{"mark": "band", "label": "Z", "from": "2026-03-02", "to": "2026-03-02"}"#;
    assert_eq!(
        render_err(&with_rules(zone)),
        ("invalid_band", "/panes/0/layers/0/to".to_owned())
    );
    // The other way round is refused among candles too.
    let reversed = r#", {"mark": "band", "label": "Z", "from": "2026-03-04", "to": "2026-03-03"}"#;
    assert_eq!(
        render_err(&stacked(&candle_pane(reversed), "")),
        ("invalid_band", "/panes/0/layers/1/to".to_owned())
    );
}

#[test]
fn a_zoom_window_keeps_the_zone_over_the_candle_at_its_edge() {
    let zone = r#", {"mark": "band", "label": "Middle", "from": "2026-03-03", "to": "2026-03-03"}"#;
    let zoom = r#", "zoomSteps": [
            {"label": "Early", "from": "2026-03-02", "to": "2026-03-03"},
            {"label": "Late", "from": "2026-03-03", "to": "2026-03-04"}
        ]"#;
    let html = html_ok(&stacked(&candle_pane(zone), zoom));
    // The marked candle is the last of one window and the first of the other.
    assert_eq!(html.matches("class=\"chartlet-zone\"").count(), 2);
}

#[test]
fn one_candlestick_layer_does_not_count_against_the_data_layers_of_its_pane() {
    let lines = |count: usize| {
        (0..count)
            .map(|index| {
                format!(
                    r##", {{"mark": "line", "name": "L{index}", "color": "#2563eb", "points": [{{"time": "2026-03-02", "value": 9}}, {{"time": "2026-03-04", "value": 1{index}}}]}}"##
                )
            })
            .collect::<String>()
    };
    // Candles with six lines over them, such as three bands and three averages.
    render_ok(&stacked(&candle_pane(&lines(6)), ""));
    assert_eq!(
        render_err(&stacked(&candle_pane(&lines(7)), "")),
        ("too_many_layers", "/panes/0/layers".to_owned())
    );
    // A second candlestick layer is one more data layer like any other.
    let second = format!(
        r#", {{"mark": "ohlc", "name": "Other", "data": {CANDLES}}}{}"#,
        lines(6)
    );
    assert_eq!(
        render_err(&stacked(&candle_pane(&second), "")),
        ("too_many_layers", "/panes/0/layers".to_owned())
    );
}

#[test]
fn invalid_candles_are_refused_with_their_path() {
    let refuse = |data: &str, layer: &str| {
        render_err(&stacked(
            &format!(r#"{{"layers": [{{"mark": "ohlc", "data": {data}{layer}}}]}}"#),
            "",
        ))
    };
    let valid = CANDLES;
    assert_eq!(
        refuse(&valid.replace("\"low\": 8,", "\"low\": 9.5,"), ""),
        ("invalid_candle", "/panes/0/layers/0/data/1/low".to_owned())
    );
    assert_eq!(
        refuse(&valid.replace("\"high\": 12,", "\"high\": 10.5,"), ""),
        ("invalid_candle", "/panes/0/layers/0/data/0/high".to_owned())
    );
    assert_eq!(
        refuse(&valid.replace("2026-03-03", "2026-03-02"), ""),
        ("unordered_time", "/panes/0/layers/0/data/1/time".to_owned())
    );
    assert_eq!(
        refuse(&valid.replace("\"open\": 10,", "\"open\": 1e101,"), ""),
        (
            "unsupported_numeric_range",
            "/panes/0/layers/0/data/0/open".to_owned()
        )
    );
    assert_eq!(
        refuse(
            r#"[{"time": "2026-03-02", "open": 1, "high": 2, "low": 0, "close": 1}]"#,
            ""
        ),
        ("empty_series", "/panes/0/layers/0/data".to_owned())
    );
    assert_eq!(
        refuse(valid, r##", "color": "#123""##),
        ("option_not_supported", "/panes/0/layers/0/color".to_owned())
    );
    assert_eq!(
        refuse(valid, r#", "points": [{"time": "2026-03-02", "value": 1}]"#),
        (
            "option_not_supported",
            "/panes/0/layers/0/points".to_owned()
        )
    );
    assert_eq!(
        render_err(&multiples(&format!(
            r#"{{"title": "A", "layers": [{{"mark": "ohlc", "data": {valid}}}]}}, {}"#,
            panel("B", 1.0)
        ))),
        ("option_not_supported", "/panes/0/layers/0/mark".to_owned())
    );
}

#[test]
fn panes_stack_by_their_ratio_and_share_one_time_axis_at_the_bottom() {
    let svg = render_ok(&stacked(&format!("{}, {VOLUME_PANE}", candle_pane("")), "")).content;
    // 450 high: the plot runs from 102 (title and one legend row) to 394 (time axis title).
    // Less the gap of 36, the ratio 3:1 splits 256 pixels into 192 and 64.
    let titles = placed_texts(&svg, "chartlet-axis-title");
    let at = |name: &str| {
        titles
            .iter()
            .find(|(_, _, content)| content == name)
            .map(|(_, y, _)| *y)
            .expect("the axis title is drawn")
    };
    assert!((at("Price") - 82.0).abs() < 1e-9);
    assert!((at("Volume") - 310.0).abs() < 1e-9);
    // Time tick labels sit only below the bottom pane; value ticks at the left of both.
    let ticks = placed_texts(&svg, "chartlet-tick");
    let time_ticks: Vec<_> = ticks.iter().filter(|(x, _, _)| *x > 72.0).collect();
    assert_ne!(time_ticks.len(), 0);
    assert!(
        time_ticks.iter().all(|(_, y, _)| (*y - 418.0).abs() < 1e-9),
        "{time_ticks:?}"
    );
    assert!(
        ticks
            .iter()
            .any(|(x, _, content)| *x <= 72.0 && !content.ends_with('%'))
    );
    assert!(ticks.iter().any(|(_, _, content)| content.ends_with('%')));
    // Vertical grid lines run through both panes, each within its own.
    assert!(svg.contains("y1=\"102\" x2="));
    assert!(svg.contains("y1=\"330\" x2="));
    assert!(svg.contains("y2=\"394\" class=\"chartlet-grid\""));
    // The panes share one legend, and the area of the second pane takes the next palette
    // color, since candles take none.
    assert!(svg.contains("Volume</text>"));
    assert!(svg.contains("chartlet-area chartlet-band-series-1"));
    assert!(svg.contains(" 2 stacked panes on one time axis: Price, Volume."));
    assert!(
        svg.contains(
            " Volume – Highest: 20% (Volume in 2026-03-03). Lowest: 8% (Volume in 2026-03-04)."
        ),
        "{svg}"
    );
}

#[test]
fn every_pane_writes_its_values_by_its_own_axis() {
    let html = html_ok(&stacked(
        &format!(
            "{}, {VOLUME_PANE}",
            candle_pane(
                r#", {"mark": "line", "name": "Average", "points": [{"time": "2026-03-02", "value": 10.5}, {"time": "2026-03-04", "value": 10}]}, {"mark": "annotation", "label": "Target", "value": 11}"#
            )
        ),
        "",
    ));
    assert!(html.contains("<td>10.50</td>"), "{html}");
    assert!(html.contains("<td>12%</td>"));
    assert!(html.contains("Reference lines: Target at 11.00."));
    assert!(
        html.contains("chartlet-line-series-1"),
        "the line takes the first palette color"
    );
}

#[test]
fn pane_limits_are_refused_by_name() {
    let line = |name: &str| {
        format!(
            r#"{{"layers": [{{"mark": "line", "name": "{name}", "points": [{{"time": "2026-03-02", "value": 1}}, {{"time": "2026-03-03", "value": 2}}]}}]}}"#
        )
    };
    let five = (0..5)
        .map(|index| line(&format!("L{index}")))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        render_err(&stacked(&five, "")),
        ("too_many_panes", "/panes".to_owned())
    );
    let four = (0..4)
        .map(|index| line(&format!("L{index}")))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(render_ok(&stacked(&four, "")).warnings, []);
    let tall = format!(
        "{}, {}",
        line("A"),
        line("B").replacen('{', r#"{"heightRatio": 11, "#, 1)
    );
    assert_eq!(
        render_err(&stacked(&tall, "")),
        ("invalid_height_ratio", "/panes/1/heightRatio".to_owned())
    );
    assert_eq!(
        render_err(&stacked(&format!("{}, {}", line("A"), line("A")), "")),
        ("duplicate_series", "/panes/1/layers/0/name".to_owned())
    );
    let anonymous = line("A").replace(r#""name": "A", "#, "");
    assert_eq!(
        render_err(&stacked(&format!("{anonymous}, {}", line("B")), "")),
        ("missing_name", "/panes/0/layers/0/name".to_owned())
    );
    let two = r#"{"layers": [{"mark": "line", "name": "X", "points": [{"time": "2026-03-02", "value": 1}, {"time": "2026-03-03", "value": 2}]}, {"mark": "line", "name": "Y", "points": [{"time": "2026-03-02", "value": 1}, {"time": "2026-03-03", "value": 2}]}]}"#;
    assert_eq!(
        render_err(&stacked(
            &format!("{four}, {two}")
                .replacen(&line("L3"), "", 1)
                .replace(",,", ","),
            ""
        )),
        ("too_many_layers", "/panes/3/layers/1/color".to_owned())
    );
    let late = line("Late")
        .replace("2026-03-02", "2026-03-10")
        .replace("2026-03-03", "2026-03-11");
    let zoomed = stacked(
        &format!("{}, {late}", line("Early")),
        r#", "zoomSteps": [{"label": "All", "from": "2026-03-01", "to": "2026-03-12"}, {"label": "Early", "from": "2026-03-01", "to": "2026-03-04"}]"#,
    );
    assert_eq!(
        render_err(&zoomed),
        ("zoom_out_of_range", "/zoomSteps/1".to_owned())
    );
    let squeezed = stacked(&tall.replace("11", "10"), r#", "height": 240"#);
    let output = render_ok(&squeezed);
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "dense_chart" && warning.path == "/panes/0/heightRatio"),
        "{:?}",
        output.warnings
    );
}

#[test]
fn german_candles_and_panes_write_german_texts() {
    let spec = stacked(
        &format!("{}, {VOLUME_PANE}", candle_pane("")),
        r#", "locale": "de""#,
    );
    let html = html_ok(&spec);
    assert!(html.contains("Price (hohl: steigend, gefüllt: fallend)</text>"));
    assert!(html.contains(
        "<title>02.03.2026 – Price: Eröffnung 10,00; Hoch 12,00; Tief 9,00; Schluss 11,00</title>"
    ));
    assert!(
        html.contains(" 2 übereinanderliegende Teildiagramme auf einer Zeitachse: Price, Volume.")
    );
    assert!(html.contains(
        "Price: Eröffnung 10,00 am 02.03.2026, Schluss 9,00 am 04.03.2026, Veränderung \u{2212}1,00 (\u{2212}10,0\u{202f}%); Hoch 12,00 am 02.03.2026, Tief 8,00 am 03.03.2026."
    ), "{html}");
    assert!(html.contains("<th scope=\"col\">Price (Eröffnung)</th><th scope=\"col\">Price (Hoch)</th><th scope=\"col\">Price (Tief)</th><th scope=\"col\">Price (Schluss)</th>"));
    assert!(html.contains("<td>12\u{202f}%</td>"));
}
