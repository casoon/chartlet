//! A local HTML report that sets each example's reviewed SVG beside its current rendering, for
//! judging a change by eye. The golden tests stay the binding check: the report never fails on a
//! difference and never writes to `examples/`.

use std::{
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
};

use chartlet::{RenderFormat, RenderOptions, render_json};

/// One reviewed reference of the catalog: an example, the variant it is rendered in and its SVG.
pub struct Entry<'a> {
    pub name: &'a str,
    /// `desktop`, `mobile`, `print` or `social`.
    pub variant: &'static str,
    pub specification: &'a str,
    pub reference: &'a str,
    pub options: RenderOptions,
}

enum Status {
    Changed { offset: usize },
    Unchanged,
    Failed(String),
    Missing,
}

impl Status {
    fn class(&self) -> &'static str {
        match self {
            Self::Changed { .. } => "changed",
            Self::Unchanged => "unchanged",
            Self::Failed(_) => "failed",
            Self::Missing => "missing",
        }
    }
}

/// How many references of each kind the report shows.
#[derive(Default)]
pub struct Summary {
    pub changed: usize,
    pub failed: usize,
    pub unchanged: usize,
    pub missing: usize,
}

/// Writes `index.html` and the SVGs it shows into `directory`, which is emptied first. `missing`
/// are examples, as name and specification, that have no reviewed reference in the catalog.
pub fn write(
    directory: &Path,
    entries: &[Entry],
    missing: &[(String, String)],
) -> io::Result<Summary> {
    if directory.exists() {
        fs::remove_dir_all(directory)?;
    }
    fs::create_dir_all(directory.join("svg"))?;
    let rows = entries
        .iter()
        .map(|entry| {
            let Entry {
                name,
                variant,
                specification,
                reference,
                ..
            } = *entry;
            (
                name,
                variant,
                specification,
                Some(reference),
                entry.options.clone(),
            )
        })
        .chain(missing.iter().map(|(name, specification)| {
            let options = RenderOptions::default();
            (
                name.as_str(),
                "desktop",
                specification.as_str(),
                None,
                options,
            )
        }));
    let mut summary = Summary::default();
    let mut cards = String::new();
    for (index, (name, variant, specification, reference, options)) in rows.enumerate() {
        let current = render_json(specification, RenderFormat::Svg, &options)
            .map(|output| output.content)
            .map_err(|error| error.to_string());
        let status = match (&current, reference) {
            (Err(error), _) => Status::Failed(error.clone()),
            (Ok(_), None) => Status::Missing,
            (Ok(current), Some(reference)) if current == reference => Status::Unchanged,
            (Ok(current), Some(reference)) => Status::Changed {
                offset: first_difference(reference, current),
            },
        };
        let file = file_name(name, variant);
        if let Some(reference) = reference {
            fs::write(
                directory.join(format!("svg/{file}.reference.svg")),
                reference,
            )?;
        }
        if let Ok(current) = &current {
            fs::write(directory.join(format!("svg/{file}.current.svg")), current)?;
        }
        match status {
            Status::Changed { .. } => summary.changed += 1,
            Status::Unchanged => summary.unchanged += 1,
            Status::Failed(_) => summary.failed += 1,
            Status::Missing => summary.missing += 1,
        }
        let versions = (reference, current.as_deref().ok());
        card(&mut cards, index, name, variant, &status, versions);
    }
    fs::write(directory.join("index.html"), page(&summary, &cards))?;
    Ok(summary)
}

/// The base name of an example's files: `name`, then the variant unless it is the desktop one,
/// as in `examples/`.
fn file_name(name: &str, variant: &str) -> String {
    if variant == "desktop" {
        name.to_owned()
    } else {
        format!("{name}.{variant}")
    }
}

/// The byte offset of the first difference between two texts.
fn first_difference(reference: &str, current: &str) -> usize {
    reference
        .bytes()
        .zip(current.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| reference.len().min(current.len()))
}

/// Up to 80 characters of `text` around byte `offset`.
fn excerpt(text: &str, offset: usize) -> String {
    let mut start = offset.saturating_sub(30).min(text.len());
    while !text.is_char_boundary(start) {
        start -= 1;
    }
    text[start..].chars().take(80).collect()
}

/// The command that accepts the current rendering as the reviewed reference.
fn approve_command(name: &str, variant: &str) -> String {
    let file = file_name(name, variant);
    if variant == "desktop" {
        format!("cargo run -- render examples/{name}.json -o examples/{file}.svg")
    } else {
        format!(
            "cargo run -- render examples/{name}.json --variant {variant} -o examples/{file}.svg"
        )
    }
}

