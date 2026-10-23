use super::*;

/// Thursday 2026-03-05 to Wednesday 2026-03-11 without the weekend, as Unix seconds.
const TRADING_DAYS: [i64; 5] = [
    1_772_668_800,
    1_772_755_200,
    1_773_014_400,
    1_773_100_800,
    1_773_187_200,
];

/// A time chart with gaps collapsed: one line over `days`, then `extra` layers, then `tail`
/// at the top level.
fn collapsed(days: &[i64], extra: &str, tail: &str) -> String {
    let points = days
        .iter()
        .enumerate()
        .map(|(index, day)| format!(r#"{{"time": {day}, "value": {}}}"#, index + 1))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{
                "schemaVersion": 1,
                "type": "time",
                "title": "Trading days",
                "showValues": false,
                "timeAxis": {{"gaps": "collapse"}},
                "panes": [{{"layers": [{{"mark": "line", "points": [{points}]}}{extra}]}}]{tail}
            }}"#
    )
}

/// The horizontal centers of the line's point markers, in order.
fn point_xs(svg: &str) -> Vec<f64> {
    svg.split("<circle cx=\"")
        .skip(1)
        .map(|rest| rest[..rest.find('"').unwrap()].parse().unwrap())
        .collect()
}

/// The labels and positions of the time ticks, in order.
fn time_ticks(svg: &str) -> Vec<(String, f64)> {
    svg.split("<text x=\"")
        .skip(1)
        .filter(|rest| rest.contains("class=\"chartlet-tick\">2026"))
        .map(|rest| {
            let x = rest[..rest.find('"').unwrap()].parse().unwrap();
            let label = rest.split("class=\"chartlet-tick\">").nth(1).unwrap();
            (label[..label.find('<').unwrap()].to_owned(), x)
        })
        .collect()
}

#[test]
fn collapsed_gaps_place_observations_at_equal_distances() {
    let svg = render_ok(&collapsed(&TRADING_DAYS, "", "")).content;
    let xs = point_xs(&svg);
    assert_eq!(xs.len(), 5);
    let step = xs[1] - xs[0];
    assert!(step > 0.0);
    for pair in xs.windows(2) {
        assert!((pair[1] - pair[0] - step).abs() < 0.01, "{xs:?}");
    }
    // The weekend takes no space: Friday and Monday are neighbours.
    assert!(svg.contains("<title>2026-03-06: 2</title>"));
    assert!(svg.contains("<title>2026-03-09: 3</title>"));
    // Daily ticks: the weekend's boundaries fall on Monday, which is labelled once.
    let labels: Vec<String> = time_ticks(&svg)
        .into_iter()
        .map(|(label, _)| label)
        .collect();
    assert_eq!(
        labels,
        [
            "2026-03-05",
            "2026-03-06",
            "2026-03-09",
            "2026-03-10",
            "2026-03-11"
        ]
    );

    // The same data with gaps shown leaves room for the weekend.
    let shown = render_ok(&collapsed(&TRADING_DAYS, "", "").replace("collapse", "show")).content;
    let xs = point_xs(&shown);
    assert!(xs[2] - xs[1] > 2.0 * (xs[1] - xs[0]));
}

#[test]
fn collapsed_ticks_sit_on_the_first_observation_after_their_boundary() {
    // Five weeks of weekdays from Monday 2026-03-02, without Monday 2026-03-16.
    let monday: i64 = 1_772_409_600;
    let days: Vec<i64> = (0..33)
        .map(|day| monday + day * 86_400)
        .filter(|epoch| (epoch / 86_400 + 3).rem_euclid(7) < 5)
        .filter(|epoch| *epoch != monday + 14 * 86_400)
        .collect();
    let svg = render_ok(&collapsed(&days, "", "")).content;
    let ticks = time_ticks(&svg);
    let labels: Vec<&str> = ticks.iter().map(|(label, _)| label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "2026-03-02",
            "2026-03-09",
            "2026-03-17",
            "2026-03-23",
            "2026-03-30"
        ]
    );
    // The tick of the week without its Monday sits on the Tuesday's observation.
    let xs = point_xs(&svg);
    let tuesday = days
        .iter()
        .position(|epoch| *epoch == monday + 15 * 86_400)
        .unwrap();
    assert!((ticks[2].1 - xs[tuesday]).abs() < 0.01, "{ticks:?}");
}

#[test]
fn collapsed_gaps_move_annotations_onto_observations() {
    let extra = r#", {"mark": "annotation", "label": "Rule", "time": "2026-03-07"},
            {"mark": "annotation", "label": "Event", "time": "2026-03-08", "value": 2, "shape": "square"},
            {"mark": "band", "label": "Zone", "from": "2026-03-07", "to": "2026-03-10T12:00"}"#;
    let output = render_ok(&collapsed(&TRADING_DAYS, extra, ""));
    let svg = output.content;
    let xs = point_xs(&svg);
    let (monday, tuesday) = (xs[2], xs[3]);
    // A reference line on Saturday stands on Monday, the next observation.
    assert!(svg.contains(&format!(
        "<polyline points=\"{monday},78 {monday},414\" class=\"chartlet-rule\">"
    )));
    // So does a point marker on Sunday.
    let marker = svg
        .split("class=\"chartlet-marker\"")
        .next()
        .and_then(|head| head.rsplit("<polyline points=\"").next())
        .unwrap();
    let corners: Vec<f64> = marker
        .split([' ', '"'])
        .filter_map(|pair| pair.split(',').next()?.parse().ok())
        .collect();
    let center = f64::midpoint(corners[0], corners[1]);
    assert!((center - monday).abs() < 0.01, "{marker}");
    // A zone covers the observations between its edges: from Monday to Tuesday.
    let zone = svg
        .split(" class=\"chartlet-zone\"")
        .next()
        .and_then(|head| head.rsplit("<rect ").next())
        .expect("a zone");
    let attribute = |name: &str| -> f64 {
        zone.split(&format!("{name}=\""))
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .and_then(|value| value.parse().ok())
            .expect("a numeric attribute")
    };
    assert!((attribute("x") - monday).abs() < 0.01, "{zone}");
    assert!(
        (attribute("width") - (tuesday - monday)).abs() < 0.01,
        "{zone}"
    );
    assert!((attribute("y") - 78.0).abs() < 0.01, "{zone}");
    // Tooltips and description keep the real times.
    assert!(svg.contains("<title>Rule: 2026-03-07</title>"));
    assert!(svg.contains("Reference lines: Rule at 2026-03-07."));
}

