use super::*;
use crate::text_alternative;

/// One small diagram per type that takes emphasis, with `EXTRA` where top-level fields go.
const FLOW: &str = r#"{"schemaVersion": 1, "type": "flow", "title": "Checkout", "width": 800, "height": 500, EXTRA
    "flow": {"nodes": [
        {"id": "start", "label": "Cart", "kind": "start"},
        {"id": "pay", "label": "Pay"},
        {"id": "ok", "label": "Paid?", "kind": "decision"},
        {"id": "done", "label": "Done", "kind": "end"}],
    "edges": [{"from": "start", "to": "pay"}, {"from": "pay", "to": "ok"},
        {"from": "ok", "to": "done", "label": "yes"}, {"from": "ok", "to": "pay", "dash": "dashed"}],
    "mainPath": ["start", "pay", "ok", "done"]}}"#;

const STATE: &str = r#"{"schemaVersion": 1, "type": "state", "title": "Door", "width": 800, "height": 500, EXTRA
    "state": {"initial": "closed", "states": [
        {"id": "closed", "label": "Closed"}, {"id": "open", "label": "Open"},
        {"id": "box", "label": "Box", "kind": "composite"}, {"id": "locked", "label": "Locked", "in": "box"}],
    "transitions": [{"from": "closed", "to": "open", "event": "push"},
        {"from": "open", "to": "closed", "event": "pull", "dash": "dashed"},
        {"from": "closed", "to": "locked", "event": "lock"}]}}"#;

const ARCHITECTURE: &str = r#"{"schemaVersion": 1, "type": "architecture", "title": "Shop", "width": 800, "height": 500, EXTRA
    "architecture": {"components": [
        {"id": "web", "label": "Web", "kind": "frontend"}, {"id": "api", "label": "API"},
        {"id": "db", "label": "Orders", "kind": "database"}],
    "connections": [{"from": "web", "to": "api", "label": "calls"},
        {"from": "api", "to": "db", "label": "stores", "dash": "dashed"}]}}"#;

const CLASSES: &str = r#"{"schemaVersion": 1, "type": "classes", "title": "Model", "width": 800, "height": 500, EXTRA
    "classes": {"classes": [
        {"id": "order", "label": "Order", "attributes": ["+ id: Int"]},
        {"id": "line", "label": "Line", "attributes": ["+ qty: Int"]},
        {"id": "item", "label": "Item", "kind": "interface"}],
    "relations": [{"from": "order", "to": "line", "kind": "composition"},
        {"from": "line", "to": "item"}]}}"#;

const TREE: &str = r#"{"schemaVersion": 1, "type": "tree", "title": "Group", "width": 800, "height": 500, EXTRA
    "tree": {"nodes": [
        {"id": "top", "label": "Holding"},
        {"id": "a", "label": "Retail", "parent": "top"},
        {"id": "b", "label": "Logistics", "parent": "top", "kind": "external"},
        {"id": "c", "label": "Stores", "parent": "a"}]}}"#;

const NETWORK: &str = r#"{"schemaVersion": 1, "type": "network", "title": "Calls", "width": 800, "height": 500, EXTRA
    "network": {"layout": "circle", "directed": true, "nodes": [
        {"id": "a", "label": "Checkout"}, {"id": "b", "label": "Payments"}, {"id": "c", "label": "Ledger"}],
    "edges": [{"from": "a", "to": "b"}, {"from": "b", "to": "c", "dash": "dashed"}]}}"#;

const SEQUENCE: &str = r#"{"schemaVersion": 1, "type": "sequence", "title": "Pay", "width": 800, "height": 500, EXTRA
    "sequence": {"participants": [
        {"id": "web", "label": "Web"}, {"id": "api", "label": "API"}, {"id": "bank", "label": "Bank"}],
    "messages": [{"from": "web", "to": "api", "label": "pay"},
        {"from": "api", "to": "bank", "label": "charge"},
        {"from": "bank", "to": "api", "label": "ok", "kind": "reply"},
        {"from": "api", "to": "web", "label": "receipt", "kind": "async"}]}}"#;

