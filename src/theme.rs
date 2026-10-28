//! Tokens: the colors of a chart's palette, by the name of the custom property that carries them
//! without its `--chartlet-` prefix. One list of names serves every way to restyle a chart: the
//! `tokens` of a specification, a theme file applied to many of them, and the CSS of a host page
//! that sets the properties itself.

use std::{collections::BTreeMap, fmt::Write};

use serde_json::Value;

use crate::error::ChartError;

/// The names that are not numbered.
const NAMES: [&str; 22] = [
    "text",
    "muted",
    "grid",
    "zero",
    "accent",
    "background",
    "node",
    "lane",
    "lane-head",
    "shadow",
    "rise",
    "fall",
    "role-blue",
    "role-blue-fill",
    "role-green",
    "role-green-fill",
    "role-amber",
    "role-amber-fill",
    "role-teal",
    "role-teal-fill",
    "role-violet",
    "role-violet-fill",
];

/// The names that end in a number: the prefix and the numbers it takes.
const NUMBERED: [(&str, std::ops::RangeInclusive<u8>); 6] = [
    ("color-", 1..=4),
    ("heat-", 0..=5),
    ("diverging-", 0..=16),
    ("net-", 5..=8),
    ("parl-", 5..=8),
    ("part-", 5..=8),
];

/// The two roles left over by [`NAMES`], which would not fit its length.
const MORE_NAMES: [&str; 4] = ["role-gray", "role-gray-fill", "role-red", "role-red-fill"];

/// The themes that ship with chartlet, as the theme files they are: the same mechanism as a theme
/// file of your own, which can start from a copy of one of them.
const BUNDLED: [(&str, &str); 5] = [
    ("light", include_str!("../themes/light.json")),
    ("dark", include_str!("../themes/dark.json")),
    ("blueprint", include_str!("../themes/blueprint.json")),
    ("classic", include_str!("../themes/classic.json")),
    ("editorial", include_str!("../themes/editorial.json")),
];

/// The text of the bundled theme file called `name`: `light`, `dark` or `blueprint`, which are
/// palettes, or `classic` or `editorial`, which are looks of diagrams.
#[must_use]
pub fn bundled_theme(name: &str) -> Option<&'static str> {
    BUNDLED
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, theme)| *theme)
}

/// Whether `name` is a token.
pub(crate) fn is_token(name: &str) -> bool {
    NAMES.contains(&name)
        || MORE_NAMES.contains(&name)
        || NUMBERED.iter().any(|(prefix, range)| {
            name.strip_prefix(prefix)
                .and_then(|number| number.parse::<u8>().ok())
                .is_some_and(|number| {
                    range.contains(&number) && name.len() == prefix.len() + number.to_string().len()
                })
        })
}

/// Every token, in a fixed order; the list the documentation and the schema state.
#[cfg(test)]
pub(crate) fn all_tokens() -> Vec<String> {
    let mut names: Vec<String> = NAMES
        .iter()
        .chain(&MORE_NAMES)
        .map(|name| (*name).to_owned())
        .collect();
    for (prefix, range) in NUMBERED {
        names.extend(range.map(|number| format!("{prefix}{number}")));
    }
    names
}

/// The custom properties of `tokens`, as the declarations of one rule.
pub(crate) fn declarations(tokens: &BTreeMap<String, String>) -> String {
    let mut output = String::new();
    for (name, value) in tokens {
        write!(output, "--chartlet-{name}:{value};").expect("writing to String cannot fail");
    }
    output.pop();
    output
}

/// The error for a token that is not one, or whose value is not a hex color.
pub(crate) fn validate(tokens: &BTreeMap<String, String>) -> Result<(), ChartError> {
    for (name, value) in tokens {
        if !is_token(name) {
            return Err(ChartError::new(
                "invalid_token",
                format!("/tokens/{name}"),
                format!(
                    "`{name}` is not a token; the tokens are the names of the --chartlet-* custom properties without the prefix, such as accent, text or color-1"
                ),
            ));
        }
        if !is_color(value) {
            return Err(ChartError::new(
                "invalid_token",
                format!("/tokens/{name}"),
                "a token is a hex color (#rgb, #rrggbb or #rrggbbaa) or an rgb()/rgba() color",
            ));
        }
    }
    Ok(())
}

/// A hex color, or `rgb()`/`rgba()` of numbers only: nothing that could end a declaration, and
/// nothing the print variant would have to resolve.
fn is_color(value: &str) -> bool {
    if crate::color::sanitize(value).is_some_and(|color| color.starts_with('#')) {
        return true;
    }
    value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|rest| rest.strip_suffix(')'))
        .is_some_and(|numbers| {
            !numbers.is_empty()
                && numbers
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || matches!(byte, b'.' | b',' | b' ' | b'%'))
        })
}

