use chartlet::{RenderFormat, RenderOptions, Variant, render_json};

/// Every example as name, specification and reviewed SVG.
const EXAMPLES: [(&str, &str, &str); 38] = [
    (
        "monthly-revenue",
        include_str!("../examples/monthly-revenue.json"),
        include_str!("../examples/monthly-revenue.svg"),
    ),
    (
        "quarterly-change",
        include_str!("../examples/quarterly-change.json"),
        include_str!("../examples/quarterly-change.svg"),
    ),
    (
        "budget-vs-actual",
        include_str!("../examples/budget-vs-actual.json"),
        include_str!("../examples/budget-vs-actual.svg"),
    ),
    (
        "monthly-trend",
        include_str!("../examples/monthly-trend.json"),
        include_str!("../examples/monthly-trend.svg"),
    ),
    (
        "operating-costs",
        include_str!("../examples/operating-costs.json"),
        include_str!("../examples/operating-costs.svg"),
    ),
    (
        "support-volume",
        include_str!("../examples/support-volume.json"),
        include_str!("../examples/support-volume.svg"),
    ),
    (
        "revenue-by-channel",
        include_str!("../examples/revenue-by-channel.json"),
        include_str!("../examples/revenue-by-channel.svg"),
    ),
    (
        "signup-conversion",
        include_str!("../examples/signup-conversion.json"),
        include_str!("../examples/signup-conversion.svg"),
    ),
    (
        "headcount",
        include_str!("../examples/headcount.json"),
        include_str!("../examples/headcount.svg"),
    ),
    (
        "csat-by-region",
        include_str!("../examples/csat-by-region.json"),
        include_str!("../examples/csat-by-region.svg"),
    ),
    (
        "daily-orders",
        include_str!("../examples/daily-orders.json"),
        include_str!("../examples/daily-orders.svg"),
    ),
    (
        "revenue-vs-forecast",
        include_str!("../examples/revenue-vs-forecast.json"),
        include_str!("../examples/revenue-vs-forecast.svg"),
    ),
    (
        "topicmap-sample",
        include_str!("../examples/topicmap-sample.json"),
        include_str!("../examples/topicmap-sample.svg"),
    ),
    (
        "knowledge-landscape",
        include_str!("../examples/knowledge-landscape.json"),
        include_str!("../examples/knowledge-landscape.svg"),
    ),
    (
        "temperature-projection",
        include_str!("../examples/temperature-projection.json"),
        include_str!("../examples/temperature-projection.svg"),
    ),
    (
        "annual-mean-threshold",
        include_str!("../examples/annual-mean-threshold.json"),
        include_str!("../examples/annual-mean-threshold.svg"),
    ),
    (
        "warming-stripes",
        include_str!("../examples/warming-stripes.json"),
        include_str!("../examples/warming-stripes.svg"),
    ),
    (
        "daily-anomaly-calendar",
        include_str!("../examples/daily-anomaly-calendar.json"),
        include_str!("../examples/daily-anomaly-calendar.svg"),
    ),
    (
        "warming-contributions",
        include_str!("../examples/warming-contributions.json"),
        include_str!("../examples/warming-contributions.svg"),
    ),
    (
        "emission-pathways",
        include_str!("../examples/emission-pathways.json"),
        include_str!("../examples/emission-pathways.svg"),
    ),
    (
        "sensor-readings",
        include_str!("../examples/sensor-readings.json"),
        include_str!("../examples/sensor-readings.svg"),
    ),
    (
        "release-incidents",
        include_str!("../examples/release-incidents.json"),
        include_str!("../examples/release-incidents.svg"),
    ),
    (
        "share-price",
        include_str!("../examples/share-price.json"),
        include_str!("../examples/share-price.svg"),
    ),
    (
        "mobile-revenue",
        include_str!("../examples/mobile-revenue.json"),
        include_str!("../examples/mobile-revenue.svg"),
    ),
    (
        "cache-lookup",
        include_str!("../examples/cache-lookup.json"),
        include_str!("../examples/cache-lookup.svg"),
    ),
    (
        "async-export",
        include_str!("../examples/async-export.json"),
        include_str!("../examples/async-export.svg"),
    ),
    (
        "release-flow",
        include_str!("../examples/release-flow.json"),
        include_str!("../examples/release-flow.svg"),
    ),
    (
        "order-flow",
        include_str!("../examples/order-flow.json"),
        include_str!("../examples/order-flow.svg"),
    ),
    (
        "ticket-states",
        include_str!("../examples/ticket-states.json"),
        include_str!("../examples/ticket-states.svg"),
    ),
    (
        "shop-architecture",
        include_str!("../examples/shop-architecture.json"),
        include_str!("../examples/shop-architecture.svg"),
    ),
    (
        "tenant-architecture",
        include_str!("../examples/tenant-architecture.json"),
        include_str!("../examples/tenant-architecture.svg"),
    ),
    (
        "sign-in-sequence",
        include_str!("../examples/sign-in-sequence.json"),
        include_str!("../examples/sign-in-sequence.svg"),
    ),
    (
        "ownership-structure",
        include_str!("../examples/ownership-structure.json"),
        include_str!("../examples/ownership-structure.svg"),
    ),
    (
        "family-tree",
        include_str!("../examples/family-tree.json"),
        include_str!("../examples/family-tree.svg"),
    ),
    (
        "framework-benchmarks",
        include_str!("../examples/framework-benchmarks.json"),
        include_str!("../examples/framework-benchmarks.svg"),
    ),
    (
        "benefit-and-harm",
        include_str!("../examples/benefit-and-harm.json"),
        include_str!("../examples/benefit-and-harm.svg"),
    ),
    (
        "response-times",
        include_str!("../examples/response-times.json"),
        include_str!("../examples/response-times.svg"),
    ),
    (
        "satisfaction-scores",
        include_str!("../examples/satisfaction-scores.json"),
        include_str!("../examples/satisfaction-scores.svg"),
    ),
];

