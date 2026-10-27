//! Diagrams generated from fixed seeds: long labels, cycles, nested boundaries and narrow
//! canvases. Each must render, render the same bytes twice, be well-formed SVG and keep every
//! drawn element on the canvas it declares. What a layout guarantees beyond that, such as that
//! no two boxes overlap, is left to the tests of each layout.

use chartlet::{RenderFormat, RenderOptions, Variant, render_json};
use serde_json::{Value, json};

/// How many diagrams of each kind are generated.
const SEEDS: u64 = 40;

/// A small, fixed pseudo-random sequence (splitmix64), so that a seed always gives the same
/// diagram on every platform.
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number in `0..bound`.
    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % bound as u64).expect("the bound fits")
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }

    /// A label of one to several words, now and then up to 60 characters or one long word.
    fn label(&mut self) -> String {
        const WORDS: [&str; 12] = [
            "Order",
            "check",
            "service",
            "payment",
            "gateway",
            "retry",
            "queue",
            "store",
            "validated",
            "customer",
            "export",
            "approval",
        ];
        match self.below(10) {
            0 => "Interoperabilitätsanforderungsprüfung".to_owned(),
            1 => {
                let mut label = String::new();
                loop {
                    let word = WORDS[self.below(WORDS.len())];
                    if label.len() + word.len() > 60 {
                        return label.trim_end().to_owned();
                    }
                    label.push_str(word);
                    label.push(' ');
                }
            }
            _ => (0..=self.below(3))
                .map(|_| WORDS[self.below(WORDS.len())])
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    /// A canvas width, often a narrow one.
    fn width(&mut self) -> u64 {
        [200, 240, 320, 480, 800, 1200][self.below(6)]
    }
}

/// Edges along a chain of `count` nodes, plus edges back (cycles) and across.
fn edges(random: &mut Random, count: usize) -> Vec<(usize, usize)> {
    let mut edges: Vec<(usize, usize)> = (1..count).map(|to| (random.below(to), to)).collect();
    for _ in 0..random.below(count) {
        let (from, to) = (random.below(count), random.below(count));
        if from != to && !edges.contains(&(from, to)) {
            edges.push((from, to));
        }
    }
    edges
}

fn flow(seed: u64) -> Value {
    let mut random = Random(seed);
    let count = 2 + random.below(12);
    let lanes = random.below(4);
    let nodes: Vec<Value> = (0..count)
        .map(|index| {
            let mut node = json!({
                "id": format!("n{index}"),
                "label": random.label(),
                "kind": random.pick(&["process", "process", "io", "subprocess", "store", "external"]),
            });
            if lanes > 0 {
                node["lane"] = json!(format!("l{}", random.below(lanes)));
            }
            if random.chance(20) {
                node["sublabel"] = json!(random.label());
            }
            node
        })
        .collect();
    let edges: Vec<Value> = edges(&mut random, count)
        .into_iter()
        .map(|(from, to)| {
            let mut edge = json!({ "from": format!("n{from}"), "to": format!("n{to}") });
            if random.chance(30) {
                edge["label"] = json!(random.label());
            }
            edge
        })
        .collect();
    let mut flow = json!({ "nodes": nodes, "edges": edges });
    if lanes > 0 {
        flow["lanes"] = (0..lanes)
            .map(|lane| json!({ "id": format!("l{lane}"), "label": format!("Lane {lane}") }))
            .collect();
    }
    json!({
        "schemaVersion": 1,
        "type": "flow",
        "title": random.label(),
        "width": random.width(),
        "flow": flow,
    })
}

fn architecture(seed: u64) -> Value {
    let mut random = Random(seed);
    let depth = random.below(5);
    let boundaries: Vec<Value> = (0..depth)
        .map(|index| {
            let mut boundary = json!({ "id": format!("b{index}"), "label": random.label() });
            if index > 0 {
                boundary["in"] = json!(format!("b{}", index - 1));
            }
            boundary
        })
        .collect();
    let count = (2 + random.below(10)).max(depth);
    let components: Vec<Value> = (0..count)
        .map(|index| {
            let mut component = json!({
                "id": format!("c{index}"),
                "label": random.label(),
                "kind": random.pick(&["service", "database", "queue", "person", "external", "cache"]),
            });
            // The first components fill the boundaries from the inside out, so none is empty.
            if index < depth {
                component["in"] = json!(format!("b{}", depth - 1 - index));
            } else if depth > 0 && random.chance(60) {
                component["in"] = json!(format!("b{}", random.below(depth)));
            }
            if random.chance(30) {
                component["sublabel"] = json!(random.label());
            }
            component
        })
        .collect();
    let connections: Vec<Value> = edges(&mut random, count)
        .into_iter()
        .map(|(from, to)| {
            let mut connection = json!({ "from": format!("c{from}"), "to": format!("c{to}") });
            if random.chance(40) {
                connection["label"] = json!(random.label());
            }
            connection
        })
        .collect();
    json!({
        "schemaVersion": 1,
        "type": "architecture",
        "title": random.label(),
        "width": random.width(),
        "architecture": {
            "components": components,
            "connections": connections,
            "boundaries": boundaries,
        },
    })
}

fn tree(seed: u64) -> Value {
    let mut random = Random(seed);
    let count = 1 + random.below(16);
    let nodes: Vec<Value> = (0..count)
        .map(|index| {
            let mut node = json!({ "id": format!("t{index}"), "label": random.label() });
            if index > 0 {
                node["parent"] = json!(format!("t{}", random.below(index)));
            }
            node
        })
        .collect();
    let mut tree = json!({ "nodes": nodes });
    if random.chance(30) {
        tree["layout"] = json!("radial");
    }
    json!({
        "schemaVersion": 1,
        "type": "tree",
        "title": random.label(),
        "width": random.width(),
        "tree": tree,
    })
}

fn network(seed: u64) -> Value {
    let mut random = Random(seed);
    let count = 2 + random.below(14);
    let nodes: Vec<Value> = (0..count)
        .map(|index| json!({ "id": format!("v{index}"), "label": random.label() }))
        .collect();
    let edges: Vec<Value> = edges(&mut random, count)
        .into_iter()
        .map(|(from, to)| json!({ "from": format!("v{from}"), "to": format!("v{to}") }))
        .collect();
    let mut network = json!({ "nodes": nodes, "edges": edges, "directed": random.chance(50) });
    if random.chance(50) {
        network["layout"] = json!("rings");
        network["center"] = json!("v0");
    }
    json!({
        "schemaVersion": 1,
        "type": "network",
        "title": random.label(),
        "width": random.width(),
        "network": network,
    })
}

fn cycle(seed: u64) -> Value {
    let mut random = Random(seed);
    let count = 3 + random.below(6);
    let steps: Vec<Value> = (0..count)
        .map(|index| json!({ "id": format!("s{index}"), "label": random.label() }))
        .collect();
    let mut cycle = json!({ "steps": steps });
    if random.chance(50) {
        cycle["labels"] = json!("inside");
    }
    if random.chance(50) {
        cycle["hub"] = json!({ "label": random.label() });
    }
    json!({
        "schemaVersion": 1,
        "type": "cycle",
        "title": random.label(),
        "width": random.width(),
        "cycle": cycle,
    })
}

fn layers(seed: u64) -> Value {
    let mut random = Random(seed);
    let count = 1 + random.below(6);
    let nested = random.chance(40);
    let layers: Vec<Value> = (0..count)
        .map(|index| {
            let items: Vec<Value> = (0..random.below(6))
                .map(|_| json!(random.pick(&["Compute", "Storage", "Network", "Queue", "Cache"])))
                .collect();
            let mut layer =
                json!({ "id": format!("l{index}"), "label": random.label(), "items": items });
            if random.chance(40) {
                layer["note"] = json!(random.label());
            }
            layer
        })
        .collect();
    let mut block = json!({ "layers": layers });
    if nested {
        block["display"] = json!("nested");
    } else if random.chance(50) {
        block["projection"] = json!("exploded");
    }
    json!({
        "schemaVersion": 1,
        "type": "layers",
        "title": random.label(),
        "width": random.width(),
        "layers": block,
    })
}

fn plan(seed: u64) -> Value {
    let mut random = Random(seed);
    let count = 1 + random.below(8);
    let mut x = 0;
    let areas: Vec<Value> = (0..count)
        .map(|index| {
            let width = 2 + random.below(5);
            let depth = 2 + random.below(5);
            let mut area = json!({
                "id": format!("a{index}"),
                "label": random.label(),
                "x": x,
                "y": random.below(3),
                "w": width,
                "h": depth,
            });
            x += width;
            match random.below(3) {
                0 => area["kind"] = json!("building"),
                1 => area["phase"] = json!(format!("Phase {}", random.below(4))),
                _ => {}
            }
            if random.chance(40) {
                area["items"] = json!([{ "kind": "table", "x": 0, "y": 0 }]);
            }
            area
        })
        .collect();
    json!({
        "schemaVersion": 1,
        "type": "plan",
        "title": random.label(),
        "width": random.width(),
        "plan": { "areas": areas },
    })
}

/// Whether `svg` is one well-formed element: every tag closed in order, attributes quoted, and
/// no number that did not resolve.
fn check_well_formed(svg: &str) -> Result<(), String> {
    if !svg.starts_with("<svg ") || !svg.ends_with("</svg>") {
        return Err("not a single <svg> element".to_owned());
    }
    for needle in ["NaN", "inf\"", "=\"-inf", "Infinity"] {
        if svg.contains(needle) {
            return Err(format!("contains {needle}"));
        }
    }
    let mut open: Vec<&str> = Vec::new();
    let mut rest = svg;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        if let Some(comment) = rest.strip_prefix("!--") {
            let end = comment.find("-->").ok_or("unclosed comment")?;
            rest = &comment[end + 3..];
            continue;
        }
        if let Some(cdata) = rest.strip_prefix("![CDATA[") {
            let end = cdata.find("]]>").ok_or("unclosed CDATA")?;
            rest = &cdata[end + 3..];
            continue;
        }
        let end = tag_end(rest).ok_or("unclosed tag")?;
        let tag = &rest[..end];
        rest = &rest[end + 1..];
        if let Some(name) = tag.strip_prefix('/') {
            match open.pop() {
                Some(expected) if expected == name.trim() => {}
                other => return Err(format!("</{name}> closes {other:?}")),
            }
        } else if !tag.ends_with('/') {
            let name = tag.split_whitespace().next().ok_or("empty tag")?;
            if !matches!(name, "style" | "script") {
                open.push(name);
                continue;
            }
            let close = format!("</{name}>");
            let end = rest.find(&close).ok_or("unclosed raw text element")?;
            rest = &rest[end + close.len()..];
        }
    }
    if open.is_empty() {
        Ok(())
    } else {
        Err(format!("unclosed elements {open:?}"))
    }
}

