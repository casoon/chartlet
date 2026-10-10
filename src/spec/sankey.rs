use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many links and nodes a Sankey diagram holds.
const MAX_LINKS: usize = 100;
const MAX_NODES: usize = 40;

/// A Sankey diagram: nodes in columns, joined by bands as thick as the flow between them. The
/// nodes are the labels the links name, in the order they first appear.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SankeySpec {
    pub links: Vec<SankeyLinkSpec>,
    /// Nodes whose order or column the specification fixes; the others follow from the links.
    /// The listed nodes come first, in this order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<SankeyNodeSpec>,
    /// `auto` (default) orders the nodes of a column to cross as few bands as possible; `listed`
    /// keeps them in the order of `nodes`, then of the links.
    #[serde(default, skip_serializing_if = "SankeyOrder::is_auto")]
    pub order: SankeyOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SankeyNodeSpec {
    pub label: String,
    /// The column, counted from 0 at the left. A node cannot stand left of the column its
    /// incoming links lead to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    /// The palette color of the node and the bands that leave it, 1 to 4; without it the nodes
    /// take the colors in turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<u8>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SankeyOrder {
    #[default]
    Auto,
    Listed,
}

impl SankeyOrder {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }
}

/// The most columns a Sankey diagram takes.
const MAX_COLUMNS: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SankeyLinkSpec {
    pub from: String,
    pub to: String,
    pub value: f64,
}

/// The nodes and links of a Sankey diagram as indexes, with the column of every node.
pub(crate) struct SankeyGraph {
    pub labels: Vec<String>,
    /// `(from, to, value)` in the order of the specification.
    pub links: Vec<(usize, usize, f64)>,
    /// The longest path from a node without incoming links: its column. `None` for a cycle.
    pub ranks: Option<Vec<usize>>,
    /// Keep the nodes of a column in the listed order instead of ordering them by their links.
    pub listed: bool,
    /// The palette color a node names for itself, from 1.
    pub colors: Vec<Option<usize>>,
}

impl SankeySpec {
    pub(crate) fn graph(&self) -> SankeyGraph {
        let mut labels: Vec<String> = self.nodes.iter().map(|node| node.label.clone()).collect();
        let mut index = |label: &str| {
            labels
                .iter()
                .position(|known| known == label)
                .unwrap_or_else(|| {
                    labels.push(label.to_owned());
                    labels.len() - 1
                })
        };
        let links: Vec<(usize, usize, f64)> = self
            .links
            .iter()
            .map(|link| (index(&link.from), index(&link.to), link.value))
            .collect();
        let count = labels.len();
        let mut ranks: Vec<usize> = (0..labels.len())
            .map(|node| self.nodes.get(node).and_then(|n| n.column).unwrap_or(0))
            .collect();
        let mut settled = false;
        // Without a cycle the ranks settle within one pass for every node on the longest path.
        for _ in 0..=labels.len() {
            settled = true;
            for &(from, to, _) in &links {
                if ranks[to] < ranks[from] + 1 {
                    ranks[to] = ranks[from] + 1;
                    settled = false;
                }
            }
            if settled {
                break;
            }
        }
        SankeyGraph {
            labels,
            links,
            ranks: settled.then_some(ranks),
            listed: self.order == SankeyOrder::Listed,
            colors: (0..count)
                .map(|node| self.nodes.get(node).and_then(|n| n.color).map(usize::from))
                .collect(),
        }
    }
}

impl SankeyGraph {
    /// What flows into and out of a node.
    pub(crate) fn flows(&self, node: usize) -> (f64, f64) {
        self.links
            .iter()
            .fold((0.0, 0.0), |(into, out), &(from, to, value)| {
                (
                    into + if to == node { value } else { 0.0 },
                    out + if from == node { value } else { 0.0 },
                )
            })
    }

    /// The size of a node: the larger of what flows in and out.
    pub(crate) fn size(&self, node: usize) -> f64 {
        let (into, out) = self.flows(node);
        into.max(out)
    }

    /// What enters the diagram: the outflow of the nodes without incoming links.
    pub(crate) fn total(&self) -> f64 {
        (0..self.labels.len())
            .filter(|node| self.links.iter().all(|link| link.1 != *node))
            .map(|node| self.flows(node).1)
            .sum()
    }
}

