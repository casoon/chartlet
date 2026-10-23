//! Views on the model of an architecture diagram. A view selects components and connections of
//! the model and resolves to an architecture diagram of its own, which is validated, laid out
//! and described like any other. Nothing outside the selection is drawn or described, and no
//! connection is drawn that the model does not have.

use serde::{Deserialize, Serialize};

use super::{
    ArchitectureSpec, BoundarySpec, ChartSpec, ConnectionSpec, MainPath, MobileSpec,
    sequence::is_identifier, validate_text,
};
use crate::error::ChartError;

/// Views of one model; each is rendered on its own.
pub(crate) const MAX_VIEWS: usize = 20;
/// The occurrences of a dynamic view, numbered like the steps of connections.
const MAX_VIEW_STEPS: usize = 99;

/// A view on an architecture model: a key that names it, its own title and description, and
/// the components it shows. Connections between them come along unless the view lists its own;
/// a dynamic view instead lists `steps`, occurrences of existing connections in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewSpec {
    /// A stable name of the view, used to select it; independent of its title.
    pub key: String,
    /// The title of the diagram the view renders, in place of the model's.
    pub title: String,
    /// The description of the diagram the view renders. Without one, it is generated from what
    /// the view shows; the model's description is never used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The ids of the components the view shows. A dynamic view without them shows the ends of
    /// its steps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    /// The ids of the connections the view shows; without them, every connection between its
    /// components.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connections: Option<Vec<String>>,
    /// The way the view draws stronger, along its connections; without one, the main path of
    /// the model when the view shows all of it.
    #[serde(default, skip_serializing_if = "MainPath::is_empty")]
    pub main_path: MainPath,
    /// A dynamic view: connections of the model in the order they occur, numbered from 1. The
    /// same connection may occur more than once.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<ViewStepSpec>,
    /// The `key` of the perspective the view shows, in place of the model's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perspective: Option<String>,
    /// The width of the diagram the view renders, in place of the model's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// The height of the diagram the view renders, in place of the model's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// The mobile layout of the diagram the view renders, in place of the model's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile: Option<MobileSpec>,
    /// Tags that select more components: one carrying any of them is shown, unless it carries
    /// an `exclude` tag. With `connections`, also the connections between shown components that
    /// carry one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<String>,
    /// Tags that leave out what `include` or the model brings: a component or connection
    /// carrying any of them is not shown, unless the view lists it. A view with `exclude` alone
    /// starts from every component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// A deployment view: the name of the environment, such as `Production`. Such a view shows
    /// `instances` of the model's components on its `hosts`, joined by its
    /// `instanceConnections`, in place of components, connections and steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// The machines, clusters or zones of a deployment view, nested like boundaries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hosts: Vec<BoundarySpec>,
    /// The running instances of a deployment view, each of one component of the model.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<InstanceSpec>,
    /// The connections between instances of a deployment view; nothing else joins them. One
    /// with the `id` of a model connection takes what it leaves out from it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instance_connections: Vec<ConnectionSpec>,
    /// Whether a deployment view also derives instance connections from every model connection
    /// whose `id` its `instanceConnections` do not name: each instance of its `from` component
    /// joins the nearest instances of its `to` component, by hosts.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub derive_connections: bool,
}

/// One running instance of a component in a deployment view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstanceSpec {
    /// A stable id of the instance, unique among the instances and hosts of the view.
    pub id: String,
    /// The `id` of the component of the model it runs.
    pub of: String,
    /// The `id` of the host it runs on.
    #[serde(default, rename = "in", skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

/// One occurrence of a connection in a dynamic view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewStepSpec {
    /// The `id` of the connection.
    pub connection: String,
    /// What happens in this step, in place of the connection's label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// How much of its model a resolved view shows: components and connections, shown and in all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ViewScope {
    pub components: (usize, usize),
    pub connections: (usize, usize),
}

/// A view resolved to a diagram of its own, with what each of its components, connections and
/// boundaries is in the model, so that diagnostics point into the model.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedView {
    pub spec: ChartSpec,
    pub key: String,
    pub(super) index: usize,
    pub(super) components: Vec<usize>,
    pub(super) connections: Vec<usize>,
    pub(super) boundaries: Vec<usize>,
    /// The shared ranks the view keeps: the place of each in the model and the places of its
    /// shown members in it.
    pub(super) same_rank: Vec<(usize, Vec<usize>)>,
    /// The notes the view keeps, those on shown components, by their places in the model.
    pub(super) notes: Vec<usize>,
    /// The connections are the steps of the view, in order.
    pub(super) dynamic: bool,
    pub(super) own_main_path: bool,
    /// The view is a deployment: its components are the view's instances, its boundaries its
    /// hosts and its connections its instance connections.
    pub(super) deployment: bool,
    /// The fields of the size the view sets in place of the model's: `width`, `height`, `mobile`.
    pub(super) own_size: Vec<&'static str>,
    /// How many instance connections a deployment view lists, and the place in the model of the
    /// connection each derived one after them comes from.
    pub(super) explicit_connections: usize,
    pub(super) derived: Vec<usize>,
}

impl ChartSpec {
    /// The keys of the views of an architecture model, in order; empty without views.
    #[must_use]
    pub fn view_keys(&self) -> Vec<&str> {
        self.views().iter().map(|view| view.key.as_str()).collect()
    }

    /// The view `key` of an architecture model, or its default view with `None`, as a
    /// specification of its own: the selected components, connections and boundaries, the
    /// view's title and description, and everything else of the model. A specification without
    /// views is returned as it is when no view is asked for.
    ///
    /// # Errors
    ///
    /// Returns a structured error when the model or the view is invalid, or no view has `key`.
    pub fn view(&self, key: Option<&str>) -> Result<Self, ChartError> {
        self.validate()?;
        Ok(self
            .resolve_view(key)?
            .map_or_else(|| self.clone(), |view| view.spec))
    }

    fn views(&self) -> &[ViewSpec] {
        self.architecture
            .as_ref()
            .map_or(&[], |architecture| architecture.views.as_slice())
    }

    pub(crate) fn has_views(&self) -> bool {
        !self.views().is_empty()
    }

    /// The view `key`, or the default view with `None`, of a validated specification; `None`
    /// for a specification without views when no view is asked for.
    pub(crate) fn resolve_view(
        &self,
        key: Option<&str>,
    ) -> Result<Option<ResolvedView>, ChartError> {
        let views = self.views();
        let index = match key {
            None if views.is_empty() => return Ok(None),
            None => self
                .architecture
                .as_ref()
                .and_then(|architecture| architecture.default_view.as_deref())
                .and_then(|key| views.iter().position(|view| view.key == key))
                .unwrap_or(0),
            Some(key) => views
                .iter()
                .position(|view| view.key == key)
                .ok_or_else(|| unknown_view(key, &self.view_keys()))?,
        };
        Ok(Some(self.resolved(index)))
    }

