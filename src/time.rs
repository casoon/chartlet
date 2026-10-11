//! The time contract of `type: "time"` charts: a timestamp is either Unix seconds or an ISO 8601
//! date, the timezone option is a fixed UTC offset, and axis ticks sit on calendar boundaries.
//!
//! No timezone database is involved: `"timezone"` accepts `UTC` or a fixed offset such as
//! `+02:00`, so rendering stays deterministic and dependency-free. A bare ISO date is read as
//! wall-clock time in that offset; a date that carries `Z` or its own offset is absolute.

use serde::{Deserialize, Serialize};

use crate::spec::{Locale, NumberStyle};

/// A numeric axis keeps its values as whole millionths, so that it runs through the same integer
/// arithmetic as timestamps: 0.0117 Ma becomes 11 700.
pub(crate) const NUMERIC_UNITS: f64 = 1_000_000.0;
/// The largest magnitude a numeric axis takes; with [`NUMERIC_UNITS`] it stays far inside `i64`.
const MAX_NUMERIC: f64 = 1e9;

/// Supported range, 1700-01-01 to 2200-01-01 as Unix seconds. Keeps the civil-date arithmetic
/// and every tick label far away from the edges of `i64`. The lower bound reaches back before
/// the instrumental climate record (1850) and the pre-industrial reference (1750).
pub(crate) const MIN_TIMESTAMP: i64 = -8_520_336_000;
pub(crate) const MAX_TIMESTAMP: i64 = 7_258_118_400;

const SECONDS_PER_DAY: i64 = 86_400;
const SECONDS_PER_HOUR: i64 = 3_600;
/// 1969-12-29 was a Monday, so weekly ticks start on Mondays instead of on 1970-01-01.
const MONDAY_EPOCH: i64 = -3 * SECONDS_PER_DAY;

/// One timestamp as written in a specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TimeValue {
    /// Unix seconds; a fractional value is rejected by validation with a precise message.
    Number(f64),
    /// ISO 8601 date, optionally with a time and an offset.
    Text(String),
}

impl TimeValue {
    /// Resolves the timestamp to Unix seconds. The error text names the accepted forms.
    pub(crate) fn resolve(&self, zone: TimeZone) -> Result<i64, &'static str> {
        if zone.is_numeric() {
            return match self {
                Self::Number(number) if number.is_finite() && number.abs() <= MAX_NUMERIC => {
                    // Bounded by MAX_NUMERIC, the product stays far inside i64.
                    #[allow(clippy::cast_possible_truncation)]
                    let units = (number * NUMERIC_UNITS).round() as i64;
                    Ok(units)
                }
                Self::Number(_) => Err("a numeric axis takes numbers up to 1e9 in magnitude"),
                Self::Text(_) => Err("a numeric axis takes numbers such as 66 or 0.0117"),
            };
        }
        let epoch = match self {
            Self::Number(number) => {
                if !number.is_finite() || number.fract() != 0.0 {
                    return Err("Unix seconds must be a whole number");
                }
                // A whole number that survives the range check below stays inside the 1700–2200
                // contract, which is orders of magnitude below 2^53: the conversion is exact.
                #[allow(clippy::cast_possible_truncation)]
                let epoch = *number as i64;
                epoch
            }
            Self::Text(text) => parse_iso(text, zone).ok_or(
                "expected an ISO 8601 date such as 2026-03-01 or 2026-03-01T12:00:00Z, a month such as 2026-03, or a year such as 1850",
            )?,
        };
        if !(MIN_TIMESTAMP..MAX_TIMESTAMP).contains(&epoch) {
            return Err("timestamps must lie between 1700-01-01 and 2200-01-01");
        }
        Ok(epoch)
    }
}

/// How the positions of a time axis are read and written: as timestamps in a fixed offset from
/// UTC, or as plain numbers on a numeric axis (distance, depth, millions of years); either way
/// written in `locale`, ISO dates in English and `31.08.2026` in German.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct TimeZone {
    offset: i64,
    numeric: bool,
    locale: Locale,
    /// The rule of daylight saving time that adds an hour to `offset`, for a named zone.
    saving: Saving,
}

/// When a zone moves its clocks an hour ahead, by the rules in force today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Saving {
    #[default]
    None,
    /// From the last Sunday of March, 01:00 UTC, to the last Sunday of October, 01:00 UTC.
    Europe,
    /// From the second Sunday of March, 02:00 local time, to the first Sunday of November,
    /// 02:00 local daylight time.
    America,
    /// From the first Sunday of October, 02:00 local standard time, to the first Sunday of April,
    /// 03:00 local daylight time (southern Australia).
    Australia,
    /// From the last Sunday of September, 02:00 local standard time, to the first Sunday of April,
    /// 03:00 local daylight time (New Zealand).
    NewZealand,
}

impl TimeZone {
    pub(crate) const fn utc() -> Self {
        Self {
            offset: 0,
            numeric: false,
            locale: Locale::En,
            saving: Saving::None,
        }
    }

    /// A numeric axis whose numbers are written in `locale`.
    pub(crate) const fn numeric(locale: Locale) -> Self {
        Self {
            offset: 0,
            numeric: true,
            locale,
            saving: Saving::None,
        }
    }

    /// The same offset, its dates written in `locale`.
    pub(crate) const fn with_locale(self, locale: Locale) -> Self {
        Self { locale, ..self }
    }

    pub(crate) const fn is_numeric(self) -> bool {
        self.numeric
    }

    pub(crate) const fn offset(self) -> i64 {
        self.offset
    }

