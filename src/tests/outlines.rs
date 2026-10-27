use super::*;

const VENN: &str = r#"{"schemaVersion": 1, "type": "venn", "title": "Skills", "width": 760, "height": 520,
    "venn": {"sets": ["Design", "Code", "Research"], "regions": [
        {"sets": ["Design"], "label": "Visual", "count": 3},
        {"sets": ["Code", "Design"], "label": "Prototyping", "count": 4},
        {"sets": ["Design", "Code", "Research"], "count": 1}
    ]}}"#;

const TWO: &str = r#"{"schemaVersion": 1, "type": "venn", "title": "Overlap", "width": 600, "height": 360,
    "venn": {"sets": ["Buyers", "Readers"], "regions": [
        {"sets": ["Buyers"], "label": "Buy only", "count": 40},
        {"sets": ["Buyers", "Readers"], "label": "Both", "count": 15}
    ]}}"#;

fn html(spec: &str) -> String {
    render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content
}

#[test]
fn two_and_three_sets_are_drawn_as_circles() {
    for (spec, circles) in [(VENN, 3), (TWO, 2)] {
        let output = render_ok(spec);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-venn chartlet-venn-")
                .count(),
            circles
        );
    }
}

#[test]
fn labels_stand_inside_where_they_fit_and_each_circle_is_named() {
    let svg = render_ok(VENN).content;
    for text in ["Visual", "Prototyping", "Design", "Code", "Research"] {
        assert!(svg.contains(&format!(">{text}</text>")), "{text}");
    }
    // Everything fits at this size: no leader lines.
    assert_eq!(svg.matches("class=\"chartlet-venn-leader\"").count(), 0);
    // The count of each region that has one is written, in the region.
    assert_eq!(svg.matches("class=\"chartlet-venn-count\"").count(), 3);
}

#[test]
fn a_label_that_does_not_fit_moves_to_a_leader_line() {
    let long = VENN.replace("\"Visual\"", "\"Interdisciplinary specialists\"");
    let output = render_ok(&long.replace(
        "\"width\": 760, \"height\": 520",
        "\"width\": 520, \"height\": 360",
    ));
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-venn-leader\"").count(), 1);
    assert_eq!(svg.matches("class=\"chartlet-venn-dot\"").count(), 1);
}

#[test]
fn the_description_and_the_table_name_every_region() {
    let html = html(VENN);
    assert!(html.contains("Venn diagram of 3 sets: Design, Code and Research. only Design: Visual (3). Design and Code: Prototyping (4). Design, Code and Research: 1."));
    assert!(html.contains(
        "<th scope=\"col\">Region</th><th scope=\"col\">Label</th><th scope=\"col\">Count</th>"
    ));
    // Seven rows, also for the regions nobody said anything about.
    assert_eq!(html.matches("<th scope=\"row\">").count(), 7);
    assert!(html.contains("<th scope=\"row\">only Code</th><td></td><td></td>"));
    assert!(html.contains("<th scope=\"row\">Design and Code</th><td>Prototyping</td><td>4</td>"));
}

#[test]
fn the_texts_are_german_in_german() {
    let html = html(&german(TWO, "venn"));
    assert!(html.contains("Venn-Diagramm aus 2 Mengen: Buyers und Readers. nur Buyers: Buy only (40). Buyers und Readers: Both (15)."));
    assert!(html.contains("<th scope=\"col\">Bereich</th>"));
}

