//! Brand typography: the families a specification names, the width scale of the layout, and
//! measuring with a font file the caller supplies.

use chartlet::{ChartSpec, RenderFormat, RenderOptions, render, render_with_font};

const INTER: &[u8] = include_bytes!("../fonts/Inter-Regular-latin.ttf");

fn bar(typography: &str, title: &str) -> ChartSpec {
    ChartSpec::from_json(&format!(
        r#"{{ "schemaVersion": 1, "type": "bar", "title": "{title}", "width": 600, "height": 300,
            {typography}
            "data": [ {{ "label": "Nord", "value": 4 }}, {{ "label": "Süd", "value": 7 }} ] }}"#
    ))
    .expect("parses")
}

fn svg(spec: &ChartSpec) -> chartlet::RenderOutput {
    render(spec, RenderFormat::Svg, &RenderOptions::default()).expect("renders")
}

#[test]
fn families_come_first_in_the_stack_and_are_disclosed() {
    let spec = bar(
        r#""typography": { "family": "Source Sans 3", "technicalFamily": "Fira Code" },"#,
        "Umsatz",
    );
    let output = svg(&spec);
    assert!(
        output
            .content
            .contains("font-family:\"Source Sans 3\",Inter,")
    );
    assert!(output.content.contains("\"Fira Code\",ui-monospace"));
    let warning = output
        .warnings
        .iter()
        .find(|warning| warning.code == "font_not_embedded")
        .expect("the output does not carry the font");
    assert_eq!(warning.path, "/typography/family");
}

#[test]
fn a_color_only_specification_is_unchanged() {
    let output = svg(&bar("", "Umsatz"));
    assert!(!output.content.contains("font-family:\""));
    assert!(
        output
            .warnings
            .iter()
            .all(|w| w.code != "font_not_embedded")
    );
}

#[test]
fn the_width_scale_reserves_more_room() {
    let title = "abcdefghijklmnopqrstuvwxyzabcdefghijklmn";
    let narrow = svg(&bar("", title));
    let wide = svg(&bar(r#""typography": { "widthScale": 1.5 },"#, title));
    assert!(narrow.warnings.iter().all(|w| w.code != "text_truncated"));
    assert!(wide.warnings.iter().any(|w| w.code == "text_truncated"));
}

#[test]
fn invalid_typography_has_a_code_and_a_path() {
    for (typography, path) in [
        (
            r#""typography": { "family": "A;B" },"#,
            "/typography/family",
        ),
        (
            r#""typography": { "widthScale": 2 },"#,
            "/typography/widthScale",
        ),
    ] {
        let error = render(
            &bar(typography, "x"),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .unwrap_err();
        assert_eq!(
            (error.code, error.path.as_str()),
            ("invalid_typography", path)
        );
    }
}

#[test]
fn a_font_file_measures_and_discloses_missing_glyphs() {
    let spec = bar(r#""typography": { "family": "Brand" },"#, "Größe 語");
    let output =
        render_with_font(&spec, RenderFormat::Svg, &RenderOptions::default(), INTER).expect("ok");
    assert!(
        output
            .warnings
            .iter()
            .all(|w| w.code != "font_not_embedded")
    );
    let missing = output
        .warnings
        .iter()
        .find(|warning| warning.code == "font_glyphs_missing")
        .expect("the font has no glyph for the ideograph");
    assert!(missing.message.contains('語') && !missing.message.contains('ö'));
}

#[test]
fn a_font_file_needs_a_family_and_a_plain_font() {
    let options = RenderOptions::default();
    let without = render_with_font(&bar("", "x"), RenderFormat::Svg, &options, INTER);
    assert_eq!(without.unwrap_err().code, "invalid_font");
    let named = bar(r#""typography": { "family": "Brand" },"#, "x");
    let wrong = render_with_font(&named, RenderFormat::Svg, &options, b"wOFF....");
    assert_eq!(wrong.unwrap_err().code, "invalid_font");
}

#[test]
fn the_same_font_gives_the_same_bytes() {
    let spec = bar(
        r#""typography": { "family": "Brand" },"#,
        "Umsatz nach Region",
    );
    let options = RenderOptions::default();
    let first = render_with_font(&spec, RenderFormat::Svg, &options, INTER).expect("ok");
    let second = render_with_font(&spec, RenderFormat::Svg, &options, INTER).expect("ok");
    assert_eq!(first.content, second.content);
}

#[cfg(feature = "png")]
#[test]
fn png_is_rasterized_with_the_supplied_font() {
    let spec = bar(r#""typography": { "family": "Brand" },"#, "Umsatz");
    let options = chartlet::PngOptions::default();
    let plain = chartlet::render_png(&spec, &options).expect("png");
    let branded = chartlet::render_png_with_font(&spec, &options, INTER).expect("png");
    assert!(branded.png.starts_with(b"\x89PNG"));
    assert!(
        plain
            .warnings
            .iter()
            .any(|warning| warning.code == "font_not_embedded")
    );
    assert!(
        branded
            .warnings
            .iter()
            .all(|warning| warning.code != "font_not_embedded")
    );
}
