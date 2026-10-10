//! Treemaps: rectangles whose areas follow the values of the items, packed into one rectangle by
//! the squarified method: the largest items first, in rows along the shorter side, so that the
//! rectangles come out close to squares. Names and values stand inside the rectangles that have
//! room for them; the legend, the description and the data table name every item.

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{LABEL_SIZE, fit_text, format_value, push_title, title_extra, warn_if_labels_omitted},
    metrics::TextMetrics,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, NumberStyle, TmNode, TmTree, TreemapSpec, ValueFormat},
    text,
};

/// The gap between two rectangles, and the room a rectangle needs for its name and for its value.
const GAP: f64 = 2.0;
const NAME_WIDTH: f64 = 44.0;
const NAME_HEIGHT: f64 = 24.0;
const VALUE_HEIGHT: f64 = 44.0;

fn treemap(spec: &ChartSpec) -> &TreemapSpec {
    spec.treemap
        .as_ref()
        .expect("validated treemaps carry a treemap block")
}

const CELLS: [&str; 5] = [
    "chartlet-tm-cell chartlet-tm-0",
    "chartlet-tm-cell chartlet-tm-1",
    "chartlet-tm-cell chartlet-tm-2",
    "chartlet-tm-cell chartlet-tm-3",
    "chartlet-tm-cell chartlet-tm-4",
];
const FRAMES: [&str; 5] = [
    "chartlet-tm-frame chartlet-tm-0",
    "chartlet-tm-frame chartlet-tm-1",
    "chartlet-tm-frame chartlet-tm-2",
    "chartlet-tm-frame chartlet-tm-3",
    "chartlet-tm-frame chartlet-tm-4",
];
/// The room inside a frame on every side, and the height of its name above its parts.
const PAD: f64 = 3.0;
const HEADER: f64 = 18.0;

fn share(spec: &ChartSpec, value: f64, total: f64) -> String {
    format_value(
        value / total,
        NumberStyle {
            format: ValueFormat::Percent,
            decimals: Some(1),
            ..spec.number_style()
        },
    )
}

/// How far from a square the worst rectangle of a row comes out, for a row of `sum` area laid
/// along a side of the given length: always at least one.
fn worst_ratio(row: &[f64], sum: f64, side: f64) -> f64 {
    let (largest, smallest) = row
        .iter()
        .fold((f64::MIN, f64::MAX), |(most, least), area| {
            (most.max(*area), least.min(*area))
        });
    f64::max(
        side * side * largest / (sum * sum),
        sum * sum / (side * side * smallest),
    )
}

/// The rectangles, as `(x, y, width, height)`, for areas in decreasing order that add up to the
/// area of the box: rows along the shorter side while they make the worst rectangle no worse.
fn squarify(
    areas: &[f64],
    (mut x, mut y, mut width, mut height): (f64, f64, f64, f64),
) -> Vec<(f64, f64, f64, f64)> {
    let mut out = vec![(0.0, 0.0, 0.0, 0.0); areas.len()];
    let mut first = 0;
    while first < areas.len() {
        let side = width.min(height);
        let mut end = first + 1;
        let mut sum = areas[first];
        let mut worst = worst_ratio(&areas[first..end], sum, side);
        while end < areas.len() {
            let more = sum + areas[end];
            let next = worst_ratio(&areas[first..=end], more, side);
            if next > worst {
                break;
            }
            sum = more;
            worst = next;
            end += 1;
        }
        let thickness = sum / side;
        if width >= height {
            let mut at = y;
            for index in first..end {
                let extent = areas[index] / thickness;
                out[index] = (x, at, thickness, extent);
                at += extent;
            }
            x += thickness;
            width -= thickness;
        } else {
            let mut at = x;
            for index in first..end {
                let extent = areas[index] / thickness;
                out[index] = (at, y, extent, thickness);
                at += extent;
            }
            y += thickness;
            height -= thickness;
        }
        first = end;
    }
    out
}

/// The items from the largest to the smallest, equal ones in their order.
/// The items of `nodes` from the largest to the smallest, equal ones in their order.
fn by_size(tree: &TmTree, nodes: &[usize]) -> Vec<usize> {
    let mut order = nodes.to_vec();
    crate::sort::by(&mut order, |a, b| {
        tree.nodes[*b].value.total_cmp(&tree.nodes[*a].value)
    });
    order
}