/// The emphasis each diagram gets: one node and a path of two or three of its nodes.
const EMPHASIS: [(&str, &str, &str); 7] = [
    (
        "flow",
        FLOW,
        r#""emphasis": {"nodes": ["ok"], "path": ["start", "pay", "ok"]},"#,
    ),
    (
        "state",
        STATE,
        r#""emphasis": {"nodes": ["open"], "path": ["closed", "open"]},"#,
    ),
    (
        "architecture",
        ARCHITECTURE,
        r#""emphasis": {"nodes": ["api"], "path": ["web", "api", "db"]},"#,
    ),
    (
        "classes",
        CLASSES,
        r#""emphasis": {"nodes": ["line"], "path": ["order", "line"]},"#,
    ),
    (
        "tree",
        TREE,
        r#""emphasis": {"nodes": ["b"], "path": ["top", "a", "c"]},"#,
    ),
    (
        "network",
        NETWORK,
        r#""emphasis": {"nodes": ["b"], "path": ["a", "b", "c"]},"#,
    ),
    (
        "sequence",
        SEQUENCE,
        r#""emphasis": {"nodes": ["bank"], "path": ["web", "api"]},"#,
    ),
];

fn with(base: &str, extra: &str) -> String {
    base.replace("EXTRA", extra)
}

