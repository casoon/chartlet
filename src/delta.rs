//! The delta of an architecture model: two of its views side by side, the one before and the one
//! after, with the changes between them written out. Both are drawn from one layout of the
//! components and connections of the two together, so that a component present in both stands at
//! the same place in both; what a state lacks is left out of its panel. A component that moved to
//! another boundary stands twice, once in each place. Added, removed, changed, moved and rewired
//! carry a sign of their own, so that color is never the only cue, and the ledger below the panels,
//! the description and the table list every change.

use crate::{
    DataTable,
    diagram::{self, pixels},
    error::ChartWarning,
    layers::wrap_lines,
    layout::{NARROW, count, push_title, title_extra},
    metrics::TextMetrics,
    roles,
    scene::{Circle, Element, Hook, Line, Scene, Text, TextAnchor},
    spec::{ArchitectureSpec, ChartSpec, ConnectionSpec, DiagramRole},
    text::{self, DeltaKind as Kind},
};

const MARGIN: f64 = 24.0;
const COMPACT_MARGIN: f64 = 12.0;
const GAP: f64 = 32.0;
const HEADING: f64 = 32.0;
const LEDGER_LINE: f64 = 16.0;
const BOTTOM: f64 = 16.0;
/// Panels side by side from this canvas width on, one above the other below it.
const SIDE_BY_SIDE: u32 = 900;
/// The size of a sign's badge and the space it takes beside the next.
const SIGN_STEP: f64 = 19.0;

/// The sign that marks a kind of change, which needs no color.
pub(crate) const fn sign(kind: Kind) -> &'static str {
    match kind {
        Kind::Added => "+",
        Kind::Removed => "−",
        Kind::Changed => "~",
        Kind::Moved => "→",
        Kind::Rewired => "↔",
    }
}

const ORDER: [Kind; 5] = [
    Kind::Added,
    Kind::Removed,
    Kind::Changed,
    Kind::Moved,
    Kind::Rewired,
];

/// One entry of the ledger: what changed and, where it needs saying, how.
#[derive(Debug, Clone)]
pub(crate) struct Change {
    pub kind: Kind,
    pub subject: String,
    pub detail: String,
}

/// The two states and what lies between them.
struct Delta {
    before: ChartSpec,
    after: ChartSpec,
    /// The components and connections of both states together.
    union: ArchitectureSpec,
    /// Where each component and connection of the union is present: before, after.
    component_in: Vec<(bool, bool)>,
    connection_in: Vec<(bool, bool)>,
    /// The signs each component of the union wears.
    signs: Vec<Vec<Kind>>,
    /// Per component of the union, what the state before writes where the union has the text of
    /// the state after.
    swaps: Vec<(String, Vec<(String, String)>)>,
    changes: Vec<Change>,
}

fn architecture(spec: &ChartSpec) -> &ArchitectureSpec {
    spec.architecture
        .as_ref()
        .expect("validated architecture diagrams carry an architecture block")
}

/// The id a component that moved has in the state after: it stands twice, and its second copy has
/// a name of its own.
fn moved_id(id: &str) -> String {
    format!("{id}~after")
}

fn boundary_label(
    architecture: &ArchitectureSpec,
    id: Option<&str>,
    locale: crate::spec::Locale,
) -> String {
    id.and_then(|id| {
        architecture
            .boundaries
            .iter()
            .find(|boundary| boundary.id == id)
    })
    .map_or_else(
        || text::delta_unbounded(locale).to_owned(),
        |boundary| boundary.label.clone(),
    )
}

type ConnectionKey = (String, String, Option<String>, Option<String>);

fn connection_key(connection: &ConnectionSpec) -> ConnectionKey {
    (
        connection.from.clone(),
        connection.to.clone(),
        connection.label.clone(),
        connection.technology.clone(),
    )
}

/// A connection as the diagram writes it: label and technology.
fn written(connection: &ConnectionSpec) -> String {
    match (&connection.label, &connection.technology) {
        (Some(label), Some(technology)) => format!("{label} [{technology}]"),
        (Some(label), None) => label.clone(),
        (None, Some(technology)) => format!("[{technology}]"),
        (None, None) => String::new(),
    }
}