/// The tokens of a bundled theme by name; empty for a name that is none.
pub(crate) fn bundled_tokens(name: &str) -> BTreeMap<String, String> {
    bundled_theme(name)
        .and_then(|theme| serde_json::from_str::<Value>(theme).ok())
        .and_then(|theme| {
            theme["tokens"].as_object().map(|tokens| {
                tokens
                    .iter()
                    .filter_map(|(name, token)| Some((name.clone(), token.as_str()?.to_owned())))
                    .collect()
            })
        })
        .unwrap_or_default()
}

/// Applies a theme file to a specification, both as JSON text, and returns the specification.
/// A theme file holds a `theme`, a `look` and `tokens`: the theme and the look are used where the
/// specification names none, and a token of the specification replaces the token of the file with
/// the same name.
///
/// # Errors
///
/// Returns `invalid_theme` when the theme file is not a JSON object of these fields, and
/// `invalid_json` when the specification is not an object.
pub fn apply_theme(spec: &str, theme: &str) -> Result<String, ChartError> {
    let mut spec: Value = serde_json::from_str(spec)
        .map_err(|error| ChartError::new("invalid_json", "/", error.to_string()))?;
    let theme: Value = serde_json::from_str(theme)
        .map_err(|error| ChartError::new("invalid_theme", "/", error.to_string()))?;
    let Value::Object(theme) = theme else {
        return Err(ChartError::new(
            "invalid_theme",
            "/",
            "a theme file is a JSON object with `theme`, `look` and `tokens`",
        ));
    };
    let Value::Object(spec) = &mut spec else {
        return Err(ChartError::new(
            "invalid_json",
            "/",
            "a specification is a JSON object",
        ));
    };
    let mut tokens = serde_json::Map::new();
    for (key, value) in theme {
        match key.as_str() {
            "$schema" => {}
            "theme" | "look" => {
                spec.entry(key).or_insert(value);
            }
            "tokens" => {
                let Value::Object(own) = value else {
                    return Err(ChartError::new(
                        "invalid_theme",
                        "/tokens",
                        "`tokens` is an object of token names and colors",
                    ));
                };
                tokens = own;
            }
            _ => {
                return Err(ChartError::new(
                    "invalid_theme",
                    format!("/{key}"),
                    "a theme file has `theme`, `look` and `tokens` only",
                ));
            }
        }
    }
    // What the theme the chart ends up with already gives a token is not repeated in the chart.
    let base = spec.get("theme").and_then(Value::as_str).unwrap_or("light");
    let known = bundled_tokens(base);
    let kept = tokens
        .into_iter()
        .filter(|(name, token)| token.as_str() != known.get(name).map(String::as_str));
    // A `tokens` that is no object is reported when the specification is parsed.
    if let Value::Object(own) = spec
        .entry("tokens")
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
    {
        for (name, token) in kept {
            own.entry(name).or_insert(token);
        }
        if own.is_empty() {
            spec.remove("tokens");
        }
    }
    Ok(Value::Object(std::mem::take(spec)).to_string())
}

#[cfg(test)]
mod tests {
    use super::{all_tokens, apply_theme, bundled_theme, is_token};
    use crate::{RenderFormat, RenderOptions, Variant, render_json};

    const BAR: &str = r#"{"schemaVersion":1,"type":"bar","title":"Revenue","data":[{"label":"A","value":1},{"label":"B","value":2}]}"#;

    /// The custom properties the stylesheet gives a selector, by token name.
    fn declared(selector: &str) -> std::collections::BTreeMap<String, String> {
        let stylesheet = crate::stylesheet();
        let mut tokens = std::collections::BTreeMap::new();
        for rule in stylesheet.split_inclusive('}') {
            let Some((rule_selector, body)) = rule.split_once('{') else {
                continue;
            };
            if rule_selector != selector {
                continue;
            }
            for declaration in body.trim_end_matches('}').split(';') {
                if let Some((name, value)) = declaration.split_once(':')
                    && let Some(token) = name.strip_prefix("--chartlet-")
                {
                    tokens.insert(token.to_owned(), value.to_owned());
                }
            }
        }
        tokens
    }

    fn bundled(name: &str) -> std::collections::BTreeMap<String, String> {
        let theme: serde_json::Value =
            serde_json::from_str(bundled_theme(name).expect("bundled")).expect("json");
        assert_eq!(theme["theme"], name);
        theme["tokens"]
            .as_object()
            .expect("tokens")
            .iter()
            .map(|(name, value)| (name.clone(), value.as_str().expect("string").to_owned()))
            .collect()
    }

