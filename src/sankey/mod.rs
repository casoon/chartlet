//! Sankey diagrams: nodes in columns by the longest path to them, joined by bands as thick as the
//! flow between them. Within a column the nodes are ordered by the average place of what they are
//! joined to, so that the bands cross as little as they can. A node takes the palette color of the
//! first node it descends from, and a band the color of the node it leaves. The description and
//! the data table list every link with its share of what enters the diagram.

use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{LABEL_SIZE, fit_text, format_value, push_title, title_extra, warn_if_labels_omitted},
    metrics::TextMetrics,
    sankey::geometry::Plan,
    scene::{Element, Polyline, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, NumberStyle, SankeySpec, ValueFormat},
    text,
};

mod geometry;

fn sankey(spec: &ChartSpec) -> &SankeySpec {
    spec.sankey
        .as_ref()
        .expect("validated Sankey diagrams carry a sankey block")
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

const NODE_CLASSES: [&str; 4] = [
    "chartlet-sk-node chartlet-sk-1",
    "chartlet-sk-node chartlet-sk-2",
    "chartlet-sk-node chartlet-sk-3",
    "chartlet-sk-node chartlet-sk-4",
];
const LINK_CLASSES: [&str; 4] = [
    "chartlet-sk-link chartlet-sk-1",
    "chartlet-sk-link chartlet-sk-2",
    "chartlet-sk-link chartlet-sk-3",
    "chartlet-sk-link chartlet-sk-4",
];

/// Room a label needs beside the first and last column, at most this share of the width.
const LABEL_SHARE: f64 = 0.28;

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let graph = sankey(spec).graph();
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
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let texts: Vec<String> = (0..graph.labels.len())
        .map(|node| format_value(graph.size(node), spec.number_style()))
        .collect();
    let reach = |last: bool| {
        let widest = (0..graph.labels.len())
            .filter(|node| graph.is_edge(*node, last))
            .map(|node| {
                metrics.width(
                    &format!("{} {}", graph.labels[node], texts[node]),
                    LABEL_SIZE,
                )
            })
            .fold(0.0, f64::max);
        widest.min(width * LABEL_SHARE) + 8.0
    };
    let (left, right) = (margin + reach(false), margin + reach(true));
    let plan = Plan::new(
        &graph,
        (
            left,
            top,
            width - left - right,
            (height - top - 16.0).max(60.0),
        ),
        if compact { 10.0 } else { 14.0 },
    );
    for (index, link) in graph.links.iter().enumerate() {
        let (from, to, value) = *link;
        elements.push(Element::Polyline(Polyline {
            points: plan.band(index),
            class: LINK_CLASSES[plan.color[from] - 1],
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: Some(format!(
                "{} → {}: {}",
                graph.labels[from],
                graph.labels[to],
                format_value(value, spec.number_style())
            )),
        }));
    }
    let mut omitted = false;
    for node in 0..graph.labels.len() {
        let (x, y, h) = plan.node(node);
        elements.push(Element::Rect(Rect {
            x,
            y,
            width: plan.node_width,
            height: h,
            class: NODE_CLASSES[plan.color[node] - 1],
            series_index: None,
            style_index: None,
            tooltip: Some(format!("{}: {}", graph.labels[node], texts[node])),
        }));
        omitted |= push_label(
            &mut elements,
            warnings,
            metrics,
            (&graph.labels[node], &texts[node]),
            (&plan, node, (left, right, width)),
        );
    }
    warn_if_labels_omitted(omitted, warnings);
    Scene {
        width: spec.width,
        height: pixels(height),
        elements,
    }
}

/// The name and value of a node beside it: left of the first column, right of the others. Two
/// lines when the node is tall enough, else one; returns whether the room was too small.
fn push_label(
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
    (label, value): (&str, &str),
    (plan, node, (left, right, width)): (&Plan, usize, (f64, f64, f64)),
) -> bool {
    let (x, y, h) = plan.node(node);
    let first = plan.rank[node] == 0;
    let (anchor, at, room) = if first {
        (TextAnchor::End, x - 6.0, left - 6.0 - 6.0)
    } else {
        let at = x + plan.node_width + 6.0;
        let room = if plan.rank[node] == plan.columns - 1 {
            right - 6.0 - 6.0
        } else {
            plan.spacing - plan.node_width - 12.0
        };
        (TextAnchor::Start, at, room.min(width))
    };
    let path = format!("/sankey/links/{node}");
    let centre = y + h / 2.0;
    let mut text = |content: String, y: f64, class: &'static str, warnings: &mut Vec<_>| {
        elements.push(Element::Text(Text {
            x: at,
            y,
            class,
            anchor,
            content: fit_text(
                &content,
                room.max(0.0),
                LABEL_SIZE,
                metrics,
                warnings,
                &path,
            ),
        }));
    };
    if h >= 28.0 {
        text(
            label.to_owned(),
            centre - 1.0,
            "chartlet-sk-label",
            warnings,
        );
        text(
            value.to_owned(),
            centre + 12.0,
            "chartlet-sk-value",
            warnings,
        );
    } else {
        text(
            format!("{label} {value}"),
            centre + 4.0,
            "chartlet-sk-label",
            warnings,
        );
    }
    room < 24.0
}

/// The diagram in sentences: its nodes, links and total, and its biggest links.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let graph = sankey(spec).graph();
    let total = graph.total();
    let mut order: Vec<usize> = (0..graph.links.len()).collect();
    crate::sort::by(&mut order, |a, b| {
        graph.links[*b].2.total_cmp(&graph.links[*a].2)
    });
    let mut largest = String::new();
    for (position, index) in order.iter().take(3).enumerate() {
        let (from, to, value) = graph.links[*index];
        if position > 0 {
            largest.push_str(", ");
        }
        write!(
            largest,
            "{} → {} ({})",
            graph.labels[from],
            graph.labels[to],
            format_value(value, spec.number_style())
        )
        .expect("write");
    }
    text::sankey_summary(
        spec.locale,
        (graph.labels.len(), graph.links.len()),
        &format_value(total, spec.number_style()),
        &largest,
    )
}

/// One row per link from the biggest to the smallest: its ends, value and share of the total.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let graph = sankey(spec).graph();
    let words = spec.locale.words();
    let total = graph.total();
    let mut order: Vec<usize> = (0..graph.links.len()).collect();
    crate::sort::by(&mut order, |a, b| {
        graph.links[*b].2.total_cmp(&graph.links[*a].2)
    });
    let rows = order
        .into_iter()
        .map(|index| {
            let (from, to, value) = graph.links[index];
            vec![
                graph.labels[from].clone(),
                graph.labels[to].clone(),
                format_value(value, spec.number_style()),
                share(spec, value, total),
            ]
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: vec![
            words.from.to_owned(),
            words.to.to_owned(),
            words.value.to_owned(),
            words.share.to_owned(),
        ],
        rows,
    }
}
