//! Story maps: a backbone of activities across the top, the user tasks under each, and below them
//! the stories of every task in release slices, each slice opened by a dashed cut line with the
//! name of the release. Where the tasks have room for 110 pixels each they stand as columns; on a
//! narrower canvas the map reads as a list: each activity, its tasks, and under each task its
//! stories with the release written beside them. The description and the data table name every
//! story with its activity, task and release.

use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::{pixels, warn_growth_advising, wrap},
    draw::{line, rect, text},
    error::ChartWarning,
    layout::{count, fit_text, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Scene, TextAnchor},
    spec::{ChartSpec, StorymapSpec},
    text,
};

const SIZE: f64 = 12.0;
const LINE: f64 = 14.0;
const PAD: f64 = 7.0;
const GAP: f64 = 8.0;
/// The narrowest a task column may be, and the widest.
const MIN_COLUMN: f64 = 110.0;
const MAX_COLUMN: f64 = 220.0;
/// The band of a slice that holds the name of its release and the cut line.
const CUT: f64 = 26.0;

fn storymap(spec: &ChartSpec) -> &StorymapSpec {
    spec.storymap
        .as_ref()
        .expect("validated story maps carry a storymap block")
}

/// A box with its label on up to two lines, `width` wide: its lines and its height.
fn boxed(
    label: &str,
    width: f64,
    path: &str,
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) -> (Vec<String>, f64) {
    let lines = wrap(label, width - 2.0 * PAD, SIZE, metrics, warnings, path);
    let height = 2.0 * PAD + LINE * count(lines.len());
    (lines, height)
}

/// Draws a box at `x`, `y` with its lines of text.
fn push_box(
    elements: &mut Vec<Element>,
    (lines, height): (&[String], f64),
    (x, y, width): (f64, f64, f64),
    (class, text_class): (&'static str, &'static str),
    tooltip: &str,
) {
    elements.push(rect((x, y, width, height), class, Some(tooltip.to_owned())));
    for (position, content) in lines.iter().enumerate() {
        elements.push(text(
            (x + PAD, y + PAD + 11.0 + LINE * count(position)),
            TextAnchor::Start,
            text_class,
            content.clone(),
        ));
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let map = storymap(spec);
    let compact = spec.width < crate::layout::NARROW;
    let margin = if compact { 12.0 } else { 24.0 };
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        width - 2.0 * margin,
        metrics,
        warnings,
    );
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let inner = width - 2.0 * margin;
    let tasks = map.task_count();
    let column = ((inner - GAP * (count(tasks) - 1.0)) / count(tasks)).floor();
    let needed = if column >= MIN_COLUMN {
        columns(
            spec,
            (margin, top, column.min(MAX_COLUMN)),
            (warnings, metrics),
            &mut elements,
        )
    } else {
        list(
            spec,
            (margin, top, inner),
            (warnings, metrics),
            &mut elements,
        )
    };
    warn_growth_advising(
        spec,
        (width, needed),
        "shorten the labels or split the map",
        warnings,
    );
    Scene {
        width: spec.width,
        height: pixels(needed.max(height)),
        elements,
    }
}

