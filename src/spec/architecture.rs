use serde::{Deserialize, Serialize};

use super::{
    ChartSpec, Dash, DiagramOrientation, DiagramRole, DiagramSpacing, MainPath, NoteSpec, PortSide,
    flow::validate_notes,
    sequence::is_identifier,
    validate_text,
    view::{ViewScope, ViewSpec},
};
use crate::error::{ChartError, ChartWarning};

/// Components of one diagram, connections between them, and boundaries around them.
pub(crate) const MAX_COMPONENTS: usize = 40;
pub(crate) const MAX_CONNECTIONS: usize = 80;
pub(crate) const MAX_BOUNDARIES: usize = 12;
/// How deeply boundaries may nest inside each other.
pub(crate) const MAX_BOUNDARY_DEPTH: usize = 4;
/// Perspectives of one model, and categories of one perspective: one color each.
const MAX_PERSPECTIVES: usize = 8;
const MAX_CATEGORIES: usize = 8;
/// Interfaces one component provides: each takes a ball on a stem at its box.
const MAX_INTERFACES: usize = 4;

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
    #[serde(default, skip_serializing_if = "MainPath::is_empty")]
    pub main_path: MainPath,
    /// Notes beside components, inside boundaries or on the whole diagram, such as an assumption
    /// or a limitation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<NoteSpec>,
    /// Sets of component ids that share one rank: each set lies at one position along the flow.
    /// A layout hint only; it adds no connection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub same_rank: Vec<Vec<String>>,
    #[serde(default, skip_serializing_if = "DiagramOrientation::is_auto")]
    pub orientation: DiagramOrientation,
    #[serde(default, skip_serializing_if = "DiagramSpacing::is_normal")]
    pub spacing: DiagramSpacing,
    /// Whether connections that leave a component together, or reach one together, are drawn on
    /// one shared line that branches; only those that read the same on the shared part.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub shared_trunks: bool,
    /// Views on this model: each shows a selection of its components and connections as a
    /// diagram of its own. A specification with views renders one of them, never the whole
    /// model.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub views: Vec<ViewSpec>,
    /// The `key` of the view rendered when none is asked for; the first view without it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_view: Option<String>,
    /// Questions the model can be seen under, such as which team runs a component: each names
    /// its categories, and components state theirs by the perspective's `key`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub perspectives: Vec<PerspectiveSpec>,
    /// The `key` of the perspective the diagram shows, and every view without one of its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perspective: Option<String>,
    /// How much of its model a resolved view shows, for its description.
    #[serde(skip)]
    pub(crate) scope: Option<ViewScope>,
    /// The environment a resolved deployment view shows, and how many components of the model
    /// its instances run, for its description.
    #[serde(skip)]
    pub(crate) environment: Option<(String, usize)>,
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
    /// The name of a symbol drawn above the label, which names the component's technology
    /// category, such as `cache` or `load-balancer`; the kind keeps its shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// The `id` of the boundary the component lies in.
    #[serde(default, rename = "in", skip_serializing_if = "Option::is_none")]
    pub boundary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<DiagramRole>,
    /// The interfaces the component provides, each drawn as a ball on a stem at its box, which
    /// connections can require by `toInterface`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provides: Vec<InterfaceSpec>,
}

/// An interface a component provides, such as an API: an `id` unique within the component and
/// the `label` written beside its ball.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InterfaceSpec {
    pub id: String,
    pub label: String,
}

/// A question the model can be seen under, such as who runs a component or how much technical
/// debt it carries, with the categories that answer it and the components in each. Only what
/// the categories list is shown; nothing is derived.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerspectiveSpec {
    /// A stable name of the perspective, by which views and the model select it.
    pub key: String,
    /// What the perspective is called in the legend, the description and the table: `Team`.
    pub title: String,
    /// A sentence that says what the perspective means, for the description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The categories in their order, which also fixes their colors: at most eight. A component
    /// that none lists is shown as not stated.
    pub categories: Vec<CategorySpec>,
}

