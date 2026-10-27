//! Panes and their layers: the marks, their observations and how a stacked area lies on the
//! ones below.

use serde::{Deserialize, Serialize};

use super::{TimePrecision, default_height_ratio};
use crate::{
    spec::{ChartSpec, LayerRef, Stack, ValueAxisSpec, is_false},
    time::TimeValue,
};

/// One pane of a time chart: a value axis over the shared time axis, with its own layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PaneSpec {
    /// Heading of a small-multiples panel; a time chart's single pane has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// A short finding under a panel's title, such as a verdict: small multiples only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Draws the note in the text color and semibold instead of muted, to set one finding apart.
    #[serde(default, skip_serializing_if = "is_false")]
    pub note_emphasis: bool,
    /// Share of the plot area this pane takes; a single pane always takes all of it.
    #[serde(default = "default_height_ratio")]
    pub height_ratio: u32,
    #[serde(default)]
    pub value_axis: ValueAxisSpec,
    #[serde(default)]
    pub layers: Vec<LayerSpec>,
    /// Small multiples of bars: this panel's value for each of the chart's `categories`, in their
    /// order; `null` for a category without a value. Instead of `layers`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<Option<f64>>,
    /// `"normal"` stacks the area layers of a time chart's pane in layer order, each on top of
    /// the ones before it, so that the top edge shows their total.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<Stack>,
}

/// How a layer draws its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    Line,
    Area,
    /// A dot for every observation and no line between them, as in a scatter plot.
    Point,
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
    /// How a line runs between observations: straight, or as steps that hold each value until
    /// the next, for values that apply to a whole period.
    #[serde(default, skip_serializing_if = "Curve::is_linear")]
    pub curve: Curve,
    /// How finely this layer's times are written in tooltips, the table and the description,
    /// for a layer coarser than the rest of the chart, such as yearly means beside monthly
    /// values. Overrides `timeAxis.precision`; calendar axes only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precision: Option<TimePrecision>,
    /// `false` draws a line or area without markers, such as a fitted line from two points;
    /// its observations keep their tooltips on invisible targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub markers: Option<bool>,
    /// Where the last step of a `curve: "step"` layer ends: the last value holds until this time,
    /// which is no observation of its own — no marker, tooltip or table row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_end: Option<TimeValue>,
}

/// Which observations of a `type: "time"` chart carry a tooltip.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tooltips {
    /// Every observation, while observations stand at least 4 pixels apart: on its marker, or
    /// on an invisible target where the line is too dense for markers.
    #[default]
    Observations,
    /// Only drawn markers; a line too dense for markers has no tooltips, which keeps the SVG
    /// small. The values stay in the data table and the description.
    Markers,
}

impl Tooltips {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(crate) const fn is_observations(&self) -> bool {
        matches!(self, Self::Observations)
    }
}

/// How a line or an area runs from one observation to the next.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Curve {
    #[default]
    Linear,
    /// Holds each value until the next observation, then jumps.
    Step,
}

impl Curve {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_linear(&self) -> bool {
        matches!(self, Self::Linear)
    }

    /// The points of a line through `points` in this curve: for steps, a corner before every
    /// jump.
    pub(crate) fn points(self, points: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
        if self == Self::Linear {
            return points;
        }
        let mut stepped = Vec::with_capacity(points.len() * 2);
        for (index, point) in points.iter().enumerate() {
            if index > 0 {
                stepped.push((point.0, points[index - 1].1));
            }
            stepped.push(*point);
        }
        stepped
    }
}

/// The weight of a line.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Stroke {
    #[default]
    Regular,
    Thin,
    Bold,
    /// Between thin and regular, for the lines of small panels.
    Medium,
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
    pub(super) fn observed_times(&self, zone: crate::time::TimeZone) -> impl Iterator<Item = i64> {
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
    pub(super) fn has_two_points(&self, zone: crate::time::TimeZone) -> bool {
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
    /// What lies below each observation of a stacked area: the sum of the area layers before
    /// it in its pane. `None` for a layer that is not stacked.
    pub(crate) fn stack_base(
        &self,
        entry: LayerRef,
        zone: crate::time::TimeZone,
    ) -> Option<Vec<f64>> {
        let pane = &self.panes[entry.pane];
        if pane.stack.is_none() || entry.layer.mark != Mark::Area {
            return None;
        }
        let mut base = vec![0.0; entry.layer.points.len()];
        for layer in pane.layers[..entry.local]
            .iter()
            .filter(|layer| layer.mark == Mark::Area)
        {
            for (sum, (_, value)) in base.iter_mut().zip(layer.resolved_points(zone)) {
                *sum += value;
            }
        }
        // A stream starts below zero by half the total, so that the whole stack is centered.
        if pane.stack == Some(Stack::Stream) {
            for (sum, total) in base.iter_mut().zip(self.stream_totals(entry.pane, zone)) {
                *sum -= total / 2.0;
            }
        }
        Some(base)
    }

    /// The total of the area layers of a pane at every time of its first area.
    fn stream_totals(&self, pane: usize, zone: crate::time::TimeZone) -> Vec<f64> {
        let mut totals: Vec<f64> = Vec::new();
        for layer in self.panes[pane]
            .layers
            .iter()
            .filter(|layer| layer.mark == Mark::Area)
        {
            totals.resize(layer.points.len(), 0.0);
            for (sum, (_, value)) in totals.iter_mut().zip(layer.resolved_points(zone)) {
                *sum += value;
            }
        }
        totals
    }

    /// Whether any pane is a stream: its areas stacked around a moving baseline.
    pub(crate) fn has_stream(&self) -> bool {
        self.panes
            .iter()
            .any(|pane| pane.stack == Some(Stack::Stream))
    }

    /// A layer's observations where they are drawn: a stacked area on top of the ones below.
    pub(crate) fn drawn_points(
        &self,
        entry: LayerRef,
        zone: crate::time::TimeZone,
    ) -> Vec<(i64, f64)> {
        let points = entry.layer.resolved_points(zone);
        match self.stack_base(entry, zone) {
            Some(base) => points
                .into_iter()
                .zip(base)
                .map(|((epoch, value), below)| (epoch, value + below))
                .collect(),
            None => points,
        }
    }
}
