//! Calendar heatmaps: one cell per day of a year, on the diverging scale around a reference
//! value. Either twelve month rows of up to 31 days, or seven weekday rows of calendar weeks.
//! A day without a value keeps an empty, outlined cell, so a gap reads as a gap and not as the
//! reference value.

use std::collections::BTreeMap;

use crate::{
    diverging,
    error::ChartWarning,
    layout::{count, fit_text, format_value},
    metrics::TextMetrics,
    scene::{Element, Rect, Scene, Text, TextAnchor},
    spec::{CalendarLayout, CalendarSpec, ChartSpec, Diverging, ValueFormat, calendar_date},
    time::{civil_from_days, days_from_civil, days_in_month},
};

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
/// Left of the grid, for the month or weekday names.
const GUTTER: f64 = 44.0;
const MARGIN: f64 = 16.0;
/// Space between two cells.
const GAP: f64 = 2.0;
/// Below the grid, for the color key.
const KEY_SPACE: f64 = 48.0;

/// The values of a calendar by day of the year (0-based), with the number of days of the year.
fn values_by_day(calendar: &CalendarSpec) -> (BTreeMap<i64, f64>, i64) {
    let year = i64::from(calendar.year);
    let start = days_from_civil(year, 1, 1);
    let length = days_from_civil(year + 1, 1, 1) - start;
    let values = calendar
        .days
        .iter()
        .filter_map(|day| {
            let (_, month, day_of_month) = calendar_date(&day.date)?;
            Some((
                days_from_civil(year, month, day_of_month) - start,
                day.value?,
            ))
        })
        .collect();
    (values, length)
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let calendar = spec
        .calendar
        .as_ref()
        .expect("validated calendar charts carry a calendar block");
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let mut elements = vec![Element::Text(Text {
        x: MARGIN,
        y: 30.0,
        class: "chartlet-title",
        anchor: TextAnchor::Start,
        content: fit_text(
            &spec.title,
            width - 2.0 * MARGIN,
            22.0,
            metrics,
            warnings,
            "/title",
        ),
    })];
    let (values, length) = values_by_day(calendar);
    let year = i64::from(calendar.year);
    let cells = Cells {
        grid: Grid {
            left: MARGIN + GUTTER,
            top: 72.0,
            width: width - 2.0 * MARGIN - GUTTER,
            height: height - 72.0 - KEY_SPACE,
        },
        scale: calendar.diverging(),
        values,
        year,
        start: days_from_civil(year, 1, 1),
    };
    let labels = match calendar.layout {
        CalendarLayout::Months => cells.push_months(&mut elements),
        CalendarLayout::Weeks => cells.push_weeks(&mut elements, length),
    };
    elements.extend(labels.into_iter().map(Element::Text));
    diverging::push_key(
        &mut elements,
        cells.grid.left + cells.grid.width - diverging::key_width(),
        height - KEY_SPACE + 12.0,
        &cells.scale,
    );

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// The grid of a calendar and what its cells show.
struct Cells {
    grid: Grid,
    scale: Diverging,
    /// Values by day of the year, 0-based.
    values: BTreeMap<i64, f64>,
    year: i64,
    /// January 1st as days since 1970-01-01.
    start: i64,
}

impl Cells {
    /// One day's cell at column and row `at` of a grid of `size` columns × rows.
    fn push(&self, elements: &mut Vec<Element>, at: (i64, i64), size: (i64, i64), day: i64) {
        let size_x = self.grid.width / small(size.0);
        let size_y = self.grid.height / small(size.1);
        let (_, month, day_of_month) = civil_from_days(self.start + day);
        let date = format!("{:04}-{month:02}-{day_of_month:02}", self.year);
        let value = self.values.get(&day).copied();
        elements.push(Element::Rect(Rect {
            x: self.grid.left + size_x * small(at.0) + GAP / 2.0,
            y: self.grid.top + size_y * small(at.1) + GAP / 2.0,
            width: size_x - GAP,
            height: size_y - GAP,
            class: value.map_or("chartlet-calendar-empty", |value| {
                diverging::CLASSES[self.scale.step(value)]
            }),
            series_index: None,
            style_index: None,
            tooltip: Some(value.map_or_else(
                || format!("{date}: no value"),
                |value| format!("{date}: {}", format_value(value, ValueFormat::Number)),
            )),
        }));
    }

    /// Twelve month rows of up to 31 days; returns the month and day labels.
    fn push_months(&self, elements: &mut Vec<Element>) -> Vec<Text> {
        for month in 1..=12_u32 {
            for day_of_month in 1..=days_in_month(self.year, month) {
                let day = days_from_civil(self.year, month, day_of_month) - self.start;
                self.push(
                    elements,
                    (i64::from(day_of_month) - 1, i64::from(month) - 1),
                    (31, 12),
                    day,
                );
            }
        }
        let size_x = self.grid.width / 31.0;
        let size_y = self.grid.height / 12.0;
        let mut labels: Vec<Text> = MONTHS
            .iter()
            .enumerate()
            .map(|(index, name)| Text {
                x: self.grid.left - 8.0,
                y: self.grid.top + size_y * (count(index) + 0.5) + 4.0,
                class: "chartlet-label",
                anchor: TextAnchor::End,
                content: (*name).to_owned(),
            })
            .collect();
        labels.extend([1_u32, 5, 10, 15, 20, 25, 31].map(|day_of_month| Text {
            x: self.grid.left + size_x * (f64::from(day_of_month) - 0.5),
            y: self.grid.top - 8.0,
            class: "chartlet-tick",
            anchor: TextAnchor::Middle,
            content: day_of_month.to_string(),
        }));
        labels
    }

    /// Seven weekday rows, Monday first, and one column per calendar week; returns the weekday
    /// and month labels.
    fn push_weeks(&self, elements: &mut Vec<Element>, length: i64) -> Vec<Text> {
        // Monday is 0; 1970-01-01 was a Thursday.
        let first_weekday = (self.start + 3).rem_euclid(7);
        let columns = (length - 1 + first_weekday) / 7 + 1;
        for day in 0..length {
            let slot = day + first_weekday;
            self.push(elements, (slot / 7, slot % 7), (columns, 7), day);
        }
        let size_x = self.grid.width / small(columns);
        let size_y = self.grid.height / 7.0;
        let mut labels: Vec<Text> = WEEKDAYS
            .iter()
            .enumerate()
            .step_by(2)
            .map(|(index, name)| Text {
                x: self.grid.left - 8.0,
                y: self.grid.top + size_y * (count(index) + 0.5) + 4.0,
                class: "chartlet-label",
                anchor: TextAnchor::End,
                content: (*name).to_owned(),
            })
            .collect();
        labels.extend(MONTHS.iter().enumerate().map(|(index, name)| {
            let month = u32::try_from(index + 1).expect("twelve months");
            let column = (days_from_civil(self.year, month, 1) - self.start + first_weekday) / 7;
            Text {
                x: self.grid.left + size_x * small(column),
                y: self.grid.top - 8.0,
                class: "chartlet-tick",
                anchor: TextAnchor::Start,
                content: (*name).to_owned(),
            }
        }));
        labels
    }
}

/// A small, non-negative grid coordinate as a float.
fn small(value: i64) -> f64 {
    count(usize::try_from(value).expect("grid coordinates are small and positive"))
}

struct Grid {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

/// What the calendar shows in one sentence: the year, the coverage, the scale and the extremes.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let calendar = spec
        .calendar
        .as_ref()
        .expect("validated calendar charts carry a calendar block");
    let scale = calendar.diverging();
    let show = |value| format_value(value, ValueFormat::Number);
    let (values, length) = values_by_day(calendar);
    let days: Vec<(&str, f64)> = calendar
        .sorted_days()
        .into_iter()
        .filter_map(|day| day.value.map(|value| (day.date.as_str(), value)))
        .collect();
    let lowest = days
        .iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("validated calendars hold a value");
    let highest = days
        .iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("validated calendars hold a value");
    let arrangement = match calendar.layout {
        CalendarLayout::Months => "one row per month",
        CalendarLayout::Weeks => "one row per weekday and one column per week",
    };
    let missing = length - i64::try_from(values.len()).expect("a year has at most 366 days");
    format!(
        "Calendar of {} with {arrangement}, on a diverging color scale around {} with its outermost steps at {} and {}. {} of {length} days have a value. Lowest: {} ({}). Highest: {} ({}).",
        calendar.year,
        show(scale.reference),
        show(scale.min),
        show(scale.max),
        length - missing,
        show(lowest.1),
        lowest.0,
        show(highest.1),
        highest.0,
    )
}
