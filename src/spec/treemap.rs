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
    /// The value of a leaf, above zero. An item with `children` has none: it is as big as they
    /// are together.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub value: f64,
    /// The parts of this item, drawn as rectangles inside its own. Up to three levels deep.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TreemapItemSpec>,
    /// Items of one group share a palette color and an entry in the legend: on every item or none,
    /// at most four groups.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

// serde hands this function a reference, so the signature follows serde's shape.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(value: &f64) -> bool {
    *value == 0.0
}

/// How deep items nest at most.
const MAX_DEPTH: usize = 3;

/// An item of the treemap, flattened: with its size, level and palette color.
#[derive(Debug, Clone)]
pub(crate) struct TmNode {
    pub label: String,
    /// The sum of the leaves below an item with children.
    pub value: f64,
    /// The palette color from 1, or 0 for the accent when there are no groups.
    pub color: usize,
    pub children: Vec<usize>,
    /// The labels from the top down to this item, joined by ` › `.
    pub path: String,
    /// The group (flat) or the top-level item (nested) this item belongs to.
    pub group: Option<String>,
}

/// The items of a treemap in pre-order, and the indexes of those at the top.
pub(crate) struct TmTree {
    pub nodes: Vec<TmNode>,
    pub roots: Vec<usize>,
}

impl TreemapSpec {
    /// Whether any item has parts.
    pub(crate) fn nested(&self) -> bool {
        self.items.iter().any(|item| !item.children.is_empty())
    }

    /// The items flattened. Nested items take the color of the item at the top they belong to;
    /// flat items the color of their group.
    pub(crate) fn tree(&self) -> TmTree {
        let nested = self.nested();
        let groups = self.groups();
        let mut tree = TmTree {
            nodes: Vec::new(),
            roots: Vec::new(),
        };
        for (top, item) in self.items.iter().enumerate() {
            let color = if nested {
                top % 4 + 1
            } else {
                item.group
                    .as_deref()
                    .and_then(|group| groups.iter().position(|known| *known == group))
                    .map_or(0, |position| position + 1)
            };
            let group = if nested {
                Some(item.label.clone())
            } else {
                item.group.clone()
            };
            let root = flatten(&mut tree, item, color, (String::new(), &group));
            tree.roots.push(root);
        }
        tree
    }

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
        self.items.iter().map(item_value).sum()
    }
}

/// An item's size: its own value, or the sum of its parts.
fn item_value(item: &TreemapItemSpec) -> f64 {
    if item.children.is_empty() {
        item.value
    } else {
        item.children.iter().map(item_value).sum()
    }
}

fn flatten(
    tree: &mut TmTree,
    item: &TreemapItemSpec,
    color: usize,
    (above, group): (String, &Option<String>),
) -> usize {
    let path = if above.is_empty() {
        item.label.clone()
    } else {
        format!("{above} › {}", item.label)
    };
    let index = tree.nodes.len();
    tree.nodes.push(TmNode {
        label: item.label.clone(),
        value: item_value(item),
        color,
        children: Vec::new(),
        path: path.clone(),
        group: group.clone(),
    });
    for child in &item.children {
        let at = flatten(tree, child, color, (path.clone(), group));
        tree.nodes[index].children.push(at);
    }
    index
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
        if treemap.tree().nodes.len() > MAX_ITEMS {
            return Err(ChartError::new(
                "too_many_data_points",
                "/treemap/items",
                format!("at most {MAX_ITEMS} items are supported, counting the parts of items"),
            ));
        }
        let mut seen = Seen {
            labels: BTreeSet::new(),
            groups: Vec::new(),
            grouped: treemap.items.iter().any(|item| item.group.is_some()),
            nested: treemap.nested(),
        };
        validate_items(&treemap.items, ("/treemap/items".to_owned(), 0), &mut seen)?;
        Ok(Vec::new())
    }
}

/// What the items checked so far have in common.
struct Seen<'a> {
    labels: BTreeSet<&'a str>,
    groups: Vec<&'a str>,
    grouped: bool,
    nested: bool,
}

/// Unique labels; a leaf with a value above zero, an item with parts without one; groups on
/// every item at the top or on none, and only on a flat treemap.
fn validate_items<'a>(
    items: &'a [TreemapItemSpec],
    (parent, depth): (String, usize),
    seen: &mut Seen<'a>,
) -> Result<(), ChartError> {
    for (index, item) in items.iter().enumerate() {
        let path = format!("{parent}/{index}");
        let path = if depth == 0 {
            format!("/treemap/items/{index}")
        } else {
            path
        };
        validate_text(&item.label, &format!("{path}/label"), 100)?;
        if !seen.labels.insert(item.label.as_str()) {
            return Err(ChartError::new(
                "duplicate_label",
                format!("{path}/label"),
                "item labels must be unique, parts included",
            ));
        }
        if item.children.is_empty() {
            validate_number(item.value, &format!("{path}/value"))?;
            if item.value <= 0.0 {
                return Err(ChartError::new(
                    "invalid_value",
                    format!("{path}/value"),
                    "the value of an item must be above zero",
                ));
            }
        } else {
            if item.value != 0.0 {
                return Err(ChartError::new(
                    "value_with_children",
                    format!("{path}/value"),
                    "an item with children is as big as they are together; remove its value",
                ));
            }
            if depth + 1 >= MAX_DEPTH {
                return Err(ChartError::new(
                    "too_deep",
                    format!("{path}/children"),
                    format!("items nest at most {MAX_DEPTH} levels deep"),
                ));
            }
            validate_items(
                &item.children,
                (format!("{path}/children"), depth + 1),
                seen,
            )?;
        }
        validate_group(item, &path, seen)?;
    }
    Ok(())
}

fn validate_group<'a>(
    item: &'a TreemapItemSpec,
    path: &str,
    seen: &mut Seen<'a>,
) -> Result<(), ChartError> {
    match (&item.group, seen.grouped) {
        (Some(group), _) => {
            validate_text(group, &format!("{path}/group"), 60)?;
            if seen.nested {
                return Err(ChartError::new(
                    "group_with_children",
                    format!("{path}/group"),
                    "a nested treemap is colored by its top-level items; remove the groups",
                ));
            }
            if !seen.groups.contains(&group.as_str()) {
                seen.groups.push(group);
            }
            if seen.groups.len() > MAX_SERIES {
                return Err(ChartError::new(
                    "too_many_series",
                    format!("{path}/group"),
                    format!("at most {MAX_SERIES} groups are supported, one per palette color"),
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
    Ok(())
}
