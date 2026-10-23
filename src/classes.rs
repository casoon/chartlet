//! Class and entity-relationship diagrams: boxes with a name and compartments for attributes and
//! methods, joined by relations with the marks of their kind and their cardinalities, laid out
//! by the layered layout of flow charts. A relation that makes one class the parent of another
//! (inheritance, realization) is laid out from the parent, so that the parent stands first.
//! Packages frame the classes that name them, nested like the boundaries of an architecture.
//! Notes stand beside the classes they explain, joined by a dotted line that is no relation.
//! The description and the data table give every class with its package, its members, its
//! relations and its notes.

use std::fmt::Write as _;

use crate::{
    DataTable,
    error::ChartWarning,
    flow::MARGIN,
    flow::{Diagram, Edge, End, Ends, Group, Node, Shape, layout_diagram},
    metrics::TextMetrics,
    roles,
    scene::Scene,
    spec::{ChartSpec, ClassesSpec, ConnectionDirection, DiagramRole, RelationKind, RelationSpec},
    structure::{Facts, Structure},
    text,
};

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let classes = classes(spec);
    let mut scene = layout_diagram(spec, &diagram(spec), warnings, metrics);
    let used = roles::used(
        classes.classes.iter().map(|class| class.role),
        classes.relations.iter().map(|relation| relation.role),
    );
    roles::push_legend(
        &mut scene,
        spec.locale,
        &used,
        (MARGIN, roles::push_badge),
        metrics,
    );
    scene
}

fn classes(spec: &ChartSpec) -> &ClassesSpec {
    spec.classes
        .as_ref()
        .expect("validated class diagrams carry a classes block")
}

/// Whether the layout starts a relation at its `to` class: the parent of an inheritance stands
/// before its child.
const fn reversed(kind: RelationKind) -> bool {
    matches!(kind, RelationKind::Inheritance | RelationKind::Realization)
}

/// The marks at the two ends of a relation as the layered layout draws it, from the first class
/// of the edge to the second.
fn ends(relation: &RelationSpec) -> Ends {
    let (from_text, to_text) = (
        relation.from_cardinality.clone(),
        relation.to_cardinality.clone(),
    );
    let (tail, head) = match relation.kind {
        RelationKind::Association => match relation.direction {
            Some(ConnectionDirection::Forward) => (End::None, End::Open),
            Some(ConnectionDirection::Both) => (End::Open, End::Open),
            Some(ConnectionDirection::None) | None => (End::None, End::None),
        },
        RelationKind::Inheritance | RelationKind::Realization => (End::Triangle, End::None),
        RelationKind::Composition => (End::DiamondFilled, End::None),
        RelationKind::Aggregation => (End::DiamondHollow, End::None),
        RelationKind::Dependency => (End::None, End::Open),
    };
    if reversed(relation.kind) {
        Ends {
            tail,
            head,
            tail_text: to_text,
            head_text: from_text,
        }
    } else {
        Ends {
            tail,
            head,
            tail_text: from_text,
            head_text: to_text,
        }
    }
}

fn diagram(spec: &ChartSpec) -> Diagram {
    let classes = classes(spec);
    let mut diagram = model_diagram(spec);
    for (index, note) in classes.notes.iter().enumerate() {
        let target = classes.class(&note.on).expect("validated");
        diagram.push_note(target, &note.text, format!("/classes/notes/{index}"));
    }
    diagram
}

