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
fn a_narrow_chart_keeps_frames_and_labels_on_the_page() {
    let json = include_str!("../../examples/ticket-states.json")
        .replace("\"width\": 900", "\"width\": 360");
    let output = svg(&json);
    let width: f64 = {
        let start = output.content.find(" width=\"").expect("a width") + 8;
        output.content[start..]
            .split('"')
            .next()
            .expect("a value")
            .parse()
            .expect("a number")
    };
    for shape in output.content.split('<').filter(|shape| {
        shape.contains("class=\"chartlet-flow-group\"")
            || shape.contains("class=\"chartlet-diagram-chip\"")
    }) {
        let number = |name: &str| -> f64 {
            let start = shape.find(&format!(" {name}=\"")).expect("an attribute") + name.len() + 3;
            shape[start..]
                .split('"')
                .next()
                .expect("a value")
                .parse()
                .expect("a number")
        };
        assert!(number("x") >= 0.0, "{shape}");
        assert!(number("x") + number("width") <= width, "{shape}");
    }
}

#[test]
fn the_html_profile_lets_every_step_be_focused() {
    let html = render_json(SPEC, RenderFormat::Html, &RenderOptions::default())
        .expect("renders")
        .content;
    // Five steps: a radio button each, besides the one for all of them, and a label of each
    // laid over its step.
    assert_eq!(html.matches("class=\"chartlet-focus-node\"").count(), 5);
    assert_eq!(html.matches("class=\"chartlet-hotspot\"").count(), 5);
    assert!(html.contains("> Looks good?</label>"));
    assert!(html.contains("-focus-1:checked) #"));
    // The edge from Draft to Looks good? belongs to both steps' focus.
    assert!(html.contains("<g class=\"chartlet-f chartlet-f-0 chartlet-f-1\">"));
    // The SVG profile has no controls; the root stays an image.
    let svg = svg(SPEC).content;
    assert!(!svg.contains("<label") && !svg.contains("<input"));
    assert!(svg.contains("role=\"img\""));
}