/// The map as columns; returns the height it needs.
#[allow(clippy::too_many_lines)]
fn columns(
    spec: &ChartSpec,
    (margin, top, column): (f64, f64, f64),
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
    elements: &mut Vec<Element>,
) -> f64 {
    let map = storymap(spec);
    let width = f64::from(spec.width);
    let tasks = map.task_count();
    let used = column * count(tasks) + GAP * (count(tasks) - 1.0);
    let left = margin + ((width - 2.0 * margin - used) / 2.0).floor().max(0.0);
    let x_of = |task: usize| left + (column + GAP) * count(task);

    // The backbone: each activity spans the columns of its tasks.
    let mut first = 0;
    let mut backbone = Vec::new();
    for (index, activity) in map.activities.iter().enumerate() {
        let span = column * count(activity.tasks.len()) + GAP * (count(activity.tasks.len()) - 1.0);
        let (lines, height) = boxed(
            &activity.label,
            span,
            &format!("/storymap/activities/{index}/label"),
            (warnings, metrics),
        );
        backbone.push((first, span, lines, height));
        first += activity.tasks.len();
    }
    let activity_height = backbone.iter().map(|entry| entry.3).fold(30.0, f64::max);
    for (index, (first, span, lines, _)) in backbone.iter().enumerate() {
        push_box(
            elements,
            (lines, activity_height),
            (x_of(*first), top, *span),
            ("chartlet-story-activity", "chartlet-story-activity-text"),
            &map.activities[index].label,
        );
    }
    // The tasks under them.
    let mut task_boxes = Vec::new();
    let mut at = 0;
    for (activity_index, activity) in map.activities.iter().enumerate() {
        for (task_index, task) in activity.tasks.iter().enumerate() {
            let (lines, height) = boxed(
                &task.label,
                column,
                &format!("/storymap/activities/{activity_index}/tasks/{task_index}/label"),
                (warnings, metrics),
            );
            task_boxes.push((at, lines, height, &task.label));
            at += 1;
        }
    }
    let task_height = task_boxes.iter().map(|entry| entry.2).fold(30.0, f64::max);
    let task_top = top + activity_height + GAP;
    for (at, lines, _, label) in &task_boxes {
        push_box(
            elements,
            (lines, task_height),
            (x_of(*at), task_top, column),
            ("chartlet-story-task", "chartlet-story-task-text"),
            label,
        );
    }

    // The slices: for each release a cut line, and the stories of every task in their column.
    let mut y = task_top + task_height + GAP * 2.0;
    for (release_index, release) in map.releases.iter().enumerate() {
        elements.push(text(
            (left, y + 15.0),
            TextAnchor::Start,
            "chartlet-story-release",
            fit_text(
                release,
                used,
                SIZE,
                metrics,
                warnings,
                &format!("/storymap/releases/{release_index}"),
            ),
        ));
        elements.push(line(
            (left, y + CUT - 4.0),
            (left + used, y + CUT - 4.0),
            "chartlet-story-cut",
        ));
        let body_top = y + CUT;
        let mut slice_height: f64 = 36.0;
        let mut task_at = 0;
        for (activity_index, activity) in map.activities.iter().enumerate() {
            for (task_index, task) in activity.tasks.iter().enumerate() {
                let mut card_y = body_top;
                for (story_index, story) in task.stories.iter().enumerate() {
                    if story.release != *release {
                        continue;
                    }
                    let (lines, card_height) = boxed(
                        &story.title,
                        column,
                        &format!(
                            "/storymap/activities/{activity_index}/tasks/{task_index}/stories/{story_index}/title"
                        ),
                        (warnings, metrics),
                    );
                    push_box(
                        elements,
                        (&lines, card_height),
                        (x_of(task_at), card_y, column),
                        ("chartlet-story-card", "chartlet-story-card-text"),
                        &story.title,
                    );
                    card_y += card_height + GAP;
                }
                slice_height = slice_height.max(card_y - body_top);
                task_at += 1;
            }
        }
        y = body_top + slice_height + GAP;
    }
    y + 8.0
}

