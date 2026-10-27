//! User journeys and service blueprints: stages as columns, and for each stage what the customer
//! does, where, thinks and feels, with the sentiment drawn as a line over a scale from -2 to 2 and
//! every value written. A blueprint adds what the service does in front of the customer, behind
//! it and underneath, parted by the line of interaction and the line of visibility. Where each
//! stage has room for 100 pixels the stages stand as columns; on a narrower canvas every stage is
//! a block of its own, from top to bottom. The description walks through the stages and the data
//! table has a row for each.

use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::{pixels, warn_growth_advising},
    draw::{circle, line, rect, text},
    error::ChartWarning,
    layout::{count, fit_text, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Polyline, Scene, TextAnchor},
    spec::{ChartSpec, JourneyRow, JourneySpec, JourneyStageSpec},
    text::{self, JourneyItem},
};

const SIZE: f64 = 12.0;
const LINE: f64 = 14.0;
const PAD: f64 = 7.0;
const GAP: f64 = 6.0;
/// The width of the column of row names, the narrowest a stage column may be and the widest.
const LABEL_WIDTH: f64 = 104.0;
const MIN_COLUMN: f64 = 100.0;
const MAX_COLUMN: f64 = 200.0;
/// The lines a cell may take, the height of the sentiment row, and of a band that holds a line's
/// name.
const MAX_LINES: usize = 4;
const SENTIMENT_HEIGHT: f64 = 96.0;
const BAND: f64 = 24.0;

fn journey(spec: &ChartSpec) -> &JourneySpec {
    spec.journey
        .as_ref()
        .expect("validated journeys carry a journey block")
}

/// What a row of the journey is, in the order they are drawn.
#[derive(Clone, Copy, PartialEq)]
enum Item {
    Text(JourneyRow),
    Sentiment,
    /// The line of interaction, between the customer and the service.
    Interaction,
    /// The line of visibility, between what the customer sees and what they do not.
    Visibility,
}

impl Item {
    const fn word(self) -> JourneyItem {
        match self {
            Self::Text(JourneyRow::Action) => JourneyItem::Action,
            Self::Text(JourneyRow::Touchpoint) => JourneyItem::Touchpoint,
            Self::Text(JourneyRow::Thought) => JourneyItem::Thought,
            Self::Sentiment => JourneyItem::Sentiment,
            Self::Interaction => JourneyItem::Interaction,
            Self::Visibility => JourneyItem::Visibility,
            Self::Text(JourneyRow::Frontstage) => JourneyItem::Frontstage,
            Self::Text(JourneyRow::Backstage) => JourneyItem::Backstage,
            Self::Text(JourneyRow::Support) => JourneyItem::Support,
        }
    }

    const fn is_line(self) -> bool {
        matches!(self, Self::Interaction | Self::Visibility)
    }
}

/// The rows to draw: a row of text where some stage has text for it, the sentiment where some
/// stage has one, and in a blueprint the two lines between the customer and the service.
fn items(journey: &JourneySpec) -> Vec<Item> {
    let has = |row| journey.stages.iter().any(|stage| stage.text(row).is_some());
    let mut items = Vec::new();
    for row in [
        JourneyRow::Action,
        JourneyRow::Touchpoint,
        JourneyRow::Thought,
    ] {
        if has(row) {
            items.push(Item::Text(row));
        }
    }
    if journey.stages.iter().any(|stage| stage.sentiment.is_some()) {
        items.push(Item::Sentiment);
    }
    if journey.is_blueprint() {
        items.push(Item::Interaction);
        if has(JourneyRow::Frontstage) {
            items.push(Item::Text(JourneyRow::Frontstage));
        }
        items.push(Item::Visibility);
        for row in [JourneyRow::Backstage, JourneyRow::Support] {
            if has(row) {
                items.push(Item::Text(row));
            }
        }
    }
    items
}

/// `text` on up to `max_lines` lines of at most `width`, broken at spaces; what does not fit on
/// the last line is shortened.
fn wrap_words(
    content: &str,
    (width, max_lines): (f64, usize),
    path: &str,
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in content.split_whitespace() {
        if lines.len() + 1 == max_lines {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
            continue;
        }
        let candidate = if current.is_empty() {
            word.to_owned()
        } else {
            format!("{current} {word}")
        };
        if current.is_empty() || metrics.width(&candidate, SIZE) <= width {
            current = candidate;
        } else {
            lines.push(std::mem::replace(&mut current, word.to_owned()));
        }
    }
    lines.push(current);
    lines
        .iter()
        .map(|line| fit_text(line, width, SIZE, metrics, warnings, path))
        .collect()
}

