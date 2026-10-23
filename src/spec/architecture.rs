use serde::{Deserialize, Serialize};

use super::{
    ChartSpec, Dash, DiagramOrientation, DiagramRole, MainPath, NoteSpec,
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
    /// Names of groups the component belongs to, such as `public` or `payments`, by which
    /// views select it; not drawn.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
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

/// A category of a perspective, such as a team, and the ids of the components and of the
/// connections in it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CategorySpec {
    pub label: String,
    #[serde(default)]
    pub components: Vec<String>,
    /// The `id`s of the connections in the category; a connection that no category lists is
    /// drawn as without the perspective.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<String>,
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
    /// Names of groups the connection belongs to, by which views select or leave it out; not
    /// drawn.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// In a resolved view, the `id` of the model connection that a repeated occurrence stands
    /// for, which keeps its id only the first time.
    #[serde(skip)]
    pub(crate) model_id: Option<String>,
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

    /// The category of the shown perspective that connection `index` is in, by its place among
    /// the perspective's categories.
    pub(crate) fn connection_category(&self, index: usize) -> Option<usize> {
        let connection = &self.connections[index];
        let id = connection.id.as_ref().or(connection.model_id.as_ref())?;
        self.shown_perspective()?
            .categories
            .iter()
            .position(|category| category.connections.contains(id))
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
    /// The specification with the perspective `key` shown in place of the one the model and
    /// each of its views select, as a render with [`crate::RenderOptions::perspective`] shows it.
    ///
    /// # Errors
    ///
    /// Returns a structured error when the specification is invalid, and `unknown_perspective`
    /// at `/render/perspective` when it has no perspective `key`.
    pub fn with_perspective(&self, key: &str) -> Result<Self, ChartError> {
        self.validate()?;
        let keys: Vec<&str> = self
            .architecture
            .iter()
            .flat_map(|architecture| &architecture.perspectives)
            .map(|perspective| perspective.key.as_str())
            .collect();
        if !keys.contains(&key) {
            let message = if keys.is_empty() {
                "the specification has no perspectives; only an architecture diagram with perspectives has them"
                    .to_owned()
            } else {
                format!(
                    "no perspective has the key \"{key}\"; the perspectives are {}",
                    keys.iter()
                        .map(|key| format!("\"{key}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            return Err(ChartError::new(
                "unknown_perspective",
                "/render/perspective",
                message,
            ));
        }
        let mut spec = self.clone();
        // Only an architecture diagram has the perspective.
        if let Some(architecture) = &mut spec.architecture {
            architecture.perspective = Some(key.to_owned());
            for view in &mut architecture.views {
                view.perspective = Some(key.to_owned());
            }
        }
        Ok(spec)
    }

    /// Whether an architecture diagram shows a perspective.
    pub(crate) fn has_perspective(&self) -> bool {
        self.architecture
            .as_ref()
            .is_some_and(|architecture| architecture.perspective.is_some())
    }

    /// Whether an architecture diagram shows a perspective with a category of connections.
    pub(crate) fn has_connection_categories(&self) -> bool {
        self.architecture
            .as_ref()
            .and_then(ArchitectureSpec::shown_perspective)
            .is_some_and(|perspective| {
                perspective
                    .categories
                    .iter()
                    .any(|category| !category.connections.is_empty())
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

/// Tags of one component or connection.
const MAX_TAGS: usize = 8;

/// A tag, named at `path`: well formed like an identifier.
pub(super) fn validate_tag(tag: &str, path: &str) -> Result<(), ChartError> {
    if is_identifier(tag) {
        return Ok(());
    }
    Err(ChartError::new(
        "invalid_tag",
        path,
        "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
    ))
}

/// The `tags` of the component or connection at `path`: at most eight, well formed, each once.
fn validate_tags(tags: &[String], path: &str) -> Result<(), ChartError> {
    if tags.len() > MAX_TAGS {
        return Err(ChartError::new(
            "too_many_tags",
            format!("{path}/tags"),
            format!("at most {MAX_TAGS} tags are supported"),
        ));
    }
    for (at, tag) in tags.iter().enumerate() {
        let path = format!("{path}/tags/{at}");
        validate_tag(tag, &path)?;
        if tags[..at].contains(tag) {
            return Err(ChartError::new(
                "duplicate_tag",
                path,
                format!("the tag \"{tag}\" is already listed"),
            ));
        }
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
            validate_tags(&component.tags, &path)?;
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
            validate_tags(&connection.tags, &path)?;
            super::flow::validate_weight(connection.weight, &path)?;
            super::flow::validate_role(connection.role, connection.dash, &path)?;
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
    /// categories, each listing existing components and connections, none of them twice in one
    /// perspective; and a shown perspective the model has.
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
            let mut connections: Vec<&str> = Vec::new();
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
                for (member, id) in category.connections.iter().enumerate() {
                    let path = format!("{path}/connections/{member}");
                    if !self
                        .connections
                        .iter()
                        .any(|connection| connection.id.as_ref() == Some(id))
                    {
                        return Err(ChartError::new(
                            "unknown_connection",
                            path,
                            format!("no connection has the id \"{id}\""),
                        ));
                    }
                    if connections.contains(&id.as_str()) {
                        return Err(ChartError::new(
                            "duplicate_member",
                            path,
                            format!(
                                "the connection \"{id}\" already has a category in this perspective"
                            ),
                        ));
                    }
                    connections.push(id);
                }
            }
        }
        if let Some(key) = &self.perspective {
            self.check_perspective(key, "/architecture/perspective")?;
        }
        Ok(())
    }
}
