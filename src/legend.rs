//! The legend strip of a diagram: one horizontal band below the drawing that lists the node
//! kinds, the line patterns and the emphasis the diagram uses, each as a small sample and its
//! name. It is opt-in (`legend: "auto"`), shown only when the diagram uses more than one kind or
//! line style, wraps into rows on a narrow canvas and is left out in compact diagrams, which the
//! description says. The canvas grows by the strip, so it never lies inside the drawing.

use crate::{
    diagram::{arrowhead, pixels, polyline},
    flow::{Shape, shape},
    layout::{LABEL_SIZE, count},
    metrics::TextMetrics,
    roles::capitalized,
    scene::{Circle, Element, Line, Scene, Text, TextAnchor},
    spec::{
        ChartSpec, ChartType, ComponentKind, Dash, DiagramSpacing, Locale, MessageKind, NodeKind,
        StateKind, TreeNodeKind,
    },
    text,
};

/// The classes of the samples and of the names. The samples take their colors from the tokens of
/// the chart, so they follow every theme and the print variant.
pub(crate) const STYLE: &str = ".chartlet-legend-line{fill:none;stroke:var(--chartlet-zero);stroke-width:1.4;stroke-linejoin:round}.chartlet-legend-dashed{stroke-dasharray:5 4}.chartlet-legend-dotted{stroke-dasharray:1 4;stroke-linecap:round}.chartlet-legend-main{stroke:var(--chartlet-accent);stroke-width:2.2}.chartlet-legend-path{stroke:var(--chartlet-accent);stroke-width:3.2}.chartlet-legend-head{fill:var(--chartlet-zero);stroke:var(--chartlet-zero);stroke-width:1;stroke-linejoin:round}.chartlet-legend-open{fill:none;stroke:var(--chartlet-zero);stroke-width:1.4;stroke-linecap:round;stroke-linejoin:round}.chartlet-legend-emph{fill:var(--chartlet-accent);fill-opacity:.14;stroke:var(--chartlet-accent);stroke-width:3.5;rx:4px}.chartlet-legend-text{font-size:12px;fill:var(--chartlet-muted)}";

/// The width of a sample, its height, and the gap between it and its name.
const SAMPLE_WIDTH: f64 = 26.0;
const SAMPLE_HEIGHT: f64 = 16.0;
const SAMPLE_GAP: f64 = 8.0;
/// The room between two entries, and the height of a row.
const ENTRY_GAP: f64 = 22.0;
const ROW: f64 = 24.0;

/// What a legend entry shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Sample {
    /// A node of this shape.
    Shape(Shape),
    /// The box of an emphasized node.
    Emphasized,
    /// The ring of an emphasized node of a network, whose nodes are dots.
    EmphasizedRing,
    /// A line in a pattern.
    Line(LineStyle),
    /// A message of a sequence diagram: its line and its head.
    Message(MessageKind),
}

/// The patterns of a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineStyle {
    Solid,
    Dashed,
    Dotted,
    /// The main path, in the accent color.
    Main,
    /// The emphasized path, in the accent color and heavier.
    Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub sample: Sample,
    pub label: String,
}

/// The name of a shape in a flow chart, a state diagram or a tree.
const fn shape_name(locale: Locale, shape: Shape) -> &'static str {
    match (locale, shape) {
        (Locale::En, Shape::State) => "state",
        (Locale::En, Shape::Final) => "final state",
        (Locale::En, Shape::Decision) => "choice",
        (Locale::De, Shape::State) => "Zustand",
        (Locale::De, Shape::Final) => "Endzustand",
        (Locale::De, Shape::Decision) => "Wahl",
        (Locale::En, _) => "node",
        (Locale::De, _) => "Knoten",
    }
}

const fn line_name(locale: Locale, style: LineStyle) -> &'static str {
    match (locale, style) {
        (Locale::En, LineStyle::Solid) => "solid line",
        (Locale::En, LineStyle::Dashed) => "dashed line",
        (Locale::En, LineStyle::Dotted) => "dotted line",
        (Locale::En, LineStyle::Main) => "main path",
        (Locale::En, LineStyle::Path) => "emphasized path",
        (Locale::De, LineStyle::Solid) => "durchgezogene Linie",
        (Locale::De, LineStyle::Dashed) => "gestrichelte Linie",
        (Locale::De, LineStyle::Dotted) => "gepunktete Linie",
        (Locale::De, LineStyle::Main) => "Hauptweg",
        (Locale::De, LineStyle::Path) => "hervorgehobener Weg",
    }
}

