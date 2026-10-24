//! Networks: nodes as dots joined by edges, with no direction of flow to lay them out by. On a
//! circle, the nodes stand clockwise from twelve o'clock, groups together, in the order that lets
//! the fewest edges cross; edges bend a little towards the middle. In rings, one node stands in
//! the middle and every other on the ring of its distance from it in edges. Labels stand outside
//! their dots, stacked so that none covers another. The description names the structure of the
//! graph and every node with its neighbours.

use std::f64::consts::TAU;
use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::{self, HEAD, arrowhead, pixels, warn_growth_advising},
    error::ChartWarning,
    flow::{focus_classes, heaviest_edge},
    layout::{LABEL_SIZE, NARROW, count, fit_text, format_value, push_title, title_extra},
    metrics::TextMetrics,
    roles,
    scene::{Circle, Element, Hotspot, Scene, Text, TextAnchor},
    spec::{ChartSpec, DiagramRole, NetworkLayout, NetworkOrder, NetworkSpec},
    structure::{Facts, Structure},
    text,
};

const MARGIN: f64 = 24.0;
const COMPACT_MARGIN: f64 = 12.0;
const BOTTOM: f64 = 16.0;
const DOT: f64 = 6.0;
const CENTER_DOT: f64 = 9.0;
/// Space between a dot and its label, and the room a label takes along the side it is stacked
/// on.
const LABEL_GAP: f64 = 10.0;
const LABEL_LINE: f64 = 16.0;
const MAX_LABEL: f64 = 150.0;
const COMPACT_MAX_LABEL: f64 = 96.0;
/// The arc between two dots on a circle, at least.
const DOT_SPACING: f64 = 18.0;
/// How far an edge on a circle bends towards the middle, as a share of the way from its middle.
const BEND: f64 = 0.3;
const BEND_STEPS: usize = 12;
/// The least distance between two rings.
const RING_GAP: f64 = 56.0;
/// How often the order around a circle is swept for swaps that save crossings.
const SWEEPS: usize = 30;
const LEGEND_ROW: f64 = 22.0;

const NODE_CLASSES: [&str; 9] = [
    "chartlet-net-node",
    "chartlet-net-node chartlet-net-g1",
    "chartlet-net-node chartlet-net-g2",
    "chartlet-net-node chartlet-net-g3",
    "chartlet-net-node chartlet-net-g4",
    "chartlet-net-node chartlet-net-g5",
    "chartlet-net-node chartlet-net-g6",
    "chartlet-net-node chartlet-net-g7",
    "chartlet-net-node chartlet-net-g8",
];
const CENTER_CLASSES: [&str; 9] = [
    "chartlet-net-node chartlet-net-center",
    "chartlet-net-node chartlet-net-center chartlet-net-g1",
    "chartlet-net-node chartlet-net-center chartlet-net-g2",
    "chartlet-net-node chartlet-net-center chartlet-net-g3",
    "chartlet-net-node chartlet-net-center chartlet-net-g4",
    "chartlet-net-node chartlet-net-center chartlet-net-g5",
    "chartlet-net-node chartlet-net-center chartlet-net-g6",
    "chartlet-net-node chartlet-net-center chartlet-net-g7",
    "chartlet-net-node chartlet-net-center chartlet-net-g8",
];

fn network(spec: &ChartSpec) -> &NetworkSpec {
    spec.network
        .as_ref()
        .expect("validated networks carry a network block")
}

fn node_class(network: &NetworkSpec, node: usize, center: bool) -> &'static str {
    let index = network.group_of(node).map_or(0, |group| group + 1);
    if center {
        CENTER_CLASSES[index]
    } else {
        NODE_CLASSES[index]
    }
}

/// Where every node stands: an angle clockwise from twelve o'clock and a ring, the middle being
/// ring 0. On a circle every node lies on ring 1.
struct Placement {
    angle: Vec<f64>,
    ring: Vec<usize>,
    /// The nodes in reading order: clockwise, ring by ring from the middle.
    order: Vec<usize>,
    /// How many rings there are, the middle not counted.
    rings: usize,
}

/// How many steps every node is from `center` along edges in either direction; `None` when no
/// way leads there.
fn distances(nodes: usize, ends: &[(usize, usize)], center: usize) -> Vec<Option<usize>> {
    let mut distance = vec![None; nodes];
    distance[center] = Some(0);
    let mut frontier = vec![center];
    let mut step = 0;
    while !frontier.is_empty() {
        step += 1;
        let mut next = Vec::new();
        for &(a, b) in ends {
            for (from, to) in [(a, b), (b, a)] {
                if frontier.contains(&from) && distance[to].is_none() {
                    distance[to] = Some(step);
                    next.push(to);
                }
            }
        }
        next.sort_unstable();
        frontier = next;
    }
    distance
}

