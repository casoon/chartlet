//! PNG output, behind the `png` feature: an SVG with a resolved stylesheet, the print or the
//! social variant, rasterized by resvg with the bundled Inter faces and no system fonts, so that
//! the same specification yields the same image on every machine.

use std::sync::Arc;

use resvg::{tiny_skia, usvg};

use crate::error::{ChartError, ChartWarning};

/// Inter, regular and semibold, in the Latin and Latin Extended subsets; see `fonts/README.md`.
const FONTS: [&[u8]; 4] = [
    include_bytes!("../fonts/Inter-Regular-latin.ttf"),
    include_bytes!("../fonts/Inter-SemiBold-latin.ttf"),
    include_bytes!("../fonts/Inter-Regular-latin-ext.ttf"),
    include_bytes!("../fonts/Inter-SemiBold-latin-ext.ttf"),
];

/// The largest scale factor: a 2400 × 1600 chart becomes 9600 × 6400 pixels.
pub(crate) const MAX_SCALE: f32 = 4.0;

/// Rasterizes `svg` at `scale` times its size in pixels.
pub(crate) fn rasterize(svg: &str, scale: f32) -> Result<Vec<u8>, ChartError> {
    let mut fonts = usvg::fontdb::Database::new();
    for font in FONTS {
        fonts.load_font_data(font.to_vec());
    }
    // The semibold subsets name their family "Inter SemiBold" and carry no typographic family,
    // so a lookup for Inter at weight 600 would fall back to the regular face. They join the
    // Inter family here.
    let semibold: Vec<usvg::fontdb::FaceInfo> = fonts
        .faces()
        .filter(|face| {
            face.families
                .iter()
                .any(|(name, _)| name == "Inter SemiBold")
        })
        .cloned()
        .collect();
    for mut face in semibold {
        fonts.remove_face(face.id);
        face.families = vec![(
            "Inter".to_owned(),
            usvg::fontdb::Language::English_UnitedStates,
        )];
        fonts.push_face_info(face);
    }
    fonts.set_sans_serif_family("Inter");
    let options = usvg::Options {
        font_family: "Inter".to_owned(),
        fontdb: Arc::new(fonts),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(svg, &options).map_err(|error| {
        ChartError::new(
            "png_failed",
            "/render/format",
            format!("could not read the chart SVG: {error}"),
        )
    })?;
    let size = tree
        .size()
        .to_int_size()
        .scale_by(scale)
        .expect("a validated scale keeps the chart larger than a pixel");
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())
        .expect("a validated scale keeps the image within memory limits");
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|error| {
        ChartError::new(
            "png_failed",
            "/render/format",
            format!("could not encode the PNG: {error}"),
        )
    })
}

/// The characters the bundled fonts have a glyph for, read from the format 4 character maps of
/// the regular faces; the semibold faces cover the same characters.
fn covered() -> &'static [u32] {
    static COVERED: std::sync::OnceLock<Vec<u32>> = std::sync::OnceLock::new();
    COVERED.get_or_init(|| {
        let mut codes: Vec<u32> = [FONTS[0], FONTS[2]]
            .into_iter()
            .flat_map(cmap_codes)
            .collect();
        codes.sort_unstable();
        codes.dedup();
        codes
    })
}

fn read_u16(data: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([data[at], data[at + 1]])
}

fn read_u32(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

/// The code points a TrueType font's Windows Unicode (3, 1) character map, format 4, gives a
/// glyph other than the missing glyph. The bundled files are known to carry one.
fn cmap_codes(font: &[u8]) -> Vec<u32> {
    let tables = usize::from(read_u16(font, 4));
    let cmap = (0..tables)
        .map(|index| 12 + 16 * index)
        .find(|record| &font[*record..*record + 4] == b"cmap")
        .map(|record| read_u32(font, record + 8) as usize)
        .expect("the bundled fonts have a character map");
    let subtables = usize::from(read_u16(font, cmap + 2));
    let table = (0..subtables)
        .map(|index| cmap + 4 + 8 * index)
        .find(|record| read_u16(font, *record) == 3 && read_u16(font, *record + 2) == 1)
        .map(|record| cmap + read_u32(font, record + 4) as usize)
        .expect("the bundled fonts have a Windows Unicode character map");
    let segments = usize::from(read_u16(font, table + 6)) / 2;
    let ends = table + 14;
    let starts = ends + 2 * segments + 2;
    let deltas = starts + 2 * segments;
    let offsets = deltas + 2 * segments;
    let mut codes = Vec::new();
    for segment in 0..segments {
        let end = read_u16(font, ends + 2 * segment);
        let start = read_u16(font, starts + 2 * segment);
        let delta = read_u16(font, deltas + 2 * segment);
        let offset_at = offsets + 2 * segment;
        let offset = usize::from(read_u16(font, offset_at));
        for code in start..=end {
            if code == 0xFFFF {
                continue;
            }
            let glyph = if offset == 0 {
                code.wrapping_add(delta)
            } else {
                let at = offset_at + offset + 2 * usize::from(code - start);
                match read_u16(font, at) {
                    0 => 0,
                    glyph => glyph.wrapping_add(delta),
                }
            };
            if glyph != 0 {
                codes.push(u32::from(code));
            }
        }
    }
    codes
}

/// A warning naming the characters of the drawn text, such as a subscript two or a Greek
/// letter, that the bundled fonts have no glyph for: the PNG draws an empty box for each.
pub(crate) fn missing_glyphs(svg: &str) -> Option<ChartWarning> {
    let mut missing: Vec<char> = Vec::new();
    for text in svg.split("<text").skip(1) {
        let content = text
            .split_once('>')
            .and_then(|(_, rest)| rest.split_once("</text>"))
            .map_or("", |(content, _)| content);
        for character in content.chars() {
            let covered =
                character.is_whitespace() || covered().binary_search(&u32::from(character)).is_ok();
            if !covered && !missing.contains(&character) {
                missing.push(character);
            }
        }
    }
    (!missing.is_empty()).then(|| {
        ChartWarning::new(
            "glyph_missing",
            "/render/format",
            format!(
                "the PNG font has no glyph for {}; these characters are drawn as boxes, while the SVG and HTML output show them",
                missing.iter().collect::<String>()
            ),
        )
    })
}
