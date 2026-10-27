//! `--advice`: structured help next to the diagnostics of the CLI, only when asked for.

use std::{
    io::Write,
    process::{Command, Stdio},
};

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

fn stderr_of(args: &[&str]) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("chartlet CLI should start");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(NARROW.as_bytes())
        .expect("the specification is written");
    let output = child.wait_with_output().expect("chartlet finishes");
    String::from_utf8(output.stderr).expect("diagnostics are UTF-8")
}

#[test]
fn json_diagnostics_carry_help_with_a_checked_patch_when_asked() {
    let stderr = stderr_of(&["render", "-", "--diagnostics", "json", "--advice"]);
    let report: serde_json::Value = serde_json::from_str(stderr.trim()).expect("one JSON document");
    // The existing keys are unchanged.
    assert_eq!(report["ok"], true);
    assert_eq!(report["warnings"][0]["code"], "canvas_too_small");
    let help = &report["help"][0];
    assert_eq!(help["code"], "canvas_too_small");
    assert_eq!(help["field"], "/width");
    let remedy = &help["remedies"][0];
    assert_eq!(remedy["validated"], true);
    assert_eq!(remedy["resolves"], true);
    assert_eq!(remedy["patch"][0]["path"], "/width");
}

#[test]
fn help_is_absent_unless_asked_for() {
    let stderr = stderr_of(&["render", "-", "--diagnostics", "json"]);
    let report: serde_json::Value = serde_json::from_str(stderr.trim()).expect("one JSON document");
    assert!(report.get("help").is_none());
    assert!(!stderr_of(&["render", "-"]).contains("help["));
}

#[test]
fn text_diagnostics_show_the_help_lines() {
    let stderr = stderr_of(&["render", "-", "--advice"]);
    assert!(stderr.contains("warning[canvas_too_small]"));
    assert!(stderr.contains("help[canvas_too_small] at /width"));
    assert!(stderr.contains("checked: resolves it"));
}