impl Placement {
    fn new(network: &NetworkSpec) -> Self {
        let ends = network.ends();
        if let (NetworkLayout::Rings, Some(center)) = (network.layout, &network.center) {
            return Self::rings(network, &ends, network.node(center).expect("validated"));
        }
        let order = circle_order(network, &ends);
        let mut angle = vec![0.0; order.len()];
        for (position, &node) in order.iter().enumerate() {
            angle[node] = TAU * count(position) / count(order.len());
        }
        Self {
            angle,
            ring: vec![1; order.len()],
            order,
            rings: 1,
        }
    }

    /// The center in the middle and every other node on the ring of its distance; nodes no way
    /// reaches on one more ring outside. Ring 1 follows the specification; every further ring
    /// is sorted by the middle angle of each node's neighbours on the ring before, and turned so
    /// that its nodes stand as close to that angle as they can.
    fn rings(network: &NetworkSpec, ends: &[(usize, usize)], center: usize) -> Self {
        let nodes = network.nodes.len();
        let distance = distances(nodes, ends, center);
        let deepest = distance.iter().flatten().copied().max().unwrap_or(0);
        let unreached = distance.iter().any(Option::is_none);
        let ring: Vec<usize> = distance
            .iter()
            .map(|distance| distance.unwrap_or(deepest + 1))
            .collect();
        let rings = deepest + usize::from(unreached);
        let mut angle = vec![0.0_f64; nodes];
        let mut order = vec![center];
        for level in 1..=rings {
            let mut members: Vec<(f64, usize)> = (0..nodes)
                .filter(|&node| ring[node] == level)
                .map(|node| {
                    let (x, y) = ends
                        .iter()
                        .filter_map(|&(a, b)| match (a == node, b == node) {
                            (true, _) => Some(b),
                            (_, true) => Some(a),
                            _ => None,
                        })
                        .filter(|&other| ring[other] + 1 == level && level > 1)
                        .fold((0.0_f64, 0.0_f64), |(x, y), other: usize| {
                            (x + angle[other].sin(), y + angle[other].cos())
                        });
                    let wanted = if x == 0.0 && y == 0.0 {
                        f64::INFINITY
                    } else {
                        x.atan2(y).rem_euclid(TAU)
                    };
                    (wanted, node)
                })
                .collect();
            // Ring 1 and nodes with nothing on the ring before keep the specification's order.
            if level > 1 {
                members.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            }
            let size = count(members.len());
            let even = |position: usize| TAU * count(position) / size;
            let (sin, cos) = members
                .iter()
                .enumerate()
                .filter(|(_, (wanted, _))| wanted.is_finite())
                .fold((0.0, 0.0), |(sin, cos), (position, (wanted, _))| {
                    let off = wanted - even(position);
                    (sin + off.sin(), cos + off.cos())
                });
            let turn = if sin == 0.0 && cos == 0.0 {
                0.0
            } else {
                sin.atan2(cos)
            };
            for (position, (_, node)) in members.iter().enumerate() {
                angle[*node] = (even(position) + turn).rem_euclid(TAU);
            }
            order.extend(members.iter().map(|(_, node)| *node));
        }
        Self {
            angle,
            ring,
            order,
            rings,
        }
    }
}

/// The order of the nodes around a circle: as listed, or groups together in the order they
/// first appear, with neighbours within a group swapped as long as that saves crossings.
fn circle_order(network: &NetworkSpec, ends: &[(usize, usize)]) -> Vec<usize> {
    let nodes = network.nodes.len();
    if network.order == NetworkOrder::Listed {
        return (0..nodes).collect();
    }
    let groups = network.groups().len();
    let mut order: Vec<usize> = (0..nodes).collect();
    order.sort_by_key(|&node| (network.group_of(node).unwrap_or(groups), node));
    let mut position = vec![0; nodes];
    let place = |order: &[usize], position: &mut [usize]| {
        for (at, &node) in order.iter().enumerate() {
            position[node] = at;
        }
    };
    place(&order, &mut position);
    let mut crossings = count_crossings(ends, &position);
    for _ in 0..SWEEPS {
        let mut better = false;
        for at in 0..nodes {
            let next = (at + 1) % nodes;
            if next == at
                || network.group_of(order[at]) != network.group_of(order[next])
                || crossings == 0
            {
                continue;
            }
            order.swap(at, next);
            place(&order, &mut position);
            let after = count_crossings(ends, &position);
            if after < crossings {
                crossings = after;
                better = true;
            } else {
                order.swap(at, next);
                place(&order, &mut position);
            }
        }
        if !better {
            break;
        }
    }
    order
}

/// How many pairs of edges cross as chords of a circle whose nodes stand at `position`.
fn count_crossings(ends: &[(usize, usize)], position: &[usize]) -> usize {
    let nodes = position.len();
    let mut crossings = 0;
    for (index, &(a, b)) in ends.iter().enumerate() {
        let (start, span) = (position[a], (position[b] + nodes - position[a]) % nodes);
        let inside = |node: usize| {
            let off = (position[node] + nodes - start) % nodes;
            off > 0 && off < span
        };
        for &(c, d) in &ends[index + 1..] {
            if [c, d].iter().any(|end| *end == a || *end == b) {
                continue;
            }
            if inside(c) != inside(d) {
                crossings += 1;
            }
        }
    }
    crossings
}

