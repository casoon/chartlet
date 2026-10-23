use super::*;

#[test]
fn a_boundary_is_as_wide_as_its_name_and_a_self_message_of_the_last_participant_finds_room() {
    let architecture = r#"{"schemaVersion": 1, "type": "architecture", "title": "S", "width": 900, "height": 400,
            "architecture": {"boundaries": [{"id": "s", "label": "V8 sandbox without network and file system"}],
            "components": [{"id": "m", "label": "Model"}, {"id": "c", "label": "Code", "in": "s"}],
            "connections": [{"from": "m", "to": "c"}]}}"#;
    let output = render_ok(architecture);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert!(
        output
            .content
            .contains(">V8 sandbox without network and file system</text>")
    );
    let sequence = r#"{"schemaVersion": 1, "type": "sequence", "title": "S", "width": 470, "height": 500,
            "sequence": {"participants": [{"id": "a", "label": "A"}, {"id": "b", "label": "B"}, {"id": "c", "label": "Infrastructure"}],
            "messages": [{"from": "a", "to": "c", "label": "call"}, {"from": "c", "to": "c", "label": "executing now"}]}}"#;
    let output = render_ok(sequence);
    assert!(
        output
            .warnings
            .iter()
            .all(|warning| warning.code != "text_truncated"),
        "{:?}",
        output.warnings
    );
    assert!(output.content.contains(">executing now</text>"));
}

/// The width a render asks for with `canvas_too_small`, if it does.
fn asked_width(json: &str) -> Option<u32> {
    let warnings = render_ok(json).warnings;
    let warning = warnings
        .iter()
        .find(|warning| warning.code == "canvas_too_small" && warning.path == "/width")?;
    let after = warning.message.split("raise width to ").nth(1)?;
    after.split(',').next()?.parse().ok()
}

#[test]
fn the_width_a_diagram_asks_for_is_a_width_that_works() {
    let sequence = |width: u32| {
        format!(
            r#"{{"schemaVersion": 1, "type": "sequence", "title": "S", "width": {width}, "height": 600,
                "sequence": {{"participants": [{{"id": "a", "label": "A"}}, {{"id": "b", "label": "B"}}, {{"id": "c", "label": "Infrastructure"}}],
                "messages": [{{"from": "a", "to": "c", "label": "call"}}, {{"from": "c", "to": "c", "label": "run the tool locally for half a second"}}]}}}}"#
        )
    };
    let asked = asked_width(&sequence(400)).expect("400 pixels are too few");
    assert_eq!(asked_width(&sequence(asked)), None, "{asked}");
    let architecture = |width: u32| {
        format!(
            r#"{{"schemaVersion": 1, "type": "architecture", "title": "A", "width": {width}, "height": 500,
                "architecture": {{"orientation": "landscape",
                "components": [{{"id": "a", "label": "Application"}}, {{"id": "b", "label": "Database", "kind": "database"}}, {{"id": "c", "label": "Monitoring", "kind": "external"}}],
                "connections": [{{"from": "a", "to": "b", "label": "reads and writes"}}, {{"from": "a", "to": "c", "label": "forwards every event it sees"}}]}}}}"#
        )
    };
    for start in [300, 600, 900] {
        if let Some(asked) = asked_width(&architecture(start)) {
            assert_eq!(
                asked_width(&architecture(asked)),
                None,
                "{start} -> {asked}"
            );
        }
    }
}

const CLASSES: &str = r#"{"schemaVersion": 1, "type": "classes", "title": "Model", "width": 800, "height": 520,
        "classes": {"classes": [
            {"id": "animal", "label": "Animal", "kind": "abstract", "attributes": ["+ name: String"], "methods": ["+ speak()"]},
            {"id": "dog", "label": "Dog", "methods": ["+ speak()"]},
            {"id": "owner", "label": "Owner", "kind": "entity", "attributes": ["id (PK)"]}
        ], "relations": [
            {"from": "dog", "to": "animal", "kind": "inheritance"},
            {"from": "owner", "to": "dog", "kind": "aggregation", "fromCardinality": "1", "toCardinality": "0..*", "label": "keeps"}
        ]}}"#;

