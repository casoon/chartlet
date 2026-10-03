use serde::{Deserialize, Serialize};

use super::{ChartSpec, Dash, DiagramOrientation, sequence::is_identifier, validate_text};
use crate::error::{ChartError, ChartWarning};

/// Components of one diagram, connections between them, and boundaries around them.
pub(crate) const MAX_COMPONENTS: usize = 40;
pub(crate) const MAX_CONNECTIONS: usize = 80;
pub(crate) const MAX_BOUNDARIES: usize = 12;
/// How deeply boundaries may nest inside each other.
pub(crate) const MAX_BOUNDARY_DEPTH: usize = 4;

/// An architecture diagram: components joined by connections, inside nested boundaries such as
/// a cloud region, a network or a system, laid out like a flow chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchitectureSpec {
    pub components: Vec<ComponentSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<ConnectionSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundaries: Vec<BoundarySpec>,
    /// The usual way a request takes, as component ids joined by connections: kept in line and
    /// drawn stronger.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub main_path: Vec<String>,
    #[serde(default, skip_serializing_if = "DiagramOrientation::is_auto")]
    pub orientation: DiagramOrientation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComponentSpec {
    pub id: String,
    pub label: String,
    /// A second line, such as the technology or the host.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
    #[serde(default, skip_serializing_if = "ComponentKind::is_service")]
    pub kind: ComponentKind,
    /// The `id` of the boundary the component lies in.
    #[serde(default, rename = "in", skip_serializing_if = "Option::is_none")]
    pub boundary: Option<String>,
}

/// What a component is; every kind has its own shape.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ComponentKind {
    /// Someone who uses the system: a box with a head on top.
    Person,
    /// What a person sees, such as a web or mobile app: a box with a window bar.
    Frontend,
    /// A service, an application or a function: a box.
    #[default]
    Service,
    /// A database: a cylinder.
    Database,
    /// A queue, a topic or an event stream: a box with a stack behind it.
    Queue,
    /// File or object storage: a bucket.
    Storage,
    /// A cache: a hexagon.
    Cache,
    /// A system outside the scope: a dashed box.
    External,
}

impl ComponentKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_service(&self) -> bool {
        matches!(self, Self::Service)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConnectionSpec {
    pub from: String,
    pub to: String,
    /// What the connection does, such as `reads orders`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// How it does it, such as `HTTPS` or `SQL/TLS`; written in brackets under the label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash: Option<Dash>,
}

/// A frame around components, such as a cloud region, a network, a zone or a system. Boundaries
/// nest by naming the boundary they lie `in`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoundarySpec {
    pub id: String,
    pub label: String,
    #[serde(default, rename = "in", skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

impl ArchitectureSpec {
    pub(crate) fn component(&self, id: &str) -> Option<usize> {
        self.components
            .iter()
            .position(|component| component.id == id)
    }

    pub(crate) fn boundary(&self, id: &str) -> Option<usize> {
        self.boundaries
            .iter()
            .position(|boundary| boundary.id == id)
    }

    /// Whether connection `index` joins two consecutive components of the main path.
    pub(crate) fn on_main_path(&self, index: usize) -> bool {
        let connection = &self.connections[index];
        self.main_path
            .windows(2)
            .any(|pair| pair[0] == connection.from && pair[1] == connection.to)
    }

    /// The boundaries around boundary `index`, outermost first, ending with it; `None` when
    /// they run in a circle.
    pub(crate) fn chain(&self, index: usize) -> Option<Vec<usize>> {
        let mut chain = vec![index];
        let mut at = index;
        while let Some(parent) = self.boundaries[at].parent.as_deref() {
            at = self.boundary(parent)?;
            if chain.contains(&at) {
                return None;
            }
            chain.push(at);
        }
        chain.reverse();
        Some(chain)
    }
}

impl ChartSpec {
    /// An architecture diagram: components and boundaries with identifiers unique among both,
    /// connections between components, boundaries that nest without a circle and each hold at
    /// least one component, and a main path along connections.
    pub(super) fn validate_architecture(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("an architecture diagram", "architecture.components")?;
        let Some(architecture) = &self.architecture else {
            return Err(ChartError::new(
                "missing_architecture",
                "/architecture",
                "an architecture diagram requires an architecture block",
            ));
        };
        architecture.validate_boundaries()?;
        architecture.validate_components()?;
        architecture.validate_connections()?;
        Ok(Vec::new())
    }
}

/// An identifier: well formed, and not taken by any of `earlier`.
fn validate_id<'a>(
    id: &str,
    mut earlier: impl Iterator<Item = &'a str>,
    path: &str,
) -> Result<(), ChartError> {
    if !is_identifier(id) {
        return Err(ChartError::new(
            "invalid_id",
            format!("{path}/id"),
            "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
        ));
    }
    if earlier.any(|other| other == id) {
        return Err(ChartError::new(
            "duplicate_id",
            format!("{path}/id"),
            format!("the id \"{id}\" is already taken by an earlier component or boundary"),
        ));
    }
    Ok(())
}