/// A category of a perspective, such as a team, and the ids of the components in it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CategorySpec {
    pub label: String,
    pub components: Vec<String>,
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
    /// A security component, such as an identity provider, a firewall or a vault: a shield.
    Security,
    /// A system outside the scope: a dashed box.
    External,
    /// A machine, a virtual machine or a cluster node that runs other components: a box seen
    /// from a corner.
    Node,
    /// Something that is deployed, such as an image, a package or a file: a box with a folded
    /// corner.
    Artifact,
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
    /// A stable name by which views select the connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
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
    /// The place of this connection in a numbered sequence of calls, from 1 to 99; every step
    /// number is used once. The number is written before the label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<u8>,
    /// How much the connection carries, such as requests per second: the line is drawn as much
    /// stronger as it carries more.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<DiagramRole>,
    /// Which way the connection runs; `forward`, from `from` to `to`, when absent.
    #[serde(default, skip_serializing_if = "ConnectionDirection::is_forward")]
    pub direction: ConnectionDirection,
    /// The side of `from` the connection leaves, and the side of `to` it reaches; where the
    /// layout chooses when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_side: Option<PortSide>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_side: Option<PortSide>,
    /// The `id` of an interface `to` provides: `from` requires that interface, and the
    /// connection ends in a socket around its ball.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_interface: Option<String>,
}

/// Which way a connection runs. A connection both ways stays one connection: one relationship
/// that is used in either direction, such as a socket or a synchronisation.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionDirection {
    /// From `from` to `to`: an arrowhead at `to`.
    #[default]
    Forward,
    /// Both ways: an arrowhead at either end.
    Both,
    /// Without a direction, such as replication between peers: no arrowhead.
    None,
}

impl ConnectionDirection {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(crate) const fn is_forward(&self) -> bool {
        matches!(self, Self::Forward)
    }

