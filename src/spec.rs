use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_path_to_error::Segment;

use crate::error::{ChartError, ChartWarning};

const MAX_DATA_POINTS: usize = 100;
/// Limited so that every series keeps a color that stays distinguishable for common
/// color-vision deficiencies.
pub(crate) const MAX_SERIES: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChartSpec {
    pub schema_version: u8,
    #[serde(rename = "type")]
    pub chart_type: ChartType,
    #[serde(default)]
    pub orientation: Orientation,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub category_axis: CategoryAxisSpec,
    #[serde(default)]
    pub value_axis: ValueAxisSpec,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_show_values")]
    pub show_values: bool,
    /// Single-series data. Use either `data` or `categories` with `series`.
    #[serde(default)]
    pub data: Vec<DataPoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub series: Vec<SeriesSpec>,
    /// Optional zoom steps rendered as radio-selectable, pre-computed variants (HTML profile).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zoom_steps: Vec<ZoomStep>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Bar,
    Line,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CategoryAxisSpec {
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValueAxisSpec {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub format: ValueFormat,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValueFormat {
    #[default]
    Number,
    Percent,
}

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

/// A pre-computed zoom step: shows categories `from..=to` as its own chart variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZoomStep {
    pub label: String,
    pub from: usize,
    pub to: usize,
}

/// Categories × series: the single shape that layout, description and data table work with.
pub(crate) struct Dataset<'a> {
    pub categories: Vec<&'a str>,
    pub series: Vec<Series<'a>>,
}

pub(crate) struct Series<'a> {
    /// `None` when the chart was given as a single `data` list.
    pub name: Option<&'a str>,
    pub values: Vec<Option<f64>>,
}

impl Dataset<'_> {
    pub(crate) fn values(&self) -> impl Iterator<Item = f64> + '_ {
        self.series
            .iter()
            .flat_map(|series| series.values.iter().flatten().copied())
    }

    /// JSON pointer to a category label in the specification.
    pub(crate) fn category_path(&self, index: usize) -> String {
        if self.series[0].name.is_none() {
            format!("/data/{index}/label")
        } else {
            format!("/categories/{index}")
        }
    }
}

