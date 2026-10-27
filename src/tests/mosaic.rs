use super::*;

const MOSAIC: &str = r#"{"schemaVersion": 1, "type": "mosaic", "title": "Revenue", "width": 640, "height": 420,
    "mosaic": {"columns": [
        {"label": "Europe", "segments": [{"label": "Hardware", "value": 30}, {"label": "Software", "value": 70}]},
        {"label": "Asia", "segments": [{"label": "Hardware", "value": 60}, {"label": "Software", "value": 40}]}
    ]}}"#;

fn html(spec: &str) -> String {
    render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content
}

#[test]
fn a_mosaic_chart_sizes_columns_by_total_and_segments_by_share() {
    let output = render_ok(MOSAIC);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    // Two columns of two segments, and one color per segment label across the columns.
    assert_eq!(
        svg.matches("class=\"chartlet-mosaic-cell chartlet-mosaic-1\"")
            .count(),
        2
    );
    assert_eq!(
        svg.matches("class=\"chartlet-mosaic-cell chartlet-mosaic-2\"")
            .count(),
        2
    );
    assert_eq!(svg.matches("class=\"chartlet-mosaic-swatch").count(), 2);
    // Both columns total 100, so they are equally wide; the shares are written in the segments.
    assert!(
        svg.contains(
            "<title>Europe, Hardware: 30 (30.0% of the column, 15.0% of the whole)</title>"
        )
    );
    for share in ["30.0%", "70.0%", "60.0%", "40.0%"] {
        assert!(
            svg.contains(&format!("class=\"chartlet-mosaic-value\">{share}</text>")),
            "{share}"
        );
    }
    // The name of a column, with its total and its share of the whole, stands under it.
    assert!(svg.contains(">Europe</text>"));
    assert!(svg.contains(">100 (50.0%)</text>"));
}

#[test]
fn the_description_and_the_table_have_both_levels() {
    let html = html(MOSAIC);
    assert!(html.contains("Mosaic chart of 2 columns and 2 segments, 200 in all."));
    assert!(html.contains("Europe 50.0% (100): Hardware 30.0%, Software 70.0%"));
    assert!(html.contains("<th scope=\"col\">Column</th><th scope=\"col\">Segment</th><th scope=\"col\">Value</th><th scope=\"col\">Share of column</th><th scope=\"col\">Share of total</th>"));
    assert!(html.contains(
        "<th scope=\"row\">Asia</th><td>Hardware</td><td>60</td><td>60.0%</td><td>30.0%</td>"
    ));
    assert!(html.contains(
        "<th scope=\"row\">Asia</th><td>Total</td><td>100</td><td>100.0%</td><td>50.0%</td>"
    ));
}

#[test]
fn a_narrow_column_is_wider_than_its_share_of_a_small_total_only_by_the_gap() {
    let json = MOSAIC.replace("\"value\": 60", "\"value\": 600");
    let svg = render_ok(&json).content;
    // Asia (640 of 740) is much wider than Europe (100 of 740).
    let width_of = |name: &str| -> f64 {
        let at = svg.find(&format!("<title>{name}, Hardware")).unwrap();
        let rect = &svg[svg[..at].rfind("<rect").unwrap()..at];
        let from = rect.find(" width=\"").unwrap() + 8;
        rect[from..from + rect[from..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    assert!(width_of("Asia") > 5.0 * width_of("Europe"));
}

#[test]
fn mosaic_charts_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            "\"value\": 30",
            "\"value\": 0",
            "invalid_value",
            "/mosaic/columns/0/segments/0/value",
        ),
        (
            "\"label\": \"Asia\"",
            "\"label\": \"Europe\"",
            "duplicate_label",
            "/mosaic/columns/1/label",
        ),
        (
            "{\"label\": \"Software\", \"value\": 70}",
            "{\"label\": \"Hardware\", \"value\": 70}",
            "duplicate_label",
            "/mosaic/columns/0/segments/1/label",
        ),
        (
            "{\"label\": \"Software\", \"value\": 40}",
            "{\"label\": \"A\", \"value\": 1}, {\"label\": \"B\", \"value\": 1}, {\"label\": \"C\", \"value\": 1}, {\"label\": \"D\", \"value\": 1}",
            "too_many_series",
            "/mosaic/columns",
        ),
    ] {
        assert_eq!(
            render_err(&MOSAIC.replace(from, to)),
            (code, path.to_owned()),
            "{from}"
        );
    }
    let none = r#"{"schemaVersion": 1, "type": "mosaic", "title": "T"}"#;
    assert_eq!(render_err(none), ("missing_mosaic", "/mosaic".to_owned()));
}