    /// The offset from UTC at the instant `epoch`: the standard offset, and an hour more while
    /// daylight saving time is in force.
    pub(crate) fn offset_at(self, epoch: i64) -> i64 {
        let year = civil_from_days(epoch.div_euclid(SECONDS_PER_DAY)).0;
        let day = |month: u32, nth: i64, last: bool| nth_sunday(year, month, nth, last);
        let at = |days: i64, seconds: i64| days * SECONDS_PER_DAY + seconds;
        let (start, end, southern) = match self.saving {
            Saving::None => return self.offset,
            Saving::Europe => (
                at(day(3, 1, true), SECONDS_PER_HOUR),
                at(day(10, 1, true), SECONDS_PER_HOUR),
                false,
            ),
            Saving::America => (
                at(day(3, 2, false), 2 * SECONDS_PER_HOUR - self.offset),
                at(
                    day(11, 1, false),
                    2 * SECONDS_PER_HOUR - self.offset - SECONDS_PER_HOUR,
                ),
                false,
            ),
            Saving::Australia => (
                at(day(10, 1, false), 2 * SECONDS_PER_HOUR - self.offset),
                at(day(4, 1, false), 2 * SECONDS_PER_HOUR - self.offset),
                true,
            ),
            Saving::NewZealand => (
                at(day(9, 1, true), 2 * SECONDS_PER_HOUR - self.offset),
                at(day(4, 1, false), 2 * SECONDS_PER_HOUR - self.offset),
                true,
            ),
        };
        let saving = if southern {
            epoch >= start || epoch < end
        } else {
            (start..end).contains(&epoch)
        };
        self.offset + if saving { SECONDS_PER_HOUR } else { 0 }
    }

    /// Parses a zone for a timeline: `UTC`, a fixed offset, or an IANA name from the table of
    /// common zones, such as `Europe/Berlin`.
    pub(crate) fn parse_named(text: &str) -> Option<Self> {
        Self::parse(text).or_else(|| named(text.trim()))
    }

    /// Parses `UTC`, `Z` or a fixed offset such as `+02:00`, `+0200`, `-05:00`.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let trimmed = text.trim();
        if trimmed.eq_ignore_ascii_case("utc") || trimmed.eq_ignore_ascii_case("z") {
            return Some(Self::utc());
        }
        let (sign, rest) = match trimmed.as_bytes().first()? {
            b'+' => (1_i64, &trimmed[1..]),
            b'-' => (-1_i64, &trimmed[1..]),
            _ => return None,
        };
        let (hours, minutes) = match rest.len() {
            4 => (&rest[..2], &rest[2..]),
            5 if rest.as_bytes()[2] == b':' => (&rest[..2], &rest[3..]),
            _ => return None,
        };
        let hours: i64 = hours.parse().ok()?;
        let minutes: i64 = minutes.parse().ok()?;
        if hours > 14 || minutes > 59 {
            return None;
        }
        Some(Self {
            offset: sign * (hours * SECONDS_PER_HOUR + minutes * 60),
            ..Self::utc()
        })
    }
}

/// The day number (days since 1970-01-01) of a Sunday of `month`: the `nth` counted from the
/// first, or the last one.
fn nth_sunday(year: i64, month: u32, nth: i64, last: bool) -> i64 {
    // 1970-01-01 was a Thursday: day + 4, modulo 7, is 0 on a Sunday.
    let sunday_on_or_after = |day: i64| day + (7 - (day + 4).rem_euclid(7)) % 7;
    if last {
        let end = days_from_civil(year, month, days_in_month(year, month));
        end - (end + 4).rem_euclid(7)
    } else {
        sunday_on_or_after(days_from_civil(year, month, 1)) + 7 * (nth - 1)
    }
}

/// The zones the table knows: name, standard offset in minutes, and rule of daylight saving
/// time. The rules are those in force today; earlier years may differ.
const ZONES: &[(&str, i32, Saving)] = &[
    ("GMT", 0, Saving::None),
    ("Europe/London", 0, Saving::Europe),
    ("Europe/Dublin", 0, Saving::Europe),
    ("Europe/Lisbon", 0, Saving::Europe),
    ("Atlantic/Reykjavik", 0, Saving::None),
    ("Europe/Berlin", 60, Saving::Europe),
    ("Europe/Paris", 60, Saving::Europe),
    ("Europe/Madrid", 60, Saving::Europe),
    ("Europe/Rome", 60, Saving::Europe),
    ("Europe/Amsterdam", 60, Saving::Europe),
    ("Europe/Brussels", 60, Saving::Europe),
    ("Europe/Vienna", 60, Saving::Europe),
    ("Europe/Zurich", 60, Saving::Europe),
    ("Europe/Stockholm", 60, Saving::Europe),
    ("Europe/Oslo", 60, Saving::Europe),
    ("Europe/Copenhagen", 60, Saving::Europe),
    ("Europe/Prague", 60, Saving::Europe),
    ("Europe/Warsaw", 60, Saving::Europe),
    ("Europe/Budapest", 60, Saving::Europe),
    ("Europe/Athens", 120, Saving::Europe),
    ("Europe/Helsinki", 120, Saving::Europe),
    ("Europe/Kyiv", 120, Saving::Europe),
    ("Europe/Bucharest", 120, Saving::Europe),
    ("Europe/Sofia", 120, Saving::Europe),
    ("Europe/Istanbul", 180, Saving::None),
    ("Europe/Moscow", 180, Saving::None),
    ("Africa/Lagos", 60, Saving::None),
    ("Africa/Johannesburg", 120, Saving::None),
    ("Africa/Nairobi", 180, Saving::None),
    ("America/New_York", -300, Saving::America),
    ("America/Toronto", -300, Saving::America),
    ("America/Chicago", -360, Saving::America),
    ("America/Denver", -420, Saving::America),
    ("America/Phoenix", -420, Saving::None),
    ("America/Los_Angeles", -480, Saving::America),
    ("America/Vancouver", -480, Saving::America),
    ("America/Anchorage", -540, Saving::America),
    ("Pacific/Honolulu", -600, Saving::None),
    ("America/Mexico_City", -360, Saving::None),
    ("America/Sao_Paulo", -180, Saving::None),
    ("America/Argentina/Buenos_Aires", -180, Saving::None),
    ("Asia/Dubai", 240, Saving::None),
    ("Asia/Tehran", 210, Saving::None),
    ("Asia/Karachi", 300, Saving::None),
    ("Asia/Kolkata", 330, Saving::None),
    ("Asia/Bangkok", 420, Saving::None),
    ("Asia/Jakarta", 420, Saving::None),
    ("Asia/Shanghai", 480, Saving::None),
    ("Asia/Hong_Kong", 480, Saving::None),
    ("Asia/Singapore", 480, Saving::None),
    ("Asia/Manila", 480, Saving::None),
    ("Asia/Tokyo", 540, Saving::None),
    ("Asia/Seoul", 540, Saving::None),
    ("Australia/Perth", 480, Saving::None),
    ("Australia/Adelaide", 570, Saving::Australia),
    ("Australia/Brisbane", 600, Saving::None),
    ("Australia/Sydney", 600, Saving::Australia),
    ("Australia/Melbourne", 600, Saving::Australia),
    ("Pacific/Auckland", 720, Saving::NewZealand),
];

