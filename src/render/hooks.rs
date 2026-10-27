//! The `data-*` hooks of the interactive module: the groups around plots and layers, the hooks
//! of the data table, and the JSON data block of the SVG profile.

use std::fmt::Write;

use super::{
    escape, number,
    table::{data_table, table_extra},
};
use crate::{
    scene::Hook,
    spec::{ChartSpec, ChartType, Mark},
};

/// Which `data-*` hooks of the interactive module a chart carries: none, the hooks alone (the
/// HTML profile, whose data table carries the values), or the hooks with the values in a JSON
/// data block (the SVG profile).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hooks {
    Off,
    Attributes,
    WithData,
}

/// Writes a hook: the plot of a pane as an empty group, or the group around one layer.
pub(super) fn emit_hook(hook: &Hook, spec: &ChartSpec, output: &mut String) {
    let pairs = |pairs: &[(f64, f64)]| {
        let (values, pixels): (Vec<String>, Vec<String>) = pairs
            .iter()
            .map(|(value, pixel)| (number(*value), number(*pixel)))
            .unzip();
        (values.join(" "), pixels.join(" "))
    };
    match hook {
        Hook::Plot { pane, x, y, log } => {
            let (x_domain, x_range) = pairs(x);
            let (y_domain, y_range) = pairs(y);
            let scale = if *log { " data-y-scale=\"log\"" } else { "" };
            write!(
                output,
                "<g data-chartlet-plot=\"\" data-pane=\"{pane}\" data-x-domain=\"{x_domain}\" data-x-range=\"{x_range}\" data-y-domain=\"{y_domain}\" data-y-range=\"{y_range}\"{scale}/>"
            )
            .expect("write");
        }
        Hook::Layer(global) => {
            let entry = spec
                .indexed_layers()
                .nth(*global)
                .expect("a hook names an existing layer");
            if entry.layer.is_data() {
                let series = spec
                    .data_layers()
                    .position(|data| data.global == *global)
                    .expect("a data layer is among the data layers");
                write!(
                    output,
                    "<g data-series=\"{series}\" data-pane=\"{}\"",
                    entry.pane
                )
                .expect("write");
                if let Some(name) = &entry.layer.name {
                    write!(output, " data-name=\"{}\"", escape(name)).expect("write");
                }
                output.push('>');
            } else {
                write!(
                    output,
                    "<g data-annotation=\"{global}\" data-pane=\"{}\">",
                    entry.pane
                )
                .expect("write");
            }
        }
        Hook::Object(attribute, id) => {
            write!(output, "<g {attribute}=\"{}\">", escape(id)).expect("write");
        }
        Hook::End => output.push_str("</g>"),
    }
}

