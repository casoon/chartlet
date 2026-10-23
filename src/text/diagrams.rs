//! The words of diagrams of nodes and edges: sequences, flows, networks, dendrograms, states
//! and trees, and what they share.

use super::{listed, plural};
use crate::spec::{
    DiagramRole, FragmentSpec, Linkage, Locale, MessageKind, NodeKind, ParticipantKind,
};
use std::fmt::Write as _;

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

/// The most connected nodes of a diagram and how many connections each has.
pub(crate) fn most_connected(locale: Locale, nodes: &[String], degree: usize) -> String {
    let names = listed(locale, nodes);
    match (locale, nodes.len()) {
        (Locale::En, 1) => format!("Most connected: {names} ({degree} connections)."),
        (Locale::En, _) => format!("Most connected: {names} ({degree} connections each)."),
        (Locale::De, 1) => format!("Am stärksten verbunden: {names} ({degree} Verbindungen)."),
        (Locale::De, _) => format!("Am stärksten verbunden: {names} (je {degree} Verbindungen)."),
    }
}

/// The nodes nothing leads into, other than those marked as a start.
pub(crate) fn sources(locale: Locale, nodes: &[String]) -> String {
    let names = listed(locale, nodes);
    match locale {
        Locale::En => format!("Nothing leads into {names}."),
        Locale::De => format!("In {names} führt nichts hinein."),
    }
}

/// The nodes nothing leads out of, other than those marked as an end.
pub(crate) fn sinks(locale: Locale, nodes: &[String]) -> String {
    let names = listed(locale, nodes);
    match locale {
        Locale::En => format!("Nothing leads out of {names}."),
        Locale::De => format!("Aus {names} führt nichts heraus."),
    }
}

/// The loops of a diagram, each as `A → B → A`.
pub(crate) fn loops(locale: Locale, loops: &[String]) -> String {
    let joined = loops.join("; ");
    match (locale, loops.len()) {
        (Locale::En, 1) => format!("Loop: {joined}."),
        (Locale::En, _) => format!("Loops: {joined}."),
        (Locale::De, 1) => format!("Schleife: {joined}."),
        (Locale::De, _) => format!("Schleifen: {joined}."),
    }
}

/// How many unconnected parts a diagram has.
pub(crate) fn unconnected_parts(locale: Locale, parts: usize) -> String {
    match locale {
        Locale::En => format!("The diagram has {parts} unconnected parts."),
        Locale::De => format!("Das Diagramm hat {parts} unverbundene Teile."),
    }
}

/// The nodes without which a diagram falls apart.
pub(crate) fn bottlenecks(locale: Locale, nodes: &[String]) -> String {
    let names = listed(locale, nodes);
    match (locale, nodes.len()) {
        (Locale::En, 1) => format!("Bottleneck: {names}; without it the diagram falls apart."),
        (Locale::En, _) => {
            format!("Bottlenecks: {names}; without either the diagram falls apart.")
        }
        (Locale::De, 1) => format!("Engstelle: {names}; ohne sie zerfällt das Diagramm."),
        (Locale::De, _) => {
            format!("Engstellen: {names}; ohne eine davon zerfällt das Diagramm.")
        }
    }
}

/// The weight of an edge in a note: `weight 120`.
pub(crate) fn edge_weight(locale: Locale, weight: &str) -> String {
    match locale {
        Locale::En => format!("weight {weight}"),
        Locale::De => format!("Gewicht {weight}"),
    }
}

/// The edge that carries most, as `A → B`, with its weight.
pub(crate) fn heaviest_edge(locale: Locale, edge: &str, weight: &str) -> String {
    match locale {
        Locale::En => format!("Heaviest connection: {edge} ({weight})."),
        Locale::De => format!("Stärkste Verbindung: {edge} ({weight})."),
    }
}

/// The opening sentence of a network: how many nodes and connections, and its groups.
pub(crate) fn network_opening(
    locale: Locale,
    (nodes, edges): (usize, usize),
    groups: &[String],
) -> String {
    let groups = match (locale, groups.len()) {
        (_, 0) => String::new(),
        (Locale::En, count) => format!(
            " in {count} group{}: {}",
            plural(count, "", "s"),
            listed(locale, groups)
        ),
        (Locale::De, count) => format!(
            " in {count} {}: {}",
            plural(count, "Gruppe", "Gruppen"),
            listed(locale, groups)
        ),
    };
    match locale {
        Locale::En => format!(
            "Network with {nodes} node{} and {edges} connection{}{groups}.",
            plural(nodes, "", "s"),
            plural(edges, "", "s"),
        ),
        Locale::De => format!(
            "Netzwerk mit {nodes} Knoten und {edges} {}{groups}.",
            plural(edges, "Verbindung", "Verbindungen"),
        ),
    }
}

