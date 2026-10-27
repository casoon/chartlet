//! Kanban boards: columns for the states of the work, each with its cards. A column with a limit
//! shows its count as `3/4`, and one that holds more cards than that is marked with a sign and
//! its own outline, not by color alone. A blocked card carries a sign and the word. Columns stand
//! side by side where each has room for at least 150 pixels, and one under the other where not.
//! The description counts the cards of every column and names the blocked ones; the data table
//! has a row for every card.

use crate::{
    DataTable,
    diagram::{pixels, warn_growth_advising, wrap},
    draw::{circle, line, rect, text},
    error::ChartWarning,
    layout::{count, fit_text, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Element, Scene, TextAnchor},
    spec::{ChartSpec, KanbanCardSpec, KanbanColumnSpec, KanbanSpec},
    text,
};

const SIZE: f64 = 12.0;
const SMALL: f64 = 11.0;
const LINE: f64 = 14.0;
/// The padding inside a card, and the space between cards, lanes and a lane's edge.
const PAD: f64 = 8.0;
const GAP: f64 = 8.0;
const HEADER: f64 = 34.0;
/// The narrowest a column may be before the columns stand one under the other.
const MIN_COLUMN: f64 = 150.0;
const MAX_COLUMN: f64 = 280.0;

fn kanban(spec: &ChartSpec) -> &KanbanSpec {
    spec.kanban
        .as_ref()
        .expect("validated kanban boards carry a kanban block")
}

/// A card laid out for a column of a given width: its lines of title and its height.
struct Card<'a> {
    spec: &'a KanbanCardSpec,
    title: Vec<String>,
    height: f64,
}

fn card<'a>(
    (column, index, card): (usize, usize, &'a KanbanCardSpec),
    width: f64,
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) -> Card<'a> {
    let title = wrap(
        &card.title,
        width - 2.0 * PAD,
        SIZE,
        metrics,
        warnings,
        &format!("/kanban/columns/{column}/cards/{index}/title"),
    );
    let meta = card.owner.is_some() || card.tag.is_some();
    let height = 2.0 * PAD
        + LINE * count(title.len())
        + if meta { 22.0 } else { 0.0 }
        + if card.blocked { 20.0 } else { 0.0 };
    Card {
        spec: card,
        title,
        height,
    }
}

/// The text of a column's count: the cards, and the limit after a slash.
fn load(column: &KanbanColumnSpec) -> String {
    column.limit.map_or_else(
        || column.cards.len().to_string(),
        |limit| format!("{}/{limit}", column.cards.len()),
    )
}

/// The header of a column in the lane at `x`, `y`.
fn push_header(
    elements: &mut Vec<Element>,
    (index, column): (usize, &KanbanColumnSpec),
    (x, y, width): (f64, f64, f64),
    (warnings, metrics): (&mut Vec<ChartWarning>, &impl TextMetrics),
) {
    let over = column.exceeds_limit();
    let load = load(column);
    let load_width = metrics.width(&load, SIZE) + if over { 22.0 } else { 0.0 };
    elements.push(text(
        (x + PAD, y + 21.0),
        TextAnchor::Start,
        "chartlet-kan-head",
        fit_text(
            &column.label,
            width - 3.0 * PAD - load_width,
            SIZE,
            metrics,
            warnings,
            &format!("/kanban/columns/{index}/label"),
        ),
    ));
    elements.push(text(
        (x + width - PAD, y + 21.0),
        TextAnchor::End,
        if over {
            "chartlet-kan-load chartlet-kan-over-text"
        } else {
            "chartlet-kan-load"
        },
        load,
    ));
    if over {
        let sign_x = x + width - PAD - load_width + 7.0;
        elements.push(circle(
            (sign_x, y + 17.0, 7.0),
            "chartlet-kan-sign",
            Some(format!("{}: over the limit", column.label)),
        ));
        elements.push(text(
            (sign_x, y + 21.0),
            TextAnchor::Middle,
            "chartlet-kan-sign-text",
            "!",
        ));
    }
}