    /// The sign between the two ends of a connection: `→`, `↔` or `—`.
    pub(crate) const fn sign(self) -> &'static str {
        match self {
            Self::Forward => "→",
            Self::Both => "↔",
            Self::None => "—",
        }
    }
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

    /// The interface a connection requires: the index of the component that provides it and
    /// its place among that component's interfaces.
    pub(crate) fn required(&self, connection: &ConnectionSpec) -> Option<(usize, usize)> {
        let id = connection.to_interface.as_deref()?;
        let component = self.component(&connection.to)?;
        let interface = self.components[component]
            .provides
            .iter()
            .position(|interface| interface.id == id)?;
        Some((component, interface))
    }

    pub(crate) fn boundary(&self, id: &str) -> Option<usize> {
        self.boundaries
            .iter()
            .position(|boundary| boundary.id == id)
    }

    /// The perspective the diagram shows, if any.
    pub(crate) fn shown_perspective(&self) -> Option<&PerspectiveSpec> {
        let key = self.perspective.as_deref()?;
        self.perspectives
            .iter()
            .find(|perspective| perspective.key == key)
    }

    /// The perspective `key`, named at `path`: one the model has.
    pub(crate) fn check_perspective(&self, key: &str, path: &str) -> Result<(), ChartError> {
        if self
            .perspectives
            .iter()
            .any(|perspective| perspective.key == key)
        {
            return Ok(());
        }
        Err(ChartError::new(
            "unknown_perspective",
            path,
            format!("no perspective has the key \"{key}\""),
        ))
    }

    /// Whether every connection runs one way, from `from` to `to`.
    pub(crate) fn is_directed(&self) -> bool {
        self.connections
            .iter()
            .all(|connection| connection.direction.is_forward())
    }

    /// Whether connection `index` joins two consecutive components of the main path.
    pub(crate) fn on_main_path(&self, index: usize) -> bool {
        let connection = &self.connections[index];
        self.main_path_steps()
            .windows(2)
            .any(|pair| pair[0] == connection.from && pair[1] == connection.to)
    }

    /// The ids of the components along the main path; empty without one.
    pub(crate) fn main_path_steps(&self) -> Vec<String> {
        self.main_path
            .steps(&self.connection_ids())
            .expect("validated main paths have a way")
    }

    fn connection_ids(&self) -> impl Iterator<Item = (&str, &str)> + Clone {
        self.connections
            .iter()
            .map(|connection| (connection.from.as_str(), connection.to.as_str()))
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
    /// Whether an architecture diagram shows a perspective.
    pub(crate) fn has_perspective(&self) -> bool {
        self.architecture
            .as_ref()
            .is_some_and(|architecture| architecture.perspective.is_some())
    }

    /// Whether an architecture diagram has a component that provides an interface.
    pub(crate) fn has_interfaces(&self) -> bool {
        self.architecture.as_ref().is_some_and(|architecture| {
            architecture
                .components
                .iter()
                .any(|component| !component.provides.is_empty())
        })
    }

    /// Whether an architecture diagram has a component with a symbol.
    pub(crate) fn has_component_symbols(&self) -> bool {
        self.architecture.as_ref().is_some_and(|architecture| {
            architecture
                .components
                .iter()
                .any(|component| component.symbol.is_some())
        })
    }

    /// An architecture diagram: components and boundaries with identifiers unique among both,
    /// connections between components, boundaries that nest without a circle and each hold at
    /// least one component, a main path along connections, and views that each resolve to a
    /// valid diagram.
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
        architecture.validate_perspectives()?;
        validate_notes(
            &architecture.notes,
            "architecture",
            |id| architecture.component(id).is_some() || architecture.boundary(id).is_some(),
            "component or boundary",
        )?;
        self.validate_views()?;
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

/// The interfaces a component provides: at most [`MAX_INTERFACES`], each with a well-formed id
/// of its own and a label, and none on a person or a node, which are no software components.
fn validate_interfaces(component: &ComponentSpec, path: &str) -> Result<(), ChartError> {
    if component.provides.is_empty() {
        return Ok(());
    }
    if matches!(component.kind, ComponentKind::Person | ComponentKind::Node) {
        return Err(ChartError::new(
            "option_not_supported",
            format!("{path}/provides"),
            "a person or a node provides no interface; give it to the component that implements it",
        ));
    }
    if component.provides.len() > MAX_INTERFACES {
        return Err(ChartError::new(
            "too_many_interfaces",
            format!("{path}/provides"),
            format!("a component provides at most {MAX_INTERFACES} interfaces"),
        ));
    }
    for (index, interface) in component.provides.iter().enumerate() {
        let path = format!("{path}/provides/{index}");
        if !is_identifier(&interface.id) {
            return Err(ChartError::new(
                "invalid_id",
                format!("{path}/id"),
                "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
            ));
        }
        if component.provides[..index]
            .iter()
            .any(|other| other.id == interface.id)
        {
            return Err(ChartError::new(
                "duplicate_id",
                format!("{path}/id"),
                format!(
                    "the id \"{}\" is already taken by an earlier interface of this component",
                    interface.id
                ),
            ));
        }
        validate_text(&interface.label, &format!("{path}/label"), 40)?;
    }
    Ok(())
}

/// The `id` of a connection: well formed, and not taken by an `earlier` connection.
fn validate_connection_id(
    id: &str,
    earlier: &[ConnectionSpec],
    path: &str,
) -> Result<(), ChartError> {
    if !is_identifier(id) {
        return Err(ChartError::new(
            "invalid_id",
            format!("{path}/id"),
            "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
        ));
    }
    if earlier.iter().any(|other| other.id.as_deref() == Some(id)) {
        return Err(ChartError::new(
            "duplicate_id",
            format!("{path}/id"),
            format!("the id \"{id}\" is already taken by an earlier connection"),
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
            if let Some(symbol) = &component.symbol
                && crate::symbols::find(symbol).is_none()
            {
                return Err(ChartError::new(
                    "unknown_symbol",
                    format!("{path}/symbol"),
                    format!(
                        "no symbol is named \"{symbol}\"; use one of {}",
                        crate::symbols::names()
                    ),
                ));
            }
            validate_interfaces(component, &path)?;
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
            if let Some(id) = &connection.id {
                validate_connection_id(id, &self.connections[..index], &path)?;
            }
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
            if let Some(interface) = &connection.to_interface {
                if self.required(connection).is_none() {
                    return Err(ChartError::new(
                        "unknown_interface",
                        format!("{path}/toInterface"),
                        format!(
                            "\"{}\" provides no interface with the id \"{interface}\"",
                            connection.to
                        ),
                    ));
                }
                if !connection.direction.is_forward() {
                    return Err(ChartError::new(
                        "option_not_supported",
                        format!("{path}/direction"),
                        "a connection that requires an interface runs forward, from the component that requires it",
                    ));
                }
                if connection.to_side.is_some() {
                    return Err(ChartError::new(
                        "option_not_supported",
                        format!("{path}/toSide"),
                        "a connection that requires an interface ends at its ball, ahead of the component; it takes no toSide",
                    ));
                }
            }
            super::flow::validate_weight(connection.weight, &path)?;
            super::flow::validate_role(connection.role, connection.dash, &path)?;
            super::flow::validate_sides(
                (&connection.from, &connection.to),
                (connection.from_side, connection.to_side),
                &path,
            )?;
            if let Some(technology) = &connection.technology {
                validate_text(technology, &format!("{path}/technology"), 40)?;
            }
            if let Some(step) = connection.step {
                if !(1..=99).contains(&step) {
                    return Err(ChartError::new(
                        "invalid_step",
                        format!("{path}/step"),
                        "a step is a number from 1 to 99",
                    ));
                }
                if self.connections[..index]
                    .iter()
                    .any(|earlier| earlier.step == Some(step))
                {
                    return Err(ChartError::new(
                        "duplicate_step",
                        format!("{path}/step"),
                        format!("step {step} is already used by an earlier connection"),
                    ));
                }
            }
        }
        self.main_path.validate(
            "architecture",
            |id| self.component(id).is_some(),
            &self.connection_ids(),
            ("component", "connection"),
        )?;
        super::flow::validate_same_rank(
            &self.same_rank,
            "architecture",
            |id| self.component(id).is_some(),
            &self.connection_ids(),
            "component",
        )?;
        Ok(())
    }

    /// Perspectives with well-formed keys used once, a title and one to eight distinct
    /// categories, each listing existing components, none of them twice in one perspective; and
    /// a shown perspective the model has.
    fn validate_perspectives(&self) -> Result<(), ChartError> {
        if self.perspectives.len() > MAX_PERSPECTIVES {
            return Err(ChartError::new(
                "too_many_perspectives",
                "/architecture/perspectives",
                format!("at most {MAX_PERSPECTIVES} perspectives are supported"),
            ));
        }
        for (index, perspective) in self.perspectives.iter().enumerate() {
            let path = format!("/architecture/perspectives/{index}");
            if !is_identifier(&perspective.key)
                || self.perspectives[..index]
                    .iter()
                    .any(|other| other.key == perspective.key)
            {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/key"),
                    "use a key of its own: 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            validate_text(&perspective.title, &format!("{path}/title"), 40)?;
            if let Some(description) = &perspective.description {
                validate_text(description, &format!("{path}/description"), 200)?;
            }
            let categories = &perspective.categories;
            if categories.is_empty() || categories.len() > MAX_CATEGORIES {
                return Err(ChartError::new(
                    "invalid_categories",
                    format!("{path}/categories"),
                    format!("a perspective has 1 to {MAX_CATEGORIES} categories"),
                ));
            }
            let mut listed: Vec<&str> = Vec::new();
            for (at, category) in categories.iter().enumerate() {
                let path = format!("{path}/categories/{at}");
                validate_text(&category.label, &format!("{path}/label"), 40)?;
                if categories[..at]
                    .iter()
                    .any(|other| other.label == category.label)
                {
                    return Err(ChartError::new(
                        "duplicate_category",
                        format!("{path}/label"),
                        format!("the category \"{}\" is already listed", category.label),
                    ));
                }
                for (member, id) in category.components.iter().enumerate() {
                    let path = format!("{path}/components/{member}");
                    if self.component(id).is_none() {
                        return Err(ChartError::new(
                            "unknown_node",
                            path,
                            format!("no component has the id \"{id}\""),
                        ));
                    }
                    if listed.contains(&id.as_str()) {
                        return Err(ChartError::new(
                            "duplicate_member",
                            path,
                            format!("\"{id}\" already has a category in this perspective"),
                        ));
                    }
                    listed.push(id);
                }
            }
        }
        if let Some(key) = &self.perspective {
            self.check_perspective(key, "/architecture/perspective")?;
        }
        Ok(())
    }
}