impl ArchitectureSpec {
    fn validate_boundaries(&self) -> Result<(), ChartError> {
        if self.boundaries.len() > MAX_BOUNDARIES {
            return Err(ChartError::new(
                "too_many_boundaries",
                "/architecture/boundaries",
                format!("at most {MAX_BOUNDARIES} boundaries are supported"),
            ));
        }
        for (index, boundary) in self.boundaries.iter().enumerate() {
            let path = format!("/architecture/boundaries/{index}");
            let earlier = self.boundaries[..index]
                .iter()
                .map(|other| other.id.as_str());
            validate_id(&boundary.id, earlier, &path)?;
            validate_text(&boundary.label, &format!("{path}/label"), 60)?;
        }
        for (index, boundary) in self.boundaries.iter().enumerate() {
            let path = format!("/architecture/boundaries/{index}/in");
            if let Some(parent) = &boundary.parent
                && self.boundary(parent).is_none()
            {
                return Err(ChartError::new(
                    "unknown_boundary",
                    path,
                    format!("no boundary has the id \"{parent}\""),
                ));
            }
            let Some(chain) = self.chain(index) else {
                return Err(ChartError::new(
                    "boundary_cycle",
                    path,
                    "boundaries lie in each other in a circle",
                ));
            };
            if chain.len() > MAX_BOUNDARY_DEPTH {
                return Err(ChartError::new(
                    "boundaries_too_deep",
                    path,
                    format!("boundaries nest at most {MAX_BOUNDARY_DEPTH} deep"),
                ));
            }
        }
        Ok(())
    }

    fn validate_components(&self) -> Result<(), ChartError> {
        if self.components.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/architecture/components",
                "provide at least one component",
            ));
        }
        if self.components.len() > MAX_COMPONENTS {
            return Err(ChartError::new(
                "too_many_nodes",
                "/architecture/components",
                format!("at most {MAX_COMPONENTS} components are supported"),
            ));
        }
        for (index, component) in self.components.iter().enumerate() {
            let path = format!("/architecture/components/{index}");
            let earlier = self
                .boundaries
                .iter()
                .map(|boundary| boundary.id.as_str())
                .chain(
                    self.components[..index]
                        .iter()
                        .map(|other| other.id.as_str()),
                );
            validate_id(&component.id, earlier, &path)?;
            validate_text(&component.label, &format!("{path}/label"), 60)?;
            if let Some(sublabel) = &component.sublabel {
                validate_text(sublabel, &format!("{path}/sublabel"), 60)?;
            }
            if let Some(boundary) = &component.boundary
                && self.boundary(boundary).is_none()
            {
                return Err(ChartError::new(
                    "unknown_boundary",
                    format!("{path}/in"),
                    format!("no boundary has the id \"{boundary}\""),
                ));
            }
        }
        for (index, boundary) in self.boundaries.iter().enumerate() {
            let holds = self.components.iter().any(|component| {
                component
                    .boundary
                    .as_deref()
                    .and_then(|id| self.boundary(id))
                    .and_then(|inner| self.chain(inner))
                    .is_some_and(|chain| chain.contains(&index))
            });
            if !holds {
                return Err(ChartError::new(
                    "empty_boundary",
                    format!("/architecture/boundaries/{index}"),
                    format!("no component lies in \"{}\"", boundary.id),
                ));
            }
        }
        Ok(())
    }

    fn validate_connections(&self) -> Result<(), ChartError> {
        if self.connections.len() > MAX_CONNECTIONS {
            return Err(ChartError::new(
                "too_many_edges",
                "/architecture/connections",
                format!("at most {MAX_CONNECTIONS} connections are supported"),
            ));
        }
        for (index, connection) in self.connections.iter().enumerate() {
            let path = format!("/architecture/connections/{index}");
            for (field, id) in [("from", &connection.from), ("to", &connection.to)] {
                if self.component(id).is_none() {
                    return Err(ChartError::new(
                        "unknown_node",
                        format!("{path}/{field}"),
                        format!("no component has the id \"{id}\""),
                    ));
                }
            }
            if let Some(label) = &connection.label {
                validate_text(label, &format!("{path}/label"), 60)?;
            }
            if let Some(technology) = &connection.technology {
                validate_text(technology, &format!("{path}/technology"), 40)?;
            }
        }
        for (index, id) in self.main_path.iter().enumerate() {
            if self.component(id).is_none() {
                return Err(ChartError::new(
                    "unknown_node",
                    format!("/architecture/mainPath/{index}"),
                    format!("no component has the id \"{id}\""),
                ));
            }
        }
        for (index, pair) in self.main_path.windows(2).enumerate() {
            if !self
                .connections
                .iter()
                .any(|connection| connection.from == pair[0] && connection.to == pair[1])
            {
                return Err(ChartError::new(
                    "main_path_gap",
                    format!("/architecture/mainPath/{}", index + 1),
                    format!(
                        "no connection leads from \"{}\" to \"{}\"; the main path follows connections",
                        pair[0], pair[1]
                    ),
                ));
            }
        }
        Ok(())
    }
}
