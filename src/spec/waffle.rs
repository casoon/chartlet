use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_SERIES, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many squares a waffle may have, and how many columns.
pub(crate) const MIN_CELLS: u32 = 10;
pub(crate) const MAX_CELLS: u32 = 400;
const MAX_COLUMNS: u32 = 40;

/// A waffle: a grid of squares, each of which stands for the same share of a whole. The parts
/// fill the squares one after the other, reading order, so that shares can be counted.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WaffleSpec {
    pub parts: Vec<WafflePartSpec>,
    /// How many squares the whole has; 100 by default, so that a square is one percent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cells: Option<u32>,
    /// How many squares a row has; by default the grid is about square.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u32>,
    /// The size of the whole, when the parts do not make it up: the squares that are left stand
    /// for the rest. At least the sum of the parts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WafflePartSpec {
    pub label: String,
    pub value: f64,
}

impl WaffleSpec {
    pub(crate) fn cells(&self) -> u32 {
        self.cells.unwrap_or(100)
    }

    /// The squares per row.
    pub(crate) fn columns(&self) -> u32 {
        self.columns.unwrap_or_else(|| {
            let cells = self.cells();
            let mut columns = 1;
            while columns * columns < cells {
                columns += 1;
            }
            columns
        })
    }

    /// The whole the squares stand for.
    pub(crate) fn whole(&self) -> f64 {
        let sum: f64 = self.parts.iter().map(|part| part.value).sum();
        self.total.map_or(sum, |total| total.max(sum))
    }

    /// How many squares each part takes, and the rest as a last entry when the parts do not make
    /// up the whole: by the largest remainder, and no part with a value is left without a square.
    pub(crate) fn squares(&self) -> Vec<u32> {
        let cells = self.cells();
        let whole = self.whole();
        let mut shares: Vec<f64> = self
            .parts
            .iter()
            .map(|part| part.value / whole * f64::from(cells))
            .collect();
        let sum: f64 = self.parts.iter().map(|part| part.value).sum();
        let rest = (whole - sum) / whole * f64::from(cells);
        if rest > 1e-9 {
            shares.push(rest);
        }
        let mut squares: Vec<u32> = shares
            .iter()
            .map(|share| whole_part(*share).max(1))
            .collect();
        // Hand out what is left, or take back what was added, one square at a time.
        loop {
            let used: u32 = squares.iter().sum();
            if used == cells {
                break;
            }
            let remainder = |index: usize| shares[index] - f64::from(squares[index]);
            if used < cells {
                let next = (0..squares.len())
                    .max_by(|a, b| remainder(*a).total_cmp(&remainder(*b)))
                    .expect("a waffle has a part");
                squares[next] += 1;
            } else {
                let next = (0..squares.len())
                    .filter(|index| squares[*index] > 1)
                    .min_by(|a, b| remainder(*a).total_cmp(&remainder(*b)))
                    .expect("a part can give a square back");
                squares[next] -= 1;
            }
        }
        squares
    }
}

/// The whole squares of a share: its floor.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn whole_part(share: f64) -> u32 {
    share.floor() as u32
}

impl ChartSpec {
    /// Parts of the whole with unique labels and values above zero, and a grid that has room for
    /// a square for each.
    pub(super) fn validate_waffle(&self) -> Result<Vec<ChartWarning>, ChartError> {
        for (field, present) in [
            ("/data", !self.data.is_empty()),
            ("/categories", !self.categories.is_empty()),
            ("/series", !self.series.is_empty()),
            ("/zoomSteps", !self.zoom_steps.is_empty()),
            ("/panes", !self.panes.is_empty()),
            ("/references", !self.references.is_empty()),
            ("/timeAxis", !self.time_axis.is_default()),
            (
                "/valueAxis",
                self.value_axis != super::ValueAxisSpec::default(),
            ),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    field,
                    "a waffle is drawn from parts; remove this field",
                ));
            }
        }
        let Some(waffle) = &self.waffle else {
            return Err(ChartError::new(
                "missing_waffle",
                "/waffle",
                "a waffle requires a waffle block",
            ));
        };
        if waffle.parts.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/waffle/parts",
                "provide at least one part",
            ));
        }
        if waffle.parts.len() > MAX_SERIES {
            return Err(ChartError::new(
                "too_many_series",
                "/waffle/parts",
                format!("at most {MAX_SERIES} parts are supported, one per palette color"),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, part) in waffle.parts.iter().enumerate() {
            let path = format!("/waffle/parts/{index}");
            validate_text(&part.label, &format!("{path}/label"), 100)?;
            if !labels.insert(part.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "part labels must be unique",
                ));
            }
            validate_number(part.value, &format!("{path}/value"))?;
            if part.value <= 0.0 {
                return Err(ChartError::new(
                    "invalid_value",
                    format!("{path}/value"),
                    "the value of a part must be above zero",
                ));
            }
        }
        let cells = waffle.cells();
        if !(MIN_CELLS..=MAX_CELLS).contains(&cells) {
            return Err(ChartError::new(
                "invalid_cells",
                "/waffle/cells",
                format!("cells must be between {MIN_CELLS} and {MAX_CELLS}"),
            ));
        }
        if usize::try_from(cells).expect("small") < waffle.parts.len() + 1 {
            return Err(ChartError::new(
                "invalid_cells",
                "/waffle/cells",
                "every part needs a square",
            ));
        }
        if !(2..=MAX_COLUMNS).contains(&waffle.columns()) {
            return Err(ChartError::new(
                "invalid_columns",
                "/waffle/columns",
                format!("columns must be between 2 and {MAX_COLUMNS}"),
            ));
        }
        if let Some(total) = waffle.total {
            validate_number(total, "/waffle/total")?;
            let sum: f64 = waffle.parts.iter().map(|part| part.value).sum();
            if total < sum {
                return Err(ChartError::new(
                    "invalid_total",
                    "/waffle/total",
                    "the total must be at least the sum of the parts",
                ));
            }
        }
        Ok(Vec::new())
    }
}
