//! The color contract, taken from kestrel-chartkit so that chartkit's own colors run through
//! chartlet unchanged: `#rgb`, `#rrggbb`, `#rrggbbaa`, or `var(--name)`, optionally with a hex
//! color as its fallback: `var(--name, #2563eb)`. The fallback is what the print variant draws,
//! since it cannot see the page that defines the variable.
//!
//! Anything else is not rejected: the layer keeps its declared color in the specification and is
//! painted in the neutral tone, with a warning that names the field.

/// Painted when a declared color does not pass the contract.
pub(crate) const NEUTRAL: &str = "#667085";

/// Returns the color unchanged when it passes the contract.
pub(crate) fn sanitize(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.as_bytes()[0] {
        b'#' => hex(trimmed).then_some(trimmed),
        _ => variable(trimmed).then_some(trimmed),
    }
}

/// The CSS value written into the stylesheet for a color that passed [`sanitize`].
///
/// A `var(--name)` without a fallback gets the chart's own text color as its fallback. The host
/// page stays in control of the color, but a page that forgets to define the variable loses
/// neither the series nor its legibility, which a bare `var()` would.
pub(crate) fn emit(value: &str) -> String {
    let trimmed = value.trim();
    let Some((name, fallback)) = variable_parts(trimmed) else {
        return trimmed.to_owned();
    };
    format!("var({name},{})", fallback.unwrap_or("currentColor"))
}

/// Whether a color that passed [`sanitize`] is a variable without a fallback of its own.
pub(crate) fn lacks_fallback(value: &str) -> bool {
    variable_parts(value.trim()).is_some_and(|(_, fallback)| fallback.is_none())
}

/// `#rgb`, `#rrggbb` or `#rrggbbaa`, hex digits only.
fn hex(value: &str) -> bool {
    let digits = &value[1..];
    matches!(digits.len(), 3 | 6 | 8) && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// `var(--name)` or `var(--name, #hex)`.
fn variable(value: &str) -> bool {
    variable_parts(value).is_some()
}

/// The name of a variable color, `--name`, and its hex fallback if it has one. CSS allows any
/// ident after `--` and any value as a fallback, but anything beyond a plain custom property and a
/// hex color is more likely a typo than a deliberate color.
fn variable_parts(value: &str) -> Option<(&str, Option<&str>)> {
    let inner = value.strip_prefix("var(")?.strip_suffix(')')?;
    let (name, fallback) = match inner.split_once(',') {
        Some((name, fallback)) => (name.trim(), Some(fallback.trim())),
        None => (inner, None),
    };
    let ident = name.strip_prefix("--")?;
    let plain = !ident.is_empty()
        && ident
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    let fallback_ok = fallback.is_none_or(|fallback| fallback.starts_with('#') && hex(fallback));
    (plain && fallback_ok).then_some((name, fallback))
}

#[cfg(test)]
mod tests {
    use super::{NEUTRAL, emit, sanitize};

    #[test]
    fn accepts_the_chartkit_whitelist() {
        for color in [
            "#abc",
            "#ABC",
            "#2563eb",
            "#2563eb80",
            "var(--chart-basis)",
            "var(--x-1)",
            "var(--name, #fff)",
            "var(--name,#2563eb)",
        ] {
            assert_eq!(sanitize(color), Some(color), "{color}");
        }
    }

    #[test]
    fn rejects_everything_else() {
        for color in [
            "",
            "red",
            "rgb(1, 2, 3)",
            "#12",
            "#12345",
            "#gggggg",
            "var(--name, red)",
            "var(--name, #12)",
            "var(--name, var(--other))",
            "var(name)",
            "var(--)",
            "url(#gradient)",
            "#2563eb;fill:red",
        ] {
            assert_eq!(sanitize(color), None, "{color}");
        }
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(sanitize("  #fff "), Some("#fff"));
    }

    #[test]
    fn neutral_fallback_is_a_defined_gray() {
        assert_eq!(NEUTRAL, "#667085");
    }

    #[test]
    fn emitted_colors_keep_a_way_back_when_the_page_defines_no_variable() {
        assert_eq!(emit("#2563eb"), "#2563eb");
        assert_eq!(emit("#2563eb80"), "#2563eb80");
        assert_eq!(
            emit("var(--chart-basis)"),
            "var(--chart-basis,currentColor)"
        );
        assert_eq!(
            emit("var(--chart-basis, #0f766e)"),
            "var(--chart-basis,#0f766e)"
        );
    }
}
