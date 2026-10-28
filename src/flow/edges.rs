//! Drawing edges: their lines, the marks and texts at their ends, their labels, and the loops
//! from a step to itself.

use super::{
    LINE, LOOP, SOCKET,
    measure::Model,
    model::{End, Ends, Shape},
    place::Plan,
    route::Route,
};
use crate::{
    diagram::{self, CHIP_LINE, CHIP_REACH, HEAD, arrowhead, chip, chip_box, rounded},
    layout::count,
    metrics::TextMetrics,
    scene::{Element, Text, TextAnchor},
    spec::Dash,
};

/// Half the width and the height of the hop of an edge over one it crosses, and how far from the
/// end of a run it must lie.
const HOP_REACH: f64 = 4.0;
const HOP_HEIGHT: f64 = 4.0;
const HOP_CLEAR: f64 = 12.0;

/// Where an edge ends: its tip and the direction the line runs into it.
type EndPlace = ((f64, f64), (f64, f64));

/// The marks and texts at both ends of an edge: each end's tip and the direction the line runs
/// into it.
fn push_ends(
    ends: &Ends,
    ((tip, direction), (start, tail_direction)): (EndPlace, EndPlace),
    head_class: &'static str,
    elements: &mut Vec<Element>,
) {
    for (end, at, along, text) in [
        (ends.head, tip, direction, &ends.head_text),
        (ends.tail, start, tail_direction, &ends.tail_text),
    ] {
        if let Some(marker) = end_marker(end, at, along, head_class) {
            elements.push(Element::Polyline(marker));
        }
        if let Some(text) = text {
            elements.push(Element::Text(end_text(text, end, at, along)));
        }
    }
}

/// How far a mark at the end of an edge reaches back along the line.
fn end_length(end: End) -> f64 {
    match end {
        End::None | End::Socket => 0.0,
        End::Arrow | End::Open => HEAD,
        End::Triangle => 12.0,
        End::DiamondFilled | End::DiamondHollow => 16.0,
    }
}

fn unit(vector: (f64, f64)) -> (f64, f64) {
    let length = vector.0.hypot(vector.1);
    if length > 0.0 {
        (vector.0 / length, vector.1 / length)
    } else {
        (0.0, 1.0)
    }
}

/// The mark of `end` with its tip at `tip`, pointing along the unit vector `direction`.
fn end_marker(
    end: End,
    tip: (f64, f64),
    direction: (f64, f64),
    arrow_class: &'static str,
) -> Option<crate::scene::Polyline> {
    let back = |distance: f64, across: f64| {
        let side = (-direction.1, direction.0);
        (
            tip.0 - direction.0 * distance + side.0 * across,
            tip.1 - direction.1 * distance + side.1 * across,
        )
    };
    let points = match end {
        End::None => return None,
        End::Socket => {
            // A half circle around the ball ahead of the tip, open towards it.
            let center = (tip.0 + direction.0 * SOCKET, tip.1 + direction.1 * SOCKET);
            let back = (-direction.1).atan2(-direction.0);
            let points = (0..=12)
                .map(|step| {
                    let angle = back + std::f64::consts::PI * (f64::from(step) / 12.0 - 0.5);
                    (
                        center.0 + SOCKET * angle.cos(),
                        center.1 + SOCKET * angle.sin(),
                    )
                })
                .collect();
            let class = if arrow_class.ends_with("main-head") {
                "chartlet-arch-socket chartlet-arch-socket-main"
            } else {
                "chartlet-arch-socket"
            };
            return Some(diagram::polyline(points, class, None));
        }
        End::Arrow => return Some(arrowhead(tip, direction, true, arrow_class)),
        End::Open => return Some(arrowhead(tip, direction, false, "chartlet-flow-open")),
        End::Triangle => vec![tip, back(12.0, 6.0), back(12.0, -6.0), tip],
        End::DiamondFilled | End::DiamondHollow => {
            vec![tip, back(8.0, 5.0), back(16.0, 0.0), back(8.0, -5.0), tip]
        }
    };
    let class = match end {
        End::DiamondFilled => "chartlet-flow-head chartlet-flow-solid",
        _ => "chartlet-flow-head chartlet-flow-hollow",
    };
    Some(diagram::polyline(points, class, None))
}

