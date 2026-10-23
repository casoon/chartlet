use chartlet::{RenderFormat, RenderOptions, render_json};

/// Every example specification as name and JSON text.
fn examples() -> Vec<(String, String)> {
    let mut examples: Vec<(String, String)> = std::fs::read_dir("examples")
        .expect("the examples directory should exist")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .map(|path| {
            (
                path.file_stem()
                    .expect("a file name")
                    .to_string_lossy()
                    .into_owned(),
                std::fs::read_to_string(&path).expect("readable example"),
            )
        })
        .collect();
    examples.sort();
    examples
}

fn render(specification: &str, format: RenderFormat, hooks: bool) -> String {
    render_json(
        specification,
        format,
        &RenderOptions {
            hooks,
            ..RenderOptions::default()
        },
    )
    .expect("every example renders")
    .content
}

/// Removes the attribute `name="…"`, with its leading space, wherever it occurs.
fn remove_attribute(content: &str, name: &str) -> String {
    let needle = format!(" {name}=\"");
    let mut output = String::new();
    let mut rest = content;
    while let Some(start) = rest.find(&needle) {
        output.push_str(&rest[..start]);
        let value = &rest[start + needle.len()..];
        rest = &value[value.find('"').expect("a closed attribute") + 1..];
    }
    output.push_str(rest);
    output
}

/// Removes every element from `open` up to and including `close`.
fn remove_between(content: &str, open: &str, close: &str) -> String {
    let mut output = String::new();
    let mut rest = content;
    while let Some(start) = rest.find(open) {
        output.push_str(&rest[..start]);
        let after = &rest[start..];
        rest = &after[after.find(close).expect("a closed element") + close.len()..];
    }
    output.push_str(rest);
    output
}

/// The output with every hook taken out again. Without hooks chartlet writes no `<g>` but the
/// focus groups of a diagram, and a layer group is left as a bare `<g>` once its attributes are
/// gone.
fn without_hooks(content: &str) -> String {
    // The hooks of a table column name the series before the pane; a bar's own `data-series`
    // is never followed by a pane.
    let mut content = content.to_owned();
    while let Some(start) = content.find(" data-pane=\"") {
        let series = content[..start]
            .rfind(" data-series=\"")
            .filter(|series| !content[*series..start].contains('>'));
        let end = start + content[start + 12..].find('"').expect("closed") + 13;
        content.replace_range(series.unwrap_or(start)..end, "");
    }
    for name in [
        "data-chartlet-type",
        "data-chartlet-id",
        "data-chartlet-locale",
        "data-chartlet-table",
        "data-chartlet-node",
        "data-chartlet-edge",
        "data-chartlet-frame",
        "data-chartlet-lane",
        "data-part",
        "data-stacked",
        "data-x",
        "data-value",
        "data-name",
        "data-annotation",
    ] {
        content = remove_attribute(&content, name);
    }
    content = remove_between(&content, "<script type=\"application/json\"", "</script>");
    content = remove_between(&content, "<g data-chartlet-plot", "/>");
    without_groups(&content)
}

/// The output without any group: the bare groups the hooks leave, and the focus groups of a
/// diagram, which are there with and without hooks.
fn without_groups(content: &str) -> String {
    remove_between(content, "<g class=\"chartlet-f", ">")
        .replace("<g>", "")
        .replace("</g>", "")
}

#[test]
fn hooks_only_add_to_every_example() {
    for (name, specification) in examples() {
        for format in [RenderFormat::Svg, RenderFormat::Html] {
            let plain = render(&specification, format, false);
            let hooked = render(&specification, format, true);
            assert!(
                !plain.contains("data-chartlet-"),
                "{name}: hooks without the option"
            );
            assert!(hooked.contains("data-chartlet-type"), "{name}: no hooks");
            assert_eq!(
                without_hooks(&hooked),
                without_groups(&plain),
                "{name} ({format:?})"
            );
        }
    }
}

/// The JSON data block of an SVG, unescaped.
fn data_block(svg: &str) -> serde_json::Value {
    let start = svg.find("data-chartlet-data=\"\">").expect("a data block") + 22;
    let end = start + svg[start..].find("</script>").expect("a closed block");
    let json = svg[start..end]
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    serde_json::from_str(&json).expect("the data block is JSON")
}

fn numbers(text: &str) -> Vec<f64> {
    text.split_whitespace()
        .map(|number| number.parse().expect("a number"))
        .collect()
}

