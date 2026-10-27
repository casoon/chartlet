//! Drawing a planned diagram: lanes and frames, every edge and step in a group of its own, and
//! the classes that tie a drawn part to the steps whose focus brings it forward.

use std::fmt::Write as _;

use super::{
    BRANCH, GROUP_HEAD, GROUP_PAD, LANE_HEAD, SUBLABEL_SIZE, WEIGHT_STEPS, blocks,
    measure::Model,
    model::{Diagram, Shape},
    place::Plan,
    route::Route,
    trunks,
};
use crate::{
    error::ChartWarning,
    heat,
    metrics::TextMetrics,
    roles,
    scene::{Circle, Element, Hook, Hotspot, Rect, Text, TextAnchor},
};

impl Model<'_> {
    /// The focus classes of a step: the step itself and every step it shares an edge with.
    pub(super) fn node_classes(&self, node: usize) -> String {
        // A diagram that draws part of a model names the nodes of the whole.
        let (node, ends) = match &self.diagram.whole {
            Some(whole) => (whole.nodes[node], &whole.all_ends),
            None => (node, &self.ends),
        };
        let mut related: Vec<usize> = ends
            .iter()
            .filter_map(|&(from, to)| match (from == node, to == node) {
                (true, _) => Some(to),
                (_, true) => Some(from),
                _ => None,
            })
            .chain(std::iter::once(node))
            .collect();
        related.sort_unstable();
        related.dedup();
        focus_classes(Some(node), &related)
    }

    /// The focus classes of an edge: the steps at both its ends.
    pub(super) fn edge_classes(&self, edge: usize) -> String {
        let (from, to) = self
            .diagram
            .whole
            .as_ref()
            .map_or(self.ends[edge], |whole| whole.ends[edge]);
        focus_classes(None, &[from.min(to), from.max(to)])
    }
}

/// The width step of every edge with a weight: in proportion to its weight against the heaviest
/// edge, from 0 to [`WEIGHT_STEPS`]. Edges without a weight keep the usual line.
fn weight_widths(diagram: &Diagram) -> Vec<Option<u8>> {
    weight_widths_of(diagram.edges.iter().map(|edge| edge.weight))
}

/// [`weight_widths`] for any edges, given by their weights.
pub(crate) fn weight_widths_of(
    weights: impl Iterator<Item = Option<f64>> + Clone,
) -> Vec<Option<u8>> {
    let heaviest = weights.clone().flatten().fold(0.0_f64, f64::max);
    weights
        .map(|weight| {
            weight.map(|weight| {
                let share = if heaviest > 0.0 {
                    weight / heaviest
                } else {
                    0.0
                };
                // At most WEIGHT_STEPS, which fits in a u8.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let step = (share * WEIGHT_STEPS).round() as u8;
                step
            })
        })
        .collect()
}

/// The classes that tie a drawn part to the nodes whose focus brings it forward, and to its own
/// node when it is one: `chartlet-f chartlet-n-3 chartlet-f-1 chartlet-f-3`.
pub(crate) fn focus_classes(own: Option<usize>, related: &[usize]) -> String {
    let mut classes = String::from("chartlet-f");
    if let Some(own) = own {
        write!(classes, " chartlet-n-{own}").expect("writing to String cannot fail");
    }
    let mut last = None;
    for node in related {
        if last != Some(*node) {
            write!(classes, " chartlet-f-{node}").expect("writing to String cannot fail");
        }
        last = Some(*node);
    }
    classes
}