/// The rings of a network around its center: which nodes stand how many steps away, and which
/// no way reaches (the ring without a distance).
pub(crate) fn network_rings(
    locale: Locale,
    center: &str,
    rings: &[(Option<usize>, Vec<String>)],
) -> String {
    let rings: Vec<String> = rings
        .iter()
        .map(|(distance, members)| {
            let members = listed(locale, members);
            match (locale, distance) {
                (Locale::En, Some(1)) => format!("1 step away: {members}"),
                (Locale::En, Some(steps)) => format!("{steps} steps away: {members}"),
                (Locale::En, None) => format!("not connected: {members}"),
                (Locale::De, Some(1)) => format!("1 Schritt entfernt: {members}"),
                (Locale::De, Some(steps)) => format!("{steps} Schritte entfernt: {members}"),
                (Locale::De, None) => format!("nicht verbunden: {members}"),
            }
        })
        .collect();
    let rings = rings.join("; ");
    match locale {
        Locale::En => format!("In rings around {center}: {rings}."),
        Locale::De => format!("In Ringen um {center}: {rings}."),
    }
}

/// One node of a network with its group and the nodes it leads to or is connected with.
pub(crate) fn network_node(
    locale: Locale,
    label: &str,
    group: Option<&str>,
    directed: bool,
    neighbours: &[String],
) -> String {
    let group = match (locale, group) {
        (_, None) => String::new(),
        (Locale::En, Some(group)) => format!(" (group {group})"),
        (Locale::De, Some(group)) => format!(" (Gruppe {group})"),
    };
    let names = listed(locale, neighbours);
    match (locale, directed, neighbours.is_empty()) {
        (Locale::En, false, true) => format!("{label}{group}: no connection."),
        (Locale::En, true, true) => format!("{label}{group}: no outgoing connection."),
        (Locale::En, false, false) => format!("{label}{group}: connected to {names}."),
        (Locale::En, true, false) => format!("{label}{group}: leads to {names}."),
        (Locale::De, false, true) => format!("{label}{group}: keine Verbindung."),
        (Locale::De, true, true) => format!("{label}{group}: keine ausgehende Verbindung."),
        (Locale::De, false, false) => format!("{label}{group}: verbunden mit {names}."),
        (Locale::De, true, false) => format!("{label}{group}: führt zu {names}."),
    }
}

/// The ring of a node in a network's table: the center, a distance, or none.
pub(crate) fn ring_name(locale: Locale, distance: Option<usize>) -> String {
    match (locale, distance) {
        (Locale::En, Some(0)) => "Center".to_owned(),
        (Locale::De, Some(0)) => "Mitte".to_owned(),
        (_, Some(distance)) => distance.to_string(),
        (Locale::En, None) => "not connected".to_owned(),
        (Locale::De, None) => "nicht verbunden".to_owned(),
    }
}

/// What the colors of a diagram's nodes show, the values at both ends of the scale, and every
/// node's value as `label value`.
pub(crate) fn node_values(
    locale: Locale,
    name: &str,
    (low, high): (&str, &str),
    nodes: &[String],
) -> String {
    let nodes = nodes.join(", ");
    match locale {
        Locale::En => {
            format!("Color shows {name}, from {low} (lightest) to {high} (darkest): {nodes}.")
        }
        Locale::De => format!(
            "Die Farbe zeigt {name}, von {low} (am hellsten) bis {high} (am dunkelsten): {nodes}."
        ),
    }
}

/// The parts of the nodes that have some, each as `label: part value (share), …`.
pub(crate) fn node_parts(locale: Locale, nodes: &[String]) -> String {
    let nodes = nodes.join("; ");
    match locale {
        Locale::En => format!("Parts: {nodes}."),
        Locale::De => format!("Anteile: {nodes}."),
    }
}

/// The name of a role of a diagram's nodes and edges.
pub(crate) const fn role(locale: Locale, role: DiagramRole) -> &'static str {
    match (locale, role) {
        (Locale::En, DiagramRole::Critical) => "critical",
        (Locale::En, DiagramRole::Planned) => "planned",
        (Locale::En, DiagramRole::Deprecated) => "deprecated",
        (Locale::De, DiagramRole::Critical) => "kritisch",
        (Locale::De, DiagramRole::Planned) => "geplant",
        (Locale::De, DiagramRole::Deprecated) => "veraltet",
    }
}

/// The sentence with the roles of a diagram, each as `role: what has it`.
pub(crate) fn roles(locale: Locale, roles: &[String]) -> String {
    let roles = roles.join("; ");
    match locale {
        Locale::En => format!("Roles: {roles}."),
        Locale::De => format!("Rollen: {roles}."),
    }
}

/// How a linkage measures the distance between two clusters, as a phrase.
const fn linkage_phrase(locale: Locale, linkage: Linkage) -> &'static str {
    match (locale, linkage) {
        (Locale::En, Linkage::Average) => {
            "average linkage (the mean distance between two clusters)"
        }
        (Locale::En, Linkage::Complete) => {
            "complete linkage (the largest distance between two clusters)"
        }
        (Locale::En, Linkage::Single) => {
            "single linkage (the smallest distance between two clusters)"
        }
        (Locale::De, Linkage::Average) => "mittleren Abstand zwischen zwei Gruppen",
        (Locale::De, Linkage::Complete) => "größten Abstand zwischen zwei Gruppen",
        (Locale::De, Linkage::Single) => "kleinsten Abstand zwischen zwei Gruppen",
    }
}

