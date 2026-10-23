//! The words of statistical plots: scatter plots, boxplots and Kaplan-Meier curves.

use super::plural;
use crate::spec::{Locale, Quadrant};
use std::fmt::Write as _;

/// What a boxplot shows: how many boxes, which median is highest and lowest, and where there are
/// outliers.
pub(crate) fn boxplot_summary(
    locale: Locale,
    boxes: usize,
    highest: (&str, &str),
    lowest: (&str, &str),
    outliers: &[&str],
) -> String {
    let outliers = match (locale, outliers.is_empty()) {
        (_, true) => String::new(),
        (Locale::En, false) => format!(" Outliers beyond the whiskers: {}.", outliers.join(", ")),
        (Locale::De, false) => format!(" Ausreißer jenseits der Whisker: {}.", outliers.join(", ")),
    };
    match locale {
        Locale::En => format!(
            "Box plot with {boxes} box{}, each from the first to the third quartile with its median. Highest median: {} ({}). Lowest median: {} ({}).{outliers}",
            plural(boxes, "", "es"),
            highest.0,
            highest.1,
            lowest.0,
            lowest.1,
        ),
        Locale::De => format!(
            "Boxplot mit {boxes} {}, jeweils vom ersten bis zum dritten Quartil mit dem Median. Höchster Median: {} ({}). Niedrigster Median: {} ({}).{outliers}",
            plural(boxes, "Box", "Boxen"),
            highest.0,
            highest.1,
            lowest.0,
            lowest.1,
        ),
    }
}

/// Where the values of a scatter axis lie: `from 1 to 5`, or `at 3` when they are all equal.
pub(crate) fn value_span(locale: Locale, low: &str, high: &str) -> String {
    match locale {
        Locale::En if low == high => format!("at {low}"),
        Locale::De if low == high => format!("bei {low}"),
        Locale::En => format!("from {low} to {high}"),
        Locale::De => format!("von {low} bis {high}"),
    }
}

/// What a scatter plot shows: its points, the spans of both axes, the groups and the lines.
pub(crate) fn scatter_summary(
    locale: Locale,
    points: usize,
    (x_span, y_span): (&str, &str),
    (groups, lines): (&str, &str),
    fits: &str,
) -> String {
    let mut text = match locale {
        Locale::En => format!(
            "Scatter plot of {points} point{}: x {x_span}, y {y_span}.",
            plural(points, "", "s")
        ),
        Locale::De => format!(
            "Streudiagramm aus {points} {}: x {x_span}, y {y_span}.",
            plural(points, "Punkt", "Punkten")
        ),
    };
    if !groups.is_empty() {
        let word = match locale {
            Locale::En => "Groups (points)",
            Locale::De => "Gruppen (Punkte)",
        };
        write!(text, " {word}: {groups}.").expect("writing to String cannot fail");
    }
    if !lines.is_empty() {
        let word = match locale {
            Locale::En => "Lines",
            Locale::De => "Linien",
        };
        write!(text, " {word}: {lines}.").expect("writing to String cannot fail");
    }
    if !fits.is_empty() {
        write!(text, " {fits}").expect("writing to String cannot fail");
    }
    text
}

/// How many points lie above a reference line of a scatter plot.
pub(crate) fn points_above_line(locale: Locale, count: usize) -> String {
    match locale {
        Locale::En => format!("{count} point{} above it", plural(count, "", "s")),
        Locale::De => format!("{count} {} darüber", plural(count, "Punkt", "Punkte")),
    }
}

