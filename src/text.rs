//! The words chartlet writes itself, per locale: descriptions, legend additions, tooltips and
//! the HTML figure. Text from the specification is never translated.

use std::fmt::Write as _;

use crate::spec::{ComponentKind, FragmentSpec, Locale, MessageKind, NodeKind, ParticipantKind};

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
    /// The first column of a chart along a numeric axis without a title.
    pub position: &'static str,
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
    /// The columns of a sequence diagram's table.
    pub number: &'static str,
    pub sender: &'static str,
    pub receiver: &'static str,
    pub message: &'static str,
    pub message_kind: &'static str,
    pub fragment: &'static str,
    /// The columns of a flow chart's table.
    pub step: &'static str,
    pub lane: &'static str,
    pub group: &'static str,
    pub leads_to: &'static str,
    /// The columns of a state diagram's table, and the name of its initial dot.
    pub event: &'static str,
    pub guard: &'static str,
    pub action: &'static str,
    pub next_state: &'static str,
    pub start: &'static str,
    /// The columns of an architecture diagram's table.
    pub component: &'static str,
    pub boundary: &'static str,
    pub connects_to: &'static str,
    /// The columns of a tree's table.
    pub node: &'static str,
    pub level: &'static str,
    pub parent: &'static str,
    pub link: &'static str,
    pub children: &'static str,
    pub partner: &'static str,
    /// The columns of a boxplot's table.
    pub minimum: &'static str,
    pub quartile_1: &'static str,
    pub median: &'static str,
    pub quartile_3: &'static str,
    pub maximum: &'static str,
    pub outliers: &'static str,
    pub observations: &'static str,
    pub interval: &'static str,
    pub weight: &'static str,
    pub summary: &'static str,
    /// The columns of a timeline's table.
    pub item: &'static str,
    pub phase: &'static str,
    pub milestone: &'static str,
    pub begin: &'static str,
    pub finish: &'static str,
    pub follows: &'static str,
    /// The names of a diagram's focus links, and of the link that clears the focus.
    pub focus: &'static str,
    pub show_all: &'static str,
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
    position: "Position",
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
    number: "No.",
    sender: "From",
    receiver: "To",
    message: "Message",
    message_kind: "Kind",
    fragment: "Fragment",
    step: "Step",
    lane: "Lane",
    group: "Group",
    leads_to: "Leads to",
    event: "Event",
    guard: "Guard",
    action: "Action",
    next_state: "To",
    start: "Start",
    component: "Component",
    node: "Node",
    level: "Level",
    parent: "Parent",
    link: "Link",
    children: "Children",
    partner: "Partner",
    minimum: "Minimum",
    quartile_1: "Q1",
    median: "Median",
    quartile_3: "Q3",
    maximum: "Maximum",
    outliers: "Outliers",
    observations: "Observations",
    interval: "Interval",
    weight: "Weight",
    summary: "summary",
    item: "Item",
    phase: "Phase",
    milestone: "Milestone",
    begin: "Start",
    finish: "End",
    follows: "After",
    boundary: "Boundary",
    connects_to: "Connects to",
    focus: "Focus",
    show_all: "Show all",
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
    position: "Position",
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
    number: "Nr.",
    sender: "Von",
    receiver: "An",
    message: "Nachricht",
    message_kind: "Art",
    fragment: "Abschnitt",
    step: "Schritt",
    lane: "Bahn",
    group: "Gruppe",
    leads_to: "Führt zu",
    event: "Ereignis",
    guard: "Bedingung",
    action: "Aktion",
    next_state: "Nach",
    start: "Start",
    component: "Komponente",
    node: "Knoten",
    level: "Ebene",
    parent: "Übergeordnet",
    link: "Verknüpfung",
    children: "Untergeordnet",
    partner: "Partner",
    minimum: "Minimum",
    quartile_1: "Q1",
    median: "Median",
    quartile_3: "Q3",
    maximum: "Maximum",
    outliers: "Ausreißer",
    observations: "Beobachtungen",
    interval: "Intervall",
    weight: "Gewicht",
    summary: "Gesamt",
    item: "Eintrag",
    phase: "Phase",
    milestone: "Meilenstein",
    begin: "Beginn",
    finish: "Ende",
    follows: "Nach",
    boundary: "Grenze",
    connects_to: "Verbunden mit",
    focus: "Fokus",
    show_all: "Alle zeigen",
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

