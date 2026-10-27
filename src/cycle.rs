//! Cycles: steps on a ring, clockwise from twelve o'clock, each feeding the next, the last
//! feeding the first. The labels stand outside the ring beside numbered dots, or inside boxes on
//! it; a hub in the middle is fed by every step through a link of its own. The ring is as large
//! as the canvas lets it be and grows the canvas where the labels do not fit. The description
//! reads the cycle in order.

use std::f64::consts::TAU;

use crate::{
    DataTable,
    diagram::{self, HEAD, arrowhead, chip, pixels, warn_growth_advising, wrap},
    error::ChartWarning,
    flow::focus_classes,
    layout::{NARROW, count, fit_text, push_title, title_extra},
    metrics::TextMetrics,
    roles,
    scene::{Circle, Element, Hotspot, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, CycleLabels, CycleSpec},
    text,
};

const MARGIN: f64 = 24.0;
const COMPACT_MARGIN: f64 = 12.0;
const BOTTOM: f64 = 16.0;
/// The size of the labels of the steps and of their notes.
const LABEL_SIZE: f64 = 13.0;
const NOTE_SIZE: f64 = 11.0;
const LINE: f64 = 16.0;
const NOTE_LINE: f64 = 14.0;
/// The radius of the numbered dot of a step with its label outside.
const DOT: f64 = 15.0;
/// The space between a dot and its label.
const LABEL_GAP: f64 = 8.0;
/// The space an arrow along the ring keeps from the step it leaves and the step it reaches.
const ARROW_GAP: f64 = 3.0;
const MAX_LABEL: f64 = 150.0;
const COMPACT_MAX_LABEL: f64 = 84.0;
/// The widest label inside a box, before it wraps.
const MAX_BOX_LABEL: f64 = 120.0;
const COMPACT_MAX_BOX_LABEL: f64 = 84.0;
/// The room between a step and the hub for a link and its arrowhead.
const LINK_ROOM: f64 = 48.0;
const LABELLED_LINK_ROOM: f64 = 66.0;
/// The largest ring.
const MAX_RADIUS: f64 = 280.0;

fn cycle(spec: &ChartSpec) -> &CycleSpec {
    spec.cycle
        .as_ref()
        .expect("validated cycles carry a cycle block")
}

/// The angle of step `index` of `steps`, clockwise from twelve o'clock.
fn angle(index: usize, steps: usize) -> f64 {
    TAU * count(index) / count(steps)
}

/// A point at `radius` and `angle` from the middle.
fn polar(radius: f64, angle: f64) -> (f64, f64) {
    (radius * angle.sin(), -radius * angle.cos())
}

/// What a step shows: its label on one or two lines and its note on one.
struct Words {
    lines: Vec<String>,
    note: Option<String>,
    /// The width of the widest line.
    width: f64,
}

impl Words {
    fn height(&self) -> f64 {
        LINE * count(self.lines.len()) + self.note.as_ref().map_or(0.0, |_| NOTE_LINE)
    }
}

/// Where the steps stand, measured before the ring has a radius.
struct Measure {
    words: Vec<Words>,
    /// The size of the box of a step with its label inside; every step has the same.
    box_size: (f64, f64),
    hub: Option<HubWords>,
}

struct HubWords {
    lines: Vec<String>,
    note: Option<String>,
    radius: f64,
}

/// A box `(left, top, right, bottom)` around `(x, y)`.
type Bounds = (f64, f64, f64, f64);

