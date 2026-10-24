//! The WebAssembly entry of chartlet for the npm package.
//!
//! A plain C ABI instead of generated bindings: the host copies one UTF-8 JSON request into a
//! buffer from `alloc`, calls `render`, reads `result_len()` bytes of JSON response from the
//! returned pointer, and hands both buffers back to `dealloc`.
//!
//! Request: `{ "spec": <JSON text or object>, "options": { "format", "table", "idPrefix",
//! "variant", "view", "allViews", "perspective", "theme", "strict", "allowWarnings", "manifest",
//! "alternative", "styles", "hooks" } }`, or
//! `{ "stylesheet": true, "types": ["bar", …] }` for the shared stylesheet of
//! `"styles": "external"`, for every chart type when `types` is absent, or
//! `{ "inspect": true, "spec", "disable": ["code", …] }` for the inspections of a model, answered
//! with `{ "ok": true, "findings": [{ "code", "severity", "path", "message", "hint" }] }`. Response: `{ "ok": true, "content", "styleHashes",
//! "warnings" }`, with `"manifest"` when the request asks for it, or
//! `{ "ok": false, "error": { "code", "path", "message" }, "warnings" }`, the same diagnostics
//! the CLI reports with `--diagnostics json`. `styleHashes` are the CSP source expressions of the
//! inline styles in `content`, see [`csp::style_hashes`]; `manifest` is the object the CLI writes
//! with `--manifest`; `alternative`, when asked for, is the chart's text alternative,
//! `{ "description", "table": { "caption", "columns", "rows" } }`; with `allViews` that of the
//! default view, and `alternatives` lists that of every view on the page, in its order, each with
//! its `view` key.

mod csp;

use std::{
    borrow::Cow,
    sync::atomic::{AtomicUsize, Ordering},
};

use chartlet::{
    ChartSpec, ChartType, ChartWarning, INSPECTIONS, Manifest, RenderFormat, RenderOptions, Styles,
    TableMode, TextAlternative, Variant, inspect_json, render_json, stylesheet, stylesheet_for,
    stylesheet_types, text_alternative,
};
use serde_json::{Value, json};

static RESULT_LEN: AtomicUsize = AtomicUsize::new(0);

/// Allocates a buffer of `len` bytes for the host to write a request into.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    Box::into_raw(vec![0u8; len].into_boxed_slice()).cast::<u8>()
}

/// Frees a buffer from `alloc` or `render`.
///
/// # Safety
///
/// `ptr` and `len` must describe a buffer returned by `alloc` or `render` that was not freed yet.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)) });
}

/// Renders the request in the `len` bytes at `ptr` and returns the response; its length is
/// `result_len()`.
///
/// # Safety
///
/// `ptr` must point to `len` initialized bytes, such as a buffer from `alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn render(ptr: *const u8, len: usize) -> *mut u8 {
    let request = unsafe { std::slice::from_raw_parts(ptr, len) };
    let response = respond(request).to_string().into_bytes().into_boxed_slice();
    RESULT_LEN.store(response.len(), Ordering::Relaxed);
    Box::into_raw(response).cast::<u8>()
}

/// The length of the response the last `render` call returned.
#[unsafe(no_mangle)]
pub extern "C" fn result_len() -> usize {
    RESULT_LEN.load(Ordering::Relaxed)
}

fn respond(request: &[u8]) -> Value {
    if let Ok(request) = serde_json::from_slice::<Value>(request)
        && request["stylesheet"] == true
    {
        return shared_stylesheet(&request["types"], request["common"] != false);
    }
    if let Ok(request) = serde_json::from_slice::<Value>(request)
        && request["inspect"] == true
    {
        return inspections(&request["spec"], &request["disable"]);
    }
    let mut warnings = Vec::new();
    let result = run(request, &mut warnings);
    let warnings: Vec<Value> = warnings.iter().map(warning_json).collect();
    match result {
        Ok(Rendered {
            content,
            manifest,
            alternative,
            alternatives,
        }) => {
            let mut response = json!({
                "ok": true,
                "styleHashes": csp::style_hashes(&content),
                "content": content,
                "warnings": warnings,
            });
            if let Some(manifest) = manifest {
                response["manifest"] =
                    serde_json::from_str(&manifest.to_json()).expect("a manifest is valid JSON");
            }
            if let Some(alternative) = alternative {
                response["alternative"] = alternative_json(alternative);
            }
            if !alternatives.is_empty() {
                response["alternatives"] = alternatives
                    .into_iter()
                    .map(|(view, alternative)| {
                        let mut entry = json!({ "view": view });
                        entry.as_object_mut().expect("an object").extend(
                            alternative_json(alternative)
                                .as_object()
                                .cloned()
                                .expect("an object"),
                        );
                        entry
                    })
                    .collect();
            }
            response
        }
        Err(error) => json!({ "ok": false, "error": error, "warnings": warnings }),
    }
}

