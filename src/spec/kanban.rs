use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many columns a board has, cards a column holds, and how long the texts of a card may be.
const MAX_COLUMNS: usize = 6;
const MAX_CARDS: usize = 20;

/// A kanban board: columns, each a state, with cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KanbanSpec {
    pub columns: Vec<KanbanColumnSpec>,
}

/// A column of the board with an optional limit on the work in progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KanbanColumnSpec {
    pub label: String,
    /// The most cards the column should hold; shown as `3/4`, and marked when exceeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cards: Vec<KanbanCardSpec>,
}

/// A card: its title and, if known, who owns it and what kind of work it is.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KanbanCardSpec {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Whether the card cannot move on; drawn with a sign and a word, not by color alone.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub blocked: bool,
}

impl KanbanColumnSpec {
    /// Whether the column holds more cards than its limit allows.
    pub(crate) fn exceeds_limit(&self) -> bool {
        self.limit
            .is_some_and(|limit| self.cards.len() > usize::try_from(limit).unwrap_or(usize::MAX))
    }
}

impl ChartSpec {
    /// One to six columns with unique labels, at most 20 cards each, and limits of at least one.
    pub(super) fn validate_kanban(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_chart_fields("a kanban board is drawn from its columns and cards")?;
        let Some(kanban) = &self.kanban else {
            return Err(ChartError::new(
                "missing_kanban",
                "/kanban",
                "a kanban board requires a kanban block",
            ));
        };
        if kanban.columns.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/kanban/columns",
                "provide at least one column",
            ));
        }
        if kanban.columns.len() > MAX_COLUMNS {
            return Err(ChartError::new(
                "too_many_nodes",
                "/kanban/columns",
                format!("at most {MAX_COLUMNS} columns are supported; merge states"),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, column) in kanban.columns.iter().enumerate() {
            let path = format!("/kanban/columns/{index}");
            validate_text(&column.label, &format!("{path}/label"), 60)?;
            if !labels.insert(column.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "column labels must be unique",
                ));
            }
            if column.limit == Some(0) {
                return Err(ChartError::new(
                    "invalid_limit",
                    format!("{path}/limit"),
                    "a limit is at least 1; leave it out for a column without one",
                ));
            }
            if column.cards.len() > MAX_CARDS {
                return Err(ChartError::new(
                    "too_many_nodes",
                    format!("{path}/cards"),
                    format!("at most {MAX_CARDS} cards per column are supported; split the board"),
                ));
            }
            for (card_index, card) in column.cards.iter().enumerate() {
                let card_path = format!("{path}/cards/{card_index}");
                validate_text(&card.title, &format!("{card_path}/title"), 100)?;
                for (name, value, length) in [("owner", &card.owner, 40), ("tag", &card.tag, 30)] {
                    if let Some(value) = value {
                        validate_text(value, &format!("{card_path}/{name}"), length)?;
                    }
                }
            }
        }
        Ok(Vec::new())
    }
}
