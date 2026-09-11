use chartlet::{RenderFormat, RenderOptions, render_json};

/// The committed SVGs are the reviewed reference output. Each example must render to exactly the
/// bytes that are checked into `examples/`, on every supported platform.
#[test]
fn every_example_matches_its_reviewed_svg() {
    let examples: [(&str, &str, &str); 10] = [
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
    ];

    for (name, specification, expected) in examples {
        let actual = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
            .unwrap_or_else(|error| panic!("{name} should render: {error}"));
        assert_eq!(
            actual.content, expected,
            "{name} drifted from its reviewed SVG"
        );
    }
}