/// The map as a list; returns the height it needs.
fn list(
    spec: &ChartSpec,
    (margin, top, inner): (f64, f64, f64),
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
    elements: &mut Vec<Element>,
) -> f64 {
    let map = storymap(spec);
    let mut y = top;
    for (activity_index, activity) in map.activities.iter().enumerate() {
        let (lines, height) = boxed(
            &activity.label,
            inner,
            &format!("/storymap/activities/{activity_index}/label"),
            (warnings, metrics),
        );
        push_box(
            elements,
            (&lines, height),
            (margin, y, inner),
            ("chartlet-story-activity", "chartlet-story-activity-text"),
            &activity.label,
        );
        y += height + GAP;
        for (task_index, task) in activity.tasks.iter().enumerate() {
            let indent = 12.0;
            let (lines, height) = boxed(
                &task.label,
                inner - indent,
                &format!("/storymap/activities/{activity_index}/tasks/{task_index}/label"),
                (warnings, metrics),
            );
            push_box(
                elements,
                (&lines, height),
                (margin + indent, y, inner - indent),
                ("chartlet-story-task", "chartlet-story-task-text"),
                &task.label,
            );
            y += height + GAP;
            // The stories, in the order of the releases; the release stands beside each.
            for release in &map.releases {
                for (story_index, story) in task
                    .stories
                    .iter()
                    .enumerate()
                    .filter(|(_, story)| story.release == *release)
                {
                    let tag =
                        fit_text(release, 80.0, SIZE, metrics, warnings, "/storymap/releases");
                    let tag_width = metrics.width(&tag, SIZE);
                    let (lines, height) = boxed(
                        &story.title,
                        inner - 2.0 * indent - tag_width - 2.0 * PAD,
                        &format!(
                            "/storymap/activities/{activity_index}/tasks/{task_index}/stories/{story_index}/title"
                        ),
                        (warnings, metrics),
                    );
                    let x = margin + 2.0 * indent;
                    push_box(
                        elements,
                        (&lines, height),
                        (x, y, inner - 2.0 * indent),
                        ("chartlet-story-card", "chartlet-story-card-text"),
                        &story.title,
                    );
                    elements.push(text(
                        (x + inner - 2.0 * indent - PAD, y + PAD + 11.0),
                        TextAnchor::End,
                        "chartlet-story-release",
                        tag,
                    ));
                    y += height + GAP;
                }
            }
        }
        y += GAP;
    }
    y + 8.0
}

/// The map in words: its size, then each activity with its tasks and, by release, their stories.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let map = storymap(spec);
    let mut summary = text::storymap_opening(
        spec.locale,
        (
            map.activities.len(),
            map.task_count(),
            map.story_count(),
            map.releases.len(),
        ),
    );
    for activity in &map.activities {
        let tasks: Vec<String> = activity
            .tasks
            .iter()
            .map(|task| {
                let by_release: Vec<String> = map
                    .releases
                    .iter()
                    .filter_map(|release| {
                        let titles: Vec<&str> = task
                            .stories
                            .iter()
                            .filter(|story| story.release == *release)
                            .map(|story| story.title.as_str())
                            .collect();
                        (!titles.is_empty()).then(|| format!("{release}: {}", titles.join(", ")))
                    })
                    .collect();
                if by_release.is_empty() {
                    task.label.clone()
                } else {
                    format!("{} ({})", task.label, by_release.join("; "))
                }
            })
            .collect();
        let _ = write!(summary, " {}: {}.", activity.label, tasks.join("; "));
    }
    summary
}

/// One row per story with its activity, task and release, in the order of the map: by activity,
/// task, then release. A task without stories has a row of its own.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let map = storymap(spec);
    let mut rows = Vec::new();
    for activity in &map.activities {
        for task in &activity.tasks {
            if task.stories.is_empty() {
                rows.push(vec![
                    activity.label.clone(),
                    task.label.clone(),
                    String::new(),
                    String::new(),
                ]);
            }
            for release in &map.releases {
                for story in task
                    .stories
                    .iter()
                    .filter(|story| story.release == *release)
                {
                    rows.push(vec![
                        activity.label.clone(),
                        task.label.clone(),
                        release.clone(),
                        story.title.clone(),
                    ]);
                }
            }
        }
    }
    DataTable {
        caption: format!("{} {}", spec.locale.words().data_for, spec.title),
        columns: text::storymap_columns(spec.locale)
            .iter()
            .map(|column| (*column).to_owned())
            .collect(),
        rows,
    }
}