/// The zone of an IANA name from the table, matched without regard to case.
fn named(name: &str) -> Option<TimeZone> {
    ZONES
        .iter()
        .find(|(known, _, _)| known.eq_ignore_ascii_case(name))
        .map(|(_, minutes, saving)| TimeZone {
            offset: i64::from(*minutes) * 60,
            saving: *saving,
            ..TimeZone::utc()
        })
}

impl TimeValue {
    /// Resolves the timestamp to wall-clock seconds in `zone`: a date or time without an offset
    /// is the time on the wall there, as written; an instant (Unix seconds, or a time with its own
    /// offset) is shifted by the offset in force in `zone` at that instant. The result is meant
    /// to be placed and written as if it were UTC.
    pub(crate) fn resolve_wall(&self, zone: TimeZone) -> Result<i64, &'static str> {
        let written = |text: &str| {
            let text = text.trim();
            text.len() > 10 && starts_offset_somewhere(text)
        };
        let naive = self.resolve(TimeZone::utc().with_locale(zone.locale))?;
        let instant = match self {
            Self::Number(_) => true,
            Self::Text(text) => written(text),
        };
        Ok(if instant {
            naive + zone.offset_at(naive)
        } else {
            naive
        })
    }
}

/// Whether a date-time text ends in an offset or `Z`: a time of day, then `Z`, `+hh:mm` or
/// `-hh:mm`.
fn starts_offset_somewhere(text: &str) -> bool {
    let time = text.get(10..).unwrap_or("");
    time.rfind(['Z', 'z', '+']).is_some() || time.get(1..).is_some_and(|rest| rest.contains('-'))
}

/// Splits a timestamp into the local calendar fields of `zone`.
fn local_fields(epoch: i64, zone: TimeZone) -> (i64, u32, u32, u32, u32) {
    let shifted = epoch + zone.offset();
    let days = shifted.div_euclid(SECONDS_PER_DAY);
    let seconds = shifted.rem_euclid(SECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    (
        year,
        month,
        day,
        u32::try_from(seconds / SECONDS_PER_HOUR).expect("hours fit in a day"),
        u32::try_from((seconds % SECONDS_PER_HOUR) / 60).expect("minutes fit in an hour"),
    )
}

/// `2026-03-01`, the ISO 8601 date, or `01.03.2026` in German.
pub(crate) fn format_date(epoch: i64, zone: TimeZone) -> String {
    let (year, month, day, _, _) = local_fields(epoch, zone);
    match zone.locale {
        Locale::En => format!("{year:04}-{month:02}-{day:02}"),
        Locale::De => format!("{day:02}.{month:02}.{year:04}"),
    }
}

/// `2026-03-01 12:00`, or `01.03.2026 12:00` in German.
pub(crate) fn format_datetime(epoch: i64, zone: TimeZone) -> String {
    let (_, _, _, hour, minute) = local_fields(epoch, zone);
    format!("{} {hour:02}:{minute:02}", format_date(epoch, zone))
}

/// `2026-03`, or `03.2026` in German, for month steps.
fn format_month(epoch: i64, zone: TimeZone) -> String {
    let (year, month, _, _, _) = local_fields(epoch, zone);
    match zone.locale {
        Locale::En => format!("{year:04}-{month:02}"),
        Locale::De => format!("{month:02}.{year:04}"),
    }
}

fn format_year(epoch: i64, zone: TimeZone) -> String {
    let (year, _, _, _, _) = local_fields(epoch, zone);
    format!("{year:04}")
}

/// How finely the labels of a time chart name an observation: a year, a month, a date, or a date
/// with a time of day. The coarsest form that tells every observation apart is chosen from the
/// data, so an annual series reads `1850` rather than `1850-01-01` and a monthly one `2026-03`;
/// `timeAxis.precision` can set it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Precision {
    Year,
    Month,
    Day,
    Minute,
    /// A numeric axis, with as many decimals as its most precise value needs.
    Number(u8),
}

impl Precision {
    /// The precision that fits every timestamp in `zone`.
    pub(crate) fn of(timestamps: impl Iterator<Item = i64> + Clone, zone: TimeZone) -> Self {
        if zone.is_numeric() {
            return Self::Number(timestamps.map(numeric_decimals).max().unwrap_or(0));
        }
        if any_has_time_of_day(timestamps.clone(), zone) {
            Self::Minute
        } else if timestamps.clone().all(|epoch| {
            let (_, month, day, _, _) = local_fields(epoch, zone);
            month == 1 && day == 1
        }) {
            Self::Year
        } else if timestamps
            .into_iter()
            .all(|epoch| local_fields(epoch, zone).2 == 1)
        {
            Self::Month
        } else {
            Self::Day
        }
    }

