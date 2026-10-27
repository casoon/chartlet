//! The recipes of `docs/guides/recipes.md`: every specification renders without warnings, with the
//! brand theme, in the outputs the recipe promises.

use chartlet::{RenderFormat, RenderOptions, Variant, apply_theme, render_json};

const BRAND: &str = include_str!("../examples/recipes/brand.theme.json");
const DARK: &str = include_str!("../examples/recipes/brand.theme.dark.json");

const RECIPES: &[(&str, &str)] = &[
    (
        "report-from-csv",
        include_str!("../examples/recipes/report-from-csv/spec.json"),
    ),
    (
        "blog-explanation",
        include_str!("../examples/recipes/blog-explanation/spec.json"),
    ),
    (
        "architecture-views",
        include_str!("../examples/recipes/architecture-views/spec.json"),
    ),
    (
        "release-flow",
        include_str!("../examples/recipes/release-flow/spec.json"),
    ),
    (
        "slide-figure",
        include_str!("../examples/recipes/slide-figure/spec.json"),
    ),
];

fn render(
    spec: &str,
    theme: &str,
    format: RenderFormat,
    options: &RenderOptions,
) -> chartlet::RenderOutput {
    let themed = apply_theme(spec, theme).expect("the theme applies");
    render_json(&themed, format, options).expect("the recipe renders")
}

fn quiet(name: &str, output: &chartlet::RenderOutput) {
    assert!(
        output.warnings.is_empty(),
        "{name} warns: {:?}",
        output.warnings
    );
}

#[test]
fn every_recipe_renders_without_warnings_in_html_with_both_themes() {
    for (name, spec) in RECIPES {
        for theme in [BRAND, DARK] {
            let options = RenderOptions {
                view: None,
                ..RenderOptions::default()
            };
            let output = render(spec, theme, RenderFormat::Html, &options);
            quiet(name, &output);
            assert!(output.content.contains("<table"), "{name} has a data table");
        }
    }
}

#[test]
fn the_brand_theme_is_in_the_output() {
    let (_, spec) = RECIPES[0];
    let svg = render(spec, BRAND, RenderFormat::Svg, &RenderOptions::default());
    assert!(svg.content.contains("#0f766e"));
}

#[test]
fn the_recipes_deliver_the_variants_they_promise() {
    let print = RenderOptions {
        variant: Variant::Print,
        ..RenderOptions::default()
    };
    for name in ["report-from-csv", "slide-figure"] {
        let spec = RECIPES.iter().find(|(n, _)| *n == name).unwrap().1;
        quiet(name, &render(spec, BRAND, RenderFormat::Svg, &print));
    }
    let mobile = RenderOptions {
        variant: Variant::Mobile,
        ..RenderOptions::default()
    };
    for name in ["report-from-csv", "blog-explanation", "release-flow"] {
        let spec = RECIPES.iter().find(|(n, _)| *n == name).unwrap().1;
        quiet(name, &render(spec, BRAND, RenderFormat::Svg, &mobile));
    }
    let views = RenderOptions {
        all_views: true,
        ..RenderOptions::default()
    };
    let spec = RECIPES
        .iter()
        .find(|(n, _)| *n == "architecture-views")
        .unwrap()
        .1;
    let page = render(spec, BRAND, RenderFormat::Html, &views);
    quiet("architecture-views", &page);
    assert!(page.content.contains("one booking"));
}