/// The geometry of the drawing: the middle, the radius of every ring, and where every label
/// stands.
struct Plan {
    center: (f64, f64),
    radius: Vec<f64>,
    width: f64,
    height: f64,
    labels: Vec<(f64, f64, TextAnchor)>,
    legend_top: f64,
}

impl Plan {
    fn point(&self, placement: &Placement, node: usize) -> (f64, f64) {
        let radius = self.radius[placement.ring[node]];
        let angle = placement.angle[node];
        (
            self.center.0 + radius * angle.sin(),
            self.center.1 - radius * angle.cos(),
        )
    }
}

/// The legend entries in rows that fit `room`: each entry's group, its x offset and its row.
fn legend_rows(
    groups: &[&str],
    room: f64,
    metrics: &impl TextMetrics,
) -> (Vec<(usize, f64, usize)>, usize) {
    let mut entries = Vec::new();
    let (mut x, mut row) = (0.0, 0);
    for (group, name) in groups.iter().enumerate() {
        let width = 18.0 + metrics.width(name, LABEL_SIZE) + 20.0;
        if x > 0.0 && x + width > room {
            x = 0.0;
            row += 1;
        }
        entries.push((group, x, row));
        x += width;
    }
    let rows = if groups.is_empty() { 0 } else { row + 1 };
    (entries, rows)
}

#[allow(clippy::too_many_lines)]
fn plan(
    spec: &ChartSpec,
    placement: &Placement,
    label_widths: &[f64],
    (top, margin, legend_height): (f64, f64, f64),
) -> Plan {
    let network = network(spec);
    let nodes = network.nodes.len();
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let rings = placement.rings.max(1);
    let center_ring = network.layout == NetworkLayout::Rings;
    // The labels of a ring must fit between it and the next one outwards.
    let widest_on = |ring: usize| {
        (0..nodes)
            .filter(|&node| placement.ring[node] == ring)
            .map(|node| label_widths[node])
            .fold(0.0, f64::max)
    };
    // Every ring wide enough for its dots to keep apart, and rings far enough apart for a line
    // of text between them.
    let ring_least = (1..=rings)
        .map(|ring| {
            let on = (0..nodes)
                .filter(|&node| placement.ring[node] == ring)
                .count();
            DOT_SPACING * count(on) / TAU / count(ring)
        })
        .fold(0.0, f64::max);
    let step_least = if center_ring {
        ring_least.max(RING_GAP)
    } else {
        ring_least.max(DOT_SPACING)
    };
    let outer_widest = widest_on(rings);
    let room_x = (width - 2.0 * margin) / 2.0 - LABEL_GAP - outer_widest - DOT;
    let room_y = (height - top - BOTTOM - legend_height) / 2.0 - LABEL_LINE - LABEL_GAP;
    let step = (room_x.min(room_y) / count(rings)).max(step_least);
    let radius: Vec<f64> = (0..=rings).map(|ring| step * count(ring)).collect();
    let outer = radius[rings];
    let page_width = width.max(2.0 * (margin + outer + LABEL_GAP + DOT + outer_widest));
    let page_height =
        height.max(top + 2.0 * (outer + LABEL_GAP + LABEL_LINE) + BOTTOM + legend_height);
    let center = (
        page_width / 2.0,
        top + (page_height - top - BOTTOM - legend_height) / 2.0,
    );
    let mut plan = Plan {
        center,
        radius,
        width: page_width,
        height: page_height,
        labels: vec![(0.0, 0.0, TextAnchor::Start); nodes],
        legend_top: page_height - BOTTOM - legend_height + 6.0,
    };
    let bounds = (
        margin,
        top,
        page_width - margin,
        page_height - BOTTOM - legend_height,
    );
    place_labels(&mut plan, placement, label_widths, bounds);
    plan
}

/// The radius of a node's dot: larger for the middle of the rings.
fn dot_radius(placement: &Placement, node: usize) -> f64 {
    if placement.ring[node] == 0 {
        CENTER_DOT
    } else {
        DOT
    }
}

/// Whether two boxes `(left, top, right, bottom)` overlap.
fn overlap(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
}

