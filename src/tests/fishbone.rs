use super::*;

const FISHBONE: &str = r#"{"schemaVersion": 1, "type": "fishbone", "title": "Late", "width": 800, "height": 420,
    "fishbone": {"effect": "Late deliveries", "bones": [
        {"label": "People", "causes": [{"label": "New drivers", "causes": [{"label": "No training"}]}, {"label": "Shifts"}]},
        {"label": "Process", "causes": [{"label": "Sorting"}]},
        {"label": "Tools"}
    ]}}"#;

fn html(spec: &str) -> String {
    render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content
}

/// The numbers after `name="` in `tag`.
fn attribute(tag: &str, name: &str) -> f64 {
    let start = tag.find(&format!("{name}=\"")).expect("attribute") + name.len() + 2;
    let end = start + tag[start..].find('"').unwrap();
    tag[start..end].parse().expect("a number")
}

#[test]
fn bones_alternate_above_and_below_the_spine() {
    let output = render_ok(FISHBONE);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    let spine = &svg[svg.find("chartlet-fish-spine").unwrap() - 80..];
    let spine = &spine[spine.find("<line").unwrap()..];
    let spine_y = attribute(&spine[..spine.find("/>").unwrap()], "y1");
    let mut sides = Vec::new();
    let mut rest = svg.as_str();
    while let Some(at) = rest.find("class=\"chartlet-fish-bone\"") {
        let start = rest[..at].rfind("<line").unwrap();
        let tag = &rest[start..at + rest[at..].find("/>").unwrap()];
        let (y1, y2) = (attribute(tag, "y1"), attribute(tag, "y2"));
        // Every bone ends on the spine and starts above or below it.
        assert!((y2 - spine_y).abs() < 0.01, "{tag}");
        sides.push(y1 < spine_y);
        rest = &rest[at + 10..];
    }
    assert_eq!(sides, vec![true, false, true]);
}

#[test]
fn no_label_is_turned() {
    let svg = render_ok(FISHBONE).content;
    assert!(!svg.contains("rotate"));
    assert!(!svg.contains("transform"));
    for label in [
        "Late deliveries",
        "People",
        "New drivers",
        "No training",
        "Tools",
    ] {
        assert!(svg.contains(&format!(">{label}</text>")), "{label}");
    }
}

#[test]
fn the_description_and_the_table_list_effect_categories_and_causes_in_order() {
    let html = html(FISHBONE);
    assert!(html.contains("Fishbone diagram of the causes of “Late deliveries”: 3 categories with 4 causes. People: New drivers (No training); Shifts. Process: Sorting. Tools."));
    assert!(html.contains(
        "<th scope=\"col\">Part</th><th scope=\"col\">Label</th><th scope=\"col\">Belongs to</th>"
    ));
    let order: Vec<usize> = [
        "<th scope=\"row\">Effect</th><td>Late deliveries</td>",
        "<th scope=\"row\">Category</th><td>People</td><td>Late deliveries</td>",
        "<th scope=\"row\">Cause</th><td>New drivers</td><td>People</td>",
        "<th scope=\"row\">Sub-cause</th><td>No training</td><td>New drivers</td>",
        "<th scope=\"row\">Cause</th><td>Shifts</td><td>People</td>",
        "<th scope=\"row\">Category</th><td>Tools</td><td>Late deliveries</td>",
    ]
    .iter()
    .map(|row| html.find(row).unwrap_or_else(|| panic!("{row}")))
    .collect();
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn the_texts_are_german_in_german() {
    let html = html(&german(FISHBONE, "fishbone"));
    assert!(html.contains(
        "Fischgräten-Diagramm zu den Ursachen von „Late deliveries“: 3 Kategorien mit 4 Ursachen."
    ));
    assert!(html.contains("<th scope=\"row\">Teilursache</th>"));
}

#[test]
fn a_canvas_that_is_too_small_grows_and_says_so() {
    // Two columns of bones and the head need 508 pixels.
    let output = render_ok(&FISHBONE.replace("\"width\": 800", "\"width\": 500"));
    assert!(output.warnings.iter().any(|w| w.path == "/width"));
    assert!(
        output
            .content
            .starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"508")
    );
    let output = render_ok(&FISHBONE.replace(
        "\"width\": 800, \"height\": 420",
        "\"width\": 320, \"height\": 160",
    ));
    assert!(output.warnings.iter().any(|w| w.path == "/height"));
}

#[test]
fn a_narrow_canvas_stacks_the_bones_beside_a_vertical_spine() {
    let svg = render_ok(&FISHBONE.replace("\"width\": 800", "\"width\": 360")).content;
    let spine = &svg[svg.find("chartlet-fish-spine").unwrap() - 120..];
    let spine = &spine[spine.find("<line").unwrap()..];
    let tag = &spine[..spine.find("/>").unwrap()];
    assert_eq!(attribute(tag, "x1"), attribute(tag, "x2"), "{tag}");
    assert!(!svg.contains("rotate"));
    assert!(svg.contains(">No training</text>"));
}

#[test]
fn fishbones_are_validated_by_name() {
    for (spec, code, path) in [
        (
            FISHBONE.replace("\"type\": \"fishbone\"", "\"type\": \"fishbone\", \"data\": [{\"label\": \"A\", \"value\": 1}]"),
            "option_not_supported",
            "/data",
        ),
        (
            FISHBONE.replace("\"Process\"", "\"People\""),
            "duplicate_label",
            "/fishbone/bones/1/label",
        ),
        (
            FISHBONE.replace("\"Late deliveries\"", "\" \""),
            "empty_text",
            "/fishbone/effect",
        ),
        (
            FISHBONE.replace(
                "\"bones\": [",
                "\"bones\": [{\"label\": \"1\"}, {\"label\": \"2\"}, {\"label\": \"3\"}, {\"label\": \"4\"}, {\"label\": \"5\"}, {\"label\": \"6\"},",
            ),
            "too_many_nodes",
            "/fishbone/bones",
        ),
    ] {
        let (found, found_path) = render_err(&spec);
        assert_eq!((found, found_path.as_str()), (code, path), "{spec}");
    }
    let (code, path) = render_err(
        r#"{"schemaVersion": 1, "type": "fishbone", "title": "x", "fishbone": {"effect": "E", "bones": []}}"#,
    );
    assert_eq!((code, path.as_str()), ("empty_data", "/fishbone/bones"));
    let (code, path) = render_err(r#"{"schemaVersion": 1, "type": "fishbone", "title": "x"}"#);
    assert_eq!((code, path.as_str()), ("missing_fishbone", "/fishbone"));
}

#[test]
fn a_third_level_of_causes_is_refused() {
    let deep = FISHBONE.replace(
        "{\"label\": \"No training\"}",
        "{\"label\": \"No training\", \"causes\": [{\"label\": \"Deeper\"}]}",
    );
    let error = render_json(&deep, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert!(error.message.contains("causes"), "{}", error.message);
}
