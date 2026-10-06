mod architecture;
mod atlas;
mod boxplot;
mod calendar;
mod categorical;
mod dataset;
mod flow;
mod parliament;
mod rangebar;
mod sequence;
mod state;
mod stripes;
mod timechart;
mod timeline;
mod topicmap;
mod tree;
mod treemap;
mod waffle;
mod waterfall;

use serde::{Deserialize, Serialize};
use serde_path_to_error::Segment;

use crate::{
    error::{ChartError, ChartWarning},
    time::TimeValue,
};
pub use architecture::{
    ArchitectureSpec, BoundarySpec, ComponentKind, ComponentSpec, ConnectionSpec,
};
pub use atlas::{AtlasSpec, PlaceSpec, RegionSpec};
pub use boxplot::BoxSpec;
pub(crate) use boxplot::BoxSummary;
pub(crate) use calendar::calendar_date;
pub use calendar::{CalendarDay, CalendarLayout, CalendarSpec};
pub use categorical::{DataPoint, SeriesSpec};
pub use flow::{FlowEdgeSpec, FlowNodeSpec, FlowSpec, GroupSpec, LaneSpec, NodeKind};
pub use parliament::{ParliamentSpec, PartySpec};
pub use rangebar::RangeSpec;
pub use sequence::{
    BranchSpec, DiagramOrientation, FragmentKind, FragmentSpec, MessageKind, MessageSpec,
    ParticipantKind, ParticipantSpec, SequenceSpec,
};
pub use state::{StateKind, StateNodeSpec, StateSpec, TransitionSpec};
pub(crate) use stripes::Diverging;
pub use stripes::StripesSpec;
pub use timechart::{
    Curve, Dash, Gaps, LayerSpec, Mark, OhlcPoint, PaneSpec, Shape, Stroke, TimeAxisKind,
    TimeAxisSpec, TimePoint, TimePrecision, Tooltips,
};
pub(crate) use timechart::{LayerContext, MAX_TIME_POINTS_PER_LAYER, validate_layer_name};
pub use timeline::{MarkerSpec, TimelineItemSpec, TimelineSpec};
pub(crate) use timeline::{Span, zone as timeline_zone};
pub use topicmap::{CartoucheSpec, Corner, TopicLinkSpec, TopicMapSpec, TopicSpec};
pub use tree::{TreeNodeKind, TreeNodeSpec, TreeSpec};
pub use treemap::{TreemapItemSpec, TreemapSpec};
pub use waffle::{WafflePartSpec, WaffleSpec};
pub(crate) use waterfall::WaterfallBar;
pub use waterfall::{StepKind, StepSpec, WaterfallSpec};