fn measure(
    cycle: &CycleSpec,
    compact: bool,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Measure {
    let inside = cycle.labels == CycleLabels::Inside;
    let max_label = match (inside, compact) {
        (false, false) => MAX_LABEL,
        (false, true) => COMPACT_MAX_LABEL,
        (true, false) => MAX_BOX_LABEL,
        (true, true) => COMPACT_MAX_BOX_LABEL,
    };
    let words: Vec<Words> = cycle
        .steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let path = format!("/cycle/steps/{index}");
            let lines = wrap(
                &step.label,
                max_label,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("{path}/label"),
            );
            let note = step.note.as_ref().map(|note| {
                fit_text(
                    note,
                    max_label,
                    NOTE_SIZE,
                    metrics,
                    warnings,
                    &format!("{path}/note"),
                )
            });
            let width = lines
                .iter()
                .map(|line| metrics.width(line, LABEL_SIZE))
                .chain(note.iter().map(|note| metrics.width(note, NOTE_SIZE)))
                .fold(0.0, f64::max);
            Words { lines, note, width }
        })
        .collect();
    let box_size = (
        words.iter().map(|words| words.width).fold(60.0, f64::max) + 24.0,
        words
            .iter()
            .map(Words::height)
            .fold(0.0, f64::max)
            .max(LINE)
            + 20.0,
    );
    let hub = cycle.hub.as_ref().map(|hub| {
        let lines = wrap(
            &hub.label,
            110.0,
            LABEL_SIZE,
            metrics,
            warnings,
            "/cycle/hub/label",
        );
        let widest = lines
            .iter()
            .map(|line| metrics.width(line, LABEL_SIZE))
            .fold(0.0, f64::max);
        let note = hub
            .note
            .as_ref()
            .map(|note| fit_text(note, 120.0, NOTE_SIZE, metrics, warnings, "/cycle/hub/note"));
        let note_width = note
            .as_ref()
            .map_or(0.0, |note| metrics.width(note, NOTE_SIZE));
        let height = LINE * count(lines.len()) + note.as_ref().map_or(0.0, |_| NOTE_LINE);
        // The text lies in a circle: its corners must stay inside.
        let half = (widest.max(note_width) / 2.0).hypot(height / 2.0) + 8.0;
        HubWords {
            lines,
            note,
            radius: half.max(38.0),
        }
    });
    Measure {
        words,
        box_size,
        hub,
    }
}

/// How the labels of one step stand: where its first baseline is and which way the text runs.
struct LabelPlace {
    x: f64,
    first_baseline: f64,
    anchor: TextAnchor,
    bounds: Bounds,
}

/// Everything that depends on the radius of the ring, relative to its middle.
struct Ring {
    radius: f64,
    inside: bool,
    centres: Vec<(f64, f64)>,
    labels: Vec<LabelPlace>,
}

impl Ring {
    fn new(cycle: &CycleSpec, measure: &Measure, radius: f64) -> Self {
        let steps = cycle.steps.len();
        let inside = cycle.labels == CycleLabels::Inside;
        let centres: Vec<(f64, f64)> = (0..steps)
            .map(|index| polar(radius, angle(index, steps)))
            .collect();
        let labels = if inside {
            Vec::new()
        } else {
            (0..steps)
                .map(|index| outside_label(index, steps, centres[index], &measure.words[index]))
                .collect()
        };
        Self {
            radius,
            inside,
            centres,
            labels,
        }
    }

    /// The box of the shape of step `index` itself: its dot or its box.
    fn shape(&self, index: usize, measure: &Measure) -> Bounds {
        let (x, y) = self.centres[index];
        let (half_width, half_height) = if self.inside {
            (measure.box_size.0 / 2.0, measure.box_size.1 / 2.0)
        } else {
            (DOT, DOT)
        };
        (
            x - half_width,
            y - half_height,
            x + half_width,
            y + half_height,
        )
    }

    /// The box around everything the ring draws, hub included.
    fn extent(&self, measure: &Measure) -> Bounds {
        let mut extent = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
        let mut cover = |bounds: Bounds| {
            extent = (
                extent.0.min(bounds.0),
                extent.1.min(bounds.1),
                extent.2.max(bounds.2),
                extent.3.max(bounds.3),
            );
        };
        for index in 0..self.centres.len() {
            cover(self.shape(index, measure));
            if let Some(place) = self.labels.get(index) {
                cover(place.bounds);
            }
        }
        if let Some(hub) = &measure.hub {
            cover((-hub.radius, -hub.radius, hub.radius, hub.radius));
        }
        extent
    }
}

/// The label of a step beside its dot: above a dot at the top, below one at the bottom, beside it
/// otherwise, on the side that faces away from the middle.
fn outside_label(index: usize, steps: usize, (x, y): (f64, f64), words: &Words) -> LabelPlace {
    let theta = angle(index, steps);
    let (sin, cos) = (theta.sin(), theta.cos());
    let (width, height) = (words.width, words.height());
    let (anchor, label_x, top) = if sin.abs() < 0.25 {
        let top = if cos > 0.0 {
            y - DOT - 6.0 - height
        } else {
            y + DOT + 6.0
        };
        (TextAnchor::Middle, x, top)
    } else if sin > 0.0 {
        (TextAnchor::Start, x + DOT + LABEL_GAP, y - height / 2.0)
    } else {
        (TextAnchor::End, x - DOT - LABEL_GAP, y - height / 2.0)
    };
    let left = match anchor {
        TextAnchor::Start => label_x,
        TextAnchor::Middle => label_x - width / 2.0,
        TextAnchor::End => label_x - width,
    };
    LabelPlace {
        x: label_x,
        first_baseline: top + 11.0,
        anchor,
        bounds: (
            left - 2.0,
            top - 2.0,
            left + width + 2.0,
            top + height + 2.0,
        ),
    }
}

