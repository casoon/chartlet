use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_path_to_error::Segment;

use crate::error::{ChartError, ChartWarning};
use crate::layout::plot_pixels;
use crate::time::TimeValue;

const MAX_DATA_POINTS: usize = 100;
/// Limited so that every series keeps a color that stays distinguishable for common
/// color-vision deficiencies.
pub(crate) const MAX_SERIES: usize = 4;
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
/// Warming stripes: one stripe per year; beyond this the stripes get thinner than a pixel at the
/// default width.
pub(crate) const MAX_STRIPES: usize = 500;
/// Fixed decimal places; beyond this a value stops being readable as a number.
pub(crate) const MAX_DECIMALS: u8 = 6;

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
    /// Whether the title is drawn in the chart. It always remains the accessible name and the
    /// HTML caption; a page that heads the chart itself can leave the drawing out.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub show_title: bool,
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
    /// Palette a chart is rendered in. Skipped while it is the default so that adding the field
    /// did not change the serialized form, and with it the generated accessibility IDs, of every
    /// existing specification.
    #[serde(default, skip_serializing_if = "Theme::is_light")]
    pub theme: Theme,
    /// The time axis of a `type: "time"` chart.
    #[serde(default, skip_serializing_if = "TimeAxisSpec::is_default")]
    pub time_axis: TimeAxisSpec,
    /// The panes of a `type: "time"` chart; every pane stacks one value axis over the shared
    /// time axis and holds its own layers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panes: Vec<PaneSpec>,
    /// The topics of a `type: "topicmap"` chart. Skipped while absent, for the same reason as
    /// `theme`: adding the field must not change the serialized form, and with it the generated
    /// accessibility IDs, of every existing specification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topicmap: Option<TopicMapSpec>,
    /// The landscape of a `type: "atlas"` chart. Skipped while absent, for the same reason as
    /// `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atlas: Option<AtlasSpec>,
    /// The yearly values of a `type: "stripes"` chart. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stripes: Option<StripesSpec>,
    /// The daily values of a `type: "calendar"` chart. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calendar: Option<CalendarSpec>,
    /// The spans of a `type: "rangebar"` chart, one per category.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ranges: Vec<RangeSpec>,
    /// Number of grid columns of a `type: "multiples"` chart; defaults to up to three.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u32>,
    /// Language of every text chartlet generates: description, legend additions, tooltips, the
    /// HTML figure and its data table, and the number format. Skipped while it is the default,
    /// like `theme`.
    #[serde(default, skip_serializing_if = "Locale::is_en")]
    pub locale: Locale,
    /// A second layout for narrow containers. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile: Option<MobileSpec>,
}

/// The size of the mobile variant, and the container width below which the HTML profile shows it
/// instead of the chart at `width` × `height`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MobileSpec {
    pub width: u32,
    #[serde(default = "default_mobile_height")]
    pub height: u32,
    /// Container width in CSS pixels; the mobile variant shows below it.
    #[serde(default = "default_breakpoint")]
    pub breakpoint: u32,
}

/// A diverging color scale around a reference value, shared by stripes and calendars. Values are
/// sorted into eight steps on either side of the reference; `min` and `max` set where the outermost
/// step begins, and default to the largest distance from the reference on either side.
pub(crate) struct Diverging {
    pub reference: f64,
    pub min: f64,
    pub max: f64,
}

/// Warming stripes: one colored stripe per year.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StripesSpec {
    /// The year of the first value; every further value is the following year.
    pub first_year: i32,
    /// One value per year; `null` leaves the year empty.
    pub values: Vec<Option<f64>>,
    /// The value the scale diverges from, drawn in the neutral middle color.
    #[serde(default)]
    pub reference: f64,
    /// Where the coldest step begins; defaults to the reference minus the largest distance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Where the warmest step begins; defaults to the reference plus the largest distance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Label the first and the last year under the stripes.
    #[serde(default = "default_true")]
    pub year_labels: bool,
}

/// How a calendar arranges the days of its year.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CalendarLayout {
    /// Twelve rows, one per month, and one column per day of the month.
    #[default]
    Months,
    /// Seven rows, Monday to Sunday, and one column per calendar week.
    Weeks,
}

/// One day of a calendar heatmap.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarDay {
    /// ISO 8601 date such as `2024-03-01`.
    pub date: String,
    /// `null` leaves the day empty, like an absent day.
    pub value: Option<f64>,
}

/// A calendar heatmap of one year.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarSpec {
    pub year: i32,
    #[serde(default)]
    pub layout: CalendarLayout,
    /// The days with a value, in any order; a day that is absent stays empty.
    pub days: Vec<CalendarDay>,
    #[serde(default)]
    pub reference: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// One span of a `rangebar` chart: a category with a low and a high value and, optionally, a
/// central estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RangeSpec {
    pub label: String,
    pub low: f64,
    pub high: f64,
    /// A central estimate, drawn as a mark across the bar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mid: Option<f64>,
    /// The span comes from a model rather than a measurement; it is hatched.
    #[serde(default, skip_serializing_if = "is_false")]
    pub modeled: bool,
}

impl StripesSpec {
    /// The year of every value, in order.
    pub(crate) fn years(&self) -> impl Iterator<Item = i32> + '_ {
        (0..self.values.len()).map(|offset| {
            self.first_year + i32::try_from(offset).expect("stripes are limited to 500 values")
        })
    }

    pub(crate) fn diverging(&self) -> Diverging {
        Diverging::new(
            self.reference,
            self.min,
            self.max,
            self.values.iter().flatten().copied(),
        )
    }
}

impl CalendarSpec {
    /// The days in calendar order. Validation guarantees every date is a unique ISO date of the
    /// calendar's year, so the text sorts like the date.
    pub(crate) fn sorted_days(&self) -> Vec<&CalendarDay> {
        let mut days: Vec<&CalendarDay> = self.days.iter().collect();
        days.sort_by(|a, b| a.date.cmp(&b.date));
        days
    }

    pub(crate) fn diverging(&self) -> Diverging {
        Diverging::new(
            self.reference,
            self.min,
            self.max,
            self.days.iter().filter_map(|day| day.value),
        )
    }
}

impl Diverging {
    /// Steps on each side of the reference; with the middle this makes 17 colors.
    pub(crate) const STEPS: usize = 8;

    fn new(
        reference: f64,
        min: Option<f64>,
        max: Option<f64>,
        values: impl Iterator<Item = f64>,
    ) -> Self {
        let reach = values
            .map(|value| (value - reference).abs())
            .fold(0.0, f64::max);
        // All values on the reference: any positive reach draws them in the middle color.
        let reach = if reach > 0.0 { reach } else { 1.0 };
        Self {
            reference,
            min: min.unwrap_or(reference - reach),
            max: max.unwrap_or(reference + reach),
        }
    }

    /// The color step of a value, from 0 (coldest) over [`Self::STEPS`] (the reference) to
    /// `2 × STEPS` (warmest). Each step is equally wide; values beyond `min` or `max` take the
    /// outermost step.
    pub(crate) fn step(&self, value: f64) -> usize {
        let (distance, reach) = if value >= self.reference {
            (value - self.reference, self.max - self.reference)
        } else {
            (self.reference - value, self.reference - self.min)
        };
        // `STEPS` is 8, so the product stays tiny and the cast is exact.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let arm = ((distance / reach * Self::STEPS as f64).round() as usize).min(Self::STEPS);
        if value >= self.reference {
            Self::STEPS + arm
        } else {
            Self::STEPS - arm
        }
    }
}