/// A least-squares line of a scatter plot: its slope, intercept and r², for a group or all points.
pub(crate) fn scatter_fit(
    locale: Locale,
    group: Option<&str>,
    (slope, intercept, r_squared): (String, String, String),
    (log, points): (bool, usize),
) -> String {
    let of = match (locale, group) {
        (Locale::En, Some(group)) => format!("Line for {group}"),
        (Locale::En, None) => "Line".to_owned(),
        (Locale::De, Some(group)) => format!("Gerade für {group}"),
        (Locale::De, None) => "Gerade".to_owned(),
    };
    match (locale, log) {
        (Locale::En, false) => {
            format!("{of} through {points} points: y = {intercept} + {slope} x, r² = {r_squared}.")
        }
        (Locale::En, true) => format!(
            "{of} through {points} points, fitted to the logarithms: ln y = {intercept} + {slope} ln x, r² = {r_squared}."
        ),
        (Locale::De, false) => {
            format!("{of} durch {points} Punkte: y = {intercept} + {slope} x, r² = {r_squared}.")
        }
        (Locale::De, true) => format!(
            "{of} durch {points} Punkte, an die Logarithmen angepasst: ln y = {intercept} + {slope} ln x, r² = {r_squared}."
        ),
    }
}

/// Where an area of a scatter plot lies: `top left`, or `Top left` at the start of a sentence.
pub(crate) fn quadrant_position(locale: Locale, quadrant: Quadrant, capital: bool) -> String {
    let words = match (locale, quadrant) {
        (Locale::En, Quadrant::TopLeft) => "top left",
        (Locale::En, Quadrant::TopRight) => "top right",
        (Locale::En, Quadrant::BottomLeft) => "bottom left",
        (Locale::En, Quadrant::BottomRight) => "bottom right",
        (Locale::De, Quadrant::TopLeft) => "oben links",
        (Locale::De, Quadrant::TopRight) => "oben rechts",
        (Locale::De, Quadrant::BottomLeft) => "unten links",
        (Locale::De, Quadrant::BottomRight) => "unten rechts",
    };
    if capital {
        let mut chars = words.chars();
        chars.next().map_or_else(String::new, |first| {
            first.to_uppercase().chain(chars).collect()
        })
    } else {
        words.to_owned()
    }
}

/// Where a point on a dividing line lies, for the table.
pub(crate) const fn on_dividing_line(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "on a dividing line",
        Locale::De => "auf einer Trennlinie",
    }
}

/// The opening of the areas of a scatter plot: where they are divided.
pub(crate) fn quadrants_opening(locale: Locale, x: &str, y: &str) -> String {
    match locale {
        Locale::En => format!("Quadrants divided at x = {x} and y = {y}."),
        Locale::De => format!("Quadranten geteilt bei x = {x} und y = {y}."),
    }
}

/// One area of a scatter plot, or the points on a dividing line (`area` `None`): how many points
/// it holds and the names of up to the first ten, with how many more there are.
pub(crate) fn quadrant_points(
    locale: Locale,
    area: Option<(&str, Option<&str>)>,
    count: usize,
    names: &[&str],
) -> String {
    let head = match (locale, area) {
        (_, Some((position, Some(label)))) => format!("{position}, {label}"),
        (_, Some((position, None))) => position.to_owned(),
        (Locale::En, None) => "On a dividing line".to_owned(),
        (Locale::De, None) => "Auf einer Trennlinie".to_owned(),
    };
    let points = match (locale, count) {
        (Locale::En, 0) => "no points".to_owned(),
        (Locale::De, 0) => "keine Punkte".to_owned(),
        (Locale::En, _) => format!("{count} point{}", plural(count, "", "s")),
        (Locale::De, _) => format!("{count} {}", plural(count, "Punkt", "Punkte")),
    };
    let shown = &names[..names.len().min(10)];
    let more = count - shown.len();
    let list = match (locale, shown.is_empty(), more) {
        (_, true, _) => String::new(),
        (_, false, 0) => format!(" ({})", shown.join(", ")),
        (Locale::En, false, _) => format!(" ({}, and {more} more)", shown.join(", ")),
        (Locale::De, false, _) => format!(" ({} und {more} weitere)", shown.join(", ")),
    };
    format!("{head}: {points}{list}.")
}

