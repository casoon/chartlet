//! Parliaments: the seats of an assembly as dots in a semicircle, in blocks by party from left to
//! right. A dashed line marks the seat that makes a majority, a ring marks the seats of a
//! coalition. The legend, the description and the data table give every party's seats and share.

use std::f64::consts::PI;

use crate::{
    DataTable,
    diagram::pixels,
    error::ChartWarning,
    layout::{LABEL_SIZE, count, fit_text, format_value, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, NumberStyle, ParliamentSpec, ValueFormat},
    text,
};

const PARTY_CLASSES: [&str; 8] = [
    "chartlet-parl-seat chartlet-parl-1",
    "chartlet-parl-seat chartlet-parl-2",
    "chartlet-parl-seat chartlet-parl-3",
    "chartlet-parl-seat chartlet-parl-4",
    "chartlet-parl-seat chartlet-parl-5",
    "chartlet-parl-seat chartlet-parl-6",
    "chartlet-parl-seat chartlet-parl-7",
    "chartlet-parl-seat chartlet-parl-8",
];
const ENTRY: f64 = 22.0;
/// The inner radius of the semicircle, as a share of the outer.
const HOLE: f64 = 0.38;

fn parliament(spec: &ChartSpec) -> &ParliamentSpec {
    spec.parliament
        .as_ref()
        .expect("validated parliaments carry a parliament block")
}

fn share(spec: &ChartSpec, seats: u32, total: u32) -> String {
    format_value(
        f64::from(seats) / f64::from(total),
        NumberStyle {
            format: ValueFormat::Percent,
            decimals: Some(1),
            ..spec.number_style()
        },
    )
}

/// One seat: where it lies on the semicircle, as an angle from the left and a radius.
#[derive(Debug, Clone, Copy)]
struct Seat {
    angle: f64,
    radius: f64,
}

/// The seats of an assembly on rows of a semicircle of radius `outer`, and the radius of a dot:
/// of the numbers of rows that were tried, the one whose dots come out biggest. Seats come
/// left to right.
fn seats(total: u32, outer: f64) -> (Vec<Seat>, f64) {
    let total_f = f64::from(total);
    let mut best: (f64, Vec<Seat>) = (0.0, Vec::new());
    for rows in 1..=24_usize {
        let inner = outer * HOLE;
        let radii: Vec<f64> = (0..rows)
            .map(|row| {
                if rows == 1 {
                    outer
                } else {
                    inner + (outer - inner) * count(row) / count(rows - 1)
                }
            })
            .collect();
        let sum: f64 = radii.iter().sum();
        // Each row takes seats in proportion to its radius, by the largest remainder.
        let shares: Vec<f64> = radii.iter().map(|radius| total_f * radius / sum).collect();
        let mut per_row: Vec<u32> = shares.iter().map(|share| whole(*share).max(1)).collect();
        if per_row.iter().sum::<u32>() > total {
            continue;
        }
        while per_row.iter().sum::<u32>() < total {
            let next = (0..rows)
                .max_by(|a, b| {
                    (shares[*a] - f64::from(per_row[*a]))
                        .total_cmp(&(shares[*b] - f64::from(per_row[*b])))
                })
                .expect("a row");
            per_row[next] += 1;
        }
        let gap = if rows > 1 {
            (outer - inner) / count(rows - 1)
        } else {
            f64::INFINITY
        };
        let reach = radii
            .iter()
            .zip(&per_row)
            .map(|(radius, seats)| {
                if *seats > 1 {
                    PI * radius / f64::from(seats - 1)
                } else {
                    f64::INFINITY
                }
            })
            .fold(gap, f64::min);
        let dot = reach * 0.42;
        if dot > best.0 {
            let mut all = Vec::new();
            for (radius, seats) in radii.iter().zip(&per_row) {
                for seat in 0..*seats {
                    let angle = if *seats == 1 {
                        PI / 2.0
                    } else {
                        PI - PI * f64::from(seat) / f64::from(seats - 1)
                    };
                    all.push(Seat {
                        angle,
                        radius: *radius,
                    });
                }
            }
            best = (dot, all);
        }
    }
    let (dot, mut all) = best;
    // Left to right, the inner seats of a column before the outer ones.
    all.sort_by(|a, b| {
        b.angle
            .total_cmp(&a.angle)
            .then(a.radius.total_cmp(&b.radius))
    });
    (all, dot.min(outer * 0.12))
}

/// The whole squares of a share: its floor.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn whole(share: f64) -> u32 {
    share.floor() as u32
}

