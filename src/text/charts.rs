//! The words of charts along an axis or in panels: time, bars, stripes, calendars, candles,
//! ranges, timelines, waterfalls, waffles, parliaments, treemaps and Sankey diagrams.

use super::plural;
use crate::spec::Locale;
use std::fmt::Write as _;

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
            format!("Time chart with {points} {point} {range}, modelled.")
        }
        (Locale::De, false) if modeled => {
            format!("Zeitreihe mit {points} {punkt} {range}, modelliert.")
        }
        (Locale::En, false) => format!("Time chart with {points} {point} {range}."),
        (Locale::De, false) => format!("Zeitreihe mit {points} {punkt} {range}."),
    }
}

/// An opening sentence for a chart along a numeric axis, which is a line chart rather than a time
/// chart: the noun is replaced, the rest stays.
pub(crate) fn axis_noun(locale: Locale, zone: crate::time::TimeZone, opening: String) -> String {
    if !zone.is_numeric() {
        return opening;
    }
    let (time, line) = match locale {
        Locale::En => ("Time chart", "Line chart"),
        Locale::De => ("Zeitreihe", "Liniendiagramm"),
    };
    match opening.strip_prefix(time) {
        Some(rest) => format!("{line}{rest}"),
        None => opening,
    }
}

/// The opening sentence of small multiples: a time chart in panels, laid out in a grid.
pub(crate) fn multiples_opening(
    locale: Locale,
    panels: &[String],
    range: &str,
    names: &[String],
    shared: bool,
) -> String {
    let axis = match (locale, shared) {
        (Locale::En, true) => "a shared value axis",
        (Locale::En, false) => "a value axis of their own",
        (Locale::De, true) => "gemeinsamer Werteachse",
        (Locale::De, false) => "je eigener Werteachse",
    };
    let mut text = match locale {
        Locale::En => format!(
            "Time chart in {} {} ({}) with {axis}, each {range}",
            panels.len(),
            plural(panels.len(), "panel", "panels"),
            panels.join(", ")
        ),
        Locale::De => format!(
            "Zeitreihe in {} {} ({}) mit {axis}, jeweils {range}",
            panels.len(),
            plural(panels.len(), "Feld", "Feldern"),
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
            " {} a shaded band from {} lower to {} upper bound{}; the data table lists both.",
            plural(banded, "One line has", "Lines have"),
            plural(banded, "its", "their"),
            plural(banded, "its", "their"),
            if hatched {
                ", hatched where modelled"
            } else {
                ""
            }
        ),
        Locale::De => format!(
            " {} ein schattiertes Band von der unteren bis zur oberen Grenze{}; die Datentabelle nennt beide.",
            plural(banded, "Eine Linie hat", "Linien haben je"),
            if hatched {
                ", schraffiert, wo modelliert"
            } else {
                ""
            }
        ),
    }
}

/// The opening sentence of small multiples of bars: a bar chart in panels, laid out in a grid.
pub(crate) fn panel_bars_opening(locale: Locale, panels: usize, categories: usize) -> String {
    match locale {
        Locale::En => format!(
            "Bar chart in {panels} {} over {categories} {}, each with its own value axis.",
            plural(panels, "panel", "panels"),
            plural(categories, "category", "categories")
        ),
        Locale::De => format!(
            "Balkendiagramm in {panels} {} über {categories} {}, jedes mit eigener Wertachse.",
            plural(panels, "Feld", "Feldern"),
            plural(categories, "Kategorie", "Kategorien")
        ),
    }
}

/// The notes under the panel titles of small multiples, as `title: note` separated by semicolons.
pub(crate) fn panel_notes(locale: Locale, notes: &str) -> String {
    match locale {
        Locale::En => format!(" Notes: {notes}."),
        Locale::De => format!(" Hinweise: {notes}."),
    }
}