/// The opening sentence of small multiples.
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
            "Small multiples of {} panels ({}) with {axis}, each {range}",
            panels.len(),
            panels.join(", ")
        ),
        Locale::De => format!(
            "Kleine Vielfache aus {} Feldern ({}) mit {axis}, jeweils {range}",
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

/// The opening sentence of small multiples of bars.
pub(crate) fn panel_bars_opening(locale: Locale, panels: usize, categories: usize) -> String {
    match locale {
        Locale::En => format!(
            "Small multiples of bars: {panels} panels over {categories} categories, each with its own value axis."
        ),
        Locale::De => format!(
            "Kleine Vielfache aus Balken: {panels} Felder über {categories} Kategorien, jedes mit eigener Wertachse."
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

/// A participant of a sequence diagram by its label, and by its kind where that is not a plain
/// system.
pub(crate) fn participant(locale: Locale, label: &str, kind: ParticipantKind) -> String {
    let kind = match (locale, kind) {
        (_, ParticipantKind::Service) => return label.to_owned(),
        (Locale::En, ParticipantKind::Actor) => "actor",
        (Locale::En, ParticipantKind::Database) => "database",
        (Locale::En, ParticipantKind::Queue) => "queue",
        (Locale::En, ParticipantKind::External) => "external",
        (Locale::De, ParticipantKind::Actor) => "Akteur",
        (Locale::De, ParticipantKind::Database) => "Datenbank",
        (Locale::De, ParticipantKind::Queue) => "Warteschlange",
        (Locale::De, ParticipantKind::External) => "extern",
    };
    format!("{label} ({kind})")
}

/// Names in a running list: `A`, `A and B`, `A, B and C`.
fn listed(locale: Locale, names: &[String]) -> String {
    let and = match locale {
        Locale::En => "and",
        Locale::De => "und",
    };
    match names {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} {and} {last}", rest.join(", ")),
    }
}

/// The opening sentence of a sequence diagram: its participants and how many messages follow.
pub(crate) fn sequence_opening(locale: Locale, participants: &[String], messages: usize) -> String {
    let count = participants.len();
    let names = listed(locale, participants);
    match locale {
        Locale::En => format!(
            "Sequence diagram with {count} participant{}: {names}. {messages} message{}, in order:",
            plural(count, "", "s"),
            plural(messages, "", "s"),
        ),
        Locale::De => format!(
            "Sequenzdiagramm mit {count} {}: {names}. {messages} {}, in Reihenfolge:",
            plural(count, "Beteiligtem", "Beteiligten"),
            plural(messages, "Nachricht", "Nachrichten"),
        ),
    }
}

/// One message of a sequence diagram as a sentence; `receiver` is `None` for a message to its
/// sender.
pub(crate) fn sequence_message(
    locale: Locale,
    number: usize,
    sender: &str,
    receiver: Option<&str>,
    label: &str,
    kind: MessageKind,
) -> String {
    let kind = match (locale, kind) {
        (_, MessageKind::Call) => "",
        (Locale::En, MessageKind::Reply) => " (reply)",
        (Locale::En, MessageKind::Async) => " (asynchronous)",
        (Locale::De, MessageKind::Reply) => " (Antwort)",
        (Locale::De, MessageKind::Async) => " (asynchron)",
    };
    match (locale, receiver) {
        (Locale::En, Some(receiver)) => format!("{number}. {sender} to {receiver}: {label}{kind}."),
        (Locale::En, None) => format!("{number}. {sender} to itself: {label}{kind}."),
        (Locale::De, Some(receiver)) => format!("{number}. {sender} an {receiver}: {label}{kind}."),
        (Locale::De, None) => format!("{number}. {sender} an sich selbst: {label}{kind}."),
    }
}

/// How a message is sent, for the data table.
pub(crate) const fn message_kind(locale: Locale, kind: MessageKind) -> &'static str {
    match (locale, kind) {
        (Locale::En, MessageKind::Call) => "call",
        (Locale::En, MessageKind::Reply) => "reply",
        (Locale::En, MessageKind::Async) => "asynchronous",
        (Locale::De, MessageKind::Call) => "Aufruf",
        (Locale::De, MessageKind::Reply) => "Antwort",
        (Locale::De, MessageKind::Async) => "asynchron",
    }
}

/// The branch of an `alt` fragment that has no label of its own.
pub(crate) const fn otherwise(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "else",
        Locale::De => "sonst",
    }
}

/// Messages counted from 1: `message 3` or `messages 2–4`.
fn message_range(locale: Locale, from: usize, to: usize) -> String {
    match (locale, from == to) {
        (Locale::En, true) => format!("message {}", from + 1),
        (Locale::En, false) => format!("messages {}–{}", from + 1, to + 1),
        (Locale::De, true) => format!("Nachricht {}", from + 1),
        (Locale::De, false) => format!("Nachrichten {}–{}", from + 1, to + 1),
    }
}

/// A fragment of a sequence diagram as a sentence: its kind and label, the messages it frames,
/// and where each further branch begins.
pub(crate) fn fragment(locale: Locale, fragment: &FragmentSpec) -> String {
    let label = fragment
        .label
        .as_ref()
        .map_or_else(String::new, |label| match locale {
            Locale::En => format!(" \"{label}\""),
            Locale::De => format!(" „{label}“"),
        });
    let range = message_range(locale, fragment.from, fragment.to);
    let branches: String = fragment
        .branches
        .iter()
        .map(|branch| {
            let label = branch.label.as_deref().unwrap_or(otherwise(locale));
            match locale {
                Locale::En => format!(", else \"{label}\" from message {}", branch.from + 1),
                Locale::De => format!(", sonst „{label}“ ab Nachricht {}", branch.from + 1),
            }
        })
        .collect();
    let keyword = fragment.kind.keyword();
    match locale {
        Locale::En => format!("Fragment {keyword}{label} spans {range}{branches}."),
        Locale::De => format!("Abschnitt {keyword}{label} umfasst {range}{branches}."),
    }
}

/// The opening sentence of a flow chart: how many steps and edges, and its lanes.
pub(crate) fn flow_opening(locale: Locale, steps: usize, edges: usize, lanes: &[String]) -> String {
    let lanes = match (locale, lanes.len()) {
        (_, 0) => String::new(),
        (Locale::En, count) => format!(
            " in {count} lane{}: {}",
            plural(count, "", "s"),
            listed(locale, lanes)
        ),
        (Locale::De, count) => format!(
            " in {count} {}: {}",
            plural(count, "Bahn", "Bahnen"),
            listed(locale, lanes)
        ),
    };
    match locale {
        Locale::En => format!(
            "Flow chart with {steps} step{} and {edges} connection{}{lanes}.",
            plural(steps, "", "s"),
            plural(edges, "", "s"),
        ),
        Locale::De => format!(
            "Ablaufdiagramm mit {steps} {} und {edges} {}{lanes}.",
            plural(steps, "Schritt", "Schritten"),
            plural(edges, "Verbindung", "Verbindungen"),
        ),
    }
}

/// The main path of a flow chart, as step labels.
pub(crate) fn main_path(locale: Locale, steps: &[&str]) -> String {
    let path = steps.join(" → ");
    match locale {
        Locale::En => format!("Main path: {path}."),
        Locale::De => format!("Hauptweg: {path}."),
    }
}

/// What a step of a flow chart is, for the description and the data table.
pub(crate) const fn node_kind(locale: Locale, kind: NodeKind) -> &'static str {
    match (locale, kind) {
        (Locale::En, NodeKind::Start) => "start",
        (Locale::En, NodeKind::End) => "end",
        (Locale::En, NodeKind::Process) => "step",
        (Locale::En, NodeKind::Decision) => "decision",
        (Locale::En, NodeKind::Io) => "input or output",
        (Locale::En, NodeKind::Subprocess) => "subprocess",
        (Locale::En, NodeKind::Store) => "data store",
        (Locale::En, NodeKind::External) => "external",
        (Locale::De, NodeKind::Start) => "Start",
        (Locale::De, NodeKind::End) => "Ende",
        (Locale::De, NodeKind::Process) => "Schritt",
        (Locale::De, NodeKind::Decision) => "Entscheidung",
        (Locale::De, NodeKind::Io) => "Ein- oder Ausgabe",
        (Locale::De, NodeKind::Subprocess) => "Teilprozess",
        (Locale::De, NodeKind::Store) => "Datenspeicher",
        (Locale::De, NodeKind::External) => "extern",
    }
}