fn alternative(json: &str) -> crate::TextAlternative {
    let spec = ChartSpec::from_json(json).expect("a specification");
    text_alternative(&spec).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn without_emphasis_or_legend_nothing_changes() {
    for (kind, base, _) in EMPHASIS {
        let svg = render_ok(&with(base, "")).content;
        assert!(!svg.contains("chartlet-emph"), "{kind}");
        assert!(!svg.contains("chartlet-legend-"), "{kind}");
        // An explicit "none" is the same as no legend.
        let none = render_ok(&with(base, r#""legend": "none","#)).content;
        assert_eq!(svg, none, "{kind}");
    }
}

#[test]
fn every_diagram_marks_the_emphasized_nodes_and_the_edges_of_the_path() {
    for (kind, base, emphasis) in EMPHASIS {
        let output = render_ok(&with(base, emphasis));
        assert!(output.warnings.is_empty(), "{kind}: {:?}", output.warnings);
        let svg = output.content;
        assert!(
            svg.contains("chartlet-emph\">") || svg.contains("chartlet-emph\""),
            "{kind}: an emphasized node"
        );
        assert!(
            svg.contains("chartlet-emph-edge\""),
            "{kind}: an edge of the path"
        );
        // Weight says it without color: the outline and the line are heavier than the rest.
        assert!(
            svg.contains("stroke:var(--chartlet-accent);stroke-width:3.5"),
            "{kind}"
        );
        assert!(svg.contains("stroke-width:3.2"), "{kind}");
    }
}

#[test]
fn the_ring_of_an_emphasized_network_node_is_drawn_around_its_dot() {
    let svg = render_ok(&with(NETWORK, EMPHASIS[5].2)).content;
    assert_eq!(svg.matches("class=\"chartlet-emph-ring\"").count(), 1);
}

#[test]
fn a_path_given_makes_the_main_path_neutral_and_none_leaves_it_in_the_accent() {
    let plain = render_ok(&with(FLOW, "")).content;
    assert!(plain.contains("chartlet-flow-main"));
    assert!(!plain.contains("chartlet-emph-quiet\""));
    let nodes_only = render_ok(&with(FLOW, r#""emphasis": {"nodes": ["pay"]},"#)).content;
    assert!(!nodes_only.contains("chartlet-emph-quiet\""));
    assert!(!nodes_only.contains("chartlet-emph-edge\""));
    let path = render_ok(&with(FLOW, r#""emphasis": {"path": ["pay", "ok"]},"#)).content;
    assert!(path.contains("chartlet-emph-quiet\""));
    assert!(path.contains("chartlet-emph-edge\""));
}

#[test]
fn the_description_and_the_table_name_what_is_emphasized() {
    for (kind, base, emphasis) in EMPHASIS {
        let json = with(base, emphasis);
        let text = alternative(&json);
        assert!(
            text.description.contains("Emphasized: "),
            "{kind}: {}",
            text.description
        );
        assert!(
            text.description.contains("Emphasized path: "),
            "{kind}: {}",
            text.description
        );
        assert_eq!(
            text.table.columns.last().map(String::as_str),
            Some("Emphasis"),
            "{kind}"
        );
        let cells: Vec<&String> = text
            .table
            .rows
            .iter()
            .map(|row| row.last().expect("a row"))
            .collect();
        assert!(
            cells.iter().any(|cell| cell.contains("on emphasized path")),
            "{kind}: {cells:?}"
        );
        assert!(
            kind == "sequence" || cells.iter().any(|cell| cell.starts_with("emphasized")),
            "{kind}: {cells:?}"
        );
        // Without emphasis the table has no such column.
        let plain = alternative(&with(base, ""));
        assert!(
            !plain.table.columns.contains(&"Emphasis".to_owned()),
            "{kind}"
        );
        assert!(!plain.description.contains("Emphasized"), "{kind}");
    }
}

#[test]
fn the_texts_are_german_in_a_german_chart() {
    let json = german(&with(FLOW, EMPHASIS[0].2), "flow");
    let text = alternative(&json);
    assert!(
        text.description.contains("Hervorgehoben: Paid?"),
        "{}",
        text.description
    );
    assert!(text.description.contains("Hervorgehobener Weg: "));
    assert!(text.table.columns.contains(&"Hervorhebung".to_owned()));
}

#[test]
fn the_names_of_a_sequence_diagram_are_those_of_its_participants() {
    let text = alternative(&with(SEQUENCE, EMPHASIS[6].2));
    assert!(
        text.description.contains("Emphasized: Bank."),
        "{}",
        text.description
    );
    assert!(text.description.contains("Emphasized path: Web → API."));
    // The messages between the two participants of the path are marked in the table.
    let marked: Vec<&Vec<String>> = text
        .table
        .rows
        .iter()
        .filter(|row| row.last().is_some_and(|cell| !cell.is_empty()))
        .collect();
    assert_eq!(marked.len(), 2, "{marked:?}");
}

#[test]
fn an_unknown_node_is_refused_at_its_place_with_the_code_unknown_node() {
    for (kind, base, _) in EMPHASIS {
        let (code, path) = render_err(&with(base, r#""emphasis": {"nodes": ["nope"]},"#));
        assert_eq!(
            (code, path.as_str()),
            ("unknown_node", "/emphasis/nodes/0"),
            "{kind}"
        );
        let (code, path) = render_err(&with(base, r#""emphasis": {"path": ["nope", "nope2"]},"#));
        assert_eq!(
            (code, path.as_str()),
            ("unknown_node", "/emphasis/path/0"),
            "{kind}"
        );
    }
}

#[test]
fn a_path_must_follow_edges_and_have_two_steps() {
    let (code, path) = render_err(&with(FLOW, r#""emphasis": {"path": ["start", "done"]},"#));
    assert_eq!(
        (code, path.as_str()),
        ("emphasis_path_gap", "/emphasis/path/1")
    );
    let (code, path) = render_err(&with(FLOW, r#""emphasis": {"path": ["start"]},"#));
    assert_eq!(
        (code, path.as_str()),
        ("emphasis_path_short", "/emphasis/path")
    );
    let (code, path) = render_err(&with(SEQUENCE, r#""emphasis": {"path": ["web", "bank"]},"#));
    assert_eq!(
        (code, path.as_str()),
        ("emphasis_path_gap", "/emphasis/path/1")
    );
    // The composite state of a state diagram is a frame, not a node.
    let (code, path) = render_err(&with(STATE, r#""emphasis": {"nodes": ["box"]},"#));
    assert_eq!((code, path.as_str()), ("unknown_node", "/emphasis/nodes/0"));
}

#[test]
fn a_node_is_named_once_and_unknown_fields_are_refused() {
    let (code, path) = render_err(&with(FLOW, r#""emphasis": {"nodes": ["pay", "pay"]},"#));
    assert_eq!(
        (code, path.as_str()),
        ("duplicate_emphasis", "/emphasis/nodes/1")
    );
    let (code, _) = render_err(&with(
        FLOW,
        r#""emphasis": {"nodes": ["pay"], "color": "red"},"#,
    ));
    assert_eq!(code, "invalid_spec");
}

#[test]
fn more_than_two_emphasized_nodes_warn_with_too_much_emphasis() {
    let json = with(FLOW, r#""emphasis": {"nodes": ["start", "pay", "ok"]},"#);
    let output = render_ok(&json);
    let warning = output
        .warnings
        .iter()
        .find(|warning| warning.code == "too_much_emphasis")
        .expect("a warning");
    assert_eq!(warning.path, "/emphasis/nodes");
    let two = render_ok(&with(FLOW, r#""emphasis": {"nodes": ["start", "pay"]},"#));
    assert!(
        two.warnings
            .iter()
            .all(|warning| warning.code != "too_much_emphasis")
    );
}

#[test]
fn a_chart_that_is_no_diagram_refuses_emphasis_and_the_diagram_legend() {
    let bar = r#"{"schemaVersion": 1, "type": "bar", "title": "T", EXTRA "data": [{"label": "A", "value": 1}]}"#;
    let (code, path) = render_err(&with(bar, r#""emphasis": {"nodes": ["a"]},"#));
    assert_eq!((code, path.as_str()), ("option_not_supported", "/emphasis"));
    let (code, path) = render_err(&with(bar, r#""legend": "auto","#));
    assert_eq!((code, path.as_str()), ("option_not_supported", "/legend"));
    let (code, path) = render_err(&with(FLOW, r#""legend": "end","#));
    assert_eq!((code, path.as_str()), ("option_not_supported", "/legend"));
}

#[test]
fn a_legend_below_the_drawing_lists_what_occurs_and_grows_the_canvas() {
    for (kind, base, emphasis) in EMPHASIS {
        let json = with(base, &format!(r#"{emphasis} "legend": "auto","#));
        let svg = render_ok(&json).content;
        assert!(svg.contains("class=\"chartlet-legend-text\""), "{kind}");
        assert!(svg.contains(">Emphasized</text>"), "{kind}");
        assert!(svg.contains(">Emphasized path</text>"), "{kind}");
        // The strip is part of the canvas: it grows by the strip.
        let without = render_ok(&with(base, emphasis)).content;
        assert!(height_of(&svg) > height_of(&without), "{kind}");
        let text = alternative(&json);
        assert!(text.description.contains("Legend: "), "{kind}");
        assert!(text.description.contains("emphasized path"), "{kind}");
    }
}

fn height_of(svg: &str) -> u32 {
    let tag = &svg[..svg.find('>').expect("an svg tag")];
    let after = tag.split("height=\"").nth(1).expect("a height");
    after
        .split('"')
        .next()
        .expect("a value")
        .parse()
        .expect("a number")
}

#[test]
fn the_legend_names_the_kinds_and_line_patterns_of_each_diagram() {
    let flow = render_ok(&with(FLOW, r#""legend": "auto","#)).content;
    for name in [
        "Start",
        "End",
        "Step",
        "Decision",
        "Solid line",
        "Dashed line",
        "Main path",
    ] {
        assert!(flow.contains(&format!(">{name}</text>")), "{name}");
    }
    let sequence = render_ok(&with(SEQUENCE, r#""legend": "auto","#)).content;
    for name in ["Call", "Reply", "Asynchronous"] {
        assert!(sequence.contains(&format!(">{name}</text>")), "{name}");
    }
    let architecture = render_ok(&with(ARCHITECTURE, r#""legend": "auto","#)).content;
    for name in [
        "Frontend",
        "Service",
        "Database",
        "Solid line",
        "Dashed line",
    ] {
        assert!(architecture.contains(&format!(">{name}</text>")), "{name}");
    }
    let state = render_ok(&with(STATE, r#""legend": "auto","#)).content;
    assert!(state.contains(">State</text>") && state.contains(">Dashed line</text>"));
    let tree = render_ok(&with(TREE, r#""legend": "auto","#)).content;
    assert!(tree.contains(">Unit</text>") && tree.contains(">External</text>"));
    let german = render_ok(&german(&with(FLOW, r#""legend": "auto","#), "flow")).content;
    assert!(german.contains(">Gestrichelte Linie</text>"));
}

#[test]
fn a_diagram_with_one_kind_and_one_line_needs_no_legend() {
    let json = r#"{"schemaVersion": 1, "type": "flow", "title": "T", "width": 600, "height": 400, "legend": "auto",
        "flow": {"nodes": [{"id": "a", "label": "A"}, {"id": "b", "label": "B"}], "edges": [{"from": "a", "to": "b"}]}}"#;
    assert!(
        !render_ok(json)
            .content
            .contains("class=\"chartlet-legend-text\"")
    );
    assert!(!alternative(json).description.contains("Legend"));
}

#[test]
fn compact_diagrams_drop_the_legend_and_the_description_says_so() {
    let json = with(FLOW, r#""legend": "auto","#)
        .replace(r#""mainPath""#, r#""spacing": "compact", "mainPath""#);
    let svg = render_ok(&json).content;
    assert!(!svg.contains("class=\"chartlet-legend-text\""));
    assert!(
        alternative(&json)
            .description
            .contains("The legend is left out in compact mode.")
    );
}

#[test]
fn the_legend_wraps_into_rows_on_a_narrow_canvas() {
    let wide = render_ok(&with(ARCHITECTURE, r#""legend": "auto","#)).content;
    let narrow = ARCHITECTURE
        .replace("\"width\": 800", "\"width\": 300")
        .replace("EXTRA", r#""legend": "auto","#);
    let narrow = render_ok(&narrow).content;
    let rows = |svg: &str| {
        let mut ys: Vec<String> = placed_texts(svg, "chartlet-legend-text")
            .into_iter()
            .map(|(_, y, _)| format!("{y:.0}"))
            .collect();
        ys.dedup();
        ys.len()
    };
    assert!(
        rows(&narrow) > rows(&wide),
        "{} {}",
        rows(&narrow),
        rows(&wide)
    );
    // No entry leaves the canvas.
    for (x, _, _) in placed_texts(&narrow, "chartlet-legend-text") {
        assert!(x < 300.0);
    }
}

#[test]
fn the_legend_lies_below_everything_the_diagram_draws() {
    let json = with(FLOW, r#""legend": "auto","#);
    let plain = height_of(&render_ok(&with(FLOW, "")).content);
    for (_, y, _) in placed_texts(&render_ok(&json).content, "chartlet-legend-text") {
        assert!(y > f64::from(plain) - 4.0, "{y} against {plain}");
    }
}

#[test]
fn the_stylesheet_of_emphasis_and_legend_comes_with_the_diagram_types() {
    let sheet = crate::stylesheet_for(&[crate::ChartType::Flow, crate::ChartType::Sequence]);
    assert!(sheet.contains(".chartlet-emph-edge .chartlet-flow-edge"));
    assert!(sheet.contains(".chartlet-legend-text{"));
}