/// Every label in reading order at the first of its places that covers no dot and no label
/// placed before, and lies inside `bounds`: beside its dot on the outer side, then outwards above
/// or below it, then on the other side of the line, then beside it on the inner side. When none
/// is free, the first.
fn place_labels(
    plan: &mut Plan,
    placement: &Placement,
    widths: &[f64],
    bounds: (f64, f64, f64, f64),
) {
    let dots: Vec<(f64, f64, f64, f64)> = (0..widths.len())
        .map(|node| {
            let (x, y) = plan.point(placement, node);
            let reach = dot_radius(placement, node) + 2.0;
            (x - reach, y - reach, x + reach, y + reach)
        })
        .collect();
    let mut taken: Vec<(f64, f64, f64, f64)> = Vec::new();
    for &node in &placement.order {
        let (x, y) = plan.point(placement, node);
        let reach = dot_radius(placement, node) + LABEL_GAP / 2.0;
        let width = widths[node];
        let angle = placement.angle[node];
        let right = placement.ring[node] == 0 || angle.sin() >= -1e-9;
        let upper = placement.ring[node] != 0 && angle.cos() > 0.0;
        let side = |right: bool| {
            if right {
                (x + reach, y + 4.0, TextAnchor::Start)
            } else {
                (x - reach, y + 4.0, TextAnchor::End)
            }
        };
        let vertical = |above: bool| {
            if above {
                (x, y - reach - 3.0, TextAnchor::Middle)
            } else {
                (x, y + reach + 11.0, TextAnchor::Middle)
            }
        };
        let candidates = [side(right), vertical(upper), vertical(!upper), side(!right)];
        let text_box = |(x, y, anchor): (f64, f64, TextAnchor)| {
            let left = match anchor {
                TextAnchor::Start => x,
                TextAnchor::End => x - width,
                TextAnchor::Middle => x - width / 2.0,
            };
            (left - 1.0, y - 11.0, left + width + 1.0, y + 3.0)
        };
        let free = |candidate: (f64, f64, TextAnchor)| {
            let area = text_box(candidate);
            area.0 >= bounds.0
                && area.2 <= bounds.2
                && area.1 >= bounds.1
                && area.3 <= bounds.3
                && dots
                    .iter()
                    .enumerate()
                    .all(|(other, dot)| other == node || !overlap(area, *dot))
                && taken.iter().all(|label| !overlap(area, *label))
        };
        let chosen = candidates
            .iter()
            .copied()
            .find(|candidate| free(*candidate))
            .unwrap_or(candidates[0]);
        taken.push(text_box(chosen));
        plan.labels[node] = chosen;
    }
}

