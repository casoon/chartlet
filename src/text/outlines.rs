//! The words of the outline diagrams: fishbone, Venn, kanban, story map, journey and value map
//! diagrams.

use super::plural;
use crate::spec::Locale;

/// The opening sentence of a fishbone diagram: its effect and how many categories and causes it
/// has.
pub(crate) fn fishbone_opening(
    locale: Locale,
    effect: &str,
    (bones, causes): (usize, usize),
) -> String {
    match locale {
        Locale::En => format!(
            "Fishbone diagram of the causes of “{effect}”: {bones} categor{} with {causes} cause{}.",
            plural(bones, "y", "ies"),
            plural(causes, "", "s"),
        ),
        Locale::De => format!(
            "Fischgräten-Diagramm zu den Ursachen von „{effect}“: {bones} {} mit {causes} {}.",
            plural(bones, "Kategorie", "Kategorien"),
            plural(causes, "Ursache", "Ursachen"),
        ),
    }
}

/// What a row of a fishbone diagram's table is, and the column heads.
#[derive(Clone, Copy)]
pub(crate) enum FishboneRow {
    Effect,
    Category,
    Cause,
    Subcause,
}

pub(crate) const fn fishbone_row(locale: Locale, row: FishboneRow) -> &'static str {
    match (locale, row) {
        (Locale::En, FishboneRow::Effect) => "Effect",
        (Locale::En, FishboneRow::Category) => "Category",
        (Locale::En, FishboneRow::Cause) => "Cause",
        (Locale::En, FishboneRow::Subcause) => "Sub-cause",
        (Locale::De, FishboneRow::Effect) => "Wirkung",
        (Locale::De, FishboneRow::Category) => "Kategorie",
        (Locale::De, FishboneRow::Cause) => "Ursache",
        (Locale::De, FishboneRow::Subcause) => "Teilursache",
    }
}

/// The heads of the columns of a table of items that belong to others: the kind, the label, and
/// what it belongs to.
pub(crate) const fn belongs_columns(locale: Locale) -> [&'static str; 3] {
    match locale {
        Locale::En => ["Part", "Label", "Belongs to"],
        Locale::De => ["Teil", "Bezeichnung", "Gehört zu"],
    }
}