/// The palette a chart is rendered in. Both palettes are expressed as CSS custom properties, so a
/// host page can override any single color.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_light(&self) -> bool {
        matches!(self, Self::Light)
    }
}

/// The language of generated texts and numbers.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    #[default]
    En,
    De,
}

impl Locale {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_en(&self) -> bool {
        matches!(self, Self::En)
    }
}

/// How a value is written: format, fixed decimals if any, and locale.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct NumberStyle {
    pub format: ValueFormat,
    pub decimals: Option<u8>,
    pub locale: Locale,
}

impl From<ValueFormat> for NumberStyle {
    fn from(format: ValueFormat) -> Self {
        Self {
            format,
            ..Self::default()
        }
    }
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
}

impl TimeAxisSpec {
    fn is_default(&self) -> bool {
        self.timezone == default_timezone() && self.gaps == Gaps::Show && self.title.is_none()
    }
}

impl Default for TimeAxisSpec {
    fn default() -> Self {
        Self {
            timezone: default_timezone(),
            gaps: Gaps::default(),
            title: None,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    Bar,
    Line,
    /// A time series on a shared time axis, drawn from panes and layers.
    Time,
    /// Topic areas as a landmass map; area proportional to each topic's value.
    Topicmap,
    /// A knowledge landscape: realms of adjacent regions, holding places. Where `topicmap` packs
    /// separate landmasses and lets size carry the whole statement, an `atlas` tiles one
    /// continuous land, so that *where* something lies says as much as how large it is.
    Atlas,
    /// Warming stripes: one colored stripe per year on a diverging scale.
    Stripes,
    /// A calendar heatmap: one cell per day of a year on a diverging scale.
    Calendar,
    /// Spans with a low and a high value per category, optionally with a central estimate.
    Rangebar,
    /// Small multiples: several small time charts in a grid, sharing both axes.
    Multiples,
}

/// One topic (or island) of a `topicmap` chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TopicSpec {
    pub label: String,
    pub value: f64,
    /// Number of paths through this topic; drawn as points inside its area.
    #[serde(default)]
    pub points: u32,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// A neighborhood between two topics, drawn as a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TopicLinkSpec {
    /// Label of the topic (or island) the route starts at.
    pub from: String,
    /// Label of the topic (or island) the route ends at.
    pub to: String,
    #[serde(default = "one")]
    pub weight: f64,
}

/// Which corner of the canvas a cartouche is anchored to.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Corner {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

/// The legend box printed on the map: heading, metadata line, and its corner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CartoucheSpec {
    pub heading: String,
    pub meta: String,
    #[serde(default)]
    pub corner: Corner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TopicMapSpec {
    pub topics: Vec<TopicSpec>,
    #[serde(default)]
    pub links: Vec<TopicLinkSpec>,
    /// Fringe topics, drawn as small fixed-size islands rather than area-proportional landmasses.
    #[serde(default)]
    pub islands: Vec<TopicSpec>,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub cartouche: Option<CartoucheSpec>,
    #[serde(default = "default_true")]
    pub graticule: bool,
    #[serde(default = "default_true")]
    pub compass: bool,
    /// Number of depth-line rings drawn around each coastline, 0 to 3.
    #[serde(default = "default_depth_bands")]
    pub depth_bands: u8,
}

/// One place on an `atlas` map: a single entry, drawn as a point inside its region.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceSpec {
    pub label: String,
    /// How prominent the point is. 1 is an ordinary place; larger stands out, which is how a
    /// map shows what matters without needing a second color.
    #[serde(default = "one")]
    pub weight: f64,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// One region of an `atlas` map: a named area inside a realm, holding places.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegionSpec {
    pub label: String,
    /// How much the region holds. It drives the region's share of the land, damped by
    /// `areaDamping` so that the largest one does not swallow the map.
    pub value: f64,
    #[serde(default)]
    pub places: Vec<PlaceSpec>,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// One realm of an `atlas` map: the regions that belong together and are drawn as one land.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RealmSpec {
    pub label: String,
    pub regions: Vec<RegionSpec>,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// The landscape of an `atlas` chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AtlasSpec {
    pub realms: Vec<RealmSpec>,
    /// Kinship between two regions, named by label and valid across realm borders. These are the
    /// edges that pull a region towards the edge of its own realm, so that something which
    /// mediates between two subjects ends up lying between them.
    #[serde(default)]
    pub links: Vec<TopicLinkSpec>,
    #[serde(default)]
    pub seed: u64,
    /// The exponent a region's area follows: 1 is proportional to its value, 0.5 its square root.
    #[serde(default = "default_area_damping")]
    pub area_damping: f64,
    /// Contour lines drawn from the density of places, so that the terrain carries the quantity
    /// statement the area no longer has to.
    #[serde(default = "default_true")]
    pub contours: bool,
}

const fn default_area_damping() -> f64 {
    0.5
}

/// Bounds on an `atlas`, set by what stays legible rather than by anything technical.
const MAX_REALMS: usize = 8;
const MAX_REGIONS: usize = 60;
const MAX_PLACES: usize = 4000;

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
    /// Fixed number of decimal places for values in labels, tooltips, the description and the
    /// data table. Axis ticks take theirs from the tick step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
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

/// A pre-computed zoom step: shows categories `from..=to`, or on a time chart the observations
/// from `from` to `to`, as its own chart variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ZoomStep {
    pub label: String,
    pub from: ZoomBound,
    pub to: ZoomBound,
}

/// One end of a zoom step: a category index on a bar or line chart, a timestamp on a time chart.
/// A whole number from 0 up reads as an index first, so that a category chart keeps its
/// serialized form; on a time chart the same number is Unix seconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ZoomBound {
    Index(usize),
    Time(TimeValue),
}

impl ZoomBound {
    /// The category index, `None` for a timestamp that is not a whole number from 0.
    pub(crate) const fn index(&self) -> Option<usize> {
        match self {
            Self::Index(index) => Some(*index),
            Self::Time(_) => None,
        }
    }

    /// The bound as Unix seconds, in the forms a point's `time` accepts.
    pub(crate) fn resolve(&self, zone: crate::time::TimeZone) -> Result<i64, &'static str> {
        match self {
            Self::Index(seconds) => {
                // Anything beyond the 1700–2200 contract is refused by `resolve` anyway.
                #[allow(clippy::cast_precision_loss)]
                let seconds = *seconds as f64;
                TimeValue::Number(seconds).resolve(zone)
            }
            Self::Time(time) => time.resolve(zone),
        }
    }
}

/// A layer together with where it sits in the specification.
#[derive(Clone, Copy)]
pub(crate) struct LayerRef<'a> {
    /// Index among all layers of all panes; keys a declared color and a hatch pattern.
    pub global: usize,
    pub pane: usize,
    /// Index within its pane, as the specification path names it.
    pub local: usize,
    pub layer: &'a LayerSpec,
}

/// Categories × series: the single shape that layout, description and data table work with.
pub(crate) struct Dataset {
    pub categories: Vec<String>,
    pub series: Vec<Series>,
}

