use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    ChartSpec, MAX_SERIES, ValueAxisSpec, ZoomBound, is_false, validate_number,
    validate_optional_text, validate_text,
};
use crate::{
    error::{ChartError, ChartWarning},
    layout::plot_pixels,
    time::TimeValue,
};

/// Per layer, measured against the size and the render time of the SVG and the HTML profile.
pub(crate) const MAX_TIME_POINTS_PER_LAYER: usize = 2_000;
/// Data layers per pane. Beyond the palette's [`MAX_SERIES`] colors a layer has to bring its own
/// color, and past six lines the legend and the plot stop being readable.
pub(crate) const MAX_TIME_LAYERS: usize = 6;
/// Stacked panes of a time chart; more than this stop being readable at the minimum chart height.
pub(crate) const MAX_TIME_PANES: usize = 4;
/// The largest share one pane may claim against another.
pub(crate) const MAX_HEIGHT_RATIO: u32 = 10;
/// Reference lines per pane. They are drawn over the data, so more than a handful hide it.
pub(crate) const MAX_ANNOTATIONS: usize = 6;
/// Small multiples: fewer than two panels is a time chart, more than twelve no longer fit a
/// page width with readable axes.
pub(crate) const MIN_PANELS: usize = 2;
pub(crate) const MAX_PANELS: usize = 12;
pub(crate) const MAX_COLUMNS: u32 = 6;

/// What the positions of a time axis are: calendar timestamps, or plain numbers such as a
/// distance, a depth or an age in millions of years.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TimeAxisKind {
    #[default]
    Calendar,
    Number,
}

/// Whether the time axis leaves a gap where an observation is missing, or collapses it.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Gaps {
    /// Keep the distance between timestamps; a missing observation is simply not drawn.
    #[default]
    Show,
    /// Close the gap so the neighbours move together.
    Collapse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimeAxisSpec {
    /// `UTC` (default) or a fixed offset such as `+02:00`. Timestamps that do not carry their own
    /// offset are read as wall-clock time in this zone.
    #[serde(default = "default_timezone")]
    pub timezone: String,
    #[serde(default)]
    pub gaps: Gaps,
    #[serde(default)]
    pub title: Option<String>,
    /// `calendar` (default): timestamps. `number`: every `time` is a plain number, for profiles
    /// along a distance or ages in millions of years.
    #[serde(default, skip_serializing_if = "TimeAxisKind::is_calendar")]
    pub kind: TimeAxisKind,
    /// Runs the axis from right to left, the largest position at the left: ages before present
    /// read from the oldest on the left to today on the right.
    #[serde(default, skip_serializing_if = "is_false")]
    pub reverse: bool,
}

impl TimeAxisKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_calendar(&self) -> bool {
        matches!(self, Self::Calendar)
    }
}

impl TimeAxisSpec {
    pub(super) fn is_default(&self) -> bool {
        self.timezone == default_timezone()
            && self.gaps == Gaps::Show
            && self.title.is_none()
            && self.kind == TimeAxisKind::Calendar
            && !self.reverse
    }
}

impl Default for TimeAxisSpec {
    fn default() -> Self {
        Self {
            timezone: default_timezone(),
            gaps: Gaps::default(),
            title: None,
            kind: TimeAxisKind::default(),
            reverse: false,
        }
    }
}

/// One pane of a time chart: a value axis over the shared time axis, with its own layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaneSpec {
    /// Heading of a small-multiples panel; a time chart's single pane has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Share of the plot area this pane takes; a single pane always takes all of it.
    #[serde(default = "default_height_ratio")]
    pub height_ratio: u32,
    #[serde(default)]
    pub value_axis: ValueAxisSpec,
    #[serde(default)]
    pub layers: Vec<LayerSpec>,
}

/// How a layer draws its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    Line,
    Area,
    Ohlc,
    Band,
    Annotation,
}

