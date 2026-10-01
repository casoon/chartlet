//! The words chartlet writes itself, per locale: descriptions, legend additions, tooltips and
//! the HTML figure. Text from the specification is never translated.

use crate::spec::Locale;

pub(crate) struct Words {
    pub value: &'static str,
    pub lower: &'static str,
    pub upper: &'static str,
    pub modeled: &'static str,
    pub from: &'static str,
    pub to: &'static str,
    pub range: &'static str,
    pub at: &'static str,
    pub highest: &'static str,
    pub lowest: &'static str,
    pub all_values: &'static str,
    pub reference_lines: &'static str,
    pub zones: &'static str,
    pub markers: &'static str,
    pub source: &'static str,
    pub show_data: &'static str,
    pub data_for: &'static str,
    pub time: &'static str,
    pub missing: &'static str,
    pub series: &'static str,
    pub view: &'static str,
    pub category: &'static str,
    pub year: &'static str,
    pub date: &'static str,
    pub topic: &'static str,
    pub region: &'static str,
    pub area: &'static str,
    pub entries: &'static str,
    pub paths: &'static str,
    pub share: &'static str,
    pub places: &'static str,
    pub range_low: &'static str,
    pub range_mid: &'static str,
    pub range_high: &'static str,
    pub hatched_modeled: &'static str,
    pub no_value: &'static str,
    pub open: &'static str,
    pub high: &'static str,
    pub low: &'static str,
    pub close: &'static str,
    pub candle_key: &'static str,
    pub pane: &'static str,
    pub months: [&'static str; 12],
    pub weekdays: [&'static str; 7],
}

const EN: Words = Words {
    value: "Value",
    lower: "lower",
    upper: "upper",
    modeled: "modeled",
    from: "from",
    to: "to",
    range: "range",
    at: "at",
    highest: "Highest",
    lowest: "Lowest",
    all_values: "All values",
    reference_lines: "Reference lines",
    zones: "Zones",
    markers: "Markers",
    source: "Source",
    show_data: "Show chart data",
    data_for: "Data for",
    time: "Time",
    missing: "Missing",
    series: "Series",
    view: "View",
    category: "Category",
    year: "Year",
    date: "Date",
    topic: "Topic",
    region: "Region",
    area: "Area",
    entries: "Entries",
    paths: "Paths",
    share: "Share",
    places: "Places",
    range_low: "Low",
    range_mid: "Mid",
    range_high: "High",
    hatched_modeled: "Hatched: modeled",
    no_value: "no value",
    open: "open",
    high: "high",
    low: "low",
    close: "close",
    candle_key: "hollow: rising, filled: falling",
    pane: "Pane",
    months: [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ],
    weekdays: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
};

const DE: Words = Words {
    value: "Wert",
    lower: "unten",
    upper: "oben",
    modeled: "modelliert",
    from: "von",
    to: "bis",
    range: "Spanne",
    at: "bei",
    highest: "Höchster Wert",
    lowest: "Niedrigster Wert",
    all_values: "Alle Werte",
    reference_lines: "Bezugslinien",
    zones: "Bereiche",
    markers: "Markierungen",
    source: "Quelle",
    show_data: "Diagrammdaten anzeigen",
    data_for: "Daten zu",
    time: "Zeit",
    missing: "fehlt",
    series: "Reihen",
    view: "Ansicht",
    category: "Kategorie",
    year: "Jahr",
    date: "Datum",
    topic: "Thema",
    region: "Region",
    area: "Gebiet",
    entries: "Einträge",
    paths: "Pfade",
    share: "Anteil",
    places: "Orte",
    range_low: "Unterer Wert",
    range_mid: "Mittlerer Wert",
    range_high: "Oberer Wert",
    hatched_modeled: "Schraffiert: modelliert",
    no_value: "kein Wert",
    open: "Eröffnung",
    high: "Hoch",
    low: "Tief",
    close: "Schluss",
    candle_key: "hohl: steigend, gefüllt: fallend",
    pane: "Teildiagramm",
    months: [
        "Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez",
    ],
    weekdays: ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"],
};

impl Locale {
    pub(crate) const fn words(self) -> &'static Words {
        match self {
            Self::En => &EN,
            Self::De => &DE,
        }
    }
}

/// Where a value lies, for the extremes of a description: `Budget in Feb` or `Budget, Feb`.
pub(crate) fn series_at(locale: Locale, name: &str, category: &str) -> String {
    match locale {
        Locale::En => format!("{name} in {category}"),
        Locale::De => format!("{name}, {category}"),
    }
}

/// The span of time a zone covers: both ends, or the one that is given.
pub(crate) fn zone_times(locale: Locale, from: Option<&str>, to: Option<&str>) -> Option<String> {
    match (locale, from, to) {
        (Locale::En, Some(from), Some(to)) => Some(format!("{from} to {to}")),
        (Locale::De, Some(from), Some(to)) => Some(format!("{from} bis {to}")),
        (Locale::En, Some(from), None) => Some(format!("from {from}")),
        (Locale::De, Some(from), None) => Some(format!("ab {from}")),
        (Locale::En, None, Some(to)) => Some(format!("until {to}")),
        (Locale::De, None, Some(to)) => Some(format!("bis {to}")),
        (_, None, None) => None,
    }
}

/// The range of values a zone covers: both edges, or the one that is given.
pub(crate) fn zone_values(
    locale: Locale,
    bottom: Option<&str>,
    top: Option<&str>,
) -> Option<String> {
    match (locale, bottom, top) {
        (Locale::En, Some(bottom), Some(top)) => Some(format!("values {bottom} to {top}")),
        (Locale::De, Some(bottom), Some(top)) => Some(format!("Werte {bottom} bis {top}")),
        (Locale::En, Some(bottom), None) => Some(format!("values above {bottom}")),
        (Locale::De, Some(bottom), None) => Some(format!("Werte über {bottom}")),
        (Locale::En, None, Some(top)) => Some(format!("values below {top}")),
        (Locale::De, None, Some(top)) => Some(format!("Werte unter {top}")),
        (_, None, None) => None,
    }
}

/// The opening sentence of a time chart.
pub(crate) fn time_opening(
    locale: Locale,
    points: usize,
    range: &str,
    names: &[String],
    modeled: bool,
) -> String {
    let point = plural(points, "point", "points");
    let punkt = plural(points, "Punkt", "Punkten");
    match (locale, names.len() > 1) {
        (Locale::En, true) => format!(
            "Time chart with {points} {point} {range} and {} series ({}).",
            names.len(),
            names.join(", ")
        ),
        (Locale::De, true) => format!(
            "Zeitreihe mit {points} {punkt} {range} und {} Reihen ({}).",
            names.len(),
            names.join(", ")
        ),
        (Locale::En, false) if modeled => {
            format!("Time chart with {points} {point} {range}, modeled.")
        }
        (Locale::De, false) if modeled => {
            format!("Zeitreihe mit {points} {punkt} {range}, modelliert.")
        }
        (Locale::En, false) => format!("Time chart with {points} {point} {range}."),
        (Locale::De, false) => format!("Zeitreihe mit {points} {punkt} {range}."),
    }
}

/// The opening sentence of small multiples.
pub(crate) fn multiples_opening(
    locale: Locale,
    panels: &[String],
    range: &str,
    names: &[String],
) -> String {
    let mut text = match locale {
        Locale::En => format!(
            "Small multiples of {} panels ({}) with a shared value axis, each {range}",
            panels.len(),
            panels.join(", ")
        ),
        Locale::De => format!(
            "Kleine Vielfache aus {} Feldern ({}) mit gemeinsamer Werteachse, jeweils {range}",
            panels.len(),
            panels.join(", ")
        ),
    };
    if names.len() > 1 {
        let showing = match locale {
            Locale::En => "showing",
            Locale::De => "mit",
        };
        text.push_str(", ");
        text.push_str(showing);
        text.push(' ');
        text.push_str(&names.join(", "));
    }
    text.push('.');
    text
}

/// The sentence that names the stacked panes of a time chart.
pub(crate) fn panes(locale: Locale, labels: &[String]) -> String {
    match locale {
        Locale::En => format!(
            " {} stacked panes on one time axis: {}.",
            labels.len(),
            labels.join(", ")
        ),
        Locale::De => format!(
            " {} übereinanderliegende Teildiagramme auf einer Zeitachse: {}.",
            labels.len(),
            labels.join(", ")
        ),
    }
}

/// The four values of a candle, for its tooltip: open, high, low and close, already written.
pub(crate) fn candle_values(locale: Locale, values: &[String; 4]) -> String {
    let words = locale.words();
    // A German number carries a decimal comma, so its values are kept apart by semicolons.
    let separator = match locale {
        Locale::En => ", ",
        Locale::De => "; ",
    };
    [words.open, words.high, words.low, words.close]
        .iter()
        .zip(values)
        .map(|(word, value)| format!("{word} {value}"))
        .collect::<Vec<_>>()
        .join(separator)
}

/// What a candlestick layer says in a description: where it opened and closed, how much it
/// changed, and its highest high and lowest low. Every value comes written, each with its time;
/// `percent` is missing when the first open is zero.
pub(crate) struct CandleSummary<'a> {
    pub name: Option<&'a str>,
    pub open: (&'a str, &'a str),
    pub close: (&'a str, &'a str),
    pub change: &'a str,
    pub percent: Option<&'a str>,
    pub high: (&'a str, &'a str),
    pub low: (&'a str, &'a str),
}