/// How far `(x, y)` lies from the box `(left, top, right, bottom)`; 0 inside it.
fn distance_to(bounds: Bounds, (x, y): (f64, f64)) -> f64 {
    let dx = (bounds.0 - x).max(x - bounds.2).max(0.0);
    let dy = (bounds.1 - y).max(y - bounds.3).max(0.0);
    dx.hypot(dy)
}

/// The smallest radius at which the steps keep apart, and keep the room the hub needs.
fn least_radius(cycle: &CycleSpec, measure: &Measure) -> f64 {
    let steps = cycle.steps.len();
    let linked = if cycle.steps.iter().any(|step| step.hub_label.is_some()) {
        LABELLED_LINK_ROOM
    } else {
        LINK_ROOM
    };
    let mut radius = 60.0;
    loop {
        let ring = Ring::new(cycle, measure, radius);
        let shapes: Vec<Bounds> = (0..steps).map(|step| ring.shape(step, measure)).collect();
        // A gap that leaves an arrow along the ring room to be seen.
        let apart = shapes.iter().enumerate().all(|(index, one)| {
            shapes[index + 1..].iter().all(|other| {
                let gap = 2.0 * ARROW_GAP + 3.0 * HEAD;
                one.0 - other.2 >= gap
                    || other.0 - one.2 >= gap
                    || one.1 - other.3 >= gap
                    || other.1 - one.3 >= gap
            })
        });
        let room = measure.hub.as_ref().is_none_or(|hub| {
            shapes
                .iter()
                .all(|shape| distance_to(*shape, (0.0, 0.0)) >= hub.radius + linked)
        });
        if (apart && room) || radius >= MAX_RADIUS * 2.0 {
            return radius;
        }
        radius += 4.0;
    }
}

struct Plan {
    ring: Ring,
    center: (f64, f64),
    width: f64,
    height: f64,
}

fn plan(cycle: &CycleSpec, measure: &Measure, spec: &ChartSpec, (top, margin): (f64, f64)) -> Plan {
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let least = least_radius(cycle, measure);
    let room = (width - 2.0 * margin, height - top - BOTTOM);
    let mut radius = MAX_RADIUS.max(least);
    let ring = loop {
        let ring = Ring::new(cycle, measure, radius);
        let extent = ring.extent(measure);
        let fits = extent.2 - extent.0 <= room.0 && extent.3 - extent.1 <= room.1;
        if fits || radius <= least {
            break ring;
        }
        radius = (radius - 4.0).max(least);
    };
    let extent = ring.extent(measure);
    let page_width = width.max(extent.2 - extent.0 + 2.0 * margin);
    let page_height = height.max(top + extent.3 - extent.1 + BOTTOM);
    let center = (
        page_width / 2.0 - f64::midpoint(extent.0, extent.2),
        // Spare height is shared above and below, but never more than a margin above.
        top + ((page_height - top - BOTTOM - (extent.3 - extent.1)) / 2.0).min(24.0) - extent.1,
    );
    Plan {
        ring,
        center,
        width: page_width,
        height: page_height,
    }
}

