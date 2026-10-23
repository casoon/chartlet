//! `chartlet render --watch`: renders the specification again whenever its file changes. Part of
//! the command-line program, not of the library.

use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use crate::{Content, Failure, Job, preview::Preview, read_input, render_job, report_result};

/// How often the specification file is read to see whether it changed.
const POLL: Duration = Duration::from_millis(200);

/// Renders `job` once and then after every change of its specification file, until the process
/// is stopped. Each render reports its warnings and errors as a single render does; a failed one
/// leaves the last output in place. With `preview_port`, also serves a preview on the loopback
/// interface. Returns only when watching cannot start.
pub fn run(job: &Job, preview_port: Option<u16>, json_diagnostics: bool) -> Result<(), Failure> {
    check(job, preview_port.is_some())?;
    let preview = preview_port
        .map(|port| Preview::serve(port, job))
        .transpose()?;
    if !json_diagnostics {
        eprintln!("chartlet: watching {}, Ctrl-C stops", job.input.display());
    }
    let mut last = read_input(&job.input);
    render(job, &last, preview.as_deref(), json_diagnostics);
    loop {
        thread::sleep(POLL);
        let mut current = read_input(&job.input);
        if current == last {
            continue;
        }
        // An editor may save in several writes, or remove the file before it puts the new one in
        // its place: wait until the file holds still.
        loop {
            thread::sleep(POLL);
            let next = read_input(&job.input);
            if next == current {
                break;
            }
            current = next;
        }
        if current != last {
            last = current;
            render(job, &last, preview.as_deref(), json_diagnostics);
        }
    }
}

/// The options that watching does not combine with.
fn check(job: &Job, preview: bool) -> Result<(), Failure> {
    if job.input == Path::new("-") {
        return Err(
            "--watch needs a specification file; it cannot watch standard input"
                .to_owned()
                .into(),
        );
    }
    if preview && job.png {
        return Err("--preview shows SVG or HTML; use --watch alone for a PNG"
            .to_owned()
            .into());
    }
    if job.output.is_none() && !preview {
        return Err(
            "--watch needs -o <path>; it does not write to standard output"
                .to_owned()
                .into(),
        );
    }
    for (option, path) in [("-o", &job.output), ("--manifest", &job.manifest)] {
        if path
            .as_ref()
            .is_some_and(|path| same_file(path, &job.input))
        {
            return Err(
                format!("{option} must not be the specification file that is watched").into(),
            );
        }
    }
    Ok(())
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// Renders the specification `input`, reports the outcome and, if it succeeds, replaces the
/// output and the manifest.
fn render(
    job: &Job,
    input: &Result<String, String>,
    preview: Option<&Preview>,
    json_diagnostics: bool,
) {
    let mut warnings = Vec::new();
    let result = input
        .clone()
        .map_err(Failure::from)
        .and_then(|input| render_job(job, &input))
        .and_then(|(content, rendered, manifest)| {
            crate::report_warnings(
                rendered,
                &mut warnings,
                json_diagnostics,
                job.strict.as_ref(),
            )?;
            if let Some(output) = &job.output {
                replace(output, content.bytes())?;
            }
            if let (Some(path), Some(manifest)) = (&job.manifest, manifest) {
                replace(path, (manifest.to_json() + "\n").as_bytes())?;
            }
            Ok(content)
        });
    // The preview first, so that whoever reacts to the report finds the page of this render.
    if let Some(preview) = preview {
        preview.update(
            match &result {
                Ok(Content::Text(text)) => Ok(text.clone()),
                Ok(Content::Png(_)) => unreachable!("the preview refuses PNG output"),
                Err(failure) => Err(failure.to_string()),
            },
            &warnings,
        );
    }
    report_result(result.as_ref().err(), &warnings, json_diagnostics);
    if !json_diagnostics && result.is_ok() {
        match &job.output {
            Some(output) => eprintln!("chartlet: wrote {}", output.display()),
            None => eprintln!("chartlet: rendered {}", job.input.display()),
        }
    }
}

/// Replaces `path` with `bytes` in one step: written next to it first, then renamed over it, so
/// that a reader never sees half an output.
fn replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("{} is not a file path", path.display()))?;
    let mut temporary = PathBuf::from(path);
    temporary.set_file_name(format!(".{}.chartlet-tmp", name.to_string_lossy()));
    fs::write(&temporary, bytes)
        .and_then(|()| fs::rename(&temporary, path))
        .map_err(|error| format!("could not write {}: {error}", path.display()))
}
