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
fn interleaved_groups_keep_a_plausible_canvas_and_name_the_overlap() {
    let json = r#"{
        "schemaVersion": 1, "type": "flow", "title": "Layers", "width": 600, "height": 400,
        "flow": {
            "orientation": "landscape",
            "nodes": [
                {"id": "mq", "label": "media-query-parse"}, {"id": "xp", "label": "xpath-eval"},
                {"id": "se", "label": "schematron-engine"}, {"id": "sp", "label": "stylesheet-parse"},
                {"id": "hc", "label": "html-conform"}, {"id": "ar", "label": "a11y-rules"},
                {"id": "wc", "label": "web-checks"}
            ],
            "edges": [
                {"from": "sp", "to": "ar"}, {"from": "xp", "to": "se"}, {"from": "mq", "to": "sp"}
            ],
            "groups": [
                {"label": "L0", "nodes": ["mq", "xp", "se"]},
                {"label": "L1", "nodes": ["sp", "hc"]},
                {"label": "L2", "nodes": ["ar", "wc"]}
            ]
        }
    }"#;
    for orientation in ["landscape", "portrait"] {
        let json = json.replace("landscape", orientation);
        let output = svg(&json);
        let height: u64 = output
            .content
            .split(" height=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .and_then(|value| value.parse().ok())
            .expect("a height");
        assert!(height < 5000, "{orientation}: {height}px");
        assert!(
            output.warnings.iter().any(|w| w.code == "group_overlap"),
            "{orientation}: {:?}",
            output.warnings
        );
    }
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn random_groups_never_ask_for_an_absurd_canvas() {
    let mut state: u64 = 12345;
    let mut next = move |bound: u64| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) % bound
    };
    for case in 0..400 {
        let count = 5 + next(12) as usize;
        let nodes: Vec<String> = (0..count)
            .map(|i| format!(r#"{{"id":"n{i}","label":"Node {i}"}}"#))
            .collect();
        let mut edges: Vec<String> = (1..count)
            .map(|i| format!(r#"{{"from":"n{}","to":"n{i}"}}"#, next(i as u64)))
            .collect();
        for _ in 0..next(6) {
            let (from, to) = (next(count as u64), next(count as u64));
            if from != to {
                edges.push(format!(r#"{{"from":"n{from}","to":"n{to}"}}"#));
            }
        }
        let mut members: Vec<Vec<usize>> = vec![vec![]; 1 + next(4) as usize];
        for node in 0..count {
            if next(4) != 0 {
                let group = next(members.len() as u64) as usize;
                members[group].push(node);
            }
        }
        let groups: Vec<String> = members
            .iter()
            .filter(|nodes| !nodes.is_empty())
            .enumerate()
            .map(|(group, nodes)| {
                let ids: Vec<String> = nodes.iter().map(|node| format!("\"n{node}\"")).collect();
                format!(r#"{{"label":"G{group}","nodes":[{}]}}"#, ids.join(","))
            })
            .collect();
        let (width, height) = [(1000, 700), (470, 800), (800, 500)][next(3) as usize];
        let json = format!(
            r#"{{"schemaVersion":1,"type":"flow","title":"F","width":{width},"height":{height},
"flow":{{"nodes":[{}],"edges":[{}],"groups":[{}]}}}}"#,
            nodes.join(","),
            edges.join(","),
            groups.join(",")
        );
        let output = render_json(&json, RenderFormat::Svg, &RenderOptions::default())
            .unwrap_or_else(|error| {
                panic!("case {case}: {} {}\n{json}", error.code, error.message)
            });
        for warning in output
            .warnings
            .iter()
            .filter(|warning| warning.code == "canvas_too_small")
        {
            let needed: u64 = warning
                .message
                .split("needs ")
                .nth(1)
                .and_then(|rest| rest.split(' ').next())
                .and_then(|value| value.parse().ok())
                .expect("a pixel count");
            assert!(needed < 20_000, "case {case}: {needed}px\n{json}");
        }
    }
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

/// A layer whose steps the specification lists with a stranger between the members of a group
/// is reordered, even when that order has no crossings: the frame keeps the stranger out and the
/// canvas keeps its size.
#[test]
fn a_group_split_by_the_order_of_the_specification_stays_together() {
    let json = r#"{
        "schemaVersion": 1, "type": "architecture", "title": "Split", "width": 900, "height": 600,
        "architecture": {
            "boundaries": [
                {"id": "ordering", "label": "Ordering"},
                {"id": "billing", "label": "Billing"}
            ],
            "components": [
                {"id": "order", "label": "Order", "in": "ordering"},
                {"id": "line", "label": "Line", "in": "ordering"},
                {"id": "payment", "label": "Payment", "in": "billing"},
                {"id": "status", "label": "Status", "in": "ordering"},
                {"id": "card", "label": "Card", "in": "billing"}
            ],
            "connections": [
                {"from": "order", "to": "line"},
                {"from": "order", "to": "payment"},
                {"from": "order", "to": "status"},
                {"from": "payment", "to": "card"}
            ]
        }
    }"#;
    let output = svg(json);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
}

const FULFILMENT: &str = include_str!("../../examples/order-fulfilment.json");

fn branches(content: &str) -> usize {
    content
        .matches("r=\"3\" class=\"chartlet-flow-head\"")
        .count()
}

/// Where each edge whose tooltip starts with `from` leaves its step.
fn starts(content: &str, from: &str) -> Vec<String> {
    content
        .split("<polyline points=\"")
        .filter(|line| line.contains(&format!("<title>{from} → ")))
        .map(|line| line.split(' ').next().expect("a point").to_owned())
        .collect()
}

/// The points of the line of the edge with `title`, from where it starts.
fn edge_points(svg: &str, title: &str) -> Vec<(f64, f64)> {
    let chunks: Vec<&str> = svg.split('<').collect();
    let index = chunks
        .iter()
        .position(|chunk| *chunk == format!("title>{title}"))
        .unwrap_or_else(|| panic!("{title} is drawn"));
    let line = chunks[index - 1];
    let start = line.find("points=\"").expect("a line has points") + 8;
    line[start..]
        .split('"')
        .next()
        .expect("points")
        .split(' ')
        .map(|pair| {
            let (x, y) = pair.split_once(',').expect("a point");
            (x.parse().expect("x"), y.parse().expect("y"))
        })
        .collect()
}

#[test]
fn shared_trunks_in_a_flow_chart_branch_and_keep_every_edge() {
    let separate = FULFILMENT.replace("\"sharedTrunks\": true", "\"sharedTrunks\": false");
    let unset = FULFILMENT.replace(",\n    \"sharedTrunks\": true", "");
    assert_ne!(unset, FULFILMENT);
    assert!(
        svg(FULFILMENT).warnings.is_empty(),
        "{:?}",
        svg(FULFILMENT).warnings
    );
    assert_eq!(svg(&separate).content, svg(&unset).content);
    assert_eq!(branches(&svg(&unset).content), 0);
    for orientation in ["portrait", "landscape"] {
        let json = FULFILMENT.replace(
            "\"sharedTrunks\"",
            &format!("\"orientation\": \"{orientation}\", \"sharedTrunks\""),
        );
        let output = svg(&json);
        // Out of the split: the trunk turns to either side; into packing likewise.
        assert_eq!(branches(&output.content), 4, "{orientation}");
        assert_eq!(svg(&json).content, output.content);
        for title in [
            "<title>Split into tasks → Reserve stock</title>",
            "<title>Check fraud score → Pack parcel</title>",
            "<title>Express? → Send by post: no</title>",
            "<title>Send by post → Shipped</title>",
        ] {
            assert_eq!(output.content.matches(title).count(), 1, "{title}");
        }
    }
    let shared =
        text_alternative(&ChartSpec::from_json(FULFILMENT).expect("parses")).expect("valid");
    let own = text_alternative(&ChartSpec::from_json(&unset).expect("parses")).expect("valid");
    assert_eq!(shared.description, own.description);
    assert_eq!(shared.table.rows, own.table.rows);
}

#[test]
fn decision_branches_labels_and_lanes_keep_a_line_of_their_own() {
    // Without their labels and off the main path, the branches of a decision still keep a line
    // each.
    let unlabeled = FULFILMENT
        .replace(r#", "label": "yes""#, "")
        .replace(r#", "label": "no""#, "")
        .replace(r#", "courier", "shipped"]"#, "]");
    let content = svg(&unlabeled).content;
    assert!(content.contains("<title>Express? → Hand to courier</title>"));
    let decision = starts(&content, "Express?");
    assert_eq!(decision.len(), 2);
    assert_ne!(decision[0], decision[1]);
    // A label takes an edge off the trunk; the others keep sharing theirs.
    let labeled = FULFILMENT.replace(
        r#"{ "from": "split", "to": "stock" }"#,
        r#"{ "from": "split", "to": "stock", "label": "first" }"#,
    );
    let split = starts(&svg(&labeled).content, "Split into tasks");
    assert_ne!(split[0], split[2]);
    assert_eq!(split[2], split[3]);
    // Steps in another lane do not share a trunk with the rest.
    let mut lanes = FULFILMENT.replace(
        r#""groups": [{ "label": "Checks", "nodes": ["stock", "payment", "fraud", "address"] }]"#,
        r#""lanes": [{"id": "a", "label": "Warehouse"}, {"id": "b", "label": "Finance"}]"#,
    );
    for id in [
        "received", "split", "stock", "payment", "fraud", "address", "pack", "express", "courier",
        "post", "shipped",
    ] {
        let lane = if matches!(id, "fraud" | "address") {
            "b"
        } else {
            "a"
        };
        lanes = lanes.replace(
            &format!(r#"{{ "id": "{id}", "#),
            &format!(r#"{{ "id": "{id}", "lane": "{lane}", "#),
        );
    }
    let lanes = lanes.replace("\"height\": 700", "\"height\": 720");
    let output = svg(&lanes);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    // Payment is on the main path; the two Finance checks share a trunk, stock goes alone.
    let split = starts(&output.content, "Split into tasks");
    assert_eq!(split.len(), 4);
    assert_eq!(split[2], split[3]);
    assert_ne!(split[0], split[2]);
    let unlaned = starts(&svg(FULFILMENT).content, "Split into tasks");
    assert_eq!(unlaned[0], unlaned[2]);
    assert_eq!(svg(&lanes).content, output.content);
}

#[test]
fn an_edge_meets_the_sides_it_names_in_either_orientation_and_on_a_phone() {
    let json = SPEC
        .replace(
            r#""label": "again"}"#,
            r#""label": "again", "fromSide": "right", "toSide": "right"}"#,
        )
        .replacen('{', r#"{"mobile": {"width": 360, "height": 640},"#, 1);
    for (orientation, variant) in [
        ("portrait", crate::Variant::Desktop),
        ("landscape", crate::Variant::Desktop),
        ("portrait", crate::Variant::Mobile),
    ] {
        let json = json.replace(
            r#""mainPath""#,
            &format!(r#""orientation": "{orientation}", "mainPath""#),
        );
        let options = RenderOptions {
            variant,
            ..RenderOptions::default()
        };
        let output = render_json(&json, RenderFormat::Svg, &options).expect("renders");
        assert!(
            output
                .warnings
                .iter()
                .all(|warning| warning.code == "canvas_too_small"),
            "{:?}",
            output.warnings
        );
        let again = render_json(&json, RenderFormat::Svg, &options).expect("renders");
        assert_eq!(again.content, output.content, "{orientation} {variant:?}");
        let centers = centers(&output.content);
        let center = |name: &str| {
            let (_, x, y) = centers
                .iter()
                .find(|(title, _, _)| title == name)
                .unwrap_or_else(|| panic!("{name} is drawn"));
            (*x, *y)
        };
        let points = edge_points(&output.content, "Edit → Looks good?: again");
        let (first, last) = (points[0], points[points.len() - 1]);
        // Right is right on the page in portrait and the bottom in landscape: the edge leaves
        // Edit and reaches the decision there, level with their middles.
        for (point, name) in [(first, "Edit"), (last, "Looks good?")] {
            let (x, y) = center(name);
            let beside = if orientation == "landscape" {
                (point.0 - x).abs() < 0.01 && point.1 > y + 10.0
            } else {
                (point.1 - y).abs() < 0.01 && point.0 > x + 10.0
            };
            assert!(beside, "{orientation} {variant:?}: {name} at {point:?}");
        }
    }
}

#[test]
fn a_side_along_the_flow_that_turns_an_edge_back_is_reported() {
    // Where the flow leads anyway, a side changes nothing.
    let along = SPEC.replace(
        r#"{"from": "start", "to": "check"}"#,
        r#"{"from": "start", "to": "check", "fromSide": "end", "toSide": "start"}"#,
    );
    let output = svg(&along);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let first = |content: &str| edge_points(content, "Draft → Looks good?");
    assert_eq!(first(&output.content), first(&svg(SPEC).content));
    // The edge from Edit back to the decision leaves Edit on its start side, towards the steps
    // before, and reaches the decision on its end side; the other way round would turn it back
    // over both.
    let back = SPEC.replace(
        r#""label": "again"}"#,
        r#""label": "again", "fromSide": "end", "toSide": "start"}"#,
    );
    let output = svg(&back);
    let warnings: Vec<(&str, &str)> = output
        .warnings
        .iter()
        .map(|warning| (warning.code, warning.path.as_str()))
        .collect();
    assert_eq!(
        warnings,
        [
            ("port_side_ignored", "/flow/edges/3/fromSide"),
            ("port_side_ignored", "/flow/edges/3/toSide")
        ]
    );
    let back = |content: &str| edge_points(content, "Edit → Looks good?: again");
    assert_eq!(back(&output.content), back(&svg(SPEC).content));
}

#[test]
fn a_loop_takes_no_side() {
    let json = SPEC.replace(
        r#"{"from": "publish", "to": "done"}"#,
        r#"{"from": "publish", "to": "done"}, {"from": "edit", "to": "edit", "toSide": "left"}"#,
    );
    assert_eq!(
        error(&json),
        ("option_not_supported", "/flow/edges/5/toSide".to_owned())
    );
    let json = SPEC.replace(r#""to": "done"}"#, r#""to": "done", "fromSide": "up"}"#);
    assert_eq!(
        error(&json),
        ("invalid_spec", "/flow/edges/4/fromSide".to_owned())
    );
}

#[test]
fn spacing_profiles_scale_the_gaps_and_nothing_else() {
    let profile = |spacing: &str, variant: crate::Variant| {
        let json = SPEC
            .replace(
                r#""mainPath""#,
                &format!(r#""orientation": "portrait", "spacing": "{spacing}", "mainPath""#),
            )
            .replacen('{', r#"{"mobile": {"width": 360, "height": 640},"#, 1);
        let options = RenderOptions {
            variant,
            ..RenderOptions::default()
        };
        let output = render_json(&json, RenderFormat::Svg, &options).expect("renders");
        let again = render_json(&json, RenderFormat::Svg, &options).expect("renders");
        assert_eq!(output.content, again.content, "{spacing} is deterministic");
        output.content
    };
    for variant in [crate::Variant::Desktop, crate::Variant::Mobile] {
        let measures = |content: &str| {
            let centers = centers(content);
            let at = |name: &str| {
                let (_, x, y) = centers
                    .iter()
                    .find(|(title, _, _)| title == name)
                    .unwrap_or_else(|| panic!("{name} is drawn"));
                (*x, *y)
            };
            // The distance between two layers, and between two neighbours in a layer.
            (
                at("Looks good?").1 - at("Draft").1,
                at("Publish").0 - at("Edit").0,
            )
        };
        let compact = profile("compact", variant);
        let normal = profile("normal", variant);
        let roomy = profile("roomy", variant);
        let (compact, normal, roomy) = (measures(&compact), measures(&normal), measures(&roomy));
        assert!(compact.0 < normal.0 && normal.0 < roomy.0, "{variant:?}");
        assert!(normal.1 < roomy.1, "{variant:?}");
        assert!(compact.1 <= normal.1, "{variant:?}");
    }
    // The steps and their texts keep their size.
    let box_of = |content: &str| {
        let start = content.find("<title>Edit</title>").expect("Edit is drawn");
        let rect = &content[..start];
        let rect = &rect[rect.rfind("<rect").expect("a box")..];
        rect[rect.find("width").expect("a width")..]
            .split(' ')
            .take(2)
            .collect::<Vec<_>>()
            .join(" ")
    };
    let normal = profile("normal", crate::Variant::Desktop);
    for spacing in ["compact", "roomy"] {
        assert_eq!(
            box_of(&profile(spacing, crate::Variant::Desktop)),
            box_of(&normal)
        );
    }
    // Normal is the default.
    let pinned = SPEC.replace(r#""mainPath""#, r#""spacing": "normal", "mainPath""#);
    let points = |content: &str| edge_points(content, "Edit → Looks good?: again");
    assert_eq!(points(&svg(&pinned).content), points(&svg(SPEC).content));
    let json = SPEC.replace(r#""mainPath""#, r#""spacing": "tight", "mainPath""#);
    assert_eq!(error(&json), ("invalid_spec", "/flow/spacing".to_owned()));
}

#[test]
fn class_state_and_architecture_diagrams_take_a_spacing_profile() {
    for (json, block) in [
        (include_str!("../../examples/order-model.json"), "classes"),
        (include_str!("../../examples/ticket-states.json"), "state"),
        (
            include_str!("../../examples/shop-architecture.json"),
            "architecture",
        ),
    ] {
        let with = |spacing: &str| {
            let open = format!("\"{block}\": {{");
            assert!(json.contains(&open), "{block}");
            svg(&json.replacen(&open, &format!("{open} \"spacing\": \"{spacing}\","), 1))
        };
        let normal = with("normal");
        let drawing = |content: &str| {
            content
                .split_once("</style>")
                .map(|(_, rest)| rest.to_owned())
        };
        assert_eq!(
            drawing(&normal.content),
            drawing(&svg(json).content),
            "{block}"
        );
        for spacing in ["compact", "roomy"] {
            assert_ne!(with(spacing).content, normal.content, "{block} {spacing}");
        }
    }
}
#[test]
fn notes_explain_a_step_or_the_whole_chart_without_adding_an_edge() {
    let json = SPEC.replace(
        r#""mainPath""#,
        r#""notes": [
            {"on": "edit", "text": "At most two rounds"},
            {"text": "Applies to every article"}
        ],
        "mainPath""#,
    );
    let plain = svg(SPEC);
    let output = svg(&json);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    for text in ["At most two rounds", "Applies to every article"] {
        assert!(output.content.contains(&format!("<title>{text}</title>")));
    }
    // Only the note on a step has a line, a dotted one without an arrowhead.
    assert_eq!(
        output
            .content
            .matches("chartlet-flow-edge chartlet-flow-dotted")
            .count(),
        1
    );
    assert_eq!(
        output.content.matches("chartlet-flow-head").count(),
        plain.content.matches("chartlet-flow-head").count()
    );
    let plain = text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
    let noted = text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
    assert_eq!(
        noted.description,
        format!(
            "{} Note on Edit: At most two rounds. Note: Applies to every article.",
            plain.description
        )
    );
    assert_eq!(
        noted.table.columns.last().map(String::as_str),
        Some("Notes")
    );
    let edit = noted
        .table
        .rows
        .iter()
        .find(|row| row[0] == "Edit")
        .expect("a row");
    assert_eq!(edit.last().map(String::as_str), Some("At most two rounds"));
    assert_eq!(
        noted.table.caption,
        format!("{}. Note: Applies to every article.", plain.table.caption)
    );
    let unknown = json.replace(r#"{"on": "edit""#, r#"{"on": "nowhere""#);
    assert_eq!(
        error(&unknown),
        ("unknown_node", "/flow/notes/0/on".to_owned())
    );
}

#[test]
fn note_lines_never_join_a_shared_trunk_in_a_flow_chart() {
    let json = FULFILMENT.replace(
        "\"sharedTrunks\"",
        r#""notes": [
            {"on": "split", "text": "One task per team"},
            {"on": "split", "text": "Started at once"},
            {"text": "Same day for orders before noon"}
        ],
        "sharedTrunks""#,
    );
    let output = svg(&json);
    assert_eq!(
        branches(&output.content),
        branches(&svg(FULFILMENT).content)
    );
    let start = |line: &str| line.split(' ').next().expect("a point").to_owned();
    let notes: Vec<String> = output
        .content
        .split("<polyline points=\"")
        .filter(|line| line.contains("chartlet-flow-dotted\"/>"))
        .map(start)
        .collect();
    assert_eq!(notes.len(), 2);
    let trunk = starts(&output.content, "Split into tasks")[0].clone();
    assert!(notes.iter().all(|note| *note != trunk));
    assert_eq!(svg(&json).content, output.content);
}