/// The points of the arrow along the ring from step `from` to step `to`, ending at the tip of
/// its head: the arc between the two shapes, `ARROW_GAP` clear of both.
fn arc(plan: &Plan, measure: &Measure, from: usize, to: usize) -> Option<Vec<(f64, f64)>> {
    let steps = plan.ring.centres.len();
    let start = angle(from, steps);
    let end = start + TAU / count(steps);
    let grown = |index: usize| {
        let shape = plan.ring.shape(index, measure);
        (
            shape.0 - ARROW_GAP,
            shape.1 - ARROW_GAP,
            shape.2 + ARROW_GAP,
            shape.3 + ARROW_GAP,
        )
    };
    let (leaving, reaching) = (grown(from), grown(to));
    let outside = |(x, y): (f64, f64), bounds: Bounds| {
        if plan.ring.inside {
            x < bounds.0 || x > bounds.2 || y < bounds.1 || y > bounds.3
        } else {
            let middle = (
                f64::midpoint(bounds.0, bounds.2),
                f64::midpoint(bounds.1, bounds.3),
            );
            (x - middle.0).hypot(y - middle.1) > (bounds.2 - bounds.0) / 2.0
        }
    };
    let samples = 240;
    let points: Vec<(f64, f64)> = (0..=samples)
        .map(|sample| {
            let theta = start + (end - start) * f64::from(sample) / f64::from(samples);
            polar(plan.ring.radius, theta)
        })
        .filter(|&point| outside(point, leaving) && outside(point, reaching))
        .collect();
    if points.len() < 2 {
        return None;
    }
    Some(points)
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let cycle = cycle(spec);
    let compact = spec.width < NARROW;
    let margin = if compact { COMPACT_MARGIN } else { MARGIN };
    let width = f64::from(spec.width);
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let measure = measure(cycle, compact, metrics, warnings);
    let plan = plan(cycle, &measure, spec, (top, margin));
    let steps = cycle.steps.len();
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        plan.width - 2.0 * margin,
        metrics,
        warnings,
    );
    warn_growth_advising(spec, (plan.width, plan.height), "use fewer steps", warnings);
    let at = |(x, y): (f64, f64)| (plan.center.0 + x, plan.center.1 + y);
    // Arrows along the ring.
    for from in 0..steps {
        let to = (from + 1) % steps;
        let Some(mut points) = arc(&plan, &measure, from, to) else {
            continue;
        };
        let tip = points[points.len() - 1];
        // The line ends where the head begins.
        let cut = points
            .iter()
            .rposition(|point| (point.0 - tip.0).hypot(point.1 - tip.1) >= HEAD)
            .unwrap_or(0);
        points.truncate(cut + 1);
        let before = points[points.len() - 1];
        let length = (tip.0 - before.0).hypot(tip.1 - before.1).max(1e-9);
        let direction = ((tip.0 - before.0) / length, (tip.1 - before.1) / length);
        let tooltip = format!("{} → {}", cycle.steps[from].label, cycle.steps[to].label);
        elements.push(Element::Group(focus_classes(None, &sorted(&[from, to]))));
        elements.push(Element::Polyline(diagram::polyline(
            points.into_iter().map(at).collect(),
            "chartlet-flow-edge",
            Some(tooltip),
        )));
        elements.push(Element::Polyline(arrowhead(
            at(tip),
            direction,
            true,
            "chartlet-flow-head",
        )));
        elements.push(Element::GroupEnd);
    }
    // The hub and its links.
    if let (Some(hub), Some(spec_hub)) = (&measure.hub, &cycle.hub) {
        let middle = at((0.0, 0.0));
        for (index, step) in cycle.steps.iter().enumerate() {
            let centre = plan.ring.centres[index];
            let length = centre.0.hypot(centre.1).max(1e-9);
            let toward = (-centre.0 / length, -centre.1 / length);
            let leave = if plan.ring.inside {
                let (half_width, half_height) =
                    (measure.box_size.0 / 2.0, measure.box_size.1 / 2.0);
                let along_x = if toward.0.abs() < 1e-9 {
                    f64::INFINITY
                } else {
                    half_width / toward.0.abs()
                };
                let along_y = if toward.1.abs() < 1e-9 {
                    f64::INFINITY
                } else {
                    half_height / toward.1.abs()
                };
                along_x.min(along_y)
            } else {
                DOT
            } + ARROW_GAP;
            let from = (centre.0 + toward.0 * leave, centre.1 + toward.1 * leave);
            let tip_distance = hub.radius + ARROW_GAP;
            let tip = (-toward.0 * tip_distance, -toward.1 * tip_distance);
            let end = (tip.0 - toward.0 * HEAD, tip.1 - toward.1 * HEAD);
            elements.push(Element::Group(focus_classes(None, &[index])));
            elements.push(Element::Polyline(diagram::polyline(
                vec![at(from), at(end)],
                "chartlet-flow-edge chartlet-cycle-link",
                Some(format!("{} → {}", step.label, spec_hub.label)),
            )));
            elements.push(Element::Polyline(arrowhead(
                at(tip),
                toward,
                true,
                "chartlet-flow-head",
            )));
            if let Some(label) = &step.hub_label {
                let label = fit_text(
                    label,
                    120.0,
                    diagram::CHIP_SIZE,
                    metrics,
                    warnings,
                    &format!("/cycle/steps/{index}/hubLabel"),
                );
                let half = (f64::midpoint(from.0, end.0), f64::midpoint(from.1, end.1));
                let (x, y) = at(half);
                chip(
                    &[label],
                    (x, y + 4.0),
                    TextAnchor::Middle,
                    metrics,
                    &mut elements,
                );
            }
            elements.push(Element::GroupEnd);
        }
        elements.push(Element::Circle(Circle {
            cx: middle.0,
            cy: middle.1,
            radius: hub.radius,
            class: "chartlet-flow-node chartlet-role-teal chartlet-cycle-hub",
            series_index: None,
            style_index: None,
            tooltip: Some(spec_hub.label.clone()),
        }));
        let height = LINE * count(hub.lines.len()) + hub.note.as_ref().map_or(0.0, |_| NOTE_LINE);
        let mut baseline = middle.1 - height / 2.0 + 12.0;
        for line in &hub.lines {
            elements.push(Element::Text(Text {
                x: middle.0,
                y: baseline,
                class: "chartlet-flow-label",
                anchor: TextAnchor::Middle,
                content: line.clone(),
            }));
            baseline += LINE;
        }
        if let Some(note) = &hub.note {
            elements.push(Element::Text(Text {
                x: middle.0,
                y: baseline - 2.0,
                class: "chartlet-flow-sublabel",
                anchor: TextAnchor::Middle,
                content: note.clone(),
            }));
        }
    }
    // The steps.
    for (index, step) in cycle.steps.iter().enumerate() {
        let previous = (index + steps - 1) % steps;
        let next = (index + 1) % steps;
        let mut classes = focus_classes(Some(index), &sorted(&[previous, index, next]));
        if let Some(role) = step.role {
            classes.push_str(roles::class(role));
        }
        elements.push(Element::NodeGroup(classes, step.id.clone()));
        let (x, y) = at(plan.ring.centres[index]);
        let words = &measure.words[index];
        let tooltip = step.note.as_ref().map_or_else(
            || step.label.clone(),
            |note| format!("{}: {note}", step.label),
        );
        let shape = plan.ring.shape(index, &measure);
        let shape = (
            plan.center.0 + shape.0,
            plan.center.1 + shape.1,
            plan.center.0 + shape.2,
            plan.center.1 + shape.3,
        );
        let mut hotspot = shape;
        if plan.ring.inside {
            diagram::with_shadow(
                Element::Rect(Rect {
                    x: shape.0,
                    y: shape.1,
                    width: shape.2 - shape.0,
                    height: shape.3 - shape.1,
                    class: "chartlet-flow-node chartlet-role-blue",
                    series_index: None,
                    style_index: None,
                    tooltip: Some(tooltip),
                }),
                &mut elements,
            );
            let mut baseline = y - words.height() / 2.0 + 12.0;
            for line in &words.lines {
                elements.push(Element::Text(Text {
                    x,
                    y: baseline,
                    class: "chartlet-flow-label",
                    anchor: TextAnchor::Middle,
                    content: line.clone(),
                }));
                baseline += LINE;
            }
            if let Some(note) = &words.note {
                elements.push(Element::Text(Text {
                    x,
                    y: baseline - 2.0,
                    class: "chartlet-flow-sublabel",
                    anchor: TextAnchor::Middle,
                    content: note.clone(),
                }));
            }
            diagram::badge(index + 1, (shape.0 + 2.0, shape.1 + 2.0), &mut elements);
        } else {
            elements.push(Element::Circle(Circle {
                cx: x,
                cy: y,
                radius: DOT,
                class: "chartlet-flow-node chartlet-role-blue",
                series_index: None,
                style_index: None,
                tooltip: Some(tooltip),
            }));
            elements.push(Element::Text(Text {
                x,
                y: y + 4.5,
                class: "chartlet-cycle-num",
                anchor: TextAnchor::Middle,
                content: (index + 1).to_string(),
            }));
            let place = &plan.ring.labels[index];
            let mut baseline = plan.center.1 + place.first_baseline;
            for line in &words.lines {
                elements.push(Element::Text(Text {
                    x: plan.center.0 + place.x,
                    y: baseline,
                    class: "chartlet-flow-label",
                    anchor: place.anchor,
                    content: line.clone(),
                }));
                baseline += LINE;
            }
            if let Some(note) = &words.note {
                elements.push(Element::Text(Text {
                    x: plan.center.0 + place.x,
                    y: baseline - 2.0,
                    class: "chartlet-flow-sublabel",
                    anchor: place.anchor,
                    content: note.clone(),
                }));
            }
            let label = place.bounds;
            hotspot = (
                shape.0.min(plan.center.0 + label.0),
                shape.1.min(plan.center.1 + label.1),
                shape.2.max(plan.center.0 + label.2),
                shape.3.max(plan.center.1 + label.3),
            );
        }
        if let Some(role) = step.role {
            roles::push_badge(role, (shape.2 - 2.0, shape.1 + 2.0), &mut elements);
        }
        elements.push(Element::GroupEnd);
        elements.push(Element::Hotspot(Hotspot {
            node: index,
            label: step.label.clone(),
            x: hotspot.0,
            y: hotspot.1,
            width: hotspot.2 - hotspot.0,
            height: hotspot.3 - hotspot.1,
        }));
    }
    let mut scene = Scene {
        width: pixels(plan.width),
        height: pixels(plan.height),
        elements,
    };
    let used = roles::used(cycle.steps.iter().map(|step| step.role), std::iter::empty());
    roles::push_legend(
        &mut scene,
        spec.locale,
        &used,
        (margin, roles::push_badge),
        metrics,
    );
    scene
}