    /// The finer of two precisions: an annotation is named at least as finely as the chart's
    /// observations, so that a marker on March 1st of a daily series reads as a day, not a month.
    pub(crate) fn at_least(self, other: Self) -> Self {
        let rank = |precision: Self| match precision {
            Self::Year => 0,
            Self::Month => 1,
            Self::Day => 2,
            Self::Minute => 3,
            Self::Number(decimals) => 4 + u32::from(decimals),
        };
        if rank(other) > rank(self) {
            other
        } else {
            self
        }
    }

    /// The label of one observation at this precision.
    pub(crate) fn format(self, epoch: i64, zone: TimeZone) -> String {
        match self {
            Self::Year => format_year(epoch, zone),
            Self::Month => format_month(epoch, zone),
            Self::Day => format_date(epoch, zone),
            Self::Minute => format_datetime(epoch, zone),
            Self::Number(decimals) => format_numeric(epoch, decimals, zone),
        }
    }
}

/// The decimals a numeric position needs: 11 700 millionths is 0.0117, four.
fn numeric_decimals(units: i64) -> u8 {
    let mut decimals = 6;
    let mut rest = units;
    while decimals > 0 && rest % 10 == 0 {
        rest /= 10;
        decimals -= 1;
    }
    decimals
}

/// A numeric position as text, in the locale of its axis.
fn format_numeric(units: i64, decimals: u8, zone: TimeZone) -> String {
    // A position is bounded by MAX_NUMERIC millionths, well inside f64's exact integers.
    #[allow(clippy::cast_precision_loss)]
    let value = units as f64 / NUMERIC_UNITS;
    crate::layout::format_value(
        value,
        NumberStyle {
            decimals: Some(decimals),
            locale: zone.locale,
            ..NumberStyle::default()
        },
    )
}

/// Ticks of a numeric axis: round steps of 1, 2 or 5 times a power of ten, at most `max_ticks`.
fn numeric_ticks(
    min: i64,
    max: i64,
    zone: TimeZone,
    max_ticks: usize,
    fixed: Option<f64>,
) -> Vec<Tick> {
    #[allow(clippy::cast_precision_loss)]
    let (low, high) = (min as f64 / NUMERIC_UNITS, max as f64 / NUMERIC_UNITS);
    #[allow(clippy::cast_precision_loss)]
    let raw = (high - low).max(f64::MIN_POSITIVE) / max_ticks.max(1) as f64;
    let magnitude = 10.0_f64.powf(raw.log10().floor());
    let step = fixed.unwrap_or_else(|| {
        [1.0, 2.0, 5.0, 10.0]
            .into_iter()
            .map(|factor| factor * magnitude)
            .find(|step| *step >= raw)
            .unwrap_or(10.0 * magnitude)
    });
    let decimals = (0..=6_u8)
        .find(|decimals| {
            let scaled = step * 10.0_f64.powi(i32::from(*decimals));
            (scaled - scaled.round()).abs() < 1e-9
        })
        .unwrap_or(6);
    let mut ticks = Vec::new();
    let mut index = (low / step).ceil();
    while index * step <= high + step * 1e-9 && ticks.len() < 64 {
        // Bounded by MAX_NUMERIC, the position stays far inside i64.
        #[allow(clippy::cast_possible_truncation)]
        let units = (index * step * NUMERIC_UNITS).round() as i64;
        ticks.push(Tick {
            epoch: units,
            label: format_numeric(units, decimals, zone),
        });
        index += 1.0;
    }
    ticks
}

/// Whether any timestamp carries a time of day in `zone`, which decides if the data table and
/// the axis labels show `HH:MM` as well.
pub(crate) fn any_has_time_of_day(timestamps: impl Iterator<Item = i64>, zone: TimeZone) -> bool {
    timestamps
        .into_iter()
        .any(|epoch| (epoch + zone.offset()).rem_euclid(SECONDS_PER_DAY) != 0)
}

/// One axis tick: its position in Unix seconds and its rendered label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Tick {
    pub epoch: i64,
    pub label: String,
}

/// A candidate tick distance: whole multiples of a second, or whole months because month
/// lengths differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Seconds(i64),
    Months(i64),
}

impl Step {
    /// The ticks this step places across the span, counted up to one beyond `limit`: a step
    /// whose boundaries fall well inside the span is not refused for ticks it would not draw.
    fn exact_count(self, min: i64, max: i64, zone: TimeZone, limit: i64) -> i64 {
        let mut count = 0;
        let mut current = self.first(min, zone);
        while current <= max && count <= limit {
            count += 1;
            current = self.next(current, zone);
        }
        count
    }

    /// First tick at or after `min`, sitting on a boundary of this step.
    fn first(self, min: i64, zone: TimeZone) -> i64 {
        match self {
            Self::Seconds(step) => {
                // Whole hours sit on local hour boundaries; days on local midnight; week and
                // fortnight steps on a Monday.
                let anchor = if step % (7 * SECONDS_PER_DAY) == 0 {
                    MONDAY_EPOCH - zone.offset()
                } else {
                    -zone.offset()
                };
                let remainder = (min - anchor).rem_euclid(step);
                min + if remainder == 0 { 0 } else { step - remainder }
            }
            Self::Months(months) if months % 12 == 0 => {
                // Year steps align with the calendar year so they land on round decades.
                let years = months / 12;
                let year = local_fields(min, zone).0;
                let aligned = year.div_euclid(years) * years;
                let start = months_start(aligned * 12, zone);
                if start >= min {
                    start
                } else {
                    months_start((aligned + years) * 12, zone)
                }
            }
            Self::Months(months) => {
                let index = month_index(min, zone);
                let aligned = index.div_euclid(months) * months;
                let start = months_start(aligned, zone);
                if start >= min {
                    start
                } else {
                    months_start(aligned + months, zone)
                }
            }
        }
    }

