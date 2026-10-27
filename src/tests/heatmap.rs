use super::*;

const NUMERIC: &str = r#"{"schemaVersion": 1, "type": "heatmap", "title": "Load", "width": 600, "height": 360,
    "heatmap": {"rows": ["Mon", "Tue"], "columns": ["AM", "PM", "Night"], "valueLabel": "Tickets",
    "cells": [
        {"row": "Mon", "column": "AM", "value": 10}, {"row": "Mon", "column": "PM", "value": 50},
        {"row": "Mon", "column": "Night", "value": 90}, {"row": "Tue", "column": "AM", "value": 30},
        {"row": "Tue", "column": "PM", "value": 70}
    ]}}"#;

const CATEGORICAL: &str = r#"{"schemaVersion": 1, "type": "heatmap", "title": "Access", "width": 600, "height": 360,
    "heatmap": {"rows": ["Reader", "Owner"], "columns": ["Pages", "Settings"],
    "categories": ["Read", "Write", "None"],
    "cells": [
        {"row": "Reader", "column": "Pages", "category": "Read"},
        {"row": "Reader", "column": "Settings", "category": "None"},
        {"row": "Owner", "column": "Pages", "category": "Write"},
        {"row": "Owner", "column": "Settings", "category": "Write"}
    ]}}"#;

fn html(spec: &str) -> String {
    render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content
}

#[test]
fn a_numeric_heatmap_colors_values_writes_them_and_marks_the_missing_cell() {
    let output = render_ok(NUMERIC);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    // Five cells and six steps of the key; the sixth cell has no value.
    assert_eq!(
        svg.matches("class=\"chartlet-hm-cell chartlet-hm-s")
            .count(),
        5
    );
    assert_eq!(
        svg.matches("class=\"chartlet-hm-key chartlet-hm-s").count(),
        6
    );
    // The missing cell and its entry in the key.
    assert_eq!(svg.matches("class=\"chartlet-hm-missing\"").count(), 2);
    assert!(svg.contains("<title>Tue, Night: no value</title>"));
    assert!(svg.contains("<title>Mon, Night: 90</title>"));
    // The smallest value is the lightest step with dark text, the largest the darkest with white.
    assert!(svg.contains("chartlet-hm-cell chartlet-hm-s0\""));
    assert!(svg.contains("chartlet-hm-cell chartlet-hm-s5\""));
    assert!(svg.contains("class=\"chartlet-hm-value chartlet-hm-on-light\""));
    assert!(svg.contains("class=\"chartlet-hm-value chartlet-hm-on-dark\""));
    assert!(svg.contains(">Tickets</text>"));
}

#[test]
fn the_table_and_the_description_hold_every_cell() {
    let html = html(NUMERIC);
    assert!(html.contains(
        "<th scope=\"col\">Row</th><th scope=\"col\">Column</th><th scope=\"col\">Tickets</th>"
    ));
    assert!(html.contains("<th scope=\"row\">Mon</th><td>Night</td><td>90</td>"));
    assert!(html.contains("<th scope=\"row\">Tue</th><td>Night</td><td>no value</td>"));
    assert!(html.contains("Heatmap of 2 rows by 3 columns. Color shows Tickets, from 10 (lowest: Mon, AM (10)) to 90 (highest: Mon, Night (90)). 1 without a value."));
}

#[test]
fn categories_have_a_fill_and_a_symbol_each() {
    let output = render_ok(CATEGORICAL);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    for (class, cells) in [("c1", 1), ("c2", 2), ("c3", 1)] {
        assert_eq!(
            svg.matches(&format!("class=\"chartlet-hm-cell chartlet-hm-{class}\""))
                .count(),
            cells,
            "{class}"
        );
    }
    // One symbol in every cell and one in every entry of the key: circle, square, triangle.
    assert_eq!(svg.matches("class=\"chartlet-hm-symbol\"").count(), 4 + 3);
    assert!(svg.contains("<circle"));
    assert!(svg.contains("<polyline"));
    let html = html(CATEGORICAL);
    assert!(html.contains("<th scope=\"col\">Category</th>"));
    assert!(html.contains("<th scope=\"row\">Reader</th><td>Settings</td><td>None</td>"));
    assert!(html.contains("Cells by category: Read 1, Write 2, None 1."));
}