/// The points of an edge between two dots: on a circle a curve bent towards the middle, in rings
/// a straight line; both end at the edges of the dots.
fn edge_points(
    plan: &Plan,
    (from, to): ((f64, f64), (f64, f64)),
    (from_dot, to_dot): (f64, f64),
    bend: bool,
) -> Vec<(f64, f64)> {
    let mut points = if bend {
        let middle = (f64::midpoint(from.0, to.0), f64::midpoint(from.1, to.1));
        let control = (
            middle.0 + (plan.center.0 - middle.0) * BEND,
            middle.1 + (plan.center.1 - middle.1) * BEND,
        );
        (0..=BEND_STEPS)
            .map(|step| {
                let t = count(step) / count(BEND_STEPS);
                let u = 1.0 - t;
                (
                    u * u * from.0 + 2.0 * u * t * control.0 + t * t * to.0,
                    u * u * from.1 + 2.0 * u * t * control.1 + t * t * to.1,
                )
            })
            .collect()
    } else {
        vec![from, to]
    };
    // Cut both ends back to the edge of their dots.
    for (end, inner, reach) in [
        (0, 1, from_dot),
        (points.len() - 1, points.len() - 2, to_dot),
    ] {
        let (dx, dy) = (
            points[inner].0 - points[end].0,
            points[inner].1 - points[end].1,
        );
        let length = dx.hypot(dy);
        if length > reach {
            points[end] = (
                points[end].0 + dx / length * reach,
                points[end].1 + dy / length * reach,
            );
        }
    }
    points
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let network = network(spec);
    let compact = spec.width < NARROW;
    let margin = if compact { COMPACT_MARGIN } else { MARGIN };
    let width = f64::from(spec.width);
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let max_label = if compact {
        COMPACT_MAX_LABEL
    } else {
        MAX_LABEL
    };
    let labels: Vec<String> = network
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            fit_text(
                &node.label,
                max_label,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/network/nodes/{index}/label"),
            )
        })
        .collect();
    let label_widths: Vec<f64> = labels
        .iter()
        .map(|label| metrics.width(label, LABEL_SIZE))
        .collect();
    let groups = network.groups();
    let (legend, legend_lines) = legend_rows(&groups, width - 2.0 * margin, metrics);
    let legend_height = LEGEND_ROW * count(legend_lines);
    let placement = Placement::new(network);
    let plan = plan(
        spec,
        &placement,
        &label_widths,
        (top, margin, legend_height),
    );
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        plan.width - 2.0 * margin,
        metrics,
        warnings,
    );
    warn_growth_advising(
        spec,
        (plan.width, plan.height),
        "shorten labels or use fewer nodes",
        warnings,
    );
    let rings = network.layout == NetworkLayout::Rings;
    if rings {
        for ring in 1..=placement.rings {
            elements.push(Element::Circle(Circle {
                cx: plan.center.0,
                cy: plan.center.1,
                radius: plan.radius[ring],
                class: "chartlet-net-ring",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
    }
    let center = network
        .center
        .as_deref()
        .and_then(|id| network.node(id))
        .filter(|_| rings);
    let dot = |node: usize| {
        if Some(node) == center {
            CENTER_DOT
        } else {
            DOT
        }
    };
    // An edge ends outside the ring of a node's role.
    let reach = |node: usize| {
        dot(node)
            + if network.nodes[node].role.is_some() {
                roles::RING_GAP + 2.5
            } else {
                1.0
            }
    };
    let ends = network.ends();
    let widths = crate::flow::weight_widths_of(network.edges.iter().map(|edge| edge.weight));
    for (index, &(from, to)) in ends.iter().enumerate() {
        let edge = &network.edges[index];
        let mut classes = focus_classes(None, &[from.min(to), from.max(to)]);
        if let Some(width) = widths[index] {
            write!(classes, " chartlet-flow-w{width}").expect("writing to String cannot fail");
        }
        if let Some(role) = edge.role {
            classes.push_str(roles::class(role));
        }
        elements.push(Element::Group(classes));
        let head = if network.directed { HEAD } else { 0.0 };
        let mut points = edge_points(
            &plan,
            (plan.point(&placement, from), plan.point(&placement, to)),
            (reach(from), reach(to) + head),
            !rings,
        );
        let joint = if network.directed { "→" } else { "–" };
        let mut tooltip = format!(
            "{} {joint} {}",
            network.nodes[from].label, network.nodes[to].label
        );
        if let Some(weight) = edge.weight {
            write!(
                tooltip,
                ": {}",
                text::edge_weight(spec.locale, &format_value(weight, spec.number_style()))
            )
            .expect("writing to String cannot fail");
        }
        let class = match edge.dash {
            Some(crate::spec::Dash::Dashed) => "chartlet-flow-edge chartlet-flow-dashed",
            Some(crate::spec::Dash::Dotted) => "chartlet-flow-edge chartlet-flow-dotted",
            _ => "chartlet-flow-edge",
        };
        if network.directed {
            let last = points[points.len() - 1];
            let before = points[points.len() - 2];
            let length = (last.0 - before.0).hypot(last.1 - before.1).max(1e-9);
            let direction = ((last.0 - before.0) / length, (last.1 - before.1) / length);
            let tip = (last.0 + direction.0 * HEAD, last.1 + direction.1 * HEAD);
            elements.push(Element::Polyline(diagram::polyline(
                std::mem::take(&mut points),
                class,
                Some(tooltip),
            )));
            elements.push(Element::Polyline(arrowhead(
                tip,
                direction,
                true,
                "chartlet-flow-head",
            )));
        } else {
            elements.push(Element::Polyline(diagram::polyline(
                points,
                class,
                Some(tooltip),
            )));
        }
        elements.push(Element::GroupEnd);
    }
    for &node in &placement.order {
        let related: Vec<usize> = {
            let mut related: Vec<usize> = ends
                .iter()
                .filter_map(|&(a, b)| match (a == node, b == node) {
                    (true, _) => Some(b),
                    (_, true) => Some(a),
                    _ => None,
                })
                .chain(std::iter::once(node))
                .collect();
            related.sort_unstable();
            related.dedup();
            related
        };
        let spec_node = &network.nodes[node];
        let mut classes = focus_classes(Some(node), &related);
        if let Some(role) = spec_node.role {
            classes.push_str(roles::class(role));
        }
        elements.push(Element::NodeGroup(classes, spec_node.id.clone()));
        let (x, y) = plan.point(&placement, node);
        let tooltip = match &spec_node.group {
            Some(group) => format!("{} ({group})", spec_node.label),
            None => spec_node.label.clone(),
        };
        elements.push(Element::Circle(Circle {
            cx: x,
            cy: y,
            radius: dot(node),
            class: node_class(network, node, Some(node) == center),
            series_index: None,
            style_index: None,
            tooltip: Some(tooltip),
        }));
        if spec_node.role.is_some() {
            roles::push_ring((x, y), dot(node) + roles::RING_GAP, &mut elements);
        }
        let (label_x, label_y, anchor) = plan.labels[node];
        elements.push(Element::Text(Text {
            x: label_x,
            y: label_y,
            class: "chartlet-net-label",
            anchor,
            content: labels[node].clone(),
        }));
        elements.push(Element::GroupEnd);
        let label_left = match anchor {
            TextAnchor::Start => label_x,
            TextAnchor::End => label_x - label_widths[node],
            TextAnchor::Middle => label_x - label_widths[node] / 2.0,
        };
        let left = (x - dot(node)).min(label_left) - 2.0;
        let right = (x + dot(node)).max(label_left + label_widths[node]) + 2.0;
        let upper = (y - dot(node)).min(label_y - 13.0);
        let lower = (y + dot(node)).max(label_y + 4.0);
        elements.push(Element::Hotspot(Hotspot {
            node,
            label: spec_node.label.clone(),
            x: left,
            y: upper,
            width: right - left,
            height: lower - upper,
        }));
    }
    for (group, offset, row) in legend {
        let y = plan.legend_top + LEGEND_ROW * count(row);
        elements.push(Element::Circle(Circle {
            cx: margin + offset + 6.0,
            cy: y - 4.0,
            radius: DOT,
            class: NODE_CLASSES[group + 1],
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: margin + offset + 18.0,
            y,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: groups[group].to_owned(),
        }));
    }
    let mut scene = Scene {
        width: pixels(plan.width),
        height: pixels(plan.height),
        elements,
    };
    let used = roles::used(
        network.nodes.iter().map(|node| node.role),
        network.edges.iter().map(|edge| edge.role),
    );
    roles::push_legend(
        &mut scene,
        spec.locale,
        &used,
        (margin, ring_sample),
        metrics,
    );
    scene
}

/// The ring of a role as the legend shows it, as wide as a badge.
fn ring_sample(_: DiagramRole, at: (f64, f64), elements: &mut Vec<Element>) {
    roles::push_ring(at, diagram::BADGE - 1.0, elements);
}

/// Where a node leads, or whom it is connected with: labels in the order of the edges, each with
/// its weight and, `with_roles`, the role of the edge.
fn neighbours(spec: &ChartSpec, node: usize, with_roles: bool) -> Vec<String> {
    let network = network(spec);
    network
        .ends()
        .iter()
        .zip(&network.edges)
        .filter_map(|(&(from, to), edge)| {
            let other = match (from == node, to == node) {
                (true, _) => to,
                (_, true) if !network.directed => from,
                _ => return None,
            };
            let label = &network.nodes[other].label;
            let notes: Vec<String> = edge
                .weight
                .map(|weight| {
                    text::edge_weight(spec.locale, &format_value(weight, spec.number_style()))
                })
                .into_iter()
                .chain(
                    edge.role
                        .filter(|_| with_roles)
                        .map(|role| text::role(spec.locale, role).to_owned()),
                )
                .collect();
            Some(if notes.is_empty() {
                label.clone()
            } else {
                format!("{label} ({})", notes.join(", "))
            })
        })
        .collect()
}

/// The network in sentences: its size, its groups or its rings, the structure of the graph, the
/// heaviest edge, and every node in reading order with its neighbours.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let network = network(spec);
    let locale = spec.locale;
    let placement = Placement::new(network);
    let groups: Vec<String> = network
        .groups()
        .iter()
        .map(|&group| group.to_owned())
        .collect();
    let mut description =
        text::network_opening(locale, (network.nodes.len(), network.edges.len()), &groups);
    if let (NetworkLayout::Rings, Some(center)) = (network.layout, network.center.as_deref()) {
        let center = network.node(center).expect("validated");
        let reached = distances(network.nodes.len(), &network.ends(), center);
        let rings: Vec<(Option<usize>, Vec<String>)> = (1..=placement.rings)
            .map(|ring| {
                let members: Vec<String> = placement
                    .order
                    .iter()
                    .filter(|&&node| placement.ring[node] == ring)
                    .map(|&node| network.nodes[node].label.clone())
                    .collect();
                let unreached = placement
                    .order
                    .iter()
                    .any(|&node| placement.ring[node] == ring && reached[node].is_none());
                ((!unreached).then_some(ring), members)
            })
            .collect();
        description.push(' ');
        description.push_str(&text::network_rings(
            locale,
            &network.nodes[center].label,
            &rings,
        ));
    }
    let ends = network.ends();
    let structure = Structure::of(
        network.nodes.len(),
        &ends,
        &[],
        Facts {
            directed: network.directed,
        },
    )
    .sentences(locale, |node| network.nodes[node].label.clone());
    if !structure.is_empty() {
        description.push(' ');
        description.push_str(&structure);
    }
    let weights: Vec<Option<f64>> = network.edges.iter().map(|edge| edge.weight).collect();
    if let Some(heaviest) = heaviest_edge(spec, &weights, |edge| {
        let (from, to) = ends[edge];
        format!(
            "{} → {}",
            network.nodes[from].label, network.nodes[to].label
        )
    }) {
        description.push(' ');
        description.push_str(&heaviest);
    }
    for &node in &placement.order {
        let spec_node = &network.nodes[node];
        write!(
            description,
            " {}",
            text::network_node(
                locale,
                &spec_node.label,
                spec_node.group.as_deref(),
                network.directed,
                &neighbours(spec, node, false),
            )
        )
        .expect("writing to String cannot fail");
    }
    if let Some(sentence) = roles::sentence(locale, &with_roles(network, &placement)) {
        description.push(' ');
        description.push_str(&sentence);
    }
    description
}

/// Every node with a role, in reading order, then every edge with one, as `From → To` or, in a
/// network without direction, `From – To`.
fn with_roles(network: &NetworkSpec, placement: &Placement) -> Vec<(DiagramRole, String)> {
    let joint = if network.directed { "→" } else { "–" };
    placement
        .order
        .iter()
        .filter_map(|&node| {
            let spec_node = &network.nodes[node];
            spec_node.role.map(|role| (role, spec_node.label.clone()))
        })
        .chain(
            network
                .ends()
                .into_iter()
                .zip(&network.edges)
                .filter_map(|((from, to), edge)| {
                    edge.role.map(|role| {
                        (
                            role,
                            format!(
                                "{} {joint} {}",
                                network.nodes[from].label, network.nodes[to].label
                            ),
                        )
                    })
                }),
        )
        .collect()
}

/// One row per node in reading order: its label, group and ring where the network has them, and
/// its neighbours.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let network = network(spec);
    let words = spec.locale.words();
    let placement = Placement::new(network);
    let groups = !network.groups().is_empty();
    let rings = network.layout == NetworkLayout::Rings;
    let mut columns = vec![words.node.to_owned()];
    if groups {
        columns.push(words.group.to_owned());
    }
    if rings {
        columns.push(words.ring.to_owned());
    }
    let with_roles = network.nodes.iter().any(|node| node.role.is_some());
    if with_roles {
        columns.push(words.role.to_owned());
    }
    columns.push(if network.directed {
        words.leads_to.to_owned()
    } else {
        words.connects_to.to_owned()
    });
    let reached = network.center.as_deref().filter(|_| rings).map(|center| {
        distances(
            network.nodes.len(),
            &network.ends(),
            network.node(center).expect("validated"),
        )
    });
    let rows = placement
        .order
        .iter()
        .map(|&node| {
            let spec_node = &network.nodes[node];
            let mut row = vec![spec_node.label.clone()];
            if groups {
                row.push(spec_node.group.clone().unwrap_or_default());
            }
            if let Some(reached) = &reached {
                row.push(text::ring_name(spec.locale, reached[node]));
            }
            if with_roles {
                row.push(
                    spec_node
                        .role
                        .map_or_else(String::new, |role| text::role(spec.locale, role).to_owned()),
                );
            }
            row.push(neighbours(spec, node, true).join(", "));
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
    use super::{Placement, count_crossings};
    use crate::{ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative};

    const SPEC: &str = r#"{
        "schemaVersion": 1,
        "type": "network",
        "title": "Who works with whom",
        "network": {
            "nodes": [
                {"id": "ana", "label": "Ana", "group": "Design"},
                {"id": "ben", "label": "Ben", "group": "Code"},
                {"id": "cem", "label": "Cem", "group": "Design"},
                {"id": "dora", "label": "Dora", "group": "Code"},
                {"id": "eli", "label": "Eli"}
            ],
            "edges": [
                {"from": "ana", "to": "dora"},
                {"from": "ben", "to": "cem"},
                {"from": "ana", "to": "cem"},
                {"from": "ben", "to": "dora"},
                {"from": "dora", "to": "eli"}
            ]
        }
    }"#;

    fn error(json: &str) -> (&'static str, String) {
        let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("the test specification is invalid");
        (error.code, error.path)
    }

    fn placement(json: &str) -> Placement {
        let spec = ChartSpec::from_json(json).expect("parses");
        Placement::new(spec.network.as_ref().expect("a network"))
    }

    #[test]
    fn groups_stand_together_and_swaps_save_crossings() {
        let placement = placement(SPEC);
        // Design (Ana, Cem), then Code (Ben, Dora), then Eli without a group.
        assert_eq!(placement.order.len(), 5);
        let spec = ChartSpec::from_json(SPEC).expect("parses");
        let network = spec.network.as_ref().expect("a network");
        let groups: Vec<Option<usize>> = placement
            .order
            .iter()
            .map(|&node| network.group_of(node))
            .collect();
        assert_eq!(groups, [Some(0), Some(0), Some(1), Some(1), None]);
        let mut position = vec![0; 5];
        for (at, &node) in placement.order.iter().enumerate() {
            position[node] = at;
        }
        let listed: Vec<usize> = (0..5).collect();
        assert!(
            count_crossings(&network.ends(), &position)
                <= count_crossings(&network.ends(), &listed)
        );
    }

    #[test]
    fn rings_follow_the_distance_from_the_center() {
        let json = SPEC.replace(
            "\"nodes\": [",
            "\"layout\": \"rings\", \"center\": \"ana\", \"nodes\": [",
        );
        let placement = placement(&json);
        // Ana in the middle; Cem and Dora one step away; Ben and Eli two.
        assert_eq!(placement.ring, [0, 2, 1, 1, 2]);
        assert_eq!(placement.order[0], 0);
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(
            alternative
                .description
                .contains("In rings around Ana: 1 step away: Cem and Dora; 2 steps away:"),
            "{}",
            alternative.description
        );
        assert_eq!(
            alternative.table.columns,
            ["Node", "Group", "Ring", "Connects to"]
        );
        assert_eq!(alternative.table.rows[0][2], "Center");
    }

    #[test]
    fn a_node_no_way_reaches_stands_on_a_ring_outside() {
        let json = SPEC
            .replace(
                "\"nodes\": [",
                "\"layout\": \"rings\", \"center\": \"ana\", \"nodes\": [",
            )
            .replace(
                r#"{"from": "dora", "to": "eli"}"#,
                r#"{"from": "ben", "to": "cem"}"#,
            );
        let placement = placement(&json);
        assert_eq!(placement.ring[4], 3);
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(alternative.description.contains("not connected: Eli"));
    }

    #[test]
    fn the_description_names_every_node_with_its_neighbours() {
        let alternative =
            text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
        assert!(
            alternative.description.starts_with(
                "Network with 5 nodes and 5 connections in 2 groups: Design and Code."
            ),
            "{}",
            alternative.description
        );
        assert!(
            alternative
                .description
                .contains("Dora (group Code): connected to Ana, Ben and Eli.")
        );
        let directed = SPEC.replace("\"nodes\": [", "\"directed\": true, \"nodes\": [");
        let alternative =
            text_alternative(&ChartSpec::from_json(&directed).expect("parses")).expect("valid");
        assert!(
            alternative
                .description
                .contains("Dora (group Code): leads to Eli."),
            "{}",
            alternative.description
        );
        assert!(
            alternative
                .description
                .contains("Eli: no outgoing connection.")
        );
    }

    #[test]
    fn a_network_renders_the_same_twice_with_focus_and_arrowheads() {
        let directed = SPEC.replace("\"nodes\": [", "\"directed\": true, \"nodes\": [");
        let html =
            render_json(&directed, RenderFormat::Html, &RenderOptions::default()).expect("renders");
        assert!(html.warnings.is_empty(), "{:?}", html.warnings);
        assert_eq!(
            html.content
                .matches("class=\"chartlet-focus-node\"")
                .count(),
            5
        );
        assert_eq!(html.content.matches("chartlet-flow-head").count(), 5 + 1);
        let again =
            render_json(&directed, RenderFormat::Html, &RenderOptions::default()).expect("renders");
        assert_eq!(html.content, again.content);
    }

    #[test]
    fn invalid_networks_name_the_field() {
        let rings = SPEC.replace("\"nodes\": [", "\"layout\": \"rings\", \"nodes\": [");
        for (json, code, path) in [
            (rings.clone(), "missing_center", "/network/center"),
            (
                rings.replace("\"layout\"", "\"center\": \"zoe\", \"layout\""),
                "unknown_node",
                "/network/center",
            ),
            (
                rings.replace(
                    "\"layout\"",
                    "\"center\": \"ana\", \"order\": \"listed\", \"layout\"",
                ),
                "option_not_supported",
                "/network/order",
            ),
            (
                SPEC.replace("\"nodes\": [", "\"center\": \"ana\", \"nodes\": ["),
                "option_not_supported",
                "/network/center",
            ),
            (
                SPEC.replace(r#""to": "eli""#, r#""to": "dora""#),
                "self_loop",
                "/network/edges/4/to",
            ),
            (
                SPEC.replace(r#""to": "eli""#, r#""to": "zoe""#),
                "unknown_node",
                "/network/edges/4/to",
            ),
            (
                SPEC.replace(r#""id": "cem""#, r#""id": "ana""#),
                "duplicate_id",
                "/network/nodes/2/id",
            ),
            (
                SPEC.replace(r#""to": "eli"}"#, r#""to": "eli", "weight": -2}"#),
                "invalid_edge_weight",
                "/network/edges/4/weight",
            ),
            (
                SPEC.replace("\"type\": \"network\"", "\"type\": \"flow\""),
                "option_not_supported",
                "/network",
            ),
        ] {
            assert_eq!(error(&json), (code, path.to_owned()), "{json}");
        }
    }

    #[test]
    fn nodes_and_edges_take_roles_as_rings_and_lines_with_a_legend() {
        let json = SPEC
            .replace(
                r#"{"id": "eli", "label": "Eli"}"#,
                r#"{"id": "eli", "label": "Eli", "role": "planned"}"#,
            )
            .replace(
                r#"{"from": "ana", "to": "dora"}"#,
                r#"{"from": "ana", "to": "dora", "role": "critical", "weight": 4}"#,
            );
        let output =
            render_json(&json, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        // The ring around Eli's dot, and one in the legend.
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-flow-node chartlet-net-role\"")
                .count(),
            2
        );
        assert!(
            output
                .content
                .contains(" chartlet-flow-w8 chartlet-flow-role-critical")
        );
        assert!(output.content.contains(">Planned</text>"));
        assert!(output.content.contains(">Critical</text>"));
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(
            alternative
                .description
                .ends_with("Roles: critical: Ana – Dora; planned: Eli."),
            "{}",
            alternative.description
        );
        assert_eq!(
            alternative.table.columns,
            ["Node", "Group", "Role", "Connects to"]
        );
        let ana = alternative
            .table
            .rows
            .iter()
            .find(|row| row[0] == "Ana")
            .expect("a row");
        assert_eq!(ana[2], "");
        assert!(ana[3].contains("Dora (weight 4, critical)"), "{ana:?}");
        let german = json.replace(
            "\"schemaVersion\": 1,",
            "\"schemaVersion\": 1, \"locale\": \"de\",",
        );
        let alternative =
            text_alternative(&ChartSpec::from_json(&german).expect("parses")).expect("valid");
        assert!(
            alternative
                .description
                .ends_with("Rollen: kritisch: Ana – Dora; geplant: Eli."),
            "{}",
            alternative.description
        );
        let dotted = SPEC.replace(
            r#"{"from": "ben", "to": "cem"}"#,
            r#"{"from": "ben", "to": "cem", "role": "deprecated", "dash": "dotted"}"#,
        );
        assert_eq!(
            error(&dotted),
            ("role_conflict", "/network/edges/1/dash".to_owned())
        );
    }
}
