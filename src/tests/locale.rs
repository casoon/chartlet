use super::*;

#[test]
fn german_locale_writes_description_table_and_numbers_in_german() {
    let spec = TIME
        .replace(
            "\"type\": \"time\",",
            "\"type\": \"time\", \"locale\": \"de\", \"source\": \"Shop\",",
        )
        .replace(
            "\"valueAxis\": {\"title\": \"Orders\"}",
            "\"valueAxis\": {\"title\": \"Orders\", \"decimals\": 1}",
        )
        .replace("\"value\": 15}", "\"value\": -1.5}");
    let html = render_json(&spec, RenderFormat::Html, &RenderOptions::default()).unwrap();
    let content = &html.content;
    assert!(content.contains(
        "Zeitreihe mit 3 Punkten von 01.03.2026 bis 03.03.2026. Höchster Wert: 20,0 (03.03.2026). Niedrigster Wert: \u{2212}1,5 (02.03.2026)."
    ));
    assert!(content.contains("<summary>Diagrammdaten anzeigen</summary>"));
    assert!(content.contains("<caption>Daten zu Daily orders</caption>"));
    assert!(content.contains("<th scope=\"col\">Zeit</th>"));
    assert!(content.contains("<td>\u{2212}1,5</td>"));
    assert!(content.contains("<p class=\"chartlet-source\">Quelle: Shop</p>"));
}

#[test]
fn german_bar_chart_writes_description_table_and_numbers_in_german() {
    let html = html_ok(&german(SPEC, "bar").replace("\"value\": -4}", "\"value\": -4.5}"));
    assert!(html.contains(
        "Balkendiagramm mit 2 Kategorien. Höchster Wert: 12 (North &lt;East&gt;). Niedrigster Wert: \u{2212}4,5 (South)."
    ));
    assert!(html.contains("<th scope=\"col\">Kategorie</th><th scope=\"col\">Wert</th>"));
    assert!(html.contains("<td>\u{2212}4,5</td>"));
}

#[test]
fn german_stripes_write_description_tooltips_and_table_in_german() {
    let html = html_ok(&german(STRIPES, "stripes"));
    assert!(html.contains("<title>1851: \u{2212}0,5</title>"));
    assert!(html.contains(
        "Wärmestreifen von 1850 bis 1855, ein Streifen pro Jahr, auf einer divergierenden Farbskala um 0 mit den äußersten Stufen bei \u{2212}1 und 1. Niedrigster Wert: \u{2212}1 (1850). Höchster Wert: 2 (1855). 1 Jahr hat keinen Wert."
    ));
    assert!(html.contains("<th scope=\"col\">Jahr</th>"));
    assert!(html.contains("<th scope=\"row\">1852</th><td>fehlt</td>"));
}

#[test]
fn german_calendar_names_months_and_weekdays_in_german() {
    let days = DAYS.replace("\"value\": -2}", "\"value\": -2.5}");
    let months = html_ok(&german(&calendar("months", &days), "calendar"));
    for month in ["Jan", "Mär", "Mai", "Okt", "Dez"] {
        assert!(months.contains(&format!(">{month}</text>")), "{month}");
    }
    assert!(!months.contains(">Mar</text>"));
    assert!(months.contains("<title>2024-01-01: \u{2212}2,5</title>"));
    assert!(months.contains("<title>2024-12-31: kein Wert</title>"));
    assert!(months.contains(
        "Kalender 2024 mit einer Zeile pro Monat, auf einer divergierenden Farbskala um "
    ));
    assert!(months.contains(
        "2 von 366 Tagen haben einen Wert. Niedrigster Wert: \u{2212}2,5 (2024-01-01). Höchster Wert: 4 (2024-02-29)."
    ));
    assert!(months.contains("<th scope=\"col\">Datum</th>"));
    let weeks = render_ok(&german(&calendar("weeks", DAYS), "calendar")).content;
    for weekday in ["Mo", "Mi", "Fr", "So"] {
        assert!(weeks.contains(&format!(">{weekday}</text>")), "{weekday}");
    }
    assert!(weeks.contains(">Mär</text>"));
    assert!(weeks.contains("einer Zeile pro Wochentag und einer Spalte pro Woche"));
}

#[test]
fn german_range_chart_writes_spans_legend_and_table_in_german() {
    let html = html_ok(&german(RANGES, "rangebar"));
    assert!(html.contains("<title>Observed: 1,05 (0,9 bis 1,2)</title>"));
    assert!(html.contains("<title>Natural: \u{2212}0,1 bis 0,1, modelliert</title>"));
    assert!(html.contains(">Schraffiert: modelliert<"));
    assert!(html.contains(
        "Spannweitendiagramm mit 2 Kategorien, jeweils eine Spanne vom unteren zum oberen Wert mit einem mittleren Wert. Niedrigster unterer Wert: \u{2212}0,1 (Natural). Höchster oberer Wert: 1,2 (Observed). Modelliert, schraffiert gezeichnet: Natural."
    ));
    assert!(html.contains(
        "<th scope=\"col\">Kategorie</th><th scope=\"col\">Unterer Wert</th><th scope=\"col\">Mittlerer Wert</th><th scope=\"col\">Oberer Wert</th>"
    ));
    assert!(html.contains("<th scope=\"row\">Natural (modelliert)</th>"));
}

#[test]
fn german_maps_write_description_picker_and_table_in_german() {
    let topicmap = html_ok(&german(TOPICMAP, "topicmap"));
    assert!(topicmap.contains(
        "Themenkarte mit 5 Gebieten und einer Insel. Größtes: AI in practice mit 291 Einträgen, kleinstes: Management mit 69. Engste Nachbarn: AI in practice und Engineering."
    ));
    assert!(topicmap.contains("<legend>Gebiet</legend>"));
    assert!(topicmap.contains(
        "<th scope=\"col\">Thema</th><th scope=\"col\">Einträge</th><th scope=\"col\">Pfade</th><th scope=\"col\">Anteil</th>"
    ));
    let atlas = html_ok(&german(ATLAS, "atlas"));
    assert!(atlas.contains(
        "Wissenslandschaft aus 2 Bereichen und 4 Regionen; die größte ist Models mit 40. 1 Ort ist markiert."
    ));
    assert!(atlas.contains(
        "<th scope=\"col\">Region</th><th scope=\"col\">Einträge</th><th scope=\"col\">Orte</th>"
    ));
}

#[test]
fn the_text_alternative_matches_the_description_and_the_table() {
    let spec = ChartSpec::from_json(&german(TIME, "time")).unwrap();
    let alternative = text_alternative(&spec).unwrap();
    let html = html_ok(&german(TIME, "time"));
    assert!(html.contains(&crate::render::escape(&alternative.description)));
    assert!(html.contains(&format!(
        "<caption>{}</caption>",
        crate::render::escape(&alternative.table.caption)
    )));
    assert_eq!(alternative.table.columns[0], "Zeit");
    assert_eq!(alternative.table.rows.len(), 3);
    assert!(alternative.table.rows.iter().all(|row| row.len() == 2));
}