/// A marker shape for annotating a single point.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Shape {
    TriangleUp,
    TriangleDown,
    Square,
    Circle,
    Diamond,
}

/// One layer of a pane. Which fields apply depends on `mark`; anything that belongs to a
/// different mark is rejected with a path, never ignored.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LayerSpec {
    pub mark: Mark,
    /// Required as soon as a pane has more than one layer: the legend needs a name.
    #[serde(default)]
    pub name: Option<String>,
    /// `#rgb`, `#rrggbb`, `#rrggbbaa` or `var(--name)`; anything else is reported and replaced by
    /// the neutral tone.
    #[serde(default)]
    pub color: Option<String>,
    /// Observations for `line` and `area`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<TimePoint>,
    /// Observations for `ohlc`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<OhlcPoint>,
    /// Start of a `band`.
    #[serde(default)]
    pub from: Option<TimeValue>,
    /// End of a `band`.
    #[serde(default)]
    pub to: Option<TimeValue>,
    /// Upper edge of a `band`.
    #[serde(default)]
    pub top: Option<f64>,
    /// Lower edge of a `band`.
    #[serde(default)]
    pub bottom: Option<f64>,
    /// Position of an `annotation`.
    #[serde(default)]
    pub time: Option<TimeValue>,
    /// Value an `annotation` points at.
    #[serde(default)]
    pub value: Option<f64>,
    /// Text of an `annotation`.
    #[serde(default)]
    pub label: Option<String>,
    /// Marker shape of an `annotation`.
    #[serde(default)]
    pub shape: Option<Shape>,
    /// The line comes from a model rather than a measurement: it is dashed and its band, if the
    /// points carry one, is hatched.
    #[serde(default, skip_serializing_if = "is_false")]
    pub modeled: bool,
    /// Line weight: `thin` sets a line back, for example single years under their mean, and
    /// `bold` brings one forward, so that two lines differ in more than color.
    #[serde(default, skip_serializing_if = "Stroke::is_regular")]
    pub stroke: Stroke,
    /// Line pattern. Absent, a modeled line is dashed and every other line solid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dash: Option<Dash>,
}

/// The weight of a line.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Stroke {
    #[default]
    Regular,
    Thin,
    Bold,
}

/// The pattern of a line.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Dash {
    Solid,
    Dashed,
    Dotted,
}

impl Stroke {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_regular(&self) -> bool {
        matches!(self, Self::Regular)
    }
}

impl LayerSpec {
    /// The color to paint this layer with: the declared color when it passes the contract, the
    /// neutral tone when it does not, and `None` when the layer declares no color at all.
    pub(crate) fn resolved_color(&self) -> Option<&str> {
        self.color
            .as_deref()
            .map(|color| crate::color::sanitize(color).unwrap_or(crate::color::NEUTRAL))
    }

    /// The layer's observations as Unix seconds and values. Points that do not resolve are left
    /// out; validation rejects them before layout ever runs.
    pub(crate) fn resolved_points(&self, zone: crate::time::TimeZone) -> Vec<(i64, f64)> {
        self.points
            .iter()
            .filter_map(|point| {
                let epoch = point.time.resolve(zone).ok()?;
                Some((epoch, point.value?))
            })
            .collect()
    }

    /// The layer's band as Unix seconds with lower and upper edge, empty when the points carry
    /// none. Validation guarantees that either every point has both edges or none has.
    pub(crate) fn resolved_band(&self, zone: crate::time::TimeZone) -> Vec<(i64, f64, f64)> {
        self.points
            .iter()
            .filter_map(|point| {
                let epoch = point.time.resolve(zone).ok()?;
                Some((epoch, point.lower?, point.upper?))
            })
            .collect()
    }