impl Delta {
    #[allow(clippy::too_many_lines)]
    fn new(spec: &ChartSpec) -> Self {
        let delta = architecture(spec)
            .delta
            .as_ref()
            .expect("a delta is asked for");
        let locale = spec.locale;
        let state = |key: &str| {
            spec.resolve_view(Some(key))
                .expect("validated deltas name views")
                .expect("a view was asked for")
                .spec
        };
        let (before, after) = (state(&delta.before), state(&delta.after));
        let (old, new) = (architecture(&before), architecture(&after));
        let mut changes: Vec<Change> = Vec::new();
        let mut union = new.clone();
        union.components.clone_from(&old.components);
        union.connections = Vec::new();
        union.boundaries.clone_from(&old.boundaries);
        union.views = Vec::new();
        union.delta = None;
        union.notes = Vec::new();
        union.main_path = crate::spec::MainPath::default();
        union.same_rank = Vec::new();
        union.perspectives = Vec::new();
        union.perspective = None;
        union.default_view = None;
        let mut component_in: Vec<(bool, bool)> =
            old.components.iter().map(|_| (true, false)).collect();
        let mut signs: Vec<Vec<Kind>> = old.components.iter().map(|_| Vec::new()).collect();
        // A component that takes the place of another stands in its slot: the id of the one it
        // replaces, where that one is gone in the state after.
        let slot_of = |component: &crate::spec::ComponentSpec| -> String {
            component
                .replaces
                .as_deref()
                .filter(|id| old.component(id).is_some() && new.component(id).is_none())
                .map_or_else(|| component.id.clone(), str::to_owned)
        };
        // The slots of replacing components: the id in the state after, the id of the slot.
        let mut pairs: Vec<(String, String)> = Vec::new();
        // What the state before writes where the union writes the text of the state after.
        let mut swaps: Vec<(String, Vec<(String, String)>)> = Vec::new();
        // The slots of components that moved, with the id in the state after.
        let mut moved: Vec<(String, String)> = Vec::new();
        for component in &new.components {
            let slot = slot_of(component);
            let Some(position) = old.component(&slot) else {
                union.components.push(component.clone());
                component_in.push((false, true));
                signs.push(vec![Kind::Added]);
                changes.push(Change {
                    kind: Kind::Added,
                    subject: component.label.clone(),
                    detail: String::new(),
                });
                continue;
            };
            let was = old.components[position].clone();
            component_in[position].1 = true;
            if slot != component.id {
                pairs.push((component.id.clone(), slot.clone()));
                union.components[position] = crate::spec::ComponentSpec {
                    id: slot.clone(),
                    boundary: was.boundary.clone(),
                    replaces: None,
                    ..component.clone()
                };
            }
            let mut details: Vec<String> = Vec::new();
            let mut swap: Vec<(String, String)> = Vec::new();
            if was.label != component.label {
                details.push(text::delta_field(
                    locale,
                    text::DeltaField::Label,
                    &was.label,
                    &component.label,
                ));
                swap.push((component.label.clone(), was.label.clone()));
            }
            if was.sublabel != component.sublabel {
                details.push(text::delta_field(
                    locale,
                    text::DeltaField::Technology,
                    was.sublabel.as_deref().unwrap_or("–"),
                    component.sublabel.as_deref().unwrap_or("–"),
                ));
                if let (Some(old_text), Some(new_text)) = (&was.sublabel, &component.sublabel) {
                    swap.push((new_text.clone(), old_text.clone()));
                }
            }
            if was.kind != component.kind {
                details.push(text::delta_field(
                    locale,
                    text::DeltaField::Kind,
                    text::component_kind(locale, was.kind),
                    text::component_kind(locale, component.kind),
                ));
            }
            if !swap.is_empty() {
                swaps.push((slot.clone(), swap));
            }
            if !details.is_empty() {
                signs[position].push(Kind::Changed);
                changes.push(Change {
                    kind: Kind::Changed,
                    subject: component.label.clone(),
                    detail: details.join("; "),
                });
            }
            if was.boundary != component.boundary {
                moved.push((slot.clone(), component.id.clone()));
                signs[position].push(Kind::Moved);
                changes.push(Change {
                    kind: Kind::Moved,
                    subject: component.label.clone(),
                    detail: text::delta_moved(
                        locale,
                        &boundary_label(old, was.boundary.as_deref(), locale),
                        &boundary_label(new, component.boundary.as_deref(), locale),
                    ),
                });
            }
        }
        for (position, component) in old.components.iter().enumerate() {
            let kept = new.component(&component.id).is_some()
                || pairs.iter().any(|(_, slot)| *slot == component.id);
            if !kept {
                signs[position].push(Kind::Removed);
                changes.push(Change {
                    kind: Kind::Removed,
                    subject: component.label.clone(),
                    detail: String::new(),
                });
            }
        }
        // A component that moved stands again in its new boundary, under a name of its own; its
        // first copy stands in the state before only.
        for (slot, new_id) in &moved {
            let mut copy = new.components[new.component(new_id).expect("moved")].clone();
            copy.id = moved_id(slot);
            copy.replaces = None;
            let position = old.component(slot).expect("moved");
            component_in[position] = (true, false);
            union.components.push(copy);
            component_in.push((false, true));
            let worn = signs[position].clone();
            signs.push(worn);
        }
        for boundary in &new.boundaries {
            if !union.boundaries.iter().any(|known| known.id == boundary.id) {
                union.boundaries.push(boundary.clone());
            }
        }
        let name = |after: bool, id: &str| {
            let slot = if after {
                pairs
                    .iter()
                    .find(|(new_id, _)| new_id == id)
                    .map_or(id, |(_, slot)| slot.as_str())
            } else {
                id
            };
            if after && moved.iter().any(|(known, _)| known == slot) {
                moved_id(slot)
            } else {
                slot.to_owned()
            }
        };
        let mut connection_in: Vec<(bool, bool)> = Vec::new();
        let mut keys: Vec<ConnectionKey> = Vec::new();
        for (after, source) in [(false, &old.connections), (true, &new.connections)] {
            for connection in source {
                let mut entry = connection.clone();
                entry.from = name(after, &connection.from);
                entry.to = name(after, &connection.to);
                entry.step = None;
                let key = connection_key(&entry);
                if let Some(position) = keys.iter().position(|known| *known == key) {
                    connection_in[position].1 = true;
                } else {
                    entry.id = Some(format!("delta-{}", union.connections.len()));
                    union.connections.push(entry);
                    keys.push(key);
                    connection_in.push((!after, after));
                }
            }
        }
        // The connections only one state has: removed, added, or changed in their text.
        let label_of = |id: &str| {
            let id = id.trim_end_matches("~after");
            new.component(id)
                .map(|index| new.components[index].label.clone())
                .or_else(|| {
                    old.component(id)
                        .map(|index| old.components[index].label.clone())
                })
                .unwrap_or_else(|| id.to_owned())
        };
        let ends = |index: usize| {
            let connection = &union.connections[index];
            (
                connection.from.trim_end_matches("~after").to_owned(),
                connection.to.trim_end_matches("~after").to_owned(),
            )
        };
        let lone: Vec<(usize, bool)> = (0..union.connections.len())
            .filter_map(|index| match connection_in[index] {
                (true, false) => Some((index, false)),
                (false, true) => Some((index, true)),
                _ => None,
            })
            .collect();
        let mut handled = vec![false; lone.len()];
        for first in 0..lone.len() {
            if handled[first] {
                continue;
            }
            let (index, added) = lone[first];
            let pair = (first + 1..lone.len()).find(|&other| {
                !handled[other] && lone[other].1 != added && ends(lone[other].0) == ends(index)
            });
            let (from, to) = ends(index);
            let arrow = format!("{} → {}", label_of(&from), label_of(&to));
            let detail = if let Some(other) = pair {
                handled[other] = true;
                let (earlier, later) = if added {
                    (lone[other].0, index)
                } else {
                    (index, lone[other].0)
                };
                let (was, now) = (
                    written(&union.connections[earlier]),
                    written(&union.connections[later]),
                );
                // The same line to the same place, drawn again beside a component that moved.
                if was == now {
                    continue;
                }
                text::delta_connection_text(locale, &was, &now)
            } else {
                text::delta_connection(locale, added)
            };
            changes.push(Change {
                kind: Kind::Rewired,
                subject: arrow,
                detail,
            });
            let connection = &union.connections[index];
            for id in [&connection.from, &connection.to] {
                if let Some(position) = union.component(id)
                    && component_in[position] == (true, true)
                    && !signs[position].contains(&Kind::Rewired)
                {
                    signs[position].push(Kind::Rewired);
                }
            }
        }
        // The ledger reads by kind, in the order of the signs.
        changes.sort_by_key(|change| ORDER.iter().position(|kind| *kind == change.kind));
        Self {
            before,
            after,
            union,
            component_in,
            connection_in,
            signs,
            swaps,
            changes,
        }
    }

