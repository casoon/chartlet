//! Point-and-figure charts: columns of X (rising) and O (falling), one mark per filled box, on a
//! grid of boxes. The horizontal axis counts columns, not time; the vertical axis is the value.
//! Reference lines mark levels such as a count line, and highlights tint a range of columns.
//! The columns come computed: the chart draws them and decides nothing about reversals.

use std::fmt::Write as _;

use crate::{
    DataTable,
    error::ChartWarning,
    layout::{
        LABEL_SIZE, NumericScale, PlotArea, add_bottom_category_title, base_elements, count,
        fit_text, format_value, title_extra,
    },
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Rect, Scene, Text, TextAnchor},
    spec::{ChartSpec, PnfColumnSpec, PnfDirection, PointFigureSpec},
    text,
};

/// The share of a box, and of a column's width, that a mark reaches from its center, and the
/// least and most it reaches.
const MARK_SHARE: f64 = 0.36;
const MIN_MARK: f64 = 1.5;
const MAX_MARK: f64 = 9.0;
/// The box grid is drawn only while a box is at least this tall: below, its lines would merge.
const MIN_GRID_BOX: f64 = 6.0;
/// Below this width of a column or height of a box the marks cannot be told apart.
const MIN_COLUMN: f64 = 4.0;
const MIN_BOX: f64 = 3.0;
/// How much wider the semibold label of a reference line runs than the regular text the metrics
/// measure, and the largest share of the plot its room may take from the columns.
const LABEL_WEIGHT: f64 = 1.08;
const MAX_LABEL_SHARE: f64 = 0.35;

fn chart(spec: &ChartSpec) -> &PointFigureSpec {
    spec.pointfigure
        .as_ref()
        .expect("validated point-and-figure charts carry a pointfigure block")
}

/// The mark of a column, as a word with its direction.
fn direction(spec: &ChartSpec, column: &PnfColumnSpec) -> &'static str {
    let words = spec.locale.words();
    match column.direction {
        PnfDirection::Up => words.rising_x,
        PnfDirection::Down => words.falling_o,
    }
}

/// The values of the boxes a column fills, from its lowest up.
fn box_values(chart: &PointFigureSpec, column: &PnfColumnSpec) -> impl Iterator<Item = f64> {
    let box_size = chart.box_size;
    (0..chart.boxes(column)).map(move |index| column.low + count(index) * box_size)
}