/// The diagram of the classes and their relations, without notes.
fn model_diagram(spec: &ChartSpec) -> Diagram {
    let classes = classes(spec);
    let package = |id: &Option<String>| {
        id.as_deref()
            .map(|id| classes.package(id).expect("validated"))
    };
    let nodes = classes
        .classes
        .iter()
        .enumerate()
        .map(|(index, class)| Node {
            label: class.label.clone(),
            sublabel: class.kind.stereotype().map(str::to_owned),
            shape: Shape::Class,
            lane: None,
            group: package(&class.package),
            id: Some(class.id.clone()),
            path: format!("/classes/classes/{index}"),
            compartments: vec![class.attributes.clone(), class.methods.clone()],
            value: None,
            parts: Vec::new(),
            symbol: None,
            role: class.role,
            category: None,
        })
        .collect();
    let edges = classes
        .relations
        .iter()
        .enumerate()
        .map(|(index, relation)| {
            let (from, to) = (
                classes.class(&relation.from).expect("validated"),
                classes.class(&relation.to).expect("validated"),
            );
            let (from, to) = if reversed(relation.kind) {
                (to, from)
            } else {
                (from, to)
            };
            Edge {
                from,
                to,
                id: None,
                label: relation.label.clone(),
                technology: None,
                dash: relation.kind.dash(),
                main: false,
                label_path: format!("/classes/relations/{index}/label"),
                ends: ends(relation),
                sign: relation.direction.map_or("→", ConnectionDirection::sign),
                weight: None,
                role: relation.role,
            }
        })
        .collect();
    Diagram {
        nodes,
        edges,
        lanes: Vec::new(),
        groups: classes
            .packages
            .iter()
            .enumerate()
            .map(|(index, group)| Group {
                label: group.label.clone(),
                id: Some(group.id.clone()),
                path: format!("/classes/packages/{index}"),
                parent: package(&group.parent),
            })
            .collect(),
        ranks: Vec::new(),
        orientation: classes.orientation,
    }
}

/// The relation of `class` to another as a sentence part: `inherits from Animal`, with its
/// cardinalities when it has any.
fn relation_sentence(spec: &ChartSpec, relation: &RelationSpec, from_side: bool) -> String {
    let classes = classes(spec);
    let label = |id: &str| {
        classes.classes[classes.class(id).expect("validated")]
            .label
            .as_str()
    };
    let other = if from_side {
        &relation.to
    } else {
        &relation.from
    };
    let cardinality = match (&relation.from_cardinality, &relation.to_cardinality) {
        (Some(from), Some(to)) => Some(format!("{from} : {to}")),
        (Some(one), None) | (None, Some(one)) => Some(one.clone()),
        (None, None) => None,
    };
    let mut phrase = text::relation_phrase(
        spec.locale,
        (relation.kind, from_side),
        label(other),
        cardinality.as_deref(),
        relation.label.as_deref(),
    );
    if let Some(navigable) = relation
        .direction
        .and_then(|direction| text::navigable(spec.locale, direction, label(&relation.to)))
    {
        write!(phrase, ", {navigable}").expect("writing to String cannot fail");
    }
    phrase
}

/// The packages around a class, outermost first, by label: `Shop › Ordering`.
fn package_path(classes: &ClassesSpec, class: usize) -> String {
    classes
        .packages_of(class)
        .iter()
        .map(|index| classes.packages[*index].label.as_str())
        .collect::<Vec<_>>()
        .join(" › ")
}

/// The diagram in sentences: how many classes and relations, the packages and what each holds,
/// then every class with its members and its relations.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let classes = classes(spec);
    let mut description = text::classes_opening(
        spec.locale,
        classes.classes.len(),
        classes.relations.len(),
        classes.packages.len(),
    );
    for package in &classes.packages {
        let parent = package
            .parent
            .as_deref()
            .and_then(|id| classes.package(id))
            .map(|parent| classes.packages[parent].label.as_str());
        let held: Vec<&str> = classes
            .classes
            .iter()
            .filter(|class| class.package.as_deref() == Some(package.id.as_str()))
            .map(|class| class.label.as_str())
            .chain(
                classes
                    .packages
                    .iter()
                    .filter(|inner| inner.parent.as_deref() == Some(package.id.as_str()))
                    .map(|inner| inner.label.as_str()),
            )
            .collect();
        description.push(' ');
        description.push_str(&text::boundary(&package.label, parent, &held));
    }
    let ends: Vec<(usize, usize)> = classes
        .relations
        .iter()
        .map(|relation| {
            let index = |id: &str| classes.class(id).expect("validated");
            (index(&relation.from), index(&relation.to))
        })
        .collect();
    let structure = Structure::of(classes.classes.len(), &ends, &[], Facts { directed: false })
        .sentences(spec.locale, |class| classes.classes[class].label.clone());
    if !structure.is_empty() {
        description.push(' ');
        description.push_str(&structure);
    }
    for class in &classes.classes {
        let relations: Vec<String> = classes
            .relations
            .iter()
            .filter(|relation| relation.from == class.id)
            .map(|relation| relation_sentence(spec, relation, true))
            .collect();
        write!(
            description,
            " {}",
            text::class_sentence(
                spec.locale,
                &class.label,
                (class.attributes.len(), class.methods.len()),
                &relations,
            )
        )
        .expect("writing to String cannot fail");
    }
    let notes: Vec<(&str, &str)> = classes
        .notes
        .iter()
        .map(|note| {
            let target = classes.class(&note.on).expect("validated");
            (classes.classes[target].label.as_str(), note.text.as_str())
        })
        .collect();
    if !notes.is_empty() {
        description.push(' ');
        description.push_str(&text::notes(spec.locale, &notes));
    }
    if let Some(sentence) = roles::sentence(spec.locale, &with_roles(classes)) {
        description.push(' ');
        description.push_str(&sentence);
    }
    description
}

