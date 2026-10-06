use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_SERIES, sequence::is_identifier, validate_text};
use crate::{
    error::{ChartError, ChartWarning},
    time::{TimeValue, TimeZone},
};

/// How many items and markers a timeline holds.
pub(crate) const MAX_ITEMS: usize = 60;
pub(crate) const MAX_MARKERS: usize = 6;

/// A timeline: phases that run from a start to an end, milestones on a day, lines for days that
/// matter, and which item follows which. Gantt charts, roadmaps, the course of a procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimelineSpec {
    pub items: Vec<TimelineItemSpec>,
    /// Days drawn as a line across the whole timeline, such as today or a deadline.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<MarkerSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TimelineItemSpec {
    /// Names the item for `after`; needed only by an item another one follows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub label: String,
    /// A phase runs from `start` to `end`, a milestone happens `at` a day: one or the other.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<TimeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<TimeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<TimeValue>,
    /// Items of one group share a color and an entry in the legend: either every item names a
    /// group or none does, and there are at most four.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// The `id`s of the items this one follows: an arrow runs from each of them to it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub after: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkerSpec {
    pub label: String,
    pub at: TimeValue,
}

/// An item with its days resolved to Unix seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Span {
    Phase { start: i64, end: i64 },
    Milestone { at: i64 },
}

impl Span {
    pub(crate) const fn begin(self) -> i64 {
        match self {
            Self::Phase { start, .. } => start,
            Self::Milestone { at } => at,
        }
    }

    pub(crate) const fn finish(self) -> i64 {
        match self {
            Self::Phase { end, .. } => end,
            Self::Milestone { at } => at,
        }
    }
}

pub(crate) const fn zone() -> TimeZone {
    TimeZone::utc()
}

impl TimelineItemSpec {
    /// The days of the item; only on a validated timeline.
    pub(crate) fn span(&self) -> Span {
        let resolve = |value: &Option<TimeValue>| {
            value
                .as_ref()
                .expect("validated")
                .resolve(zone())
                .expect("validated")
        };
        match (&self.start, &self.end) {
            (Some(_), Some(_)) => Span::Phase {
                start: resolve(&self.start),
                end: resolve(&self.end),
            },
            _ => Span::Milestone {
                at: resolve(&self.at),
            },
        }
    }
}

impl TimelineSpec {
    pub(crate) fn item(&self, id: &str) -> Option<usize> {
        self.items
            .iter()
            .position(|item| item.id.as_deref() == Some(id))
    }

    /// The groups in the order they first appear.
    pub(crate) fn groups(&self) -> Vec<&str> {
        let mut groups: Vec<&str> = Vec::new();
        for group in self.items.iter().filter_map(|item| item.group.as_deref()) {
            if !groups.contains(&group) {
                groups.push(group);
            }
        }
        groups
    }
}

impl ChartSpec {
    /// Items that are a phase or a milestone, with unique ids, groups all or none, and arrows
    /// between existing items that run in no circle.
    pub(super) fn validate_timeline(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a timeline", "timeline.items")?;
        let Some(timeline) = &self.timeline else {
            return Err(ChartError::new(
                "missing_timeline",
                "/timeline",
                "a timeline requires a timeline block",
            ));
        };
        if !self.time_axis.is_default() {
            return Err(ChartError::new(
                "option_not_supported",
                "/timeAxis",
                "a timeline reads its days from its items; remove this field",
            ));
        }
        if timeline.items.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/timeline/items",
                "provide at least one item",
            ));
        }
        if timeline.items.len() > MAX_ITEMS {
            return Err(ChartError::new(
                "too_many_nodes",
                "/timeline/items",
                format!("at most {MAX_ITEMS} items are supported"),
            ));
        }
        let mut warnings = Vec::new();
        timeline.validate_items(&mut warnings)?;
        timeline.validate_markers()?;
        timeline.validate_follows(&mut warnings)?;
        Ok(warnings)
    }
}