/// The sentence about one candlestick layer, see [`CandleSummary`].
pub(crate) fn candles(locale: Locale, summary: &CandleSummary) -> String {
    let percent = summary
        .percent
        .map_or_else(String::new, |percent| format!(" ({percent})"));
    let name = summary
        .name
        .map_or_else(String::new, |name| format!("{name}: "));
    let CandleSummary {
        open,
        close,
        high,
        low,
        change,
        ..
    } = summary;
    match (locale, summary.name.is_some()) {
        (Locale::En, named) => format!(
            " {name}{} at {} on {} and closed at {} on {}, a change of {change}{percent}; high {} on {}, low {} on {}.",
            if named { "opened" } else { "Opened" },
            open.0,
            open.1,
            close.0,
            close.1,
            high.0,
            high.1,
            low.0,
            low.1,
        ),
        (Locale::De, _) => format!(
            " {name}Eröffnung {} am {}, Schluss {} am {}, Veränderung {change}{percent}; Hoch {} am {}, Tief {} am {}.",
            open.0, open.1, close.0, close.1, high.0, high.1, low.0, low.1,
        ),
    }
}

/// The sentence about uncertainty bands.
pub(crate) fn bands(locale: Locale, banded: usize, hatched: bool) -> String {
    match locale {
        Locale::En => format!(
            " {} a shaded band from its lower to its upper bound{}; the data table lists both.",
            if banded == 1 {
                "One line has"
            } else {
                "Lines have"
            },
            if hatched {
                ", hatched where modeled"
            } else {
                ""
            }
        ),
        Locale::De => format!(
            " {} ein schattiertes Band von der unteren bis zur oberen Grenze{}; die Datentabelle nennt beide.",
            if banded == 1 {
                "Eine Linie hat"
            } else {
                "Linien haben"
            },
            if hatched {
                ", schraffiert, wo modelliert"
            } else {
                ""
            }
        ),
    }
}

