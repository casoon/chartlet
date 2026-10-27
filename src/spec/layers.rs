use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, DiagramRole, sequence::is_identifier, validate_text};
use crate::error::{ChartError, ChartWarning};

pub(crate) const MAX_STACK: usize = 10;
pub(crate) const MAX_NESTED: usize = 6;
pub(crate) const MAX_ITEMS: usize = 12;

/// Layers: stacked abstraction levels, or scopes drawn inside each other.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LayersSpec {
    /// The layers: from the bottom up in a stack, from the outermost inwards in nested scopes.
    pub layers: Vec<StackLayerSpec>,
    #[serde(default, skip_serializing_if = "LayersDisplay::is_stack")]
    pub display: LayersDisplay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StackLayerSpec {
    pub id: String,
    pub label: String,
    /// What lies in the layer: named parts, drawn as small boxes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    /// A remark at the side of the layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<DiagramRole>,
}

/// How the layers relate to each other.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LayersDisplay {
    /// Levels of abstraction on top of each other, the first at the bottom.
    #[default]
    Stack,
    /// Scopes inside each other, the first the outermost.
    Nested,
}

impl LayersDisplay {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_stack(&self) -> bool {
        matches!(self, Self::Stack)
    }
}

impl ChartSpec {
    /// Layers with unique ids and texts; an exploded projection only for a stack.
    pub(super) fn validate_layers(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a layer diagram", "layers.layers")?;
        let Some(layers) = &self.layer_stack else {
            return Err(ChartError::new(
                "missing_layers",
                "/layers",
                "a layer diagram requires a layers block",
            ));
        };
        if layers.layers.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/layers/layers",
                "provide at least one layer",
            ));
        }
        let most = match layers.display {
            LayersDisplay::Stack => MAX_STACK,
            LayersDisplay::Nested => MAX_NESTED,
        };
        if layers.layers.len() > most {
            return Err(ChartError::new(
                "too_many_layers",
                "/layers/layers",
                format!("at most {most} layers are supported in this display"),
            ));
        }
        let mut ids = BTreeSet::new();
        for (index, layer) in layers.layers.iter().enumerate() {
            let path = format!("/layers/layers/{index}");
            if !is_identifier(&layer.id) {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/id"),
                    "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            if !ids.insert(layer.id.as_str()) {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    format!(
                        "the id \"{}\" is already taken by an earlier layer",
                        layer.id
                    ),
                ));
            }
            validate_text(&layer.label, &format!("{path}/label"), 60)?;
            if layer.items.len() > MAX_ITEMS {
                return Err(ChartError::new(
                    "too_many_items",
                    format!("{path}/items"),
                    format!("at most {MAX_ITEMS} items fit a layer; group them or split the layer"),
                ));
            }
            for (item, text) in layer.items.iter().enumerate() {
                validate_text(text, &format!("{path}/items/{item}"), 40)?;
            }
            if let Some(note) = &layer.note {
                validate_text(note, &format!("{path}/note"), 120)?;
            }
        }
        Ok(Vec::new())
    }
}
