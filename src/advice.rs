//! Actionable diagnostics: structured help for a few layout warnings, computed on request.
//!
//! A warning already carries a code, a JSON Pointer and a message. Help adds what a person or an
//! agent would otherwise parse out of the message: the field to change, the documentation, and a
//! small number of remedies. A remedy that can be written as a change to the specification is a
//! reviewable patch (JSON Patch `add`/`replace` operations), and it is only offered after the
//! patched specification was rendered with the compiler: `validated` says that happened and
//! `resolves` says what the render showed. The specification the caller passed is never changed;
//! applying a patch is the caller's choice. A remedy never touches data, labels or the
//! relationships of a model: it changes the canvas, or it says in words what only the author can
//! decide and carries no patch.

use serde_json::{Value, json};

use crate::{
    ChartSpec, RenderFormat, RenderOptions,
    error::{ChartError, ChartWarning},
    render_json,
};

/// Where the reference for every warning lives, relative to the root of the documentation site.
const DOCS: &str = "docs/guides/warnings-and-errors/";
/// The widest canvas a specification may have.
const MAX_WIDTH: u32 = 2_400;
/// How much wider a canvas is tried for a truncated text, in order.
const WIDEN: [f64; 3] = [1.25, 1.5, 2.0];

/// What kind of change a remedy is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemedyKind {
    /// Changes how the figure is laid out: the canvas size. The data stays as it is.
    Layout,
    /// Needs a decision about the content, which only its author can make; carries no patch.
    Content,
}

impl RemedyKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Layout => "layout",
            Self::Content => "content",
        }
    }
}

/// One operation of a proposed change: `op` is `add` or `replace`, `path` a JSON Pointer.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchOperation {
    pub op: &'static str,
    pub path: String,
    pub value: Value,
}

/// A candidate correction.
#[derive(Debug, Clone, PartialEq)]
pub struct Remedy {
    pub id: &'static str,
    pub kind: RemedyKind,
    pub summary: String,
    /// Empty for a remedy that is guidance only.
    pub patch: Vec<PatchOperation>,
    /// Whether the patched specification was rendered to find out.
    pub validated: bool,
    /// Whether that render no longer had the warning, and no layout warning that was not there.
    pub resolves: bool,
}

/// Help for one warning.
#[derive(Debug, Clone, PartialEq)]
pub struct Help {
    pub code: &'static str,
    /// The path of the warning.
    pub path: String,
    /// The field of the specification the help is about: where to look first.
    pub field: String,
    /// The documentation page, relative to the root of the site.
    pub docs: &'static str,
    pub note: Option<String>,
    pub remedies: Vec<Remedy>,
}

impl Help {
    /// The help as JSON: `code`, `path`, `field`, `docs`, `note` when there is one, and `remedies`
    /// with `id`, `kind`, `summary`, `validated`, `resolves` and `patch`.
    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut help = json!({
            "code": self.code,
            "path": self.path,
            "field": self.field,
            "docs": self.docs,
            "remedies": self.remedies.iter().map(|remedy| json!({
                "id": remedy.id,
                "kind": remedy.kind.name(),
                "summary": remedy.summary,
                "validated": remedy.validated,
                "resolves": remedy.resolves,
                "patch": remedy.patch.iter().map(|operation| json!({
                    "op": operation.op,
                    "path": operation.path,
                    "value": operation.value,
                })).collect::<Vec<Value>>(),
            })).collect::<Vec<Value>>(),
        });
        if let Some(note) = &self.note {
            help["note"] = json!(note);
        }
        help
    }
}

/// Help for the warnings of rendering `spec` (the JSON text) with `options`. Only warnings that
/// have help are answered; a warning whose correction the compiler cannot verify gets honest
/// guidance without a patch. Returns nothing for a specification that does not render.
#[must_use]
pub fn advise(spec: &str, options: &RenderOptions, warnings: &[ChartWarning]) -> Vec<Help> {
    let Ok(parsed) = serde_json::from_str::<Value>(spec) else {
        return Vec::new();
    };
    let Ok(chart) = ChartSpec::from_json(spec) else {
        return Vec::new();
    };
    let baseline = layout_warnings(spec, options);
    let mut help = Vec::new();
    for warning in warnings {
        let entry = match warning.code {
            "canvas_too_small" => canvas_too_small(warning, &parsed, options, &baseline),
            "text_truncated" => Some(text_truncated(warning, &parsed, &chart, options, &baseline)),
            "label_overlap" | "dense_chart" => Some(guidance(warning)),
            _ => None,
        };
        help.extend(entry);
    }
    help
}

/// The `(code, path)` of every layout warning the render of `spec` has.
fn layout_warnings(spec: &str, options: &RenderOptions) -> Vec<(&'static str, String)> {
    render_json(spec, RenderFormat::Svg, options).map_or_else(
        |_| Vec::new(),
        |output| {
            output
                .warnings
                .into_iter()
                .filter(|warning| matches!(warning.code, "canvas_too_small" | "text_truncated"))
                .map(|warning| (warning.code, warning.path))
                .collect()
        },
    )
}