    /// The layer's observations split at every `null` value: each run of consecutive values is
    /// drawn as its own piece of line, so a gap stays visible.
    pub(crate) fn resolved_segments(&self, zone: crate::time::TimeZone) -> Vec<Vec<(i64, f64)>> {
        let mut segments = vec![Vec::new()];
        for point in &self.points {
            let Ok(epoch) = point.time.resolve(zone) else {
                continue;
            };
            match point.value {
                Some(value) => segments
                    .last_mut()
                    .expect("there is always a current segment")
                    .push((epoch, value)),
                None => segments.push(Vec::new()),
            }
        }
        segments.retain(|segment| !segment.is_empty());
        segments
    }

    /// The timestamp of every observation, a missing value and a candle included.
    pub(crate) fn resolved_times(&self, zone: crate::time::TimeZone) -> Vec<i64> {
        self.points
            .iter()
            .map(|point| &point.time)
            .chain(self.data.iter().map(|candle| &candle.time))
            .filter_map(|time| time.resolve(zone).ok())
            .collect()
    }

    /// The candles of an `ohlc` layer as Unix seconds with open, high, low and close.
    pub(crate) fn resolved_candles(&self, zone: crate::time::TimeZone) -> Vec<(i64, [f64; 4])> {
        self.data
            .iter()
            .filter_map(|candle| {
                let epoch = candle.time.resolve(zone).ok()?;
                Some((epoch, [candle.open, candle.high, candle.low, candle.close]))
            })
            .collect()
    }

    /// The timestamps that carry a value: observations without a gap, and every candle.
    fn observed_times(&self, zone: crate::time::TimeZone) -> impl Iterator<Item = i64> {
        self.resolved_points(zone)
            .into_iter()
            .map(|(epoch, _)| epoch)
            .chain(
                self.resolved_candles(zone)
                    .into_iter()
                    .map(|(epoch, _)| epoch),
            )
    }

    /// The number of observations whose value is `null`.
    pub(crate) fn missing_values(&self) -> usize {
        self.points
            .iter()
            .filter(|point| point.value.is_none())
            .count()
    }

    /// Whether the points carry an uncertainty band. A missing value has no band, so any point
    /// with a lower edge decides.
    pub(crate) fn has_band(&self) -> bool {
        self.points.iter().any(|point| point.lower.is_some())
    }

    /// The line pattern: as declared, otherwise dashed for a modeled line and solid for any other.
    pub(crate) fn effective_dash(&self) -> Dash {
        self.dash.unwrap_or(if self.modeled {
            Dash::Dashed
        } else {
            Dash::Solid
        })
    }

    /// Whether the layer draws data rather than annotating it.
    pub(crate) fn is_data(&self) -> bool {
        !matches!(self.mark, Mark::Annotation | Mark::Band)
    }

    /// Whether the layer takes a palette color: a data layer other than candles, which are drawn
    /// in the rise and fall colors, and without a color of its own.
    pub(crate) fn takes_palette(&self) -> bool {
        self.is_data() && self.mark != Mark::Ohlc && self.color.is_none()
    }

    /// Whether the layer is a reference line: an annotation at either a time or a value.
    pub(crate) fn is_rule(&self) -> bool {
        self.mark == Mark::Annotation && (self.time.is_some() != self.value.is_some())
    }

    /// Whether the layer is a point marker: an annotation at both a time and a value.
    pub(crate) fn is_marker(&self) -> bool {
        self.mark == Mark::Annotation && self.time.is_some() && self.value.is_some()
    }

    /// The marker shape of a point annotation; a circle unless the specification names another.
    pub(crate) fn marker_shape(&self) -> Shape {
        self.shape.unwrap_or(Shape::Circle)
    }

    /// Whether the layer keeps at least two observations in `zone`.
    fn has_two_points(&self, zone: crate::time::TimeZone) -> bool {
        self.resolved_points(zone).len() >= 2
    }
}

