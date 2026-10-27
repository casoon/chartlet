//! The words of the outline diagrams: fishbone, Venn, kanban, story map, journey and value map
//! diagrams.

use super::{listed, plural};
use crate::spec::Locale;
use crate::spec::ValuemapStage;

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

/// The opening sentence of a Venn diagram: how many sets it has and their names.
pub(crate) fn venn_opening(locale: Locale, sets: &[String]) -> String {
    let names = listed(locale, sets);
    match locale {
        Locale::En => format!("Venn diagram of {} sets: {names}.", sets.len()),
        Locale::De => format!("Venn-Diagramm aus {} Mengen: {names}.", sets.len()),
    }
}

/// The name of the region that lies in exactly the sets `names`: `only A`, `A and B`.
pub(crate) fn venn_region(locale: Locale, names: &[&str]) -> String {
    let owned: Vec<String> = names.iter().map(|name| (*name).to_owned()).collect();
    match (locale, names) {
        (Locale::En, [only]) => format!("only {only}"),
        (Locale::De, [only]) => format!("nur {only}"),
        _ => listed(locale, &owned),
    }
}

/// The heads of the columns of a Venn diagram's table.
pub(crate) const fn venn_columns(locale: Locale) -> [&'static str; 3] {
    match locale {
        Locale::En => ["Region", "Label", "Count"],
        Locale::De => ["Bereich", "Bezeichnung", "Anzahl"],
    }
}

/// The opening sentence of a kanban board: its columns and cards.
pub(crate) fn kanban_opening(locale: Locale, (columns, cards): (usize, usize)) -> String {
    match locale {
        Locale::En => format!(
            "Kanban board with {columns} column{} and {cards} card{}.",
            plural(columns, "", "s"),
            plural(cards, "", "s"),
        ),
        Locale::De => format!(
            "Kanban-Tafel mit {columns} {} und {cards} {}.",
            plural(columns, "Spalte", "Spalten"),
            plural(cards, "Karte", "Karten"),
        ),
    }
}

/// A column of a kanban board: its cards, and its limit with a word where the limit is exceeded.
pub(crate) fn kanban_column(
    locale: Locale,
    label: &str,
    (cards, limit): (usize, Option<u32>),
) -> String {
    let limit = limit.map_or_else(String::new, |limit| {
        let over = usize::try_from(limit).is_ok_and(|limit| cards > limit);
        match (locale, over) {
            (Locale::En, false) => format!(" (limit {limit})"),
            (Locale::En, true) => format!(", over its limit of {limit}"),
            (Locale::De, false) => format!(" (Grenze {limit})"),
            (Locale::De, true) => format!(", über der Grenze von {limit}"),
        }
    });
    match locale {
        Locale::En => format!("{label}: {cards} card{}{limit}.", plural(cards, "", "s")),
        Locale::De => format!(
            "{label}: {cards} {}{limit}.",
            plural(cards, "Karte", "Karten")
        ),
    }
}

/// The word for a card that cannot move on.
pub(crate) const fn kanban_blocked(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "Blocked",
        Locale::De => "Blockiert",
    }
}

/// The blocked cards by title and column.
pub(crate) fn kanban_blocked_cards(locale: Locale, cards: &[(&str, &str)]) -> String {
    let names: Vec<String> = cards
        .iter()
        .map(|(title, column)| match locale {
            Locale::En => format!("“{title}” in {column}"),
            Locale::De => format!("„{title}“ in {column}"),
        })
        .collect();
    match locale {
        Locale::En => format!("Blocked: {}.", listed(locale, &names)),
        Locale::De => format!("Blockiert: {}.", listed(locale, &names)),
    }
}

/// The heads of the columns of a kanban board's table.
pub(crate) const fn kanban_columns(locale: Locale) -> [&'static str; 6] {
    match locale {
        Locale::En => ["Column", "Cards", "Card", "Owner", "Tag", "Status"],
        Locale::De => [
            "Spalte",
            "Karten",
            "Karte",
            "Verantwortlich",
            "Tag",
            "Status",
        ],
    }
}

