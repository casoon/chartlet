//! The diagram the layered layout works on, whatever specification it comes from: steps with a
//! shape, edges with what they have at their ends, and the lanes and groups the steps name.

use super::blocks::{Block, Whole};
use crate::{
    spec::{
        Dash, DiagramOrientation, DiagramRole, DiagramSpacing, FlowSpec, NodeKind, PartSpec,
        PortSide,
    },
    symbols::Symbol,
};

/// A diagram for the layered layout, whatever specification it comes from: steps with a shape,
/// edges between them, and the lanes and groups the steps name by index.
#[derive(Default)]
pub(crate) struct Diagram {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub lanes: Vec<Lane>,
    pub groups: Vec<Group>,
    /// Sets of steps that share one layer, by index; no step lies in two.
    pub ranks: Vec<Vec<usize>>,
    pub orientation: DiagramOrientation,
    /// How much room the diagram leaves between its layers and between neighbours in a layer.
    pub spacing: DiagramSpacing,
    /// The blocks the nodes of shape [`Shape::Block`] stand for.
    pub blocks: Vec<Block>,
    /// For a diagram that draws part of a model, how its nodes and edges stand in the whole.
    pub whole: Option<Whole>,
}

pub(crate) struct Node {
    pub label: String,
    pub sublabel: Option<String>,
    pub shape: Shape,
    pub lane: Option<usize>,
    pub group: Option<usize>,
    /// The node's ID in the specification, which the hooks carry; `None` for a node the
    /// specification does not name, such as the start dot of a state diagram.
    pub id: Option<String>,
    /// Where the node stands in the specification, for warnings about its texts.
    pub path: String,
    /// The compartments of a class box below its name, each a list of lines such as the
    /// attributes or the methods; empty for every other shape.
    pub compartments: Vec<Vec<String>>,
    /// The value that colors the node's fill.
    pub value: Option<f64>,
    /// The parts drawn as a stripe inside the bottom of the box.
    pub parts: Vec<PartSpec>,
    /// The symbol drawn above the label.
    pub symbol: Option<&'static Symbol>,
    /// The tag the specification gives the node, which replaces the tag its shape carries in the
    /// editorial look.
    pub tag: Option<String>,
    /// What the node means beyond its kind: its outline and a badge on its corner.
    pub role: Option<DiagramRole>,
    /// The node's category in the perspective a diagram shows, as the class of the stripe drawn
    /// across the bottom of its box and the category's name.
    pub category: Option<(&'static str, String)>,
}

impl Node {
    /// Whether a stripe runs inside the bottom of the box: of parts, or of a category.
    pub(super) fn has_stripe(&self) -> bool {
        !self.parts.is_empty() || self.category.is_some()
    }
}

pub(crate) struct Edge {
    pub from: usize,
    pub to: usize,
    /// The edge's ID in the specification, which the hooks carry; `None` for an edge the
    /// specification gives no ID.
    pub id: Option<String>,
    pub label: Option<String>,
    /// How the connection is made, written in brackets on a line below the label.
    pub technology: Option<String>,
    pub dash: Option<Dash>,
    /// Whether the edge belongs to the main path.
    pub main: bool,
    /// Where the edge's label stands in the specification, for warnings about it.
    pub label_path: String,
    /// What is drawn at either end of the line.
    pub ends: Ends,
    /// The sign between the two ends in the edge's tooltip: `→`, or `↔` and `—` for an edge
    /// that runs both ways or has no direction.
    pub sign: &'static str,
    /// How much the edge carries; the line is drawn stronger the more.
    pub weight: Option<f64>,
    /// What the edge means beyond its kind: the color and pattern of its line.
    pub role: Option<DiagramRole>,
    /// The class its group gains for the category of a perspective it is in: the color of its
    /// line and arrowheads.
    pub category: Option<&'static str>,
    /// The sides of its steps the edge is asked to leave and reach.
    pub sides: Sides,
}

/// The sides of its steps an edge is asked to leave and reach, and where the edge stands in the
/// specification, for warnings about them.
#[derive(Debug, Clone, Default)]
pub(crate) struct Sides {
    pub from: Option<PortSide>,
    pub to: Option<PortSide>,
    pub path: String,
}

impl Sides {
    /// Whether the edge asks for a side at either end.
    pub(super) const fn any(&self) -> bool {
        self.from.is_some() || self.to.is_some()
    }
}

/// What an edge has at one of its ends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum End {
    /// Nothing.
    None,
    /// A filled arrowhead: the default at the end of an edge.
    #[default]
    Arrow,
    /// Two strokes: a dependency.
    Open,
    /// A hollow triangle: inheritance, realization.
    Triangle,
    /// A filled diamond: composition.
    DiamondFilled,
    /// A hollow diamond: aggregation.
    DiamondHollow,
    /// A half circle around the ball of an interface: the interface is required.
    Socket,
}

/// Both ends of an edge, with the text each carries, such as a cardinality.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Ends {
    pub tail: End,
    pub head: End,
    pub tail_text: Option<String>,
    pub head_text: Option<String>,
}

impl Ends {
    /// The usual edge: nothing at its start, an arrowhead at its end.
    pub(crate) fn arrow() -> Self {
        Self {
            tail: End::None,
            head: End::Arrow,
            tail_text: None,
            head_text: None,
        }
    }
}

/// A lane: its name and its ID in the specification, which the hooks carry.
pub(crate) struct Lane {
    pub label: String,
    pub id: String,
}

pub(crate) struct Group {
    pub label: String,
    /// The group's ID in the specification, which the hooks carry; `None` for a group the
    /// specification gives no ID.
    pub id: Option<String>,
    pub path: String,
    /// The group this one lies in.
    pub parent: Option<usize>,
}

