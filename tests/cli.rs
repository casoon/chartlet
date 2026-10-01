use std::{
    fmt::Write as _,
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

/// A bar chart whose first label is one word wider than its band, so that it is shortened.
const TRUNCATED: &str = r#"{"schemaVersion": 1, "type": "bar", "title": "Quarterly change",
    "data": [
        {"label": "Antidisestablishmentarianism-Antidisestablishmentarianism", "value": 1},
        {"label": "B", "value": 2}, {"label": "C", "value": 3}, {"label": "D", "value": 4}
    ]}"#;

/// Runs the CLI with `args`, passing `input` on standard input.
fn run_with_stdin(args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("chartlet CLI should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("stdin should accept the specification");
    child.wait_with_output().expect("chartlet should finish")
}

#[test]
fn strict_mode_rejects_layout_warnings() {
    let output = run_with_stdin(&["render", "-", "--format", "svg", "--strict"], TRUNCATED);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("warnings should be UTF-8");
    assert!(stderr.contains("warning[text_truncated]"));
    assert!(stderr.contains("strict mode rejected"));
}

#[test]
fn strict_mode_lets_allowed_warning_codes_through() {
    let output = run_with_stdin(
        &[
            "render",
            "-",
            "--format",
            "svg",
            "--strict",
            "--allow-warning",
            "text_truncated",
        ],
        TRUNCATED,
    );

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("warnings should be UTF-8");
    assert!(stderr.contains("warning[text_truncated]"));
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

#[test]
fn json_diagnostics_report_warnings_and_strict_failure() {
    let output = run_with_stdin(
        &["render", "-", "--strict", "--diagnostics", "json"],
        TRUNCATED,
    );

    assert!(!output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should be one JSON document");
    assert_eq!(report["ok"], false);
    assert_eq!(report["error"]["code"], "strict_warnings");
    assert_eq!(report["warnings"][0]["code"], "text_truncated");
    assert!(report["warnings"][0]["path"].is_string());
}

#[test]
fn json_diagnostics_report_a_specification_error_with_its_path() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(["render", "-", "--diagnostics=json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("chartlet CLI should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(br#"{"schemaVersion": 1, "type": "bar", "title": "T", "data": []}"#)
        .expect("stdin should accept the specification");
    let output = child.wait_with_output().expect("chartlet should finish");

    assert!(!output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("stderr should be one JSON document");
    assert_eq!(report["ok"], false);
    assert!(report["error"]["code"].is_string());
    assert!(report["error"]["path"].as_str().unwrap().starts_with('/'));
    assert_eq!(report["warnings"], serde_json::json!([]));
}

#[test]
fn help_after_the_subcommand_prints_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(["render", "--help"])
        .output()
        .expect("chartlet CLI should start");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("usage:"));
}

#[test]
fn cli_renders_the_mobile_variant_alone() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/mobile-revenue.json",
            "--variant",
            "mobile",
        ])
        .output()
        .expect("chartlet CLI should start");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("SVG output should be UTF-8");
    assert!(stdout.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"360\""));
    assert!(stdout.contains("-m-title"));

    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/monthly-revenue.json",
            "--variant",
            "mobile",
            "--diagnostics",
            "json",
        ])
        .output()
        .expect("chartlet CLI should start");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");
    assert!(stderr.contains("\"code\":\"missing_mobile\""), "{stderr}");
    assert!(stderr.contains("\"path\":\"/mobile\""), "{stderr}");
}

#[test]
fn cli_renders_the_print_variant() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/temperature-projection.json",
            "--variant",
            "print",
        ])
        .output()
        .expect("chartlet CLI should start");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("SVG output should be UTF-8");
    assert_eq!(
        stdout.trim_end(),
        include_str!("../examples/temperature-projection.print.svg")
    );
}

#[test]
fn cli_renders_the_social_variant() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/temperature-projection.json",
            "--variant",
            "social",
        ])
        .output()
        .expect("chartlet CLI should start");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("SVG output should be UTF-8");
    assert_eq!(
        stdout.trim_end(),
        include_str!("../examples/temperature-projection.social.svg")
    );
}

#[test]
fn cli_rejects_the_social_variant_as_html() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/temperature-projection.json",
            "--variant",
            "social",
            "--format",
            "html",
        ])
        .output()
        .expect("chartlet CLI should start");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");
    assert!(stderr.contains("option_not_supported"), "{stderr}");
}

#[cfg(feature = "png")]
#[test]
fn cli_renders_a_png() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args([
            "render",
            "examples/temperature-projection.json",
            "--variant",
            "social",
            "--format",
            "png",
        ])
        .output()
        .expect("chartlet CLI should start");

    assert!(output.status.success());
    assert_eq!(
        chartlet::sha256(&output.stdout),
        chartlet::sha256(
            &chartlet::render_png(
                &chartlet::ChartSpec::from_json(include_str!(
                    "../examples/temperature-projection.json"
                ))
                .unwrap(),
                &chartlet::PngOptions {
                    variant: chartlet::Variant::Social,
                    scale: 1.0,
                },
            )
            .unwrap()
            .png
        )
    );
}

#[cfg(not(feature = "png"))]
#[test]
fn cli_without_the_png_feature_says_how_to_get_it() {
    let output = Command::new(env!("CARGO_BIN_EXE_chartlet"))
        .args(["render", "examples/monthly-revenue.json", "--format", "png"])
        .output()
        .expect("chartlet CLI should start");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");
    assert!(stderr.contains("--features png"), "{stderr}");
}

#[test]
fn manifest_records_the_provenance_of_the_render() {
    let directory = std::env::temp_dir().join(format!("chartlet-cli-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("temporary directory should be writable");
    let (svg, manifest) = (directory.join("chart.svg"), directory.join("manifest.json"));
    let render = |input: &str| {
        let output = run_with_stdin(
            &[
                "render",
                "-",
                "--id-prefix",
                "change",
                "-o",
                svg.to_str().expect("UTF-8 path"),
                "--manifest",
                manifest.to_str().expect("UTF-8 path"),
            ],
            input,
        );
        assert!(output.status.success(), "{output:?}");
        let content = std::fs::read(&svg).expect("the SVG should be written");
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&manifest).expect("the manifest should be written"),
        )
        .expect("the manifest should be JSON");
        (content, manifest)
    };

    let (content, first) = render(TRUNCATED);
    assert_eq!(first["chartlet"], env!("CARGO_PKG_VERSION"));
    assert_eq!(first["schemaVersion"], 1);
    assert_eq!(first["format"], "svg");
    assert_eq!(first["variant"], "desktop");
    assert_eq!(first["idPrefix"], "change");
    assert_eq!(first["warnings"][0]["code"], "text_truncated");
    assert_eq!(first["warnings"][0]["path"], "/data/0/label");
    let hash = chartlet::sha256(&content)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").expect("writing to String cannot fail");
            hex
        });
    assert_eq!(first["outputHash"], format!("sha256:{hash}"));
    assert!(
        first["specHash"]
            .as_str()
            .is_some_and(|hash| hash.starts_with("sha256:") && hash.len() == 71)
    );
    assert!(first.get("timestamp").is_none());

    // The same specification with other whitespace and key order hashes the same.
    let reordered: serde_json::Value = serde_json::from_str(TRUNCATED).expect("valid JSON");
    let (_, second) = render(&serde_json::to_string_pretty(&reordered).expect("serializes"));
    assert_eq!(first, second);

    std::fs::remove_dir_all(&directory).expect("temporary directory should be removable");
}
