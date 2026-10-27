//! Typography: the font families a chart names, the width scale that makes the layout reserve the
//! room a brand font needs, and measuring text with a font file that is supplied explicitly.
//!
//! Nothing here fetches a font. A family named in the specification is only named: the SVG does
//! not embed it unless the caller passes the font file, which `chartlet render --font` does.

use std::{collections::HashMap, fmt::Write};

use serde::{Deserialize, Serialize};

use crate::{
    error::{ChartError, ChartWarning},
    metrics::{BuiltinMetrics, TextMetrics},
};

/// The fonts of a chart: titles and labels in `family`, technical sublabels in
/// `technical_family`, and the width the layout reserves for text.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypographySpec {
    /// The family of every text, put before Inter and the system fonts in the stack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// The family of technical sublabels such as the technology of a node; defaults to `family`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technical_family: Option<String>,
    /// Measured width of the family relative to the built-in metrics, 0.7 to 1.5: 1.1 reserves 10
    /// percent more room for every text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width_scale: Option<f64>,
}

const MAX_FAMILY: usize = 64;

impl TypographySpec {
    pub(crate) fn validate(&self) -> Result<(), ChartError> {
        for (path, family) in [
            ("/typography/family", &self.family),
            ("/typography/technicalFamily", &self.technical_family),
        ] {
            if let Some(family) = family {
                let valid = !family.trim().is_empty()
                    && family.chars().count() <= MAX_FAMILY
                    && family
                        .chars()
                        .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'));
                if !valid {
                    return Err(ChartError::new(
                        "invalid_typography",
                        path,
                        format!(
                            "a font family is 1 to {MAX_FAMILY} letters, digits, spaces, hyphens, dots or underscores, such as \"Source Sans 3\"; a list of fallbacks is not accepted because the stack is fixed"
                        ),
                    ));
                }
            }
        }
        if let Some(scale) = self.width_scale
            && !(0.7..=1.5).contains(&scale)
        {
            return Err(ChartError::new(
                "invalid_typography",
                "/typography/widthScale",
                "widthScale must be a number from 0.7 to 1.5; measure the family against Inter and enter the ratio, or pass the font file to the CLI with --font",
            ));
        }
        Ok(())
    }

    /// The scale the layout uses, 1 without one.
    pub(crate) fn scale(&self) -> f64 {
        self.width_scale.unwrap_or(1.0)
    }

    /// The rules that put the families in front of the default stack, scoped to the chart `id`.
    pub(crate) fn css(&self, id: &str) -> String {
        let mut css = String::new();
        if let Some(family) = &self.family {
            write!(
                css,
                "svg#{id}.chartlet-root{{font-family:\"{family}\",Inter,ui-sans-serif,system-ui,sans-serif}}"
            )
            .expect("write");
        }
        if let Some(family) = self.technical_family.as_ref().or(self.family.as_ref()) {
            write!(
                css,
                "svg#{id} .chartlet-flow-sublabel,svg#{id} .chartlet-seq-sublabel{{font-family:\"{family}\",ui-monospace,Inter,sans-serif}}"
            )
            .expect("write");
        }
        css
    }

    /// The disclosure that the SVG names a family without carrying it.
    pub(crate) fn disclosure(&self) -> Option<ChartWarning> {
        let family = self.family.as_ref().or(self.technical_family.as_ref())?;
        Some(ChartWarning::new(
            "font_not_embedded",
            "/typography/family",
            format!(
                "the output names the font \"{family}\" but does not carry it: a viewer without it shows Inter or a system font, and the layout reserved room for widthScale {}; render with --font <file.ttf> to measure with the font, embed it in the SVG with --embed-font, and rasterize PNG with it",
                self.scale()
            ),
        ))
    }
}

/// The built-in metrics scaled by the `widthScale` of a specification.
pub(crate) struct ScaledMetrics(pub f64);

impl TextMetrics for ScaledMetrics {
    fn width(&self, text: &str, font_size: f64) -> f64 {
        BuiltinMetrics.width(text, font_size) * self.0
    }
}

/// Text measured with the advance widths of a TrueType or OpenType font that the caller supplied.
/// Kerning and shaping are not applied, and only the basic multilingual plane is mapped: scripts
/// that need shaping (Arabic, Indic scripts) are outside what chartlet lays out.
#[derive(Debug, Clone)]
pub struct FontMetrics {
    units_per_em: f64,
    advances: Vec<u16>,
    glyphs: HashMap<u32, u16>,
    identity: String,
}

