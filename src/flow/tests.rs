use crate::{ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative};

const SPEC: &str = r#"{
    "schemaVersion": 1,
    "type": "flow",
    "title": "Review",
    "flow": {
        "nodes": [
            {"id": "start", "label": "Draft", "kind": "start"},
            {"id": "check", "label": "Looks good?", "kind": "decision"},
            {"id": "edit", "label": "Edit"},
            {"id": "publish", "label": "Publish"},
            {"id": "done", "label": "Live", "kind": "end"}
        ],
        "edges": [
            {"from": "start", "to": "check"},
            {"from": "check", "to": "publish", "label": "yes"},
            {"from": "check", "to": "edit", "label": "no"},
            {"from": "edit", "to": "check", "label": "again"},
            {"from": "publish", "to": "done"}
        ],
        "mainPath": ["start", "check", "publish", "done"]
    }
}"#;

fn svg(json: &str) -> crate::RenderOutput {
    render_json(json, RenderFormat::Svg, &RenderOptions::default())
        .expect("the test specification renders")
}

fn error(json: &str) -> (&'static str, String) {
    let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
        .expect_err("the test specification is invalid");
    (error.code, error.path)
}

/// The middle of every step's shape, by its tooltip, in page coordinates.
fn centers(svg: &str) -> Vec<(String, f64, f64)> {
    let chunks: Vec<&str> = svg.split('<').collect();
    let mut centers = Vec::new();
    for (index, shape) in chunks.iter().enumerate() {
        if !shape.contains("class=\"chartlet-flow-node") {
            continue;
        }
        let Some(title) = chunks
            .get(index + 1)
            .and_then(|next| next.strip_prefix("title>"))
        else {
            continue;
        };
        let number = |name: &str| -> Option<f64> {
            let start = shape.find(&format!(" {name}=\""))? + name.len() + 3;
            shape[start..].split('"').next()?.parse().ok()
        };
        let (x, y) = if let (Some(x), Some(y), Some(width), Some(height)) =
            (number("x"), number("y"), number("width"), number("height"))
        {
            (x + width / 2.0, y + height / 2.0)
        } else {
            let start = shape.find("points=\"").expect("a shape has points") + 8;
            let points: Vec<(f64, f64)> = shape[start..]
                .split('"')
                .next()
                .expect("points")
                .split(' ')
                .map(|pair| {
                    let (x, y) = pair.split_once(',').expect("a point");
                    (x.parse().expect("x"), y.parse().expect("y"))
                })
                .collect();
            let low_x = points.iter().map(|point| point.0).fold(f64::MAX, f64::min);
            let high_x = points.iter().map(|point| point.0).fold(f64::MIN, f64::max);
            let low_y = points.iter().map(|point| point.1).fold(f64::MAX, f64::min);
            let high_y = points.iter().map(|point| point.1).fold(f64::MIN, f64::max);
            (f64::midpoint(low_x, high_x), f64::midpoint(low_y, high_y))
        };
        centers.push((title.to_owned(), x, y));
    }
    centers
}

#[test]
fn every_edge_is_drawn_once_with_its_arrowhead_even_in_a_cycle() {
    let output = svg(SPEC);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let edges = |class: &str| {
        output
            .content
            .matches(&format!("class=\"{class}\""))
            .count()
    };
    assert_eq!(edges("chartlet-flow-edge"), 2);
    assert_eq!(edges("chartlet-flow-edge chartlet-flow-main"), 3);
    assert_eq!(
        output.content.matches("class=\"chartlet-flow-head").count(),
        5
    );
    assert!(
        output
            .content
            .contains("<title>Edit → Looks good?: again</title>")
    );
    assert!(output.content.contains(">again</text>"));
}

#[test]
fn the_main_path_runs_straight() {
    let centers = centers(&svg(SPEC).content);
    let x = |name: &str| {
        centers
            .iter()
            .find(|(title, _, _)| title == name)
            .unwrap_or_else(|| panic!("{name} is drawn"))
            .1
    };
    for step in ["Looks good?", "Publish", "Live"] {
        assert!(
            (x("Draft") - x(step)).abs() < 0.01,
            "{step} is off the line"
        );
    }
}

#[test]
fn steps_never_overlap_in_either_orientation() {
    for example in [
        include_str!("../../examples/release-flow.json"),
        include_str!("../../examples/order-flow.json"),
    ] {
        for orientation in ["portrait", "landscape"] {
            let json = example.replacen(
                "\"nodes\": [",
                &format!("\"orientation\": \"{orientation}\", \"nodes\": ["),
                1,
            );
            let spec = ChartSpec::from_json(&json).expect("parses");
            let flow = spec.flow.expect("a flow block");
            let output = svg(&json);
            let boxes: Vec<(f64, f64, f64, f64)> = output
                .content
                .split('<')
                .filter(|shape| shape.contains("class=\"chartlet-flow-node"))
                .filter_map(|shape| {
                    let number = |name: &str| -> Option<f64> {
                        let start = shape.find(&format!(" {name}=\""))? + name.len() + 3;
                        shape[start..].split('"').next()?.parse().ok()
                    };
                    Some((
                        number("x")?,
                        number("y")?,
                        number("width")?,
                        number("height")?,
                    ))
                })
                .collect();
            for (index, a) in boxes.iter().enumerate() {
                for b in &boxes[index + 1..] {
                    let apart = a.0 + a.2 <= b.0
                        || b.0 + b.2 <= a.0
                        || a.1 + a.3 <= b.1
                        || b.1 + b.3 <= a.1;
                    assert!(apart, "{orientation}: {a:?} overlaps {b:?}");
                }
            }
            assert_eq!(
                centers(&output.content).len(),
                flow.nodes.len(),
                "{orientation}"
            );
        }
    }
}