fn alternative_json(TextAlternative { description, table }: TextAlternative) -> Value {
    json!({
        "description": description,
        "table": {
            "caption": table.caption,
            "columns": table.columns,
            "rows": table.rows,
        },
    })
}

/// A successful render: the content, and the manifest and text alternative when asked for; with
/// all views, the text alternative of each view by its key, in the order of the page.
struct Rendered {
    content: String,
    manifest: Option<Manifest>,
    alternative: Option<TextAlternative>,
    alternatives: Vec<(String, TextAlternative)>,
}

fn run(request: &[u8], warnings: &mut Vec<ChartWarning>) -> Result<Rendered, Value> {
    let request: Value = serde_json::from_slice(request)
        .map_err(|error| failure(&format!("invalid render request: {error}")))?;
    let spec = match &request["spec"] {
        Value::String(spec) => Cow::Borrowed(spec.as_str()),
        spec => Cow::Owned(spec.to_string()),
    };
    let options = &request["options"];
    // A theme is a bundled one by its name or a theme file as an object, applied to the spec.
    let spec = match &options["theme"] {
        Value::Null => spec,
        Value::String(name) => {
            let theme = chartlet::bundled_theme(name)
                .ok_or_else(|| failure("theme must be light, dark, blueprint or a theme object"))?;
            Cow::Owned(chartlet::apply_theme(&spec, theme).map_err(|error| error_json(&error))?)
        }
        theme => Cow::Owned(
            chartlet::apply_theme(&spec, &theme.to_string()).map_err(|error| error_json(&error))?,
        ),
    };
    let format = match options["format"].as_str() {
        Some("svg") => RenderFormat::Svg,
        Some("html") | None => RenderFormat::Html,
        _ => return Err(failure("format must be svg or html")),
    };
    let table_mode = match options["table"].as_str() {
        Some("details") | None => TableMode::Details,
        Some("visible") => TableMode::Visible,
        _ => return Err(failure("table must be details or visible")),
    };
    let variant = match options["variant"].as_str() {
        Some("desktop") | None => Variant::Desktop,
        Some("mobile") => Variant::Mobile,
        Some("print") => Variant::Print,
        Some("social") => Variant::Social,
        _ => return Err(failure("variant must be desktop, mobile, print or social")),
    };
    let styles = match options["styles"].as_str() {
        Some("inline") | None => Styles::Inline,
        Some("external") => Styles::External,
        _ => return Err(failure("styles must be inline or external")),
    };
    let rendered = render_json(
        &spec,
        format,
        &RenderOptions {
            id_prefix: options["idPrefix"].as_str().map(str::to_owned),
            table_mode,
            variant,
            manifest: options["manifest"].as_bool() == Some(true),
            styles,
            hooks: options["hooks"].as_bool() == Some(true),
            view: options["view"].as_str().map(str::to_owned),
            perspective: options["perspective"].as_str().map(str::to_owned),
            all_views: options["allViews"].as_bool() == Some(true),
        },
    )
    .map_err(|error| error_json(&error))?;

    // Strict mode rejects every warning except those whose code is listed in `allowWarnings`.
    let allowed = options["allowWarnings"].as_array();
    let warning_count = rendered
        .warnings
        .iter()
        .filter(|warning| {
            allowed.is_none_or(|codes| !codes.iter().any(|code| code == warning.code))
        })
        .count();
    warnings.extend(rendered.warnings);
    if options["strict"].as_bool() == Some(true) && warning_count > 0 {
        return Err(json!({
            "code": "strict_warnings",
            "path": null,
            "message": format!("strict mode rejected {warning_count} warning(s)"),
        }));
    }
    let alternative = if options["alternative"].as_bool() == Some(true) {
        let spec = ChartSpec::from_json(&spec)
            .and_then(|spec| match options["perspective"].as_str() {
                Some(key) => spec.with_perspective(key),
                None => Ok(spec),
            })
            .and_then(|spec| spec.view(options["view"].as_str()))
            .map_err(|error| error_json(&error))?;
        Some(text_alternative(&spec).map_err(|error| error_json(&error))?)
    } else {
        None
    };
    let alternatives = if alternative.is_some() && options["allViews"].as_bool() == Some(true) {
        view_alternatives(&spec, options["perspective"].as_str())
            .map_err(|error| error_json(&error))?
    } else {
        Vec::new()
    };
    Ok(Rendered {
        content: rendered.content,
        manifest: rendered.manifest,
        alternative,
        alternatives,
    })
}