fn card(
    cards: &mut String,
    index: usize,
    name: &str,
    variant: &str,
    status: &Status,
    (reference, current): (Option<&str>, Option<&str>),
) {
    let file = file_name(name, variant);
    let class = status.class();
    let _ = write!(
        cards,
        "<section class=\"card {class}\"><h2>{} <small>{variant}</small> <span class=\"badge\">{class}</span></h2>",
        escape(name)
    );
    match status {
        Status::Changed { offset } => {
            let _ = write!(
                cards,
                "<p>First difference at byte {offset}:</p><pre>reference: {}\ncurrent:   {}</pre>",
                escape(&excerpt(reference.unwrap_or_default(), *offset)),
                escape(&excerpt(current.unwrap_or_default(), *offset)),
            );
        }
        Status::Failed(error) => {
            let _ = write!(cards, "<pre>{}</pre>", escape(error));
        }
        Status::Missing => {
            cards.push_str("<p>No reviewed reference in <code>tests/golden.rs</code>.</p>");
        }
        Status::Unchanged => {}
    }
    if matches!(status, Status::Changed { .. } | Status::Missing) {
        let _ = write!(
            cards,
            "<p>After review, accept it with <code>{}</code>.</p>",
            escape(&approve_command(name, variant))
        );
    }
    if matches!(status, Status::Changed { .. }) {
        let _ = write!(
            cards,
            "<fieldset class=\"views\"><legend>View</legend>\
             <label><input type=\"radio\" name=\"view-{index}\" value=\"side\" checked> side by side</label>\
             <label><input type=\"radio\" name=\"view-{index}\" value=\"overlay\"> overlay</label>\
             <label><input type=\"radio\" name=\"view-{index}\" value=\"difference\"> difference</label>\
             </fieldset>"
        );
    }
    cards.push_str("<div class=\"stage\">");
    if reference.is_some() {
        let _ = write!(
            cards,
            "<figure class=\"reference\"><img src=\"svg/{file}.reference.svg\" alt=\"\"><figcaption>reference</figcaption></figure>"
        );
    }
    if current.is_some() {
        let _ = write!(
            cards,
            "<figure class=\"current\"><img src=\"svg/{file}.current.svg\" alt=\"\"><figcaption>current</figcaption></figure>"
        );
    }
    cards.push_str("</div></section>\n");
}

fn page(summary: &Summary, cards: &str) -> String {
    format!(
        "<!doctype html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\">
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
<title>chartlet golden report</title>
<style>
:root {{ color-scheme: light; font: 15px/1.45 system-ui, sans-serif; }}
body {{ margin: 0 auto; max-width: 1400px; padding: 16px; background: #f4f4f5; color: #18181b; }}
header {{ position: sticky; top: 0; background: #f4f4f5; padding: 8px 0; border-bottom: 1px solid #d4d4d8; z-index: 1; }}
header label {{ margin-right: 1em; }}
.card {{ background: #fff; border: 1px solid #d4d4d8; border-radius: 6px; padding: 12px 16px; margin: 16px 0; }}
.card h2 {{ font-size: 1.05rem; margin: 0 0 8px; }}
.card h2 small {{ color: #52525b; font-weight: normal; }}
.badge {{ font-size: .75rem; font-weight: 600; padding: 2px 6px; border-radius: 4px; background: #e4e4e7; }}
.changed .badge {{ background: #fde68a; }}
.failed .badge {{ background: #fecaca; }}
.missing .badge {{ background: #bfdbfe; }}
pre {{ background: #f4f4f5; padding: 8px; overflow-x: auto; font-size: .8rem; }}
.views {{ border: 0; padding: 0; margin: 8px 0; }}
.views legend {{ float: left; margin-right: .75em; font-weight: 600; }}
.stage {{ display: flex; flex-wrap: wrap; gap: 16px; align-items: flex-start; }}
.stage figure {{ margin: 0; flex: 1 1 320px; min-width: 0; }}
.stage img {{ display: block; max-width: 100%; height: auto; background: #fff; outline: 1px solid #e4e4e7; }}
figcaption {{ font-size: .8rem; color: #52525b; }}
.card:has([value=overlay]:checked) .stage, .card:has([value=difference]:checked) .stage {{ display: grid; }}
.card:has([value=overlay]:checked) .stage figure, .card:has([value=difference]:checked) .stage figure {{ grid-area: 1 / 1; }}
.card:has([value=overlay]:checked) .current {{ opacity: .5; }}
.card:has([value=difference]:checked) .current img {{ mix-blend-mode: difference; }}
.card:has([value=overlay]:checked) figcaption, .card:has([value=difference]:checked) figcaption {{ display: none; }}
body:has(#show-unchanged:not(:checked)) .card.unchanged {{ display: none; }}
body:has(#show-missing:not(:checked)) .card.missing {{ display: none; }}
</style>
</head>
<body>
<header>
<h1>chartlet golden report</h1>
<p>{} changed, {} failed, {} unchanged, {} without reference.
<label><input type=\"checkbox\" id=\"show-unchanged\"> show unchanged</label>
<label><input type=\"checkbox\" id=\"show-missing\"> show without reference</label></p>
</header>
<main>
{cards}</main>
</body>
</html>
",
        summary.changed, summary.failed, summary.unchanged, summary.missing
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The report's directory below Cargo's scratch directory for integration tests.
pub fn directory() -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join("golden-report")
}
