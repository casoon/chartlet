//! The words of topic maps and knowledge landscapes.

use super::plural;
use crate::spec::Locale;

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