const MAX_DATA_POINTS: usize = 100;
/// Limited so that every series keeps a color that stays distinguishable for common
/// color-vision deficiencies.
pub(crate) const MAX_SERIES: usize = 4;
/// The most ticks a declared step may put on an axis.
pub(crate) const MAX_AXIS_TICKS: f64 = 50.0;
/// Reference lines on a bar chart; beyond this the lines crowd the bars they explain.
pub(crate) const MAX_REFERENCES: usize = 4;
/// Fixed decimal places; beyond this a value stops being readable as a number.
pub(crate) const MAX_DECIMALS: u8 = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "every bool is a switch of the JSON specification"
)]
pub struct ChartSpec {
    pub schema_version: u8,
    #[serde(rename = "type")]
    pub chart_type: ChartType,
    #[serde(default)]
    pub orientation: Orientation,
    /// Draws every other series of a `type: "bar"` chart as an outline, so that series differ
    /// in form as well as in color.
    #[serde(default, skip_serializing_if = "is_false")]
    pub patterns: bool,
    /// Stacks the series of a `type: "bar"` chart instead of setting them side by side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<Stack>,
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
    /// Whether the SVG profile draws the title in the chart. It always remains the accessible name;
    /// the HTML profile never draws it, its caption is the visible title.
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
    /// The participants and messages of a `type: "sequence"` diagram. Skipped while absent, like
    /// `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<SequenceSpec>,
    /// The steps and edges of a `type: "flow"` chart. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<FlowSpec>,
    /// The states and transitions of a `type: "state"` diagram. Skipped while absent, like
    /// `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateSpec>,
    /// The components, connections and boundaries of a `type: "architecture"` diagram. Skipped
    /// while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ArchitectureSpec>,
    /// The nodes of a `type: "tree"` diagram. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<TreeSpec>,
    /// A waterfall: a running total that rises and falls step by step. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waterfall: Option<WaterfallSpec>,
    /// A waffle: squares that each stand for a share of a whole. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waffle: Option<WaffleSpec>,
    /// A parliament: seats as dots in a semicircle, in blocks by party. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parliament: Option<ParliamentSpec>,
    /// A treemap: rectangles whose areas follow the values. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treemap: Option<TreemapSpec>,
    /// The items of a `type: "timeline"` chart. Skipped while absent, like `topicmap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeline: Option<TimelineSpec>,
    /// The boxes of a `type: "boxplot"` chart, one per category.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boxes: Vec<BoxSpec>,
    /// The spans of a `type: "rangebar"` chart, one per category.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ranges: Vec<RangeSpec>,
    /// Reference lines across a `type: "bar"` chart, such as an average or a target.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<ReferenceSpec>,
    /// Draws a `type: "time"` chart as a sparkline: only its lines and areas, no axes, title or
    /// legend, at a size down to 60 × 16. Title and description stay its accessible name.
    #[serde(default, skip_serializing_if = "is_false")]
    pub sparkline: bool,
    /// Which observations of a `type: "time"` chart carry a tooltip.
    #[serde(default, skip_serializing_if = "Tooltips::is_observations")]
    pub tooltips: Tooltips,
    /// Where a `type: "time"` chart names its series: in a legend above the plot, or at the end
    /// of every line.
    #[serde(default, skip_serializing_if = "LegendPlacement::is_top")]
    pub legend: LegendPlacement,
    /// Gives every panel of a `type: "multiples"` chart its own value axis, for panels whose
    /// values differ in unit or size; they can then no longer be compared by height.
    #[serde(default, skip_serializing_if = "is_false")]
    pub independent_axes: bool,
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
    /// Grid columns of small multiples in the mobile variant, usually fewer than on the wide
    /// chart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u32>,
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
    /// The language code the specification uses: `en` or `de`.
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::De => "de",
        }
    }

    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_en(&self) -> bool {
        matches!(self, Self::En)
    }
}

/// How a value is written: format, fixed decimals if any, locale, and whether thousands are
/// separated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct NumberStyle {
    pub format: ValueFormat,
    pub decimals: Option<u8>,
    pub locale: Locale,
    pub thousands: bool,
}

impl From<ValueFormat> for NumberStyle {
    fn from(format: ValueFormat) -> Self {
        Self {
            format,
            ..Self::default()
        }
    }
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
    /// A sequence diagram: participants and the messages they exchange, in order.
    Sequence,
    /// A flow chart: steps joined by arrows, in layers along the flow.
    Flow,
    /// A state diagram: states joined by transitions, laid out like a flow chart.
    State,
    /// An architecture diagram: components and connections inside nested boundaries.
    Architecture,
    /// A tree: a root and the nodes below it, such as an organization chart.
    Tree,
    /// Boxes with whiskers and outliers, one per category: distributions side by side.
    Boxplot,
    /// A timeline: phases, milestones and the days that matter, in rows over one time axis.
    Timeline,
    /// A waterfall: a running total that rises and falls step by step.
    Waterfall,
    /// A waffle: squares that each stand for a share of a whole.
    Waffle,
    /// A parliament: seats as dots in a semicircle, in blocks by party.
    Parliament,
    /// A treemap: rectangles whose areas follow the values.
    Treemap,
}

impl ChartType {
    /// Every chart type, in the order of the specification's documentation.
    pub const ALL: [Self; 20] = [
        Self::Bar,
        Self::Line,
        Self::Time,
        Self::Topicmap,
        Self::Atlas,
        Self::Stripes,
        Self::Calendar,
        Self::Rangebar,
        Self::Multiples,
        Self::Sequence,
        Self::Flow,
        Self::State,
        Self::Architecture,
        Self::Tree,
        Self::Boxplot,
        Self::Timeline,
        Self::Waterfall,
        Self::Waffle,
        Self::Parliament,
        Self::Treemap,
    ];

    /// The chart type that `type` names, such as `"bar"`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|chart_type| type_name(*chart_type) == name)
    }
}