    /// Whether component `index` of the union stands in the state before, or after.
    fn shows(&self, index: usize, after: bool) -> bool {
        let present = self.component_in[index];
        if after { present.1 } else { present.0 }
    }

    fn shows_connection(&self, index: usize, after: bool) -> bool {
        let present = self.connection_in[index];
        if after { present.1 } else { present.0 }
    }

    /// Whether boundary `id` holds a component the state shows, in itself or in a boundary in it.
    fn shows_boundary(&self, id: &str, after: bool) -> bool {
        self.union
            .components
            .iter()
            .enumerate()
            .any(|(index, component)| {
                self.shows(index, after) && self.inside(component.boundary.as_deref(), id)
            })
    }

    fn inside(&self, boundary: Option<&str>, ancestor: &str) -> bool {
        let mut current = boundary;
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            current = self
                .union
                .boundaries
                .iter()
                .find(|boundary| boundary.id == id)
                .and_then(|boundary| boundary.parent.as_deref());
        }
        false
    }

    fn title(&self, after: bool) -> &str {
        if after {
            &self.after.title
        } else {
            &self.before.title
        }
    }
}

/// Moves an element by `(dx, dy)`.
fn shifted(element: &Element, dx: f64, dy: f64) -> Element {
    let mut element = element.clone();
    match &mut element {
        Element::Circle(circle) => {
            circle.cx += dx;
            circle.cy += dy;
        }
        Element::Line(line) => {
            line.x1 += dx;
            line.x2 += dx;
            line.y1 += dy;
            line.y2 += dy;
        }
        Element::Polyline(polyline) => {
            for point in &mut polyline.points {
                point.0 += dx;
                point.1 += dy;
            }
        }
        Element::Rect(rect) => {
            rect.x += dx;
            rect.y += dy;
        }
        Element::Text(text) | Element::SeriesText(text, _) => {
            text.x += dx;
            text.y += dy;
        }
        Element::Hotspot(hotspot) => {
            hotspot.x += dx;
            hotspot.y += dy;
        }
        Element::Symbol(symbol) => {
            symbol.x += dx;
            symbol.y += dy;
        }
        Element::Hook(_)
        | Element::Group(_)
        | Element::NodeGroup(..)
        | Element::EdgeGroup(..)
        | Element::GroupEnd => {}
    }
    element
}

