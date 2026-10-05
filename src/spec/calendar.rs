use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    ChartSpec,
    stripes::{Diverging, validate_diverging},
    validate_number,
};
use crate::error::{ChartError, ChartWarning};

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

impl CalendarSpec {
    /// The days in calendar order. Validation guarantees every date is a unique ISO date of the
    /// calendar's year, so the text sorts like the date.
    pub(crate) fn sorted_days(&self) -> Vec<&CalendarDay> {
        let mut days: Vec<&CalendarDay> = self.days.iter().collect();
        crate::sort::by(&mut days, |a, b| a.date.cmp(&b.date));
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

impl ChartSpec {
    /// A calendar heatmap: dated values of one year and a diverging scale.
    pub(super) fn validate_calendar(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
