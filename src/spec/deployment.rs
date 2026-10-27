//! Deployment views: one environment of an architecture model, such as a developer's machine or
//! production, as the running instances of the model's components on hosts. The view resolves
//! to an architecture diagram of its own whose components are the instances, whose boundaries
//! are the hosts and whose connections are only those the view lists; limits and diagnostics are
//! those of that diagram, pointing into the view. A view that derives its connections adds,
//! for every model connection it does not list, one instance connection from each instance of
//! its `from` component to the nearest instances of its `to` component.

use super::{
    ArchitectureSpec, ChartSpec, ConnectionSpec, MainPath, ViewSpec,
    sequence::is_identifier,
    validate_text,
    view::{ResolvedView, shown_notes},
};
use crate::error::ChartError;

impl ChartSpec {
    /// The deployment view at `index` of a model whose views were checked.
    #[allow(clippy::too_many_lines)]
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
        let carried = model.instance_connections(view);
        architecture.connections = carried
            .iter()
            .enumerate()
            .map(|(at, (connection, _))| {
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
                    connection.to_interface =
                        connection.to_interface.or_else(|| of.to_interface.clone());
                    connection.from_side = connection.from_side.or(of.from_side);
                    connection.to_side = connection.to_side.or(of.to_side);
                    // A model connection carried by several instances keeps its id once.
                    if carried[..at]
                        .iter()
                        .any(|(earlier, _)| earlier.id == connection.id)
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
        // Only the notes on the whole diagram: the view shows instances and hosts instead.
        let notes = shown_notes(model, |_| false);
        architecture.notes = notes
            .iter()
            .map(|&note| model.notes[note].clone())
            .collect();
        architecture.views = Vec::new();
        architecture.delta = None;
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
                    carried
                        .iter()
                        .any(|(connection, _)| connection.id.as_ref() == Some(id))
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
            notes,
            dynamic: false,
            own_main_path: false,
            deployment: true,
            own_size,
            explicit_connections: view.instance_connections.len(),
            derived: carried.iter().filter_map(|(_, of)| *of).collect(),
        }
    }
}

