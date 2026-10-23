//! Validation of a time chart: its panes, the sparkline form and the fields it does not take.

use std::collections::BTreeSet;

use super::{
    LayerContext, MAX_HEIGHT_RATIO, MAX_TIME_PANES, Mark, validate::validate_pane_layers,
    validate::validate_pane_stack,
};
use crate::{
    error::{ChartError, ChartWarning},
    layout::plot_pixels,
    spec::{
        AxisScale, ChartSpec, ChartType, MAX_SERIES, Orientation, ValueFormat,
        validate_optional_text,
    },
};

impl ChartSpec {
    /// A sparkline draws one pane of lines and areas and nothing that needs a label or an axis.
    fn validate_sparkline(&self) -> Result<(), ChartError> {
        if !self.sparkline {
            return Ok(());
        }
        let refuse = |path: String, reason: &str| {
            Err(ChartError::new(
                "option_not_supported",
                path,
                format!("a sparkline {reason}"),
            ))
        };
        if self.panes.len() > 1 {
            return refuse("/panes".to_owned(), "has one pane");
        }
        for (field, present) in [
            ("/timeAxis/title", self.time_axis.title.is_some()),
            ("/timeAxis/step", self.time_axis.step.is_some()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/mobile", self.mobile.is_some()),
            ("/legend", self.legend != super::LegendPlacement::Top),
        ] {
            if present {
                return refuse(
                    field.to_owned(),
                    "has no axes, legend, zoom or mobile variant",
                );
            }
        }
        for (pane_index, pane) in self.panes.iter().enumerate() {
            // Ticks, their step and the unit after the top one belong to axes a sparkline has not.
            for (field, present) in [
                ("title", pane.value_axis.title.is_some()),
                ("unit", pane.value_axis.unit.is_some()),
                ("step", pane.value_axis.step.is_some()),
            ] {
                if present {
                    return refuse(
                        format!("/panes/{pane_index}/valueAxis/{field}"),
                        "has no axes",
                    );
                }
            }
            for (index, layer) in pane.layers.iter().enumerate() {
                if !matches!(layer.mark, Mark::Line | Mark::Area) {
                    return refuse(
                        format!("/panes/{pane_index}/layers/{index}/mark"),
                        "draws lines and areas only",
                    );
                }
            }
        }
        Ok(())
    }

    /// Rejects the fields that only a bar or line chart has, naming the field and the fix. Small
    /// multiples keep the top-level value axis, which all their panels share.
    pub(super) fn reject_bar_and_line_options(&self, noun: &str) -> Result<(), ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            (
                "/zoomSteps",
                !self.zoom_steps.is_empty() && self.chart_type == ChartType::Multiples,
            ),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    format!("{noun} is drawn from panes and layers; remove this field"),
                ));
            }
        }
        if self.orientation != Orientation::Vertical {
            return Err(ChartError::new(
                "option_not_supported",
                "/orientation",
                "orientation is only available for bar charts",
            ));
        }
        if self.category_axis.title.is_some() {
            return Err(ChartError::new(
                "option_not_supported",
                "/categoryAxis/title",
                format!("{noun} labels its time axis with timeAxis.title"),
            ));
        }
        if self.chart_type == ChartType::Multiples {
            return Ok(());
        }
        if self.value_axis.title.is_some() {
            return Err(ChartError::new(
                "option_not_supported",
                "/valueAxis/title",
                "a time chart sets the value axis title inside its pane",
            ));
        }
        if self.value_axis.format != ValueFormat::Number {
            return Err(ChartError::new(
                "option_not_supported",
                "/valueAxis/format",
                "a time chart sets the value format inside its pane",
            ));
        }
        if self.value_axis.min.is_some()
            || self.value_axis.max.is_some()
            || self.value_axis.thousands_separator
            || self.value_axis.scale != AxisScale::Linear
            || self.value_axis.reverse
            || self.value_axis.step.is_some()
            || self.value_axis.exact
        {
            return Err(ChartError::new(
                "option_not_supported",
                "/valueAxis",
                "a time chart sets the value axis range and number style inside its pane",
            ));
        }
        Ok(())
    }

    pub(in crate::spec) fn validate_time(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_bar_and_line_options("a time chart")?;
        self.validate_sparkline()?;
        if self.legend == super::LegendPlacement::End
            && self.layers().any(|layer| layer.mark == Mark::Ohlc)
        {
            return Err(ChartError::new(
                "option_not_supported",
                "/legend",
                "candles need the legend that says which bodies rise and which fall",
            ));
        }
        let zone = self.validate_time_axis()?;
        if self.panes.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/panes",
                "provide at least one pane",
            ));
        }
        if self.panes.len() > MAX_TIME_PANES {
            return Err(ChartError::new(
                "too_many_panes",
                "/panes",
                format!("at most {MAX_TIME_PANES} panes are supported"),
            ));
        }

        let mut warnings = Vec::new();
        // Observations beyond one per horizontal pixel cannot be told apart in the drawing.
        let plot_pixels =
            usize::try_from(plot_pixels(self.width)).expect("a usize is at least 32 bits wide");
        // The panes share one legend, so a name has to be unique across all of them, and every
        // layer needs one as soon as the chart has more than one.
        let data_layers = self.data_layers().count();
        let mut names = BTreeSet::new();
        for (pane_index, pane) in self.panes.iter().enumerate() {
            let pane_path = format!("/panes/{pane_index}");
            if pane.title.is_some() {
                return Err(ChartError::new(
                    "option_not_supported",
                    format!("{pane_path}/title"),
                    "pane titles head the panels of small multiples; a time chart uses its title, and a pane is named by its valueAxis title",
                ));
            }
            if !(1..=MAX_HEIGHT_RATIO).contains(&pane.height_ratio) {
                return Err(ChartError::new(
                    "invalid_height_ratio",
                    format!("{pane_path}/heightRatio"),
                    format!("heightRatio must be between 1 and {MAX_HEIGHT_RATIO}"),
                ));
            }
            validate_optional_text(
                pane.value_axis.title.as_ref(),
                &format!("{pane_path}/valueAxis/title"),
                100,
            )?;
            let context = LayerContext {
                zone,
                plot_pixels,
                named: data_layers > 1,
            };
            validate_pane_layers(pane, &pane_path, &context, &mut names, &mut warnings)?;
            validate_pane_stack(pane, &pane_path, zone)?;
        }
        // One legend: at most as many layers take a palette color as the palette has colors.
        if let Some(entry) = self
            .data_layers()
            .filter(|entry| entry.layer.takes_palette())
            .nth(MAX_SERIES)
        {
            return Err(ChartError::new(
                "too_many_layers",
                format!("/panes/{}/layers/{}/color", entry.pane, entry.local),
                format!(
                    "at most {MAX_SERIES} data layers of a chart take a palette color; give this layer a color of its own"
                ),
            ));
        }
        self.validate_collapsed_times(zone)?;
        self.validate_time_zoom(zone)?;
        Ok(warnings)
    }
}