impl FontMetrics {
    /// Reads the font.
    ///
    /// # Errors
    ///
    /// Returns `invalid_font` for anything but a TrueType or OpenType file with a Windows Unicode
    /// (format 4) character map, such as a WOFF or a font collection.
    pub fn from_ttf(font: &[u8]) -> Result<Self, ChartError> {
        let fail = |message: &str| ChartError::new("invalid_font", "/render/font", message);
        let magic = font.get(..4).unwrap_or_default();
        if !matches!(magic, [0, 1, 0, 0] | b"OTTO" | b"true") {
            return Err(fail(
                "the font must be a plain TrueType or OpenType file (.ttf or .otf); WOFF and font collections are not read",
            ));
        }
        let unreadable = || fail("the font file is truncated or malformed");
        let table = |tag: &[u8; 4]| -> Option<usize> {
            let count = usize::from(u16_at(font, 4)?);
            (0..count)
                .map(|index| 12 + 16 * index)
                .find(|record| font.get(*record..*record + 4) == Some(tag))
                .and_then(|record| u32_at(font, record + 8))
                .map(|offset| offset as usize)
        };
        let head = table(b"head").ok_or_else(unreadable)?;
        let units_per_em = u16_at(font, head + 18).ok_or_else(unreadable)?;
        if units_per_em == 0 {
            return Err(unreadable());
        }
        let hhea = table(b"hhea").ok_or_else(unreadable)?;
        let metric_count = usize::from(u16_at(font, hhea + 34).ok_or_else(unreadable)?);
        let hmtx = table(b"hmtx").ok_or_else(unreadable)?;
        let advances = (0..metric_count)
            .map(|index| u16_at(font, hmtx + 4 * index))
            .collect::<Option<Vec<u16>>>()
            .ok_or_else(unreadable)?;
        let cmap = table(b"cmap").ok_or_else(unreadable)?;
        let glyphs = cmap_glyphs(font, cmap).ok_or_else(|| {
            fail("the font has no Windows Unicode (3, 1) character map in format 4")
        })?;
        let hash = crate::sha256(font);
        Ok(Self {
            units_per_em: f64::from(units_per_em),
            advances,
            glyphs,
            identity: format!("font-sha256:{}", &crate::sha256::hex(&hash)[..16]),
        })
    }

    /// Names the font by the first 16 hexadecimal digits of the SHA-256 of its file.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Whether the font has a glyph for `character`.
    #[must_use]
    pub fn covers(&self, character: char) -> bool {
        character.is_whitespace()
            || self
                .glyphs
                .get(&u32::from(character))
                .is_some_and(|glyph| *glyph != 0)
    }

    fn advance(&self, character: char) -> f64 {
        let glyph = usize::from(self.glyphs.get(&u32::from(character)).copied().unwrap_or(0));
        let advance = self
            .advances
            .get(glyph)
            .or_else(|| self.advances.last())
            .copied()
            .unwrap_or(0);
        f64::from(advance) / self.units_per_em
    }

    /// A warning naming the characters of the drawn text that the font has no glyph for.
    pub(crate) fn missing_glyphs(&self, svg: &str) -> Option<ChartWarning> {
        let mut missing: Vec<char> = Vec::new();
        for text in svg.split("<text").skip(1) {
            let content = text
                .split_once('>')
                .and_then(|(_, rest)| rest.split_once("</text>"))
                .map_or("", |(content, _)| content);
            for character in content.chars() {
                if !self.covers(character) && !missing.contains(&character) {
                    missing.push(character);
                }
            }
        }
        (!missing.is_empty()).then(|| {
            ChartWarning::new(
                "font_glyphs_missing",
                "/typography/family",
                format!(
                    "the supplied font has no glyph for {}; a viewer draws them in a fallback font whose width the layout did not measure",
                    missing.iter().collect::<String>()
                ),
            )
        })
    }
}

impl TextMetrics for FontMetrics {
    fn width(&self, text: &str, font_size: f64) -> f64 {
        text.chars().map(|c| self.advance(c)).sum::<f64>() * font_size
    }
}

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

