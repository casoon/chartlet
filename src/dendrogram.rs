//! Dendrograms: a tree that chartlet builds itself from the distances between items, joining
//! the two closest clusters again and again (agglomerative clustering) until one is left. The
//! items stand in rows on the left, in the order the tree gives them; every join is a bracket at
//! the distance where it happens, on a distance axis below. The description names the closest
//! pair and the two groups of the last join; the table lists every join.

use crate::{
    DataTable,
    diagram::{pixels, polyline, warn_growth_advising},
    error::ChartWarning,
    layout::{
        LABEL_SIZE, NARROW, NumericScale, count, fit_text, format_value, push_title, title_extra,
    },
    metrics::TextMetrics,
    scene::{Element, Line, Scene, Text, TextAnchor},
    spec::{ChartSpec, DistancesSpec, Linkage},
    text,
};

const MARGIN: f64 = 24.0;
const COMPACT_MARGIN: f64 = 12.0;
const BOTTOM: f64 = 16.0;
const ROW: f64 = 22.0;
const MAX_ROW: f64 = 34.0;
const AXIS: f64 = 46.0;
const MAX_LABEL: f64 = 180.0;
const COMPACT_MAX_LABEL: f64 = 110.0;
/// How many members a cluster's name lists before it counts the rest.
const NAMED: usize = 3;

/// One join: the two clusters it brings together and the distance at which it happens. Clusters
/// are numbered: the items first, in the order of their labels, then every join in turn. The
/// left cluster is the one with the earlier item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Join {
    pub left: usize,
    pub right: usize,
    pub distance: f64,
}

fn distances(spec: &ChartSpec) -> &DistancesSpec {
    spec.tree
        .as_ref()
        .and_then(|tree| tree.distances.as_ref())
        .expect("a dendrogram carries distances")
}

fn linkage(spec: &ChartSpec) -> Linkage {
    spec.tree
        .as_ref()
        .map_or(Linkage::Average, |tree| tree.linkage)
}

/// The joins of agglomerative clustering, in order: always the two closest clusters, at a tie
/// the pair whose numbers come first. The distance from a join to every other cluster follows
/// from its two parts by the linkage (Lance–Williams).
pub(crate) fn joins(distances: &DistancesSpec, linkage: Linkage) -> Vec<Join> {
    let items = distances.labels.len();
    let total = 2 * items - 1;
    let mut distance = vec![vec![0.0; total]; total];
    for (row, values) in distances.matrix.iter().enumerate() {
        distance[row][..items].copy_from_slice(values);
    }
    let mut size = vec![1.0_f64; total];
    let mut first = (0..total).collect::<Vec<usize>>();
    let mut active: Vec<usize> = (0..items).collect();
    let mut joins = Vec::with_capacity(items - 1);
    while active.len() > 1 {
        let mut best: Option<(f64, usize, usize)> = None;
        for (at, &a) in active.iter().enumerate() {
            for &b in &active[at + 1..] {
                let d = distance[a][b];
                if best.is_none_or(|(least, _, _)| d < least - 1e-12) {
                    best = Some((d, a, b));
                }
            }
        }
        let (d, a, b) = best.expect("two clusters are left");
        let (left, right) = if first[a] <= first[b] { (a, b) } else { (b, a) };
        let joined = items + joins.len();
        size[joined] = size[a] + size[b];
        first[joined] = first[a].min(first[b]);
        for &other in active.iter().filter(|&&other| other != a && other != b) {
            let (da, db) = (distance[a][other], distance[b][other]);
            let value = match linkage {
                Linkage::Single => da.min(db),
                Linkage::Complete => da.max(db),
                Linkage::Average => (size[a] * da + size[b] * db) / (size[a] + size[b]),
            };
            distance[joined][other] = value;
            distance[other][joined] = value;
        }
        active.retain(|&cluster| cluster != a && cluster != b);
        active.push(joined);
        joins.push(Join {
            left,
            right,
            distance: d,
        });
    }
    joins
}

/// The items of cluster `cluster`, in the order the tree gives them: left before right.
fn members(joins: &[Join], items: usize, cluster: usize) -> Vec<usize> {
    if cluster < items {
        return vec![cluster];
    }
    let join = joins[cluster - items];
    let mut all = members(joins, items, join.left);
    all.extend(members(joins, items, join.right));
    all
}