/// Stacked areas, named from the bottom up.
pub(crate) fn stacked_areas(locale: Locale, names: &str) -> String {
    match locale {
        Locale::En => {
            format!(" Stacked areas, from the bottom up: {names}; the top edge shows their total.")
        }
        Locale::De => format!(
            " Gestapelte Flächen, von unten nach oben: {names}; die Oberkante zeigt ihre Summe."
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

/// The sentence about a stack by value: its highest and lowest total, with their categories.
pub(crate) fn stacked_totals(
    locale: Locale,
    (highest, highest_at): (&str, &str),
    (lowest, lowest_at): (&str, &str),
) -> String {
    match locale {
        Locale::En => format!(
            " The series are stacked. Highest total: {highest} ({highest_at}). Lowest total: {lowest} ({lowest_at})."
        ),
        Locale::De => format!(
            " Die Reihen sind gestapelt. Höchste Summe: {highest} ({highest_at}). Niedrigste Summe: {lowest} ({lowest_at})."
        ),
    }
}

/// The sentence about a percent stack: every bar shows shares of its category's total.
pub(crate) const fn stacked_shares(locale: Locale) -> &'static str {
    match locale {
        Locale::En => " Each bar shows the shares of the series in its category's total.",
        Locale::De => " Jeder Balken zeigt die Anteile der Reihen an der Summe seiner Kategorie.",
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
        Locale::En if first == last => format!("Warming stripes for {first}, one stripe, {scale}."),
        Locale::De if first == last => format!("Wärmestreifen für {first}, ein Streifen, {scale}."),
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

/// The groups of a range bar chart, each with the labels of its spans, as `group: labels`
/// separated by semicolons.
pub(crate) fn range_groups(locale: Locale, groups: &str) -> String {
    match locale {
        Locale::En => format!(" Colors show groups: {groups}."),
        Locale::De => format!(" Farben zeigen Gruppen: {groups}."),
    }
}

/// The sentence that names the modeled spans of a range chart.
pub(crate) fn modeled_ranges(locale: Locale, labels: &str) -> String {
    match locale {
        Locale::En => format!(" Modelled, drawn hatched: {labels}."),
        Locale::De => format!(" Modelliert, schraffiert gezeichnet: {labels}."),
    }
}

/// The sentence that says what the error bars of a bar or line chart show.
pub(crate) fn error_bars(locale: Locale, line: bool) -> String {
    match (locale, line) {
        (Locale::En, false) => " Error bars show the interval from lower to upper; the table gives it for every bar.".to_owned(),
        (Locale::En, true) => " Error bars show the interval from lower to upper; the table gives it for every point.".to_owned(),
        (Locale::De, false) => " Fehlerbalken zeigen das Intervall von unten bis oben; die Tabelle nennt es für jeden Balken.".to_owned(),
        (Locale::De, true) => " Fehlerbalken zeigen das Intervall von unten bis oben; die Tabelle nennt es für jeden Punkt.".to_owned(),
    }
}

/// The overall results among the spans of a range chart, drawn as diamonds.
pub(crate) fn summary_ranges(locale: Locale, summaries: &str) -> String {
    match locale {
        Locale::En => format!(" Overall result, drawn as a diamond: {summaries}."),
        Locale::De => format!(" Gesamtergebnis, als Raute gezeichnet: {summaries}."),
    }
}

/// What the squares of a range chart mean.
pub(crate) fn weighted_ranges(locale: Locale) -> String {
    match locale {
        Locale::En => " The area of each square follows the weight of its span.".to_owned(),
        Locale::De => " Die Fläche jedes Quadrats folgt dem Gewicht seiner Spanne.".to_owned(),
    }
}

/// The opening sentence of a timeline: its phases and milestones, and the days it spans.
pub(crate) fn timeline_opening(
    locale: Locale,
    (phases, milestones): (usize, usize),
    (first, last): (&str, &str),
) -> String {
    let days = match locale {
        Locale::En if first == last => format!("on {first}"),
        Locale::De if first == last => format!("am {first}"),
        Locale::En => format!("from {first} to {last}"),
        Locale::De => format!("vom {first} bis zum {last}"),
    };
    match locale {
        Locale::En => format!(
            "Timeline with {phases} phase{} and {milestones} milestone{} {days}.",
            plural(phases, "", "s"),
            plural(milestones, "", "s"),
        ),
        Locale::De => format!(
            "Zeitstrahl mit {phases} {} und {milestones} {} {days}.",
            plural(phases, "Phase", "Phasen"),
            plural(milestones, "Meilenstein", "Meilensteinen"),
        ),
    }
}

/// One item of a timeline as a sentence: its days, and what it follows.
pub(crate) fn timeline_item(locale: Locale, label: &str, days: &str, follows: &[&str]) -> String {
    let after = match (locale, follows.is_empty()) {
        (_, true) => String::new(),
        (Locale::En, false) => format!(", after {}", follows.join(" and ")),
        (Locale::De, false) => format!(", nach {}", follows.join(" und ")),
    };
    format!("{label}: {days}{after}.")
}

/// The days that matter, drawn as lines across the timeline.
pub(crate) fn timeline_markers(locale: Locale, markers: &str) -> String {
    match locale {
        Locale::En => format!(" Marked days: {markers}."),
        Locale::De => format!(" Markierte Tage: {markers}."),
    }
}

/// What a waterfall shows: its steps, where it ends, and its biggest rise and fall.
pub(crate) fn waterfall_summary(
    locale: Locale,
    (steps, end): (usize, &str),
    rise: Option<(&str, String)>,
    fall: Option<(&str, String)>,
) -> String {
    let (rise_word, fall_word) = match locale {
        Locale::En => ("Biggest rise", "Biggest fall"),
        Locale::De => ("Größter Anstieg", "Größter Rückgang"),
    };
    let mut text = match locale {
        Locale::En => format!("Waterfall with {steps} steps, ending at {end}."),
        Locale::De => format!("Wasserfall mit {steps} Schritten, endend bei {end}."),
    };
    for (word, entry) in [(rise_word, rise), (fall_word, fall)] {
        if let Some((label, value)) = entry {
            write!(text, " {word}: {label} ({value}).").expect("writing to String cannot fail");
        }
    }
    text
}

/// What a slope chart shows: the series compared between two categories, each from its first
/// value to its second, and the biggest rise and fall.
pub(crate) fn slope_summary(
    locale: Locale,
    (series, from, to): (usize, &str, &str),
    changes: &str,
    (rise, fall): (Option<&str>, Option<&str>),
) -> String {
    let (rise_word, fall_word) = match locale {
        Locale::En => ("Largest rise", "Largest fall"),
        Locale::De => ("Größter Anstieg", "Größter Rückgang"),
    };
    let mut ends = String::new();
    for (word, name) in [(rise_word, rise), (fall_word, fall)] {
        if let Some(name) = name {
            write!(ends, " {word}: {name}.").expect("writing to String cannot fail");
        }
    }
    match locale {
        Locale::En => {
            format!("Slope chart of {series} series from {from} to {to}: {changes}.{ends}")
        }
        Locale::De => {
            format!("Steigungsdiagramm von {series} Reihen von {from} nach {to}: {changes}.{ends}")
        }
    }
}

/// What a rank chart shows: the rank of every series in every category, 1 for the largest value,
/// and the biggest rise and fall in rank between the first category and the last.
pub(crate) fn rank_summary(
    locale: Locale,
    (series, categories, from, to): (usize, usize, &str, &str),
    ranks: &str,
    (rise, fall): (Option<&str>, Option<&str>),
) -> String {
    let (rise_word, fall_word) = match locale {
        Locale::En => ("Largest rise in rank", "Largest fall in rank"),
        Locale::De => ("Größter Aufstieg im Rang", "Größter Abstieg im Rang"),
    };
    let mut ends = String::new();
    for (word, name) in [(rise_word, rise), (fall_word, fall)] {
        if let Some(name) = name {
            write!(ends, " {word}: {name}.").expect("writing to String cannot fail");
        }
    }
    match locale {
        Locale::En => format!(
            "Rank chart of {series} series in {categories} categories from {from} to {to}; rank 1 is the largest value. Ranks per category: {ranks}.{ends}"
        ),
        Locale::De => format!(
            "Rangdiagramm von {series} Reihen in {categories} Kategorien von {from} bis {to}; Rang 1 ist der größte Wert. Ränge je Kategorie: {ranks}.{ends}"
        ),
    }
}

/// What a heatmap of values shows: its size, what the colors stand for, both ends of the scale
/// with the cell that holds them, and the cells without a value.
pub(crate) fn heatmap_values(
    locale: Locale,
    (rows, columns): (usize, usize),
    name: &str,
    ((low, lowest), (high, highest)): ((&str, &str), (&str, &str)),
    missing: usize,
) -> String {
    let rest = match (locale, missing) {
        (_, 0) => String::new(),
        (Locale::En, missing) => format!(" {missing} without a value."),
        (Locale::De, missing) => format!(" {missing} ohne Wert."),
    };
    match locale {
        Locale::En => format!(
            "Heatmap of {rows} rows by {columns} columns. Color shows {name}, from {low} (lowest: {lowest}) to {high} (highest: {highest}).{rest}"
        ),
        Locale::De => format!(
            "Heatmap aus {rows} Zeilen und {columns} Spalten. Die Farbe zeigt {name}, von {low} (am niedrigsten: {lowest}) bis {high} (am höchsten: {highest}).{rest}"
        ),
    }
}

/// What a heatmap of categories shows: its size, how many cells each category has, and the cells
/// without a value.
pub(crate) fn heatmap_categories(
    locale: Locale,
    (rows, columns): (usize, usize),
    counts: &str,
    missing: usize,
) -> String {
    let rest = match (locale, missing) {
        (_, 0) => String::new(),
        (Locale::En, missing) => format!(" {missing} without a value."),
        (Locale::De, missing) => format!(" {missing} ohne Wert."),
    };
    match locale {
        Locale::En => format!(
            "Heatmap of {rows} rows by {columns} columns, each cell one category. Cells by category: {counts}.{rest}"
        ),
        Locale::De => format!(
            "Heatmap aus {rows} Zeilen und {columns} Spalten, jede Zelle eine Kategorie. Zellen je Kategorie: {counts}.{rest}"
        ),
    }
}

/// What the percentages between the bars of a funnel mean.
pub(crate) const fn funnel_note(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Between the bars: the share kept from the stage before.",
        Locale::De => "Zwischen den Balken: der Anteil der vorigen Stufe.",
    }
}

/// What a funnel shows: its stages, both ends, the share that is left, every stage with its rate,
/// and the biggest drop.
pub(crate) fn funnel_summary(
    locale: Locale,
    stages: usize,
    ((first, first_value), (last, last_value)): ((&str, &str), (&str, &str)),
    overall: &str,
    stages_list: &str,
    drop: Option<(&str, &str, &str)>,
) -> String {
    let drop = match (locale, drop) {
        (_, None) => String::new(),
        (Locale::En, Some((from, to, rate))) => {
            format!(" The biggest drop is from {from} to {to}: {rate} remain.")
        }
        (Locale::De, Some((from, to, rate))) => {
            format!(" Der größte Verlust liegt zwischen {from} und {to}: {rate} bleiben.")
        }
    };
    match locale {
        Locale::En => format!(
            "Funnel of {stages} stages, from {first} ({first_value}) to {last} ({last_value}): {overall} of the first stage remain. Stages, with the share of the stage before: {stages_list}.{drop}"
        ),
        Locale::De => format!(
            "Trichter aus {stages} Stufen, von {first} ({first_value}) bis {last} ({last_value}): {overall} der ersten Stufe bleiben. Stufen, mit dem Anteil der vorigen Stufe: {stages_list}.{drop}"
        ),
    }
}

/// What a pyramid shows: its levels from the top, with their values if it has them.
pub(crate) fn pyramid_summary(locale: Locale, levels: usize, list: &str) -> String {
    match locale {
        Locale::En => format!("Pyramid of {levels} levels, from the top: {list}."),
        Locale::De => format!("Pyramide aus {levels} Ebenen, von oben: {list}."),
    }
}

/// What a radar chart shows: its axes and how many series it draws.
pub(crate) fn radar_summary(locale: Locale, (axes, series): (usize, usize), list: &str) -> String {
    match locale {
        Locale::En => {
            format!("Radar chart of {axes} axes ({list}) with {series} series.")
        }
        Locale::De => format!(
            "Netzdiagramm aus {axes} Achsen ({list}) mit {series} {}.",
            plural(series, "Reihe", "Reihen")
        ),
    }
}

/// Where one series of a radar chart stands highest and where lowest.
pub(crate) fn radar_series(
    locale: Locale,
    name: &str,
    ((strongest, high), (weakest, low)): ((&str, &str), (&str, &str)),
) -> String {
    match locale {
        Locale::En => {
            format!("{name}: strongest on {strongest} ({high}), weakest on {weakest} ({low}).")
        }
        Locale::De => format!(
            "{name}: am stärksten bei {strongest} ({high}), am schwächsten bei {weakest} ({low})."
        ),
    }
}

/// The words after a rate in the tooltip of a funnel stage.
pub(crate) const fn of_previous_stage(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "of the stage before",
        Locale::De => "der vorigen Stufe",
    }
}

/// What a polar chart shows: its categories, the highest and the lowest, and the outer ring.
pub(crate) fn polar_summary(
    locale: Locale,
    (points, labels): (usize, &str),
    ((high, highest), (low, lowest)): ((&str, &str), (&str, &str)),
    (name, ring): (&str, &str),
) -> String {
    match locale {
        Locale::En => format!(
            "Polar chart of {points} categories, from the top clockwise: {labels}. Highest: {high} ({highest}); lowest: {low} ({lowest}). {name} runs from 0 at the center to {ring} at the outer ring."
        ),
        Locale::De => format!(
            "Polardiagramm aus {points} Kategorien, von oben im Uhrzeigersinn: {labels}. Höchster Wert: {high} ({highest}); niedrigster: {low} ({lowest}). {name} reicht von 0 in der Mitte bis {ring} am äußeren Ring."
        ),
    }
}

/// What a mosaic chart shows: its size and, for every column, its share of the whole and the share
/// of its segments.
pub(crate) fn mosaic_summary(
    locale: Locale,
    (columns, segments, total): (usize, usize, &str),
    list: &str,
) -> String {
    match locale {
        Locale::En => format!(
            "Mosaic chart of {columns} columns and {segments} segments, {total} in all. The width of a column is its share of the whole; its segments are shares of the column. {list}."
        ),
        Locale::De => format!(
            "Mosaikdiagramm aus {columns} Spalten und {segments} Segmenten, insgesamt {total}. Die Breite einer Spalte ist ihr Anteil am Ganzen; ihre Segmente sind Anteile der Spalte. {list}."
        ),
    }
}

/// The tooltip of a segment of a mosaic chart.
pub(crate) fn mosaic_tooltip(
    locale: Locale,
    (column, segment, value): (&str, &str, &str),
    (of_column, of_whole): (&str, &str),
) -> String {
    match locale {
        Locale::En => {
            format!(
                "{column}, {segment}: {value} ({of_column} of the column, {of_whole} of the whole)"
            )
        }
        Locale::De => {
            format!("{column}, {segment}: {value} ({of_column} der Spalte, {of_whole} des Ganzen)")
        }
    }
}

/// What the size of the points of a scatter plot shows: its name, both ends and the largest point.
pub(crate) fn scatter_sizes(
    locale: Locale,
    name: &str,
    (low, high): (&str, &str),
    largest: &str,
) -> String {
    match locale {
        Locale::En => format!(
            "The area of a point shows {name}, from {low} to {high}; the largest is {largest}."
        ),
        Locale::De => format!(
            "Die Fläche eines Punktes zeigt {name}, von {low} bis {high}; am größten ist {largest}."
        ),
    }
}

/// What a waffle shows: its squares, what one stands for, and every part.
pub(crate) fn waffle_summary(locale: Locale, (squares, each): (u32, &str), parts: &str) -> String {
    match locale {
        Locale::En => format!(
            "Waffle chart of {squares} squares, each standing for {each}. Parts, as value (share, squares): {parts}."
        ),
        Locale::De => format!(
            "Waffeldiagramm aus {squares} Feldern, jedes für {each}. Teile als Wert (Anteil, Felder): {parts}."
        ),
    }
}

/// What a parliament chart shows: its seats and parties, with every party, and the majority.
pub(crate) fn parliament_summary(
    locale: Locale,
    (seats, parties): (u32, usize),
    list: &str,
    majority: Option<u32>,
) -> String {
    let majority = match (locale, majority) {
        (_, None) => String::new(),
        (Locale::En, Some(seats)) => format!(" A majority takes {seats} seats."),
        (Locale::De, Some(seats)) => format!(" Eine Mehrheit braucht {seats} Sitze."),
    };
    match locale {
        Locale::En => format!(
            "Parliament with {seats} seats and {parties} part{}: {list}.{majority}",
            plural(parties, "y", "ies")
        ),
        Locale::De => format!(
            "Parlament mit {seats} Sitzen und {parties} {}: {list}.{majority}",
            plural(parties, "Partei", "Parteien")
        ),
    }
}

/// The parties that govern together, their seats and whether those are a majority.
pub(crate) fn parliament_coalition(
    locale: Locale,
    parties: &str,
    seats: u32,
    majority: bool,
) -> String {
    match (locale, majority) {
        (Locale::En, true) => format!("Coalition {parties}: {seats} seats, a majority."),
        (Locale::En, false) => format!("Coalition {parties}: {seats} seats, no majority."),
        (Locale::De, true) => format!("Koalition {parties}: {seats} Sitze, eine Mehrheit."),
        (Locale::De, false) => format!("Koalition {parties}: {seats} Sitze, keine Mehrheit."),
    }
}

/// How much of a phase is done, as a percentage.
pub(crate) fn percent_done(locale: Locale, progress: f64) -> String {
    let percent = (progress * 100.0).round();
    match locale {
        Locale::En => format!("{percent}% done"),
        Locale::De => format!("{percent} % erledigt"),
    }
}

/// What a Sankey diagram shows: its nodes, links and total, and its biggest links.
pub(crate) fn sankey_summary(
    locale: Locale,
    (nodes, links): (usize, usize),
    total: &str,
    largest: &str,
) -> String {
    match locale {
        Locale::En => format!(
            "Sankey diagram of {nodes} nodes and {links} link{}; {total} enter it. Biggest links: {largest}.",
            plural(links, "", "s")
        ),
        Locale::De => format!(
            "Sankey-Diagramm aus {nodes} Knoten und {links} {}; {total} fließen hinein. Größte Verbindungen: {largest}.",
            plural(links, "Verbindung", "Verbindungen")
        ),
    }
}

/// What a treemap shows: its items and total, the biggest ones, and the shares of its groups.
pub(crate) fn treemap_summary(
    locale: Locale,
    (items, total): (usize, &str),
    largest: &str,
    groups: &str,
) -> String {
    let groups = match (locale, groups.is_empty()) {
        (_, true) => String::new(),
        (Locale::En, false) => format!(" Groups: {groups}."),
        (Locale::De, false) => format!(" Gruppen: {groups}."),
    };
    match locale {
        Locale::En => format!(
            "Treemap of {items} item{}, together {total}; the rectangle of each has the area of its value. Largest: {largest}.{groups}",
            plural(items, "", "s")
        ),
        Locale::De => format!(
            "Treemap aus {items} {}, zusammen {total}; die Fläche jedes Rechtecks folgt seinem Wert. Am größten: {largest}.{groups}",
            plural(items, "Eintrag", "Einträgen")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{bands, multiples_opening, panel_bars_opening};
    use crate::spec::Locale;

    #[test]
    fn panels_are_named_in_plain_words() {
        let two = ["A".to_owned(), "B".to_owned()];
        assert_eq!(
            multiples_opening(Locale::En, &two, "from 2000 to 2050", &[], true),
            "Time chart in 2 panels (A, B) with a shared value axis, each from 2000 to 2050."
        );
        assert_eq!(
            multiples_opening(Locale::De, &two, "von 2000 bis 2050", &[], false),
            "Zeitreihe in 2 Feldern (A, B) mit je eigener Werteachse, jeweils von 2000 bis 2050."
        );
        let one = ["A".to_owned()];
        assert!(
            multiples_opening(Locale::En, &one, "from 2000 to 2050", &[], true)
                .starts_with("Time chart in 1 panel (A)")
        );
        assert!(
            multiples_opening(Locale::De, &one, "von 2000 bis 2050", &[], true)
                .starts_with("Zeitreihe in 1 Feld (A)")
        );
        assert_eq!(
            panel_bars_opening(Locale::En, 2, 3),
            "Bar chart in 2 panels over 3 categories, each with its own value axis."
        );
        assert_eq!(
            panel_bars_opening(Locale::De, 2, 1),
            "Balkendiagramm in 2 Feldern über 1 Kategorie, jedes mit eigener Wertachse."
        );
    }

    #[test]
    fn bands_agree_with_the_number_of_lines() {
        assert_eq!(
            bands(Locale::En, 1, true),
            " One line has a shaded band from its lower to its upper bound, hatched where modelled; the data table lists both."
        );
        assert_eq!(
            bands(Locale::En, 2, false),
            " Lines have a shaded band from their lower to their upper bound; the data table lists both."
        );
        assert!(bands(Locale::De, 2, false).starts_with(" Linien haben je ein schattiertes Band"));
    }
}
