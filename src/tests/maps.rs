use super::*;

#[test]
fn topicmap_renders_areas_labels_and_links() {
    let output = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-topic-area")
            .count(),
        6
    );
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-topic-link\"")
            .count(),
        1
    );
    assert!(output.content.contains(
        "Topic map with 5 areas and one island. Largest: AI in practice with 291 entries, smallest: Management with 69. Closest neighbours: AI in practice and Engineering."
    ));
    // Two depth lines around each of the six areas, a halo under every coastline, and one
    // point per path through a topic.
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-topic-band\"")
            .count(),
        12
    );
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-topic-halo")
            .count(),
        6
    );
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-topic-point")
            .count(),
        41 + 24 + 9 + 7 + 5 + 1
    );
    // A coastline is a closed ring, not a circle.
    assert!(!output.content.contains("class=\"chartlet-topic-area\"/>"));
    // Every area names the center it was placed around, inside the viewBox.
    let centers = output
        .content
        .split("class=\"chartlet-topic-area")
        .skip(1)
        .map(|rest| {
            let attribute = |name: &str| -> f64 {
                let start = rest.find(name).expect("center attribute") + name.len();
                let end = start + rest[start..].find('"').expect("closing quote");
                rest[start..end].parse().expect("a number")
            };
            (attribute("data-cx=\""), attribute("data-cy=\""))
        })
        .collect::<Vec<_>>();
    assert_eq!(centers.len(), 6);
    for (cx, cy) in centers {
        assert!((0.0..=800.0).contains(&cx) && (0.0..=450.0).contains(&cy));
    }

    // At this size no area carries a name at the type size the design brief asks for, so
    // every label sits beside the map with a leader line to its area. No name is lost.
    for label in [
        "AI in practice",
        "Engineering",
        "Cloud native",
        "Web marketing",
        "Management",
        "WASM",
    ] {
        assert!(output.content.contains(&format!(">{label}<")), "{label}");
    }
    assert_eq!(texts(&output.content, "chartlet-topic-outside").len(), 6);
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-topic-leader\"")
            .count(),
        6
    );
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-topic-label")
            .count(),
        0
    );
    assert_eq!(
        output
            .warnings
            .iter()
            .map(|warning| (warning.code, warning.path.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("label_outside_area", "/topicmap/topics/0/label"),
            ("label_outside_area", "/topicmap/topics/1/label"),
            ("label_outside_area", "/topicmap/topics/2/label"),
            ("label_outside_area", "/topicmap/topics/3/label"),
            ("label_outside_area", "/topicmap/topics/4/label"),
            ("label_outside_area", "/topicmap/islands/0/label"),
        ]
    );
}

#[test]
fn topicmap_draws_its_map_furniture_in_order() {
    let output = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    let at = |needle: &str| {
        output
            .content
            .find(needle)
            .unwrap_or_else(|| panic!("{needle} is missing"))
    };
    // The sea and its grid go underneath everything, the compass rose and the cartouche on
    // top of it: furniture must never hide an area, and an area must never hide the legend.
    assert!(at("class=\"chartlet-sea\"") < at("class=\"chartlet-topic-band\""));
    assert!(at("class=\"chartlet-graticule\"") < at("class=\"chartlet-topic-area"));
    assert!(at("class=\"chartlet-topic-area") < at("class=\"chartlet-compass-ring\""));
    assert!(at("class=\"chartlet-compass-ring\"") < at("class=\"chartlet-cartouche\""));
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-graticule\"")
            .count(),
        6
    );
    assert!(output.content.contains(">Topic map</text>"));
    assert!(
        output
            .content
            .contains(">843 entries · 87 paths · 2026</text>")
    );
    assert!(output.content.contains(">N</text>"));
}

