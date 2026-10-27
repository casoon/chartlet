use super::*;

const SLOPE: &str = r#"{"schemaVersion": 1, "type": "line", "slope": true, "title": "Revenue", "width": 600, "height": 360,
    "categories": ["2024", "2025"],
    "series": [
        {"name": "North", "values": [42, 55]}, {"name": "South", "values": [38, 36]},
        {"name": "East", "values": [51, 49]}
    ]}"#;

const RANK: &str = r#"{"schemaVersion": 1, "type": "line", "rank": true, "title": "Channels", "width": 600, "height": 360,
    "categories": ["Q1", "Q2", "Q3"],
    "series": [
        {"name": "Email", "values": [820, 780, 700]}, {"name": "Chat", "values": [610, 700, 790]},
        {"name": "Forum", "values": [300, 340, 410]}
    ]}"#;

fn html(spec: &str) -> String {
    render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content
}

#[test]
fn a_slope_chart_names_every_line_at_both_ends_and_has_no_legend() {
    let output = render_ok(SLOPE);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("<polyline").count(), 3);
    assert_eq!(
        svg.matches("chartlet-point chartlet-point-series-").count(),
        6
    );
    for label in [
        ">North  42<",
        ">55  North<",
        ">South  38<",
        ">36  South<",
        ">East  51<",
        ">49  East<",
    ] {
        assert!(svg.contains(label), "{label}");
    }
    assert!(svg.contains("<title>North, 2025: 55</title>"));
    assert!(!svg.contains("class=\"chartlet-legend"));
}

#[test]
fn the_description_names_the_biggest_rise_and_fall() {
    let html = html(SLOPE);
    assert!(html.contains("Slope chart of 3 series from 2024 to 2025: North 42 → 55 (+13); South 38 → 36 (−2); East 51 → 49 (−2). Largest rise: North (+13). Largest fall: South (−2)."), "{html}");
    // The table keeps the plain values of the line chart.
    assert!(html.contains("<th scope=\"row\">2024</th>"));
}

#[test]
fn a_rank_chart_plots_ranks_with_one_at_the_top() {
    let svg = render_ok(RANK).content;
    for label in [
        ">Email  #1<",
        ">#1  Chat<",
        ">Forum  #3<",
        ">#3  Forum<",
        ">#2  Email<",
    ] {
        assert!(svg.contains(label), "{label}");
    }
    assert!(svg.contains("<title>Chat, Q3: Rank 1 (790)</title>"));
    let html = html(RANK);
    assert!(html.contains("Rank chart of 3 series in 3 categories from Q1 to Q3; rank 1 is the largest value. Ranks per category: Email: 1, 1, 2; Chat: 2, 2, 1; Forum: 3, 3, 3. Largest rise in rank: Chat (2 → 1). Largest fall in rank: Email (1 → 2)."), "{html}");
}

#[test]
fn names_at_one_end_stay_apart() {
    // Two series end on almost the same value; their names must not overlap.
    let json = SLOPE.replace("[38, 36]", "[38, 49.2]");
    let svg = render_ok(&json).content;
    let y_of = |name: &str| -> f64 {
        let at = svg.find(&format!(">{name}<")).expect("label");
        let tag = &svg[svg[..at].rfind("<text").unwrap()..at];
        let start = tag.find(" y=\"").unwrap() + 4;
        tag[start..].split('"').next().unwrap().parse().unwrap()
    };
    let gap = (y_of("49  East") - y_of("49.2  South")).abs();
    assert!(gap >= 16.0 - 1e-6, "{gap}");
}

#[test]
fn slope_and_rank_charts_are_validated_by_name() {
    // A third category makes a slope chart a line chart that needs `rank`.
    let three = r#"{"schemaVersion": 1, "type": "line", "slope": true, "title": "T", "width": 600, "height": 360,
        "categories": ["a", "b", "c"], "series": [{"name": "X", "values": [1, 2, 3]}]}"#;
    assert_eq!(
        render_err(three),
        ("invalid_categories", "/categories".to_owned())
    );
    for (spec, from, to, code, path) in [
        (
            SLOPE,
            "[42, 55]",
            "[42, null]",
            "missing_value",
            "/series/0/values/1",
        ),
        (
            SLOPE,
            "\"slope\": true",
            "\"slope\": true, \"rank\": true",
            "option_not_supported",
            "/rank",
        ),
        (
            RANK,
            "\"rank\": true",
            "\"rank\": true, \"valueAxis\": {\"min\": 0}",
            "option_not_supported",
            "/valueAxis",
        ),
        (
            RANK,
            ", {\"name\": \"Chat\", \"values\": [610, 700, 790]},\n        {\"name\": \"Forum\", \"values\": [300, 340, 410]}",
            "",
            "invalid_categories",
            "/categories",
        ),
        (
            SLOPE,
            "\"type\": \"line\"",
            "\"type\": \"bar\"",
            "option_not_supported",
            "/slope",
        ),
    ] {
        assert_eq!(
            render_err(&spec.replace(from, to)),
            (code, path.to_owned()),
            "{from}"
        );
    }
}