/// The dots: seats go to the parties in blocks, left to right, a coalition's seats with a ring.
fn push_seats(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    (center_x, center_y): (f64, f64),
    (positions, dot): (&[Seat], f64),
) {
    let parliament = parliament(spec);
    let mut party = 0;
    let mut taken = 0;
    for seat in positions {
        while taken == parliament.parties[party].seats {
            party += 1;
            taken = 0;
        }
        taken += 1;
        let (x, y) = (
            center_x + seat.radius * seat.angle.cos(),
            center_y - seat.radius * seat.angle.sin(),
        );
        let tooltip = Some(format!(
            "{}: {} {taken} / {}",
            parliament.parties[party].label,
            spec.locale.words().seat,
            parliament.parties[party].seats
        ));
        elements.push(Element::Circle(Circle {
            cx: x,
            cy: y,
            radius: dot,
            class: PARTY_CLASSES[party],
            topic: None,
            series_index: None,
            style_index: None,
            tooltip,
        }));
        if parliament.in_coalition(party) {
            elements.push(Element::Circle(Circle {
                cx: x,
                cy: y,
                radius: dot + 2.0,
                class: "chartlet-parl-coalition",
                topic: None,
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
    }
}

/// The legend: a swatch and the seats and share of every party, then the coalition in a line.
fn push_legend(
    elements: &mut Vec<Element>,
    spec: &ChartSpec,
    legend_top: f64,
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) {
    let parliament = parliament(spec);
    let total = parliament.total();
    let width = f64::from(spec.width);
    let margin = if spec.width < crate::layout::NARROW {
        12.0
    } else {
        24.0
    };
    let legend_left = (width / 2.0 - 150.0).max(margin);
    for (index, entry) in parliament.parties.iter().enumerate() {
        let y = legend_top + ENTRY * count(index);
        elements.push(Element::Rect(Rect {
            x: legend_left,
            y,
            width: 12.0,
            height: 12.0,
            class: PARTY_CLASSES[index],
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: legend_left + 20.0,
            y: y + 10.5,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                &format!(
                    "{} – {} ({})",
                    entry.label,
                    entry.seats,
                    share(spec, entry.seats, total)
                ),
                width - legend_left - margin - 20.0,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/parliament/parties/{index}/label"),
            ),
        }));
    }
    if !parliament.coalition.is_empty() {
        elements.push(Element::Text(Text {
            x: legend_left,
            y: legend_top + ENTRY * count(parliament.parties.len()) + 10.5,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: fit_text(
                &coalition_note(spec),
                width - legend_left - margin,
                LABEL_SIZE,
                metrics,
                warnings,
                "/parliament/coalition",
            ),
        }));
    }
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let parliament = parliament(spec);
    let total = parliament.total();
    let compact = spec.width < crate::layout::NARROW;
    let (width, height) = (f64::from(spec.width), f64::from(spec.height));
    let margin = if compact { 12.0 } else { 24.0 };
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
    let legend_rows = count(parliament.parties.len())
        + if parliament.coalition.is_empty() {
            0.0
        } else {
            1.0
        };
    let legend_height = ENTRY * legend_rows + 12.0;
    let outer = ((width - 2.0 * margin) / 2.0)
        .min(height - top - legend_height - 40.0)
        .max(60.0);
    let (positions, dot) = seats(total, outer);
    let (center_x, center_y) = (width / 2.0, top + outer + dot + 4.0);

    push_seats(&mut elements, spec, (center_x, center_y), (&positions, dot));
    if parliament.majority && total > 1 {
        // Between the last seat short of a majority and the first one that makes it.
        let at = parliament.majority_of() as usize;
        let angle = f64::midpoint(
            positions[at - 1].angle,
            positions[at.min(positions.len() - 1)].angle,
        );
        let (inner, far) = (outer * HOLE - dot - 6.0, outer + dot + 6.0);
        elements.push(Element::Line(Line {
            x1: center_x + inner * angle.cos(),
            y1: center_y - inner * angle.sin(),
            x2: center_x + far * angle.cos(),
            y2: center_y - far * angle.sin(),
            class: "chartlet-parl-majority",
        }));
    }
    elements.push(Element::Text(Text {
        x: center_x,
        y: center_y - 6.0,
        class: "chartlet-parl-total",
        anchor: TextAnchor::Middle,
        content: total.to_string(),
    }));
    let words = spec.locale.words();
    let mut note = words.seats.to_lowercase();
    if parliament.majority {
        note = format!("{note} · {} {}", words.majority, parliament.majority_of());
    }
    elements.push(Element::Text(Text {
        x: center_x,
        y: center_y + 14.0,
        class: "chartlet-parl-note",
        anchor: TextAnchor::Middle,
        content: note,
    }));
    let legend_top = center_y + 28.0;
    push_legend(&mut elements, spec, legend_top, (warnings, metrics));
    Scene {
        width: spec.width,
        height: pixels(height.max(legend_top + legend_height)),
        elements,
    }
}

/// The coalition in a line: its parties, its seats and whether they make a majority.
fn coalition_note(spec: &ChartSpec) -> String {
    let parliament = parliament(spec);
    text::parliament_coalition(
        spec.locale,
        &parliament.coalition.join(" + "),
        parliament.coalition_seats(),
        parliament.coalition_seats() >= parliament.majority_of(),
    )
}

/// The parliament in sentences: its seats and parties, the majority, and the coalition.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let parliament = parliament(spec);
    let total = parliament.total();
    let parties: Vec<String> = parliament
        .parties
        .iter()
        .map(|party| {
            format!(
                "{} {} ({})",
                party.label,
                party.seats,
                share(spec, party.seats, total)
            )
        })
        .collect();
    let mut description = text::parliament_summary(
        spec.locale,
        (total, parliament.parties.len()),
        &parties.join(", "),
        parliament.majority.then(|| parliament.majority_of()),
    );
    if !parliament.coalition.is_empty() {
        description.push(' ');
        description.push_str(&coalition_note(spec));
    }
    description
}

/// One row per party: its seats and share, and whether it is in the coalition.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let parliament = parliament(spec);
    let words = spec.locale.words();
    let total = parliament.total();
    let coalition = !parliament.coalition.is_empty();
    let mut columns = vec![
        words.party.to_owned(),
        words.seats.to_owned(),
        words.share.to_owned(),
    ];
    if coalition {
        columns.push(words.coalition.to_owned());
    }
    let rows = parliament
        .parties
        .iter()
        .enumerate()
        .map(|(index, party)| {
            let mut row = vec![
                party.label.clone(),
                party.seats.to_string(),
                share(spec, party.seats, total),
            ];
            if coalition {
                row.push(if parliament.in_coalition(index) {
                    words.yes.to_owned()
                } else {
                    String::new()
                });
            }
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}
