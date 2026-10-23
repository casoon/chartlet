//! Deployment views: one environment of an architecture model, such as a developer's machine or
//! production, as the running instances of the model's components on hosts. The view resolves
//! to an architecture diagram of its own whose components are the instances, whose boundaries
//! are the hosts and whose connections are only those the view lists; limits and diagnostics are
//! those of that diagram, pointing into the view.

use super::{
    ArchitectureSpec, ChartSpec, MainPath, ViewSpec, sequence::is_identifier, validate_text,
    view::ResolvedView,
};
use crate::error::ChartError;

impl ChartSpec {
    /// The deployment view at `index` of a model whose views were checked.
    pub(super) fn resolved_deployment(&self, index: usize) -> ResolvedView {
        let model = self
            .architecture
            .as_ref()
            .expect("only architecture diagrams have views");
        let view = &model.views[index];
        let logical = |id: &str| model.component(id).expect("checked instances");
        let mut architecture = model.clone();
        architecture.components = view
            .instances
            .iter()
            .map(|instance| {
                let mut component = model.components[logical(&instance.of)].clone();
                // Two replicas share the label of their component; the id tells them apart.
                component.label = format!("{} ({})", component.label, instance.id);
                component.id.clone_from(&instance.id);
                component.boundary.clone_from(&instance.host);
                component
            })
            .collect();
        architecture.connections = view
            .instance_connections
            .iter()
            .enumerate()
            .map(|(at, connection)| {
                let mut connection = connection.clone();
                if let Some(of) = connection.id.as_deref().and_then(|id| {
                    model
                        .connections
                        .iter()
                        .find(|other| other.id.as_deref() == Some(id))
                }) {
                    // What the instance connection leaves out, it takes from the model's.
                    connection.label = connection.label.or_else(|| of.label.clone());
                    connection.technology = connection.technology.or_else(|| of.technology.clone());
                    connection.dash = connection.dash.or(of.dash);
                    connection.role = connection.role.or(of.role);
                    // A model connection carried by several instances keeps its id once.
                    if view.instance_connections[..at]
                        .iter()
                        .any(|earlier| earlier.id == connection.id)
                    {
                        connection.model_id = connection.id.take();
                    }
                }
                connection
            })
            .collect();
        architecture.boundaries.clone_from(&view.hosts);
        architecture.main_path = MainPath::default();
        architecture.same_rank = Vec::new();
        architecture.notes = Vec::new();
        architecture.views = Vec::new();
        architecture.default_view = None;
        architecture.perspective = view.perspective.clone().or(architecture.perspective);
        // A category holds the instances of its components, and the model connections that
        // instance connections carry.
        for perspective in &mut architecture.perspectives {
            for category in &mut perspective.categories {
                category.components = view
                    .instances
                    .iter()
                    .filter(|instance| category.components.contains(&instance.of))
                    .map(|instance| instance.id.clone())
                    .collect();
                category.connections.retain(|id| {
                    view.instance_connections
                        .iter()
                        .any(|connection| connection.id.as_ref() == Some(id))
                });
            }
        }
        let (components, connections, _) = model.deployment_selection(view);
        architecture.environment = Some((
            view.environment.clone().expect("a deployment view"),
            components.len(),
        ));

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
            boundaries: Vec::new(),
            same_rank: Vec::new(),
            notes: Vec::new(),
            dynamic: false,
            own_main_path: false,
            deployment: true,
            own_size,
        }
    }
}

/// A view without an environment, which has no hosts, instances or instance connections.
pub(super) fn reject_deployment_fields(view: &ViewSpec, path: &str) -> Result<(), ChartError> {
    match [
        (!view.hosts.is_empty(), "hosts"),
        (!view.instances.is_empty(), "instances"),
        (!view.instance_connections.is_empty(), "instanceConnections"),
    ]
    .into_iter()
    .find_map(|(given, field)| given.then_some(field))
    {
        Some(field) => Err(ChartError::new(
            "option_not_supported",
            format!("{path}/{field}"),
            "only a deployment view, one with an environment, has hosts, instances and instance connections",
        )),
        None => Ok(()),
    }
}

