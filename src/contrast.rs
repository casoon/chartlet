//! The contrast of the colors a specification sets itself. The bundled palettes are verified;
//! `tokens` are not, so a pair of colors that lie on top of each other in the chart is measured
//! when one of the two comes from `tokens`, and a ratio below WCAG's is reported as
//! `low_contrast` at the token. Only pairs that really overlap are measured: text on the ground
//! of the chart, the marks of the chart on that ground, and in a diagram the outline of a role
//! against its fill and against the ground, and the text on the fill. The roles are those the
//! drawn diagram wears: a box of a kind that is blue is measured, a role nobody draws is not.

use std::collections::BTreeMap;

use crate::{
    error::ChartWarning,
    spec::{ChartSpec, ChartType, Theme},
    theme::bundled_tokens,
};

/// The ratio WCAG 2.2 asks of text (1.4.3) and of graphics (1.4.11).
const TEXT: f64 = 4.5;
const MARK: f64 = 3.0;

/// The roles whose outline and fill a diagram draws.
const ROLES: [&str; 7] = ["blue", "green", "amber", "teal", "violet", "gray", "red"];

/// One pair of tokens that overlap, what they stand for and the ratio they need.
struct Pair {
    front: String,
    back: String,
    what: &'static str,
    needs: f64,
}

fn pair(front: &str, back: &str, what: &'static str, needs: f64) -> Pair {
    Pair {
        front: front.to_owned(),
        back: back.to_owned(),
        what,
        needs,
    }
}

/// The warnings for the colors `spec` sets in `tokens`, for what its type draws on any chart.
pub(crate) fn warnings(spec: &ChartSpec) -> Vec<ChartWarning> {
    check(spec, &pairs(spec))
}

/// The warnings for the role colors of a diagram, from the SVG it was drawn as: only the roles its
/// boxes and lines wear are measured.
pub(crate) fn role_warnings(spec: &ChartSpec, svg: &str) -> Vec<ChartWarning> {
    if spec.tokens.is_empty() {
        return Vec::new();
    }
    let mut pairs = Vec::new();
    for (color, kind) in drawn_roles(svg) {
        let (stroke, fill) = (format!("role-{color}"), format!("role-{color}-fill"));
        match kind {
            // A kind of box: its fill carries the label and its outline stands on the ground.
            RoleUse::Kind => {
                pairs.push(Pair {
                    front: "text".into(),
                    back: fill.clone(),
                    what: "labels on a box",
                    needs: TEXT,
                });
                pairs.push(Pair {
                    front: stroke.clone(),
                    back: fill,
                    what: "outlines",
                    needs: MARK,
                });
                pairs.push(Pair {
                    front: stroke,
                    back: "background".into(),
                    what: "outlines and lines",
                    needs: MARK,
                });
            }
            // A critical, planned or deprecated role recolors the outline and the line only.
            RoleUse::Mark => {
                pairs.push(Pair {
                    front: stroke,
                    back: "background".into(),
                    what: "outlines and lines",
                    needs: MARK,
                });
            }
        }
    }
    check(spec, &pairs)
}

/// How a diagram uses a role color.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RoleUse {
    Kind,
    Mark,
}