#[test]
fn venn_diagrams_are_validated_by_name() {
    for (spec, code, path) in [
        (
            VENN.replace("[\"Design\", \"Code\", \"Research\"]", "[\"Design\"]"),
            "not_enough_sets",
            "/venn/sets",
        ),
        (
            VENN.replace(
                "[\"Design\", \"Code\", \"Research\"]",
                "[\"Design\", \"Code\", \"Research\", \"Ops\"]",
            ),
            "too_many_sets",
            "/venn/sets",
        ),
        (
            VENN.replace("\"Code\", \"Research\"]", "\"Design\", \"Research\"]"),
            "duplicate_label",
            "/venn/sets/1",
        ),
        (
            VENN.replace("[\"Code\", \"Design\"]", "[\"Code\", \"Ops\"]"),
            "unknown_set",
            "/venn/regions/1/sets/1",
        ),
        (
            VENN.replace("[\"Code\", \"Design\"]", "[\"Design\"]"),
            "duplicate_region",
            "/venn/regions/1/sets",
        ),
        (
            VENN.replace("\"count\": 4", "\"count\": -4"),
            "invalid_value",
            "/venn/regions/1/count",
        ),
        (
            VENN.replace("\"Visual\"", "\" \""),
            "empty_text",
            "/venn/regions/0/label",
        ),
        (
            VENN.replace("\"type\": \"venn\"", "\"type\": \"venn\", \"series\": []")
                .replace(
                    "\"series\": []",
                    "\"series\": [{\"name\": \"A\", \"values\": [1]}]",
                ),
            "option_not_supported",
            "/series",
        ),
    ] {
        let (found, found_path) = render_err(&spec);
        assert_eq!((found, found_path.as_str()), (code, path), "{spec}");
    }
    let (code, path) = render_err(r#"{"schemaVersion": 1, "type": "venn", "title": "x"}"#);
    assert_eq!((code, path.as_str()), ("missing_venn", "/venn"));
}

const KANBAN: &str = r#"{"schemaVersion": 1, "type": "kanban", "title": "Board", "width": 800, "height": 400,
    "kanban": {"columns": [
        {"label": "To do", "limit": 4, "cards": [{"title": "A", "owner": "Ann", "tag": "Bug"}, {"title": "B"}]},
        {"label": "Doing", "limit": 1, "cards": [{"title": "C", "blocked": true}, {"title": "D"}]},
        {"label": "Done"}
    ]}}"#;

#[test]
fn a_kanban_board_shows_loads_and_marks_what_is_over_the_limit_and_blocked() {
    let output = render_ok(KANBAN);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-kan-lane").count(), 3);
    assert!(svg.contains(">2/4</text>"));
    assert!(svg.contains(">2/1</text>"));
    // The column over its limit has its own outline, a sign and a red load; the others do not.
    assert_eq!(
        svg.matches("class=\"chartlet-kan-lane chartlet-kan-over\"")
            .count(),
        1
    );
    assert_eq!(
        svg.matches("class=\"chartlet-kan-load chartlet-kan-over-text\"")
            .count(),
        1
    );
    assert_eq!(svg.matches(">!</text>").count(), 1);
    // The blocked card has a sign and the word, not only a color.
    assert_eq!(
        svg.matches("class=\"chartlet-kan-card chartlet-kan-blocked\"")
            .count(),
        1
    );
    assert_eq!(svg.matches(">Blocked</text>").count(), 1);
}

#[test]
fn kanban_columns_stand_one_under_the_other_on_a_narrow_canvas() {
    let x_of = |svg: &str| -> Vec<String> {
        svg.match_indices("class=\"chartlet-kan-lane")
            .map(|(at, _)| {
                let start = svg[..at].rfind("<rect").unwrap();
                svg[start..at]
                    .split("x=\"")
                    .nth(1)
                    .unwrap()
                    .split('"')
                    .next()
                    .unwrap()
                    .to_owned()
            })
            .collect()
    };
    let wide = x_of(&render_ok(KANBAN).content);
    assert_eq!(wide.len(), 3);
    assert_ne!(wide[0], wide[1]);
    let narrow = x_of(&render_ok(&KANBAN.replace("800", "360")).content);
    assert!(narrow.iter().all(|x| *x == narrow[0]), "{narrow:?}");
}

#[test]
fn the_kanban_description_and_table_count_cards_and_name_blocked_ones() {
    let page = html(KANBAN);
    assert!(page.contains("Kanban board with 3 columns and 4 cards. To do: 2 cards (limit 4). Doing: 2 cards, over its limit of 1. Done: 0 cards. Blocked: “C” in Doing."));
    assert!(page.contains(
        "<th scope=\"col\">Column</th><th scope=\"col\">Cards</th><th scope=\"col\">Card</th>"
    ));
    assert!(page.contains(
        "<th scope=\"row\">To do</th><td>2/4</td><td>A</td><td>Ann</td><td>Bug</td><td></td>"
    ));
    assert!(page.contains(
        "<th scope=\"row\">Doing</th><td>2/1</td><td>C</td><td></td><td></td><td>Blocked</td>"
    ));
    assert!(page.contains("<th scope=\"row\">Done</th><td>0</td><td></td>"));
    let german = html(&german(KANBAN, "kanban"));
    assert!(german.contains("Kanban-Tafel mit 3 Spalten und 4 Karten."));
    assert!(german.contains("Blockiert: „C“ in Doing."));
}