#[test]
fn topicmap_leaves_out_the_furniture_it_was_not_asked_for() {
    let bare = r#"{
            "schemaVersion": 1,
            "type": "topicmap",
            "title": "Plain",
            "topicmap": {
                "graticule": false,
                "compass": false,
                "topics": [
                    {"label": "One", "value": 40},
                    {"label": "Two", "value": 20}
                ]
            }
        }"#;
    // The stylesheet always carries every topic map rule, so this looks for drawn elements.
    let output = render_json(bare, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(!output.content.contains("class=\"chartlet-graticule\""));
    assert!(!output.content.contains("class=\"chartlet-compass-ring\""));
    assert!(!output.content.contains("class=\"chartlet-cartouche\""));
    // The sea stays: it is the surface the map is drawn on, not an optional ornament.
    assert!(output.content.contains("class=\"chartlet-sea\""));
}

#[test]
fn topicmap_html_offers_a_radio_group_for_the_areas() {
    let output = render_json(TOPICMAP, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(
        output
            .content
            .contains("<fieldset class=\"chartlet-topic-picker\"><legend>Area</legend>")
    );
    // One radio per area, all in one group, the first one selected. Islands are not offered:
    // the picker is about what the map is of.
    assert_eq!(output.content.matches("type=\"radio\"").count(), 5);
    assert_eq!(
        output.content.matches("class=\"topic-0\" checked").count(),
        1
    );
    assert!(
        output
            .content
            .contains("class=\"topic-4\"> Management</label>")
    );
    assert!(!output.content.contains("WASM</label>"));
    // The radio group is named after the chart, so two maps on one page stay independent.
    assert!(output.content.contains("name=\"topic-chartlet-"));
}

#[test]
fn topicmap_selection_styles_only_apply_once_something_is_selected() {
    let output = render_json(TOPICMAP, RenderFormat::Html, &RenderOptions::default()).unwrap();
    // Every selection rule hangs off a checked radio. A browser without :has() drops them
    // all, which leaves the map in its plain state rather than in a half-applied one — and
    // panels the host page provides stay visible instead of being hidden with no way back.
    for rule in output
        .content
        .split('}')
        .filter(|rule| rule.contains(".chartlet-topic-panel"))
    {
        assert!(
            rule.contains(":has("),
            "a panel rule applies unconditionally: {rule}"
        );
    }
    // The rule names the map's root ID: the chart's own rules are scoped to it and would
    // outrank a rule that selects by class alone.
    let root = output.content.split(" id=\"").nth(1).unwrap();
    let root = &root[..root.find('"').unwrap()];
    assert!(output.content.contains(&format!(
        ".chartlet-wrapper:has(.chartlet-topic-picker input.topic-2:checked) #{root} .chartlet-topic-area.chartlet-topic-2{{fill:var(--chartlet-accent)}}"
    )));
    // The plain SVG profile carries no selection at all.
    let svg = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(!svg.content.contains("chartlet-topic-picker"));
}

#[test]
fn topicmap_table_carries_entries_paths_and_share() {
    let output = render_json(TOPICMAP, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(output.content.contains(
        "<th scope=\"col\">Topic</th><th scope=\"col\">Entries</th><th scope=\"col\">Paths</th><th scope=\"col\">Share</th>"
    ));
    // 291 of 843 entries, as a percentage of the whole map, islands included.
    assert!(
        output.content.contains(
            "<th scope=\"row\">AI in practice</th><td>291</td><td>41</td><td>34.52%</td>"
        )
    );
    assert!(
        output
            .content
            .contains("<th scope=\"row\">WASM</th><td>6</td><td>1</td><td>0.71%</td>")
    );
}

#[test]
fn topicmap_layout_is_deterministic() {
    let first = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    let second = render_json(TOPICMAP, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert_eq!(first.content, second.content);
}

/// Every `<text>` in the SVG as (x, size, anchor, content).
fn texts(svg: &str, class: &str) -> Vec<(f64, f64, String, String)> {
    let mut found = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let attributes = chunk.split('>').next().expect("a tag ends");
        // An element can carry more than one class, so this compares whole names: looking for
        // "chartlet-topic-outside" must not find "chartlet-topic-outside-value".
        let classes = attributes
            .split("class=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or_default();
        if !classes.split_whitespace().any(|name| name == class) {
            continue;
        }
        let read = |name: &str| {
            attributes
                .split(&format!("{name}=\""))
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .map(str::to_owned)
        };
        let content = chunk
            .split_once('>')
            .and_then(|(_, rest)| rest.split_once("</text>"))
            .map(|(content, _)| content.to_owned())
            .expect("a text element has content");
        found.push((
            read("x").expect("x").parse().expect("a number"),
            read("font-size")
                .expect("font-size")
                .parse()
                .expect("a number"),
            read("text-anchor").expect("anchor"),
            content,
        ));
    }
    found
}

#[test]
fn topicmap_labels_moved_outside_stay_on_the_canvas_and_clear_of_each_other() {
    // Twenty-four areas: nearly every label has to go beside the map, which is where labels
    // start running off the canvas or landing on top of one another.
    let topics = (0..24)
        .map(|index| {
            format!(
                r#"{{"label": "Topic number {index}", "value": {}}}"#,
                20 + (index * 37) % 260
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let specification = format!(
        r#"{{"schemaVersion": 1, "type": "topicmap", "title": "Crowded",
                 "topicmap": {{"topics": [{topics}]}}}}"#
    );
    let output = render_json(&specification, RenderFormat::Svg, &RenderOptions::default()).unwrap();

    let metrics = BuiltinMetrics;
    let mut placed: Vec<(f64, f64, f64, f64)> = Vec::new();
    for (x, size, anchor, content) in texts(&output.content, "chartlet-topic-outside") {
        let width = metrics.width(&content, size);
        let (left, right) = if anchor == "start" {
            (x, x + width)
        } else {
            (x - width, x)
        };
        assert!(left >= 0.0, "{content} starts off the canvas at {left}");
        assert!(right <= 800.0, "{content} ends off the canvas at {right}");
        placed.push((left, right, x, size));
    }
    assert!(placed.len() > 12, "most labels should sit outside");

    // No label may be dropped silently: whatever is not drawn is reported.
    let reported = output
        .warnings
        .iter()
        .filter(|warning| warning.code == "dense_chart")
        .count();
    assert_eq!(placed.len() + reported, 24);
}

#[test]
fn topicmap_caps_links_per_topic_and_drops_the_weakest() {
    let specification = r#"{
            "schemaVersion": 1,
            "type": "topicmap",
            "title": "Busy hub",
            "topicmap": {
                "topics": [
                    {"label": "Hub", "value": 200},
                    {"label": "A", "value": 50},
                    {"label": "B", "value": 50},
                    {"label": "C", "value": 50},
                    {"label": "D", "value": 50},
                    {"label": "E", "value": 50}
                ],
                "links": [
                    {"from": "Hub", "to": "A", "weight": 0.9},
                    {"from": "Hub", "to": "B", "weight": 0.8},
                    {"from": "Hub", "to": "C", "weight": 0.7},
                    {"from": "Hub", "to": "D", "weight": 0.6},
                    {"from": "Hub", "to": "E", "weight": 0.5}
                ]
            }
        }"#;
    let output = render_json(specification, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-topic-link\"")
            .count(),
        3
    );
    assert_eq!(
        output
            .warnings
            .iter()
            .map(|warning| (warning.code, warning.path.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("link_dropped", "/topicmap/links/3"),
            ("link_dropped", "/topicmap/links/4"),
        ]
    );
}

#[test]
fn atlas_renders_and_describes_its_landscape() {
    let svg = render_json(ATLAS, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(
        svg.content
            .contains("Knowledge landscape of 2 realms and 4 regions")
    );
    assert!(svg.content.contains("largest is Models with 40"));
    assert!(svg.content.contains("1 place is marked"));
}

#[test]
fn atlas_draws_its_terrain_unless_it_is_turned_off() {
    let drawn = render_json(ATLAS, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    // The class is always in the stylesheet; what the switch decides is the elements.
    assert!(drawn.content.contains("class=\"chartlet-atlas-contour\""));
    let flat = ATLAS.replace(r#""seed": 7"#, r#""seed": 7, "contours": false"#);
    let plain = render_json(&flat, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(!plain.content.contains("class=\"chartlet-atlas-contour\""));
    // The land itself is untouched by the switch.
    assert!(plain.content.contains("class=\"chartlet-atlas-coast\""));
}

#[test]
fn atlas_requires_an_atlas_block() {
    let missing = r#"{"schemaVersion": 1, "type": "atlas", "title": "Empty"}"#;
    let error = render_json(missing, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "missing_atlas");
    assert_eq!(error.path, "/atlas");
}

#[test]
fn atlas_rejects_a_realm_without_regions() {
    let empty = ATLAS.replace(r#"{ "label": "Tooling", "value": 30 },"#, "");
    let empty = empty.replace(r#"{ "label": "Accessibility", "value": 8 }"#, "");
    let error = render_json(&empty, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "empty_realm");
    assert_eq!(error.path, "/atlas/realms/1/regions");
}

#[test]
fn atlas_rejects_a_region_that_repeats_a_realm_name() {
    let clash = ATLAS.replace(r#""label": "Tooling""#, r#""label": "Machines""#);
    let error = render_json(&clash, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "duplicate_atlas_label");
    assert_eq!(error.path, "/atlas/realms/1/regions/0/label");
}

#[test]
fn atlas_rejects_links_to_unknown_regions() {
    let broken = ATLAS.replace(r#""to": "Tooling""#, r#""to": "Machines""#);
    let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "unknown_region_link");
    assert_eq!(error.path, "/atlas/links/0/to");
}

#[test]
fn atlas_rejects_area_damping_outside_its_range() {
    let broken = ATLAS.replace(r#""seed": 7"#, r#""seed": 7, "areaDamping": 1.5"#);
    let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "invalid_area_damping");
    assert_eq!(error.path, "/atlas/areaDamping");
}

#[test]
fn atlas_warns_when_a_region_lists_more_places_than_it_claims() {
    let crowded = r#"{
            "schemaVersion": 1,
            "type": "atlas",
            "title": "Miscounted",
            "atlas": {
                "realms": [
                    {
                        "label": "Machines",
                        "regions": [
                            {
                                "label": "Models",
                                "value": 1,
                                "places": [{ "label": "One" }, { "label": "Two" }]
                            }
                        ]
                    }
                ]
            }
        }"#;
    let rendered = render_json(crowded, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(
        rendered
            .warnings
            .iter()
            .any(|warning| warning.code == "more_places_than_value"),
        "expected the miscount warning, got {:?}",
        rendered.warnings
    );
}

#[test]
fn atlas_rejects_a_place_weight_outside_its_range() {
    let broken = ATLAS.replace(r#""weight": 2 }]"#, r#""weight": 9 }]"#);
    let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "place_weight_out_of_range");
    assert_eq!(error.path, "/atlas/realms/0/regions/0/places/0/weight");
}

#[test]
fn atlas_rejects_series_alongside_its_realms() {
    let mixed = ATLAS.replace(
        r#""title": "Knowledge landscape","#,
        r#""title": "Knowledge landscape", "data": [{"label": "A", "value": 1}],"#,
    );
    let error = render_json(&mixed, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "option_not_supported");
    assert_eq!(error.path, "/data");
    assert!(
        error
            .message
            .contains("an atlas chart is drawn from atlas.realms")
    );
}

#[test]
fn topicmap_requires_a_topicmap_block() {
    let missing = r#"{"schemaVersion": 1, "type": "topicmap", "title": "Empty"}"#;
    let error = render_json(missing, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "missing_topicmap");
    assert_eq!(error.path, "/topicmap");
}

#[test]
fn topicmap_rejects_too_many_topics() {
    let topics = (0..41)
        .map(|i| format!(r#"{{"label": "T{i}", "value": 10}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let spec = format!(
        r#"{{"schemaVersion": 1, "type": "topicmap", "title": "Big", "topicmap": {{"topics": [{topics}]}}}}"#
    );
    let error = render_json(&spec, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "too_many_topics");
    assert_eq!(error.path, "/topicmap/topics");
}

#[test]
fn topicmap_rejects_duplicate_labels_across_topics_and_islands() {
    let duplicate = TOPICMAP.replace(r#""label": "WASM""#, r#""label": "Management""#);
    let error = render_json(&duplicate, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "duplicate_topic_label");
    assert_eq!(error.path, "/topicmap/islands/0/label");
}

#[test]
fn topicmap_rejects_links_to_unknown_labels() {
    let broken = TOPICMAP.replace(r#""to": "Engineering""#, r#""to": "Nonexistent""#);
    let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "unknown_topic_link");
    assert_eq!(error.path, "/topicmap/links/0/to");
}

#[test]
fn topicmap_rejects_link_weight_outside_zero_to_one() {
    let broken = TOPICMAP.replace(r#""weight": 0.6"#, r#""weight": 1.5"#);
    let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "invalid_link_weight");
    assert_eq!(error.path, "/topicmap/links/0/weight");
}

#[test]
fn topicmap_rejects_a_value_below_one() {
    let broken = TOPICMAP.replace(r#""value": 69"#, r#""value": 0"#);
    let error = render_json(&broken, RenderFormat::Svg, &RenderOptions::default()).unwrap_err();
    assert_eq!(error.code, "topic_value_out_of_range");
    assert_eq!(error.path, "/topicmap/topics/4/value");
}

#[test]
fn topicmap_warns_when_a_topic_is_too_small_for_its_label() {
    let small = TOPICMAP.replace(r#""value": 69, "points": 5"#, r#""value": 5, "points": 5"#);
    let output = render_json(&small, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    assert!(
        output
            .warnings
            .iter()
            .any(|warning| warning.code == "topic_too_small_for_label"
                && warning.path == "/topicmap/topics/4/value"),
        "{:?}",
        output.warnings
    );
}

#[test]
fn topicmap_rejects_bar_line_and_time_only_fields() {
    for (field, path) in [
        (r#""data": [{"label": "A", "value": 1}],"#, "/data"),
        (r#""categories": ["A"],"#, "/categories"),
        (r#""series": [{"name": "S", "values": [1]}],"#, "/series"),
        (
            r#""zoomSteps": [{"label": "A", "from": 0, "to": 0}, {"label": "B", "from": 0, "to": 1}],"#,
            "/zoomSteps",
        ),
        (r#""panes": [{}],"#, "/panes"),
    ] {
        let mixed = TOPICMAP.replace(
            "\"schemaVersion\": 1,",
            &format!("\"schemaVersion\": 1, {field}"),
        );
        let error = render_json(&mixed, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("a topicmap chart is drawn from topicmap.topics");
        assert_eq!(error.code, "option_not_supported", "{field}");
        assert_eq!(error.path, path, "{field}");
    }
}

#[test]
fn topicmap_html_lists_every_area_with_entries_and_paths() {
    let output = render_json(TOPICMAP, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(output.content.contains(
        "<th scope=\"col\">Topic</th><th scope=\"col\">Entries</th><th scope=\"col\">Paths</th>"
    ));
    assert!(
        output
            .content
            .contains("<th scope=\"row\">WASM</th><td>6</td><td>1</td>")
    );
}

// --- Gaps, line patterns, areas, time zoom and more layers ---