/// The text alternative of every view of a model, by its key, in the order of the page of all
/// views: the default view first; each with the perspective asked for, if any.
fn view_alternatives(
    spec: &str,
    perspective: Option<&str>,
) -> Result<Vec<(String, TextAlternative)>, chartlet::ChartError> {
    let mut spec = ChartSpec::from_json(spec)?;
    if let Some(key) = perspective {
        spec = spec.with_perspective(key)?;
    }
    let mut keys: Vec<&str> = spec.view_keys();
    let default = spec
        .architecture
        .as_ref()
        .and_then(|architecture| architecture.default_view.as_deref())
        .unwrap_or(keys[0]);
    keys.retain(|key| *key != default);
    keys.insert(0, default);
    keys.into_iter()
        .map(|key| Ok((key.to_owned(), text_alternative(&spec.view(Some(key))?)?)))
        .collect()
}

fn error_json(error: &chartlet::ChartError) -> Value {
    json!({ "code": error.code, "path": error.path, "message": error.message })
}

/// The shared stylesheet, for every chart type or for the names in `types`.
fn shared_stylesheet(types: &Value, common: bool) -> Value {
    let Some(names) = types.as_array() else {
        let stylesheet = if common {
            stylesheet()
        } else {
            stylesheet_types(&ChartType::ALL)
        };
        return json!({ "ok": true, "stylesheet": stylesheet });
    };
    let mut chart_types = Vec::new();
    for name in names {
        match name.as_str().and_then(ChartType::from_name) {
            Some(chart_type) => chart_types.push(chart_type),
            None => {
                return json!({
                    "ok": false,
                    "error": failure(&format!("unknown chart type {name}")),
                    "warnings": [],
                });
            }
        }
    }
    let stylesheet = if common {
        stylesheet_for(&chart_types)
    } else {
        stylesheet_types(&chart_types)
    };
    json!({ "ok": true, "stylesheet": stylesheet })
}

/// The findings of the inspections of `spec`, leaving out those whose codes are in `disable`.
fn inspections(spec: &Value, disable: &Value) -> Value {
    let mut disabled = Vec::new();
    for code in disable.as_array().into_iter().flatten() {
        match code.as_str().filter(|code| INSPECTIONS.contains(code)) {
            Some(code) => disabled.push(code),
            None => {
                return json!({
                    "ok": false,
                    "error": failure(&format!(
                        "unknown inspection {code}; the inspections are {}",
                        INSPECTIONS.join(", ")
                    )),
                });
            }
        }
    }
    let spec = match spec {
        Value::String(spec) => Cow::Borrowed(spec.as_str()),
        spec => Cow::Owned(spec.to_string()),
    };
    match inspect_json(&spec, &disabled) {
        Ok(findings) => {
            let findings: Vec<Value> = findings
                .iter()
                .map(|finding| {
                    json!({
                        "code": finding.code,
                        "severity": finding.severity.name(),
                        "path": finding.path,
                        "message": finding.message,
                        "hint": finding.hint,
                    })
                })
                .collect();
            json!({ "ok": true, "findings": findings })
        }
        Err(error) => json!({ "ok": false, "error": error_json(&error) }),
    }
}

fn failure(message: &str) -> Value {
    json!({ "code": "cli_error", "path": null, "message": message })
}

fn warning_json(warning: &ChartWarning) -> Value {
    json!({ "code": warning.code, "path": warning.path, "message": warning.message })
}
