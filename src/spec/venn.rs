use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// A Venn diagram of two or three sets, with a label and a count for any of its regions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VennSpec {
    /// The names of the sets; two or three, unique.
    pub sets: Vec<String>,
    /// What to say about a region, which is named by the sets it lies in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<VennRegionSpec>,
}

/// A region of the diagram: the part that lies in exactly the sets it names.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VennRegionSpec {
    pub sets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<f64>,
}

impl VennSpec {
    /// The regions in drawing order, as bit masks over the sets: the part of each set alone, then
    /// the overlaps of two, then of three.
    pub(crate) fn masks(&self) -> Vec<u8> {
        if self.sets.len() == 2 {
            vec![1, 2, 3]
        } else {
            vec![1, 2, 4, 3, 5, 6, 7]
        }
    }

    /// The mask of a region by the sets it names; sets that do not exist were refused.
    pub(crate) fn mask_of(&self, names: &[String]) -> u8 {
        names
            .iter()
            .filter_map(|name| self.sets.iter().position(|set| set == name))
            .fold(0, |mask, index| mask | (1 << index))
    }

    /// The specification of the region with `mask`, if the specification has one.
    pub(crate) fn region(&self, mask: u8) -> Option<&VennRegionSpec> {
        self.regions
            .iter()
            .find(|region| self.mask_of(&region.sets) == mask)
    }

    /// The names of the sets in `mask`.
    pub(crate) fn names(&self, mask: u8) -> Vec<&str> {
        self.sets
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .map(|(_, set)| set.as_str())
            .collect()
    }
}

impl ChartSpec {
    /// Two or three sets with unique names, and regions that name sets that exist, once each.
    pub(super) fn validate_venn(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_chart_fields("a Venn diagram is drawn from its sets and regions")?;
        let Some(venn) = &self.venn else {
            return Err(ChartError::new(
                "missing_venn",
                "/venn",
                "a Venn diagram requires a venn block",
            ));
        };
        if venn.sets.len() < 2 {
            return Err(ChartError::new(
                "not_enough_sets",
                "/venn/sets",
                "a Venn diagram needs two or three sets",
            ));
        }
        if venn.sets.len() > 3 {
            return Err(ChartError::new(
                "too_many_sets",
                "/venn/sets",
                "circles overlap in every region for two or three sets only; a diagram of more sets is no Venn diagram, so use a heatmap of the sets against each other, or three sets at a time",
            ));
        }
        let mut names = BTreeSet::new();
        for (index, set) in venn.sets.iter().enumerate() {
            validate_text(set, &format!("/venn/sets/{index}"), 60)?;
            if !names.insert(set.as_str()) {
                return Err(ChartError::new(
                    "duplicate_label",
                    format!("/venn/sets/{index}"),
                    "set names must be unique",
                ));
            }
        }
        let mut masks = BTreeSet::new();
        for (index, region) in venn.regions.iter().enumerate() {
            let path = format!("/venn/regions/{index}");
            if region.sets.is_empty() {
                return Err(ChartError::new(
                    "empty_data",
                    format!("{path}/sets"),
                    "name the sets the region lies in",
                ));
            }
            for (position, name) in region.sets.iter().enumerate() {
                if !names.contains(name.as_str()) {
                    return Err(ChartError::new(
                        "unknown_set",
                        format!("{path}/sets/{position}"),
                        "a region names a set from /venn/sets",
                    ));
                }
                if region.sets[..position].contains(name) {
                    return Err(ChartError::new(
                        "duplicate_label",
                        format!("{path}/sets/{position}"),
                        "name every set of a region once",
                    ));
                }
            }
            if !masks.insert(venn.mask_of(&region.sets)) {
                return Err(ChartError::new(
                    "duplicate_region",
                    format!("{path}/sets"),
                    "a region may be given once",
                ));
            }
            if let Some(label) = &region.label {
                validate_text(label, &format!("{path}/label"), 100)?;
            }
            if let Some(count) = region.count {
                validate_number(count, &format!("{path}/count"))?;
                if count < 0.0 {
                    return Err(ChartError::new(
                        "invalid_value",
                        format!("{path}/count"),
                        "a count must be zero or more",
                    ));
                }
            }
        }
        Ok(Vec::new())
    }
}