const fn emphasized_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "emphasized",
        Locale::De => "hervorgehoben",
    }
}

/// The line patterns of edges whose `dash` is given, in their fixed order: solid, dashed, dotted.
fn dash_styles<'a>(dashes: impl Iterator<Item = &'a Option<Dash>>) -> Vec<LineStyle> {
    let dashes: Vec<Option<Dash>> = dashes.copied().collect();
    let mut styles = Vec::new();
    if dashes
        .iter()
        .any(|dash| matches!(dash, None | Some(Dash::Solid)))
    {
        styles.push(LineStyle::Solid);
    }
    if dashes.contains(&Some(Dash::Dashed)) {
        styles.push(LineStyle::Dashed);
    }
    if dashes.contains(&Some(Dash::Dotted)) {
        styles.push(LineStyle::Dotted);
    }
    styles
}

/// The node kinds (as shapes with their names) and the line patterns a diagram uses.
#[allow(clippy::too_many_lines)]
fn kinds_and_lines(spec: &ChartSpec) -> (Vec<(Sample, String)>, Vec<LineStyle>) {
    let locale = spec.locale;
    let shaped = |shape: Shape, name: &str| (Sample::Shape(shape), capitalized(name));
    match spec.chart_type {
        ChartType::Flow => {
            let flow = spec.flow.as_ref().expect("validated");
            let kinds = [
                NodeKind::Start,
                NodeKind::End,
                NodeKind::Process,
                NodeKind::Decision,
                NodeKind::Io,
                NodeKind::Subprocess,
                NodeKind::Store,
                NodeKind::External,
            ]
            .into_iter()
            .filter(|kind| flow.nodes.iter().any(|node| node.kind == *kind))
            .map(|kind| shaped(Shape::from(kind), text::node_kind(locale, kind)))
            .collect();
            (kinds, dash_styles(flow.edges.iter().map(|edge| &edge.dash)))
        }
        ChartType::State => {
            let machine = spec.state.as_ref().expect("validated");
            let uses = |wanted: Shape| {
                machine.states.iter().any(|state| {
                    let own = match (state.kind, state.is_final) {
                        (StateKind::Composite, _) => return false,
                        (StateKind::Choice, _) => Shape::Decision,
                        (_, true) => Shape::Final,
                        (_, false) => Shape::State,
                    };
                    own == wanted
                })
            };
            let kinds = [Shape::State, Shape::Final, Shape::Decision]
                .into_iter()
                .filter(|shape| uses(*shape))
                .map(|shape| shaped(shape, shape_name(locale, shape)))
                .collect();
            (
                kinds,
                dash_styles(machine.transitions.iter().map(|edge| &edge.dash)),
            )
        }
        ChartType::Architecture => {
            let architecture = spec.architecture.as_ref().expect("validated");
            let kinds = [
                ComponentKind::Person,
                ComponentKind::Frontend,
                ComponentKind::Service,
                ComponentKind::Database,
                ComponentKind::Queue,
                ComponentKind::Storage,
                ComponentKind::Cache,
                ComponentKind::Security,
                ComponentKind::External,
                ComponentKind::Node,
                ComponentKind::Artifact,
            ]
            .into_iter()
            .filter(|kind| {
                architecture
                    .components
                    .iter()
                    .any(|component| component.kind == *kind)
            })
            .map(|kind| {
                shaped(
                    crate::architecture::shape(kind),
                    text::component_kind(locale, kind),
                )
            })
            .collect();
            (
                kinds,
                dash_styles(architecture.connections.iter().map(|edge| &edge.dash)),
            )
        }
        ChartType::Tree => {
            let tree = spec.tree.as_ref().expect("validated");
            let kinds = [
                (TreeNodeKind::Unit, Shape::Process, "unit", "Einheit"),
                (TreeNodeKind::Person, Shape::Person, "person", "Person"),
                (
                    TreeNodeKind::External,
                    Shape::External,
                    "external",
                    "extern",
                ),
            ]
            .into_iter()
            .filter(|(kind, ..)| tree.nodes.iter().any(|node| node.kind == *kind))
            .map(|(_, shape, en, de)| {
                shaped(
                    shape,
                    match locale {
                        Locale::En => en,
                        Locale::De => de,
                    },
                )
            })
            .collect();
            (kinds, Vec::new())
        }
        ChartType::Network => {
            let network = spec.network.as_ref().expect("validated");
            (
                Vec::new(),
                dash_styles(network.edges.iter().map(|edge| &edge.dash)),
            )
        }
        ChartType::Sequence => {
            let sequence = spec.sequence.as_ref().expect("validated");
            let kinds = [MessageKind::Call, MessageKind::Reply, MessageKind::Async]
                .into_iter()
                .filter(|kind| {
                    sequence
                        .messages
                        .iter()
                        .any(|message| message.kind == *kind)
                })
                .map(|kind| {
                    (
                        Sample::Message(kind),
                        capitalized(text::message_kind(locale, kind)),
                    )
                })
                .collect();
            // The messages are the lines of a sequence diagram: their kinds are the entries.
            (kinds, Vec::new())
        }
        _ => (Vec::new(), Vec::new()),
    }
}