/// One step of a flow chart as a sentence: its label, its kind unless it is a plain step, its
/// lane, and where it leads.
pub(crate) fn flow_step(
    locale: Locale,
    label: &str,
    kind: NodeKind,
    lane: Option<&str>,
    next: &[String],
) -> String {
    let kind = if kind == NodeKind::Process {
        String::new()
    } else {
        format!(" ({})", node_kind(locale, kind))
    };
    // "in" in both languages.
    let lane = lane.map_or_else(String::new, |lane| format!(" in {lane}"));
    let next = match (locale, next.is_empty()) {
        (Locale::En, true) => "no further step".to_owned(),
        (Locale::De, true) => "kein weiterer Schritt".to_owned(),
        (Locale::En, false) => format!("leads to {}", listed(locale, next)),
        (Locale::De, false) => format!("führt zu {}", listed(locale, next)),
    };
    format!("{label}{kind}{lane}: {next}.")
}

/// A group of a flow chart as a sentence.
pub(crate) fn flow_group(locale: Locale, label: &str, members: &[&str]) -> String {
    let members = members.join(", ");
    match locale {
        Locale::En => format!("Group \"{label}\": {members}."),
        Locale::De => format!("Gruppe „{label}“: {members}."),
    }
}

/// The opening sentence of a state diagram: how many states and transitions, where it starts and
/// which states it ends in.
pub(crate) fn state_opening(
    locale: Locale,
    states: usize,
    transitions: usize,
    initial: Option<&str>,
    finals: &[String],
) -> String {
    let mut opening = match locale {
        Locale::En => format!(
            "State diagram with {states} state{} and {transitions} transition{}.",
            plural(states, "", "s"),
            plural(transitions, "", "s"),
        ),
        Locale::De => format!(
            "Zustandsdiagramm mit {states} {} und {transitions} {}.",
            plural(states, "Zustand", "Zuständen"),
            plural(transitions, "Übergang", "Übergängen"),
        ),
    };
    if let Some(initial) = initial {
        let starts = match locale {
            Locale::En => "It starts in",
            Locale::De => "Es beginnt in",
        };
        write!(opening, " {starts} {initial}.").expect("writing to String cannot fail");
    }
    if !finals.is_empty() {
        let noun = match (locale, finals.len()) {
            (Locale::En, 1) => "Final state",
            (Locale::En, _) => "Final states",
            (Locale::De, 1) => "Endzustand",
            (Locale::De, _) => "Endzustände",
        };
        write!(opening, " {noun}: {}.", listed(locale, finals))
            .expect("writing to String cannot fail");
    }
    opening
}