impl ArchitectureSpec {
    /// What a deployment view shows of the model: the components its instances run and the
    /// connections its instance connections name, by their places in the model.
    pub(super) fn deployment_selection(
        &self,
        view: &ViewSpec,
    ) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        let components = (0..self.components.len())
            .filter(|&index| {
                view.instances
                    .iter()
                    .any(|instance| instance.of == self.components[index].id)
            })
            .collect();
        let connections = (0..self.connections.len())
            .filter(|&index| {
                self.connections[index].id.is_some()
                    && view
                        .instance_connections
                        .iter()
                        .any(|connection| connection.id == self.connections[index].id)
            })
            .collect();
        (components, connections, Vec::new())
    }

    /// A deployment view lists instances of components the model has, in place of components,
    /// connections, steps and a main path; an instance connection with an `id` names a model
    /// connection between the components its ends run.
    pub(super) fn validate_deployment(
        &self,
        view: &ViewSpec,
        path: &str,
    ) -> Result<(), ChartError> {
        validate_text(
            view.environment.as_deref().expect("a deployment view"),
            &format!("{path}/environment"),
            60,
        )?;
        if let Some(field) = [
            (!view.components.is_empty(), "components"),
            (view.connections.is_some(), "connections"),
            (!view.steps.is_empty(), "steps"),
            (!view.main_path.is_empty(), "mainPath"),
        ]
        .into_iter()
        .find_map(|(given, field)| given.then_some(field))
        {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                "a deployment view shows instances, hosts and instance connections; leave out components, connections, steps and mainPath",
            ));
        }
        if view.instances.is_empty() {
            return Err(ChartError::new(
                "empty_view",
                path,
                "a deployment view shows at least one instance; list its instances",
            ));
        }
        for (at, instance) in view.instances.iter().enumerate() {
            let Some(component) = self.component(&instance.of) else {
                return Err(ChartError::new(
                    "unknown_node",
                    format!("{path}/instances/{at}/of"),
                    format!("no component has the id \"{}\"", instance.of),
                ));
            };
            // The instance is drawn as "Label (id)", which has to fit like any label.
            let label = format!("{} ({})", self.components[component].label, instance.id);
            if is_identifier(&instance.id) && label.chars().count() > 60 {
                return Err(ChartError::new(
                    "text_too_long",
                    format!("{path}/instances/{at}/id"),
                    format!(
                        "the instance is labelled \"{label}\", the label of its component and its id, which must not exceed 60 characters; shorten the id or the component's label"
                    ),
                ));
            }
        }
        let of = |id: &str| {
            view.instances
                .iter()
                .find(|instance| instance.id == id)
                .map(|instance| instance.of.as_str())
        };
        for (at, connection) in view.instance_connections.iter().enumerate() {
            let Some(id) = &connection.id else {
                continue;
            };
            let at = format!("{path}/instanceConnections/{at}/id");
            let Some(model) = self
                .connections
                .iter()
                .find(|other| other.id.as_ref() == Some(id))
            else {
                return Err(ChartError::new(
                    "unknown_connection",
                    at,
                    format!("no connection of the model has the id \"{id}\""),
                ));
            };
            // Ends that are no instances are reported by the diagram of the view.
            let (Some(from), Some(to)) = (of(&connection.from), of(&connection.to)) else {
                continue;
            };
            if (from, to) != (model.from.as_str(), model.to.as_str()) {
                return Err(ChartError::new(
                    "instance_mismatch",
                    at,
                    format!(
                        "the connection \"{id}\" joins \"{}\" to \"{}\"; its instance connection must join instances of them",
                        model.from, model.to
                    ),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ChartSpec, RenderFormat, RenderOptions, inspect_json, render_json, text_alternative,
    };

    const MODEL: &str = include_str!("../../examples/api-environments.json");

    fn view(json: &str, key: &str) -> ChartSpec {
        ChartSpec::from_json(json).unwrap().view(Some(key)).unwrap()
    }

    fn error(json: &str) -> (&'static str, String) {
        let error = ChartSpec::from_json(json)
            .and_then(|spec| spec.view(None))
            .expect_err("the specification is invalid");
        (error.code, error.path)
    }

    fn edited(from: &str, to: &str) -> String {
        assert!(MODEL.contains(from), "{from} is not in the model");
        MODEL.replacen(from, to, 1)
    }

    #[test]
    fn an_environment_resolves_to_its_instances_hosts_and_listed_connections() {
        let production = view(MODEL, "production");
        let architecture = production.architecture.as_ref().unwrap();
        let labels: Vec<&str> = architecture
            .components
            .iter()
            .map(|component| component.label.as_str())
            .collect();
        assert_eq!(
            labels,
            [
                "Load balancer (lb-prod)",
                "Orders API (api-1)",
                "Orders API (api-2)",
                "Orders (db-prod)"
            ]
        );
        assert_eq!(architecture.boundaries.len(), 4);
        assert_eq!(architecture.connections.len(), 4);
        // The connection to the front end, which the view does not list, is not drawn.
        assert!(
            architecture
                .connections
                .iter()
                .all(|connection| connection.from != "client" && connection.to != "client")
        );
        // Labels and technologies come from the model; a repeated id is kept once.
        assert_eq!(architecture.connections[1].label.as_deref(), Some("routes"));
        assert_eq!(architecture.connections[1].id, None);
        let alternative = text_alternative(&production).unwrap();
        assert!(
            alternative
                .description
                .contains("environment Production: 4 instances of 3 components"),
            "{}",
            alternative.description
        );
        assert_eq!(alternative.table.columns[0], "Instance");
        // The plain specification of the environment renders like any other.
        let json = serde_json::to_string(&production).unwrap();
        render_json(&json, RenderFormat::Svg, &RenderOptions::default()).unwrap();
    }

    #[test]
    fn a_renamed_component_reaches_every_environment() {
        let json = edited(r#""label": "Orders API""#, r#""label": "Order service""#);
        for key in ["local", "production"] {
            let svg = render_json(
                &json,
                RenderFormat::Svg,
                &RenderOptions {
                    view: Some(key.to_owned()),
                    ..RenderOptions::default()
                },
            )
            .unwrap()
            .content;
            assert!(svg.contains("Order service ("), "{key}");
        }
    }

    #[test]
    fn deployment_views_are_checked_in_the_view() {
        let cases = [
            (
                edited(
                    r#""of": "api", "in": "node-a""#,
                    r#""of": "app", "in": "node-a""#,
                ),
                "unknown_node",
                "/architecture/views/2/instances/1/of",
            ),
            (
                edited(r#""in": "node-b" }"#, r#""in": "node-c" }"#),
                "unknown_boundary",
                "/architecture/views/2/instances/2/in",
            ),
            (
                edited(r#""id": "api-2", "of""#, r#""id": "api-1", "of""#),
                "duplicate_id",
                "/architecture/views/2/instances/2/id",
            ),
            (
                edited(
                    r#"{ "id": "query", "from": "api-1""#,
                    r#"{ "id": "route", "from": "api-1""#,
                ),
                "instance_mismatch",
                "/architecture/views/2/instanceConnections/2/id",
            ),
            (
                edited(
                    r#"{ "id": "query", "from": "api-dev""#,
                    r#"{ "id": "save", "from": "api-dev""#,
                ),
                "unknown_connection",
                "/architecture/views/1/instanceConnections/0/id",
            ),
            (
                edited(r#""to": "api-2" }"#, r#""to": "api-3" }"#),
                "unknown_node",
                "/architecture/views/2/instanceConnections/1/to",
            ),
            (
                edited(
                    r#""environment": "Local","#,
                    r#""environment": "Local", "components": ["api"],"#,
                ),
                "option_not_supported",
                "/architecture/views/1/components",
            ),
            (
                edited(r#""environment": "Local","#, ""),
                "option_not_supported",
                "/architecture/views/1/hosts",
            ),
            (
                edited(
                    r#""instances": [
          { "id": "api-dev", "of": "api", "in": "laptop" },
          { "id": "db-dev", "of": "db", "in": "laptop" }
        ],"#,
                    r#""instances": [],"#,
                ),
                "empty_view",
                "/architecture/views/1",
            ),
        ];
        for (json, code, path) in cases {
            assert_eq!(error(&json), (code, path.to_owned()));
        }
    }

    #[test]
    fn a_too_long_instance_label_points_to_the_instance() {
        // "Orders API (" and ")" leave 47 characters for the id.
        let id = format!("api-{}", "x".repeat(44));
        let json = edited(r#""id": "api-dev""#, &format!(r#""id": "{id}""#))
            .replace(r#""from": "api-dev""#, &format!(r#""from": "{id}""#));
        let error = ChartSpec::from_json(&json)
            .and_then(|spec| spec.view(None))
            .unwrap_err();
        assert_eq!(
            (error.code, error.path.as_str()),
            ("text_too_long", "/architecture/views/1/instances/0/id")
        );
        assert!(error.message.contains(&format!("\"Orders API ({id})\"")));
        assert!(
            error
                .message
                .contains("shorten the id or the component's label")
        );
        // One character less fits.
        let fits = json.replace(&id, &id[1..]);
        ChartSpec::from_json(&fits).unwrap().view(None).unwrap();
    }

    #[test]
    fn every_instance_connection_carries_the_category_of_its_model_connection() {
        let json = edited(
            r#""views": ["#,
            r#""perspectives": [{ "key": "trust", "title": "Trust", "categories": [
                { "label": "internal", "connections": ["query"] }] }],
               "perspective": "trust",
               "views": ["#,
        );
        let svg = render_json(
            &json,
            RenderFormat::Svg,
            &RenderOptions {
                view: Some("production".to_owned()),
                ..RenderOptions::default()
            },
        )
        .unwrap()
        .content;
        assert_eq!(svg.matches(" chartlet-flow-part-1\"").count(), 2);
        assert_eq!(svg.matches(">reads, writes · internal</text>").count(), 2);
        let production = view(&json, "production");
        let trust = &production.architecture.as_ref().unwrap().perspectives[0];
        assert_eq!(trust.categories[0].connections, ["query"]);
    }

    #[test]
    fn limits_hold_after_expansion() {
        let replicas: Vec<String> = (0..41)
            .map(|replica| format!(r#"{{ "id": "api-{replica}", "of": "api" }}"#))
            .collect();
        let json = edited(
            r#"{ "id": "api-dev", "of": "api", "in": "laptop" },"#,
            &format!("{},", replicas.join(",")),
        );
        assert_eq!(
            error(&json),
            (
                "too_many_nodes",
                "/architecture/views/1/instances".to_owned()
            )
        );
    }

    #[test]
    fn components_running_only_in_an_environment_are_in_a_view() {
        let json = edited(
            r#""components": ["client", "lb", "api", "db"]"#,
            r#""components": ["client"]"#,
        );
        let found = inspect_json(&json, &[]).unwrap();
        assert!(
            found
                .iter()
                .all(|inspection| inspection.code != "component_in_no_view"
                    && inspection.code != "view_adds_nothing"),
            "{found:?}"
        );
    }
}
