use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    ChartSpec, Dash, DiagramOrientation, NoteSpec, flow::validate_notes, sequence::is_identifier,
    validate_text,
};
use crate::error::{ChartError, ChartWarning};

/// Classes and relations of one diagram; beyond this a layered layout stops being readable.
pub(crate) const MAX_CLASSES: usize = 40;
pub(crate) const MAX_RELATIONS: usize = 100;
/// The members in one compartment of a class.
const MAX_MEMBERS: usize = 30;
/// Packages of one diagram, and how deeply they may nest inside each other.
pub(crate) const MAX_PACKAGES: usize = 12;
pub(crate) const MAX_PACKAGE_DEPTH: usize = 4;

/// A class diagram or entity-relationship diagram: boxes with an attribute and a method
/// compartment, joined by relations with their cardinalities, laid out like a flow chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClassesSpec {
    pub classes: Vec<ClassSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<RelationSpec>,
    /// Frames that group classes into modules, such as ordering or payment.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<PackageSpec>,
    /// Notes beside classes, such as an assumption or a limitation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<NoteSpec>,
    #[serde(default, skip_serializing_if = "DiagramOrientation::is_auto")]
    pub orientation: DiagramOrientation,
}

/// A frame around classes that belong together. Packages nest by naming the package they lie
/// `in`; relations still join classes, never packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSpec {
    pub id: String,
    pub label: String,
    #[serde(default, rename = "in", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
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
    /// The `id` of the package the class lies in.
    #[serde(default, rename = "in", skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
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

    pub(crate) fn package(&self, id: &str) -> Option<usize> {
        self.packages.iter().position(|package| package.id == id)
    }

    /// The packages around package `index`, outermost first, ending with it; `None` when they
    /// run in a circle.
    pub(crate) fn chain(&self, index: usize) -> Option<Vec<usize>> {
        let mut chain = vec![index];
        let mut at = index;
        while let Some(parent) = self.packages[at].parent.as_deref() {
            at = self.package(parent)?;
            if chain.contains(&at) {
                return None;
            }
            chain.push(at);
        }
        chain.reverse();
        Some(chain)
    }

    /// The packages around class `index`, outermost first; empty outside every package.
    pub(crate) fn packages_of(&self, index: usize) -> Vec<usize> {
        self.classes[index]
            .package
            .as_deref()
            .and_then(|id| self.package(id))
            .and_then(|package| self.chain(package))
            .unwrap_or_default()
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
        classes.validate_packages()?;
        classes.validate_classes()?;
        classes.validate_relations()?;
        validate_notes(
            &classes.notes,
            "classes",
            |id| classes.class(id).is_some(),
            "class",
        )?;
        Ok(Vec::new())
    }
}

impl ClassesSpec {
    /// Packages with unique ids that nest in existing packages, without a circle and at most
    /// [`MAX_PACKAGE_DEPTH`] deep.
    fn validate_packages(&self) -> Result<(), ChartError> {
        if self.packages.len() > MAX_PACKAGES {
            return Err(ChartError::new(
                "too_many_packages",
                "/classes/packages",
                format!("at most {MAX_PACKAGES} packages are supported"),
            ));
        }
        let mut ids = BTreeSet::new();
        for (index, package) in self.packages.iter().enumerate() {
            let path = format!("/classes/packages/{index}");
            if !is_identifier(&package.id) {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/id"),
                    "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            if !ids.insert(package.id.as_str()) {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    format!(
                        "the id \"{}\" is already taken by an earlier package",
                        package.id
                    ),
                ));
            }
            validate_text(&package.label, &format!("{path}/label"), 60)?;
        }
        for (index, package) in self.packages.iter().enumerate() {
            let path = format!("/classes/packages/{index}/in");
            if let Some(parent) = &package.parent
                && self.package(parent).is_none()
            {
                return Err(ChartError::new(
                    "unknown_package",
                    path,
                    format!("no package has the id \"{parent}\""),
                ));
            }
            let Some(chain) = self.chain(index) else {
                return Err(ChartError::new(
                    "package_cycle",
                    path,
                    "packages lie in each other in a circle",
                ));
            };
            if chain.len() > MAX_PACKAGE_DEPTH {
                return Err(ChartError::new(
                    "packages_too_deep",
                    path,
                    format!("packages nest at most {MAX_PACKAGE_DEPTH} deep"),
                ));
            }
        }
        Ok(())
    }

    fn validate_classes(&self) -> Result<(), ChartError> {
        let mut ids: BTreeSet<&str> = self
            .packages
            .iter()
            .map(|package| package.id.as_str())
            .collect();
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
                        "the id \"{}\" is already taken by a package or an earlier class",
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
            if let Some(package) = &class.package
                && self.package(package).is_none()
            {
                return Err(ChartError::new(
                    "unknown_package",
                    format!("{path}/in"),
                    format!("no package has the id \"{package}\""),
                ));
            }
        }
        for (index, package) in self.packages.iter().enumerate() {
            let holds =
                (0..self.classes.len()).any(|class| self.packages_of(class).contains(&index));
            if !holds {
                return Err(ChartError::new(
                    "empty_package",
                    format!("/classes/packages/{index}"),
                    format!("no class lies in \"{}\"", package.id),
                ));
            }
        }
        Ok(())
    }

    fn validate_relations(&self) -> Result<(), ChartError> {
        for (index, relation) in self.relations.iter().enumerate() {
            let path = format!("/classes/relations/{index}");
            for (name, id) in [("from", &relation.from), ("to", &relation.to)] {
                if self.class(id).is_none() {
                    let message = if self.package(id).is_some() {
                        format!("\"{id}\" is a package; relations join classes")
                    } else {
                        format!("no class has the id \"{id}\"")
                    };
                    return Err(ChartError::new(
                        "unknown_node",
                        format!("{path}/{name}"),
                        message,
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