#[test]
fn the_text_alternative_follows_the_reading_order() {
    let alternative =
        text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
    assert_eq!(
        alternative.description,
        "Flow chart with 5 steps and 5 connections. Main path: Draft → Looks good? → Publish → Live. Most connected: Looks good? (4 connections). Loop: Looks good? → Edit → Looks good?. Draft (start): leads to Looks good?. Looks good? (decision): leads to Publish (yes) and Edit (no). Edit: leads to Looks good? (again). Publish: leads to Live. Live (end): no further step."
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
fn a_main_path_between_two_steps_takes_the_shortest_way() {
    let between = SPEC.replace(
        r#"["start", "check", "publish", "done"]"#,
        r#"{"from": "start", "to": "done"}"#,
    );
    // The same drawing; only the id prefix, a hash of the specification, differs.
    let options = RenderOptions {
        id_prefix: Some("review".to_owned()),
        ..RenderOptions::default()
    };
    let render = |json: &str| render_json(json, RenderFormat::Svg, &options).expect("renders");
    assert_eq!(render(&between).content, render(SPEC).content);
}

#[test]
fn weighted_edges_are_drawn_stronger_the_more_they_carry() {
    let weighted = SPEC
        .replace(
            r#""to": "publish", "label": "yes""#,
            r#""to": "publish", "label": "yes", "weight": 80"#,
        )
        .replace(
            r#""to": "edit", "label": "no""#,
            r#""to": "edit", "label": "no", "weight": 20"#,
        );
    let content = svg(&weighted).content;
    assert!(content.contains(" chartlet-flow-w8\""), "{content}");
    assert!(content.contains(" chartlet-flow-w2\""), "{content}");
    assert!(content.contains(".chartlet-flow-w8 .chartlet-flow-edge{stroke-width:5}"));
    assert!(!svg(SPEC).content.contains("chartlet-flow-w"));
    let alternative =
        text_alternative(&ChartSpec::from_json(&weighted).expect("parses")).expect("valid");
    assert!(
        alternative
            .description
            .contains("Heaviest connection: Looks good? → Publish (80).")
    );
    assert_eq!(
        alternative.table.rows[1][2],
        "Publish (yes, weight 80), Edit (no, weight 20)"
    );
}

#[test]
fn values_color_the_steps_and_parts_draw_a_stripe() {
    let colored = SPEC
        .replace(r#""label": "Edit"}"#, r#""label": "Edit", "value": 9, "parts": [{"label": "kept", "value": 3}, {"label": "dropped", "value": 1}]}"#)
        .replace(r#""label": "Publish"}"#, r#""label": "Publish", "value": 1}"#)
        .replace("\"mainPath\"", "\"valueLabel\": \"Days\", \"mainPath\"");
    let content = svg(&colored).content;
    assert!(content.contains(" chartlet-heat-5\""), "{content}");
    assert!(content.contains(" chartlet-heat-0\""), "{content}");
    assert!(content.contains("class=\"chartlet-part chartlet-part-1\""));
    assert!(content.contains("<title>kept 3 (75%)</title>"), "{content}");
    let alternative =
        text_alternative(&ChartSpec::from_json(&colored).expect("parses")).expect("valid");
    assert!(
        alternative
            .description
            .contains("Color shows Days, from 1 (lightest) to 9 (darkest): Edit 9, Publish 1."),
        "{}",
        alternative.description
    );
    assert!(
        alternative
            .description
            .contains("Parts: Edit: kept 3 (75%), dropped 1 (25%).")
    );
    assert_eq!(
        alternative.table.columns,
        ["Step", "Kind", "Days", "Parts", "Leads to"]
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
        (SPEC.replace(r#"["start", "check", "publish", "done"]"#, r#"{"from": "start", "to": "nowhere"}"#), "unknown_node", "/flow/mainPath/to"),
        (SPEC.replace(r#"["start", "check", "publish", "done"]"#, r#"{"from": "done", "to": "start"}"#), "main_path_unreachable", "/flow/mainPath"),
        (SPEC.replace(r#""to": "publish", "label": "yes""#, r#""to": "publish", "label": "yes", "weight": -1"#), "invalid_edge_weight", "/flow/edges/1/weight"),
        (SPEC.replace(r#""kind": "decision"}"#, r#""kind": "decision", "parts": [{"label": "a", "value": 1}]}"#), "option_not_supported", "/flow/nodes/1/parts"),
        (SPEC.replace(r#""label": "Edit"}"#, r#""label": "Edit", "parts": [{"label": "a", "value": 0}]}"#), "invalid_value", "/flow/nodes/2/parts"),
        (SPEC.replace(r#""label": "Edit"}"#, r#""label": "Edit", "parts": [{"label": "a", "value": -1}]}"#), "invalid_value", "/flow/nodes/2/parts/0/value"),
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

/// Two frames that share a layer but reach over different ones must not be drawn into each other.
#[test]
fn frames_of_unrelated_groups_do_not_overlap() {
    let json = r#"{
        "schemaVersion": 1, "type": "architecture", "title": "Two sides", "width": 900, "height": 900,
        "architecture": {
            "boundaries": [
                {"id": "client", "label": "Client"},
                {"id": "server", "label": "Server"}
            ],
            "components": [
                {"id": "form", "label": "Form", "kind": "frontend", "in": "client"},
                {"id": "local", "label": "Local module", "in": "client"},
                {"id": "caller", "label": "Caller", "kind": "external"},
                {"id": "api", "label": "API", "in": "server"},
                {"id": "rpc", "label": "RPC", "in": "server"},
                {"id": "core", "label": "Core", "in": "server"}
            ],
            "connections": [
                {"from": "form", "to": "local"},
                {"from": "form", "to": "api", "dash": "dashed"},
                {"from": "local", "to": "api", "dash": "dashed"},
                {"from": "caller", "to": "rpc"},
                {"from": "api", "to": "core"},
                {"from": "rpc", "to": "core"}
            ],
            "mainPath": ["form", "local"]
        }
    }"#;
    let output = svg(json);
    let frames: Vec<(f64, f64, f64, f64)> = output
        .content
        .split('<')
        .filter(|shape| shape.contains("class=\"chartlet-flow-group\""))
        .map(|shape| {
            let number = |name: &str| -> f64 {
                let start =
                    shape.find(&format!(" {name}=\"")).expect("an attribute") + name.len() + 3;
                shape[start..]
                    .split('"')
                    .next()
                    .expect("a value")
                    .parse()
                    .expect("a number")
            };
            (number("x"), number("y"), number("width"), number("height"))
        })
        .collect();
    assert_eq!(frames.len(), 2);
    let (a, b) = (frames[0], frames[1]);
    let apart = a.0 + a.2 <= b.0 || b.0 + b.2 <= a.0 || a.1 + a.3 <= b.1 || b.1 + b.3 <= a.1;
    assert!(apart, "{a:?} overlaps {b:?}");
}

#[test]
fn a_long_sublabel_wraps_instead_of_being_cut_off() {
    let json = r#"{
        "schemaVersion": 1, "type": "architecture", "title": "Wrap", "width": 900, "height": 400,
        "architecture": {
            "components": [
                {"id": "gen", "label": "Generators", "sublabel": "Puzzles · worksheets · certificates · invoices"},
                {"id": "out", "label": "Writer"}
            ],
            "connections": [{"from": "gen", "to": "out"}]
        }
    }"#;
    let output = svg(json);
    assert!(
        output
            .warnings
            .iter()
            .all(|warning| warning.code != "text_truncated"),
        "{:?}",
        output.warnings
    );
    assert!(output.content.contains("Puzzles · worksheets"));
    assert!(output.content.contains("certificates · invoices"));
}

const REVIEWS: &str = r#"{
    "schemaVersion": 1, "type": "flow", "title": "Contract", "width": 1200, "height": 480,
    "flow": {
        "nodes": [
            {"id": "request", "label": "Requested", "kind": "start"},
            {"id": "quotes", "label": "Collect quotes"},
            {"id": "legal", "label": "Legal review"},
            {"id": "finance", "label": "Finance review"},
            {"id": "done", "label": "Approved", "kind": "end"}
        ],
        "edges": [
            {"from": "request", "to": "quotes"},
            {"from": "request", "to": "legal"},
            {"from": "quotes", "to": "finance"},
            {"from": "legal", "to": "done"},
            {"from": "finance", "to": "done"}
        ],
        "sameRank": [["legal", "finance"]]
    },
    "mobile": {"width": 360, "height": 900}
}"#;

/// Where a step lies along the flow: down in portrait, right in landscape.
fn along(svg: &str, name: &str, landscape: bool) -> f64 {
    let (_, x, y) = centers(svg)
        .into_iter()
        .find(|(title, _, _)| title == name)
        .unwrap_or_else(|| panic!("{name} is drawn"));
    if landscape { x } else { y }
}

#[test]
fn steps_that_share_a_rank_line_up_in_every_orientation_and_on_a_phone() {
    let without = REVIEWS.replace(r#"[["legal", "finance"]]"#, "[]");
    assert_ne!(without, REVIEWS);
    for orientation in ["portrait", "landscape"] {
        let landscape = orientation == "landscape";
        let pin = |json: &str| {
            json.replacen(
                "\"nodes\": [",
                &format!("\"orientation\": \"{orientation}\", \"nodes\": ["),
                1,
            )
        };
        let before = svg(&pin(&without)).content;
        assert!(
            along(&before, "Legal review", landscape) < along(&before, "Finance review", landscape),
            "{orientation}: without the hint legal review comes first"
        );
        for variant in [crate::Variant::Desktop, crate::Variant::Mobile] {
            let options = RenderOptions {
                variant,
                ..RenderOptions::default()
            };
            let output = render_json(&pin(REVIEWS), RenderFormat::Svg, &options).expect("renders");
            // A landscape flow is wider than a phone; that is all the layout may report.
            assert!(
                output
                    .warnings
                    .iter()
                    .all(|warning| warning.code == "canvas_too_small"),
                "{:?}",
                output.warnings
            );
            let after = output.content;
            assert!(
                (along(&after, "Legal review", landscape)
                    - along(&after, "Finance review", landscape))
                .abs()
                    < 0.01,
                "{orientation} {variant:?}: the reviews share a rank"
            );
            let again = render_json(&pin(REVIEWS), RenderFormat::Svg, &options)
                .expect("renders")
                .content;
            assert_eq!(after, again, "{orientation} {variant:?} is deterministic");
        }
    }
}

#[test]
fn a_shared_rank_gathers_the_stores_of_a_data_tier() {
    let example = include_str!("../../examples/data-tier.json");
    let without = example.replace(r#"[["catalogDb", "ordersDb", "billingDb"]]"#, "[]");
    assert_ne!(without, example);
    let spread = |json: &str| {
        let content = svg(json).content;
        let positions: Vec<f64> = ["Catalog", "Orders", "Invoices"]
            .iter()
            .map(|store| along(&content, &format!("{store} – Postgres"), true))
            .collect();
        let low = positions.iter().copied().fold(f64::MAX, f64::min);
        let high = positions.iter().copied().fold(f64::MIN, f64::max);
        high - low
    };
    assert!(spread(&without) > 100.0);
    assert!(spread(example) < 0.01);
}

#[test]
fn an_edge_against_a_shared_rank_is_drawn_back() {
    // "b" follows "x", which follows "a"; with "a" and "b" on one rank the edge into "b" turns
    // around like the edge that closes a cycle.
    let json = r#"{
        "schemaVersion": 1, "type": "flow", "title": "Back",
        "flow": {
            "orientation": "portrait",
            "nodes": [
                {"id": "a", "label": "A"}, {"id": "x", "label": "X"}, {"id": "b", "label": "B"}
            ],
            "edges": [{"from": "a", "to": "x"}, {"from": "x", "to": "b"}],
            "sameRank": [["a", "b"]]
        }
    }"#;
    let output = svg(json);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let content = output.content;
    assert!((along(&content, "A", false) - along(&content, "B", false)).abs() < 0.01);
    assert!(along(&content, "X", false) > along(&content, "A", false));
    assert_eq!(content.matches("class=\"chartlet-flow-head").count(), 2);
}

#[test]
fn invalid_shared_ranks_name_the_field() {
    let flow = |ranks: &str| {
        REVIEWS.replace(
            r#""sameRank": [["legal", "finance"]]"#,
            &format!(r#""sameRank": {ranks}"#),
        )
    };
    for (json, code, path) in [
        (
            flow(r#"[["legal", "nowhere"]]"#),
            "unknown_node",
            "/flow/sameRank/0/1",
        ),
        (flow(r#"[["legal"]]"#), "empty_data", "/flow/sameRank/0"),
        (
            flow(r#"[["legal", "finance", "legal"]]"#),
            "duplicate_member",
            "/flow/sameRank/0/2",
        ),
        (
            flow(r#"[["legal", "finance"], ["quotes", "finance"]]"#),
            "duplicate_member",
            "/flow/sameRank/1/1",
        ),
        (
            flow(r#"[["quotes", "legal", "finance"]]"#),
            "same_rank_edge",
            "/flow/sameRank/0/2",
        ),
        (
            flow(r#"[["done", "legal"]]"#),
            "same_rank_edge",
            "/flow/sameRank/0/1",
        ),
    ] {
        assert_eq!(error(&json), (code, path.to_owned()), "{json}");
    }
    let architecture = include_str!("../../examples/data-tier.json");
    for (ranks, code, path) in [
        (
            r#"[["catalogDb", "db"]]"#,
            "unknown_node",
            "/architecture/sameRank/0/1",
        ),
        (
            r#"[["catalog", "catalogDb"]]"#,
            "same_rank_edge",
            "/architecture/sameRank/0/1",
        ),
    ] {
        let json = architecture.replace(r#"[["catalogDb", "ordersDb", "billingDb"]]"#, ranks);
        assert_eq!(error(&json), (code, path.to_owned()), "{json}");
    }
}

#[test]
fn a_flow_chart_gives_steps_and_edges_roles_in_picture_legend_and_text() {
    let json = SPEC
        .replace(
            r#"{"id": "edit", "label": "Edit"}"#,
            r#"{"id": "edit", "label": "Edit", "role": "deprecated", "value": 3}"#,
        )
        .replace(
            r#"{"from": "publish", "to": "done"}"#,
            r#"{"from": "publish", "to": "done", "role": "critical"}"#,
        );
    let output = svg(&json);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    // The role joins the value's fill on the step's group; the outline is the role's.
    assert!(
        output
            .content
            .contains("chartlet-heat-3 chartlet-flow-role-deprecated")
    );
    assert!(output.content.contains(">−</text>"));
    assert!(output.content.contains(">Deprecated</text>"));
    assert!(output.content.contains(">Critical</text>"));
    let alternative =
        text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
    assert!(
        alternative
            .description
            .ends_with("Roles: critical: Publish → Live; deprecated: Edit."),
        "{}",
        alternative.description
    );
    assert!(alternative.table.columns.contains(&"Role".to_owned()));
    let publish = alternative
        .table
        .rows
        .iter()
        .find(|row| row[0] == "Publish")
        .expect("a row");
    assert_eq!(publish.last().map(String::as_str), Some("Live (critical)"));
    let dashed = SPEC.replace(
        r#"{"from": "publish", "to": "done"}"#,
        r#"{"from": "publish", "to": "done", "role": "planned", "dash": "dotted"}"#,
    );
    assert_eq!(
        error(&dashed),
        ("role_conflict", "/flow/edges/4/dash".to_owned())
    );
}
