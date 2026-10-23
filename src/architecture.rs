//! Architecture diagrams: components joined by connections, inside nested boundaries such as a
//! cloud region, a network or a system, laid out by the layered layout of flow charts. A
//! connection says what it does and, in brackets below, how. Notes stand beside the components
//! they explain, joined by a dotted line that is no connection. The description and the data
//! table name every component with its boundaries, where it connects to and its notes.

use std::fmt::Write as _;

use crate::{
    DataTable,
    error::ChartWarning,
    flow::{
        Diagram, Edge, End, Ends, Group, MARGIN, Node, Shape, heaviest_edge, layout_with_trunks,
        reading_order,
    },
    layout::format_value,
    metrics::TextMetrics,
    parts, roles,
    scene::Scene,
    spec::{
        ArchitectureSpec, ChartSpec, ComponentKind, ComponentSpec, ConnectionDirection,
        ConnectionSpec, DiagramRole, Locale, PerspectiveSpec,
    },
    structure::{Facts, Structure},
    symbols, text,
};

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let architecture = architecture(spec);
    let mut scene = layout_with_trunks(
        spec,
        &diagram(architecture, spec.locale),
        architecture.shared_trunks,
        warnings,
        metrics,
    );
    let used = roles::used(
        architecture
            .components
            .iter()
            .map(|component| component.role),
        architecture
            .connections
            .iter()
            .map(|connection| connection.role),
    );
    roles::push_legend(&mut scene, spec.locale, &used, MARGIN, metrics);
    if let Some((perspective, categories)) = categories(architecture) {
        // The categories used, in the perspective's order, and not stated last.
        let mut entries: Vec<(&'static str, &str)> = (0..perspective.categories.len())
            .filter(|index| categories.contains(&Some(*index)))
            .map(|index| {
                (
                    parts::CLASSES[index],
                    perspective.categories[index].label.as_str(),
                )
            })
            .collect();
        if categories.contains(&None) {
            entries.push((parts::UNSTATED, text::not_stated(spec.locale)));
        }
        parts::push_entries(
            &mut scene,
            Some(&perspective.title),
            &entries,
            MARGIN,
            metrics,
        );
    }
    scene
}

/// The perspective a diagram shows and the category of each component in it, by its place
/// among the perspective's categories; `None` where the component does not state one.
fn categories(architecture: &ArchitectureSpec) -> Option<(&PerspectiveSpec, Vec<Option<usize>>)> {
    let perspective = architecture.shown_perspective()?;
    let categories = architecture
        .components
        .iter()
        .map(|component| {
            perspective
                .categories
                .iter()
                .position(|category| category.components.contains(&component.id))
        })
        .collect();
    Some((perspective, categories))
}

/// The name of category `category` of `perspective`, or not stated.
fn category_name(locale: Locale, perspective: &PerspectiveSpec, category: Option<usize>) -> &str {
    category.map_or(text::not_stated(locale), |index| {
        perspective.categories[index].label.as_str()
    })
}

fn architecture(spec: &ChartSpec) -> &ArchitectureSpec {
    spec.architecture
        .as_ref()
        .expect("validated architecture diagrams carry an architecture block")
}

const fn shape(kind: ComponentKind) -> Shape {
    match kind {
        ComponentKind::Person => Shape::Person,
        ComponentKind::Frontend => Shape::Frontend,
        ComponentKind::Service => Shape::Process,
        ComponentKind::Database => Shape::Store,
        ComponentKind::Queue => Shape::Queue,
        ComponentKind::Storage => Shape::Bucket,
        ComponentKind::Cache => Shape::Cache,
        ComponentKind::Security => Shape::Shield,
        ComponentKind::External => Shape::External,
        ComponentKind::Node => Shape::Node,
        ComponentKind::Artifact => Shape::Artifact,
    }
}

fn diagram(architecture: &ArchitectureSpec, locale: Locale) -> Diagram {
    let mut diagram = model_diagram(architecture, locale);
    for (index, note) in architecture.notes.iter().enumerate() {
        let target = architecture.component(&note.on).expect("validated");
        diagram.push_note(target, &note.text, format!("/architecture/notes/{index}"));
    }
    diagram
}

/// The components in reading order, without the notes; the order does not depend on texts.
fn components_in_order(architecture: &ArchitectureSpec) -> Vec<usize> {
    reading_order(&diagram(architecture, Locale::En))
        .into_iter()
        .filter(|node| *node < architecture.components.len())
        .collect()
}

/// The diagram of the components, connections and boundaries, without notes; in a
/// perspective, every box names its category after its sublabel and carries a stripe in the
/// category's color.
fn model_diagram(architecture: &ArchitectureSpec, locale: Locale) -> Diagram {
    let boundary = |id: &Option<String>| {
        id.as_deref()
            .map(|id| architecture.boundary(id).expect("validated"))
    };
    let categories = categories(architecture);
    let category = |index: usize| {
        categories.as_ref().map(|(perspective, categories)| {
            let category = categories[index];
            (
                category.map_or(parts::UNSTATED, |at| parts::CLASSES[at]),
                category_name(locale, perspective, category).to_owned(),
            )
        })
    };
    Diagram {
        nodes: architecture
            .components
            .iter()
            .enumerate()
            .map(|(index, component)| Node {
                label: component.label.clone(),
                sublabel: match (&component.sublabel, category(index)) {
                    (Some(sublabel), Some((_, name))) => Some(format!("{sublabel} · {name}")),
                    (sublabel, named) => named.map(|(_, name)| name).or_else(|| sublabel.clone()),
                },
                category: category(index),
                shape: shape(component.kind),
                lane: None,
                group: boundary(&component.boundary),
                id: Some(component.id.clone()),
                path: format!("/architecture/components/{index}"),
                compartments: Vec::new(),
                value: None,
                parts: Vec::new(),
                symbol: component
                    .symbol
                    .as_deref()
                    .map(|name| symbols::find(name).expect("validated")),
                role: component.role,
            })
            .collect(),
        edges: architecture
            .connections
            .iter()
            .enumerate()
            .map(|(index, connection)| Edge {
                from: architecture.component(&connection.from).expect("validated"),
                to: architecture.component(&connection.to).expect("validated"),
                label: step_label(connection.step, connection.label.as_deref()),
                technology: connection.technology.clone(),
                dash: connection.dash,
                main: architecture.on_main_path(index),
                label_path: format!("/architecture/connections/{index}/label"),
                ends: ends(connection.direction),
                sign: connection.direction.sign(),
                weight: connection.weight,
                role: connection.role,
            })
            .collect(),
        lanes: Vec::new(),
        groups: architecture
            .boundaries
            .iter()
            .enumerate()
            .map(|(index, group)| Group {
                label: group.label.clone(),
                path: format!("/architecture/boundaries/{index}"),
                parent: boundary(&group.parent),
            })
            .collect(),
        ranks: architecture
            .same_rank
            .iter()
            .map(|set| {
                set.iter()
                    .map(|id| architecture.component(id).expect("validated"))
                    .collect()
            })
            .collect(),
        orientation: architecture.orientation,
    }
}