#[test]
fn a_wide_low_canvas_turns_the_flow_landscape() {
    let json = SPEC.replace(
        "\"title\": \"Review\",",
        "\"title\": \"Review\", \"width\": 1400, \"height\": 260,",
    );
    let output = svg(&json);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let centers = centers(&output.content);
    let y = |name: &str| {
        centers
            .iter()
            .find(|(title, _, _)| title == name)
            .expect("drawn")
            .2
    };
    assert!((y("Draft") - y("Live")).abs() < 0.01);
}

#[test]
fn a_loop_to_the_same_step_is_drawn_beside_it() {
    let json = SPEC.replace(
        r#"{"from": "publish", "to": "done"}"#,
        r#"{"from": "publish", "to": "done"}, {"from": "edit", "to": "edit", "label": "polish"}"#,
    );
    let output = svg(&json);
    assert!(
        output
            .content
            .contains("<title>Edit → Edit: polish</title>")
    );
    let table = text_alternative(&ChartSpec::from_json(&json).expect("parses"))
        .expect("valid")
        .table;
    let edit = table
        .rows
        .iter()
        .find(|row| row[0] == "Edit")
        .expect("a row");
    assert_eq!(edit[2], "Looks good? (again), Edit (polish)");
}

#[test]
fn the_text_alternative_follows_the_reading_order() {
    let alternative =
        text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
    assert_eq!(
        alternative.description,
        "Flow chart with 5 steps and 5 connections. Main path: Draft → Looks good? → Publish → Live. Draft (start): leads to Looks good?. Looks good? (decision): leads to Publish (yes) and Edit (no). Edit: leads to Looks good? (again). Publish: leads to Live. Live (end): no further step."
    );
    assert_eq!(alternative.table.columns, ["Step", "Kind", "Leads to"]);
    assert_eq!(
        alternative.table.rows[1],
        ["Looks good?", "decision", "Publish (yes), Edit (no)"]
    );
}

#[test]
fn lanes_and_groups_appear_in_the_table_and_in_german() {
    let json = include_str!("../../examples/release-flow.json").replace(
        "\"schemaVersion\": 1,",
        "\"schemaVersion\": 1, \"locale\": \"de\",",
    );
    let alternative =
        text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
    assert_eq!(
        alternative.table.columns,
        ["Schritt", "Art", "Bahn", "Gruppe", "Führt zu"]
    );
    assert!(alternative.description.starts_with(
        "Ablaufdiagramm mit 10 Schritten und 11 Verbindungen in 3 Bahnen: Development, Continuous integration und Operations."
    ));
    assert!(
        alternative
            .description
            .ends_with("Gruppe „Quality gates“: Build, Run tests, All green?.")
    );
}

#[test]
fn invalid_flows_name_the_field() {
    let lanes = SPEC.replace(
        "\"nodes\": [",
        "\"lanes\": [{\"id\": \"a\", \"label\": \"A\"}], \"nodes\": [",
    );
    for (json, code, path) in [
        (SPEC.replace(r#""to": "publish", "label": "yes""#, r#""to": "nowhere", "label": "yes""#), "unknown_node", "/flow/edges/1/to"),
        (SPEC.replace(r#""id": "edit""#, r#""id": "check""#), "duplicate_id", "/flow/nodes/2/id"),
        (SPEC.replace(r#""id": "edit""#, r#""id": "9edit""#), "invalid_id", "/flow/nodes/2/id"),
        (lanes.clone(), "missing_lane", "/flow/nodes/0/lane"),
        (SPEC.replace(r#""kind": "start"}"#, r#""kind": "start", "lane": "a"}"#), "unknown_lane", "/flow/nodes/0/lane"),
        (SPEC.replace(r#"["start", "check", "publish", "done"]"#, r#"["start", "publish"]"#), "main_path_gap", "/flow/mainPath/1"),
        (
            SPEC.replace("\"mainPath\"", r#""groups": [{"label": "G", "nodes": ["edit", "publish"]}, {"label": "H", "nodes": ["edit"]}], "mainPath""#),
            "duplicate_member",
            "/flow/groups/1/nodes/0",
        ),
        (SPEC.replace("\"type\": \"flow\"", "\"type\": \"bar\""), "option_not_supported", "/flow"),
    ] {
        assert_eq!(error(&json), (code, path.to_owned()), "{json}");
    }
    let spans = lanes
        .replace(
            "\"lanes\": [{\"id\": \"a\", \"label\": \"A\"}]",
            "\"lanes\": [{\"id\": \"a\", \"label\": \"A\"}, {\"id\": \"b\", \"label\": \"B\"}]",
        )
        .replace(r#""kind": "start"}"#, r#""kind": "start", "lane": "a"}"#)
        .replace(
            r#""kind": "decision"}"#,
            r#""kind": "decision", "lane": "a"}"#,
        )
        .replace(r#""label": "Edit"}"#, r#""label": "Edit", "lane": "b"}"#)
        .replace(
            r#""label": "Publish"}"#,
            r#""label": "Publish", "lane": "a"}"#,
        )
        .replace(r#""kind": "end"}"#, r#""kind": "end", "lane": "a"}"#)
        .replace(
            "\"mainPath\"",
            r#""groups": [{"label": "G", "nodes": ["check", "edit"]}], "mainPath""#,
        );
    assert_eq!(
        error(&spans),
        ("group_spans_lanes", "/flow/groups/0/nodes/1".to_owned())
    );
    let missing = r#"{"schemaVersion": 1, "type": "flow", "title": "Empty"}"#;
    assert_eq!(error(missing), ("missing_flow", "/flow".to_owned()));
}