#[test]
fn a_tree_can_collapse_a_node_and_everything_below_it() {
    let spec = r#"{"schemaVersion": 1, "type": "tree", "title": "T", "width": 800, "height": 400,
            "tree": {"nodes": [
                {"id": "root", "label": "Root"},
                {"id": "a", "label": "Alpha", "parent": "root", "collapsed": true},
                {"id": "b", "label": "Beta", "parent": "root"},
                {"id": "a1", "label": "Alpha one", "parent": "a"},
                {"id": "a2", "label": "Alpha two", "parent": "a1"},
                {"id": "b1", "label": "Beta one", "parent": "b"}
            ]}}"#;
    let output = render_ok(spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert!(svg.contains(">+2</text>"));
    assert!(!svg.contains(">Alpha one</text>"));
    assert!(svg.contains(">Beta one</text>"));
    let html = render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(
        html.contains("Collapsed in the drawing, with everything below them listed here: Alpha."),
        "{html}"
    );
    assert!(
        html.contains("<th scope=\"row\">Alpha two</th>"),
        "the table keeps every node"
    );
    assert!(html.contains("<td>+2</td>"));
    let leaf = spec.replace(
        r#""id": "b1", "label": "Beta one", "parent": "b""#,
        r#""id": "b1", "label": "Beta one", "parent": "b", "collapsed": true"#,
    );
    let output = render_ok(&leaf);
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "nothing_to_collapse"
                && warning.path == "/tree/nodes/5/collapsed")
    );
}

