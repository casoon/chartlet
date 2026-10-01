use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    ChartSpec, default_true, one, topicmap::TopicLinkSpec, validate_number, validate_optional_text,
    validate_text,
};
use crate::error::{ChartError, ChartWarning};

/// One place on an `atlas` map: a single entry, drawn as a point inside its region.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaceSpec {
    pub label: String,
    /// How prominent the point is. 1 is an ordinary place; larger stands out, which is how a
    /// map shows what matters without needing a second color.
    #[serde(default = "one")]
    pub weight: f64,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// One region of an `atlas` map: a named area inside a realm, holding places.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegionSpec {
    pub label: String,
    /// How much the region holds. It drives the region's share of the land, damped by
    /// `areaDamping` so that the largest one does not swallow the map.
    pub value: f64,
    #[serde(default)]
    pub places: Vec<PlaceSpec>,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// One realm of an `atlas` map: the regions that belong together and are drawn as one land.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RealmSpec {
    pub label: String,
    pub regions: Vec<RegionSpec>,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// The landscape of an `atlas` chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AtlasSpec {
    pub realms: Vec<RealmSpec>,
    /// Kinship between two regions, named by label and valid across realm borders. These are the
    /// edges that pull a region towards the edge of its own realm, so that something which
    /// mediates between two subjects ends up lying between them.
    #[serde(default)]
    pub links: Vec<TopicLinkSpec>,
    #[serde(default)]
    pub seed: u64,
    /// The exponent a region's area follows: 1 is proportional to its value, 0.5 its square root.
    #[serde(default = "default_area_damping")]
    pub area_damping: f64,
    /// Contour lines drawn from the density of places, so that the terrain carries the quantity
    /// statement the area no longer has to.
    #[serde(default = "default_true")]
    pub contours: bool,
}

const fn default_area_damping() -> f64 {
    0.5
}

/// Bounds on an `atlas`, set by what stays legible rather than by anything technical.
const MAX_REALMS: usize = 8;
const MAX_REGIONS: usize = 60;
const MAX_PLACES: usize = 4000;

impl ChartSpec {
    /// An atlas is valid when every realm holds regions, every label is unique across both
    /// levels, and every link names a region that exists. Labels share one namespace because a
    /// reader does not distinguish them either: on the finished map both are just names of
    /// places, and two of them reading alike would be a defect, not a subtlety.
    pub(super) fn validate_atlas(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("an atlas chart", "atlas.realms")?;
        let Some(atlas) = &self.atlas else {
            return Err(ChartError::new(
                "missing_atlas",
                "/atlas",
                "an atlas chart requires an atlas block",
            ));
        };
        if atlas.realms.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/atlas/realms",
                "provide at least one realm",
            ));
        }
        if atlas.realms.len() > MAX_REALMS {
            return Err(ChartError::new(
                "too_many_realms",
                "/atlas/realms",
                format!("at most {MAX_REALMS} realms are supported"),
            ));
        }
        if !(0.2..=1.0).contains(&atlas.area_damping) {
            return Err(ChartError::new(
                "invalid_area_damping",
                "/atlas/areaDamping",
                "areaDamping must be between 0.2 and 1",
            ));
        }

        let mut labels = BTreeSet::new();
        let mut regions = BTreeSet::new();
        let mut warnings = Vec::new();
        let mut places = 0usize;

        for (index, realm) in atlas.realms.iter().enumerate() {
            places += validate_realm(
                realm,
                &format!("/atlas/realms/{index}"),
                &mut labels,
                &mut regions,
                &mut warnings,
            )?;
        }

        if regions.len() > MAX_REGIONS {
            return Err(ChartError::new(
                "too_many_regions",
                "/atlas/realms",
                format!("at most {MAX_REGIONS} regions are supported across all realms"),
            ));
        }
        if places > MAX_PLACES {
            return Err(ChartError::new(
                "too_many_places",
                "/atlas/realms",
                format!("at most {MAX_PLACES} places are supported across all regions"),
            ));
        }

        for (index, link) in atlas.links.iter().enumerate() {
            let path = format!("/atlas/links/{index}");
            for (side, label) in [("from", &link.from), ("to", &link.to)] {
                if !regions.contains(label.as_str()) {
                    return Err(ChartError::new(
                        "unknown_region_link",
                        format!("{path}/{side}"),
                        format!("{label:?} is not a declared region label"),
                    ));
                }
            }
            if !(0.0..=1.0).contains(&link.weight) {
                return Err(ChartError::new(
                    "invalid_link_weight",
                    format!("{path}/weight"),
                    "weight must be between 0 and 1",
                ));
            }
        }

        Ok(warnings)
    }
}

