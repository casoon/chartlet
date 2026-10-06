use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, MAX_SERIES, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// How many items a treemap holds.
pub(crate) const MAX_ITEMS: usize = 100;

/// A treemap: rectangles whose areas follow the values of the items, packed into one rectangle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TreemapSpec {
    pub items: Vec<TreemapItemSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TreemapItemSpec {
    pub label: String,
    pub value: f64,
    /// Items of one group share a palette color and an entry in the legend: on every item or none,
    /// at most four groups.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl TreemapSpec {
    /// The groups in the order they first appear.
    pub(crate) fn groups(&self) -> Vec<&str> {
        let mut groups: Vec<&str> = Vec::new();
        for group in self.items.iter().filter_map(|item| item.group.as_deref()) {
            if !groups.contains(&group) {
                groups.push(group);
            }
        }
        groups
    }

    pub(crate) fn total(&self) -> f64 {
        self.items.iter().map(|item| item.value).sum()
    }
}

impl ChartSpec {
    /// Items with unique labels and values above zero; groups on all items or none.
    pub(super) fn validate_treemap(&self) -> Result<Vec<ChartWarning>, ChartError> {
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
                    "a treemap is drawn from items; remove this field",
                ));
            }
        }
        let Some(treemap) = &self.treemap else {
            return Err(ChartError::new(
                "missing_treemap",
                "/treemap",
                "a treemap requires a treemap block",
            ));
        };
        if treemap.items.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/treemap/items",
                "provide at least one item",
            ));
        }
        if treemap.items.len() > MAX_ITEMS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/treemap/items",
                format!("at most {MAX_ITEMS} items are supported"),
            ));
        }
        let grouped = treemap.items.iter().any(|item| item.group.is_some());
        let mut labels = BTreeSet::new();
        let mut groups: Vec<&str> = Vec::new();
        for (index, item) in treemap.items.iter().enumerate() {
            let path = format!("/treemap/items/{index}");
            validate_text(&item.label, &format!("{path}/label"), 100)?;
            if !labels.insert(item.label.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("{path}/label"),
                    "item labels must be unique",
                ));
            }
            validate_number(item.value, &format!("{path}/value"))?;
            if item.value <= 0.0 {
                return Err(ChartError::new(
                    "invalid_value",
                    format!("{path}/value"),
                    "the value of an item must be above zero",
                ));
            }
            match (&item.group, grouped) {
                (Some(group), _) => {
                    validate_text(group, &format!("{path}/group"), 60)?;
                    if !groups.contains(&group.as_str()) {
                        groups.push(group);
                    }
                    if groups.len() > MAX_SERIES {
                        return Err(ChartError::new(
                            "too_many_series",
                            format!("{path}/group"),
                            format!(
                                "at most {MAX_SERIES} groups are supported, one per palette color"
                            ),
                        ));
                    }
                }
                (None, true) => {
                    return Err(ChartError::new(
                        "missing_group",
                        format!("{path}/group"),
                        "give every item a group, or none",
                    ));
                }
                (None, false) => {}
            }
        }
        Ok(Vec::new())
    }
}