    /// The tick that follows `current`.
    fn next(self, current: i64, zone: TimeZone) -> i64 {
        match self {
            Self::Seconds(seconds) => current + seconds,
            Self::Months(months) => months_start(month_index(current, zone) + months, zone),
        }
    }

    /// Whether this step resolves finer than a day.
    const fn is_sub_day(self) -> bool {
        matches!(self, Self::Seconds(seconds) if seconds < SECONDS_PER_DAY)
    }

    /// A tick label that matches the step's resolution.
    fn label(self, epoch: i64, zone: TimeZone) -> String {
        match self {
            Self::Seconds(seconds) if seconds < SECONDS_PER_DAY => format_datetime(epoch, zone),
            Self::Seconds(_) => format_date(epoch, zone),
            Self::Months(months) if months < 12 => format_month(epoch, zone),
            Self::Months(_) => format_year(epoch, zone),
        }
    }
}

/// Tick candidates from an hour up to a century. Business time series need the finer end; the
/// coarse end keeps very long histories readable. The two- and three-day steps keep a two- to
/// three-week span from falling back to weekly ticks, which would leave only a couple of labels;
/// the 20- and 25-year steps do the same for a climate record since 1850.
const STEPS: [Step; 19] = [
    Step::Seconds(SECONDS_PER_HOUR),
    Step::Seconds(6 * SECONDS_PER_HOUR),
    Step::Seconds(12 * SECONDS_PER_HOUR),
    Step::Seconds(SECONDS_PER_DAY),
    Step::Seconds(2 * SECONDS_PER_DAY),
    Step::Seconds(3 * SECONDS_PER_DAY),
    Step::Seconds(7 * SECONDS_PER_DAY),
    Step::Seconds(14 * SECONDS_PER_DAY),
    Step::Months(1),
    Step::Months(3),
    Step::Months(6),
    Step::Months(12),
    Step::Months(24),
    Step::Months(60),
    Step::Months(120),
    Step::Months(240),
    Step::Months(300),
    Step::Months(600),
    Step::Months(1200),
];

/// Calendar-aligned ticks across `min..=max`, at most about `max_ticks` of them.
///
/// `allow_sub_day` is false for a series whose observations all sit on local midnight: a daily
/// series then gets daily ticks instead of labels that repeat a time of day that carries no
/// information.
pub(crate) fn ticks(
    min: i64,
    max: i64,
    zone: TimeZone,
    max_ticks: usize,
    allow_sub_day: bool,
    fixed: Option<f64>,
) -> Vec<Tick> {
    if zone.is_numeric() {
        return numeric_ticks(min, max, zone, max_ticks, fixed);
    }
    let step = fixed_or_chosen(fixed, || {
        tick_step(min, max, zone, max_ticks, allow_sub_day)
    });
    let mut ticks = Vec::new();
    let mut current = step.first(min, zone);
    // The chosen step always terminates; the bound only keeps a rounding surprise from spinning.
    while current <= max && ticks.len() < 64 {
        ticks.push(Tick {
            epoch: current,
            label: step.label(current, zone),
        });
        current = step.next(current, zone);
    }
    ticks
}

/// Ticks for a time axis whose gaps are collapsed into evenly spaced `slots`, the observed
/// timestamps in order. The step is chosen as for [`ticks`] across the observed span; each
/// calendar boundary moves to the first observation on or after it and is labelled with that
/// observation's date at the step's resolution, so a label never names a day without an
/// observation. Boundaries that fall into one gap give a single tick.
pub(crate) fn collapsed_ticks(
    slots: &[i64],
    zone: TimeZone,
    max_ticks: usize,
    allow_sub_day: bool,
    fixed: Option<f64>,
) -> Vec<Tick> {
    let (Some(&min), Some(&max)) = (slots.first(), slots.last()) else {
        return Vec::new();
    };
    let step = fixed_or_chosen(fixed, || {
        tick_step(min, max, zone, max_ticks, allow_sub_day)
    });
    let mut ticks: Vec<Tick> = Vec::new();
    let mut current = step.first(min, zone);
    while current <= max && ticks.len() < 64 {
        let epoch = slots[slots.partition_point(|slot| *slot < current)];
        if ticks.last().is_none_or(|tick| tick.epoch != epoch) {
            ticks.push(Tick {
                epoch,
                label: step.label(epoch, zone),
            });
        }
        current = step.next(current, zone);
    }
    ticks
}

/// How many ticks a declared step places across `min..=max`, counted up to `limit + 1`: whole
/// years on a calendar axis, units on a numeric one.
pub(crate) fn declared_tick_count(
    min: i64,
    max: i64,
    zone: TimeZone,
    step: f64,
    limit: i64,
) -> i64 {
    if zone.is_numeric() {
        #[allow(clippy::cast_precision_loss)]
        let (low, high) = (min as f64 / NUMERIC_UNITS, max as f64 / NUMERIC_UNITS);
        // Bounded by the comparison with `limit` below; tick limits are small.
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let count = ((high / step + 1e-9).floor() - (low / step - 1e-9).ceil() + 1.0)
            .clamp(0.0, (limit + 1) as f64) as i64;
        return count;
    }
    // Validated: a calendar axis declares whole years, at most a thousand.
    #[allow(clippy::cast_possible_truncation)]
    Step::Months(step as i64 * 12).exact_count(min, max, zone, limit)
}

/// A declared step of whole years, or the one `choose` picks from the width.
fn fixed_or_chosen(fixed: Option<f64>, choose: impl FnOnce() -> Step) -> Step {
    // Validated: a calendar axis declares whole years, at most a few thousand.
    #[allow(clippy::cast_possible_truncation)]
    fixed.map_or_else(choose, |years| Step::Months(years as i64 * 12))
}

