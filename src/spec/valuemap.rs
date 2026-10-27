use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many components a value chain map may have, and how many dependencies.
const MAX_COMPONENTS: usize = 40;
const MAX_DEPENDENCIES: usize = 80;

/// The stages of evolution and where each begins on the evolution axis, from 0 to 1.
pub(crate) const VALUEMAP_STAGES: [(f64, ValuemapStage); 4] = [
    (0.0, ValuemapStage::Genesis),
    (0.17, ValuemapStage::Custom),
    (0.4, ValuemapStage::Product),
    (0.7, ValuemapStage::Commodity),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValuemapStage {
    Genesis,
    Custom,
    Product,
    Commodity,
}

/// A value chain map: components placed by how visible they are to the user (up) and how far
/// they have evolved (right), with the dependencies between them and where each is moving.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValuemapSpec {
    pub components: Vec<ValuemapComponentSpec>,
    /// `from` needs `to`; drawn as a line from the one to the other.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<ValuemapDependencySpec>,
}

/// A component and its place on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValuemapComponentSpec {
    pub id: String,
    pub label: String,
    /// How visible it is to the user, from 0 (hidden) to 1 (the user's need itself).
    pub visibility: f64,
    /// How far it has evolved, from 0 (genesis) to 1 (commodity).
    pub evolution: f64,
    /// Where it is expected to move, drawn as a dashed arrow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moves_to: Option<ValuemapMoveSpec>,
    /// Whether something holds it back from evolving; drawn as a bar across its path.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub inertia: bool,
}

/// The place a component is moving to; the visibility stays unless it is given.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValuemapMoveSpec {
    pub evolution: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValuemapDependencySpec {
    pub from: String,
    pub to: String,
}

impl ValuemapStage {
    /// The stage a position on the evolution axis lies in.
    pub(crate) fn of(evolution: f64) -> Self {
        VALUEMAP_STAGES
            .iter()
            .rev()
            .find(|(start, _)| evolution >= *start)
            .map_or(Self::Genesis, |(_, stage)| *stage)
    }
}

impl ValuemapSpec {
    pub(crate) fn index_of(&self, id: &str) -> Option<usize> {
        self.components
            .iter()
            .position(|component| component.id == id)
    }
}

impl ChartSpec {
    /// Components with unique ids and positions between 0 and 1, dependencies between components,
    /// and movements that stay on the map.
    #[allow(clippy::too_many_lines)]
    pub(super) fn validate_valuemap(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_chart_fields("a value chain map is drawn from its components")?;
        let Some(map) = &self.valuemap else {
            return Err(ChartError::new(
                "missing_valuemap",
                "/valuemap",
                "a value chain map requires a valuemap block",
            ));
        };
        if map.components.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/valuemap/components",
                "provide at least one component",
            ));
        }
        if map.components.len() > MAX_COMPONENTS {
            return Err(ChartError::new(
                "too_many_nodes",
                "/valuemap/components",
                format!(
                    "at most {MAX_COMPONENTS} components are supported; map one part of the chain"
                ),
            ));
        }
        if map.dependencies.len() > MAX_DEPENDENCIES {
            return Err(ChartError::new(
                "too_many_edges",
                "/valuemap/dependencies",
                format!("at most {MAX_DEPENDENCIES} dependencies are supported"),
            ));
        }
        let mut ids = BTreeSet::new();
        let mut labels = BTreeSet::new();
        for (index, component) in map.components.iter().enumerate() {
            let path = format!("/valuemap/components/{index}");
            validate_text(&component.id, &format!("{path}/id"), 60)?;
            if !ids.insert(component.id.as_str()) {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    "component ids must be unique",
                ));
            }
            validate_text(&component.label, &format!("{path}/label"), 100)?;
            if !labels.insert(component.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "component labels must be unique",
                ));
            }
            let mut positions = vec![
                ("visibility", component.visibility),
                ("evolution", component.evolution),
            ];
            if let Some(target) = &component.moves_to {
                positions.push(("movesTo/evolution", target.evolution));
                if let Some(visibility) = target.visibility {
                    positions.push(("movesTo/visibility", visibility));
                }
            }
            for (name, value) in positions {
                validate_number(value, &format!("{path}/{name}"))?;
                if !(0.0..=1.0).contains(&value) {
                    return Err(ChartError::new(
                        "invalid_position",
                        format!("{path}/{name}"),
                        "positions are between 0 and 1",
                    ));
                }
            }
            if let Some(target) = &component.moves_to
                && target.evolution <= component.evolution
            {
                return Err(ChartError::new(
                    "invalid_position",
                    format!("{path}/movesTo"),
                    "a movement goes to a place with more evolution than the component's own",
                ));
            }
        }
        let mut pairs = BTreeSet::new();
        for (index, dependency) in map.dependencies.iter().enumerate() {
            let path = format!("/valuemap/dependencies/{index}");
            for (name, id) in [("from", &dependency.from), ("to", &dependency.to)] {
                if !ids.contains(id.as_str()) {
                    return Err(ChartError::new(
                        "unknown_node",
                        format!("{path}/{name}"),
                        "a dependency joins components by their ids",
                    ));
                }
            }
            if dependency.from == dependency.to {
                return Err(ChartError::new(
                    "self_loop",
                    &path,
                    "a component does not depend on itself",
                ));
            }
            if !pairs.insert((dependency.from.as_str(), dependency.to.as_str())) {
                return Err(ChartError::new(
                    "duplicate_edge",
                    &path,
                    "a dependency is given once",
                ));
            }
        }
        Ok(Vec::new())
    }
}