fn attribute<'a>(tag: &'a str, name: &str) -> &'a str {
    let start = tag.find(&format!(" {name}=\"")).expect("the attribute") + name.len() + 3;
    &tag[start..start + tag[start..].find('"').expect("closed")]
}

#[test]
fn the_svg_profile_carries_the_table_as_a_data_block() {
    let specification = include_str!("../examples/sensor-readings.json");
    let block = data_block(&render(specification, RenderFormat::Svg, true));
    let html = render(specification, RenderFormat::Html, true);
    let columns = block["columns"].as_array().expect("columns");
    let rows = block["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), html.matches("<tr data-x=").count());
    assert_eq!(columns[1]["name"], "Servers");
    assert_eq!(columns[1]["series"], 1);
    assert_eq!(columns[1]["part"], "value");
    assert!(html.contains(
        "<th scope=\"col\" data-series=\"1\" data-pane=\"0\" data-part=\"value\">Servers</th>"
    ));
    assert_eq!(rows[0]["label"], "2026-03-01");
    assert_eq!(rows[0]["value"][1], 22.0);
    assert_eq!(rows[0]["text"][1], "22.0");
    // The HTML profile has its table and carries no data block.
    assert!(!html.contains("<script"));
}

#[test]
fn a_plot_hook_maps_an_observation_onto_its_drawn_point() {
    let svg = render(
        include_str!("../examples/sensor-readings.json"),
        RenderFormat::Svg,
        true,
    );
    let block = data_block(&svg);
    let plot = &svg[svg.find("<g data-chartlet-plot").expect("a plot hook")..];
    let plot = &plot[..plot.find("/>").expect("closed")];
    let (x_domain, x_range) = (
        numbers(attribute(plot, "data-x-domain")),
        numbers(attribute(plot, "data-x-range")),
    );
    let (y_domain, y_range) = (
        numbers(attribute(plot, "data-y-domain")),
        numbers(attribute(plot, "data-y-range")),
    );
    let linear = |domain: &[f64], range: &[f64], value: f64| {
        range[0] + (value - domain[0]) / (domain[1] - domain[0]) * (range[1] - range[0])
    };
    let row = &block["rows"][0];
    let x = linear(&x_domain, &x_range, row["x"].as_f64().expect("x"));
    let y = linear(
        &y_domain,
        &y_range,
        row["value"][1].as_f64().expect("a value"),
    );

    let group = &svg[svg.find("<g data-series=\"1\"").expect("the Servers layer")..];
    let points = attribute(
        &group[group.find("<polyline").expect("its line")..],
        "points",
    );
    let first: Vec<f64> = points
        .split_whitespace()
        .next()
        .expect("a point")
        .split(',')
        .map(|number| number.parse().expect("a number"))
        .collect();
    assert!((first[0] - x).abs() < 0.01, "{first:?} vs {x}");
    assert!((first[1] - y).abs() < 0.01, "{first:?} vs {y}");
}

#[test]
fn collapsed_gaps_list_every_slot() {
    let svg = render(
        include_str!("../examples/share-price.json"),
        RenderFormat::Svg,
        true,
    );
    let plots: Vec<&str> = svg.split("<g data-chartlet-plot").skip(1).collect();
    assert_eq!(plots.len(), 2, "one plot per pane");
    let rows = data_block(&svg)["rows"].as_array().expect("rows").len();
    assert_eq!(numbers(attribute(plots[0], "data-x-domain")).len(), rows);
    assert_eq!(attribute(plots[1], "data-pane"), "1");
}

#[test]
fn the_cli_writes_hooks_with_the_flag() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/monthly-trend.json",
            "--format",
            "svg",
            "--hooks",
        ])
        .output()
        .expect("the CLI runs");
    assert!(output.status.success(), "{output:?}");
    let svg = String::from_utf8(output.stdout).expect("UTF-8");
    assert!(svg.contains("data-chartlet-type=\"line\""));
    assert!(svg.contains("data-x-domain=\"0 5\""));
}

/// The group a diagram node's spec ID hooks onto, up to where its drawing starts.
fn node_group<'a>(content: &'a str, id: &str) -> &'a str {
    let hook = format!(" data-chartlet-node=\"{id}\">");
    let end = content
        .find(&hook)
        .unwrap_or_else(|| panic!("a hook for {id}"))
        + hook.len();
    let group = &content[end..];
    &group[..group.find("</g>").expect("a closed group")]
}