impl Plan {
    /// Draws the diagram and returns how far right and down the edge labels reached, which the
    /// canvas must hold too.
    pub(super) fn draw(
        &self,
        model: &Model,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
        elements: &mut Vec<Element>,
    ) -> (f64, f64) {
        self.draw_lanes(model, elements);
        self.draw_groups(model, metrics, warnings, elements);
        let mut labels = Vec::new();
        // Every edge, loop and step sits in a group whose classes name the steps it belongs to,
        // so that focusing a step can bring them forward.
        for (edge, width) in weight_widths(model.diagram).into_iter().enumerate() {
            let mut classes = model.edge_classes(edge);
            if let Some(width) = width {
                write!(classes, " chartlet-flow-w{width}").expect("writing to String cannot fail");
            }
            if let Some(category) = model.diagram.edges[edge].category {
                classes.push_str(category);
            }
            if let Some(role) = model.diagram.edges[edge].role {
                classes.push_str(roles::class(role));
            }
            elements.push(match &model.diagram.edges[edge].id {
                Some(id) => Element::EdgeGroup(classes, id.clone()),
                None => Element::Group(classes),
            });
            self.draw_edge(model, edge, elements, &mut labels);
            elements.push(Element::GroupEnd);
        }
        self.draw_branches(model, elements);
        for node in 0..model.diagram.nodes.len() {
            if model.diagram.nodes[node].shape == Shape::Block {
                continue;
            }
            elements.push(Element::Group(model.node_classes(node)));
            self.draw_loop(model, node, elements, &mut labels);
            elements.push(Element::GroupEnd);
        }
        let reach = self.draw_labels(model, labels, metrics, elements);
        let heat = heat::steps(
            &model
                .diagram
                .nodes
                .iter()
                .map(|node| node.value)
                .collect::<Vec<_>>(),
        );
        for (node, step) in heat.into_iter().enumerate() {
            // A block draws what it holds, each node with its own groups and hotspot.
            if let Some(block) = model.diagram.blocks.iter().find(|block| block.node == node) {
                let (x, y, _, _) = self.node_box(model, node);
                elements.extend(blocks::placed(&block.elements, (x, y)));
                continue;
            }
            let mut classes = model.node_classes(node);
            if let Some(step) = step {
                classes.push_str(&heat::class(step));
            }
            let spec = &model.diagram.nodes[node];
            if let Some(role) = spec.role {
                classes.push_str(roles::class(role));
            }
            elements.push(match &spec.id {
                Some(id) => Element::NodeGroup(classes, id.clone()),
                None => Element::Group(classes),
            });
            self.draw_node(model, node, elements);
            if let Some(role) = spec.role {
                let (x, y, _, _) = self.node_box(model, node);
                roles::push_badge(role, (x, y), elements);
            }
            elements.push(Element::GroupEnd);
            if spec.shape != Shape::Initial {
                let (x, y, width, height) = self.node_box(model, node);
                elements.push(Element::Hotspot(Hotspot {
                    node: model
                        .diagram
                        .whole
                        .as_ref()
                        .map_or(node, |whole| whole.nodes[node]),
                    label: spec.label.clone(),
                    x,
                    y,
                    width,
                    height,
                }));
            }
        }
        reach
    }

    /// A dot at every branch of a shared trunk, tied to the steps of all its edges.
    fn draw_branches(&self, model: &Model, elements: &mut Vec<Element>) {
        for trunk in &model.trunks.shared {
            let routes: Vec<&Route> = trunk
                .edges
                .iter()
                .filter_map(|&edge| self.routes[edge].as_ref())
                .collect();
            let Some(bus) = routes
                .iter()
                .find(|route| route.points.len() == 4)
                .map(|route| route.points[1].0)
            else {
                continue;
            };
            let shared = if trunk.leaving {
                routes[0].points[0].1
            } else {
                routes[0].points[routes[0].points.len() - 1].1
            };
            let others: Vec<f64> = routes
                .iter()
                .map(|route| match (route.points.len(), trunk.leaving) {
                    (4, true) => route.points[2].1,
                    (4, false) => route.points[1].1,
                    _ => shared,
                })
                .collect();
            let points = trunks::branches(shared, bus, &others);
            if points.is_empty() {
                continue;
            }
            let mut related: Vec<usize> = trunk
                .edges
                .iter()
                .flat_map(|&edge| [model.ends[edge].0, model.ends[edge].1])
                .collect();
            related.sort_unstable();
            elements.push(Element::Group(focus_classes(None, &related)));
            for point in points {
                let (cx, cy) = self.page(point);
                elements.push(Element::Circle(Circle {
                    cx,
                    cy,
                    radius: BRANCH,
                    class: "chartlet-flow-head",
                    series_index: None,
                    style_index: None,
                    tooltip: None,
                }));
            }
            elements.push(Element::GroupEnd);
        }
    }

