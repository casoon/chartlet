//! The time contract of `type: "time"` charts: a timestamp is either Unix seconds or an ISO 8601
//! date, the timezone option is a fixed UTC offset, and axis ticks sit on calendar boundaries.
//!
//! No timezone database is involved: `"timezone"` accepts `UTC` or a fixed offset such as
//! `+02:00`, so rendering stays deterministic and dependency-free. A bare ISO date is read as
//! wall-clock time in that offset; a date that carries `Z` or its own offset is absolute.

use serde::{Deserialize, Serialize};

/// Supported range, 1700-01-01 to 2200-01-01 as Unix seconds. Keeps the civil-date arithmetic
/// and every tick label far away from the edges of `i64`. The lower bound reaches back before
/// the instrumental climate record (1850) and the pre-industrial reference (1750).
pub(crate) const MIN_TIMESTAMP: i64 = -8_520_336_000;
pub(crate) const MAX_TIMESTAMP: i64 = 7_258_118_400;

const SECONDS_PER_DAY: i64 = 86_400;
const SECONDS_PER_HOUR: i64 = 3_600;
/// 1969-12-29 was a Monday, so weekly ticks start on Mondays instead of on 1970-01-01.
const MONDAY_EPOCH: i64 = -4 * SECONDS_PER_DAY;

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
                "expected an ISO 8601 date such as 2026-03-01 or 2026-03-01T12:00:00Z, or a year such as 1850",
            )?,
        };
        if !(MIN_TIMESTAMP..MAX_TIMESTAMP).contains(&epoch) {
            return Err("timestamps must lie between 1700-01-01 and 2200-01-01");
        }
        Ok(epoch)
    }
}

/// A timezone as a fixed offset from UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct TimeZone {
    offset: i64,
}

impl TimeZone {
    pub(crate) const fn utc() -> Self {
        Self { offset: 0 }
    }

    pub(crate) const fn offset(self) -> i64 {
        self.offset
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
        })
    }
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

/// `2026-03-01`, the locale-neutral date an ISO 8601 label uses.
pub(crate) fn format_date(epoch: i64, zone: TimeZone) -> String {
    let (year, month, day, _, _) = local_fields(epoch, zone);
    format!("{year:04}-{month:02}-{day:02}")
}

/// `2026-03-01 12:00`.
pub(crate) fn format_datetime(epoch: i64, zone: TimeZone) -> String {
    let (year, month, day, hour, minute) = local_fields(epoch, zone);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}

/// `2026-03` for month steps, `2026` for year steps.
fn format_month(epoch: i64, zone: TimeZone) -> String {
    let (year, month, _, _, _) = local_fields(epoch, zone);
    format!("{year:04}-{month:02}")
}

fn format_year(epoch: i64, zone: TimeZone) -> String {
    let (year, _, _, _, _) = local_fields(epoch, zone);
    format!("{year:04}")
}

/// How finely the labels of a time chart name an observation: a year, a date, or a date with a
/// time of day. The coarsest form that tells every observation apart is chosen from the data, so
/// an annual series reads `1850` rather than `1850-01-01`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Precision {
    Year,
    Day,
    Minute,
}

impl Precision {
    /// The precision that fits every timestamp in `zone`.
    pub(crate) fn of(timestamps: impl Iterator<Item = i64> + Clone, zone: TimeZone) -> Self {
        if any_has_time_of_day(timestamps.clone(), zone) {
            Self::Minute
        } else if timestamps.into_iter().all(|epoch| {
            let (_, month, day, _, _) = local_fields(epoch, zone);
            month == 1 && day == 1
        }) {
            Self::Year
        } else {
            Self::Day
        }
    }

    /// The label of one observation at this precision.
    pub(crate) fn format(self, epoch: i64, zone: TimeZone) -> String {
        match self {
            Self::Year => format_year(epoch, zone),
            Self::Day => format_date(epoch, zone),
            Self::Minute => format_datetime(epoch, zone),
        }
    }
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
    /// Rough number of ticks this step produces across the span; used to pick the finest step
    /// that still fits.
    fn tick_count(self, min: i64, max: i64, zone: TimeZone) -> i64 {
        match self {
            Self::Seconds(step) => (max - min) / step + 1,
            Self::Months(months) => {
                let (min_year, min_month, _, _, _) = local_fields(min, zone);
                let (max_year, max_month, _, _, _) = local_fields(max, zone);
                let span = (max_year - min_year) * 12 + i64::from(max_month) - i64::from(min_month);
                span / months + 1
            }
        }
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
) -> Vec<Tick> {
    let limit = i64::try_from(max_ticks.max(2)).expect("tick counts are small");
    let step = STEPS
        .into_iter()
        .filter(|step| allow_sub_day || !step.is_sub_day())
        .find(|step| step.tick_count(min, max, zone) <= limit)
        .unwrap_or(Step::Months(1200));
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
        let labels: Vec<String> = ticks(min, max, UTC, 8, true)
            .into_iter()
            .map(|tick| tick.label)
            .collect();
        assert_eq!(labels, vec!["2026-04", "2026-07", "2026-10"]);
    }

    #[test]
    fn monthly_spans_use_month_steps() {
        let min = parse_iso("2026-01-01", UTC).expect("date");
        let max = parse_iso("2026-08-31", UTC).expect("date");
        let labels: Vec<String> = ticks(min, max, UTC, 8, true)
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
        assert_eq!(ticks(MARCH, MARCH + 5 * 3_600, UTC, 8, true).len(), 6);
        assert_eq!(ticks(MARCH, MARCH + 6 * 86_400, UTC, 8, true).len(), 7);
        // A five-week span falls back to whole weeks, counted from a Monday.
        let weekly = ticks(MARCH, MARCH + 40 * 86_400, UTC, 8, true);
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
        let labels: Vec<String> = ticks(min, max, UTC, 8, true)
            .into_iter()
            .map(|tick| tick.label)
            .collect();
        assert_eq!(labels, vec!["1900", "1950", "2000", "2050", "2100", "2150"]);
    }

    #[test]
    fn single_timestamp_span_still_produces_ticks() {
        assert_eq!(ticks(MARCH, MARCH, UTC, 8, true).len(), 1);
    }

    #[test]
    fn a_daily_series_keeps_daily_ticks() {
        // Two days of midnight observations: date labels instead of a repeated 00:00.
        let daily = ticks(MARCH, MARCH + 2 * 86_400, UTC, 8, false);
        assert_eq!(daily.len(), 3);
        assert!(
            daily
                .iter()
                .all(|tick| format_date(tick.epoch, UTC).len() == 10)
        );

        // The same span with a time of day in the data keeps the finer steps.
        let intraday = ticks(MARCH, MARCH + 2 * 86_400, UTC, 8, true);
        assert!(intraday.len() > 3);
        assert!(
            intraday
                .iter()
                .all(|tick| format_datetime(tick.epoch, UTC).len() == 16)
        );
    }
}