/// The arrowheads of a connection: at its end, at both ends, or none.
fn ends(direction: ConnectionDirection) -> Ends {
    let (tail, head) = match direction {
        ConnectionDirection::Forward => return Ends::arrow(),
        ConnectionDirection::Both => (End::Arrow, End::Arrow),
        ConnectionDirection::None => (End::None, End::None),
    };
    Ends {
        tail,
        head,
        tail_text: None,
        head_text: None,
    }
}

/// The label of a connection with its step number in front: `2. reads`.
fn step_label(step: Option<u8>, label: Option<&str>) -> Option<String> {
    match (step, label) {
        (Some(step), Some(label)) => Some(format!("{step}. {label}")),
        (Some(step), None) => Some(format!("{step}.")),
        (None, label) => label.map(str::to_owned),
    }
}

/// What a component is: its kind, and after it the technology category of its symbol, as
/// `database, search index`; a category that names the kind stands alone, as `object storage`.
/// The sublabel is not repeated.
fn kind(locale: Locale, component: &ComponentSpec) -> String {
    let kind = text::component_kind(locale, component.kind);
    match component
        .symbol
        .as_deref()
        .and_then(symbols::find)
        .map(|symbol| symbol.category(locale))
    {
        Some(category) if category.to_lowercase().contains(&kind.to_lowercase()) => {
            category.to_owned()
        }
        Some(category) => format!("{kind}, {category}"),
        None => kind.to_owned(),
    }
}

/// The boundaries around a component, outermost first, by label: `Region › Network`.
fn boundaries_of(architecture: &ArchitectureSpec, component: usize) -> String {
    architecture.components[component]
        .boundary
        .as_deref()
        .and_then(|id| architecture.boundary(id))
        .and_then(|index| architecture.chain(index))
        .unwrap_or_default()
        .iter()
        .map(|index| architecture.boundaries[*index].label.as_str())
        .collect::<Vec<_>>()
        .join(" › ")
}

/// Where a component connects to, as `label (step, what, how, weight, role)`, in the order of
/// the connections; the role only `with_roles`.
fn connects_to(spec: &ChartSpec, component: usize, with_roles: bool) -> Vec<String> {
    let (locale, architecture) = (spec.locale, architecture(spec));
    let id = &architecture.components[component].id;
    architecture
        .connections
        .iter()
        .filter(|connection| &connection.from == id)
        .map(|connection| {
            let target = &architecture.components
                [architecture.component(&connection.to).expect("validated")]
            .label;
            let note = text::connection_note(
                locale,
                connection.label.as_deref(),
                connection.technology.as_deref(),
            );
            let notes: Vec<String> = text::connection_direction(locale, connection.direction)
                .map(str::to_owned)
                .into_iter()
                .chain(connection.step.map(|step| text::step_number(locale, step)))
                .chain(note)
                .chain(connection.weight.map(|weight| {
                    text::edge_weight(locale, &format_value(weight, spec.number_style()))
                }))
                .chain(
                    connection
                        .role
                        .filter(|_| with_roles)
                        .map(|role| text::role(locale, role).to_owned()),
                )
                .collect();
            if notes.is_empty() {
                target.clone()
            } else {
                format!("{target} ({})", notes.join(", "))
            }
        })
        .collect()
}

/// A connection by the labels of its ends: `API → Posts`, `API ↔ Images`.
fn written(architecture: &ArchitectureSpec, connection: &ConnectionSpec) -> String {
    let label = |id: &str| {
        architecture.components[architecture.component(id).expect("validated")]
            .label
            .as_str()
    };
    format!(
        "{} {} {}",
        label(&connection.from),
        connection.direction.sign(),
        label(&connection.to)
    )
}

/// The numbered connections in the order of their steps, as `1 Reader → Site (reads)`.
fn steps_in_order(architecture: &ArchitectureSpec) -> Vec<String> {
    let mut numbered: Vec<_> = architecture
        .connections
        .iter()
        .filter_map(|connection| connection.step.map(|step| (step, connection)))
        .collect();
    numbered.sort_by_key(|(step, _)| *step);
    numbered
        .into_iter()
        .map(|(step, connection)| {
            let ends = written(architecture, connection);
            match &connection.label {
                Some(note) => format!("{step} {ends} ({note})"),
                None => format!("{step} {ends}"),
            }
        })
        .collect()
}

/// For a view that leaves out part of its model, a sentence that says how much it shows, with a
/// space in front; empty otherwise.
fn view_scope(architecture: &ArchitectureSpec, locale: Locale) -> String {
    if let Some((environment, components)) = &architecture.environment {
        return format!(
            " {}",
            text::deployment(
                locale,
                environment,
                architecture.components.len(),
                *components
            )
        );
    }
    match architecture.scope {
        Some(scope)
            if scope.components.0 < scope.components.1
                || scope.connections.0 < scope.connections.1 =>
        {
            format!(
                " {}",
                text::view_scope(locale, scope.components, scope.connections)
            )
        }
        _ => String::new(),
    }
}