/// What the interactive module needs to know about the data table beyond its text: per value
/// column the series it belongs to (its index among the data layers, or among the series of a
/// category chart), the pane and which part of the series it holds; per row its position on the
/// axis (Unix seconds on a time axis, the row index otherwise); and every value unformatted.
/// The hooks of the data table: per value column its series, pane, part and whether it is a
/// stacked area, drawn on top of the stacked areas before it; per row its position and values.
pub(super) struct TableHooks {
    /// The panes whose stacked areas are centered on a moving baseline, a stream.
    pub(super) streams: Vec<usize>,
    pub(super) columns: Vec<(usize, usize, &'static str, bool)>,
    pub(super) x: Vec<f64>,
    pub(super) values: Vec<Vec<Option<f64>>>,
}

#[allow(clippy::too_many_lines)]
pub(super) fn table_hooks(spec: &ChartSpec) -> TableHooks {
    // A diagram's table holds text only: its columns belong to no series and its rows carry no
    // values.
    if matches!(
        spec.chart_type,
        ChartType::Sequence
            | ChartType::Flow
            | ChartType::State
            | ChartType::Classes
            | ChartType::Network
            | ChartType::Cycle
            | ChartType::Architecture
            | ChartType::Tree
            | ChartType::Timeline
            | ChartType::Scatter
            | ChartType::Survival
            | ChartType::Sankey
            | ChartType::Treemap
            | ChartType::Parliament
            | ChartType::Waffle
            | ChartType::Heatmap
            | ChartType::Venn
            | ChartType::Valuemap
            | ChartType::Journey
            | ChartType::Storymap
            | ChartType::Kanban
            | ChartType::Fishbone
            | ChartType::Waterfall
    ) {
        let table = data_table(spec);
        return TableHooks {
            streams: Vec::new(),
            columns: (1..table.columns.len())
                .map(|index| (index - 1, 0, "text", false))
                .collect(),
            x: (0..table.rows.len())
                .map(|row| f64::from(u32::try_from(row).expect("rows are limited")))
                .collect(),
            values: vec![vec![None; table.columns.len() - 1]; table.rows.len()],
        };
    }
    let dataset = spec.table_dataset();
    let rows = dataset.categories.len();
    // The text columns after the values have no value.
    let extras = table_extra(spec).len();
    let values = (0..rows)
        .map(|row| {
            dataset
                .series
                .iter()
                .map(|series| series.values[row])
                .chain(std::iter::repeat_n(None, extras))
                .collect()
        })
        .collect();
    if !matches!(spec.chart_type, ChartType::Time | ChartType::Multiples) || spec.is_bar_multiples()
    {
        return TableHooks {
            streams: Vec::new(),
            columns: (0..dataset.series.len())
                .map(|index| (index, 0, "value", false))
                .chain((0..extras).map(|extra| (dataset.series.len() + extra, 0, "text", false)))
                .collect(),
            x: (0..rows)
                .map(|row| f64::from(u32::try_from(row).expect("rows are limited")))
                .collect(),
            values,
        };
    }
    let zone = spec.time_zone().unwrap_or_default();
    let mut epochs: Vec<i64> = spec
        .data_layers()
        .flat_map(|entry| entry.layer.resolved_times(zone))
        .collect();
    epochs.sort_unstable();
    epochs.dedup();
    let mut columns = Vec::new();
    for (series, entry) in spec.data_layers().enumerate() {
        let parts: &[&'static str] = if entry.layer.mark == Mark::Ohlc {
            &["open", "high", "low", "close"]
        } else if entry.layer.has_band() {
            &["value", "lower", "upper"]
        } else {
            &["value"]
        };
        let stacked = spec.stack_base(entry, zone).is_some();
        columns.extend(
            parts
                .iter()
                .map(|part| (series, entry.pane, *part, stacked)),
        );
    }
    // The specification allows 1700 to 2200, well within the integers an f64 holds exactly.
    #[allow(clippy::cast_precision_loss)]
    let x = epochs.iter().map(|epoch| *epoch as f64).collect();
    let streams = spec
        .panes
        .iter()
        .enumerate()
        .filter(|(_, pane)| pane.stack == Some(crate::spec::Stack::Stream))
        .map(|(index, _)| index)
        .collect();
    TableHooks {
        streams,
        columns,
        x,
        values,
    }
}

/// The data table as a JSON data block for the SVG profile, which has no table to read: the
/// columns with their hooks, and per row its position, its text and its unformatted values. A
/// script element of type `application/json` is never executed, so a strict CSP allows it.
pub(super) fn emit_data_block(spec: &ChartSpec, output: &mut String) {
    let table = data_table(spec);
    let hooks = table_hooks(spec);
    let columns: Vec<serde_json::Value> = table
        .columns
        .iter()
        .skip(1)
        .zip(&hooks.columns)
        .map(|(name, (series, pane, part, stacked))| {
            let mut column =
                serde_json::json!({ "name": name, "series": series, "pane": pane, "part": part });
            if *stacked {
                column["stacked"] = serde_json::Value::Bool(true);
                if hooks.streams.contains(pane) {
                    column["stream"] = serde_json::Value::Bool(true);
                }
            }
            column
        })
        .collect();
    let rows: Vec<serde_json::Value> = table
        .rows
        .iter()
        .zip(hooks.x.iter().zip(&hooks.values))
        .map(|(row, (x, values))| {
            serde_json::json!({ "x": x, "label": row[0], "text": row[1..], "value": values })
        })
        .collect();
    let block = serde_json::json!({ "columns": columns, "rows": rows });
    write!(
        output,
        "<script type=\"application/json\" data-chartlet-data=\"\">{}</script>",
        escape_text(&block.to_string())
    )
    .expect("write");
}

/// Escapes text content for XML and HTML alike; quotes may stay as they are.
fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