/// Examples with a mobile variant, and its reviewed SVG.
const MOBILE_EXAMPLES: [(&str, &str, &str); 7] = [
    (
        "mobile-revenue",
        include_str!("../examples/mobile-revenue.json"),
        include_str!("../examples/mobile-revenue.mobile.svg"),
    ),
    (
        "tenant-architecture",
        include_str!("../examples/tenant-architecture.json"),
        include_str!("../examples/tenant-architecture.mobile.svg"),
    ),
    (
        "sign-in-sequence",
        include_str!("../examples/sign-in-sequence.json"),
        include_str!("../examples/sign-in-sequence.mobile.svg"),
    ),
    (
        "ownership-structure",
        include_str!("../examples/ownership-structure.json"),
        include_str!("../examples/ownership-structure.mobile.svg"),
    ),
    (
        "family-tree",
        include_str!("../examples/family-tree.json"),
        include_str!("../examples/family-tree.mobile.svg"),
    ),
    (
        "framework-benchmarks",
        include_str!("../examples/framework-benchmarks.json"),
        include_str!("../examples/framework-benchmarks.mobile.svg"),
    ),
    (
        "response-times",
        include_str!("../examples/response-times.json"),
        include_str!("../examples/response-times.mobile.svg"),
    ),
];

/// Examples with a reviewed print variant: hatched bands and modeled lines on a light chart, and
/// a dark chart with a color declared as a CSS variable.
const PRINT_EXAMPLES: [(&str, &str, &str); 4] = [
    (
        "temperature-projection",
        include_str!("../examples/temperature-projection.json"),
        include_str!("../examples/temperature-projection.print.svg"),
    ),
    (
        "revenue-vs-forecast",
        include_str!("../examples/revenue-vs-forecast.json"),
        include_str!("../examples/revenue-vs-forecast.print.svg"),
    ),
    (
        "tenant-architecture",
        include_str!("../examples/tenant-architecture.json"),
        include_str!("../examples/tenant-architecture.print.svg"),
    ),
    (
        "sign-in-sequence",
        include_str!("../examples/sign-in-sequence.json"),
        include_str!("../examples/sign-in-sequence.print.svg"),
    ),
];