fn sorted(nodes: &[usize]) -> Vec<usize> {
    let mut nodes = nodes.to_vec();
    nodes.sort_unstable();
    nodes.dedup();
    nodes
}

/// `1 Label` for the description, with the note in parentheses.
fn step_phrase(cycle: &CycleSpec, index: usize) -> String {
    let step = &cycle.steps[index];
    match &step.note {
        Some(note) => format!("{} {} ({note})", index + 1, step.label),
        None => format!("{} {}", index + 1, step.label),
    }
}

/// The cycle in sentences: its steps in order and the way back, the hub and its links, and the
/// roles.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let cycle = cycle(spec);
    let locale = spec.locale;
    let phrases: Vec<String> = (0..cycle.steps.len())
        .map(|index| step_phrase(cycle, index))
        .collect();
    let mut description = text::cycle_opening(locale, &phrases, &cycle.steps[0].label);
    if let Some(hub) = &cycle.hub {
        let links: Vec<String> = cycle
            .steps
            .iter()
            .filter_map(|step| {
                step.hub_label
                    .as_ref()
                    .map(|label| format!("{}: {label}", step.label))
            })
            .collect();
        let name = hub.note.as_ref().map_or_else(
            || hub.label.clone(),
            |note| format!("{} ({note})", hub.label),
        );
        description.push(' ');
        description.push_str(&text::cycle_hub(locale, &name, &links));
    }
    let named: Vec<_> = cycle
        .steps
        .iter()
        .filter_map(|step| step.role.map(|role| (role, step.label.clone())))
        .collect();
    if let Some(sentence) = roles::sentence(locale, &named) {
        description.push(' ');
        description.push_str(&sentence);
    }
    description
}