/// Where a time chart names its series.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LegendPlacement {
    /// A legend above the plot.
    #[default]
    Top,
    /// The name of each line at its last observation, right of the plot.
    End,
}

impl LegendPlacement {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_top(&self) -> bool {
        matches!(self, Self::Top)
    }
}

/// How the series of a bar chart share a category: stacked by value, or as shares of the
/// category's total.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Stack {
    Normal,
    Percent,
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
    /// The axis reaches at least down to this value; without it, a line never pads below zero
    /// when all values are zero or more.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// The axis reaches at least up to this value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Separates thousands in every written value: `12,500` or, in German, `12.500`. Off by
    /// default, because four digits are often years.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub thousands_separator: bool,
    /// Linear by default; `log` spaces powers of ten evenly, for values across several orders of
    /// magnitude.
    #[serde(default, skip_serializing_if = "AxisScale::is_linear")]
    pub scale: AxisScale,
    /// Runs the axis the other way: larger values down (or left), as δ18O records are drawn.
    #[serde(default, skip_serializing_if = "is_false")]
    pub reverse: bool,
    /// The distance between two ticks, instead of a round step chosen from the values.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// `min` and `max` are the ends of the axis, not only values it reaches; every value has to
    /// lie between them.
    #[serde(default, skip_serializing_if = "is_false")]
    pub exact: bool,
    /// The unit of the values, such as `W/m²`, written after the top tick label instead of in a
    /// title above the plot, and after the column names of the data table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

impl ChartSpec {
    /// The unit of the value axis that pane `pane` of a time chart or of small multiples is
    /// drawn on.
    pub(crate) fn axis_unit(&self, pane: usize) -> Option<&str> {
        match self.chart_type {
            ChartType::Time => self.panes.get(pane)?.value_axis.unit.as_deref(),
            ChartType::Multiples if self.is_bar_multiples() => {
                self.panes.get(pane)?.value_axis.unit.as_deref()
            }
            _ => self.value_axis.unit.as_deref(),
        }
    }
}

/// How a value axis spaces its values.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AxisScale {
    #[default]
    Linear,
    Log,
}

impl AxisScale {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_linear(&self) -> bool {
        matches!(self, Self::Linear)
    }
}

/// A reference line across a bar chart: a value every bar is read against.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceSpec {
    pub value: f64,
    pub label: String,
}