/// Examples with a reviewed social variant.
const SOCIAL_EXAMPLES: [(&str, &str, &str); 1] = [(
    "temperature-projection",
    include_str!("../examples/temperature-projection.json"),
    include_str!("../examples/temperature-projection.social.svg"),
)];

/// The committed SVGs are the reviewed reference output. Each example must render to exactly the
/// bytes that are checked into `examples/`, on every supported platform.
#[test]
fn every_example_matches_its_reviewed_svg() {
    for (name, specification, expected) in EXAMPLES {
        let actual = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
            .unwrap_or_else(|error| panic!("{name} should render: {error}"));
        assert_eq!(
            actual.content, expected,
            "{name} drifted from its reviewed SVG"
        );
    }
}

/// The mobile variant is reference output too.
#[test]
fn every_mobile_variant_matches_its_reviewed_svg() {
    for (name, specification, expected) in MOBILE_EXAMPLES {
        let actual = render_json(
            specification,
            RenderFormat::Svg,
            &RenderOptions {
                variant: Variant::Mobile,
                ..RenderOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("{name} should render its mobile variant: {error}"));
        assert_eq!(
            actual.content, expected,
            "{name} drifted from its reviewed mobile SVG"
        );
    }
}

fn print_options() -> RenderOptions {
    RenderOptions {
        variant: Variant::Print,
        ..RenderOptions::default()
    }
}

/// The print variant is reference output too.
#[test]
fn every_print_variant_matches_its_reviewed_svg() {
    for (name, specification, expected) in PRINT_EXAMPLES {
        let actual = render_json(specification, RenderFormat::Svg, &print_options())
            .unwrap_or_else(|error| panic!("{name} should render its print variant: {error}"));
        assert_eq!(
            actual.content, expected,
            "{name} drifted from its reviewed print SVG"
        );
    }
}

/// No example's print variant leaves a color to CSS custom properties, `currentColor` or a rule
/// that needs a browser.
#[test]
fn every_print_variant_carries_literal_colors_only() {
    for (name, specification, _) in EXAMPLES {
        let svg = render_json(specification, RenderFormat::Svg, &print_options())
            .unwrap_or_else(|error| panic!("{name} should render its print variant: {error}"))
            .content;
        for needle in [
            "var(",
            "--chartlet",
            "currentColor",
            ":has(",
            "@container",
            "[class",
        ] {
            assert!(
                !svg.contains(needle),
                "{name}'s print variant contains {needle}"
            );
        }
    }
}

/// The social variant is reference output too.
#[test]
fn every_social_variant_matches_its_reviewed_svg() {
    for (name, specification, expected) in SOCIAL_EXAMPLES {
        let actual = render_json(
            specification,
            RenderFormat::Svg,
            &RenderOptions {
                variant: Variant::Social,
                ..RenderOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("{name} should render its social variant: {error}"));
        assert_eq!(
            actual.content, expected,
            "{name} drifted from its reviewed social SVG"
        );
    }
}

/// Every example renders a social variant at 1200 × 630 with literal colors only and without the
/// tooltips of its marks.
#[test]
fn every_social_variant_is_a_resolved_1200_by_630_canvas() {
    for (name, specification, _) in EXAMPLES {
        let svg = render_json(
            specification,
            RenderFormat::Svg,
            &RenderOptions {
                variant: Variant::Social,
                ..RenderOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("{name} should render its social variant: {error}"))
        .content;
        assert!(
            svg.starts_with(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"630\" viewBox=\"0 0 1200 630\""
            ),
            "{name}'s social variant is not 1200 × 630"
        );
        for needle in ["var(", "--chartlet", "currentColor", ":has(", "<title>"] {
            assert!(
                !svg.contains(needle),
                "{name}'s social variant contains {needle}"
            );
        }
    }
}
