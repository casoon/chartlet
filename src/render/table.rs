//! The data table: its text for every chart type, and its HTML with the optional hooks.

use std::fmt::Write;

use super::{escape, hooks::table_hooks, number};
use crate::{
    DataTable, TableMode,
    layout::format_value,
    spec::{ChartSpec, ChartType},
};

/// Writes the data table; with `hooks`, the chart's ID prefix, it also carries the
/// [`TableHooks`](super::hooks::TableHooks) as `data-*` attributes.
pub(super) fn render_data_table(
    output: &mut String,
    spec: &ChartSpec,
    table_mode: TableMode,
    hooks: Option<&str>,
) {
    if table_mode == TableMode::Details {
        write!(
            output,
            "<details class=\"chartlet-data\" data-viz-text><summary>{}</summary>",
            spec.locale.words().show_data
        )
        .expect("write");
    } else {
        output.push_str("<div class=\"chartlet-data\" data-viz-text>");
    }
    let table = data_table(spec);
    let table_hooks = hooks.map(|_| table_hooks(spec));
    let table_id = hooks.map_or_else(String::new, |id_prefix| {
        format!(" data-chartlet-table=\"{id_prefix}\"")
    });
    write!(
        output,
        "<table{table_id}><caption>{}</caption><thead><tr>",
        escape(&table.caption)
    )
    .expect("write");
    for (index, column) in table.columns.iter().enumerate() {
        let column_hooks = match (&table_hooks, index.checked_sub(1)) {
            (Some(table_hooks), Some(index)) => {
                let (series, pane, part, stacked) = table_hooks.columns[index];
                let stacked = if stacked { " data-stacked=\"\"" } else { "" };
                format!(
                    " data-series=\"{series}\" data-pane=\"{pane}\" data-part=\"{part}\"{stacked}"
                )
            }
            _ => String::new(),
        };
        write!(
            output,
            "<th scope=\"col\"{column_hooks}>{}</th>",
            escape(column)
        )
        .expect("write");
    }
    output.push_str("</tr></thead><tbody>");
    for (row_index, row) in table.rows.iter().enumerate() {
        let (head, cells) = row.split_first().expect("a row starts with its category");
        match &table_hooks {
            Some(table_hooks) => write!(
                output,
                "<tr data-x=\"{}\"><th scope=\"row\">{}</th>",
                number(table_hooks.x[row_index]),
                escape(head)
            ),
            None => write!(output, "<tr><th scope=\"row\">{}</th>", escape(head)),
        }
        .expect("write");
        for (column, cell) in cells.iter().enumerate() {
            match table_hooks
                .as_ref()
                .and_then(|table_hooks| table_hooks.values[row_index][column])
            {
                Some(value) => write!(output, "<td data-value=\"{value}\">{}</td>", escape(cell)),
                None => write!(output, "<td>{}</td>", escape(cell)),
            }
            .expect("write");
        }
        output.push_str("</tr>");
    }
    output.push_str("</tbody></table>");
    if table_mode == TableMode::Details {
        output.push_str("</details>");
    } else {
        output.push_str("</div>");
    }
}

