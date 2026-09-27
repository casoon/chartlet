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
fn a_closed_stdout_ends_quietly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(["render", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("chartlet CLI should start");
    // Close the reading end before chartlet receives its input, so its write has to fail.
    drop(child.stdout.take());
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(include_bytes!("../examples/monthly-revenue.json"))
        .expect("specification should be written");
    let output = child.wait_with_output().expect("chartlet should finish");

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(output.status.success(), "unexpected stderr: {stderr}");
    assert!(!stderr.contains("panicked"), "unexpected stderr: {stderr}");
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

#[test]
fn cli_renders_a_time_chart_with_its_time_axis() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(["render", "examples/daily-orders.json", "--format", "svg"])
        .output()
        .expect("chartlet CLI should start");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("SVG output should be UTF-8");
    assert!(stdout.contains("<polyline"));
    assert!(stdout.contains(">2026-02-14<"));
    assert!(stdout.contains(">Day<"));
    assert!(
        output.stderr.is_empty(),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_carries_the_dark_theme_and_the_colors_a_layer_declares() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/revenue-vs-forecast.json",
            "--format",
            "svg",
        ])
        .output()
        .expect("chartlet CLI should start");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("SVG output should be UTF-8");
    assert!(stdout.contains("chartlet-theme-dark"));
    assert!(stdout.contains("--chartlet-background:#0e131c"));
    assert!(stdout.contains("stroke:var(--chart-forecast,currentColor)"));
}

#[test]
fn strict_mode_rejects_a_series_that_outnumbers_the_plot_pixels() {
    // A thousand hourly observations do not fit the 704 pixels of the default plot width.
    let points = (0..1_000)
        .map(|index| {
            let offset = i64::from(index) * 3_600;
            format!(
                "{{\"time\": {}, \"value\": {}}}",
                1_770_000_000 + offset,
                index % 5 + 1
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let specification = format!(
        r#"{{"schemaVersion":1,"type":"time","title":"Hourly load","timeAxis":{{"timezone":"UTC"}},
            "panes":[{{"valueAxis":{{"title":"Load"}},
            "layers":[{{"mark":"line","name":"Load","points":[{points}]}}]}}]}}"#
    );

    let mut child = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(["render", "-", "--format", "svg", "--strict"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("chartlet CLI should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(specification.as_bytes())
        .expect("specification should be written");
    let output = child.wait_with_output().expect("chartlet should finish");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("warnings should be UTF-8");
    assert!(stderr.contains("warning[dense_chart]"), "{stderr}");
    assert!(
        stderr.contains("exceed the 704 horizontal pixels"),
        "{stderr}"
    );
    assert!(stderr.contains("strict mode rejected"), "{stderr}");
}