    #[test]
    fn the_bundled_themes_are_the_palettes_of_the_stylesheet() {
        let light = declared(".chartlet-root");
        let mut dark = light.clone();
        dark.extend(declared(".chartlet-root.chartlet-theme-dark"));
        let mut blueprint = dark.clone();
        blueprint.extend(declared(".chartlet-root.chartlet-blueprint"));
        assert_eq!(bundled("light"), light);
        assert_eq!(bundled("dark"), dark);
        assert_eq!(bundled("blueprint"), blueprint);
        assert_eq!(light.len(), all_tokens().len());
    }

    #[test]
    fn a_bundled_theme_adds_nothing_to_its_own_chart() {
        let themed = apply_theme(BAR, bundled_theme("dark").expect("bundled")).expect("applies");
        let themed: serde_json::Value = serde_json::from_str(&themed).expect("json");
        assert_eq!(themed["theme"], "dark");
        assert!(themed.get("tokens").is_none());
    }

    #[test]
    fn tokens_reach_the_chart_and_the_print_variant() {
        let spec = BAR.replace(
            r#""data""#,
            r##""tokens":{"accent":"#ff5a5f","background":"#101010"},"data""##,
        );
        let svg =
            render_json(&spec, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
        assert!(
            svg.content
                .contains("{--chartlet-accent:#ff5a5f;--chartlet-background:#101010}")
        );
        assert!(svg.content.contains(r#"<rect class="chartlet-background""#));
        let print = render_json(
            &spec,
            RenderFormat::Svg,
            &RenderOptions {
                variant: Variant::Print,
                ..RenderOptions::default()
            },
        )
        .expect("renders");
        assert!(print.content.contains("#ff5a5f"));
        assert!(!print.content.contains("--chartlet"));
    }

    #[test]
    fn a_token_that_is_none_is_refused() {
        for (tokens, path) in [
            (r##"{"nope":"#fff"}"##, "/tokens/nope"),
            (r#"{"accent":"red"}"#, "/tokens/accent"),
            (r#"{"accent":"url(x);}"}"#, "/tokens/accent"),
        ] {
            let spec = BAR.replace(r#""data""#, &format!(r#""tokens":{tokens},"data""#));
            let error = render_json(&spec, RenderFormat::Svg, &RenderOptions::default())
                .expect_err("refused");
            assert_eq!((error.code, error.path.as_str()), ("invalid_token", path));
        }
    }

    #[test]
    fn knows_every_custom_property_the_stylesheets_define() {
        let stylesheet = crate::stylesheet();
        let defined: Vec<&str> = stylesheet
            .match_indices("--chartlet-")
            .filter_map(|(start, _)| {
                let rest = &stylesheet[start + 11..];
                let end = rest.find([':', ')', ',', ';'])?;
                (rest.as_bytes()[end] == b':').then_some(&rest[..end])
            })
            .collect();
        assert!(defined.len() > 50);
        for name in defined {
            assert!(is_token(name), "{name} is not a token");
        }
        for name in all_tokens() {
            assert!(is_token(&name));
        }
    }

    #[test]
    fn rejects_names_that_only_look_like_tokens() {
        for name in [
            "color-0", "color-5", "heat-6", "color-01", "colour-1", "net-4", "",
        ] {
            assert!(!is_token(name), "{name}");
        }
    }

    #[test]
    fn a_spec_keeps_its_own_theme_and_tokens() {
        let merged = apply_theme(
            r##"{"theme":"dark","tokens":{"accent":"#111"}}"##,
            r##"{"theme":"light","tokens":{"accent":"#222","text":"#333"}}"##,
        )
        .expect("merges");
        let merged: serde_json::Value = serde_json::from_str(&merged).expect("json");
        assert_eq!(merged["theme"], "dark");
        assert_eq!(merged["tokens"]["accent"], "#111");
        assert_eq!(merged["tokens"]["text"], "#333");
    }

    #[test]
    fn a_theme_file_sets_the_look_where_the_specification_names_none() {
        let themed =
            apply_theme(BAR, bundled_theme("editorial").expect("bundled")).expect("applies");
        let themed: serde_json::Value = serde_json::from_str(&themed).expect("json");
        assert_eq!(themed["look"], "editorial");
        let own = apply_theme(
            r#"{"look":"classic"}"#,
            bundled_theme("editorial").expect("bundled"),
        )
        .expect("applies");
        assert!(own.contains("classic"));
    }

    #[test]
    fn refuses_a_theme_file_with_other_fields() {
        let error = apply_theme("{}", r#"{"title":"x"}"#).expect_err("refused");
        assert_eq!(error.code, "invalid_theme");
        assert_eq!(error.path, "/title");
    }
}