pub(crate) struct Series {
    /// `None` when the chart was given as a single `data` list.
    pub name: Option<String>,
    pub values: Vec<Option<f64>>,
    /// How this column is written when it is not written like the chart's values: a topic map's
    /// share column is a percentage while the counts beside it are plain numbers, and every pane
    /// of a time chart has its own value axis.
    pub style: Option<NumberStyle>,
}

impl Dataset {
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
        self.validate_sized(true)
    }

    /// The chart laid out at the size of its mobile variant, or `None` without one.
    pub(crate) fn mobile_variant(&self) -> Option<ChartSpec> {
        let mobile = self.mobile.as_ref()?;
        let mut spec = self.clone();
        spec.width = mobile.width;
        spec.height = mobile.height;
        spec.mobile = None;
        Some(spec)
    }

    /// Validates a mobile variant. Its size was already checked against the limits of `mobile`,
    /// which admit narrower charts than `width`; the checks that depend on the size, such as
    /// observations per plot pixel, run at that size.
    pub(crate) fn validate_mobile_variant(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.validate_sized(false)
    }

    fn validate_sized(&self, check_size: bool) -> Result<Vec<ChartWarning>, ChartError> {
        self.validate_metadata(check_size)?;
        self.reject_foreign_blocks()?;
        match self.chart_type {
            ChartType::Time => return self.validate_time(),
            ChartType::Multiples => return self.validate_multiples(),
            ChartType::Topicmap => return self.validate_topicmap(),
            ChartType::Atlas => return self.validate_atlas(),
            ChartType::Stripes => return self.validate_stripes(),
            ChartType::Calendar => return self.validate_calendar(),
            ChartType::Rangebar => return self.validate_rangebar(),
            ChartType::Bar | ChartType::Line => {}
        }
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

    /// Fixed decimals stay in a readable range.
    fn validate_decimals(&self) -> Result<(), ChartError> {
        let axes = std::iter::once(("/valueAxis/decimals".to_owned(), &self.value_axis)).chain(
            self.panes.iter().enumerate().map(|(index, pane)| {
                (
                    format!("/panes/{index}/valueAxis/decimals"),
                    &pane.value_axis,
                )
            }),
        );
        for (path, axis) in axes {
            if axis
                .decimals
                .is_some_and(|decimals| decimals > MAX_DECIMALS)
            {
                return Err(ChartError::new(
                    "invalid_decimals",
                    path,
                    format!("use 0 to {MAX_DECIMALS} decimal places"),
                ));
            }
        }
        Ok(())
    }

    /// The size of the chart and of its mobile variant.
    fn validate_size(&self) -> Result<(), ChartError> {
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
        if let Some(mobile) = &self.mobile {
            for (path, value, range) in [
                ("/mobile/width", mobile.width, 280..=600),
                ("/mobile/height", mobile.height, 240..=1_600),
                ("/mobile/breakpoint", mobile.breakpoint, 320..=1_600),
            ] {
                if !range.contains(&value) {
                    return Err(ChartError::new(
                        "invalid_dimension",
                        path,
                        format!(
                            "{} must be between {} and {}",
                            &path[1..],
                            range.start(),
                            range.end()
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_metadata(&self, check_size: bool) -> Result<(), ChartError> {
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
        self.validate_decimals()?;
        if !self.show_title && self.chart_type != ChartType::Time {
            return Err(ChartError::new(
                "option_not_supported",
                "/showTitle",
                "leaving the drawn title out is available for time charts so far",
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

        if check_size {
            self.validate_size()?;
        }
        // Zoom steps slice `data` or `categories`, which only bar and line charts have; every
        // other type rejects the field by name.
        if !matches!(self.chart_type, ChartType::Bar | ChartType::Line) {
            return Ok(());
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
            let index = |field: &str, bound: &ZoomBound| {
                bound.index().ok_or_else(|| {
                    ChartError::new(
                        "invalid_spec",
                        format!("/zoomSteps/{i}/{field}"),
                        "expected a category index, a whole number from 0",
                    )
                })
            };
            let from = index("from", &step.from)?;
            let to = index("to", &step.to)?;
            if from > to {
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
            if to >= count {
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

    pub(crate) fn dataset(&self) -> Dataset {
        if self.series.is_empty() {
            Dataset {
                categories: self.data.iter().map(|point| point.label.clone()).collect(),
                series: vec![Series {
                    name: None,
                    values: self.data.iter().map(|point| point.value).collect(),
                    style: None,
                }],
            }
        } else {
            Dataset {
                categories: self.categories.clone(),
                series: self
                    .series
                    .iter()
                    .map(|series| Series {
                        name: Some(series.name.clone()),
                        values: series.values.clone(),
                        style: None,
                    })
                    .collect(),
            }
        }
    }

    /// The dataset behind the data table: a time chart reports one row per timestamp, a topicmap
    /// chart one row per topic and island, an atlas one row per region, every other chart one row
    /// per category.
    pub(crate) fn table_dataset(&self) -> Dataset {
        match self.chart_type {
            ChartType::Time | ChartType::Multiples => {
                let zone = self.time_zone().unwrap_or_default();
                self.time_dataset(zone, self.time_precision(zone), true)
            }
            ChartType::Topicmap => self.topicmap_dataset(),
            ChartType::Atlas => self.atlas_dataset(),
            ChartType::Stripes => self.stripes_dataset(),
            ChartType::Calendar => self.calendar_dataset(),
            ChartType::Rangebar => self.rangebar_dataset(),
            ChartType::Bar | ChartType::Line => self.dataset(),
        }
    }

    /// One row per year, `Missing` where the year has no value.
    fn stripes_dataset(&self) -> Dataset {
        let stripes = self
            .stripes
            .as_ref()
            .expect("validated stripes charts carry a stripes block");
        Dataset {
            categories: stripes.years().map(|year| year.to_string()).collect(),
            series: vec![Series {
                name: None,
                values: stripes.values.clone(),
                style: None,
            }],
        }
    }

    /// One row per day that has an entry, in calendar order.
    fn calendar_dataset(&self) -> Dataset {
        let calendar = self
            .calendar
            .as_ref()
            .expect("validated calendar charts carry a calendar block");
        let days = calendar.sorted_days();
        Dataset {
            categories: days.iter().map(|day| day.date.clone()).collect(),
            series: vec![Series {
                name: None,
                values: days.iter().map(|day| day.value).collect(),
                style: None,
            }],
        }
    }

    /// One row per range with its low, high and, where any range has one, its central value.
    /// A modeled range says so in its row label, since the table cannot show the hatching.
    fn rangebar_dataset(&self) -> Dataset {
        let words = self.locale.words();
        let mut series = vec![
            Series {
                name: Some(words.range_low.to_owned()),
                values: self.ranges.iter().map(|range| Some(range.low)).collect(),
                style: None,
            },
            Series {
                name: Some(words.range_high.to_owned()),
                values: self.ranges.iter().map(|range| Some(range.high)).collect(),
                style: None,
            },
        ];
        if self.ranges.iter().any(|range| range.mid.is_some()) {
            series.insert(
                1,
                Series {
                    name: Some(words.range_mid.to_owned()),
                    values: self.ranges.iter().map(|range| range.mid).collect(),
                    style: None,
                },
            );
        }
        Dataset {
            categories: self
                .ranges
                .iter()
                .map(|range| {
                    if range.modeled {
                        format!("{} ({})", range.label, words.modeled)
                    } else {
                        range.label.clone()
                    }
                })
                .collect(),
            series,
        }
    }

    /// Topics and islands as rows, with their value and path count as columns.
    /// The table behind an atlas: one row per region, naming the realm it lies in. The realms
    /// themselves get no row — they hold no number of their own, only the sum of their regions,
    /// and a row repeating that sum would invite reading it as a sixth region.
    fn atlas_dataset(&self) -> Dataset {
        let atlas = self
            .atlas
            .as_ref()
            .expect("validated atlas charts carry an atlas block");
        let regions = || {
            atlas
                .realms
                .iter()
                .flat_map(|realm| realm.regions.iter().map(move |region| (realm, region)))
        };
        Dataset {
            categories: regions()
                .map(|(realm, region)| format!("{} · {}", realm.label, region.label))
                .collect(),
            series: vec![
                Series {
                    name: Some(self.locale.words().entries.to_owned()),
                    values: regions().map(|(_, region)| Some(region.value)).collect(),
                    style: None,
                },
                Series {
                    name: Some(self.locale.words().places.to_owned()),
                    values: regions()
                        .map(|(_, region)| Some(place_count(region.places.len())))
                        .collect(),
                    style: None,
                },
            ],
        }
    }

    fn topicmap_dataset(&self) -> Dataset {
        let topicmap = self
            .topicmap
            .as_ref()
            .expect("validated topicmap charts carry a topicmap block");
        let all = || topicmap.topics.iter().chain(topicmap.islands.iter());
        let total: f64 = all().map(|topic| topic.value).sum();
        Dataset {
            categories: all().map(|topic| topic.label.clone()).collect(),
            series: vec![
                Series {
                    name: Some(self.locale.words().entries.to_owned()),
                    values: all().map(|topic| Some(topic.value)).collect(),
                    style: None,
                },
                Series {
                    name: Some(self.locale.words().paths.to_owned()),
                    values: all().map(|topic| Some(f64::from(topic.points))).collect(),
                    style: None,
                },
                Series {
                    name: Some(self.locale.words().share.to_owned()),
                    // Of every entry on the map, islands included: the table lists them too.
                    // Rounded to two decimal places of a percent — a share carried to twelve
                    // digits would claim a precision the underlying counts do not have.
                    values: all()
                        .map(|topic| Some(((topic.value / total) * 10_000.0).round() / 10_000.0))
                        .collect(),
                    style: Some(NumberStyle {
                        format: ValueFormat::Percent,
                        decimals: None,
                        locale: self.locale,
                    }),
                },
            ],
        }
    }

    /// The time axis as configured, or its default when the specification omitted it.
    pub(crate) fn time_zone(&self) -> Result<crate::time::TimeZone, ChartError> {
        crate::time::TimeZone::parse(&self.time_axis.timezone).ok_or_else(|| {
            ChartError::new(
                "invalid_timezone",
                "/timeAxis/timezone",
                "expected UTC or a fixed offset such as +02:00",
            )
        })
    }

    /// Every layer of every pane, in the order they are drawn.
    pub(crate) fn layers(&self) -> impl Iterator<Item = &LayerSpec> {
        self.panes.iter().flat_map(|pane| pane.layers.iter())
    }

    /// Every layer with its index among all layers (which keys its declared color), the index of
    /// its pane and its index within that pane (which its specification path uses).
    pub(crate) fn indexed_layers(&self) -> impl Iterator<Item = LayerRef<'_>> {
        self.panes
            .iter()
            .enumerate()
            .flat_map(|(pane, spec)| {
                spec.layers
                    .iter()
                    .enumerate()
                    .map(move |(local, layer)| (pane, local, layer))
            })
            .enumerate()
            .map(|(global, (pane, local, layer))| LayerRef {
                global,
                pane,
                local,
                layer,
            })
    }

    /// Every layer that draws data, see [`Self::indexed_layers`].
    pub(crate) fn data_layers(&self) -> impl Iterator<Item = LayerRef<'_>> {
        self.indexed_layers().filter(|entry| entry.layer.is_data())
    }

    /// With `gaps: "collapse"`, the slots of the time axis in order: every distinct timestamp an
    /// observation of a data layer uses in any pane, a missing value and a candle included. Each
    /// slot takes the same share of the axis. `None` while the axis keeps the distances in time.
    pub(crate) fn time_slots(&self, zone: crate::time::TimeZone) -> Option<Vec<i64>> {
        (self.time_axis.gaps == Gaps::Collapse).then(|| {
            let mut slots: Vec<i64> = self
                .data_layers()
                .flat_map(|entry| entry.layer.resolved_times(zone))
                .collect();
            slots.sort_unstable();
            slots.dedup();
            slots
        })
    }

    /// The names of the data layers, once each in order of first appearance. In small multiples
    /// a name is one series across every panel, so it keeps one color and one legend entry.
    pub(crate) fn series_names(&self) -> Vec<Option<String>> {
        let mut names: Vec<Option<String>> = Vec::new();
        for entry in self.data_layers() {
            if !names.contains(&entry.layer.name) {
                names.push(entry.layer.name.clone());
            }
        }
        names
    }

    /// Which palette color a data layer takes when it declares none. In a time chart that is its
    /// position among the chart's data layers other than candles, across all panes, so that no two
    /// entries of the one legend share a color; a layer from the fifth position on takes the first
    /// palette color no layer before it holds, which validation guarantees exists because at most
    /// [`MAX_SERIES`] layers go without a color of their own. In small multiples it is the
    /// position of its name among all names.
    pub(crate) fn palette_index(&self, layer: &LayerSpec) -> usize {
        let index = if self.chart_type == ChartType::Multiples {
            self.series_names()
                .iter()
                .position(|name| *name == layer.name)
                .unwrap_or(0)
        } else {
            let data = || {
                self.layers()
                    .filter(|candidate| candidate.is_data() && candidate.mark != Mark::Ohlc)
                    .enumerate()
            };
            let mut taken = [false; MAX_SERIES];
            for (position, candidate) in data() {
                if position < MAX_SERIES && candidate.color.is_none() {
                    taken[position] = true;
                }
            }
            let mut index = 0;
            for (position, candidate) in data() {
                let slot = if position < MAX_SERIES {
                    position
                } else if candidate.color.is_none() {
                    let free = taken.iter().position(|taken| !taken).unwrap_or(0);
                    taken[free] = true;
                    free
                } else {
                    0
                };
                if std::ptr::eq(candidate, layer) {
                    index = slot;
                    break;
                }
            }
            index
        };
        index.min(MAX_SERIES - 1)
    }

    /// The precision of every label that names an observation, chosen from the data layers.
    pub(crate) fn time_precision(&self, zone: crate::time::TimeZone) -> crate::time::Precision {
        let epochs: Vec<i64> = self
            .data_layers()
            .flat_map(|entry| entry.layer.resolved_times(zone))
            .collect();
        crate::time::Precision::of(epochs.into_iter(), zone)
    }

    /// How values are written: the value format, the fixed decimals of the axis that applies, and
    /// the locale.
    pub(crate) fn number_style(&self) -> NumberStyle {
        let decimals = match self.chart_type {
            ChartType::Time => self.panes.first().and_then(|pane| pane.value_axis.decimals),
            ChartType::Bar | ChartType::Line | ChartType::Rangebar | ChartType::Multiples => {
                self.value_axis.decimals
            }
            ChartType::Topicmap | ChartType::Atlas | ChartType::Stripes | ChartType::Calendar => {
                None
            }
        };
        NumberStyle {
            format: self.value_format(),
            decimals,
            locale: self.locale,
        }
    }

    /// How the values of one pane are written: a time chart's pane has its own value axis, every
    /// other chart writes all values alike.
    pub(crate) fn pane_style(&self, pane_index: usize) -> NumberStyle {
        match self.chart_type {
            ChartType::Time => NumberStyle {
                format: self.panes[pane_index].value_axis.format,
                decimals: self.panes[pane_index].value_axis.decimals,
                locale: self.locale,
            },
            _ => self.number_style(),
        }
    }

    /// The value format that applies to the chart: the pane's format for a time chart, otherwise
    /// the single top-level value axis.
    pub(crate) fn value_format(&self) -> ValueFormat {
        match self.chart_type {
            ChartType::Time => self
                .panes
                .first()
                .map_or(ValueFormat::Number, |pane| pane.value_axis.format),
            ChartType::Topicmap | ChartType::Atlas | ChartType::Stripes | ChartType::Calendar => {
                ValueFormat::Number
            }
            ChartType::Bar | ChartType::Line | ChartType::Rangebar | ChartType::Multiples => {
                self.value_axis.format
            }
        }
    }

    /// Timestamps × layers: one row per timestamp that any data layer uses, one column per data
    /// layer, and `None` where a layer has no observation or a `null` value at that timestamp.
    /// With `bounds`, a layer with a band adds a column for its lower and one for its upper edge.
    ///
    /// Timestamps and layer values are owned because the row labels are formatted timestamps
    /// rather than text taken from the specification.
    pub(crate) fn time_dataset(
        &self,
        zone: crate::time::TimeZone,
        precision: crate::time::Precision,
        bounds: bool,
    ) -> Dataset {
        let mut categories: Vec<i64> = self
            .data_layers()
            .flat_map(|entry| entry.layer.resolved_times(zone))
            .collect();
        categories.sort_unstable();
        categories.dedup();

        let column = |pairs: Vec<(i64, f64)>| -> Vec<Option<f64>> {
            categories
                .iter()
                .map(|epoch| {
                    pairs
                        .iter()
                        .find_map(|(point, value)| (point == epoch).then_some(*value))
                })
                .collect()
        };

        // A part of a layer, such as a band edge or a candle's open, is named after the layer.
        let part_name = |name: &Option<String>, part: &str| {
            if let Some(name) = name {
                format!("{name} ({part})")
            } else {
                let mut chars = part.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().chain(chars).collect())
                    .unwrap_or_default()
            }
        };
        let words = self.locale.words();
        let mut series = Vec::new();
        for (entry, name) in self.data_layers().zip(self.layer_names()) {
            let layer = entry.layer;
            // Every pane of a time chart writes its values by its own value axis.
            let style = (self.chart_type == ChartType::Time).then(|| self.pane_style(entry.pane));
            if layer.mark == Mark::Ohlc {
                let candles = layer.resolved_candles(zone);
                for (index, part) in [words.open, words.high, words.low, words.close]
                    .into_iter()
                    .enumerate()
                {
                    series.push(Series {
                        name: Some(part_name(&name, part)),
                        values: column(
                            candles
                                .iter()
                                .map(|(epoch, values)| (*epoch, values[index]))
                                .collect(),
                        ),
                        style,
                    });
                }
                continue;
            }
            series.push(Series {
                name: name.clone(),
                values: column(layer.resolved_points(zone)),
                style,
            });
            if bounds && layer.has_band() {
                let band = layer.resolved_band(zone);
                for (edge, pick) in [(words.lower, 0), (words.upper, 1)] {
                    series.push(Series {
                        name: Some(part_name(&name, edge)),
                        values: column(
                            band.iter()
                                .map(|(epoch, lower, upper)| {
                                    (*epoch, if pick == 0 { *lower } else { *upper })
                                })
                                .collect(),
                        ),
                        style,
                    });
                }
            }
        }

        Dataset {
            categories: categories
                .iter()
                .map(|epoch| precision.format(*epoch, zone))
                .collect(),
            series,
        }
    }

    /// The name of every data layer, defaulting to `Value` for a single unnamed layer. In small
    /// multiples the panel title comes first, so that a column names both.
    pub(crate) fn layer_names(&self) -> impl Iterator<Item = Option<String>> + '_ {
        let named = self.data_layers().count() > 1;
        self.data_layers().map(move |entry| {
            let (pane_index, layer) = (entry.pane, entry.layer);
            let name = layer
                .name
                .clone()
                .or_else(|| named.then(|| self.locale.words().value.to_owned()));
            match (&self.panes[pane_index].title, name) {
                (Some(title), Some(name)) if self.chart_type == ChartType::Multiples => {
                    Some(if layer.name.is_some() {
                        format!("{title} · {name}")
                    } else {
                        title.clone()
                    })
                }
                (_, name) => name,
            }
        })
    }

    /// Returns a copy of this time chart restricted to the observations from `from` to `to`, both
    /// included. Every data layer stays, even when the window leaves it empty, so that colors and
    /// the legend match the full chart; a vertical reference line outside the window is left out.
    pub(crate) fn windowed(&self, from: i64, to: i64) -> ChartSpec {
        let zone = self.time_zone().unwrap_or_default();
        let inside = |time: &TimeValue| {
            time.resolve(zone)
                .is_ok_and(|epoch| (from..=to).contains(&epoch))
        };
        let mut spec = self.clone();
        for pane in &mut spec.panes {
            pane.layers
                .retain(|layer| layer.time.as_ref().is_none_or(&inside));
            // A zone that misses the window is left out; one that reaches beyond it is cut at
            // the window's edges, so it cannot widen the time axis.
            pane.layers.retain(|layer| {
                let resolve = |time: &Option<TimeValue>| {
                    time.as_ref().and_then(|time| time.resolve(zone).ok())
                };
                layer.mark != Mark::Band
                    || (resolve(&layer.from).is_none_or(|start| start < to)
                        && resolve(&layer.to).is_none_or(|end| end > from))
            });
            for layer in &mut pane.layers {
                layer.points.retain(|point| inside(&point.time));
                layer.data.retain(|candle| inside(&candle.time));
                if layer.mark == Mark::Band {
                    #[allow(clippy::cast_precision_loss)]
                    let clamp = |time: &mut Option<TimeValue>| {
                        if let Some(epoch) = time.as_ref().and_then(|time| time.resolve(zone).ok())
                            && !(from..=to).contains(&epoch)
                        {
                            *time = Some(TimeValue::Number(epoch.clamp(from, to) as f64));
                        }
                    };
                    clamp(&mut layer.from);
                    clamp(&mut layer.to);
                }
            }
        }
        // With gaps collapsed, an annotation needs an observation of the window to stand on: a
        // reference line or marker the first one at or after its time, a zone one between its
        // edges.
        if let Some(slots) = spec.time_slots(zone) {
            let resolve =
                |time: &Option<TimeValue>| time.as_ref().and_then(|time| time.resolve(zone).ok());
            for pane in &mut spec.panes {
                pane.layers.retain(|layer| match layer.mark {
                    Mark::Annotation => {
                        resolve(&layer.time).is_none_or(|time| slots.last() >= Some(&time))
                    }
                    Mark::Band if layer.from.is_some() || layer.to.is_some() => {
                        let start = resolve(&layer.from).unwrap_or(i64::MIN);
                        let end = resolve(&layer.to).unwrap_or(i64::MAX);
                        slots.iter().any(|slot| (start..=end).contains(slot))
                    }
                    _ => true,
                });
            }
        }
        spec
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

    /// Rejects the fields that only a bar or line chart has, naming the field and the fix. Small
    /// multiples keep the top-level value axis, which all their panels share.
    fn reject_bar_and_line_options(&self, noun: &str) -> Result<(), ChartError> {
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
        Ok(())
    }

    /// Rejects a block that belongs to another chart type, so that it is never silently ignored.
    fn reject_foreign_blocks(&self) -> Result<(), ChartError> {
        let own = self.chart_type;
        for (field, present, owner) in [
            ("/stripes", self.stripes.is_some(), ChartType::Stripes),
            ("/calendar", self.calendar.is_some(), ChartType::Calendar),
            ("/ranges", !self.ranges.is_empty(), ChartType::Rangebar),
            ("/columns", self.columns.is_some(), ChartType::Multiples),
        ] {
            if present && own != owner {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    format!(
                        "{field} belongs to a {} chart; remove it or change the type",
                        type_name(owner)
                    ),
                ));
            }
        }
        if matches!(
            own,
            ChartType::Stripes | ChartType::Calendar | ChartType::Rangebar | ChartType::Multiples
        ) {
            for (field, present, owner) in [
                ("/topicmap", self.topicmap.is_some(), ChartType::Topicmap),
                ("/atlas", self.atlas.is_some(), ChartType::Atlas),
            ] {
                if present {
                    return Err(ChartError::new(
                        "option_not_supported",
                        field,
                        format!(
                            "{field} belongs to a {} chart; remove it or change the type",
                            type_name(owner)
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    /// The time axis options both time charts and small multiples share.
    fn validate_time_axis(&self) -> Result<crate::time::TimeZone, ChartError> {
        let zone = self.time_zone()?;
        validate_optional_text(self.time_axis.title.as_ref(), "/timeAxis/title", 100)?;
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

    fn validate_time(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
    fn validate_multiples(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
            if pane.value_axis.title.is_some()
                || pane.value_axis.format != ValueFormat::Number
                || pane.value_axis.decimals.is_some()
            {
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

    /// Warming stripes: a block of consecutive yearly values and a diverging scale.
    fn validate_stripes(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a stripes chart", "stripes.values")?;
        let Some(stripes) = &self.stripes else {
            return Err(ChartError::new(
                "missing_stripes",
                "/stripes",
                "a stripes chart requires a stripes block",
            ));
        };
        if stripes.values.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/stripes/values",
                "provide at least one value",
            ));
        }
        if stripes.values.len() > MAX_STRIPES {
            return Err(ChartError::new(
                "too_many_data_points",
                "/stripes/values",
                format!("at most {MAX_STRIPES} yearly values are supported"),
            ));
        }
        let last_year = i64::from(stripes.first_year) + count_i64(stripes.values.len()) - 1;
        if stripes.first_year < 1 || last_year > 9_999 {
            return Err(ChartError::new(
                "invalid_year",
                "/stripes/firstYear",
                "the years must lie between 1 and 9999",
            ));
        }
        for (index, value) in stripes.values.iter().enumerate() {
            if let Some(value) = value {
                validate_number(*value, &format!("/stripes/values/{index}"))?;
            }
        }
        if stripes.values.iter().all(Option::is_none) {
            return Err(ChartError::new(
                "empty_series",
                "/stripes/values",
                "provide at least one numeric value",
            ));
        }
        validate_diverging(stripes.reference, stripes.min, stripes.max, "/stripes")?;
        Ok(Vec::new())
    }

    /// A calendar heatmap: dated values of one year and a diverging scale.
    fn validate_calendar(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a calendar chart", "calendar.days")?;
        let Some(calendar) = &self.calendar else {
            return Err(ChartError::new(
                "missing_calendar",
                "/calendar",
                "a calendar chart requires a calendar block",
            ));
        };
        if !(1_700..=2_199).contains(&calendar.year) {
            return Err(ChartError::new(
                "invalid_year",
                "/calendar/year",
                "year must lie between 1700 and 2199",
            ));
        }
        if calendar.days.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/calendar/days",
                "provide at least one day",
            ));
        }
        let mut dates = BTreeSet::new();
        for (index, day) in calendar.days.iter().enumerate() {
            let path = format!("/calendar/days/{index}");
            let Some((year, _, _)) = calendar_date(&day.date) else {
                return Err(ChartError::new(
                    "invalid_date",
                    format!("{path}/date"),
                    "expected an ISO 8601 date such as 2024-03-01",
                ));
            };
            if year != i64::from(calendar.year) {
                return Err(ChartError::new(
                    "date_outside_year",
                    format!("{path}/date"),
                    format!("the date must lie in {}", calendar.year),
                ));
            }
            if !dates.insert(day.date.as_str()) {
                return Err(ChartError::new(
                    "duplicate_date",
                    format!("{path}/date"),
                    "every day may appear only once",
                ));
            }
            if let Some(value) = day.value {
                validate_number(value, &format!("{path}/value"))?;
            }
        }
        if calendar.days.iter().all(|day| day.value.is_none()) {
            return Err(ChartError::new(
                "empty_series",
                "/calendar/days",
                "provide at least one numeric value",
            ));
        }
        validate_diverging(calendar.reference, calendar.min, calendar.max, "/calendar")?;
        Ok(Vec::new())
    }

    /// Spans per category: a low no higher than its high, and a central value between them.
    fn validate_rangebar(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
                    "a rangebar chart is drawn from ranges; remove this field",
                ));
            }
        }
        if self.ranges.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/ranges",
                "provide at least one range",
            ));
        }
        if self.ranges.len() > MAX_DATA_POINTS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/ranges",
                format!("at most {MAX_DATA_POINTS} ranges are supported"),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, range) in self.ranges.iter().enumerate() {
            let path = format!("/ranges/{index}");
            validate_text(&range.label, &format!("{path}/label"), 200)?;
            if !labels.insert(range.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "range labels must be unique",
                ));
            }
            validate_number(range.low, &format!("{path}/low"))?;
            validate_number(range.high, &format!("{path}/high"))?;
            if range.low > range.high {
                return Err(ChartError::new(
                    "invalid_range",
                    format!("{path}/high"),
                    "high must not be below low",
                ));
            }
            if let Some(mid) = range.mid {
                validate_number(mid, &format!("{path}/mid"))?;
                if !(range.low..=range.high).contains(&mid) {
                    return Err(ChartError::new(
                        "mid_outside_range",
                        format!("{path}/mid"),
                        "mid must lie between low and high",
                    ));
                }
            }
        }
        let mut warnings = Vec::new();
        if self.ranges.len() > 16 {
            warnings.push(ChartWarning::new(
                "dense_chart",
                "/ranges",
                "more than 16 categories can be difficult to read at the configured size",
            ));
        }
        Ok(warnings)
    }

    /// Rejects the fields that only a bar, line, or time chart has, naming the field and the fix.
    /// Neither map has axes, series or panes. `noun` and `source` name the type in the message,
    /// so the error says what to do rather than only what is wrong.
    fn reject_map_options(&self, noun: &str, source: &str) -> Result<(), ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    format!("{noun} is drawn from {source}; remove this field"),
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
                format!("{noun} has no category axis"),
            ));
        }
        if self.value_axis.title.is_some() || self.value_axis.format != ValueFormat::Number {
            return Err(ChartError::new(
                "option_not_supported",
                "/valueAxis",
                format!("{noun} has no value axis"),
            ));
        }
        if !self.time_axis.is_default() {
            return Err(ChartError::new(
                "option_not_supported",
                "/timeAxis",
                format!("{noun} has no time axis"),
            ));
        }
        Ok(())
    }

    fn validate_topicmap(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a topicmap chart", "topicmap.topics")?;
        let Some(topicmap) = &self.topicmap else {
            return Err(ChartError::new(
                "missing_topicmap",
                "/topicmap",
                "a topicmap chart requires a topicmap block",
            ));
        };
        if topicmap.topics.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/topicmap/topics",
                "provide at least one topic",
            ));
        }
        if topicmap.topics.len() > 40 {
            return Err(ChartError::new(
                "too_many_topics",
                "/topicmap/topics",
                "at most 40 topics are supported",
            ));
        }
        if topicmap.islands.len() > 6 {
            return Err(ChartError::new(
                "too_many_islands",
                "/topicmap/islands",
                "at most 6 islands are supported",
            ));
        }
        if !(0..=3).contains(&topicmap.depth_bands) {
            return Err(ChartError::new(
                "invalid_depth_bands",
                "/topicmap/depthBands",
                "depthBands must be between 0 and 3",
            ));
        }

        let mut labels = BTreeSet::new();
        let mut warnings = Vec::new();
        for (index, topic) in topicmap.topics.iter().enumerate() {
            validate_topic(
                topic,
                &format!("/topicmap/topics/{index}"),
                true,
                &mut labels,
                &mut warnings,
            )?;
        }
        for (index, island) in topicmap.islands.iter().enumerate() {
            // An island is already drawn at a small fixed size, so the label-fit warning that
            // steers an oversized topic towards becoming one would be meaningless here.
            validate_topic(
                island,
                &format!("/topicmap/islands/{index}"),
                false,
                &mut labels,
                &mut warnings,
            )?;
        }
        for (index, link) in topicmap.links.iter().enumerate() {
            let path = format!("/topicmap/links/{index}");
            if !labels.contains(link.from.as_str()) {
                return Err(ChartError::new(
                    "unknown_topic_link",
                    format!("{path}/from"),
                    format!("{:?} is not a declared topic or island label", link.from),
                ));
            }
            if !labels.contains(link.to.as_str()) {
                return Err(ChartError::new(
                    "unknown_topic_link",
                    format!("{path}/to"),
                    format!("{:?} is not a declared topic or island label", link.to),
                ));
            }
            if !(0.0..=1.0).contains(&link.weight) {
                return Err(ChartError::new(
                    "invalid_link_weight",
                    format!("{path}/weight"),
                    "weight must be between 0 and 1",
                ));
            }
        }
        if let Some(cartouche) = &topicmap.cartouche {
            validate_text(&cartouche.heading, "/topicmap/cartouche/heading", 100)?;
            validate_text(&cartouche.meta, "/topicmap/cartouche/meta", 200)?;
        }
        Ok(warnings)
    }

    /// An atlas is valid when every realm holds regions, every label is unique across both
    /// levels, and every link names a region that exists. Labels share one namespace because a
    /// reader does not distinguish them either: on the finished map both are just names of
    /// places, and two of them reading alike would be a defect, not a subtlety.
    fn validate_atlas(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("an atlas chart", "atlas.realms")?;
        let Some(atlas) = &self.atlas else {
            return Err(ChartError::new(
                "missing_atlas",
                "/atlas",
                "an atlas chart requires an atlas block",
            ));
        };
        if atlas.realms.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/atlas/realms",
                "provide at least one realm",
            ));
        }
        if atlas.realms.len() > MAX_REALMS {
            return Err(ChartError::new(
                "too_many_realms",
                "/atlas/realms",
                format!("at most {MAX_REALMS} realms are supported"),
            ));
        }
        if !(0.2..=1.0).contains(&atlas.area_damping) {
            return Err(ChartError::new(
                "invalid_area_damping",
                "/atlas/areaDamping",
                "areaDamping must be between 0.2 and 1",
            ));
        }

        let mut labels = BTreeSet::new();
        let mut regions = BTreeSet::new();
        let mut warnings = Vec::new();
        let mut places = 0usize;

        for (index, realm) in atlas.realms.iter().enumerate() {
            places += validate_realm(
                realm,
                &format!("/atlas/realms/{index}"),
                &mut labels,
                &mut regions,
                &mut warnings,
            )?;
        }

        if regions.len() > MAX_REGIONS {
            return Err(ChartError::new(
                "too_many_regions",
                "/atlas/realms",
                format!("at most {MAX_REGIONS} regions are supported across all realms"),
            ));
        }
        if places > MAX_PLACES {
            return Err(ChartError::new(
                "too_many_places",
                "/atlas/realms",
                format!("at most {MAX_PLACES} places are supported across all regions"),
            ));
        }

        for (index, link) in atlas.links.iter().enumerate() {
            let path = format!("/atlas/links/{index}");
            for (side, label) in [("from", &link.from), ("to", &link.to)] {
                if !regions.contains(label.as_str()) {
                    return Err(ChartError::new(
                        "unknown_region_link",
                        format!("{path}/{side}"),
                        format!("{label:?} is not a declared region label"),
                    ));
                }
            }
            if !(0.0..=1.0).contains(&link.weight) {
                return Err(ChartError::new(
                    "invalid_link_weight",
                    format!("{path}/weight"),
                    "weight must be between 0 and 1",
                ));
            }
        }

        Ok(warnings)
    }
}

