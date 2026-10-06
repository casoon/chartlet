use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many parties and seats a parliament chart holds.
pub(crate) const MAX_PARTIES: usize = 8;
pub(crate) const MAX_SEATS: u32 = 800;

/// A parliament: the seats of an assembly as dots in a semicircle, in blocks by party from left to
/// right in the order of the list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParliamentSpec {
    pub parties: Vec<PartySpec>,
    /// Marks the seat that makes a majority with a dashed line and says how many seats it takes.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub majority: bool,
    /// The labels of parties that govern together: their seats are ringed, and the legend adds
    /// their sum and whether it is a majority.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coalition: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PartySpec {
    pub label: String,
    pub seats: u32,
}

impl ParliamentSpec {
    pub(crate) fn total(&self) -> u32 {
        self.parties.iter().map(|party| party.seats).sum()
    }

    /// The seats a majority takes: more than half.
    pub(crate) fn majority_of(&self) -> u32 {
        self.total() / 2 + 1
    }

    pub(crate) fn in_coalition(&self, party: usize) -> bool {
        self.coalition.contains(&self.parties[party].label)
    }

    /// The seats of the parties of the coalition.
    pub(crate) fn coalition_seats(&self) -> u32 {
        (0..self.parties.len())
            .filter(|party| self.in_coalition(*party))
            .map(|party| self.parties[party].seats)
            .sum()
    }
}

impl ChartSpec {
    /// Parties with unique labels and seats, up to eight and up to 800 seats, and a coalition of
    /// parties that exist.
    pub(super) fn validate_parliament(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
                    "a parliament is drawn from parties; remove this field",
                ));
            }
        }
        let Some(parliament) = &self.parliament else {
            return Err(ChartError::new(
                "missing_parliament",
                "/parliament",
                "a parliament requires a parliament block",
            ));
        };
        if parliament.parties.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/parliament/parties",
                "provide at least one party",
            ));
        }
        if parliament.parties.len() > MAX_PARTIES {
            return Err(ChartError::new(
                "too_many_series",
                "/parliament/parties",
                format!("at most {MAX_PARTIES} parties are supported"),
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, party) in parliament.parties.iter().enumerate() {
            let path = format!("/parliament/parties/{index}");
            validate_text(&party.label, &format!("{path}/label"), 100)?;
            if !labels.insert(party.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "party labels must be unique",
                ));
            }
            if party.seats == 0 {
                return Err(ChartError::new(
                    "invalid_value",
                    format!("{path}/seats"),
                    "a party has at least one seat",
                ));
            }
        }
        if parliament.total() > MAX_SEATS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/parliament/parties",
                format!("at most {MAX_SEATS} seats are supported"),
            ));
        }
        for (index, label) in parliament.coalition.iter().enumerate() {
            if !labels.contains(label.as_str()) {
                return Err(ChartError::new(
                    "unknown_node",
                    format!("/parliament/coalition/{index}"),
                    format!("no party is labeled \"{label}\""),
                ));
            }
        }
        Ok(Vec::new())
    }
}
