mod architecture;
mod boxplot;
mod calendar;
mod categorical;
mod classes;
mod dataset;
mod deployment;
mod flow;
mod heatmap;
mod network;
mod parliament;
mod rangebar;
mod sankey;
mod scatter;
mod sequence;
mod state;
mod stripes;
mod survival;
mod timechart;
mod timeline;
mod tree;
mod treemap;
pub(crate) mod untagged;
mod validate;
mod view;
mod waffle;
mod waterfall;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    error::{ChartError, ChartWarning},
    time::TimeValue,
};
pub use architecture::{
    ArchitectureSpec, BoundarySpec, CategorySpec, ComponentKind, ComponentSpec,
    ConnectionDirection, ConnectionSpec, InterfaceSpec, PerspectiveSpec,
};
pub(crate) use boxplot::BoxSummary;
pub use boxplot::{BoxDisplay, BoxSpec, ViolinScale};
pub(crate) use calendar::calendar_date;
pub use calendar::{CalendarDay, CalendarLayout, CalendarSpec};
pub use categorical::{DataPoint, SeriesSpec};
pub use classes::{ClassKind, ClassSpec, ClassesSpec, PackageSpec, RelationKind, RelationSpec};
pub use flow::{
    DiagramRole, DiagramSpacing, FlowEdgeSpec, FlowNodeSpec, FlowSpec, GroupSpec, LaneSpec,
    MainPath, NodeKind, NoteSpec, PartSpec, PortSide,
};
pub use heatmap::{HeatmapCellSpec, HeatmapOrder, HeatmapScale, HeatmapSpec};
pub use network::{NetworkEdgeSpec, NetworkLayout, NetworkNodeSpec, NetworkOrder, NetworkSpec};
pub use parliament::{ParliamentSpec, PartySpec};
pub use rangebar::{RangeDisplay, RangeSort, RangeSpec};
pub(crate) use sankey::SankeyGraph;
pub use sankey::{SankeyLinkSpec, SankeyNodeSpec, SankeyOrder, SankeySpec};
pub(crate) use scatter::Quadrant;
pub use scatter::{
    ScatterAxis, ScatterLineSpec, ScatterPointSpec, ScatterQuadrantLabels, ScatterQuadrantsSpec,
    ScatterSpec, ScatterTickSpec,
};
pub use sequence::{
    BranchSpec, DiagramOrientation, FragmentKind, FragmentSpec, MessageKind, MessageSpec,
    ParticipantKind, ParticipantSpec, SequenceSpec,
};
pub use state::{StateKind, StateNodeSpec, StateSpec, TransitionSpec};
pub(crate) use stripes::Diverging;
pub use stripes::StripesSpec;
pub(crate) use survival::{KmCurve, KmStep};
pub use survival::{SurvivalGroupSpec, SurvivalObservationSpec, SurvivalSpec};
pub use timechart::{
    Curve, Dash, Gaps, LayerSpec, Mark, OhlcPoint, PaneSpec, Shape, Stroke, TimeAxisKind,
    TimeAxisSpec, TimePoint, TimePrecision, Tooltips,
};
pub(crate) use timechart::{LayerContext, MAX_TIME_POINTS_PER_LAYER, validate_layer_name};
pub(crate) use timeline::Span;
pub use timeline::{MarkerSpec, TimelineItemSpec, TimelineSpec};
pub use tree::{DistancesSpec, Linkage, TreeLayout, TreeNodeKind, TreeNodeSpec, TreeSpec};
pub(crate) use treemap::{TmNode, TmTree};
pub use treemap::{TreemapItemSpec, TreemapSpec};
pub(crate) use validate::validate_number;
use validate::{json_pointer, validate_optional_text, validate_text};
pub use view::{InstanceSpec, ViewSpec, ViewStepSpec};
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
    /// How a `type: "boxplot"` chart draws its observations: boxes (the default), violins or strips.
    #[serde(default, skip_serializing_if = "BoxDisplay::is_box")]
    pub box_display: BoxDisplay,
    /// How a `type: "rangebar"` chart draws its spans: bars (the default) or dumbbells.
    #[serde(default, skip_serializing_if = "RangeDisplay::is_bar")]
    pub display: RangeDisplay,
    /// The order of the spans of a `type: "rangebar"` chart: as listed (the default) or the
    /// widest first.
    #[serde(default, skip_serializing_if = "RangeSort::is_none")]
    pub sort: RangeSort,
    /// With `"boxDisplay": "violin"`: `width` (default) makes every violin as wide as the others at
    /// its widest, `area` gives them the same area, `count` an area that follows their number of
    /// observations.
    #[serde(default, skip_serializing_if = "ViolinScale::is_width")]
    pub violin_scale: ViolinScale,
    /// Marks the mean of the observations of every box of a `type: "boxplot"` chart with a
    /// diamond and adds a column to the table. Needs `values` on every box.
    #[serde(default, skip_serializing_if = "is_false")]
    pub show_mean: bool,
    /// Draws every other series of a `type: "bar"` chart as an outline, so that series differ
    /// in form as well as in color.
    #[serde(default, skip_serializing_if = "is_false")]
    pub patterns: bool,
    /// Draws a `type: "line"` chart of two categories as a slope chart: one line per series from
    /// the first value to the second, with the names at both ends instead of a legend.
    #[serde(default, skip_serializing_if = "is_false")]
    pub slope: bool,
    /// Draws a `type: "line"` chart as a rank chart: the rank of each series, 1 at the top, in
    /// every category, with the names at both ends instead of a legend.
    #[serde(default, skip_serializing_if = "is_false")]
    pub rank: bool,
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
    /// A named canvas size: sets `width` and `height` unless the specification gives them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<SizePreset>,
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
    /// How strokes and shapes are drawn: `clean` (the default) or `sketch`, slightly wobbly, for
    /// essays and slides. Text is not affected.
    #[serde(default, skip_serializing_if = "StrokeStyle::is_clean")]
    pub stroke: StrokeStyle,
    /// Colors of the palette, by the name of the `--chartlet-*` custom property without its
    /// prefix, as hex values. They win over the theme. Skipped while empty, like `theme`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tokens: BTreeMap<String, String>,
    /// The time axis of a `type: "time"` chart.
    #[serde(default, skip_serializing_if = "TimeAxisSpec::is_default")]
    pub time_axis: TimeAxisSpec,
    /// The panes of a `type: "time"` chart; every pane stacks one value axis over the shared
    /// time axis and holds its own layers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panes: Vec<PaneSpec>,
    /// The yearly values of a `type: "stripes"` chart. Skipped while absent, for the same reason as `theme`:
    /// adding the field must not change the serialized form, and with it the generated
    /// accessibility IDs, of every existing specification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stripes: Option<StripesSpec>,
    /// The daily values of a `type: "calendar"` chart. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calendar: Option<CalendarSpec>,
    /// The participants and messages of a `type: "sequence"` diagram. Skipped while absent, like
    /// `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<SequenceSpec>,
    /// The steps and edges of a `type: "flow"` chart. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<FlowSpec>,
    /// The states and transitions of a `type: "state"` diagram. Skipped while absent, like
    /// `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateSpec>,
    /// The components, connections and boundaries of a `type: "architecture"` diagram. Skipped
    /// while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ArchitectureSpec>,
    /// The nodes of a `type: "tree"` diagram. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<TreeSpec>,
    /// A waterfall: a running total that rises and falls step by step. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waterfall: Option<WaterfallSpec>,
    /// A waffle: squares that each stand for a share of a whole. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waffle: Option<WaffleSpec>,
    /// A heatmap: values or categories in a grid of rows and columns. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heatmap: Option<HeatmapSpec>,
    /// A parliament: seats as dots in a semicircle, in blocks by party. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parliament: Option<ParliamentSpec>,
    /// A treemap: rectangles whose areas follow the values. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treemap: Option<TreemapSpec>,
    /// A Sankey diagram: bands between nodes in columns, as thick as the flow. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sankey: Option<SankeySpec>,
    /// Kaplan-Meier survival curves: step curves of groups, with censoring marks, optional confidence bands and the number at risk. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub survival: Option<SurvivalSpec>,
    /// A scatter plot: points on two numeric axes, in groups, with threshold lines and named points. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scatter: Option<ScatterSpec>,
    /// A class or entity-relationship diagram: classes with compartments, joined by relations. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classes: Option<ClassesSpec>,
    /// A network: nodes joined by edges on a circle or in rings around one node. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkSpec>,
    /// The items of a `type: "timeline"` chart. Skipped while absent, like `stripes`.
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
    /// A second layout for narrow containers. Skipped while absent, like `stripes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile: Option<MobileSpec>,
}