/// One transition as a sentence; `to` is `None` for a transition to the same state.
pub(crate) fn transition(
    locale: Locale,
    from: &str,
    to: Option<&str>,
    label: Option<&str>,
) -> String {
    let on = label.map_or_else(String::new, |label| match locale {
        Locale::En => format!(" on {label}"),
        Locale::De => format!(" bei {label}"),
    });
    match (locale, to) {
        (Locale::En, Some(to)) => format!("{from} to {to}{on}."),
        (Locale::En, None) => format!("{from} to itself{on}."),
        (Locale::De, Some(to)) => format!("{from} nach {to}{on}."),
        (Locale::De, None) => format!("{from} zu sich selbst{on}."),
    }
}

/// A composite state and the states directly inside it, as a sentence.
pub(crate) fn composite(locale: Locale, label: &str, members: &[&str]) -> String {
    let members = members.join(", ");
    match locale {
        Locale::En => format!("Composite state {label} contains {members}."),
        Locale::De => format!("Zusammengesetzter Zustand {label} enthält {members}."),
    }
}

/// The choices of a state diagram, as a sentence.
pub(crate) fn choices(locale: Locale, labels: &[&str]) -> String {
    let labels = labels.join(", ");
    match locale {
        Locale::En => format!("Choices: {labels}."),
        Locale::De => format!("Auswahlpunkte: {labels}."),
    }
}