/// One realm and everything under it. Returns how many places it holds, so the caller can keep
/// the running total without walking the regions a second time.
fn validate_realm(
    realm: &RealmSpec,
    path: &str,
    labels: &mut BTreeSet<String>,
    regions: &mut BTreeSet<String>,
    warnings: &mut Vec<ChartWarning>,
) -> Result<usize, ChartError> {
    validate_text(&realm.label, &format!("{path}/label"), 200)?;
    validate_optional_text(realm.tooltip.as_ref(), &format!("{path}/tooltip"), 300)?;
    if !labels.insert(realm.label.clone()) {
        return Err(ChartError::new(
            "duplicate_atlas_label",
            format!("{path}/label"),
            "realm and region labels must be unique",
        ));
    }
    if realm.regions.is_empty() {
        return Err(ChartError::new(
            "empty_realm",
            format!("{path}/regions"),
            "a realm needs at least one region",
        ));
    }
    if realm.regions.len() == 1 {
        warnings.push(ChartWarning::new(
            "realm_without_structure",
            format!("{path}/regions"),
            "a realm with a single region has no inner structure to show; it is drawn as one area",
        ));
    }

    let mut places = 0usize;
    for (index, region) in realm.regions.iter().enumerate() {
        let region_path = format!("{path}/regions/{index}");
        validate_text(&region.label, &format!("{region_path}/label"), 200)?;
        validate_number(region.value, &format!("{region_path}/value"))?;
        if region.value < 1.0 {
            return Err(ChartError::new(
                "region_value_out_of_range",
                format!("{region_path}/value"),
                "value must be at least 1",
            ));
        }
        validate_optional_text(
            region.tooltip.as_ref(),
            &format!("{region_path}/tooltip"),
            300,
        )?;
        if !labels.insert(region.label.clone()) {
            return Err(ChartError::new(
                "duplicate_atlas_label",
                format!("{region_path}/label"),
                "realm and region labels must be unique",
            ));
        }
        regions.insert(region.label.clone());
        places += region.places.len();
        // More places than the region claims to hold means the two numbers come from different
        // counts; the map would then draw one and label the other.
        if places_exceed_value(region.places.len(), region.value) {
            warnings.push(ChartWarning::new(
                "more_places_than_value",
                format!("{region_path}/places"),
                "the region lists more places than its value; the label will not match what is drawn",
            ));
        }
        for (place_index, place) in region.places.iter().enumerate() {
            validate_place(place, &format!("{region_path}/places/{place_index}"))?;
        }
    }
    Ok(places)
}