type Bounds = (f64, f64, f64, f64);

/// Where an item stands and whether it has a header for its name above its parts.
struct Placed {
    node: usize,
    rect: Bounds,
    header: f64,
}

/// Packs `nodes` into `bounds`, and the parts of each inside its rectangle, in pre-order.
fn place(tree: &TmTree, nodes: &[usize], bounds: Bounds, out: &mut Vec<Placed>) {
    let order = by_size(tree, nodes);
    let sum: f64 = order.iter().map(|node| tree.nodes[*node].value).sum();
    let scale = bounds.2 * bounds.3 / sum;
    let areas: Vec<f64> = order
        .iter()
        .map(|node| tree.nodes[*node].value * scale)
        .collect();
    for (node, rect) in order.into_iter().zip(squarify(&areas, bounds)) {
        let header = if !tree.nodes[node].children.is_empty() && rect.2 >= 60.0 && rect.3 >= 44.0 {
            HEADER
        } else {
            0.0
        };
        out.push(Placed { node, rect, header });
        if !tree.nodes[node].children.is_empty() {
            let inner = (
                rect.0 + PAD + GAP / 2.0,
                rect.1 + PAD + GAP / 2.0 + header,
                (rect.2 - 2.0 * PAD - GAP).max(1.0),
                (rect.3 - 2.0 * PAD - GAP - header).max(1.0),
            );
            place(tree, &tree.nodes[node].children, inner, out);
        }
    }
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let treemap = treemap(spec);
    let tree = treemap.tree();
    let compact = spec.width < crate::layout::NARROW;
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let margin = if compact { 12.0 } else { 24.0 };
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        width - 2.0 * margin,
        metrics,
        warnings,
    );
    let mut top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    top = push_legend(&mut elements, warnings, spec, (margin, top - 10.0), metrics).max(top);
    let bounds = (
        margin,
        top,
        width - 2.0 * margin,
        (height - top - 16.0).max(60.0),
    );
    let total = treemap.total();
    let mut placed = Vec::new();
    place(&tree, &tree.roots, bounds, &mut placed);
    let mut omitted = false;
    for Placed {
        node,
        rect: (x, y, w, h),
        header,
    } in placed
    {
        let item = &tree.nodes[node];
        let (rx, ry, rw, rh) = (x + GAP / 2.0, y + GAP / 2.0, w - GAP, h - GAP);
        let parent = !item.children.is_empty();
        elements.push(Element::Rect(Rect {
            x: rx,
            y: ry,
            width: rw,
            height: rh,
            class: if parent {
                FRAMES[item.color]
            } else {
                CELLS[item.color]
            },
            series_index: None,
            style_index: None,
            tooltip: Some(format!(
                "{}: {} ({})",
                item.path,
                format_value(item.value, spec.number_style()),
                share(spec, item.value, total)
            )),
        }));
        if parent {
            if header > 0.0 {
                elements.push(Element::Text(Text {
                    x: rx + 6.0,
                    y: ry + 15.0,
                    class: "chartlet-tm-head",
                    anchor: TextAnchor::Start,
                    content: fit_text(
                        &item.label,
                        rw - 12.0,
                        LABEL_SIZE,
                        metrics,
                        warnings,
                        "/treemap/items",
                    ),
                }));
            }
            continue;
        }
        omitted |= !push_names(
            &mut elements,
            spec,
            item,
            (rx, ry, rw, rh),
            metrics,
            warnings,
        );
    }
    warn_if_labels_omitted(omitted, warnings);
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

/// What the colors stand for, in order of their palette color: the groups of a flat treemap, or
/// up to four items at the top of a nested one.
fn legend_entries(spec: &ChartSpec) -> Vec<String> {
    let treemap = treemap(spec);
    if treemap.nested() {
        if treemap.items.len() <= 4 {
            return treemap
                .items
                .iter()
                .map(|item| item.label.clone())
                .collect();
        }
        return Vec::new();
    }
    treemap.groups().into_iter().map(str::to_owned).collect()
}