/// A named canvas size for a destination: documents, slides, print or social previews.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SizePreset {
    Doc,
    DocWide,
    #[serde(rename = "slide-16x9")]
    Slide16x9,
    #[serde(rename = "slide-4x3")]
    Slide4x3,
    SocialOg,
    SocialSquare,
    PrintA4Landscape,
    PrintA3Landscape,
    PrintLetterLandscape,
}

impl SizePreset {
    /// Width and height in CSS pixels. Print sizes are the paper at 96 dpi.
    #[must_use]
    pub const fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Doc => (800, 450),
            Self::DocWide => (1_200, 500),
            Self::Slide16x9 => (1_920, 1_080),
            Self::Slide4x3 => (1_600, 1_200),
            Self::SocialOg => (1_200, 630),
            Self::SocialSquare => (1_080, 1_080),
            Self::PrintA4Landscape => (1_123, 794),
            Self::PrintA3Landscape => (1_587, 1_123),
            Self::PrintLetterLandscape => (1_056, 816),
        }
    }
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
    /// Diagrams only: a deep blue ground with pale lines.
    Blueprint,
}

impl Theme {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_light(&self) -> bool {
        matches!(self, Self::Light)
    }
}

/// How the lines and shapes of a chart are drawn.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StrokeStyle {
    #[default]
    Clean,
    /// Lines and shapes are displaced by a fraction of a pixel to a few, from a seed derived from
    /// the specification: the same bytes every time. Text, colors and geometry that carries data
    /// stay where they are within that distance. Not applied to the print variant or PNG.
    Sketch,
}