#[test]
fn a_diverging_scale_parts_at_the_midpoint() {
    let json = NUMERIC.replace(
        "\"valueLabel\"",
        "\"scale\": \"diverging\", \"midpoint\": 50, \"valueLabel\"",
    );
    let svg = render_ok(&json).content;
    assert_eq!(
        svg.matches("class=\"chartlet-hm-key chartlet-hm-d").count(),
        7
    );
    // 50 sits on the middle step, 10 and 90 on the outermost ones, with white text on those.
    assert!(svg.contains("<title>Mon, PM: 50</title>"));
    for (cell, step) in [("Mon, AM", "d0"), ("Mon, PM", "d3"), ("Mon, Night", "d6")] {
        let at = svg
            .find(&format!("<title>{cell}:"))
            .expect("the cell is drawn");
        let rect = &svg[svg[..at].rfind("<rect").unwrap()..at];
        assert!(
            rect.contains(&format!("chartlet-hm-{step}\"")),
            "{cell}: {rect}"
        );
    }
}

#[test]
fn rows_and_columns_follow_the_order_asked_for() {
    let positions = |svg: &str, labels: &[&str]| -> Vec<usize> {
        let mut found: Vec<(usize, &str)> = labels
            .iter()
            .map(|label| {
                (
                    svg.find(&format!(">{label}</text>")).expect("label"),
                    *label,
                )
            })
            .collect();
        found.sort_unstable();
        found
            .iter()
            .map(|(_, label)| labels.iter().position(|l| l == label).unwrap())
            .collect()
    };
    let listed = render_ok(NUMERIC).content;
    assert_eq!(positions(&listed, &["Mon", "Tue"]), [0, 1]);
    let by_value = render_ok(&NUMERIC.replace(
        "\"valueLabel\"",
        "\"rowOrder\": \"value\", \"columnOrder\": \"value\", \"valueLabel\"",
    ))
    .content;
    // Tuesday's mean is 50, Monday's 50: equal keep their order; the columns by mean are
    // Night (90), PM (60), AM (20).
    assert_eq!(positions(&by_value, &["Night", "PM", "AM"]), [0, 1, 2]);
    let by_label = render_ok(&NUMERIC.replace(
        "\"valueLabel\"",
        "\"columnOrder\": \"label\", \"valueLabel\"",
    ))
    .content;
    assert_eq!(positions(&by_label, &["AM", "Night", "PM"]), [0, 1, 2]);
}

#[test]
fn heatmaps_are_validated_by_name() {
    for (spec, from, to, code, path) in [
        (
            NUMERIC,
            "\"row\": \"Mon\", \"column\": \"AM\"",
            "\"row\": \"Sun\", \"column\": \"AM\"",
            "unknown_row",
            "/heatmap/cells/0/row",
        ),
        (
            NUMERIC,
            "\"column\": \"AM\", \"value\": 10",
            "\"column\": \"Dawn\", \"value\": 10",
            "unknown_column",
            "/heatmap/cells/0/column",
        ),
        (
            NUMERIC,
            "\"column\": \"PM\", \"value\": 50",
            "\"column\": \"AM\", \"value\": 50",
            "duplicate_cell",
            "/heatmap/cells/1",
        ),
        (
            NUMERIC,
            "\"columns\": [\"AM\", \"PM\"",
            "\"columns\": [\"AM\", \"AM\"",
            "duplicate_label",
            "/heatmap/columns/1",
        ),
        (
            NUMERIC,
            "\"value\": 10}",
            "\"category\": \"Read\"}",
            "invalid_cell",
            "/heatmap/cells/0",
        ),
        (
            NUMERIC,
            "\"valueLabel\"",
            "\"midpoint\": 5, \"valueLabel\"",
            "option_not_supported",
            "/heatmap/midpoint",
        ),
        (
            CATEGORICAL,
            "\"category\": \"Read\"",
            "\"category\": \"Admin\"",
            "unknown_category",
            "/heatmap/cells/0/category",
        ),
        (
            CATEGORICAL,
            "\"categories\"",
            "\"rowOrder\": \"value\", \"categories\"",
            "invalid_order",
            "/heatmap/rowOrder",
        ),
        (
            CATEGORICAL,
            "\"categories\": [\"Read\", \"Write\", \"None\"]",
            "\"categories\": [\"A\", \"B\", \"C\", \"D\", \"E\"]",
            "too_many_series",
            "/heatmap/categories",
        ),
    ] {
        assert_eq!(
            render_err(&spec.replace(from, to)),
            (code, path.to_owned()),
            "{from}"
        );
    }
}