/// The role colors the elements of `svg` wear, by their `class` attributes, each once.
fn drawn_roles(svg: &str) -> Vec<(&'static str, RoleUse)> {
    let mut found: Vec<(&'static str, RoleUse)> = Vec::new();
    for (start, _) in svg.match_indices("class=\"") {
        let rest = &svg[start + 7..];
        let Some(end) = rest.find('"') else { continue };
        for class in rest[..end].split(' ') {
            let used = ROLES
                .iter()
                .find(|color| class.strip_prefix("chartlet-role-") == Some(**color))
                .map(|color| (*color, RoleUse::Kind))
                .or(match class {
                    "chartlet-flow-role-critical" => Some(("red", RoleUse::Mark)),
                    "chartlet-flow-role-planned" => Some(("violet", RoleUse::Mark)),
                    "chartlet-flow-role-deprecated" => Some(("gray", RoleUse::Mark)),
                    _ => None,
                });
            if let Some(used) = used
                && !found.contains(&used)
            {
                found.push(used);
            }
        }
    }
    found
}

/// The warnings for `pairs` that involve a color `spec` sets itself.
fn check(spec: &ChartSpec, pairs: &[Pair]) -> Vec<ChartWarning> {
    if spec.tokens.is_empty() {
        return Vec::new();
    }
    let mut palette = bundled_tokens(match spec.theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
        Theme::Blueprint => "blueprint",
    });
    palette.extend(spec.tokens.clone());
    pairs
        .iter()
        .filter_map(|pair| {
            // A pair the specification did not touch is the bundled palette's own.
            let path = [&pair.front, &pair.back]
                .into_iter()
                .find(|name| spec.tokens.contains_key(*name))?;
            let ratio = ratio(&palette, &pair.front, &pair.back)?;
            (ratio < pair.needs).then(|| {
                ChartWarning::new(
                    "low_contrast",
                    format!("/tokens/{path}"),
                    format!(
                        "{} ({}) on {} ({}) has a contrast of {ratio:.1}:1; {} need {:.1}:1",
                        pair.front,
                        palette[&pair.front],
                        pair.back,
                        palette[&pair.back],
                        pair.what,
                        pair.needs
                    ),
                )
            })
        })
        .collect()
}

/// The pairs of colors that lie on top of each other in a chart of this specification.
fn pairs(spec: &ChartSpec) -> Vec<Pair> {
    let mut pairs = vec![
        pair("text", "background", "text and values", TEXT),
        pair("muted", "background", "axis labels and legends", TEXT),
    ];
    match spec.chart_type {
        ChartType::Bar | ChartType::Line => {
            let series = spec.series.len().max(spec.data_groups().len());
            if series > 1 {
                pairs.extend(series_pairs(series));
            } else {
                pairs.push(pair("accent", "background", "bars and lines", MARK));
            }
        }
        ChartType::Time | ChartType::Multiples => {
            let used = spec
                .data_layers()
                .filter(|entry| entry.layer.resolved_color().is_none())
                .map(|entry| spec.palette_index(entry.layer) + 1)
                .max()
                .unwrap_or(0);
            pairs.extend(series_pairs(used));
        }
        _ => {}
    }
    if spec.has_emphasis() {
        pairs.push(pair(
            "accent",
            "background",
            "the outline and the line of emphasis",
            MARK,
        ));
    }
    pairs
}

/// The marks of the first `count` series of the palette against the ground.
fn series_pairs(count: usize) -> impl Iterator<Item = Pair> {
    (1..=count.min(4)).map(|number| Pair {
        front: format!("color-{number}"),
        back: "background".to_owned(),
        what: "series marks",
        needs: MARK,
    })
}

/// The contrast ratio of two tokens of `palette`, `None` for a color that cannot be read.
fn ratio(palette: &BTreeMap<String, String>, front: &str, back: &str) -> Option<f64> {
    let back = color(palette.get(back)?)?;
    let front = color(palette.get(front)?)?;
    let front = blend(front, back);
    let (light, dark) = {
        let (a, b) = (luminance(front), luminance(back));
        if a > b { (a, b) } else { (b, a) }
    };
    Some((light + 0.05) / (dark + 0.05))
}

/// Red, green and blue from 0 to 255, and alpha from 0 to 1.
type Rgba = [f64; 4];