/// How a node is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shape {
    Start,
    End,
    Process,
    Decision,
    Io,
    Subprocess,
    Store,
    External,
    /// Where a state machine begins: a small filled circle without a label inside.
    Initial,
    /// A state: a box with well rounded corners.
    State,
    /// A final state: a state with a double outline.
    Final,
    /// Someone who uses a system: a box with a head on top.
    Person,
    /// What a person sees: a box with a window bar.
    Frontend,
    /// A queue: a box with a stack behind it.
    Queue,
    /// File or object storage: a bucket, wider at the top.
    Bucket,
    /// A cache: a hexagon.
    Cache,
    /// A security component: a shield.
    Shield,
    /// A class or an entity: a box with its name on top and compartments below.
    Class,
    /// A machine that runs other components: a box seen from a corner.
    Node,
    /// Something that is deployed: a box with a folded corner.
    Artifact,
    /// A note that explains another node: a pale box with a folded corner and its text set left,
    /// joined to that node by a dotted line without marks.
    Note,
    /// An interface a component provides: a ball with its name beside it, joined to the
    /// component by a stem, a line without marks.
    Interface,
    /// Nodes laid out on their own and placed as one, see [`Block`].
    Block,
}

impl From<NodeKind> for Shape {
    fn from(kind: NodeKind) -> Self {
        match kind {
            NodeKind::Start => Self::Start,
            NodeKind::End => Self::End,
            NodeKind::Process => Self::Process,
            NodeKind::Decision => Self::Decision,
            NodeKind::Io => Self::Io,
            NodeKind::Subprocess => Self::Subprocess,
            NodeKind::Store => Self::Store,
            NodeKind::External => Self::External,
        }
    }
}

impl Diagram {
    pub(super) fn from_flow(flow: &FlowSpec) -> Self {
        let ends = flow.ends();
        Self {
            nodes: flow
                .nodes
                .iter()
                .enumerate()
                .map(|(index, node)| Node {
                    label: node.label.clone(),
                    sublabel: node.sublabel.clone(),
                    shape: node.kind.into(),
                    lane: flow.lane_of(index),
                    group: flow.group_of(index),
                    id: Some(node.id.clone()),
                    path: format!("/flow/nodes/{index}"),
                    compartments: Vec::new(),
                    value: node.value,
                    parts: node.parts.clone(),
                    symbol: None,
                    tag: node.tag.clone(),
                    role: node.role,
                    category: None,
                })
                .collect(),
            edges: flow
                .edges
                .iter()
                .zip(ends)
                .enumerate()
                .map(|(index, (edge, (from, to)))| Edge {
                    from,
                    to,
                    id: None,
                    label: edge.label.clone(),
                    technology: None,
                    dash: edge.dash,
                    main: flow.on_main_path(index),
                    label_path: format!("/flow/edges/{index}/label"),
                    ends: Ends::arrow(),
                    sign: "→",
                    weight: edge.weight,
                    role: edge.role,
                    category: None,
                    sides: Sides {
                        from: edge.from_side,
                        to: edge.to_side,
                        path: format!("/flow/edges/{index}"),
                    },
                })
                .collect(),
            lanes: flow
                .lanes
                .iter()
                .map(|lane| Lane {
                    label: lane.label.clone(),
                    id: lane.id.clone(),
                })
                .collect(),
            groups: flow
                .groups
                .iter()
                .enumerate()
                .map(|(index, group)| Group {
                    label: group.label.clone(),
                    id: None,
                    path: format!("/flow/groups/{index}"),
                    parent: None,
                })
                .collect(),
            ranks: flow
                .same_rank
                .iter()
                .map(|set| {
                    set.iter()
                        .map(|id| flow.node(id).expect("validated ranks name steps"))
                        .collect()
                })
                .collect(),
            orientation: flow.orientation,
            spacing: flow.spacing,
            ..Self::default()
        }
    }

    /// The groups around `group`, outermost first, ending with `group` itself.
    pub(crate) fn chain(&self, group: Option<usize>) -> Vec<usize> {
        let mut chain = Vec::new();
        let mut at = group;
        while let Some(group) = at {
            chain.push(group);
            at = self.groups[group].parent;
        }
        chain.reverse();
        chain
    }

    /// Adds a note with `text` on node `target`: a node of its own in the target's lane and group,
    /// and a dotted line from the target to it. The line is no edge of the model; adapters keep
    /// both out of their descriptions, tables and structure.
    pub(crate) fn push_note(&mut self, target: usize, text: &str, path: String) {
        let note = self.nodes.len();
        self.edges.push(Edge {
            from: target,
            to: note,
            id: None,
            label: None,
            technology: None,
            dash: Some(Dash::Dotted),
            main: false,
            label_path: format!("{path}/text"),
            ends: Ends {
                tail: End::None,
                head: End::None,
                tail_text: None,
                head_text: None,
            },
            sign: "—",
            weight: None,
            role: None,
            category: None,
            sides: Sides::default(),
        });
        self.nodes.push(Node {
            label: text.to_owned(),
            sublabel: None,
            shape: Shape::Note,
            lane: self.nodes[target].lane,
            group: self.nodes[target].group,
            path,
            compartments: Vec::new(),
            value: None,
            parts: Vec::new(),
            id: None,
            symbol: None,
            tag: None,
            role: None,
            category: None,
        });
    }

    /// Whether node `node` lies in group `group`, directly or in a group inside it.
    pub(crate) fn inside(&self, node: usize, group: usize) -> bool {
        self.chain(self.nodes[node].group).contains(&group)
    }

    pub(super) fn ends(&self) -> Vec<(usize, usize)> {
        self.edges.iter().map(|edge| (edge.from, edge.to)).collect()
    }
}
