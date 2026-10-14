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
    /// `layered` (default) gives every level a row; `radial` puts the root in the middle and
    /// every level on a ring around it, as in a mind map.
    #[serde(default, skip_serializing_if = "TreeLayout::is_layered")]
    pub layout: TreeLayout,
    /// What the `value` of the nodes stands for, such as `Budget (m €)`: the name of the color
    /// key and of the table column.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_label: Option<String>,
}

/// How a tree is laid out.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TreeLayout {
    /// One row per level (a column in landscape), parents centered over their children.
    #[default]
    Layered,
    /// The root in the middle, every level on a ring around it, and each subtree in a sector of
    /// its own, as wide as its share of the leaves: a mind map.
    Radial,
}

impl TreeLayout {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_layered(&self) -> bool {
        matches!(self, Self::Layered)
    }
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
    /// What the node is; every kind has its own shape.
    #[serde(default, skip_serializing_if = "TreeNodeKind::is_unit")]
    pub kind: TreeNodeKind,
    /// The `id` of the node this one is joined to as a couple: the two stand side by side with a
    /// line between them, and the children of either hang from the middle of that line. A
    /// partner has no `parent` of its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner: Option<String>,
    /// Draws the node without what hangs below it (and below its partner): a small `+n` says how
    /// many nodes are left out. The description and the data table still list all of them.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub collapsed: bool,
    /// A number that colors the node's fill, light for the smallest value of the tree and dark
    /// for the largest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// The parts of what the node holds, drawn as a stripe inside the bottom of its box.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<super::PartSpec>,
}

/// What a node is.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TreeNodeKind {
    /// A unit, a company, a norm: a box.
    #[default]
    Unit,
    /// A person: a box with a head on top.
    Person,
    /// Something outside the scope: a dashed box.
    External,
}

impl TreeNodeKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_unit(&self) -> bool {
        matches!(self, Self::Unit)
    }
}

impl TreeSpec {
    pub(crate) fn node(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }

    /// The node a partner is joined to, `None` on every other node.
    pub(crate) fn head_of(&self, node: usize) -> Option<usize> {
        self.nodes[node]
            .partner
            .as_deref()
            .map(|id| self.node(id).expect("validated"))
    }

    /// The partner joined to `node`, if one is.
    pub(crate) fn partner_of(&self, node: usize) -> Option<usize> {
        let id = self.nodes[node].id.as_str();
        (0..self.nodes.len()).find(|other| self.nodes[*other].partner.as_deref() == Some(id))
    }

    /// The index of the root: the only node with neither a parent nor a partner.
    pub(crate) fn root(&self) -> usize {
        (0..self.nodes.len())
            .find(|node| self.nodes[*node].parent.is_none() && self.nodes[*node].partner.is_none())
            .expect("a validated tree has a root")
    }

    /// The node above `node`, `None` on the root and on a partner. A child of a partner hangs
    /// from the node the partner is joined to.
    pub(crate) fn parent_of(&self, node: usize) -> Option<usize> {
        let parent = self.node(self.nodes[node].parent.as_deref()?)?;
        Some(self.head_of(parent).unwrap_or(parent))
    }

    /// The children of `node` and of its partner, in the order of the list.
    pub(crate) fn children(&self, node: usize) -> Vec<usize> {
        (0..self.nodes.len())
            .filter(|child| self.parent_of(*child) == Some(node))
            .collect()
    }

    /// How many nodes lie between `node` and the root; a partner lies as deep as its head.
    pub(crate) fn depth(&self, node: usize) -> usize {
        let mut depth = 0;
        let mut at = self.head_of(node).unwrap_or(node);
        while let Some(parent) = self.parent_of(at) {
            depth += 1;
            at = parent;
        }
        depth
    }

    /// The nodes below `node` and its partner, partners included.
    fn below(&self, node: usize) -> Vec<usize> {
        let head = self.head_of(node).unwrap_or(node);
        let mut found = Vec::new();
        let mut stack = self.children(head);
        while let Some(next) = stack.pop() {
            if found.contains(&next) {
                continue;
            }
            found.push(next);
            if let Some(partner) = self.partner_of(next) {
                found.push(partner);
            }
            stack.extend(self.children(next));
        }
        found
    }

    /// How many nodes a collapsed `node` leaves out of the drawing.
    pub(crate) fn hidden_below(&self, node: usize) -> usize {
        if self.nodes[node].collapsed {
            self.below(node).len()
        } else {
            0
        }
    }

