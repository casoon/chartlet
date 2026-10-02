//! PNG output, behind the `png` feature: an SVG with a resolved stylesheet, the print or the
//! social variant, rasterized by resvg with the bundled Inter faces and no system fonts, so that
//! the same specification yields the same image on every machine.

use std::sync::Arc;

use resvg::{tiny_skia, usvg};

use crate::error::ChartError;

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
