use serde::{Deserialize, Serialize};

use super::{AxisScale, ChartSpec, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// The most columns one point-and-figure chart draws, the most boxes in all of them, and the
/// most highlighted ranges of columns.
pub(crate) const MAX_PNF_COLUMNS: usize = 200;
pub(crate) const MAX_PNF_BOXES: usize = 5_000;
pub(crate) const MAX_HIGHLIGHTS: usize = 4;

/// A point-and-figure chart: columns of X (rising) and O (falling) on a grid of boxes. The
/// horizontal axis counts columns, not time; the vertical axis is the value, in boxes of
/// `boxSize`. The columns come from the caller, already computed: chartlet draws one mark per
/// filled box and decides nothing about reversals.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PointFigureSpec {
    /// The height of one box, in the units of the value axis.
    pub box_size: f64,
    pub columns: Vec<PnfColumnSpec>,
    /// Ranges of columns set apart by a tint behind them, each with a label.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub highlights: Vec<PnfHighlightSpec>,
}

/// Which way a column runs, and so which mark fills its boxes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PnfDirection {
    /// A rising column of X.
    Up,
    /// A falling column of O.
    Down,
}

/// One column: its direction and the values of its lowest and highest filled box. Every box
/// from `low` to `high` is filled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PnfColumnSpec {
    pub direction: PnfDirection,
    pub low: f64,
    pub high: f64,
}

/// Columns `from` to `to`, numbered from 1 as the axis numbers them, set apart with a label.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PnfHighlightSpec {
    pub from: usize,
    pub to: usize,
    pub label: String,
}

impl PointFigureSpec {
    /// How many boxes a column fills: its span in boxes, both ends included.
    pub(crate) fn boxes(&self, column: &PnfColumnSpec) -> usize {
        boxes_between(column.low, column.high, self.box_size).unwrap_or(1)
    }
}

/// The boxes from `low` to `high`, both included, if `high - low` is a whole number of boxes.
fn boxes_between(low: f64, high: f64, box_size: f64) -> Option<usize> {
    let steps = whole_steps(high - low, box_size)?;
    usize::try_from(steps).ok().map(|steps| steps + 1)
}

/// `distance / box_size` when it is a whole number, allowing for the rounding of decimal values
/// such as 0.1.
#[allow(clippy::cast_possible_truncation)]
fn whole_steps(distance: f64, box_size: f64) -> Option<i64> {
    let steps = distance / box_size;
    let rounded = steps.round();
    ((steps - rounded).abs() <= 1e-6 && rounded.abs() < 1e9).then_some(rounded as i64)
}

impl ChartSpec {
    /// Columns on one grid of boxes with their lowest box not above their highest, few enough to
    /// draw, and highlights over existing columns.
    pub(super) fn validate_pointfigure(&self) -> Result<Vec<ChartWarning>, ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
            ("/timeAxis", !self.time_axis.is_default()),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    "a point-and-figure chart is drawn from columns; remove this field",
                ));
            }
        }
        if self.value_axis.scale != AxisScale::Linear {
            return Err(ChartError::new(
                "option_not_supported",
                "/valueAxis/scale",
                "the boxes of a point-and-figure chart are equal steps on a linear axis",
            ));
        }
        if self.value_axis.reverse {
            return Err(ChartError::new(
                "option_not_supported",
                "/valueAxis/reverse",
                "a point-and-figure chart rises upward",
            ));
        }
        let Some(chart) = &self.pointfigure else {
            return Err(ChartError::new(
                "missing_pointfigure",
                "/pointfigure",
                "a point-and-figure chart requires a pointfigure block",
            ));
        };
        validate_number(chart.box_size, "/pointfigure/boxSize")?;
        if chart.box_size <= 0.0 {
            return Err(ChartError::new(
                "invalid_box_size",
                "/pointfigure/boxSize",
                "the box size must be above zero",
            ));
        }
        if chart.columns.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/pointfigure/columns",
                "provide at least one column",
            ));
        }
        if chart.columns.len() > MAX_PNF_COLUMNS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/pointfigure/columns",
                format!("at most {MAX_PNF_COLUMNS} columns are supported"),
            ));
        }
        let origin = chart.columns[0].low;
        let mut boxes = 0;
        for (index, column) in chart.columns.iter().enumerate() {
            let path = format!("/pointfigure/columns/{index}");
            validate_number(column.low, &format!("{path}/low"))?;
            validate_number(column.high, &format!("{path}/high"))?;
            if column.low > column.high {
                return Err(ChartError::new(
                    "invalid_range",
                    format!("{path}/high"),
                    "high must not be below low",
                ));
            }
            if whole_steps(column.low - origin, chart.box_size).is_none() {
                return Err(ChartError::new(
                    "off_grid",
                    format!("{path}/low"),
                    "every box lies on the grid of the first column's low value, a whole number of boxes away",
                ));
            }
            let Some(count) = boxes_between(column.low, column.high, chart.box_size) else {
                return Err(ChartError::new(
                    "off_grid",
                    format!("{path}/high"),
                    "high must lie a whole number of boxes above low",
                ));
            };
            boxes += count;
        }
        if boxes > MAX_PNF_BOXES {
            return Err(ChartError::new(
                "too_many_data_points",
                "/pointfigure/columns",
                format!(
                    "the columns fill {boxes} boxes; at most {MAX_PNF_BOXES} are supported, so raise boxSize or show fewer columns"
                ),
            ));
        }
        validate_highlights(chart)?;
        self.validate_references()?;
        Ok(Vec::new())
    }
}

/// At most [`MAX_HIGHLIGHTS`] highlights, each over columns that exist, from the first to the
/// last.
fn validate_highlights(chart: &PointFigureSpec) -> Result<(), ChartError> {
    if chart.highlights.len() > MAX_HIGHLIGHTS {
        return Err(ChartError::new(
            "too_many_highlights",
            "/pointfigure/highlights",
            format!("use at most {MAX_HIGHLIGHTS} highlighted ranges"),
        ));
    }
    for (index, highlight) in chart.highlights.iter().enumerate() {
        let path = format!("/pointfigure/highlights/{index}");
        validate_text(&highlight.label, &format!("{path}/label"), 60)?;
        for (field, column) in [("from", highlight.from), ("to", highlight.to)] {
            if !(1..=chart.columns.len()).contains(&column) {
                return Err(ChartError::new(
                    "invalid_highlight",
                    format!("{path}/{field}"),
                    format!(
                        "columns are numbered from 1 to {}, as the axis numbers them",
                        chart.columns.len()
                    ),
                ));
            }
        }
        if highlight.from > highlight.to {
            return Err(ChartError::new(
                "invalid_highlight",
                format!("{path}/to"),
                "to must not come before from",
            ));
        }
    }
    Ok(())
}