/// Whether the diagram has a main path that is drawn in the accent color.
fn has_main_path(spec: &ChartSpec) -> bool {
    spec.flow
        .as_ref()
        .is_some_and(|flow| !flow.main_path.is_empty())
        || spec
            .state
            .as_ref()
            .is_some_and(|machine| !machine.main_path.is_empty())
        || spec
            .architecture
            .as_ref()
            .is_some_and(|architecture| !architecture.main_path.is_empty())
}

/// The entries of the legend: node kinds, then the line patterns. Empty when the diagram uses at
/// most one kind and one pattern, which a legend would only repeat.
pub(crate) fn entries(spec: &ChartSpec) -> Vec<Entry> {
    let (mut kinds, mut lines) = kinds_and_lines(spec);
    let emphasis = spec.emphasis.as_ref();
    if emphasis.is_some_and(|emphasis| !emphasis.nodes.is_empty()) {
        kinds.push((
            if spec.chart_type == ChartType::Network {
                Sample::EmphasizedRing
            } else {
                Sample::Emphasized
            },
            capitalized(emphasized_name(spec.locale)),
        ));
    }
    if emphasis.is_some_and(|emphasis| !emphasis.path.is_empty()) {
        lines.push(LineStyle::Path);
    } else if has_main_path(spec) {
        lines.push(LineStyle::Main);
    }
    // Emphasis is worth a legend of its own: the accent says nothing until it is named.
    if kinds.len() < 2
        && lines.len() < 2
        && emphasis.is_none_or(|emphasis| emphasis.nodes.is_empty() && emphasis.path.is_empty())
    {
        return Vec::new();
    }
    let locale = spec.locale;
    kinds
        .into_iter()
        .map(|(sample, label)| Entry { sample, label })
        .chain(lines.into_iter().map(|style| Entry {
            sample: Sample::Line(style),
            label: capitalized(line_name(locale, style)),
        }))
        .collect()
}

/// Whether the diagram is drawn in compact spacing, which leaves no room for a strip.
fn is_compact(spec: &ChartSpec) -> bool {
    let spacing = match spec.chart_type {
        ChartType::Flow => spec.flow.as_ref().map(|flow| flow.spacing),
        ChartType::State => spec.state.as_ref().map(|machine| machine.spacing),
        ChartType::Architecture => spec
            .architecture
            .as_ref()
            .map(|architecture| architecture.spacing),
        ChartType::Classes => spec.classes.as_ref().map(|classes| classes.spacing),
        _ => None,
    };
    spacing == Some(DiagramSpacing::Compact)
}

/// What the legend of the chart is: its entries when it is drawn, `None` when it is not asked
/// for, not needed, or dropped in compact mode.
pub(crate) fn drawn(spec: &ChartSpec) -> Option<Vec<Entry>> {
    if !spec.wants_legend() || is_compact(spec) {
        return None;
    }
    Some(entries(spec)).filter(|entries| !entries.is_empty())
}

/// The sentence for the description: the legend's entries, or the note that compact mode left
/// the legend out. `None` when the chart asks for no legend or needs none.
pub(crate) fn sentence(spec: &ChartSpec) -> Option<String> {
    if !spec.wants_legend() {
        return None;
    }
    let entries = entries(spec);
    if entries.is_empty() {
        return None;
    }
    let locale = spec.locale;
    if is_compact(spec) {
        return Some(match locale {
            Locale::En => "The legend is left out in compact mode.".to_owned(),
            Locale::De => "Die Legende entfällt im kompakten Modus.".to_owned(),
        });
    }
    let names: Vec<String> = entries
        .iter()
        .map(|entry| entry.label.to_lowercase())
        .collect();
    Some(match locale {
        Locale::En => format!("Legend: {}.", names.join(", ")),
        Locale::De => format!("Legende: {}.", names.join(", ")),
    })
}

