use super::*;

#[test]
fn stripes_draw_one_colored_stripe_per_year() {
    let output = render_ok(STRIPES);
    let svg = &output.content;
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    // Five values; the missing year stays empty.
    assert_eq!(svg.matches("class=\"chartlet-diverging-").count(), 5);
    assert!(svg.contains("class=\"chartlet-diverging-0\"><title>1850: −1</title>"));
    assert!(svg.contains("class=\"chartlet-diverging-4\"><title>1851: −0.5</title>"));
    assert!(svg.contains("class=\"chartlet-diverging-8\"><title>1853: 0</title>"));
    // Beyond max takes the outermost step.
    assert!(svg.contains("class=\"chartlet-diverging-16\"><title>1855: 2</title>"));
    assert!(svg.contains(">1850</text>"));
    assert!(svg.contains(">1855</text>"));
    assert!(svg.contains("--chartlet-diverging-8:#eeeeee"));
    assert!(svg.contains(
        "Warming stripes from 1850 to 1855, one stripe per year, on a diverging color scale around 0 with its outermost steps at −1 and 1. Lowest: −1 (1850). Highest: 2 (1855). 1 year has no value."
    ));
    let html = render_json(STRIPES, RenderFormat::Html, &RenderOptions::default()).unwrap();
    assert!(html.content.contains("<th scope=\"col\">Year</th>"));
    assert!(
        html.content
            .contains("<th scope=\"row\">1852</th><td>Missing</td>")
    );
}

#[test]
fn stripes_are_validated_by_name() {
    for (from, to, code, path) in [
        ("\"min\": -1", "\"min\": 0", "invalid_scale", "/stripes/min"),
        ("\"max\": 1", "\"max\": -2", "invalid_scale", "/stripes/max"),
        (
            "\"firstYear\": 1850",
            "\"firstYear\": 9998",
            "invalid_year",
            "/stripes/firstYear",
        ),
        (
            "[-1, -0.5, null, 0, 0.5, 2]",
            "[null]",
            "empty_series",
            "/stripes/values",
        ),
        (
            "[-1, -0.5, null, 0, 0.5, 2]",
            "[]",
            "empty_data",
            "/stripes/values",
        ),
        (
            "\"title\": \"Stripes\",",
            "\"title\": \"Stripes\", \"panes\": [{}],",
            "option_not_supported",
            "/panes",
        ),
    ] {
        assert_eq!(
            render_err(&STRIPES.replace(from, to)),
            (code, path.to_owned()),
            "{to}"
        );
    }
    let missing = STRIPES.replace(
        r#""stripes": {"firstYear": 1850, "values": [-1, -0.5, null, 0, 0.5, 2], "min": -1, "max": 1}"#,
        "\"width\": 800",
    );
    assert_eq!(
        render_err(&missing),
        ("missing_stripes", "/stripes".to_owned())
    );
}

#[test]
fn a_block_of_another_type_is_rejected() {
    let foreign = SPEC.replace(
        "\"title\": \"Profit & loss\",",
        "\"title\": \"Profit & loss\", \"stripes\": {\"firstYear\": 2000, \"values\": [1]},",
    );
    assert_eq!(
        render_err(&foreign),
        ("option_not_supported", "/stripes".to_owned())
    );
    let columns = TIME.replace("\"type\": \"time\",", "\"type\": \"time\", \"columns\": 2,");
    assert_eq!(
        render_err(&columns),
        ("option_not_supported", "/columns".to_owned())
    );
}

#[test]
fn a_calendar_has_a_cell_for_every_day_and_leaves_gaps_empty() {
    for layout in ["months", "weeks"] {
        let output = render_ok(&calendar(layout, DAYS));
        let svg = &output.content;
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        // 2024 is a leap year: 366 cells, of which two carry a value; the key adds 17.
        assert_eq!(svg.matches("<rect").count(), 366 + 17, "{layout}");
        assert_eq!(
            svg.matches("class=\"chartlet-calendar-empty\"").count(),
            364
        );
        assert!(svg.contains("class=\"chartlet-diverging-4\"><title>2024-01-01: −2</title>"));
        assert!(svg.contains("class=\"chartlet-diverging-16\"><title>2024-02-29: 4</title>"));
        assert!(svg.contains("<title>2024-12-31: no value</title>"));
        assert!(svg.contains(
            "2 of 366 days have a value. Lowest: −2 (2024-01-01). Highest: 4 (2024-02-29)."
        ));
    }
    let weeks = render_ok(&calendar("weeks", DAYS)).content;
    assert!(weeks.contains(">Mon</text>"));
    let months = render_ok(&calendar("months", DAYS)).content;
    assert!(months.contains(">Feb</text>"));
    let html = render_json(
        &calendar("months", DAYS),
        RenderFormat::Html,
        &RenderOptions::default(),
    )
    .unwrap();
    assert!(
        html.content
            .contains("<th scope=\"row\">2024-02-29</th><td>4</td>")
    );
}

#[test]
fn calendar_days_are_validated_by_name() {
    for (days, code, path) in [
        (
            r#"{"date": "2023-12-31", "value": 1}"#,
            "date_outside_year",
            "/calendar/days/0/date",
        ),
        (
            r#"{"date": "2024-02-30", "value": 1}"#,
            "invalid_date",
            "/calendar/days/0/date",
        ),
        (
            r#"{"date": "2024-03-01T12:00", "value": 1}"#,
            "invalid_date",
            "/calendar/days/0/date",
        ),
        (
            r#"{"date": "2024-03-01", "value": 1}, {"date": "2024-03-01", "value": 2}"#,
            "duplicate_date",
            "/calendar/days/1/date",
        ),
        (
            r#"{"date": "2024-03-01", "value": null}"#,
            "empty_series",
            "/calendar/days",
        ),
        ("", "empty_data", "/calendar/days"),
    ] {
        assert_eq!(
            render_err(&calendar("months", days)),
            (code, path.to_owned()),
            "{days}"
        );
    }
}
