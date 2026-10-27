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

/// The theme file a project sets its look in: found in the working directory and applied like
/// `--theme` unless another theme is given.
const PROJECT_THEME: &str = "chartlet.theme.json";

mod inspect_command;
mod preview;
mod watch;

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

impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.path {
            Some(path) => write!(formatter, "{} at {path}: {}", self.code, self.message),
            None => formatter.write_str(&self.message),
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
    report_result(result.as_ref().err(), &warnings, json_diagnostics);
    if result.is_err() {
        process::exit(1);
    }
}

/// Writes the outcome of one render to stderr: with JSON diagnostics as one JSON document of the
/// failure and the warnings, otherwise the failure as a line of text.
fn report_result(failure: Option<&Failure>, warnings: &[ChartWarning], json_diagnostics: bool) {
    if json_diagnostics {
        let error = failure.map(|failure| {
            json!({ "code": failure.code, "path": failure.path, "message": failure.message })
        });
        let report = json!({
            "ok": failure.is_none(),
            "error": error,
            "warnings": warnings.iter().map(warning_json).collect::<Vec<Value>>(),
        });
        eprintln!("{report}");
    } else if let Some(failure) = failure {
        eprintln!("chartlet: {failure}");
    }
}

fn warning_json(warning: &ChartWarning) -> Value {
    json!({ "code": warning.code, "path": warning.path, "message": warning.message })
}

/// What `chartlet render` is asked to render, and where to.
struct Job {
    input: PathBuf,
    format: RenderFormat,
    png: bool,
    scale: Option<f32>,
    output: Option<PathBuf>,
    manifest: Option<PathBuf>,
    options: RenderOptions,
    /// A bundled theme or the path of a theme file, applied to the specification.
    theme: Option<String>,
    /// A size preset, set as `size` of the specification; an explicit `width` or `height` still wins.
    size: Option<String>,
    /// The allowed warning codes in strict mode, `None` outside it.
    strict: Option<Vec<String>>,
    /// A TrueType or OpenType file the text is measured with and, for PNG, rasterized with.
    font: Option<PathBuf>,
    /// Carry the font in the SVG as a base64 `@font-face`.
    embed_font: bool,
}

/// The rendered content, its warnings and, if asked for, its manifest.
type Rendered = (Content, Vec<ChartWarning>, Option<chartlet::Manifest>);

