use serde::{Deserialize, Serialize};

use super::{ChartSpec, Dash, DiagramOrientation, sequence::is_identifier, validate_text};
use crate::error::{ChartError, ChartWarning};

/// Steps of one flow chart; beyond this a layered layout stops being readable at a glance.
pub(crate) const MAX_NODES: usize = 40;
pub(crate) const MAX_EDGES: usize = 80;
pub(crate) const MAX_LANES: usize = 8;
pub(crate) const MAX_GROUPS: usize = 8;

/// A flow chart: steps joined by arrows, laid out in layers along the direction of the flow,
/// optionally in lanes and framed in groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlowSpec {
    pub nodes: Vec<FlowNodeSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<FlowEdgeSpec>,
    /// Bands across the flow, one per role or system; every step then names its lane.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lanes: Vec<LaneSpec>,
    /// Frames around steps that belong together.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<GroupSpec>,
    /// The usual way through the flow, as step ids joined by edges or as the shortest way between
    /// two steps: kept in line and drawn stronger.
    #[serde(default, skip_serializing_if = "MainPath::is_empty")]
    pub main_path: MainPath,
    /// Which way the flow runs: down in portrait, right in landscape.
    #[serde(default, skip_serializing_if = "DiagramOrientation::is_auto")]
    pub orientation: DiagramOrientation,
}

/// The main path of a diagram: the ids of its steps in order, each joined to the next by an edge,
/// or the two ends of the way, which the shortest way along the edges joins.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged, deny_unknown_fields)]
pub enum MainPath {
    Steps(Vec<String>),
    Between { from: String, to: String },
}

impl Default for MainPath {
    fn default() -> Self {
        Self::Steps(Vec::new())
    }
}

impl MainPath {
    pub(crate) fn is_empty(&self) -> bool {
        matches!(self, Self::Steps(steps) if steps.is_empty())
    }

    /// The ids along the path; for `Between`, the shortest way along `edges` (pairs of ids), the
    /// first found in the order of the edges at a tie, or `None` when there is no way.
    pub(crate) fn steps<'a>(
        &'a self,
        edges: &(impl Iterator<Item = (&'a str, &'a str)> + Clone),
    ) -> Option<Vec<String>> {
        let (from, to) = match self {
            Self::Steps(steps) => return Some(steps.clone()),
            Self::Between { from, to } => (from.as_str(), to.as_str()),
        };
        let mut reached: Vec<(&str, Option<usize>)> = vec![(from, None)];
        let mut next = 0;
        while next < reached.len() && !reached.iter().any(|(id, _)| *id == to) {
            let at = reached[next].0;
            for (start, end) in edges.clone() {
                if start == at && !reached.iter().any(|(id, _)| *id == end) {
                    reached.push((end, Some(next)));
                }
            }
            next += 1;
        }
        let mut index = reached.iter().position(|(id, _)| *id == to)?;
        let mut path = vec![to.to_owned()];
        while let Some(previous) = reached[index].1 {
            path.push(reached[previous].0.to_owned());
            index = previous;
        }
        path.reverse();
        Some(path)
    }

    /// Checks the path of a diagram whose steps `known` recognizes and whose edges join the pairs
    /// of ids in `edges`; `section` is the block in the JSON path and `noun`/`edge` name a step
    /// and an edge in messages. Returns the ids along the path.
    pub(crate) fn validate<'a>(
        &'a self,
        section: &str,
        known: impl Fn(&str) -> bool,
        edges: &(impl Iterator<Item = (&'a str, &'a str)> + Clone),
        (noun, edge): (&str, &str),
    ) -> Result<Vec<String>, ChartError> {
        let unknown = |id: &str, path: String| {
            ChartError::new(
                "unknown_node",
                path,
                format!("no {noun} has the id \"{id}\""),
            )
        };
        if let Self::Between { from, to } = self {
            for (end, id) in [("from", from), ("to", to)] {
                if !known(id) {
                    return Err(unknown(id, format!("/{section}/mainPath/{end}")));
                }
            }
            return self.steps(edges).ok_or_else(|| {
                ChartError::new(
                    "main_path_unreachable",
                    format!("/{section}/mainPath"),
                    format!("no way along the {edge}s leads from \"{from}\" to \"{to}\""),
                )
            });
        }
        let steps = self.steps(edges).expect("a list of steps");
        for (index, id) in steps.iter().enumerate() {
            if !known(id) {
                return Err(unknown(id, format!("/{section}/mainPath/{index}")));
            }
        }
        for (index, pair) in steps.windows(2).enumerate() {
            if !edges
                .clone()
                .any(|(start, end)| start == pair[0] && end == pair[1])
            {
                return Err(ChartError::new(
                    "main_path_gap",
                    format!("/{section}/mainPath/{}", index + 1),
                    format!(
                        "no {edge} leads from \"{}\" to \"{}\"; the main path follows {edge}s",
                        pair[0], pair[1]
                    ),
                ));
            }
        }
        Ok(steps)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlowNodeSpec {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
    #[serde(default, skip_serializing_if = "NodeKind::is_process")]
    pub kind: NodeKind,
    /// The `id` of the lane the step belongs to; required once the chart has lanes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lane: Option<String>,
}

/// What a step is; every kind has its own shape.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    /// Where the flow begins: a rounded pill.
    Start,
    /// Where the flow ends: a pill with a strong outline.
    End,
    /// A step: a box.
    #[default]
    Process,
    /// A question with several ways on: a diamond.
    Decision,
    /// Input or output: a slanted box.
    Io,
    /// A step that is a flow of its own: a box with double sides.
    Subprocess,
    /// A data store: a cylinder.
    Store,
    /// Something outside the flow's scope: a dashed box.
    External,
}