/// The caption of a table that lists only some of the points of a big plot.
pub(crate) fn scatter_caption(
    locale: Locale,
    caption: &str,
    (shown, all): (usize, usize),
) -> String {
    match locale {
        Locale::En => format!("{caption} — {shown} of {all} points, the named or the highest"),
        Locale::De => {
            format!("{caption} — {shown} von {all} Punkten, die benannten oder die höchsten")
        }
    }
}

/// One group of Kaplan-Meier curves: its observations, events, median time and last survival.
pub(crate) fn survival_group(
    locale: Locale,
    (label, observations, events): (&str, usize, usize),
    median: Option<&str>,
    last: &str,
) -> String {
    let event = plural(events, "event", "events");
    let ereignis = plural(events, "Ereignis", "Ereignisse");
    match (locale, median) {
        (Locale::En, Some(median)) => format!(
            "{label}: {observations} observations, {events} {event}, median {median}, {last} at the end."
        ),
        (Locale::En, None) => format!(
            "{label}: {observations} observations, {events} {event}, median not reached, {last} at the end."
        ),
        (Locale::De, Some(median)) => format!(
            "{label}: {observations} Beobachtungen, {events} {ereignis}, Median {median}, {last} am Ende."
        ),
        (Locale::De, None) => format!(
            "{label}: {observations} Beobachtungen, {events} {ereignis}, Median nicht erreicht, {last} am Ende."
        ),
    }
}

/// The log-rank test in the plot: the p value, as `< 0.001` or `= 0.013`.
pub(crate) fn survival_test_short(locale: Locale, p: &str) -> String {
    match locale {
        Locale::En => format!("Log-rank p {p}"),
        Locale::De => format!("Log-Rank-Test p {p}"),
    }
}

/// The log-rank test in the description: statistic, degrees of freedom and p.
pub(crate) fn survival_test(locale: Locale, (chi, degrees): (&str, usize), p: &str) -> String {
    match locale {
        Locale::En => format!(
            "Log-rank test: χ² = {chi}, {degrees} degree{} of freedom, p {p}.",
            plural(degrees, "", "s")
        ),
        Locale::De => format!(
            "Log-Rank-Test: χ² = {chi}, {degrees} Freiheitsgrad{}, p {p}.",
            plural(degrees, "", "e")
        ),
    }
}

/// The hazard ratio of two groups with its 95 % limits.
pub(crate) fn survival_hazard(
    locale: Locale,
    (group, reference): (&str, &str),
    (ratio, lower, upper): (&str, &str, &str),
) -> String {
    match locale {
        Locale::En => format!(
            "Hazard ratio of {group} against {reference}: {ratio} (95% confidence interval {lower} to {upper})."
        ),
        Locale::De => format!(
            "Hazard Ratio von {group} gegenüber {reference}: {ratio} (95-%-Konfidenzintervall {lower} bis {upper})."
        ),
    }
}

/// What Kaplan-Meier curves show: how many groups, whether bands are drawn, and each group.
pub(crate) fn survival_summary(
    locale: Locale,
    groups: usize,
    confidence: bool,
    each: &str,
) -> String {
    match locale {
        Locale::En => format!(
            "Kaplan-Meier curves of {groups} group{}{}.{each}",
            plural(groups, "", "s"),
            if confidence {
                ", with 95% confidence bands"
            } else {
                ""
            }
        ),
        Locale::De => format!(
            "Kaplan-Meier-Kurven von {groups} {}{}.{each}",
            plural(groups, "Gruppe", "Gruppen"),
            if confidence {
                ", mit 95-%-Konfidenzbändern"
            } else {
                ""
            }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::survival_group;
    use crate::spec::Locale;

    #[test]
    fn one_event_is_singular() {
        assert_eq!(
            survival_group(Locale::En, ("A", 5, 1), None, "80%"),
            "A: 5 observations, 1 event, median not reached, 80% at the end."
        );
        assert_eq!(
            survival_group(Locale::De, ("A", 5, 1), Some("3"), "80 %"),
            "A: 5 Beobachtungen, 1 Ereignis, Median 3, 80 % am Ende."
        );
    }
}
