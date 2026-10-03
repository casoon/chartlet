//! Architecture diagrams: components joined by connections, inside nested boundaries such as a
//! cloud region, a network or a system, laid out by the layered layout of flow charts. A
//! connection says what it does and, in brackets below, how. The description and the data table
//! name every component with its boundaries and where it connects to.

use std::fmt::Write as _;

use crate::{
    DataTable,
    error::ChartWarning,
    flow::{Diagram, Edge, Group, Node, Shape, layout_diagram, reading_order},
    metrics::TextMetrics,
    scene::Scene,
    spec::{ArchitectureSpec, ChartSpec, ComponentKind, Locale},
    text,
};

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    layout_diagram(spec, &diagram(architecture(spec)), warnings, metrics)
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
        ComponentKind::External => Shape::External,
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
            })
            .collect(),
        edges: architecture
            .connections
            .iter()
            .enumerate()
            .map(|(index, connection)| Edge {
                from: architecture.component(&connection.from).expect("validated"),
                to: architecture.component(&connection.to).expect("validated"),
                label: connection.label.clone(),
                technology: connection.technology.clone(),
                dash: connection.dash,
                main: architecture.on_main_path(index),
                label_path: format!("/architecture/connections/{index}/label"),
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

/// Where a component connects to, as `label (what, how)`, in the order of the connections.
fn connects_to(locale: Locale, architecture: &ArchitectureSpec, component: usize) -> Vec<String> {
    let id = &architecture.components[component].id;
    architecture
        .connections
        .iter()
        .filter(|connection| &connection.from == id)
        .map(|connection| {
            let target = &architecture.components
                [architecture.component(&connection.to).expect("validated")]
            .label;
            match text::connection_note(
                locale,
                connection.label.as_deref(),
                connection.technology.as_deref(),
            ) {
                Some(note) => format!("{target} ({note})"),
                None => target.clone(),
            }
        })
        .collect()
}

/// The diagram in sentences: its size, the boundaries and what each holds, the main path, and
/// every component in reading order with where it connects to.
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
    if !architecture.main_path.is_empty() {
        let path: Vec<&str> = architecture
            .main_path
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
    for component in reading_order(&diagram(architecture)) {
        let spec_component = &architecture.components[component];
        write!(
            description,
            " {}",
            text::component(
                locale,
                &spec_component.label,
                spec_component.kind,
                &connects_to(locale, architecture, component),
            )
        )
        .expect("writing to String cannot fail");
    }
    description
}

/// One row per component in reading order: its label, kind, boundaries and where it connects to.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let architecture = architecture(spec);
    let words = spec.locale.words();
    let bounded = !architecture.boundaries.is_empty();
    let mut columns = vec![words.component.to_owned(), words.message_kind.to_owned()];
    if bounded {
        columns.push(words.boundary.to_owned());
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
            row.push(connects_to(spec.locale, architecture, component).join(", "));
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
        "height": 700,
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
                output.warnings.is_empty(),
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
}
