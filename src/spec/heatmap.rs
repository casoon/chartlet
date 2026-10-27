use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_SERIES, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many rows and columns a heatmap may have.
const MAX_LINES: usize = 60;

/// A heatmap: values, or categories, in a grid of rows and columns. A cell that is not listed has
/// no value and is drawn as such.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatmapSpec {
    pub rows: Vec<String>,
    pub columns: Vec<String>,
    pub cells: Vec<HeatmapCellSpec>,
    /// The categories of a heatmap whose cells name one instead of a value, each with a fill and a
    /// symbol. Empty for a numeric heatmap.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    /// The color scale of a numeric heatmap; sequential by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<HeatmapScale>,
    /// Where a diverging scale turns; 0 by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub midpoint: Option<f64>,
    /// What the values stand for, named in the key; by default the word for a value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_label: Option<String>,
    /// The order of the rows; as listed by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_order: Option<HeatmapOrder>,
    /// The order of the columns; as listed by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_order: Option<HeatmapOrder>,
}

/// One cell: its row and column by label, and its value or its category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeatmapCellSpec {
    pub row: String,
    pub column: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HeatmapScale {
    /// One hue from the smallest value to the largest.
    #[default]
    Sequential,
    /// Two hues that part at the midpoint.
    Diverging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HeatmapOrder {
    /// As the rows or columns are listed.
    #[default]
    Listed,
    /// By label, alphabetically.
    Label,
    /// By the mean of the values, the largest first.
    Value,
}

impl HeatmapSpec {
    /// Whether the cells name a category instead of a value.
    pub(crate) fn is_categorical(&self) -> bool {
        !self.categories.is_empty()
    }

    pub(crate) fn midpoint(&self) -> f64 {
        self.midpoint.unwrap_or(0.0)
    }

    /// The indices of the rows and of the columns in the order they are drawn.
    pub(crate) fn order(&self) -> (Vec<usize>, Vec<usize>) {
        (
            ordered(&self.rows, self.row_order, &self.means(true)),
            ordered(&self.columns, self.column_order, &self.means(false)),
        )
    }

    /// The mean of the values in every row, or in every column.
    fn means(&self, rows: bool) -> Vec<Option<f64>> {
        let labels = if rows { &self.rows } else { &self.columns };
        labels
            .iter()
            .map(|label| {
                let values: Vec<f64> = self
                    .cells
                    .iter()
                    .filter(|cell| (if rows { &cell.row } else { &cell.column }) == label)
                    .filter_map(|cell| cell.value)
                    .collect();
                (!values.is_empty())
                    .then(|| values.iter().sum::<f64>() / crate::layout::count(values.len()))
            })
            .collect()
    }
}

/// The indices of `labels` in the order asked for. By mean, the largest first and the lines
/// without a value last, in the order listed.
fn ordered(labels: &[String], order: Option<HeatmapOrder>, means: &[Option<f64>]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..labels.len()).collect();
    match order.unwrap_or_default() {
        HeatmapOrder::Listed => {}
        HeatmapOrder::Label => {
            crate::sort::by_key(&mut indices, |index| (labels[*index].clone(), *index));
        }
        HeatmapOrder::Value => crate::sort::by(&mut indices, |a, b| match (means[*a], means[*b]) {
            (Some(a), Some(b)) => b.total_cmp(&a),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }),
    }
    indices
}