fn render_with(specification: &str, format: RenderFormat, id_prefix: &str) -> String {
    render_json(
        specification,
        format,
        &RenderOptions {
            hooks: true,
            id_prefix: Some(id_prefix.to_owned()),
            ..RenderOptions::default()
        },
    )
    .expect("the diagram renders")
    .content
}

#[test]
fn diagram_nodes_carry_their_spec_ids() {
    // Two classes of the same name are told apart by their IDs.
    let classes = r#"{"schemaVersion": 1, "type": "classes", "title": "Two orders",
        "classes": {"classes": [
            {"id": "sales-order", "label": "Order", "attributes": ["+ total: Money"]},
            {"id": "stock_order", "label": "Order", "attributes": ["+ items: Int"]}
        ], "relations": [{"from": "sales-order", "to": "stock_order", "kind": "association"}]}}"#;
    let svg = render(classes, RenderFormat::Svg, true);
    assert_eq!(svg.matches("data-chartlet-node=").count(), 2);
    assert!(node_group(&svg, "sales-order").contains("+ total: Money"));
    assert!(node_group(&svg, "stock_order").contains("+ items: Int"));
    assert!(!render(classes, RenderFormat::Svg, false).contains("data-chartlet-node"));

    // Flow charts and state diagrams take the same path; the start dot of a state diagram has
    // no ID and so no hook.
    let flow = render(
        include_str!("../examples/order-flow.json"),
        RenderFormat::Svg,
        true,
    );
    assert!(node_group(&flow, "received").contains("Order"));
    let states = include_str!("../examples/ticket-states.json");
    let svg = render(states, RenderFormat::Svg, true);
    assert!(node_group(&svg, "new").contains("New"));
    let spec: serde_json::Value = serde_json::from_str(states).expect("JSON");
    let composites = spec["state"]["states"]
        .as_array()
        .expect("states")
        .iter()
        .filter(|state| {
            spec["state"]["states"]
                .as_array()
                .expect("states")
                .iter()
                .any(|inner| inner["in"] == state["id"])
        })
        .count();
    assert_eq!(
        svg.matches("data-chartlet-node=").count(),
        spec["state"]["states"].as_array().expect("states").len() - composites
    );
}

#[test]
fn reordering_keeps_the_ids_of_diagram_nodes() {
    let architecture = |components: &str| {
        format!(
            r#"{{"schemaVersion": 1, "type": "architecture", "title": "Shop",
            "architecture": {{"components": [{components}],
            "connections": [{{"from": "web", "to": "db"}}]}}}}"#
        )
    };
    let (web, db) = (
        r#"{"id": "web", "label": "Shop", "kind": "service"}"#,
        r#"{"id": "db", "label": "Shop", "sublabel": "orders", "kind": "database"}"#,
    );
    for components in [format!("{web}, {db}"), format!("{db}, {web}")] {
        let svg = render(&architecture(&components), RenderFormat::Svg, true);
        assert!(!node_group(&svg, "web").contains("orders"));
        assert!(node_group(&svg, "db").contains("orders"));
    }
}

#[test]
fn node_hooks_repeat_across_variants_and_embeds_without_dom_id_collisions() {
    let specification = include_str!("../examples/tenant-architecture.json");
    let page: Vec<String> = ["first", "second"]
        .into_iter()
        .map(|prefix| render_with(specification, RenderFormat::Html, prefix))
        .collect();
    // Each figure holds the desktop and the mobile variant, so every node is there twice.
    let nodes = page[0].matches(" data-chartlet-node=\"proxy\"").count();
    assert_eq!(nodes, 2);
    let page = page.concat();
    assert_eq!(page.matches(" data-chartlet-node=\"proxy\"").count(), 4);
    let mut ids: Vec<&str> = page
        .match_indices(" id=\"")
        .map(|(start, _)| attribute(&page[start..], "id"))
        .collect();
    let all = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), all, "a DOM ID repeats");
}

#[test]
fn a_class_in_a_package_keeps_its_hook_and_a_note_has_none() {
    let classes = r#"{"schemaVersion": 1, "type": "classes", "title": "Packaged",
        "classes": {"packages": [{"id": "sales", "label": "Sales"}], "classes": [
            {"id": "sales-order", "label": "Order", "attributes": ["+ total: Money"], "in": "sales"},
            {"id": "stock_order", "label": "Order", "attributes": ["+ items: Int"]}
        ], "relations": [{"from": "sales-order", "to": "stock_order"}],
        "notes": [{"on": "sales-order", "text": "Only paid orders"}]}}"#;
    let svg = render(classes, RenderFormat::Svg, true);
    assert_eq!(svg.matches("data-chartlet-node=").count(), 2);
    assert!(node_group(&svg, "sales-order").contains("+ total: Money"));
    let note = svg.find(">Only paid orders<").expect("the note is drawn");
    let group = &svg[svg[..note].rfind("<g").expect("the note's group")..note];
    assert!(!group.contains("data-chartlet-node"), "{group}");
}

