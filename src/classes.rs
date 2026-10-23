//! Class and entity-relationship diagrams: boxes with a name and compartments for attributes and
//! methods, joined by relations with the marks of their kind and their cardinalities, laid out
//! by the layered layout of flow charts. A relation that makes one class the parent of another
//! (inheritance, realization) is laid out from the parent, so that the parent stands first.
//! The description and the data table give every class with its members and its relations.

use std::fmt::Write as _;

use crate::{
    DataTable,
    error::ChartWarning,
    flow::{Diagram, Edge, End, Ends, Node, Shape, layout_diagram},
    metrics::TextMetrics,
    scene::Scene,
    spec::{ChartSpec, ClassesSpec, RelationKind, RelationSpec},
    structure::{Facts, Structure},
    text,
};

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    layout_diagram(spec, &diagram(spec), warnings, metrics)
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
        RelationKind::Association => (End::None, End::None),
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
    let nodes = classes
        .classes
        .iter()
        .enumerate()
        .map(|(index, class)| Node {
            label: class.label.clone(),
            sublabel: class.kind.stereotype().map(str::to_owned),
            shape: Shape::Class,
            lane: None,
            group: None,
            id: Some(class.id.clone()),
            path: format!("/classes/classes/{index}"),
            compartments: vec![class.attributes.clone(), class.methods.clone()],
            value: None,
            parts: Vec::new(),
            symbol: None,
            role: None,
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
                label: relation.label.clone(),
                technology: None,
                dash: relation.kind.dash(),
                main: false,
                label_path: format!("/classes/relations/{index}/label"),
                ends: ends(relation),
                weight: None,
                role: None,
            }
        })
        .collect();
    Diagram {
        nodes,
        edges,
        lanes: Vec::new(),
        groups: Vec::new(),
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
    text::relation_phrase(
        spec.locale,
        (relation.kind, from_side),
        label(other),
        cardinality.as_deref(),
        relation.label.as_deref(),
    )
}

/// The diagram in sentences: how many classes and relations, then every class with its members
/// and its relations.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let classes = classes(spec);
    let mut description =
        text::classes_opening(spec.locale, classes.classes.len(), classes.relations.len());
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
    description
}

/// One row per class: its kind, attributes, methods and the relations it starts.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let classes = classes(spec);
    let words = spec.locale.words();
    let rows = classes
        .classes
        .iter()
        .map(|class| {
            let relations: Vec<String> = classes
                .relations
                .iter()
                .filter(|relation| relation.from == class.id)
                .map(|relation| relation_sentence(spec, relation, true))
                .collect();
            vec![
                class.label.clone(),
                text::class_kind(spec.locale, class.kind).to_owned(),
                class.attributes.join("; "),
                class.methods.join("; "),
                relations.join("; "),
            ]
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: vec![
            words.item.to_owned(),
            words.message_kind.to_owned(),
            words.attributes.to_owned(),
            words.methods.to_owned(),
            words.relations.to_owned(),
        ],
        rows,
    }
}