#[test]
fn kanban_boards_are_validated_by_name() {
    for (spec, code, path) in [
        (
            KANBAN.replace("\"limit\": 4", "\"limit\": 0"),
            "invalid_limit",
            "/kanban/columns/0/limit",
        ),
        (
            KANBAN.replace("\"Done\"", "\"To do\""),
            "duplicate_label",
            "/kanban/columns/2/label",
        ),
        (
            KANBAN.replace("\"title\": \"B\"", "\"title\": \" \""),
            "empty_text",
            "/kanban/columns/0/cards/1/title",
        ),
        (
            KANBAN.replace(
                "\"columns\": [",
                "\"columns\": [{\"label\": \"1\"}, {\"label\": \"2\"}, {\"label\": \"3\"}, {\"label\": \"4\"}, {\"label\": \"5\"}, {\"label\": \"6\"}, {\"label\": \"7\"},",
            ),
            "too_many_nodes",
            "/kanban/columns",
        ),
    ] {
        let (found, found_path) = render_err(&spec);
        assert_eq!((found, found_path.as_str()), (code, path), "{spec}");
    }
    let (code, path) = render_err(
        r#"{"schemaVersion": 1, "type": "kanban", "title": "x", "kanban": {"columns": []}}"#,
    );
    assert_eq!((code, path.as_str()), ("empty_data", "/kanban/columns"));
    let (code, _) = render_err(r#"{"schemaVersion": 1, "type": "kanban", "title": "x"}"#);
    assert_eq!(code, "missing_kanban");
}

const STORYMAP: &str = r#"{"schemaVersion": 1, "type": "storymap", "title": "Map", "width": 600, "height": 480,
    "storymap": {"releases": ["MVP", "Later"], "activities": [
        {"label": "Find", "tasks": [
            {"label": "Search", "stories": [{"title": "By name", "release": "MVP"}, {"title": "Suggest", "release": "Later"}]},
            {"label": "Browse"}, {"label": "Filter"}]},
        {"label": "Buy", "tasks": [{"label": "Pay", "stories": [{"title": "Card", "release": "MVP"}]}]}
    ]}}"#;

#[test]
fn a_story_map_has_columns_slices_and_cut_lines() {
    let output = render_ok(STORYMAP);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-story-cut\"").count(), 2);
    assert_eq!(svg.matches("class=\"chartlet-story-activity\"").count(), 2);
    assert_eq!(svg.matches("class=\"chartlet-story-task\"").count(), 4);
    assert_eq!(svg.matches("class=\"chartlet-story-card\"").count(), 3);
    // The release stands once, on its cut line.
    assert_eq!(svg.matches(">MVP</text>").count(), 1);
}

#[test]
fn a_narrow_story_map_becomes_a_list_with_the_release_beside_each_story() {
    let svg = render_ok(&STORYMAP.replace("600", "480")).content;
    assert_eq!(svg.matches("class=\"chartlet-story-cut\"").count(), 0);
    assert_eq!(svg.matches(">MVP</text>").count(), 2);
    assert_eq!(svg.matches(">Later</text>").count(), 1);
}

#[test]
fn the_story_map_text_lists_stories_by_activity_task_and_release() {
    let page = html(STORYMAP);
    assert!(page.contains("Story map of 2 activities, 4 tasks and 3 stories in 2 releases. Find: Search (MVP: By name; Later: Suggest); Browse; Filter. Buy: Pay (MVP: Card)."));
    assert!(page.contains("<th scope=\"col\">Activity</th><th scope=\"col\">Task</th><th scope=\"col\">Release</th><th scope=\"col\">Story</th>"));
    assert!(
        page.contains("<th scope=\"row\">Find</th><td>Search</td><td>MVP</td><td>By name</td>")
    );
    assert!(page.contains("<th scope=\"row\">Find</th><td>Browse</td><td></td><td></td>"));
    assert!(
        html(&german(STORYMAP, "storymap"))
            .contains("Story Map aus 2 Aktivitäten, 4 Aufgaben und 3 Stories in 2 Releases.")
    );
}

#[test]
fn story_maps_are_validated_by_name() {
    for (spec, code, path) in [
        (
            STORYMAP.replace("\"release\": \"Later\"", "\"release\": \"Soon\""),
            "unknown_release",
            "/storymap/activities/0/tasks/0/stories/1/release",
        ),
        (
            STORYMAP.replace("[\"MVP\", \"Later\"]", "[\"MVP\", \"MVP\"]"),
            "duplicate_label",
            "/storymap/releases/1",
        ),
        (
            STORYMAP.replace("\"Buy\"", "\"Find\""),
            "duplicate_label",
            "/storymap/activities/1/label",
        ),
    ] {
        let (found, found_path) = render_err(&spec);
        assert_eq!((found, found_path.as_str()), (code, path), "{spec}");
    }
    let (code, path) = render_err(
        r#"{"schemaVersion": 1, "type": "storymap", "title": "x", "storymap": {"releases": ["A"], "activities": [{"label": "x", "tasks": []}]}}"#,
    );
    assert_eq!(
        (code, path.as_str()),
        ("empty_data", "/storymap/activities/0/tasks")
    );
}

