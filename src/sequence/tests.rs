use crate::{ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative};

const SPEC: &str = r#"{
    "schemaVersion": 1,
    "type": "sequence",
    "title": "Login",
    "sequence": {
        "participants": [
            {"id": "user", "label": "User", "kind": "actor"},
            {"id": "app", "label": "App"},
            {"id": "db", "label": "Users", "kind": "database"}
        ],
        "messages": [
            {"from": "user", "to": "app", "label": "sign in"},
            {"from": "app", "to": "db", "label": "find user"},
            {"from": "db", "to": "app", "label": "user", "kind": "reply"},
            {"from": "app", "to": "app", "label": "check password"},
            {"from": "app", "to": "user", "label": "welcome", "kind": "reply"}
        ],
        "fragments": [{"kind": "opt", "label": "known user", "from": 2, "to": 3}]
    }
}"#;

fn spec(json: &str) -> ChartSpec {
    ChartSpec::from_json(json).expect("the test specification parses")
}

fn svg(json: &str) -> crate::RenderOutput {
    render_json(json, RenderFormat::Svg, &RenderOptions::default())
        .expect("the test specification renders")
}

fn error(json: &str) -> (&'static str, String) {
    let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
        .expect_err("the test specification is invalid");
    (error.code, error.path)
}

#[test]
fn every_message_is_an_arrow_with_its_label_and_number() {
    let output = svg(SPEC);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-seq-message\"")
            .count(),
        3
    );
    assert_eq!(
        output
            .content
            .matches("chartlet-seq-message chartlet-seq-reply")
            .count(),
        2
    );
    assert_eq!(
        output
            .content
            .matches("class=\"chartlet-seq-head\"")
            .count(),
        3
    );
    assert!(
        output
            .content
            .contains("<title>2. App → Users: find user</title>")
    );
    assert!(output.content.contains(">opt</text>"));
    assert!(output.content.contains(">[known user]</text>"));
}

#[test]
fn a_call_activates_its_receiver_until_the_reply() {
    let sequence = spec(SPEC).sequence.expect("a sequence block");
    let activations = super::model::activations(&sequence, &sequence.ends());
    let spans: Vec<(usize, usize, usize)> = activations
        .iter()
        .map(|activation| (activation.participant, activation.from, activation.to))
        .collect();
    // The app is busy from the sign-in to its reply, the store from the lookup to its reply.
    assert_eq!(spans, vec![(1, 0, 4), (2, 1, 2)]);
}

#[test]
fn an_activation_without_reply_ends_with_the_last_message_of_its_participant() {
    let json = SPEC.replace(
        r#""label": "user", "kind": "reply""#,
        r#""label": "user", "kind": "async""#,
    );
    let sequence = spec(&json).sequence.expect("a sequence block");
    let activations = super::model::activations(&sequence, &sequence.ends());
    assert_eq!((activations[1].from, activations[1].to), (1, 2));
}

#[test]
fn auto_orientation_turns_landscape_when_only_that_fits() {
    let messages: Vec<String> = (0..30)
        .map(|index| format!(r#"{{"from": "a", "to": "b", "label": "m{index}"}}"#))
        .collect();
    let json = format!(
        r#"{{"schemaVersion": 1, "type": "sequence", "title": "Many", "width": 2400, "height": 300,
        "sequence": {{"participants": [{{"id": "a", "label": "A"}}, {{"id": "b", "label": "B"}}],
        "messages": [{}]}}}}"#,
        messages.join(",")
    );
    let output = svg(&json);
    // Thirty messages are over the readability budget on purpose; nothing else is reported.
    assert!(
        output
            .warnings
            .iter()
            .all(|warning| warning.code == "diagram_over_budget"),
        "{:?}",
        output.warnings
    );
    assert!(
        output
            .content
            .starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"2400\" height=\"300\"")
    );
    // In landscape the lifelines run across: the first one starts and ends at the same height.
    let lifeline = output
        .content
        .split("<line ")
        .skip(1)
        .find(|line| line.contains("chartlet-seq-lifeline"))
        .expect("a lifeline");
    let y = |name: &str| {
        lifeline
            .split(&format!("{name}=\""))
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("a coordinate")
            .to_owned()
    };
    assert_eq!(y("y1"), y("y2"));

    let portrait = json.replace(r#""messages""#, r#""orientation": "portrait", "messages""#);
    let output = svg(&portrait);
    let too_small = output
        .warnings
        .iter()
        .find(|warning| warning.code == "canvas_too_small")
        .expect("the portrait canvas is too small");
    assert_eq!(too_small.path, "/height");
}

#[test]
fn the_text_alternative_lists_every_message_in_order() {
    let alternative = text_alternative(&spec(SPEC)).expect("valid");
    assert_eq!(
        alternative.description,
        "Sequence diagram with 3 participants: User (actor), App and Users (database). 5 messages, in order: 1. User to App: sign in. 2. App to Users: find user. 3. Users to App: user (reply). 4. App to itself: check password. 5. App to User: welcome (reply). Fragment opt \"known user\" spans messages 3–4."
    );
    assert_eq!(
        alternative.table.columns,
        ["No.", "From", "To", "Message", "Kind", "Fragment"]
    );
    assert_eq!(
        alternative.table.rows[2],
        ["3", "Users", "App", "user", "reply", "opt [known user]"]
    );
    assert_eq!(alternative.table.rows[0][5], "");
}

