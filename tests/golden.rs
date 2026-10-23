use chartlet::{RenderFormat, RenderOptions, Variant, render_json};

mod report;

/// Every example as name, specification and reviewed SVG.
const EXAMPLES: [(&str, &str, &str); 75] = [
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
        "shop-symbols",
        include_str!("../examples/shop-symbols.json"),
        include_str!("../examples/shop-symbols.svg"),
    ),
    (
        "billing-migration",
        include_str!("../examples/billing-migration.json"),
        include_str!("../examples/billing-migration.svg"),
    ),
    (
        "checkout-calls",
        include_str!("../examples/checkout-calls.json"),
        include_str!("../examples/checkout-calls.svg"),
    ),
    (
        "shop-deployment",
        include_str!("../examples/shop-deployment.json"),
        include_str!("../examples/shop-deployment.svg"),
    ),
    (
        "request-volume",
        include_str!("../examples/request-volume.json"),
        include_str!("../examples/request-volume.svg"),
    ),
    (
        "co-authors",
        include_str!("../examples/co-authors.json"),
        include_str!("../examples/co-authors.svg"),
    ),
    (
        "service-dependencies",
        include_str!("../examples/service-dependencies.json"),
        include_str!("../examples/service-dependencies.svg"),
    ),
    (
        "department-budgets",
        include_str!("../examples/department-budgets.json"),
        include_str!("../examples/department-budgets.svg"),
    ),
    (
        "claim-handling",
        include_str!("../examples/claim-handling.json"),
        include_str!("../examples/claim-handling.svg"),
    ),
    (
        "climate-similarity",
        include_str!("../examples/climate-similarity.json"),
        include_str!("../examples/climate-similarity.svg"),
    ),
    (
        "service-fan-out",
        include_str!("../examples/service-fan-out.json"),
        include_str!("../examples/service-fan-out.svg"),
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
        "product-strategy",
        include_str!("../examples/product-strategy.json"),
        include_str!("../examples/product-strategy.svg"),
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
    (
        "trial-effects",
        include_str!("../examples/trial-effects.json"),
        include_str!("../examples/trial-effects.svg"),
    ),
    (
        "product-roadmap",
        include_str!("../examples/product-roadmap.json"),
        include_str!("../examples/product-roadmap.svg"),
    ),
    (
        "revenue-to-profit",
        include_str!("../examples/revenue-to-profit.json"),
        include_str!("../examples/revenue-to-profit.svg"),
    ),
    (
        "energy-sources",
        include_str!("../examples/energy-sources.json"),
        include_str!("../examples/energy-sources.svg"),
    ),
    (
        "election-result",
        include_str!("../examples/election-result.json"),
        include_str!("../examples/election-result.svg"),
    ),
    (
        "budget-by-department",
        include_str!("../examples/budget-by-department.json"),
        include_str!("../examples/budget-by-department.svg"),
    ),
    (
        "energy-flow",
        include_str!("../examples/energy-flow.json"),
        include_str!("../examples/energy-flow.svg"),
    ),
    (
        "response-distribution",
        include_str!("../examples/response-distribution.json"),
        include_str!("../examples/response-distribution.svg"),
    ),
    (
        "overall-survival",
        include_str!("../examples/overall-survival.json"),
        include_str!("../examples/overall-survival.svg"),
    ),
    (
        "gene-expression",
        include_str!("../examples/gene-expression.json"),
        include_str!("../examples/gene-expression.svg"),
    ),
    (
        "allometry",
        include_str!("../examples/allometry.json"),
        include_str!("../examples/allometry.svg"),
    ),
    (
        "genome-scan",
        include_str!("../examples/genome-scan.json"),
        include_str!("../examples/genome-scan.svg"),
    ),
    (
        "feature-priorities",
        include_str!("../examples/feature-priorities.json"),
        include_str!("../examples/feature-priorities.svg"),
    ),
    (
        "latency-by-region",
        include_str!("../examples/latency-by-region.json"),
        include_str!("../examples/latency-by-region.svg"),
    ),
    (
        "order-model",
        include_str!("../examples/order-model.json"),
        include_str!("../examples/order-model.svg"),
    ),
    (
        "water-cycle",
        include_str!("../examples/water-cycle.json"),
        include_str!("../examples/water-cycle.svg"),
    ),
    (
        "shop-schema",
        include_str!("../examples/shop-schema.json"),
        include_str!("../examples/shop-schema.svg"),
    ),
    (
        "satisfaction-by-quarter",
        include_str!("../examples/satisfaction-by-quarter.json"),
        include_str!("../examples/satisfaction-by-quarter.svg"),
    ),
    (
        "voter-movement",
        include_str!("../examples/voter-movement.json"),
        include_str!("../examples/voter-movement.svg"),
    ),
    (
        "budget-by-sector",
        include_str!("../examples/budget-by-sector.json"),
        include_str!("../examples/budget-by-sector.svg"),
    ),
    (
        "monthly-satisfaction",
        include_str!("../examples/monthly-satisfaction.json"),
        include_str!("../examples/monthly-satisfaction.svg"),
    ),
    (
        "study-selection",
        include_str!("../examples/study-selection.json"),
        include_str!("../examples/study-selection.svg"),
    ),
    (
        "court-instances",
        include_str!("../examples/court-instances.json"),
        include_str!("../examples/court-instances.svg"),
    ),
    (
        "data-tier",
        include_str!("../examples/data-tier.json"),
        include_str!("../examples/data-tier.svg"),
    ),
    (
        "shop-views",
        include_str!("../examples/shop-views.json"),
        include_str!("../examples/shop-views.svg"),
    ),
];

