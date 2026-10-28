//! The editorial look of diagrams: opt-in, neutral, tagged, with label masks that keep clear.

use std::collections::BTreeMap;

use chartlet::{RenderFormat, RenderOptions, Variant, render_json};

fn svg(spec: &str) -> String {
    render_json(spec, RenderFormat::Svg, &RenderOptions::default())
        .expect("renders")
        .content
}

fn example(name: &str) -> String {
    std::fs::read_to_string(format!("examples/{name}.json")).expect("example")
}

/// The rectangles of `svg` with `class` among their classes, as x, y, width, height.
fn rects(svg: &str, class: &str) -> Vec<(f64, f64, f64, f64)> {
    let mut found = Vec::new();
    for part in svg.split("<rect ").skip(1) {
        let tag = &part[..part.find('>').expect("closed")];
        let attribute = |name: &str| -> Option<f64> {
            let start = tag.find(&format!("{name}=\""))? + name.len() + 2;
            tag[start..start + tag[start..].find('"')?].parse().ok()
        };
        let classes = tag
            .split("class=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or("");
        if classes.split(' ').any(|own| own == class)
            && let (Some(x), Some(y), Some(width), Some(height)) = (
                attribute("x"),
                attribute("y"),
                attribute("width"),
                attribute("height"),
            )
        {
            found.push((x, y, width, height));
        }
    }
    found
}

fn overlap(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

#[test]
fn classic_stays_the_default_and_draws_no_tags() {
    let classic = svg(&example("editorial-architecture").replace("\"look\": \"editorial\",", ""));
    assert!(!classic.contains("chartlet-look-editorial"));
    assert!(!classic.contains("class=\"chartlet-diagram-tag\""));
    let named = svg(&example("editorial-architecture").replace("editorial", "classic"));
    assert_eq!(classic, named);
}

#[test]
fn the_editorial_look_tags_nodes_and_can_switch_tags_off() {
    let tagged = svg(&example("editorial-architecture"));
    assert!(tagged.contains("chartlet-look-editorial"));
    for tag in ["SVC", "DB", "QUEUE", "EXT", "UI", "USER"] {
        assert!(tagged.contains(&format!(">{tag}</text>")), "{tag}");
    }
    let off =
        svg(&example("editorial-architecture").replace("\"look\"", "\"tags\": false, \"look\""));
    assert!(!off.contains("class=\"chartlet-diagram-tag\""));
    let own = svg(&example("editorial-architecture").replace(
        "\"label\": \"Shop API\"",
        "\"label\": \"Shop API\", \"tag\": \"api\"",
    ));
    assert!(own.contains(">API</text>"));
}

#[test]
fn a_narrow_canvas_drops_the_tags() {
    let narrow = render_json(
        &example("editorial-architecture"),
        RenderFormat::Svg,
        &RenderOptions {
            variant: Variant::Mobile,
            ..RenderOptions::default()
        },
    )
    .expect("renders")
    .content;
    assert!(!narrow.contains("class=\"chartlet-diagram-tag\""));
}

#[test]
fn a_tag_that_is_too_long_or_in_the_wrong_place_is_refused() {
    let long = example("editorial-architecture").replace(
        "\"label\": \"Shop API\"",
        "\"label\": \"Shop API\", \"tag\": \"much too long\"",
    );
    let error =
        render_json(&long, RenderFormat::Svg, &RenderOptions::default()).expect_err("refused");
    assert_eq!(error.code, "text_too_long");
    assert_eq!(error.path, "/architecture/components/2/tag");
    let bar = example("editorial-bars").replace("\"look\"", "\"tags\": false, \"look\"");
    let error =
        render_json(&bar, RenderFormat::Svg, &RenderOptions::default()).expect_err("refused");
    assert_eq!(
        (error.code, error.path.as_str()),
        ("option_not_supported", "/tags")
    );
}

#[test]
fn label_masks_clear_nodes_and_each_other() {
    for (name, node_class) in [
        ("editorial-flow", "chartlet-flow-node"),
        ("editorial-architecture", "chartlet-flow-node"),
        ("editorial-states", "chartlet-flow-node"),
        ("editorial-dense-architecture", "chartlet-flow-node"),
        ("editorial-dense-flow", "chartlet-flow-node"),
        ("editorial-sequence", "chartlet-seq-box"),
    ] {
        let svg = svg(&example(name));
        let masks = rects(&svg, "chartlet-diagram-chip");
        let nodes = rects(&svg, node_class);
        for (index, mask) in masks.iter().enumerate() {
            for node in &nodes {
                assert!(!overlap(*mask, *node), "{name}: a mask covers a node");
            }
            for other in &masks[index + 1..] {
                assert!(!overlap(*mask, *other), "{name}: two masks overlap");
            }
        }
    }
}

#[test]
fn masks_have_no_border_and_sublabels_are_monospace() {
    let svg = svg(&example("editorial-architecture"));
    assert!(svg.contains(".chartlet-look-editorial .chartlet-diagram-chip{fill:var(--chartlet-background);stroke:none"));
    assert!(svg.contains("font-family:ui-monospace"));
    assert!(svg.contains(">calls · </text>"));
}

/// The custom properties of the rule with `selector` in the shared stylesheet.
fn palette(selector: &str) -> BTreeMap<String, String> {
    let sheet = chartlet::stylesheet();
    let start = sheet.find(&format!("{selector}{{")).expect("rule") + selector.len() + 1;
    let body = &sheet[start..start + sheet[start..].find('}').expect("closed")];
    body.split(';')
        .filter_map(|declaration| {
            let (name, value) = declaration.split_once(':')?;
            Some((
                name.strip_prefix("--chartlet-")?.to_owned(),
                value.to_owned(),
            ))
        })
        .collect()
}

fn luminance(hex: &str) -> f64 {
    let channel = |at: usize| {
        let value = f64::from(u8::from_str_radix(&hex[at..at + 2], 16).expect("hex")) / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5)
}

fn ratio(a: &str, b: &str) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn the_editorial_palettes_keep_their_contrast() {
    for selector in [
        ".chartlet-root.chartlet-look-editorial",
        ".chartlet-root.chartlet-look-editorial.chartlet-theme-dark:not(.chartlet-blueprint)",
    ] {
        let tokens = palette(selector);
        for ground in ["background", "node"] {
            for text in ["text", "muted"] {
                let value = ratio(&tokens[text], &tokens[ground]);
                assert!(value >= 4.5, "{selector}: {text} on {ground} is {value:.2}");
            }
            for mark in [
                "zero",
                "accent",
                "role-red",
                "role-violet",
                "color-1",
                "color-2",
            ] {
                let value = ratio(&tokens[mark], &tokens[ground]);
                assert!(value >= 3.0, "{selector}: {mark} on {ground} is {value:.2}");
            }
        }
        // The accent text on the paper, as in the legend of an emphasized element.
        assert!(
            ratio(&tokens["accent"], &tokens["background"]) >= 4.5,
            "{selector}"
        );
    }
}

#[test]
fn trees_and_class_diagrams_tag_their_kinds() {
    let tree = std::fs::read_to_string("examples/family-tree.json")
        .expect("example")
        .replacen('{', "{\"look\":\"editorial\",", 1);
    let tree = svg(&tree);
    assert!(tree.contains(">USER</text>") || tree.contains(">EXT</text>"));
    let classes = svg(&example("editorial-classes"));
    assert!(classes.contains(">ENUM</text>"));
}

#[test]
fn only_the_technology_of_an_edge_label_is_monospace() {
    let svg = svg(&example("editorial-architecture"));
    assert!(svg.contains("class=\"chartlet-diagram-tech\""));
    assert!(svg.contains(">HTTPS/JSON</text>"));
    assert!(svg.contains(">calls · </text>"));
}

#[test]
fn low_contrast_is_measured_against_the_editorial_palette() {
    let spec = example("editorial-architecture")
        .replace("\"look\"", "\"tokens\": {\"text\": \"#f0eee8\"}, \"look\"");
    let output = render_json(&spec, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "low_contrast")
    );
}
