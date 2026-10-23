use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    ChartSpec, Dash, DiagramRole,
    flow::{validate_role, validate_weight},
    sequence::is_identifier,
    validate_text,
};
use crate::error::{ChartError, ChartWarning};

/// Nodes and edges of one network; beyond this the labels around a circle crowd each other.
pub(crate) const MAX_NODES: usize = 40;
pub(crate) const MAX_EDGES: usize = 120;
/// Groups, each with its own color and an entry in the legend.
pub(crate) const MAX_GROUPS: usize = 8;

/// A network: nodes joined by edges, with no direction of flow, on a circle or in rings around
/// one node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkSpec {
    pub nodes: Vec<NetworkNodeSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<NetworkEdgeSpec>,
    #[serde(default, skip_serializing_if = "NetworkLayout::is_circle")]
    pub layout: NetworkLayout,
    /// The node in the middle of the rings; required with `layout: "rings"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center: Option<String>,
    /// Whether edges lead from one node to the other: drawn with an arrowhead.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub directed: bool,
    /// The order of the nodes around a circle.
    #[serde(default, skip_serializing_if = "NetworkOrder::is_crossings")]
    pub order: NetworkOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkNodeSpec {
    pub id: String,
    pub label: String,
    /// The group a node belongs to: its color, a legend entry, and on a circle a neighbourhood.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<DiagramRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkEdgeSpec {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash: Option<Dash>,
    /// How much the edge carries, a number of at least 0: the line is drawn as much stronger as
    /// it carries more.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<DiagramRole>,
}

/// How the nodes of a network are placed.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkLayout {
    /// Every node on one circle.
    #[default]
    Circle,
    /// The center in the middle, every other node on the ring of its distance from it in edges.
    Rings,
}

impl NetworkLayout {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_circle(&self) -> bool {
        matches!(self, Self::Circle)
    }
}

/// The order of the nodes around a circle, clockwise from twelve o'clock.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkOrder {
    /// Groups together, and within them the order that lets the fewest edges cross.
    #[default]
    Crossings,
    /// The order of the specification.
    Listed,
}

impl NetworkOrder {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_crossings(&self) -> bool {
        matches!(self, Self::Crossings)
    }
}

impl NetworkSpec {
    pub(crate) fn node(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }

    /// Both ends of every edge, as node indices.
    pub(crate) fn ends(&self) -> Vec<(usize, usize)> {
        self.edges
            .iter()
            .map(|edge| {
                (
                    self.node(&edge.from).expect("validated edges name nodes"),
                    self.node(&edge.to).expect("validated edges name nodes"),
                )
            })
            .collect()
    }

    /// The groups in the order they first appear.
    pub(crate) fn groups(&self) -> Vec<&str> {
        let mut groups: Vec<&str> = Vec::new();
        for group in self.nodes.iter().filter_map(|node| node.group.as_deref()) {
            if !groups.contains(&group) {
                groups.push(group);
            }
        }
        groups
    }

    /// The index of every node's group among [`Self::groups`].
    pub(crate) fn group_of(&self, node: usize) -> Option<usize> {
        let group = self.nodes[node].group.as_deref()?;
        self.groups().iter().position(|known| *known == group)
    }
}

impl ChartSpec {
    /// Nodes with unique ids, edges between two different existing nodes, a center exactly where
    /// the rings need one, and at most eight groups.
    pub(super) fn validate_network(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a network", "network.nodes")?;
        let Some(network) = &self.network else {
            return Err(ChartError::new(
                "missing_network",
                "/network",
                "a network requires a network block",
            ));
        };
        if network.nodes.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/network/nodes",
                "provide at least one node",
            ));
        }
        if network.nodes.len() > MAX_NODES {
            return Err(ChartError::new(
                "too_many_nodes",
                "/network/nodes",
                format!("at most {MAX_NODES} nodes are supported"),
            ));
        }
        if network.edges.len() > MAX_EDGES {
            return Err(ChartError::new(
                "too_many_edges",
                "/network/edges",
                format!("at most {MAX_EDGES} edges are supported"),
            ));
        }
        network.validate_nodes()?;
        network.validate_edges()?;
        network.validate_layout()?;
        Ok(Vec::new())
    }
}

impl NetworkSpec {
    fn validate_nodes(&self) -> Result<(), ChartError> {
        let mut ids = BTreeSet::new();
        let mut groups = Vec::new();
        for (index, node) in self.nodes.iter().enumerate() {
            let path = format!("/network/nodes/{index}");
            if !is_identifier(&node.id) {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/id"),
                    "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            if !ids.insert(node.id.as_str()) {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    format!("the id \"{}\" is already taken by an earlier node", node.id),
                ));
            }
            validate_text(&node.label, &format!("{path}/label"), 60)?;
            if let Some(group) = &node.group {
                validate_text(group, &format!("{path}/group"), 40)?;
                if !groups.contains(group) {
                    groups.push(group.clone());
                }
                if groups.len() > MAX_GROUPS {
                    return Err(ChartError::new(
                        "too_many_groups",
                        format!("{path}/group"),
                        format!("at most {MAX_GROUPS} groups are supported"),
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_edges(&self) -> Result<(), ChartError> {
        for (index, edge) in self.edges.iter().enumerate() {
            let path = format!("/network/edges/{index}");
            for (field, id) in [("from", &edge.from), ("to", &edge.to)] {
                if self.node(id).is_none() {
                    return Err(ChartError::new(
                        "unknown_node",
                        format!("{path}/{field}"),
                        format!("no node has the id \"{id}\""),
                    ));
                }
            }
            if edge.from == edge.to {
                return Err(ChartError::new(
                    "self_loop",
                    format!("{path}/to"),
                    "an edge of a network joins two different nodes",
                ));
            }
            validate_weight(edge.weight, &path)?;
            validate_role(edge.role, edge.dash, &path)?;
        }
        Ok(())
    }

    fn validate_layout(&self) -> Result<(), ChartError> {
        match (self.layout, &self.center) {
            (NetworkLayout::Rings, None) => Err(ChartError::new(
                "missing_center",
                "/network/center",
                "rings need a center: the id of the node in the middle",
            )),
            (NetworkLayout::Rings, Some(center)) if self.node(center).is_none() => {
                Err(ChartError::new(
                    "unknown_node",
                    "/network/center",
                    format!("no node has the id \"{center}\""),
                ))
            }
            (NetworkLayout::Rings, Some(_)) if self.order != NetworkOrder::Crossings => {
                Err(ChartError::new(
                    "option_not_supported",
                    "/network/order",
                    "rings order their nodes by the ring before; order belongs to a circle",
                ))
            }
            (NetworkLayout::Circle, Some(_)) => Err(ChartError::new(
                "option_not_supported",
                "/network/center",
                "a circle has no center node; use \"layout\": \"rings\"",
            )),
            _ => Ok(()),
        }
    }
}