    /// Checks every view against the model and validates the diagram it resolves to.
    pub(super) fn validate_views(&self) -> Result<(), ChartError> {
        let architecture = self
            .architecture
            .as_ref()
            .expect("an architecture diagram carries an architecture block");
        if let Some(key) = &architecture.default_view
            && !architecture.views.iter().any(|view| &view.key == key)
        {
            return Err(ChartError::new(
                "unknown_view",
                "/architecture/defaultView",
                format!("no view has the key \"{key}\""),
            ));
        }
        if architecture.views.len() > MAX_VIEWS {
            return Err(ChartError::new(
                "too_many_views",
                "/architecture/views",
                format!("at most {MAX_VIEWS} views are supported"),
            ));
        }
        for index in 0..architecture.views.len() {
            architecture.validate_view(index)?;
            let view = self.resolved(index);
            view.spec.validate().map_err(|error| {
                ChartError::new(error.code, view.model_path(&error.path), error.message)
            })?;
        }
        Ok(())
    }

    /// The view at `index` of a model whose views were checked.
    #[allow(clippy::too_many_lines)]
    fn resolved(&self, index: usize) -> ResolvedView {
        let model = self
            .architecture
            .as_ref()
            .expect("only architecture diagrams have views");
        let view = &model.views[index];
        if view.environment.is_some() {
            return self.resolved_deployment(index);
        }
        let dynamic = !view.steps.is_empty();
        let (components, connections, boundaries) = model.selection(view);
        let is_shown = |id: &str| {
            components
                .iter()
                .any(|&component| model.components[component].id == id)
        };

        let mut architecture = model.clone();
        architecture.components = components
            .iter()
            .map(|&index| model.components[index].clone())
            .collect();
        architecture.connections = connections
            .iter()
            .enumerate()
            .map(|(occurrence, &index)| {
                let mut connection = model.connections[index].clone();
                if dynamic {
                    let step = &view.steps[occurrence];
                    connection.step = Some(u8::try_from(occurrence + 1).expect("at most 99 steps"));
                    if step.label.is_some() {
                        connection.label.clone_from(&step.label);
                    }
                    // A connection that occurs again keeps its id only the first time.
                    if connections[..occurrence].contains(&index) {
                        connection.model_id = connection.id.take();
                    }
                }
                connection
            })
            .collect();
        architecture.boundaries = boundaries
            .iter()
            .map(|&index| model.boundaries[index].clone())
            .collect();
        architecture.main_path = if view.main_path.is_empty() {
            let steps = model.main_path_steps();
            let carried = steps.windows(2).all(|pair| {
                architecture
                    .connections
                    .iter()
                    .any(|connection| connection.from == pair[0] && connection.to == pair[1])
            }) && steps.iter().all(|id| is_shown(id));
            if carried {
                MainPath::Steps(steps)
            } else {
                MainPath::default()
            }
        } else {
            view.main_path.clone()
        };
        let same_rank = shown_ranks(model, &is_shown);
        architecture.same_rank = same_rank
            .iter()
            .map(|(set, shown)| {
                shown
                    .iter()
                    .map(|&member| model.same_rank[*set][member].clone())
                    .collect()
            })
            .collect();
        let notes = shown_notes(model, &is_shown);
        architecture.notes = notes
            .iter()
            .map(|&note| model.notes[note].clone())
            .collect();
        architecture.views = Vec::new();
        architecture.default_view = None;
        architecture.perspective = view.perspective.clone().or(architecture.perspective);
        // A category keeps the components and connections the view shows.
        for perspective in &mut architecture.perspectives {
            for category in &mut perspective.categories {
                category.components.retain(|id| is_shown(id));
                category.connections.retain(|id| {
                    connections
                        .iter()
                        .any(|&index| model.connections[index].id.as_ref() == Some(id))
                });
            }
        }
        let mut distinct = connections.clone();
        distinct.sort_unstable();
        distinct.dedup();
        architecture.scope = Some(ViewScope {
            components: (components.len(), model.components.len()),
            connections: (distinct.len(), model.connections.len()),
        });

        let mut spec = self.clone();
        spec.title.clone_from(&view.title);
        spec.description.clone_from(&view.description);
        spec.architecture = Some(architecture);
        let own_size = view.size_of(&mut spec);
        ResolvedView {
            spec,
            key: view.key.clone(),
            index,
            components,
            connections,
            boundaries,
            same_rank,
            notes,
            dynamic,
            own_main_path: !view.main_path.is_empty(),
            deployment: false,
            own_size,
            explicit_connections: 0,
            derived: Vec::new(),
        }
    }
}

/// The shared ranks a view keeps, by their places in the model, each with the places of its
/// shown members: a rank keeps its shown members, and only while they are two or more.
fn shown_ranks(
    model: &ArchitectureSpec,
    is_shown: &impl Fn(&str) -> bool,
) -> Vec<(usize, Vec<usize>)> {
    model
        .same_rank
        .iter()
        .enumerate()
        .map(|(set, members)| {
            let shown = (0..members.len())
                .filter(|&member| is_shown(&members[member]))
                .collect::<Vec<_>>();
            (set, shown)
        })
        .filter(|(_, shown)| shown.len() >= 2)
        .collect()
}

/// The places in the model of the notes a view keeps: a note stays with its component, and one
/// on a hidden component is left out.
fn shown_notes(model: &ArchitectureSpec, is_shown: &impl Fn(&str) -> bool) -> Vec<usize> {
    (0..model.notes.len())
        .filter(|&note| is_shown(&model.notes[note].on))
        .collect()
}

impl ViewSpec {
    /// Whether a static view shows the component `id` with `tags`: one it lists always; another
    /// when one of its tags is in `include` (or, for a view that lists neither components nor
    /// `include`, any) and none in `exclude`.
    fn selects(&self, id: &str, tags: &[String]) -> bool {
        if self.components.iter().any(|listed| listed == id) {
            return true;
        }
        let tagged = |with: &[String]| tags.iter().any(|tag| with.contains(tag));
        let selected = tagged(&self.include)
            || (self.components.is_empty() && self.include.is_empty() && !self.exclude.is_empty());
        selected && !tagged(&self.exclude)
    }

    /// Whether the view selects components by tags.
    fn has_tags(&self) -> bool {
        !self.include.is_empty() || !self.exclude.is_empty()
    }

    /// Gives `spec` the size the view sets in place of the model's, and returns the fields it
    /// set.
    pub(super) fn size_of(&self, spec: &mut ChartSpec) -> Vec<&'static str> {
        let mut own = Vec::new();
        if let Some(width) = self.width {
            spec.width = width;
            own.push("width");
        }
        if let Some(height) = self.height {
            spec.height = height;
            own.push("height");
        }
        if let Some(mobile) = &self.mobile {
            spec.mobile = Some(mobile.clone());
            own.push("mobile");
        }
        own
    }
}