/// Examples with a mobile variant, and its reviewed SVG.
const MOBILE_EXAMPLES: [(&str, &str, &str); 29] = [
    (
        "co-authors",
        include_str!("../examples/co-authors.json"),
        include_str!("../examples/co-authors.mobile.svg"),
    ),
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
    (
        "trial-effects",
        include_str!("../examples/trial-effects.json"),
        include_str!("../examples/trial-effects.mobile.svg"),
    ),
    (
        "product-roadmap",
        include_str!("../examples/product-roadmap.json"),
        include_str!("../examples/product-roadmap.mobile.svg"),
    ),
    (
        "revenue-to-profit",
        include_str!("../examples/revenue-to-profit.json"),
        include_str!("../examples/revenue-to-profit.mobile.svg"),
    ),
    (
        "energy-sources",
        include_str!("../examples/energy-sources.json"),
        include_str!("../examples/energy-sources.mobile.svg"),
    ),
    (
        "election-result",
        include_str!("../examples/election-result.json"),
        include_str!("../examples/election-result.mobile.svg"),
    ),
    (
        "budget-by-department",
        include_str!("../examples/budget-by-department.json"),
        include_str!("../examples/budget-by-department.mobile.svg"),
    ),
    (
        "energy-flow",
        include_str!("../examples/energy-flow.json"),
        include_str!("../examples/energy-flow.mobile.svg"),
    ),
    (
        "response-distribution",
        include_str!("../examples/response-distribution.json"),
        include_str!("../examples/response-distribution.mobile.svg"),
    ),
    (
        "overall-survival",
        include_str!("../examples/overall-survival.json"),
        include_str!("../examples/overall-survival.mobile.svg"),
    ),
    (
        "gene-expression",
        include_str!("../examples/gene-expression.json"),
        include_str!("../examples/gene-expression.mobile.svg"),
    ),
    (
        "allometry",
        include_str!("../examples/allometry.json"),
        include_str!("../examples/allometry.mobile.svg"),
    ),
    (
        "genome-scan",
        include_str!("../examples/genome-scan.json"),
        include_str!("../examples/genome-scan.mobile.svg"),
    ),
    (
        "feature-priorities",
        include_str!("../examples/feature-priorities.json"),
        include_str!("../examples/feature-priorities.mobile.svg"),
    ),
    (
        "latency-by-region",
        include_str!("../examples/latency-by-region.json"),
        include_str!("../examples/latency-by-region.mobile.svg"),
    ),
    (
        "satisfaction-by-quarter",
        include_str!("../examples/satisfaction-by-quarter.json"),
        include_str!("../examples/satisfaction-by-quarter.mobile.svg"),
    ),
    (
        "voter-movement",
        include_str!("../examples/voter-movement.json"),
        include_str!("../examples/voter-movement.mobile.svg"),
    ),
    (
        "budget-by-sector",
        include_str!("../examples/budget-by-sector.json"),
        include_str!("../examples/budget-by-sector.mobile.svg"),
    ),
    (
        "monthly-satisfaction",
        include_str!("../examples/monthly-satisfaction.json"),
        include_str!("../examples/monthly-satisfaction.mobile.svg"),
    ),
    (
        "study-selection",
        include_str!("../examples/study-selection.json"),
        include_str!("../examples/study-selection.mobile.svg"),
    ),
    (
        "court-instances",
        include_str!("../examples/court-instances.json"),
        include_str!("../examples/court-instances.mobile.svg"),
    ),
    (
        "data-tier",
        include_str!("../examples/data-tier.json"),
        include_str!("../examples/data-tier.mobile.svg"),
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

/// Views of an architecture model besides its default view, and their reviewed SVGs.
const VIEW_EXAMPLES: [(&str, &str, &str, &str); 1] = [(
    "shop-views",
    "checkout",
    include_str!("../examples/shop-views.json"),
    include_str!("../examples/shop-views.checkout.svg"),
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

/// The views besides the default view are reference output too.
#[test]
fn every_view_matches_its_reviewed_svg() {
    for (name, view, specification, expected) in VIEW_EXAMPLES {
        let actual = render_json(
            specification,
            RenderFormat::Svg,
            &RenderOptions {
                view: Some(view.to_owned()),
                ..RenderOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("{name} should render its view {view}: {error}"));
        assert_eq!(
            actual.content, expected,
            "{name} drifted from its reviewed SVG of the view {view}"
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

/// Writes a visual report of every reference above against the current rendering, plus the
/// examples without one, to `target/tmp/golden-report/index.html`:
/// `cargo test --test golden visual_report -- --ignored --nocapture`. It shows the changed
/// references by default and never fails on a difference or accepts one.
#[test]
#[ignore = "writes a local report for review; run on demand"]
fn visual_report() {
    let mut missing: Vec<(String, String)> = std::fs::read_dir("examples")
        .expect("the examples directory should exist")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .filter_map(|path| {
            let name = path.file_stem()?.to_str()?.to_owned();
            (!EXAMPLES.iter().any(|(known, _, _)| *known == name)).then(|| {
                let specification = std::fs::read_to_string(&path).expect("readable example");
                (name, specification)
            })
        })
        .collect();
    missing.sort();
    let directory = report::directory();
    let summary = report::write(&directory, &report_entries(), &missing)
        .expect("the report should be written");
    println!(
        "{} changed, {} failed, {} unchanged, {} without reference: {}",
        summary.changed,
        summary.failed,
        summary.unchanged,
        summary.missing,
        directory.join("index.html").display()
    );
}

/// Every reference of the catalogs above, with the options its variant renders with.
fn report_entries() -> Vec<report::Entry<'static>> {
    let catalogs = [
        ("desktop", &EXAMPLES[..], Variant::Desktop),
        ("mobile", &MOBILE_EXAMPLES[..], Variant::Mobile),
        ("print", &PRINT_EXAMPLES[..], Variant::Print),
        ("social", &SOCIAL_EXAMPLES[..], Variant::Social),
    ];
    catalogs
        .into_iter()
        .flat_map(|(label, catalog, variant)| {
            catalog
                .iter()
                .map(move |&(name, specification, reference)| report::Entry {
                    name,
                    variant: label,
                    specification,
                    reference,
                    options: RenderOptions {
                        variant,
                        ..RenderOptions::default()
                    },
                })
        })
        .collect()
}

/// An altered reference of a flow chart, an architecture diagram and a data chart appears in the
/// report with both versions; an untouched one counts as unchanged.
#[test]
fn visual_report_tells_altered_references_from_unchanged_ones() {
    let altered = [
        ("release-flow", "desktop"),
        ("tenant-architecture", "mobile"),
        ("monthly-revenue", "desktop"),
    ];
    let mut entries: Vec<report::Entry> = report_entries()
        .into_iter()
        .filter(|entry| {
            altered.contains(&(entry.name, entry.variant))
                || (entry.name, entry.variant) == ("quarterly-change", "desktop")
        })
        .collect();
    let references: Vec<String> = entries
        .iter()
        .map(|entry| {
            entry
                .reference
                .replacen("<title", "<title data-altered=\"\"", 1)
        })
        .collect();
    for (entry, reference) in entries.iter_mut().zip(&references) {
        if altered.contains(&(entry.name, entry.variant)) {
            entry.reference = reference;
        }
    }
    let directory = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("golden-report-check");
    let summary = report::write(&directory, &entries, &[]).expect("the report should be written");
    assert_eq!(
        (summary.changed, summary.unchanged, summary.failed),
        (3, 1, 0)
    );
    let page = std::fs::read_to_string(directory.join("index.html")).expect("the report page");
    for file in [
        "release-flow",
        "tenant-architecture.mobile",
        "monthly-revenue",
    ] {
        for version in ["reference", "current"] {
            assert!(page.contains(&format!("<img src=\"svg/{file}.{version}.svg\"")));
            assert!(directory.join(format!("svg/{file}.{version}.svg")).exists());
        }
    }
    assert!(page.contains("<section class=\"card unchanged\"><h2>quarterly-change"));
    assert!(
        !page.contains("<svg"),
        "the report embeds every SVG as an image of its own"
    );
}
