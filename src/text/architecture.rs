//! The words of architecture and class diagrams: components, connections, views and classes.

use super::{listed, plural};
use crate::spec::{ComponentKind, ConnectionDirection, Locale};
use std::fmt::Write as _;

/// What stands for a category a component does not state in a perspective.
pub(crate) const fn not_stated(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "not stated",
        Locale::De => "nicht angegeben",
    }
}

/// The sentence of the perspective a diagram shows: its title, then each category with what has
/// it, as `Payments: API, Orders`.
pub(crate) fn perspective(locale: Locale, title: &str, entries: &[String]) -> String {
    let entries = entries.join("; ");
    match locale {
        Locale::En => format!("Perspective {title}: {entries}."),
        Locale::De => format!("Perspektive {title}: {entries}."),
    }
}

/// The numbered steps of an architecture diagram, in order.
pub(crate) fn steps(locale: Locale, steps: &[String]) -> String {
    let steps = steps.join("; ");
    match locale {
        Locale::En => format!("Steps: {steps}."),
        Locale::De => format!("Schritte: {steps}."),
    }
}

/// One step number in a note: `step 2`.
pub(crate) fn step_number(locale: Locale, step: u8) -> String {
    match locale {
        Locale::En => format!("step {step}"),
        Locale::De => format!("Schritt {step}"),
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

/// How much of its model a view shows: `(shown, in all)` components and connections.
pub(crate) fn view_scope(
    locale: Locale,
    (components, all_components): (usize, usize),
    (connections, all_connections): (usize, usize),
) -> String {
    match locale {
        Locale::En => format!(
            "It is a view that shows {components} of the model's {all_components} component{} and {connections} of its {all_connections} connection{}.",
            plural(all_components, "", "s"),
            plural(all_connections, "", "s"),
        ),
        Locale::De => format!(
            "Die Sicht zeigt {components} der {all_components} {} und {connections} der {all_connections} {} des Modells.",
            plural(all_components, "Komponente", "Komponenten"),
            plural(all_connections, "Verbindung", "Verbindungen"),
        ),
    }
}

/// What a deployment view shows: the environment and how many instances run how many
/// components of the model.
pub(crate) fn deployment(
    locale: Locale,
    environment: &str,
    instances: usize,
    components: usize,
) -> String {
    match locale {
        Locale::En => format!(
            "It shows the environment {environment}: {instances} instance{} of {components} component{} of the model; only the connections listed for the instances are drawn.",
            plural(instances, "", "s"),
            plural(components, "", "s"),
        ),
        Locale::De => format!(
            "Sie zeigt die Umgebung {environment}: {instances} {} von {components} {} des Modells; gezeichnet sind nur die für die Instanzen angegebenen Verbindungen.",
            plural(instances, "Instanz", "Instanzen"),
            plural(components, "Komponente", "Komponenten"),
        ),
    }
}

/// A boundary and what it holds, components and boundaries, as a sentence; a package of a class
/// diagram reads the same.
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
        (Locale::En, ComponentKind::Node) => "node",
        (Locale::En, ComponentKind::Artifact) => "artifact",
        (Locale::De, ComponentKind::Person) => "Person",
        (Locale::De, ComponentKind::Frontend) => "Oberfläche",
        (Locale::De, ComponentKind::Service) => "Dienst",
        (Locale::De, ComponentKind::Database) => "Datenbank",
        (Locale::De, ComponentKind::Queue) => "Warteschlange",
        (Locale::De, ComponentKind::Storage) => "Speicher",
        (Locale::De, ComponentKind::Cache) => "Cache",
        (Locale::De, ComponentKind::External) => "externes System",
        (Locale::De, ComponentKind::Security) => "Sicherheit",
        (Locale::De, ComponentKind::Node) => "Knoten",
        (Locale::De, ComponentKind::Artifact) => "Artefakt",
    }
}

/// Which way a connection runs, for a parenthesis; nothing for the usual way, from `from` to `to`.
pub(crate) const fn connection_direction(
    locale: Locale,
    direction: ConnectionDirection,
) -> Option<&'static str> {
    match (locale, direction) {
        (_, ConnectionDirection::Forward) => None,
        (Locale::En, ConnectionDirection::Both) => Some("both ways"),
        (Locale::En, ConnectionDirection::None) => Some("without direction"),
        (Locale::De, ConnectionDirection::Both) => Some("in beide Richtungen"),
        (Locale::De, ConnectionDirection::None) => Some("ohne Richtung"),
    }
}