/// A cluster by its items: one label, or up to three, or three and how many more.
fn cluster_name(distances: &DistancesSpec, joins: &[Join], cluster: usize) -> String {
    let items = members(joins, distances.labels.len(), cluster);
    if items.len() == 1 {
        return distances.labels[items[0]].clone();
    }
    let named: Vec<&str> = items
        .iter()
        .take(NAMED)
        .map(|&item| distances.labels[item].as_str())
        .collect();
    if items.len() > NAMED {
        format!("{{{}, … +{}}}", named.join(", "), items.len() - NAMED)
    } else {
        format!("{{{}}}", named.join(", "))
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let distances = distances(spec);
    let items = distances.labels.len();
    let joins = joins(distances, linkage(spec));
    let root = items + joins.len() - 1;
    let order = members(&joins, items, root);
    let compact = spec.width < NARROW;
    let margin = if compact { COMPACT_MARGIN } else { MARGIN };
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let max_label = if compact {
        COMPACT_MAX_LABEL
    } else {
        MAX_LABEL
    };
    let labels: Vec<String> = distances
        .labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            fit_text(
                label,
                max_label,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/tree/distances/labels/{index}"),
            )
        })
        .collect();
    let label_width = labels
        .iter()
        .map(|label| metrics.width(label, LABEL_SIZE))
        .fold(0.0, f64::max);
    let page_height = height.max(top + ROW * count(items) + AXIS + BOTTOM);
    let page_width = width.max(2.0 * margin + label_width + 10.0 + 160.0);
    let row = ((page_height - top - AXIS - BOTTOM) / count(items)).min(MAX_ROW);
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        page_width - 2.0 * margin,
        metrics,
        warnings,
    );
    warn_growth_advising(
        spec,
        (page_width, page_height),
        "shorten labels or use fewer items",
        warnings,
    );
    let tallest = joins.last().map_or(0.0, |join| join.distance);
    let scale = NumericScale::from_values([0.0, tallest].into_iter(), true, (None, None));
    let (start, end) = (margin + label_width + 10.0, page_width - margin - 12.0);
    let x = |value: f64| scale.map(value, start, end);
    let rows_top = top;
    let rows_bottom = top + row * count(items);
    let axis_y = rows_bottom + 6.0;
    let style = spec.number_style();
    for tick in scale.ticks() {
        let at = x(tick);
        elements.push(Element::Line(Line {
            x1: at,
            y1: rows_top,
            x2: at,
            y2: axis_y,
            class: if scale.is_zero(tick) {
                "chartlet-zero"
            } else {
                "chartlet-grid"
            },
        }));
        elements.push(Element::Text(Text {
            x: at,
            y: axis_y + 16.0,
            class: "chartlet-tick",
            anchor: TextAnchor::Middle,
            content: scale.tick_label(tick, style),
        }));
    }
    elements.push(Element::Text(Text {
        x: f64::midpoint(start, end),
        y: axis_y + 36.0,
        class: "chartlet-axis-title",
        anchor: TextAnchor::Middle,
        content: text::distance_axis(spec.locale, linkage(spec)),
    }));
    // Where every cluster stands: an item at distance 0 on its row, a join at its distance
    // halfway between its parts.
    let mut place = vec![(0.0, 0.0); items + joins.len()];
    for (position, &item) in order.iter().enumerate() {
        place[item] = (x(0.0), rows_top + row * (count(position) + 0.5));
    }
    for (index, join) in joins.iter().enumerate() {
        let (left, right) = (place[join.left], place[join.right]);
        let at = x(join.distance);
        place[items + index] = (at, f64::midpoint(left.1, right.1));
        elements.push(Element::Polyline(polyline(
            vec![
                (left.0, left.1),
                (at, left.1),
                (at, right.1),
                (right.0, right.1),
            ],
            "chartlet-flow-edge",
            Some(format!(
                "{} + {}: {}",
                cluster_name(distances, &joins, join.left),
                cluster_name(distances, &joins, join.right),
                format_value(join.distance, style)
            )),
        )));
    }
    for (position, &item) in order.iter().enumerate() {
        elements.push(Element::Text(Text {
            x: start - 8.0,
            y: rows_top + row * (count(position) + 0.5) + 4.0,
            class: "chartlet-label",
            anchor: TextAnchor::End,
            content: labels[item].clone(),
        }));
    }
    Scene {
        width: pixels(page_width),
        height: pixels(page_height),
        elements,
    }
}

/// The dendrogram in sentences: how many items and by which linkage, the closest pair, the two
/// groups of the last join, and the items in the order of the tree.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let distances = distances(spec);
    let locale = spec.locale;
    let items = distances.labels.len();
    let joins = joins(distances, linkage(spec));
    let style = spec.number_style();
    let group = |cluster: usize| -> Vec<String> {
        members(&joins, items, cluster)
            .into_iter()
            .map(|item| distances.labels[item].clone())
            .collect()
    };
    let first = joins[0];
    let last = joins[joins.len() - 1];
    let root = items + joins.len() - 1;
    [
        text::dendrogram_opening(locale, items, linkage(spec)),
        text::closest_pair(
            locale,
            (&group(first.left), &group(first.right)),
            &format_value(first.distance, style),
        ),
        text::last_join(
            locale,
            (&group(last.left), &group(last.right)),
            &format_value(last.distance, style),
        ),
        text::dendrogram_order(locale, &group(root)),
    ]
    .join(" ")
}