/// Renders `patched` and says whether the warning `(code, path)` is gone and no layout warning
/// appeared that the original did not have.
fn resolves(
    patched: &str,
    options: &RenderOptions,
    code: &str,
    path: &str,
    baseline: &[(&'static str, String)],
) -> bool {
    let Ok(output) = render_json(patched, RenderFormat::Svg, options) else {
        return false;
    };
    output.warnings.iter().all(|warning| {
        (warning.code != code || warning.path != path)
            && (!matches!(warning.code, "canvas_too_small" | "text_truncated")
                || baseline
                    .iter()
                    .any(|(known, at)| *known == warning.code && *at == warning.path))
    })
}

/// The specification with `field` set to `value`, as text, and the operation that does it.
fn with_field(parsed: &Value, field: &str, value: u32) -> Option<(String, PatchOperation)> {
    let mut patched = parsed.clone();
    let key = field.strip_prefix('/')?;
    let exists = patched.get(key).is_some();
    patched
        .as_object_mut()?
        .insert(key.to_owned(), json!(value));
    Some((
        patched.to_string(),
        PatchOperation {
            op: if exists { "replace" } else { "add" },
            path: field.to_owned(),
            value: json!(value),
        },
    ))
}

fn canvas_too_small(
    warning: &ChartWarning,
    parsed: &Value,
    options: &RenderOptions,
    baseline: &[(&'static str, String)],
) -> Option<Help> {
    let field = warning.path.as_str();
    if (field != "/width" && field != "/height")
        || !baseline
            .iter()
            .any(|(code, path)| *code == warning.code && path == field)
    {
        return None;
    }
    let needed: u32 = warning
        .message
        .split("needs ")
        .nth(1)?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    let (patched, operation) = with_field(parsed, field, needed)?;
    let fixed = resolves(&patched, options, warning.code, field, baseline);
    Some(Help {
        code: "canvas_too_small",
        path: warning.path.clone(),
        field: warning.path.clone(),
        docs: DOCS,
        note: Some(
            "Changing the canvas is a layout change: no label, value or connection is touched. Shorter labels need a smaller canvas."
                .to_owned(),
        ),
        remedies: vec![Remedy {
            id: "raise-canvas",
            kind: RemedyKind::Layout,
            summary: format!("Set {} to {needed} pixels, the size the diagram needs.", &field[1..]),
            patch: vec![operation],
            validated: true,
            resolves: fixed,
        }],
    })
}

fn text_truncated(
    warning: &ChartWarning,
    parsed: &Value,
    chart: &ChartSpec,
    options: &RenderOptions,
    baseline: &[(&'static str, String)],
) -> Help {
    let mut remedies = Vec::new();
    let mut note =
        "The full text stays in the specification and in the data table; only the drawing is shortened."
            .to_owned();
    let in_desktop_layout = baseline
        .iter()
        .any(|(code, path)| *code == warning.code && *path == warning.path);
    if in_desktop_layout {
        for factor in WIDEN {
            // Canvas sizes are at most 2400, far below where the conversion could lose a digit.
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_lossless
            )]
            let width = (f64::from(chart.width) * factor).ceil() as u32;
            if width > MAX_WIDTH || width <= chart.width {
                break;
            }
            let Some((patched, operation)) = with_field(parsed, "/width", width) else {
                break;
            };
            if resolves(&patched, options, warning.code, &warning.path, baseline) {
                remedies.push(Remedy {
                    id: "widen-canvas",
                    kind: RemedyKind::Layout,
                    summary: format!(
                        "Set width to {width} pixels (from {}): the text fits in the compiler's render.",
                        chart.width
                    ),
                    patch: vec![operation],
                    validated: true,
                    resolves: true,
                });
                break;
            }
        }
    } else {
        // The desktop render does not have this warning, so a change of the desktop canvas
        // cannot be checked against it: the text is shortened in the mobile layout.
        note.push_str(" This text is shortened in the mobile layout: widen the mobile width, or shorten the text.");
    }
    remedies.push(Remedy {
        id: "shorten-text",
        kind: RemedyKind::Content,
        summary: format!(
            "Shorten the text at {} yourself, or move detail to the description. chartlet does not rewrite your content.",
            warning.path
        ),
        patch: Vec::new(),
        validated: false,
        resolves: false,
    });
    Help {
        code: "text_truncated",
        path: warning.path.clone(),
        field: warning.path.clone(),
        docs: DOCS,
        note: Some(note),
        remedies,
    }
}

/// Honest guidance for a warning chartlet cannot correct by itself.
fn guidance(warning: &ChartWarning) -> Help {
    let (code, note) = match warning.code {
        "label_overlap" => (
            "label_overlap",
            "chartlet cannot move the labels apart without changing what they say or where they point. Options: a larger canvas, fewer or shorter labels, or a different orientation. None is applied for you.",
        ),
        _ => (
            "dense_chart",
            "A dense chart cannot be thinned automatically without dropping or aggregating data, which chartlet never does for you. Options: split it into several charts, or filter the data yourself.",
        ),
    };
    Help {
        code,
        path: warning.path.clone(),
        field: warning.path.clone(),
        docs: DOCS,
        note: Some(note.to_owned()),
        remedies: Vec::new(),
    }
}

/// Advice for the warnings of a render, for callers that hold only the specification text: renders
/// it, and answers the help for the warnings it gives.
///
/// # Errors
///
/// Returns the error of the render when the specification is invalid.
pub fn advise_json(spec: &str, options: &RenderOptions) -> Result<Vec<Help>, ChartError> {
    let output = render_json(spec, RenderFormat::Svg, options)?;
    Ok(advise(spec, options, &output.warnings))
}