/// Renders the specification `input` as `job` asks.
fn render_job(job: &Job, input: &str) -> Result<Rendered, Failure> {
    let themed;
    let input = match &job.theme {
        Some(theme) => {
            themed = chartlet::apply_theme(input, &read_theme(theme)?).map_err(spec_failure)?;
            themed.as_str()
        }
        None => input,
    };
    let sized;
    let input = match &job.size {
        Some(size) => {
            let mut spec: serde_json::Value = serde_json::from_str(input)
                .map_err(|error| format!("could not read the specification: {error}"))?;
            if let Some(object) = spec.as_object_mut() {
                object.insert("size".to_owned(), size.clone().into());
            }
            sized = spec.to_string();
            sized.as_str()
        }
        None => input,
    };
    let font = match &job.font {
        Some(path) => Some(
            fs::read(path)
                .map_err(|error| format!("could not read {}: {error}", path.display()))?,
        ),
        None => None,
    };
    if job.embed_font && (font.is_none() || job.png || job.format != RenderFormat::Svg) {
        return Err("--embed-font needs --font and --format svg"
            .to_owned()
            .into());
    }
    if job.png && job.options.all_views {
        Err("--all-views requires --format html".to_owned().into())
    } else if job.png {
        let (png, warnings) = render_png(
            input,
            job.options.view.as_deref(),
            job.options.perspective.as_deref(),
            job.options.variant,
            job.scale,
            job.manifest.is_some(),
            font.as_deref(),
        )?;
        Ok((Content::Png(png), warnings, None))
    } else if job.scale.is_some() {
        Err("--scale requires --format png".to_owned().into())
    } else {
        let rendered = match &font {
            Some(font) => chartlet::ChartSpec::from_json(input)
                .and_then(|spec| chartlet::render_with_font(&spec, job.format, &job.options, font))
                .map_err(spec_failure)?,
            None => render_json(input, job.format, &job.options).map_err(spec_failure)?,
        };
        let manifest = rendered.manifest.map(|manifest| chartlet::Manifest {
            theme: job.theme.clone(),
            ..manifest
        });
        let content = match (&font, job.embed_font) {
            (Some(font), true) => {
                let family = chartlet::ChartSpec::from_json(input)
                    .ok()
                    .and_then(|spec| spec.typography.and_then(|typography| typography.family))
                    .unwrap_or_default();
                chartlet::embed_font(&rendered.content, &family, font)
            }
            _ => rendered.content,
        };
        Ok((Content::Text(content), rendered.warnings, manifest))
    }
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
        Some("inspect") => return inspect_command::run(arguments),
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
    let mut view = None;
    let mut perspective = None;
    let mut theme = None;
    let mut size = None;
    let mut all_views = false;
    let mut watch = false;
    let mut preview = false;
    let mut port = None;
    let mut font = None;
    let mut embed_font = false;

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
            "--view" => {
                view = Some(
                    arguments
                        .next()
                        .ok_or_else(|| "--view requires the key of a view".to_owned())?,
                );
            }
            "--perspective" => {
                perspective =
                    Some(arguments.next().ok_or_else(|| {
                        "--perspective requires the key of a perspective".to_owned()
                    })?);
            }
            "--theme" => {
                theme = Some(arguments.next().ok_or_else(|| {
                    "--theme requires light, dark, blueprint or the path of a theme file".to_owned()
                })?);
            }
            "--size" => {
                size = Some(arguments.next().ok_or_else(|| {
                    "--size requires a preset such as slide-16x9 or print-a4-landscape".to_owned()
                })?);
            }
            "--all-views" => all_views = true,
            "--font" => font = Some(path_after(&mut arguments, "--font")?),
            "--embed-font" => embed_font = true,
            "--styles" => styles = parse_styles(arguments.next().as_deref())?,
            "--hooks" => hooks = true,
            "--manifest" => manifest = Some(path_after(&mut arguments, "--manifest")?),
            "--diagnostics" => match arguments.next().as_deref() {
                Some("json" | "text") => {}
                _ => return Err("--diagnostics must be text or json".to_owned().into()),
            },
            "--diagnostics=json" | "--diagnostics=text" => {}
            "--watch" => watch = true,
            "--preview" => preview = true,
            "--port" => port = Some(parse_port(arguments.next().as_deref())?),
            "-h" | "--help" => return Ok(write_stdout(&usage())?),
            unknown => return Err(format!("unknown argument {unknown:?}\n\n{}", usage()).into()),
        }
    }

    // The project's own look applies unless another theme is asked for.
    let theme = theme.or_else(|| {
        Path::new(PROJECT_THEME)
            .is_file()
            .then(|| PROJECT_THEME.to_owned())
    });
    let job = Job {
        input,
        format,
        png,
        scale,
        output,
        options: RenderOptions {
            id_prefix,
            table_mode,
            variant,
            manifest: manifest.is_some(),
            styles,
            hooks,
            view,
            perspective,
            all_views,
        },
        theme,
        size,
        manifest,
        strict: strict.then_some(allowed),
        font,
        embed_font,
    };
    if watch || preview {
        return watch::run(&job, preview.then(|| port.unwrap_or(0)), json_diagnostics);
    } else if port.is_some() {
        return Err("--port requires --preview".to_owned().into());
    }

    let input_content = read_input(&job.input)?;
    let (content, rendered_warnings, rendered_manifest) = render_job(&job, &input_content)?;
    report_warnings(
        rendered_warnings,
        warnings,
        json_diagnostics,
        job.strict.as_ref(),
    )?;

    content.write(job.output.as_deref())?;
    if let (Some(path), Some(manifest)) = (&job.manifest, rendered_manifest) {
        fs::write(path, manifest.to_json() + "\n")
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
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Text(text) => text.as_bytes(),
            Self::Png(png) => png,
        }
    }

    /// Writes the content to `output`, or to stdout without one.
    fn write(&self, output: Option<&Path>) -> Result<(), String> {
        match (output, self) {
            (Some(output), content) => fs::write(output, content.bytes())
                .map_err(|error| format!("could not write {}: {error}", output.display())),
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
    view: Option<&str>,
    perspective: Option<&str>,
    variant: Variant,
    scale: Option<f32>,
    manifest: bool,
    font: Option<&[u8]>,
) -> Result<(Vec<u8>, Vec<ChartWarning>), Failure> {
    if manifest {
        return Err("--manifest is not available with --format png"
            .to_owned()
            .into());
    }
    let scale = scale.unwrap_or(1.0);
    let spec = chartlet::ChartSpec::from_json(input)
        .and_then(|spec| match perspective {
            Some(key) => spec.with_perspective(key),
            None => Ok(spec),
        })
        .and_then(|spec| spec.view(view))
        .map_err(spec_failure)?;
    let options = chartlet::PngOptions { variant, scale };
    let output = match font {
        Some(font) => chartlet::render_png_with_font(&spec, &options, font),
        None => chartlet::render_png(&spec, &options),
    }
    .map_err(spec_failure)?;
    Ok((output.png, output.warnings))
}

#[cfg(not(feature = "png"))]
fn render_png(
    _input: &str,
    _view: Option<&str>,
    _perspective: Option<&str>,
    _variant: Variant,
    _scale: Option<f32>,
    _manifest: bool,
    _font: Option<&[u8]>,
) -> Result<(Vec<u8>, Vec<ChartWarning>), Failure> {
    Err(
        "--format png needs chartlet built with the png feature: cargo install chartlet --features png"
            .to_owned()
            .into(),
    )
}

fn parse_port(value: Option<&str>) -> Result<u16, String> {
    value
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| "--port requires a port number such as 4000".to_owned())
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
    let css = match (types, common, common_only) {
        (None, true, true) => stylesheet_common(),
        (_, _, true) => return Err("--common takes neither --types nor --no-common".to_owned()),
        (None, true, false) => stylesheet(),
        (None, false, false) => stylesheet_types(&ChartType::ALL),
        (Some(types), true, false) => stylesheet_for(&types),
        (Some(types), false, false) => stylesheet_types(&types),
    };
    // The file lands in a project's source tree, where formatters would treat it as hand-written.
    Ok(format!("{STYLESHEET_HEADER}\n{css}"))
}

/// The first line of the stylesheet the CLI prints; `stylesheet()` of the libraries has none.
const STYLESHEET_HEADER: &str = "/* generated by chartlet - do not edit */";

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
    "usage: chartlet render <spec.json|-> [--format svg|html|png] [--scale <factor>] [-o <path>] [--id-prefix <prefix>] [--table details|visible] [--variant desktop|mobile|print|social] [--view <key> | --all-views] [--perspective <key>] [--theme <light|dark|blueprint|theme.json>] [--size <preset>] [--manifest <path>] [--font <file.ttf> [--embed-font]] [--styles inline|external] [--hooks] [--strict [--allow-warning <code>]...] [--diagnostics text|json] [--watch] [--preview [--port <port>]]
       chartlet inspect <spec.json|-> [--disable <code>]... [--strict] [--format text|json] [--diagnostics text|json]
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

/// The text of a theme file: a bundled theme by its name, otherwise the file at that path.
fn read_theme(theme: &str) -> Result<String, String> {
    match chartlet::bundled_theme(theme) {
        Some(bundled) => Ok(bundled.to_owned()),
        None => {
            fs::read_to_string(theme).map_err(|error| format!("could not read {theme}: {error}"))
        }
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
