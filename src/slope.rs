//! Slope and rank charts: a `line` chart with `"slope": true` compares each series between two
//! categories, a `"rank": true` chart plots the rank of each series, 1 at the top, across
//! several. The names stand at both ends of the lines, with the value or rank beside them, so
//! that the chart needs no legend; names that would overlap are spread apart. Every series keeps
//! its own color, pattern and weight, and the description names the biggest rise and fall.

use crate::{
    error::ChartWarning,
    layout::{
        LABEL_SIZE, NumericScale, SERIES_LINE_CLASSES, SERIES_POINT_CLASSES, count, fit_text,
        format_value, push_title, title_extra,
    },
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Polyline, Scene, Text, TextAnchor},
    spec::{ChartSpec, NumberStyle},
    text,
};

/// The least distance between the names at one end, and the room above and below the lines.
const NAME_GAP: f64 = 16.0;
const HEADER: f64 = 36.0;
const FOOT: f64 = 24.0;

/// Whether the line chart is drawn as a slope or a rank chart.
pub(crate) fn applies(spec: &ChartSpec) -> bool {
    spec.slope || spec.rank
}

/// The series with a value in every category, and their ranks.
struct Lines {
    names: Vec<String>,
    categories: Vec<String>,
    values: Vec<Vec<f64>>,
    /// The rank of every series in every category, 1 for the largest value; equal values keep
    /// the order of the series.
    ranks: Vec<Vec<usize>>,
}

impl Lines {
    fn new(spec: &ChartSpec) -> Self {
        let dataset = spec.dataset();
        let values: Vec<Vec<f64>> = dataset
            .series
            .iter()
            .map(|series| {
                series
                    .values
                    .iter()
                    .map(|value| value.expect("validated lines have a value in every category"))
                    .collect()
            })
            .collect();
        let ranks = (0..values.len())
            .map(|series| {
                (0..dataset.categories.len())
                    .map(|category| {
                        1 + (0..values.len())
                            .filter(|other| {
                                let (mine, theirs) =
                                    (values[series][category], values[*other][category]);
                                match theirs.total_cmp(&mine) {
                                    std::cmp::Ordering::Greater => true,
                                    std::cmp::Ordering::Equal => *other < series,
                                    std::cmp::Ordering::Less => false,
                                }
                            })
                            .count()
                    })
                    .collect()
            })
            .collect();
        Self {
            names: dataset
                .series
                .iter()
                .map(|series| series.name.clone().unwrap_or_default())
                .collect(),
            categories: dataset.categories,
            values,
            ranks,
        }
    }

    fn last(&self) -> usize {
        self.categories.len() - 1
    }

    /// What stands beside a line at a category: its value in a slope chart, its rank in a rank
    /// chart.
    fn mark(&self, spec: &ChartSpec, series: usize, category: usize) -> String {
        if spec.rank {
            format!("#{}", self.ranks[series][category])
        } else {
            format_value(self.values[series][category], spec.number_style())
        }
    }
}

/// The positions of `ys` moved apart to at least `gap`, within `low` and `high`, in the same
/// order.
fn spread(ys: &[f64], gap: f64, (low, high): (f64, f64)) -> Vec<f64> {
    let mut order: Vec<usize> = (0..ys.len()).collect();
    crate::sort::by(&mut order, |a, b| ys[*a].total_cmp(&ys[*b]));
    let mut placed: Vec<f64> = order.iter().map(|index| ys[*index].max(low)).collect();
    for position in 1..placed.len() {
        placed[position] = placed[position].max(placed[position - 1] + gap);
    }
    for position in (0..placed.len()).rev() {
        let room = high - gap * count(placed.len() - 1 - position);
        placed[position] = placed[position].min(room);
    }
    let mut spread = vec![0.0; ys.len()];
    for (position, index) in order.iter().enumerate() {
        spread[*index] = placed[position];
    }
    spread
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let lines = Lines::new(spec);
    let words = spec.locale.words();
    let compact = spec.width < crate::layout::NARROW;
    let width = f64::from(spec.width);
    let margin = if compact { 12.0 } else { 24.0 };
    let inner = width - 2.0 * margin;
    let mut elements = Vec::new();
    push_title(&mut elements, spec, margin, inner, metrics, warnings);
    let top = 56.0 + title_extra(spec, inner, metrics) + HEADER;
    let series = lines.names.len();
    let height = f64::from(spec.height).max(top + NAME_GAP * count(series) + FOOT);
    let bottom = height - FOOT;
    let (first, last) = (0, lines.last());
    let end_label = |index: usize, category: usize, left: bool| {
        let (name, mark) = (&lines.names[index], lines.mark(spec, index, category));
        if left {
            format!("{name}  {mark}")
        } else {
            format!("{mark}  {name}")
        }
    };
    let label_width = |category: usize, left: bool| {
        (0..series)
            .map(|index| metrics.width(&end_label(index, category, left), LABEL_SIZE))
            .fold(0.0, f64::max)
            .min(inner * 0.3)
    };
    let (left_width, right_width) = (label_width(first, true), label_width(last, false));
    let (x0, x1) = (
        margin + left_width + 14.0,
        width - margin - right_width - 14.0,
    );
    let x_at = |category: usize| x0 + (x1 - x0) * count(category) / count(last);
    let scale = NumericScale::for_axis(
        lines.values.iter().flatten().copied(),
        false,
        &spec.value_axis,
    );
    let y_of = |index: usize, category: usize| {
        if spec.rank {
            let share = count(lines.ranks[index][category] - 1) / count(series - 1);
            top + share * (bottom - top)
        } else {
            scale.map(lines.values[index][category], bottom, top)
        }
    };
    let spacing = (x1 - x0) / count(last);
    for category in 0..lines.categories.len() {
        let x = x_at(category);
        elements.push(Element::Line(Line {
            x1: x,
            y1: top - 8.0,
            x2: x,
            y2: bottom + 8.0,
            class: "chartlet-grid",
        }));
        elements.push(Element::Text(Text {
            x,
            y: top - 18.0,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content: fit_text(
                &lines.categories[category],
                (spacing - 8.0).max(60.0),
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/categories/{category}"),
            ),
        }));
    }
    let style: NumberStyle = spec.number_style();
    for index in 0..series {
        let points: Vec<(f64, f64)> = (0..lines.categories.len())
            .map(|category| (x_at(category), y_of(index, category)))
            .collect();
        elements.push(Element::Polyline(Polyline {
            points: points.clone(),
            class: SERIES_LINE_CLASSES[index],
            series_index: Some(index),
            style_index: None,
            tooltip: None,
        }));
        for (category, (x, y)) in points.into_iter().enumerate() {
            let value = format_value(lines.values[index][category], style);
            let tooltip = if spec.rank {
                format!(
                    "{}, {}: {} {} ({value})",
                    lines.names[index],
                    lines.categories[category],
                    words.rank,
                    lines.ranks[index][category]
                )
            } else {
                format!(
                    "{}, {}: {value}",
                    lines.names[index], lines.categories[category]
                )
            };
            elements.push(Element::Circle(Circle {
                cx: x,
                cy: y,
                radius: 4.0,
                class: SERIES_POINT_CLASSES[index],
                series_index: Some(index),
                style_index: None,
                tooltip: Some(tooltip),
            }));
        }
    }
    for (category, left) in [(first, true), (last, false)] {
        let ys: Vec<f64> = (0..series).map(|index| y_of(index, category)).collect();
        let placed = spread(&ys, NAME_GAP, (top, bottom));
        for (index, y) in placed.into_iter().enumerate() {
            let (x, anchor, room) = if left {
                (x0 - 10.0, TextAnchor::End, left_width)
            } else {
                (x1 + 10.0, TextAnchor::Start, right_width)
            };
            elements.push(Element::Text(Text {
                x,
                y: y + 4.0,
                class: "chartlet-label",
                anchor,
                content: fit_text(
                    &end_label(index, category, left),
                    room,
                    LABEL_SIZE,
                    metrics,
                    warnings,
                    &format!("/series/{index}/name"),
                ),
            }));
        }
    }
    Scene {
        width: spec.width,
        height: crate::diagram::pixels(height),
        elements,
    }
}