/// The text at an end of an edge, such as a cardinality, beside the line a little way from the
/// mark.
fn end_text(text: &str, end: End, at: (f64, f64), direction: (f64, f64)) -> Text {
    let reach = end_length(end) + 8.0;
    // On the left of the line, or above it, where the labels of the edges do not stand.
    let mut side = (-direction.1, direction.0);
    if side.0 > 1e-6 || (side.0.abs() <= 1e-6 && side.1 > 0.0) {
        side = (-side.0, -side.1);
    }
    let (x, y) = (
        at.0 - direction.0 * reach + side.0 * 12.0,
        at.1 - direction.1 * reach + side.1 * 12.0,
    );
    Text {
        x,
        y: y + 4.0,
        class: "chartlet-flow-card",
        anchor: if side.0 >= 0.0 {
            TextAnchor::Start
        } else {
            TextAnchor::End
        },
        content: text.to_owned(),
    }
}

/// An edge label waiting to be drawn: its lines, the first baseline and how it is anchored.
pub(super) struct Label {
    lines: Vec<String>,
    /// The focus classes of the edge it labels.
    classes: String,
    at: (f64, f64),
    anchor: TextAnchor,
    /// Where the line it labels runs across the page, when the label stands beside it and may
    /// move to its other side.
    beside: Option<f64>,
    /// Whether the label of an edge that runs against the flow was set above its end, and moves
    /// further up, not down, when it meets a step.
    up: bool,
}

impl Label {
    /// Moves a label that would run off the side of the page to the other side of its line, or,
    /// beside nothing, back onto the page.
    fn keep_on_page(&mut self, width: f64, metrics: &impl TextMetrics) {
        let (left, _, chip_width, _) = chip_box(&self.lines, self.at, self.anchor, metrics);
        let reach = CHIP_REACH + 2.0;
        let over = left + chip_width - (width - 2.0);
        if over > 0.0 {
            match (self.beside, self.anchor) {
                (Some(line), TextAnchor::Start) => {
                    self.at.0 = line - reach;
                    self.anchor = TextAnchor::End;
                }
                _ => self.at.0 -= over,
            }
        } else if left < 2.0 {
            match (self.beside, self.anchor) {
                (Some(line), TextAnchor::End) => {
                    self.at.0 = line + reach;
                    self.anchor = TextAnchor::Start;
                }
                _ => self.at.0 += 2.0 - left,
            }
        }
        // A label wider than the page, moved back from the right edge, ends up past the left
        // one: it starts at the left edge instead, and the canvas grows to its right.
        let (left, _, _, _) = chip_box(&self.lines, self.at, self.anchor, metrics);
        if left < 2.0 {
            self.at.0 += 2.0 - left;
        }
    }
}

impl Plan {
    /// The label of an edge that leaves a step beside others that go the same way. Beyond their
    /// lines it would read as theirs, so in portrait it goes under the longest stretch the edge
    /// runs across instead.
    fn run_label(
        &self,
        model: &Model,
        edge: usize,
        route: &Route,
        lines: &[String],
    ) -> Option<Label> {
        if self.landscape || route.clear <= 0.5 {
            return None;
        }
        let corners: Vec<(f64, f64)> = route.points.iter().map(|p| self.page(*p)).collect();
        let run = corners
            .windows(2)
            .filter(|pair| (pair[0].1 - pair[1].1).abs() < 0.5)
            .max_by(|a, b| (a[0].0 - a[1].0).abs().total_cmp(&(b[0].0 - b[1].0).abs()))?;
        Some(Label {
            lines: lines.to_vec(),
            classes: model.edge_classes(edge),
            at: (f64::midpoint(run[0].0, run[1].0), run[0].1 + 18.0),
            anchor: TextAnchor::Middle,
            beside: None,
            up: false,
        })
    }