#[test]
fn collapsed_gaps_refuse_times_without_an_observation() {
    for (extra, path) in [
        (
            r#", {"mark": "annotation", "label": "Rule", "time": "2026-03-04"}"#,
            "/panes/0/layers/1/time",
        ),
        (
            r#", {"mark": "annotation", "label": "Event", "time": "2026-03-12", "value": 2}"#,
            "/panes/0/layers/1/time",
        ),
        (
            r#", {"mark": "band", "label": "Weekend", "from": "2026-03-07", "to": "2026-03-08"}"#,
            "/panes/0/layers/1",
        ),
        (
            r#", {"mark": "band", "label": "Before", "from": "2026-03-01", "to": "2026-03-04"}"#,
            "/panes/0/layers/1",
        ),
    ] {
        assert_eq!(
            render_err(&collapsed(&TRADING_DAYS, extra, "")),
            ("time_out_of_range", path.to_owned()),
            "{extra}"
        );
    }
    // A zone reaching beyond the observations is cut at the first and the last of them.
    let wide = r#", {"mark": "band", "label": "All", "from": "2026-03-01", "to": "2026-03-20"}"#;
    let svg = render_ok(&collapsed(&TRADING_DAYS, wide, "")).content;
    let xs = point_xs(&svg);
    assert!(svg.contains(&format!(
        "<rect x=\"{}\" y=\"78\" width=\"{}\"",
        xs[0],
        xs[4] - xs[0]
    )));
    // With the distances kept, the same times are fine.
    let rule = r#", {"mark": "annotation", "label": "Rule", "time": "2026-03-04"}"#;
    render_ok(&collapsed(&TRADING_DAYS, rule, "").replace("collapse", "show"));
}

#[test]
fn collapsed_zoom_windows_close_their_gaps_too() {
    let zones =
        r#", {"mark": "band", "label": "Early", "from": "2026-03-05", "to": "2026-03-07T12:00"}"#;
    let zoom = r#", "zoomSteps": [
            {"label": "All", "from": "2026-03-05", "to": "2026-03-11"},
            {"label": "Weekend on", "from": "2026-03-07", "to": "2026-03-11"}
        ]"#;
    let html = html_ok(&collapsed(&TRADING_DAYS, zones, zoom));
    let panel = |index: usize| {
        html.split(&format!("chartlet-panel chartlet-panel-{index}"))
            .nth(1)
            .and_then(|panel| panel.split("</div>").next())
            .unwrap()
            .to_owned()
    };
    let (all, late) = (panel(0), panel(1));
    assert_eq!(point_xs(&all).len(), 5);
    let xs = point_xs(&late);
    assert_eq!(xs.len(), 3);
    assert!(((xs[2] - xs[1]) - (xs[1] - xs[0])).abs() < 0.01, "{xs:?}");
    // The zone keeps Thursday and Friday in the full window; the late window starts on the
    // Saturday, so its share of the zone holds no observation and is left out.
    assert!(all.contains("class=\"chartlet-zone\""));
    assert!(!late.contains("class=\"chartlet-zone\""));
}

#[test]
fn collapsed_gaps_are_named_in_the_description() {
    let spec = collapsed(&TRADING_DAYS, "", "");
    assert!(
        render_ok(&spec)
            .content
            .contains("Lowest: 1 (2026-03-05). Gaps in time are closed up.</desc>")
    );
    let html = html_ok(&german(&spec, "time"));
    assert!(
        html.contains("Zeiträume ohne Beobachtung sind ausgelassen."),
        "{html}"
    );
    let shown = render_ok(&spec.replace("collapse", "show")).content;
    assert!(!shown.contains("Gaps in time"));
}

#[test]
fn collapsed_small_multiples_share_their_slots() {
    let points = |days: &[i64]| {
        days.iter()
            .map(|day| format!(r#"{{"time": {day}, "value": 1}}"#))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let spec = format!(
        r#"{{
                "schemaVersion": 1,
                "type": "multiples",
                "title": "Desks",
                "timeAxis": {{"gaps": "collapse"}},
                "panes": [
                    {{"title": "A", "layers": [{{"mark": "line", "points": [{}]}}]}},
                    {{"title": "B", "layers": [{{"mark": "line", "points": [{}]}}]}}
                ]
            }}"#,
        points(&TRADING_DAYS[..3]),
        points(&TRADING_DAYS[2..]),
    );
    let svg = render_ok(&spec).content;
    let xs = point_xs(&svg);
    assert_eq!(xs.len(), 6);
    // Panel B, one grid cell of (800 - 2 × 16) / 2 pixels to the right, starts on the third
    // of the five shared slots, where panel A ends, and keeps the same distances.
    let cell = 384.0;
    let step = xs[1] - xs[0];
    assert!((xs[3] - cell - xs[2]).abs() < 0.01, "{xs:?}");
    assert!((xs[5] - xs[4] - step).abs() < 0.01, "{xs:?}");
}
