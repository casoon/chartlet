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

/// A `chartlet render --watch` process whose output lines arrive on channels; it is killed when
/// dropped, as Ctrl-C would stop it.
struct Watcher {
    child: std::process::Child,
    stdout: std::sync::mpsc::Receiver<String>,
    stderr: std::sync::mpsc::Receiver<String>,
}

impl Watcher {
    fn start(args: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_chartlet"))
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("chartlet CLI should start");
        let lines = |stream: Box<dyn std::io::Read + Send>| {
            let (sender, receiver) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                for line in std::io::BufRead::lines(std::io::BufReader::new(stream)) {
                    let Ok(line) = line else { break };
                    if sender.send(line).is_err() {
                        break;
                    }
                }
            });
            receiver
        };
        let stdout = lines(Box::new(
            child.stdout.take().expect("stdout should be piped"),
        ));
        let stderr = lines(Box::new(
            child.stderr.take().expect("stderr should be piped"),
        ));
        Self {
            child,
            stdout,
            stderr,
        }
    }

    /// The next line on stderr, within ten seconds.
    fn next_line(&self) -> String {
        self.stderr
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("chartlet should report within ten seconds")
    }

    /// Skips stderr lines up to the first that contains `needle`, and returns it.
    fn wait_for(&self, needle: &str) -> String {
        loop {
            let line = self.next_line();
            if line.contains(needle) {
                return line;
            }
        }
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// An empty temporary directory for one test.
fn temporary_directory(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!("chartlet-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("temporary directory should be writable");
    directory
}

fn path_text(path: &std::path::Path) -> &str {
    path.to_str().expect("UTF-8 path")
}

#[test]
fn watch_renders_every_change_and_keeps_the_last_output_on_error() {
    let directory = temporary_directory("watch");
    let (spec, svg) = (directory.join("spec.json"), directory.join("chart.svg"));
    let single = directory.join("single.svg");
    // What a single `chartlet render` writes for `specification`.
    let render_once = |specification: &str| {
        let output = run_with_stdin(&["render", "-", "-o", path_text(&single)], specification);
        assert!(output.status.success(), "{output:?}");
        std::fs::read(&single).expect("the single render")
    };
    let first = include_str!("../examples/monthly-revenue.json");
    let second = include_str!("../examples/quarterly-change.json");
    std::fs::write(&spec, first).expect("the specification should be written");
    let watcher = Watcher::start(&["render", path_text(&spec), "-o", path_text(&svg), "--watch"]);

    watcher.wait_for("chartlet: wrote");
    assert_eq!(std::fs::read(&svg).expect("the SVG"), render_once(first));
    // Writing the output does not count as a change of the specification.
    assert!(
        watcher
            .stderr
            .recv_timeout(std::time::Duration::from_secs(1))
            .is_err()
    );

    std::fs::write(&spec, "{").expect("the specification should be written");
    let error = watcher.next_line();
    assert!(error.starts_with("chartlet: invalid_json at /"), "{error}");
    assert_eq!(std::fs::read(&svg).expect("the SVG"), render_once(first));

    // Editors save by writing a new file and renaming it over the old one.
    let replacement = directory.join("spec.json.new");
    std::fs::write(&replacement, second).expect("the specification should be written");
    std::fs::rename(&replacement, &spec).expect("the specification should be replaced");
    watcher.wait_for("chartlet: wrote");
    assert_eq!(std::fs::read(&svg).expect("the SVG"), render_once(second));

    drop(watcher);
    std::fs::remove_dir_all(&directory).expect("temporary directory should be removable");
}

#[test]
fn watch_refuses_standard_input_and_a_missing_output() {
    let output = run_with_stdin(&["render", "-", "--watch", "-o", "chart.svg"], "{}");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot watch standard input"));

    let output = run_with_stdin(&["render", "examples/monthly-revenue.json", "--watch"], "");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--watch needs -o <path>"));
}

/// Sends a GET request for `target` to the preview at `port` with the given `Host` header.
fn get(port: u16, target: &str, host: &str) -> String {
    let mut stream =
        std::net::TcpStream::connect(("127.0.0.1", port)).expect("the preview should listen");
    write!(stream, "GET {target} HTTP/1.1\r\nHost: {host}\r\n\r\n")
        .expect("the request should be sent");
    let mut response = String::new();
    std::io::Read::read_to_string(&mut stream, &mut response).expect("a response");
    response
}

#[test]
fn preview_shows_the_chart_and_marks_it_stale_after_an_error() {
    let directory = temporary_directory("preview");
    let spec = directory.join("spec.json");
    std::fs::write(&spec, include_str!("../examples/monthly-revenue.json"))
        .expect("the specification should be written");
    let watcher = Watcher::start(&[
        "render",
        path_text(&spec),
        "--preview",
        "--diagnostics",
        "json",
    ]);
    let address = watcher
        .stdout
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("the preview address");
    let port: u16 = address
        .strip_prefix("chartlet: preview at http://127.0.0.1:")
        .and_then(|rest| rest.strip_suffix('/'))
        .and_then(|port| port.parse().ok())
        .unwrap_or_else(|| panic!("unexpected address line {address}"));
    let report: serde_json::Value =
        serde_json::from_str(&watcher.next_line()).expect("one JSON report per render");
    assert_eq!(report["ok"], true);
    let host = format!("127.0.0.1:{port}");

    let page = get(port, "/", &host);
    assert!(page.starts_with("HTTP/1.1 200 OK"), "{page}");
    assert!(page.contains("Rendered, 0 warning(s)."));
    assert!(page.contains("<main><svg"));
    assert!(get(port, "/version", &host).ends_with("\r\n\r\n1"));

    std::fs::write(&spec, "{").expect("the specification should be written");
    let report: serde_json::Value =
        serde_json::from_str(&watcher.next_line()).expect("one JSON report per render");
    assert_eq!(report["error"]["code"], "invalid_json");
    let page = get(port, "/", &host);
    assert!(page.contains("invalid_json at /"));
    assert!(page.contains("Stale:"));
    assert!(page.contains("<main class=\"stale\"><svg"));

    assert!(get(port, "/", "attacker.example").starts_with("HTTP/1.1 403"));
    assert!(get(port, "/spec.json", &host).starts_with("HTTP/1.1 404"));

    drop(watcher);
    std::fs::remove_dir_all(&directory).expect("temporary directory should be removable");
}
