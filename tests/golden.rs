use chartlet::{RenderFormat, RenderOptions, render_json};

/// Every example as name, specification and reviewed SVG.
const EXAMPLES: [(&str, &str, &str); 20] = [
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
];

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