#[test]
fn a_class_diagram_draws_boxes_members_and_the_marks_of_its_relations() {
    let output = render_ok(CLASSES);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(
        svg.matches("chartlet-class\"").count() + svg.matches("chartlet-class ").count(),
        3
    );
    assert!(svg.contains(">«abstract»</text>"));
    assert!(svg.contains(">+ name: String</text>"));
    // A hollow triangle for the inheritance, a hollow diamond for the aggregation.
    assert_eq!(
        svg.matches("chartlet-flow-hollow").count(),
        2 + 1,
        "two marks and one rule"
    );
    assert!(svg.contains(">0..*</text>"));
    let html = render_json(CLASSES, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(
        html.contains("Class diagram with 3 classes and 2 relations."),
        "{html}"
    );
    assert!(html.contains("Dog inherits from Animal"));
    assert!(html.contains("<th scope=\"col\">Attributes</th>"));
    let german = CLASSES.replace(
        r#""type": "classes""#,
        r#""type": "classes", "locale": "de""#,
    );
    let html = render_json(&german, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("Klassendiagramm mit 3 Klassen und 2 Beziehungen."));
    assert!(html.contains("Dog erbt von Animal"));
}

#[test]
fn class_diagrams_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""id": "dog""#,
            r#""id": "animal""#,
            "duplicate_id",
            "/classes/classes/1/id",
        ),
        (
            r#""id": "owner""#,
            r#""id": "9owner""#,
            "invalid_id",
            "/classes/classes/2/id",
        ),
        (
            r#""to": "animal""#,
            r#""to": "cat""#,
            "unknown_node",
            "/classes/relations/0/to",
        ),
    ] {
        let error = render_json(
            &CLASSES.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}

/// [`CLASSES`] with Animal and Dog in a package Pets inside a package Model.
fn packaged_classes() -> String {
    CLASSES
        .replace(
            r#""classes": {"classes": ["#,
            r#""classes": {"packages": [
                    {"id": "model", "label": "Model"},
                    {"id": "pets", "label": "Pets", "in": "model"}
                ], "classes": ["#,
        )
        .replace(
            r#"{"id": "dog", "label": "Dog","#,
            r#"{"id": "dog", "label": "Dog", "in": "pets","#,
        )
        .replace(
            r#"{"id": "animal", "label": "Animal","#,
            r#"{"id": "animal", "label": "Animal", "in": "pets","#,
        )
}

#[test]
fn packages_frame_their_classes_and_name_them_in_the_text() {
    let spec = packaged_classes();
    for orientation in ["portrait", "landscape"] {
        let json = spec.replace(
            r#""relations": ["#,
            &format!(r#""orientation": "{orientation}", "relations": ["#),
        );
        let output = render_ok(&json);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert_eq!(output.content.matches("chartlet-flow-group\"").count(), 2);
        assert!(output.content.contains(">Pets</text>"));
    }
    let alternative =
        text_alternative(&ChartSpec::from_json(&spec).expect("parses")).expect("valid");
    assert!(
        alternative.description.starts_with(
            "Class diagram with 3 classes and 2 relations in 2 packages. Model: Pets. Pets (in Model): Animal, Dog."
        ),
        "{}",
        alternative.description
    );
    assert_eq!(
        alternative.table.columns,
        [
            "Item",
            "Kind",
            "Package",
            "Attributes",
            "Methods",
            "Relations"
        ]
    );
    assert_eq!(alternative.table.rows[1][2], "Model › Pets");
    assert_eq!(alternative.table.rows[2][2], "");
    let german = spec.replace(
        r#""type": "classes""#,
        r#""type": "classes", "locale": "de""#,
    );
    let alternative =
        text_alternative(&ChartSpec::from_json(&german).expect("parses")).expect("valid");
    assert!(
        alternative
            .description
            .starts_with("Klassendiagramm mit 3 Klassen und 2 Beziehungen in 2 Paketen.")
    );
    assert_eq!(alternative.table.columns[2], "Paket");
}

#[test]
fn packages_are_validated_by_name() {
    let spec = packaged_classes();
    for (from, to, code, path) in [
        (
            r#""in": "pets","#,
            r#""in": "toys","#,
            "unknown_package",
            "/classes/classes/0/in",
        ),
        (
            r#"{"id": "model", "label": "Model"}"#,
            r#"{"id": "model", "label": "Model", "in": "zoo"}"#,
            "unknown_package",
            "/classes/packages/0/in",
        ),
        (
            r#"{"id": "model", "label": "Model"}"#,
            r#"{"id": "model", "label": "Model", "in": "pets"}"#,
            "package_cycle",
            "/classes/packages/0/in",
        ),
        (
            r#"{"id": "model", "label": "Model"}"#,
            r#"{"id": "model", "label": "Model"}, {"id": "empty", "label": "Empty"}"#,
            "empty_package",
            "/classes/packages/1",
        ),
        (
            r#"{"id": "pets", "label": "Pets", "in": "model"}"#,
            r#"{"id": "model", "label": "Pets", "in": "model"}"#,
            "duplicate_id",
            "/classes/packages/1/id",
        ),
        (
            r#"{"id": "owner""#,
            r#"{"id": "pets""#,
            "duplicate_id",
            "/classes/classes/2/id",
        ),
        (
            r#""to": "animal""#,
            r#""to": "pets""#,
            "unknown_node",
            "/classes/relations/0/to",
        ),
    ] {
        assert!(spec.contains(from), "{from}");
        let error = render_json(
            &spec.replacen(from, to, 1),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{to}");
    }
    let deep = spec
        .replace(
            r#"{"id": "model", "label": "Model"}"#,
            r#"{"id": "a", "label": "A"}, {"id": "b", "label": "B", "in": "a"},
                   {"id": "c", "label": "C", "in": "b"}, {"id": "model", "label": "Model", "in": "c"}"#,
        );
    let error = render_json(&deep, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("packages_too_deep", "/classes/packages/4/in")
    );
}

/// [`CLASSES`] with a note on Dog, inside the package of Dog when `packaged`.
fn noted_classes(packaged: bool) -> String {
    let spec = if packaged {
        packaged_classes()
    } else {
        CLASSES.to_owned()
    };
    spec.replace(
        r#""relations": ["#,
        r#""notes": [{"on": "dog", "text": "Only dogs that live in the household"}], "relations": ["#,
    )
}

#[test]
fn a_note_stands_beside_its_class_and_is_no_relation() {
    for packaged in [false, true] {
        let output = render_ok(&noted_classes(packaged));
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        let svg = &output.content;
        assert!(svg.contains("<title>Only dogs that live in the household</title>"));
        assert!(svg.contains(">Only dogs that live in the household<"));
        assert_eq!(
            svg.matches("chartlet-flow-edge chartlet-flow-dotted")
                .count(),
            1
        );
        assert!(!svg.contains("Dog → Only dogs"));
    }
    let plain = text_alternative(&ChartSpec::from_json(CLASSES).expect("parses")).expect("valid");
    let with_notes =
        text_alternative(&ChartSpec::from_json(&noted_classes(false)).expect("parses"))
            .expect("valid");
    // The structure and every class read as before; the note follows them.
    assert_eq!(
        with_notes.description,
        format!(
            "{} Note on Dog: Only dogs that live in the household.",
            plain.description
        )
    );
    assert_eq!(
        with_notes.table.columns.last().map(String::as_str),
        Some("Notes")
    );
    assert_eq!(with_notes.table.rows.len(), 3);
    assert_eq!(
        with_notes.table.rows[1][5],
        "Only dogs that live in the household"
    );
    assert_eq!(with_notes.table.rows[0][5], "");
    let german = noted_classes(false).replace(
        r#""type": "classes""#,
        r#""type": "classes", "locale": "de""#,
    );
    let german = text_alternative(&ChartSpec::from_json(&german).expect("parses")).expect("valid");
    assert!(
        german
            .description
            .ends_with("Notiz zu Dog: Only dogs that live in the household.")
    );
    assert_eq!(german.table.columns[5], "Notizen");
}

#[test]
fn a_note_on_a_package_or_the_whole_diagram_joins_nothing() {
    let spec = packaged_classes().replace(
        r#""relations": ["#,
        r#""notes": [{"on": "pets", "text": "Owned by the pets team"},
                         {"text": "Simplified for the overview"}], "relations": ["#,
    );
    let output = render_ok(&spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert!(svg.contains("<title>Owned by the pets team</title>"));
    assert!(svg.contains("<title>Simplified for the overview</title>"));
    assert!(!svg.contains("chartlet-flow-edge chartlet-flow-dotted"));
    let plain = text_alternative(&ChartSpec::from_json(&packaged_classes()).expect("parses"))
        .expect("valid");
    let noted = text_alternative(&ChartSpec::from_json(&spec).expect("parses")).expect("valid");
    assert_eq!(
        noted.description,
        format!(
            "{} Note on Pets: Owned by the pets team. Note: Simplified for the overview.",
            plain.description
        )
    );
    // No class row carries them, so the table keeps its columns and the caption names them.
    assert_eq!(noted.table.columns, plain.table.columns);
    assert_eq!(
        noted.table.caption,
        format!(
            "{}. Note on Pets: Owned by the pets team. Note: Simplified for the overview.",
            plain.table.caption
        )
    );
    let german = spec.replace(
        r#""type": "classes""#,
        r#""type": "classes", "locale": "de""#,
    );
    let german = text_alternative(&ChartSpec::from_json(&german).expect("parses")).expect("valid");
    assert!(
        german.description.ends_with(
            "Notiz zu Pets: Owned by the pets team. Notiz: Simplified for the overview."
        )
    );
}

#[test]
fn notes_are_validated_by_name() {
    let spec = noted_classes(false);
    let note = r#"{"on": "dog", "text": "Only dogs that live in the household"}"#;
    let long = format!(r#"{{"on": "dog", "text": "{}"}}"#, "x".repeat(161));
    let many = vec![note; 13].join(", ");
    for (to, code, path) in [
        (
            r#"{"on": "cat", "text": "Only dogs"}"#.to_owned(),
            "unknown_node",
            "/classes/notes/0/on",
        ),
        (long, "text_too_long", "/classes/notes/0/text"),
        (
            r#"{"on": "dog", "text": " "}"#.to_owned(),
            "empty_text",
            "/classes/notes/0/text",
        ),
        (many, "too_many_notes", "/classes/notes"),
    ] {
        let error = render_json(
            &spec.replace(note, &to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{to}");
    }
    // A long note wraps onto lines of its own instead of being shortened.
    let long = spec.replace(
        "Only dogs that live in the household",
        "Only dogs that live in the household and have been registered with the town for at least a year",
    );
    let output = render_ok(&long);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
}

const SANKEY: &str = r#"{"schemaVersion": 1, "type": "sankey", "title": "Flow", "width": 640, "height": 400,
        "sankey": {"links": [
            {"from": "A", "to": "Mid", "value": 30},
            {"from": "B", "to": "Mid", "value": 10},
            {"from": "Mid", "to": "X", "value": 25},
            {"from": "Mid", "to": "Y", "value": 15}
        ]}}"#;

#[test]
fn a_sankey_diagram_draws_a_band_for_every_link_and_lists_them() {
    let output = render_ok(SANKEY);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-sk-link ").count(), 4);
    assert_eq!(svg.matches("class=\"chartlet-sk-node ").count(), 5);
    assert!(svg.contains("<title>A → Mid: 30</title>"));
    let html = render_json(SANKEY, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("Sankey diagram of 5 nodes and 4 links; 40 enter it. Biggest links: A → Mid (30), Mid → X (25), Mid → Y (15)."));
    assert!(html.contains("<th scope=\"row\">A</th><td>Mid</td><td>30</td><td>75.0%</td>"));
}

#[test]
fn a_sankey_diagram_can_pin_columns_order_and_colors() {
    let spec = SANKEY.replace(
        r#""sankey": {"#,
        r#""sankey": {"order": "listed", "nodes": [{"label": "Y", "column": 3, "color": 2}, {"label": "X", "column": 2}],"#,
    );
    let output = render_ok(&spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    // Y is listed first, so it takes the first place of its column and its own color.
    assert!(svg.contains("class=\"chartlet-sk-node chartlet-sk-2\""));
    let early = SANKEY.replace(
        r#""sankey": {"#,
        r#""sankey": {"nodes": [{"label": "X", "column": 1}],"#,
    );
    let error = render_json(&early, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("column_too_early", "/sankey/nodes/0/column")
    );
    let unknown = SANKEY.replace(
        r#""sankey": {"#,
        r#""sankey": {"nodes": [{"label": "Nowhere"}],"#,
    );
    let error = render_json(&unknown, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("unknown_node", "/sankey/nodes/0/label")
    );
}

#[test]
fn a_sankey_diagram_draws_a_flow_back_as_a_loop() {
    let spec = SANKEY.replace(
        r#"{"from": "Mid", "to": "Y", "value": 15}"#,
        r#"{"from": "Mid", "to": "Y", "value": 15}, {"from": "Y", "to": "A", "value": 5, "back": true}"#,
    );
    let output = render_ok(&spec);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    // Five links now: a band for each, the flow back with a 12-point outline.
    assert_eq!(
        output.content.matches("class=\"chartlet-sk-link ").count(),
        5
    );
    assert!(output.content.contains("<title>Y → A: 5</title>"));
    // The total counts what enters, not what returns.
    let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content;
    assert!(html.contains("40 enter it"), "{html}");
    // Not a flow back: B leads forward.
    let forward = spec.replace(
        r#""from": "Y", "to": "A", "value": 5, "back": true"#,
        r#""from": "A", "to": "X", "value": 5, "back": true"#,
    );
    let error = render_json(&forward, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(
        (error.code, error.path.as_str()),
        ("link_not_back", "/sankey/links/4/back")
    );
    // Without the mark the same link is a cycle.
    let cycle = spec.replace(r#", "back": true"#, "");
    let error = render_json(&cycle, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "link_cycle");
}

#[test]
fn sankey_diagrams_are_validated_by_name() {
    for (from, to, code, path) in [
        (
            r#""value": 30"#,
            r#""value": 0"#,
            "invalid_value",
            "/sankey/links/0/value",
        ),
        (
            r#""to": "Mid", "value": 10"#,
            r#""to": "B", "value": 10"#,
            "self_link",
            "/sankey/links/1/to",
        ),
        (
            r#"{"from": "B", "to": "Mid""#,
            r#"{"from": "A", "to": "Mid""#,
            "duplicate_link",
            "/sankey/links/1",
        ),
        (
            r#"{"from": "Mid", "to": "Y", "value": 15}"#,
            r#"{"from": "Mid", "to": "Y", "value": 15}, {"from": "Y", "to": "A", "value": 1}"#,
            "link_cycle",
            "/sankey/links",
        ),
    ] {
        let error = render_json(
            &SANKEY.replace(from, to),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!((error.code, error.path.as_str()), (code, path), "{from}");
    }
}