/// The id number of a connection of the union, from the id its group carries.
fn connection_number(id: &str) -> Option<usize> {
    id.strip_prefix("delta-")
        .and_then(|number| number.parse().ok())
}

/// The elements of the layout of the union that belong to one state: components and connections
/// the state lacks are left out, with the labels of those connections and the frames of boundaries
/// that then hold nothing.
#[allow(clippy::too_many_lines)]
fn panel(delta: &Delta, union: &Scene, after: bool) -> Vec<Element> {
    // Where every connection runs, to tell whose label a chip is.
    let mut routes: Vec<(usize, Vec<(f64, f64)>)> = Vec::new();
    let mut open = false;
    for element in &union.elements {
        match element {
            Element::EdgeGroup(_, id) => {
                if let Some(number) = connection_number(id) {
                    routes.push((number, Vec::new()));
                    open = true;
                }
            }
            Element::GroupEnd => open = false,
            Element::Polyline(polyline) if open && polyline.class.contains("flow-edge") => {
                if let Some((_, points)) = routes.last_mut() {
                    points.extend(polyline.points.iter().copied());
                }
            }
            _ => {}
        }
    }
    let nearest = |at: (f64, f64)| {
        routes
            .iter()
            .map(|(number, points)| {
                let distance = points
                    .iter()
                    .map(|point| (point.0 - at.0).hypot(point.1 - at.1))
                    .fold(f64::INFINITY, f64::min);
                (distance, *number)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, number)| number)
    };
    let mut output = Vec::new();
    let mut skipping = 0_usize;
    // The texts the state before writes in place of the union's, for the component being drawn.
    let mut swap: Vec<(String, String)> = Vec::new();
    let mut depth = 0_usize;
    let mut frame_skip = false;
    let mut chip_skip = false;
    for element in &union.elements {
        if skipping > 0 {
            match element {
                Element::Group(_) | Element::NodeGroup(..) | Element::EdgeGroup(..) => {
                    skipping += 1;
                }
                Element::GroupEnd => skipping -= 1,
                _ => {}
            }
            continue;
        }
        if frame_skip {
            if matches!(element, Element::Hook(Hook::End)) {
                frame_skip = false;
            }
            continue;
        }
        let mut chip_text = false;
        match element {
            Element::NodeGroup(classes, id) => {
                let index = delta
                    .union
                    .components
                    .iter()
                    .position(|component| &component.id == id);
                match index {
                    Some(index) if delta.shows(index, after) => {
                        let mut classes = classes.clone();
                        if delta.signs[index].contains(&Kind::Added) {
                            classes.push_str(roles::class(DiagramRole::Planned));
                        }
                        if delta.signs[index].contains(&Kind::Removed) {
                            classes.push_str(roles::class(DiagramRole::Deprecated));
                        }
                        output.push(Element::NodeGroup(classes, id.clone()));
                        swap = if after {
                            Vec::new()
                        } else {
                            delta
                                .swaps
                                .iter()
                                .find(|(slot, _)| slot == id)
                                .map_or_else(Vec::new, |(_, swap)| swap.clone())
                        };
                        depth = 1;
                    }
                    _ => skipping = 1,
                }
            }
            Element::EdgeGroup(classes, id) => match connection_number(id) {
                Some(number) if delta.shows_connection(number, after) => {
                    output.push(Element::EdgeGroup(classes.clone(), id.clone()));
                }
                _ => skipping = 1,
            },
            Element::Hook(Hook::Object("data-chartlet-frame", id)) => {
                if delta.shows_boundary(id, after) {
                    output.push(element.clone());
                } else {
                    frame_skip = true;
                }
            }
            Element::Hotspot(hotspot) => {
                // One focus link per component: in the panel after, or in the one before when the
                // state after lacks it.
                if delta.shows(hotspot.node, after) && (after || !delta.shows(hotspot.node, true)) {
                    output.push(element.clone());
                }
            }
            Element::Text(text) if depth > 0 && !swap.is_empty() => {
                let mut text = text.clone();
                if let Some((_, old_text)) =
                    swap.iter().find(|(new_text, _)| *new_text == text.content)
                {
                    text.content.clone_from(old_text);
                }
                output.push(Element::Text(text));
            }
            Element::Rect(rect) if rect.class == "chartlet-diagram-chip" => {
                let middle = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
                chip_skip =
                    nearest(middle).is_some_and(|number| !delta.shows_connection(number, after));
                chip_text = true;
                if !chip_skip {
                    output.push(element.clone());
                }
            }
            Element::Text(text) if text.class == "chartlet-diagram-chip-text" => {
                chip_text = true;
                if !chip_skip {
                    output.push(element.clone());
                }
            }
            Element::GroupEnd if depth > 0 => {
                depth -= 1;
                output.push(element.clone());
            }
            Element::Group(_) if depth > 0 => {
                depth += 1;
                output.push(element.clone());
            }
            _ => output.push(element.clone()),
        }
        if !chip_text {
            chip_skip = false;
        }
    }
    output
}

