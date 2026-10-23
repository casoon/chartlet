//! Small multiples: panels of time layers or of bars, and the grid they fill.

use std::collections::BTreeSet;

use super::{
    LayerContext, MAX_BAR_PANEL_CATEGORIES, MAX_COLUMNS, MAX_PANELS, MIN_PANELS, Mark, PaneSpec,
    default_height_ratio, validate::validate_pane_layers,
};
use crate::{
    error::{ChartError, ChartWarning},
    spec::{ChartSpec, MAX_SERIES, ValueAxisSpec, validate_number, validate_text},
};

impl ChartSpec {
    /// Small multiples: two to twelve titled panels, one shared value axis at the top level, and
    /// the layers of every panel validated like those of a time chart.
    /// Whether this is small multiples of bars: one panel per measure over shared categories.
    pub(crate) fn is_bar_multiples(&self) -> bool {
        self.chart_type == super::ChartType::Multiples && !self.categories.is_empty()
    }

    /// The dataset behind small multiples of bars: the shared categories, and one series per
    /// panel named by its title.
    pub(crate) fn bar_multiples_dataset(&self) -> super::Dataset {
        super::Dataset {
            categories: self.categories.clone(),
            series: self
                .panes
                .iter()
                .enumerate()
                .map(|(index, pane)| super::Series {
                    // The unit goes into the column head of the table.
                    name: pane
                        .title
                        .as_ref()
                        .map(|title| match pane.value_axis.unit.as_deref() {
                            Some(unit) => format!("{title} ({unit})"),
                            None => title.clone(),
                        }),
                    values: pane.values.clone(),
                    style: Some(self.pane_style(index)),
                })
                .collect(),
        }
    }