/// The sentence about area layers; `names` lists them, empty for a single unnamed one.
pub(crate) fn areas(locale: Locale, names: &[String]) -> String {
    match (locale, names.len()) {
        (Locale::En, 0) => " The area below the line is filled down to zero.".to_owned(),
        (Locale::En, 1) => format!(" The area below {} is filled down to zero.", names[0]),
        (Locale::En, _) => format!(
            " The areas below {} are filled down to zero.",
            names.join(", ")
        ),
        (Locale::De, 0) => " Die Fläche unter der Linie ist bis null gefüllt.".to_owned(),
        (Locale::De, 1) => format!(" Die Fläche unter {} ist bis null gefüllt.", names[0]),
        (Locale::De, _) => format!(
            " Die Flächen unter {} sind bis null gefüllt.",
            names.join(", ")
        ),
    }
}

/// `one` for a count of one, otherwise `many`.
const fn plural(count: usize, one: &'static str, many: &'static str) -> &'static str {
    if count == 1 { one } else { many }
}

/// The name of a bar or a line chart.
const fn chart_name(locale: Locale, line: bool) -> &'static str {
    match (locale, line) {
        (Locale::En, false) => "Bar chart",
        (Locale::En, true) => "Line chart",
        (Locale::De, false) => "Balkendiagramm",
        (Locale::De, true) => "Liniendiagramm",
    }
}

