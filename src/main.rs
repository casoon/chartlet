use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process,
};

use chartlet::{
    ChartType, ChartWarning, RenderFormat, RenderOptions, Styles, TableMode, Variant, render_json,
    stylesheet, stylesheet_common, stylesheet_for, stylesheet_types,
};
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

#[allow(
    clippy::too_many_lines,
    reason = "one match arm per command-line option"
)]
fn run(warnings: &mut Vec<ChartWarning>, json_diagnostics: bool) -> Result<(), Failure> {
    let mut arguments = env::args().skip(1);
    match arguments.next().as_deref() {
        Some("render") => {}
        Some("stylesheet") => return Ok(write_stdout(&shared_stylesheet(arguments)?)?),
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
    let mut allowed: Vec<String> = Vec::new();
    let mut table_mode = TableMode::Details;
    let mut variant = Variant::Desktop;
    let mut manifest = None;
    let mut styles = Styles::Inline;
    let mut hooks = false;
    let mut png = false;
    let mut scale = None;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--format" => (format, png) = parse_format(arguments.next().as_deref())?,
            "--scale" => scale = Some(parse_scale(arguments.next().as_deref())?),
            "-o" | "--output" => output = Some(path_after(&mut arguments, "--output")?),
            "--id-prefix" => {
                id_prefix = Some(
                    arguments
                        .next()
                        .ok_or_else(|| "--id-prefix requires a value".to_owned())?,
                );
            }
            "--strict" => strict = true,
            "--allow-warning" => allowed.push(
                arguments
                    .next()
                    .ok_or_else(|| "--allow-warning requires a warning code".to_owned())?,
            ),
            "--table" => {
                table_mode = match arguments.next().as_deref() {
                    Some("details") => TableMode::Details,
                    Some("visible") => TableMode::Visible,
                    _ => return Err("--table must be details or visible".to_owned().into()),
                };
            }
            "--variant" => variant = parse_variant(arguments.next().as_deref())?,
            "--styles" => styles = parse_styles(arguments.next().as_deref())?,
            "--hooks" => hooks = true,
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
    let (content, rendered_warnings, rendered_manifest) = if png {
        let (png, warnings) = render_png(&input_content, variant, scale, manifest.is_some())?;
        (Content::Png(png), warnings, None)
    } else if scale.is_some() {
        return Err("--scale requires --format png".to_owned().into());
    } else {
        let rendered = render_json(
            &input_content,
            format,
            &RenderOptions {
                id_prefix,
                table_mode,
                variant,
                manifest: manifest.is_some(),
                styles,
                hooks,
            },
        )
        .map_err(spec_failure)?;
        (
            Content::Text(rendered.content),
            rendered.warnings,
            rendered.manifest,
        )
    };
    report_warnings(
        rendered_warnings,
        warnings,
        json_diagnostics,
        strict.then_some(&allowed),
    )?;

    content.write(output)?;
    if let (Some(path), Some(manifest)) = (manifest, rendered_manifest) {
        fs::write(&path, manifest.to_json() + "\n")
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
    }
    Ok(())
}

/// Writes the warnings of a render to stderr as text lines, unless diagnostics are JSON, and adds
/// them to `warnings`. In strict mode, with the allowed warning codes, a warning is a failure.
fn report_warnings(
    rendered: Vec<ChartWarning>,
    warnings: &mut Vec<ChartWarning>,
    json_diagnostics: bool,
    strict: Option<&Vec<String>>,
) -> Result<(), Failure> {
    if !json_diagnostics {
        for warning in &rendered {
            eprintln!(
                "warning[{}] at {}: {}",
                warning.code, warning.path, warning.message
            );
        }
    }
    let rejected = strict.and_then(|allowed| strict_failure(&rendered, allowed));
    warnings.extend(rendered);
    rejected.map_or(Ok(()), Err)
}

/// What `chartlet render` writes: SVG or HTML text, or the bytes of a PNG.
enum Content {
    Text(String),
    Png(Vec<u8>),
}

impl Content {
    /// Writes the content to `output`, or to stdout without one.
    fn write(&self, output: Option<PathBuf>) -> Result<(), String> {
        match (output, self) {
            (Some(output), content) => {
                let bytes = match content {
                    Self::Text(text) => text.as_bytes(),
                    Self::Png(png) => png,
                };
                fs::write(&output, bytes)
                    .map_err(|error| format!("could not write {}: {error}", output.display()))
            }
            (None, Self::Text(text)) => write_stdout(text),
            (None, Self::Png(png)) => write_stdout_bytes(png),
        }
    }
}

fn spec_failure(error: chartlet::ChartError) -> Failure {
    Failure {
        code: error.code,
        path: Some(error.path),
        message: error.message,
    }
}

#[cfg(feature = "png")]
fn render_png(
    input: &str,
    variant: Variant,
    scale: Option<f32>,
    manifest: bool,
) -> Result<(Vec<u8>, Vec<ChartWarning>), Failure> {
    if manifest {
        return Err("--manifest is not available with --format png"
            .to_owned()
            .into());
    }
    let scale = scale.unwrap_or(1.0);
    let spec = chartlet::ChartSpec::from_json(input).map_err(spec_failure)?;
    let output = chartlet::render_png(&spec, &chartlet::PngOptions { variant, scale })
        .map_err(spec_failure)?;
    Ok((output.png, output.warnings))
}

#[cfg(not(feature = "png"))]
fn render_png(
    _input: &str,
    _variant: Variant,
    _scale: Option<f32>,
    _manifest: bool,
) -> Result<(Vec<u8>, Vec<ChartWarning>), Failure> {
    Err(
        "--format png needs chartlet built with the png feature: cargo install chartlet --features png"
            .to_owned()
            .into(),
    )
}

fn parse_scale(value: Option<&str>) -> Result<f32, String> {
    value
        .and_then(|value| value.parse::<f32>().ok())
        .ok_or_else(|| "--scale requires a number such as 2".to_owned())
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

/// `chartlet stylesheet [--types bar,time] [--no-common] | --common`: the shared stylesheet, for
/// every chart type or for the listed ones; `--no-common` leaves out the common part and
/// `--common` writes only that part.
fn shared_stylesheet(mut arguments: impl Iterator<Item = String>) -> Result<String, String> {
    let mut types = None;
    let mut common = true;
    let mut common_only = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--types" => {
                let names = arguments
                    .next()
                    .ok_or_else(|| "--types requires a list such as bar,time".to_owned())?;
                let list = names
                    .split(',')
                    .map(|name| {
                        ChartType::from_name(name.trim())
                            .ok_or_else(|| format!("unknown chart type {name:?}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                types = Some(list);
            }
            "--no-common" => common = false,
            "--common" => common_only = true,
            unknown => return Err(format!("unknown argument {unknown:?}\n\n{}", usage())),
        }
    }
    match (types, common, common_only) {
        (None, true, true) => Ok(stylesheet_common()),
        (_, _, true) => Err("--common takes neither --types nor --no-common".to_owned()),
        (None, true, false) => Ok(stylesheet()),
        (None, false, false) => Ok(stylesheet_types(&ChartType::ALL)),
        (Some(types), true, false) => Ok(stylesheet_for(&types)),
        (Some(types), false, false) => Ok(stylesheet_types(&types)),
    }
}

/// The format, and whether it is a PNG, which is rasterized from an SVG.
fn parse_format(value: Option<&str>) -> Result<(RenderFormat, bool), String> {
    match value {
        Some("svg") => Ok((RenderFormat::Svg, false)),
        Some("html") => Ok((RenderFormat::Html, false)),
        Some("png") => Ok((RenderFormat::Svg, true)),
        _ => Err("--format must be svg, html or png".to_owned()),
    }
}

fn parse_styles(value: Option<&str>) -> Result<Styles, String> {
    match value {
        Some("inline") => Ok(Styles::Inline),
        Some("external") => Ok(Styles::External),
        _ => Err("--styles must be inline or external".to_owned()),
    }
}

fn parse_variant(value: Option<&str>) -> Result<Variant, String> {
    match value {
        Some("desktop") => Ok(Variant::Desktop),
        Some("mobile") => Ok(Variant::Mobile),
        Some("print") => Ok(Variant::Print),
        Some("social") => Ok(Variant::Social),
        _ => Err("--variant must be desktop, mobile, print or social".to_owned()),
    }
}

/// The failure of strict mode: every warning counts except those whose code is allowed.
fn strict_failure(warnings: &[ChartWarning], allowed: &[String]) -> Option<Failure> {
    let count = warnings
        .iter()
        .filter(|warning| !allowed.iter().any(|code| code == warning.code))
        .count();
    (count > 0).then(|| Failure {
        code: "strict_warnings",
        path: None,
        message: format!("strict mode rejected {count} warning(s)"),
    })
}

fn usage() -> String {
    "usage: chartlet render <spec.json|-> [--format svg|html|png] [--scale <factor>] [-o <path>] [--id-prefix <prefix>] [--table details|visible] [--variant desktop|mobile|print|social] [--manifest <path>] [--styles inline|external] [--hooks] [--strict [--allow-warning <code>]...] [--diagnostics text|json]
       chartlet stylesheet [--types bar,time,...] [--no-common] | --common".to_owned()
}

/// Writes binary content, such as a PNG, to stdout as it is.
fn write_stdout_bytes(content: &[u8]) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    match stdout.write_all(content).and_then(|()| stdout.flush()) {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(|error| format!("could not write to stdout: {error}")),
    }
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
