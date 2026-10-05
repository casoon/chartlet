use serde::{Deserialize, Serialize};

use super::{ChartSpec, DiagramOrientation, sequence::is_identifier, validate_text};
use crate::error::{ChartError, ChartWarning};

/// Nodes of one tree, and how deep the tree may go.
pub(crate) const MAX_NODES: usize = 150;
pub(crate) const MAX_DEPTH: usize = 12;

/// A tree: one root and, below it, nodes that each name their parent; the children of a node
/// keep the order of the list. Organization charts, ownership structures, hierarchies of norms
/// or components.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TreeSpec {
    pub nodes: Vec<TreeNodeSpec>,
    /// `portrait` runs down from the root, `landscape` to the right; `auto` takes the one that
    /// grows the canvas less.
    #[serde(default, skip_serializing_if = "DiagramOrientation::is_auto")]
    pub orientation: DiagramOrientation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TreeNodeSpec {
    pub id: String,
    pub label: String,
    /// A second line, such as a role or a legal form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
    /// The `id` of the node above; absent on the root, required on every other node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// What joins the node to its parent, such as a share of `60 %`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

impl TreeSpec {
    pub(crate) fn node(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }

    /// The index of the root: the only node without a parent.
    pub(crate) fn root(&self) -> usize {
        self.nodes
            .iter()
            .position(|node| node.parent.is_none())
            .expect("a validated tree has a root")
    }

    /// The index of `node`'s parent, `None` on the root.
    pub(crate) fn parent_of(&self, node: usize) -> Option<usize> {
        self.nodes[node]
            .parent
            .as_deref()
            .map(|id| self.node(id).expect("validated"))
    }

    /// The children of `node`, in the order of the list.
    pub(crate) fn children(&self, node: usize) -> Vec<usize> {
        let id = self.nodes[node].id.as_str();
        (0..self.nodes.len())
            .filter(|child| self.nodes[*child].parent.as_deref() == Some(id))
            .collect()
    }

    /// How many nodes lie between `node` and the root.
    pub(crate) fn depth(&self, node: usize) -> usize {
        let mut depth = 0;
        let mut at = node;
        while let Some(parent) = self.parent_of(at) {
            depth += 1;
            at = parent;
        }
        depth
    }

    /// The nodes in reading order: each before its children, children in list order.
    pub(crate) fn reading_order(&self) -> Vec<usize> {
        let mut order = Vec::with_capacity(self.nodes.len());
        let mut stack = vec![self.root()];
        while let Some(node) = stack.pop() {
            order.push(node);
            stack.extend(self.children(node).into_iter().rev());
        }
        order
    }
}

impl ChartSpec {
    /// A tree: unique identifiers, exactly one root, parents that exist, no circle, and a depth
    /// and size within the limits.
    pub(super) fn validate_tree(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a tree", "tree.nodes")?;
        let Some(tree) = &self.tree else {
            return Err(ChartError::new(
                "missing_tree",
                "/tree",
                "a tree requires a tree block",
            ));
        };
        if tree.nodes.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/tree/nodes",
                "provide at least one node",
            ));
        }
        if tree.nodes.len() > MAX_NODES {
            return Err(ChartError::new(
                "too_many_nodes",
                "/tree/nodes",
                format!("at most {MAX_NODES} nodes are supported"),
            ));
        }
        tree.validate_nodes()?;
        tree.validate_structure()?;
        Ok(Vec::new())
    }
}

impl TreeSpec {
    /// Every node on its own: identifier, texts, link and a parent that exists.
    fn validate_nodes(&self) -> Result<(), ChartError> {
        let tree = self;
        for (index, node) in tree.nodes.iter().enumerate() {
            let path = format!("/tree/nodes/{index}");
            if !is_identifier(&node.id) {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/id"),
                    "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            if tree.nodes[..index].iter().any(|other| other.id == node.id) {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    format!("the id \"{}\" is already taken by an earlier node", node.id),
                ));
            }
            validate_text(&node.label, &format!("{path}/label"), 60)?;
            if let Some(sublabel) = &node.sublabel {
                validate_text(sublabel, &format!("{path}/sublabel"), 60)?;
            }
            if let Some(link) = &node.link {
                validate_text(link, &format!("{path}/link"), 40)?;
                if node.parent.is_none() {
                    return Err(ChartError::new(
                        "option_not_supported",
                        format!("{path}/link"),
                        "the root has no parent to be joined to; remove the link",
                    ));
                }
            }
            if let Some(parent) = &node.parent
                && tree.node(parent).is_none()
            {
                return Err(ChartError::new(
                    "unknown_node",
                    format!("{path}/parent"),
                    format!("no node has the id \"{parent}\""),
                ));
            }
        }
        Ok(())
    }

    /// The nodes together: exactly one root, parents that run up to it, and a depth within the
    /// limit.
    fn validate_structure(&self) -> Result<(), ChartError> {
        let tree = self;
        let roots: Vec<usize> = (0..tree.nodes.len())
            .filter(|index| tree.nodes[*index].parent.is_none())
            .collect();
        if roots.len() != 1 {
            let (path, message) = match roots.get(1) {
                Some(second) => (
                    format!("/tree/nodes/{second}"),
                    "a tree has exactly one root; give this node a parent".to_owned(),
                ),
                None => (
                    "/tree/nodes".to_owned(),
                    "a tree has exactly one root: leave the parent off one node".to_owned(),
                ),
            };
            return Err(ChartError::new("invalid_root", path, message));
        }
        for index in 0..tree.nodes.len() {
            // Walking up from every node ends at the root, or runs in a circle.
            let mut at = index;
            let mut steps = 0;
            while let Some(parent) = tree.parent_of(at) {
                at = parent;
                steps += 1;
                if steps > tree.nodes.len() {
                    return Err(ChartError::new(
                        "circular_parent",
                        format!("/tree/nodes/{index}/parent"),
                        "the parents of these nodes run in a circle",
                    ));
                }
            }
            if steps > MAX_DEPTH {
                return Err(ChartError::new(
                    "tree_too_deep",
                    format!("/tree/nodes/{index}/parent"),
                    format!("at most {MAX_DEPTH} levels below the root are supported"),
                ));
            }
        }
        Ok(())
    }
}