fn unknown_view(key: &str, keys: &[&str]) -> ChartError {
    let message = if keys.is_empty() {
        "the specification has no views; only an architecture diagram with views has them"
            .to_owned()
    } else {
        format!(
            "no view has the key \"{key}\"; the views are {}",
            keys.iter()
                .map(|key| format!("\"{key}\""))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    ChartError::new("unknown_view", "/render/view", message)
}

impl ArchitectureSpec {
    /// What `view` shows of the model: its components, its connections (one per step of a
    /// dynamic view) and the boundaries around its components, by their places in the model.
    pub(crate) fn selection(&self, view: &ViewSpec) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        let model = self;
        if view.environment.is_some() {
            return model.deployment_selection(view);
        }
        let connection = |id: &str| {
            model
                .connections
                .iter()
                .position(|connection| connection.id.as_deref() == Some(id))
                .expect("checked views name existing connections")
        };
        let dynamic = !view.steps.is_empty();
        let steps: Vec<usize> = view
            .steps
            .iter()
            .map(|step| connection(&step.connection))
            .collect();

        let shown = |component: usize| {
            let id = &model.components[component].id;
            if dynamic && view.components.is_empty() {
                steps.iter().any(|&step| {
                    let connection = &model.connections[step];
                    &connection.from == id || &connection.to == id
                })
            } else {
                view.selects(id, &model.components[component].tags)
            }
        };
        let components: Vec<usize> = (0..model.components.len())
            .filter(|&component| shown(component))
            .collect();
        let is_shown = |id: &str| {
            components
                .iter()
                .any(|&component| model.components[component].id == id)
        };

        let tagged = |tags: &[String], with: &[String]| tags.iter().any(|tag| with.contains(tag));
        let connections: Vec<usize> = if dynamic {
            steps.clone()
        } else if let Some(ids) = &view.connections {
            // The listed connections, and those between shown components that an `include`
            // tag selects and no `exclude` tag leaves out.
            (0..model.connections.len())
                .filter(|&index| {
                    let connection = &model.connections[index];
                    connection.id.as_ref().is_some_and(|id| ids.contains(id))
                        || (is_shown(&connection.from)
                            && is_shown(&connection.to)
                            && tagged(&connection.tags, &view.include)
                            && !tagged(&connection.tags, &view.exclude))
                })
                .collect()
        } else {
            (0..model.connections.len())
                .filter(|&index| {
                    let connection = &model.connections[index];
                    is_shown(&connection.from)
                        && is_shown(&connection.to)
                        && !tagged(&connection.tags, &view.exclude)
                })
                .collect()
        };

        let boundaries: Vec<usize> = (0..model.boundaries.len())
            .filter(|&boundary| {
                components.iter().any(|&component| {
                    model.components[component]
                        .boundary
                        .as_deref()
                        .and_then(|id| model.boundary(id))
                        .and_then(|inner| model.chain(inner))
                        .is_some_and(|chain| chain.contains(&boundary))
                })
            })
            .collect();
        (components, connections, boundaries)
    }

    /// Checks the references of view `index`: a well-formed key used once, components and
    /// connections that the model has, connections between shown components, and at least one
    /// component.
    fn validate_view(&self, index: usize) -> Result<(), ChartError> {
        let view = &self.views[index];
        let path = format!("/architecture/views/{index}");
        if !is_identifier(&view.key) {
            return Err(ChartError::new(
                "invalid_id",
                format!("{path}/key"),
                "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
            ));
        }
        if self.views[..index]
            .iter()
            .any(|other| other.key == view.key)
        {
            return Err(ChartError::new(
                "duplicate_id",
                format!("{path}/key"),
                format!(
                    "the key \"{}\" is already taken by an earlier view",
                    view.key
                ),
            ));
        }
        validate_text(&view.title, &format!("{path}/title"), 200)?;
        if let Some(key) = &view.perspective {
            self.check_perspective(key, &format!("{path}/perspective"))?;
        }
        if view.environment.is_some() {
            return self.validate_deployment(view, &path);
        }
        super::deployment::reject_deployment_fields(view, &path)?;
        if view.components.is_empty() && view.steps.is_empty() && !view.has_tags() {
            return Err(ChartError::new(
                "empty_view",
                path,
                "a view shows at least one component; list its components, its steps or tags to include",
            ));
        }
        if !view.steps.is_empty() && view.connections.is_some() {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/connections"),
                "a view with steps shows the connections of its steps; leave out connections",
            ));
        }
        for (position, id) in view.components.iter().enumerate() {
            let at = format!("{path}/components/{position}");
            if self.component(id).is_none() {
                return Err(ChartError::new(
                    "unknown_node",
                    at,
                    format!("no component has the id \"{id}\""),
                ));
            }
            if view.components[..position].contains(id) {
                return Err(ChartError::new(
                    "duplicate_id",
                    at,
                    format!("the component \"{id}\" is already listed"),
                ));
            }
        }
        self.validate_view_tags(view, &path)?;
        let ids = view.connections.iter().flatten().enumerate();
        for (position, id) in ids {
            let at = format!("{path}/connections/{position}");
            self.check_view_connection(view, id, &at)?;
            if view
                .connections
                .iter()
                .flatten()
                .take(position)
                .any(|other| other == id)
            {
                return Err(ChartError::new(
                    "duplicate_id",
                    at,
                    format!("the connection \"{id}\" is already listed"),
                ));
            }
        }
        if view.steps.len() > MAX_VIEW_STEPS {
            return Err(ChartError::new(
                "too_many_steps",
                format!("{path}/steps"),
                format!("a view has at most {MAX_VIEW_STEPS} steps"),
            ));
        }
        for (position, step) in view.steps.iter().enumerate() {
            let at = format!("{path}/steps/{position}");
            self.check_view_connection(view, &step.connection, &format!("{at}/connection"))?;
            if let Some(label) = &step.label {
                validate_text(label, &format!("{at}/label"), 60)?;
            }
        }
        Ok(())
    }

    /// The tags a view selects by: well formed, each once in `include` and `exclude` together,
    /// carried by a component or a connection of the model, and selecting at least one
    /// component; none in a dynamic view.
    fn validate_view_tags(&self, view: &ViewSpec, path: &str) -> Result<(), ChartError> {
        let lists = [("include", &view.include), ("exclude", &view.exclude)];
        if let Some((field, _)) = lists
            .iter()
            .find(|(_, tags)| !view.steps.is_empty() && !tags.is_empty())
        {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                "a view with steps shows the components of its steps; leave out include and exclude",
            ));
        }
        let mut seen: Vec<&str> = Vec::new();
        for (field, tags) in lists {
            for (at, tag) in tags.iter().enumerate() {
                let at = format!("{path}/{field}/{at}");
                super::architecture::validate_tag(tag, &at)?;
                if seen.contains(&tag.as_str()) {
                    return Err(ChartError::new(
                        "duplicate_tag",
                        at,
                        format!("the tag \"{tag}\" is already listed in include or exclude"),
                    ));
                }
                seen.push(tag);
                let carried = self
                    .components
                    .iter()
                    .map(|component| &component.tags)
                    .chain(self.connections.iter().map(|connection| &connection.tags))
                    .any(|tags| tags.contains(tag));
                if !carried {
                    return Err(ChartError::new(
                        "unknown_tag",
                        at,
                        format!("no component or connection carries the tag \"{tag}\""),
                    ));
                }
            }
        }
        if view.has_tags()
            && !self
                .components
                .iter()
                .any(|component| view.selects(&component.id, &component.tags))
        {
            return Err(ChartError::new(
                "empty_view",
                path,
                "the view's tags select no component; check include and exclude",
            ));
        }
        Ok(())
    }

    /// A connection a view names: one the model has, between components the view shows.
    fn check_view_connection(
        &self,
        view: &ViewSpec,
        id: &str,
        path: &str,
    ) -> Result<(), ChartError> {
        let Some(connection) = self
            .connections
            .iter()
            .find(|connection| connection.id.as_deref() == Some(id))
        else {
            return Err(ChartError::new(
                "unknown_connection",
                path,
                format!("no connection has the id \"{id}\""),
            ));
        };
        if view.components.is_empty() && !view.has_tags() {
            return Ok(());
        }
        for end in [&connection.from, &connection.to] {
            let shown = self
                .component(end)
                .is_some_and(|index| view.selects(end, &self.components[index].tags));
            if !shown {
                return Err(ChartError::new(
                    "connection_outside_view",
                    path,
                    format!(
                        "the connection \"{id}\" joins \"{end}\", which the view does not show; add it to the view's components"
                    ),
                ));
            }
        }
        Ok(())
    }
}