/// A badge with a sign that is not a role's, centred on a point.
fn sign_badge(kind: Kind, (cx, cy): (f64, f64), elements: &mut Vec<Element>) {
    elements.push(Element::Circle(Circle {
        cx,
        cy,
        radius: diagram::BADGE,
        class: "chartlet-diagram-badge",
        series_index: None,
        style_index: None,
        tooltip: Some(text::delta_kind(crate::spec::Locale::En, kind).to_owned()),
    }));
    elements.push(Element::Text(Text {
        x: cx,
        y: cy + 3.5,
        class: "chartlet-diagram-badge-text",
        anchor: TextAnchor::Middle,
        content: sign(kind).to_owned(),
    }));
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let delta = Delta::new(spec);
    let locale = spec.locale;
    let compact = spec.width < NARROW;
    let margin = if compact { COMPACT_MARGIN } else { MARGIN };
    let width = f64::from(spec.width);
    let side = spec.width >= SIDE_BY_SIDE;
    let top = crate::flow::up(
        56.0 + title_extra(spec, width - 2.0 * margin, metrics),
        crate::flow::GRID,
    );
    let panel_width = if side {
        (width - 2.0 * margin - GAP) / 2.0
    } else {
        width - 2.0 * margin
    };
    let mut union_spec = delta.after.clone();
    union_spec.architecture = Some(delta.union.clone());
    union_spec.width = pixels(panel_width);
    // Each panel gets the height the canvas leaves after the headings and the ledger, so that the
    // layout takes the size its content needs and does not stretch to the whole canvas.
    let kinds = ORDER
        .iter()
        .filter(|kind| delta.changes.iter().any(|change| change.kind == **kind))
        .count();
    let ledger = 40.0 + LEDGER_LINE * count(kinds.max(1));
    let panels = if side { 1.0 } else { 2.0 };
    let room = (f64::from(spec.height) - top - ledger - BOTTOM) / panels - HEADING;
    union_spec.height = pixels(room.max(200.0));
    union_spec.show_title = false;
    union_spec.mobile = None;
    union_spec.locale = locale;
    union_spec.theme = spec.theme;
    union_spec.tokens.clone_from(&spec.tokens);
    let union = crate::architecture::layout(&union_spec, warnings, metrics);
    let (union_width, union_height) = (f64::from(union.width), f64::from(union.height));
    let page_width = if side {
        width.max(2.0 * margin + GAP + 2.0 * union_width)
    } else {
        width.max(2.0 * margin + union_width)
    };
    let first = (margin, top + HEADING);
    let second = if side {
        (
            margin + crate::flow::up(union_width, crate::flow::GRID) + GAP,
            top + HEADING,
        )
    } else {
        (
            margin,
            top + 2.0 * HEADING + crate::flow::up(union_height, crate::flow::GRID) + GAP,
        )
    };
    let panels_bottom = second.1 + union_height;
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        page_width - 2.0 * margin,
        metrics,
        warnings,
    );
    for (after, (x, y)) in [(false, first), (true, second)] {
        let heading = text::delta_state(locale, after, delta.title(after));
        elements.push(Element::Text(Text {
            x,
            y: y - 10.0,
            class: "chartlet-delta-heading",
            anchor: TextAnchor::Start,
            content: crate::layout::fit_text(
                &heading,
                union_width,
                14.0,
                metrics,
                warnings,
                "/architecture/delta",
            ),
        }));
        for element in panel(&delta, &union, after) {
            elements.push(shifted(&element, x, y));
        }
        // The signs: on the corner of every component the state shows.
        for element in &union.elements {
            let crate::scene::Element::Hotspot(hotspot) = element else {
                continue;
            };
            if !delta.shows(hotspot.node, after) {
                continue;
            }
            for (place, kind) in delta.signs[hotspot.node].iter().enumerate() {
                let center = (
                    x + hotspot.x + 2.0 + SIGN_STEP * count(place),
                    y + hotspot.y + 2.0,
                );
                match kind {
                    Kind::Added | Kind::Removed => {
                        let role = if *kind == Kind::Added {
                            DiagramRole::Planned
                        } else {
                            DiagramRole::Deprecated
                        };
                        elements.push(Element::Group(roles::class(role).trim_start().to_owned()));
                        roles::push_badge(role, center, &mut elements);
                        elements.push(Element::GroupEnd);
                    }
                    _ => sign_badge(*kind, center, &mut elements),
                }
            }
        }
    }
    if side {
        let x = margin + union_width + GAP / 2.0;
        elements.push(Element::Line(Line {
            x1: x,
            y1: top + 4.0,
            x2: x,
            y2: panels_bottom,
            class: "chartlet-delta-rule",
        }));
    }
    // The ledger below the panels.
    let mut y = panels_bottom + 26.0;
    elements.push(Element::Text(Text {
        x: margin,
        y,
        class: "chartlet-delta-heading",
        anchor: TextAnchor::Start,
        content: text::delta_ledger(locale).to_owned(),
    }));
    y += 6.0;
    let room = page_width - 2.0 * margin - 24.0;
    if delta.changes.is_empty() {
        y += LEDGER_LINE;
        elements.push(Element::Text(Text {
            x: margin,
            y,
            class: "chartlet-delta-entry",
            anchor: TextAnchor::Start,
            content: text::delta_none(locale).to_owned(),
        }));
    }
    for kind in ORDER {
        let entries = entries(&delta, kind);
        if entries.is_empty() {
            continue;
        }
        let line = text::delta_group(locale, kind, &entries);
        let lines = wrap_lines(
            &line,
            room,
            12.0,
            4,
            metrics,
            warnings,
            "/architecture/delta",
        );
        y += LEDGER_LINE;
        elements.push(Element::Text(Text {
            x: margin + 8.0,
            y,
            class: "chartlet-delta-sign",
            anchor: TextAnchor::Middle,
            content: sign(kind).to_owned(),
        }));
        for (index, text_line) in lines.into_iter().enumerate() {
            if index > 0 {
                y += LEDGER_LINE;
            }
            elements.push(Element::Text(Text {
                x: margin + 24.0,
                y,
                class: "chartlet-delta-entry",
                anchor: TextAnchor::Start,
                content: text_line,
            }));
        }
    }
    let page_height = y + BOTTOM;
    crate::diagram::warn_growth_advising(
        spec,
        (page_width, page_height),
        "use fewer components",
        warnings,
    );
    Scene {
        width: pixels(page_width),
        height: pixels(page_height),
        elements,
    }
}