/// Which way an association can be navigated, given the class at its `to` end: `navigable to
/// Order`; nothing when that is not said.
pub(crate) fn navigable(
    locale: Locale,
    direction: ConnectionDirection,
    to: &str,
) -> Option<String> {
    match (locale, direction) {
        (_, ConnectionDirection::None) => None,
        (Locale::En, ConnectionDirection::Forward) => Some(format!("navigable to {to}")),
        (Locale::En, ConnectionDirection::Both) => Some("navigable both ways".to_owned()),
        (Locale::De, ConnectionDirection::Forward) => Some(format!("navigierbar zu {to}")),
        (Locale::De, ConnectionDirection::Both) => {
            Some("in beide Richtungen navigierbar".to_owned())
        }
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

/// What a connection to an interface says: which interface it requires.
pub(crate) fn requires_interface(locale: Locale, interface: &str) -> String {
    match locale {
        Locale::En => format!("requires {interface}"),
        Locale::De => format!("benötigt {interface}"),
    }
}

/// The interfaces each component provides, one sentence per component, as `Orders provides
/// Payments API and Refunds API.`
pub(crate) fn interfaces(locale: Locale, provided: &[(&str, Vec<String>)]) -> String {
    provided
        .iter()
        .map(|(component, interfaces)| {
            let interfaces = listed(locale, interfaces);
            match locale {
                Locale::En => format!("{component} provides {interfaces}."),
                Locale::De => format!("{component} stellt {interfaces} bereit."),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A component as a sentence: its label and kind, and where it connects to.
pub(crate) fn component(locale: Locale, label: &str, kind: &str, next: &[String]) -> String {
    let next = match (locale, next.is_empty()) {
        (Locale::En, true) => "no outgoing connection".to_owned(),
        (Locale::De, true) => "keine ausgehende Verbindung".to_owned(),
        (Locale::En, false) => format!("connects to {}", listed(locale, next)),
        (Locale::De, false) => format!("verbunden mit {}", listed(locale, next)),
    };
    format!("{label} ({kind}): {next}.")
}

/// What a class diagram shows: how many classes and relations, in how many packages.
pub(crate) fn classes_opening(
    locale: Locale,
    classes: usize,
    relations: usize,
    packages: usize,
) -> String {
    let packages = match (locale, packages) {
        (_, 0) => String::new(),
        (Locale::En, count) => format!(" in {count} package{}", plural(count, "", "s")),
        (Locale::De, count) => format!(" in {count} {}", plural(count, "Paket", "Paketen")),
    };
    match locale {
        Locale::En => format!(
            "Class diagram with {classes} class{} and {relations} relation{}{packages}.",
            plural(classes, "", "es"),
            plural(relations, "", "s")
        ),
        Locale::De => format!(
            "Klassendiagramm mit {classes} {} und {relations} {}{packages}.",
            plural(classes, "Klasse", "Klassen"),
            plural(relations, "Beziehung", "Beziehungen")
        ),
    }
}

/// A class with the number of its members and the relations it starts.
pub(crate) fn class_sentence(
    locale: Locale,
    label: &str,
    (attributes, methods): (usize, usize),
    relations: &[String],
) -> String {
    let members = match locale {
        Locale::En => format!(
            "{attributes} attribute{}, {methods} method{}",
            plural(attributes, "", "s"),
            plural(methods, "", "s")
        ),
        Locale::De => format!(
            "{attributes} {}, {methods} {}",
            plural(attributes, "Attribut", "Attribute"),
            plural(methods, "Methode", "Methoden")
        ),
    };
    let mut sentence = format!("{label}: {members}.");
    if !relations.is_empty() {
        let joined = relations.join("; ");
        sentence.push(' ');
        write!(sentence, "{label} {joined}.").expect("writing to String cannot fail");
    }
    sentence
}

/// The kind of a class as the table writes it.
pub(crate) fn class_kind(locale: Locale, kind: crate::spec::ClassKind) -> &'static str {
    use crate::spec::ClassKind;
    match (locale, kind) {
        (Locale::En, ClassKind::Class) => "Class",
        (Locale::En, ClassKind::Interface) => "Interface",
        (Locale::En, ClassKind::Abstract) => "Abstract class",
        (Locale::En, ClassKind::Enum) => "Enumeration",
        (Locale::En, ClassKind::Entity) => "Entity",
        (Locale::De, ClassKind::Class) => "Klasse",
        (Locale::De, ClassKind::Interface) => "Schnittstelle",
        (Locale::De, ClassKind::Abstract) => "Abstrakte Klasse",
        (Locale::De, ClassKind::Enum) => "Aufzählung",
        (Locale::De, ClassKind::Entity) => "Entität",
    }
}

/// One relation of a class in words: `inherits from Animal`, with its cardinalities and its
/// label. `from_side` says whether the class starts the relation.
pub(crate) fn relation_phrase(
    locale: Locale,
    (kind, from_side): (crate::spec::RelationKind, bool),
    other: &str,
    cardinality: Option<&str>,
    label: Option<&str>,
) -> String {
    use crate::spec::RelationKind;
    let verb = match (locale, kind, from_side) {
        (Locale::En, RelationKind::Association, _) => "is associated with",
        (Locale::En, RelationKind::Inheritance, true) => "inherits from",
        (Locale::En, RelationKind::Inheritance, false) => "is the parent of",
        (Locale::En, RelationKind::Realization, true) => "implements",
        (Locale::En, RelationKind::Realization, false) => "is implemented by",
        (Locale::En, RelationKind::Composition, true) => "is composed of",
        (Locale::En, RelationKind::Composition, false) => "is part of",
        (Locale::En, RelationKind::Aggregation, true) => "holds",
        (Locale::En, RelationKind::Aggregation, false) => "is held by",
        (Locale::En, RelationKind::Dependency, true) => "depends on",
        (Locale::En, RelationKind::Dependency, false) => "is needed by",
        (Locale::De, RelationKind::Association, _) => "ist verbunden mit",
        (Locale::De, RelationKind::Inheritance, true) => "erbt von",
        (Locale::De, RelationKind::Inheritance, false) => "ist Elternklasse von",
        (Locale::De, RelationKind::Realization, true) => "implementiert",
        (Locale::De, RelationKind::Realization, false) => "wird implementiert von",
        (Locale::De, RelationKind::Composition, true) => "besteht aus",
        (Locale::De, RelationKind::Composition, false) => "ist Teil von",
        (Locale::De, RelationKind::Aggregation, true) => "enthält",
        (Locale::De, RelationKind::Aggregation, false) => "wird enthalten von",
        (Locale::De, RelationKind::Dependency, true) => "hängt ab von",
        (Locale::De, RelationKind::Dependency, false) => "wird gebraucht von",
    };
    let mut phrase = format!("{verb} {other}");
    if let Some(cardinality) = cardinality {
        write!(phrase, " ({cardinality})").expect("writing to String cannot fail");
    }
    if let Some(label) = label {
        write!(phrase, " [{label}]").expect("writing to String cannot fail");
    }
    phrase
}