/// The group that carries `attribute="id"`, up to where its drawing starts.
fn hooked_group<'a>(content: &'a str, attribute: &str, id: &str) -> &'a str {
    let hook = format!(" {attribute}=\"{id}\">");
    let end = content
        .find(&hook)
        .unwrap_or_else(|| panic!("a hook for {id}"))
        + hook.len();
    let group = &content[end..];
    &group[..group.find("</g>").expect("a closed group")]
}

#[test]
fn connections_with_an_id_carry_an_edge_hook() {
    let architecture = |connections: &str| {
        format!(
            r#"{{"schemaVersion": 1, "type": "architecture", "title": "Shop",
            "architecture": {{"boundaries": [{{"id": "cloud", "label": "Cloud"}}],
            "components": [
                {{"id": "web", "label": "Web"}},
                {{"id": "db", "label": "DB", "kind": "database", "in": "cloud"}},
                {{"id": "cache", "label": "Cache", "kind": "cache", "in": "cloud"}}],
            "connections": [{connections}],
            "notes": [{{"on": "db", "text": "Nightly backup"}}]}}}}"#
        )
    };
    let (reads, warms, plain) = (
        r#"{"id": "reads", "from": "web", "to": "db", "label": "Query"}"#,
        r#"{"id": "warms", "from": "web", "to": "cache", "label": "Query"}"#,
        r#"{"from": "db", "to": "cache"}"#,
    );
    for connections in [
        format!("{reads}, {warms}, {plain}"),
        format!("{plain}, {warms}, {reads}"),
    ] {
        let specification = architecture(&connections);
        let svg = render(&specification, RenderFormat::Svg, true);
        // Two connections of the same label stay apart; the one without an ID and the note's
        // line carry none.
        assert_eq!(svg.matches("data-chartlet-edge=").count(), 2);
        assert!(hooked_group(&svg, "data-chartlet-edge", "reads").contains("Web → DB"));
        assert!(hooked_group(&svg, "data-chartlet-edge", "warms").contains("Web → Cache"));
        assert_eq!(svg.matches(" data-chartlet-frame=\"cloud\">").count(), 1);
        assert!(hooked_group(&svg, "data-chartlet-frame", "cloud").contains("chartlet-flow-group"));
        let plain = render(&specification, RenderFormat::Svg, false);
        assert!(!plain.contains("data-chartlet-"));
    }
}

#[test]
fn a_repeated_step_of_a_view_hooks_its_connection_once() {
    let svg = render_json(
        include_str!("../examples/shop-views.json"),
        RenderFormat::Svg,
        &RenderOptions {
            hooks: true,
            view: Some("checkout".to_owned()),
            ..RenderOptions::default()
        },
    )
    .expect("the view renders")
    .content;
    // The checkout runs `orders` twice; only its first step keeps the ID.
    assert_eq!(svg.matches(" data-chartlet-edge=\"orders\"").count(), 1);
    assert_eq!(svg.matches(" data-chartlet-edge=\"call\"").count(), 1);
}

#[test]
fn shared_trunks_keep_one_edge_hook_per_connection() {
    let specification = include_str!("../examples/service-fan-out.json").replace(
        r#"{ "from": "gateway", "to": "cart" }"#,
        r#"{"id": "to-cart", "from": "gateway", "to": "cart"}"#,
    );
    let svg = render(&specification, RenderFormat::Svg, true);
    assert_eq!(svg.matches("data-chartlet-edge=").count(), 1);
    assert!(hooked_group(&svg, "data-chartlet-edge", "to-cart").contains("Gateway → Cart"));
    for frame in ["cluster", "services", "ops"] {
        assert_eq!(
            svg.matches(&format!(" data-chartlet-frame=\"{frame}\""))
                .count(),
            1
        );
    }
}