impl ChartSpec {
    /// Rows and columns with unique labels, cells that name a row and a column once, and either a
    /// value or a category in every cell.
    #[allow(clippy::too_many_lines)]
    pub(super) fn validate_heatmap(&self) -> Result<Vec<ChartWarning>, ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
            ("/references", !self.references.is_empty()),
            ("/timeAxis", !self.time_axis.is_default()),
            (
                "/valueAxis",
                self.value_axis != super::ValueAxisSpec::default(),
            ),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    "a heatmap is drawn from rows, columns and cells; remove this field",
                ));
            }
        }
        let Some(heatmap) = &self.heatmap else {
            return Err(ChartError::new(
                "missing_heatmap",
                "/heatmap",
                "a heatmap requires a heatmap block",
            ));
        };
        for (field, labels) in [("rows", &heatmap.rows), ("columns", &heatmap.columns)] {
            if labels.is_empty() {
                return Err(ChartError::new(
                    "empty_data",
                    format!("/heatmap/{field}"),
                    format!("provide at least one of {field}"),
                ));
            }
            if labels.len() > MAX_LINES {
                return Err(ChartError::new(
                    "too_many_data_points",
                    format!("/heatmap/{field}"),
                    format!("at most {MAX_LINES} {field} are supported"),
                ));
            }
            let mut seen = BTreeSet::new();
            for (index, label) in labels.iter().enumerate() {
                let path = format!("/heatmap/{field}/{index}");
                validate_text(label, &path, 100)?;
                if !seen.insert(label.as_str()) {
                    return Err(ChartError::new(
                        "duplicate_label",
                        path,
                        format!("the labels of {field} must be unique"),
                    ));
                }
            }
        }
        if heatmap.categories.len() > MAX_SERIES {
            return Err(ChartError::new(
                "too_many_series",
                "/heatmap/categories",
                format!("at most {MAX_SERIES} categories are supported, one per palette color"),
            ));
        }
        let mut seen = BTreeSet::new();
        for (index, label) in heatmap.categories.iter().enumerate() {
            let path = format!("/heatmap/categories/{index}");
            validate_text(label, &path, 100)?;
            if !seen.insert(label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    path,
                    "category labels must be unique",
                ));
            }
        }
        if heatmap.cells.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/heatmap/cells",
                "provide at least one cell",
            ));
        }
        Self::validate_heatmap_cells(heatmap)?;
        if heatmap.is_categorical() {
            for (field, present) in [
                ("scale", heatmap.scale.is_some()),
                ("midpoint", heatmap.midpoint.is_some()),
                ("valueLabel", heatmap.value_label.is_some()),
            ] {
                if present {
                    return Err(ChartError::new(
                        "option_not_supported",
                        format!("/heatmap/{field}"),
                        "a heatmap of categories has no scale; remove this field",
                    ));
                }
            }
            for (field, order) in [
                ("rowOrder", heatmap.row_order),
                ("columnOrder", heatmap.column_order),
            ] {
                if order == Some(HeatmapOrder::Value) {
                    return Err(ChartError::new(
                        "invalid_order",
                        format!("/heatmap/{field}"),
                        "categories have no mean; order by `label` or as `listed`",
                    ));
                }
            }
        } else if let Some(midpoint) = heatmap.midpoint {
            validate_number(midpoint, "/heatmap/midpoint")?;
            if heatmap.scale != Some(HeatmapScale::Diverging) {
                return Err(ChartError::new(
                    "option_not_supported",
                    "/heatmap/midpoint",
                    "a midpoint belongs to a diverging scale; set `scale` to `diverging`",
                ));
            }
        }
        if let Some(label) = &heatmap.value_label {
            validate_text(label, "/heatmap/valueLabel", 100)?;
        }
        Ok(Vec::new())
    }

    /// Every cell names a row and a column that exist, once, and holds a value or a category,
    /// according to what the heatmap shows.
    fn validate_heatmap_cells(heatmap: &HeatmapSpec) -> Result<(), ChartError> {
        let mut seen = BTreeSet::new();
        for (index, cell) in heatmap.cells.iter().enumerate() {
            let path = format!("/heatmap/cells/{index}");
            if !heatmap.rows.contains(&cell.row) {
                return Err(ChartError::new(
                    "unknown_row",
                    format!("{path}/row"),
                    "the row is not one of `rows`",
                ));
            }
            if !heatmap.columns.contains(&cell.column) {
                return Err(ChartError::new(
                    "unknown_column",
                    format!("{path}/column"),
                    "the column is not one of `columns`",
                ));
            }
            if !seen.insert((cell.row.as_str(), cell.column.as_str())) {
                return Err(ChartError::new(
                    "duplicate_cell",
                    path,
                    "a row and a column take one cell; leave a cell out to mark it as missing",
                ));
            }
            if heatmap.is_categorical() {
                match (&cell.category, cell.value) {
                    (Some(category), None) => {
                        if !heatmap.categories.contains(category) {
                            return Err(ChartError::new(
                                "unknown_category",
                                format!("{path}/category"),
                                "the category is not one of `categories`",
                            ));
                        }
                    }
                    _ => {
                        return Err(ChartError::new(
                            "invalid_cell",
                            path,
                            "a cell of a heatmap with `categories` names a `category` and has no `value`",
                        ));
                    }
                }
            } else {
                match (cell.value, &cell.category) {
                    (Some(value), None) => validate_number(value, &format!("{path}/value"))?,
                    _ => {
                        return Err(ChartError::new(
                            "invalid_cell",
                            path,
                            "a cell of a numeric heatmap has a `value` and no `category`; list `categories` to show categories",
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
