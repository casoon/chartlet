use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, Dash, DiagramOrientation, sequence::is_identifier, validate_text};
use crate::error::{ChartError, ChartWarning};

/// Classes and relations of one diagram; beyond this a layered layout stops being readable.
pub(crate) const MAX_CLASSES: usize = 40;
pub(crate) const MAX_RELATIONS: usize = 100;
/// The members in one compartment of a class.
const MAX_MEMBERS: usize = 30;

/// A class diagram or entity-relationship diagram: boxes with an attribute and a method
/// compartment, joined by relations with their cardinalities, laid out like a flow chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClassesSpec {
    pub classes: Vec<ClassSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<RelationSpec>,
    #[serde(default, skip_serializing_if = "DiagramOrientation::is_auto")]
    pub orientation: DiagramOrientation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClassSpec {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "ClassKind::is_class")]
    pub kind: ClassKind,
    /// The attributes, one line each, such as `+ id: Int` or, in an entity, `id (PK)`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<String>,
    /// The methods, one line each, such as `+ total(): Money`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<String>,
}

/// What a class is: it decides the stereotype written above its name.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClassKind {
    #[default]
    Class,
    Interface,
    Abstract,
    Enum,
    /// An entity of an entity-relationship diagram: no stereotype.
    Entity,
}

impl ClassKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_class(&self) -> bool {
        matches!(self, Self::Class)
    }

    /// The stereotype written above the name, if the kind has one.
    pub(crate) const fn stereotype(self) -> Option<&'static str> {
        match self {
            Self::Class | Self::Entity => None,
            Self::Interface => Some("«interface»"),
            Self::Abstract => Some("«abstract»"),
            Self::Enum => Some("«enumeration»"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationSpec {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "RelationKind::is_association")]
    pub kind: RelationKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// How many of the class at either end take part, such as `1`, `0..1`, `0..*`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_cardinality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_cardinality: Option<String>,
}

/// How two classes are related; each kind has its own marks at the line's ends.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RelationKind {
    /// A plain line.
    #[default]
    Association,
    /// `from` inherits from `to`: a hollow triangle at `to`.
    Inheritance,
    /// `from` implements `to`: a dashed line with a hollow triangle at `to`.
    Realization,
    /// `from` is made of `to`: a filled diamond at `from`.
    Composition,
    /// `from` holds `to`: a hollow diamond at `from`.
    Aggregation,
    /// `from` depends on `to`: a dashed line with an open arrowhead at `to`.
    Dependency,
}

impl RelationKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_association(&self) -> bool {
        matches!(self, Self::Association)
    }

    pub(crate) const fn dash(self) -> Option<Dash> {
        match self {
            Self::Realization | Self::Dependency => Some(Dash::Dashed),
            _ => None,
        }
    }
}

impl ClassesSpec {
    pub(crate) fn class(&self, id: &str) -> Option<usize> {
        self.classes.iter().position(|class| class.id == id)
    }
}

impl ChartSpec {
    /// Classes with unique ids and bounded members, and relations between existing classes.
    pub(super) fn validate_classes(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a class diagram", "classes.classes")?;
        let Some(classes) = &self.classes else {
            return Err(ChartError::new(
                "missing_classes",
                "/classes",
                "a class diagram requires a classes block",
            ));
        };
        if classes.classes.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/classes/classes",
                "provide at least one class",
            ));
        }
        if classes.classes.len() > MAX_CLASSES {
            return Err(ChartError::new(
                "too_many_nodes",
                "/classes/classes",
                format!("at most {MAX_CLASSES} classes are supported"),
            ));
        }
        if classes.relations.len() > MAX_RELATIONS {
            return Err(ChartError::new(
                "too_many_edges",
                "/classes/relations",
                format!("at most {MAX_RELATIONS} relations are supported"),
            ));
        }
        classes.validate_classes()?;
        classes.validate_relations()?;
        Ok(Vec::new())
    }
}

impl ClassesSpec {
    fn validate_classes(&self) -> Result<(), ChartError> {
        let mut ids = BTreeSet::new();
        for (index, class) in self.classes.iter().enumerate() {
            let path = format!("/classes/classes/{index}");
            if !is_identifier(&class.id) {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/id"),
                    "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            if !ids.insert(class.id.as_str()) {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    format!(
                        "the id \"{}\" is already taken by an earlier class",
                        class.id
                    ),
                ));
            }
            validate_text(&class.label, &format!("{path}/label"), 60)?;
            for (name, members) in [
                ("attributes", &class.attributes),
                ("methods", &class.methods),
            ] {
                if members.len() > MAX_MEMBERS {
                    return Err(ChartError::new(
                        "too_many_data_points",
                        format!("{path}/{name}"),
                        format!("at most {MAX_MEMBERS} {name} are supported"),
                    ));
                }
                for (at, member) in members.iter().enumerate() {
                    validate_text(member, &format!("{path}/{name}/{at}"), 100)?;
                }
            }
        }
        Ok(())
    }

    fn validate_relations(&self) -> Result<(), ChartError> {
        for (index, relation) in self.relations.iter().enumerate() {
            let path = format!("/classes/relations/{index}");
            for (name, id) in [("from", &relation.from), ("to", &relation.to)] {
                if self.class(id).is_none() {
                    return Err(ChartError::new(
                        "unknown_node",
                        format!("{path}/{name}"),
                        format!("no class has the id \"{id}\""),
                    ));
                }
            }
            if let Some(label) = &relation.label {
                validate_text(label, &format!("{path}/label"), 60)?;
            }
            for (name, cardinality) in [
                ("fromCardinality", &relation.from_cardinality),
                ("toCardinality", &relation.to_cardinality),
            ] {
                if let Some(cardinality) = cardinality {
                    validate_text(cardinality, &format!("{path}/{name}"), 12)?;
                }
            }
        }
        Ok(())
    }
}