/// The glyph of every code point of the format 4 Windows Unicode character map at `cmap`.
fn cmap_glyphs(font: &[u8], cmap: usize) -> Option<HashMap<u32, u16>> {
    let subtables = usize::from(u16_at(font, cmap + 2)?);
    let table = (0..subtables)
        .map(|index| cmap + 4 + 8 * index)
        .find(|record| u16_at(font, *record) == Some(3) && u16_at(font, *record + 2) == Some(1))
        .and_then(|record| u32_at(font, record + 4))
        .map(|offset| cmap + offset as usize)?;
    if u16_at(font, table)? != 4 {
        return None;
    }
    let segments = usize::from(u16_at(font, table + 6)?) / 2;
    let ends = table + 14;
    let starts = ends + 2 * segments + 2;
    let deltas = starts + 2 * segments;
    let offsets = deltas + 2 * segments;
    let mut glyphs = HashMap::new();
    for segment in 0..segments {
        let end = u16_at(font, ends + 2 * segment)?;
        let start = u16_at(font, starts + 2 * segment)?;
        let delta = u16_at(font, deltas + 2 * segment)?;
        let offset_at = offsets + 2 * segment;
        let offset = usize::from(u16_at(font, offset_at)?);
        for code in start..=end {
            if code == 0xFFFF {
                continue;
            }
            let glyph = if offset == 0 {
                code.wrapping_add(delta)
            } else {
                let at = offset_at + offset + 2 * usize::from(code - start);
                match u16_at(font, at)? {
                    0 => 0,
                    glyph => glyph.wrapping_add(delta),
                }
            };
            if glyph != 0 {
                glyphs.insert(u32::from(code), glyph);
            }
        }
    }
    Some(glyphs)
}

/// The SVG with the font carried as a base64 `@font-face` for `family`, placed behind the opening
/// tag of the root. The font is embedded as given; whether its license allows that is the
/// caller's to check.
#[must_use]
pub fn embed_font(svg: &str, family: &str, font: &[u8]) -> String {
    let Some(open) = svg
        .find("<svg")
        .and_then(|start| svg[start..].find('>').map(|end| start + end + 1))
    else {
        return svg.to_owned();
    };
    let mut output = String::with_capacity(svg.len() + font.len() * 4 / 3 + 128);
    output.push_str(&svg[..open]);
    write!(
        output,
        "<style>@font-face{{font-family:\"{family}\";src:url(data:font/ttf;base64,{}) format(\"truetype\")}}</style>",
        base64(font)
    )
    .expect("write");
    output.push_str(&svg[open..]);
    output
}

fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let bytes = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let value = (u32::from(bytes[0]) << 16) | (u32::from(bytes[1]) << 8) | u32::from(bytes[2]);
        for position in 0..4 {
            if position <= chunk.len() {
                output.push(ALPHABET[((value >> (18 - 6 * position)) & 63) as usize] as char);
            } else {
                output.push('=');
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const INTER: &[u8] = include_bytes!("../fonts/Inter-Regular-latin.ttf");

    #[test]
    fn base64_matches_the_known_encodings() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn measures_with_the_advances_of_the_font() {
        let font = FontMetrics::from_ttf(INTER).expect("Inter is readable");
        let narrow = font.width("iii", 12.0);
        let wide = font.width("WWW", 12.0);
        assert!(narrow > 0.0 && wide > narrow * 2.0);
        assert!(font.covers('ä') && !font.covers('語'));
        assert!(font.identity().starts_with("font-sha256:"));
    }

    #[test]
    fn refuses_what_is_not_a_plain_font() {
        assert_eq!(
            FontMetrics::from_ttf(b"wOFF0000").unwrap_err().code,
            "invalid_font"
        );
        assert_eq!(
            FontMetrics::from_ttf(&INTER[..200]).unwrap_err().code,
            "invalid_font"
        );
    }

    #[test]
    fn names_the_glyphs_the_font_lacks() {
        let font = FontMetrics::from_ttf(INTER).expect("readable");
        let warning = font
            .missing_glyphs("<svg><text x=\"1\">Größe 語</text></svg>")
            .expect("one character is missing");
        assert_eq!(warning.code, "font_glyphs_missing");
        assert!(warning.message.contains('語') && !warning.message.contains('ö'));
    }

    #[test]
    fn rejects_invalid_typography_with_a_path() {
        for (json, path) in [
            (r#"{"family":"Brand;}"}"#, "/typography/family"),
            (r#"{"technicalFamily":""}"#, "/typography/technicalFamily"),
            (r#"{"widthScale":3}"#, "/typography/widthScale"),
        ] {
            let spec: TypographySpec = serde_json::from_str(json).expect("parses");
            let error = spec.validate().unwrap_err();
            assert_eq!(
                (error.code, error.path.as_str()),
                ("invalid_typography", path)
            );
        }
    }

    #[test]
    fn embeds_the_font_behind_the_root_tag() {
        let svg = embed_font("<svg a=\"b\"><g/></svg>", "Brand", b"foo");
        assert!(svg.starts_with("<svg a=\"b\"><style>@font-face{font-family:\"Brand\""));
        assert!(svg.contains("Zm9v") && svg.ends_with("<g/></svg>"));
    }
}
