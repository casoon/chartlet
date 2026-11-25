//! `chartlet mdbook`: an mdBook preprocessor that renders every fenced block in the language
//! `chartlet` as the chart's HTML figure. Part of the command-line program, not of the library.
//!
//! mdBook first asks `chartlet mdbook supports <renderer>`, then passes `[context, book]` as JSON
//! on stdin and reads the book back from stdout.

use std::{
    io::{self, Read},
    path::Path,
    process,
};

use chartlet::{RenderFormat, RenderOptions, TableMode, render_json};
use serde_json::Value;

use crate::{Failure, PROJECT_THEME, read_theme, usage, write_stdout};

/// The settings a fence may carry after its language, as in `chartlet id=revenue view=checkout`.
const FENCE_SETTINGS: [&str; 3] = ["id", "view", "perspective"];

pub fn run(mut arguments: impl Iterator<Item = String>) -> Result<(), Failure> {
    match arguments.next().as_deref() {
        // Only the HTML renderer shows the figure; mdBook skips the preprocessor for the others.
        Some("supports") => match arguments.next().as_deref() {
            Some("html") => return Ok(()),
            Some(_) => process::exit(1),
            None => {
                return Err("chartlet mdbook supports requires a renderer"
                    .to_owned()
                    .into());
            }
        },
        Some("-h" | "--help") => return Ok(write_stdout(&usage())?),
        Some(unknown) => {
            return Err(format!("unknown argument {unknown:?}\n\n{}", usage()).into());
        }
        None => {}
    }

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("could not read stdin: {error}"))?;
    let (context, mut book) = match serde_json::from_str::<Value>(&input) {
        Ok(Value::Array(mut pair)) if pair.len() == 2 => {
            let book = pair.pop().expect("two elements");
            (pair.pop().expect("two elements"), book)
        }
        _ => {
            return Err(
                "chartlet mdbook expects [context, book] from mdBook on stdin"
                    .to_owned()
                    .into(),
            );
        }
    };
    let settings = Settings::from_context(&context)?;
    for_each_chapter(&mut book, &mut |chapter| render_chapter(chapter, &settings))?;
    Ok(write_stdout(&book.to_string())?)
}

/// What `[preprocessor.chartlet]` in `book.toml` sets for every chart of the book.
struct Settings {
    table_mode: TableMode,
    /// The text of the theme file applied to every specification.
    theme: Option<String>,
}

impl Settings {
    fn from_context(context: &Value) -> Result<Self, Failure> {
        let config = &context["config"]["preprocessor"]["chartlet"];
        let table_mode = match config["table"].as_str() {
            None | Some("details") => TableMode::Details,
            Some("visible") => TableMode::Visible,
            Some(_) => {
                return Err("preprocessor.chartlet.table must be details or visible"
                    .to_owned()
                    .into());
            }
        };
        // A theme file is found next to book.toml, as the CLI finds one in the working directory.
        let root = Path::new(context["root"].as_str().unwrap_or("."));
        let theme = match config["theme"].as_str() {
            Some(theme) if chartlet::bundled_theme(theme).is_some() => Some(read_theme(theme)?),
            Some(theme) => Some(read_theme(&root.join(theme).to_string_lossy())?),
            None if root.join(PROJECT_THEME).is_file() => {
                Some(read_theme(&root.join(PROJECT_THEME).to_string_lossy())?)
            }
            None => None,
        };
        Ok(Self { table_mode, theme })
    }
}

