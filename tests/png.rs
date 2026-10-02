//! PNG output, with `cargo test --features png`.
#![cfg(feature = "png")]

use chartlet::{ChartSpec, PngOptions, Variant, render_png};

fn spec(json: &str) -> ChartSpec {
    ChartSpec::from_json(json).expect("the example parses")
}

/// How far a rendering may stray from its reviewed image: resvg rasterizes with floating point,
/// and edge pixels can differ by a step or two between CPU architectures. Identical bytes are the
/// rule on one platform; elsewhere at most 0.5 % of the pixels may differ, by at most 8 in any
/// channel.
fn assert_matches_reviewed(name: &str, png: &[u8], reviewed: &[u8]) {
    if png == reviewed {
        return;
    }
    let decode = |bytes: &[u8]| {
        resvg::tiny_skia::Pixmap::decode_png(bytes).expect("a PNG chartlet wrote decodes")
    };
    let (actual, expected) = (decode(png), decode(reviewed));
    assert_eq!(
        (actual.width(), actual.height()),
        (expected.width(), expected.height()),
        "{name} changed its size"
    );
    let mut differing = 0_usize;
    for (a, b) in actual.data().chunks(4).zip(expected.data().chunks(4)) {
        let largest = a
            .iter()
            .zip(b)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        assert!(
            largest <= 8,
            "{name} drifted from its reviewed PNG by {largest}"
        );
        if largest > 0 {
            differing += 1;
        }
    }
    let pixels = actual.data().len() / 4;
    assert!(
        differing * 200 <= pixels,
        "{name}: {differing} of {pixels} pixels differ from the reviewed PNG"
    );
}

/// The bundled fonts and the absence of system fonts make the PNG a function of the
/// specification. A change of chartlet's layout, of resvg or of the fonts changes the images:
/// review the new ones and replace the files in tests/png/.
#[test]
fn every_png_matches_its_reviewed_image() {
    let cases = [
        (
            "temperature-projection social",
            include_str!("../examples/temperature-projection.json"),
            PngOptions {
                variant: Variant::Social,
                scale: 1.0,
            },
            &include_bytes!("png/temperature-projection-social.png")[..],
        ),
        (
            "revenue-vs-forecast print at 2x",
            include_str!("../examples/revenue-vs-forecast.json"),
            PngOptions {
                variant: Variant::Print,
                scale: 2.0,
            },
            &include_bytes!("png/revenue-vs-forecast-print-2x.png")[..],
        ),
    ];
    for (name, json, options, reviewed) in cases {
        let output = render_png(&spec(json), &options).expect("the example renders");
        assert_matches_reviewed(name, &output.png, reviewed);
    }
}

#[test]
fn the_png_has_the_size_of_the_variant_times_the_scale() {
    let json = include_str!("../examples/monthly-revenue.json");
    let size = |options: &PngOptions| {
        let png = render_png(&spec(json), options)
            .expect("the example renders")
            .png;
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        // The IHDR chunk follows the signature: length, type, then width and height.
        let number = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
        (number(16), number(20))
    };
    assert_eq!(size(&PngOptions::default()), (800, 450));
    assert_eq!(
        size(&PngOptions {
            variant: Variant::Social,
            scale: 0.5,
        }),
        (600, 315)
    );
}

#[test]
fn the_png_takes_the_mobile_variant_only_where_there_is_one_and_rejects_a_bad_scale() {
    let monthly = spec(include_str!("../examples/monthly-revenue.json"));
    let mobile = render_png(
        &monthly,
        &PngOptions {
            variant: Variant::Mobile,
            scale: 1.0,
        },
    )
    .unwrap_err();
    assert_eq!(mobile.code, "option_not_supported");
    // A chart with a mobile variant rasterizes its mobile layout at the mobile width.
    let responsive = spec(include_str!("../examples/mobile-revenue.json"));
    let png = render_png(
        &responsive,
        &PngOptions {
            variant: Variant::Mobile,
            scale: 1.0,
        },
    )
    .expect("renders")
    .png;
    let width = u32::from_be_bytes(png[16..20].try_into().expect("an IHDR width"));
    assert_eq!(
        width,
        responsive.mobile.as_ref().expect("a mobile variant").width
    );
    for scale in [0.0, 4.5, f32::NAN] {
        let error = render_png(
            &monthly,
            &PngOptions {
                variant: Variant::Print,
                scale,
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "invalid_scale");
        assert_eq!(error.path, "/render/scale");
    }
}