/// The opening sentence of an architecture diagram.
pub(crate) fn architecture_opening(
    locale: Locale,
    components: usize,
    connections: usize,
    boundaries: usize,
) -> String {
    let boundaries = match (locale, boundaries) {
        (_, 0) => String::new(),
        (Locale::En, count) => format!(" in {count} boundar{}", plural(count, "y", "ies")),
        (Locale::De, count) => format!(" in {count} {}", plural(count, "Grenze", "Grenzen")),
    };
    match locale {
        Locale::En => format!(
            "Architecture diagram with {components} component{} and {connections} connection{}{boundaries}.",
            plural(components, "", "s"),
            plural(connections, "", "s"),
        ),
        Locale::De => format!(
            "Architekturdiagramm mit {components} {} und {connections} {}{boundaries}.",
            plural(components, "Komponente", "Komponenten"),
            plural(connections, "Verbindung", "Verbindungen"),
        ),
    }
}

/// A boundary and what it holds, components and boundaries, as a sentence.
/// The same in English and German.
pub(crate) fn boundary(label: &str, parent: Option<&str>, held: &[&str]) -> String {
    let parent = parent.map_or_else(String::new, |parent| format!(" (in {parent})"));
    format!("{label}{parent}: {}.", held.join(", "))
}

/// What a component is, for the description and the data table.
pub(crate) const fn component_kind(locale: Locale, kind: ComponentKind) -> &'static str {
    match (locale, kind) {
        (Locale::En, ComponentKind::Person) => "person",
        (Locale::En, ComponentKind::Frontend) => "frontend",
        (Locale::En, ComponentKind::Service) => "service",
        (Locale::En, ComponentKind::Database) => "database",
        (Locale::En, ComponentKind::Queue) => "queue",
        (Locale::En, ComponentKind::Storage) => "storage",
        (Locale::En, ComponentKind::Cache) => "cache",
        (Locale::En, ComponentKind::External) => "external system",
        (Locale::En, ComponentKind::Security) => "security",
        (Locale::De, ComponentKind::Person) => "Person",
        (Locale::De, ComponentKind::Frontend) => "Oberfläche",
        (Locale::De, ComponentKind::Service) => "Dienst",
        (Locale::De, ComponentKind::Database) => "Datenbank",
        (Locale::De, ComponentKind::Queue) => "Warteschlange",
        (Locale::De, ComponentKind::Storage) => "Speicher",
        (Locale::De, ComponentKind::Cache) => "Cache",
        (Locale::De, ComponentKind::External) => "externes System",
        (Locale::De, ComponentKind::Security) => "Sicherheit",
    }
}