impl ChartSpec {
    /// Parses a chart specification from JSON.
    ///
    /// # Errors
    ///
    /// Returns `invalid_json` for malformed JSON and `invalid_spec` for JSON that does not match
    /// the versioned specification. Like every other error, both carry a JSON Pointer path; a
    /// syntax error points at the document root and names line and column in its message.
    pub fn from_json(input: &str) -> Result<Self, ChartError> {
        let mut deserializer = serde_json::Deserializer::from_str(input);
        serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
            let source = error.inner();
            let (code, path) = match source.classify() {
                serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                    ("invalid_json", "/".to_owned())
                }
                serde_json::error::Category::Data | serde_json::error::Category::Io => {
                    ("invalid_spec", json_pointer(error.path()))
                }
            };
            ChartError::new(code, path, source.to_string())
        })
    }

    pub(crate) fn validate(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.validate_metadata()?;
        let warnings = self.validate_data()?;
        if self.chart_type == ChartType::Line && self.data.iter().all(|point| point.value.is_none())
        {
            return Err(ChartError::new(
                "empty_series",
                "/data",
                "line charts require at least one numeric value",
            ));
        }
        Ok(warnings)
    }

    fn validate_metadata(&self) -> Result<(), ChartError> {
        if self.schema_version != 1 {
            return Err(ChartError::new(
                "unsupported_schema_version",
                "/schemaVersion",
                "expected schemaVersion 1",
            ));
        }
        if self.chart_type == ChartType::Line && self.orientation != Orientation::Vertical {
            return Err(ChartError::new(
                "option_not_supported",
                "/orientation",
                "orientation is only available for bar charts",
            ));
        }
        validate_text(&self.title, "/title", 200)?;
        if let Some(description) = &self.description {
            validate_text(description, "/description", 1_000)?;
        }
        if let Some(source) = &self.source {
            validate_text(source, "/source", 300)?;
        }
        validate_optional_text(
            self.category_axis.title.as_ref(),
            "/categoryAxis/title",
            100,
        )?;
        validate_optional_text(self.value_axis.title.as_ref(), "/valueAxis/title", 100)?;

        if !(320..=2_400).contains(&self.width) {
            return Err(ChartError::new(
                "invalid_dimension",
                "/width",
                "width must be between 320 and 2400",
            ));
        }
        if !(240..=1_600).contains(&self.height) {
            return Err(ChartError::new(
                "invalid_dimension",
                "/height",
                "height must be between 240 and 1600",
            ));
        }
        if self.zoom_steps.len() == 1 {
            return Err(ChartError::new(
                "not_enough_zoom_steps",
                "/zoomSteps",
                "provide at least 2 zoom steps so the view can be switched",
            ));
        }
        for (i, step) in self.zoom_steps.iter().enumerate() {
            validate_text(&step.label, &format!("/zoomSteps/{i}/label"), 40)?;
            if step.from > step.to {
                return Err(ChartError::new(
                    "invalid_zoom_step",
                    format!("/zoomSteps/{i}/from"),
                    "from must not be greater than to",
                ));
            }
            let count = if self.data.is_empty() {
                self.categories.len()
            } else {
                self.data.len()
            };
            if step.to >= count {
                return Err(ChartError::new(
                    "zoom_out_of_range",
                    format!("/zoomSteps/{i}/to"),
                    format!("to must be less than the number of categories ({count})"),
                ));
            }
        }
        if self.zoom_steps.len() > 4 {
            return Err(ChartError::new(
                "too_many_zoom_steps",
                "/zoomSteps",
                "at most 4 zoom steps are supported",
            ));
        }
        Ok(())
    }

    pub(crate) fn dataset(&self) -> Dataset<'_> {
        if self.series.is_empty() {
            Dataset {
                categories: self.data.iter().map(|point| point.label.as_str()).collect(),
                series: vec![Series {
                    name: None,
                    values: self.data.iter().map(|point| point.value).collect(),
                }],
            }
        } else {
            Dataset {
                categories: self.categories.iter().map(String::as_str).collect(),
                series: self
                    .series
                    .iter()
                    .map(|series| Series {
                        name: Some(series.name.as_str()),
                        values: series.values.clone(),
                    })
                    .collect(),
            }
        }
    }

    /// Returns a copy of this specification with categories and values sliced to `from..=to`.
    pub(crate) fn sliced(&self, from: usize, to: usize) -> ChartSpec {
        let mut spec = self.clone();
        if self.data.is_empty() {
            spec.categories = self.categories[from..=to].to_vec();
            spec.series = self
                .series
                .iter()
                .map(|s| {
                    let mut series = s.clone();
                    series.values = s.values[from..=to].to_vec();
                    series
                })
                .collect();
        } else {
            spec.data = self.data[from..=to].to_vec();
        }
        spec
    }

    fn validate_data(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
        if self.chart_type == ChartType::Line {
            return Err(ChartError::new(
                "option_not_supported",
                "/series",
                "series are only available for bar charts in this alpha; use data for a line chart",
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

/// Converts a deserialization path such as `data[0].value` into the JSON Pointer
/// `/data/0/value` that validation errors use. The document root is `/`.
fn json_pointer(path: &serde_path_to_error::Path) -> String {
    let pointer: String = path
        .iter()
        .filter_map(|segment| match segment {
            Segment::Seq { index } => Some(format!("/{index}")),
            Segment::Map { key } => Some(format!("/{}", key.replace('~', "~0").replace('/', "~1"))),
            Segment::Enum { variant } => Some(format!("/{variant}")),
            Segment::Unknown => None,
        })
        .collect();
    if pointer.is_empty() {
        "/".to_owned()
    } else {
        pointer
    }
}

fn validate_number(value: f64, path: &str) -> Result<(), ChartError> {
    if !value.is_finite() {
        return Err(ChartError::new(
            "non_finite_value",
            path,
            "value must be finite",
        ));
    }
    let magnitude = value.abs();
    if magnitude > 1e100 || (magnitude > 0.0 && magnitude < 1e-100) {
        return Err(ChartError::new(
            "unsupported_numeric_range",
            path,
            "value magnitude must be zero or between 1e-100 and 1e100",
        ));
    }
    Ok(())
}

fn validate_optional_text(
    value: Option<&String>,
    path: &str,
    max_length: usize,
) -> Result<(), ChartError> {
    if let Some(value) = value {
        validate_text(value, path, max_length)?;
    }
    Ok(())
}

fn validate_text(value: &str, path: &str, max_length: usize) -> Result<(), ChartError> {
    let length = value.chars().count();
    if value.trim().is_empty() {
        return Err(ChartError::new(
            "empty_text",
            path,
            "value must contain visible text",
        ));
    }
    if length > max_length {
        return Err(ChartError::new(
            "text_too_long",
            path,
            format!("value must not exceed {max_length} characters"),
        ));
    }
    if value.chars().any(|character| !is_xml_character(character)) {
        return Err(ChartError::new(
            "invalid_xml_character",
            path,
            "text contains a control character that XML 1.0 cannot represent",
        ));
    }
    Ok(())
}

fn is_xml_character(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
        || ('\u{20}'..='\u{D7FF}').contains(&character)
        || ('\u{E000}'..='\u{FFFD}').contains(&character)
        || ('\u{10000}'..='\u{10FFFF}').contains(&character)
}

const fn default_width() -> u32 {
    800
}

const fn default_height() -> u32 {
    450
}

const fn default_show_values() -> bool {
    true
}