/// The whole description of a bar or line chart whose values are all equal.
pub(crate) fn equal_values(locale: Locale, line: bool, categories: usize, value: &str) -> String {
    let chart = chart_name(locale, line);
    match locale {
        Locale::En if categories == 1 => format!("{chart} with 1 value: {value}."),
        Locale::En => format!("{chart} with {categories} equal values: {value} each."),
        Locale::De if categories == 1 => format!("{chart} mit 1 Wert: {value}."),
        Locale::De => format!("{chart} mit {categories} gleichen Werten: jeweils {value}."),
    }
}

/// The opening sentence of a bar or line chart; `names` lists the series of a grouped chart.
pub(crate) fn categories_opening(
    locale: Locale,
    line: bool,
    categories: usize,
    series: usize,
    names: &str,
) -> String {
    let chart = chart_name(locale, line);
    match (locale, series > 1) {
        (Locale::En, true) => format!(
            "{chart} with {categories} {} and {series} series ({names}).",
            plural(categories, "category", "categories")
        ),
        (Locale::De, true) => format!(
            "{chart} mit {categories} {} und {series} Reihen ({names}).",
            plural(categories, "Kategorie", "Kategorien")
        ),
        (Locale::En, false) => format!(
            "{chart} with {categories} {}.",
            plural(categories, "category", "categories")
        ),
        (Locale::De, false) => format!(
            "{chart} mit {categories} {}.",
            plural(categories, "Kategorie", "Kategorien")
        ),
    }
}

/// The sentence about missing values of a grouped chart or a time chart, empty when none is
/// missing.
pub(crate) fn missing_values(locale: Locale, missing: usize) -> String {
    match (locale, missing) {
        (_, 0) => String::new(),
        (Locale::En, 1) => " 1 value is missing.".to_owned(),
        (Locale::En, missing) => format!(" {missing} values are missing."),
        (Locale::De, 1) => " 1 Wert fehlt.".to_owned(),
        (Locale::De, missing) => format!(" {missing} Werte fehlen."),
    }
}

/// The sentence of a time axis whose gaps are collapsed: observations sit side by side, so the
/// distance between them no longer stands for time.
pub(crate) const fn gaps_collapsed(locale: Locale) -> &'static str {
    match locale {
        Locale::En => " Gaps in time are closed up.",
        Locale::De => " Zeiträume ohne Beobachtung sind ausgelassen.",
    }
}

/// The diverging color scale of stripes and calendars, as part of a sentence.
pub(crate) fn diverging_scale(locale: Locale, reference: &str, min: &str, max: &str) -> String {
    match locale {
        Locale::En => format!(
            "on a diverging color scale around {reference} with its outermost steps at {min} and {max}"
        ),
        Locale::De => format!(
            "auf einer divergierenden Farbskala um {reference} mit den äußersten Stufen bei {min} und {max}"
        ),
    }
}

/// The opening sentence of warming stripes.
pub(crate) fn stripes_opening(locale: Locale, first: i32, last: i32, scale: &str) -> String {
    match locale {
        Locale::En => {
            format!("Warming stripes from {first} to {last}, one stripe per year, {scale}.")
        }
        Locale::De => {
            format!("Wärmestreifen von {first} bis {last}, ein Streifen pro Jahr, {scale}.")
        }
    }
}

/// The sentence about years of warming stripes without a value, empty when there is none.
pub(crate) fn years_without_value(locale: Locale, missing: usize) -> String {
    match (locale, missing) {
        (_, 0) => String::new(),
        (Locale::En, 1) => " 1 year has no value.".to_owned(),
        (Locale::En, missing) => format!(" {missing} years have no value."),
        (Locale::De, 1) => " 1 Jahr hat keinen Wert.".to_owned(),
        (Locale::De, missing) => format!(" {missing} Jahre haben keinen Wert."),
    }
}