    /// `points`, the way of edge `edge`, with a small hop where a horizontal run crosses a vertical
    /// run of another edge on its own. A crossing that lies close to the end of a run or to
    /// another crossing on it stays a plain crossing.
    fn hop_over_crossings(
        &self,
        model: &Model,
        edge: usize,
        points: &[(f64, f64)],
    ) -> Vec<(f64, f64)> {
        let shares =
            |edge: usize| model.trunks.leave[edge].is_some() || model.trunks.reach[edge].is_some();
        let verticals: Vec<(f64, f64, f64)> = self
            .routes
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != edge && !(shares(edge) && shares(*other)))
            .filter_map(|(_, route)| route.as_ref())
            .flat_map(|route| {
                route
                    .points
                    .windows(2)
                    .map(|pair| (self.page(pair[0]), self.page(pair[1])))
                    .filter(|(a, b)| (a.0 - b.0).abs() < 1e-6)
                    .map(|(a, b)| (a.0, a.1.min(b.1), a.1.max(b.1)))
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut output = Vec::with_capacity(points.len());
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            output.push(a);
            if (a.1 - b.1).abs() > 1e-6 || (b.0 - a.0).abs() < 2.0 * (HOP_REACH + HOP_CLEAR) {
                continue;
            }
            let sign = (b.0 - a.0).signum();
            let (low, high) = (a.0.min(b.0) + HOP_CLEAR, a.0.max(b.0) - HOP_CLEAR);
            let mut crossings: Vec<f64> = verticals
                .iter()
                .filter(|(x, from, to)| {
                    *x > low && *x < high && *from + 0.5 < a.1 && a.1 < *to - 0.5
                })
                .map(|(x, _, _)| *x)
                .collect();
            crossings.sort_by(f64::total_cmp);
            crossings.dedup_by(|p, q| (*p - *q).abs() < 1e-6);
            let single: Vec<f64> = crossings
                .iter()
                .enumerate()
                .filter(|(index, x)| {
                    let near = |other: Option<&f64>| {
                        other.is_some_and(|other| (**x - *other).abs() < 2.0 * HOP_REACH + 4.0)
                    };
                    !near(
                        index
                            .checked_sub(1)
                            .and_then(|before| crossings.get(before)),
                    ) && !near(crossings.get(index + 1))
                })
                .map(|(_, x)| *x)
                .collect();
            let mut hops = single;
            if sign < 0.0 {
                hops.reverse();
            }
            for x in hops {
                let (near, far) = (x - sign * HOP_REACH, x + sign * HOP_REACH);
                output.push((near, a.1));
                output.push((x - sign * HOP_REACH * 0.6, a.1 - HOP_HEIGHT));
                output.push((x + sign * HOP_REACH * 0.6, a.1 - HOP_HEIGHT));
                output.push((far, a.1));
            }
        }
        output.push(points[points.len() - 1]);
        output
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn draw_edge(
        &self,
        model: &Model,
        edge: usize,
        elements: &mut Vec<Element>,
        labels: &mut Vec<Label>,
    ) {
        let Some(route) = &self.routes[edge] else {
            return;
        };
        let diagram = model.diagram;
        let spec = &diagram.edges[edge];
        let (from, to) = model.ends[edge];
        let mut points: Vec<(f64, f64)> =
            route.points.iter().map(|point| self.page(*point)).collect();
        let tip = points[points.len() - 1];
        let before = points[points.len() - 2];
        let length = (tip.0 - before.0).hypot(tip.1 - before.1);
        let direction = if length > 0.0 {
            ((tip.0 - before.0) / length, (tip.1 - before.1) / length)
        } else {
            (0.0, 1.0)
        };
        let last = points.len() - 1;
        let (start, next) = (points[0], points[1]);
        let tail_direction = unit((start.0 - next.0, start.1 - next.1));
        let (head_length, tail_length) = (end_length(spec.ends.head), end_length(spec.ends.tail));
        points[last] = (
            tip.0 - direction.0 * head_length,
            tip.1 - direction.1 * head_length,
        );
        points[0] = (
            start.0 - tail_direction.0 * tail_length,
            start.1 - tail_direction.1 * tail_length,
        );
        // Edges that share the way into a step are drawn from there, so that a dashed pattern
        // runs in step on the shared part.
        if model.trunks.reach[edge].is_some() {
            points.reverse();
        }
        let main = spec.main;
        let class = match (main, spec.dash) {
            (true, _) => "chartlet-flow-edge chartlet-flow-main",
            (false, Some(Dash::Dashed)) => "chartlet-flow-edge chartlet-flow-dashed",
            (false, Some(Dash::Dotted)) => "chartlet-flow-edge chartlet-flow-dotted",
            (false, _) => "chartlet-flow-edge",
        };
        let note = match (&spec.label, &spec.technology) {
            (Some(label), Some(technology)) => Some(format!("{label} [{technology}]")),
            (Some(label), None) => Some(label.clone()),
            (None, Some(technology)) => Some(format!("[{technology}]")),
            (None, None) => None,
        };
        let (from_label, to_label) = match &diagram.whole {
            Some(whole) => {
                let (from, to) = whole.ends[edge];
                (&whole.labels[from], &whole.labels[to])
            }
            None => (&diagram.nodes[from].label, &diagram.nodes[to].label),
        };
        // The line to a note only points at it; the note carries its own text.
        let tooltip = match note {
            _ if diagram.nodes[to].shape == Shape::Note => None,
            // A stem belongs to its component; it is no connection.
            _ if diagram.nodes[from].shape == Shape::Interface => None,
            Some(note) => Some(format!("{from_label} {} {to_label}: {note}", spec.sign)),
            None => Some(format!("{from_label} {} {to_label}", spec.sign)),
        };
        let points = self.hop_over_crossings(model, edge, &points);
        elements.push(Element::Polyline(diagram::polyline(points, class, tooltip)));
        let head_class = if main {
            "chartlet-flow-head chartlet-flow-main-head"
        } else {
            "chartlet-flow-head"
        };
        push_ends(
            &spec.ends,
            ((tip, direction), (start, tail_direction)),
            head_class,
            elements,
        );
        let lines = &model.edge_labels[edge];
        if lines.is_empty() {
            return;
        }
        if let Some(label) = self.run_label(model, edge, route, lines) {
            labels.push(label);
            return;
        }
        let (x, y) = self.page(route.label_at);
        let last = count(lines.len() - 1);
        let reach = CHIP_REACH + 2.0 + route.clear;
        let side = |x: f64| {
            if route.after {
                (x + reach, TextAnchor::Start)
            } else {
                (x - reach, TextAnchor::End)
            }
        };
        // In landscape a label sits above its edge, or below it on the lower side of the step.
        let above = |y: f64| {
            if route.after {
                y + 18.0 + route.clear
            } else {
                y - 11.0 - LINE * last - route.clear
            }
        };
        let along = CHIP_REACH + 2.0;
        let (x, y, anchor) = match (self.landscape, route.against) {
            (true, false) => (x + along, above(y), TextAnchor::Start),
            (true, true) => (x - along, above(y), TextAnchor::End),
            (false, false) => {
                let (x, anchor) = side(x);
                (x, y + 19.0, anchor)
            }
            (false, true) => {
                let (x, anchor) = side(x);
                (x, y - 12.0 - LINE * last, anchor)
            }
        };
        let (line, _) = self.page(route.label_at);
        labels.push(Label {
            lines: lines.clone(),
            classes: model.edge_classes(edge),
            at: (x, y),
            anchor,
            beside: (!self.landscape).then_some(line),
            up: !self.landscape && route.against,
        });
    }

    /// Draws the edge labels over all edges, each moved along the flow until it no longer covers
    /// one drawn before it.
    pub(super) fn draw_labels(
        &self,
        model: &Model,
        labels: Vec<Label>,
        metrics: &impl TextMetrics,
        elements: &mut Vec<Element>,
    ) -> (f64, f64) {
        const AIR: f64 = 3.0;
        let mut reach = (0.0_f64, 0.0_f64);
        // Labels keep clear of the steps as well as of each other.
        let mut placed: Vec<(f64, f64, f64, f64)> = (0..model.diagram.nodes.len())
            .map(|node| self.node_box(model, node))
            .collect();
        // In portrait the borders of the frames are kept clear too, as thin boxes.
        let borders: Vec<(f64, f64, f64, f64)> = self
            .group_frames(model, metrics)
            .into_iter()
            .flat_map(|(left, top, right, bottom)| {
                if self.landscape {
                    [
                        (left, top, 0.0, bottom - top),
                        (right, top, 0.0, bottom - top),
                    ]
                } else {
                    [
                        (left, top, right - left, 0.0),
                        (left, bottom, right - left, 0.0),
                    ]
                }
            })
            .collect();
        for mut label in labels {
            label.keep_on_page(self.width, metrics);
            for _ in 0..12 {
                let (x, y, width, height) = chip_box(&label.lines, label.at, label.anchor, metrics);
                let hits = |other: &&(f64, f64, f64, f64)| {
                    x < other.0 + other.2 + AIR
                        && other.0 < x + width + AIR
                        && y < other.1 + other.3 + AIR
                        && other.1 < y + height + AIR
                };
                if let Some(border) = borders.iter().find(hits) {
                    label.at.1 += border.1 + border.3 + AIR - y;
                    continue;
                }
                let Some(other) = placed.iter().find(|other| {
                    x < other.0 + other.2 + AIR
                        && other.0 < x + width + AIR
                        && y < other.1 + other.3 + AIR
                        && other.1 < y + height + AIR
                }) else {
                    break;
                };
                if self.landscape {
                    label.at.0 += other.0 + other.2 + AIR - x;
                } else if label.up {
                    label.at.1 -= y + height + AIR - other.1;
                } else {
                    label.at.1 += other.1 + other.3 + AIR - y;
                }
            }
            let placed_at = chip_box(&label.lines, label.at, label.anchor, metrics);
            reach = (
                reach.0.max(placed_at.0 + placed_at.2),
                reach.1.max(placed_at.1 + placed_at.3),
            );
            placed.push(placed_at);
            elements.push(Element::Group(label.classes.clone()));
            chip(&label.lines, label.at, label.anchor, metrics, elements);
            elements.push(Element::GroupEnd);
        }
        reach
    }

    /// An edge from a step to itself: a loop beside the step, right in portrait and below in
    /// landscape, with the labels of all such edges joined.
    pub(super) fn draw_loop(
        &self,
        model: &Model,
        node: usize,
        elements: &mut Vec<Element>,
        labels: &mut Vec<Label>,
    ) {
        let Some(label) = &model.loops[node] else {
            return;
        };
        let (x, y, width, height) = self.node_box(model, node);
        let (points, tip, direction, text) = if self.landscape {
            let (middle, bottom) = (x + width / 2.0, y + height);
            (
                vec![
                    (middle - 8.0, bottom),
                    (middle - 8.0, bottom + LOOP),
                    (middle + 8.0, bottom + LOOP),
                    (middle + 8.0, bottom + HEAD),
                ],
                (middle + 8.0, bottom),
                (0.0, -1.0),
                (middle, bottom + LOOP + 18.0, TextAnchor::Middle),
            )
        } else {
            let (right, middle) = (x + width, y + height / 2.0);
            (
                vec![
                    (right, middle - 8.0),
                    (right + LOOP, middle - 8.0),
                    (right + LOOP, middle + 8.0),
                    (right + HEAD, middle + 8.0),
                ],
                (right, middle + 8.0),
                (-1.0, 0.0),
                (
                    right + LOOP + CHIP_REACH + 2.0,
                    middle + 4.0,
                    TextAnchor::Start,
                ),
            )
        };
        let name = &model.diagram.nodes[node].label;
        let tooltip = if label.is_empty() {
            format!("{name} → {name}")
        } else {
            format!("{name} → {name}: {label}")
        };
        elements.push(Element::Polyline(diagram::polyline(
            rounded(&points),
            "chartlet-flow-edge",
            Some(tooltip),
        )));
        elements.push(Element::Polyline(arrowhead(
            tip,
            direction,
            true,
            "chartlet-flow-head",
        )));
        let lines = &model.loop_lines[node];
        if !lines.is_empty() {
            // Beside the loop, the lines centred on it; below it, they run down.
            let rise = if self.landscape {
                0.0
            } else {
                CHIP_LINE * count(lines.len() - 1) / 2.0
            };
            labels.push(Label {
                lines: lines.clone(),
                classes: model.node_classes(node),
                at: (text.0, text.1 - rise),
                anchor: text.2,
                beside: None,
                up: false,
            });
        }
    }
}
