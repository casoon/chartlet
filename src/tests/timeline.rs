use super::*;

const TIMELINE: &str = r#"{"schemaVersion": 1, "type": "timeline", "title": "Plan", "width": 800, "height": 320,
        "timeline": {
            "items": [
                {"id": "a", "label": "Design", "start": "2027-01-04", "end": "2027-02-26", "group": "Early"},
                {"id": "b", "label": "Build", "start": "2027-03-01", "end": "2027-05-28", "group": "Late", "after": ["a"]},
                {"label": "Launch", "at": "2027-06-15", "group": "Late", "after": ["b"]}
            ],
            "markers": [{"label": "Today", "at": "2027-02-10"}]
        }}"#;

#[test]
fn a_timeline_draws_phases_milestones_markers_and_arrows_and_says_them_in_text() {
    let output = render_ok(TIMELINE);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(
        svg.matches("chartlet-timeline-phase chartlet-timeline-group-")
            .count(),
        2 + 2
    );
    assert_eq!(
        svg.matches("class=\"chartlet-timeline-milestone chartlet-timeline-group-2\"")
            .count(),
        1
    );
    assert_eq!(svg.matches("class=\"chartlet-timeline-marker\"").count(), 1);
    assert_eq!(svg.matches("class=\"chartlet-timeline-link\"").count(), 2);
    assert_eq!(svg.matches("class=\"chartlet-timeline-head\"").count(), 2);
    assert!(svg.contains("<title>Design: 2027-01-04 – 2027-02-26</title>"));
    let html = render_json(TIMELINE, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains(
        "Timeline with 2 phases and 1 milestone from 2027-01-04 to 2027-06-15. Design: 2027-01-04 – 2027-02-26. Build: 2027-03-01 – 2027-05-28, after Design. Launch: 2027-06-15, after Build. Marked days: Today (2027-02-10)."
    ));
    assert!(html.contains("<th scope=\"col\">Item</th>"));
    assert!(html.contains("<th scope=\"row\">Launch</th><td>Milestone</td><td>2027-06-15</td><td></td><td>Late</td><td>Build</td>"));
}

#[test]
fn the_height_of_a_timeline_grows_with_its_items() {
    let rows: Vec<String> = (0..20)
        .map(|index| {
            format!(r#"{{"label": "Item {index}", "start": "2027-01-04", "end": "2027-02-26"}}"#)
        })
        .collect();
    let json = format!(
        r#"{{"schemaVersion": 1, "type": "timeline", "title": "Many", "width": 700, "height": 300, "timeline": {{"items": [{}]}}}}"#,
        rows.join(",")
    );
    let output = render_ok(&json);
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "canvas_too_small" && warning.path == "/height")
    );
}

