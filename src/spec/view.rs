//! Views on the model of an architecture diagram. A view selects components and connections of
//! the model and resolves to an architecture diagram of its own, which is validated, laid out
//! and described like any other. Nothing outside the selection is drawn or described, and no
//! connection is drawn that the model does not have.

use serde::{Deserialize, Serialize};

use super::{ArchitectureSpec, ChartSpec, MainPath, sequence::is_identifier, validate_text};
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
    index: usize,
    components: Vec<usize>,
    connections: Vec<usize>,
    boundaries: Vec<usize>,
    /// The shared ranks the view keeps: the place of each in the model and the places of its
    /// shown members in it.
    same_rank: Vec<(usize, Vec<usize>)>,
    /// The notes the view keeps, those on shown components, by their places in the model.
    notes: Vec<usize>,
    /// The connections are the steps of the view, in order.
    dynamic: bool,
    own_main_path: bool,
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
    fn resolved(&self, index: usize) -> ResolvedView {
        let model = self
            .architecture
            .as_ref()
            .expect("only architecture diagrams have views");
        let view = &model.views[index];
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
                        connection.id = None;
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
            if view.components.is_empty() {
                steps.iter().any(|&step| {
                    let connection = &model.connections[step];
                    &connection.from == id || &connection.to == id
                })
            } else {
                view.components.contains(id)
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

        let connections: Vec<usize> = if dynamic {
            steps.clone()
        } else if let Some(ids) = &view.connections {
            (0..model.connections.len())
                .filter(|&index| {
                    model.connections[index]
                        .id
                        .as_ref()
                        .is_some_and(|id| ids.contains(id))
                })
                .collect()
        } else {
            (0..model.connections.len())
                .filter(|&index| {
                    let connection = &model.connections[index];
                    is_shown(&connection.from) && is_shown(&connection.to)
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
        if view.components.is_empty() && view.steps.is_empty() {
            return Err(ChartError::new(
                "empty_view",
                path,
                "a view shows at least one component; list its components or its steps",
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
        if view.components.is_empty() {
            return Ok(());
        }
        for end in [&connection.from, &connection.to] {
            if !view.components.contains(end) {
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
