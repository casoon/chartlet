use serde::{Deserialize, Serialize};

use super::{ChartSpec, ChartType, LegendPlacement};
use crate::error::{ChartError, ChartWarning};

/// Emphasized nodes beyond which the accent stops pointing anywhere.
pub(crate) const MAX_EMPHASIZED: usize = 2;
/// Entries of a path; a longer one is no longer a path the eye follows.
const MAX_PATH: usize = 40;

/// What the reader should look at first in a diagram: up to two nodes, which get the accent
/// outline and tint, and a path, whose edges get the accent line. Everything else stays neutral.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmphasisSpec {
    /// The ids of the emphasized nodes: steps, states, components, classes, participants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<String>,
    /// The ids of the nodes along the emphasized path in order, each joined to the next by an
    /// edge (a message, in a sequence diagram).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<String>,
}

impl EmphasisSpec {
    /// Whether the node with `id` is emphasized.
    pub(crate) fn has_node(&self, id: &str) -> bool {
        self.nodes.iter().any(|node| node == id)
    }
}

/// The nodes of a diagram and what joins them, by id, as emphasis needs them.
struct Graph<'a> {
    /// What a node is called in messages.
    noun: &'static str,
    /// What joins two nodes is called in messages.
    edge: &'static str,
    ids: Vec<&'a str>,
    edges: Vec<(&'a str, &'a str)>,
}

impl ChartSpec {
    /// Whether this chart type can emphasize nodes and a path.
    pub(crate) const fn takes_emphasis(&self) -> bool {
        matches!(
            self.chart_type,
            ChartType::Flow
                | ChartType::State
                | ChartType::Architecture
                | ChartType::Classes
                | ChartType::Tree
                | ChartType::Network
                | ChartType::Sequence
        )
    }

    /// Whether this chart type can show a legend strip.
    pub(crate) const fn takes_legend(&self) -> bool {
        self.takes_emphasis()
    }

    /// Whether the diagram asks for a legend strip.
    pub(crate) fn wants_legend(&self) -> bool {
        self.takes_legend() && self.legend == LegendPlacement::Auto
    }

    /// Whether the diagram emphasizes anything.
    pub(crate) fn has_emphasis(&self) -> bool {
        self.emphasis
            .as_ref()
            .is_some_and(|emphasis| !emphasis.nodes.is_empty() || !emphasis.path.is_empty())
    }

