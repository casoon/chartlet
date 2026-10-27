use crate::{ChartSpec, RenderFormat, RenderOptions, advise_json, render_json};

const BASE: &str = r#"{ "schemaVersion": 1, "type": "bar", "title": "x", "data": [ { "label": "a", "value": 1 } ] }"#;

#[test]
fn a_payload_beyond_the_cap_is_refused_before_it_is_parsed() {
    let padded = format!("{}{}", BASE, " ".repeat(5 * 1024 * 1024));
    let error = ChartSpec::from_json(&padded).expect_err("too large");
    assert_eq!((error.code, error.path.as_str()), ("spec_too_large", "/"));
    assert!(error.message.contains("4 MiB"));
    let error =
        render_json(&padded, RenderFormat::Svg, &RenderOptions::default()).expect_err("too large");
    assert_eq!(error.code, "spec_too_large");
}

#[test]
fn a_payload_within_the_cap_is_read() {
    let padded = format!("{}{}", BASE, " ".repeat(1024 * 1024));
    assert!(ChartSpec::from_json(&padded).is_ok());
}

#[test]
fn deep_nesting_and_a_failed_render_leave_the_next_render_unharmed() {
    let nested = format!("{}{}", "[".repeat(50_000), "]".repeat(50_000));
    assert!(ChartSpec::from_json(&nested).is_err());
    let first = render_json(BASE, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
    let second = render_json(BASE, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
    assert_eq!(first.content, second.content);
}

#[test]
fn escaped_text_never_reaches_the_output_unescaped() {
    let spec = r#"{ "schemaVersion": 1, "type": "bar", "title": "<script>alert(1)</script> & \"q\"",
        "data": [ { "label": "</text><g onload=x>", "value": 1 } ] }"#;
    let output = render_json(spec, RenderFormat::Html, &RenderOptions::default()).expect("renders");
    assert!(!output.content.contains("<script>alert"));
    assert!(!output.content.contains("<g onload"));
    assert!(output.content.contains("&lt;script&gt;"));
}

/// The size the render asks for with `canvas_too_small`, as a number.
fn needed(spec: &str, path: &str) -> Option<u32> {
    let output = render_json(spec, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
    let warning = output
        .warnings
        .iter()
        .find(|warning| warning.code == "canvas_too_small" && warning.path == path)?;
    warning
        .message
        .split("needs ")
        .nth(1)?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// Issue 6: groups and boundaries add a frame around their members. The size a diagram asks for
/// stays plausible, and rendering again at that size resolves the warning.
#[test]
fn groups_and_boundaries_ask_for_a_plausible_canvas_that_resolves_the_warning() {
    let nodes: Vec<String> = (0..12)
        .map(|index| format!(r#"{{ "id": "n{index}", "label": "Step {index}" }}"#))
        .collect();
    let edges: Vec<String> = (0..11)
        .map(|index| format!(r#"{{ "from": "n{index}", "to": "n{}" }}"#, index + 1))
        .collect();
    let flow = format!(
        r#"{{ "schemaVersion": 1, "type": "flow", "title": "Grouped", "width": 320, "height": 300,
            "flow": {{ "nodes": [{}], "edges": [{}],
              "groups": [ {{ "label": "A", "nodes": ["n0","n1","n2"] }}, {{ "label": "B", "nodes": ["n3","n4","n5"] }} ] }} }}"#,
        nodes.join(","),
        edges.join(",")
    );
    let components: Vec<String> = (0..9)
        .map(|index| {
            format!(
                r#"{{ "id": "c{index}", "label": "Component {index}", "in": "b{}" }}"#,
                index % 3
            )
        })
        .collect();
    let connections: Vec<String> = (0..8)
        .map(|index| format!(r#"{{ "from": "c{index}", "to": "c{}" }}"#, index + 1))
        .collect();
    let architecture = format!(
        r#"{{ "schemaVersion": 1, "type": "architecture", "title": "Nested", "width": 400, "height": 400,
            "architecture": {{ "boundaries": [ {{ "id": "b0", "label": "Zone" }},
              {{ "id": "b1", "label": "Inner", "in": "b0" }}, {{ "id": "b2", "label": "Core", "in": "b1" }} ],
              "components": [{}], "connections": [{}] }} }}"#,
        components.join(","),
        connections.join(",")
    );
    for (name, spec) in [("flow", flow), ("architecture", architecture)] {
        for path in ["/width", "/height"] {
            if let Some(size) = needed(&spec, path) {
                assert!(
                    (200..=6_000).contains(&size),
                    "{name} asks for {size} pixels at {path}"
                );
            }
        }
        for help in advise_json(&spec, &RenderOptions::default()).expect("renders") {
            if help.code == "canvas_too_small" {
                assert!(
                    help.remedies.iter().all(|remedy| remedy.resolves),
                    "{name}: the recommended size at {} resolves the warning",
                    help.path
                );
            }
        }
    }
}