/// The finest step that keeps the ticks across `min..=max` within about `max_ticks`.
fn tick_step(min: i64, max: i64, zone: TimeZone, max_ticks: usize, allow_sub_day: bool) -> Step {
    let limit = i64::try_from(max_ticks.max(2)).expect("tick counts are small");
    STEPS
        .into_iter()
        .filter(|step| allow_sub_day || !step.is_sub_day())
        .find(|step| step.exact_count(min, max, zone, limit) <= limit)
        .unwrap_or(Step::Months(1200))
}

fn month_index(epoch: i64, zone: TimeZone) -> i64 {
    let (year, month, _, _, _) = local_fields(epoch, zone);
    year * 12 + i64::from(month) - 1
}

/// Start of the given month index (year × 12 + month − 1) as Unix seconds in `zone`.
fn months_start(index: i64, zone: TimeZone) -> i64 {
    let (year, month) = (index.div_euclid(12), index.rem_euclid(12));
    days_from_civil(
        year,
        u32::try_from(month + 1).expect("month index is 0..=11"),
        1,
    ) * SECONDS_PER_DAY
        - zone.offset()
}

/// Parses an ISO 8601 date, optional time, optional offset, or a bare four-digit year, which
/// stands for January 1st of that year. Returns Unix seconds.
pub(crate) fn parse_iso(text: &str, zone: TimeZone) -> Option<i64> {
    let text = text.trim();
    let bytes = text.as_bytes();
    if bytes.len() == 4 && bytes.iter().all(u8::is_ascii_digit) {
        let year: i64 = text.parse().ok()?;
        return Some(days_from_civil(year, 1, 1) * SECONDS_PER_DAY - zone.offset());
    }
    // A month such as 2026-03 stands for its first day.
    if bytes.len() == 7 && bytes[4] == b'-' {
        let year: i64 = text.get(0..4)?.parse().ok()?;
        let month: u32 = text.get(5..7)?.parse().ok()?;
        if !(1..=12).contains(&month) {
            return None;
        }
        return Some(days_from_civil(year, month, 1) * SECONDS_PER_DAY - zone.offset());
    }
    if bytes.len() < 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year: i64 = text.get(0..4)?.parse().ok()?;
    let month: u32 = text.get(5..7)?.parse().ok()?;
    let day: u32 = text.get(8..10)?.parse().ok()?;
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return None;
    }

    let mut rest = text.get(10..).unwrap_or("");
    let mut hour = 0_u32;
    let mut minute = 0_u32;
    if let Some(after) = rest.strip_prefix(['T', ' ']) {
        if after.len() < 5 || after.as_bytes()[2] != b':' {
            return None;
        }
        hour = after.get(0..2)?.parse().ok()?;
        minute = after.get(3..5)?.parse().ok()?;
        if hour > 23 || minute > 59 {
            return None;
        }
        rest = &after[5..];
        if let Some(seconds) = rest.strip_prefix(':') {
            let second: u32 = seconds.get(0..2)?.parse().ok()?;
            if second > 59 {
                return None;
            }
            rest = &seconds[2..];
            if !rest.is_empty() && !starts_offset(rest) {
                return None;
            }
        }
    }

    // A date that names its own offset is absolute; otherwise the text is wall-clock time in the
    // configured timezone.
    let offset = if rest.is_empty() {
        zone.offset()
    } else if rest.eq_ignore_ascii_case("z") {
        0
    } else {
        TimeZone::parse(rest)?.offset()
    };

    let days = days_from_civil(year, month, day);
    Some(
        days * SECONDS_PER_DAY + i64::from(hour) * SECONDS_PER_HOUR + i64::from(minute) * 60
            - offset,
    )
}

fn starts_offset(rest: &str) -> bool {
    matches!(rest.as_bytes().first(), Some(b'Z' | b'z' | b'+' | b'-'))
}

pub(crate) fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if is_leap_year(year) => 29,
        _ => 28,
    }
}

const fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days since 1970-01-01 for a proleptic Gregorian date (Howard Hinnant's algorithm).
pub(crate) fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (i64::from(month) + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The inverse of [`days_from_civil`].
pub(crate) fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    (
        year,
        u32::try_from(month).expect("month is 1..=12"),
        u32::try_from(day).expect("day is 1..=31"),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        Precision, SECONDS_PER_DAY, SECONDS_PER_HOUR, TimeValue, TimeZone, any_has_time_of_day,
        civil_from_days, days_from_civil, format_date, format_datetime, parse_iso, ticks,
    };

    const UTC: TimeZone = TimeZone::utc();
    /// 2026-03-01T00:00:00Z, the reference instant of these tests.
    const MARCH: i64 = 1_772_323_200;

    fn zone(text: &str) -> TimeZone {
        TimeZone::parse(text).expect("test timezone should parse")
    }

    #[test]
    fn civil_dates_round_trip() {
        for (year, month, day) in [
            (1900, 1, 1),
            (1970, 1, 1),
            (2000, 2, 29),
            (2026, 9, 15),
            (2199, 12, 31),
        ] {
            let days = days_from_civil(year, month, day);
            assert_eq!(
                civil_from_days(days),
                (year, month, day),
                "{year}-{month}-{day}"
            );
        }
    }

    #[test]
    fn parses_dates_times_and_offsets() {
        assert_eq!(parse_iso("1970-01-01", UTC), Some(0));
        assert_eq!(parse_iso("2026-03-01", UTC), Some(MARCH));
        assert_eq!(parse_iso("2026-03-01T00:00:00Z", UTC), Some(MARCH));
        assert_eq!(parse_iso("2026-03-01 00:00", UTC), Some(MARCH));
        // Wall-clock time in the configured offset.
        assert_eq!(
            parse_iso("2026-03-01T02:30", zone("+02:00")),
            Some(MARCH + 1_800)
        );
        // The same wall-clock time means something else without the offset.
        assert_eq!(parse_iso("2026-03-01T02:30", UTC), Some(MARCH + 9_000));
        // An offset in the date wins over the configured timezone.
        assert_eq!(parse_iso("2026-03-01T00:30:00+00:30", UTC), Some(MARCH));
        assert_eq!(
            parse_iso("2026-03-01T12:00:00Z", zone("+02:00")),
            Some(MARCH + 43_200)
        );
    }

    #[test]
    fn rejects_malformed_dates() {
        for text in [
            "2026-3-01",
            "2026-13-01",
            "2026-02-30",
            "2023-02-29",
            "2026-03-01T25:00",
            "2026-03-01T12:60",
            "2026-03-01T12",
            "2026-03-01T12:00:00nonsense",
            "nonsense",
            "",
        ] {
            assert_eq!(parse_iso(text, UTC), None, "{text}");
        }
    }

    #[test]
    fn parses_fixed_offsets_and_rejects_named_zones() {
        assert_eq!(zone("+02:00").offset(), 7_200);
        assert_eq!(zone("-0500").offset(), -18_000);
        assert_eq!(zone("UTC").offset(), 0);
        assert!(TimeZone::parse("Europe/Berlin").is_none());
        assert!(TimeZone::parse("+15:00").is_none());
        assert!(TimeZone::parse("+02:70").is_none());
    }

    #[test]
    fn named_zones_follow_their_daylight_saving_rules() {
        let instant = |year: i64, month: u32, day: u32, hour: i64, minute: i64, second: i64| {
            days_from_civil(year, month, day) * SECONDS_PER_DAY
                + hour * 3_600
                + minute * 60
                + second
        };
        let hours =
            |zone: &str, epoch: i64| TimeZone::parse_named(zone).unwrap().offset_at(epoch) / 3_600;
        // Europe: the clocks move at 01:00 UTC on the last Sunday of March (28th in 2027) and
        // of October (31st).
        assert_eq!(hours("Europe/Berlin", instant(2027, 3, 28, 0, 59, 59)), 1);
        assert_eq!(hours("Europe/Berlin", instant(2027, 3, 28, 1, 0, 0)), 2);
        assert_eq!(hours("Europe/Berlin", instant(2027, 10, 31, 0, 59, 59)), 2);
        assert_eq!(hours("Europe/Berlin", instant(2027, 10, 31, 1, 0, 0)), 1);
        assert_eq!(hours("Europe/London", instant(2027, 7, 1, 12, 0, 0)), 1);
        assert_eq!(hours("Europe/London", instant(2027, 1, 1, 12, 0, 0)), 0);
        // America: 02:00 local on the second Sunday of March (14th) and the first of November (7th).
        assert_eq!(
            hours("America/New_York", instant(2027, 3, 14, 6, 59, 59)),
            -5
        );
        assert_eq!(hours("America/New_York", instant(2027, 3, 14, 7, 0, 0)), -4);
        assert_eq!(
            hours("America/New_York", instant(2027, 11, 7, 5, 59, 59)),
            -4
        );
        assert_eq!(hours("America/New_York", instant(2027, 11, 7, 6, 0, 0)), -5);
        assert_eq!(hours("America/Phoenix", instant(2027, 7, 1, 12, 0, 0)), -7);
        // The south: summer time in January, standard time in July.
        assert_eq!(hours("Australia/Sydney", instant(2027, 1, 15, 0, 0, 0)), 11);
        assert_eq!(hours("Australia/Sydney", instant(2027, 7, 15, 0, 0, 0)), 10);
        assert_eq!(hours("Pacific/Auckland", instant(2027, 1, 15, 0, 0, 0)), 13);
        assert_eq!(hours("Pacific/Auckland", instant(2027, 7, 15, 0, 0, 0)), 12);
        // Names are matched without regard to case; unknown ones are refused.
        assert!(TimeZone::parse_named("europe/berlin").is_some());
        assert!(TimeZone::parse_named("Europe/Atlantis").is_none());
    }

    #[test]
    fn a_wall_clock_time_is_kept_and_an_instant_is_shifted() {
        let berlin = TimeZone::parse_named("Europe/Berlin").unwrap();
        // 2027-07-01 00:00 UTC is 02:00 in Berlin; a bare date is midnight there already.
        let midnight = days_from_civil(2027, 7, 1) * SECONDS_PER_DAY;
        assert_eq!(midnight, 1_814_400_000);
        assert_eq!(
            TimeValue::Number(1_814_400_000.0).resolve_wall(berlin),
            Ok(midnight + 7_200)
        );
        assert_eq!(
            TimeValue::Text("2027-07-01".to_owned()).resolve_wall(berlin),
            Ok(midnight)
        );
        assert_eq!(
            TimeValue::Text("2027-07-01T10:00".to_owned()).resolve_wall(berlin),
            Ok(midnight + 36_000)
        );
        assert_eq!(
            TimeValue::Text("2027-07-01T10:00:00Z".to_owned()).resolve_wall(berlin),
            Ok(midnight + 36_000 + 7_200)
        );
        assert_eq!(
            TimeValue::Text("2027-07-01T10:00:00+05:00".to_owned()).resolve_wall(berlin),
            Ok(midnight + 5 * 3_600 + 7_200)
        );
    }

    #[test]
    fn resolves_number_and_text_timestamps() {
        let epoch = TimeValue::Number(1_772_323_200.0);
        assert_eq!(epoch.resolve(UTC), Ok(MARCH));
        let text = TimeValue::Text("2026-03-01".to_owned());
        assert_eq!(text.resolve(UTC), Ok(MARCH));
    }

    #[test]
    fn rejects_fractional_and_out_of_range_timestamps() {
        assert!(TimeValue::Number(1.5).resolve(UTC).is_err());
        assert!(TimeValue::Number(f64::NAN).resolve(UTC).is_err());
        assert!(TimeValue::Number(-9_000_000_000.0).resolve(UTC).is_err());
        assert!(TimeValue::Number(9e18).resolve(UTC).is_err());
        // 1970-01-01 lies inside the supported range and is a valid timestamp.
        assert_eq!(TimeValue::Number(0.0).resolve(UTC), Ok(0));
        // So does 1850, the start of the instrumental climate record.
        assert!(TimeValue::Number(-3_786_825_600.0).resolve(UTC).is_ok());
    }

    #[test]
    fn a_bare_year_is_january_first() {
        assert_eq!(parse_iso("1850", UTC), parse_iso("1850-01-01", UTC));
        assert_eq!(
            parse_iso("2026", zone("+02:00")),
            parse_iso("2026-01-01", zone("+02:00"))
        );
        assert_eq!(parse_iso("185", UTC), None);
        assert_eq!(parse_iso("18a0", UTC), None);
    }

    #[test]
    fn the_label_precision_follows_the_data() {
        let years = [
            parse_iso("1850", UTC).expect("year"),
            parse_iso("1851", UTC).expect("year"),
        ];
        assert_eq!(Precision::of(years.into_iter(), UTC), Precision::Year);
        assert_eq!(Precision::Year.format(years[0], UTC), "1850");
        let days = [years[0], years[0] + SECONDS_PER_DAY];
        assert_eq!(Precision::of(days.into_iter(), UTC), Precision::Day);
        let hours = [years[0], years[0] + SECONDS_PER_HOUR];
        assert_eq!(Precision::of(hours.into_iter(), UTC), Precision::Minute);
    }

    #[test]
    fn formats_local_labels() {
        let berlin = zone("+02:00");
        assert_eq!(format_date(0, UTC), "1970-01-01");
        assert_eq!(format_datetime(0, UTC), "1970-01-01 00:00");
        assert_eq!(format_date(0, berlin), "1970-01-01");
        assert_eq!(format_datetime(0, berlin), "1970-01-01 02:00");
        assert_eq!(format_date(MARCH, UTC), "2026-03-01");
    }

    #[test]
    fn detects_time_of_day_in_a_zone() {
        assert!(!any_has_time_of_day([0, 86_400].into_iter(), UTC));
        assert!(any_has_time_of_day([3_600].into_iter(), UTC));
        // Midnight UTC is 02:00 in this offset, so the table needs times.
        assert!(any_has_time_of_day([0].into_iter(), zone("+02:00")));
        // Midnight in that offset is midnight UTC again.
        assert!(!any_has_time_of_day([-7_200].into_iter(), zone("+02:00")));
    }

    #[test]
    fn yearly_spans_use_quarterly_steps() {
        let min = parse_iso("2026-01-15", UTC).expect("date");
        let max = parse_iso("2026-12-31", UTC).expect("date");
        let labels: Vec<String> = ticks(min, max, UTC, 8, true, None)
            .into_iter()
            .map(|tick| tick.label)
            .collect();
        assert_eq!(labels, vec!["2026-04", "2026-07", "2026-10"]);
    }

    #[test]
    fn monthly_spans_use_month_steps() {
        let min = parse_iso("2026-01-01", UTC).expect("date");
        let max = parse_iso("2026-08-31", UTC).expect("date");
        let labels: Vec<String> = ticks(min, max, UTC, 8, true, None)
            .into_iter()
            .map(|tick| tick.label)
            .collect();
        assert_eq!(
            labels,
            vec![
                "2026-01", "2026-02", "2026-03", "2026-04", "2026-05", "2026-06", "2026-07",
                "2026-08"
            ]
        );
    }

    #[test]
    fn short_spans_use_hour_and_day_steps() {
        assert_eq!(ticks(MARCH, MARCH + 5 * 3_600, UTC, 8, true, None).len(), 6);
        assert_eq!(
            ticks(MARCH, MARCH + 6 * 86_400, UTC, 8, true, None).len(),
            7
        );
        // A five-week span falls back to whole weeks, counted from a Monday.
        let weekly = ticks(MARCH, MARCH + 40 * 86_400, UTC, 8, true, None);
        assert_eq!(weekly.len(), 6);
        assert!(
            weekly
                .iter()
                .all(|tick| format_date(tick.epoch, UTC).len() == 10)
        );
    }

    #[test]
    fn long_spans_use_year_steps_on_round_decades() {
        let min = parse_iso("1900-01-01", UTC).expect("date");
        let max = parse_iso("2199-01-01", UTC).expect("date");
        let labels: Vec<String> = ticks(min, max, UTC, 8, true, None)
            .into_iter()
            .map(|tick| tick.label)
            .collect();
        assert_eq!(labels, vec!["1900", "1950", "2000", "2050", "2100", "2150"]);
    }

    #[test]
    fn single_timestamp_span_still_produces_ticks() {
        assert_eq!(ticks(MARCH, MARCH, UTC, 8, true, None).len(), 1);
    }

    #[test]
    fn a_daily_series_keeps_daily_ticks() {
        // Two days of midnight observations: date labels instead of a repeated 00:00.
        let daily = ticks(MARCH, MARCH + 2 * 86_400, UTC, 8, false, None);
        assert_eq!(daily.len(), 3);
        assert!(
            daily
                .iter()
                .all(|tick| format_date(tick.epoch, UTC).len() == 10)
        );

        // The same span with a time of day in the data keeps the finer steps.
        let intraday = ticks(MARCH, MARCH + 2 * 86_400, UTC, 8, true, None);
        assert!(intraday.len() > 3);
        assert!(
            intraday
                .iter()
                .all(|tick| format_datetime(tick.epoch, UTC).len() == 16)
        );
    }
}
