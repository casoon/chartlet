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
    pub source: &'static str,
    pub show_data: &'static str,
    pub data_for: &'static str,
    pub time: &'static str,
    pub missing: &'static str,
    pub series: &'static str,
    pub view: &'static str,
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
    source: "Source",
    show_data: "Show chart data",
    data_for: "Data for",
    time: "Time",
    missing: "Missing",
    series: "Series",
    view: "View",
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
    source: "Quelle",
    show_data: "Diagrammdaten anzeigen",
    data_for: "Daten zu",
    time: "Zeit",
    missing: "fehlt",
    series: "Reihen",
    view: "Ansicht",
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

/// The opening sentence of a time chart.
pub(crate) fn time_opening(
    locale: Locale,
    points: usize,
    range: &str,
    names: &[String],
    modeled: bool,
) -> String {
    match (locale, names.len() > 1) {
        (Locale::En, true) => format!(
            "Time chart with {points} points {range} and {} series ({}).",
            names.len(),
            names.join(", ")
        ),
        (Locale::De, true) => format!(
            "Zeitreihe mit {points} Punkten {range} und {} Reihen ({}).",
            names.len(),
            names.join(", ")
        ),
        (Locale::En, false) if modeled => {
            format!("Time chart with {points} points {range}, modeled.")
        }
        (Locale::De, false) if modeled => {
            format!("Zeitreihe mit {points} Punkten {range}, modelliert.")
        }
        (Locale::En, false) => format!("Time chart with {points} points {range}."),
        (Locale::De, false) => format!("Zeitreihe mit {points} Punkten {range}."),
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