const JOURNEY: &str = r#"{"schemaVersion": 1, "type": "journey", "title": "Trip", "width": 700, "height": 420,
    "journey": {"stages": [
        {"label": "A", "action": "Opens", "sentiment": -1},
        {"label": "B", "action": "Reads", "thought": "Hmm", "sentiment": 2}
    ]}}"#;

#[test]
fn a_journey_draws_rows_and_a_sentiment_line_with_every_value_written() {
    let output = render_ok(JOURNEY);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(
        svg.matches("class=\"chartlet-journey-sentiment\"").count(),
        1
    );
    assert_eq!(svg.matches("class=\"chartlet-journey-dot\"").count(), 2);
    assert_eq!(svg.matches("class=\"chartlet-journey-value\"").count(), 2);
    for text in ["Actions", "Thoughts", "Sentiment", "Opens", "Hmm"] {
        assert!(svg.contains(&format!(">{text}</text>")), "{text}");
    }
    // A row that no stage fills is left out, and a journey has no blueprint lines.
    assert!(!svg.contains(">Touchpoints</text>"));
    assert!(!svg.contains("chartlet-journey-line\""));
}

#[test]
fn a_blueprint_adds_the_service_rows_between_two_lines() {
    let blueprint = JOURNEY
        .replace(
            "\"journey\": {",
            "\"journey\": {\"display\": \"blueprint\",",
        )
        .replace(
            "\"action\": \"Opens\"",
            "\"action\": \"Opens\", \"frontstage\": \"Greets\", \"backstage\": \"Routes\"",
        );
    let svg = render_ok(&blueprint).content;
    assert_eq!(svg.matches("class=\"chartlet-journey-line\"").count(), 2);
    for text in [
        "Line of interaction",
        "Line of visibility",
        "Frontstage",
        "Backstage",
        "Greets",
        "Routes",
    ] {
        assert!(svg.contains(&format!(">{text}</text>")), "{text}");
    }
    let order = |needle: &str| svg.find(&format!(">{needle}</text>")).unwrap();
    assert!(order("Actions") < order("Line of interaction"));
    assert!(order("Line of interaction") < order("Greets"));
    assert!(order("Greets") < order("Line of visibility"));
    assert!(order("Line of visibility") < order("Routes"));
}

#[test]
fn a_narrow_journey_is_one_block_per_stage() {
    let output = render_ok(&JOURNEY.replace("700", "320").replace("420", "700"));
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-journey-stage\"").count(), 2);
    assert_eq!(svg.matches("class=\"chartlet-journey-step").count(), 10);
    assert!(svg.contains(">+2</text>"));
}

#[test]
fn the_journey_text_walks_through_the_stages() {
    let page = html(JOURNEY);
    assert!(page.contains("User journey of 2 stages: A: actions: Opens; sentiment: -1. B: actions: Reads; thoughts: Hmm; sentiment: +2."));
    assert!(page.contains("<th scope=\"col\">Stage</th><th scope=\"col\">Actions</th><th scope=\"col\">Thoughts</th><th scope=\"col\">Sentiment</th>"));
    assert!(page.contains("<th scope=\"row\">A</th><td>Opens</td><td></td><td>-1</td>"));
    assert!(html(&german(JOURNEY, "journey")).contains("Customer Journey aus 2 Phasen:"));
}

#[test]
fn journeys_are_validated_by_name() {
    for (spec, code, path) in [
        (
            JOURNEY.replace(
                "\"action\": \"Opens\"",
                "\"action\": \"Opens\", \"frontstage\": \"x\"",
            ),
            "option_not_supported",
            "/journey/stages/0/frontstage",
        ),
        (
            JOURNEY.replace("\"sentiment\": 2", "\"sentiment\": 3"),
            "invalid_value",
            "/journey/stages/1/sentiment",
        ),
        (
            JOURNEY.replace("\"label\": \"B\"", "\"label\": \"A\""),
            "duplicate_label",
            "/journey/stages/1/label",
        ),
    ] {
        let (found, found_path) = render_err(&spec);
        assert_eq!((found, found_path.as_str()), (code, path), "{spec}");
    }
    let (code, path) = render_err(
        r#"{"schemaVersion": 1, "type": "journey", "title": "x", "journey": {"stages": [{"label": "A"}]}}"#,
    );
    assert_eq!((code, path.as_str()), ("empty_data", "/journey/stages"));
}

