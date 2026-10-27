//! The words chartlet writes itself, per locale: descriptions, legend additions, tooltips and
//! the HTML figure. Text from the specification is never translated.

mod architecture;
mod charts;
mod diagrams;
mod statistics;

pub(crate) use architecture::{
    architecture_opening, boundary, class_kind, class_sentence, classes_opening, component,
    component_kind, connection_direction, connection_note, deployment, interfaces, navigable,
    not_stated, perspective, relation_phrase, requires_interface, step_number, steps, view_scope,
};
pub(crate) use charts::{
    CandleSummary, areas, axis_noun, bands, calendar_opening, candle_values, candles,
    categories_opening, diverging_scale, dumbbell_note, equal_values, error_bars, gaps_collapsed,
    heatmap_categories, heatmap_values, missing_values, modeled_ranges, multiples_opening,
    panel_bars_opening, panel_notes, panes, parliament_coalition, parliament_summary, percent_done,
    range_groups, rangebar_opening, rank_summary, sankey_summary, series_at, slope_summary,
    stacked_areas, stacked_shares, stacked_totals, stripes_opening, summary_ranges, time_opening,
    timeline_item, timeline_markers, timeline_opening, treemap_summary, waffle_summary,
    waterfall_summary, weighted_ranges, years_without_value, zone_times, zone_values,
};
pub(crate) use diagrams::{
    bottlenecks, choices, closest_pair, composite, dendrogram_opening, dendrogram_order,
    distance_axis, edge_weight, flow_group, flow_opening, flow_step, fragment, heaviest_edge,
    last_join, loops, main_path, message_kind, most_connected, network_node, network_opening,
    network_rings, node_kind, node_parts, node_values, notes, otherwise, participant, ring_name,
    role, roles, sequence_message, sequence_opening, sinks, sources, state_opening, transition,
    tree_collapsed, tree_opening, tree_root, unconnected_parts,
};
pub(crate) use statistics::{
    box_display_note, boxplot_summary, on_dividing_line, points_above_line, quadrant_points,
    quadrant_position, quadrants_opening, scatter_caption, scatter_fit, scatter_summary,
    survival_group, survival_hazard, survival_summary, survival_test, survival_test_short,
    value_span,
};

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
    /// The first column of a chart along a numeric axis without a title.
    pub position: &'static str,
    pub missing: &'static str,
    pub series: &'static str,
    pub view: &'static str,
    /// The navigation between the views of an architecture model on one page.
    pub views: &'static str,
    pub category: &'static str,
    pub year: &'static str,
    pub date: &'static str,
    pub share: &'static str,
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
    pub at_risk: &'static str,
    pub point: &'static str,
    pub quadrant: &'static str,
    pub number_at_risk: &'static str,
    pub survival: &'static str,
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
    /// The instance column of a deployment view.
    pub instance: &'static str,
    /// The package column of a class diagram's table.
    pub package: &'static str,
    /// The notes column of a class or architecture diagram's table.
    pub notes: &'static str,
    /// The column of an architecture diagram's table with the interfaces a component provides.
    pub provides: &'static str,
    /// The columns of a tree's table.
    pub node: &'static str,
    pub level: &'static str,
    pub parent: &'static str,
    pub link: &'static str,
    pub children: &'static str,
    pub collapsed: &'static str,
    pub partner: &'static str,
    /// The columns of a boxplot's table.
    pub minimum: &'static str,
    pub quartile_1: &'static str,
    pub median: &'static str,
    pub mean: &'static str,
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
    pub attributes: &'static str,
    pub methods: &'static str,
    pub relations: &'static str,
    pub progress: &'static str,
    pub milestone: &'static str,
    pub begin: &'static str,
    pub finish: &'static str,
    pub follows: &'static str,
    /// The columns of a waterfall's table.
    pub total: &'static str,
    pub step_delta: &'static str,
    pub step_start: &'static str,
    /// The columns of a waffle's table, and the name of what its parts leave over.
    pub part: &'static str,
    pub squares: &'static str,
    pub rest: &'static str,
    /// The word for a rank in a rank chart.
    pub rank: &'static str,
    /// The columns of a heatmap's table.
    pub row: &'static str,
    pub column: &'static str,
    /// The words of a parliament chart.
    pub party: &'static str,
    pub seats: &'static str,
    pub seat: &'static str,
    pub coalition: &'static str,
    pub majority: &'static str,
    pub yes: &'static str,
    /// The names of a diagram's focus links, and of the link that clears the focus.
    pub focus: &'static str,
    pub show_all: &'static str,
    /// The ring column of a network's table.
    pub ring: &'static str,
    /// The column of the parts of a node.
    pub parts: &'static str,
    /// The column of the role of a node.
    pub role: &'static str,
    /// The columns of a dendrogram's table.
    pub joins: &'static str,
    pub distance: &'static str,
    pub items: &'static str,
    pub months: [&'static str; 12],
    pub weekdays: [&'static str; 7],
}

const EN: Words = Words {
    value: "Value",
    lower: "lower",
    upper: "upper",
    modeled: "modelled",
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
    views: "Views",
    category: "Category",
    year: "Year",
    date: "Date",
    share: "Share",
    range_low: "Low",
    range_mid: "Mid",
    range_high: "High",
    hatched_modeled: "Hatched: modelled",
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
    at_risk: "At risk",
    point: "Point",
    quadrant: "Quadrant",
    number_at_risk: "Number at risk",
    survival: "Survival",
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
    collapsed: "Collapsed",
    partner: "Partner",
    minimum: "Minimum",
    quartile_1: "Q1",
    median: "Median",
    mean: "Mean",
    quartile_3: "Q3",
    maximum: "Maximum",
    outliers: "Outliers",
    observations: "Observations",
    interval: "Interval",
    weight: "Weight",
    summary: "summary",
    item: "Item",
    phase: "Phase",
    attributes: "Attributes",
    methods: "Methods",
    relations: "Relations",
    progress: "Progress",
    milestone: "Milestone",
    begin: "Start",
    finish: "End",
    follows: "After",
    total: "Total",
    step_delta: "Change",
    step_start: "Start",
    part: "Part",
    squares: "Squares",
    rest: "Rest",
    rank: "Rank",
    row: "Row",
    column: "Column",
    party: "Party",
    seats: "Seats",
    seat: "seat",
    coalition: "Coalition",
    majority: "majority",
    yes: "yes",
    boundary: "Boundary",
    connects_to: "Connects to",
    instance: "Instance",
    package: "Package",
    notes: "Notes",
    provides: "Provides",
    focus: "Focus",
    show_all: "Show all",
    ring: "Ring",
    parts: "Parts",
    role: "Role",
    joins: "Joins",
    distance: "Distance",
    items: "Items",
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
    views: "Ansichten",
    category: "Kategorie",
    year: "Jahr",
    date: "Datum",
    share: "Anteil",
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
    at_risk: "Unter Risiko",
    point: "Punkt",
    quadrant: "Quadrant",
    number_at_risk: "Anzahl unter Risiko",
    survival: "Überleben",
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
    collapsed: "Eingeklappt",
    partner: "Partner",
    minimum: "Minimum",
    quartile_1: "Q1",
    median: "Median",
    mean: "Mittelwert",
    quartile_3: "Q3",
    maximum: "Maximum",
    outliers: "Ausreißer",
    observations: "Beobachtungen",
    interval: "Intervall",
    weight: "Gewicht",
    summary: "Gesamt",
    item: "Eintrag",
    phase: "Phase",
    attributes: "Attribute",
    methods: "Methoden",
    relations: "Beziehungen",
    progress: "Fortschritt",
    milestone: "Meilenstein",
    begin: "Beginn",
    finish: "Ende",
    follows: "Nach",
    total: "Summe",
    step_delta: "Änderung",
    step_start: "Start",
    part: "Teil",
    squares: "Felder",
    rest: "Rest",
    rank: "Rang",
    row: "Zeile",
    column: "Spalte",
    party: "Partei",
    seats: "Sitze",
    seat: "Sitz",
    coalition: "Koalition",
    majority: "Mehrheit",
    yes: "ja",
    boundary: "Grenze",
    connects_to: "Verbunden mit",
    instance: "Instanz",
    package: "Paket",
    notes: "Notizen",
    provides: "Stellt bereit",
    focus: "Fokus",
    show_all: "Alle zeigen",
    ring: "Ring",
    parts: "Anteile",
    role: "Rolle",
    joins: "Verbindet",
    distance: "Abstand",
    items: "Elemente",
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

/// `one` for a count of one, otherwise `many`.
const fn plural(count: usize, one: &'static str, many: &'static str) -> &'static str {
    if count == 1 { one } else { many }
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

#[cfg(test)]
mod tests {
    use super::{
        calendar_opening, categories_opening, equal_values, missing_values, stripes_opening,
        time_opening, timeline_opening, value_span, years_without_value,
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
            "Time chart with 1 point in 2024, modelled."
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
    }

    #[test]
    fn equal_ends_are_named_once() {
        assert_eq!(value_span(Locale::En, "3", "3"), "at 3");
        assert_eq!(value_span(Locale::De, "3", "3"), "bei 3");
        assert_eq!(value_span(Locale::En, "1", "5"), "from 1 to 5");
        assert_eq!(value_span(Locale::De, "1", "5"), "von 1 bis 5");
        assert_eq!(
            stripes_opening(Locale::En, 2020, 2020, "scale"),
            "Warming stripes for 2020, one stripe, scale."
        );
        assert_eq!(
            stripes_opening(Locale::De, 2020, 2020, "Skala"),
            "Wärmestreifen für 2020, ein Streifen, Skala."
        );
        assert_eq!(
            timeline_opening(Locale::En, (0, 1), ("3 Mar 2026", "3 Mar 2026")),
            "Timeline with 0 phases and 1 milestone on 3 Mar 2026."
        );
        assert_eq!(
            timeline_opening(Locale::De, (0, 1), ("3. Mär 2026", "3. Mär 2026")),
            "Zeitstrahl mit 0 Phasen und 1 Meilenstein am 3. Mär 2026."
        );
    }
}