/// The value of the attribute `name` of `tag`, as a number.
fn number_attribute(tag: &str, name: &str) -> Option<f64> {
    let marker = format!(" {name}=\"");
    let start = tag.find(&marker)? + marker.len();
    tag[start..start + tag[start..].find('"')?].parse().ok()
}

/// Whether every rectangle, line, circle, text anchor and polyline point of `svg` lies on the
/// canvas of its `viewBox`, to the pixel: a layout that places something outside it draws it
/// where nobody sees it, whatever the warnings say.
fn check_on_canvas(svg: &str) -> Result<(), String> {
    let root = &svg[..svg.find('>').ok_or("no root tag")?];
    let view_box = root
        .split("viewBox=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .ok_or("no viewBox")?;
    let sizes: Vec<f64> = view_box
        .split_whitespace()
        .filter_map(|part| part.parse().ok())
        .collect();
    let (width, height) = (sizes[2], sizes[3]);
    let inside =
        |x: f64, y: f64| (-1.0..=width + 1.0).contains(&x) && (-1.0..=height + 1.0).contains(&y);
    // Symbols are drawn in a viewBox of their own.
    let drawn = svg.split("<symbol").next().unwrap_or(svg);
    for tag in drawn.split('<').skip(1) {
        let name = tag
            .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .next()
            .unwrap_or("");
        let problem = match name {
            "rect" => match (
                number_attribute(tag, "x"),
                number_attribute(tag, "y"),
                number_attribute(tag, "width"),
                number_attribute(tag, "height"),
            ) {
                (Some(x), Some(y), Some(w), Some(h)) => (!(inside(x, y) && inside(x + w, y + h)))
                    .then(|| format!("rect {x} {y} {w} {h}")),
                _ => None,
            },
            "line" => match (
                number_attribute(tag, "x1"),
                number_attribute(tag, "y1"),
                number_attribute(tag, "x2"),
                number_attribute(tag, "y2"),
            ) {
                // The stroke of a pattern is drawn in a tile of its own.
                (Some(x1), Some(y1), Some(x2), Some(y2)) => {
                    let drawn_in_tile = tag.contains("chartlet-hatch-line");
                    let on_canvas = inside(x1, y1) && inside(x2, y2);
                    (!drawn_in_tile && !on_canvas).then(|| format!("line {x1} {y1} {x2} {y2}"))
                }
                _ => None,
            },
            "circle" => match (number_attribute(tag, "cx"), number_attribute(tag, "cy")) {
                (Some(x), Some(y)) => (!inside(x, y)).then(|| format!("circle {x} {y}")),
                _ => None,
            },
            "text" => match (number_attribute(tag, "x"), number_attribute(tag, "y")) {
                (Some(x), Some(y)) => (!inside(x, y)).then(|| format!("text {x} {y}")),
                _ => None,
            },
            "polyline" => {
                let points = tag
                    .split(" points=\"")
                    .nth(1)
                    .and_then(|rest| rest.split('"').next())
                    .unwrap_or("");
                points.split_whitespace().find_map(|pair| {
                    let (x, y) = pair.split_once(',')?;
                    let (x, y): (f64, f64) = (x.parse().ok()?, y.parse().ok()?);
                    (!inside(x, y)).then(|| format!("polyline point {x},{y}"))
                })
            }
            _ => None,
        };
        if let Some(problem) = problem {
            return Err(format!(
                "{problem} lies outside the {width} x {height} canvas"
            ));
        }
    }
    Ok(())
}

/// The index of the `>` that ends a tag, skipping quoted attribute values.
fn tag_end(tag: &str) -> Option<usize> {
    let mut quote = None;
    for (index, character) in tag.char_indices() {
        match (quote, character) {
            (None, '"' | '\'') => quote = Some(character),
            (Some(open), _) if open == character => quote = None,
            (None, '>') => return Some(index),
            _ => {}
        }
    }
    None
}

/// Renders every seed of `generate` twice and checks the output. `off_canvas` lists the seeds whose
/// drawing is known to leave its canvas: open findings of this test, each a layout defect that has
/// to be fixed in its layout. The list is exact in both directions: a seed that starts to fit must
/// be taken off it, and a new seed that does not fit fails the test.
fn check(kind: &str, generate: fn(u64) -> Value, off_canvas: &[u64]) {
    let mut found: Vec<(u64, String)> = Vec::new();
    for seed in 0..SEEDS {
        let specification = generate(seed).to_string();
        assert_eq!(
            specification,
            generate(seed).to_string(),
            "{kind} seed {seed}: the generator is not reproducible"
        );
        for variant in [Variant::Desktop, Variant::Print] {
            let options = RenderOptions {
                variant,
                ..RenderOptions::default()
            };
            let render = || {
                render_json(&specification, RenderFormat::Svg, &options).unwrap_or_else(|error| {
                    panic!(
                        "{kind} seed {seed} ({variant:?}) should render: {error}\n{specification}"
                    )
                })
            };
            let first = render().content;
            assert_eq!(
                first,
                render().content,
                "{kind} seed {seed} ({variant:?}) renders differently the second time"
            );
            if let Err(problem) = check_well_formed(&first) {
                panic!(
                    "{kind} seed {seed} ({variant:?}) is not well-formed SVG: {problem}\n{specification}"
                );
            }
            if let Err(problem) = check_on_canvas(&first)
                && !found.iter().any(|(known, _)| *known == seed)
            {
                found.push((seed, format!("{variant:?}: {problem}")));
            }
        }
    }
    let seeds: Vec<u64> = found.iter().map(|(seed, _)| *seed).collect();
    assert_eq!(
        seeds, off_canvas,
        "{kind}: the seeds that leave their canvas changed; found {found:#?}"
    );
}

#[test]
fn generated_flow_charts_render_deterministically() {
    check("flow", flow, &[]);
}

#[test]
fn generated_architecture_diagrams_render_deterministically() {
    check("architecture", architecture, &[]);
}

#[test]
fn generated_trees_render_deterministically() {
    check("tree", tree, &[]);
}

#[test]
fn generated_networks_render_deterministically() {
    check("network", network, &[]);
}

#[test]
fn generated_cycles_render_deterministically() {
    check("cycle", cycle, &[]);
}

#[test]
fn the_canvas_check_finds_an_element_that_is_drawn_outside() {
    let svg = |inner: &str| format!("<svg viewBox=\"0 0 100 50\">{inner}</svg>");
    assert!(check_on_canvas(&svg("<rect x=\"0\" y=\"0\" width=\"100\" height=\"50\"/>")).is_ok());
    assert!(check_on_canvas(&svg("<rect x=\"90\" y=\"0\" width=\"20\" height=\"10\"/>")).is_err());
    assert!(check_on_canvas(&svg("<text x=\"10\" y=\"80\">a</text>")).is_err());
    assert!(check_on_canvas(&svg("<polyline points=\"1,1 2,90\"/>")).is_err());
}

#[test]
fn generated_layers_render_deterministically() {
    check("layers", layers, &[]);
}

#[test]
fn generated_plans_render_deterministically() {
    check("plan", plan, &[]);
}

#[test]
fn the_well_formedness_check_finds_a_misnested_tag() {
    assert!(check_well_formed("<svg a=\"1>2\"><g><text>x</text></g></svg>").is_ok());
    assert!(check_well_formed("<svg ><g><text>x</g></text></svg>").is_err());
    assert!(check_well_formed("<svg ><g></svg>").is_err());
}
