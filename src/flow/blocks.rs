//! Blocks: nodes laid out on their own, in an orientation of their own, and placed as one node
//! in the diagram around them, such as the classes of a package that runs landscape inside a
//! portrait diagram. The block is laid out first, by the same layered layout, and drawn from its
//! top left corner; the diagram around it then takes it as a node of its size. An edge from
//! outside meets the block where the node it joins lies inside and runs on, straight across the
//! flow of the diagram around, to the side of that node. That way is open only where nothing of
//! the block lies between the node and the side of the block the edge comes from: the block
//! knows, for each of its nodes and each of its four sides, whether it is, and a diagram whose
//! edges would cross a block in an orientation is not laid out in that orientation.

use super::{
    Diagram, Graph, SUBLABEL_SIZE, count,
    graph::Chain,
    measure::{Model, Spacing},
    place::Plan,
    ports::Ports,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::Element,
    spec::{ChartSpec, DiagramOrientation},
};

/// How far around the middle of a node's side edges from outside spread, as a share of the side,
/// and at most; the way to the node must be open that wide.
const SPREAD: f64 = 0.3;
const MOST_SPREAD: f64 = 24.0;
/// The distance between two edges from outside that meet the same side of one node.
const STEP: f64 = 12.0;

/// Nodes laid out on their own and placed as one node of a diagram.
pub(crate) struct Block {
    /// The node of the diagram around that stands for the block.
    pub node: usize,
    /// The size of the block: its frames, nodes, edges and labels.
    pub size: (f64, f64),
    /// What the block draws, from its top left corner.
    pub elements: Vec<Element>,
    /// The nodes of the whole model in the block.
    pub members: Vec<Member>,
}

/// A node of the whole model inside a block.
pub(crate) struct Member {
    /// Its index in the whole model.
    pub node: usize,
    /// Its box from the block's top left corner: left, top, width, height.
    pub bounds: (f64, f64, f64, f64),
    /// Whether an edge from outside reaches it straight from the top, right, bottom and left
    /// side of the block.
    pub open: [bool; 4],
}

/// How the nodes and edges of a diagram that draws part of a model stand in the whole, so that
/// focus classes, hotspots and tooltips name the nodes of the whole.
pub(crate) struct Whole {
    /// Per node of the diagram, its index in the whole; for a block node, that of its first
    /// member.
    pub nodes: Vec<usize>,
    /// Per edge of the diagram, the nodes of the whole it joins.
    pub ends: Vec<(usize, usize)>,
    /// The ends of every edge of the whole, and the label of every node of the whole.
    pub all_ends: Vec<(usize, usize)>,
    pub labels: Vec<String>,
}

impl Block {
    /// Lays out `inner`, whose `whole` names its nodes in the whole model, in its own orientation,
    /// portrait or landscape, as the block that node `node` of the diagram around stands for.
    pub(crate) fn new(
        spec: &ChartSpec,
        inner: &Diagram,
        node: usize,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
    ) -> Self {
        let editorial = spec.look == crate::spec::Look::Editorial;
        let spacing = Spacing::new(spec.width, inner.spacing, (editorial, spec.tags));
        let face = crate::metrics::Face::new(metrics, editorial);
        let metrics = &face;
        let model = Model::new(inner, (spacing, spec.number_style()), metrics, warnings);
        let landscape = inner.orientation == DiagramOrientation::Landscape;
        // On a canvas of no size, nothing stretches and nothing is centred.
        let mut canvas = spec.clone();
        canvas.width = 0;
        canvas.height = 0;
        let plan = Plan::new(&canvas, &model, landscape, 0.0, metrics);
        let mut elements = Vec::new();
        plan.draw(&model, metrics, warnings, &mut elements);
        let (left, top, right, bottom) = reach(&elements);
        let elements = placed(&elements, (-left, -top));
        let size = (right - left, bottom - top);
        let boxes: Vec<(f64, f64, f64, f64)> = (0..inner.nodes.len())
            .map(|node| {
                let (x, y, width, height) = plan.node_box(&model, node);
                (x - left, y - top, width, height)
            })
            .collect();
        let obstacles = obstacles(&elements, metrics);
        let whole = inner
            .whole
            .as_ref()
            .expect("a block names its nodes in the whole");
        let members = boxes
            .iter()
            .enumerate()
            .map(|(index, &bounds)| {
                let others: Vec<(f64, f64, f64, f64)> = boxes
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != index)
                    .map(|(_, &(x, y, width, height))| (x, y, x + width, y + height))
                    .chain(obstacles.iter().copied())
                    .collect();
                Member {
                    node: whole.nodes[index],
                    bounds,
                    open: open(bounds, size, &others),
                }
            })
            .collect();
        Self {
            node,
            size,
            elements,
            members,
        }
    }

    /// The member that is node `node` of the whole.
    fn member(&self, node: usize) -> usize {
        self.members
            .iter()
            .position(|member| member.node == node)
            .expect("an edge into a block joins one of its members")
    }

    /// The edges of the diagram around that meet the block: the member each joins, whether it
    /// leaves the block in the order the layout runs it, and the edge.
    fn meeting(&self, chains: &[Option<Chain>], whole: &Whole) -> Vec<(usize, bool, usize)> {
        let mut meeting = Vec::new();
        for (edge, chain) in chains.iter().enumerate() {
            let Some(chain) = chain else { continue };
            let (from, to) = whole.ends[edge];
            let (first, last) = if chain.reversed {
                (to, from)
            } else {
                (from, to)
            };
            if chain.items[0] == self.node {
                meeting.push((self.member(first), true, edge));
            }
            if chain.items[chain.items.len() - 1] == self.node {
                meeting.push((self.member(last), false, edge));
            }
        }
        meeting
    }
}

