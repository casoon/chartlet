use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    ChartSpec, ChartType, MAX_DATA_POINTS, MAX_SERIES, Stack, validate_number, validate_text,
};
use crate::error::{ChartError, ChartWarning};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataPoint {
    pub label: String,
    pub value: Option<f64>,
}

/// One named series with one value per category; `None` marks a missing value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeriesSpec {
    pub name: String,
    pub values: Vec<Option<f64>>,
}

impl ChartSpec {
    pub(super) fn validate_data(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.validate_stack()?;
        if self.categories.is_empty() && self.series.is_empty() {
            self.validate_points()
        } else {
            self.validate_series()
        }
    }

    fn validate_series_shape(&self) -> Result<(), ChartError> {
        if !self.data.is_empty() {
            return Err(ChartError::new(
                "conflicting_data_shape",
                "/data",
                "use either data or categories with series, not both",
            ));
        }
        if self.categories.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/categories",
                "provide at least one category",
            ));
        }
        if self.categories.len() > MAX_DATA_POINTS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/categories",
                format!("at most {MAX_DATA_POINTS} categories are supported"),
            ));
        }
        if self.series.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/series",
                "provide at least one series",
            ));
        }
        if self.series.len() > MAX_SERIES {
            return Err(ChartError::new(
                "too_many_series",
                "/series",
                format!("at most {MAX_SERIES} series are supported"),
            ));
        }
        Ok(())
    }

    /// A stack needs several series; a percent stack takes no negative values, since a share of
    /// a total cannot be negative.
    fn validate_stack(&self) -> Result<(), ChartError> {
        if self.stack.is_none() {
            return Ok(());
        }
        if self.series.len() < 2 {
            return Err(ChartError::new(
                "option_not_supported",
                "/stack",
                "stacking needs categories with at least two series",
            ));
        }
        if self.stack == Some(Stack::Percent) {
            for (series_index, series) in self.series.iter().enumerate() {
                if let Some(value_index) = series
                    .values
                    .iter()
                    .position(|value| value.is_some_and(|value| value < 0.0))
                {
                    return Err(ChartError::new(
                        "invalid_value",
                        format!("/series/{series_index}/values/{value_index}"),
                        "a percent stack shows shares of a total and takes no negative values",
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_series(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.validate_series_shape()?;
        let mut labels = BTreeSet::new();
        for (index, category) in self.categories.iter().enumerate() {
            let path = format!("/categories/{index}");
            validate_text(category, &path, 200)?;
            if !labels.insert(category.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    path,
                    "categories must be unique",
                ));
            }
        }

        let mut names = BTreeSet::new();
        for (series_index, series) in self.series.iter().enumerate() {
            let name_path = format!("/series/{series_index}/name");
            validate_text(&series.name, &name_path, 100)?;
            if !names.insert(series.name.as_str()) {
                return Err(ChartError::new(
                    "duplicate_series",
                    name_path,
                    "series names must be unique",
                ));
            }
            if series.values.len() != self.categories.len() {
                return Err(ChartError::new(
                    "series_length_mismatch",
                    format!("/series/{series_index}/values"),
                    format!(
                        "expected {} values, one per category; use null for a missing value",
                        self.categories.len()
                    ),
                ));
            }
            for (value_index, value) in series.values.iter().enumerate() {
                if let Some(value) = value {
                    validate_number(
                        *value,
                        &format!("/series/{series_index}/values/{value_index}"),
                    )?;
                }
            }
        }
        if self
            .series
            .iter()
            .flat_map(|series| &series.values)
            .all(Option::is_none)
        {
            return Err(ChartError::new(
                "empty_series",
                "/series",
                "provide at least one numeric value",
            ));
        }

        let mut warnings = Vec::new();
        if self.categories.len() > 16 {
            warnings.push(ChartWarning::new(
                "dense_chart",
                "/categories",
                "more than 16 categories can be difficult to read at the configured size",
            ));
        }
        Ok(warnings)
    }

    fn validate_points(&self) -> Result<Vec<ChartWarning>, ChartError> {
        if self.data.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/data",
                "provide at least one data point",
            ));
        }
        if self.data.len() > MAX_DATA_POINTS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/data",
                format!("at most {MAX_DATA_POINTS} data points are supported"),
            ));
        }

        let mut labels = BTreeSet::new();
        let mut warnings = Vec::new();
        for (index, point) in self.data.iter().enumerate() {
            let label_path = format!("/data/{index}/label");
            validate_text(&point.label, &label_path, 200)?;
            if let Some(value) = point.value {
                validate_number(value, &format!("/data/{index}/value"))?;
            } else if self.chart_type == ChartType::Bar {
                return Err(ChartError::new(
                    "missing_bar_value",
                    format!("/data/{index}/value"),
                    "bar charts require a numeric value for every category",
                ));
            }
            if !labels.insert(point.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    label_path,
                    "labels must be unique in a single-series chart",
                ));
            }
        }

        if self.data.len() > 16 {
            warnings.push(ChartWarning::new(
                "dense_chart",
                "/data",
                "more than 16 categories can be difficult to read at the configured size",
            ));
        }
        Ok(warnings)
    }
}
