use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ChartSpec, default_true, one, validate_number, validate_text};
use crate::error::{ChartError, ChartWarning};

/// One topic (or island) of a `topicmap` chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TopicSpec {
    pub label: String,
    pub value: f64,
    /// Number of paths through this topic; drawn as points inside its area.
    #[serde(default)]
    pub points: u32,
    #[serde(default)]
    pub tooltip: Option<String>,
}

/// A neighborhood between two topics, drawn as a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TopicLinkSpec {
    /// Label of the topic (or island) the route starts at.
    pub from: String,
    /// Label of the topic (or island) the route ends at.
    pub to: String,
    #[serde(default = "one")]
    pub weight: f64,
}

/// Which corner of the canvas a cartouche is anchored to.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Corner {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

/// The legend box printed on the map: heading, metadata line, and its corner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CartoucheSpec {
    pub heading: String,
    pub meta: String,
    #[serde(default)]
    pub corner: Corner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TopicMapSpec {
    pub topics: Vec<TopicSpec>,
    #[serde(default)]
    pub links: Vec<TopicLinkSpec>,
    /// Fringe topics, drawn as small fixed-size islands rather than area-proportional landmasses.
    #[serde(default)]
    pub islands: Vec<TopicSpec>,
    #[serde(default)]
    pub seed: u64,
    #[serde(default)]
    pub cartouche: Option<CartoucheSpec>,
    #[serde(default = "default_true")]
    pub graticule: bool,
    #[serde(default = "default_true")]
    pub compass: bool,
    /// Number of depth-line rings drawn around each coastline, 0 to 3.
    #[serde(default = "default_depth_bands")]
    pub depth_bands: u8,
}

impl ChartSpec {
    pub(super) fn validate_topicmap(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a topicmap chart", "topicmap.topics")?;
        let Some(topicmap) = &self.topicmap else {
            return Err(ChartError::new(
                "missing_topicmap",
                "/topicmap",
                "a topicmap chart requires a topicmap block",
            ));
        };
        if topicmap.topics.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/topicmap/topics",
                "provide at least one topic",
            ));
        }
        if topicmap.topics.len() > 40 {
            return Err(ChartError::new(
                "too_many_topics",
                "/topicmap/topics",
                "at most 40 topics are supported",
            ));
        }
        if topicmap.islands.len() > 6 {
            return Err(ChartError::new(
                "too_many_islands",
                "/topicmap/islands",
                "at most 6 islands are supported",
            ));
        }
        if !(0..=3).contains(&topicmap.depth_bands) {
            return Err(ChartError::new(
                "invalid_depth_bands",
                "/topicmap/depthBands",
                "depthBands must be between 0 and 3",
            ));
        }

        let mut labels = BTreeSet::new();
        let mut warnings = Vec::new();
        for (index, topic) in topicmap.topics.iter().enumerate() {
            validate_topic(
                topic,
                &format!("/topicmap/topics/{index}"),
                true,
                &mut labels,
                &mut warnings,
            )?;
        }
        for (index, island) in topicmap.islands.iter().enumerate() {
            // An island is already drawn at a small fixed size, so the label-fit warning that
            // steers an oversized topic towards becoming one would be meaningless here.
            validate_topic(
                island,
                &format!("/topicmap/islands/{index}"),
                false,
                &mut labels,
                &mut warnings,
            )?;
        }
        for (index, link) in topicmap.links.iter().enumerate() {
            let path = format!("/topicmap/links/{index}");
            if !labels.contains(link.from.as_str()) {
                return Err(ChartError::new(
                    "unknown_topic_link",
                    format!("{path}/from"),
                    format!("{:?} is not a declared topic or island label", link.from),
                ));
            }
            if !labels.contains(link.to.as_str()) {
                return Err(ChartError::new(
                    "unknown_topic_link",
                    format!("{path}/to"),
                    format!("{:?} is not a declared topic or island label", link.to),
                ));
            }
            if !(0.0..=1.0).contains(&link.weight) {
                return Err(ChartError::new(
                    "invalid_link_weight",
                    format!("{path}/weight"),
                    "weight must be between 0 and 1",
                ));
            }
        }
        if let Some(cartouche) = &topicmap.cartouche {
            validate_text(&cartouche.heading, "/topicmap/cartouche/heading", 100)?;
            validate_text(&cartouche.meta, "/topicmap/cartouche/meta", 200)?;
        }
        Ok(warnings)
    }
}

/// A topic's label and value, shared by `topics` and `islands`: unique text, a finite value of at
/// least 1, and, for a `topic`, a warning once the area would be too small to hold its own label.
fn validate_topic(
    topic: &TopicSpec,
    path: &str,
    warn_if_small: bool,
    labels: &mut BTreeSet<String>,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    validate_text(&topic.label, &format!("{path}/label"), 200)?;
    validate_number(topic.value, &format!("{path}/value"))?;
    if topic.value < 1.0 {
        return Err(ChartError::new(
            "topic_value_out_of_range",
            format!("{path}/value"),
            "value must be at least 1",
        ));
    }
    if let Some(tooltip) = &topic.tooltip {
        validate_text(tooltip, &format!("{path}/tooltip"), 300)?;
    }
    if !labels.insert(topic.label.clone()) {
        return Err(ChartError::new(
            "duplicate_topic_label",
            format!("{path}/label"),
            "topic and island labels must be unique",
        ));
    }
    if warn_if_small && topic.value < 12.0 {
        warnings.push(ChartWarning::new(
            "topic_too_small_for_label",
            format!("{path}/value"),
            "below 12, an area is too small to hold its own label at this scale; consider listing it as an island instead",
        ));
    }
    Ok(())
}

/// A middle value in the 0..3 range: visible depth without crowding a small map.
const fn default_depth_bands() -> u8 {
    2
}