#[test]
fn the_html_table_carries_text_hooks_without_values() {
    let output = render_json(
        SPEC,
        RenderFormat::Html,
        &RenderOptions {
            hooks: true,
            ..RenderOptions::default()
        },
    )
    .expect("renders");
    assert!(output.content.contains(
        "<th scope=\"col\" data-series=\"0\" data-pane=\"0\" data-part=\"text\">From</th>"
    ));
    assert!(!output.content.contains("data-value"));
}

#[test]
fn invalid_sequences_name_the_field() {
    for (from, to, code, path) in [
        (
            r#""to": "db""#,
            r#""to": "nobody""#,
            "unknown_participant",
            "/sequence/messages/1/to",
        ),
        (
            r#""id": "db""#,
            r#""id": "app""#,
            "duplicate_id",
            "/sequence/participants/2/id",
        ),
        (
            r#""id": "db""#,
            r#""id": "2db""#,
            "invalid_id",
            "/sequence/participants/2/id",
        ),
        (
            r#""from": 2, "to": 3"#,
            r#""from": 3, "to": 2"#,
            "invalid_fragment",
            "/sequence/fragments/0/from",
        ),
        (
            r#""from": 2, "to": 3"#,
            r#""from": 2, "to": 5"#,
            "invalid_fragment",
            "/sequence/fragments/0/to",
        ),
        (
            r#""from": 2, "to": 3}"#,
            r#""from": 2, "to": 3, "else": [{"from": 3}]}"#,
            "option_not_supported",
            "/sequence/fragments/0/else",
        ),
        (
            r#""from": 2, "to": 3}]"#,
            r#""from": 2, "to": 3}, {"kind": "loop", "from": 3, "to": 4}]"#,
            "fragments_cross",
            "/sequence/fragments/1/from",
        ),
        (
            r#""type": "sequence""#,
            r#""type": "bar""#,
            "option_not_supported",
            "/sequence",
        ),
    ] {
        assert!(SPEC.contains(from), "{from}");
        assert_eq!(
            error(&SPEC.replace(from, to)),
            (code, path.to_owned()),
            "{to}"
        );
    }
    let missing = r#"{"schemaVersion": 1, "type": "sequence", "title": "Empty"}"#;
    assert_eq!(error(missing), ("missing_sequence", "/sequence".to_owned()));
}

#[test]
fn nested_fragments_and_branches_are_accepted() {
    let json = SPEC.replace(
        r#"[{"kind": "opt", "label": "known user", "from": 2, "to": 3}]"#,
        r#"[{"kind": "alt", "label": "found", "from": 1, "to": 4, "else": [{"from": 3, "label": "unknown"}]},
            {"kind": "loop", "from": 1, "to": 2}]"#,
    );
    let sequence = spec(&json).sequence.expect("a sequence block");
    assert_eq!((sequence.depth(0), sequence.depth(1)), (0, 1));
    let output = svg(&json);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert!(output.content.contains(">[unknown]</text>"));
}

#[test]
fn a_name_that_fills_its_box_exactly_is_not_shortened() {
    // 56.29 pixels of text plus the inset, less the inset again, is a hair under the text.
    let json = SPEC.replace(
        r#""label": "Users", "kind": "database""#,
        r#""label": "Keycloak", "kind": "database""#,
    );
    let output = svg(&json);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert!(output.content.contains(">Keycloak</text>"));
}

#[test]
fn a_narrow_diagram_lets_message_labels_reach_over_lifelines() {
    let json = SPEC.replace(
        "\"title\": \"Login\",",
        "\"title\": \"Login\", \"width\": 400, \"height\": 600,",
    );
    let output = svg(&json);
    assert!(
        output
            .warnings
            .iter()
            .all(|warning| warning.code != "text_truncated"),
        "{:?}",
        output.warnings
    );
}

#[test]
fn a_narrow_diagram_wraps_participant_names_instead_of_shortening_them() {
    let json = SPEC
        .replace(
            "\"title\": \"Login\",",
            "\"title\": \"Login\", \"width\": 320, \"height\": 600,",
        )
        .replace(
            r#""label": "Users", "kind": "database""#,
            r#""label": "Registered users", "kind": "database""#,
        );
    let output = svg(&json);
    assert!(
        output
            .warnings
            .iter()
            .all(|warning| !warning.path.starts_with("/sequence/participants")),
        "{:?}",
        output.warnings
    );
    assert!(output.content.contains(">Registered</text>"));
    assert!(output.content.contains(">users</text>"));
}

#[test]
fn rendering_twice_gives_the_same_bytes() {
    assert_eq!(svg(SPEC).content, svg(SPEC).content);
}
