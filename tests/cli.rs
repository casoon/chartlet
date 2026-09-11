use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn cli_renders_svg_to_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(["render", "examples/monthly-revenue.json", "--format", "svg"])
        .output()
        .expect("chartlet CLI should start");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("SVG output should be UTF-8");
    assert!(stdout.starts_with("<svg"));
    assert!(stdout.contains("Monthly revenue"));
}

#[test]
fn strict_mode_rejects_layout_warnings() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/quarterly-change.json",
            "--format",
            "svg",
            "--strict",
        ])
        .output()
        .expect("chartlet CLI should start");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("warnings should be UTF-8");
    assert!(stderr.contains("warning[text_truncated]"));
    assert!(stderr.contains("strict mode rejected"));
}

#[test]
fn help_exits_successfully() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .arg("--help")
        .output()
        .expect("chartlet CLI should start");

    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .expect("help should be UTF-8")
            .starts_with("usage: chartlet")
    );
}

#[test]
fn cli_accepts_a_specification_on_stdin() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(["render", "-", "--id-prefix", "stdin-test"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("chartlet CLI should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(include_bytes!("../examples/monthly-trend.json"))
        .expect("specification should be written");
    let output = child.wait_with_output().expect("chartlet should finish");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("SVG output should be UTF-8");
    assert!(stdout.contains("stdin-test-title"));
    assert!(stdout.contains("class=\"chartlet-line\""));
}