/// The opening sentence of a story map: its activities, tasks, stories and releases.
pub(crate) fn storymap_opening(
    locale: Locale,
    (activities, tasks, stories, releases): (usize, usize, usize, usize),
) -> String {
    match locale {
        Locale::En => format!(
            "Story map of {activities} activit{}, {tasks} task{} and {stories} stor{} in {releases} release{}.",
            plural(activities, "y", "ies"),
            plural(tasks, "", "s"),
            plural(stories, "y", "ies"),
            plural(releases, "", "s"),
        ),
        Locale::De => format!(
            "Story Map aus {activities} {}, {tasks} {} und {stories} {} in {releases} {}.",
            plural(activities, "Aktivität", "Aktivitäten"),
            plural(tasks, "Aufgabe", "Aufgaben"),
            plural(stories, "Story", "Stories"),
            plural(releases, "Release", "Releases"),
        ),
    }
}

/// The heads of the columns of a story map's table.
pub(crate) const fn storymap_columns(locale: Locale) -> [&'static str; 4] {
    match locale {
        Locale::En => ["Activity", "Task", "Release", "Story"],
        Locale::De => ["Aktivität", "Aufgabe", "Release", "Story"],
    }
}

/// The rows of a journey and what names them: the table's heads, the row names of the diagram and
/// the words of the description.
#[derive(Clone, Copy)]
pub(crate) enum JourneyItem {
    Stage,
    Action,
    Touchpoint,
    Thought,
    Sentiment,
    Frontstage,
    Backstage,
    Support,
    Interaction,
    Visibility,
}

#[allow(clippy::match_same_arms)]
pub(crate) fn journey_item(locale: Locale, item: JourneyItem) -> String {
    let word = match (locale, item) {
        (Locale::En, JourneyItem::Stage) => "Stage",
        (Locale::En, JourneyItem::Action) => "Actions",
        (Locale::En, JourneyItem::Touchpoint) => "Touchpoints",
        (Locale::En, JourneyItem::Thought) => "Thoughts",
        (Locale::En, JourneyItem::Sentiment) => "Sentiment",
        (Locale::En, JourneyItem::Frontstage) => "Frontstage",
        (Locale::En, JourneyItem::Backstage) => "Backstage",
        (Locale::En, JourneyItem::Support) => "Support",
        (Locale::En, JourneyItem::Interaction) => "Line of interaction",
        (Locale::En, JourneyItem::Visibility) => "Line of visibility",
        (Locale::De, JourneyItem::Stage) => "Phase",
        (Locale::De, JourneyItem::Action) => "Aktionen",
        (Locale::De, JourneyItem::Touchpoint) => "Kontaktpunkte",
        (Locale::De, JourneyItem::Thought) => "Gedanken",
        (Locale::De, JourneyItem::Sentiment) => "Stimmung",
        (Locale::De, JourneyItem::Frontstage) => "Frontstage",
        (Locale::De, JourneyItem::Backstage) => "Backstage",
        (Locale::De, JourneyItem::Support) => "Unterstützung",
        (Locale::De, JourneyItem::Interaction) => "Interaktionslinie",
        (Locale::De, JourneyItem::Visibility) => "Sichtbarkeitslinie",
    };
    word.to_owned()
}

/// The name of a text row in the paths of warnings, which is its name in the specification.
pub(crate) const fn journey_path(row: crate::spec::JourneyRow) -> &'static str {
    use crate::spec::JourneyRow;
    match row {
        JourneyRow::Action => "action",
        JourneyRow::Touchpoint => "touchpoint",
        JourneyRow::Thought => "thought",
        JourneyRow::Frontstage => "frontstage",
        JourneyRow::Backstage => "backstage",
        JourneyRow::Support => "support",
    }
}

/// The opening sentence of a journey: its stages, and whether it is a service blueprint.
pub(crate) fn journey_opening(locale: Locale, (stages, blueprint): (usize, bool)) -> String {
    match (locale, blueprint) {
        (Locale::En, false) => format!("User journey of {stages} stages:"),
        (Locale::En, true) => format!(
            "Service blueprint of {stages} stages, with the customer's rows above the line of interaction and the service's below it:"
        ),
        (Locale::De, false) => format!("Customer Journey aus {stages} Phasen:"),
        (Locale::De, true) => format!(
            "Service Blueprint aus {stages} Phasen, mit den Zeilen des Kunden über der Interaktionslinie und denen des Services darunter:"
        ),
    }
}