/// A token value: `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb()` or `rgba()`.
fn color(value: &str) -> Option<Rgba> {
    if let Some(hex) = value.strip_prefix('#') {
        let digit = |index: usize| {
            u8::from_str_radix(hex.get(index..=index)?, 16)
                .ok()
                .map(f64::from)
        };
        let pair = |index: usize| {
            u8::from_str_radix(hex.get(index..index + 2)?, 16)
                .ok()
                .map(f64::from)
        };
        return match hex.len() {
            3 => Some([digit(0)? * 17.0, digit(1)? * 17.0, digit(2)? * 17.0, 1.0]),
            6 => Some([pair(0)?, pair(2)?, pair(4)?, 1.0]),
            8 => Some([pair(0)?, pair(2)?, pair(4)?, pair(6)? / 255.0]),
            _ => None,
        };
    }
    let inner = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let numbers: Vec<f64> = inner
        .split([',', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| match part.strip_suffix('%') {
            Some(percent) => percent.parse::<f64>().ok().map(|value| value / 100.0),
            None => part.parse::<f64>().ok(),
        })
        .collect::<Option<_>>()?;
    let channel = |part: &str, number: f64| {
        if part.ends_with('%') {
            number * 255.0
        } else {
            number
        }
    };
    let parts: Vec<&str> = inner
        .split([',', ' '])
        .filter(|part| !part.is_empty())
        .collect();
    match numbers.len() {
        3 | 4 => Some([
            channel(parts[0], numbers[0]),
            channel(parts[1], numbers[1]),
            channel(parts[2], numbers[2]),
            numbers.get(3).copied().unwrap_or(1.0),
        ]),
        _ => None,
    }
}

/// A translucent color as it looks over `back`; the ground itself is taken as opaque.
fn blend(front: Rgba, back: Rgba) -> Rgba {
    let alpha = front[3].clamp(0.0, 1.0);
    [
        front[0] * alpha + back[0] * (1.0 - alpha),
        front[1] * alpha + back[1] * (1.0 - alpha),
        front[2] * alpha + back[2] * (1.0 - alpha),
        1.0,
    ]
}

/// The relative luminance of WCAG 2.2.
fn luminance(color: Rgba) -> f64 {
    let linear = |channel: f64| {
        let channel = (channel / 255.0).clamp(0.0, 1.0);
        if channel <= 0.039_28 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color[0]) + 0.7152 * linear(color[1]) + 0.0722 * linear(color[2])
}

#[cfg(test)]
mod tests {
    use super::{TEXT, blend, bundled_tokens, color, luminance};
    use crate::{RenderFormat, RenderOptions, render_json};

    /// The contrast of two colors.
    fn between(front: [f64; 4], back: [f64; 4]) -> f64 {
        let (a, b) = (luminance(front), luminance(back));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn the_tint_of_an_emphasized_node_keeps_its_text_and_outline_readable_in_every_theme() {
        // The tint is the accent at 14 % over the ground; the outline is the accent itself.
        for theme in ["light", "dark", "blueprint"] {
            let palette = bundled_tokens(theme);
            let token = |name: &str| color(&palette[name]).expect("a color");
            let tint = blend(
                [
                    token("accent")[0],
                    token("accent")[1],
                    token("accent")[2],
                    0.14,
                ],
                token("background"),
            );
            assert!(
                between(token("text"), tint) >= TEXT,
                "{theme}: text on the tint"
            );
            assert!(
                between(token("muted"), tint) >= TEXT,
                "{theme}: muted text on the tint"
            );
            assert!(
                between(token("accent"), tint) >= 3.0,
                "{theme}: the outline against the tint"
            );
            assert!(
                between(token("accent"), token("background")) >= 3.0,
                "{theme}: the outline against the ground"
            );
        }
    }

    fn spec(extra: &str) -> String {
        format!(
            r#"{{"schemaVersion":1,"type":"bar","title":"Revenue",{extra}"data":[{{"label":"A","value":1}},{{"label":"B","value":2}}]}}"#
        )
    }

    fn warnings(extra: &str) -> Vec<(String, String)> {
        let rendered = render_json(&spec(extra), RenderFormat::Svg, &RenderOptions::default())
            .expect("renders");
        rendered
            .warnings
            .into_iter()
            .filter(|warning| warning.code == "low_contrast")
            .map(|warning| (warning.path, warning.message))
            .collect()
    }

    #[test]
    fn a_chart_without_tokens_is_not_measured() {
        assert_eq!(warnings("").len(), 0);
        assert_eq!(warnings(r#""theme":"dark","#).len(), 0);
    }

    #[test]
    fn text_that_is_too_pale_for_its_ground_is_reported_at_the_token() {
        let found = warnings(r##""tokens":{"text":"#aaaaaa"},"##);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "/tokens/text");
        assert!(
            found[0].1.contains("text (#aaaaaa) on background (#fff)"),
            "{}",
            found[0].1
        );
    }

    #[test]
    fn a_background_is_measured_against_the_text_it_carries() {
        let found = warnings(r##""tokens":{"background":"#222222"},"##);
        assert!(
            found.iter().any(|(path, _)| path == "/tokens/background"),
            "{found:?}"
        );
    }

    #[test]
    fn only_the_colors_a_chart_draws_are_measured() {
        // A bar chart with one series draws `accent`; no `color-3`, no role.
        let pale = r##""tokens":{"accent":"#eeeeee","color-3":"#eeeeee","role-red":"#eeeeee"},"##;
        let found = warnings(pale);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "/tokens/accent");

        // Two series draw `color-1` and `color-2`, and no longer `accent`.
        let two = r##""tokens":{"accent":"#eeeeee","color-2":"#eeeeee","color-3":"#eeeeee"},"categories":["A","B"],"series":[{"name":"x","values":[1,2]},{"name":"y","values":[2,3]}],"##;
        let source = format!(r#"{{"schemaVersion":1,"type":"bar","title":"T",{two}"data":[]}}"#);
        let rendered =
            render_json(&source, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
        let paths: Vec<_> = rendered
            .warnings
            .iter()
            .filter(|warning| warning.code == "low_contrast")
            .map(|warning| warning.path.as_str())
            .collect();
        assert_eq!(paths, ["/tokens/color-2"]);
    }

    #[test]
    fn a_diagram_measures_the_outline_and_the_labels_of_its_roles() {
        let flow = std::fs::read_to_string("examples/release-flow.json").expect("example");
        let themed =
            crate::apply_theme(&flow, r##"{"tokens":{"role-green":"#dddddd"}}"##).expect("applies");
        let rendered =
            render_json(&themed, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
        let found: Vec<_> = rendered
            .warnings
            .iter()
            .filter(|warning| warning.code == "low_contrast")
            .collect();
        assert!(
            found
                .iter()
                .all(|warning| warning.path == "/tokens/role-green"),
            "{found:?}"
        );
        assert!(
            found
                .iter()
                .any(|warning| warning.message.contains("outlines")),
            "{found:?}"
        );
    }

    #[test]
    fn a_diagram_measures_only_the_roles_it_draws() {
        let flow = std::fs::read_to_string("examples/release-flow.json").expect("example");
        let plain =
            render_json(&flow, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
        let drawn: Vec<&str> = super::drawn_roles(&plain.content)
            .iter()
            .map(|(color, _)| *color)
            .collect();
        assert!(
            drawn.contains(&"green") && drawn.len() < super::ROLES.len(),
            "{drawn:?}"
        );
        for color in super::ROLES {
            let token = format!("role-{color}");
            let themed =
                crate::apply_theme(&flow, &format!(r##"{{"tokens":{{"{token}":"#eeeeee"}}}}"##))
                    .expect("applies");
            let rendered = render_json(&themed, RenderFormat::Svg, &RenderOptions::default())
                .expect("renders");
            let warned = rendered.warnings.iter().any(|warning| {
                warning.code == "low_contrast" && warning.path == format!("/tokens/{token}")
            });
            assert_eq!(warned, drawn.contains(&color), "{color}");
        }
    }

    #[test]
    fn translucent_and_rgb_colors_are_read() {
        let found = warnings(r#""tokens":{"text":"rgba(0, 0, 0, 0.2)"},"#);
        assert_eq!(found.len(), 1);
        assert_eq!(warnings(r#""tokens":{"text":"rgb(10 10 10)"},"#).len(), 0);
    }

    #[test]
    fn the_brand_theme_of_the_examples_passes() {
        let theme = std::fs::read_to_string("examples/themes/brand.json").expect("brand theme");
        for example in ["monthly-revenue", "release-flow", "revenue-by-channel"] {
            let source =
                std::fs::read_to_string(format!("examples/{example}.json")).expect("example");
            let themed = crate::apply_theme(&source, &theme).expect("applies");
            let rendered = render_json(&themed, RenderFormat::Svg, &RenderOptions::default())
                .expect("renders");
            assert!(
                rendered
                    .warnings
                    .iter()
                    .all(|warning| warning.code != "low_contrast"),
                "{example}: {:?}",
                rendered.warnings
            );
        }
    }
}
