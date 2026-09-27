//! The color contract, taken from kestrel-chartkit so that chartkit's own colors run through
//! chartlet unchanged: `#rgb`, `#rrggbb`, `#rrggbbaa`, or `var(--name)` without a fallback.
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
/// A `var(--name)` gets the chart's own text color as its fallback. The host page stays in
/// control of the color, but a page that forgets to define the variable loses neither the series
/// nor its legibility, which a bare `var()` would.
pub(crate) fn emit(value: &str) -> String {
    let trimmed = value.trim();
    if !variable(trimmed) {
        return trimmed.to_owned();
    }
    let name = trimmed
        .strip_prefix("var(")
        .and_then(|rest| rest.strip_suffix(')'))
        .expect("a variable color was checked by the contract");
    format!("var({name},currentColor)")
}

/// `#rgb`, `#rrggbb` or `#rrggbbaa`, hex digits only.
fn hex(value: &str) -> bool {
    let digits = &value[1..];
    matches!(digits.len(), 3 | 6 | 8) && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// `var(--name)` without a fallback. CSS allows any ident after `--`, but a name that is not a
/// plain custom property is more likely a typo than a deliberate color.
fn variable(value: &str) -> bool {
    let Some(name) = value
        .strip_prefix("var(")
        .and_then(|rest| rest.strip_suffix(')'))
    else {
        return false;
    };
    let Some(name) = name.strip_prefix("--") else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
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
            "var(--name, #fff)",
            "var(--name, red)",
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
    }
}
