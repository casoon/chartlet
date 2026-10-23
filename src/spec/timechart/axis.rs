//! The time axis: what its positions are, its ends, ticks and gaps, and the zoom steps over it.

use serde::{Deserialize, Serialize};

use super::{Curve, Mark, default_timezone};
use crate::{
    error::ChartError,
    spec::{ChartSpec, ZoomBound, is_false, validate_optional_text, validate_text},
    time::TimeValue,
};

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
    /// How finely tooltips, the data table and the description name an observation. Absent, the
    /// coarsest form that tells the observations apart is chosen from the data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precision: Option<TimePrecision>,
    /// The distance between ticks instead of one chosen from the width: whole years on a
    /// calendar axis, units on a numeric one. Ticks sit on multiples of it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// Positions the axis reaches at least, such as a round year before the first observation;
    /// they extend the axis and never cut an observation off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<TimeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<TimeValue>,
}

/// The precision a time axis names its observations in.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TimePrecision {
    Year,
    Month,
    Day,
    Minute,
}

impl From<TimePrecision> for crate::time::Precision {
    fn from(precision: TimePrecision) -> Self {
        match precision {
            TimePrecision::Year => Self::Year,
            TimePrecision::Month => Self::Month,
            TimePrecision::Day => Self::Day,
            TimePrecision::Minute => Self::Minute,
        }
    }
}

impl TimeAxisKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_calendar(&self) -> bool {
        matches!(self, Self::Calendar)
    }
}

impl TimeAxisSpec {
    pub(in crate::spec) fn is_default(&self) -> bool {
        self.timezone == default_timezone()
            && self.gaps == Gaps::Show
            && self.title.is_none()
            && self.kind == TimeAxisKind::Calendar
            && !self.reverse
            && self.precision.is_none()
            && self.step.is_none()
            && self.min.is_none()
            && self.max.is_none()
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
            precision: None,
            step: None,
            min: None,
            max: None,
        }
    }
}

impl ChartSpec {
    /// A declared time step places 1 to 50 ticks across everything the axis shows: none leaves
    /// the axis without labels, more draw too many.
    fn validate_time_tick_count(
        &self,
        zone: crate::time::TimeZone,
        step: f64,
    ) -> Result<(), ChartError> {
        let times: Vec<i64> = self
            .data_layers()
            .flat_map(|entry| entry.layer.resolved_times(zone))
            .chain(
                self.layers()
                    .flat_map(|layer| [&layer.time, &layer.from, &layer.to, &layer.step_end])
                    .chain([&self.time_axis.min, &self.time_axis.max])
                    .filter_map(Option::as_ref)
                    .filter_map(|time| time.resolve(zone).ok()),
            )
            .collect();
        let (Some(&min), Some(&max)) = (times.iter().min(), times.iter().max()) else {
            return Ok(());
        };
        let limit = 50;
        let ticks = crate::time::declared_tick_count(min, max, zone, step, limit);
        if (1..=limit).contains(&ticks) {
            return Ok(());
        }
        Err(ChartError::new(
            "invalid_axis_range",
            "/timeAxis/step",
            format!(
                "a step of {step} places {} on this time axis; choose one that places 1 to {limit} ticks",
                if ticks > limit {
                    format!("more than {limit} ticks")
                } else {
                    "no tick".to_owned()
                }
            ),
        ))
    }

