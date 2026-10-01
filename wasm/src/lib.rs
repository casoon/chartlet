//! The WebAssembly entry of chartlet for the npm package.
//!
//! A plain C ABI instead of generated bindings: the host copies one UTF-8 JSON request into a
//! buffer from `alloc`, calls `render`, reads `result_len()` bytes of JSON response from the
//! returned pointer, and hands both buffers back to `dealloc`.
//!
//! Request: `{ "spec": <JSON text or object>, "options": { "format", "table", "idPrefix",
//! "variant", "strict", "manifest" } }`. Response: `{ "ok": true, "content", "styleHashes",
//! "warnings" }`, with `"manifest"` when the request asks for it, or
//! `{ "ok": false, "error": { "code", "path", "message" }, "warnings" }`, the same diagnostics
//! the CLI reports with `--diagnostics json`. `styleHashes` are the CSP source expressions of the
//! inline styles in `content`, see [`csp::style_hashes`]; `manifest` is the object the CLI writes
//! with `--manifest`.

mod csp;

use std::{
    borrow::Cow,
    sync::atomic::{AtomicUsize, Ordering},
};

use chartlet::{
    ChartWarning, Manifest, RenderFormat, RenderOptions, TableMode, Variant, render_json,
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
    let mut warnings = Vec::new();
    let result = run(request, &mut warnings);
    let warnings: Vec<Value> = warnings.iter().map(warning_json).collect();
    match result {
        Ok((content, manifest)) => {
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
            response
        }
        Err(error) => json!({ "ok": false, "error": error, "warnings": warnings }),
    }
}

fn run(
    request: &[u8],
    warnings: &mut Vec<ChartWarning>,
) -> Result<(String, Option<Manifest>), Value> {
    let request: Value = serde_json::from_slice(request)
        .map_err(|error| failure(&format!("invalid render request: {error}")))?;
    let spec = match &request["spec"] {
        Value::String(spec) => Cow::Borrowed(spec.as_str()),
        spec => Cow::Owned(spec.to_string()),
    };
    let options = &request["options"];
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
        _ => return Err(failure("variant must be desktop or mobile")),
    };
    let rendered = render_json(
        &spec,
        format,
        &RenderOptions {
            id_prefix: options["idPrefix"].as_str().map(str::to_owned),
            table_mode,
            variant,
            manifest: options["manifest"].as_bool() == Some(true),
        },
    )
    .map_err(|error| json!({ "code": error.code, "path": error.path, "message": error.message }))?;

    let warning_count = rendered.warnings.len();
    warnings.extend(rendered.warnings);
    if options["strict"].as_bool() == Some(true) && warning_count > 0 {
        return Err(json!({
            "code": "strict_warnings",
            "path": null,
            "message": format!("strict mode rejected {warning_count} warning(s)"),
        }));
    }
    Ok((rendered.content, rendered.manifest))
}

fn failure(message: &str) -> Value {
    json!({ "code": "cli_error", "path": null, "message": message })
}

fn warning_json(warning: &ChartWarning) -> Value {
    json!({ "code": warning.code, "path": warning.path, "message": warning.message })
}