/// The diagram in sentences: its size, the boundaries and what each holds, the main path, and
/// every component in reading order with where it connects to.
#[allow(clippy::too_many_lines)]
pub(crate) fn description(spec: &ChartSpec) -> String {
    let architecture = architecture(spec);
    let locale = spec.locale;
    let mut description = text::architecture_opening(
        locale,
        architecture.components.len(),
        architecture.connections.len(),
        architecture.boundaries.len(),
    );
    description.push_str(&view_scope(architecture, locale));
    for boundary in &architecture.boundaries {
        let parent = boundary
            .parent
            .as_deref()
            .and_then(|id| architecture.boundary(id))
            .map(|parent| architecture.boundaries[parent].label.as_str());
        let held: Vec<&str> = architecture
            .components
            .iter()
            .filter(|component| component.boundary.as_deref() == Some(boundary.id.as_str()))
            .map(|component| component.label.as_str())
            .chain(
                architecture
                    .boundaries
                    .iter()
                    .filter(|inner| inner.parent.as_deref() == Some(boundary.id.as_str()))
                    .map(|inner| inner.label.as_str()),
            )
            .collect();
        description.push(' ');
        description.push_str(&text::boundary(&boundary.label, parent, &held));
    }
    let main_path = architecture.main_path_steps();
    if !main_path.is_empty() {
        let path: Vec<&str> = main_path
            .iter()
            .map(|id| {
                architecture.components[architecture.component(id).expect("validated")]
                    .label
                    .as_str()
            })
            .collect();
        description.push(' ');
        description.push_str(&text::main_path(locale, &path));
    }
    let ends: Vec<(usize, usize)> = architecture
        .connections
        .iter()
        .map(|connection| {
            let index = |id: &str| architecture.component(id).expect("validated");
            (index(&connection.from), index(&connection.to))
        })
        .collect();
    let structure = Structure::of(
        architecture.components.len(),
        &ends,
        &[],
        // Where a connection runs both ways or none, there are no sources, sinks or loops to
        // tell.
        Facts {
            directed: architecture.is_directed(),
        },
    )
    .sentences(locale, |component| {
        architecture.components[component].label.clone()
    });
    if !structure.is_empty() {
        description.push(' ');
        description.push_str(&structure);
    }
    let weights: Vec<Option<f64>> = architecture
        .connections
        .iter()
        .map(|connection| connection.weight)
        .collect();
    if let Some(heaviest) = heaviest_edge(spec, &weights, |edge| {
        written(architecture, &architecture.connections[edge])
    }) {
        description.push(' ');
        description.push_str(&heaviest);
    }
    let steps = steps_in_order(architecture);
    if !steps.is_empty() {
        description.push(' ');
        description.push_str(&text::steps(locale, &steps));
    }
    for component in components_in_order(architecture) {
        let spec_component = &architecture.components[component];
        write!(
            description,
            " {}",
            text::component(
                locale,
                &spec_component.label,
                &kind(locale, spec_component),
                &connects_to(spec, component, false),
            )
        )
        .expect("writing to String cannot fail");
    }
    if let Some(sentence) = roles::sentence(locale, &with_roles(architecture)) {
        description.push(' ');
        description.push_str(&sentence);
    }
    if let Some((perspective, categories)) = categories(architecture) {
        // Each category in the perspective's order with what has it, not stated last.
        let order = components_in_order(architecture);
        let entries: Vec<String> = (0..perspective.categories.len())
            .map(Some)
            .chain([None])
            .filter_map(|category| {
                let members: Vec<&str> = order
                    .iter()
                    .filter(|&&component| categories[component] == category)
                    .map(|&component| architecture.components[component].label.as_str())
                    .collect();
                (!members.is_empty()).then(|| {
                    format!(
                        "{}: {}",
                        category_name(locale, perspective, category),
                        members.join(", ")
                    )
                })
            })
            .collect();
        description.push(' ');
        description.push_str(&text::perspective(locale, &perspective.title, &entries));
        if let Some(meaning) = &perspective.description {
            description.push(' ');
            description.push_str(meaning);
        }
    }
    description.push_str(&notes_sentences(spec));
    description
}

/// Every component with a role, in reading order, then every connection with one, as
/// `From → To`.
fn with_roles(architecture: &ArchitectureSpec) -> Vec<(DiagramRole, String)> {
    let label = |id: &str| {
        architecture.components[architecture.component(id).expect("validated")]
            .label
            .as_str()
    };
    components_in_order(architecture)
        .into_iter()
        .filter_map(|component| {
            let component = &architecture.components[component];
            component.role.map(|role| (role, component.label.clone()))
        })
        .chain(architecture.connections.iter().filter_map(|connection| {
            connection.role.map(|role| {
                (
                    role,
                    format!("{} → {}", label(&connection.from), label(&connection.to)),
                )
            })
        }))
        .collect()
}

/// Every note with the component it explains, one sentence each after a space; nothing without
/// notes.
fn notes_sentences(spec: &ChartSpec) -> String {
    let architecture = architecture(spec);
    if architecture.notes.is_empty() {
        return String::new();
    }
    let notes: Vec<(&str, &str)> = architecture
        .notes
        .iter()
        .map(|note| {
            let target = architecture.component(&note.on).expect("validated");
            (
                architecture.components[target].label.as_str(),
                note.text.as_str(),
            )
        })
        .collect();
    format!(" {}", text::notes(spec.locale, &notes))
}

