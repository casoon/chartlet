use super::{ChartType, Dataset, LegendPlacement, Orientation, Series};

mod axis;
mod chart;
mod layers;
mod multiples;
mod validate;

pub use axis::{Gaps, TimeAxisKind, TimeAxisSpec, TimePrecision};
pub use layers::{
    Curve, Dash, LayerSpec, Mark, OhlcPoint, PaneSpec, Shape, Stroke, TimePoint, Tooltips,
};
pub(crate) use validate::{LayerContext, validate_layer_name};

/// Per layer, measured against the size and the render time of the SVG and the HTML profile.
pub(crate) const MAX_TIME_POINTS_PER_LAYER: usize = 2_000;
/// Data layers per pane. Beyond the palette's [`MAX_SERIES`] colors a layer has to bring its own
/// color, and past six lines the legend and the plot stop being readable.
pub(crate) const MAX_TIME_LAYERS: usize = 6;
/// Data layers of a stacked pane: stacked areas lie side by side instead of crossing, so a few
/// more stay readable, such as the sources of an electricity mix.
pub(crate) const MAX_STACKED_LAYERS: usize = 8;
/// Stacked panes of a time chart; more than this stop being readable at the minimum chart height.
pub(crate) const MAX_TIME_PANES: usize = 4;
/// The largest share one pane may claim against another.
pub(crate) const MAX_HEIGHT_RATIO: u32 = 10;
/// Reference lines per pane. They are drawn over the data, so more than a handful hide it.
pub(crate) const MAX_ANNOTATIONS: usize = 6;
/// Small multiples: fewer than two panels is a time chart, more than twelve no longer fit a
/// page width with readable axes.
pub(crate) const MIN_PANELS: usize = 2;
pub(crate) const MAX_PANELS: usize = 12;
pub(crate) const MAX_COLUMNS: u32 = 6;
/// How many categories small multiples of bars may share.
const MAX_BAR_PANEL_CATEGORIES: usize = 20;

fn default_timezone() -> String {
    "UTC".to_owned()
}

const fn default_height_ratio() -> u32 {
    1
}