/// A count as a measured value, for the data table.
#[allow(clippy::cast_precision_loss)]
fn place_count(places: usize) -> f64 {
    places as f64
}

/// Whether a region lists more places than its value claims. Written out because the comparison
/// crosses from a count to a measured value, and the cast is the only place that can go wrong.
#[allow(clippy::cast_precision_loss)]
fn places_exceed_value(places: usize, value: f64) -> bool {
    places as f64 > value
}

/// A place carries a name, a prominence and, if it wants one, a line of detail. Labels are not
/// required to be unique: two entries may well share a title.
fn validate_place(place: &PlaceSpec, path: &str) -> Result<(), ChartError> {
    validate_text(&place.label, &format!("{path}/label"), 200)?;
    validate_number(place.weight, &format!("{path}/weight"))?;
    if !(0.25..=4.0).contains(&place.weight) {
        return Err(ChartError::new(
            "place_weight_out_of_range",
            format!("{path}/weight"),
            "weight must be between 0.25 and 4",
        ));
    }
    validate_optional_text(place.tooltip.as_ref(), &format!("{path}/tooltip"), 300)?;
    Ok(())
}

/// A topic's label and value, shared by `topics` and `islands`: unique text, a finite value of at
/// least 1, and, for a `topic`, a warning once the area would be too small to hold its own label.
fn validate_topic(
    topic: &TopicSpec,
    path: &str,
    warn_if_small: bool,
    labels: &mut BTreeSet<String>,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    validate_text(&topic.label, &format!("{path}/label"), 200)?;
    validate_number(topic.value, &format!("{path}/value"))?;
    if topic.value < 1.0 {
        return Err(ChartError::new(
            "topic_value_out_of_range",
            format!("{path}/value"),
            "value must be at least 1",
        ));
    }
    if let Some(tooltip) = &topic.tooltip {
        validate_text(tooltip, &format!("{path}/tooltip"), 300)?;
    }
    if !labels.insert(topic.label.clone()) {
        return Err(ChartError::new(
            "duplicate_topic_label",
            format!("{path}/label"),
            "topic and island labels must be unique",
        ));
    }
    if warn_if_small && topic.value < 12.0 {
        warnings.push(ChartWarning::new(
            "topic_too_small_for_label",
            format!("{path}/value"),
            "below 12, an area is too small to hold its own label at this scale; consider listing it as an island instead",
        ));
    }
    Ok(())
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
/// edge is the edge of the plot, but at least one pair has to be complete, and the zone needs a
/// label.
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
    let values = layer.bottom.is_some() && layer.top.is_some();
    let times = from.is_some() && to.is_some();
    if !values && !times {
        return Err(ChartError::new(
            "missing_position",
            path,
            "give bottom and top for a range of values, from and to for a span of time, or both",
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

/// The reference and the optional ends of a diverging scale: `min` below the reference and `max`
/// above it, so that both arms of the scale have a direction.
fn validate_diverging(
    reference: f64,
    min: Option<f64>,
    max: Option<f64>,
    path: &str,
) -> Result<(), ChartError> {
    validate_number(reference, &format!("{path}/reference"))?;
    if let Some(min) = min {
        validate_number(min, &format!("{path}/min"))?;
        if min >= reference {
            return Err(ChartError::new(
                "invalid_scale",
                format!("{path}/min"),
                "min must lie below reference",
            ));
        }
    }
    if let Some(max) = max {
        validate_number(max, &format!("{path}/max"))?;
        if max <= reference {
            return Err(ChartError::new(
                "invalid_scale",
                format!("{path}/max"),
                "max must lie above reference",
            ));
        }
    }
    Ok(())
}

/// Year, month and day of a plain ISO date such as `2024-03-01`; `None` for anything else,
/// including a date with a time of day.
pub(crate) fn calendar_date(text: &str) -> Option<(i64, u32, u32)> {
    if text.len() != 10 {
        return None;
    }
    let epoch = crate::time::parse_iso(text, crate::time::TimeZone::utc())?;
    Some(crate::time::civil_from_days(epoch.div_euclid(86_400)))
}

/// The value of `type` that selects a chart type.
pub(crate) const fn type_name(chart_type: ChartType) -> &'static str {
    match chart_type {
        ChartType::Bar => "bar",
        ChartType::Line => "line",
        ChartType::Time => "time",
        ChartType::Topicmap => "topicmap",
        ChartType::Atlas => "atlas",
        ChartType::Stripes => "stripes",
        ChartType::Calendar => "calendar",
        ChartType::Rangebar => "rangebar",
        ChartType::Multiples => "multiples",
    }
}

/// A count as a signed number, for year arithmetic.
fn count_i64(value: usize) -> i64 {
    i64::try_from(value).expect("counts are limited by validation")
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

pub(crate) fn validate_number(value: f64, path: &str) -> Result<(), ChartError> {
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

const fn default_mobile_height() -> u32 {
    360
}

const fn default_breakpoint() -> u32 {
    640
}

const fn default_show_values() -> bool {
    true
}

fn default_timezone() -> String {
    "UTC".to_owned()
}

const fn default_height_ratio() -> u32 {
    1
}

const fn one() -> f64 {
    1.0
}

const fn default_true() -> bool {
    true
}

// serde hands this function a reference, so the signature follows serde's shape.
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_false(value: &bool) -> bool {
    !*value
}

// serde hands this function a reference, so the signature follows serde's shape.
#[allow(clippy::trivially_copy_pass_by_ref)]
const fn is_true(value: &bool) -> bool {
    *value
}

/// A middle value in the 0..3 range: visible depth without crowding a small map.
const fn default_depth_bands() -> u8 {
    2
}
