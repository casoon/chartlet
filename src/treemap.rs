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
    spec::{ChartSpec, NumberStyle, TreemapSpec, ValueFormat},
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

fn class(spec: &ChartSpec, index: usize) -> &'static str {
    const GROUPED: [&str; 4] = [
        "chartlet-tm-cell chartlet-tm-1",
        "chartlet-tm-cell chartlet-tm-2",
        "chartlet-tm-cell chartlet-tm-3",
        "chartlet-tm-cell chartlet-tm-4",
    ];
    let treemap = treemap(spec);
    treemap.items[index]
        .group
        .as_deref()
        .and_then(|group| treemap.groups().iter().position(|known| *known == group))
        .map_or("chartlet-tm-cell chartlet-tm-0", |position| {
            GROUPED[position]
        })
}

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
fn by_size(treemap: &TreemapSpec) -> Vec<usize> {
    let mut order: Vec<usize> = (0..treemap.items.len()).collect();
    crate::sort::by(&mut order, |a, b| {
        treemap.items[*b].value.total_cmp(&treemap.items[*a].value)
    });
    order
}

/// The rectangle of every item, by its index in the specification.
fn rectangles(treemap: &TreemapSpec, bounds: (f64, f64, f64, f64)) -> Vec<(f64, f64, f64, f64)> {
    let order = by_size(treemap);
    let scale = bounds.2 * bounds.3 / treemap.total();
    let areas: Vec<f64> = order
        .iter()
        .map(|index| treemap.items[*index].value * scale)
        .collect();
    let placed = squarify(&areas, bounds);
    let mut out = vec![(0.0, 0.0, 0.0, 0.0); order.len()];
    for (rectangle, index) in placed.into_iter().zip(order) {
        out[index] = rectangle;
    }
    out
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let treemap = treemap(spec);
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
    let mut omitted = false;
    for (index, (x, y, w, h)) in rectangles(treemap, bounds).into_iter().enumerate() {
        let item = &treemap.items[index];
        let (rx, ry, rw, rh) = (x + GAP / 2.0, y + GAP / 2.0, w - GAP, h - GAP);
        elements.push(Element::Rect(Rect {
            x: rx,
            y: ry,
            width: rw,
            height: rh,
            class: class(spec, index),
            series_index: None,
            style_index: None,
            tooltip: Some(format!(
                "{}: {} ({})",
                item.label,
                format_value(item.value, spec.number_style()),
                share(spec, item.value, total)
            )),
        }));
        if rw >= NAME_WIDTH && rh >= NAME_HEIGHT {
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
                    &format!("/treemap/items/{index}/label"),
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
                        &format!("/treemap/items/{index}/value"),
                    ),
                }));
            }
        } else {
            omitted = true;
        }
    }
    warn_if_labels_omitted(omitted, warnings);
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

/// The legend of the groups in a row under the title; returns where it ends.
fn push_legend(
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    spec: &ChartSpec,
    (left, top): (f64, f64),
    metrics: &impl TextMetrics,
) -> f64 {
    let groups = treemap(spec).groups();
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
            class: [
                "chartlet-tm-cell chartlet-tm-1",
                "chartlet-tm-cell chartlet-tm-2",
                "chartlet-tm-cell chartlet-tm-3",
                "chartlet-tm-cell chartlet-tm-4",
            ][index],
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
    let total = treemap.total();
    let order = by_size(treemap);
    let largest: Vec<String> = order
        .iter()
        .take(3)
        .map(|index| {
            let item = &treemap.items[*index];
            format!("{} ({})", item.label, share(spec, item.value, total))
        })
        .collect();
    let groups: Vec<String> = treemap
        .groups()
        .into_iter()
        .map(|group| {
            let sum: f64 = treemap
                .items
                .iter()
                .filter(|item| item.group.as_deref() == Some(group))
                .map(|item| item.value)
                .sum();
            format!("{group} {}", share(spec, sum, total))
        })
        .collect();
    text::treemap_summary(
        spec.locale,
        (
            treemap.items.len(),
            &format_value(total, spec.number_style()),
        ),
        &largest.join(", "),
        &groups.join(", "),
    )
}

/// One row per item from the largest to the smallest: its value, share and group.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let treemap = treemap(spec);
    let words = spec.locale.words();
    let total = treemap.total();
    let grouped = !treemap.groups().is_empty();
    let mut columns = vec![
        words.item.to_owned(),
        words.value.to_owned(),
        words.share.to_owned(),
    ];
    if grouped {
        columns.push(words.group.to_owned());
    }
    let rows = by_size(treemap)
        .into_iter()
        .map(|index| {
            let item = &treemap.items[index];
            let mut row = vec![
                item.label.clone(),
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