    fn draw_lanes(&self, model: &Model, elements: &mut Vec<Element>) {
        let lanes = &model.diagram.lanes;
        for (index, (lane, band)) in lanes.iter().zip(&self.bands).enumerate() {
            elements.push(Element::Hook(Hook::Object(
                "data-chartlet-lane",
                lane.id.clone(),
            )));
            let (x1, y1) = self.page((0.0, band.0));
            let (x2, y2) = self.page((self.main_length, band.1));
            elements.push(Element::Rect(Rect {
                x: x1.min(x2),
                y: y1.min(y2),
                width: (x2 - x1).abs(),
                height: (y2 - y1).abs(),
                class: if index % 2 == 0 {
                    "chartlet-flow-lane"
                } else {
                    "chartlet-flow-lane chartlet-flow-lane-alt"
                },
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
            // A strip along the lane's start holds its name.
            let (h1, h2) = if self.landscape {
                (
                    self.page((0.0, band.0)),
                    self.page((self.main_length, band.0 + LANE_HEAD)),
                )
            } else {
                (self.page((0.0, band.0)), self.page((LANE_HEAD, band.1)))
            };
            elements.push(Element::Rect(Rect {
                x: h1.0.min(h2.0) + 0.5,
                y: h1.1.min(h2.1) + 0.5,
                width: (h2.0 - h1.0).abs() - 1.0,
                height: (h2.1 - h1.1).abs() - 1.0,
                class: "chartlet-flow-lane-head",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
            let (x, y, anchor) = if self.landscape {
                (x1 + 10.0, y1.min(y2) + 17.0, TextAnchor::Start)
            } else {
                (f64::midpoint(x1, x2), y1.min(y2) + 17.0, TextAnchor::Middle)
            };
            elements.push(Element::Text(Text {
                x,
                y,
                class: "chartlet-flow-lane-label",
                anchor,
                content: lane.label.clone(),
            }));
            elements.push(Element::Hook(Hook::End));
        }
    }

    /// The frame of every group: around its own steps and the frames inside it.
    pub(super) fn group_frames(
        &self,
        model: &Model,
        metrics: &impl TextMetrics,
    ) -> Vec<(f64, f64, f64, f64)> {
        let diagram = model.diagram;
        let groups = diagram.groups.len();
        let depth = |group: usize| diagram.chain(Some(group)).len();
        // Frames from the innermost out.
        let mut frames: Vec<(f64, f64, f64, f64)> = vec![(0.0, 0.0, 0.0, 0.0); groups];
        let mut inner_first: Vec<usize> = (0..groups).collect();
        crate::sort::by_key(&mut inner_first, |group| {
            (std::cmp::Reverse(depth(*group)), *group)
        });
        for &group in &inner_first {
            let boxes = (0..diagram.nodes.len())
                .filter(|node| diagram.nodes[*node].group == Some(group))
                .map(|node| {
                    let (x, y, width, height) = self.node_box(model, node);
                    (x, y, x + width, y + height)
                })
                .chain(
                    (0..groups)
                        .filter(|child| diagram.groups[*child].parent == Some(group))
                        .map(|child| frames[child]),
                );
            let (left, top, right, bottom) = boxes.fold(
                (
                    f64::INFINITY,
                    f64::INFINITY,
                    f64::NEG_INFINITY,
                    f64::NEG_INFINITY,
                ),
                |(left, top, right, bottom), b| {
                    (left.min(b.0), top.min(b.1), right.max(b.2), bottom.max(b.3))
                },
            );
            // A frame is at least as wide as its own name, which sits in its top left corner.
            let named = (metrics.width(&diagram.groups[group].label, SUBLABEL_SIZE) + 16.0).ceil();
            let extra = (named - (right - left + 2.0 * GROUP_PAD)).max(0.0);
            // Along the flow the frame grows to the right only, away from the frames around it;
            // across it, on both sides.
            let (before, after) = if self.landscape {
                (0.0, extra)
            } else {
                (extra / 2.0, extra / 2.0)
            };
            frames[group] = (
                left - GROUP_PAD - before,
                top - GROUP_PAD - GROUP_HEAD,
                right + GROUP_PAD + after,
                bottom + GROUP_PAD,
            );
        }
        frames
    }

    fn draw_groups(
        &self,
        model: &Model,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
        elements: &mut Vec<Element>,
    ) {
        let diagram = model.diagram;
        let groups = diagram.groups.len();
        let frames = self.group_frames(model, metrics);
        let depth = |group: usize| diagram.chain(Some(group)).len();
        let mut inner_first: Vec<usize> = (0..groups).collect();
        crate::sort::by_key(&mut inner_first, |group| {
            (std::cmp::Reverse(depth(*group)), *group)
        });
        for &group in inner_first.iter().rev() {
            let (left, top, right, bottom) = frames[group];
            let intruder = (0..diagram.nodes.len())
                .filter(|node| !diagram.inside(*node, group))
                .map(|node| self.node_box(model, node))
                .any(|(x, y, width, height)| {
                    x < right && x + width > left && y < bottom && y + height > top
                });
            if intruder {
                warnings.push(ChartWarning::new(
                    "group_overlap",
                    diagram.groups[group].path.clone(),
                    "a step outside the frame lies inside it; move it elsewhere or regroup the steps",
                ));
            }
            if let Some(id) = &diagram.groups[group].id {
                elements.push(Element::Hook(Hook::Object(
                    "data-chartlet-frame",
                    id.clone(),
                )));
            }
            elements.push(Element::Rect(Rect {
                x: left,
                y: top,
                width: right - left,
                height: bottom - top,
                class: "chartlet-flow-group",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
            let path = format!("{}/label", diagram.groups[group].path);
            elements.push(Element::Text(Text {
                x: left + 8.0,
                y: top + 13.0,
                class: "chartlet-flow-group-label",
                anchor: TextAnchor::Start,
                content: crate::layout::fit_text(
                    &diagram.groups[group].label,
                    right - left - 16.0,
                    SUBLABEL_SIZE,
                    metrics,
                    warnings,
                    &path,
                ),
            }));
            if diagram.groups[group].id.is_some() {
                elements.push(Element::Hook(Hook::End));
            }
        }
    }
}