impl ValueAxisSpec {
    /// The declared lower and upper reach of the axis.
    pub(crate) const fn bounds(&self) -> (Option<f64>, Option<f64>) {
        (self.min, self.max)
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValueFormat {
    #[default]
    Number,
    Percent,
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
        if mobile.columns.is_some() {
            spec.columns = mobile.columns;
        }
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
        self.validate_value_axes()?;
        match self.chart_type {
            ChartType::Time => return self.validate_time(),
            ChartType::Multiples => return self.validate_multiples(),
            ChartType::Topicmap => return self.validate_topicmap(),
            ChartType::Atlas => return self.validate_atlas(),
            ChartType::Stripes => return self.validate_stripes(),
            ChartType::Calendar => return self.validate_calendar(),
            ChartType::Rangebar => return self.validate_rangebar(),
            ChartType::Sequence => return self.validate_sequence(),
            ChartType::Flow => return self.validate_flow(),
            ChartType::State => return self.validate_state(),
            ChartType::Architecture => return self.validate_architecture(),
            ChartType::Tree => return self.validate_tree(),
            ChartType::Boxplot => return self.validate_boxplot(),
            ChartType::Timeline => return self.validate_timeline(),
            ChartType::Treemap => return self.validate_treemap(),
            ChartType::Parliament => return self.validate_parliament(),
            ChartType::Waffle => return self.validate_waffle(),
            ChartType::Waterfall => return self.validate_waterfall(),
            ChartType::Bar | ChartType::Line => {}
        }
        let warnings = self.validate_data()?;
        self.validate_references()?;
        if self.chart_type == ChartType::Line
            && self.series.is_empty()
            && self.data.iter().all(|point| point.value.is_none())
        {
            return Err(ChartError::new(
                "empty_series",
                "/data",
                "line charts require at least one numeric value",
            ));
        }
        Ok(warnings)
    }

    /// Every value a value axis has to place, with its path: the values, band and zone edges,
    /// candles, reference lines and point markers, by the axis they belong to — `None` for the
    /// top-level value axis, `Some(pane)` for the own axis of a time chart's pane. Small multiples
    /// share the top-level axis.
    fn axis_values(&self) -> Vec<(Option<usize>, f64, String)> {
        let mut values = Vec::new();
        for (index, point) in self.data.iter().enumerate() {
            if let Some(value) = point.value {
                values.push((None, value, format!("/data/{index}/value")));
            }
        }
        for (series_index, series) in self.series.iter().enumerate() {
            for (index, value) in series.values.iter().enumerate() {
                if let Some(value) = value {
                    values.push((
                        None,
                        *value,
                        format!("/series/{series_index}/values/{index}"),
                    ));
                }
            }
        }
        for (index, point) in self.data.iter().enumerate() {
            for (name, value) in [("lower", point.lower), ("upper", point.upper)] {
                if let Some(value) = value {
                    values.push((None, value, format!("/data/{index}/{name}")));
                }
            }
        }
        for (index, reference) in self.references.iter().enumerate() {
            values.push((None, reference.value, format!("/references/{index}/value")));
        }
        for (index, range) in self.ranges.iter().enumerate() {
            for (name, value) in [
                ("low", Some(range.low)),
                ("high", Some(range.high)),
                ("mid", range.mid),
            ] {
                if let Some(value) = value {
                    values.push((None, value, format!("/ranges/{index}/{name}")));
                }
            }
        }
        for (pane_index, pane) in self.panes.iter().enumerate() {
            let axis = (self.chart_type != ChartType::Multiples || self.is_bar_multiples())
                .then_some(pane_index);
            for (index, value) in pane.values.iter().enumerate() {
                if let Some(value) = value {
                    values.push((axis, *value, format!("/panes/{pane_index}/values/{index}")));
                }
            }
            for (layer_index, layer) in pane.layers.iter().enumerate() {
                let path = format!("/panes/{pane_index}/layers/{layer_index}");
                for (index, point) in layer.points.iter().enumerate() {
                    for (name, value) in [
                        ("value", point.value),
                        ("lower", point.lower),
                        ("upper", point.upper),
                    ] {
                        if let Some(value) = value {
                            values.push((axis, value, format!("{path}/points/{index}/{name}")));
                        }
                    }
                }
                for (index, candle) in layer.data.iter().enumerate() {
                    values.push((axis, candle.low, format!("{path}/data/{index}/low")));
                    values.push((axis, candle.high, format!("{path}/data/{index}/high")));
                }
                for (name, value) in [
                    ("value", layer.value),
                    ("bottom", layer.bottom),
                    ("top", layer.top),
                ] {
                    if let Some(value) = value {
                        values.push((axis, value, format!("{path}/{name}")));
                    }
                }
            }
        }
        values
    }

    /// The value axes and their paths: the top-level one, and the own axis of every time chart
    /// pane.
    fn value_axes(&self) -> Vec<(Option<usize>, &ValueAxisSpec, String)> {
        let mut axes = vec![(None, &self.value_axis, "/valueAxis".to_owned())];
        if self.chart_type == ChartType::Time || self.is_bar_multiples() {
            for (index, pane) in self.panes.iter().enumerate() {
                axes.push((
                    Some(index),
                    &pane.value_axis,
                    format!("/panes/{index}/valueAxis"),
                ));
            }
        }
        axes
    }

    /// The positive and the negative total of every category of a stacked bar chart.
    fn stack_totals(&self) -> Vec<f64> {
        (0..self.categories.len())
            .flat_map(|index| {
                let values = self.series.iter().filter_map(|series| series.values[index]);
                let positive: f64 = values.clone().filter(|value| *value > 0.0).sum();
                let negative: f64 = values.filter(|value| *value < 0.0).sum();
                [positive, negative]
            })
            .collect()
    }

    /// A declared value step yields at most [`MAX_AXIS_TICKS`] ticks across the values and the
    /// declared ends, and an exact axis at least two: a tiny step would otherwise draw millions.
    fn validate_tick_count(
        &self,
        (axis, axis_index): (&ValueAxisSpec, Option<usize>),
        step: f64,
        values: &[(Option<usize>, f64, String)],
        path: &str,
    ) -> Result<(), ChartError> {
        let own = values
            .iter()
            .filter(|(owner, _, _)| *owner == axis_index)
            .map(|(_, value, _)| *value);
        // A stacked pane reaches the totals of its areas.
        let zone = self.time_zone().unwrap_or_default();
        let stacked: Vec<f64> = match axis_index {
            Some(pane) if self.panes[pane].stack.is_some() => self
                .data_layers()
                .filter(|entry| entry.pane == pane)
                .flat_map(|entry| self.drawn_points(entry, zone))
                .map(|(_, value)| value)
                // Stacked areas grow from zero.
                .chain([0.0])
                .collect(),
            _ => Vec::new(),
        };
        let mut values: Vec<f64> = own.chain(stacked).chain(axis.min).chain(axis.max).collect();
        // Bars are measured from zero; a stack reaches the totals of its categories, a percent
        // stack runs from 0 to 1.
        if self.chart_type == ChartType::Bar {
            values.push(0.0);
            match self.stack {
                Some(Stack::Percent) => values = vec![0.0, 1.0],
                Some(Stack::Normal) => values.extend(self.stack_totals()),
                None => {}
            }
        }
        let (low, high) = values
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
                (low.min(*value), high.max(*value))
            });
        if !low.is_finite() {
            return Ok(());
        }
        let ticks = (high / step).floor() - (low / step).ceil() + 1.0;
        let few = axis.exact && ticks < 2.0;
        if ticks > MAX_AXIS_TICKS || few {
            return Err(ChartError::new(
                "invalid_axis_range",
                format!("{path}/step"),
                format!(
                    "a step of {step} gives {} ticks across {} to {}; choose one that gives 2 to {MAX_AXIS_TICKS}",
                    ticks.max(0.0),
                    low + 0.0,
                    high + 0.0
                ),
            ));
        }
        Ok(())
    }

    /// A logarithmic axis places only values above zero, and neither a stack nor an area, which
    /// are measured from zero. A declared `step` is a positive distance. An `exact` axis has both
    /// ends, and every value lies between them, so that nothing reaches outside the plot.
    fn validate_value_axes(&self) -> Result<(), ChartError> {
        let values = self.axis_values();
        for (axis_index, axis, path) in self.value_axes() {
            let log = axis.scale == AxisScale::Log;
            let refuse = |field: &str, reason: &str| {
                Err(ChartError::new(
                    "option_not_supported",
                    format!("{path}/{field}"),
                    reason.to_owned(),
                ))
            };
            if let Some(unit) = &axis.unit {
                if !matches!(self.chart_type, ChartType::Time | ChartType::Multiples) {
                    return refuse(
                        "unit",
                        "a unit at the top tick belongs to a time chart or small multiples; use the axis title",
                    );
                }
                validate_text(unit, &format!("{path}/unit"), 20)?;
            }
            if log && self.stack.is_some() {
                return refuse(
                    "scale",
                    "a stack is measured from zero and takes no logarithmic axis",
                );
            }
            if log && (axis.step.is_some() || axis.exact) {
                return refuse("step", "a logarithmic axis ticks at powers of ten");
            }
            if axis.exact && self.stack.is_some() {
                return refuse("exact", "a stack's totals decide its axis");
            }
            if let Some(step) = axis.step {
                validate_number(step, &format!("{path}/step"))?;
                if step <= 0.0 {
                    return Err(ChartError::new(
                        "invalid_value",
                        format!("{path}/step"),
                        "the step between ticks must be above zero",
                    ));
                }
            }
            if let (true, Some(min), Some(max)) = (axis.exact, axis.min, axis.max)
                && self.chart_type == ChartType::Bar
                && !(min..=max).contains(&0.0)
            {
                return Err(ChartError::new(
                    "invalid_axis_range",
                    format!("{path}/min"),
                    "bars start at zero, so an exact axis of a bar chart includes zero",
                ));
            }
            if axis.exact && (axis.min.is_none() || axis.max.is_none()) {
                return Err(ChartError::new(
                    "invalid_axis_range",
                    format!("{path}/exact"),
                    "an exact axis needs both min and max",
                ));
            }
            if log {
                positive_bounds(axis, &path)?;
            }
            for (_, value, value_path) in values.iter().filter(|(owner, _, _)| *owner == axis_index)
            {
                if log {
                    positive(*value, value_path.clone())?;
                }
                if let (true, Some(min), Some(max)) = (axis.exact, axis.min, axis.max)
                    && !(min..=max).contains(value)
                {
                    return Err(ChartError::new(
                        "value_outside_axis",
                        value_path.clone(),
                        format!("the value lies outside the exact axis from {min} to {max}"),
                    ));
                }
            }
            if let Some(step) = axis.step {
                self.validate_tick_count((axis, axis_index), step, &values, &path)?;
            }
            if log {
                let panes: Vec<usize> = match axis_index {
                    Some(pane) => vec![pane],
                    None if self.chart_type == ChartType::Multiples => {
                        (0..self.panes.len()).collect()
                    }
                    None => Vec::new(),
                };
                for pane in panes {
                    reject_log_area(&self.panes[pane], pane)?;
                }
            }
        }
        Ok(())
    }

    /// Every reference line has a usable value and a label; a chart takes at most four.
    pub(super) fn validate_references(&self) -> Result<(), ChartError> {
        if self.references.len() > MAX_REFERENCES {
            return Err(ChartError::new(
                "too_many_references",
                "/references",
                format!("use at most {MAX_REFERENCES} reference lines"),
            ));
        }
        for (index, reference) in self.references.iter().enumerate() {
            validate_number(reference.value, &format!("/references/{index}/value"))?;
            validate_text(&reference.label, &format!("/references/{index}/label"), 60)?;
        }
        Ok(())
    }

    /// Fixed decimals stay in a readable range; a declared axis range is finite and ordered.
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
            let base = path.trim_end_matches("/decimals");
            for (name, bound) in [("min", axis.min), ("max", axis.max)] {
                if let Some(bound) = bound {
                    validate_number(bound, &format!("{base}/{name}"))?;
                }
            }
            if let (Some(min), Some(max)) = (axis.min, axis.max)
                && min >= max
            {
                return Err(ChartError::new(
                    "invalid_axis_range",
                    format!("{base}/max"),
                    "valueAxis.max must be greater than valueAxis.min",
                ));
            }
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
        if self.sparkline && self.chart_type == ChartType::Time {
            for (path, value, range) in [
                ("/width", self.width, 60..=600),
                ("/height", self.height, 16..=200),
            ] {
                if !range.contains(&value) {
                    return Err(ChartError::new(
                        "invalid_dimension",
                        path,
                        format!(
                            "a sparkline's {} must be between {} and {}",
                            &path[1..],
                            range.start(),
                            range.end()
                        ),
                    ));
                }
            }
            return Ok(());
        }
        if !(200..=2_400).contains(&self.width) {
            return Err(ChartError::new(
                "invalid_dimension",
                "/width",
                "width must be between 200 and 2400",
            ));
        }
        if !(160..=1_600).contains(&self.height) {
            return Err(ChartError::new(
                "invalid_dimension",
                "/height",
                "height must be between 160 and 1600",
            ));
        }
        if let Some(mobile) = &self.mobile {
            for (path, value, range) in [
                ("/mobile/width", mobile.width, 200..=600),
                ("/mobile/height", mobile.height, 160..=1_600),
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

    /// Rejects a block that belongs to another chart type, so that it is never silently ignored.
    fn reject_foreign_blocks(&self) -> Result<(), ChartError> {
        let own = self.chart_type;
        for (index, pane) in self.panes.iter().enumerate() {
            let Some(note) = &pane.note else {
                if pane.note_emphasis {
                    return Err(ChartError::new(
                        "option_not_supported",
                        format!("/panes/{index}/noteEmphasis"),
                        "noteEmphasis needs a note",
                    ));
                }
                continue;
            };
            let path = format!("/panes/{index}/note");
            if own != ChartType::Multiples {
                return Err(ChartError::new(
                    "option_not_supported",
                    path,
                    "a note sits under the title of a small-multiples panel",
                ));
            }
            validate_text(note, &path, 160)?;
        }
        if own != ChartType::Time
            && let Some(index) = self.panes.iter().position(|pane| pane.stack.is_some())
        {
            return Err(ChartError::new(
                "option_not_supported",
                format!("/panes/{index}/stack"),
                "stacked areas belong to the pane of a time chart",
            ));
        }
        for (field, present, owner) in [
            ("/stripes", self.stripes.is_some(), ChartType::Stripes),
            ("/calendar", self.calendar.is_some(), ChartType::Calendar),
            ("/ranges", !self.ranges.is_empty(), ChartType::Rangebar),
            ("/boxes", !self.boxes.is_empty(), ChartType::Boxplot),
            ("/timeline", self.timeline.is_some(), ChartType::Timeline),
            ("/treemap", self.treemap.is_some(), ChartType::Treemap),
            (
                "/parliament",
                self.parliament.is_some(),
                ChartType::Parliament,
            ),
            ("/waffle", self.waffle.is_some(), ChartType::Waffle),
            ("/waterfall", self.waterfall.is_some(), ChartType::Waterfall),
            ("/sequence", self.sequence.is_some(), ChartType::Sequence),
            ("/flow", self.flow.is_some(), ChartType::Flow),
            ("/state", self.state.is_some(), ChartType::State),
            (
                "/architecture",
                self.architecture.is_some(),
                ChartType::Architecture,
            ),
            ("/tree", self.tree.is_some(), ChartType::Tree),
            ("/columns", self.columns.is_some(), ChartType::Multiples),
            (
                "/references",
                !self.references.is_empty() && own != ChartType::Rangebar,
                ChartType::Bar,
            ),
            ("/stack", self.stack.is_some(), ChartType::Bar),
            (
                "/legend",
                self.legend == LegendPlacement::End,
                ChartType::Time,
            ),
            ("/sparkline", self.sparkline, ChartType::Time),
            (
                "/tooltips",
                self.tooltips == Tooltips::Markers,
                ChartType::Time,
            ),
            ("/patterns", self.patterns, ChartType::Bar),
            (
                "/independentAxes",
                self.independent_axes,
                ChartType::Multiples,
            ),
            (
                "/mobile/columns",
                self.mobile
                    .as_ref()
                    .is_some_and(|mobile| mobile.columns.is_some()),
                ChartType::Multiples,
            ),
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
        self.reject_map_blocks()
    }

    /// Rejects a map block on the chart types that have no map at all.
    fn reject_map_blocks(&self) -> Result<(), ChartError> {
        let own = self.chart_type;
        if matches!(
            own,
            ChartType::Stripes
                | ChartType::Calendar
                | ChartType::Rangebar
                | ChartType::Boxplot
                | ChartType::Timeline
                | ChartType::Treemap
                | ChartType::Parliament
                | ChartType::Waffle
                | ChartType::Waterfall
                | ChartType::Multiples
                | ChartType::Sequence
                | ChartType::Flow
                | ChartType::State
                | ChartType::Architecture
                | ChartType::Tree
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
        if self.value_axis.title.is_some()
            || self.value_axis.format != ValueFormat::Number
            || self.value_axis.min.is_some()
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
        ChartType::Sequence => "sequence",
        ChartType::Flow => "flow",
        ChartType::State => "state",
        ChartType::Architecture => "architecture",
        ChartType::Tree => "tree",
        ChartType::Boxplot => "boxplot",
        ChartType::Timeline => "timeline",
        ChartType::Treemap => "treemap",
        ChartType::Parliament => "parliament",
        ChartType::Waffle => "waffle",
        ChartType::Waterfall => "waterfall",
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

/// A value on a logarithmic axis: above zero.
fn positive(value: f64, path: String) -> Result<(), ChartError> {
    if value > 0.0 {
        Ok(())
    } else {
        Err(ChartError::new(
            "invalid_value",
            path,
            "a logarithmic axis takes values above zero only",
        ))
    }
}

/// The declared range of a logarithmic axis: above zero.
fn positive_bounds(axis: &ValueAxisSpec, path: &str) -> Result<(), ChartError> {
    for (name, bound) in [("min", axis.min), ("max", axis.max)] {
        if let Some(bound) = bound {
            positive(bound, format!("{path}/{name}"))?;
        }
    }
    Ok(())
}

/// An area is filled down to zero, so a pane with one takes no logarithmic axis.
fn reject_log_area(pane: &PaneSpec, pane_index: usize) -> Result<(), ChartError> {
    for (layer_index, layer) in pane.layers.iter().enumerate() {
        if layer.mark == Mark::Area {
            return Err(ChartError::new(
                "option_not_supported",
                format!("/panes/{pane_index}/layers/{layer_index}/mark"),
                "an area is filled down to zero and takes no logarithmic axis",
            ));
        }
    }
    Ok(())
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