fn lines_of(
    stage: (usize, &JourneyStageSpec),
    row: JourneyRow,
    width: f64,
    context: (&mut Vec<ChartWarning>, &impl TextMetrics),
) -> Vec<String> {
    let name = text::journey_path(row);
    stage.1.text(row).map_or_else(Vec::new, |content| {
        wrap_words(
            content,
            (width - 2.0 * PAD, MAX_LINES),
            &format!("/journey/stages/{}/{name}", stage.0),
            context,
        )
    })
}

/// The sentiment as a signed whole number: `+1`, `0`, `-2`.
fn signed(value: i8) -> String {
    if value > 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

fn push_cell(
    elements: &mut Vec<Element>,
    lines: &[String],
    (x, y, width, height): (f64, f64, f64, f64),
    class: (&'static str, &'static str),
) {
    elements.push(rect((x, y, width, height), class.0, None));
    for (position, content) in lines.iter().enumerate() {
        elements.push(text(
            (x + PAD, y + PAD + 11.0 + LINE * count(position)),
            TextAnchor::Start,
            class.1,
            content.clone(),
        ));
    }
}

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let journey = journey(spec);
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
    let stages = journey.stages.len();
    let inner = width - 2.0 * margin;
    let column = ((inner - LABEL_WIDTH - GAP * count(stages)) / count(stages)).floor();
    let needed = if column >= MIN_COLUMN {
        grid(
            spec,
            (margin, top, column.min(MAX_COLUMN)),
            (warnings, metrics),
            &mut elements,
        )
    } else {
        blocks(
            spec,
            (margin, top, inner),
            (warnings, metrics),
            &mut elements,
        )
    };
    warn_growth_advising(
        spec,
        (width, needed),
        "shorten the texts or leave out a row",
        warnings,
    );
    Scene {
        width: spec.width,
        height: pixels(needed.max(height)),
        elements,
    }
}

/// The journey as a table of rows and stage columns; returns the height it needs.
#[allow(clippy::too_many_lines)]
fn grid(
    spec: &ChartSpec,
    (margin, top, column): (f64, f64, f64),
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
    elements: &mut Vec<Element>,
) -> f64 {
    let journey = journey(spec);
    let stages = journey.stages.len();
    let used = LABEL_WIDTH + (column + GAP) * count(stages);
    let left = margin
        + ((f64::from(spec.width) - 2.0 * margin - used) / 2.0)
            .floor()
            .max(0.0);
    let x_of = |stage: usize| left + LABEL_WIDTH + (column + GAP) * count(stage);
    let right = left + used - GAP;

    // The stages along the top.
    let heads: Vec<Vec<String>> = journey
        .stages
        .iter()
        .enumerate()
        .map(|(index, stage)| {
            wrap_words(
                &stage.label,
                (column - 2.0 * PAD, 2),
                &format!("/journey/stages/{index}/label"),
                (warnings, metrics),
            )
        })
        .collect();
    let head_height = heads
        .iter()
        .map(|lines| 2.0 * PAD + LINE * count(lines.len()))
        .fold(32.0, f64::max);
    for (index, lines) in heads.iter().enumerate() {
        push_cell(
            elements,
            lines,
            (x_of(index), top, column, head_height),
            ("chartlet-journey-stage", "chartlet-journey-stage-text"),
        );
    }

    let mut y = top + head_height + GAP;
    for item in items(journey) {
        let name = text::journey_item(spec.locale, item.word());
        if item.is_line() {
            elements.push(text(
                (left, y + 12.0),
                TextAnchor::Start,
                "chartlet-journey-line-label",
                name,
            ));
            elements.push(line(
                (left, y + BAND - 5.0),
                (right, y + BAND - 5.0),
                "chartlet-journey-line",
            ));
            y += BAND + 2.0;
            continue;
        }
        let row_top = y;
        match item {
            Item::Text(row) => {
                let cells: Vec<Vec<String>> = journey
                    .stages
                    .iter()
                    .enumerate()
                    .map(|(index, stage)| {
                        lines_of((index, stage), row, column, (warnings, metrics))
                    })
                    .collect();
                let row_height = cells
                    .iter()
                    .map(|lines| 2.0 * PAD + LINE * count(lines.len()))
                    .fold(34.0, f64::max);
                for (index, lines) in cells.iter().enumerate() {
                    push_cell(
                        elements,
                        lines,
                        (x_of(index), row_top, column, row_height),
                        ("chartlet-journey-cell", "chartlet-journey-text"),
                    );
                }
                y += row_height + GAP;
            }
            Item::Sentiment => {
                sentiment_row(
                    elements,
                    journey,
                    (left, row_top, column, used - GAP),
                    &x_of,
                );
                y += SENTIMENT_HEIGHT + GAP;
            }
            Item::Interaction | Item::Visibility => {}
        }
        elements.push(text(
            (left, row_top + 15.0),
            TextAnchor::Start,
            "chartlet-journey-row",
            fit_text(
                name.as_str(),
                LABEL_WIDTH - 8.0,
                SIZE,
                metrics,
                warnings,
                "/journey",
            ),
        ));
    }
    y + 10.0
}

