use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process,
};

use chartlet::{RenderFormat, RenderOptions, TableMode, render_json};

fn main() {
    if let Err(message) = run() {
        eprintln!("chartlet: {message}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);
    match arguments.next().as_deref() {
        Some("render") => {}
        Some("-h" | "--help") => return write_stdout(&usage()),
        _ => return Err(usage()),
    }
    let input = PathBuf::from(arguments.next().ok_or_else(usage)?);
    let mut format = RenderFormat::Svg;
    let mut output = None;
    let mut id_prefix = None;
    let mut strict = false;
    let mut table_mode = TableMode::Details;

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--format" => {
                format = match arguments.next().as_deref() {
                    Some("svg") => RenderFormat::Svg,
                    Some("html") => RenderFormat::Html,
                    _ => return Err("--format must be svg or html".to_owned()),
                };
            }
            "-o" | "--output" => {
                output = Some(PathBuf::from(
                    arguments
                        .next()
                        .ok_or_else(|| "--output requires a path".to_owned())?,
                ));
            }
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
                    _ => return Err("--table must be details or visible".to_owned()),
                };
            }
            "-h" | "--help" => return write_stdout(&usage()),
            unknown => return Err(format!("unknown argument {unknown:?}\n\n{}", usage())),
        }
    }

    let input_content = read_input(&input)?;
    let rendered = render_json(
        &input_content,
        format,
        &RenderOptions {
            id_prefix,
            table_mode,
        },
    )
    .map_err(|error| error.to_string())?;

    for warning in &rendered.warnings {
        eprintln!(
            "warning[{}] at {}: {}",
            warning.code, warning.path, warning.message
        );
    }
    if strict && !rendered.warnings.is_empty() {
        return Err(format!(
            "strict mode rejected {} warning(s)",
            rendered.warnings.len()
        ));
    }

    if let Some(output) = output {
        fs::write(&output, rendered.content)
            .map_err(|error| format!("could not write {}: {error}", output.display()))?;
    } else {
        write_stdout(&rendered.content)?;
    }
    Ok(())
}

fn usage() -> String {
    "usage: chartlet render <spec.json|-> [--format svg|html] [-o <path>] [--id-prefix <prefix>] [--table details|visible] [--strict]".to_owned()
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