impl NodeKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_process(&self) -> bool {
        matches!(self, Self::Process)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlowEdgeSpec {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The line's pattern; solid when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash: Option<Dash>,
    /// How much the edge carries, a number of at least 0: the line is drawn as much stronger as
    /// it carries more.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

/// Checks the weight of an edge: a finite number of at least 0.
pub(crate) fn validate_weight(weight: Option<f64>, path: &str) -> Result<(), ChartError> {
    match weight {
        Some(weight) if !weight.is_finite() || weight < 0.0 => Err(ChartError::new(
            "invalid_edge_weight",
            format!("{path}/weight"),
            "the weight of an edge is a number of at least 0",
        )),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LaneSpec {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroupSpec {
    pub label: String,
    /// The `id`s of the steps the group frames.
    pub nodes: Vec<String>,
}

impl FlowSpec {
    /// The index of the step with `id`.
    pub(crate) fn node(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }

    /// The index of each step's lane, `None` without lanes.
    pub(crate) fn lane_of(&self, node: usize) -> Option<usize> {
        let lane = self.nodes[node].lane.as_deref()?;
        self.lanes.iter().position(|candidate| candidate.id == lane)
    }

    /// The index of the group a step belongs to.
    pub(crate) fn group_of(&self, node: usize) -> Option<usize> {
        let id = &self.nodes[node].id;
        self.groups
            .iter()
            .position(|group| group.nodes.iter().any(|member| member == id))
    }

    /// Sender and receiver of every edge, as step indices.
    pub(crate) fn ends(&self) -> Vec<(usize, usize)> {
        self.edges
            .iter()
            .map(|edge| {
                (
                    self.node(&edge.from).expect("validated edges name steps"),
                    self.node(&edge.to).expect("validated edges name steps"),
                )
            })
            .collect()
    }

    /// Whether edge `index` joins two consecutive steps of the main path.
    pub(crate) fn on_main_path(&self, index: usize) -> bool {
        let edge = &self.edges[index];
        self.main_path_steps()
            .windows(2)
            .any(|pair| pair[0] == edge.from && pair[1] == edge.to)
    }

    /// The ids of the steps along the main path; empty without one.
    pub(crate) fn main_path_steps(&self) -> Vec<String> {
        self.main_path
            .steps(&self.edge_ids())
            .expect("validated main paths have a way")
    }

    fn edge_ids(&self) -> impl Iterator<Item = (&str, &str)> + Clone {
        self.edges
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
    }
}

impl ChartSpec {
    /// Whether a flow chart, an architecture diagram or a network has an edge with a weight.
    pub(crate) fn has_edge_weights(&self) -> bool {
        self.flow
            .as_ref()
            .is_some_and(|flow| flow.edges.iter().any(|edge| edge.weight.is_some()))
            || self.architecture.as_ref().is_some_and(|architecture| {
                architecture
                    .connections
                    .iter()
                    .any(|connection| connection.weight.is_some())
            })
            || self
                .network
                .as_ref()
                .is_some_and(|network| network.edges.iter().any(|edge| edge.weight.is_some()))
    }

    /// A flow chart: steps with unique identifiers, edges between them, lanes that every step
    /// names once there are any, groups of steps in one lane, and a main path along edges.
    pub(super) fn validate_flow(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a flow chart", "flow.nodes")?;
        let Some(flow) = &self.flow else {
            return Err(ChartError::new(
                "missing_flow",
                "/flow",
                "a flow chart requires a flow block",
            ));
        };
        flow.validate_lanes()?;
        flow.validate_nodes()?;
        flow.validate_edges()?;
        flow.validate_groups()?;
        flow.validate_main_path()?;
        Ok(Vec::new())
    }
}

/// An identifier that no earlier element of the same list has.
fn validate_id<'a>(
    id: &str,
    earlier: impl Iterator<Item = &'a str>,
    path: &str,
    noun: &str,
) -> Result<(), ChartError> {
    if !is_identifier(id) {
        return Err(ChartError::new(
            "invalid_id",
            format!("{path}/id"),
            "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
        ));
    }
    if earlier.into_iter().any(|other| other == id) {
        return Err(ChartError::new(
            "duplicate_id",
            format!("{path}/id"),
            format!("the id \"{id}\" is already taken by an earlier {noun}"),
        ));
    }
    Ok(())
}

impl FlowSpec {
    fn validate_lanes(&self) -> Result<(), ChartError> {
        if self.lanes.len() > MAX_LANES {
            return Err(ChartError::new(
                "too_many_lanes",
                "/flow/lanes",
                format!("at most {MAX_LANES} lanes are supported"),
            ));
        }
        for (index, lane) in self.lanes.iter().enumerate() {
            let path = format!("/flow/lanes/{index}");
            let earlier = self.lanes[..index].iter().map(|lane| lane.id.as_str());
            validate_id(&lane.id, earlier, &path, "lane")?;
            validate_text(&lane.label, &format!("{path}/label"), 40)?;
        }
        Ok(())
    }

    fn validate_nodes(&self) -> Result<(), ChartError> {
        if self.nodes.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/flow/nodes",
                "provide at least one step",
            ));
        }
        if self.nodes.len() > MAX_NODES {
            return Err(ChartError::new(
                "too_many_nodes",
                "/flow/nodes",
                format!("at most {MAX_NODES} steps are supported"),
            ));
        }
        for (index, node) in self.nodes.iter().enumerate() {
            let path = format!("/flow/nodes/{index}");
            let earlier = self.nodes[..index].iter().map(|node| node.id.as_str());
            validate_id(&node.id, earlier, &path, "step")?;
            validate_text(&node.label, &format!("{path}/label"), 60)?;
            if let Some(sublabel) = &node.sublabel {
                validate_text(sublabel, &format!("{path}/sublabel"), 60)?;
            }
            match (&node.lane, self.lanes.is_empty()) {
                (None, false) => {
                    return Err(ChartError::new(
                        "missing_lane",
                        format!("{path}/lane"),
                        "the chart has lanes, so every step names the lane it belongs to",
                    ));
                }
                (Some(_), true) => {
                    return Err(ChartError::new(
                        "unknown_lane",
                        format!("{path}/lane"),
                        "the chart declares no lanes; add them under flow.lanes",
                    ));
                }
                (Some(lane), false) if !self.lanes.iter().any(|known| &known.id == lane) => {
                    return Err(ChartError::new(
                        "unknown_lane",
                        format!("{path}/lane"),
                        format!("no lane has the id \"{lane}\""),
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn validate_edges(&self) -> Result<(), ChartError> {
        if self.edges.len() > MAX_EDGES {
            return Err(ChartError::new(
                "too_many_edges",
                "/flow/edges",
                format!("at most {MAX_EDGES} edges are supported"),
            ));
        }
        for (index, edge) in self.edges.iter().enumerate() {
            let path = format!("/flow/edges/{index}");
            for (field, id) in [("from", &edge.from), ("to", &edge.to)] {
                if self.node(id).is_none() {
                    return Err(ChartError::new(
                        "unknown_node",
                        format!("{path}/{field}"),
                        format!("no step has the id \"{id}\""),
                    ));
                }
            }
            if let Some(label) = &edge.label {
                validate_text(label, &format!("{path}/label"), 60)?;
            }
            validate_weight(edge.weight, &path)?;
        }
        Ok(())
    }

    fn validate_groups(&self) -> Result<(), ChartError> {
        if self.groups.len() > MAX_GROUPS {
            return Err(ChartError::new(
                "too_many_groups",
                "/flow/groups",
                format!("at most {MAX_GROUPS} groups are supported"),
            ));
        }
        for (index, group) in self.groups.iter().enumerate() {
            let path = format!("/flow/groups/{index}");
            validate_text(&group.label, &format!("{path}/label"), 40)?;
            if group.nodes.is_empty() {
                return Err(ChartError::new(
                    "empty_data",
                    format!("{path}/nodes"),
                    "a group frames at least one step",
                ));
            }
            for (member_index, member) in group.nodes.iter().enumerate() {
                let member_path = format!("{path}/nodes/{member_index}");
                let Some(node) = self.node(member) else {
                    return Err(ChartError::new(
                        "unknown_node",
                        member_path,
                        format!("no step has the id \"{member}\""),
                    ));
                };
                let taken = self.groups[..index]
                    .iter()
                    .any(|other| other.nodes.contains(member))
                    || group.nodes[..member_index].contains(member);
                if taken {
                    return Err(ChartError::new(
                        "duplicate_member",
                        member_path,
                        format!("the step \"{member}\" already belongs to a group"),
                    ));
                }
                // The first member was checked before every later one.
                let first = self.node(&group.nodes[0]).expect("checked above");
                if self.nodes[node].lane != self.nodes[first].lane {
                    return Err(ChartError::new(
                        "group_spans_lanes",
                        member_path,
                        "the steps of a group lie in one lane",
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_main_path(&self) -> Result<(), ChartError> {
        self.main_path
            .validate(
                "flow",
                |id| self.node(id).is_some(),
                &self.edge_ids(),
                ("step", "edge"),
            )
            .map(|_| ())
    }
}