impl StrokeStyle {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_clean(&self) -> bool {
        matches!(self, Self::Clean)
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
    /// A heatmap: values or categories in a grid of rows and columns.
    Heatmap,
    /// A parliament: seats as dots in a semicircle, in blocks by party.
    Parliament,
    /// A treemap: rectangles whose areas follow the values.
    Treemap,
    /// A Sankey diagram: bands between nodes in columns, as thick as the flow.
    Sankey,
    /// Kaplan-Meier survival curves: step curves of groups, with censoring marks, optional confidence bands and the number at risk.
    Survival,
    /// A scatter plot: points on two numeric axes, in groups, with threshold lines and named points.
    Scatter,
    /// A class or entity-relationship diagram: classes with compartments, joined by relations.
    Classes,
    /// A network: nodes joined by edges, on a circle or in rings around one node.
    Network,
}

impl ChartType {
    /// Every chart type, in the order of the specification's documentation.
    pub const ALL: [Self; 24] = [
        Self::Bar,
        Self::Line,
        Self::Time,
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
        Self::Heatmap,
        Self::Parliament,
        Self::Treemap,
        Self::Sankey,
        Self::Survival,
        Self::Scatter,
        Self::Classes,
        Self::Network,
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
    pub(crate) const fn is_linear(&self) -> bool {
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
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ZoomBound {
    Index(usize),
    Time(TimeValue),
}

impl<'de> Deserialize<'de> for ZoomBound {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        untagged::deserialize(deserializer, "ZoomBound", |shape| {
            let index = match shape {
                untagged::Shape::Unsigned(number) => usize::try_from(number).ok(),
                untagged::Shape::Signed(number) => usize::try_from(number).ok(),
                _ => None,
            };
            match index {
                Some(index) => Some(Self::Index(index)),
                None => TimeValue::from_shape(shape).map(Self::Time),
            }
        })
    }
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
    /// How this column is written when it is not written like the chart's values: every pane of a time
    /// chart has its own value axis.
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
        let mut spec: Self = serde_json::from_str(input).map_err(|error: serde_json::Error| {
            let (code, (path, message)) = match error.classify() {
                serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                    ("invalid_json", ("/".to_owned(), error.to_string()))
                }
                serde_json::error::Category::Data | serde_json::error::Category::Io => {
                    ("invalid_spec", json_pointer(input, &error))
                }
            };
            ChartError::new(code, path, message)
        })?;
        if let Some(size) = spec.size {
            // The input parsed as an object, so a second read only checks which sizes it names.
            let given: serde_json::Value = serde_json::from_str(input).unwrap_or_default();
            let (width, height) = size.dimensions();
            if given.get("width").is_none() {
                spec.width = width;
            }
            if given.get("height").is_none() {
                spec.height = height;
            }
        }
        Ok(spec)
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
        self.validate_theme()?;
        self.validate_value_axes()?;
        let mut warnings = match self.chart_type {
            ChartType::Time => self.validate_time(),
            ChartType::Multiples => self.validate_multiples(),
            ChartType::Stripes => self.validate_stripes(),
            ChartType::Calendar => self.validate_calendar(),
            ChartType::Rangebar => self.validate_rangebar(),
            ChartType::Sequence => self.validate_sequence(),
            ChartType::Flow => self.validate_flow(),
            ChartType::State => self.validate_state(),
            ChartType::Architecture => self.validate_architecture(),
            ChartType::Tree => self.validate_tree(),
            ChartType::Boxplot => self.validate_boxplot(),
            ChartType::Timeline => self.validate_timeline(),
            ChartType::Classes => self.validate_classes(),
            ChartType::Scatter => self.validate_scatter(),
            ChartType::Survival => self.validate_survival(),
            ChartType::Sankey => self.validate_sankey(),
            ChartType::Treemap => self.validate_treemap(),
            ChartType::Parliament => self.validate_parliament(),
            ChartType::Waffle => self.validate_waffle(),
            ChartType::Heatmap => self.validate_heatmap(),
            ChartType::Waterfall => self.validate_waterfall(),
            ChartType::Network => self.validate_network(),
            ChartType::Bar | ChartType::Line => self.validate_bar_and_line(),
        }?;
        warnings.extend(crate::contrast::warnings(self));
        Ok(warnings)
    }
}

/// The value of `type` that selects a chart type.
pub(crate) const fn type_name(chart_type: ChartType) -> &'static str {
    match chart_type {
        ChartType::Bar => "bar",
        ChartType::Line => "line",
        ChartType::Time => "time",
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
        ChartType::Classes => "classes",
        ChartType::Scatter => "scatter",
        ChartType::Survival => "survival",
        ChartType::Sankey => "sankey",
        ChartType::Treemap => "treemap",
        ChartType::Parliament => "parliament",
        ChartType::Waffle => "waffle",
        ChartType::Heatmap => "heatmap",
        ChartType::Waterfall => "waterfall",
        ChartType::Network => "network",
    }
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
