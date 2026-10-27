use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, sequence::is_identifier, validate_text};
use crate::error::{ChartError, ChartWarning};

pub(crate) const MAX_AREAS: usize = 30;
pub(crate) const MAX_PHASES: usize = 4;
pub(crate) const MAX_ITEMS: usize = 12;
/// The largest coordinate or size on the grid.
pub(crate) const MAX_GRID: f64 = 100.0;

/// A plan: rooms, buildings and open ground on a grid, seen at an angle in 2:1 dimetric
/// projection with the walls cut low, so that every room shows.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanSpec {
    pub areas: Vec<AreaSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AreaSpec {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "AreaKind::is_room")]
    pub kind: AreaKind,
    /// The left edge on the grid, in grid units from 0.
    pub x: f64,
    /// The top edge on the grid.
    pub y: f64,
    /// The width along `x`, more than 0.
    pub w: f64,
    /// The depth along `y`, more than 0.
    pub h: f64,
    /// When the area exists or is built: a name shared by areas, with a pattern of its own in the
    /// legend; at most four.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    /// Furniture, from a small set of symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemSpec>,
}

/// What an area is.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AreaKind {
    /// A room: a floor with low walls.
    #[default]
    Room,
    /// A building: like a room, with a heavier outline.
    Building,
    /// Open ground: a floor without walls and a dashed outline; other areas may lie on it.
    Outdoor,
}

impl AreaKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_room(&self) -> bool {
        matches!(self, Self::Room)
    }
}

/// A piece of furniture in an area, standing at `x`, `y` in grid units from the area's corner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemSpec {
    pub kind: ItemKind,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Desk,
    Shelf,
    Table,
    Bed,
    Sofa,
    Plant,
}

impl ItemKind {
    /// The name in English, for messages of the specification.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Desk => "desk",
            Self::Shelf => "shelf",
            Self::Table => "table",
            Self::Bed => "bed",
            Self::Sofa => "sofa",
            Self::Plant => "plant",
        }
    }

    /// Footprint along `x` and `y` in grid units, and height in pixels.
    pub(crate) const fn size(self) -> (f64, f64, f64) {
        match self {
            Self::Desk => (1.2, 0.6, 7.0),
            Self::Shelf => (1.0, 0.3, 16.0),
            Self::Table => (1.0, 1.0, 8.0),
            Self::Bed => (1.0, 2.0, 6.0),
            Self::Sofa => (1.6, 0.7, 9.0),
            Self::Plant => (0.4, 0.4, 12.0),
        }
    }
}

impl PlanSpec {
    /// The phases in the order they first appear.
    pub(crate) fn phases(&self) -> Vec<&str> {
        let mut phases: Vec<&str> = Vec::new();
        for phase in self.areas.iter().filter_map(|area| area.phase.as_deref()) {
            if !phases.contains(&phase) {
                phases.push(phase);
            }
        }
        phases
    }

    /// The index of an area's phase among [`Self::phases`].
    pub(crate) fn phase_of(&self, area: usize) -> Option<usize> {
        let phase = self.areas[area].phase.as_deref()?;
        self.phases().iter().position(|known| *known == phase)
    }
}

impl ChartSpec {
    #[allow(clippy::too_many_lines)]
    /// Areas with unique ids and texts, sizes above 0 on the grid, no two rooms or buildings
    /// overlapping, at most four phases, and items inside their area.
    pub(super) fn validate_plan(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a plan", "plan.areas")?;
        let Some(plan) = &self.plan else {
            return Err(ChartError::new(
                "missing_plan",
                "/plan",
                "a plan requires a plan block",
            ));
        };
        if plan.areas.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/plan/areas",
                "provide at least one area",
            ));
        }
        if plan.areas.len() > MAX_AREAS {
            return Err(ChartError::new(
                "too_many_areas",
                "/plan/areas",
                format!("at most {MAX_AREAS} areas are supported"),
            ));
        }
        let mut ids = BTreeSet::new();
        let mut phases: Vec<&str> = Vec::new();
        for (index, area) in plan.areas.iter().enumerate() {
            let path = format!("/plan/areas/{index}");
            if !is_identifier(&area.id) {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/id"),
                    "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            if !ids.insert(area.id.as_str()) {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    format!("the id \"{}\" is already taken by an earlier area", area.id),
                ));
            }
            validate_text(&area.label, &format!("{path}/label"), 60)?;
            for (field, value, least) in [
                ("x", area.x, 0.0),
                ("y", area.y, 0.0),
                ("w", area.w, f64::MIN_POSITIVE),
                ("h", area.h, f64::MIN_POSITIVE),
            ] {
                if !value.is_finite() || value < least || value > MAX_GRID {
                    return Err(ChartError::new(
                        "invalid_area",
                        format!("{path}/{field}"),
                        format!(
                            "{field} is a number of grid units from {} to {MAX_GRID}; w and h are more than 0",
                            if least == 0.0 { "0" } else { "above 0" }
                        ),
                    ));
                }
            }
            if let Some(phase) = &area.phase {
                validate_text(phase, &format!("{path}/phase"), 40)?;
                if !phases.contains(&phase.as_str()) {
                    phases.push(phase);
                }
                if phases.len() > MAX_PHASES {
                    return Err(ChartError::new(
                        "too_many_phases",
                        format!("{path}/phase"),
                        format!(
                            "at most {MAX_PHASES} phases are supported: each has a pattern of its own"
                        ),
                    ));
                }
            }
            if area.items.len() > MAX_ITEMS {
                return Err(ChartError::new(
                    "too_many_items",
                    format!("{path}/items"),
                    format!("at most {MAX_ITEMS} items fit an area"),
                ));
            }
            for (item_index, item) in area.items.iter().enumerate() {
                let (width, depth, _) = item.kind.size();
                let outside = !item.x.is_finite()
                    || !item.y.is_finite()
                    || item.x < 0.0
                    || item.y < 0.0
                    || item.x + width > area.w + 1e-9
                    || item.y + depth > area.h + 1e-9;
                if outside {
                    return Err(ChartError::new(
                        "item_outside_area",
                        format!("{path}/items/{item_index}"),
                        format!(
                            "a {} covers {width} × {depth} grid units from its x and y and has to lie inside the area ({} × {})",
                            item.kind.name(),
                            area.w,
                            area.h
                        ),
                    ));
                }
            }
        }
        for (index, area) in plan.areas.iter().enumerate() {
            if area.kind == AreaKind::Outdoor {
                continue;
            }
            for earlier in plan.areas[..index]
                .iter()
                .filter(|other| other.kind != AreaKind::Outdoor)
            {
                let apart = area.x >= earlier.x + earlier.w - 1e-9
                    || earlier.x >= area.x + area.w - 1e-9
                    || area.y >= earlier.y + earlier.h - 1e-9
                    || earlier.y >= area.y + area.h - 1e-9;
                if !apart {
                    return Err(ChartError::new(
                        "area_overlap",
                        format!("/plan/areas/{index}"),
                        format!(
                            "\"{}\" overlaps \"{}\"; rooms and buildings do not share floor (open ground may lie under them)",
                            area.id, earlier.id
                        ),
                    ));
                }
            }
        }
        Ok(Vec::new())
    }
}