impl ResolvedView {
    /// A JSON pointer into the resolved diagram as a pointer into the model: its components,
    /// connections, boundaries and notes at their places in the model, its title, description and
    /// main path, and the label of a step, in the view.
    pub(crate) fn model_path(&self, path: &str) -> String {
        let view = format!("/architecture/views/{}", self.index);
        if path == "/title" || path == "/description" {
            return format!("{view}{path}");
        }
        // A size the view sets is checked in the view.
        if self.own_size.iter().any(|field| {
            path.strip_prefix('/')
                .and_then(|rest| rest.strip_prefix(field))
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
        }) {
            return format!("{view}{path}");
        }
        if self.deployment {
            // A derived connection comes from the model's; too many of them, from deriving.
            if let Some(rest) = path.strip_prefix("/architecture/connections") {
                if rest.is_empty() && !self.derived.is_empty() {
                    return format!("{view}/deriveConnections");
                }
                let (number, rest) = rest
                    .strip_prefix('/')
                    .map(|rest| rest.split_at(rest.find('/').unwrap_or(rest.len())))
                    .unwrap_or_default();
                if let Some(&place) = number
                    .parse::<usize>()
                    .ok()
                    .and_then(|at| at.checked_sub(self.explicit_connections))
                    .and_then(|at| self.derived.get(at))
                {
                    return format!("/architecture/connections/{place}{rest}");
                }
            }
            // Instances, hosts and instance connections are the view's own lists.
            for (list, own) in [
                ("components", "instances"),
                ("boundaries", "hosts"),
                ("connections", "instanceConnections"),
            ] {
                if let Some(rest) = path.strip_prefix(&format!("/architecture/{list}")) {
                    return format!("{view}/{own}{rest}");
                }
            }
            return path.to_owned();
        }
        if let Some(rest) = path.strip_prefix("/architecture/mainPath") {
            // A main path carried over from the model lies there; one of the view's own in it.
            return if self.own_main_path {
                format!("{view}/mainPath{rest}")
            } else {
                "/architecture/mainPath".to_owned()
            };
        }
        if let Some(rest) = path.strip_prefix("/architecture/sameRank/") {
            let (set, rest) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
            let Some((place, members)) = set
                .parse::<usize>()
                .ok()
                .and_then(|at| self.same_rank.get(at))
            else {
                return path.to_owned();
            };
            let Some(rest) = rest.strip_prefix('/') else {
                return format!("/architecture/sameRank/{place}{rest}");
            };
            let (member, rest) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
            return match member.parse::<usize>().ok().and_then(|at| members.get(at)) {
                Some(member) => format!("/architecture/sameRank/{place}/{member}{rest}"),
                None => format!("/architecture/sameRank/{place}"),
            };
        }
        let lists: [(&str, &[usize]); 4] = [
            ("components", &self.components),
            ("connections", &self.connections),
            ("boundaries", &self.boundaries),
            ("notes", &self.notes),
        ];
        for (list, places) in lists {
            let Some(rest) = path.strip_prefix(&format!("/architecture/{list}/")) else {
                continue;
            };
            let (number, rest) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
            let Some(&place) = number.parse::<usize>().ok().and_then(|at| places.get(at)) else {
                return path.to_owned();
            };
            if self.dynamic && list == "connections" && matches!(rest, "/label" | "/step") {
                let at: usize = number.parse().expect("parsed above");
                return format!("{view}/steps/{at}");
            }
            return format!("/architecture/{list}/{place}{rest}");
        }
        path.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ChartError, ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative,
    };