/// A card at `x`, `y`.
fn push_card(
    elements: &mut Vec<Element>,
    card: &Card,
    (x, y, width): (f64, f64, f64),
    (metrics, warnings, word): (&impl TextMetrics, &mut Vec<ChartWarning>, &str),
) {
    let spec = card.spec;
    elements.push(rect(
        (x, y, width, card.height),
        if spec.blocked {
            "chartlet-kan-card chartlet-kan-blocked"
        } else {
            "chartlet-kan-card"
        },
        Some(spec.title.clone()),
    ));
    for (position, content) in card.title.iter().enumerate() {
        elements.push(text(
            (x + PAD, y + PAD + 11.0 + LINE * count(position)),
            TextAnchor::Start,
            "chartlet-kan-title",
            content.clone(),
        ));
    }
    let mut row = y + PAD + LINE * count(card.title.len());
    if spec.owner.is_some() || spec.tag.is_some() {
        let tag_width = spec.tag.as_ref().map_or(0.0, |tag| {
            (metrics.width(tag, SMALL) + 12.0).min(width * 0.5)
        });
        if let Some(tag) = &spec.tag {
            let left = x + width - PAD - tag_width;
            elements.push(rect(
                (left, row + 2.0, tag_width, 17.0),
                "chartlet-kan-tag",
                None,
            ));
            elements.push(text(
                (left + tag_width / 2.0, row + 14.0),
                TextAnchor::Middle,
                "chartlet-kan-tag-text",
                fit_text(tag, tag_width - 8.0, SMALL, metrics, warnings, "/kanban"),
            ));
        }
        if let Some(owner) = &spec.owner {
            let room = width - 2.0 * PAD - tag_width - if tag_width > 0.0 { 6.0 } else { 0.0 };
            elements.push(text(
                (x + PAD, row + 14.0),
                TextAnchor::Start,
                "chartlet-kan-owner",
                fit_text(owner, room, SMALL, metrics, warnings, "/kanban"),
            ));
        }
        row += 22.0;
    }
    if spec.blocked {
        elements.push(circle(
            (x + PAD + 6.0, row + 10.0, 6.0),
            "chartlet-kan-sign",
            None,
        ));
        elements.push(line(
            (x + PAD + 3.0, row + 10.0),
            (x + PAD + 9.0, row + 10.0),
            "chartlet-kan-sign-bar",
        ));
        elements.push(text(
            (x + PAD + 18.0, row + 14.0),
            TextAnchor::Start,
            "chartlet-kan-blocked-text",
            word.to_owned(),
        ));
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let kanban = kanban(spec);
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
    let columns = kanban.columns.len();
    let inner = width - 2.0 * margin;
    // Side by side where each column has room, one under the other where not.
    let side_by_side = (inner - GAP * (count(columns) - 1.0)) / count(columns) >= MIN_COLUMN;
    let lane_width = if side_by_side {
        ((inner - GAP * (count(columns) - 1.0)) / count(columns))
            .floor()
            .min(MAX_COLUMN)
    } else {
        inner
    };
    let cards: Vec<Vec<Card>> = kanban
        .columns
        .iter()
        .enumerate()
        .map(|(column, spec)| {
            spec.cards
                .iter()
                .enumerate()
                .map(|(index, card_spec)| {
                    card(
                        (column, index, card_spec),
                        lane_width - 2.0 * GAP,
                        (warnings, metrics),
                    )
                })
                .collect()
        })
        .collect();
    let lane_height = |column: usize| {
        HEADER
            + cards[column]
                .iter()
                .map(|card| card.height + GAP)
                .sum::<f64>()
            + if cards[column].is_empty() { GAP } else { 0.0 }
    };
    let tallest = (0..columns).map(lane_height).fold(0.0, f64::max);
    let used_width = if side_by_side {
        lane_width * count(columns) + GAP * (count(columns) - 1.0)
    } else {
        lane_width
    };
    let left = margin + ((inner - used_width) / 2.0).floor().max(0.0);
    let mut bottom = top;
    let mut stacked_top = top;
    for (index, column) in kanban.columns.iter().enumerate() {
        let (x, y, lane) = if side_by_side {
            (left + (lane_width + GAP) * count(index), top, tallest)
        } else {
            let lane = lane_height(index);
            let y = stacked_top;
            stacked_top += lane + 12.0;
            (left, y, lane)
        };
        elements.push(rect(
            (x, y, lane_width, lane),
            if column.exceeds_limit() {
                "chartlet-kan-lane chartlet-kan-over"
            } else {
                "chartlet-kan-lane"
            },
            Some(column.label.clone()),
        ));
        push_header(
            &mut elements,
            (index, column),
            (x, y, lane_width),
            (warnings, metrics),
        );
        let mut card_y = y + HEADER;
        for card in &cards[index] {
            push_card(
                &mut elements,
                card,
                (x + GAP, card_y, lane_width - 2.0 * GAP),
                (metrics, warnings, text::kanban_blocked(spec.locale)),
            );
            card_y += card.height + GAP;
        }
        bottom = bottom.max(y + lane);
    }
    let needed = bottom + 16.0;
    warn_growth_advising(
        spec,
        (width, needed),
        "shorten the titles or move cards to another board",
        warnings,
    );
    Scene {
        width: spec.width,
        height: pixels(needed.max(height)),
        elements,
    }
}

/// The board in words: the cards of every column, whether it is over its limit, and the blocked
/// cards by name.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let kanban = kanban(spec);
    let cards: usize = kanban.columns.iter().map(|column| column.cards.len()).sum();
    let mut summary = text::kanban_opening(spec.locale, (kanban.columns.len(), cards));
    for column in &kanban.columns {
        summary.push(' ');
        summary.push_str(&text::kanban_column(
            spec.locale,
            &column.label,
            (column.cards.len(), column.limit),
        ));
    }
    let blocked: Vec<(&str, &str)> = kanban
        .columns
        .iter()
        .flat_map(|column| {
            column
                .cards
                .iter()
                .filter(|card| card.blocked)
                .map(|card| (card.title.as_str(), column.label.as_str()))
        })
        .collect();
    if !blocked.is_empty() {
        summary.push(' ');
        summary.push_str(&text::kanban_blocked_cards(spec.locale, &blocked));
    }
    summary
}

/// One row per card with its column, the load of the column, its owner and tag, and whether it
/// is blocked; a column without cards has one row of its own.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let kanban = kanban(spec);
    let mut rows = Vec::new();
    for column in &kanban.columns {
        let load = load(column);
        if column.cards.is_empty() {
            rows.push(vec![
                column.label.clone(),
                load.clone(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ]);
        }
        for card in &column.cards {
            rows.push(vec![
                column.label.clone(),
                load.clone(),
                card.title.clone(),
                card.owner.clone().unwrap_or_default(),
                card.tag.clone().unwrap_or_default(),
                if card.blocked {
                    text::kanban_blocked(spec.locale).to_owned()
                } else {
                    String::new()
                },
            ]);
        }
    }
    DataTable {
        caption: format!("{} {}", spec.locale.words().data_for, spec.title),
        columns: text::kanban_columns(spec.locale)
            .iter()
            .map(|column| (*column).to_owned())
            .collect(),
        rows,
    }
}