/// The side of a block an edge meets it on: the far side of the flow where it leaves, the near
/// side where it arrives; as an index into [`Member::open`].
const fn side(leaving: bool, landscape: bool) -> usize {
    match (landscape, leaving) {
        (false, false) => 0,
        (true, true) => 1,
        (false, true) => 2,
        (true, false) => 3,
    }
}

/// Whether every edge that meets a block of `diagram` reaches the node it joins there straight,
/// in portrait and in landscape.
pub(crate) fn blocks_reachable(diagram: &Diagram) -> [bool; 2] {
    let Some(whole) = &diagram.whole else {
        return [true, true];
    };
    let graph = Graph::new(diagram);
    [false, true].map(|landscape| {
        diagram.blocks.iter().all(|block| {
            block
                .meeting(&graph.chains, whole)
                .into_iter()
                .all(|(member, leaving, _)| block.members[member].open[side(leaving, landscape)])
        })
    })
}

impl Ports {
    /// The ports of the edges that meet a block: across the flow where the node they join lies
    /// inside, several on one side of a node spread around its middle, and on the main axis the
    /// side of that node they run on to.
    pub(super) fn blocks(&mut self, graph: &Graph, model: &Model, landscape: bool) {
        let diagram = model.diagram;
        let Some(whole) = &diagram.whole else { return };
        for block in &diagram.blocks {
            let (width, height) = block.size;
            let meeting = block.meeting(&graph.chains, whole);
            for &(member, leaving, edge) in &meeting {
                let (x, y, member_width, member_height) = block.members[member].bounds;
                let (middle, across, depth, breadth) = if landscape {
                    (
                        x + member_width / 2.0 - width / 2.0,
                        y + member_height / 2.0 - height / 2.0,
                        member_width,
                        member_height,
                    )
                } else {
                    (
                        y + member_height / 2.0 - height / 2.0,
                        x + member_width / 2.0 - width / 2.0,
                        member_height,
                        member_width,
                    )
                };
                let siblings: Vec<usize> = meeting
                    .iter()
                    .filter(|(other, way, _)| *other == member && *way == leaving)
                    .map(|(_, _, edge)| *edge)
                    .collect();
                let index = siblings
                    .iter()
                    .position(|sibling| *sibling == edge)
                    .expect("the edge is among them");
                let spread = 2.0 * (breadth * SPREAD).min(MOST_SPREAD);
                let step = STEP.min(spread / count(siblings.len().max(2) - 1));
                let offset = across + (count(index) - count(siblings.len() - 1) / 2.0) * step;
                if leaving {
                    self.leave[edge] = offset;
                    self.leave_into[edge] = Some(middle + depth / 2.0);
                } else {
                    self.reach[edge] = offset;
                    self.reach_into[edge] = Some(middle - depth / 2.0);
                }
            }
        }
    }
}