/// The words on the axes of a value chain map, and the word for a yes in its table.
pub(crate) struct ValuemapWords {
    pub evolution: &'static str,
    pub visibility: &'static str,
    pub visible: &'static str,
    pub hidden: &'static str,
    pub yes: &'static str,
}

pub(crate) const fn valuemap_words(locale: Locale) -> ValuemapWords {
    match locale {
        Locale::En => ValuemapWords {
            evolution: "Evolution",
            visibility: "Visibility to the user",
            visible: "visible",
            hidden: "hidden",
            yes: "yes",
        },
        Locale::De => ValuemapWords {
            evolution: "Entwicklung",
            visibility: "Sichtbarkeit für den Nutzer",
            visible: "sichtbar",
            hidden: "verborgen",
            yes: "ja",
        },
    }
}

/// The name of a stage of evolution.
pub(crate) const fn valuemap_stage(locale: Locale, stage: ValuemapStage) -> &'static str {
    match (locale, stage) {
        (Locale::En, ValuemapStage::Genesis) => "Genesis",
        (Locale::En, ValuemapStage::Custom) => "Custom",
        (Locale::En, ValuemapStage::Product) => "Product",
        (Locale::En, ValuemapStage::Commodity) => "Commodity",
        (Locale::De, ValuemapStage::Genesis) => "Entstehung",
        (Locale::De, ValuemapStage::Custom) => "Individuell",
        (Locale::De, ValuemapStage::Product) => "Produkt",
        (Locale::De, ValuemapStage::Commodity) => "Standardware",
    }
}

/// The opening sentence of a value chain map.
pub(crate) fn valuemap_opening(locale: Locale, components: usize) -> String {
    match locale {
        Locale::En => format!(
            "Value chain map of {components} component{}, from the most visible to the user down, each with its visibility, its stage of evolution and where it moves:",
            plural(components, "", "s"),
        ),
        Locale::De => format!(
            "Wertschöpfungskette aus {components} {}, vom für den Nutzer sichtbarsten abwärts, jeweils mit Sichtbarkeit, Entwicklungsstufe und Bewegung:",
            plural(components, "Komponente", "Komponenten"),
        ),
    }
}

pub(crate) fn valuemap_visibility(locale: Locale, share: &str) -> String {
    match locale {
        Locale::En => format!("visibility {share}"),
        Locale::De => format!("Sichtbarkeit {share}"),
    }
}

pub(crate) fn valuemap_moves(locale: Locale, stage: &str) -> String {
    match locale {
        Locale::En => format!("moving to {stage}"),
        Locale::De => format!("bewegt sich zu {stage}"),
    }
}

pub(crate) const fn valuemap_inertia(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "with inertia",
        Locale::De => "mit Trägheit",
    }
}

/// The dependencies as sentences: the component, and what it needs.
pub(crate) fn valuemap_dependencies(locale: Locale, pairs: &[(&str, &str)]) -> String {
    let sentences: Vec<String> = pairs
        .iter()
        .map(|(from, to)| match locale {
            Locale::En => format!("{from} needs {to}"),
            Locale::De => format!("{from} benötigt {to}"),
        })
        .collect();
    match locale {
        Locale::En => format!("Dependencies: {}.", sentences.join("; ")),
        Locale::De => format!("Abhängigkeiten: {}.", sentences.join("; ")),
    }
}

/// The heads of the columns of a value chain map's table.
pub(crate) const fn valuemap_columns(locale: Locale) -> [&'static str; 7] {
    match locale {
        Locale::En => [
            "Component",
            "Visibility",
            "Evolution",
            "Stage",
            "Moves to",
            "Inertia",
            "Needs",
        ],
        Locale::De => [
            "Komponente",
            "Sichtbarkeit",
            "Entwicklung",
            "Stufe",
            "Bewegt sich zu",
            "Trägheit",
            "Benötigt",
        ],
    }
}