/// Every class with a role, in the order of the specification, then every relation with one, as
/// `From → To`.
fn with_roles(classes: &ClassesSpec) -> Vec<(DiagramRole, String)> {
    let label = |id: &str| {
        classes.classes[classes.class(id).expect("validated")]
            .label
            .as_str()
    };
    classes
        .classes
        .iter()
        .filter_map(|class| class.role.map(|role| (role, class.label.clone())))
        .chain(classes.relations.iter().filter_map(|relation| {
            relation.role.map(|role| {
                (
                    role,
                    format!("{} → {}", label(&relation.from), label(&relation.to)),
                )
            })
        }))
        .collect()
}

/// One row per class: its kind, its packages when there are any, attributes, methods and the
/// relations it starts.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let classes = classes(spec);
    let words = spec.locale.words();
    let packaged = !classes.packages.is_empty();
    let with_notes = !classes.notes.is_empty();
    let with_roles = classes.classes.iter().any(|class| class.role.is_some());
    let rows = classes
        .classes
        .iter()
        .enumerate()
        .map(|(index, class)| {
            let relations: Vec<String> = classes
                .relations
                .iter()
                .filter(|relation| relation.from == class.id)
                .map(|relation| {
                    let sentence = relation_sentence(spec, relation, true);
                    match relation.role {
                        Some(role) => format!("{sentence} ({})", text::role(spec.locale, role)),
                        None => sentence,
                    }
                })
                .collect();
            let mut row = vec![
                class.label.clone(),
                text::class_kind(spec.locale, class.kind).to_owned(),
            ];
            if packaged {
                row.push(package_path(classes, index));
            }
            if with_roles {
                row.push(
                    class
                        .role
                        .map_or_else(String::new, |role| text::role(spec.locale, role).to_owned()),
                );
            }
            row.extend([
                class.attributes.join("; "),
                class.methods.join("; "),
                relations.join("; "),
            ]);
            if with_notes {
                let notes: Vec<&str> = classes
                    .notes
                    .iter()
                    .filter(|note| note.on == class.id)
                    .map(|note| note.text.as_str())
                    .collect();
                row.push(notes.join("; "));
            }
            row
        })
        .collect();
    let mut columns = vec![words.item.to_owned(), words.message_kind.to_owned()];
    if packaged {
        columns.push(words.package.to_owned());
    }
    if with_roles {
        columns.push(words.role.to_owned());
    }
    columns.extend([
        words.attributes.to_owned(),
        words.methods.to_owned(),
        words.relations.to_owned(),
    ]);
    if with_notes {
        columns.push(words.notes.to_owned());
    }
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use crate::{ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative};

    const SPEC: &str = r#"{"schemaVersion": 1, "type": "classes", "title": "Orders",
        "classes": {"classes": [
            {"id": "customer", "label": "Customer"},
            {"id": "order", "label": "Order"}
        ], "relations": [
            {"from": "customer", "to": "order", "label": "places", "toCardinality": "0..*"}
        ]}}"#;

    fn svg(json: &str) -> Result<String, (&'static str, String)> {
        render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .map(|output| output.content)
            .map_err(|error| (error.code, error.path))
    }

    fn with(direction: &str) -> String {
        SPEC.replace(
            r#""label": "places","#,
            &format!(r#""label": "places", "direction": "{direction}","#),
        )
    }

    #[test]
    fn an_association_is_navigable_across_packages() {
        let json = with("forward")
            .replace(
                r#""classes": {"classes": ["#,
                r#""classes": {"packages": [
                    {"id": "crm", "label": "CRM"}, {"id": "sales", "label": "Sales"}
                ], "classes": ["#,
            )
            .replace(
                r#"{"id": "customer", "label": "Customer"}"#,
                r#"{"id": "customer", "label": "Customer", "in": "crm"}"#,
            )
            .replace(
                r#"{"id": "order", "label": "Order"}"#,
                r#"{"id": "order", "label": "Order", "in": "sales"}"#,
            );
        let output =
            render_json(&json, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-flow-open\"")
                .count(),
            1
        );
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-flow-group\"")
                .count(),
            2
        );
        let description = text_alternative(&ChartSpec::from_json(&json).expect("parses"))
            .expect("valid")
            .description;
        assert!(
            description.contains("[places], navigable to Order"),
            "{description}"
        );
    }

    #[test]
    fn an_association_is_navigable_one_way_both_ways_or_not_said() {
        let plain = svg(SPEC).expect("renders");
        // The drawing after the stylesheet; the id before it follows the specification.
        let drawing =
            |content: &str| content[content.find("</style>").expect("styled")..].to_owned();
        assert_eq!(
            drawing(&svg(&with("none")).expect("renders")),
            drawing(&plain).replace("→", "—")
        );
        let open = |content: &str| content.matches("class=\"chartlet-flow-open\"").count();
        assert_eq!(open(&plain), 0);
        assert_eq!(open(&svg(&with("forward")).expect("renders")), 1);
        let both = svg(&with("both")).expect("renders");
        assert_eq!(open(&both), 2);
        assert!(both.contains("<title>Customer ↔ Order: places</title>"));
        let sentence = |json: &str| {
            text_alternative(&ChartSpec::from_json(json).expect("parses"))
                .expect("valid")
                .description
        };
        assert!(
            sentence(&with("forward"))
                .contains("is associated with Order (0..*) [places], navigable to Order")
        );
        assert!(sentence(&with("both")).contains("[places], navigable both ways"));
        assert_eq!(sentence(&with("none")), sentence(SPEC));
        let dependency = with("forward").replace(
            r#""label": "places","#,
            r#""label": "places", "kind": "dependency","#,
        );
        assert_eq!(
            svg(&dependency),
            Err((
                "option_not_supported",
                "/classes/relations/0/direction".to_owned()
            ))
        );
    }

    #[test]
    fn classes_and_relations_take_roles_and_keep_their_marks() {
        let json = SPEC
            .replace(
                r#"{"id": "order", "label": "Order"}"#,
                r#"{"id": "order", "label": "Order", "role": "critical"},
                {"id": "invoice", "label": "Invoice", "role": "deprecated"}"#,
            )
            .replace(
                r#""toCardinality": "0..*"}"#,
                r#""toCardinality": "0..*", "role": "planned"},
                {"from": "invoice", "to": "order", "kind": "realization", "role": "critical"}"#,
            );
        let content = svg(&json).expect("renders");
        assert!(content.contains("chartlet-flow-role-planned"));
        // A hollow triangle stays hollow under a role.
        assert!(content.contains(".chartlet-flow-role-critical .chartlet-flow-hollow"));
        assert!(content.contains(">Critical</text>"));
        assert!(content.contains(">Planned</text>"));
        assert!(content.contains(">Deprecated</text>"));
        let alternative =
            text_alternative(&ChartSpec::from_json(&json).expect("parses")).expect("valid");
        assert!(
            alternative.description.ends_with(
                "Roles: critical: Order, Invoice → Order; planned: Customer → Order; deprecated: Invoice."
            ),
            "{}",
            alternative.description
        );
        assert_eq!(alternative.table.columns[2], "Role");
        assert_eq!(alternative.table.rows[1][2], "critical");
        assert!(
            alternative.table.rows[0][5].ends_with("(planned)"),
            "{:?}",
            alternative.table.rows[0]
        );
        let german = json.replace(
            r#""schemaVersion": 1,"#,
            r#""schemaVersion": 1, "locale": "de","#,
        );
        let alternative =
            text_alternative(&ChartSpec::from_json(&german).expect("parses")).expect("valid");
        assert!(
            alternative.description.contains("Rollen: kritisch: Order"),
            "{}",
            alternative.description
        );
        // The dashed line of a realization or a dependency is its kind: no pattern of a role
        // over it.
        for kind in ["realization", "dependency"] {
            let conflict = json.replace(
                r#""kind": "realization", "role": "critical""#,
                &format!(r#""kind": "{kind}", "role": "planned""#),
            );
            assert_eq!(
                svg(&conflict),
                Err(("role_conflict", "/classes/relations/1/role".to_owned()))
            );
        }
    }
}
