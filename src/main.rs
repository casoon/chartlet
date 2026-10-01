use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process,
};

use chartlet::{ChartWarning, RenderFormat, RenderOptions, TableMode, Variant, render_json};
use serde_json::{Value, json};

/// A failure of the CLI: a code for machine-readable diagnostics, the location in the
/// specification where there is one, and the message.
struct Failure {
    code: &'static str,
    path: Option<String>,
    message: String,
}

impl From<String> for Failure {
    fn from(message: String) -> Self {
        Self {
            code: "cli_error",
            path: None,
            message,
        }
    }
}

fn main() {
    let json_diagnostics = env::args().any(|argument| argument == "--diagnostics=json")
        || env::args()
            .collect::<Vec<_>>()
            .windows(2)
            .any(|pair| pair[0] == "--diagnostics" && pair[1] == "json");
    let mut warnings = Vec::new();
    let result = run(&mut warnings, json_diagnostics);
    if json_diagnostics {
        let error = result.as_ref().err().map(|failure| {
            json!({ "code": failure.code, "path": failure.path, "message": failure.message })
        });
        let report = json!({
            "ok": result.is_ok(),
            "error": error,
            "warnings": warnings.iter().map(warning_json).collect::<Vec<Value>>(),
        });
        eprintln!("{report}");
    } else if let Err(failure) = &result {
        match &failure.path {
            Some(path) => eprintln!("chartlet: {} at {path}: {}", failure.code, failure.message),
            None => eprintln!("chartlet: {}", failure.message),
        }
    }
    if result.is_err() {
        process::exit(1);
    }
}

fn warning_json(warning: &ChartWarning) -> Value {
    json!({ "code": warning.code, "path": warning.path, "message": warning.message })
}

fn run(warnings: &mut Vec<ChartWarning>, json_diagnostics: bool) -> Result<(), Failure> {
    let mut arguments = env::args().skip(1);
    match arguments.next().as_deref() {
        Some("render") => {}
        Some("-h" | "--help") => return Ok(write_stdout(&usage())?),
        _ => return Err(usage().into()),
    }
    let input = match arguments.next() {
        Some(help) if help == "-h" || help == "--help" => return Ok(write_stdout(&usage())?),
        Some(input) => PathBuf::from(input),
        None => return Err(usage().into()),
    };
    let mut format = RenderFormat::Svg;
    let mut output = None;
    let mut id_prefix = None;
    let mut strict = false;
    let mut table_mode = TableMode::Details;
    let mut variant = Variant::Desktop;
    let mut manifest = None;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--format" => {
                format = match arguments.next().as_deref() {
                    Some("svg") => RenderFormat::Svg,
                    Some("html") => RenderFormat::Html,
                    _ => return Err("--format must be svg or html".to_owned().into()),
                };
            }
            "-o" | "--output" => output = Some(path_after(&mut arguments, "--output")?),
            "--id-prefix" => {
                id_prefix = Some(
                    arguments
                        .next()
                        .ok_or_else(|| "--id-prefix requires a value".to_owned())?,
                );
            }
            "--strict" => strict = true,
            "--table" => {
                table_mode = match arguments.next().as_deref() {
                    Some("details") => TableMode::Details,
                    Some("visible") => TableMode::Visible,
                    _ => return Err("--table must be details or visible".to_owned().into()),
                };
            }
            "--variant" => variant = parse_variant(arguments.next().as_deref())?,
            "--manifest" => manifest = Some(path_after(&mut arguments, "--manifest")?),
            "--diagnostics" => match arguments.next().as_deref() {
                Some("json" | "text") => {}
                _ => return Err("--diagnostics must be text or json".to_owned().into()),
            },
            "--diagnostics=json" | "--diagnostics=text" => {}
            "-h" | "--help" => return Ok(write_stdout(&usage())?),
            unknown => return Err(format!("unknown argument {unknown:?}\n\n{}", usage()).into()),
        }
    }

    let input_content = read_input(&input)?;
    let rendered = render_json(
        &input_content,
        format,
        &RenderOptions {
            id_prefix,
            table_mode,
            variant,
            manifest: manifest.is_some(),
        },
    )
    .map_err(|error| Failure {
        code: error.code,
        path: Some(error.path),
        message: error.message,
    })?;

    if !json_diagnostics {
        for warning in &rendered.warnings {
            eprintln!(
                "warning[{}] at {}: {}",
                warning.code, warning.path, warning.message
            );
        }
    }
    let warning_count = rendered.warnings.len();
    warnings.extend(rendered.warnings);
    if strict && warning_count > 0 {
        return Err(Failure {
            code: "strict_warnings",
            path: None,
            message: format!("strict mode rejected {warning_count} warning(s)"),
        });
    }

    if let Some(output) = output {
        fs::write(&output, rendered.content)
            .map_err(|error| format!("could not write {}: {error}", output.display()))?;
    } else {
        write_stdout(&rendered.content)?;
    }
    if let (Some(path), Some(manifest)) = (manifest, rendered.manifest) {
        fs::write(&path, manifest.to_json() + "\n")
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
    }
    Ok(())
}

/// The path that follows `option` on the command line.
fn path_after(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<PathBuf, String> {
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{option} requires a path"))
}

fn parse_variant(value: Option<&str>) -> Result<Variant, String> {
    match value {
        Some("desktop") => Ok(Variant::Desktop),
        Some("mobile") => Ok(Variant::Mobile),
        Some("print") => Ok(Variant::Print),
        _ => Err("--variant must be desktop, mobile or print".to_owned()),
    }
}

fn usage() -> String {
    "usage: chartlet render <spec.json|-> [--format svg|html] [-o <path>] [--id-prefix <prefix>] [--table details|visible] [--variant desktop|mobile|print] [--manifest <path>] [--strict] [--diagnostics text|json]".to_owned()
}

/// Writes to stdout. A reader that stops early, such as `chartlet render … | head`, closes the
/// pipe; that ends the program quietly instead of being reported as a failure.
fn write_stdout(content: &str) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    match writeln!(stdout, "{content}").and_then(|()| stdout.flush()) {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(|error| format!("could not write to stdout: {error}")),
    }
}

fn read_input(input: &Path) -> Result<String, String> {
    if input == Path::new("-") {
        let mut content = String::new();
        io::stdin()
            .read_to_string(&mut content)
            .map_err(|error| format!("could not read stdin: {error}"))?;
        Ok(content)
    } else {
        fs::read_to_string(input)
            .map_err(|error| format!("could not read {}: {error}", input.display()))
    }
}