/// Draws the strip below the drawing, in rows that fit, and grows the canvas by its height.
pub(crate) fn push(spec: &ChartSpec, scene: &mut Scene, metrics: &impl TextMetrics) {
    let Some(entries) = drawn(spec) else {
        return;
    };
    let margin = 24.0_f64.min(f64::from(scene.width) / 12.0);
    let room = f64::from(scene.width) - 2.0 * margin;
    let top = f64::from(scene.height) + 4.0;
    let (mut x, mut row) = (0.0_f64, 0_usize);
    scene
        .elements
        .push(Element::Group("chartlet-legend-strip".to_owned()));
    for entry in &entries {
        let width = SAMPLE_WIDTH + SAMPLE_GAP + metrics.width(&entry.label, LABEL_SIZE);
        if x > 0.0 && x + width > room {
            x = 0.0;
            row += 1;
        }
        let middle = top + 8.0 + ROW * count(row);
        let left = margin + x;
        sample(&entry.sample, (left, middle), &mut scene.elements);
        scene.elements.push(Element::Text(Text {
            x: left + SAMPLE_WIDTH + SAMPLE_GAP,
            y: middle + 4.0,
            class: "chartlet-legend-text",
            anchor: TextAnchor::Start,
            content: entry.label.clone(),
        }));
        x += width + ENTRY_GAP;
    }
    scene.elements.push(Element::GroupEnd);
    scene.height = pixels(top + ROW * count(row + 1) + 4.0);
}

/// One sample, `SAMPLE_WIDTH` wide and centered vertically on `middle`.
fn sample(sample: &Sample, (left, middle): (f64, f64), elements: &mut Vec<Element>) {
    let line = |class: &'static str| {
        Element::Line(Line {
            x1: left,
            y1: middle,
            x2: left + SAMPLE_WIDTH,
            y2: middle,
            class,
        })
    };
    match sample {
        Sample::Shape(kind) => shape(
            *kind,
            (
                left,
                middle - SAMPLE_HEIGHT / 2.0,
                SAMPLE_WIDTH,
                SAMPLE_HEIGHT,
            ),
            None,
            elements,
        ),
        Sample::Emphasized => elements.push(Element::Rect(crate::scene::Rect {
            x: left,
            y: middle - SAMPLE_HEIGHT / 2.0,
            width: SAMPLE_WIDTH,
            height: SAMPLE_HEIGHT,
            class: "chartlet-legend-emph",
            series_index: None,
            style_index: None,
            tooltip: None,
        })),
        Sample::EmphasizedRing => elements.push(Element::Circle(Circle {
            cx: left + SAMPLE_WIDTH / 2.0,
            cy: middle,
            radius: SAMPLE_HEIGHT / 2.0,
            class: "chartlet-emph-ring",
            series_index: None,
            style_index: None,
            tooltip: None,
        })),
        Sample::Line(style) => elements.push(line(match style {
            LineStyle::Solid => "chartlet-legend-line",
            LineStyle::Dashed => "chartlet-legend-line chartlet-legend-dashed",
            LineStyle::Dotted => "chartlet-legend-line chartlet-legend-dotted",
            LineStyle::Main => "chartlet-legend-line chartlet-legend-main",
            LineStyle::Path => "chartlet-legend-line chartlet-legend-path",
        })),
        Sample::Message(kind) => {
            let (class, filled, head) = match kind {
                MessageKind::Call => ("chartlet-legend-line", true, "chartlet-legend-head"),
                MessageKind::Reply => (
                    "chartlet-legend-line chartlet-legend-dashed",
                    false,
                    "chartlet-legend-open",
                ),
                MessageKind::Async => ("chartlet-legend-line", false, "chartlet-legend-open"),
            };
            let tip = left + SAMPLE_WIDTH;
            elements.push(Element::Polyline(polyline(
                vec![(left, middle), (tip - 7.0, middle)],
                class,
                None,
            )));
            elements.push(Element::Polyline(arrowhead(
                (tip, middle),
                (1.0, 0.0),
                filled,
                head,
            )));
        }
    }
}
