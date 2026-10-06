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
}

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
}

impl SankeySpec {
    pub(crate) fn graph(&self) -> SankeyGraph {
        let mut labels: Vec<String> = Vec::new();
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
        let mut ranks = vec![0; labels.len()];
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
        Ok(Vec::new())
    }
}