/// The opening sentences of a calendar: its year, its arrangement, its scale and its coverage.
pub(crate) fn calendar_opening(
    locale: Locale,
    year: i32,
    weeks: bool,
    scale: &str,
    days: usize,
    length: i64,
) -> String {
    match locale {
        Locale::En => format!(
            "Calendar of {year} with {}, {scale}. {days} of {length} days {} a value.",
            if weeks {
                "one row per weekday and one column per week"
            } else {
                "one row per month"
            },
            plural(days, "has", "have")
        ),
        Locale::De => format!(
            "Kalender {year} mit {}, {scale}. {days} von {length} Tagen {} einen Wert.",
            if weeks {
                "einer Zeile pro Wochentag und einer Spalte pro Woche"
            } else {
                "einer Zeile pro Monat"
            },
            plural(days, "hat", "haben")
        ),
    }
}

/// The opening sentences of a range chart: its spans and their outermost ends, each given as
/// value and label.
pub(crate) fn rangebar_opening(
    locale: Locale,
    categories: usize,
    mid: bool,
    lowest: (&str, &str),
    highest: (&str, &str),
) -> String {
    match locale {
        Locale::En => format!(
            "Range chart with {categories} {}, each a span from low to high{}. Lowest low: {} ({}). Highest high: {} ({}).",
            plural(categories, "category", "categories"),
            if mid { " and a central value" } else { "" },
            lowest.0,
            lowest.1,
            highest.0,
            highest.1,
        ),
        Locale::De => format!(
            "Spannweitendiagramm mit {categories} {}, jeweils eine Spanne vom unteren zum oberen Wert{}. Niedrigster unterer Wert: {} ({}). Höchster oberer Wert: {} ({}).",
            plural(categories, "Kategorie", "Kategorien"),
            if mid { " mit einem mittleren Wert" } else { "" },
            lowest.0,
            lowest.1,
            highest.0,
            highest.1,
        ),
    }
}

/// The sentence that names the modeled spans of a range chart.
pub(crate) fn modeled_ranges(locale: Locale, labels: &str) -> String {
    match locale {
        Locale::En => format!(" Modeled, drawn hatched: {labels}."),
        Locale::De => format!(" Modelliert, schraffiert gezeichnet: {labels}."),
    }
}

/// The opening sentences of a topic map: its areas and islands, and the largest and smallest
/// topic, each given as label and value.
pub(crate) fn topicmap_opening(
    locale: Locale,
    areas: usize,
    islands: usize,
    largest: (&str, &str),
    smallest: (&str, &str),
) -> String {
    match locale {
        Locale::En => format!(
            "Topic map with {areas} area{}{}. Largest: {} with {} entries, smallest: {} with {}.",
            plural(areas, "", "s"),
            match islands {
                0 => String::new(),
                1 => " and one island".to_owned(),
                islands => format!(" and {islands} islands"),
            },
            largest.0,
            largest.1,
            smallest.0,
            smallest.1,
        ),
        Locale::De => format!(
            "Themenkarte mit {areas} {}{}. Größtes: {} mit {} Einträgen, kleinstes: {} mit {}.",
            plural(areas, "Gebiet", "Gebieten"),
            match islands {
                0 => String::new(),
                1 => " und einer Insel".to_owned(),
                islands => format!(" und {islands} Inseln"),
            },
            largest.0,
            largest.1,
            smallest.0,
            smallest.1,
        ),
    }
}

/// The sentence that names the closest neighbours of a topic map, as pairs of labels.
pub(crate) fn neighbours(locale: Locale, pairs: &[(&str, &str)]) -> String {
    let and = match locale {
        Locale::En => "and",
        Locale::De => "und",
    };
    let named = pairs
        .iter()
        .map(|(from, to)| format!("{from} {and} {to}"))
        .collect::<Vec<_>>()
        .join(", ");
    match locale {
        Locale::En => format!(" Closest neighbours: {named}."),
        Locale::De => format!(" Engste Nachbarn: {named}."),
    }
}

