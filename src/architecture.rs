//! Architecture diagrams: components joined by connections, inside nested boundaries such as a
//! cloud region, a network or a system, laid out by the layered layout of flow charts. A
//! connection says what it does and, in brackets below, how. The description and the data table
//! name every component with its boundaries and where it connects to.

use std::fmt::Write as _;

use crate::{
    DataTable,
    error::ChartWarning,
    flow::{
        Diagram, Edge, Ends, Group, MARGIN, Node, Shape, heaviest_edge, layout_diagram,
        reading_order,
    },
    layout::format_value,
    metrics::TextMetrics,
    roles,
    scene::Scene,
    spec::{ArchitectureSpec, ChartSpec, ComponentKind, DiagramRole},
    structure::{Facts, Structure},
    text,
};

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let architecture = architecture(spec);
    let mut scene = layout_diagram(spec, &diagram(architecture), warnings, metrics);
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
    scene
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

fn diagram(architecture: &ArchitectureSpec) -> Diagram {
    let boundary = |id: &Option<String>| {
        id.as_deref()
            .map(|id| architecture.boundary(id).expect("validated"))
    };
    Diagram {
        nodes: architecture
            .components
            .iter()
            .enumerate()
            .map(|(index, component)| Node {
                label: component.label.clone(),
                sublabel: component.sublabel.clone(),
                shape: shape(component.kind),
                lane: None,
                group: boundary(&component.boundary),
                path: format!("/architecture/components/{index}"),
                compartments: Vec::new(),
                value: None,
                parts: Vec::new(),
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
                ends: Ends::arrow(),
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
        orientation: architecture.orientation,
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
            let notes: Vec<String> = connection
                .step
                .map(|step| text::step_number(locale, step))
                .into_iter()
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
            let label = |id: &str| {
                architecture.components[architecture.component(id).expect("validated")]
                    .label
                    .as_str()
            };
            let ends = format!("{} → {}", label(&connection.from), label(&connection.to));
            match &connection.label {
                Some(note) => format!("{step} {ends} ({note})"),
                None => format!("{step} {ends}"),
            }
        })
        .collect()
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
        Facts { directed: true },
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
        let (from, to) = ends[edge];
        (
            architecture.components[from].label.clone(),
            architecture.components[to].label.clone(),
        )
    }) {
        description.push(' ');
        description.push_str(&heaviest);
    }
    let steps = steps_in_order(architecture);
    if !steps.is_empty() {
        description.push(' ');
        description.push_str(&text::steps(locale, &steps));
    }
    for component in reading_order(&diagram(architecture)) {
        let spec_component = &architecture.components[component];
        write!(
            description,
            " {}",
            text::component(
                locale,
                &spec_component.label,
                spec_component.kind,
                &connects_to(spec, component, false),
            )
        )
        .expect("writing to String cannot fail");
    }
    if let Some(sentence) = roles::sentence(locale, &with_roles(architecture)) {
        description.push(' ');
        description.push_str(&sentence);
    }
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
    reading_order(&diagram(architecture))
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

/// One row per component in reading order: its label, kind, boundaries, role and where it
/// connects to.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let architecture = architecture(spec);
    let words = spec.locale.words();
    let bounded = !architecture.boundaries.is_empty();
    let with_roles = architecture
        .components
        .iter()
        .any(|component| component.role.is_some());
    let mut columns = vec![words.component.to_owned(), words.message_kind.to_owned()];
    if bounded {
        columns.push(words.boundary.to_owned());
    }
    if with_roles {
        columns.push(words.role.to_owned());
    }
    columns.push(words.connects_to.to_owned());
    let rows = reading_order(&diagram(architecture))
        .into_iter()
        .map(|component| {
            let mut row = vec![
                architecture.components[component].label.clone(),
                text::component_kind(spec.locale, architecture.components[component].kind)
                    .to_owned(),
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
            row.push(connects_to(spec, component, true).join(", "));
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
    fn the_blueprint_theme_is_for_diagrams() {
        let blueprint = SPEC.replace("\"width\"", "\"theme\": \"blueprint\", \"width\"");
        let output = svg(&blueprint);
        assert!(output.content.contains("chartlet-blueprint"));
        let chart = r#"{"schemaVersion":1,"type":"bar","title":"A","theme":"blueprint","data":[{"label":"x","value":1}]}"#;
        assert_eq!(error(chart).0, "option_not_supported");
    }
}