#[test]
fn frames_and_lanes_carry_their_spec_ids() {
    let classes = r#"{"schemaVersion": 1, "type": "classes", "title": "Packaged",
        "classes": {"packages": [{"id": "sales", "label": "Orders"}, {"id": "stock", "label": "Orders"}],
        "classes": [
            {"id": "sales-order", "label": "Order", "in": "sales"},
            {"id": "stock_order", "label": "Order", "in": "stock"}
        ], "relations": [{"from": "sales-order", "to": "stock_order"}]}}"#;
    let svg = render(classes, RenderFormat::Svg, true);
    assert_eq!(svg.matches("data-chartlet-frame=").count(), 2);
    assert_eq!(svg.matches("data-chartlet-edge=").count(), 0);
    assert!(hooked_group(&svg, "data-chartlet-frame", "sales").contains(">Orders<"));

    let states = r#"{"schemaVersion": 1, "type": "state", "title": "Ticket",
        "state": {"initial": "new", "states": [
            {"id": "new", "label": "New"},
            {"id": "work", "label": "In progress", "kind": "composite"},
            {"id": "open", "label": "Open", "in": "work"},
            {"id": "done", "label": "Done", "final": true}
        ], "transitions": [{"from": "new", "to": "open"}, {"from": "open", "to": "done"}]}}"#;
    let svg = render(states, RenderFormat::Svg, true);
    assert!(hooked_group(&svg, "data-chartlet-frame", "work").contains(">In progress<"));
    assert!(!svg.contains("data-chartlet-node=\"work\""));

    let example = include_str!("../examples/release-flow.json");
    let flow = render(example, RenderFormat::Svg, true);
    let spec: serde_json::Value = serde_json::from_str(example).expect("JSON");
    let lanes = spec["flow"]["lanes"].as_array().expect("lanes");
    assert_eq!(flow.matches("data-chartlet-lane=").count(), lanes.len());
    let first = lanes[0]["id"].as_str().expect("an id");
    assert!(hooked_group(&flow, "data-chartlet-lane", first).contains("chartlet-flow-lane"));
}

#[test]
fn networks_trees_and_sequences_hook_their_nodes() {
    let network = r#"{"schemaVersion": 1, "type": "network", "title": "Two Annas",
        "network": {"nodes": [{"id": "anna-1", "label": "Anna"}, {"id": "anna-2", "label": "Anna"},
            {"id": "ben", "label": "Ben"}],
        "edges": [{"from": "anna-1", "to": "ben"}, {"from": "anna-2", "to": "ben"}]}}"#;
    let svg = render(network, RenderFormat::Svg, true);
    assert_eq!(svg.matches("data-chartlet-node=").count(), 3);
    assert!(hooked_group(&svg, "data-chartlet-node", "anna-1").contains("Anna"));
    assert!(hooked_group(&svg, "data-chartlet-node", "anna-2").contains("Anna"));

    for (example, kind, objects) in [
        (
            include_str!("../examples/family-tree.json"),
            "tree",
            "nodes",
        ),
        (
            include_str!("../examples/sign-in-sequence.json"),
            "sequence",
            "participants",
        ),
    ] {
        let spec: serde_json::Value = serde_json::from_str(example).expect("JSON");
        let svg = render(example, RenderFormat::Svg, true);
        for object in spec[kind][objects].as_array().expect("objects") {
            let id = object["id"].as_str().expect("an id");
            let label = object["label"].as_str().expect("a label");
            assert!(
                hooked_group(&svg, "data-chartlet-node", id).contains(label),
                "{kind}: {id}"
            );
        }
    }
}

#[test]
fn edge_and_frame_hooks_repeat_across_variants_and_embeds_without_dom_id_collisions() {
    let specification = include_str!("../examples/tenant-architecture.json").replacen(
        r#""from": "nutzer","#,
        r#""id": "use", "from": "nutzer","#,
        1,
    );
    let page: Vec<String> = ["first", "second"]
        .into_iter()
        .map(|prefix| render_with(&specification, RenderFormat::Html, prefix))
        .collect();
    // Each figure holds the desktop and the mobile variant.
    assert_eq!(page[0].matches(" data-chartlet-edge=\"use\"").count(), 2);
    assert_eq!(page[0].matches(" data-chartlet-frame=\"rz\"").count(), 2);
    let page = page.concat();
    assert_eq!(page.matches(" data-chartlet-edge=\"use\"").count(), 4);
    let mut ids: Vec<&str> = page
        .match_indices(" id=\"")
        .map(|(start, _)| attribute(&page[start..], "id"))
        .collect();
    let all = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), all, "a DOM ID repeats");
}