/// The sentiment row: a band for each level from 2 down to -2, the stages' values joined by a line
/// and written beside their dots.
fn sentiment_row(
    elements: &mut Vec<Element>,
    journey: &JourneySpec,
    (left, top, column, total): (f64, f64, f64, f64),
    x_of: &dyn Fn(usize) -> f64,
) {
    let step = (SENTIMENT_HEIGHT - 24.0) / 4.0;
    let y_of = |value: i8| top + 12.0 + step * f64::from(2 - i32::from(value));
    elements.push(rect(
        (
            left + LABEL_WIDTH,
            top,
            total - LABEL_WIDTH,
            SENTIMENT_HEIGHT,
        ),
        "chartlet-journey-cell",
        None,
    ));
    for value in -2..=2_i8 {
        elements.push(line(
            (left + LABEL_WIDTH, y_of(value)),
            (left + total, y_of(value)),
            if value == 0 {
                "chartlet-journey-zero"
            } else {
                "chartlet-journey-scale"
            },
        ));
        elements.push(text(
            (left + LABEL_WIDTH - 6.0, y_of(value) + 4.0),
            TextAnchor::End,
            "chartlet-journey-scale-text",
            signed(value),
        ));
    }
    // A run of stages with a value is one line; a stage without one breaks it.
    let mut run: Vec<(f64, f64)> = Vec::new();
    let flush = |run: &mut Vec<(f64, f64)>, elements: &mut Vec<Element>| {
        if run.len() > 1 {
            elements.push(Element::Polyline(Polyline {
                points: std::mem::take(run),
                class: "chartlet-journey-sentiment",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
        run.clear();
    };
    for (index, stage) in journey.stages.iter().enumerate() {
        if let Some(value) = stage.sentiment {
            run.push((x_of(index) + column / 2.0, y_of(value)));
        } else {
            flush(&mut run, elements);
        }
    }
    flush(&mut run, elements);
    for (index, stage) in journey.stages.iter().enumerate() {
        if let Some(value) = stage.sentiment {
            let (x, y) = (x_of(index) + column / 2.0, y_of(value));
            elements.push(circle(
                (x, y, 5.0),
                "chartlet-journey-dot",
                Some(format!("{}: {}", stage.label, signed(value))),
            ));
            elements.push(text(
                (x, if value == 2 { y + 18.0 } else { y - 9.0 }),
                TextAnchor::Middle,
                "chartlet-journey-value",
                signed(value),
            ));
        }
    }
}

/// The journey as one block per stage; returns the height it needs.
fn blocks(
    spec: &ChartSpec,
    (margin, top, inner): (f64, f64, f64),
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
    elements: &mut Vec<Element>,
) -> f64 {
    let journey = journey(spec);
    let mut y = top;
    for (index, stage) in journey.stages.iter().enumerate() {
        let head = wrap_words(
            &stage.label,
            (inner - 2.0 * PAD, 2),
            &format!("/journey/stages/{index}/label"),
            (warnings, metrics),
        );
        let head_height = 2.0 * PAD + LINE * count(head.len());
        push_cell(
            elements,
            &head,
            (margin, y, inner, head_height),
            ("chartlet-journey-stage", "chartlet-journey-stage-text"),
        );
        y += head_height + GAP;
        for item in items(journey) {
            let name = text::journey_item(spec.locale, item.word());
            match item {
                Item::Text(row) => {
                    let lines = lines_of((index, stage), row, inner, (warnings, metrics));
                    if lines.is_empty() {
                        continue;
                    }
                    elements.push(text(
                        (margin + PAD, y + 11.0),
                        TextAnchor::Start,
                        "chartlet-journey-row",
                        name,
                    ));
                    let cell_height = 2.0 * PAD + LINE * count(lines.len());
                    push_cell(
                        elements,
                        &lines,
                        (margin, y + 16.0, inner, cell_height),
                        ("chartlet-journey-cell", "chartlet-journey-text"),
                    );
                    y += 16.0 + cell_height + GAP;
                }
                Item::Sentiment => {
                    let Some(value) = stage.sentiment else {
                        continue;
                    };
                    elements.push(text(
                        (margin + PAD, y + 11.0),
                        TextAnchor::Start,
                        "chartlet-journey-row",
                        name,
                    ));
                    // Five steps from -2 to 2: the ones up to the value are filled.
                    for step in 0..5_i8 {
                        elements.push(rect(
                            (margin + PAD + 20.0 * f64::from(step), y + 18.0, 16.0, 14.0),
                            if step - 2 <= value {
                                "chartlet-journey-step chartlet-journey-step-on"
                            } else {
                                "chartlet-journey-step"
                            },
                            None,
                        ));
                    }
                    elements.push(text(
                        (margin + PAD + 108.0, y + 30.0),
                        TextAnchor::Start,
                        "chartlet-journey-value",
                        signed(value),
                    ));
                    y += 38.0 + GAP;
                }
                Item::Interaction | Item::Visibility => {
                    elements.push(text(
                        (margin + PAD, y + 12.0),
                        TextAnchor::Start,
                        "chartlet-journey-line-label",
                        name,
                    ));
                    elements.push(line(
                        (margin, y + BAND - 5.0),
                        (margin + inner, y + BAND - 5.0),
                        "chartlet-journey-line",
                    ));
                    y += BAND + 2.0;
                }
            }
        }
        y += 12.0;
    }
    y
}

/// The journey in words: its stages, and for each what is said of it.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let journey = journey(spec);
    let mut summary =
        text::journey_opening(spec.locale, (journey.stages.len(), journey.is_blueprint()));
    for stage in &journey.stages {
        let mut parts: Vec<String> = Vec::new();
        for item in items(journey) {
            match item {
                Item::Text(row) => {
                    if let Some(content) = stage.text(row) {
                        parts.push(format!(
                            "{}: {content}",
                            text::journey_item(spec.locale, item.word()).to_lowercase()
                        ));
                    }
                }
                Item::Sentiment => {
                    if let Some(value) = stage.sentiment {
                        parts.push(format!(
                            "{}: {}",
                            text::journey_item(spec.locale, item.word()).to_lowercase(),
                            signed(value)
                        ));
                    }
                }
                Item::Interaction | Item::Visibility => {}
            }
        }
        summary.push(' ');
        summary.push_str(&stage.label);
        if parts.is_empty() {
            summary.push('.');
        } else {
            let _ = write!(summary, ": {}.", parts.join("; "));
        }
    }
    summary
}

/// One row per stage with a column for each row of the journey.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let journey = journey(spec);
    let items: Vec<Item> = items(journey)
        .into_iter()
        .filter(|item| !item.is_line())
        .collect();
    let mut columns = vec![text::journey_item(spec.locale, JourneyItem::Stage)];
    columns.extend(
        items
            .iter()
            .map(|item| text::journey_item(spec.locale, item.word())),
    );
    let rows = journey
        .stages
        .iter()
        .map(|stage| {
            let mut row = vec![stage.label.clone()];
            row.extend(items.iter().map(|item| match item {
                Item::Text(kind) => stage.text(*kind).unwrap_or_default().to_owned(),
                Item::Sentiment => stage.sentiment.map(signed).unwrap_or_default(),
                Item::Interaction | Item::Visibility => String::new(),
            }));
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", spec.locale.words().data_for, spec.title),
        columns,
        rows,
    }
}