/// A view without an environment, which has no hosts, instances or instance connections.
pub(super) fn reject_deployment_fields(view: &ViewSpec, path: &str) -> Result<(), ChartError> {
    match [
        (!view.hosts.is_empty(), "hosts"),
        (!view.instances.is_empty(), "instances"),
        (!view.instance_connections.is_empty(), "instanceConnections"),
        (view.derive_connections, "deriveConnections"),
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

/// The hosts around a host, by id, outermost first and ending with it; none for an instance
/// without one. A host that is not there or nests in a circle ends the chain, which the diagram
/// of the view reports.
fn host_chain<'a>(view: &'a ViewSpec, host: Option<&'a str>) -> Vec<&'a str> {
    let mut chain = Vec::new();
    let mut at = host;
    while let Some(id) = at {
        if chain.contains(&id) || chain.len() > view.hosts.len() {
            break;
        }
        chain.push(id);
        at = view
            .hosts
            .iter()
            .find(|host| host.id == id)
            .and_then(|host| host.parent.as_deref());
    }
    chain.reverse();
    chain
}

impl ArchitectureSpec {
    /// The connections of a deployment view: those it lists, each with `None`, then, if it
    /// derives connections, those derived from every model connection whose `id` it does not
    /// list, each with the place of that connection in the model.
    ///
    /// A model connection from A to B joins each instance of A, in the order of the instances,
    /// to the nearest instances of B: those whose hosts share the most hosts with its own,
    /// counted from the outermost. At a tie, it joins all of them; an instance is never joined
    /// to itself. The derived connection is the model connection with the instances as its
    /// ends.
    pub(super) fn instance_connections(
        &self,
        view: &ViewSpec,
    ) -> Vec<(ConnectionSpec, Option<usize>)> {
        let mut connections: Vec<(ConnectionSpec, Option<usize>)> = view
            .instance_connections
            .iter()
            .map(|connection| (connection.clone(), None))
            .collect();
        if !view.derive_connections {
            return connections;
        }
        let of = |component: &str| -> Vec<&super::InstanceSpec> {
            view.instances
                .iter()
                .filter(|instance| instance.of == component)
                .collect()
        };
        for (index, model) in self.connections.iter().enumerate() {
            let listed = model.id.is_some()
                && view
                    .instance_connections
                    .iter()
                    .any(|connection| connection.id == model.id);
            if listed {
                continue;
            }
            for from in of(&model.from) {
                let near = host_chain(view, from.host.as_deref());
                let shared = |to: &super::InstanceSpec| {
                    host_chain(view, to.host.as_deref())
                        .iter()
                        .zip(&near)
                        .take_while(|(a, b)| a == b)
                        .count()
                };
                let mut targets = of(&model.to);
                targets.retain(|to| to.id != from.id);
                let Some(nearest) = targets.iter().map(|to| shared(to)).max() else {
                    continue;
                };
                for to in targets.into_iter().filter(|to| shared(to) == nearest) {
                    let mut connection = model.clone();
                    connection.from.clone_from(&from.id);
                    connection.to.clone_from(&to.id);
                    connections.push((connection, Some(index)));
                }
            }
        }
        connections
    }

    /// What a deployment view shows of the model: the components its instances run and the
    /// connections its instance connections name or derive from, by their places in the
    /// model.
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
        let derived: Vec<usize> = self
            .instance_connections(view)
            .into_iter()
            .filter_map(|(_, of)| of)
            .collect();
        let connections = (0..self.connections.len())
            .filter(|&index| {
                derived.contains(&index)
                    || (self.connections[index].id.is_some()
                        && view
                            .instance_connections
                            .iter()
                            .any(|connection| connection.id == self.connections[index].id))
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
            (!view.include.is_empty(), "include"),
            (!view.exclude.is_empty(), "exclude"),
        ]
        .into_iter()
        .find_map(|(given, field)| given.then_some(field))
        {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                "a deployment view shows instances, hosts and instance connections; leave out components, connections, steps, mainPath, include and exclude",
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
    fn an_environment_keeps_only_the_notes_on_the_whole_diagram() {
        let json = edited(
            r#""views": ["#,
            r#""notes": [
                {"on": "api", "text": "Stateless"},
                {"text": "Figures from the last audit"}
            ],
            "views": ["#,
        );
        let production = view(&json, "production");
        let notes = &production.architecture.as_ref().unwrap().notes;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].text, "Figures from the last audit");
        let resolved = ChartSpec::from_json(&json)
            .unwrap()
            .resolve_view(Some("production"))
            .unwrap()
            .unwrap();
        assert_eq!(
            resolved.model_path("/architecture/notes/0/text"),
            "/architecture/notes/1/text"
        );
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
                    r#""environment": "Local","#,
                    r#""environment": "Local", "include": ["db"],"#,
                ),
                "option_not_supported",
                "/architecture/views/1/include",
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

    /// The ends of the connections of a resolved view, as `from>to`.
    fn ends(spec: &ChartSpec) -> String {
        spec.architecture
            .as_ref()
            .unwrap()
            .connections
            .iter()
            .map(|connection| format!("{}>{}", connection.from, connection.to))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The model with one more deployment view, `derived`, that derives its connections.
    fn with_derived(view: &str) -> String {
        edited(
            r#""views": ["#,
            &format!(
                r#""views": [{{"key": "derived", "title": "Derived", "environment": "Production",
                   "deriveConnections": true, {view}}},"#
            ),
        )
    }

    #[test]
    fn derived_connections_join_the_nearest_instances() {
        // The production view without its connections derives the ones it lists.
        let start = MODEL
            .find(
                r#""hosts": [
          { "id": "region""#,
            )
            .unwrap();
        let end = MODEL[start..].find(r#""instanceConnections""#).unwrap() + start;
        let json = with_derived(MODEL[start..end].trim_end().trim_end_matches(','));
        let derived = view(&json, "derived");
        assert_eq!(
            ends(&derived),
            "lb-prod>api-1 lb-prod>api-2 api-1>db-prod api-2>db-prod"
        );
        let production = view(&json, "production");
        assert_eq!(
            serde_json::to_string(&derived.architecture.as_ref().unwrap().connections).unwrap(),
            serde_json::to_string(&production.architecture.as_ref().unwrap().connections).unwrap()
        );

        // Replicas on two nodes join the database on their own node; the load balancer, on no
        // host, is as near to both.
        let json = with_derived(
            r#""hosts": [{ "id": "node-a", "label": "Node A" }, { "id": "node-b", "label": "Node B" }],
               "instances": [
                 { "id": "lb-1", "of": "lb" },
                 { "id": "api-a", "of": "api", "in": "node-a" },
                 { "id": "api-b", "of": "api", "in": "node-b" },
                 { "id": "db-a", "of": "db", "in": "node-a" },
                 { "id": "db-b", "of": "db", "in": "node-b" }
               ]"#,
        );
        assert_eq!(
            ends(&view(&json, "derived")),
            "lb-1>api-a lb-1>api-b api-a>db-a api-b>db-b"
        );

        // A model connection that the view lists is not derived; the others still are.
        let json = with_derived(
            r#""instances": [
                 { "id": "lb-1", "of": "lb" },
                 { "id": "api-1", "of": "api" },
                 { "id": "api-2", "of": "api" },
                 { "id": "db-1", "of": "db" }
               ],
               "instanceConnections": [{ "id": "route", "from": "lb-1", "to": "api-2" }]"#,
        );
        let derived = view(&json, "derived");
        assert_eq!(ends(&derived), "lb-1>api-2 api-1>db-1 api-2>db-1");
        let alternative = text_alternative(&derived).unwrap();
        assert!(alternative.description.contains("environment Production"));
        // A diagnostic about a derived connection points to the model connection.
        let spec = ChartSpec::from_json(&json).unwrap();
        spec.validate().unwrap();
        let resolved = spec.resolve_view(Some("derived")).unwrap().unwrap();
        assert_eq!(
            resolved.model_path("/architecture/connections/1/label"),
            "/architecture/connections/2/label"
        );
        assert_eq!(
            resolved.model_path("/architecture/connections/0/to"),
            "/architecture/views/0/instanceConnections/0/to"
        );
    }

    #[test]
    fn derived_connections_are_counted_and_only_in_deployment_views() {
        let replicas = |component: &str, count: usize| {
            (0..count)
                .map(|replica| {
                    format!(r#"{{ "id": "{component}-{replica}", "of": "{component}" }}"#)
                })
                .collect::<Vec<_>>()
                .join(",")
        };
        // Nine load balancers each join nine replicas: 81 connections.
        let json = with_derived(&format!(
            r#""instances": [{}, {}]"#,
            replicas("lb", 9),
            replicas("api", 9)
        ));
        assert_eq!(
            error(&json),
            (
                "too_many_edges",
                "/architecture/views/0/deriveConnections".to_owned()
            )
        );
        let json = edited(
            r#""components": ["client", "lb", "api", "db"]"#,
            r#""components": ["client", "lb", "api", "db"], "deriveConnections": true"#,
        );
        assert_eq!(
            error(&json),
            (
                "option_not_supported",
                "/architecture/views/0/deriveConnections".to_owned()
            )
        );
        // Without deriving, a deployment view renders the bytes it did.
        let options = RenderOptions {
            view: Some("production".to_owned()),
            id_prefix: Some("orders".to_owned()),
            ..RenderOptions::default()
        };
        let off = edited(
            r#""environment": "Production","#,
            r#""environment": "Production", "deriveConnections": false,"#,
        );
        assert_eq!(
            render_json(&off, RenderFormat::Svg, &options)
                .unwrap()
                .content,
            render_json(MODEL, RenderFormat::Svg, &options)
                .unwrap()
                .content
        );
    }

    #[test]
    fn instance_connections_take_the_sides_of_their_model_connection() {
        let json = edited(
            r#"{ "id": "query", "from": "api", "to": "db","#,
            r#"{ "id": "query", "from": "api", "to": "db", "fromSide": "right", "toSide": "start","#,
        )
        .replacen(
            r#""environment": "Production","#,
            r#""environment": "Production", "deriveConnections": true,"#,
            1,
        )
        .replacen(
            ",\n          { \"id\": \"query\", \"from\": \"api-1\", \"to\": \"db-prod\" },\n          { \"id\": \"query\", \"from\": \"api-2\", \"to\": \"db-prod\" }",
            "",
            1,
        );
        assert!(!json.contains(r#""from": "api-1", "to": "db-prod""#));
        let sides = |key: &str| {
            view(&json, key)
                .architecture
                .unwrap()
                .connections
                .iter()
                .filter(|connection| connection.to.starts_with("db"))
                .map(|connection| (connection.from_side, connection.to_side))
                .collect::<Vec<_>>()
        };
        // Listed in the local view, derived in production: the same sides.
        let expected = (
            Some(crate::spec::PortSide::Right),
            Some(crate::spec::PortSide::Start),
        );
        assert_eq!(sides("local"), [expected]);
        assert_eq!(sides("production"), [expected, expected]);
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