/// Calls `render` with every chapter of the book, whatever the nesting: a chapter is an object
/// with `content` and `sub_items`.
fn for_each_chapter(
    value: &mut Value,
    render: &mut impl FnMut(&mut serde_json::Map<String, Value>) -> Result<(), Failure>,
) -> Result<(), Failure> {
    match value {
        Value::Object(object) => {
            if object.get("content").is_some_and(Value::is_string)
                && object.contains_key("sub_items")
            {
                render(object)?;
            }
            for child in object.values_mut() {
                for_each_chapter(child, render)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                for_each_chapter(item, render)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn render_chapter(
    chapter: &mut serde_json::Map<String, Value>,
    settings: &Settings,
) -> Result<(), Failure> {
    let source = chapter
        .get("source_path")
        .or_else(|| chapter.get("path"))
        .and_then(Value::as_str)
        .or_else(|| chapter.get("name").and_then(Value::as_str))
        .unwrap_or("chapter")
        .to_owned();
    let content = chapter["content"].as_str().unwrap_or_default();
    let rendered = replace_fences(content, |spec, meta, line| {
        render_fence(spec, meta, settings)
            .map_err(|message| format!("{source} line {line}: {message}"))
            .map(|(html, warnings)| {
                for warning in warnings {
                    eprintln!("{warning} ({source} line {line})");
                }
                html
            })
    })?;
    chapter.insert("content".to_owned(), Value::String(rendered));
    Ok(())
}

/// Renders one block: its HTML and its warnings as the CLI writes them.
fn render_fence(
    spec: &str,
    meta: &str,
    settings: &Settings,
) -> Result<(String, Vec<String>), String> {
    let mut options = RenderOptions {
        table_mode: settings.table_mode,
        ..RenderOptions::default()
    };
    for (key, value) in fence_settings(meta)? {
        match key.as_str() {
            "id" => options.id_prefix = Some(value),
            "view" => options.view = Some(value),
            _ => options.perspective = Some(value),
        }
    }
    let themed;
    let spec = match &settings.theme {
        Some(theme) => {
            themed = chartlet::apply_theme(spec, theme).map_err(|error| error.to_string())?;
            themed.as_str()
        }
        None => spec,
    };
    let output = render_json(spec, RenderFormat::Html, &options)
        .map_err(|error| format!("{} at {}: {}", error.code, error.path, error.message))?;
    let warnings = output
        .warnings
        .iter()
        .map(|warning| {
            format!(
                "warning[{}] at {}: {}",
                warning.code, warning.path, warning.message
            )
        })
        .collect();
    Ok((output.content, warnings))
}

/// The `key=value` settings after the fence language; a value may be quoted.
fn fence_settings(meta: &str) -> Result<Vec<(String, String)>, String> {
    let mut settings = Vec::new();
    let mut rest = meta.trim();
    while !rest.is_empty() {
        let unreadable = || {
            format!(
                "cannot read {rest:?} after the fence language; write key=value with {}",
                FENCE_SETTINGS.join(", ")
            )
        };
        let (key, after) = rest.split_once('=').ok_or_else(unreadable)?;
        if key.is_empty() || key.contains(char::is_whitespace) {
            return Err(unreadable());
        }
        if !FENCE_SETTINGS.contains(&key) {
            return Err(format!(
                "unknown fence setting {key:?}; the settings are {}",
                FENCE_SETTINGS.join(", ")
            ));
        }
        let (value, remaining) = match after.chars().next() {
            Some(quote @ ('"' | '\'')) => after[1..].split_once(quote).ok_or_else(unreadable)?,
            _ => after.split_once(char::is_whitespace).unwrap_or((after, "")),
        };
        settings.push((key.to_owned(), value.to_owned()));
        rest = remaining.trim_start();
    }
    Ok(settings)
}

/// Replaces every top-level fenced block in the language `chartlet` with what `render` returns
/// for its content, its settings and the line its fence starts on. Other fences are kept as they
/// are, with what is inside them.
fn replace_fences(
    content: &str,
    mut render: impl FnMut(&str, &str, usize) -> Result<String, String>,
) -> Result<String, Failure> {
    let mut output = String::with_capacity(content.len());
    let mut lines = content.split_inclusive('\n').enumerate();
    while let Some((index, line)) = lines.next() {
        let Some((marker, info)) = opening_fence(line) else {
            output.push_str(line);
            continue;
        };
        let (language, meta) = info.split_once(char::is_whitespace).unwrap_or((info, ""));
        let mut body = String::new();
        let mut closing = None;
        for (_, inner) in lines.by_ref() {
            if closes(inner, marker) {
                closing = Some(inner);
                break;
            }
            body.push_str(inner);
        }
        if language == "chartlet" {
            let html = render(&body, meta, index + 1).map_err(Failure::from)?;
            // A blank line ends the HTML block, so the Markdown after it is read as Markdown.
            output.push_str(&html);
            output.push_str("\n\n");
        } else {
            output.push_str(line);
            output.push_str(&body);
            output.push_str(closing.unwrap_or_default());
        }
    }
    Ok(output)
}

/// The fence a line opens, as its character and length, and the info string after it.
fn opening_fence(line: &str) -> Option<((char, usize), &str)> {
    let trimmed = line.trim_end_matches(['\n', '\r']);
    let indent = trimmed.len() - trimmed.trim_start_matches(' ').len();
    let rest = &trimmed[indent..];
    let character = rest.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let length = rest.len() - rest.trim_start_matches(character).len();
    let info = rest[length..].trim();
    (indent < 4 && length >= 3 && !(character == '`' && info.contains('`')))
        .then_some(((character, length), info))
}

/// Whether a line closes the fence `marker`: the same character, at least as many, nothing after.
fn closes(line: &str, (character, length): (char, usize)) -> bool {
    let trimmed = line.trim_end();
    let indent = trimmed.len() - trimmed.trim_start_matches(' ').len();
    let rest = &trimmed[indent..];
    indent < 4 && rest.len() >= length && rest.chars().all(|c| c == character)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replace(content: &str) -> String {
        replace_fences(content, |body, meta, line| {
            Ok(format!(
                "<chart line={line} meta={meta:?}>{}</chart>",
                body.trim()
            ))
        })
        .unwrap_or_else(|failure| panic!("{failure}"))
    }

    #[test]
    fn replaces_chartlet_fences_and_keeps_others() {
        let content = "# Title\n\n```chartlet id=a\n{}\n```\nText\n\n~~~js\n```chartlet\n~~~\n";
        assert_eq!(
            replace(content),
            "# Title\n\n<chart line=3 meta=\"id=a\">{}</chart>\n\nText\n\n~~~js\n```chartlet\n~~~\n"
        );
    }

    #[test]
    fn longer_fences_close_only_with_as_many_marks() {
        assert_eq!(
            replace("````chartlet\n{\n```\n}\n````\n"),
            "<chart line=1 meta=\"\">{\n```\n}</chart>\n\n"
        );
    }

    #[test]
    fn reads_fence_settings() {
        assert_eq!(
            fence_settings(r#"id=flow view="check out" perspective='team'"#),
            Ok(vec![
                ("id".to_owned(), "flow".to_owned()),
                ("view".to_owned(), "check out".to_owned()),
                ("perspective".to_owned(), "team".to_owned()),
            ])
        );
        assert!(
            fence_settings("idd=x")
                .is_err_and(|error| error.starts_with("unknown fence setting \"idd\""))
        );
        assert!(fence_settings("id").is_err());
    }
}