/// The legend in a row under the title; returns where it ends.
fn push_legend(
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    spec: &ChartSpec,
    (left, top): (f64, f64),
    metrics: &impl TextMetrics,
) -> f64 {
    let groups = legend_entries(spec);
    if groups.is_empty() {
        return top;
    }
    let right = f64::from(spec.width) - left;
    let (mut x, mut y) = (left, top);
    for (index, group) in groups.iter().enumerate() {
        let reach = 16.0 + metrics.width(group, LABEL_SIZE);
        if x > left && x + reach > right {
            x = left;
            y += 20.0;
        }
        elements.push(Element::Rect(Rect {
            x,
            y,
            width: 10.0,
            height: 10.0,
            class: CELLS[index + 1],
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: x + 16.0,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                group,
                (right - x - 16.0).max(0.0),
                LABEL_SIZE,
                metrics,
                warnings,
                "/treemap/items",
            ),
        }));
        x += reach + 16.0;
    }
    y + 26.0
}

/// The treemap in sentences: its items and total, its biggest items and its groups.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let treemap = treemap(spec);
    let tree = treemap.tree();
    let total = treemap.total();
    let leaves: Vec<usize> = (0..tree.nodes.len())
        .filter(|node| tree.nodes[*node].children.is_empty())
        .collect();
    let largest: Vec<String> = by_size(&tree, &leaves)
        .iter()
        .take(3)
        .map(|node| {
            let item = &tree.nodes[*node];
            format!("{} ({})", item.path, share(spec, item.value, total))
        })
        .collect();
    let mut groups: Vec<String> = Vec::new();
    let mut named: Vec<&str> = Vec::new();
    for node in tree.roots.iter().map(|root| &tree.nodes[*root]) {
        if let Some(group) = node.group.as_deref()
            && !named.contains(&group)
        {
            named.push(group);
            let sum: f64 = tree
                .roots
                .iter()
                .map(|root| &tree.nodes[*root])
                .filter(|other| other.group.as_deref() == Some(group))
                .map(|other| other.value)
                .sum();
            groups.push(format!("{group} {}", share(spec, sum, total)));
        }
    }
    text::treemap_summary(
        spec.locale,
        (leaves.len(), &format_value(total, spec.number_style())),
        &largest.join(", "),
        &groups.join(", "),
    )
}

/// One row per item, the parts of an item right after it, each level from the largest to the
/// smallest: its path, value, share and group (the top-level item of a nested treemap).
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let treemap = treemap(spec);
    let tree = treemap.tree();
    let words = spec.locale.words();
    let total = treemap.total();
    let grouped = tree.nodes.iter().any(|node| node.group.is_some());
    let mut columns = vec![
        words.item.to_owned(),
        words.value.to_owned(),
        words.share.to_owned(),
    ];
    if grouped {
        columns.push(words.group.to_owned());
    }
    let mut order = Vec::new();
    list(&tree, &tree.roots, &mut order);
    let rows = order
        .into_iter()
        .map(|index| {
            let item = &tree.nodes[index];
            let mut row = vec![
                item.path.clone(),
                format_value(item.value, spec.number_style()),
                share(spec, item.value, total),
            ];
            if grouped {
                row.push(item.group.clone().unwrap_or_default());
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

/// `nodes` and what is below them in pre-order, each level by size.
fn list(tree: &TmTree, nodes: &[usize], out: &mut Vec<usize>) {
    for node in by_size(tree, nodes) {
        out.push(node);
        list(tree, &tree.nodes[node].children, out);
    }
}

/// The name and the value of a leaf inside its rectangle; returns whether there was room.
fn push_names(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    item: &TmNode,
    (rx, ry, rw, rh): Bounds,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> bool {
    if rw < NAME_WIDTH || rh < NAME_HEIGHT {
        return false;
    }
    elements.push(Element::Text(Text {
        x: rx + 6.0,
        y: ry + 16.0,
        class: "chartlet-tm-label",
        anchor: TextAnchor::Start,
        content: fit_text(
            &item.label,
            rw - 12.0,
            LABEL_SIZE,
            metrics,
            warnings,
            "/treemap/items",
        ),
    }));
    if rh >= VALUE_HEIGHT {
        elements.push(Element::Text(Text {
            x: rx + 6.0,
            y: ry + 31.0,
            class: "chartlet-tm-value",
            anchor: TextAnchor::Start,
            content: fit_text(
                &format_value(item.value, spec.number_style()),
                rw - 12.0,
                LABEL_SIZE,
                metrics,
                warnings,
                "/treemap/items",
            ),
        }));
    }
    true
}