/// One realm and everything under it. Returns how many places it holds, so the caller can keep
/// the running total without walking the regions a second time.
fn validate_realm(
    realm: &RealmSpec,
    path: &str,
    labels: &mut BTreeSet<String>,
    regions: &mut BTreeSet<String>,
    warnings: &mut Vec<ChartWarning>,
) -> Result<usize, ChartError> {
    validate_text(&realm.label, &format!("{path}/label"), 200)?;
    validate_optional_text(realm.tooltip.as_ref(), &format!("{path}/tooltip"), 300)?;
    if !labels.insert(realm.label.clone()) {
        return Err(ChartError::new(
            "duplicate_atlas_label",
            format!("{path}/label"),
            "realm and region labels must be unique",
        ));
    }
    if realm.regions.is_empty() {
        return Err(ChartError::new(
            "empty_realm",
            format!("{path}/regions"),
            "a realm needs at least one region",
        ));
    }
    if realm.regions.len() == 1 {
        warnings.push(ChartWarning::new(
            "realm_without_structure",
            format!("{path}/regions"),
            "a realm with a single region has no inner structure to show; it is drawn as one area",
        ));
    }

    let mut places = 0usize;
    for (index, region) in realm.regions.iter().enumerate() {
        let region_path = format!("{path}/regions/{index}");
        validate_text(&region.label, &format!("{region_path}/label"), 200)?;
        validate_number(region.value, &format!("{region_path}/value"))?;
        if region.value < 1.0 {
            return Err(ChartError::new(
                "region_value_out_of_range",
                format!("{region_path}/value"),
                "value must be at least 1",
            ));
        }
        validate_optional_text(
            region.tooltip.as_ref(),
            &format!("{region_path}/tooltip"),
            300,
        )?;
        if !labels.insert(region.label.clone()) {
            return Err(ChartError::new(
                "duplicate_atlas_label",
                format!("{region_path}/label"),
                "realm and region labels must be unique",
            ));
        }
        regions.insert(region.label.clone());
        places += region.places.len();
        // More places than the region claims to hold means the two numbers come from different
        // counts; the map would then draw one and label the other.
        if places_exceed_value(region.places.len(), region.value) {
            warnings.push(ChartWarning::new(
                "more_places_than_value",
                format!("{region_path}/places"),
                "the region lists more places than its value; the label will not match what is drawn",
            ));
        }
        for (place_index, place) in region.places.iter().enumerate() {
            validate_place(place, &format!("{region_path}/places/{place_index}"))?;
        }
    }
    Ok(places)
}

/// A count as a measured value, for the data table.
#[allow(clippy::cast_precision_loss)]
pub(super) fn place_count(places: usize) -> f64 {
    places as f64
}

/// Whether a region lists more places than its value claims. Written out because the comparison
/// crosses from a count to a measured value, and the cast is the only place that can go wrong.
#[allow(clippy::cast_precision_loss)]
fn places_exceed_value(places: usize, value: f64) -> bool {
    places as f64 > value
}

/// A place carries a name, a prominence and, if it wants one, a line of detail. Labels are not
/// required to be unique: two entries may well share a title.
fn validate_place(place: &PlaceSpec, path: &str) -> Result<(), ChartError> {
    validate_text(&place.label, &format!("{path}/label"), 200)?;
    validate_number(place.weight, &format!("{path}/weight"))?;
    if !(0.25..=4.0).contains(&place.weight) {
        return Err(ChartError::new(
            "place_weight_out_of_range",
            format!("{path}/weight"),
            "weight must be between 0.25 and 4",
        ));
    }
    validate_optional_text(place.tooltip.as_ref(), &format!("{path}/tooltip"), 300)?;
    Ok(())
}