/// The column numbers the axis shows: every column if the widest number fits its column,
/// otherwise every 2nd, 5th, 10th and so on, and the first column too while it keeps clear of
/// the next number.
fn numbered(columns: usize, band: f64, metrics: &impl TextMetrics) -> Vec<usize> {
    let widest = metrics.width(&columns.to_string(), LABEL_SIZE) + 8.0;
    let step = [1, 2, 5, 10, 20, 50, 100]
        .into_iter()
        .find(|step| band * count(*step) >= widest)
        .unwrap_or(200);
    (1..=columns)
        .filter(|number| number % step == 0 || (*number == 1 && band * count(step - 1) >= widest))
        .collect()
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let chart = chart(spec);
    let half = chart.box_size / 2.0;
    let values = chart
        .columns
        .iter()
        .flat_map(|column| [column.low - half, column.high + half])
        .chain(crate::reference::values(spec));
    let scale = NumericScale::for_axis(values, false, &spec.value_axis);

    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = f64::from(crate::layout::axis_gutter(spec.width));
    let right = f64::from(crate::layout::plot_margin(spec.width));
    let bottom = if spec.category_axis.title.is_some() {
        62.0
    } else {
        42.0
    };
    let top = 78.0 + title_extra(spec, width - left - right, metrics);
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: crate::layout::plot_height(height - top - bottom, warnings),
        vertical_bars: true,
    };
    let plot_bottom = plot.top + plot.height;
    let along = |value| scale.map(value, plot_bottom, plot.top);
    // The labels of the reference lines stand right of the last column, where they cover no
    // mark; the lines themselves run across the whole plot.
    let label_room = spec
        .references
        .iter()
        .map(|reference| metrics.width(&reference.label, LABEL_SIZE) * LABEL_WEIGHT + 12.0)
        .fold(0.0, f64::max)
        .min(plot.width * MAX_LABEL_SHARE);
    let band = (plot.width - label_room) / count(chart.columns.len());
    let center = |index: usize| plot.left + band * (count(index) + 0.5);
    let box_height = (along(0.0) - along(chart.box_size)).abs();
    let reach = (box_height.min(band) * MARK_SHARE).clamp(MIN_MARK, MAX_MARK);

    let mut elements = base_elements(spec, &scale, plot, warnings, metrics);

    for highlight in &chart.highlights {
        let x = center(highlight.from - 1) - band / 2.0;
        elements.push(Element::Rect(Rect {
            x,
            y: plot.top,
            width: band * count(highlight.to - highlight.from + 1),
            height: plot.height,
            class: "chartlet-pnf-highlight",
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }

    // The box grid: a line between every two boxes, on the grid of the first column.
    if box_height >= MIN_GRID_BOX {
        let (min, max) = scale.ends();
        let origin = chart.columns[0].low - half;
        let first = ((min - origin) / chart.box_size).ceil();
        let mut line = first;
        loop {
            let value = origin + line * chart.box_size;
            if value > max + chart.box_size * 1e-9 {
                break;
            }
            let y = along(value);
            elements.push(Element::Line(Line {
                x1: plot.left,
                y1: y,
                x2: plot.left + plot.width,
                y2: y,
                class: "chartlet-pnf-box",
            }));
            line += 1.0;
        }
    }
    let under = elements.len();

    for (index, column) in chart.columns.iter().enumerate() {
        let x = center(index);
        for value in box_values(chart, column) {
            let y = along(value);
            match column.direction {
                PnfDirection::Up => {
                    for (from, to) in [(-reach, reach), (reach, -reach)] {
                        elements.push(Element::Line(Line {
                            x1: x - reach,
                            y1: y + from,
                            x2: x + reach,
                            y2: y + to,
                            class: "chartlet-pnf-x",
                        }));
                    }
                }
                PnfDirection::Down => elements.push(Element::Circle(Circle {
                    cx: x,
                    cy: y,
                    radius: reach,
                    class: "chartlet-pnf-o",
                    series_index: None,
                    style_index: None,
                    tooltip: None,
                })),
            }
        }
    }

    // One target per column carries its tooltip, so that a mark need not be hit exactly.
    for (index, column) in chart.columns.iter().enumerate() {
        let (y_top, y_low) = (along(column.high + half), along(column.low - half));
        elements.push(Element::Rect(Rect {
            x: center(index) - band / 2.0,
            y: y_top,
            width: band,
            height: y_low - y_top,
            class: "chartlet-pnf-hit",
            series_index: None,
            style_index: None,
            tooltip: Some(column_tooltip(spec, chart, index)),
        }));
    }

    for number in numbered(chart.columns.len(), band, metrics) {
        elements.push(Element::Text(Text {
            x: center(number - 1),
            y: plot_bottom + 20.0,
            class: "chartlet-tick",
            anchor: TextAnchor::Middle,
            content: number.to_string(),
        }));
    }
    add_bottom_category_title(
        spec,
        plot.left,
        plot.width,
        height,
        &mut elements,
        warnings,
        metrics,
    );

    for (index, highlight) in chart.highlights.iter().enumerate() {
        let x = center(highlight.from - 1) - band / 2.0 + 6.0;
        elements.push(Element::Text(Text {
            x,
            y: plot.top + 14.0,
            class: "chartlet-pnf-label",
            anchor: TextAnchor::Start,
            content: fit_text(
                &highlight.label,
                plot.left + plot.width - x,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/pointfigure/highlights/{index}/label"),
            ),
        }));
    }
    crate::reference::push(spec, scale, plot, under, &mut elements, warnings, metrics);

    if band < MIN_COLUMN || box_height < MIN_BOX {
        warnings.push(ChartWarning::new(
            "dense_chart",
            "/pointfigure/columns",
            format!(
                "{} columns of up to {} boxes leave less than {MIN_COLUMN} pixels per column or {MIN_BOX} per box; raise width or height, raise boxSize or show fewer columns",
                chart.columns.len(),
                chart
                    .columns
                    .iter()
                    .map(|column| chart.boxes(column))
                    .max()
                    .unwrap_or(1)
            ),
        ));
    }

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// A column in a few words: its number, its mark and direction, its lowest and highest box and
/// how many boxes it fills.
fn column_tooltip(spec: &ChartSpec, chart: &PointFigureSpec, index: usize) -> String {
    let words = spec.locale.words();
    let column = &chart.columns[index];
    format!(
        "{} {}: {}, {} {} {} ({}: {})",
        words.column,
        index + 1,
        direction(spec, column),
        format_value(column.low, spec.number_style()),
        words.to,
        format_value(column.high, spec.number_style()),
        words.boxes,
        chart.boxes(column)
    )
}

/// The chart in a few sentences: its columns and box size, the values they span, the last
/// column, the highlighted ranges and the reference lines.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let chart = chart(spec);
    let show = |value| format_value(value, spec.number_style());
    let rising = chart
        .columns
        .iter()
        .filter(|column| column.direction == PnfDirection::Up)
        .count();
    let lowest = chart
        .columns
        .iter()
        .map(|column| column.low)
        .fold(f64::INFINITY, f64::min);
    let highest = chart
        .columns
        .iter()
        .map(|column| column.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let last = chart
        .columns
        .last()
        .expect("validated point-and-figure charts have a column");
    let mut description = text::pointfigure_summary(
        spec.locale,
        (chart.columns.len(), &show(chart.box_size)),
        (rising, chart.columns.len() - rising),
        (&show(lowest), &show(highest)),
        (
            last.direction == PnfDirection::Up,
            &show(last.low),
            &show(last.high),
            chart.boxes(last),
        ),
    );
    if !chart.highlights.is_empty() {
        let ranges: Vec<(usize, usize, &str)> = chart
            .highlights
            .iter()
            .map(|highlight| (highlight.from, highlight.to, highlight.label.as_str()))
            .collect();
        description.push_str(&text::pointfigure_highlights(spec.locale, &ranges));
    }
    if !spec.references.is_empty() {
        let words = spec.locale.words();
        let references: Vec<String> = spec
            .references
            .iter()
            .map(|reference| format!("{} {} {}", reference.label, words.at, show(reference.value)))
            .collect();
        write!(
            description,
            " {}: {}.",
            words.reference_lines,
            references.join("; ")
        )
        .expect("writing to String cannot fail");
    }
    description
}

/// One row per column: its number, its mark, its lowest and highest box and how many it fills.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let words = spec.locale.words();
    let chart = chart(spec);
    let rows = chart
        .columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            vec![
                (index + 1).to_string(),
                direction(spec, column).to_owned(),
                format_value(column.low, spec.number_style()),
                format_value(column.high, spec.number_style()),
                chart.boxes(column).to_string(),
            ]
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns: vec![
            words.column.to_owned(),
            words.direction.to_owned(),
            words.lowest_box.to_owned(),
            words.highest_box.to_owned(),
            words.boxes.to_owned(),
        ],
        rows,
    }
}
