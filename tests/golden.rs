use chartlet::{RenderFormat, RenderOptions, render_json};

#[test]
fn monthly_revenue_matches_the_reviewed_svg() {
    let specification = include_str!("../examples/monthly-revenue.json");
    let expected = include_str!("../examples/monthly-revenue.svg");
    let actual = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
        .expect("example specification should render");

    assert_eq!(actual.content, expected);
}

#[test]
fn quarterly_change_matches_the_reviewed_svg() {
    let specification = include_str!("../examples/quarterly-change.json");
    let expected = include_str!("../examples/quarterly-change.svg");
    let actual = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
        .expect("example specification should render");

    assert_eq!(actual.content, expected);
}

#[test]
fn budget_vs_actual_matches_the_reviewed_svg() {
    let specification = include_str!("../examples/budget-vs-actual.json");
    let expected = include_str!("../examples/budget-vs-actual.svg");
    let actual = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
        .expect("example specification should render");

    assert_eq!(actual.content, expected);
}

#[test]
fn monthly_trend_matches_the_reviewed_svg() {
    let specification = include_str!("../examples/monthly-trend.json");
    let expected = include_str!("../examples/monthly-trend.svg");
    let actual = render_json(specification, RenderFormat::Svg, &RenderOptions::default())
        .expect("example specification should render");

    assert_eq!(actual.content, expected);
}