impl ChartSpec {
    /// Links between two different labels, once for each pair, with values above zero and no
    /// cycle.
    pub(super) fn validate_sankey(&self) -> Result<Vec<ChartWarning>, ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
            ("/references", !self.references.is_empty()),
            ("/timeAxis", !self.time_axis.is_default()),
            (
                "/valueAxis",
                self.value_axis != super::ValueAxisSpec::default(),
            ),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    "a Sankey diagram is drawn from links; remove this field",
                ));
            }
        }
        let Some(sankey) = &self.sankey else {
            return Err(ChartError::new(
                "missing_sankey",
                "/sankey",
                "a Sankey diagram requires a sankey block",
            ));
        };
        if sankey.links.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/sankey/links",
                "provide at least one link",
            ));
        }
        if sankey.links.len() > MAX_LINKS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/sankey/links",
                format!("at most {MAX_LINKS} links are supported"),
            ));
        }
        let mut pairs = BTreeSet::new();
        for (index, link) in sankey.links.iter().enumerate() {
            let path = format!("/sankey/links/{index}");
            validate_text(&link.from, &format!("{path}/from"), 100)?;
            validate_text(&link.to, &format!("{path}/to"), 100)?;
            validate_number(link.value, &format!("{path}/value"))?;
            if link.value <= 0.0 {
                return Err(ChartError::new(
                    "invalid_value",
                    format!("{path}/value"),
                    "the value of a link must be above zero",
                ));
            }
            if link.from == link.to {
                return Err(ChartError::new(
                    "self_link",
                    format!("{path}/to"),
                    "a link must join two different nodes",
                ));
            }
            if !pairs.insert((link.from.as_str(), link.to.as_str())) {
                return Err(ChartError::new(
                    "duplicate_link",
                    path,
                    "two links join the same nodes; add their values into one link",
                ));
            }
        }
        validate_nodes(sankey)?;
        let graph = sankey.graph();
        if graph.labels.len() > MAX_NODES {
            return Err(ChartError::new(
                "too_many_nodes",
                "/sankey/links",
                format!("at most {MAX_NODES} nodes are supported"),
            ));
        }
        if graph.ranks.is_none() {
            return Err(ChartError::new(
                "link_cycle",
                "/sankey/links",
                "the links run in a circle; a Sankey diagram flows from left to right",
            ));
        }
        for (index, node) in sankey.nodes.iter().enumerate() {
            let rank = graph.ranks.as_ref().map_or(0, |ranks| ranks[index]);
            if node.column.is_some_and(|column| rank > column) {
                return Err(ChartError::new(
                    "column_too_early",
                    format!("/sankey/nodes/{index}/column"),
                    format!(
                        "links lead into this node from the left of column {rank}; give it column {rank} or later"
                    ),
                ));
            }
        }
        Ok(Vec::new())
    }
}

/// Listed nodes: unique labels that the links name, in columns that exist.
fn validate_nodes(sankey: &SankeySpec) -> Result<(), ChartError> {
    let mut seen = BTreeSet::new();
    for (index, node) in sankey.nodes.iter().enumerate() {
        let path = format!("/sankey/nodes/{index}");
        validate_text(&node.label, &format!("{path}/label"), 100)?;
        if !seen.insert(node.label.as_str()) {
            return Err(ChartError::new(
                "duplicate_label",
                format!("{path}/label"),
                "node labels must be unique",
            ));
        }
        if !sankey
            .links
            .iter()
            .any(|link| link.from == node.label || link.to == node.label)
        {
            return Err(ChartError::new(
                "unknown_node",
                format!("{path}/label"),
                "no link names this node",
            ));
        }
        if node.color.is_some_and(|color| !(1..=4).contains(&color)) {
            return Err(ChartError::new(
                "invalid_value",
                format!("{path}/color"),
                "a node color is 1 to 4, one of the palette colors",
            ));
        }
        if node.column.is_some_and(|column| column >= MAX_COLUMNS) {
            return Err(ChartError::new(
                "invalid_value",
                format!("{path}/column"),
                format!("columns run from 0 to {}", MAX_COLUMNS - 1),
            ));
        }
    }
    Ok(())
}