const VALUEMAP: &str = r#"{"schemaVersion": 1, "type": "valuemap", "title": "Chain", "width": 700, "height": 460,
    "valuemap": {"components": [
        {"id": "u", "label": "User need", "visibility": 0.9, "evolution": 0.6},
        {"id": "s", "label": "Service", "visibility": 0.5, "evolution": 0.3, "movesTo": {"evolution": 0.6}, "inertia": true},
        {"id": "d", "label": "Data", "visibility": 0.2, "evolution": 0.9}
    ], "dependencies": [{"from": "u", "to": "s"}, {"from": "s", "to": "d"}]}}"#;

#[test]
fn a_value_chain_map_places_components_links_and_movement() {
    let output = render_ok(VALUEMAP);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-vmap-dot\"").count(), 3);
    assert_eq!(svg.matches("class=\"chartlet-vmap-link\"").count(), 2);
    assert_eq!(svg.matches("class=\"chartlet-vmap-ghost\"").count(), 1);
    assert_eq!(svg.matches("class=\"chartlet-vmap-move\"").count(), 1);
    assert_eq!(svg.matches("class=\"chartlet-vmap-inertia\"").count(), 1);
    for stage in ["Genesis", "Custom", "Product", "Commodity"] {
        assert!(svg.contains(&format!(">{stage}</text>")), "{stage}");
    }
    for label in ["User need", "Service", "Data"] {
        assert!(svg.contains(&format!(">{label}</text>")), "{label}");
    }
}

#[test]
fn the_dot_of_a_component_is_where_the_specification_puts_it() {
    let svg = render_ok(VALUEMAP).content;
    let dot = |label: &str| -> (f64, f64) {
        let at = svg.find(&format!("<title>{label}: visibility")).unwrap();
        let start = svg[..at].rfind("<circle").unwrap();
        let tag = &svg[start..at];
        let number = |name: &str| -> f64 {
            tag.split(&format!("{name}=\""))
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap()
                .parse()
                .unwrap()
        };
        (number("cx"), number("cy"))
    };
    let (user, service, data) = (dot("User need"), dot("Service"), dot("Data"));
    // More visible is higher up; more evolved is further right.
    assert!(user.1 < service.1 && service.1 < data.1);
    assert!(service.0 < user.0 && user.0 < data.0);
}

#[test]
fn the_value_chain_text_gives_place_stage_movement_and_needs() {
    let page = html(VALUEMAP);
    assert!(page.contains("Value chain map of 3 components, from the most visible to the user down, each with its visibility, its stage of evolution and where it moves: User need (visibility 90%, Product). Service (visibility 50%, Custom, moving to Product, with inertia). Data (visibility 20%, Commodity). Dependencies: User need needs Service; Service needs Data."));
    assert!(page.contains("<th scope=\"row\">Service</th><td>50%</td><td>30%</td><td>Custom</td><td>60%</td><td>yes</td><td>Data</td>"));
}

#[test]
fn value_chain_maps_are_validated_by_name() {
    for (spec, code, path) in [
        (
            VALUEMAP.replace("\"to\": \"d\"", "\"to\": \"x\""),
            "unknown_node",
            "/valuemap/dependencies/1/to",
        ),
        (
            VALUEMAP.replace("\"visibility\": 0.9", "\"visibility\": 1.5"),
            "invalid_position",
            "/valuemap/components/0/visibility",
        ),
        (
            VALUEMAP.replace("\"id\": \"s\"", "\"id\": \"u\""),
            "duplicate_id",
            "/valuemap/components/1/id",
        ),
        (
            VALUEMAP.replace(
                "{\"from\": \"u\", \"to\": \"s\"}",
                "{\"from\": \"u\", \"to\": \"u\"}",
            ),
            "self_loop",
            "/valuemap/dependencies/0",
        ),
        (
            VALUEMAP.replace(
                "\"movesTo\": {\"evolution\": 0.6}",
                "\"movesTo\": {\"evolution\": 0.2}",
            ),
            "invalid_position",
            "/valuemap/components/1/movesTo",
        ),
    ] {
        let (found, found_path) = render_err(&spec);
        assert_eq!((found, found_path.as_str()), (code, path), "{spec}");
    }
}
