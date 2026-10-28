//! Class and entity-relationship diagrams: boxes with a name and compartments for attributes and
//! methods, joined by relations with the marks of their kind and their cardinalities, laid out
//! by the layered layout of flow charts. A relation that makes one class the parent of another
//! (inheritance, realization) is laid out from the parent, so that the parent stands first.
//! Packages frame the classes that name them, nested like the boundaries of an architecture.
//! Notes stand beside the classes they explain, joined by a dotted line that is no relation, at
//! the head of a package, or below the diagram when they explain it as a whole.
//! The description and the data table give every class with its package, its members, its
//! relations and its notes; the table's caption carries the notes no class row does.

use std::fmt::Write as _;

use crate::{
    DataTable,
    error::ChartWarning,
    flow::{
        Block, Diagram, Edge, End, Ends, Group, MARGIN, Node, Shape, Sides, Whole,
        blocks_reachable, layout_diagram,
        notes::{self, Target},
    },
    metrics::TextMetrics,
    roles,
    scene::Scene,
    spec::{
        ChartSpec, ClassesSpec, ConnectionDirection, DiagramOrientation, DiagramRole, RelationKind,
        RelationSpec,
    },
    structure::{Facts, Structure},
    text,
};

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let classes = classes(spec);
    let mut scene = match with_blocks(spec, metrics, warnings) {
        Some(diagram) => layout_diagram(spec, &diagram, warnings, metrics),
        None => layout_diagram(spec, &diagram(spec), warnings, metrics),
    };
    notes::push_band(
        &mut scene,
        spec,
        (&classes.notes, "classes"),
        metrics,
        warnings,
    );
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
    notes::push_notes(&mut diagram, &classes.notes, "classes", |id| {
        classes.class(id).map_or_else(
            || Target::Frame(classes.package(id).expect("validated")),
            Target::Node,
        )
    });
    diagram
}

