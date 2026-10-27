use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_SERIES, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many columns a mosaic chart takes.
pub(crate) const MAX_COLUMNS: usize = 12;

/// A mosaic chart (also called a Marimekko chart): columns whose width is their share of the whole
/// and whose segments split each column by share, such as the market shares of several regions
/// split by product.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MosaicSpec {
    pub columns: Vec<MosaicColumnSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MosaicColumnSpec {
    pub label: String,
    pub segments: Vec<MosaicSegmentSpec>,
}

/// One segment of a column. Segments of the same label in several columns share a color.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MosaicSegmentSpec {
    pub label: String,
    pub value: f64,
}

impl MosaicSpec {
    /// The labels of the segments in the order they first appear; the index is the color.
    pub(crate) fn segments(&self) -> Vec<&str> {
        let mut labels: Vec<&str> = Vec::new();
        for segment in self.columns.iter().flat_map(|column| &column.segments) {
            if !labels.contains(&segment.label.as_str()) {
                labels.push(&segment.label);
            }
        }
        labels
    }

    /// The sum of a column.
    pub(crate) fn column_total(&self, column: usize) -> f64 {
        self.columns[column]
            .segments
            .iter()
            .map(|segment| segment.value)
            .sum()
    }

    /// The sum of all columns.
    pub(crate) fn total(&self) -> f64 {
        (0..self.columns.len())
            .map(|column| self.column_total(column))
            .sum()
    }
}

impl ChartSpec {
    /// One to twelve columns with unique labels, each with segments of unique labels and values
    /// above zero; up to four different segment labels in all, one per palette color.
    pub(super) fn validate_mosaic(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
                    "a mosaic chart is drawn from columns and segments; remove this field",
                ));
            }
        }
        let Some(mosaic) = &self.mosaic else {
            return Err(ChartError::new(
                "missing_mosaic",
                "/mosaic",
                "a mosaic chart requires a mosaic block",
            ));
        };
        if mosaic.columns.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/mosaic/columns",
                "provide at least one column",
            ));
        }
        if mosaic.columns.len() > MAX_COLUMNS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/mosaic/columns",
                format!("at most {MAX_COLUMNS} columns are supported"),
            ));
        }
        let mut columns = BTreeSet::new();
        for (index, column) in mosaic.columns.iter().enumerate() {
            let path = format!("/mosaic/columns/{index}");
            validate_text(&column.label, &format!("{path}/label"), 100)?;
            if !columns.insert(column.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "column labels must be unique",
                ));
            }
            if column.segments.is_empty() {
                return Err(ChartError::new(
                    "empty_data",
                    format!("{path}/segments"),
                    "provide at least one segment",
                ));
            }
            let mut segments = BTreeSet::new();
            for (position, segment) in column.segments.iter().enumerate() {
                let path = format!("{path}/segments/{position}");
                validate_text(&segment.label, &format!("{path}/label"), 100)?;
                if !segments.insert(segment.label.as_str()) {
                    return Err(ChartError::new(
                        "duplicate_label",
                        format!("{path}/label"),
                        "segment labels must be unique within a column",
                    ));
                }
                validate_number(segment.value, &format!("{path}/value"))?;
                if segment.value <= 0.0 {
                    return Err(ChartError::new(
                        "invalid_value",
                        format!("{path}/value"),
                        "a segment has a value above zero; leave a segment out of a column to give it none",
                    ));
                }
            }
        }
        if mosaic.segments().len() > MAX_SERIES {
            return Err(ChartError::new(
                "too_many_series",
                "/mosaic/columns",
                format!(
                    "at most {MAX_SERIES} different segment labels are supported, one per palette color"
                ),
            ));
        }
        Ok(Vec::new())
    }
}