/// One row per join, in order: what it joins, at which distance, and how many items the new
/// cluster holds.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let distances = distances(spec);
    let words = spec.locale.words();
    let items = distances.labels.len();
    let joins = joins(distances, linkage(spec));
    let style = spec.number_style();
    let rows = joins
        .iter()
        .enumerate()
        .map(|(index, join)| {
            vec![
                (index + 1).to_string(),
                format!(
                    "{} + {}",
                    cluster_name(distances, &joins, join.left),
                    cluster_name(distances, &joins, join.right)
                ),
                format_value(join.distance, style),
                members(&joins, items, items + index).len().to_string(),
            ]
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: vec![
            words.step.to_owned(),
            words.joins.to_owned(),
            words.distance.to_owned(),
            words.items.to_owned(),
        ],
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::{Join, joins};
    use crate::{
        ChartSpec, RenderFormat, RenderOptions, render_json, spec::Linkage, text_alternative,
    };

    const SPEC: &str = r#"{
        "schemaVersion": 1,
        "type": "tree",
        "title": "Kinship",
        "tree": {
            "distances": {
                "labels": ["A", "B", "C", "D"],
                "matrix": [
                    [0, 2, 6, 10],
                    [2, 0, 5, 9],
                    [6, 5, 0, 4],
                    [10, 9, 4, 0]
                ]
            }
        }
    }"#;

    fn distances(json: &str) -> crate::spec::DistancesSpec {
        ChartSpec::from_json(json)
            .expect("parses")
            .tree
            .and_then(|tree| tree.distances)
            .expect("distances")
    }

    #[test]
    fn the_closest_clusters_join_first_by_each_linkage() {
        let distances = distances(SPEC);
        // A+B at 2, C+D at 4; then {A,B} to {C,D}: average of 6, 10, 5, 9 = 7.5.
        assert_eq!(
            joins(&distances, Linkage::Average),
            [
                Join {
                    left: 0,
                    right: 1,
                    distance: 2.0
                },
                Join {
                    left: 2,
                    right: 3,
                    distance: 4.0
                },
                Join {
                    left: 4,
                    right: 5,
                    distance: 7.5
                },
            ]
        );
        assert!((joins(&distances, Linkage::Single)[2].distance - 5.0).abs() < 1e-9);
        assert!((joins(&distances, Linkage::Complete)[2].distance - 10.0).abs() < 1e-9);
    }

    #[test]
    fn the_description_and_the_table_follow_the_joins() {
        let alternative =
            text_alternative(&ChartSpec::from_json(SPEC).expect("parses")).expect("valid");
        assert_eq!(
            alternative.description,
            "Dendrogram of 4 items, joined by average linkage (the mean distance between two clusters). Closest: A and B (2). The last join, at 7.5, brings together two groups: A and B; C and D. Order of the items: A, B, C and D."
        );
        assert_eq!(
            alternative.table.columns,
            ["Step", "Joins", "Distance", "Items"]
        );
        assert_eq!(
            alternative.table.rows[2],
            ["3", "{A, B} + {C, D}", "7.5", "4"]
        );
        let svg = render_json(SPEC, RenderFormat::Svg, &RenderOptions::default()).expect("renders");
        assert!(svg.warnings.is_empty(), "{:?}", svg.warnings);
    }

    #[test]
    fn invalid_distances_name_the_field() {
        let error = |json: &str| {
            let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
                .expect_err("invalid");
            (error.code, error.path)
        };
        for (from, to, code, path) in [
            (
                "[0, 2, 6, 10]",
                "[0, 2, 6, 11]",
                "invalid_matrix",
                "/tree/distances/matrix/0/3",
            ),
            (
                "[0, 2, 6, 10]",
                "[1, 2, 6, 10]",
                "invalid_matrix",
                "/tree/distances/matrix/0/0",
            ),
            (
                "[0, 2, 6, 10]",
                "[0, 2, 6]",
                "invalid_matrix",
                "/tree/distances/matrix/0",
            ),
            // A short row below the one being checked for symmetry.
            (
                "[10, 9, 4, 0]",
                "[]",
                "invalid_matrix",
                "/tree/distances/matrix/3",
            ),
            (
                r#""C", "D""#,
                r#""C", "A""#,
                "duplicate_id",
                "/tree/distances/labels/3",
            ),
            (
                r#""distances""#,
                r#""layout": "radial", "distances""#,
                "option_not_supported",
                "/tree/layout",
            ),
        ] {
            assert!(SPEC.contains(from), "{from}");
            assert_eq!(error(&SPEC.replace(from, to)), (code, path.to_owned()));
        }
        let linked = include_str!("../examples/family-tree.json")
            .replace("\"nodes\"", "\"linkage\": \"single\", \"nodes\"");
        assert_eq!(
            error(&linked),
            ("option_not_supported", "/tree/linkage".to_owned())
        );
    }
}