/// The data table of a chart as text: its caption, the column heads and one row per category,
/// each starting with the category, with values written as the chart writes them.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    match spec.chart_type {
        ChartType::Sequence => return crate::sequence::data_table(spec),
        ChartType::Flow => return crate::flow::data_table(spec),
        ChartType::State => return crate::state::data_table(spec),
        ChartType::Classes => return crate::classes::data_table(spec),
        ChartType::Network => return crate::network::data_table(spec),
        ChartType::Architecture => return crate::architecture::data_table(spec),
        ChartType::Tree => return crate::tree::data_table(spec),
        ChartType::Timeline => return crate::timeline::data_table(spec),
        ChartType::Scatter => return crate::scatter::data_table(spec),
        ChartType::Survival => return crate::survival::data_table(spec),
        ChartType::Sankey => return crate::sankey::data_table(spec),
        ChartType::Treemap => return crate::treemap::data_table(spec),
        ChartType::Parliament => return crate::parliament::data_table(spec),
        ChartType::Waffle => return crate::waffle::data_table(spec),
        ChartType::Waterfall => return crate::waterfall::data_table(spec),
        _ => {}
    }
    let words = spec.locale.words();
    let dataset = spec.table_dataset();
    // A numeric axis is named by its title; it holds no times.
    let numeric = spec.time_axis.kind == crate::spec::TimeAxisKind::Number;
    let first = match spec.chart_type {
        ChartType::Multiples if spec.is_bar_multiples() => words.category,
        ChartType::Time | ChartType::Multiples if numeric => {
            spec.time_axis.title.as_deref().unwrap_or(words.position)
        }
        ChartType::Time | ChartType::Multiples => words.time,
        ChartType::Topicmap => words.topic,
        ChartType::Atlas => words.region,
        ChartType::Stripes => words.year,
        ChartType::Calendar => words.date,
        ChartType::Bar | ChartType::Line | ChartType::Rangebar | ChartType::Boxplot => {
            words.category
        }
        ChartType::Sequence
        | ChartType::Flow
        | ChartType::State
        | ChartType::Classes
        | ChartType::Network
        | ChartType::Architecture
        | ChartType::Tree
        | ChartType::Timeline
        | ChartType::Waterfall
        | ChartType::Waffle
        | ChartType::Parliament
        | ChartType::Treemap
        | ChartType::Sankey
        | ChartType::Survival
        | ChartType::Scatter => {
            unreachable!("a diagram writes its own table")
        }
    };
    // A column of text after the values: the group of a grouped bar chart, the outliers of a box.
    let extra = table_extra(spec);
    let columns = std::iter::once(first.to_owned())
        .chain(
            dataset
                .series
                .iter()
                .map(|series| series.name.as_deref().unwrap_or(words.value).to_owned()),
        )
        .chain(extra.iter().map(|(head, _)| head.clone()))
        .collect();
    let rows = dataset
        .categories
        .iter()
        .enumerate()
        .map(|(index, category)| {
            std::iter::once(category.clone())
                .chain(dataset.series.iter().map(|series| {
                    series.values[index].map_or_else(
                        || words.missing.to_owned(),
                        |value| {
                            format_value(value, series.style.unwrap_or_else(|| spec.number_style()))
                        },
                    )
                }))
                .chain(extra.iter().map(|(_, cells)| cells[index].clone()))
                .collect()
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}

/// The text columns that follow the values of a table, if the chart has any: each with its head
/// and one cell per row. A grouped bar chart names the group of every bar, a box plot its group
/// and its outliers.
pub(super) fn table_extra(spec: &ChartSpec) -> Vec<(String, Vec<String>)> {
    let words = spec.locale.words();
    if spec.chart_type == ChartType::Boxplot {
        let mut columns = Vec::new();
        if !spec.box_groups().is_empty() {
            columns.push((
                words.group.to_owned(),
                spec.boxes
                    .iter()
                    .map(|boxed| boxed.group.clone().unwrap_or_default())
                    .collect(),
            ));
        }
        columns.push((
            words.outliers.to_owned(),
            crate::boxplot::outlier_cells(spec),
        ));
        return columns;
    }
    if spec.has_error_bars() && !spec.series.is_empty() {
        let show = |value| format_value(value, spec.number_style());
        let several = spec.series.len() > 1;
        return spec
            .series
            .iter()
            .enumerate()
            .filter(|(_, series)| !series.lower.is_empty())
            .map(|(series_index, series)| {
                let head = if several {
                    format!("{} – {}", series.name, words.interval)
                } else {
                    words.interval.to_owned()
                };
                let cells = (0..spec.categories.len())
                    .map(|index| match spec.error_bounds(index, series_index) {
                        Some((lower, upper)) => {
                            format!("{} {} {}", show(lower), words.to, show(upper))
                        }
                        None => String::new(),
                    })
                    .collect();
                (head, cells)
            })
            .collect();
    }
    if spec.has_error_bars() {
        let show = |value| format_value(value, spec.number_style());
        return vec![(
            words.interval.to_owned(),
            spec.data
                .iter()
                .map(|point| match point.lower.zip(point.upper) {
                    Some((lower, upper)) => format!("{} {} {}", show(lower), words.to, show(upper)),
                    None => String::new(),
                })
                .collect(),
        )];
    }
    if spec.data_groups().is_empty() {
        return Vec::new();
    }
    vec![(
        words.group.to_owned(),
        spec.data
            .iter()
            .map(|point| point.group.clone().unwrap_or_default())
            .collect(),
    )]
}