#[test]
fn timelines_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""end": "2027-02-26""#,
            r#""end": "2026-12-01""#,
            "invalid_range",
            "/timeline/items/0/end",
        ),
        (
            r#""end": "2027-02-26""#,
            r#""at": "2027-02-26""#,
            "invalid_item",
            "/timeline/items/0",
        ),
        (
            r#""at": "2027-06-15""#,
            r#""start": "2027-06-15""#,
            "invalid_item",
            "/timeline/items/2",
        ),
        (
            r#""after": ["a"]"#,
            r#""after": ["nowhere"]"#,
            "unknown_node",
            "/timeline/items/1/after/0",
        ),
        (
            r#""after": ["b"]"#,
            r#""after": ["launch"], "id": "launch""#,
            "circular_dependency",
            "/timeline/items/2/after/0",
        ),
        (r#""group": "Early""#, r#""group": "Late""#, "", ""),
        (
            r#""start": "2027-01-04""#,
            r#""start": "soon""#,
            "invalid_time",
            "/timeline/items/0/start",
        ),
    ] {
        if code.is_empty() {
            continue;
        }
        let error = render_json(
            &TIMELINE.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
    // A circle of items that follow each other.
    let circle = TIMELINE.replace(
        r#""id": "a", "label": "Design""#,
        r#""id": "a", "after": ["b"], "label": "Design""#,
    );
    let error = render_json(&circle, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "circular_dependency");
    // A group on some items only.
    let partial = TIMELINE.replace(r#", "group": "Early""#, "");
    let error = render_json(&partial, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("missing_group", "/timeline/items/0/group")
    );
}

#[test]
fn an_item_that_starts_before_its_predecessor_ends_is_a_warning() {
    let json = TIMELINE.replace(r#""start": "2027-03-01""#, r#""start": "2027-02-01""#);
    let output = render_ok(&json);
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "follows_overlap"
                && warning.path == "/timeline/items/1/after/0")
    );
}

#[test]
fn a_summary_phase_shows_the_progress_of_the_phases_it_sums_up() {
    // Two weeks done, two weeks half done: (14 * 1 + 14 * 0.5) / 28 = 75 %.
    let spec = r#"{"schemaVersion": 1, "type": "timeline", "title": "Plan", "width": 800, "height": 300,
            "timeline": {"items": [
                {"id": "all", "label": "All", "start": "2027-01-04", "end": "2027-02-01", "rollup": ["a", "b"]},
                {"id": "a", "label": "A", "start": "2027-01-04", "end": "2027-01-18", "progress": 1},
                {"id": "b", "label": "B", "start": "2027-01-18", "end": "2027-02-01", "progress": 0.5}
            ]}}"#;
    let output = render_ok(spec);
    assert!(
        output.content.contains("All: 75% done."),
        "{}",
        output.content
    );
    for (from, to, code, path) in [
        (
            r#""rollup": ["a", "b"]"#,
            r#""rollup": ["a", "x"]"#,
            "unknown_node",
            "/timeline/items/0/rollup/1",
        ),
        (
            r#""rollup": ["a", "b"]"#,
            r#""rollup": ["a", "all"]"#,
            "invalid_rollup",
            "/timeline/items/0/rollup/1",
        ),
        (
            r#""rollup": ["a", "b"]"#,
            r#""rollup": ["a"], "progress": 0.2"#,
            "conflicting_progress",
            "/timeline/items/0/rollup",
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
fn an_arrow_turns_where_no_bar_of_the_rows_between_stands_in_the_way() {
    // A ends where B (the row between) begins, so the usual turn just after A would run
    // through B's first days; the arrow from A to C must turn elsewhere.
    let spec = r#"{"schemaVersion": 1, "type": "timeline", "title": "Plan", "width": 800, "height": 300,
            "timeline": {"items": [
                {"id": "a", "label": "A", "start": "2027-01-04", "end": "2027-01-11"},
                {"id": "b", "label": "B", "start": "2027-01-09", "end": "2027-01-25"},
                {"id": "c", "label": "C", "start": "2027-02-01", "end": "2027-02-08", "after": ["a"]}
            ]}}"#;
    let svg = render_ok(spec).content;
    let number = |text: &str, key: &str| -> f64 {
        text.split(key)
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap()
    };
    let bar = svg.split("<title>B:").next().unwrap();
    let rect = &bar[bar.rfind("<rect").unwrap()..];
    let (left, width) = (number(rect, " x=\""), number(rect, "width=\""));
    let link = svg
        .split("\" class=\"chartlet-timeline-link\"")
        .next()
        .unwrap();
    let points = &link[link.rfind("points=\"").unwrap() + 8..];
    let points: Vec<(f64, f64)> = points
        .split('"')
        .next()
        .unwrap()
        .split(' ')
        .map(|pair| {
            let (x, y) = pair.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect();
    // Every part of the arrow that runs up or down stays clear of B's bar.
    for pair in points.windows(2) {
        if (pair[0].0 - pair[1].0).abs() < 0.01 && (pair[0].1 - pair[1].1).abs() > 1.0 {
            assert!(
                pair[0].0 < left || pair[0].0 > left + width,
                "vertical at {} inside {left}..{}",
                pair[0].0,
                left + width
            );
        }
    }
}

#[test]
fn a_timeline_reads_and_writes_its_days_in_a_zone() {
    // 1798761600 is 2027-01-01 00:00 UTC, which is 02:00 at +02:00.
    let spec = r#"{"schemaVersion": 1, "type": "timeline", "title": "Plan", "width": 800, "height": 300,
            "timeline": {"timezone": "+02:00", "items": [{"label": "Launch", "at": 1798761600}]}}"#;
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("2027-01-01 02:00"), "{html}");
    let utc = spec.replace(r#""timezone": "+02:00", "#, "");
    let html = render_json(&utc, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(!html.contains("02:00"));
    // A named zone follows its daylight saving time: 1814400000 is 2027-07-01 00:00 UTC, 02:00
    // in Berlin (summer time), where the same hour of January is 01:00.
    let berlin = spec
        .replace("+02:00", "Europe/Berlin")
        .replace("1798761600", "1814400000");
    let html = render_json(&berlin, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("2027-07-01 02:00"), "{html}");
    let winter = berlin.replace("1814400000", "1798761600");
    let html = render_json(&winter, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("2027-01-01 01:00"), "{html}");
    let bad = spec.replace("+02:00", "Europe/Atlantis");
    let error = render_json(&bad, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("invalid_timezone", "/timeline/timezone")
    );
}

#[test]
fn a_timeline_can_shade_its_weekends() {
    // 2027-01-04 is a Monday: the days to 2027-01-18 hold two weekends.
    let spec = r#"{"schemaVersion": 1, "type": "timeline", "title": "Plan", "width": 800, "height": 300,
            "timeline": {"weekends": true, "items": [{"label": "One", "start": "2027-01-04", "end": "2027-01-18"}]}}"#;
    let output = render_ok(spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let shaded = output
        .content
        .matches("class=\"chartlet-timeline-weekend\"")
        .count();
    assert!((4..=6).contains(&shaded), "{shaded} weekend days");
    let long = spec.replace("2027-01-18", "2027-09-18");
    let output = render_ok(&long);
    assert_eq!(
        output
            .content
            .matches("chartlet-timeline-weekend\"")
            .count(),
        0
    );
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "weekends_hidden")
    );
}

#[test]
fn a_timeline_shows_progress_and_can_bundle_rows_by_group() {
    let spec = r#"{"schemaVersion": 1, "type": "timeline", "title": "Plan", "width": 800, "height": 300,
            "timeline": {"groupRows": true, "items": [
                {"label": "One", "start": "2027-01-04", "end": "2027-02-01", "group": "A", "progress": 0.5},
                {"label": "Two", "start": "2027-01-11", "end": "2027-02-08", "group": "B"},
                {"label": "Three", "start": "2027-02-01", "end": "2027-03-01", "group": "A"}
            ]}}"#;
    let output = render_ok(spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-timeline-done\"")
            .count(),
        1
    );
    assert!(output.content.contains("(50% done)"));
    // Rows: One, Three (group A), then Two (group B): Two's bar sits below Three's.
    let y = |label: &str| {
        let at = output.content.find(&format!("<title>{label}:")).unwrap();
        let rect = &output.content[output.content[..at].rfind("<rect").unwrap()..at];
        let y = rect
            .split("y=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        y.parse::<f64>().unwrap()
    };
    assert!(y("One") < y("Three") && y("Three") < y("Two"));
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("<th scope=\"col\">Progress</th>"));
    let milestone = spec.replace(
        r#""start": "2027-01-11", "end": "2027-02-08", "group": "B""#,
        r#""at": "2027-01-11", "group": "B", "progress": 0.2"#,
    );
    let error = render_json(&milestone, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("progress_on_milestone", "/timeline/items/1/progress")
    );
    let ungrouped = r#"{"schemaVersion": 1, "type": "timeline", "title": "Plan", "timeline": {"groupRows": true, "items": [{"label": "One", "start": "2027-01-04", "end": "2027-02-01"}]}}"#;
    let error = render_json(ungrouped, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.path, "/timeline/groupRows");
}