    /// A model of a blog with three views: an overview of what the reader reaches, where images
    /// go, and the way a comment takes.
    const MODEL: &str = r#"{
        "schemaVersion": 1,
        "type": "architecture",
        "title": "Blog",
        "description": "The whole blog.",
        "width": 1000,
        "height": 600,
        "mobile": {"width": 360, "height": 900},
        "architecture": {
            "boundaries": [
                {"id": "cloud", "label": "Cloud"},
                {"id": "private", "label": "Private network", "in": "cloud"},
                {"id": "mailzone", "label": "Mail zone"}
            ],
            "components": [
                {"id": "reader", "label": "Reader", "kind": "person"},
                {"id": "site", "label": "Site", "kind": "frontend"},
                {"id": "api", "label": "API", "in": "private"},
                {"id": "db", "label": "Posts", "kind": "database", "in": "private"},
                {"id": "files", "label": "Images", "kind": "storage", "in": "cloud"},
                {"id": "mail", "label": "Mail service", "kind": "external", "in": "mailzone"}
            ],
            "connections": [
                {"id": "read", "from": "reader", "to": "site", "label": "reads"},
                {"id": "load", "from": "site", "to": "api", "label": "loads", "technology": "HTTPS"},
                {"id": "query", "from": "api", "to": "db", "technology": "SQL"},
                {"id": "store", "from": "api", "to": "files", "label": "stores"},
                {"from": "api", "to": "mail", "label": "notifies"}
            ],
            "mainPath": ["reader", "site", "api", "db"],
            "views": [
                {"key": "overview", "title": "Blog overview",
                 "components": ["reader", "site", "api", "db"]},
                {"key": "media", "title": "Blog media", "description": "Where images go.",
                 "components": ["api", "files", "site"], "connections": ["store"]},
                {"key": "comment", "title": "Posting a comment",
                 "steps": [
                    {"connection": "load", "label": "posts comment"},
                    {"connection": "query", "label": "reads post"},
                    {"connection": "query", "label": "saves comment"}
                 ]}
            ]
        }
    }"#;

    fn with(view: Option<&str>) -> RenderOptions {
        RenderOptions {
            view: view.map(str::to_owned),
            ..RenderOptions::default()
        }
    }

    fn render(json: &str, view: Option<&str>) -> Result<crate::RenderOutput, ChartError> {
        render_json(json, RenderFormat::Html, &with(view))
    }

    fn error(json: &str) -> ChartError {
        ChartSpec::from_json(json)
            .and_then(|spec| spec.view(None))
            .expect_err("the specification is invalid")
    }

    /// The model with the first `from` replaced by `to`.
    fn edited(from: &str, to: &str) -> String {
        assert!(MODEL.contains(from), "{from} is not in the model");
        MODEL.replacen(from, to, 1)
    }

    fn architecture(spec: &ChartSpec) -> &super::ArchitectureSpec {
        spec.architecture.as_ref().expect("an architecture block")
    }

    fn ids<'a>(ids: impl Iterator<Item = &'a str>) -> String {
        ids.collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn a_view_shows_exactly_its_selection() {
        let spec = ChartSpec::from_json(MODEL).unwrap();
        let overview = spec.view(Some("overview")).unwrap();
        let shown = architecture(&overview);
        assert_eq!(
            ids(shown.components.iter().map(|c| c.id.as_str())),
            "reader site api db"
        );
        assert_eq!(
            ids(shown.connections.iter().filter_map(|c| c.id.as_deref())),
            "read load query"
        );
        // The empty mail zone is dropped, the cloud stays around the private network.
        assert_eq!(
            ids(shown.boundaries.iter().map(|b| b.id.as_str())),
            "cloud private"
        );
        assert!(
            !shown.main_path.is_empty(),
            "the model's main path lies in the view"
        );
        assert!(shown.views.is_empty());
        assert_eq!(overview.title, "Blog overview");
        assert_eq!(overview.description, None);

        // Listed connections only; the main path of the model does not lie in this view.
        let media = spec.view(Some("media")).unwrap();
        let shown = architecture(&media);
        assert_eq!(
            ids(shown.components.iter().map(|c| c.id.as_str())),
            "site api files"
        );
        assert_eq!(
            ids(shown.connections.iter().filter_map(|c| c.id.as_deref())),
            "store"
        );
        assert_eq!(shown.connections.len(), 1);
        assert!(shown.main_path.is_empty());
        assert_eq!(
            ids(shown.boundaries.iter().map(|b| b.id.as_str())),
            "cloud private"
        );
    }

    #[test]
    fn hidden_components_are_neither_drawn_nor_described() {
        let spec = ChartSpec::from_json(MODEL).unwrap();
        let output = render(MODEL, Some("overview")).unwrap().content;
        for hidden in [
            "Images",
            "Mail service",
            "Mail zone",
            "notifies",
            "The whole blog.",
        ] {
            assert!(!output.contains(hidden), "the overview shows {hidden}");
        }
        let alternative = text_alternative(&spec.view(Some("overview")).unwrap()).unwrap();
        assert!(
            alternative
                .description
                .contains("shows 4 of the model's 6 components and 3 of its 5 connections"),
            "{}",
            alternative.description
        );
        let rows: Vec<&str> = alternative
            .table
            .rows
            .iter()
            .map(|row| row[0].as_str())
            .collect();
        assert_eq!(rows.len(), 4);
        assert!(!rows.contains(&"Images"));
        assert_eq!(alternative.table.caption, "Data for Blog overview");
        // The text alternative of the model is that of its default view.
        assert_eq!(text_alternative(&spec).unwrap(), alternative);
    }

    #[test]
    fn a_changed_label_reaches_every_view() {
        let json = edited(r#""label": "API""#, r#""label": "Blog API""#);
        for view in ["overview", "media", "comment"] {
            let svg = render_json(&json, RenderFormat::Svg, &with(Some(view)))
                .unwrap()
                .content;
            assert!(svg.contains("Blog API"), "{view} misses the new label");
        }
    }

    #[test]
    fn a_dynamic_view_numbers_every_occurrence() {
        let spec = ChartSpec::from_json(MODEL).unwrap();
        let comment = spec.view(Some("comment")).unwrap();
        let shown = architecture(&comment);
        assert_eq!(
            ids(shown.components.iter().map(|c| c.id.as_str())),
            "site api db"
        );
        let steps: Vec<(Option<u8>, Option<&str>, Option<&str>)> = shown
            .connections
            .iter()
            .map(|c| (c.step, c.label.as_deref(), c.id.as_deref()))
            .collect();
        assert_eq!(
            steps,
            [
                (Some(1), Some("posts comment"), Some("load")),
                (Some(2), Some("reads post"), Some("query")),
                (Some(3), Some("saves comment"), None),
            ]
        );
        let svg = render_json(MODEL, RenderFormat::Svg, &with(Some("comment")))
            .unwrap()
            .content;
        assert!(svg.contains("2. reads post") && svg.contains("3. saves comment"));
    }

    #[test]
    fn the_default_view_is_the_first_or_the_named_one() {
        let first = render(MODEL, None).unwrap().content;
        assert_eq!(first, render(MODEL, Some("overview")).unwrap().content);
        let json = edited(r#""views": ["#, r#""defaultView": "media", "views": ["#);
        let named = render(&json, None).unwrap().content;
        assert_eq!(named, render(&json, Some("media")).unwrap().content);
        assert!(named.contains("Where images go."));
    }

    #[test]
    fn views_render_the_same_bytes_and_their_own_ids() {
        let once = render(MODEL, Some("media")).unwrap().content;
        assert_eq!(once, render(MODEL, Some("media")).unwrap().content);
        let prefix = |content: &str| {
            let start = content.find("id=\"chartlet-").unwrap() + 4;
            content[start..start + 25].to_owned()
        };
        assert_ne!(
            prefix(&once),
            prefix(&render(MODEL, Some("overview")).unwrap().content)
        );
    }

    #[test]
    fn the_manifest_names_the_view() {
        let options = RenderOptions {
            manifest: true,
            ..with(Some("comment"))
        };
        let manifest = render_json(MODEL, RenderFormat::Svg, &options)
            .unwrap()
            .manifest
            .unwrap();
        assert_eq!(manifest.view.as_deref(), Some("comment"));
        assert!(manifest.to_json().contains("\"view\": \"comment\""));
        let options = RenderOptions {
            manifest: true,
            ..RenderOptions::default()
        };
        let default = render_json(MODEL, RenderFormat::Svg, &options)
            .unwrap()
            .manifest
            .unwrap();
        assert_eq!(default.view.as_deref(), Some("overview"));
        // The hash is that of the whole model, whichever view was rendered.
        assert_eq!(default.spec_hash, manifest.spec_hash);
    }

    #[test]
    fn every_variant_renders_the_view() {
        for variant in [crate::Variant::Mobile, crate::Variant::Print] {
            let options = RenderOptions {
                variant,
                ..with(Some("media"))
            };
            let svg = render_json(MODEL, RenderFormat::Svg, &options)
                .unwrap()
                .content;
            assert!(svg.contains("Blog media"), "{variant:?}");
            assert!(
                !svg.contains("Posts"),
                "{variant:?} shows a hidden component"
            );
        }
    }

    #[test]
    fn a_specification_without_views_has_none_to_select() {
        let plain = r#"{"schemaVersion": 1, "type": "architecture", "title": "A",
            "architecture": {"components": [{"id": "a", "label": "A"}]}}"#;
        let error = render(plain, Some("overview")).unwrap_err();
        assert_eq!(
            (error.code, error.path.as_str()),
            ("unknown_view", "/render/view")
        );
        let options = RenderOptions {
            manifest: true,
            ..RenderOptions::default()
        };
        let manifest = render_json(plain, RenderFormat::Svg, &options)
            .unwrap()
            .manifest
            .unwrap();
        assert_eq!(manifest.view, None);
        assert!(!manifest.to_json().contains("\"view\""));
        let spec = ChartSpec::from_json(plain).unwrap();
        assert!(spec.view_keys().is_empty());
        assert_eq!(
            serde_json::to_string(&spec.view(None).unwrap()).unwrap(),
            serde_json::to_string(&spec).unwrap()
        );
    }

    #[test]
    fn unknown_views_and_references_are_reported() {
        let unknown = render(MODEL, Some("deploy")).unwrap_err();
        assert_eq!(
            (unknown.code, unknown.path.as_str()),
            ("unknown_view", "/render/view")
        );
        assert!(
            unknown
                .message
                .contains("\"overview\", \"media\", \"comment\"")
        );

        let cases = [
            (
                edited(
                    r#""reader", "site", "api", "db"]},"#,
                    r#""reader", "web"]},"#,
                ),
                "unknown_node",
                "/architecture/views/0/components/1",
            ),
            (
                edited(r#""connections": ["store"]"#, r#""connections": ["save"]"#),
                "unknown_connection",
                "/architecture/views/1/connections/0",
            ),
            (
                edited(r#""connections": ["store"]"#, r#""connections": ["read"]"#),
                "connection_outside_view",
                "/architecture/views/1/connections/0",
            ),
            (
                edited(
                    r#"{"connection": "load", "label""#,
                    r#"{"connection": "lode", "label""#,
                ),
                "unknown_connection",
                "/architecture/views/2/steps/0/connection",
            ),
            (
                edited(
                    r#""components": ["reader", "site", "api", "db"]"#,
                    r#""components": []"#,
                ),
                "empty_view",
                "/architecture/views/0",
            ),
            (
                edited(r#""key": "media""#, r#""key": "overview""#),
                "duplicate_id",
                "/architecture/views/1/key",
            ),
            (
                edited(r#""id": "load""#, r#""id": "read""#),
                "duplicate_id",
                "/architecture/connections/1/id",
            ),
            (
                edited(r#""views": ["#, r#""defaultView": "deploy", "views": ["#),
                "unknown_view",
                "/architecture/defaultView",
            ),
            (
                edited(
                    r#""title": "Posting a comment","#,
                    r#""title": "Posting", "connections": [],"#,
                ),
                "option_not_supported",
                "/architecture/views/2/connections",
            ),
            (
                edited(
                    r#""title": "Blog media","#,
                    r#""title": "Blog media", "mainPath": ["site", "files"],"#,
                ),
                "main_path_gap",
                "/architecture/views/1/mainPath/1",
            ),
            (
                edited(r#""title": "Blog overview""#, r#""title": " ""#),
                "empty_text",
                "/architecture/views/0/title",
            ),
            (
                edited(
                    r#""title": "Blog media","#,
                    r#""title": "Blog media", "perspective": "team","#,
                ),
                "unknown_perspective",
                "/architecture/views/1/perspective",
            ),
        ];
        for (json, code, path) in cases {
            let error = error(&json);
            assert_eq!(
                (error.code, error.path.as_str()),
                (code, path),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn a_view_keeps_shown_members_of_shared_ranks() {
        let json = edited(
            r#""mainPath": ["reader", "site", "api", "db"],"#,
            r#""mainPath": ["reader", "site", "api", "db"],
               "sameRank": [["reader", "api"], ["mail", "db", "files"]],"#,
        );
        let spec = ChartSpec::from_json(&json).unwrap();
        let media = spec.view(Some("media")).unwrap();
        // Of the first set only api is shown, of the second only files: both are dropped.
        assert!(architecture(&media).same_rank.is_empty());
        let json = json.replace(
            r#"["api", "files", "site"]"#,
            r#"["api", "files", "site", "db"]"#,
        );
        let spec = ChartSpec::from_json(&json).unwrap();
        let media = spec.view(Some("media")).unwrap();
        assert_eq!(architecture(&media).same_rank, [["db", "files"]]);
        spec.validate().unwrap();
        let resolved = spec.resolve_view(Some("media")).unwrap().unwrap();
        // The view's only set is the model's second, without its first member.
        assert_eq!(
            resolved.model_path("/architecture/sameRank/0/0"),
            "/architecture/sameRank/1/1"
        );
        assert_eq!(
            resolved.model_path("/architecture/sameRank/0/1"),
            "/architecture/sameRank/1/2"
        );
        assert_eq!(
            resolved.model_path("/architecture/sameRank/0"),
            "/architecture/sameRank/1"
        );
    }

    #[test]
    fn a_view_keeps_the_notes_on_shown_components() {
        let json = edited(
            r#""mainPath": ["reader", "site", "api", "db"],"#,
            r#""mainPath": ["reader", "site", "api", "db"],
               "notes": [
                   {"on": "db", "text": "Replaced next year"},
                   {"on": "files", "text": "Kept for ten years"}
               ],"#,
        );
        let spec = ChartSpec::from_json(&json).unwrap();
        spec.validate().unwrap();
        let ons = |key: &str| -> Vec<String> {
            let view = spec.view(Some(key)).unwrap();
            architecture(&view)
                .notes
                .iter()
                .map(|note| note.on.clone())
                .collect()
        };
        assert_eq!(ons("overview"), ["db"]);
        assert_eq!(ons("media"), ["files"]);
        let svg = render(&json, Some("media")).unwrap().content;
        assert!(svg.contains(">Kept for ten years<"));
        assert!(!svg.contains("Replaced next year"));
        // The media view keeps only the model's second note.
        let media = spec.resolve_view(Some("media")).unwrap().unwrap();
        assert_eq!(
            media.model_path("/architecture/notes/0/text"),
            "/architecture/notes/1/text"
        );
        let unknown = json.replace(r#"{"on": "files""#, r#"{"on": "cloud""#);
        let error = error(&unknown);
        assert_eq!(
            (error.code, error.path.as_str()),
            ("unknown_node", "/architecture/notes/1/on")
        );
    }

    #[test]
    fn a_view_keeps_roles_symbols_and_node_hooks() {
        let json = edited(
            r#"{"id": "api", "label": "API", "in": "private"}"#,
            r#"{"id": "api", "label": "API", "in": "private", "role": "critical", "symbol": "container"}"#,
        );
        let full = ChartSpec::from_json(&json).unwrap();
        let model = architecture(&full).components[2].clone();
        let overview = full.view(Some("overview")).unwrap();
        let shown = &architecture(&overview).components[2];
        assert_eq!(shown.id, "api");
        assert_eq!(shown.role, model.role);
        assert_eq!(shown.symbol, model.symbol);
        assert!(shown.role.is_some() && shown.symbol.is_some());
        let options = RenderOptions {
            hooks: true,
            ..with(Some("overview"))
        };
        let svg = render_json(&json, RenderFormat::Svg, &options)
            .unwrap()
            .content;
        assert!(
            svg.contains(r#"data-chartlet-node="api""#),
            "no node hook for api"
        );
        assert!(!svg.contains(r#"data-chartlet-node="files""#));
    }

    #[test]
    fn a_view_shows_its_own_perspective_or_the_model_s() {
        let json = edited(
            r#""mainPath": ["#,
            r#""perspectives": [
                {"key": "team", "title": "Team", "categories": [
                    {"label": "Web", "components": ["site", "files"]},
                    {"label": "Core", "components": ["api", "mail"]}]},
                {"key": "debt", "title": "Debt", "categories": [
                    {"label": "low", "components": ["files"]},
                    {"label": "high", "components": ["api"]}]}
               ], "perspective": "team", "mainPath": ["#,
        )
        .replacen(
            r#""title": "Blog media","#,
            r#""title": "Blog media", "perspective": "debt","#,
            1,
        );
        let spec = ChartSpec::from_json(&json).unwrap();
        let shown = |view: &str| {
            architecture(&spec.view(Some(view)).unwrap())
                .perspective
                .clone()
        };
        assert_eq!(shown("overview").as_deref(), Some("team"));
        assert_eq!(shown("media").as_deref(), Some("debt"));
        let media = render(&json, Some("media")).unwrap().content;
        assert!(media.contains(">Debt:</text>") && !media.contains(">Team:</text>"));
        // A category keeps only the components the view shows.
        let overview = spec.view(Some("overview")).unwrap();
        let team = &architecture(&overview).perspectives[0];
        assert_eq!(team.categories[0].components, ["site"]);
        assert_eq!(team.categories[1].components, ["api"]);
    }

    #[test]
    fn a_render_option_shows_its_perspective_in_every_view() {
        let json = include_str!("../../examples/service-perspectives.json");
        let options = |view: &str, perspective: Option<&str>| RenderOptions {
            perspective: perspective.map(str::to_owned),
            manifest: true,
            ..with(Some(view))
        };
        let debt = render_json(json, RenderFormat::Svg, &options("debt", None)).unwrap();
        let owners =
            render_json(json, RenderFormat::Svg, &options("owners", Some("debt"))).unwrap();
        assert!(owners.content.contains(">Technical debt:</text>"));
        assert!(!owners.content.contains(">Team:</text>"));
        let manifest = owners.manifest.unwrap();
        assert_eq!(manifest.perspective.as_deref(), Some("debt"));
        assert_eq!(manifest.view.as_deref(), Some("owners"));
        // The hash is that of the model as given.
        assert_eq!(manifest.spec_hash, debt.manifest.unwrap().spec_hash);
        // On the page of all views, every view shows the perspective asked for.
        let page = render_json(
            json,
            RenderFormat::Html,
            &RenderOptions {
                perspective: Some("debt".to_owned()),
                all_views: true,
                manifest: true,
                ..RenderOptions::default()
            },
        )
        .unwrap();
        assert!(page.content.matches(">Technical debt:</text>").count() >= 2);
        assert!(!page.content.contains(">Team:</text>"));
        let manifest = page.manifest.unwrap();
        assert_eq!(manifest.perspective.as_deref(), Some("debt"));
        // The same perspective asked for again changes nothing.
        let same = render_json(json, RenderFormat::Svg, &options("debt", Some("debt"))).unwrap();
        assert_eq!(same.content, debt.content);
        // A plain diagram with perspectives but none selected shows the one asked for.
        let mut plain = ChartSpec::from_json(json)
            .unwrap()
            .view(Some("debt"))
            .unwrap();
        plain.architecture.as_mut().unwrap().perspective = None;
        let plain = serde_json::to_string(&plain).unwrap();
        let shown = render_json(
            &plain,
            RenderFormat::Svg,
            &RenderOptions {
                perspective: Some("debt".to_owned()),
                ..RenderOptions::default()
            },
        )
        .unwrap();
        assert!(shown.content.contains(">Technical debt:</text>"));

        let error =
            render_json(json, RenderFormat::Svg, &options("owners", Some("cost"))).unwrap_err();
        assert_eq!(
            (error.code, error.path.as_str()),
            ("unknown_perspective", "/render/perspective")
        );
        assert!(
            error
                .message
                .contains("the perspectives are \"owner\", \"debt\""),
            "{}",
            error.message
        );
        let error = ChartSpec::from_json(MODEL)
            .unwrap()
            .with_perspective("cost")
            .unwrap_err();
        assert_eq!(error.code, "unknown_perspective");
        assert!(
            error
                .message
                .contains("the specification has no perspectives")
        );
    }

    #[test]
    fn a_view_keeps_the_categories_of_the_connections_it_shows() {
        let json = edited(
            r#""mainPath": ["#,
            r#""perspectives": [
                {"key": "team", "title": "Team", "categories": [
                    {"label": "Web", "connections": ["read", "store"]},
                    {"label": "Data", "components": ["db"], "connections": ["query"]}]}
               ], "perspective": "team", "mainPath": ["#,
        );
        let spec = ChartSpec::from_json(&json).unwrap();
        let media = spec.view(Some("media")).unwrap();
        let team = &architecture(&media).perspectives[0];
        assert_eq!(team.categories[0].connections, ["store"]);
        assert!(team.categories[1].connections.is_empty());
        // Every occurrence of a connection in a dynamic view is in its category.
        let svg = render_json(&json, RenderFormat::Svg, &with(Some("comment")))
            .unwrap()
            .content;
        assert!(svg.contains(">2. reads post · Data</text>"), "{svg}");
        assert!(svg.contains(">3. saves comment · Data</text>"));
        assert_eq!(svg.matches(" chartlet-flow-part-2\"").count(), 2);
        assert!(!svg.contains(">Web</text>"));
        let alternative = text_alternative(&spec.view(Some("comment")).unwrap()).unwrap();
        assert!(
            alternative
                .description
                .contains("Data: Posts, API → Posts;"),
            "{}",
            alternative.description
        );
    }

    #[test]
    fn a_view_may_set_its_own_size() {
        let json = edited(
            r#""title": "Blog media","#,
            r#""title": "Blog media", "width": 720, "height": 480,
               "mobile": {"width": 340, "height": 700},"#,
        );
        let spec = ChartSpec::from_json(&json).unwrap();
        let media = spec.view(Some("media")).unwrap();
        assert_eq!((media.width, media.height), (720, 480));
        assert_eq!(media.mobile.as_ref().map(|mobile| mobile.width), Some(340));
        // The other views keep the model's size.
        let overview = spec.view(Some("overview")).unwrap();
        assert_eq!((overview.width, overview.height), (1000, 600));
        assert_eq!(
            overview.mobile.as_ref().map(|mobile| mobile.width),
            Some(360)
        );
        let svg = render_json(&json, RenderFormat::Svg, &with(Some("media")))
            .unwrap()
            .content;
        assert!(svg.contains(r#"viewBox="0 0 720 "#), "{svg}");
        let mobile = render_json(
            &json,
            RenderFormat::Svg,
            &RenderOptions {
                variant: crate::Variant::Mobile,
                ..with(Some("media"))
            },
        )
        .unwrap()
        .content;
        assert!(mobile.contains(r#"viewBox="0 0 340 "#));
        // On the page of all views, each section has the size of its view.
        let page = render_json(
            &json,
            RenderFormat::Html,
            &RenderOptions {
                all_views: true,
                ..RenderOptions::default()
            },
        )
        .unwrap()
        .content;
        assert!(page.contains(r#"viewBox="0 0 720 "#) && page.contains(r#"viewBox="0 0 1000 "#));
        assert!(page.contains(r#"viewBox="0 0 340 "#) && page.contains(r#"viewBox="0 0 360 "#));
        // Without a size of its own, a view renders the bytes it did.
        assert_eq!(
            render(&json, Some("overview")).unwrap().content,
            render(MODEL, Some("overview")).unwrap().content
        );

        for (field, path) in [
            (r#""width": 100,"#, "/architecture/views/1/width"),
            (r#""height": 100,"#, "/architecture/views/1/height"),
            (
                r#""mobile": {"width": 100},"#,
                "/architecture/views/1/mobile/width",
            ),
        ] {
            let json = edited(
                r#""title": "Blog media","#,
                &format!(r#""title": "Blog media", {field}"#),
            );
            let error = error(&json);
            assert_eq!(error.path, path, "{}: {}", error.code, error.message);
        }
        // The model's own size is checked at the model's.
        let error = error(&edited(r#""width": 1000"#, r#""width": 100"#));
        assert_eq!(error.path, "/width");
    }

    /// The model with tags on its components and on one connection, and `views` in place of its
    /// own.
    fn tagged(views: &str) -> String {
        let mut json = MODEL.to_owned();
        for (component, tags) in [
            ("reader", r#"["public"]"#),
            ("site", r#"["public"]"#),
            ("api", r#"["core"]"#),
            ("db", r#"["core", "data"]"#),
            ("files", r#"["data"]"#),
            ("mail", r#"["external"]"#),
        ] {
            let from = format!(r#"{{"id": "{component}", "#);
            json = json.replacen(
                &from,
                &format!(r#"{{"tags": {tags}, "id": "{component}", "#),
                1,
            );
        }
        let json = json.replacen(
            r#"{"id": "load", "#,
            r#"{"tags": ["sync"], "id": "load", "#,
            1,
        );
        let start = json.find(r#""views": ["#).unwrap();
        let end = json.rfind(']').unwrap();
        format!("{}\"views\": [{views}{}", &json[..start], &json[end..])
    }

    #[test]
    fn tags_select_and_leave_out_by_one_rule() {
        let json = tagged(
            r#"{"key": "public", "title": "Public", "include": ["public"]},
               {"key": "core", "title": "Core", "components": ["site"], "include": ["core"], "exclude": ["data"]},
               {"key": "inner", "title": "Inner", "exclude": ["external", "public"]},
               {"key": "quiet", "title": "Quiet", "components": ["reader", "site", "api"], "exclude": ["sync"]},
               {"key": "listed", "title": "Listed", "components": ["site", "api", "db"], "connections": ["query"], "include": ["sync"]},
               {"key": "kept", "title": "Kept", "components": ["db"], "include": ["core"], "exclude": ["data"]}"#,
        );
        let spec = ChartSpec::from_json(&json).unwrap();
        let shown = |key: &str| {
            let view = spec.view(Some(key)).unwrap();
            let shown = architecture(&view);
            (
                ids(shown.components.iter().map(|c| c.id.as_str())),
                ids(shown.connections.iter().filter_map(|c| c.id.as_deref())),
            )
        };
        let pair =
            |components: &str, connections: &str| (components.to_owned(), connections.to_owned());
        // Included by tag; the connections between them come along.
        assert_eq!(shown("public"), pair("reader site", "read"));
        // Listed, or included and not excluded.
        assert_eq!(shown("core"), pair("site api", "load"));
        // Exclude alone starts from every component.
        assert_eq!(shown("inner"), pair("api db files", "query store"));
        // A connection with an excluded tag is left out.
        assert_eq!(shown("quiet"), pair("reader site api", "read"));
        // With listed connections, include adds the tagged ones between shown components.
        assert_eq!(shown("listed"), pair("site api db", "load query"));
        // A listed component is shown although an exclude tag would leave it out.
        assert_eq!(shown("kept"), pair("api db", "query"));
        // Tags are not drawn: with one ID prefix, a view that does not select by them renders
        // the bytes it did.
        let fixed = |json: &str| {
            let options = RenderOptions {
                id_prefix: Some("blog".to_owned()),
                ..RenderOptions::default()
            };
            render_json(json, RenderFormat::Html, &options)
                .unwrap()
                .content
        };
        let overview = r#"{"key": "overview", "title": "Blog overview",
            "components": ["reader", "site", "api", "db"]}"#;
        assert_eq!(fixed(&tagged(overview)), fixed(MODEL));
    }

    #[test]
    fn tags_are_checked_in_the_model_and_the_view() {
        for (views, code, path) in [
            (
                r#"{"key": "a", "title": "A", "include": ["private"]}"#,
                "unknown_tag",
                "/architecture/views/0/include/0",
            ),
            (
                r#"{"key": "a", "title": "A", "include": ["core"], "exclude": ["core"]}"#,
                "duplicate_tag",
                "/architecture/views/0/exclude/0",
            ),
            (
                r#"{"key": "a", "title": "A", "include": ["1st"]}"#,
                "invalid_tag",
                "/architecture/views/0/include/0",
            ),
            (
                r#"{"key": "a", "title": "A", "include": ["sync"]}"#,
                "empty_view",
                "/architecture/views/0",
            ),
            (
                r#"{"key": "a", "title": "A", "include": ["public"], "connections": ["load"]}"#,
                "connection_outside_view",
                "/architecture/views/0/connections/0",
            ),
            (
                r#"{"key": "a", "title": "A", "steps": [{"connection": "load"}], "exclude": ["data"]}"#,
                "option_not_supported",
                "/architecture/views/0/exclude",
            ),
        ] {
            let error = error(&tagged(views));
            assert_eq!(
                (error.code, error.path.as_str()),
                (code, path),
                "{}",
                error.message
            );
        }
        let view = r#"{"key": "a", "title": "A", "include": ["public"]}"#;
        for (tags, code, path) in [
            (
                r#"["public", "1st"]"#,
                "invalid_tag",
                "/architecture/components/0/tags/1",
            ),
            (
                r#"["public", "public"]"#,
                "duplicate_tag",
                "/architecture/components/0/tags/1",
            ),
            (
                r#"["public", "a", "b", "c", "d", "e", "f", "g", "h"]"#,
                "too_many_tags",
                "/architecture/components/0/tags",
            ),
        ] {
            let json = tagged(view).replacen(r#"["public"]"#, tags, 1);
            let error = error(&json);
            assert_eq!(
                (error.code, error.path.as_str()),
                (code, path),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn diagnostics_of_a_view_point_into_the_model() {
        let spec = ChartSpec::from_json(MODEL).unwrap();
        spec.validate().unwrap();
        // The view shows site, api and files: the model's components 1, 2 and 4.
        let media = spec.resolve_view(Some("media")).unwrap().unwrap();
        for (shown, model) in [
            (
                "/architecture/components/2/label",
                "/architecture/components/4/label",
            ),
            (
                "/architecture/connections/0/label",
                "/architecture/connections/3/label",
            ),
            ("/architecture/boundaries/1", "/architecture/boundaries/1"),
            ("/title", "/architecture/views/1/title"),
            ("/width", "/width"),
        ] {
            assert_eq!(media.model_path(shown), model);
        }
        let comment = spec.resolve_view(Some("comment")).unwrap().unwrap();
        for (shown, model) in [
            (
                "/architecture/connections/2/label",
                "/architecture/views/2/steps/2",
            ),
            (
                "/architecture/connections/2/technology",
                "/architecture/connections/2/technology",
            ),
        ] {
            assert_eq!(comment.model_path(shown), model);
        }
    }
}