/// What a connection does and how, for a parenthesis: `reads, via SQL`.
pub(crate) fn connection_note(
    locale: Locale,
    label: Option<&str>,
    technology: Option<&str>,
) -> Option<String> {
    let via = match locale {
        Locale::En => "via",
        Locale::De => "über",
    };
    match (label, technology) {
        (None, None) => None,
        (Some(label), None) => Some(label.to_owned()),
        (None, Some(technology)) => Some(format!("{via} {technology}")),
        (Some(label), Some(technology)) => Some(format!("{label}, {via} {technology}")),
    }
}

/// A component as a sentence: its label and kind, and where it connects to.
pub(crate) fn component(
    locale: Locale,
    label: &str,
    kind: ComponentKind,
    next: &[String],
) -> String {
    let kind = component_kind(locale, kind);
    let next = match (locale, next.is_empty()) {
        (Locale::En, true) => "no outgoing connection".to_owned(),
        (Locale::De, true) => "keine ausgehende Verbindung".to_owned(),
        (Locale::En, false) => format!("connects to {}", listed(locale, next)),
        (Locale::De, false) => format!("verbunden mit {}", listed(locale, next)),
    };
    format!("{label} ({kind}): {next}.")
}

/// The opening sentence of a tree: how many nodes it has and how many levels they lie in.
pub(crate) fn tree_opening(locale: Locale, nodes: usize, levels: usize) -> String {
    match locale {
        Locale::En => format!(
            "Tree with {nodes} node{} in {levels} level{}.",
            plural(nodes, "", "s"),
            plural(levels, "", "s"),
        ),
        Locale::De => format!(
            "Baum mit {nodes} {} in {levels} {}.",
            plural(nodes, "Knoten", "Knoten"),
            plural(levels, "Ebene", "Ebenen"),
        ),
    }
}

/// The root of a tree, as a sentence.
pub(crate) fn tree_root(locale: Locale, root: &str) -> String {
    match locale {
        Locale::En => format!("Root: {root}."),
        Locale::De => format!("Wurzel: {root}."),
    }
}

/// What a boxplot shows: how many boxes, which median is highest and lowest, and where there are
/// outliers.
pub(crate) fn boxplot_summary(
    locale: Locale,
    boxes: usize,
    highest: (&str, &str),
    lowest: (&str, &str),
    outliers: &[&str],
) -> String {
    let outliers = match (locale, outliers.is_empty()) {
        (_, true) => String::new(),
        (Locale::En, false) => format!(" Outliers beyond the whiskers: {}.", outliers.join(", ")),
        (Locale::De, false) => format!(" Ausreißer jenseits der Whisker: {}.", outliers.join(", ")),
    };
    match locale {
        Locale::En => format!(
            "Box plot with {boxes} box{}, each from the first to the third quartile with its median. Highest median: {} ({}). Lowest median: {} ({}).{outliers}",
            plural(boxes, "", "es"),
            highest.0,
            highest.1,
            lowest.0,
            lowest.1,
        ),
        Locale::De => format!(
            "Boxplot mit {boxes} {}, jeweils vom ersten bis zum dritten Quartil mit dem Median. Höchster Median: {} ({}). Niedrigster Median: {} ({}).{outliers}",
            plural(boxes, "Box", "Boxen"),
            highest.0,
            highest.1,
            lowest.0,
            lowest.1,
        ),
    }
}

/// The sentence that says what the error bars of a bar chart show.
pub(crate) fn error_bars(locale: Locale) -> String {
    match locale {
        Locale::En => " Error bars show the interval from lower to upper; the table gives it for every bar.".to_owned(),
        Locale::De => " Fehlerbalken zeigen das Intervall von unten bis oben; die Tabelle nennt es für jeden Balken.".to_owned(),
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
    match locale {
        Locale::En => format!(
            "Timeline with {phases} phase{} and {milestones} milestone{} from {first} to {last}.",
            plural(phases, "", "s"),
            plural(milestones, "", "s"),
        ),
        Locale::De => format!(
            "Zeitstrahl mit {phases} {} und {milestones} {} vom {first} bis zum {last}.",
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