    /// The declared ends and tick step of the time axis: ends that resolve, in order and not on a
    /// collapsed axis; a step above zero, whole years on a calendar axis.
    fn validate_time_axis_ends(&self, zone: crate::time::TimeZone) -> Result<(), ChartError> {
        let mut ends = [None, None];
        for (index, (field, bound)) in [("min", &self.time_axis.min), ("max", &self.time_axis.max)]
            .into_iter()
            .enumerate()
        {
            let Some(bound) = bound else {
                continue;
            };
            let path = format!("/timeAxis/{field}");
            if self.time_axis.gaps == Gaps::Collapse {
                return Err(ChartError::new(
                    "option_not_supported",
                    path,
                    "a collapsed axis runs from the first observation to the last",
                ));
            }
            ends[index] = Some(
                bound
                    .resolve(zone)
                    .map_err(|message| ChartError::new("invalid_time", path, message))?,
            );
        }
        if let [Some(min), Some(max)] = ends
            && min >= max
        {
            return Err(ChartError::new(
                "invalid_axis_range",
                "/timeAxis/max",
                "max must lie after min",
            ));
        }
        if let Some(step) = self.time_axis.step {
            let calendar = self.time_axis.kind != TimeAxisKind::Number;
            let valid = step.is_finite()
                && step > 0.0
                && (!calendar || (step.fract() == 0.0 && step <= 1_000.0));
            if !valid {
                return Err(ChartError::new(
                    "invalid_axis_range",
                    "/timeAxis/step",
                    if calendar {
                        "step on a calendar axis is a whole number of years, 1 to 1000"
                    } else {
                        "step must be a number above zero"
                    },
                ));
            }
        }
        if let Some(step) = self.time_axis.step {
            self.validate_time_tick_count(zone, step)?;
        }
        Ok(())
    }

    /// The time axis options both time charts and small multiples share.
    pub(super) fn validate_time_axis(&self) -> Result<crate::time::TimeZone, ChartError> {
        let zone = self.time_zone()?;
        validate_optional_text(self.time_axis.title.as_ref(), "/timeAxis/title", 100)?;
        self.validate_time_axis_ends(zone)?;
        if self.time_axis.kind == TimeAxisKind::Number {
            // A numeric axis has no calendar: no timezone, no calendar gaps to close, and zoom
            // steps name dates.
            for (field, present) in [
                (
                    "/timeAxis/timezone",
                    self.time_axis.timezone != default_timezone(),
                ),
                ("/timeAxis/gaps", self.time_axis.gaps != Gaps::Show),
                ("/timeAxis/precision", self.time_axis.precision.is_some()),
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
            if let Some(entry) = self
                .indexed_layers()
                .find(|entry| entry.layer.precision.is_some())
            {
                return Err(ChartError::new(
                    "option_not_supported",
                    format!("/panes/{}/layers/{}/precision", entry.pane, entry.local),
                    "a numeric axis writes its positions as they are; precision names calendar times",
                ));
            }
        }
        if let Some(entry) = self.indexed_layers().find(|entry| {
            entry.layer.step_end.is_some()
                && (entry.layer.curve != Curve::Step
                    || !matches!(entry.layer.mark, Mark::Line | Mark::Area)
                    || self.time_axis.gaps == Gaps::Collapse
                    || self.panes[entry.pane].stack.is_some())
        }) {
            return Err(ChartError::new(
                "option_not_supported",
                format!("/panes/{}/layers/{}/stepEnd", entry.pane, entry.local),
                "stepEnd belongs to a line or area layer with curve \"step\", not with collapsed gaps or in a stacked pane",
            ));
        }
        if let Some(entry) = self.indexed_layers().find(|entry| {
            entry.layer.markers.is_some() && !matches!(entry.layer.mark, Mark::Line | Mark::Area)
        }) {
            return Err(ChartError::new(
                "option_not_supported",
                format!("/panes/{}/layers/{}/markers", entry.pane, entry.local),
                "markers belongs to a line or area layer",
            ));
        }
        // Precision is how a data layer writes its times; annotations and zones name theirs.
        if let Some(entry) = self
            .indexed_layers()
            .find(|entry| entry.layer.precision.is_some() && !entry.layer.is_data())
        {
            return Err(ChartError::new(
                "option_not_supported",
                format!("/panes/{}/layers/{}/precision", entry.pane, entry.local),
                "precision belongs to a data layer",
            ));
        }
        Ok(zone)
    }

    /// With gaps collapsed, the time axis has a place only for observations: a reference line or
    /// point marker takes the first observation at or after its time, so its time has to lie
    /// between the first and the last observation, and a zone covers the observations between
    /// its edges, so it has to hold at least one.
    pub(super) fn validate_collapsed_times(
        &self,
        zone: crate::time::TimeZone,
    ) -> Result<(), ChartError> {
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

    /// Zoom steps of a time chart: two to four windows, each from one timestamp to a later one and
    /// holding at least two observations of some data layer.
    pub(super) fn validate_time_zoom(&self, zone: crate::time::TimeZone) -> Result<(), ChartError> {
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
}