/// One row per step in order: its number, label, note, the label of its link to the hub and its
/// role, where the cycle has them.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let cycle = cycle(spec);
    let words = spec.locale.words();
    let notes = cycle.steps.iter().any(|step| step.note.is_some());
    let links = cycle.steps.iter().any(|step| step.hub_label.is_some());
    let with_roles = cycle.steps.iter().any(|step| step.role.is_some());
    let mut columns = vec![words.number.to_owned(), words.step.to_owned()];
    if notes {
        columns.push(words.notes.to_owned());
    }
    if links {
        columns.push(words.hub_link.to_owned());
    }
    if with_roles {
        columns.push(words.role.to_owned());
    }
    let rows = cycle
        .steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let mut row = vec![(index + 1).to_string(), step.label.clone()];
            if notes {
                row.push(step.note.clone().unwrap_or_default());
            }
            if links {
                row.push(step.hub_label.clone().unwrap_or_default());
            }
            if with_roles {
                row.push(
                    step.role
                        .map_or_else(String::new, |role| text::role(spec.locale, role).to_owned()),
                );
            }
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use crate::{ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative};

    const SPEC: &str = r#"{
        "schemaVersion": 1,
        "type": "cycle",
        "title": "Product loop",
        "cycle": {
            "steps": [
                {"id": "plan", "label": "Plan", "hubLabel": "scope"},
                {"id": "build", "label": "Build", "note": "in small batches"},
                {"id": "ship", "label": "Ship", "role": "critical"},
                {"id": "learn", "label": "Learn"}
            ],
            "hub": {"label": "Customers"}
        }
    }"#;

    fn error(json: &str) -> (&'static str, String) {
        let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("the test specification is invalid");
        (error.code, error.path)
    }

    #[test]
    fn the_description_reads_the_cycle_in_order() {
        let alternative =
            text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
        assert_eq!(
            alternative.description,
            "Cycle of 4 steps: 1 Plan; 2 Build (in small batches); 3 Ship; 4 Learn; the last leads back to Plan. Every step feeds the hub Customers. Links: Plan: scope. Roles: critical: Ship."
        );
        assert_eq!(
            alternative.table.columns,
            ["No.", "Step", "Notes", "Hub link", "Role"]
        );
        assert_eq!(alternative.table.rows.len(), 4);
        assert_eq!(alternative.table.rows[2], ["3", "Ship", "", "", "critical"]);
    }

    #[test]
    fn german_texts_follow_the_locale() {
        let json = SPEC.replace(
            "\"type\": \"cycle\"",
            "\"type\": \"cycle\", \"locale\": \"de\"",
        );
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(
            alternative
                .description
                .starts_with("Kreislauf aus 4 Schritten: 1 Plan;")
        );
        assert!(
            alternative
                .description
                .contains("Jeder Schritt speist die Mitte Customers.")
        );
    }

    #[test]
    fn both_label_placements_draw_every_step_and_arrow() {
        for labels in ["outside", "inside"] {
            let json = SPEC.replace("\"hub\":", &format!("\"labels\": \"{labels}\", \"hub\":"));
            let svg = render_json(&json, RenderFormat::Svg, &RenderOptions::default())
                .expect("renders")
                .content;
            assert_eq!(
                svg.matches("class=\"chartlet-flow-head\"").count(),
                4 + 4,
                "{labels}"
            );
            for label in ["Plan", "Build", "Ship", "Learn", "Customers", "scope"] {
                assert!(
                    svg.contains(&format!(">{label}</text>")),
                    "{labels}: {label}"
                );
            }
        }
    }

    #[test]
    fn the_output_is_deterministic() {
        let render = || {
            render_json(SPEC, RenderFormat::Svg, &RenderOptions::default())
                .expect("renders")
                .content
        };
        assert_eq!(render(), render());
    }

    #[test]
    fn invalid_cycles_name_the_field() {
        let steps = |count: usize| {
            let steps: Vec<String> = (0..count)
                .map(|index| format!(r#"{{"id": "s{index}", "label": "Step {index}"}}"#))
                .collect();
            format!(
                r#"{{"schemaVersion": 1, "type": "cycle", "title": "T", "cycle": {{"steps": [{}]}}}}"#,
                steps.join(",")
            )
        };
        assert_eq!(error(&steps(2)), ("too_few_steps", "/cycle/steps".into()));
        assert_eq!(error(&steps(9)), ("too_many_steps", "/cycle/steps".into()));
        assert_eq!(
            error(&steps(3).replace("\"s1\"", "\"s0\"")),
            ("duplicate_id", "/cycle/steps/1/id".into())
        );
        assert_eq!(
            error(&steps(3).replace("\"Step 1\"", "\"Step 1\", \"hubLabel\": \"x\"")),
            ("option_not_supported", "/cycle/steps/1/hubLabel".into())
        );
        assert_eq!(
            error(r#"{"schemaVersion": 1, "type": "cycle", "title": "T"}"#),
            ("missing_cycle", "/cycle".into())
        );
        assert_eq!(
            error(&steps(3).replace("\"type\": \"cycle\"", "\"type\": \"bar\"")),
            ("option_not_supported", "/cycle".into())
        );
    }

    #[test]
    fn a_ring_of_eight_with_long_labels_still_renders() {
        let steps: Vec<String> = (0..8)
            .map(|index| {
                format!(
                    r#"{{"id": "s{index}", "label": "A rather long step label number {index}"}}"#
                )
            })
            .collect();
        for labels in ["outside", "inside"] {
            let json = format!(
                r#"{{"schemaVersion": 1, "type": "cycle", "title": "T", "width": 360, "cycle": {{"labels": "{labels}", "steps": [{}]}}}}"#,
                steps.join(",")
            );
            render_json(&json, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
        }
    }
}
