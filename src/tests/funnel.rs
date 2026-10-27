use super::*;

const FUNNEL: &str = r#"{"schemaVersion": 1, "type": "funnel", "title": "Checkout", "width": 600, "height": 360,
    "funnel": {"stages": [
        {"label": "Visits", "value": 1000}, {"label": "Carts", "value": 400},
        {"label": "Orders", "value": 100}
    ]}}"#;

const PYRAMID: &str = r#"{"schemaVersion": 1, "type": "funnel", "title": "Needs", "width": 600, "height": 360,
    "funnel": {"display": "pyramid", "stages": [
        {"label": "Growth"}, {"label": "Belonging"}, {"label": "Safety"}, {"label": "Survival"}
    ]}}"#;

fn html(spec: &str) -> String {
    render_json(spec, RenderFormat::Html, &RenderOptions::default())
        .unwrap()
        .content
}

#[test]
fn a_funnel_draws_centered_bars_and_the_rate_between_stages() {
    let output = render_ok(FUNNEL);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("class=\"chartlet-funnel-bar\"").count(), 3);
    assert_eq!(svg.matches("class=\"chartlet-funnel-rate\"").count(), 2);
    assert!(svg.contains(">40.0%</text>"));
    assert!(svg.contains(">25.0%</text>"));
    assert!(svg.contains("<title>Carts: 400 (40.0% of the stage before)</title>"));
    // The bars are centered: the widest and the narrowest share a middle.
    let middle = |tooltip: &str| -> f64 {
        let at = svg.find(tooltip).expect("the bar is drawn");
        let rect = &svg[svg[..at].rfind("<rect").unwrap()..at];
        let number = |name: &str| -> f64 {
            let from = rect.find(&format!(" {name}=\"")).unwrap() + name.len() + 3;
            rect[from..from + rect[from..].find('"').unwrap()]
                .parse()
                .unwrap()
        };
        number("x") + number("width") / 2.0
    };
    assert!((middle("<title>Visits") - middle("<title>Orders")).abs() < 0.01);
}

#[test]
fn a_funnel_names_every_stage_in_the_description_and_the_table() {
    let html = html(FUNNEL);
    assert!(html.contains("Funnel of 3 stages, from Visits (1000) to Orders (100): 10.0% of the first stage remain. Stages, with the share of the stage before: Visits 1000; Carts 400 (40.0%); Orders 100 (25.0%). The biggest drop is from Carts to Orders: 25.0% remain."));
    assert!(html.contains(
        "<th scope=\"col\">Stage</th><th scope=\"col\">Value</th><th scope=\"col\">Share</th><th scope=\"col\">Rate</th>"
    ));
    assert!(html.contains("<th scope=\"row\">Carts</th><td>400</td><td>40.0%</td><td>40.0%</td>"));
    assert!(html.contains("<th scope=\"row\">Visits</th><td>1000</td><td>100.0%</td><td></td>"));
}

#[test]
fn a_pyramid_stacks_levels_without_values() {
    let output = render_ok(PYRAMID);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    let svg = &output.content;
    assert_eq!(svg.matches("<polyline").count(), 4);
    assert_eq!(svg.matches("class=\"chartlet-funnel-rate\"").count(), 0);
    let html = html(PYRAMID);
    assert!(
        html.contains("Pyramid of 4 levels, from the top: Growth; Belonging; Safety; Survival.")
    );
    assert!(html.contains("<th scope=\"col\">Level</th><th scope=\"col\">Position</th>"));
    assert!(html.contains("<th scope=\"row\">Safety</th><td>3</td>"));
}

#[test]
fn a_pyramid_may_carry_values_and_a_widening_funnel_is_reported() {
    let json = PYRAMID
        .replace("\"Growth\"}", "\"Growth\", \"value\": 5}")
        .replace("\"Belonging\"}", "\"Belonging\", \"value\": 20}")
        .replace("\"Safety\"}", "\"Safety\", \"value\": 30}")
        .replace("\"Survival\"}", "\"Survival\", \"value\": 45}");
    let html = html(&json);
    assert!(html.contains("from the top: Growth 5; Belonging 20; Safety 30; Survival 45."));
    assert!(html.contains("<th scope=\"row\">Growth</th><td>1</td><td>5</td>"));
    let widening = FUNNEL.replace("\"value\": 100", "\"value\": 700");
    let output = render_ok(&widening);
    assert_eq!(output.warnings.len(), 1);
    assert_eq!(output.warnings[0].code, "stage_exceeds_previous");
    assert_eq!(output.warnings[0].path, "/funnel/stages/2/value");
}

#[test]
fn funnels_are_validated_by_name() {
    for (spec, from, to, code, path) in [
        (
            FUNNEL,
            "\"value\": 400",
            "\"value\": 0",
            "invalid_value",
            "/funnel/stages/1/value",
        ),
        (
            FUNNEL,
            "{\"label\": \"Carts\", \"value\": 400}",
            "{\"label\": \"Carts\"}",
            "invalid_value",
            "/funnel/stages/1/value",
        ),
        (
            FUNNEL,
            "\"Orders\"",
            "\"Visits\"",
            "duplicate_label",
            "/funnel/stages/2/label",
        ),
        (
            PYRAMID,
            "{\"label\": \"Growth\"}",
            "{\"label\": \"Growth\", \"value\": 3}",
            "invalid_value",
            "/funnel/stages/1/value",
        ),
        (
            FUNNEL,
            "\"stages\": [",
            "\"stages\": [{\"label\": \"A\", \"value\": 2000}, {\"label\": \"B\", \"value\": 1900}, {\"label\": \"C\", \"value\": 1800}, {\"label\": \"D\", \"value\": 1700}, {\"label\": \"E\", \"value\": 1600}, {\"label\": \"F\", \"value\": 1500}, {\"label\": \"G\", \"value\": 1400}, {\"label\": \"H\", \"value\": 1300}, {\"label\": \"I\", \"value\": 1200}, {\"label\": \"J\", \"value\": 1100},",
            "too_many_data_points",
            "/funnel/stages",
        ),
        (
            FUNNEL,
            "\"funnel\": {",
            "\"data\": [{\"label\": \"A\", \"value\": 1}], \"funnel\": {",
            "option_not_supported",
            "/data",
        ),
    ] {
        assert_eq!(
            render_err(&spec.replace(from, to)),
            (code, path.to_owned()),
            "{from}"
        );
    }
    let one = r#"{"schemaVersion": 1, "type": "funnel", "title": "T", "funnel": {"stages": [{"label": "A", "value": 1}]}}"#;
    assert_eq!(render_err(one), ("empty_data", "/funnel/stages".to_owned()));
    let none = r#"{"schemaVersion": 1, "type": "funnel", "title": "T"}"#;
    assert_eq!(render_err(none), ("missing_funnel", "/funnel".to_owned()));
}