/// One row per component in reading order: its label, kind, boundaries, role and where it
/// connects to.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let architecture = architecture(spec);
    let words = spec.locale.words();
    let bounded = !architecture.boundaries.is_empty();
    let with_notes = !architecture.notes.is_empty();
    let with_roles = architecture
        .components
        .iter()
        .any(|component| component.role.is_some());
    let mut columns = vec![words.component.to_owned(), words.message_kind.to_owned()];
    let deployed = architecture.environment.is_some();
    if deployed {
        words.instance.clone_into(&mut columns[0]);
    }
    if bounded {
        columns.push(words.boundary.to_owned());
    }
    if with_roles {
        columns.push(words.role.to_owned());
    }
    let categories = categories(architecture);
    if let Some((perspective, _)) = &categories {
        columns.push(perspective.title.clone());
    }
    columns.push(words.connects_to.to_owned());
    if with_notes {
        columns.push(words.notes.to_owned());
    }
    let rows = components_in_order(architecture)
        .into_iter()
        .map(|component| {
            let mut row = vec![
                architecture.components[component].label.clone(),
                kind(spec.locale, &architecture.components[component]),
            ];
            if bounded {
                row.push(boundaries_of(architecture, component));
            }
            if with_roles {
                row.push(
                    architecture.components[component]
                        .role
                        .map_or_else(String::new, |role| text::role(spec.locale, role).to_owned()),
                );
            }
            if let Some((perspective, categories)) = &categories {
                row.push(category_name(spec.locale, perspective, categories[component]).to_owned());
            }
            row.push(connects_to(spec, component, true).join(", "));
            if with_notes {
                let id = &architecture.components[component].id;
                let notes: Vec<&str> = architecture
                    .notes
                    .iter()
                    .filter(|note| &note.on == id)
                    .map(|note| note.text.as_str())
                    .collect();
                row.push(notes.join("; "));
            }
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use crate::{ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative};

    const SPEC: &str = r#"{
        "schemaVersion": 1,
        "type": "architecture",
        "title": "Blog",
        "width": 1000,
        "height": 800,
        "architecture": {
            "boundaries": [
                {"id": "cloud", "label": "Cloud"},
                {"id": "private", "label": "Private network", "in": "cloud"}
            ],
            "components": [
                {"id": "reader", "label": "Reader", "kind": "person"},
                {"id": "site", "label": "Site", "kind": "frontend"},
                {"id": "api", "label": "API", "in": "private"},
                {"id": "db", "label": "Posts", "kind": "database", "in": "private"},
                {"id": "files", "label": "Images", "kind": "storage", "in": "cloud"},
                {"id": "mail", "label": "Mail service", "kind": "external"}
            ],
            "connections": [
                {"from": "reader", "to": "site", "label": "reads"},
                {"from": "site", "to": "api", "label": "loads", "technology": "HTTPS"},
                {"from": "api", "to": "db", "technology": "SQL"},
                {"from": "api", "to": "files", "label": "stores"},
                {"from": "api", "to": "mail", "label": "notifies"}
            ],
            "mainPath": ["reader", "site", "api", "db"]
        }
    }"#;

    fn svg(json: &str) -> crate::RenderOutput {
        render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect("the test specification renders")
    }

    fn error(json: &str) -> (&'static str, String) {
        let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("the test specification is invalid");
        (error.code, error.path)
    }

    #[test]
    fn components_sit_in_their_nested_boundaries_and_nothing_else_does() {
        for orientation in ["portrait", "landscape"] {
            let json = SPEC.replace(
                "\"mainPath\"",
                &format!("\"orientation\": \"{orientation}\", \"mainPath\""),
            );
            let output = svg(&json);
            assert!(
                output
                    .warnings
                    .iter()
                    .all(|warning| warning.code != "group_overlap"),
                "{orientation}: {:?}",
                output.warnings
            );
            assert_eq!(
                output
                    .content
                    .matches("class=\"chartlet-flow-group\"")
                    .count(),
                2
            );
        }
        let output = svg(SPEC);
        assert!(
            output
                .content
                .contains("<title>Site → API: loads [HTTPS]</title>")
        );
        assert!(output.content.contains("<title>API → Posts: [SQL]</title>"));
        assert!(output.content.contains(">[HTTPS]</text>"));
        assert!(output.content.contains("class=\"chartlet-arch-head\""));
        let secured = SPEC.replace(
            r#"{"id": "mail", "label": "Mail service", "kind": "external"}"#,
            r#"{"id": "mail", "label": "Mail service", "kind": "external"},
                {"id": "vault", "label": "Vault", "kind": "security", "in": "private"}"#,
        ).replace(
            r#"{"from": "api", "to": "mail", "label": "notifies"}"#,
            r#"{"from": "api", "to": "mail", "label": "notifies"}, {"from": "api", "to": "vault", "label": "reads secrets"}"#,
        );
        let output = svg(&secured);
        assert!(
            output
                .content
                .contains("chartlet-role-red\"><title>Vault</title>")
        );
    }

    #[test]
    fn the_text_alternative_names_boundaries_and_connections() {
        let alternative =
            text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
        assert!(alternative.description.starts_with(
            "Architecture diagram with 6 components and 5 connections in 2 boundaries. Cloud: Images, Private network. Private network (in Cloud): API, Posts. Main path: Reader → Site → API → Posts."
        ));
        assert!(
            alternative
                .description
                .contains("Site (frontend): connects to API (loads, via HTTPS).")
        );
        assert!(
            alternative
                .description
                .contains("Posts (database): no outgoing connection.")
        );
        assert_eq!(
            alternative.table.columns,
            ["Component", "Kind", "Boundary", "Connects to"]
        );
        let api = alternative
            .table
            .rows
            .iter()
            .find(|row| row[0] == "API")
            .expect("a row");
        assert_eq!(api[2], "Cloud › Private network");
        assert_eq!(
            api[3],
            "Posts (via SQL), Images (stores), Mail service (notifies)"
        );
    }

    #[test]
    fn german_names_kinds_and_columns() {
        let json = SPEC.replace(
            "\"schemaVersion\": 1,",
            "\"schemaVersion\": 1, \"locale\": \"de\",",
        );
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(
            alternative.description.starts_with(
                "Architekturdiagramm mit 6 Komponenten und 5 Verbindungen in 2 Grenzen."
            )
        );
        assert!(
            alternative
                .description
                .contains("Site (Oberfläche): verbunden mit API (loads, über HTTPS).")
        );
        assert_eq!(
            alternative.table.columns,
            ["Komponente", "Art", "Grenze", "Verbunden mit"]
        );
    }

    #[test]
    fn a_component_that_hangs_off_a_boundary_stands_after_it_on_the_main_axis() {
        // The mail service only receives from the API inside the boundaries; it moves to the
        // layer after them instead of beside the frames.
        let json = SPEC.replace(
            "\"mainPath\"",
            "\"orientation\": \"portrait\", \"mainPath\"",
        );
        let output = svg(&json);
        let number = |shape: &str, name: &str| -> f64 {
            let start = shape.find(&format!(" {name}=\"")).expect("an attribute") + name.len() + 3;
            shape[start..]
                .split('"')
                .next()
                .expect("a value")
                .parse()
                .expect("a number")
        };
        let at = output
            .content
            .find("<title>Mail service</title>")
            .expect("drawn");
        let mail = &output.content[output.content[..at].rfind('<').expect("a shape")..at];
        // The first frame drawn is the outermost one.
        let frame_at = output
            .content
            .find("class=\"chartlet-flow-group\"")
            .expect("a frame");
        let frame =
            &output.content[output.content[..frame_at].rfind('<').expect("a rect")..frame_at];
        assert!(number(mail, "y") > number(frame, "y") + number(frame, "height"));
    }

    #[test]
    fn invalid_architectures_name_the_field() {
        for (from, to, code, path) in [
            (
                r#""to": "api", "label": "loads""#,
                r#""to": "gateway", "label": "loads""#,
                "unknown_node",
                "/architecture/connections/1/to",
            ),
            (
                r#""in": "private"}"#,
                r#""in": "public"}"#,
                "unknown_boundary",
                "/architecture/components/2/in",
            ),
            (
                r#"{"id": "api""#,
                r#"{"id": "cloud""#,
                "duplicate_id",
                "/architecture/components/2/id",
            ),
            (
                r#"{"id": "cloud", "label": "Cloud"}"#,
                r#"{"id": "cloud", "label": "Cloud", "in": "private"}"#,
                "boundary_cycle",
                "/architecture/boundaries/0/in",
            ),
            (
                r#"{"id": "cloud", "label": "Cloud"},"#,
                r#"{"id": "cloud", "label": "Cloud"}, {"id": "edge", "label": "Edge"},"#,
                "empty_boundary",
                "/architecture/boundaries/1",
            ),
            (
                r#"["reader", "site", "api", "db"]"#,
                r#"["reader", "api"]"#,
                "main_path_gap",
                "/architecture/mainPath/1",
            ),
            (
                r#"["reader", "site", "api", "db"]"#,
                r#"{"from": "db", "to": "reader"}"#,
                "main_path_unreachable",
                "/architecture/mainPath",
            ),
            (
                r#"["reader", "site", "api", "db"]"#,
                r#"{"from": "nobody", "to": "db"}"#,
                "unknown_node",
                "/architecture/mainPath/from",
            ),
            (
                r#""type": "architecture""#,
                r#""type": "flow""#,
                "option_not_supported",
                "/architecture",
            ),
        ] {
            assert!(SPEC.contains(from), "{from}");
            assert_eq!(
                error(&SPEC.replace(from, to)),
                (code, path.to_owned()),
                "{to}"
            );
        }
        let missing = r#"{"schemaVersion": 1, "type": "architecture", "title": "Empty"}"#;
        assert_eq!(
            error(missing),
            ("missing_architecture", "/architecture".to_owned())
        );
    }

    #[test]
    fn numbered_steps_are_written_before_labels_and_listed_in_order() {
        let json = SPEC
            .replace(
                r#"{"from": "site", "to": "api", "label": "loads", "technology": "HTTPS"}"#,
                r#"{"from": "site", "to": "api", "label": "loads", "technology": "HTTPS", "step": 2}"#,
            )
            .replace(
                r#"{"from": "reader", "to": "site", "label": "reads"}"#,
                r#"{"from": "reader", "to": "site", "label": "reads", "step": 1}"#,
            );
        let output = svg(&json);
        assert!(output.content.contains("1. reads") && output.content.contains("2. loads"));
        let text = text_alternative(&ChartSpec::from_json(&json).expect("parses"))
            .expect("valid")
            .description;
        assert!(text.contains("Steps: 1 Reader → Site (reads); 2 Site → API (loads)."));
        assert!(text.contains("API (step 2, loads, via HTTPS)"));
    }

    #[test]
    fn a_connection_runs_both_ways_or_has_no_direction_in_picture_and_text() {
        let both = SPEC.replace(
            r#"{"from": "api", "to": "files", "label": "stores"}"#,
            r#"{"from": "api", "to": "files", "label": "stores", "direction": "both", "step": 1}"#,
        );
        let none = both.replace(
            r#"{"from": "api", "to": "mail", "label": "notifies"}"#,
            r#"{"from": "api", "to": "mail", "label": "notifies", "direction": "none"}"#,
        );
        let heads = |json: &str| svg(json).content.matches("chartlet-flow-head").count();
        assert_eq!(heads(&both), heads(SPEC) + 1);
        assert_eq!(heads(&none), heads(SPEC));
        let output = svg(&none);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert!(
            output
                .content
                .contains("<title>API ↔ Images: 1. stores</title>")
        );
        assert!(
            output
                .content
                .contains("<title>API — Mail service: notifies</title>")
        );
        let alternative =
            text_alternative(&ChartSpec::from_json(&none).expect("parses")).expect("valid");
        // One connection each, listed once, with the way it runs.
        assert!(
            alternative
                .description
                .starts_with("Architecture diagram with 6 components and 5 connections")
        );
        assert!(
            alternative
                .description
                .contains("Steps: 1 API ↔ Images (stores).")
        );
        let api = alternative
            .table
            .rows
            .iter()
            .find(|row| row[0] == "API")
            .expect("a row");
        assert_eq!(
            api[3],
            "Posts (via SQL), Images (both ways, step 1, stores), Mail service (without direction, notifies)"
        );
        // Without one way for every connection, nothing is a source or a sink.
        let directed = text_alternative(&ChartSpec::from_json(SPEC).expect("parses"))
            .expect("valid")
            .description;
        assert!(directed.contains("Nothing leads into Reader."));
        assert!(!alternative.description.contains("Nothing leads"));
        let forward = SPEC.replace(
            r#""label": "stores"}"#,
            r#""label": "stores", "direction": "forward"}"#,
        );
        assert_eq!(svg(&forward).content, svg(SPEC).content);
    }

    const FAN_OUT: &str = include_str!("../examples/service-fan-out.json");

    fn branches(content: &str) -> usize {
        content
            .matches("r=\"3\" class=\"chartlet-flow-head\"")
            .count()
    }

    #[test]
    fn shared_trunks_branch_with_dots_and_keep_every_connection() {
        let separate = FAN_OUT.replace("\"sharedTrunks\": true", "\"sharedTrunks\": false");
        let unset = FAN_OUT.replace(",\n    \"sharedTrunks\": true", "");
        assert_ne!(unset, FAN_OUT);
        assert_eq!(svg(&separate).content, svg(&unset).content);
        assert_eq!(branches(&svg(&unset).content), 0);
        for orientation in ["portrait", "landscape"] {
            let json = FAN_OUT.replace(
                "\"sharedTrunks\"",
                &format!("\"orientation\": \"{orientation}\", \"sharedTrunks\""),
            );
            let output = svg(&json);
            assert!(output.warnings.is_empty(), "{:?}", output.warnings);
            // Out of the gateway: two inner branches on one side; into the log store: two inner
            // branches and the trunk, where the edges come from both sides.
            assert_eq!(branches(&output.content), 5, "{orientation}");
            assert_eq!(svg(&json).content, output.content);
            // Every connection is still drawn on its own, with its tooltip.
            for title in [
                "<title>Gateway → Catalog</title>",
                "<title>Gateway → Cart</title>",
                "<title>Cart → Log store</title>",
                "<title>Orders → Log store</title>",
                "<title>Gateway → Sign-in: checks token</title>",
            ] {
                assert_eq!(output.content.matches(title).count(), 1, "{title}");
            }
        }
        let shared =
            text_alternative(&ChartSpec::from_json(FAN_OUT).expect("parses")).expect("valid");
        let own = text_alternative(&ChartSpec::from_json(&unset).expect("parses")).expect("valid");
        assert_eq!(shared.description, own.description);
        assert_eq!(shared.table.rows, own.table.rows);
    }

    #[test]
    fn only_connections_that_read_the_same_share_a_trunk() {
        let separate = FAN_OUT.replace("\"sharedTrunks\": true", "\"sharedTrunks\": false");
        // Labels, a second pattern, a weight, a direction or a second connection between the
        // same two components each keep a line of their own; what is left shares no trunk.
        for (from, to) in [
            (
                r#""to": "search" }"#,
                r#""to": "search", "label": "finds" }"#,
            ),
            (
                r#""to": "search" }"#,
                r#""to": "search", "dash": "dotted" }"#,
            ),
            (r#""to": "search" }"#, r#""to": "search", "weight": 3 }"#),
            (
                r#""to": "search" }"#,
                r#""to": "search", "direction": "both" }"#,
            ),
            (
                r#""to": "search" }"#,
                r#""to": "search", "role": "critical" }"#,
            ),
            (
                r#"{ "from": "gateway", "to": "cart" },"#,
                r#"{ "from": "gateway", "to": "catalog" },"#,
            ),
        ] {
            assert!(FAN_OUT.contains(from), "{from}");
            // The log store's trunk keeps its three branches; the gateway's has at most one left.
            let all = |json: &str| branches(&svg(&json.replace(from, to)).content);
            assert!((3..5).contains(&all(FAN_OUT)), "{to}");
            assert_eq!(all(&separate), 0, "{to}");
        }
    }

    #[test]
    fn connections_of_one_role_share_a_trunk() {
        let planned = FAN_OUT
            .replace(
                r#""to": "catalog" }"#,
                r#""to": "catalog", "role": "planned" }"#,
            )
            .replace(
                r#""to": "search" }"#,
                r#""to": "search", "role": "planned" }"#,
            );
        let output = svg(&planned);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        // Catalog and Search share a planned trunk, Cart goes alone; the log store keeps its own.
        assert_eq!(branches(&output.content), 1 + 3);
    }

    #[test]
    fn shared_trunks_keep_to_shared_ranks() {
        for set in [r#"[["auth", "catalog"]]"#, r#"[["auth", "logs"]]"#] {
            let json = FAN_OUT.replace(
                "\"sharedTrunks\"",
                &format!("\"sameRank\": {set}, \"sharedTrunks\""),
            );
            let output = svg(&json);
            for title in [
                "<title>Gateway → Catalog</title>",
                "<title>Gateway → Search</title>",
                "<title>Gateway → Cart</title>",
                "<title>Gateway → Orders</title>",
                "<title>Search → Log store</title>",
                "<title>Gateway → Sign-in: checks token</title>",
            ] {
                assert_eq!(output.content.matches(title).count(), 1, "{set} {title}");
            }
            assert_eq!(svg(&json).content, output.content);
        }
    }

    #[test]
    fn a_view_keeps_the_direction_of_its_connections() {
        let json = FAN_OUT
            .replace(r#""to": "search" }"#, r#""to": "search", "direction": "both" }"#)
            .replace(
                "\"sharedTrunks\": true",
                r#""sharedTrunks": true, "views": [{"key": "search", "title": "Search", "components": ["gateway", "search", "logs"]}]"#,
            );
        let view = ChartSpec::from_json(&json)
            .expect("parses")
            .view(Some("search"))
            .expect("a view");
        let architecture = view.architecture.as_ref().expect("an architecture");
        assert_eq!(
            architecture.connections[0].direction,
            crate::ConnectionDirection::Both
        );
        let options = RenderOptions {
            view: Some("search".to_owned()),
            ..RenderOptions::default()
        };
        let output = render_json(&json, RenderFormat::Svg, &options).expect("renders");
        assert!(output.content.contains("<title>Gateway ↔ Search</title>"));
        assert!(output.content.contains("(both ways)"));
    }

    #[test]
    fn note_lines_never_join_a_shared_trunk() {
        let notes = r#""notes": [
                {"on": "gateway", "text": "Routes by path"},
                {"on": "gateway", "text": "Retries twice"},
                {"on": "logs", "text": "Kept for a day"}
            ],
            "sharedTrunks""#;
        let json = FAN_OUT.replace("\"sharedTrunks\"", notes);
        let content = svg(&json).content;
        assert!(branches(&content) > 0);
        // No arrowhead on a note line: as many as without notes.
        let arrowheads = |content: &str| {
            content.matches("class=\"chartlet-flow-head").count() - branches(content)
        };
        assert_eq!(arrowheads(&content), arrowheads(&svg(FAN_OUT).content));
        // Every note line leaves on its own, apart from the gateway's trunk, without a tooltip.
        let start = |line: &str| line.split(' ').next().expect("a point").to_owned();
        let notes: Vec<String> = content
            .split("<polyline points=\"")
            .filter(|line| line.contains("chartlet-flow-dotted\"/>"))
            .map(start)
            .collect();
        assert_eq!(notes.len(), 3);
        let trunk = content
            .split("<polyline points=\"")
            .find(|line| line.contains("<title>Gateway → Catalog</title>"))
            .map(start)
            .expect("drawn");
        assert!(notes.iter().all(|note| *note != trunk));
        assert_ne!(notes[0], notes[1]);
    }

    #[test]
    fn shared_trunks_work_on_a_phone() {
        let json = FAN_OUT.replace(
            "\"source\"",
            "\"mobile\": { \"width\": 360, \"height\": 900 }, \"source\"",
        );
        let options = RenderOptions {
            variant: crate::Variant::Mobile,
            ..RenderOptions::default()
        };
        let output = render_json(&json, RenderFormat::Svg, &options).expect("renders");
        assert_eq!(branches(&output.content), 5);
        assert_eq!(
            output.content,
            render_json(&json, RenderFormat::Svg, &options)
                .expect("renders")
                .content
        );
    }

    #[test]
    fn a_step_is_a_unique_number_from_one_to_ninety_nine() {
        let step = |first: u32, second: u32| {
            SPEC.replace(
                r#"{"from": "reader", "to": "site", "label": "reads"}"#,
                &format!(r#"{{"from": "reader", "to": "site", "step": {first}}}"#),
            )
            .replace(
                r#"{"from": "api", "to": "files", "label": "stores"}"#,
                &format!(r#"{{"from": "api", "to": "files", "step": {second}}}"#),
            )
        };
        assert_eq!(
            error(&step(1, 1)),
            (
                "duplicate_step",
                "/architecture/connections/3/step".to_owned()
            )
        );
        assert_eq!(
            error(&step(0, 2)),
            (
                "invalid_step",
                "/architecture/connections/0/step".to_owned()
            )
        );
    }

    #[test]
    fn a_node_and_an_artifact_belong_to_deployments() {
        let json = SPEC
            .replace(r#""kind": "storage""#, r#""kind": "node""#)
            .replace(r#""kind": "external""#, r#""kind": "artifact""#);
        let output = svg(&json);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        let text = text_alternative(&ChartSpec::from_json(&json).expect("parses"))
            .expect("valid")
            .description;
        assert!(
            text.contains("Images is a node") || text.contains("Images (node)"),
            "{text}"
        );
    }

    /// The blog with the API critical, the mail service planned, and the connections to both.
    fn with_roles() -> String {
        SPEC.replace(
            r#"{"id": "api", "label": "API", "in": "private"}"#,
            r#"{"id": "api", "label": "API", "in": "private", "role": "critical"}"#,
        )
        .replace(
            r#""kind": "external"}"#,
            r#""kind": "external", "role": "planned"}"#,
        )
        .replace(
            r#"{"from": "api", "to": "db", "technology": "SQL"}"#,
            r#"{"from": "api", "to": "db", "technology": "SQL", "role": "critical"}"#,
        )
        .replace(
            r#"{"from": "api", "to": "mail", "label": "notifies"}"#,
            r#"{"from": "api", "to": "mail", "label": "notifies", "role": "planned"}"#,
        )
    }

    #[test]
    fn roles_mark_outlines_lines_and_badges_and_the_legend_names_only_those_used() {
        let output = svg(&with_roles());
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        let content = &output.content;
        assert!(content.contains("chartlet-flow-role-critical"));
        assert!(content.contains("chartlet-flow-role-planned"));
        assert!(!content.contains("chartlet-flow-role-deprecated\""));
        assert!(content.contains(">!</text>") && content.contains(">+</text>"));
        assert!(content.contains(">Critical</text>") && content.contains(">Planned</text>"));
        assert!(!content.contains(">Deprecated</text>"));
        // A role outlines a box and draws a line in its color and pattern.
        assert!(content.contains(
            ".chartlet-flow-role-planned .chartlet-flow-edge{stroke:var(--chartlet-role-violet);stroke-dasharray:6 4}"
        ));
        // Without roles nothing of them is drawn or styled.
        assert!(!svg(SPEC).content.contains("chartlet-flow-role"));
        let print = render_json(
            &with_roles(),
            RenderFormat::Svg,
            &RenderOptions {
                variant: crate::Variant::Print,
                ..RenderOptions::default()
            },
        )
        .expect("renders")
        .content;
        assert!(print.contains("chartlet-flow-role-critical .chartlet-flow-node{stroke:#b42318"));
        let dark = svg(&with_roles().replace("\"width\"", "\"theme\": \"dark\", \"width\""));
        assert!(dark.content.contains("--chartlet-role-red:#f97066"));
    }

    #[test]
    fn the_text_alternative_names_the_roles() {
        let alternative =
            text_alternative(&ChartSpec::from_json(&with_roles()).expect("parses")).expect("valid");
        assert!(
            alternative.description.ends_with(
                "Roles: critical: API, API → Posts; planned: Mail service, API → Mail service."
            ),
            "{}",
            alternative.description
        );
        // The sentence of roles names them once; the connections keep their usual notes.
        assert!(
            !alternative.description.contains("critical)"),
            "{}",
            alternative.description
        );
        assert_eq!(
            alternative.table.columns,
            ["Component", "Kind", "Boundary", "Role", "Connects to"]
        );
        let api = alternative
            .table
            .rows
            .iter()
            .find(|row| row[0] == "API")
            .expect("a row");
        assert_eq!(api[3], "critical");
        assert_eq!(
            api[4],
            "Posts (via SQL, critical), Images (stores), Mail service (notifies, planned)"
        );
        let german = with_roles().replace(
            "\"schemaVersion\": 1,",
            "\"schemaVersion\": 1, \"locale\": \"de\",",
        );
        let alternative =
            text_alternative(&ChartSpec::from_json(&german).expect("parses")).expect("valid");
        assert!(alternative.description.contains("Rollen: kritisch: API"));
    }

    #[test]
    fn an_unknown_role_and_a_role_with_its_own_pattern_and_a_dash_are_refused() {
        let unknown = SPEC.replace(
            r#"{"id": "reader", "label": "Reader", "kind": "person"}"#,
            r#"{"id": "reader", "label": "Reader", "kind": "person", "role": "urgent"}"#,
        );
        let refused = render_json(&unknown, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("an unknown role");
        assert_eq!(
            (refused.code, refused.path.as_str()),
            ("invalid_spec", "/architecture/components/0/role")
        );
        assert!(
            refused.message.contains("critical") && refused.message.contains("deprecated"),
            "{}",
            refused.message
        );
        let dashed = SPEC.replace(
            r#"{"from": "reader", "to": "site", "label": "reads"}"#,
            r#"{"from": "reader", "to": "site", "label": "reads", "role": "deprecated", "dash": "dashed"}"#,
        );
        assert_eq!(
            error(&dashed),
            (
                "role_conflict",
                "/architecture/connections/0/dash".to_owned()
            )
        );
        let critical = SPEC.replace(
            r#"{"from": "reader", "to": "site", "label": "reads"}"#,
            r#"{"from": "reader", "to": "site", "label": "reads", "role": "critical", "dash": "dashed"}"#,
        );
        assert!(svg(&critical).content.contains("chartlet-flow-dashed"));
    }

    #[test]
    fn a_symbol_is_defined_once_under_the_id_prefix_and_named_in_the_text() {
        let json = SPEC
            .replace(
                r#"{"id": "api", "label": "API", "in": "private"}"#,
                r#"{"id": "api", "label": "API", "in": "private", "symbol": "container"}"#,
            )
            .replace(
                r#""kind": "frontend"}"#,
                r#""kind": "frontend", "symbol": "container"}"#,
            )
            .replace(
                r#""kind": "storage", "in": "cloud"}"#,
                r#""kind": "storage", "in": "cloud", "symbol": "object-storage"}"#,
            );
        let render = |prefix: &str| {
            render_json(
                &json,
                RenderFormat::Svg,
                &RenderOptions {
                    id_prefix: Some(prefix.to_owned()),
                    ..RenderOptions::default()
                },
            )
            .expect("renders")
            .content
        };
        let (first, second) = (render("one"), render("two"));
        assert_eq!(first.matches("<symbol ").count(), 2);
        assert_eq!(first.matches("id=\"one-symbol-container\"").count(), 1);
        assert_eq!(first.matches("href=\"#one-symbol-container\"").count(), 2);
        assert!(first.contains("id=\"one-symbol-object-storage\""));
        assert!(first.contains(".chartlet-arch-symbol{"));
        // Two charts on one page share no id.
        assert!(!second.contains("one-symbol") && second.contains("id=\"two-symbol-container\""));
        // A chart without symbols carries neither definitions nor their style.
        let plain = svg(SPEC).content;
        assert!(!plain.contains("<symbol") && !plain.contains("chartlet-arch-symbol"));

        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(
            alternative
                .description
                .contains("Site (frontend, container): connects to API"),
            "{}",
            alternative.description
        );
        let row = |label: &str| {
            alternative
                .table
                .rows
                .iter()
                .find(|row| row[0] == label)
                .expect("a row")
                .clone()
        };
        assert_eq!(row("API")[1], "service, container");
        assert_eq!(row("Images")[1], "object storage");
        assert_eq!(row("Posts")[1], "database");
        // A category that names the kind is not preceded by it.
        let cache = SPEC.replace(
            r#""kind": "database", "in": "private"}"#,
            r#""kind": "cache", "in": "private", "symbol": "cache"}"#,
        );
        let table = text_alternative(&ChartSpec::from_json(&cache).expect("parses"))
            .expect("valid")
            .table;
        assert_eq!(
            table
                .rows
                .iter()
                .find(|row| row[0] == "Posts")
                .expect("a row")[1],
            "in-memory cache"
        );
    }

    #[test]
    fn an_unknown_symbol_is_named_with_its_field() {
        let json = SPEC.replace(
            r#"{"id": "api", "label": "API", "in": "private"}"#,
            r#"{"id": "api", "label": "API", "in": "private", "symbol": "teapot"}"#,
        );
        let error = render_json(&json, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("an unknown symbol is refused");
        assert_eq!(
            (error.code, error.path.as_str()),
            ("unknown_symbol", "/architecture/components/2/symbol")
        );
        assert!(error.message.contains("load-balancer"), "{}", error.message);
    }

    #[test]
    fn notes_explain_components_without_joining_the_model() {
        let notes = r#""notes": [
                {"on": "db", "text": "Replaced step by step by the new store"},
                {"on": "api", "text": "Rate limited to 100 requests a second"}
            ],
            "mainPath""#;
        let json = SPEC.replace("\"mainPath\"", notes);
        let output = svg(&json.replace("\"height\": 800", "\"height\": 900"));
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert!(
            output
                .content
                .contains("<title>Replaced step by step by the new store</title>")
        );
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-flow-group\"")
                .count(),
            2
        );
        let plain = text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
        let with_notes =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        // Size, boundaries, main path, structure and components read as before.
        assert_eq!(
            with_notes.description,
            format!(
                "{} Note on Posts: Replaced step by step by the new store. Note on API: Rate limited to 100 requests a second.",
                plain.description
            )
        );
        assert_eq!(
            with_notes.table.columns,
            ["Component", "Kind", "Boundary", "Connects to", "Notes"]
        );
        assert_eq!(with_notes.table.rows.len(), plain.table.rows.len());
        let api = with_notes
            .table
            .rows
            .iter()
            .find(|row| row[0] == "API")
            .expect("a row");
        assert_eq!(api[4], "Rate limited to 100 requests a second");
        let on_boundary = json.replace(r#"{"on": "db""#, r#"{"on": "cloud""#);
        assert_eq!(
            error(&on_boundary),
            ("unknown_node", "/architecture/notes/0/on".to_owned())
        );
    }

    #[test]
    fn the_blueprint_theme_is_for_diagrams() {
        let blueprint = SPEC.replace("\"width\"", "\"theme\": \"blueprint\", \"width\"");
        let output = svg(&blueprint);
        assert!(output.content.contains("chartlet-blueprint"));
        let chart = r#"{"schemaVersion":1,"type":"bar","title":"A","theme":"blueprint","data":[{"label":"x","value":1}]}"#;
        assert_eq!(error(chart).0, "option_not_supported");
    }

    /// The blog with a perspective of teams: the API and the posts are stated, the rest not.
    fn with_teams(shown: &str) -> String {
        SPEC.replace(
            r#""mainPath""#,
            &format!(
                r#"{shown}"perspectives": [{{"key": "team", "title": "Team", "description": "Who runs it.", "categories": [
                    {{"label": "Frontend", "components": []}},
                    {{"label": "Backend", "components": ["api"]}},
                    {{"label": "Data", "components": ["db"]}}]}}], "mainPath""#
            ),
        )
    }

    #[test]
    fn a_perspective_names_the_category_of_every_component() {
        let output = svg(&with_teams(r#""perspective": "team", "#)).content;
        // Stripes in the colors of the categories, an outline for what is not stated, and a
        // legend of the categories used, after its title.
        assert!(output.contains(r#"class="chartlet-part chartlet-part-2""#));
        assert!(output.contains(r#"class="chartlet-part chartlet-part-3""#));
        assert!(!output.contains(r#"class="chartlet-part chartlet-part-1""#));
        assert!(output.contains("chartlet-part chartlet-part-none"));
        assert!(output.contains(">Team:</text>") && output.contains(">not stated</text>"));
        // The box names the category below its label.
        assert!(output.contains(r#"class="chartlet-flow-sublabel">Data</text>"#));
        let spec = ChartSpec::from_json(&with_teams(r#""perspective": "team", "#)).unwrap();
        let alternative = text_alternative(&spec).unwrap();
        assert!(
            alternative.description.contains(
                "Perspective Team: Backend: API; Data: Posts; not stated: Reader, Site, Images, Mail service. Who runs it."
            ),
            "{}",
            alternative.description
        );
        let column = alternative
            .table
            .columns
            .iter()
            .position(|column| column == "Team")
            .expect("a column of the perspective");
        let row = |label: &str| {
            alternative
                .table
                .rows
                .iter()
                .find(|row| row[0] == label)
                .expect("a row")[column]
                .clone()
        };
        assert_eq!(
            (row("API"), row("Reader")),
            ("Backend".into(), "not stated".into())
        );
        let options = RenderOptions {
            manifest: true,
            ..RenderOptions::default()
        };
        let manifest = render_json(
            &with_teams(r#""perspective": "team", "#),
            RenderFormat::Svg,
            &options,
        )
        .unwrap()
        .manifest
        .unwrap();
        assert!(manifest.to_json().contains("\"perspective\": \"team\""));
    }

    #[test]
    fn metadata_without_a_shown_perspective_changes_nothing() {
        let options = RenderOptions {
            id_prefix: Some("blog".to_owned()),
            ..RenderOptions::default()
        };
        let plain = render_json(SPEC, RenderFormat::Html, &options)
            .unwrap()
            .content;
        let tagged = render_json(&with_teams(""), RenderFormat::Html, &options)
            .unwrap()
            .content;
        assert_eq!(plain, tagged);
    }

    #[test]
    fn perspective_references_are_checked() {
        let cases = [
            (
                with_teams(r#""perspective": "owner", "#),
                "unknown_perspective",
                "/architecture/perspective",
            ),
            (
                with_teams("").replace(r#"["db"]"#, r#"["database"]"#),
                "unknown_node",
                "/architecture/perspectives/0/categories/2/components/0",
            ),
            (
                with_teams("").replace(r#"["db"]"#, r#"["db", "api"]"#),
                "duplicate_member",
                "/architecture/perspectives/0/categories/2/components/1",
            ),
            (
                with_teams("").replace(r#""label": "Data""#, r#""label": "Backend""#),
                "duplicate_category",
                "/architecture/perspectives/0/categories/2/label",
            ),
            (
                SPEC.replace(
                    r#""mainPath""#,
                    r#""perspectives": [{"key": "team", "title": "Team", "categories": []}], "mainPath""#,
                ),
                "invalid_categories",
                "/architecture/perspectives/0/categories",
            ),
        ];
        for (json, code, path) in cases {
            assert_eq!(error(&json), (code, path.to_owned()));
        }
    }
}