    /// The tree as drawn when nodes are collapsed: without the nodes below them, each collapsed
    /// node saying how many it leaves out, and the index in this tree of every node that
    /// remains. `None` when nothing is collapsed.
    pub(crate) fn collapsed_view(&self) -> Option<(TreeSpec, Vec<usize>)> {
        if !self.nodes.iter().any(|node| node.collapsed) {
            return None;
        }
        let mut hidden = vec![false; self.nodes.len()];
        for node in 0..self.nodes.len() {
            if self.nodes[node].collapsed && !hidden[node] {
                for below in self.below(node) {
                    hidden[below] = true;
                }
            }
        }
        let kept: Vec<usize> = (0..self.nodes.len())
            .filter(|node| !hidden[*node])
            .collect();
        let nodes = kept
            .iter()
            .map(|&node| {
                let mut copy = self.nodes[node].clone();
                let count = self.hidden_below(node);
                if count > 0 {
                    copy.sublabel = Some(match &copy.sublabel {
                        Some(sublabel) => format!("{sublabel} · +{count}"),
                        None => format!("+{count}"),
                    });
                }
                copy
            })
            .collect();
        Some((
            TreeSpec {
                nodes,
                orientation: self.orientation,
                layout: self.layout,
                value_label: self.value_label.clone(),
            },
            kept,
        ))
    }

    /// The nodes in reading order: each before its partner and its children, children in list
    /// order.
    pub(crate) fn reading_order(&self) -> Vec<usize> {
        let mut order = Vec::with_capacity(self.nodes.len());
        let mut stack = vec![self.root()];
        while let Some(node) = stack.pop() {
            order.push(node);
            order.extend(self.partner_of(node));
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
        if let Some(label) = &tree.value_label {
            validate_text(label, "/tree/valueLabel", 40)?;
        }
        super::flow::validate_parts(
            tree.nodes
                .iter()
                .enumerate()
                .map(|(index, node)| (node.parts.as_slice(), format!("/tree/nodes/{index}/parts"))),
        )?;
        if tree.layout == TreeLayout::Radial {
            tree.validate_radial()?;
        }
        let mut warnings = Vec::new();
        for (index, node) in tree.nodes.iter().enumerate() {
            if node.collapsed && tree.below(index).is_empty() {
                warnings.push(ChartWarning::new(
                    "nothing_to_collapse",
                    format!("/tree/nodes/{index}/collapsed"),
                    "nothing hangs below this node, so collapsing it changes nothing",
                ));
            }
        }
        Ok(warnings)
    }
}

impl TreeSpec {
    /// A radial tree has no direction to choose, no room beside a node for its partner and no
    /// stretch of line along which a link could be written.
    fn validate_radial(&self) -> Result<(), ChartError> {
        if !self.orientation.is_auto() {
            return Err(ChartError::new(
                "option_not_supported",
                "/tree/orientation",
                "a radial tree grows outward in every direction; remove orientation or use the layered layout",
            ));
        }
        for (index, node) in self.nodes.iter().enumerate() {
            let refuse = |field: &str, message: &str| {
                Err(ChartError::new(
                    "option_not_supported",
                    format!("/tree/nodes/{index}/{field}"),
                    message.to_owned(),
                ))
            };
            if node.partner.is_some() {
                return refuse(
                    "partner",
                    "a radial tree has no room for couples side by side; use the layered layout",
                );
            }
            if node.link.is_some() {
                return refuse(
                    "link",
                    "a radial tree writes no labels on its spokes; put the text in a sublabel or use the layered layout",
                );
            }
        }
        Ok(())
    }

    /// A partner exists, is another node, is not itself joined to a partner or named by two
    /// nodes, and does not hang from a parent.
    fn validate_partner(&self, index: usize, partner: &str, path: &str) -> Result<(), ChartError> {
        let node = &self.nodes[index];
        let invalid = |message: String| {
            Err(ChartError::new(
                "invalid_partner",
                format!("{path}/partner"),
                message,
            ))
        };
        let Some(other) = self.node(partner) else {
            return Err(ChartError::new(
                "unknown_node",
                format!("{path}/partner"),
                format!("no node has the id \"{partner}\""),
            ));
        };
        if other == index {
            return invalid("a node cannot be its own partner".to_owned());
        }
        if node.parent.is_some() {
            return invalid("a partner stands beside its partner, not below a parent".to_owned());
        }
        if self.nodes[other].partner.is_some() {
            return invalid(format!(
                "\"{partner}\" is itself a partner; a couple has one head and one partner"
            ));
        }
        if self
            .nodes
            .iter()
            .enumerate()
            .any(|(at, other)| at != index && other.partner.as_deref() == Some(partner))
        {
            return invalid(format!(
                "another node is already the partner of \"{partner}\""
            ));
        }
        Ok(())
    }

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
            if let Some(partner) = &node.partner {
                tree.validate_partner(index, partner, &path)?;
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
            .filter(|index| {
                tree.nodes[*index].parent.is_none() && tree.nodes[*index].partner.is_none()
            })
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