impl TimelineSpec {
    fn validate_items(&self, warnings: &mut Vec<ChartWarning>) -> Result<(), ChartError> {
        let grouped = self.items.iter().any(|item| item.group.is_some());
        let mut groups: Vec<&str> = Vec::new();
        for (index, item) in self.items.iter().enumerate() {
            let path = format!("/timeline/items/{index}");
            validate_text(&item.label, &format!("{path}/label"), 100)?;
            if let Some(id) = &item.id {
                if !is_identifier(id) {
                    return Err(ChartError::new(
                        "invalid_id",
                        format!("{path}/id"),
                        "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                    ));
                }
                if self.items[..index]
                    .iter()
                    .any(|other| other.id.as_deref() == Some(id))
                {
                    return Err(ChartError::new(
                        "duplicate_id",
                        format!("{path}/id"),
                        format!("the id \"{id}\" is already taken by an earlier item"),
                    ));
                }
            }
            let day = |value: &Option<TimeValue>, name: &str| -> Result<Option<i64>, ChartError> {
                value
                    .as_ref()
                    .map(|value| {
                        value.resolve(zone()).map_err(|message| {
                            ChartError::new("invalid_time", format!("{path}/{name}"), message)
                        })
                    })
                    .transpose()
            };
            let (start, end, at) = (
                day(&item.start, "start")?,
                day(&item.end, "end")?,
                day(&item.at, "at")?,
            );
            match (start, end, at) {
                (Some(start), Some(end), None) => {
                    if end < start {
                        return Err(ChartError::new(
                            "invalid_range",
                            format!("{path}/end"),
                            "end must not be before start",
                        ));
                    }
                }
                (None, None, Some(_)) => {}
                _ => {
                    return Err(ChartError::new(
                        "invalid_item",
                        path,
                        "give a phase both start and end, or a milestone at; not a mix",
                    ));
                }
            }
            match (&item.group, grouped) {
                (Some(group), _) => {
                    validate_text(group, &format!("{path}/group"), 60)?;
                    if !groups.contains(&group.as_str()) {
                        groups.push(group);
                    }
                    if groups.len() > MAX_SERIES {
                        return Err(ChartError::new(
                            "too_many_series",
                            format!("{path}/group"),
                            format!(
                                "at most {MAX_SERIES} groups are supported, one per palette color"
                            ),
                        ));
                    }
                }
                (None, true) => {
                    return Err(ChartError::new(
                        "missing_group",
                        format!("{path}/group"),
                        "give every item a group, or none",
                    ));
                }
                (None, false) => {}
            }
        }
        if self.items.len() > 30 {
            warnings.push(ChartWarning::new(
                "dense_chart",
                "/timeline/items",
                "more than 30 items need a tall canvas to be read",
            ));
        }
        Ok(())
    }

    fn validate_markers(&self) -> Result<(), ChartError> {
        if self.markers.len() > MAX_MARKERS {
            return Err(ChartError::new(
                "too_many_markers",
                "/timeline/markers",
                format!("at most {MAX_MARKERS} markers are supported"),
            ));
        }
        for (index, marker) in self.markers.iter().enumerate() {
            let path = format!("/timeline/markers/{index}");
            validate_text(&marker.label, &format!("{path}/label"), 60)?;
            marker.at.resolve(zone()).map_err(|message| {
                ChartError::new("invalid_time", format!("{path}/at"), message)
            })?;
        }
        Ok(())
    }

    /// Every `after` names another item, and following runs in no circle. An item that starts
    /// before the one it follows ends is a warning: its arrow runs backwards.
    fn validate_follows(&self, warnings: &mut Vec<ChartWarning>) -> Result<(), ChartError> {
        for (index, item) in self.items.iter().enumerate() {
            for (at, id) in item.after.iter().enumerate() {
                let path = format!("/timeline/items/{index}/after/{at}");
                let Some(before) = self.item(id) else {
                    return Err(ChartError::new(
                        "unknown_node",
                        path,
                        format!("no item has the id \"{id}\""),
                    ));
                };
                if before == index {
                    return Err(ChartError::new(
                        "circular_dependency",
                        path,
                        "an item cannot follow itself",
                    ));
                }
                if item.span().begin() < self.items[before].span().finish() {
                    warnings.push(ChartWarning::new(
                        "follows_overlap",
                        path,
                        format!("this item starts before \"{id}\" ends; the arrow runs backwards"),
                    ));
                }
            }
        }
        // A circle of items that follow each other: walk the arrows from every item.
        for start in 0..self.items.len() {
            let mut seen = vec![false; self.items.len()];
            let mut stack = vec![start];
            while let Some(at) = stack.pop() {
                for id in &self.items[at].after {
                    let next = self.item(id).expect("checked above");
                    if next == start {
                        return Err(ChartError::new(
                            "circular_dependency",
                            format!("/timeline/items/{start}/after"),
                            "these items follow each other in a circle",
                        ));
                    }
                    if !seen[next] {
                        seen[next] = true;
                        stack.push(next);
                    }
                }
            }
        }
        Ok(())
    }
}