/// The entries of the ledger of one kind, each `subject` or `subject (detail)`.
fn entries(delta: &Delta, kind: Kind) -> Vec<String> {
    delta
        .changes
        .iter()
        .filter(|change| change.kind == kind)
        .map(|change| {
            if change.detail.is_empty() {
                change.subject.clone()
            } else {
                format!("{} ({})", change.subject, change.detail)
            }
        })
        .collect()
}

/// The change in sentences: the two states, then every kind of change with what it concerns.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let delta = Delta::new(spec);
    let locale = spec.locale;
    let mut description = text::delta_opening(
        locale,
        (&delta.before.title, &delta.after.title),
        delta.changes.len(),
    );
    for kind in ORDER {
        let entries = entries(&delta, kind);
        if !entries.is_empty() {
            description.push(' ');
            description.push_str(&text::delta_group(locale, kind, &entries));
        }
    }
    description
}

/// One row per change: its kind with the sign, what it concerns and how.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let delta = Delta::new(spec);
    let words = spec.locale.words();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: vec![
            words.change.to_owned(),
            words.subject.to_owned(),
            words.detail.to_owned(),
        ],
        rows: delta
            .changes
            .iter()
            .map(|change| {
                vec![
                    format!(
                        "{} {}",
                        sign(change.kind),
                        text::delta_kind(spec.locale, change.kind)
                    ),
                    change.subject.clone(),
                    change.detail.clone(),
                ]
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BuiltinMetrics, ChartSpec, RenderFormat, RenderOptions, render_json,
        scene::{Element, Scene},
        text_alternative,
    };

    const MODEL: &str = r#"{
        "schemaVersion": 1,
        "type": "architecture",
        "title": "Orders platform",
        "width": 1200,
        "height": 560,
        "architecture": {
            "components": [
                {"id": "web", "label": "Web shop", "kind": "frontend", "in": "cloud"},
                {"id": "orders", "label": "Orders", "sublabel": "Java", "in": "onprem"},
                {"id": "orders-next", "label": "Orders", "sublabel": "Kotlin", "in": "cloud", "replaces": "orders"},
                {"id": "legacy", "label": "Legacy ERP", "in": "onprem"},
                {"id": "queue", "label": "Event bus", "kind": "queue", "in": "cloud"},
                {"id": "billing", "label": "Billing", "in": "cloud"},
                {"id": "db", "label": "Orders DB", "kind": "database", "in": "cloud"}
            ],
            "boundaries": [
                {"id": "cloud", "label": "Cloud"},
                {"id": "onprem", "label": "On premises"}
            ],
            "connections": [
                {"from": "web", "to": "orders", "label": "HTTPS"},
                {"from": "orders", "to": "legacy", "label": "syncs"},
                {"from": "orders", "to": "db", "label": "SQL"},
                {"from": "web", "to": "orders-next", "label": "HTTPS"},
                {"from": "orders-next", "to": "queue", "label": "publishes"},
                {"from": "queue", "to": "billing", "label": "delivers"},
                {"from": "orders-next", "to": "db", "label": "SQL"}
            ],
            "views": [
                {"key": "today", "title": "Today", "components": ["web", "orders", "legacy", "db"]},
                {"key": "target", "title": "Target", "components": ["web", "orders-next", "queue", "billing", "db"]}
            ],
            "delta": {"before": "today", "after": "target"}
        }
    }"#;

    fn error(json: &str) -> (&'static str, String) {
        let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("the test specification is invalid");
        (error.code, error.path)
    }

    fn scene(json: &str) -> Scene {
        let spec = ChartSpec::from_json(json).expect("parses");
        let mut warnings = Vec::new();
        crate::layout::layout(&spec, &mut warnings, &BuiltinMetrics)
    }

    /// Where every text with this content stands.
    fn places(scene: &Scene, content: &str) -> Vec<(f64, f64)> {
        scene
            .elements
            .iter()
            .filter_map(|element| match element {
                Element::Text(text) if text.content == content => Some((text.x, text.y)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_ledger_lists_every_kind_of_change() {
        let alternative =
            text_alternative(&ChartSpec::from_json(MODEL).expect("parses")).expect("valid");
        assert_eq!(
            alternative.description,
            "Architecture change from Today to Target: 8 changes. Added: Event bus; Billing. Removed: Legacy ERP. Changed: Orders (technology: Java → Kotlin). Moved: Orders (from On premises to Cloud). Rewired: Orders → Legacy ERP (removed); Orders → Event bus (added); Event bus → Billing (added)."
        );
        assert_eq!(alternative.table.columns, ["Change", "Concerns", "Detail"]);
        assert_eq!(alternative.table.rows.len(), 8);
        assert_eq!(alternative.table.rows[0], ["+ added", "Event bus", ""]);
        assert_eq!(alternative.table.rows[2], ["− removed", "Legacy ERP", ""]);
    }

    #[test]
    fn a_component_in_both_states_stands_at_the_same_place_in_both_panels() {
        let scene = scene(MODEL);
        let web = places(&scene, "Web shop");
        let database = places(&scene, "Orders DB");
        assert_eq!((web.len(), database.len()), (2, 2));
        // Same height, and the second panel lies one fixed distance to the right.
        assert!((web[0].1 - web[1].1).abs() < 1e-9);
        assert!((database[0].1 - database[1].1).abs() < 1e-9);
        let offset = web[1].0 - web[0].0;
        assert!(offset > 100.0);
        assert!(((database[1].0 - database[0].0) - offset).abs() < 1e-9);
        // What only one state has stands once.
        assert_eq!(places(&scene, "Legacy ERP").len(), 1);
        assert_eq!(places(&scene, "Event bus").len(), 1);
        assert_eq!(places(&scene, "Billing").len(), 1);
        // The component that moved stands in both boundaries.
        assert_eq!(places(&scene, "Orders").len(), 2);
    }

    #[test]
    fn changes_carry_signs_not_only_color() {
        let svg = render_json(MODEL, RenderFormat::Svg, &RenderOptions::default())
            .expect("renders")
            .content;
        for sign in ["+", "−", "~", "→"] {
            assert!(svg.contains(&format!(">{sign}</text>")), "{sign}");
        }
        // The state before writes the old technology, the state after the new one.
        assert!(svg.contains(">Java</text>") && svg.contains(">Kotlin</text>"));
        assert!(svg.contains("Before: Today") && svg.contains("After: Target"));
    }

    #[test]
    fn panels_stand_one_above_the_other_on_a_narrow_canvas() {
        let wide = scene(MODEL);
        let narrow = scene(&MODEL.replace("\"width\": 1200", "\"width\": 700"));
        assert!(narrow.height > wide.height);
        let web = places(&narrow, "Web shop");
        assert!((web[0].0 - web[1].0).abs() < 1e-9, "the same left edge");
        assert!(web[1].1 > web[0].1);
    }

    #[test]
    fn the_output_is_deterministic() {
        let render = || {
            render_json(MODEL, RenderFormat::Svg, &RenderOptions::default())
                .expect("renders")
                .content
        };
        assert_eq!(render(), render());
    }

    #[test]
    fn invalid_deltas_name_the_field() {
        assert_eq!(
            error(&MODEL.replace("\"before\": \"today\"", "\"before\": \"yesterday\"")),
            ("unknown_view", "/architecture/delta/before".into())
        );
        assert_eq!(
            error(&MODEL.replace("\"after\": \"target\"", "\"after\": \"today\"")),
            ("same_view", "/architecture/delta/after".into())
        );
        assert_eq!(
            error(&MODEL.replace("\"replaces\": \"orders\"", "\"replaces\": \"nothing\"")),
            (
                "unknown_component",
                "/architecture/components/2/replaces".into()
            )
        );
    }

    #[test]
    fn a_view_can_still_be_rendered_alone() {
        let options = RenderOptions {
            view: Some("target".to_owned()),
            ..RenderOptions::default()
        };
        let svg = render_json(MODEL, RenderFormat::Svg, &options)
            .expect("renders")
            .content;
        assert!(!svg.contains("Before: Today"));
        assert!(svg.contains(">Event bus</text>"));
    }
}