    /// Small multiples of bars: unique categories, and panels that each have a unique title and
    /// one value per category, on a value axis of their own.
    fn validate_bar_multiples(&self) -> Result<Vec<ChartWarning>, ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/timeAxis", !self.time_axis.is_default()),
            ("/valueAxis", self.value_axis != ValueAxisSpec::default()),
            ("/stack", self.stack.is_some()),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    "small multiples of bars take categories and the values and value axis of each panel; remove this field",
                ));
            }
        }
        if self.orientation != super::Orientation::Vertical {
            return Err(ChartError::new(
                "option_not_supported",
                "/orientation",
                "small multiples of bars draw horizontal bars; remove this field",
            ));
        }
        if self.categories.len() > MAX_BAR_PANEL_CATEGORIES {
            return Err(ChartError::new(
                "too_many_data_points",
                "/categories",
                format!(
                    "at most {MAX_BAR_PANEL_CATEGORIES} categories are supported in small multiples"
                ),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, label) in self.categories.iter().enumerate() {
            let path = format!("/categories/{index}");
            validate_text(label, &path, 200)?;
            if !labels.insert(label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    path,
                    "labels must be unique",
                ));
            }
        }
        self.validate_panel_count()?;
        let mut titles = BTreeSet::new();
        for (pane_index, pane) in self.panes.iter().enumerate() {
            self.validate_bar_panel(pane_index, pane, &mut titles)?;
        }
        Ok(Vec::new())
    }

    /// One panel of small multiples of bars: a unique title, no layers, and one value per
    /// category.
    fn validate_bar_panel<'a>(
        &'a self,
        pane_index: usize,
        pane: &'a PaneSpec,
        titles: &mut BTreeSet<&'a str>,
    ) -> Result<(), ChartError> {
        let path = format!("/panes/{pane_index}");
        let Some(title) = &pane.title else {
            return Err(ChartError::new(
                "missing_title",
                format!("{path}/title"),
                "every panel of small multiples needs a title",
            ));
        };
        validate_text(title, &format!("{path}/title"), 100)?;
        if !titles.insert(title.as_str()) {
            return Err(ChartError::new(
                "duplicate_title",
                format!("{path}/title"),
                "panel titles must be unique",
            ));
        }
        if !pane.layers.is_empty() || pane.stack.is_some() {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/layers"),
                "a panel of bars takes values, not layers; remove them",
            ));
        }
        if pane.height_ratio != default_height_ratio() {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/heightRatio"),
                "the panels of small multiples all have the same size",
            ));
        }
        if pane.values.len() != self.categories.len() {
            return Err(ChartError::new(
                "series_length_mismatch",
                format!("{path}/values"),
                format!(
                    "a panel needs one value per category: {} categories, {} values",
                    self.categories.len(),
                    pane.values.len()
                ),
            ));
        }
        if pane.values.iter().all(Option::is_none) {
            return Err(ChartError::new(
                "empty_data",
                format!("{path}/values"),
                "provide at least one value in every panel",
            ));
        }
        for (index, value) in pane.values.iter().enumerate() {
            if let Some(value) = value {
                validate_number(*value, &format!("{path}/values/{index}"))?;
            }
        }
        Ok(())
    }

    /// The number of panels and of columns of small multiples.
    fn validate_panel_count(&self) -> Result<(), ChartError> {
        if self.panes.len() < MIN_PANELS {
            return Err(ChartError::new(
                "not_enough_panes",
                "/panes",
                format!("provide at least {MIN_PANELS} panes; a single one is a time chart"),
            ));
        }
        if self.panes.len() > MAX_PANELS {
            return Err(ChartError::new(
                "too_many_panes",
                "/panes",
                format!("at most {MAX_PANELS} panes are supported"),
            ));
        }
        let columns = self.multiples_columns();
        if !(1..=MAX_COLUMNS).contains(&columns) {
            return Err(ChartError::new(
                "invalid_columns",
                "/columns",
                format!("columns must be between 1 and {MAX_COLUMNS}"),
            ));
        }
        Ok(())
    }

    pub(in crate::spec) fn validate_multiples(&self) -> Result<Vec<ChartWarning>, ChartError> {
        if self.is_bar_multiples() {
            return self.validate_bar_multiples();
        }
        if let Some(index) = self.panes.iter().position(|pane| !pane.values.is_empty()) {
            return Err(ChartError::new(
                "option_not_supported",
                format!("/panes/{index}/values"),
                "values belong to small multiples of bars, which take categories; time panels take layers",
            ));
        }
        self.reject_bar_and_line_options("a small-multiples chart")?;
        let zone = self.validate_time_axis()?;
        self.validate_panel_count()?;
        let columns = self.multiples_columns();

        let mut warnings = Vec::new();
        let plot_pixels = usize::try_from(crate::layout::panel_plot_pixels(self.width, columns))
            .expect("a usize is at least 32 bits wide");
        let mut titles = BTreeSet::new();
        for (pane_index, pane) in self.panes.iter().enumerate() {
            let pane_path = format!("/panes/{pane_index}");
            let Some(title) = &pane.title else {
                return Err(ChartError::new(
                    "missing_title",
                    format!("{pane_path}/title"),
                    "every panel of small multiples needs a title",
                ));
            };
            validate_text(title, &format!("{pane_path}/title"), 100)?;
            if !titles.insert(title.as_str()) {
                return Err(ChartError::new(
                    "duplicate_title",
                    format!("{pane_path}/title"),
                    "panel titles must be unique",
                ));
            }
            if pane.value_axis != ValueAxisSpec::default() {
                return Err(ChartError::new(
                    "option_not_supported",
                    format!("{pane_path}/valueAxis"),
                    "small multiples share one value axis; set valueAxis at the top level",
                ));
            }
            if pane.height_ratio != default_height_ratio() {
                return Err(ChartError::new(
                    "option_not_supported",
                    format!("{pane_path}/heightRatio"),
                    "the panels of small multiples all have the same size",
                ));
            }
            if let Some(layer_index) = pane
                .layers
                .iter()
                .position(|layer| layer.mark == Mark::Ohlc)
            {
                return Err(ChartError::new(
                    "option_not_supported",
                    format!("{pane_path}/layers/{layer_index}/mark"),
                    "candles are drawn on a time chart; small multiples take line and area layers",
                ));
            }
            // A name is one series across the panels, so names are checked per panel.
            let context = LayerContext {
                zone,
                plot_pixels,
                named: pane.layers.iter().filter(|layer| layer.is_data()).count() > 1,
            };
            validate_pane_layers(
                pane,
                &pane_path,
                &context,
                &mut BTreeSet::new(),
                &mut warnings,
            )?;
        }
        if self.series_names().len() > MAX_SERIES {
            return Err(ChartError::new(
                "too_many_series",
                "/panes",
                format!(
                    "at most {MAX_SERIES} distinct layer names are supported across all panels"
                ),
            ));
        }
        self.validate_collapsed_times(zone)?;
        Ok(warnings)
    }

    /// The number of grid columns of small multiples: as declared, or up to three.
    pub(crate) fn multiples_columns(&self) -> u32 {
        self.columns.unwrap_or_else(|| {
            u32::try_from(self.panes.len().min(3)).expect("at most three columns")
        })
    }
}