/// One observation of a `line` or `area` layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimePoint {
    pub time: TimeValue,
    /// `null` marks a missing observation: the line and its band break there.
    pub value: Option<f64>,
    /// Lower edge of the uncertainty band around the line at this point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lower: Option<f64>,
    /// Upper edge of the uncertainty band around the line at this point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upper: Option<f64>,
}

/// One candlestick of an `ohlc` layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OhlcPoint {
    pub time: TimeValue,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl ChartSpec {
    /// The time axis options both time charts and small multiples share.
    fn validate_time_axis(&self) -> Result<crate::time::TimeZone, ChartError> {
        let zone = self.time_zone()?;
        validate_optional_text(self.time_axis.title.as_ref(), "/timeAxis/title", 100)?;
        if self.time_axis.kind == TimeAxisKind::Number {
            // A numeric axis has no calendar: no timezone, no calendar gaps to close, and zoom
            // steps name dates.
            for (field, present) in [
                (
                    "/timeAxis/timezone",
                    self.time_axis.timezone != default_timezone(),
                ),
                ("/timeAxis/gaps", self.time_axis.gaps != Gaps::Show),
                ("/zoomSteps", !self.zoom_steps.is_empty()),
            ] {
                if present {
                    return Err(ChartError::new(
                        "option_not_supported",
                        field,
                        "a numeric axis takes no timezone, gap collapsing or zoom steps",
                    ));
                }
            }
        }
        Ok(zone)
    }

    /// With gaps collapsed, the time axis has a place only for observations: a reference line or
    /// point marker takes the first observation at or after its time, so its time has to lie
    /// between the first and the last observation, and a zone covers the observations between
    /// its edges, so it has to hold at least one.
    fn validate_collapsed_times(&self, zone: crate::time::TimeZone) -> Result<(), ChartError> {
        let Some(slots) = self.time_slots(zone) else {
            return Ok(());
        };
        let (Some(&first), Some(&last)) = (slots.first(), slots.last()) else {
            return Ok(());
        };
        let resolve =
            |time: &Option<TimeValue>| time.as_ref().and_then(|time| time.resolve(zone).ok());
        for entry in self.indexed_layers() {
            let layer = entry.layer;
            let path = format!("/panes/{}/layers/{}", entry.pane, entry.local);
            if layer.mark == Mark::Annotation
                && let Some(time) = resolve(&layer.time)
                && !(first..=last).contains(&time)
            {
                return Err(ChartError::new(
                    "time_out_of_range",
                    format!("{path}/time"),
                    "with gaps collapsed the time axis only has room for observations; give a time between the first and the last observation",
                ));
            }
            if layer.mark == Mark::Band && (layer.from.is_some() || layer.to.is_some()) {
                let from = resolve(&layer.from).unwrap_or(i64::MIN);
                let to = resolve(&layer.to).unwrap_or(i64::MAX);
                if !slots.iter().any(|slot| (from..=to).contains(slot)) {
                    return Err(ChartError::new(
                        "time_out_of_range",
                        path,
                        "with gaps collapsed a zone covers observations; from and to must enclose at least one",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_time(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_bar_and_line_options("a time chart")?;
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

    /// Zoom steps of a time chart: two to four windows, each from one timestamp to a later one and
    /// holding at least two observations of some data layer.
    fn validate_time_zoom(&self, zone: crate::time::TimeZone) -> Result<(), ChartError> {
        if self.zoom_steps.len() == 1 {
            return Err(ChartError::new(
                "not_enough_zoom_steps",
                "/zoomSteps",
                "provide at least 2 zoom steps so the view can be switched",
            ));
        }
        for (i, step) in self.zoom_steps.iter().enumerate() {
            validate_text(&step.label, &format!("/zoomSteps/{i}/label"), 40)?;
            let resolve = |field: &str, bound: &ZoomBound| {
                bound.resolve(zone).map_err(|message| {
                    ChartError::new("invalid_time", format!("/zoomSteps/{i}/{field}"), message)
                })
            };
            let from = resolve("from", &step.from)?;
            let to = resolve("to", &step.to)?;
            if from >= to {
                return Err(ChartError::new(
                    "invalid_zoom_step",
                    format!("/zoomSteps/{i}/from"),
                    "from must lie before to",
                ));
            }
            // Every pane is drawn in the window, so every pane needs something to draw there.
            let covered = self.panes.iter().all(|pane| {
                pane.layers
                    .iter()
                    .filter(|layer| layer.is_data())
                    .any(|layer| {
                        layer
                            .observed_times(zone)
                            .filter(|epoch| (from..=to).contains(epoch))
                            .count()
                            >= 2
                    })
            });
            if !covered {
                return Err(ChartError::new(
                    "zoom_out_of_range",
                    format!("/zoomSteps/{i}"),
                    "the window must hold at least two observations of one layer in every pane",
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

    /// Small multiples: two to twelve titled panels, one shared value axis at the top level, and
    /// the layers of every panel validated like those of a time chart.
    pub(super) fn validate_multiples(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_bar_and_line_options("a small-multiples chart")?;
        let zone = self.validate_time_axis()?;
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

/// What the layers of a pane are checked against: the chart's time zone, the width of its plot
/// in pixels, and whether every data layer needs a name.
pub(crate) struct LayerContext {
    pub zone: crate::time::TimeZone,
    pub plot_pixels: usize,
    pub named: bool,
}

/// A layer's own color has to be one of the forms in the contract; anything else is replaced by
/// the neutral tone, and the specification says which layer it was.
fn check_layer_color(layer: &LayerSpec, path: &str, warnings: &mut Vec<ChartWarning>) {
    if layer
        .color
        .as_deref()
        .is_some_and(|color| crate::color::sanitize(color).is_none())
    {
        let color = layer.color.as_deref().unwrap_or_default();
        warnings.push(ChartWarning::new(
            "color_not_supported",
            format!("{path}/color"),
            format!(
                "{color:?} is not #rgb, #rrggbb, #rrggbbaa or var(--name); the layer is painted in the neutral tone"
            ),
        ));
    }
}

/// Every observation resolves to a timestamp and follows the one before it; a `null` value marks
/// a gap and carries no band.
fn validate_points(
    layer: &LayerSpec,
    path: &str,
    zone: crate::time::TimeZone,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    let banded = layer.has_band();
    let mut previous: Option<i64> = None;
    for (point_index, point) in layer.points.iter().enumerate() {
        let point_path = format!("{path}/points/{point_index}");
        let epoch = point.time.resolve(zone).map_err(|message| {
            ChartError::new("invalid_time", format!("{point_path}/time"), message)
        })?;
        if let Some(value) = point.value {
            validate_number(value, &format!("{point_path}/value"))?;
            validate_band_point(point, &point_path, banded, value, warnings)?;
        } else if point.lower.is_some() || point.upper.is_some() {
            let edge = if point.lower.is_some() {
                "lower"
            } else {
                "upper"
            };
            return Err(ChartError::new(
                "invalid_band",
                format!("{point_path}/{edge}"),
                "a missing value has no band; leave out lower and upper where value is null",
            ));
        }
        if previous.is_some_and(|previous| epoch <= previous) {
            return Err(ChartError::new(
                "unordered_time",
                format!("{point_path}/time"),
                "timestamps must increase from point to point",
            ));
        }
        previous = Some(epoch);
    }
    Ok(())
}

/// The band edges of one point with a value: both or neither, matching the rest of the layer, and the
/// lower edge not above the upper one. A value outside its own band is allowed but reported: a
/// median can leave a percentile band only when the two come from different sources.
fn validate_band_point(
    point: &TimePoint,
    point_path: &str,
    banded: bool,
    value: f64,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    for (edge, present) in [
        ("lower", point.lower.is_some()),
        ("upper", point.upper.is_some()),
    ] {
        if present != banded {
            return Err(ChartError::new(
                "incomplete_band",
                format!("{point_path}/{edge}"),
                "give every point of a layer both lower and upper, or none of them",
            ));
        }
    }
    let (Some(lower), Some(upper)) = (point.lower, point.upper) else {
        return Ok(());
    };
    validate_number(lower, &format!("{point_path}/lower"))?;
    validate_number(upper, &format!("{point_path}/upper"))?;
    if lower > upper {
        return Err(ChartError::new(
            "invalid_band",
            format!("{point_path}/upper"),
            "upper must not be below lower",
        ));
    }
    if !(lower..=upper).contains(&value) {
        warnings.push(ChartWarning::new(
            "value_outside_band",
            format!("{point_path}/value"),
            "the value lies outside its own band; check that line and band describe the same quantity",
        ));
    }
    Ok(())
}

/// The layers of one pane: data layers within their limit and at most as many without a color of
/// their own as the palette has colors, reference lines within their own limit, and a name on
/// every data layer once there is more than one.
fn validate_pane_layers(
    pane: &PaneSpec,
    pane_path: &str,
    context: &LayerContext,
    names: &mut BTreeSet<String>,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    let zone = context.zone;
    let data_layers = pane.layers.iter().filter(|layer| layer.is_data()).count();
    let annotations = pane.layers.len() - data_layers;
    if data_layers == 0 {
        return Err(ChartError::new(
            "empty_data",
            format!("{pane_path}/layers"),
            "provide at least one line layer; a reference line alone has nothing to refer to",
        ));
    }
    if data_layers > MAX_TIME_LAYERS {
        return Err(ChartError::new(
            "too_many_layers",
            format!("{pane_path}/layers"),
            format!("at most {MAX_TIME_LAYERS} data layers are supported"),
        ));
    }
    if let Some((layer_index, _)) = pane
        .layers
        .iter()
        .enumerate()
        .filter(|(_, layer)| layer.takes_palette())
        .nth(MAX_SERIES)
    {
        return Err(ChartError::new(
            "too_many_layers",
            format!("{pane_path}/layers/{layer_index}/color"),
            format!(
                "at most {MAX_SERIES} data layers take a palette color; give this layer a color of its own"
            ),
        ));
    }
    if annotations > MAX_ANNOTATIONS {
        return Err(ChartError::new(
            "too_many_annotations",
            format!("{pane_path}/layers"),
            format!("at most {MAX_ANNOTATIONS} annotation layers are supported per pane"),
        ));
    }
    for (layer_index, layer) in pane.layers.iter().enumerate() {
        let layer_path = format!("{pane_path}/layers/{layer_index}");
        match layer.mark {
            Mark::Annotation => validate_annotation(layer, &layer_path, zone, warnings)?,
            Mark::Band => validate_zone(layer, &layer_path, zone, warnings)?,
            Mark::Ohlc => crate::ohlc::validate(layer, &layer_path, context, names, warnings)?,
            Mark::Line | Mark::Area => {
                validate_layer(layer, &layer_path, context, names, warnings)?;
            }
        }
    }
    Ok(())
}

/// An annotation: a horizontal reference line at a `value`, a vertical one at a `time`, or a
/// point marker at both, always with a label. Only a point marker takes a `shape`.
fn validate_annotation(
    layer: &LayerSpec,
    path: &str,
    zone: crate::time::TimeZone,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    check_layer_color(layer, path, warnings);
    for (field, present, reason) in [
        (
            "points",
            !layer.points.is_empty(),
            "belongs to a line layer",
        ),
        ("data", !layer.data.is_empty(), "belongs to an ohlc layer"),
        ("from", layer.from.is_some(), "belongs to a band layer"),
        ("to", layer.to.is_some(), "belongs to a band layer"),
        ("top", layer.top.is_some(), "belongs to a band layer"),
        ("bottom", layer.bottom.is_some(), "belongs to a band layer"),
        ("modeled", layer.modeled, "belongs to a line layer"),
        (
            "stroke",
            layer.stroke != Stroke::Regular,
            "belongs to a line layer",
        ),
        ("dash", layer.dash.is_some(), "belongs to a line layer"),
        (
            "name",
            layer.name.is_some(),
            "is not used; an annotation is named by its label",
        ),
        (
            "shape",
            layer.shape.is_some() && !layer.is_marker(),
            "belongs to a point marker with both time and value; a reference line has no shape",
        ),
    ] {
        if present {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                format!("{field} {reason}"),
            ));
        }
    }
    if layer.time.is_none() && layer.value.is_none() {
        return Err(ChartError::new(
            "missing_position",
            path,
            "give time for a vertical reference line, value for a horizontal one, or both for a point marker",
        ));
    }
    if let Some(time) = &layer.time {
        time.resolve(zone)
            .map_err(|message| ChartError::new("invalid_time", format!("{path}/time"), message))?;
    }
    if let Some(value) = layer.value {
        validate_number(value, &format!("{path}/value"))?;
    }
    let Some(label) = &layer.label else {
        return Err(ChartError::new(
            "missing_label",
            format!("{path}/label"),
            if layer.is_marker() {
                "a point marker needs a label that says what it marks"
            } else {
                "a reference line needs a label that says what it marks"
            },
        ));
    };
    validate_text(label, &format!("{path}/label"), 100)
}

/// A zone: a shaded area between `bottom` and `top`, between `from` and `to`, or both. A missing
/// edge is the edge of the plot, so a zone with only `bottom` covers every value above it; at
/// least one edge has to be given, and the zone needs a label.
fn validate_zone(
    layer: &LayerSpec,
    path: &str,
    zone: crate::time::TimeZone,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    check_layer_color(layer, path, warnings);
    for (field, present, reason) in [
        (
            "points",
            !layer.points.is_empty(),
            "belongs to a line layer; for an uncertainty band around a line, give its points lower and upper",
        ),
        ("data", !layer.data.is_empty(), "belongs to an ohlc layer"),
        (
            "time",
            layer.time.is_some(),
            "belongs to an annotation; a zone spans from and to",
        ),
        (
            "value",
            layer.value.is_some(),
            "belongs to an annotation; a zone spans bottom and top",
        ),
        ("shape", layer.shape.is_some(), "belongs to a point marker"),
        ("modeled", layer.modeled, "belongs to a line layer"),
        (
            "stroke",
            layer.stroke != Stroke::Regular,
            "belongs to a line layer",
        ),
        ("dash", layer.dash.is_some(), "belongs to a line layer"),
        (
            "name",
            layer.name.is_some(),
            "is not used; a zone is named by its label",
        ),
    ] {
        if present {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                format!("{field} {reason}"),
            ));
        }
    }
    let resolve = |field: &str, time: Option<&TimeValue>| {
        time.map(|time| {
            time.resolve(zone).map_err(|message| {
                ChartError::new("invalid_time", format!("{path}/{field}"), message)
            })
        })
        .transpose()
    };
    let from = resolve("from", layer.from.as_ref())?;
    let to = resolve("to", layer.to.as_ref())?;
    if let Some(bottom) = layer.bottom {
        validate_number(bottom, &format!("{path}/bottom"))?;
    }
    if let Some(top) = layer.top {
        validate_number(top, &format!("{path}/top"))?;
    }
    let edges = [
        layer.bottom.is_some(),
        layer.top.is_some(),
        from.is_some(),
        to.is_some(),
    ];
    if !edges.contains(&true) {
        return Err(ChartError::new(
            "missing_position",
            path,
            "give bottom and/or top for a range of values, from and/or to for a span of time",
        ));
    }
    if let (Some(bottom), Some(top)) = (layer.bottom, layer.top)
        && bottom >= top
    {
        return Err(ChartError::new(
            "invalid_band",
            format!("{path}/top"),
            "top must lie above bottom",
        ));
    }
    if let (Some(from), Some(to)) = (from, to)
        && from >= to
    {
        return Err(ChartError::new(
            "invalid_band",
            format!("{path}/to"),
            "to must lie after from",
        ));
    }
    let Some(label) = &layer.label else {
        return Err(ChartError::new(
            "missing_label",
            format!("{path}/label"),
            "a zone needs a label that says what it marks",
        ));
    };
    validate_text(label, &format!("{path}/label"), 100)
}

/// The name of a data layer: required once the legend lists more than one layer, and unique
/// among the names it is checked against.
pub(crate) fn validate_layer_name(
    layer: &LayerSpec,
    path: &str,
    required: bool,
    names: &mut BTreeSet<String>,
) -> Result<(), ChartError> {
    match layer.name.as_deref() {
        Some(name) => {
            validate_text(name, &format!("{path}/name"), 100)?;
            if !names.insert(name.to_owned()) {
                return Err(ChartError::new(
                    "duplicate_series",
                    format!("{path}/name"),
                    "layer names must be unique within a pane, and across the panes of a time chart",
                ));
            }
        }
        None if required => {
            return Err(ChartError::new(
                "missing_name",
                format!("{path}/name"),
                "name every layer when a pane or a chart has more than one",
            ));
        }
        None => {}
    }
    Ok(())
}

fn validate_layer(
    layer: &LayerSpec,
    path: &str,
    context: &LayerContext,
    names: &mut BTreeSet<String>,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    let (zone, plot_pixels) = (context.zone, context.plot_pixels);
    let mark = if layer.mark == Mark::Area {
        "an area"
    } else {
        "a line"
    };

    check_layer_color(layer, path, warnings);

    // Fields that belong to another mark are rejected, never ignored.
    for (field, present, owner) in [
        ("data", !layer.data.is_empty(), "ohlc"),
        ("from", layer.from.is_some(), "band"),
        ("to", layer.to.is_some(), "band"),
        ("top", layer.top.is_some(), "band"),
        ("bottom", layer.bottom.is_some(), "band"),
        ("time", layer.time.is_some(), "annotation"),
        ("value", layer.value.is_some(), "annotation"),
        ("label", layer.label.is_some(), "annotation"),
        ("shape", layer.shape.is_some(), "annotation"),
    ] {
        if present {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                format!("{field} belongs to a {owner} layer, not to {mark} layer"),
            ));
        }
    }

    validate_layer_name(layer, path, context.named, names)?;

    if layer.points.is_empty() {
        return Err(ChartError::new(
            "empty_series",
            format!("{path}/points"),
            "provide at least one observation",
        ));
    }
    if layer.points.len() > MAX_TIME_POINTS_PER_LAYER {
        return Err(ChartError::new(
            "too_many_data_points",
            format!("{path}/points"),
            format!("at most {MAX_TIME_POINTS_PER_LAYER} observations are supported per layer"),
        ));
    }

    validate_points(layer, path, zone, warnings)?;

    if !layer.has_two_points(zone) {
        return Err(ChartError::new(
            "empty_series",
            format!("{path}/points"),
            "a line needs at least two observations",
        ));
    }
    // A time axis is meant to carry many observations, so the density warning starts where the
    // drawing stops resolving them instead of at a fixed count.
    if layer.points.len() > plot_pixels {
        warnings.push(ChartWarning::new(
            "dense_chart",
            format!("{path}/points"),
            format!(
                "{} observations exceed the {plot_pixels} horizontal pixels of the plot; the line cannot show every point at this width",
                layer.points.len()
            ),
        ));
    }
    Ok(())
}

fn default_timezone() -> String {
    "UTC".to_owned()
}

const fn default_height_ratio() -> u32 {
    1
}