/// The opening sentence of a dendrogram: how many items and by which linkage.
pub(crate) fn dendrogram_opening(locale: Locale, items: usize, linkage: Linkage) -> String {
    let phrase = linkage_phrase(locale, linkage);
    match locale {
        Locale::En => format!("Dendrogram of {items} items, joined by {phrase}."),
        Locale::De => format!("Dendrogramm aus {items} Elementen, verbunden nach dem {phrase}."),
    }
}

/// The first join of a dendrogram: the two closest items (or clusters, at a tie of several).
pub(crate) fn closest_pair(
    locale: Locale,
    (left, right): (&[String], &[String]),
    distance: &str,
) -> String {
    let (left, right) = (listed(locale, left), listed(locale, right));
    match locale {
        Locale::En => format!("Closest: {left} and {right} ({distance})."),
        Locale::De => format!("Am nächsten: {left} und {right} ({distance})."),
    }
}

/// The last join of a dendrogram: the two groups it brings together.
pub(crate) fn last_join(
    locale: Locale,
    (left, right): (&[String], &[String]),
    distance: &str,
) -> String {
    let (left, right) = (listed(locale, left), listed(locale, right));
    match locale {
        Locale::En => {
            format!("The last join, at {distance}, brings together two groups: {left}; {right}.")
        }
        Locale::De => format!(
            "Die letzte Verbindung, bei {distance}, führt zwei Gruppen zusammen: {left}; {right}."
        ),
    }
}

/// The items of a dendrogram in the order of the tree.
pub(crate) fn dendrogram_order(locale: Locale, items: &[String]) -> String {
    let items = listed(locale, items);
    match locale {
        Locale::En => format!("Order of the items: {items}."),
        Locale::De => format!("Reihenfolge der Elemente: {items}."),
    }
}

/// The title of a dendrogram's distance axis.
pub(crate) fn distance_axis(locale: Locale, linkage: Linkage) -> String {
    let how = match (locale, linkage) {
        (Locale::En, Linkage::Average) => "average",
        (Locale::En, Linkage::Complete) => "complete",
        (Locale::En, Linkage::Single) => "single",
        (Locale::De, Linkage::Average) => "mittlerer",
        (Locale::De, Linkage::Complete) => "größter",
        (Locale::De, Linkage::Single) => "kleinster",
    };
    match locale {
        Locale::En => format!("Distance at the join ({how} linkage)"),
        Locale::De => format!("Abstand bei der Verbindung ({how} Abstand)"),
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

/// The opening sentence of a tree: how many nodes it has and how many levels they lie in. A
/// radial tree says it is a mind map.
pub(crate) fn tree_opening(locale: Locale, nodes: usize, levels: usize, radial: bool) -> String {
    let kind = match (locale, radial) {
        (Locale::En, false) => "Tree",
        (Locale::En, true) => "Mind map (radial tree)",
        (Locale::De, false) => "Baum",
        (Locale::De, true) => "Mindmap (radialer Baum)",
    };
    match locale {
        Locale::En => format!(
            "{kind} with {nodes} node{} in {levels} level{}.",
            plural(nodes, "", "s"),
            plural(levels, "", "s"),
        ),
        Locale::De => format!(
            "{kind} mit {nodes} {} in {levels} {}.",
            plural(nodes, "Knoten", "Knoten"),
            plural(levels, "Ebene", "Ebenen"),
        ),
    }
}

/// The root of a tree, as a sentence; a radial tree has it in the centre.
pub(crate) fn tree_root(locale: Locale, root: &str, radial: bool) -> String {
    match (locale, radial) {
        (Locale::En, false) => format!("Root: {root}."),
        (Locale::En, true) => format!("Centre: {root}."),
        (Locale::De, false) => format!("Wurzel: {root}."),
        (Locale::De, true) => format!("Mitte: {root}."),
    }
}

/// The nodes of a tree that are drawn collapsed.
pub(crate) fn tree_collapsed(locale: Locale, labels: &[&str]) -> String {
    let labels = labels.join(", ");
    match locale {
        Locale::En => {
            format!("Collapsed in the drawing, with everything below them listed here: {labels}.")
        }
        Locale::De => format!("In der Grafik eingeklappt, alles darunter steht hier: {labels}."),
    }
}

/// The notes of a diagram, one sentence each with the element or frame it explains, if any:
/// `Note on Order: only for existing customers.`, `Note: prices are net.`
pub(crate) fn notes(locale: Locale, notes: &[(Option<&str>, &str)]) -> String {
    let sentences: Vec<String> = notes
        .iter()
        .map(|(on, text)| {
            let text = text.trim();
            let stop = if text.ends_with(['.', '!', '?']) {
                ""
            } else {
                "."
            };
            match (locale, on) {
                (Locale::En, Some(on)) => format!("Note on {on}: {text}{stop}"),
                (Locale::De, Some(on)) => format!("Notiz zu {on}: {text}{stop}"),
                (Locale::En, None) => format!("Note: {text}{stop}"),
                (Locale::De, None) => format!("Notiz: {text}{stop}"),
            }
        })
        .collect();
    sentences.join(" ")
}
