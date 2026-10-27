use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many releases, activities, tasks and stories a story map may have.
const MAX_RELEASES: usize = 5;
const MAX_ACTIVITIES: usize = 8;
const MAX_TASKS: usize = 16;
const MAX_STORIES: usize = 12;

/// A user story map: a backbone of activities, the user tasks under each, and the stories of every
/// task in slices by release.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorymapSpec {
    /// The release slices from the top down, such as `MVP` and `Release 2`; unique.
    pub releases: Vec<String>,
    pub activities: Vec<StorymapActivitySpec>,
}

/// An activity of the backbone, with the tasks a user does to carry it out.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorymapActivitySpec {
    pub label: String,
    pub tasks: Vec<StorymapTaskSpec>,
}

/// A user task and the stories that make it possible.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorymapTaskSpec {
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stories: Vec<StorymapStorySpec>,
}

/// A story and the release it belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorymapStorySpec {
    pub title: String,
    pub release: String,
}

impl StorymapSpec {
    /// How many tasks the map has in all.
    pub(crate) fn task_count(&self) -> usize {
        self.activities
            .iter()
            .map(|activity| activity.tasks.len())
            .sum()
    }

    /// How many stories the map has in all.
    pub(crate) fn story_count(&self) -> usize {
        self.activities
            .iter()
            .flat_map(|activity| &activity.tasks)
            .map(|task| task.stories.len())
            .sum()
    }
}

impl ChartSpec {
    /// Releases and activities with unique labels, tasks under each activity, and stories that name
    /// a release of the map.
    pub(super) fn validate_storymap(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_chart_fields("a story map is drawn from its releases and activities")?;
        let Some(map) = &self.storymap else {
            return Err(ChartError::new(
                "missing_storymap",
                "/storymap",
                "a story map requires a storymap block",
            ));
        };
        for (path, empty) in [
            ("/storymap/releases", map.releases.is_empty()),
            ("/storymap/activities", map.activities.is_empty()),
        ] {
            if empty {
                return Err(ChartError::new(
                    "empty_data",
                    path,
                    "provide at least one entry",
                ));
            }
        }
        for (path, length, limit) in [
            ("/storymap/releases", map.releases.len(), MAX_RELEASES),
            ("/storymap/activities", map.activities.len(), MAX_ACTIVITIES),
            ("/storymap/activities", map.task_count(), MAX_TASKS),
        ] {
            if length > limit {
                return Err(ChartError::new(
                    "too_many_nodes",
                    path,
                    format!("at most {limit} are supported here; merge or split the map"),
                ));
            }
        }
        let mut releases = BTreeSet::new();
        for (index, release) in map.releases.iter().enumerate() {
            validate_text(release, &format!("/storymap/releases/{index}"), 60)?;
            if !releases.insert(release.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("/storymap/releases/{index}"),
                    "release names must be unique",
                ));
            }
        }
        let mut activities = BTreeSet::new();
        for (index, activity) in map.activities.iter().enumerate() {
            let path = format!("/storymap/activities/{index}");
            validate_text(&activity.label, &format!("{path}/label"), 100)?;
            if !activities.insert(activity.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "activity labels must be unique",
                ));
            }
            if activity.tasks.is_empty() {
                return Err(ChartError::new(
                    "empty_data",
                    format!("{path}/tasks"),
                    "an activity needs at least one task",
                ));
            }
            for (task_index, task) in activity.tasks.iter().enumerate() {
                let task_path = format!("{path}/tasks/{task_index}");
                validate_text(&task.label, &format!("{task_path}/label"), 100)?;
                if task.stories.len() > MAX_STORIES {
                    return Err(ChartError::new(
                        "too_many_nodes",
                        format!("{task_path}/stories"),
                        format!("at most {MAX_STORIES} stories per task are supported"),
                    ));
                }
                for (story_index, story) in task.stories.iter().enumerate() {
                    let story_path = format!("{task_path}/stories/{story_index}");
                    validate_text(&story.title, &format!("{story_path}/title"), 100)?;
                    if !releases.contains(story.release.as_str()) {
                        return Err(ChartError::new(
                            "unknown_release",
                            format!("{story_path}/release"),
                            "a story names a release from /storymap/releases",
                        ));
                    }
                }
            }
        }
        Ok(Vec::new())
    }
}