/// The description of a knowledge landscape; `largest` is the label and value of its largest
/// region.
pub(crate) fn atlas_sentence(
    locale: Locale,
    realms: usize,
    regions: usize,
    largest: (&str, &str),
    places: usize,
) -> String {
    match locale {
        Locale::En => format!(
            "Knowledge landscape of {realms} {} and {regions} {}; largest is {} with {}.{}",
            plural(realms, "realm", "realms"),
            plural(regions, "region", "regions"),
            largest.0,
            largest.1,
            match places {
                0 => String::new(),
                1 => " 1 place is marked.".to_owned(),
                places => format!(" {places} places are marked."),
            }
        ),
        Locale::De => format!(
            "Wissenslandschaft aus {realms} {} und {regions} {}; die größte ist {} mit {}.{}",
            plural(realms, "Bereich", "Bereichen"),
            plural(regions, "Region", "Regionen"),
            largest.0,
            largest.1,
            match places {
                0 => String::new(),
                1 => " 1 Ort ist markiert.".to_owned(),
                places => format!(" {places} Orte sind markiert."),
            }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        atlas_sentence, calendar_opening, categories_opening, equal_values, missing_values,
        time_opening, topicmap_opening, years_without_value,
    };
    use crate::spec::Locale;

    #[test]
    fn german_counts_of_one_are_singular() {
        assert_eq!(
            equal_values(Locale::De, false, 1, "5"),
            "Balkendiagramm mit 1 Wert: 5."
        );
        assert_eq!(
            time_opening(Locale::De, 1, "2024", &[], false),
            "Zeitreihe mit 1 Punkt 2024."
        );
        assert_eq!(
            time_opening(Locale::De, 2, "2024", &[], false),
            "Zeitreihe mit 2 Punkten 2024."
        );
    }

    #[test]
    fn english_counts_of_one_are_singular() {
        assert_eq!(
            equal_values(Locale::En, false, 1, "5"),
            "Bar chart with 1 value: 5."
        );
        assert_eq!(
            categories_opening(Locale::En, false, 1, 1, ""),
            "Bar chart with 1 category."
        );
        assert_eq!(
            categories_opening(Locale::En, true, 1, 2, "A, B"),
            "Line chart with 1 category and 2 series (A, B)."
        );
        assert_eq!(
            time_opening(Locale::En, 1, "in 2024", &[], false),
            "Time chart with 1 point in 2024."
        );
        assert_eq!(
            time_opening(Locale::En, 1, "in 2024", &[], true),
            "Time chart with 1 point in 2024, modeled."
        );
        assert_eq!(
            time_opening(
                Locale::En,
                1,
                "in 2024",
                &["A".to_owned(), "B".to_owned()],
                false
            ),
            "Time chart with 1 point in 2024 and 2 series (A, B)."
        );
        assert!(
            calendar_opening(Locale::En, 2024, true, "scale", 1, 366)
                .ends_with(" 1 of 366 days has a value.")
        );
        assert_eq!(
            atlas_sentence(Locale::En, 1, 1, ("A", "3"), 1),
            "Knowledge landscape of 1 realm and 1 region; largest is A with 3. 1 place is marked."
        );
        assert!(
            topicmap_opening(Locale::En, 1, 1, ("A", "3"), ("A", "3"))
                .starts_with("Topic map with 1 area and one island.")
        );
        assert_eq!(missing_values(Locale::En, 1), " 1 value is missing.");
        assert_eq!(years_without_value(Locale::En, 1), " 1 year has no value.");
    }

    #[test]
    fn english_counts_of_many_stay_plural() {
        assert_eq!(
            categories_opening(Locale::En, false, 3, 1, ""),
            "Bar chart with 3 categories."
        );
        assert_eq!(
            time_opening(Locale::En, 3, "in 2024", &[], false),
            "Time chart with 3 points in 2024."
        );
        assert!(
            calendar_opening(Locale::En, 2024, false, "scale", 2, 366)
                .ends_with(" 2 of 366 days have a value.")
        );
        assert_eq!(
            atlas_sentence(Locale::En, 2, 3, ("A", "3"), 4),
            "Knowledge landscape of 2 realms and 3 regions; largest is A with 3. 4 places are marked."
        );
    }
}