/// Which sides of the block of `size` the node with box `bounds` can be reached from straight:
/// a band as wide as edges spread, from the side of the node to the side of the block, that
/// overlaps none of the `others`, each given by its left, top, right and bottom.
fn open(
    (x, y, width, height): (f64, f64, f64, f64),
    (block_width, block_height): (f64, f64),
    others: &[(f64, f64, f64, f64)],
) -> [bool; 4] {
    let (half_width, half_height) = (
        (width * SPREAD).min(MOST_SPREAD),
        (height * SPREAD).min(MOST_SPREAD),
    );
    let (middle_x, middle_y) = (x + width / 2.0, y + height / 2.0);
    let bands = [
        (middle_x - half_width, 0.0, middle_x + half_width, y),
        (
            x + width,
            middle_y - half_height,
            block_width,
            middle_y + half_height,
        ),
        (
            middle_x - half_width,
            y + height,
            middle_x + half_width,
            block_height,
        ),
        (0.0, middle_y - half_height, x, middle_y + half_height),
    ];
    bands.map(|band| {
        !others.iter().any(|other| {
            band.0 < other.2 && other.0 < band.2 && band.1 < other.3 && other.1 < band.3
        })
    })
}

/// What an edge from outside must not cross inside a block, each by its left, top, right and
/// bottom: every stretch of an edge, every arrowhead and mark, every label and the names of
/// the frames.
fn obstacles(elements: &[Element], metrics: &impl TextMetrics) -> Vec<(f64, f64, f64, f64)> {
    let mut obstacles = Vec::new();
    for element in elements {
        match element {
            Element::Polyline(line)
                if line.class.starts_with("chartlet-flow-edge")
                    || line.class.starts_with("chartlet-flow-head") =>
            {
                obstacles.extend(line.points.windows(2).map(|pair| {
                    (
                        pair[0].0.min(pair[1].0),
                        pair[0].1.min(pair[1].1),
                        pair[0].0.max(pair[1].0),
                        pair[0].1.max(pair[1].1),
                    )
                }));
            }
            Element::Rect(rect) if rect.class == "chartlet-diagram-chip" => {
                obstacles.push((rect.x, rect.y, rect.x + rect.width, rect.y + rect.height));
            }
            Element::Text(text) if text.class == "chartlet-flow-group-label" => {
                let width = metrics.width(&text.content, SUBLABEL_SIZE);
                obstacles.push((text.x, text.y - 11.0, text.x + width, text.y + 3.0));
            }
            _ => {}
        }
    }
    obstacles
}

/// How far what `elements` draw reaches: left, top, right, bottom. Texts stand inside the boxes
/// and frames they belong to.
fn reach(elements: &[Element]) -> (f64, f64, f64, f64) {
    let mut reach = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    let mut take = |(x, y): (f64, f64)| {
        reach = (
            reach.0.min(x),
            reach.1.min(y),
            reach.2.max(x),
            reach.3.max(y),
        );
    };
    for element in elements {
        match element {
            Element::Rect(rect) => {
                take((rect.x, rect.y));
                take((rect.x + rect.width, rect.y + rect.height));
            }
            Element::Polyline(line) => {
                line.points.iter().copied().for_each(&mut take);
            }
            Element::Line(line) => {
                take((line.x1, line.y1));
                take((line.x2, line.y2));
            }
            Element::Circle(circle) => {
                take((circle.cx - circle.radius, circle.cy - circle.radius));
                take((circle.cx + circle.radius, circle.cy + circle.radius));
            }
            _ => {}
        }
    }
    reach
}

/// `elements` moved by `(dx, dy)`.
pub(super) fn placed(elements: &[Element], (dx, dy): (f64, f64)) -> Vec<Element> {
    let point = |(x, y): (f64, f64)| (x + dx, y + dy);
    elements
        .iter()
        .cloned()
        .map(|mut element| {
            match &mut element {
                Element::Circle(circle) => {
                    (circle.cx, circle.cy) = point((circle.cx, circle.cy));
                }
                Element::Line(line) => {
                    (line.x1, line.y1) = point((line.x1, line.y1));
                    (line.x2, line.y2) = point((line.x2, line.y2));
                }
                Element::Polyline(line) => {
                    line.points.iter_mut().for_each(|at| *at = point(*at));
                }
                Element::Rect(rect) => (rect.x, rect.y) = point((rect.x, rect.y)),
                Element::SeriesText(text, _) | Element::Text(text) => {
                    (text.x, text.y) = point((text.x, text.y));
                }
                Element::Hotspot(hotspot) => {
                    (hotspot.x, hotspot.y) = point((hotspot.x, hotspot.y));
                }
                Element::Symbol(symbol) => (symbol.x, symbol.y) = point((symbol.x, symbol.y)),
                Element::Hook(_)
                | Element::Group(_)
                | Element::NodeGroup(..)
                | Element::EdgeGroup(..)
                | Element::GroupEnd => {}
            }
            element
        })
        .collect()
}
