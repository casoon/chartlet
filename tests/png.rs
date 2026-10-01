//! PNG output, with `cargo test --features png`.
#![cfg(feature = "png")]

use std::fmt::Write;

use chartlet::{ChartSpec, PngOptions, Variant, render_png, sha256};

fn hash(bytes: &[u8]) -> String {
    sha256(bytes).iter().fold(String::new(), |mut hex, byte| {
        write!(hex, "{byte:02x}").expect("writing to String cannot fail");
        hex
    })
}

fn spec(json: &str) -> ChartSpec {
    ChartSpec::from_json(json).expect("the example parses")
}

/// The bundled fonts and the absence of system fonts make the PNG a function of the
/// specification: these are the SHA-256 hashes of the reviewed images. A change of chartlet's
/// layout, of resvg or of the fonts changes them; review the new images and update the hashes.
#[test]
fn every_png_matches_its_reviewed_hash() {
    let cases = [
        (
            "temperature-projection social",
            include_str!("../examples/temperature-projection.json"),
            PngOptions {
                variant: Variant::Social,
                scale: 1.0,
            },
            "5e2c1ed9b872a6e5618eb86769800f3b3762393fded31802cfb5e0bbbc1fb4e5",
        ),
        (
            "revenue-vs-forecast print at 2x",
            include_str!("../examples/revenue-vs-forecast.json"),
            PngOptions {
                variant: Variant::Print,
                scale: 2.0,
            },
            "20bd6b632aa8b80baa8385e08380de3b95ff01b1577d2c300fd145276f372f41",
        ),
    ];
    for (name, json, options, expected) in cases {
        let output = render_png(&spec(json), &options).expect("the example renders");
        assert_eq!(
            hash(&output.png),
            expected,
            "{name} drifted from its reviewed PNG"
        );
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
fn the_png_rejects_the_mobile_variant_and_an_out_of_range_scale() {
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