/// The diagram with every package that has an orientation of its own laid out on its own, as a
/// block that the diagram places as one node; `None` without such a package, or when a relation
/// from outside could not reach its class in a block straight in any orientation the diagram may
/// take, which is reported.
fn with_blocks(
    spec: &ChartSpec,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Option<Diagram> {
    let classes = classes(spec);
    let oriented: Vec<usize> = (0..classes.packages.len())
        .filter(|package| classes.packages[*package].orientation.is_some())
        .collect();
    if oriented.is_empty() {
        return None;
    }
    let mut block_warnings = Vec::new();
    let mut diagram = split(spec, &oriented, metrics, &mut block_warnings);
    let [portrait, landscape] = blocks_reachable(&diagram);
    let orientation = match (classes.orientation, portrait, landscape) {
        (DiagramOrientation::Portrait, true, _) | (DiagramOrientation::Auto, true, false) => {
            DiagramOrientation::Portrait
        }
        (DiagramOrientation::Landscape, _, true) | (DiagramOrientation::Auto, false, true) => {
            DiagramOrientation::Landscape
        }
        (DiagramOrientation::Auto, true, true) => DiagramOrientation::Auto,
        _ => {
            for package in oriented {
                warnings.push(ChartWarning::new(
                    "package_orientation_ignored",
                    format!("/classes/packages/{package}/orientation"),
                    "a relation from outside the package would have to cross it to reach its \
                     class; the package follows the orientation of the diagram",
                ));
            }
            return None;
        }
    };
    diagram.orientation = orientation;
    warnings.append(&mut block_warnings);
    Some(diagram)
}

/// The diagram of the classes, their relations and notes, with the classes and notes of each of
/// the packages `oriented` laid out as a block in the package's orientation, with the packages
/// inside it, and the relations between them; the diagram takes each block as one node, in the
/// place of its first class, inside the package's own package.
#[allow(clippy::too_many_lines)]
fn split(
    spec: &ChartSpec,
    oriented: &[usize],
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Diagram {
    let classes = classes(spec);
    let whole = diagram(spec);
    let all_ends: Vec<(usize, usize)> = whole
        .edges
        .iter()
        .map(|edge| (edge.from, edge.to))
        .collect();
    let labels: Vec<String> = whole.nodes.iter().map(|node| node.label.clone()).collect();
    let block_of: Vec<Option<usize>> = (0..whole.nodes.len())
        .map(|node| {
            oriented
                .iter()
                .position(|package| whole.inside(node, *package))
        })
        .collect();
    let group_block: Vec<Option<usize>> = (0..whole.groups.len())
        .map(|group| {
            oriented
                .iter()
                .position(|package| whole.chain(Some(group)).contains(package))
        })
        .collect();
    // The outer diagram: every node outside the blocks, and a node for each block.
    let mut outer_of = vec![0; whole.nodes.len()];
    let mut outer_nodes: Vec<usize> = Vec::new();
    for node in 0..whole.nodes.len() {
        match block_of[node] {
            None => {
                outer_of[node] = outer_nodes.len();
                outer_nodes.push(node);
            }
            Some(block) => {
                let first = block_of.iter().position(|of| *of == Some(block));
                if first == Some(node) {
                    outer_nodes.push(node);
                }
                outer_of[node] = outer_nodes.len() - 1;
            }
        }
    }
    let outer_groups: Vec<usize> = (0..whole.groups.len())
        .filter(|group| group_block[*group].is_none())
        .collect();
    let outer_group = |group: Option<usize>| {
        group.map(|group| {
            outer_groups
                .iter()
                .position(|kept| *kept == group)
                .expect("a group outside the blocks")
        })
    };
    let Diagram {
        nodes,
        edges,
        groups,
        ..
    } = whole;
    let mut nodes: Vec<Option<Node>> = nodes.into_iter().map(Some).collect();
    let mut edges: Vec<Option<Edge>> = edges.into_iter().map(Some).collect();
    let mut groups: Vec<Option<Group>> = groups.into_iter().map(Some).collect();
    let whole_parent: Vec<Option<usize>> = groups
        .iter()
        .map(|group| group.as_ref().and_then(|group| group.parent))
        .collect();
    // Each block, laid out on its own.
    let mut inner_diagrams = Vec::new();
    for (block, &package) in oriented.iter().enumerate() {
        let members: Vec<usize> = (0..nodes.len())
            .filter(|node| block_of[*node] == Some(block))
            .collect();
        let inner_groups: Vec<usize> = (0..groups.len())
            .filter(|group| group_block[*group] == Some(block))
            .collect();
        let inner_group = |group: Option<usize>| {
            group.and_then(|group| inner_groups.iter().position(|kept| *kept == group))
        };
        let inner_edges: Vec<usize> = (0..edges.len())
            .filter(|edge| {
                let (from, to) = all_ends[*edge];
                block_of[from] == Some(block) && block_of[to] == Some(block)
            })
            .collect();
        let inner_of = |node: usize| {
            members
                .iter()
                .position(|member| *member == node)
                .expect("a member of the block")
        };
        inner_diagrams.push(Diagram {
            nodes: members
                .iter()
                .map(|&node| {
                    let node = nodes[node].take().expect("each node in one place");
                    Node {
                        group: inner_group(node.group),
                        ..node
                    }
                })
                .collect(),
            edges: inner_edges
                .iter()
                .map(|&edge| {
                    let edge = edges[edge].take().expect("each edge in one place");
                    Edge {
                        from: inner_of(edge.from),
                        to: inner_of(edge.to),
                        ..edge
                    }
                })
                .collect(),
            lanes: Vec::new(),
            groups: inner_groups
                .iter()
                .map(|&group| {
                    let group = groups[group].take().expect("each group in one place");
                    Group {
                        parent: inner_group(group.parent),
                        ..group
                    }
                })
                .collect(),
            ranks: Vec::new(),
            orientation: classes.packages[package]
                .orientation
                .expect("an oriented package"),
            spacing: classes.spacing,
            blocks: Vec::new(),
            whole: Some(Whole {
                nodes: members.clone(),
                ends: inner_edges.iter().map(|edge| all_ends[*edge]).collect(),
                all_ends: all_ends.clone(),
                labels: labels.clone(),
            }),
        });
    }
    let outer_edges: Vec<usize> = (0..edges.len())
        .filter(|edge| edges[*edge].is_some())
        .collect();
    let blocks = inner_diagrams
        .iter()
        .zip(oriented)
        .map(|(inner, _)| {
            let first = inner.whole.as_ref().expect("named in the whole").nodes[0];
            Block::new(spec, inner, outer_of[first], metrics, warnings)
        })
        .collect();
    Diagram {
        nodes: outer_nodes
            .iter()
            .map(|&node| match block_of[node] {
                None => {
                    let node = nodes[node].take().expect("each node in one place");
                    Node {
                        group: outer_group(node.group),
                        ..node
                    }
                }
                Some(block) => {
                    let package = oriented[block];
                    Node {
                        label: classes.packages[package].label.clone(),
                        sublabel: None,
                        shape: Shape::Block,
                        lane: None,
                        group: outer_group(whole_parent[package]),
                        id: None,
                        path: format!("/classes/packages/{package}"),
                        compartments: Vec::new(),
                        value: None,
                        parts: Vec::new(),
                        symbol: None,
                        tag: None,
                        role: None,
                        category: None,
                    }
                }
            })
            .collect(),
        edges: outer_edges
            .iter()
            .map(|&edge| {
                let edge = edges[edge].take().expect("each edge in one place");
                Edge {
                    from: outer_of[edge.from],
                    to: outer_of[edge.to],
                    ..edge
                }
            })
            .collect(),
        lanes: Vec::new(),
        groups: outer_groups
            .iter()
            .map(|&group| {
                let group = groups[group].take().expect("each group in one place");
                Group {
                    parent: outer_group(group.parent),
                    ..group
                }
            })
            .collect(),
        ranks: Vec::new(),
        orientation: classes.orientation,
        spacing: classes.spacing,
        blocks,
        whole: Some(Whole {
            nodes: outer_nodes,
            ends: outer_edges.iter().map(|edge| all_ends[*edge]).collect(),
            all_ends,
            labels,
        }),
    }
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
            tag: None,
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
                category: None,
                sides: Sides::default(),
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
        spacing: classes.spacing,
        ..Diagram::default()
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
    description.push_str(&notes::sentences(spec.locale, &classes.notes, |id| {
        label(classes, id)
    }));
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
    let with_notes = classes.notes.iter().any(|note| {
        note.on
            .as_deref()
            .is_some_and(|on| classes.class(on).is_some())
    });
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
                row.push(notes::of(&classes.notes, &class.id));
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
        caption: notes::caption(
            format!("{} {}", words.data_for, spec.title),
            spec.locale,
            &classes.notes,
            |id| classes.class(id).is_some(),
            |id| label(classes, id),
        ),
        columns,
        rows,
    }
}

/// The label of the class or package `id`.
fn label<'a>(classes: &'a ClassesSpec, id: &str) -> &'a str {
    classes.class(id).map_or_else(
        || &classes.packages[classes.package(id).expect("validated")].label,
        |class| &classes.classes[class].label,
    )
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

    const INVOICES: &str = include_str!("../examples/invoice-model.json");

    /// The box of the class `name` in `svg`: left, top, width, height.
    fn class_box(svg: &str, name: &str) -> (f64, f64, f64, f64) {
        let end = svg
            .find(&format!("<title>{name}</title>"))
            .unwrap_or_else(|| panic!("{name} is drawn"));
        let rect = &svg[svg[..end].rfind("<rect").expect("a box")..end];
        let number = |attribute: &str| -> f64 {
            let start = rect
                .find(&format!(" {attribute}=\""))
                .expect("the attribute")
                + attribute.len()
                + 3;
            rect[start..]
                .split('"')
                .next()
                .and_then(|value| value.parse().ok())
                .expect("a number")
        };
        (number("x"), number("y"), number("width"), number("height"))
    }

    /// The points of the line with the tooltip `title`.
    fn line(svg: &str, title: &str) -> Vec<(f64, f64)> {
        let end = svg
            .find(&format!("<title>{title}</title>"))
            .unwrap_or_else(|| panic!("{title} is drawn"));
        let start = svg[..end].rfind("points=\"").expect("a line") + 8;
        svg[start..]
            .split('"')
            .next()
            .expect("points")
            .split(' ')
            .map(|pair| {
                let (x, y) = pair.split_once(',').expect("a point");
                (x.parse().expect("x"), y.parse().expect("y"))
            })
            .collect()
    }

    #[test]
    fn a_package_lays_out_its_classes_in_its_own_orientation() {
        for variant in [crate::Variant::Desktop, crate::Variant::Mobile] {
            let json = INVOICES.replacen('{', r#"{"mobile": {"width": 360, "height": 900},"#, 1);
            let options = RenderOptions {
                variant,
                ..RenderOptions::default()
            };
            let output = render_json(&json, RenderFormat::Svg, &options).expect("renders");
            // A landscape package is wider than a phone; that is all the layout may report.
            assert!(
                output
                    .warnings
                    .iter()
                    .all(|warning| warning.code == "canvas_too_small"),
                "{:?}",
                output.warnings
            );
            let again = render_json(&json, RenderFormat::Svg, &options).expect("renders");
            assert_eq!(
                again.content, output.content,
                "{variant:?} is deterministic"
            );
            let svg = output.content;
            let middle = |name: &str| {
                let (x, y, width, height) = class_box(&svg, name);
                (x + width / 2.0, y + height / 2.0)
            };
            // The diagram runs down, the billing package right.
            assert!(middle("Customer").1 < middle("Order").1);
            assert!(middle("Order").1 < middle("Invoice").1);
            assert!(middle("Invoice").0 < middle("InvoiceLine").0);
            assert!(middle("InvoiceLine").0 < middle("TaxRate").0);
            assert!((middle("InvoiceLine").1 - middle("TaxRate").1).abs() < 0.01);
            // A relation from outside reaches the top of its class inside the package, and one to
            // a class outside leaves the bottom of its class.
            let (x, y, width, height) = class_box(&svg, "Invoice");
            let billed = line(&svg, "Order → Invoice: billed as");
            let tip = billed[billed.len() - 1];
            assert!(
                (tip.1 - y).abs() < 0.01 && x < tip.0 && tip.0 < x + width,
                "{tip:?}"
            );
            let dunned = line(&svg, "Invoice → Reminder: dunned by");
            assert!((dunned[0].1 - (y + height)).abs() < 0.01, "{:?}", dunned[0]);
            // Every class keeps its own focus, hotspot and hook.
            for index in 0..6 {
                assert!(svg.contains(&format!("chartlet-n-{index}")), "{index}");
            }
        }
        // Without the package's orientation, the classes follow the diagram.
        let plain = svg(&INVOICES.replace(r#", "orientation": "landscape""#, "")).expect("renders");
        let (_, line_y, _, _) = class_box(&plain, "InvoiceLine");
        let (_, invoice_y, _, _) = class_box(&plain, "Invoice");
        assert!(line_y > invoice_y);
    }

    #[test]
    fn a_package_that_relations_would_cross_follows_the_diagram() {
        // A second line under the invoice, reached from the order: in a landscape package it lies
        // below the first, behind it from above, and behind the invoice's edges from the left.
        let json = INVOICES
            .replace(
                r#"{
        "id": "reminder","#,
                r#"{
        "id": "credit",
        "label": "CreditNote",
        "attributes": ["+ amount: Money"],
        "in": "billing"
      },
      {
        "id": "reminder","#,
            )
            .replace(
                r#"{ "from": "invoice", "to": "reminder","#,
                r#"{ "from": "invoice", "to": "credit", "label": "corrected by" },
      { "from": "order", "to": "credit", "label": "refunds" },
      { "from": "invoice", "to": "reminder","#,
            );
        let output =
            render_json(&json, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
        let warnings: Vec<(&str, &str)> = output
            .warnings
            .iter()
            .filter(|warning| warning.code != "canvas_too_small")
            .map(|warning| (warning.code, warning.path.as_str()))
            .collect();
        assert_eq!(
            warnings,
            [(
                "package_orientation_ignored",
                "/classes/packages/1/orientation"
            )]
        );
        let plain = render_json(
            &json.replace(r#", "orientation": "landscape""#, ""),
            RenderFormat::Svg,
            &RenderOptions::default(),
        )
        .expect("renders");
        let drawing = |svg: &str| svg.split_once("</style>").map(|(_, rest)| rest.to_owned());
        assert_eq!(drawing(&output.content), drawing(&plain.content));
    }

    #[test]
    fn invalid_package_orientations_name_the_field() {
        let auto = INVOICES.replace(r#""orientation": "landscape""#, r#""orientation": "auto""#);
        assert_eq!(
            svg(&auto),
            Err((
                "option_not_supported",
                "/classes/packages/1/orientation".to_owned()
            ))
        );
        let nested = INVOICES.replace(
            r#"{ "id": "sales", "label": "Sales" }"#,
            r#"{ "id": "sales", "label": "Sales", "orientation": "landscape", "in": "billing" }"#,
        );
        assert_eq!(
            svg(&nested),
            Err((
                "option_not_supported",
                "/classes/packages/0/orientation".to_owned()
            ))
        );
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