    /// The nodes and edges of the diagram, by id.
    #[allow(clippy::too_many_lines)]
    fn emphasis_graph(&self) -> Option<Graph<'_>> {
        match self.chart_type {
            ChartType::Flow => {
                let flow = self.flow.as_ref()?;
                Some(Graph {
                    noun: "step",
                    edge: "edge",
                    ids: flow.nodes.iter().map(|node| node.id.as_str()).collect(),
                    edges: flow
                        .edges
                        .iter()
                        .map(|edge| (edge.from.as_str(), edge.to.as_str()))
                        .collect(),
                })
            }
            ChartType::State => {
                let machine = self.state.as_ref()?;
                Some(Graph {
                    noun: "state",
                    edge: "transition",
                    ids: machine
                        .states
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| !machine.is_composite(*index))
                        .map(|(_, state)| state.id.as_str())
                        .collect(),
                    edges: machine
                        .transitions
                        .iter()
                        .map(|transition| (transition.from.as_str(), transition.to.as_str()))
                        .collect(),
                })
            }
            ChartType::Architecture => {
                let architecture = self.architecture.as_ref()?;
                Some(Graph {
                    noun: "component",
                    edge: "connection",
                    ids: architecture
                        .components
                        .iter()
                        .map(|component| component.id.as_str())
                        .collect(),
                    edges: architecture
                        .connections
                        .iter()
                        .map(|connection| (connection.from.as_str(), connection.to.as_str()))
                        .collect(),
                })
            }
            ChartType::Classes => {
                let classes = self.classes.as_ref()?;
                Some(Graph {
                    noun: "class",
                    edge: "relation",
                    ids: classes
                        .classes
                        .iter()
                        .map(|class| class.id.as_str())
                        .collect(),
                    edges: classes
                        .relations
                        .iter()
                        .map(|relation| (relation.from.as_str(), relation.to.as_str()))
                        .collect(),
                })
            }
            ChartType::Tree => {
                let tree = self.tree.as_ref()?;
                Some(Graph {
                    noun: "node",
                    edge: "link",
                    ids: tree.nodes.iter().map(|node| node.id.as_str()).collect(),
                    edges: tree
                        .nodes
                        .iter()
                        .filter_map(|node| {
                            let other = node.parent.as_deref().or(node.partner.as_deref())?;
                            Some((other, node.id.as_str()))
                        })
                        .collect(),
                })
            }
            ChartType::Network => {
                let network = self.network.as_ref()?;
                Some(Graph {
                    noun: "node",
                    edge: "edge",
                    ids: network.nodes.iter().map(|node| node.id.as_str()).collect(),
                    edges: network
                        .edges
                        .iter()
                        .map(|edge| (edge.from.as_str(), edge.to.as_str()))
                        .collect(),
                })
            }
            ChartType::Sequence => {
                let sequence = self.sequence.as_ref()?;
                Some(Graph {
                    noun: "participant",
                    edge: "message",
                    ids: sequence
                        .participants
                        .iter()
                        .map(|participant| participant.id.as_str())
                        .collect(),
                    edges: sequence
                        .messages
                        .iter()
                        .map(|message| (message.from.as_str(), message.to.as_str()))
                        .collect(),
                })
            }
            _ => None,
        }
    }

    /// The id and the label of every node an emphasis can name, in the order of the specification.
    pub(crate) fn emphasis_labels(&self) -> Vec<(&str, &str)> {
        match self.chart_type {
            ChartType::Flow => self
                .flow
                .iter()
                .flat_map(|flow| {
                    flow.nodes
                        .iter()
                        .map(|node| (node.id.as_str(), node.label.as_str()))
                })
                .collect(),
            ChartType::State => self
                .state
                .iter()
                .flat_map(|machine| {
                    machine
                        .states
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| !machine.is_composite(*index))
                        .map(|(_, state)| (state.id.as_str(), state.label.as_str()))
                })
                .collect(),
            ChartType::Architecture => self
                .architecture
                .iter()
                .flat_map(|architecture| {
                    architecture
                        .components
                        .iter()
                        .map(|component| (component.id.as_str(), component.label.as_str()))
                })
                .collect(),
            ChartType::Classes => self
                .classes
                .iter()
                .flat_map(|classes| {
                    classes
                        .classes
                        .iter()
                        .map(|class| (class.id.as_str(), class.label.as_str()))
                })
                .collect(),
            ChartType::Tree => self
                .tree
                .iter()
                .flat_map(|tree| {
                    tree.nodes
                        .iter()
                        .map(|node| (node.id.as_str(), node.label.as_str()))
                })
                .collect(),
            ChartType::Network => self
                .network
                .iter()
                .flat_map(|network| {
                    network
                        .nodes
                        .iter()
                        .map(|node| (node.id.as_str(), node.label.as_str()))
                })
                .collect(),
            ChartType::Sequence => {
                self.sequence
                    .iter()
                    .flat_map(|sequence| {
                        sequence.participants.iter().map(|participant| {
                            (participant.id.as_str(), participant.label.as_str())
                        })
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// Rejects `emphasis` and `legend` on a chart that is no diagram.
    pub(super) fn reject_foreign_emphasis(&self) -> Result<(), ChartError> {
        if self.emphasis.is_some() && !self.takes_emphasis() {
            return Err(ChartError::new(
                "option_not_supported",
                "/emphasis",
                "emphasis marks nodes of a flow, state, architecture, classes, tree, network or sequence diagram; remove it or change the type",
            ));
        }
        if matches!(self.legend, LegendPlacement::Auto | LegendPlacement::Off)
            && !self.takes_legend()
        {
            return Err(ChartError::new(
                "option_not_supported",
                "/legend",
                "the values auto and none belong to a flow, state, architecture, classes, tree, network or sequence diagram; a time chart takes top or end",
            ));
        }
        if matches!(self.legend, LegendPlacement::End) && self.takes_legend() {
            return Err(ChartError::new(
                "option_not_supported",
                "/legend",
                "a diagram takes auto or none; end belongs to a time chart",
            ));
        }
        Ok(())
    }

    /// Checks the emphasis of a diagram: every id names a node, the path follows edges, and warns
    /// when more than two nodes are emphasized.
    pub(super) fn validate_emphasis(&self) -> Result<Vec<ChartWarning>, ChartError> {
        let Some(emphasis) = &self.emphasis else {
            return Ok(Vec::new());
        };
        let Some(graph) = self.emphasis_graph() else {
            return Ok(Vec::new());
        };
        let known = |id: &str| graph.ids.contains(&id);
        let unknown = |id: &str, path: String| {
            ChartError::new(
                "unknown_node",
                path,
                format!(
                    "no {} has the id \"{id}\"; emphasis names the ids of the diagram",
                    graph.noun
                ),
            )
        };
        for (index, id) in emphasis.nodes.iter().enumerate() {
            if !known(id) {
                return Err(unknown(id, format!("/emphasis/nodes/{index}")));
            }
            if emphasis.nodes[..index].contains(id) {
                return Err(ChartError::new(
                    "duplicate_emphasis",
                    format!("/emphasis/nodes/{index}"),
                    format!(
                        "\"{id}\" is emphasized twice; name every {} once",
                        graph.noun
                    ),
                ));
            }
        }
        if emphasis.path.len() > MAX_PATH {
            return Err(ChartError::new(
                "too_many_nodes",
                "/emphasis/path",
                format!("a path has at most {MAX_PATH} {}s", graph.noun),
            ));
        }
        for (index, id) in emphasis.path.iter().enumerate() {
            if !known(id) {
                return Err(unknown(id, format!("/emphasis/path/{index}")));
            }
        }
        if !emphasis.path.is_empty() && emphasis.path.len() < 2 {
            return Err(ChartError::new(
                "emphasis_path_short",
                "/emphasis/path",
                format!(
                    "a path joins at least two {}s; name more or use emphasis.nodes",
                    graph.noun
                ),
            ));
        }
        for (index, pair) in emphasis.path.windows(2).enumerate() {
            let joined = graph.edges.iter().any(|&(from, to)| {
                (from == pair[0] && to == pair[1]) || (from == pair[1] && to == pair[0])
            });
            if !joined {
                return Err(ChartError::new(
                    "emphasis_path_gap",
                    format!("/emphasis/path/{}", index + 1),
                    format!(
                        "no {} joins \"{}\" and \"{}\"; the path follows {}s",
                        graph.edge, pair[0], pair[1], graph.edge
                    ),
                ));
            }
        }
        let mut warnings = Vec::new();
        if emphasis.nodes.len() > MAX_EMPHASIZED {
            warnings.push(ChartWarning::new(
                "too_much_emphasis",
                "/emphasis/nodes",
                format!(
                    "{} nodes are emphasized; beyond {MAX_EMPHASIZED} the accent no longer says what to look at first, so keep one or two and let the path carry the rest",
                    emphasis.nodes.len()
                ),
            ));
        }
        Ok(warnings)
    }
}