/// The change of a value, written with its sign.
fn signed(spec: &ChartSpec, delta: f64) -> String {
    let text = format_value(delta, spec.number_style());
    if delta > 0.0 {
        format!("+{text}")
    } else {
        text
    }
}

/// The chart in a sentence: what it compares, every series from its first value or rank to its
/// last, and the biggest rise and fall.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let lines = Lines::new(spec);
    let (from, to) = (
        lines.categories[0].as_str(),
        lines.categories[lines.last()].as_str(),
    );
    let series = lines.names.len();
    // The first series with the largest rise or the largest fall: equal changes keep their order.
    let best = |changes: &[(usize, f64)], rise: bool| {
        let mut found: Option<(usize, f64)> = None;
        for &(index, change) in changes {
            let better = if rise { change > 0.0 } else { change < 0.0 }
                && found.is_none_or(|(_, best)| if rise { change > best } else { change < best });
            if better {
                found = Some((index, change));
            }
        }
        found.map(|(index, _)| index)
    };
    if spec.rank {
        // A rise in rank is a move toward 1: the change is how many places were gained.
        let changes: Vec<(usize, f64)> = (0..series)
            .map(|index| {
                let ranks = &lines.ranks[index];
                #[allow(clippy::cast_precision_loss)]
                let gained = ranks[0] as f64 - ranks[ranks.len() - 1] as f64;
                (index, gained)
            })
            .collect();
        let listed: Vec<String> = (0..series)
            .map(|index| {
                let ranks: Vec<String> =
                    lines.ranks[index].iter().map(ToString::to_string).collect();
                format!("{}: {}", lines.names[index], ranks.join(", "))
            })
            .collect();
        let named = |rise: bool| {
            best(&changes, rise).map(|index| {
                let ranks = &lines.ranks[index];
                format!(
                    "{} ({} → {})",
                    lines.names[index],
                    ranks[0],
                    ranks[ranks.len() - 1]
                )
            })
        };
        return text::rank_summary(
            spec.locale,
            (series, lines.categories.len(), from, to),
            &listed.join("; "),
            (named(true).as_deref(), named(false).as_deref()),
        );
    }
    let changes: Vec<(usize, f64)> = (0..series)
        .map(|index| (index, lines.values[index][1] - lines.values[index][0]))
        .collect();
    let style = spec.number_style();
    let listed: Vec<String> = changes
        .iter()
        .map(|(index, change)| {
            format!(
                "{} {} → {} ({})",
                lines.names[*index],
                format_value(lines.values[*index][0], style),
                format_value(lines.values[*index][1], style),
                signed(spec, *change)
            )
        })
        .collect();
    let named = |rise: bool| {
        best(&changes, rise).map(|index| {
            let change = lines.values[index][1] - lines.values[index][0];
            format!("{} ({})", lines.names[index], signed(spec, change))
        })
    };
    text::slope_summary(
        spec.locale,
        (series, from, to),
        &listed.join("; "),
        (named(true).as_deref(), named(false).as_deref()),
    )
}
