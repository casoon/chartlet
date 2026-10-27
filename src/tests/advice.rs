use crate::{RemedyKind, RenderFormat, RenderOptions, advise_json, render_json};

const NARROW: &str = r#"{
  "schemaVersion": 1, "type": "flow", "title": "Narrow", "width": 320, "height": 240,
  "flow": {
    "nodes": [
      { "id": "a", "label": "Order received", "kind": "start" },
      { "id": "b", "label": "Payment cleared?", "kind": "decision" },
      { "id": "c", "label": "Ship parcel" }
    ],
    "edges": [ { "from": "a", "to": "b" }, { "from": "b", "to": "c", "label": "yes" } ]
  }
}"#;

#[test]
fn a_narrow_canvas_gets_a_checked_patch_and_the_field() {
    let help = advise_json(NARROW, &RenderOptions::default()).expect("renders");
    let width = help
        .iter()
        .find(|help| help.code == "canvas_too_small" && help.path == "/width")
        .expect("the canvas is too narrow");
    assert_eq!(width.field, "/width");
    let remedy = &width.remedies[0];
    assert_eq!(remedy.kind, RemedyKind::Layout);
    assert!(remedy.validated && remedy.resolves);
    assert_eq!(remedy.patch[0].path, "/width");
    assert_eq!(remedy.patch[0].op, "replace");
    // The proposal is real: applying it removes the warning.
    let fixed = NARROW.replace(
        "\"width\": 320",
        &format!("\"width\": {}", remedy.patch[0].value),
    );
    let output =
        render_json(&fixed, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
    assert!(
        output
            .warnings
            .iter()
            .all(|warning| !(warning.code == "canvas_too_small" && warning.path == "/width"))
    );
}

#[test]
fn the_specification_is_never_changed_and_help_is_stable() {
    let before = NARROW.to_owned();
    let first = advise_json(&before, &RenderOptions::default()).expect("renders");
    let second = advise_json(&before, &RenderOptions::default()).expect("renders");
    assert_eq!(before, NARROW);
    assert_eq!(first, second);
}

#[test]
fn a_truncated_title_never_gets_its_text_rewritten() {
    let spec = r#"{ "schemaVersion": 1, "type": "bar", "width": 400, "height": 300,
        "title": "A very long title that cannot possibly fit on one line of a narrow bar chart or on two",
        "data": [ { "label": "a", "value": 1 } ] }"#;
    let help = advise_json(spec, &RenderOptions::default()).expect("renders");
    let truncated = help
        .iter()
        .find(|help| help.code == "text_truncated")
        .expect("the title is shortened");
    assert_eq!(truncated.field, "/title");
    let content = truncated
        .remedies
        .iter()
        .find(|remedy| remedy.kind == RemedyKind::Content)
        .expect("guidance for the author");
    assert!(content.patch.is_empty() && !content.resolves && !content.validated);
    for remedy in truncated
        .remedies
        .iter()
        .filter(|remedy| remedy.kind == RemedyKind::Layout)
    {
        assert!(remedy.validated && remedy.resolves);
        assert_eq!(remedy.patch[0].path, "/width");
    }
}

#[test]
fn a_specification_without_warnings_has_no_help() {
    let spec = r#"{ "schemaVersion": 1, "type": "bar", "title": "Ok",
        "data": [ { "label": "a", "value": 1 } ] }"#;
    let help = advise_json(spec, &RenderOptions::default()).expect("renders");
    assert_eq!(help.len(), 0);
}

#[test]
fn help_serializes_with_every_remedy_field() {
    let help = advise_json(NARROW, &RenderOptions::default()).expect("renders");
    let json = help[0].to_json();
    assert_eq!(json["code"], "canvas_too_small");
    let remedy = &json["remedies"][0];
    for key in ["id", "kind", "summary", "validated", "resolves", "patch"] {
        assert!(remedy.get(key).is_some(), "{key}");
    }
}
