use std::collections::HashMap;
use std::fmt::Write as _;

use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    noise,
    scene::{Circle, Element, Line, Polyline, Rect, Scene, Text, TextAnchor, TextStyle},
    spec::{
        AtlasSpec, ChartSpec, ChartType, Corner, Dash, Dataset, LayerRef, LayerSpec, Locale,
        MAX_SERIES, Mark, NumberStyle, Orientation, Shape, Stroke, TopicLinkSpec, TopicMapSpec,
        TopicSpec, ValueFormat,
    },
    time::{self, Precision, TimeZone},
};

pub(crate) const LABEL_SIZE: f64 = 12.0;
/// Extra top margin that makes room for the legend of a multi-series chart.
const LEGEND_HEIGHT: f64 = 24.0;
/// Top of the legend row below a drawn title.
const LEGEND_ROW: f64 = 46.0;
/// Height of the drawn title with its spacing; a chart without a drawn title gains it.
const TITLE_BLOCK: f64 = 44.0;
/// Length of the line sample in a time chart's legend.
const LEGEND_LINE: f64 = 24.0;
/// Gutter left of the plot for the value-axis ticks, and the margin right of it.
pub(crate) const AXIS_GUTTER: u32 = 72;
pub(crate) const PLOT_MARGIN: u32 = 24;
/// Target distance between two time-axis ticks, in pixels: dates need room for `2026-02-02`, a
/// year label is less than half as wide.
const TIME_TICK_SPACING: u32 = 90;
const YEAR_TICK_SPACING: u32 = 64;

/// The tick spacing that fits the labels an axis of this precision writes.
const fn time_tick_spacing(precision: Precision) -> u32 {
    match precision {
        Precision::Year => YEAR_TICK_SPACING,
        Precision::Day | Precision::Minute => TIME_TICK_SPACING,
    }
}

/// The horizontal pixels a plot keeps out of a chart of `width`: the gutter for the value-axis
/// ticks and the margin on the right come off. Measured in whole pixels, so the density check in
/// the specification and the tick count use the same number.
pub(crate) const fn plot_pixels(width: u32) -> u32 {
    width.saturating_sub(AXIS_GUTTER + PLOT_MARGIN)
}

/// Outer margin of a small-multiples grid, left and right.
const MULTIPLES_MARGIN: f64 = 16.0;
/// Gutter left of each panel's plot for its value ticks, and the margin right of it.
const PANEL_GUTTER: u32 = 44;
const PANEL_MARGIN: u32 = 12;

/// The horizontal pixels of one small-multiples panel's plot, by the same rule as
/// [`plot_pixels`]: what is left of a grid cell once the panel's gutter and margin come off.
pub(crate) const fn panel_plot_pixels(width: u32, columns: u32) -> u32 {
    let cell = width.saturating_sub(2 * 16) / if columns == 0 { 1 } else { columns };
    cell.saturating_sub(PANEL_GUTTER + PANEL_MARGIN)
}
/// Inset that keeps the outermost points and their labels inside the plot.
const TIME_INSET: f64 = 6.0;
/// Above this many observations a layer is drawn as a line only. At the default width the
/// markers and their labels would sit closer together than about eleven pixels, and every marker
/// and tooltip costs bytes in the output; the line and the data table carry the values instead.
const MAX_TIME_MARKERS: usize = 60;
const SERIES_BAR_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-bar chartlet-series-1",
    "chartlet-bar chartlet-series-2",
    "chartlet-bar chartlet-series-3",
    "chartlet-bar chartlet-series-4",
];
/// The weight classes of a line, by [`Stroke`].
const LINE_WEIGHTS: [&str; 3] = ["", " chartlet-line-thin", " chartlet-line-bold"];
/// The pattern classes of a line: solid, dashed because modeled, dashed, dotted. A modeled line
/// keeps its own class, which the stylesheet dashes.
const LINE_DASHES: [&str; 4] = [
    "",
    " chartlet-line-modeled",
    " chartlet-line-dashed",
    " chartlet-line-dotted",
];
/// Every combination of color, pattern and weight a time line can take, built once: a declared
/// color first, then the palette colors.
static LINE_CLASSES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
/// Fill of an uncertainty band, in the palette color of its line.
const BAND_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-band chartlet-band-series-1",
    "chartlet-band chartlet-band-series-2",
    "chartlet-band chartlet-band-series-3",
    "chartlet-band chartlet-band-series-4",
];
/// Fill of an area layer, in the palette color of its line.
const AREA_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-area chartlet-band-series-1",
    "chartlet-area chartlet-band-series-2",
    "chartlet-area chartlet-band-series-3",
    "chartlet-area chartlet-band-series-4",
];
/// Markers of a layer that takes a palette color other than the first.
const POINT_CLASSES: [&str; MAX_SERIES] = [
    "chartlet-point",
    "chartlet-point chartlet-point-series-2",
    "chartlet-point chartlet-point-series-3",
    "chartlet-point chartlet-point-series-4",
];

pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let dataset = spec.dataset();
    match (spec.chart_type, spec.orientation) {
        (ChartType::Bar, Orientation::Vertical) => {
            layout_vertical(spec, &dataset, warnings, metrics)
        }
        (ChartType::Bar, Orientation::Horizontal) => {
            layout_horizontal(spec, &dataset, warnings, metrics)
        }
        (ChartType::Line, _) => layout_line(spec, warnings, metrics),
        (ChartType::Time, _) => layout_time(spec, warnings, metrics),
        (ChartType::Multiples, _) => layout_multiples(spec, warnings, metrics),
        (ChartType::Stripes, _) => crate::stripes::layout(spec, warnings, metrics),
        (ChartType::Calendar, _) => crate::calendar::layout(spec, warnings, metrics),
        (ChartType::Rangebar, _) => crate::rangebar::layout(spec, warnings, metrics),
        (ChartType::Topicmap, _) => layout_topicmap(spec, warnings, metrics),
        (ChartType::Atlas, _) => layout_atlas(spec, warnings, metrics),
    }
}

fn layout_vertical(
    spec: &ChartSpec,
    dataset: &Dataset,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = f64::from(AXIS_GUTTER);
    let right = f64::from(PLOT_MARGIN);
    let top = 78.0 + legend_space(dataset);
    let bottom = if spec.category_axis.title.is_some() {
        82.0
    } else {
        62.0
    };
    let plot_width = width - left - right;
    let plot_height = height - top - bottom;
    let scale = NumericScale::from_values(dataset.values(), true);
    let baseline = scale.map(0.0, top + plot_height, top);
    let band = plot_width / count(dataset.categories.len());
    let group = Group::new(band, dataset.series.len());
    let mut elements = base_elements(
        spec,
        &scale,
        PlotArea {
            left,
            top,
            width: plot_width,
            height: plot_height,
            vertical_bars: true,
        },
        warnings,
        metrics,
    );
    add_legend(dataset, left, plot_width, &mut elements, warnings, metrics);
    let mut labels_omitted = false;

    for (index, category) in dataset.categories.iter().enumerate() {
        let center = left + band * (count(index) + 0.5);
        for (series_index, series) in dataset.series.iter().enumerate() {
            let Some(value) = series.values[index] else {
                continue;
            };
            let (offset, thickness) = group.slot(series_index);
            let x = center - group.width / 2.0 + offset;
            let value_y = scale.map(value, top + plot_height, top);
            if value != 0.0 {
                elements.push(Element::Rect(Rect {
                    x,
                    y: baseline.min(value_y),
                    width: thickness,
                    height: (baseline - value_y).abs(),
                    class: bar_class(dataset, series_index),
                    series_index: (dataset.series.len() > 1).then_some(series_index),
                    style_index: None,
                    tooltip: Some(tooltip(
                        category,
                        value,
                        spec.number_style(),
                        series.name.as_deref(),
                    )),
                }));
            }

            if spec.show_values {
                let content = format_value(value, spec.number_style());
                if group.fits(metrics.width(&content, LABEL_SIZE)) {
                    let label = vertical_value_label(value, x + thickness / 2.0, value_y, content);
                    elements.push(series_text(label, dataset, series_index));
                } else {
                    labels_omitted = true;
                }
            }
        }

        let max_label_width = (band - 8.0).max(20.0);
        let label = fit_text(
            category,
            max_label_width,
            LABEL_SIZE,
            metrics,
            warnings,
            &dataset.category_path(index),
        );
        elements.push(Element::Text(Text {
            x: center,
            y: top + plot_height + 24.0,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content: label,
        }));
    }
    warn_if_labels_omitted(labels_omitted, warnings);

    add_bottom_category_title(
        spec,
        left,
        plot_width,
        height,
        &mut elements,
        warnings,
        metrics,
    );

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

fn layout_horizontal(
    spec: &ChartSpec,
    dataset: &Dataset,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let measured_label = dataset
        .categories
        .iter()
        .map(|category| metrics.width(category, LABEL_SIZE))
        .fold(0.0, f64::max);
    let left = (measured_label + 28.0).clamp(88.0, 210.0);
    let right = 68.0;
    let top = 78.0 + legend_space(dataset);
    let bottom = if spec.value_axis.title.is_some() {
        56.0
    } else {
        36.0
    };
    let plot_width = width - left - right;
    let plot_height = height - top - bottom;
    let scale = NumericScale::from_values(dataset.values(), true);
    let baseline = scale.map(0.0, left, left + plot_width);
    let band = plot_height / count(dataset.categories.len());
    let group = Group::new(band, dataset.series.len());
    let mut elements = base_elements(
        spec,
        &scale,
        PlotArea {
            left,
            top,
            width: plot_width,
            height: plot_height,
            vertical_bars: false,
        },
        warnings,
        metrics,
    );
    add_legend(dataset, left, plot_width, &mut elements, warnings, metrics);
    let mut labels_omitted = false;

    for (index, category) in dataset.categories.iter().enumerate() {
        let center = top + band * (count(index) + 0.5);
        for (series_index, series) in dataset.series.iter().enumerate() {
            let Some(value) = series.values[index] else {
                continue;
            };
            let (offset, thickness) = group.slot(series_index);
            let y = center - group.width / 2.0 + offset;
            let value_x = scale.map(value, left, left + plot_width);
            if value != 0.0 {
                elements.push(Element::Rect(Rect {
                    x: baseline.min(value_x),
                    y,
                    width: (baseline - value_x).abs(),
                    height: thickness,
                    class: bar_class(dataset, series_index),
                    series_index: (dataset.series.len() > 1).then_some(series_index),
                    style_index: None,
                    tooltip: Some(tooltip(
                        category,
                        value,
                        spec.number_style(),
                        series.name.as_deref(),
                    )),
                }));
            }

            if spec.show_values {
                if group.fits(LABEL_SIZE + 2.0) {
                    let label = horizontal_value_label(
                        value,
                        value_x,
                        baseline,
                        y + thickness / 2.0,
                        spec.number_style(),
                        metrics,
                    );
                    elements.push(series_text(label, dataset, series_index));
                } else {
                    labels_omitted = true;
                }
            }
        }

        let label = fit_text(
            category,
            left - 28.0,
            LABEL_SIZE,
            metrics,
            warnings,
            &dataset.category_path(index),
        );
        elements.push(Element::Text(Text {
            x: left - 12.0,
            y: center + 4.0,
            class: "chartlet-label",
            anchor: TextAnchor::End,
            content: label,
        }));
    }
    warn_if_labels_omitted(labels_omitted, warnings);

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

fn layout_line(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = f64::from(AXIS_GUTTER);
    let right = f64::from(PLOT_MARGIN);
    let top = 78.0;
    let bottom = if spec.category_axis.title.is_some() {
        82.0
    } else {
        62.0
    };
    let plot_width = width - left - right;
    let plot_height = height - top - bottom;
    let scale = NumericScale::from_values(spec.data.iter().filter_map(|point| point.value), false);
    let mut elements = base_elements(
        spec,
        &scale,
        PlotArea {
            left,
            top,
            width: plot_width,
            height: plot_height,
            vertical_bars: true,
        },
        warnings,
        metrics,
    );
    add_line_data(
        spec,
        PlotArea {
            left,
            top,
            width: plot_width,
            height: plot_height,
            vertical_bars: true,
        },
        &scale,
        &mut elements,
        warnings,
        metrics,
    );

    add_bottom_category_title(
        spec,
        left,
        plot_width,
        height,
        &mut elements,
        warnings,
        metrics,
    );

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// Lays out a time chart: one shared time axis and one value axis per pane, with the layers of
/// every pane drawn inside it.
fn layout_time(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let zone = spec.time_zone().unwrap_or_default();
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let left = f64::from(AXIS_GUTTER);
    let right = f64::from(PLOT_MARGIN);
    // Candles always take a legend entry: it says which body is rising and which is falling.
    let layered =
        spec.series_names().len() > 1 || spec.layers().any(|layer| layer.mark == Mark::Ohlc);
    // Without a drawn title the legend and the plot move up into its place.
    let head = if spec.show_title { 0.0 } else { TITLE_BLOCK };
    let legend = if layered {
        count(legend_rows(spec, width - left - right, metrics)) * LEGEND_HEIGHT
    } else {
        0.0
    };
    let top = 78.0 - head + legend;
    let bottom = if spec.time_axis.title.is_some() {
        56.0
    } else {
        36.0
    };
    let plot = PlotArea {
        left,
        top,
        width: width - left - right,
        height: height - top - bottom,
        vertical_bars: true,
    };

    let frames = pane_frames(spec, zone, plot, warnings);
    let mut elements = Vec::new();
    if spec.show_title {
        let title = fit_text(&spec.title, plot.width, 22.0, metrics, warnings, "/title");
        elements.push(Element::Text(Text {
            x: plot.left,
            y: 30.0,
            class: "chartlet-title",
            anchor: TextAnchor::Start,
            content: title,
        }));
    }
    for (pane_index, frame) in frames.iter().enumerate() {
        let bottom_pane = pane_index + 1 == frames.len();
        push_pane_axes(
            spec,
            pane_index,
            frame,
            bottom_pane,
            &mut elements,
            warnings,
            metrics,
        );
    }
    if layered {
        add_layer_legend(
            spec,
            LEGEND_ROW - head,
            plot.left,
            plot.width,
            &mut elements,
            warnings,
            metrics,
        );
    }

    for (pane_index, frame) in frames.iter().enumerate() {
        draw_pane(
            spec,
            pane_index,
            frame,
            Detail::Full,
            &mut elements,
            warnings,
            metrics,
        );
    }

    if let Some(title) = &spec.time_axis.title {
        let title = fit_text(
            title,
            plot.width,
            LABEL_SIZE,
            metrics,
            warnings,
            "/timeAxis/title",
        );
        elements.push(Element::Text(Text {
            x: plot.left + plot.width / 2.0,
            y: height - 14.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: title,
        }));
    }

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// The title, the legend and the shared value axis title above a small-multiples grid; returns
/// where the grid starts.
fn multiples_header(
    spec: &ChartSpec,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> f64 {
    let width = f64::from(spec.width);
    let title = fit_text(
        &spec.title,
        width - 2.0 * MULTIPLES_MARGIN,
        22.0,
        metrics,
        warnings,
        "/title",
    );
    elements.push(Element::Text(Text {
        x: MULTIPLES_MARGIN,
        y: 30.0,
        class: "chartlet-title",
        anchor: TextAnchor::Start,
        content: title,
    }));

    let mut cursor = 46.0;
    if spec.series_names().len() > 1 {
        add_layer_legend(
            spec,
            LEGEND_ROW,
            MULTIPLES_MARGIN,
            width - 2.0 * MULTIPLES_MARGIN,
            elements,
            warnings,
            metrics,
        );
        cursor += count(legend_rows(spec, width - 2.0 * MULTIPLES_MARGIN, metrics)) * LEGEND_HEIGHT;
    }
    if let Some(axis_title) = &spec.value_axis.title {
        elements.push(Element::Text(Text {
            x: MULTIPLES_MARGIN,
            y: cursor + 10.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Start,
            content: fit_text(
                axis_title,
                width - 2.0 * MULTIPLES_MARGIN,
                LABEL_SIZE,
                metrics,
                warnings,
                "/valueAxis/title",
            ),
        }));
        cursor += 20.0;
    }
    cursor + 8.0
}

/// Lays out small multiples: a grid of small time plots that share the value scale and the time
/// span, so that the panels can be compared by position alone.
fn layout_multiples(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let zone = spec.time_zone().unwrap_or_default();
    let width = f64::from(spec.width);
    let height = f64::from(spec.height);
    let columns = spec.multiples_columns();
    let panels = spec.panes.len();
    let rows = panels.div_ceil(usize::try_from(columns).expect("columns are limited"));

    let mut elements = Vec::new();
    let grid_top = multiples_header(spec, &mut elements, warnings, metrics);
    let grid_bottom = height
        - if spec.time_axis.title.is_some() {
            36.0
        } else {
            12.0
        };
    let cell_width = (width - 2.0 * MULTIPLES_MARGIN) / f64::from(columns);
    let cell_height = (grid_bottom - grid_top) / count(rows);

    let span = time_span(spec, zone);
    let precision = spec.time_precision(zone);
    let scale = time_scale(spec, zone, None);
    let max_ticks = usize::try_from(
        (panel_plot_pixels(spec.width, columns) / time_tick_spacing(precision)).max(2),
    )
    .expect("a usize is at least 32 bits wide");

    for (pane_index, pane) in spec.panes.iter().enumerate() {
        let column = pane_index % usize::try_from(columns).expect("columns are limited");
        let row = pane_index / usize::try_from(columns).expect("columns are limited");
        let cell_left = MULTIPLES_MARGIN + cell_width * count(column);
        let cell_top = grid_top + cell_height * count(row);
        let plot = PlotArea {
            left: cell_left + f64::from(PANEL_GUTTER),
            top: cell_top + 30.0,
            width: cell_width - f64::from(PANEL_GUTTER + PANEL_MARGIN),
            height: cell_height - 30.0 - 28.0,
            vertical_bars: true,
        };
        let frame = TimeFrame {
            plot,
            span,
            zone,
            precision,
            scale,
            style: spec.number_style(),
        };

        let panel_title = pane.title.as_deref().unwrap_or_default();
        elements.push(Element::Text(Text {
            x: plot.left,
            y: cell_top + 18.0,
            class: "chartlet-panel-title",
            anchor: TextAnchor::Start,
            content: fit_text(
                panel_title,
                plot.width,
                13.0,
                metrics,
                warnings,
                &format!("/panes/{pane_index}/title"),
            ),
        }));
        push_value_grid(&frame, &mut elements);
        push_time_ticks(&frame, max_ticks, true, width, metrics, &mut elements);
        draw_pane(
            spec,
            pane_index,
            &frame,
            Detail::Compact,
            &mut elements,
            warnings,
            metrics,
        );
    }

    if cell_height - 58.0 < 60.0 {
        warnings.push(ChartWarning::new(
            "dense_chart",
            "/height",
            "the panels are less than 60 pixels tall; raise height or use more columns",
        ));
    }

    if let Some(axis_title) = &spec.time_axis.title {
        elements.push(Element::Text(Text {
            x: width / 2.0,
            y: height - 14.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: fit_text(
                axis_title,
                width - 2.0 * MULTIPLES_MARGIN,
                LABEL_SIZE,
                metrics,
                warnings,
                "/timeAxis/title",
            ),
        }));
    }

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// Fraction of the plot area topic circles fill; the rest is sea. Matches the design brief.
const TOPIC_FILL_RATIO: f64 = 0.42;
/// Island radius, relative to the smallest topic's radius.
const ISLAND_RADIUS_FACTOR: f64 = 0.55;
/// Gap enforced between two circle edges, relative to the smaller of the two radii.
const SEA_GAP_FACTOR: f64 = 0.14;
/// The golden angle: successive spiral points never align radially, so the starting layout has
/// no seams for the relaxation to get stuck on.
const SPIRAL_ANGLE: f64 = 2.399_96;
/// Fixed iteration count, so the same specification always relaxes to the same layout.
const RELAXATION_ITERATIONS: u32 = 120;
/// Extra collision-only passes after the main relaxation: a single separation pass per iteration
/// leaves a residual overlap of a few percent of the smaller radius (attraction and centering
/// keep pulling circles back together while collisions are resolved one pair at a time); these
/// passes settle that residual without the competing forces that caused it.
const SETTLE_ITERATIONS: u32 = 60;
/// Share of the distance to a link's target length pulled back per iteration.
const LINK_PULL_FACTOR: f64 = 0.08;
/// Share of the distance to the center corrected per iteration, so the cluster does not drift.
const CENTERING_FACTOR: f64 = 0.01;
/// At most this many links stay attached to any one topic or island; weaker ones are dropped
/// with a warning rather than cluttering that area with routes.
const MAX_LINKS_PER_TOPIC: usize = 3;
const ISLAND_GRID_COLUMNS: usize = 16;
const ISLAND_GRID_ROWS: usize = 10;
/// How far a coastline may dip inside and bulge outside its nominal radius. The relaxation keeps
/// areas [`MAX_WOBBLE`] apart rather than one nominal radius, so however the noise falls, two
/// coastlines can never touch; labels are measured against [`MIN_WOBBLE`], the narrowest the
/// coast can be where the text sits.
const MIN_WOBBLE: f64 = 0.84;
const MAX_WOBBLE: f64 = 1.16;
/// The two noise octaves: the first one cuts bays, the second adds the small jags.
const COAST_AMPLITUDES: [f64; 2] = [0.16, 0.05];
const COAST_FREQUENCIES: [f64; 2] = [1.7, 4.3];
/// Each depth line sits this much further out than the coast before it.
const DEPTH_BAND_STEP: f64 = 0.05;
/// Stroke width of the coastal halo. Must match `chartlet-topic-halo` in the stylesheet: half of
/// it is kept free at the canvas edge so the halo is never cut off.
const HALO_WIDTH: f64 = 10.0;
/// Path points stay inside this share of the nominal radius, which is well within the narrowest
/// the coastline can be.
const PATH_POINT_DISC: f64 = 0.7;
/// Radius of a single path point, and the distance two of them keep from each other.
const PATH_POINT_RADIUS: f64 = 2.2;
const PATH_POINT_SPACING: f64 = 6.0;
/// An area's name grows with the room it has, between these two sizes. The count below it stays
/// put: it carries information rather than decoration, so it is never shrunk to fit.
const TOPIC_LABEL_MIN: f64 = 19.0;
const TOPIC_LABEL_MAX: f64 = 34.0;
const TOPIC_LABEL_RATIO: f64 = 0.34;
const TOPIC_VALUE_SIZE: f64 = 18.0;
/// Distance from the name's baseline down to the count's.
const TOPIC_VALUE_DROP: f64 = 28.0;
/// Size of a label that had to move out of its area. Not smaller than this: a chart authored at
/// 1200 and shown in an 880 pixel container renders text at 0.73 of its size, and 12 effective
/// pixels is the floor.
const TOPIC_OUTSIDE_SIZE: f64 = 17.0;
/// Distance the name of an outside label keeps from its count, and from the next label below it.
const TOPIC_OUTSIDE_DROP: f64 = 20.0;
const TOPIC_OUTSIDE_SPACING: f64 = 46.0;
/// Clear space kept between the end of a label and the coast beside it.
const LABEL_PADDING: f64 = 4.0;
/// Cells the graticule divides the sea into. Enough to read as a map grid, few enough that it
/// stays behind the areas rather than competing with them.
const GRATICULE_COLUMNS: usize = 5;
const GRATICULE_ROWS: usize = 3;
/// Radius of the compass rose, and the clear water it keeps around itself.
const COMPASS_RADIUS: f64 = 26.0;
const FURNITURE_MARGIN: f64 = 10.0;
/// Type sizes and padding of the cartouche.
const CARTOUCHE_HEADING_SIZE: f64 = 15.0;
const CARTOUCHE_META_SIZE: f64 = 12.0;
const CARTOUCHE_PADDING: f64 = 12.0;
const CARTOUCHE_HEIGHT: f64 = 58.0;

/// Keeps at most [`MAX_LINKS_PER_TOPIC`] links per topic or island, strongest weight first;
/// weaker links beyond that are reported as `link_dropped` rather than drawn.
fn cap_links<'a>(
    topicmap: &'a TopicMapSpec,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<&'a TopicLinkSpec> {
    let mut by_weight: Vec<(usize, &TopicLinkSpec)> = topicmap.links.iter().enumerate().collect();
    by_weight.sort_by(|a, b| b.1.weight.total_cmp(&a.1.weight));

    let mut incident: HashMap<&str, usize> = HashMap::new();
    let mut kept = Vec::new();
    for (index, link) in by_weight {
        let from_count = *incident.get(link.from.as_str()).unwrap_or(&0);
        let to_count = *incident.get(link.to.as_str()).unwrap_or(&0);
        if from_count < MAX_LINKS_PER_TOPIC && to_count < MAX_LINKS_PER_TOPIC {
            *incident.entry(link.from.as_str()).or_insert(0) += 1;
            *incident.entry(link.to.as_str()).or_insert(0) += 1;
            kept.push(link);
        } else {
            warnings.push(ChartWarning::new(
                "link_dropped",
                format!("/topicmap/links/{index}"),
                "dropped: more than 3 routes would meet at one of its ends",
            ));
        }
    }
    kept
}

/// Phyllotaxis starting positions, largest topic first, around `center`.
fn spiral_start(count_topics: usize, mean_radius: f64, center: (f64, f64)) -> Vec<(f64, f64)> {
    let spiral_step = 2.0 * mean_radius;
    (0..count_topics)
        .map(|i| {
            let angle = count(i) * SPIRAL_ANGLE;
            let r = spiral_step * count(i).sqrt();
            (center.0 + angle.cos() * r, center.1 + angle.sin() * r)
        })
        .collect()
}

/// Runs the fixed-iteration relaxation: pairwise collision, spring attraction along kept links
/// between two topics, and a light pull back towards the center.
///
/// That pull is stronger along the plot's short side than along its long one, in proportion to
/// `aspect`. A cluster relaxed with an even pull comes out round, and normalizing a round cluster
/// into a wide canvas fits it to the height and leaves the width empty.
fn relax(
    positions: &mut [(f64, f64)],
    radii: &[f64],
    center: (f64, f64),
    links: &[(usize, usize, f64)],
    aspect: f64,
    reserved: &[(f64, f64, f64, f64)],
) {
    let pull = (CENTERING_FACTOR / aspect, CENTERING_FACTOR * aspect);
    for _ in 0..RELAXATION_ITERATIONS {
        for a in 0..positions.len() {
            for b in (a + 1)..positions.len() {
                separate(positions, radii, a, b);
            }
        }
        for &(from, to, weight) in links {
            attract(positions, radii, from, to, weight);
        }
        for position in positions.iter_mut() {
            position.0 += (center.0 - position.0) * pull.0;
            position.1 += (center.1 - position.1) * pull.1;
        }
        keep_out(positions, radii, reserved);
    }
    // The settling passes repeat the reserved areas as well: a late correction between two
    // coastlines must not put one of them back under the cartouche.
    for _ in 0..SETTLE_ITERATIONS {
        for a in 0..positions.len() {
            for b in (a + 1)..positions.len() {
                separate(positions, radii, a, b);
            }
        }
        keep_out(positions, radii, reserved);
    }
}

/// Pushes every area out of the rectangles the map furniture has claimed.
fn keep_out(positions: &mut [(f64, f64)], radii: &[f64], reserved: &[(f64, f64, f64, f64)]) {
    for (position, radius) in positions.iter_mut().zip(radii) {
        for rect in reserved {
            push_out(position, radius * MAX_WOBBLE, *rect);
        }
    }
}

/// How far a point lies from a rectangle; zero while it is inside one.
fn distance_to(point: (f64, f64), rect: (f64, f64, f64, f64)) -> f64 {
    let (left, top, width, height) = rect;
    let nearest = (
        point.0.clamp(left, left + width),
        point.1.clamp(top, top + height),
    );
    (point.0 - nearest.0).hypot(point.1 - nearest.1)
}

/// Moves one area clear of a reserved rectangle, by the shortest way out.
fn push_out(position: &mut (f64, f64), reach: f64, rect: (f64, f64, f64, f64)) {
    let (left, top, width, height) = rect;
    let (right, bottom) = (left + width, top + height);
    let nearest = (position.0.clamp(left, right), position.1.clamp(top, bottom));
    let (dx, dy) = (position.0 - nearest.0, position.1 - nearest.1);
    let distance = dx.hypot(dy);
    if distance >= reach {
        return;
    }
    if distance > 1e-9 {
        let correction = (reach - distance) / distance;
        position.0 += dx * correction;
        position.1 += dy * correction;
        return;
    }
    // The center sits inside the rectangle, so there is no direction to push along: leave by
    // whichever edge is closest.
    let exits = [
        (left - reach - position.0, 0.0),
        (right + reach - position.0, 0.0),
        (0.0, top - reach - position.1),
        (0.0, bottom + reach - position.1),
    ];
    let (dx, dy) = exits
        .into_iter()
        .min_by(|a, b| a.0.hypot(a.1).total_cmp(&b.0.hypot(b.1)))
        .expect("a rectangle has four edges");
    position.0 += dx;
    position.1 += dy;
}

/// The distance two areas keep between their centers: the coastlines at their widest, plus the
/// strip of sea between them. Measuring with the nominal radius instead would let a bulge of one
/// coastline reach into its neighbour.
fn keep_apart(radii: &[f64], a: usize, b: usize) -> f64 {
    (radii[a] + radii[b]) * MAX_WOBBLE + SEA_GAP_FACTOR * radii[a].min(radii[b])
}

/// Pushes two overlapping circles apart along their connecting axis, half the correction each.
fn separate(positions: &mut [(f64, f64)], radii: &[f64], a: usize, b: usize) {
    let (dx, dy) = (
        positions[b].0 - positions[a].0,
        positions[b].1 - positions[a].1,
    );
    let distance = dx.hypot(dy);
    let min_distance = keep_apart(radii, a, b);
    if distance >= min_distance {
        return;
    }
    // Coincident starting points cannot be normalized into a direction; nudge deterministically.
    let (ux, uy) = if distance > 1e-9 {
        (dx / distance, dy / distance)
    } else {
        (1.0, 0.0)
    };
    let correction = (min_distance - distance) / 2.0;
    positions[a].0 -= ux * correction;
    positions[a].1 -= uy * correction;
    positions[b].0 += ux * correction;
    positions[b].1 += uy * correction;
}

/// Pulls (or pushes) two linked topics toward the distance at which their areas just about
/// touch, weighted by how strong the declared neighborhood is.
fn attract(positions: &mut [(f64, f64)], radii: &[f64], a: usize, b: usize, weight: f64) {
    let (dx, dy) = (
        positions[b].0 - positions[a].0,
        positions[b].1 - positions[a].1,
    );
    let distance = dx.hypot(dy);
    if distance <= 1e-9 {
        return;
    }
    // One sea gap further out than the collision distance, so a linked pair settles just clear of
    // each other instead of fighting the separation above for the rest of the iterations.
    let target = keep_apart(radii, a, b) + SEA_GAP_FACTOR * radii[a].min(radii[b]);
    let pull = (distance - target) * weight * LINK_PULL_FACTOR / 2.0;
    let (ux, uy) = (dx / distance, dy / distance);
    positions[a].0 += ux * pull;
    positions[a].1 += uy * pull;
    positions[b].0 -= ux * pull;
    positions[b].1 -= uy * pull;
}

/// How far beyond its nominal radius an area actually reaches: the widest the coastline can be,
/// pushed out once more by every depth line drawn around it.
fn outer_extent(depth_bands: u8) -> f64 {
    MAX_WOBBLE * (1.0 + DEPTH_BAND_STEP * f64::from(depth_bands))
}

/// Scales and centers the relaxed cluster so everything drawn fits the plot: only here does the
/// map's final size emerge, since the physics above works at an arbitrary scale.
///
/// `extents` says how far past its nominal radius each area actually reaches — its widest
/// coastline point, pushed out by the depth lines around it — so the fit is measured against what
/// is drawn rather than against the bare circle. The plot keeps half the halo width free at its
/// edge, otherwise the outermost ring would be cut off by the canvas.
fn normalize(
    positions: &mut [(f64, f64)],
    radii: &mut [f64],
    plot: (f64, f64, f64, f64),
    extents: &[f64],
) {
    let (margin, top, plot_width, plot_height) = plot;
    let target_center = (margin + plot_width / 2.0, top + plot_height / 2.0);
    let inset = HALO_WIDTH / 2.0;
    let (usable_width, usable_height) = (plot_width - inset * 2.0, plot_height - inset * 2.0);
    let mut bounds = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for ((&(x, y), &radius), &extent) in positions.iter().zip(radii.iter()).zip(extents) {
        let reach = radius * extent;
        bounds.0 = bounds.0.min(x - reach);
        bounds.1 = bounds.1.max(x + reach);
        bounds.2 = bounds.2.min(y - reach);
        bounds.3 = bounds.3.max(y + reach);
    }
    let bbox_width = (bounds.1 - bounds.0).max(1.0);
    let bbox_height = (bounds.3 - bounds.2).max(1.0);
    let fit = (usable_width / bbox_width).min(usable_height / bbox_height);
    let bbox_center = (
        f64::midpoint(bounds.0, bounds.1),
        f64::midpoint(bounds.2, bounds.3),
    );
    for ((x, y), r) in positions.iter_mut().zip(radii.iter_mut()) {
        *x = target_center.0 + (*x - bbox_center.0) * fit;
        *y = target_center.1 + (*y - bbox_center.1) * fit;
        *r *= fit;
    }
}

/// Places every island into the sea gap that keeps it farthest from every topic and from every
/// island placed before it: an exhaustive, and therefore deterministic, grid search.
///
/// Distances are measured between what is actually drawn — coastline plus depth lines — so an
/// island never lands on a neighbour's outermost ring.
fn place_islands<'a>(
    topicmap: &'a TopicMapSpec,
    topic_circles: &[(f64, f64, f64)],
    plot: (f64, f64, f64, f64),
    reserved: &[(f64, f64, f64, f64)],
) -> Vec<(&'a str, f64, f64, f64)> {
    let (margin, top, plot_width, plot_height) = plot;
    let extent = outer_extent(topicmap.depth_bands);
    let island_radius = topic_circles
        .iter()
        .map(|&(_, _, r)| r)
        .fold(f64::INFINITY, f64::min)
        * ISLAND_RADIUS_FACTOR;
    // The island's own rings have to stay on the canvas as much as a topic's do.
    let reach = island_radius * extent + HALO_WIDTH / 2.0;

    let mut placed: Vec<(f64, f64, f64)> = Vec::with_capacity(topicmap.islands.len());
    let mut labeled = Vec::with_capacity(topicmap.islands.len());
    for island in &topicmap.islands {
        let mut best = (margin + reach, top + reach, f64::NEG_INFINITY);
        for row in 0..ISLAND_GRID_ROWS {
            for column in 0..ISLAND_GRID_COLUMNS {
                let x = (margin + plot_width * (count(column) + 0.5) / count(ISLAND_GRID_COLUMNS))
                    .clamp(margin + reach, margin + plot_width - reach);
                let y = (top + plot_height * (count(row) + 0.5) / count(ISLAND_GRID_ROWS))
                    .clamp(top + reach, top + plot_height - reach);
                // Water the furniture has claimed is not water an island may take.
                if reserved
                    .iter()
                    .any(|rect| distance_to((x, y), *rect) < reach)
                {
                    continue;
                }
                let score = topic_circles
                    .iter()
                    .chain(placed.iter())
                    .map(|&(cx, cy, r)| (x - cx).hypot(y - cy) - (r + island_radius) * extent)
                    .fold(f64::INFINITY, f64::min);
                if score > best.2 {
                    best = (x, y, score);
                }
            }
        }
        placed.push((best.0, best.1, island_radius));
        labeled.push((island.label.as_str(), best.0, best.1, island_radius));
    }
    labeled
}

/// Where one area ended up and the coastline it will be drawn with.
struct Placement {
    center: (f64, f64),
    radius: f64,
    seed: u64,
    profile: Vec<f64>,
}

/// Places every topic and island, keyed by label: phyllotaxis starting positions,
/// fixed-iteration relaxation with the map furniture's rectangles as immovable water, a final
/// normalize to the plot, then islands into the sea that is left.
///
/// Coastlines are generated before the normalize so their real reach, rather than the worst case
/// the clamp allows, decides how much room the map needs. Their shape does not depend on the
/// scale, so generating them at the pre-normalize radius costs nothing.
fn topicmap_positions<'a>(
    topicmap: &'a TopicMapSpec,
    ordered: &[(usize, &'a TopicSpec)],
    kept_links: &[&TopicLinkSpec],
    plot: (f64, f64, f64, f64),
    reserved: &[(f64, f64, f64, f64)],
) -> HashMap<&'a str, Placement> {
    let (margin, top, plot_width, plot_height) = plot;
    let total_value: f64 = ordered.iter().map(|(_, topic)| topic.value).sum();
    let canvas_area = plot_width * plot_height;
    let area_scale = (canvas_area * TOPIC_FILL_RATIO / (std::f64::consts::PI * total_value)).sqrt();
    let mut radii: Vec<f64> = ordered
        .iter()
        .map(|(_, topic)| area_scale * topic.value.sqrt())
        .collect();
    // The seed follows a topic's position in the specification, not its rank by value: one more
    // entry must not redraw the coastline of every area it moved past in the order.
    let seeds: Vec<u64> = ordered
        .iter()
        .map(|(index, _)| seed_for(topicmap.seed, *index))
        .collect();
    let profiles: Vec<Vec<f64>> = radii
        .iter()
        .zip(&seeds)
        .map(|(radius, seed)| coastline_profile(*radius, *seed))
        .collect();
    let band_reach = 1.0 + DEPTH_BAND_STEP * f64::from(topicmap.depth_bands);
    let extents: Vec<f64> = profiles
        .iter()
        .map(|profile| profile.iter().copied().fold(0.0, f64::max) * band_reach)
        .collect();

    let label_index: HashMap<&str, usize> = ordered
        .iter()
        .enumerate()
        .map(|(position, (_, topic))| (topic.label.as_str(), position))
        .collect();
    let topic_links: Vec<(usize, usize, f64)> = kept_links
        .iter()
        .filter_map(|link| {
            let from = *label_index.get(link.from.as_str())?;
            let to = *label_index.get(link.to.as_str())?;
            Some((from, to, link.weight))
        })
        .collect();

    let center = (margin + plot_width / 2.0, top + plot_height / 2.0);
    let mean_radius = radii.iter().sum::<f64>() / count(radii.len().max(1));
    let mut positions = spiral_start(ordered.len(), mean_radius, center);
    relax(
        &mut positions,
        &radii,
        center,
        &topic_links,
        (plot_width / plot_height).sqrt(),
        reserved,
    );
    normalize(&mut positions, &mut radii, plot, &extents);
    // The normalize moves and scales everything, so the areas have to be shown the reserved
    // water once more afterwards.
    keep_out(&mut positions, &radii, reserved);

    let mut merged: HashMap<&str, Placement> = ordered
        .iter()
        .zip(positions.iter().zip(radii.iter()))
        .zip(seeds.iter().zip(profiles))
        .map(|(((_, topic), (&center, &radius)), (&seed, profile))| {
            (
                topic.label.as_str(),
                Placement {
                    center,
                    radius,
                    seed,
                    profile,
                },
            )
        })
        .collect();
    let topic_circles: Vec<(f64, f64, f64)> = merged
        .values()
        .map(|placed| (placed.center.0, placed.center.1, placed.radius))
        .collect();
    for (index, (label, x, y, radius)) in place_islands(topicmap, &topic_circles, plot, reserved)
        .into_iter()
        .enumerate()
    {
        let seed = seed_for(topicmap.seed, index);
        merged.insert(
            label,
            Placement {
                center: (x, y),
                radius,
                seed,
                profile: coastline_profile(radius, seed),
            },
        );
    }
    merged
}

/// The coastline seed of the area declared at `index`.
fn seed_for(base: u64, index: usize) -> u64 {
    base.wrapping_add(u64::try_from(index).expect("an index fits in a u64"))
}

/// One laid-out area, ready to be drawn: which topic it stands for, and where it was placed. The
/// wobble profile is carried along because the halo, the fill and every depth line are the same
/// coastline at a different distance from the center.
struct Area<'a> {
    topic: &'a TopicSpec,
    path: String,
    /// Which `chartlet-topic-N` class this area carries. Topics come first, in the order the
    /// specification lists them, then the islands — so a host page can address every area, and
    /// the picker's indices still line up with the topics it offers.
    index: Option<usize>,
    placed: &'a Placement,
    style: NumberStyle,
}

impl Area<'_> {
    fn center(&self) -> (f64, f64) {
        self.placed.center
    }

    fn radius(&self) -> f64 {
        self.placed.radius
    }

    fn tooltip(&self) -> String {
        self.topic.tooltip.clone().unwrap_or_else(|| {
            format!(
                "{}: {}",
                self.topic.label,
                format_value(self.topic.value, self.style)
            )
        })
    }
}

/// How many points a coastline is sampled at: larger areas get more, but a small island gains
/// nothing from detail its size cannot show, and every point costs bytes in the output.
fn coastline_resolution(radius: f64) -> usize {
    let wanted = 12.0 + radius / 6.0;
    (24..=48)
        .find(|points| count(*points) >= wanted)
        .unwrap_or(48)
}

/// The wobble profile of one coastline: what the nominal radius is multiplied by at each sampled
/// angle.
///
/// The profile is normalized so the shape it describes encloses exactly the area of the nominal
/// circle — the area is what carries the value, so it must not drift with the noise — and clamped
/// afterwards, which is what makes the relaxation's collision distance an upper bound.
fn coastline_profile(radius: f64, seed: u64) -> Vec<f64> {
    let resolution = coastline_resolution(radius);
    let mut profile: Vec<f64> = (0..resolution)
        .map(|step| {
            let angle = count(step) / count(resolution) * std::f64::consts::TAU;
            let (sin, cos) = angle.sin_cos();
            let wobble = COAST_AMPLITUDES.iter().zip(COAST_FREQUENCIES).fold(
                1.0,
                |wobble, (amplitude, frequency)| {
                    wobble + amplitude * noise::value_noise(cos * frequency, sin * frequency, seed)
                },
            );
            wobble.clamp(MIN_WOBBLE, MAX_WOBBLE)
        })
        .collect();

    let enclosed = unit_polygon_area(&profile);
    if enclosed > 0.0 {
        let correction = (std::f64::consts::PI / enclosed).sqrt();
        for wobble in &mut profile {
            *wobble = (*wobble * correction).clamp(MIN_WOBBLE, MAX_WOBBLE);
        }
    }
    profile
}

/// The area enclosed by a profile drawn at radius 1: the shoelace formula, simplified because
/// every angular step is the same size.
fn unit_polygon_area(profile: &[f64]) -> f64 {
    let step = std::f64::consts::TAU / count(profile.len());
    0.5 * step.sin()
        * profile
            .iter()
            .enumerate()
            .map(|(index, radius)| radius * profile[(index + 1) % profile.len()])
            .sum::<f64>()
}

/// A coastline, or one of the depth lines around it, as a closed ring of points.
fn coastline(area: &Area, scale: f64) -> Vec<(f64, f64)> {
    let resolution = area.placed.profile.len();
    let mut points: Vec<(f64, f64)> = area
        .placed
        .profile
        .iter()
        .enumerate()
        .map(|(step, wobble)| {
            let angle = count(step) / count(resolution) * std::f64::consts::TAU;
            let (sin, cos) = angle.sin_cos();
            let distance = area.radius() * wobble * scale;
            (
                area.center().0 + cos * distance,
                area.center().1 + sin * distance,
            )
        })
        .collect();
    // A polyline is filled as if it were closed, but it is not stroked that way: without the
    // repeated first point the halo and the depth lines would show a gap.
    if let Some(&first) = points.first() {
        points.push(first);
    }
    points
}

/// Every area of the map: the topics first, in the order they were laid out, then the islands.
fn build_areas<'a>(
    topicmap: &'a TopicMapSpec,
    ordered: &[(usize, &'a TopicSpec)],
    positions: &'a HashMap<&str, Placement>,
    style: NumberStyle,
) -> (Vec<Area<'a>>, Vec<Area<'a>>) {
    let areas = ordered
        .iter()
        .map(|(index, topic)| build_area(topic, "topics", *index, *index, positions, style))
        .collect();
    let islands = topicmap
        .islands
        .iter()
        .enumerate()
        .map(|(index, island)| {
            let selectable = topicmap.topics.len() + index;
            build_area(island, "islands", index, selectable, positions, style)
        })
        .collect();
    (areas, islands)
}

/// Pairs a topic with the placement it was given, and with the path its warnings point at.
fn build_area<'a>(
    topic: &'a TopicSpec,
    group: &str,
    index: usize,
    selectable: usize,
    positions: &'a HashMap<&str, Placement>,
    style: NumberStyle,
) -> Area<'a> {
    Area {
        topic,
        path: format!("/topicmap/{group}/{index}"),
        index: Some(selectable),
        placed: positions
            .get(topic.label.as_str())
            .expect("every topic and island was placed"),
        style,
    }
}

/// A route as a gently curved polyline. A straight line between two neighbouring areas reads as
/// a connector; the curve reads as something drawn on a map.
fn route(from: (f64, f64), to: (f64, f64)) -> Vec<(f64, f64)> {
    const SEGMENTS: usize = 12;
    const BULGE: f64 = 0.1;
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let control = (
        f64::midpoint(from.0, to.0) - dy * BULGE,
        f64::midpoint(from.1, to.1) + dx * BULGE,
    );
    (0..=SEGMENTS)
        .map(|step| {
            let t = count(step) / count(SEGMENTS);
            let rest = 1.0 - t;
            (
                rest * rest * from.0 + 2.0 * rest * t * control.0 + t * t * to.0,
                rest * rest * from.1 + 2.0 * rest * t * control.1 + t * t * to.1,
            )
        })
        .collect()
}

/// The cartouche: the legend box printed on the map, with the text it will actually carry.
struct Cartouche {
    heading: String,
    meta: String,
    frame: (f64, f64, f64, f64),
}

/// The parts of the drawing that are not data. Their rectangles go to the relaxation as reserved
/// water: without that, an area eventually grows underneath the cartouche.
struct Furniture {
    compass: Option<(f64, f64)>,
    cartouche: Option<Cartouche>,
}

impl Furniture {
    /// The rectangles no area may reach into, each with a little clear water around it.
    fn reserved(&self) -> Vec<(f64, f64, f64, f64)> {
        let mut rects = Vec::new();
        if let Some((x, y)) = self.compass {
            let reach = COMPASS_RADIUS + FURNITURE_MARGIN;
            rects.push((x - reach, y - reach, reach * 2.0, reach * 2.0));
        }
        if let Some(cartouche) = &self.cartouche {
            let (x, y, width, height) = cartouche.frame;
            rects.push((
                x - FURNITURE_MARGIN,
                y - FURNITURE_MARGIN,
                width + FURNITURE_MARGIN * 2.0,
                height + FURNITURE_MARGIN * 2.0,
            ));
        }
        rects
    }
}

/// The top left corner of a box of this size, tucked into one corner of the plot.
fn corner_origin(
    corner: Corner,
    plot: (f64, f64, f64, f64),
    width: f64,
    height: f64,
) -> (f64, f64) {
    let (margin, top, plot_width, plot_height) = plot;
    let (left, right) = (
        margin + FURNITURE_MARGIN,
        margin + plot_width - width - FURNITURE_MARGIN,
    );
    let (upper, lower) = (
        top + FURNITURE_MARGIN,
        top + plot_height - height - FURNITURE_MARGIN,
    );
    match corner {
        Corner::TopLeft => (left, upper),
        Corner::TopRight => (right, upper),
        Corner::BottomLeft => (left, lower),
        Corner::BottomRight => (right, lower),
    }
}

/// Decides where the compass rose and the cartouche sit, and how wide the cartouche has to be for
/// its own text. The rose takes the top left corner, unless the cartouche was put there.
fn plan_furniture(
    topicmap: &TopicMapSpec,
    plot: (f64, f64, f64, f64),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Furniture {
    let (_, _, plot_width, _) = plot;
    let widest = plot_width * 0.4;
    let cartouche = topicmap.cartouche.as_ref().map(|spec| {
        let heading = fit_text(
            &spec.heading,
            widest,
            CARTOUCHE_HEADING_SIZE,
            metrics,
            warnings,
            "/topicmap/cartouche/heading",
        );
        let meta = fit_text(
            &spec.meta,
            widest,
            CARTOUCHE_META_SIZE,
            metrics,
            warnings,
            "/topicmap/cartouche/meta",
        );
        let width = metrics
            .width(&heading, CARTOUCHE_HEADING_SIZE)
            .max(metrics.width(&meta, CARTOUCHE_META_SIZE))
            + CARTOUCHE_PADDING * 2.0;
        let (x, y) = corner_origin(spec.corner, plot, width, CARTOUCHE_HEIGHT);
        Cartouche {
            heading,
            meta,
            frame: (x, y, width, CARTOUCHE_HEIGHT),
        }
    });

    let rose_corner = match topicmap.cartouche.as_ref().map(|spec| spec.corner) {
        Some(Corner::TopLeft) => Corner::TopRight,
        _ => Corner::TopLeft,
    };
    let compass = topicmap.compass.then(|| {
        let (x, y) = corner_origin(
            rose_corner,
            plot,
            COMPASS_RADIUS * 2.0,
            COMPASS_RADIUS * 2.0,
        );
        (x + COMPASS_RADIUS, y + COMPASS_RADIUS)
    });
    Furniture { compass, cartouche }
}

/// The sea the map sits in, and the grid over it.
fn push_sea(elements: &mut Vec<Element>, plot: (f64, f64, f64, f64), graticule: bool) {
    let (margin, top, plot_width, plot_height) = plot;
    elements.push(Element::Rect(Rect {
        x: margin,
        y: top,
        width: plot_width,
        height: plot_height,
        class: "chartlet-sea",
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    if !graticule {
        return;
    }
    for column in 1..GRATICULE_COLUMNS {
        let x = margin + plot_width * count(column) / count(GRATICULE_COLUMNS);
        elements.push(Element::Line(Line {
            x1: x,
            y1: top,
            x2: x,
            y2: top + plot_height,
            class: "chartlet-graticule",
        }));
    }
    for row in 1..GRATICULE_ROWS {
        let y = top + plot_height * count(row) / count(GRATICULE_ROWS);
        elements.push(Element::Line(Line {
            x1: margin,
            y1: y,
            x2: margin + plot_width,
            y2: y,
            class: "chartlet-graticule",
        }));
    }
}

/// The compass rose: a ring, a four-point star, and the one direction worth naming.
fn push_compass(elements: &mut Vec<Element>, center: (f64, f64)) {
    elements.push(Element::Circle(Circle {
        cx: center.0,
        cy: center.1,
        radius: COMPASS_RADIUS,
        class: "chartlet-compass-ring",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    let star = |shape: &[(f64, f64)]| {
        let mut points: Vec<(f64, f64)> = shape
            .iter()
            .map(|(x, y)| (center.0 + x * COMPASS_RADIUS, center.1 + y * COMPASS_RADIUS))
            .collect();
        points.push(points[0]);
        points
    };
    elements.push(Element::Polyline(Polyline {
        points: star(&[
            (0.0, -0.74),
            (0.15, -0.15),
            (0.74, 0.0),
            (0.15, 0.15),
            (0.0, 0.74),
            (-0.15, 0.15),
            (-0.74, 0.0),
            (-0.15, -0.15),
        ]),
        class: "chartlet-compass-needle",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    elements.push(Element::Polyline(Polyline {
        points: star(&[(0.0, -0.74), (0.15, -0.15), (-0.15, -0.15)]),
        class: "chartlet-compass-north",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    elements.push(Element::Text(Text {
        x: center.0,
        y: center.1 - COMPASS_RADIUS - 6.0,
        class: "chartlet-compass-label",
        anchor: TextAnchor::Middle,
        content: "N".to_owned(),
    }));
}

/// The cartouche: a framed plate carrying the heading and the metadata line the specification
/// gave it. Nothing in it is computed, so nothing in it can be out of date with the drawing.
fn push_cartouche(elements: &mut Vec<Element>, cartouche: &Cartouche) {
    let (x, y, width, height) = cartouche.frame;
    for (class, inset) in [
        ("chartlet-cartouche", 0.0),
        ("chartlet-cartouche-frame", 4.0),
    ] {
        elements.push(Element::Rect(Rect {
            x: x + inset,
            y: y + inset,
            width: width - inset * 2.0,
            height: height - inset * 2.0,
            class,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
    elements.push(Element::StyledText(
        Text {
            x: x + CARTOUCHE_PADDING,
            y: y + CARTOUCHE_PADDING + CARTOUCHE_HEADING_SIZE,
            class: "chartlet-cartouche-heading",
            anchor: TextAnchor::Start,
            content: cartouche.heading.clone(),
        },
        TextStyle {
            size: Some(CARTOUCHE_HEADING_SIZE),
            topic: None,
        },
    ));
    elements.push(Element::StyledText(
        Text {
            x: x + CARTOUCHE_PADDING,
            y: y + height - CARTOUCHE_PADDING,
            class: "chartlet-cartouche-meta",
            anchor: TextAnchor::Start,
            content: cartouche.meta.clone(),
        },
        TextStyle {
            size: Some(CARTOUCHE_META_SIZE),
            topic: None,
        },
    ));
}

/// Lays out a topic map. Areas are placed by a phyllotaxis spiral, relaxed by a fixed number of
/// iterations and normalized to the plot; their circles then become coastlines, carrying one
/// point per path through the topic.
///
/// Drawing order follows the design brief: depth lines, routes, halos, filled areas, path points,
/// labels, and the islands last, on top. The sea, the graticule, the compass rose and the
/// cartouche are not drawn yet; they arrive with their own step.
/// One class per realm. `atlas` charts are capped at eight realms, so this covers every one of
/// them, and a host page can address a whole landscape without counting polygons.
const REALM_CLASSES: [&str; 8] = [
    "chartlet-atlas-realm-0",
    "chartlet-atlas-realm-1",
    "chartlet-atlas-realm-2",
    "chartlet-atlas-realm-3",
    "chartlet-atlas-realm-4",
    "chartlet-atlas-realm-5",
    "chartlet-atlas-realm-6",
    "chartlet-atlas-realm-7",
];

/// The knowledge landscape, built up over the steps in `plan/09-wissenslandschaft-atlas.md`.
///
/// Step 6 of 8: the landscape is complete — realms, regions, coast, contour lines, every declared
/// place, and names set where their area has room for them.
fn layout_atlas(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let atlas = spec
        .atlas
        .as_ref()
        .expect("validated atlas charts carry an atlas block");
    let margin = f64::from(PLOT_MARGIN);
    let top = 78.0;
    let plot = (
        margin,
        top,
        f64::from(spec.width) - margin * 2.0,
        f64::from(spec.height) - top - margin,
    );

    let landscape = crate::atlas::landscape(atlas, plot);
    let mut elements = Vec::new();
    push_sea(&mut elements, plot, false);
    push_landscape(&mut elements, atlas, &landscape, spec.number_style());

    push_places(&mut elements, &landscape);
    push_names(&mut elements, atlas, &landscape, metrics, warnings);

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// How large a place is drawn, before its own weight is taken into account.
const PLACE_RADIUS: f64 = 2.2;

/// How far an outline may be pulled straight, as a fraction of a cell. A quarter of a cell is
/// under what the grid could have resolved anyway, and it halves the points a map costs.
const OUTLINE_TOLERANCE: f64 = 0.25;

/// A name already on the map, as the box it takes up.
struct Taken {
    x: f64,
    y: f64,
    half_width: f64,
    half_height: f64,
}

fn clear_of(taken: &[Taken], want: &Taken) -> bool {
    taken.iter().all(|other| {
        (want.x - other.x).abs() >= want.half_width + other.half_width
            || (want.y - other.y).abs() >= want.half_height + other.half_height
    })
}

/// The places, each on the ground of its own region.
fn push_places(elements: &mut Vec<Element>, landscape: &crate::atlas::Landscape) {
    for spot in &landscape.spots {
        let detail = spot
            .place
            .tooltip
            .as_ref()
            .map(|line| format!("{} · {line}", spot.place.label));
        elements.push(Element::Circle(Circle {
            cx: spot.x,
            cy: spot.y,
            radius: PLACE_RADIUS * spot.place.weight.sqrt(),
            class: "chartlet-atlas-place",
            topic: Some(spot.region),
            series_index: None,
            style_index: None,
            tooltip: Some(detail.unwrap_or_else(|| spot.place.label.clone())),
        }));
    }
}

/// Writes one name where its area has the most room and nothing else is written yet.
///
/// The room comes from the distance transform: the point furthest from anything that is not this
/// area is the point where a name covers the least of it. If the first such point is taken, there
/// are three more behind it; if the name will not fit any of them at a readable size, it is left
/// off and said so, because a name spilling over a border is worse than no name.
fn push_name(
    elements: &mut Vec<Element>,
    taken: &mut Vec<Taken>,
    field: &crate::atlas::Field,
    name: (&str, &'static str, Option<usize>),
    room: (f64, f64),
    inside: &dyn Fn(usize, usize) -> bool,
    metrics: &impl TextMetrics,
) -> bool {
    let (text, class, topic) = name;
    let (smallest, largest) = room;
    let depth = crate::contour::depth(field.columns, field.rows, inside);
    let apart = (field.columns / 10).max(3);

    for (column, row, deep) in crate::contour::roomiest(&depth, field.columns, apart, 4) {
        let reach = f64::from(deep) * field.cell;
        let (x, y) = field.centre(column, row);
        let mut size = (reach * 0.7).clamp(smallest, largest);
        while size > smallest && metrics.width(text, size) > reach * 1.8 {
            size -= 1.0;
        }
        if metrics.width(text, size) > reach * 1.8 {
            continue;
        }
        let want = Taken {
            x,
            y,
            half_width: metrics.width(text, size) / 2.0 + 2.0,
            half_height: size * 0.6,
        };
        if !clear_of(taken, &want) {
            continue;
        }
        elements.push(Element::StyledText(
            Text {
                x,
                y: y + size * 0.34,
                class,
                anchor: TextAnchor::Middle,
                content: text.to_owned(),
            },
            TextStyle {
                size: Some(size),
                topic,
            },
        ));
        taken.push(want);
        return true;
    }
    false
}

/// All the names: realms first, because they name the whole landscape, then the regions with the
/// most room to spare, so that the ones with least room are asked last.
fn push_names(
    elements: &mut Vec<Element>,
    atlas: &AtlasSpec,
    landscape: &crate::atlas::Landscape,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) {
    let field = &landscape.field;
    let mut taken: Vec<Taken> = Vec::new();

    for (realm, entry) in atlas.realms.iter().enumerate() {
        push_name(
            elements,
            &mut taken,
            field,
            (&entry.label, "chartlet-atlas-realm-label", None),
            (12.0, 32.0),
            &|column, row| {
                field
                    .at(column, row)
                    .is_some_and(|region| landscape.sites[region].realm == realm)
            },
            metrics,
        );
    }

    let mut held = vec![0usize; landscape.sites.len()];
    for owner in field.owner.iter().flatten() {
        held[*owner] += 1;
    }
    let mut order: Vec<usize> = (0..landscape.sites.len()).collect();
    order.sort_by(|a, b| held[*b].cmp(&held[*a]).then(a.cmp(b)));

    let ranges = crate::atlas::realm_ranges(atlas);
    for region in order {
        let site = &landscape.sites[region];
        let written = push_name(
            elements,
            &mut taken,
            field,
            (&site.region.label, "chartlet-atlas-label", Some(region)),
            (9.0, 19.0),
            &|column, row| field.at(column, row) == Some(region),
            metrics,
        );
        if !written {
            let realm = site.realm;
            warnings.push(ChartWarning::new(
                "label_does_not_fit",
                format!(
                    "/atlas/realms/{realm}/regions/{}/label",
                    region - ranges[realm].start
                ),
                format!(
                    "{:?} has no room for its own name at this size; the area keeps its tooltip",
                    site.region.label
                ),
            ));
        }
    }

    let declared: usize = atlas
        .realms
        .iter()
        .flat_map(|realm| &realm.regions)
        .map(|region| region.places.len())
        .sum();
    if landscape.spots.len() < declared {
        warnings.push(ChartWarning::new(
            "places_did_not_fit",
            "/atlas/realms",
            format!(
                "{} of {declared} places were drawn; a region has more places than it has ground",
                landscape.spots.len()
            ),
        ));
    }
}

/// The landscape as outlines: the realms as shapes, the regions as the finer divisions inside
/// them, and the coast around all of it.
///
/// Every shape comes from the same grid, so they meet exactly. Order is what makes them readable:
/// the realms fill, the region borders are drawn over that fill, and the coast goes on top of
/// both — a coastline that a border crosses stops being a coastline.
fn push_landscape(
    elements: &mut Vec<Element>,
    atlas: &AtlasSpec,
    landscape: &crate::atlas::Landscape,
    style: NumberStyle,
) {
    let field = &landscape.field;
    let corner = |(column, row): (usize, usize)| {
        (
            field.origin.0 - field.cell / 2.0 + count(column) * field.cell,
            field.origin.1 - field.cell / 2.0 + count(row) * field.cell,
        )
    };
    let shapes = |inside: &dyn Fn(usize, usize) -> bool| -> Vec<Vec<(f64, f64)>> {
        crate::contour::rings(field.columns, field.rows, inside)
            .iter()
            .map(|ring| {
                let points: Vec<(f64, f64)> = ring.iter().map(|at| corner(*at)).collect();
                let mut drawn = crate::contour::simplify(
                    &crate::contour::smooth(&crate::contour::straighten(&points)),
                    field.cell * OUTLINE_TOLERANCE,
                );
                // A polyline does not close itself, and these are areas.
                if let Some(first) = drawn.first().copied() {
                    drawn.push(first);
                }
                drawn
            })
            .collect()
    };

    for (realm, entry) in atlas.realms.iter().enumerate() {
        for points in shapes(&|column, row| {
            field
                .at(column, row)
                .is_some_and(|region| landscape.sites[region].realm == realm)
        }) {
            elements.push(Element::Polyline(Polyline {
                points,
                class: REALM_CLASSES[realm],
                topic: None,
                series_index: None,
                style_index: None,
                tooltip: entry.tooltip.clone().or_else(|| Some(entry.label.clone())),
            }));
        }
    }

    // Contour lines between the realm fills and the borders: they are texture, and texture that
    // crosses a border reads as a border of its own.
    if atlas.contours {
        for level in &landscape.relief.levels {
            for points in shapes(&|column, row| {
                let cell = row * field.columns + column;
                field.owner[cell].is_some() && landscape.relief.height[cell] >= *level
            }) {
                elements.push(Element::Polyline(Polyline {
                    points,
                    class: "chartlet-atlas-contour",
                    topic: None,
                    series_index: None,
                    style_index: None,
                    tooltip: None,
                }));
            }
        }
    }

    for (region, site) in landscape.sites.iter().enumerate() {
        for points in shapes(&|column, row| field.at(column, row) == Some(region)) {
            elements.push(Element::Polyline(Polyline {
                points,
                class: "chartlet-atlas-region",
                topic: Some(region),
                series_index: None,
                style_index: None,
                tooltip: Some(site.region.tooltip.clone().unwrap_or_else(|| {
                    format!(
                        "{}: {}",
                        site.region.label,
                        format_value(site.region.value, style)
                    )
                })),
            }));
        }
    }

    for points in shapes(&|column, row| field.at(column, row).is_some()) {
        elements.push(Element::Polyline(Polyline {
            points,
            class: "chartlet-atlas-coast",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
}

fn layout_topicmap(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let topicmap = spec
        .topicmap
        .as_ref()
        .expect("validated topicmap charts carry a topicmap block");
    let margin = f64::from(PLOT_MARGIN);
    let top = 78.0;
    let plot_width = f64::from(spec.width) - margin * 2.0;
    let plot_height = f64::from(spec.height) - top - margin;

    let mut ordered: Vec<(usize, &TopicSpec)> = topicmap.topics.iter().enumerate().collect();
    ordered.sort_by(|a, b| b.1.value.total_cmp(&a.1.value));
    let kept_links = cap_links(topicmap, warnings);
    let plot = (margin, top, plot_width, plot_height);
    let furniture = plan_furniture(topicmap, plot, metrics, warnings);
    let positions =
        topicmap_positions(topicmap, &ordered, &kept_links, plot, &furniture.reserved());

    let (areas, islands) = build_areas(topicmap, &ordered, &positions, spec.number_style());

    // Labels are decided before anything is drawn, because a label that has to sit outside its
    // area needs its leader line laid down underneath the areas.
    let mut inside_labels = Vec::new();
    let mut island_labels = Vec::new();
    let mut outside: Vec<OutsideLabel> = Vec::new();
    for area in &areas {
        outside.extend(push_label(
            &mut inside_labels,
            area,
            plot,
            metrics,
            warnings,
        ));
    }
    for island in &islands {
        outside.extend(push_label(
            &mut island_labels,
            island,
            plot,
            metrics,
            warnings,
        ));
    }
    let outside = settle_outside_labels(outside, plot, warnings);

    let mut elements = vec![Element::Text(Text {
        x: margin,
        y: 30.0,
        class: "chartlet-title",
        anchor: TextAnchor::Start,
        content: fit_text(&spec.title, plot_width, 22.0, metrics, warnings, "/title"),
    })];
    push_sea(&mut elements, plot, topicmap.graticule);
    for area in &areas {
        push_depth_lines(&mut elements, area, topicmap.depth_bands);
    }
    for link in &kept_links {
        if let (Some(from), Some(to)) = (
            positions.get(link.from.as_str()),
            positions.get(link.to.as_str()),
        ) {
            elements.push(Element::Polyline(Polyline {
                points: route(from.center, to.center),
                class: "chartlet-topic-link",
                topic: None,
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
    }
    push_leaders(&mut elements, &outside);
    for area in &areas {
        push_coast(&mut elements, area);
    }
    for area in &areas {
        push_path_points(&mut elements, area, warnings);
    }
    elements.append(&mut inside_labels);
    for island in &islands {
        push_depth_lines(&mut elements, island, topicmap.depth_bands);
        push_coast(&mut elements, island);
        push_path_points(&mut elements, island, warnings);
    }
    elements.append(&mut island_labels);
    push_outside_labels(&mut elements, &outside);
    if let Some(center) = furniture.compass {
        push_compass(&mut elements, center);
    }
    if let Some(cartouche) = &furniture.cartouche {
        push_cartouche(&mut elements, cartouche);
    }

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// The dashed depth lines around an area, the outermost one first.
fn push_depth_lines(elements: &mut Vec<Element>, area: &Area, depth_bands: u8) {
    for band in (1..=u32::from(depth_bands)).rev() {
        elements.push(Element::Polyline(Polyline {
            points: coastline(area, 1.0 + DEPTH_BAND_STEP * f64::from(band)),
            class: "chartlet-topic-band",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
}

/// The coastal halo and the filled area itself, which share one outline.
fn push_coast(elements: &mut Vec<Element>, area: &Area) {
    let points = coastline(area, 1.0);
    elements.push(Element::Polyline(Polyline {
        points: points.clone(),
        class: "chartlet-topic-halo",
        topic: area.index,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    elements.push(Element::Polyline(Polyline {
        points,
        class: "chartlet-topic-area",
        topic: area.index,
        series_index: None,
        style_index: None,
        tooltip: Some(area.tooltip()),
    }));
}

/// One point per path through the topic, spread over the inner disc by the same golden angle the
/// spiral uses, so they never settle into rows.
///
/// An area only has room for so many before they merge into a smudge; beyond that the count is
/// reported rather than drawn, and the data table keeps carrying the exact number.
fn push_path_points(elements: &mut Vec<Element>, area: &Area, warnings: &mut Vec<ChartWarning>) {
    let usable = area.radius() * PATH_POINT_DISC;
    let drawn = (1..=area.topic.points)
        .take_while(|points| usable / f64::from(*points).sqrt() >= PATH_POINT_SPACING)
        .count();
    if drawn < usize::try_from(area.topic.points).expect("a path count fits in a usize") {
        warnings.push(ChartWarning::new(
            "dense_chart",
            format!("{}/points", area.path),
            format!(
                "{} path points do not stay apart in an area this size; {drawn} are drawn and the data table keeps the count",
                area.topic.points
            ),
        ));
    }
    let phase = noise::value_noise(0.5, 0.5, area.placed.seed) * std::f64::consts::TAU;
    for point in 0..drawn {
        let angle = count(point) * SPIRAL_ANGLE + phase;
        let distance = usable * ((count(point) + 0.5) / count(drawn)).sqrt();
        let (sin, cos) = angle.sin_cos();
        elements.push(Element::Circle(Circle {
            cx: area.center().0 + cos * distance,
            cy: area.center().1 + sin * distance,
            radius: PATH_POINT_RADIUS,
            class: "chartlet-topic-point",
            topic: area.index,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
}

/// A name and count that did not fit inside their area and were moved out beside it.
struct OutsideLabel {
    name: String,
    value: String,
    path: String,
    /// The area this label names, see [`Area::index`].
    topic: Option<usize>,
    /// What the area is worth, so the least important labels are the ones dropped when a side
    /// runs out of room.
    weight: f64,
    /// `End` when the label sits to the left of its area, so the text grows away from the map.
    anchor: TextAnchor,
    x: f64,
    y: f64,
    center: (f64, f64),
}

/// The size an area's name is drawn at: as large as the area can carry, within the range the
/// design brief fixes.
fn label_size(radius: f64) -> f64 {
    (radius * TOPIC_LABEL_RATIO).clamp(TOPIC_LABEL_MIN, TOPIC_LABEL_MAX)
}

/// Draws the area's name and count inside it, or hands back a label to be placed outside.
///
/// The fit is measured against the largest circle that fits inside this coastline, so a bay in
/// the coast cannot cut a corner off the text. A label that has to move out is reported as
/// `label_outside_area` — it is not a failure, but it is a fact about the drawing that the
/// caller of chartlet can act on.
fn push_label(
    elements: &mut Vec<Element>,
    area: &Area,
    plot: (f64, f64, f64, f64),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Option<OutsideLabel> {
    let value = format_value(area.topic.value, area.style);
    let size = label_size(area.radius());
    let inscribed = area.radius()
        * area
            .placed
            .profile
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
    let name_fits = fits_inside(
        metrics.width(&area.topic.label, size),
        size * 0.8,
        inscribed,
    );
    let value_fits = fits_inside(
        metrics.width(&value, TOPIC_VALUE_SIZE),
        TOPIC_VALUE_DROP + TOPIC_VALUE_SIZE * 0.3,
        inscribed,
    );
    if !name_fits || !value_fits {
        warnings.push(ChartWarning::new(
            "label_outside_area",
            format!("{}/label", area.path),
            "the label did not fit inside the area and was placed beside it, with a leader line",
        ));
        return Some(outside_label(area, plot, value, metrics));
    }
    elements.push(Element::StyledText(
        Text {
            x: area.center().0,
            y: area.center().1,
            class: "chartlet-topic-label",
            anchor: TextAnchor::Middle,
            content: area.topic.label.clone(),
        },
        TextStyle {
            size: Some(size),
            topic: area.index,
        },
    ));
    elements.push(Element::StyledText(
        Text {
            x: area.center().0,
            y: area.center().1 + TOPIC_VALUE_DROP,
            class: "chartlet-topic-value",
            anchor: TextAnchor::Middle,
            content: value,
        },
        TextStyle {
            size: Some(TOPIC_VALUE_SIZE),
            topic: area.index,
        },
    ));
    None
}

/// Parks a label beside its area, clear of the outermost depth line.
///
/// It goes on the side facing the nearer edge of the map, which keeps it out of the crowded
/// middle — unless the text would then run off the canvas, in which case it goes on the other
/// side instead. The stacking pass afterwards is what settles the final height.
fn outside_label(
    area: &Area,
    plot: (f64, f64, f64, f64),
    value: String,
    metrics: &impl TextMetrics,
) -> OutsideLabel {
    let (margin, _, plot_width, _) = plot;
    let width = metrics
        .width(&area.topic.label, TOPIC_OUTSIDE_SIZE)
        .max(metrics.width(&value, TOPIC_OUTSIDE_SIZE));
    let widest = area.placed.profile.iter().copied().fold(0.0_f64, f64::max);
    let clearance = area.radius() * widest + HALO_WIDTH / 2.0 + LABEL_PADDING * 2.0;

    let fits_right = area.center().0 + clearance + width <= margin + plot_width;
    let fits_left = area.center().0 - clearance - width >= margin;
    let to_right = if area.center().0 >= margin + plot_width / 2.0 {
        fits_right || !fits_left
    } else {
        !fits_left && fits_right
    };
    // The clamp only bites when neither side has room for the text. A label that reaches over its
    // own coast still reads; a label half off the canvas does not.
    let x = if to_right {
        (area.center().0 + clearance).min(margin + plot_width - width)
    } else {
        (area.center().0 - clearance).max(margin + width)
    };

    OutsideLabel {
        name: area.topic.label.clone(),
        value,
        path: area.path.clone(),
        topic: area.index,
        weight: area.topic.value,
        anchor: if to_right {
            TextAnchor::Start
        } else {
            TextAnchor::End
        },
        x,
        y: area.center().1,
        center: area.center(),
    }
}

/// Whether one line of text, centered on the area and reaching `drop` pixels from its center,
/// keeps all four of its corners inside a circle of `inscribed` radius.
fn fits_inside(width: f64, drop: f64, inscribed: f64) -> bool {
    (width / 2.0 + LABEL_PADDING).hypot(drop) <= inscribed
}

/// Settles the labels that were moved out of their areas. Each side of the map is a column, and
/// two labels in one column may not sit closer than a label's own height.
fn settle_outside_labels(
    labels: Vec<OutsideLabel>,
    plot: (f64, f64, f64, f64),
    warnings: &mut Vec<ChartWarning>,
) -> Vec<OutsideLabel> {
    let (_, top, _, plot_height) = plot;
    let (left, right): (Vec<_>, Vec<_>) = labels
        .into_iter()
        .partition(|label| matches!(label.anchor, TextAnchor::End));
    let mut settled = settle_column(left, top, plot_height, warnings);
    settled.extend(settle_column(right, top, plot_height, warnings));
    settled
}

/// Spreads one column of labels so none overlaps the next, inside the plot. A column that cannot
/// hold them all keeps the largest areas' labels and reports the rest: a label drawn over another
/// label carries less than no information.
fn settle_column(
    mut labels: Vec<OutsideLabel>,
    top: f64,
    plot_height: f64,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<OutsideLabel> {
    // Room for the name's ascender at the top and for the count's descender at the bottom: a
    // settled label has to stay inside the plot as a whole block.
    let first = top + TOPIC_OUTSIDE_SIZE;
    let last = top + plot_height - TOPIC_OUTSIDE_DROP - TOPIC_OUTSIDE_SIZE * 0.3;
    let capacity = (1..=labels.len())
        .take_while(|slots| count(*slots - 1) * TOPIC_OUTSIDE_SPACING <= last - first)
        .count();
    if labels.len() > capacity {
        labels.sort_by(|a, b| b.weight.total_cmp(&a.weight));
        for dropped in labels.split_off(capacity) {
            warnings.push(ChartWarning::new(
                "dense_chart",
                format!("{}/label", dropped.path),
                "there is no room beside the map for this label; the data table keeps the name",
            ));
        }
    }

    labels.sort_by(|a, b| a.y.total_cmp(&b.y));
    let mut lowest = first;
    for label in &mut labels {
        label.y = label.y.max(lowest);
        lowest = label.y + TOPIC_OUTSIDE_SPACING;
    }
    let mut highest = last;
    for label in labels.iter_mut().rev() {
        label.y = label.y.min(highest);
        highest = label.y - TOPIC_OUTSIDE_SPACING;
    }
    labels
}

/// The line from an outside label to the middle of the area it belongs to.
///
/// Drawn before the areas themselves: the fill covers everything but the stretch over open sea,
/// so the line always reaches the coast and never crosses it.
fn push_leaders(elements: &mut Vec<Element>, labels: &[OutsideLabel]) {
    for label in labels {
        elements.push(Element::Line(Line {
            x1: label.x,
            // Between the two lines of the label rather than on the name's baseline.
            y1: label.y + TOPIC_OUTSIDE_DROP / 2.0 - TOPIC_OUTSIDE_SIZE * 0.35,
            x2: label.center.0,
            y2: label.center.1,
            class: "chartlet-topic-leader",
        }));
    }
}

fn push_outside_labels(elements: &mut Vec<Element>, labels: &[OutsideLabel]) {
    for label in labels {
        elements.push(Element::StyledText(
            Text {
                x: label.x,
                y: label.y,
                class: "chartlet-topic-outside",
                anchor: label.anchor,
                content: label.name.clone(),
            },
            TextStyle {
                size: Some(TOPIC_OUTSIDE_SIZE),
                topic: label.topic,
            },
        ));
        elements.push(Element::StyledText(
            Text {
                x: label.x,
                y: label.y + TOPIC_OUTSIDE_DROP,
                class: "chartlet-topic-outside-value",
                anchor: label.anchor,
                content: label.value.clone(),
            },
            TextStyle {
                size: Some(TOPIC_OUTSIDE_SIZE),
                topic: label.topic,
            },
        ));
    }
}

/// Title, value grid, value ticks and time ticks, shared by every pane.
/// Draws one polyline per layer, plus markers and value labels while the observations stay far
/// enough apart for them to be readable.
/// Everything a pane needs to map an observation onto its plot.
pub(crate) struct TimeFrame {
    plot: PlotArea,
    span: (i64, i64),
    pub(crate) zone: TimeZone,
    pub(crate) precision: Precision,
    scale: NumericScale,
    /// How the pane writes its values: its own value axis on a time chart, the shared one in
    /// small multiples.
    pub(crate) style: NumberStyle,
}

impl TimeFrame {
    pub(crate) fn x(&self, epoch: i64) -> f64 {
        time_x(epoch, self.span, self.plot)
    }

    pub(crate) fn y(&self, value: f64) -> f64 {
        self.scale
            .map(value, self.plot.top + self.plot.height, self.plot.top)
    }
}

/// How much a pane draws besides its lines: a time chart labels its values, a small-multiples
/// panel is too small for that and keeps smaller markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Detail {
    Full,
    Compact,
}

/// The value scale of one pane of a time chart, or of all small multiples when `pane` is `None`:
/// every observation, every candle's high and low, every band edge, every horizontal reference
/// line, every point marker and every zone edge, so that none of them falls outside the plot. An
/// area is filled down to zero, so a pane with one always shows zero.
fn time_scale(spec: &ChartSpec, zone: TimeZone, pane: Option<usize>) -> NumericScale {
    let inside = |pane_index: usize| pane.is_none_or(|pane| pane == pane_index);
    let layers = || {
        spec.indexed_layers()
            .filter(|entry| inside(entry.pane))
            .map(|entry| entry.layer)
    };
    let mut values = Vec::new();
    for entry in spec.data_layers().filter(|entry| inside(entry.pane)) {
        values.extend(
            entry
                .layer
                .resolved_points(zone)
                .into_iter()
                .map(|(_, value)| value),
        );
        for (_, lower, upper) in entry.layer.resolved_band(zone) {
            values.push(lower);
            values.push(upper);
        }
        for (_, [_, high, low, _]) in entry.layer.resolved_candles(zone) {
            values.push(high);
            values.push(low);
        }
    }
    values.extend(
        layers()
            .filter(|layer| layer.mark == Mark::Annotation)
            .filter_map(|layer| layer.value),
    );
    values.extend(
        layers()
            .filter(|layer| layer.mark == Mark::Band)
            .flat_map(|layer| [layer.bottom, layer.top])
            .flatten(),
    );
    let area = layers().any(|layer| layer.mark == Mark::Area);
    NumericScale::from_values(values.into_iter(), area)
}

/// The name a tooltip gives a layer: in small multiples the panel title comes first.
pub(crate) fn tooltip_name(spec: &ChartSpec, entry: LayerRef) -> Option<String> {
    let title = if spec.chart_type == ChartType::Multiples {
        spec.panes[entry.pane].title.clone()
    } else {
        None
    };
    match (title, &entry.layer.name) {
        (Some(title), Some(name)) => Some(format!("{title} · {name}")),
        (Some(title), None) => Some(title),
        (None, name) => name.clone(),
    }
}

/// Draws the layers of one pane: zones first, filled areas and uncertainty bands over them,
/// reference lines over those, then the lines with their markers, the point markers, and the
/// labels of every annotation last so nothing covers them.
fn draw_pane(
    spec: &ChartSpec,
    pane: usize,
    frame: &TimeFrame,
    detail: Detail,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let entries: Vec<LayerRef> = spec
        .indexed_layers()
        .filter(|entry| entry.pane == pane)
        .collect();
    let mut space = LabelSpace::new(frame, &entries);
    let mut rule_labels = Vec::new();

    for entry in entries
        .iter()
        .filter(|entry| entry.layer.mark == Mark::Band)
    {
        push_zone(
            spec,
            *entry,
            frame,
            elements,
            &mut rule_labels,
            &mut space,
            warnings,
            metrics,
        );
    }
    for entry in entries
        .iter()
        .filter(|entry| entry.layer.mark == Mark::Area)
    {
        push_area(spec, *entry, frame, elements);
    }
    for entry in entries
        .iter()
        .filter(|entry| entry.layer.is_data() && entry.layer.has_band())
    {
        push_band(spec, *entry, frame, elements);
    }

    for entry in entries.iter().filter(|entry| entry.layer.is_rule()) {
        push_rule(
            *entry,
            frame,
            elements,
            &mut rule_labels,
            &mut space,
            warnings,
            metrics,
        );
    }

    // Candles go below the lines, so that a moving average stays readable across them.
    for entry in entries
        .iter()
        .filter(|entry| entry.layer.mark == Mark::Ohlc)
    {
        crate::ohlc::push_candles(spec, *entry, frame, elements);
    }
    for entry in entries
        .iter()
        .filter(|entry| entry.layer.is_data() && entry.layer.mark != Mark::Ohlc)
    {
        push_line(spec, *entry, frame, detail, elements);
    }

    let markers: Vec<LayerRef> = entries
        .iter()
        .filter(|entry| entry.layer.is_marker())
        .copied()
        .collect();
    let radius = if detail == Detail::Full { 6.0 } else { 4.5 };
    for entry in &markers {
        push_marker(*entry, frame, radius, elements, &mut space);
    }
    for entry in &markers {
        push_marker_label(
            *entry,
            frame,
            radius,
            &mut rule_labels,
            &mut space,
            warnings,
            metrics,
        );
    }
    elements.extend(rule_labels);
}

/// A label's box on the canvas: left, top, right and bottom edge.
type LabelBox = (f64, f64, f64, f64);

/// What the label of an annotation has to keep clear of: the plot's edges, the labels placed
/// before it, the data lines of its pane and the point marker symbols.
struct LabelSpace {
    plot: PlotArea,
    labels: Vec<LabelBox>,
    lines: Vec<Vec<(f64, f64)>>,
    symbols: Vec<LabelBox>,
}

/// Why a label collides, if it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Collision {
    Outside,
    Label,
    Line,
}

impl LabelSpace {
    /// The data lines of a pane as drawn, but only when the pane carries an annotation whose
    /// label could run into them.
    fn new(frame: &TimeFrame, entries: &[LayerRef]) -> Self {
        let annotated = entries.iter().any(|entry| !entry.layer.is_data());
        let lines = if annotated {
            // A candle's wick spans all of it, from high to low.
            let wicks = entries.iter().flat_map(|entry| {
                entry.layer.resolved_candles(frame.zone).into_iter().map(
                    |(epoch, [_, high, low, _])| {
                        let x = frame.x(epoch);
                        vec![(x, frame.y(high)), (x, frame.y(low))]
                    },
                )
            });
            entries
                .iter()
                .filter(|entry| entry.layer.is_data())
                .flat_map(|entry| entry.layer.resolved_segments(frame.zone))
                .map(|segment| {
                    segment
                        .iter()
                        .map(|(epoch, value)| (frame.x(*epoch), frame.y(*value)))
                        .collect()
                })
                .chain(wicks)
                .collect()
        } else {
            Vec::new()
        };
        Self {
            plot: frame.plot,
            labels: Vec::new(),
            lines,
            symbols: Vec::new(),
        }
    }

    fn outside(&self, label: LabelBox) -> bool {
        let plot = self.plot;
        label.0 < plot.left
            || label.2 > plot.left + plot.width
            || label.1 < plot.top
            || label.3 > plot.top + plot.height
    }

    fn hits_label(&self, label: LabelBox) -> bool {
        self.labels.iter().any(|other| boxes_overlap(label, *other))
    }

    fn hits_line(&self, label: LabelBox) -> bool {
        self.lines.iter().any(|line| {
            line.windows(2)
                .any(|pair| segment_hits_box(pair[0], pair[1], label))
        })
    }

    fn hits_symbol(&self, label: LabelBox) -> bool {
        self.symbols
            .iter()
            .any(|other| boxes_overlap(label, *other))
    }

    /// The first collision of a label, checked against the data lines only when `lines` asks for
    /// it.
    fn collision(&self, label: LabelBox, lines: bool) -> Option<Collision> {
        if self.outside(label) {
            Some(Collision::Outside)
        } else if self.hits_label(label) {
            Some(Collision::Label)
        } else if lines && self.hits_line(label) {
            Some(Collision::Line)
        } else {
            None
        }
    }

    /// Takes the room of a label that stays where it is, and reports whatever it collides with.
    fn place(
        &mut self,
        text: &Text,
        entry: LayerRef,
        lines: bool,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
    ) {
        let label = label_box(text, metrics);
        if let Some(collision) = self.collision(label, lines) {
            warn_label_overlap(entry, collision, warnings);
        }
        self.labels.push(label);
    }
}

/// The box a label takes: its measured width on the side its anchor points to, and the height of
/// a line of text around its baseline.
fn label_box(text: &Text, metrics: &impl TextMetrics) -> LabelBox {
    let width = metrics.width(&text.content, LABEL_SIZE);
    let left = match text.anchor {
        TextAnchor::Start => text.x,
        TextAnchor::Middle => text.x - width / 2.0,
        TextAnchor::End => text.x - width,
    };
    (left, text.y - 9.0, left + width, text.y + 3.0)
}

fn boxes_overlap(a: LabelBox, b: LabelBox) -> bool {
    a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
}

/// Whether a straight piece of line from `a` to `b` passes through a box, by clipping it against
/// the box's edges.
fn segment_hits_box(a: (f64, f64), b: (f64, f64), area: LabelBox) -> bool {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let (mut start, mut end) = (0.0_f64, 1.0_f64);
    for (p, q) in [
        (-dx, a.0 - area.0),
        (dx, area.2 - a.0),
        (-dy, a.1 - area.1),
        (dy, area.3 - a.1),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                start = start.max(t);
            } else {
                end = end.min(t);
            }
            if start > end {
                return false;
            }
        }
    }
    true
}

fn warn_label_overlap(entry: LayerRef, collision: Collision, warnings: &mut Vec<ChartWarning>) {
    let message = match collision {
        Collision::Outside => {
            "the label reaches outside the plot area; shorten it or move the annotation"
        }
        Collision::Label => {
            "the label overlaps the label of another annotation; shorten one of them or move it"
        }
        Collision::Line => "the label crosses a data line; shorten it or move the reference line",
    };
    warnings.push(ChartWarning::new(
        "label_overlap",
        format!("/panes/{}/layers/{}", entry.pane, entry.local),
        message,
    ));
}

/// Where a zone lies, for its tooltip and the description: its span of time, its range of
/// values, or both.
pub(crate) fn zone_extent(
    spec: &ChartSpec,
    layer: &LayerSpec,
    zone: TimeZone,
    style: NumberStyle,
) -> String {
    let resolve = |time: &Option<crate::time::TimeValue>| {
        time.as_ref().and_then(|time| time.resolve(zone).ok())
    };
    let (from, to) = (resolve(&layer.from), resolve(&layer.to));
    let epochs: Vec<i64> = from.into_iter().chain(to).collect();
    let times = if epochs.is_empty() {
        None
    } else {
        let precision = Precision::of(epochs.into_iter(), zone);
        crate::text::zone_times(
            spec.locale,
            from.map(|epoch| precision.format(epoch, zone)).as_deref(),
            to.map(|epoch| precision.format(epoch, zone)).as_deref(),
        )
    };
    let show = |value: f64| format_value(value, style);
    let values = crate::text::zone_values(
        spec.locale,
        layer.bottom.map(show).as_deref(),
        layer.top.map(show).as_deref(),
    );
    [times, values]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ")
}

/// Where a point marker sits, for its tooltip and the description: its time and its value.
pub(crate) fn marker_position(layer: &LayerSpec, zone: TimeZone, style: NumberStyle) -> String {
    let epoch = layer
        .time
        .as_ref()
        .and_then(|time| time.resolve(zone).ok())
        .expect("validated point markers carry a time");
    let value = layer.value.expect("validated point markers carry a value");
    format!(
        "{}, {}",
        Precision::of(std::iter::once(epoch), zone).format(epoch, zone),
        format_value(value, style)
    )
}

/// A zone: a shaded rectangle behind the data between its edges, each missing edge taken from
/// the plot. Its label sits inside the zone's top left corner when the zone is tall enough, and
/// just outside its upper or lower edge when it is not.
#[allow(clippy::too_many_arguments)]
fn push_zone(
    spec: &ChartSpec,
    entry: LayerRef,
    frame: &TimeFrame,
    elements: &mut Vec<Element>,
    labels: &mut Vec<Element>,
    space: &mut LabelSpace,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let layer = entry.layer;
    let label = layer
        .label
        .as_deref()
        .expect("validated zones carry a label");
    let plot = frame.plot;
    let resolve = |time: &Option<crate::time::TimeValue>| {
        time.as_ref().and_then(|time| time.resolve(frame.zone).ok())
    };
    let left = resolve(&layer.from).map_or(plot.left, |epoch| frame.x(epoch));
    let right = resolve(&layer.to).map_or(plot.left + plot.width, |epoch| frame.x(epoch));
    let top = layer.top.map_or(plot.top, |value| frame.y(value));
    let bottom = layer
        .bottom
        .map_or(plot.top + plot.height, |value| frame.y(value));
    elements.push(Element::Rect(Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
        class: "chartlet-zone",
        series_index: None,
        style_index: layer.resolved_color().is_some().then_some(entry.global),
        tooltip: Some(format!(
            "{label}: {}",
            zone_extent(spec, layer, frame.zone, frame.style)
        )),
    }));

    let content = fit_text(
        label,
        plot.width * 0.4,
        LABEL_SIZE,
        metrics,
        warnings,
        &format!("/panes/{}/layers/{}/label", entry.pane, entry.local),
    );
    let width = metrics.width(&content, LABEL_SIZE);
    // A zone at the right edge takes its label on its right end, so the label stays inside.
    let (x, anchor) = if left + 4.0 + width > plot.left + plot.width {
        (right - 4.0, TextAnchor::End)
    } else {
        (left + 4.0, TextAnchor::Start)
    };
    let y = if bottom - top >= 20.0 {
        top + 14.0
    } else if top - 17.0 >= plot.top {
        top - 5.0
    } else {
        bottom + 13.0
    };
    let text = Text {
        x,
        y,
        class: "chartlet-rule-label",
        anchor,
        content,
    };
    space.place(&text, entry, true, metrics, warnings);
    labels.push(Element::Text(text));
}

/// The symbol of a point marker, in its declared shape. Shape and label carry its meaning, so a
/// marker never depends on its color alone.
fn push_marker(
    entry: LayerRef,
    frame: &TimeFrame,
    radius: f64,
    elements: &mut Vec<Element>,
    space: &mut LabelSpace,
) {
    let layer = entry.layer;
    let (x, y) = marker_point(entry, frame);
    let label = layer
        .label
        .as_deref()
        .expect("validated point markers carry a label");
    let style_index = layer.resolved_color().is_some().then_some(entry.global);
    let tooltip = Some(format!(
        "{label}: {}",
        marker_position(layer, frame.zone, frame.style)
    ));
    let reach = radius * 1.25;
    space
        .symbols
        .push((x - reach, y - reach, x + reach, y + reach));
    let outline = |corners: &[(f64, f64)]| -> Vec<(f64, f64)> {
        let mut points: Vec<(f64, f64)> = corners
            .iter()
            .map(|(dx, dy)| (x + dx * radius, y + dy * radius))
            .collect();
        points.push(points[0]);
        points
    };
    let points = match layer.marker_shape() {
        Shape::Circle => {
            elements.push(Element::Circle(Circle {
                cx: x,
                cy: y,
                radius,
                class: "chartlet-marker",
                topic: None,
                series_index: None,
                style_index,
                tooltip,
            }));
            return;
        }
        Shape::Square => outline(&[(-0.9, -0.9), (0.9, -0.9), (0.9, 0.9), (-0.9, 0.9)]),
        Shape::Diamond => outline(&[(0.0, -1.25), (1.25, 0.0), (0.0, 1.25), (-1.25, 0.0)]),
        Shape::TriangleUp => outline(&[(0.0, -1.25), (1.15, 0.8), (-1.15, 0.8)]),
        Shape::TriangleDown => outline(&[(0.0, 1.25), (1.15, -0.8), (-1.15, -0.8)]),
    };
    elements.push(Element::Polyline(Polyline {
        points,
        class: "chartlet-marker",
        topic: None,
        series_index: None,
        style_index,
        tooltip,
    }));
}

/// Where a point marker sits on the plot.
fn marker_point(entry: LayerRef, frame: &TimeFrame) -> (f64, f64) {
    let layer = entry.layer;
    let epoch = layer
        .time
        .as_ref()
        .and_then(|time| time.resolve(frame.zone).ok())
        .expect("validated point markers carry a time");
    let value = layer.value.expect("validated point markers carry a value");
    (frame.x(epoch), frame.y(value))
}

/// The label of a point marker. It tries the right of the symbol first, then the left, above and
/// below, and takes the first place that stays inside the plot and clear of other labels, data
/// lines and markers; failing that, the first one that at least stays inside and clear of other
/// labels. Only when no place works is the label kept on the right and reported.
fn push_marker_label(
    entry: LayerRef,
    frame: &TimeFrame,
    radius: f64,
    labels: &mut Vec<Element>,
    space: &mut LabelSpace,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let (x, y) = marker_point(entry, frame);
    let content = fit_text(
        entry
            .layer
            .label
            .as_deref()
            .expect("validated point markers carry a label"),
        frame.plot.width * 0.4,
        LABEL_SIZE,
        metrics,
        warnings,
        &format!("/panes/{}/layers/{}/label", entry.pane, entry.local),
    );
    let gap = radius * 1.25 + 4.0;
    let candidates = [
        (x + gap, y + 4.0, TextAnchor::Start),
        (x - gap, y + 4.0, TextAnchor::End),
        (x, y - gap - 3.0, TextAnchor::Middle),
        (x, y + gap + 10.0, TextAnchor::Middle),
    ]
    .map(|(x, y, anchor)| Text {
        x,
        y,
        class: "chartlet-rule-label",
        anchor,
        content: content.clone(),
    });
    let boxes = candidates.each_ref().map(|text| label_box(text, metrics));
    let clear = boxes
        .iter()
        .position(|label| space.collision(*label, true).is_none() && !space.hits_symbol(*label));
    let readable = || {
        boxes
            .iter()
            .position(|label| space.collision(*label, false).is_none())
    };
    let chosen = clear.or_else(readable);
    let index = chosen.unwrap_or(0);
    if chosen.is_none()
        && let Some(collision) = space.collision(boxes[0], false)
    {
        warn_label_overlap(entry, collision, warnings);
    }
    space.labels.push(boxes[index]);
    let text = candidates
        .into_iter()
        .nth(index)
        .expect("the index comes from the candidates");
    labels.push(Element::Text(text));
}

/// The region between an area layer's line and zero, one closed outline per run of values: along
/// the line and back along the zero line. It takes the layer's color at the band's low opacity;
/// the line itself is drawn on top with the other lines.
fn push_area(spec: &ChartSpec, entry: LayerRef, frame: &TimeFrame, elements: &mut Vec<Element>) {
    let explicit = entry.layer.resolved_color().is_some();
    let palette = spec.palette_index(entry.layer);
    let base = frame.y(0.0);
    for segment in entry.layer.resolved_segments(frame.zone) {
        if segment.len() < 2 {
            continue;
        }
        let (first, last) = (segment[0].0, segment[segment.len() - 1].0);
        let mut outline: Vec<(f64, f64)> = segment
            .iter()
            .map(|(epoch, value)| (frame.x(*epoch), frame.y(*value)))
            .collect();
        outline.push((frame.x(last), base));
        outline.push((frame.x(first), base));
        elements.push(Element::Polyline(Polyline {
            points: outline,
            class: if explicit {
                "chartlet-area"
            } else {
                AREA_CLASSES[palette]
            },
            topic: None,
            series_index: None,
            style_index: explicit.then_some(entry.global),
            tooltip: None,
        }));
    }
}

/// The band of one layer as closed outlines, one per run of values between missing ones: along
/// the upper edge and back along the lower one. A modeled band gets a hatched copy on top, so
/// that it reads as modeled without relying on color.
fn push_band(spec: &ChartSpec, entry: LayerRef, frame: &TimeFrame, elements: &mut Vec<Element>) {
    let mut runs = vec![Vec::new()];
    for point in &entry.layer.points {
        let Ok(epoch) = point.time.resolve(frame.zone) else {
            continue;
        };
        match (point.lower, point.upper) {
            (Some(lower), Some(upper)) if point.value.is_some() => runs
                .last_mut()
                .expect("there is always a current run")
                .push((epoch, lower, upper)),
            _ => runs.push(Vec::new()),
        }
    }
    for band in runs.into_iter().filter(|run| !run.is_empty()) {
        push_band_run(spec, entry, frame, &band, elements);
    }
}

/// One closed band outline, see [`push_band`].
fn push_band_run(
    spec: &ChartSpec,
    entry: LayerRef,
    frame: &TimeFrame,
    band: &[(i64, f64, f64)],
    elements: &mut Vec<Element>,
) {
    let mut outline: Vec<(f64, f64)> = band
        .iter()
        .map(|(epoch, _, upper)| (frame.x(*epoch), frame.y(*upper)))
        .collect();
    outline.extend(
        band.iter()
            .rev()
            .map(|(epoch, lower, _)| (frame.x(*epoch), frame.y(*lower))),
    );
    let explicit = entry.layer.resolved_color().is_some();
    let palette = spec.palette_index(entry.layer);
    if entry.layer.modeled {
        elements.push(Element::Polyline(Polyline {
            points: outline.clone(),
            class: if explicit {
                "chartlet-band"
            } else {
                BAND_CLASSES[palette]
            },
            topic: None,
            series_index: None,
            style_index: explicit.then_some(entry.global),
            tooltip: None,
        }));
        elements.push(Element::Polyline(Polyline {
            points: outline,
            class: "chartlet-hatch",
            topic: None,
            series_index: None,
            style_index: Some(entry.global),
            tooltip: None,
        }));
    } else {
        elements.push(Element::Polyline(Polyline {
            points: outline,
            class: if explicit {
                "chartlet-band"
            } else {
                BAND_CLASSES[palette]
            },
            topic: None,
            series_index: None,
            style_index: explicit.then_some(entry.global),
            tooltip: None,
        }));
    }
}

/// A reference line across the plot: horizontal at a value, vertical at a time. Its label is
/// collected separately and drawn after the data; it stays where it is, and a collision with the
/// plot's edges, another label or a data line is reported.
fn push_rule(
    entry: LayerRef,
    frame: &TimeFrame,
    elements: &mut Vec<Element>,
    labels: &mut Vec<Element>,
    space: &mut LabelSpace,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let layer = entry.layer;
    let label = layer
        .label
        .as_deref()
        .expect("validated annotations carry a label");
    let path = format!("/panes/{}/layers/{}/label", entry.pane, entry.local);
    let plot = frame.plot;
    let bottom = plot.top + plot.height;
    let (points, text, tooltip) = if let Some(value) = layer.value {
        let y = frame.y(value);
        (
            vec![(plot.left, y), (plot.left + plot.width, y)],
            Text {
                x: plot.left + 4.0,
                y: y - 6.0,
                class: "chartlet-rule-label",
                anchor: TextAnchor::Start,
                content: fit_text(
                    label,
                    plot.width / 2.0,
                    LABEL_SIZE,
                    metrics,
                    warnings,
                    &path,
                ),
            },
            format!("{label}: {}", format_value(value, frame.style)),
        )
    } else {
        let epoch = layer
            .time
            .as_ref()
            .and_then(|time| time.resolve(frame.zone).ok())
            .expect("validated annotations carry a value or a time");
        let x = frame.x(epoch);
        // A line in the right part of the plot takes its label on its left, so the label stays
        // inside the chart.
        let (label_x, anchor) = if x > plot.left + plot.width * 0.7 {
            (x - 4.0, TextAnchor::End)
        } else {
            (x + 4.0, TextAnchor::Start)
        };
        let precision = Precision::of(std::iter::once(epoch), frame.zone);
        (
            vec![(x, plot.top), (x, bottom)],
            Text {
                x: label_x,
                y: plot.top + 12.0,
                class: "chartlet-rule-label",
                anchor,
                content: fit_text(
                    label,
                    plot.width * 0.4,
                    LABEL_SIZE,
                    metrics,
                    warnings,
                    &path,
                ),
            },
            format!("{label}: {}", precision.format(epoch, frame.zone)),
        )
    };
    elements.push(Element::Polyline(Polyline {
        points,
        class: "chartlet-rule",
        topic: None,
        series_index: None,
        style_index: layer.resolved_color().is_some().then_some(entry.global),
        tooltip: Some(tooltip),
    }));
    space.place(&text, entry, true, metrics, warnings);
    labels.push(Element::Text(text));
}

/// One line, broken at every missing value, with its markers and, in a full-size chart, its
/// value labels.
fn push_line(
    spec: &ChartSpec,
    entry: LayerRef,
    frame: &TimeFrame,
    detail: Detail,
    elements: &mut Vec<Element>,
) {
    let layer = entry.layer;
    let points = layer.resolved_points(frame.zone);
    let explicit = layer.resolved_color().is_some();
    let palette = spec.palette_index(layer);
    let class = line_class(spec, entry);
    // A missing value breaks the line; a lone value between two gaps keeps only its marker.
    for segment in layer.resolved_segments(frame.zone) {
        if segment.len() < 2 {
            continue;
        }
        elements.push(Element::Polyline(Polyline {
            points: segment
                .iter()
                .map(|(epoch, value)| (frame.x(*epoch), frame.y(*value)))
                .collect(),
            class,
            topic: None,
            series_index: None,
            style_index: explicit.then_some(entry.global),
            tooltip: None,
        }));
    }

    // Markers and their labels are only drawn while the observations stay far enough apart
    // for them to be readable.
    if points.len() > MAX_TIME_MARKERS {
        return;
    }
    let band = layer.resolved_band(frame.zone);
    let name = tooltip_name(spec, entry);
    for (index, (epoch, value)) in points.iter().enumerate() {
        let x = frame.x(*epoch);
        let y = frame.y(*value);
        let mut text = tooltip(
            &frame.precision.format(*epoch, frame.zone),
            *value,
            frame.style,
            name.as_deref(),
        );
        if let Some((_, lower, upper)) = band.get(index) {
            write!(
                text,
                " ({} {} {} {})",
                spec.locale.words().range,
                format_value(*lower, frame.style),
                spec.locale.words().to,
                format_value(*upper, frame.style)
            )
            .expect("writing to String cannot fail");
        }
        elements.push(Element::Circle(Circle {
            cx: x,
            cy: y,
            radius: if detail == Detail::Full { 4.0 } else { 2.5 },
            class: if explicit {
                "chartlet-point"
            } else {
                POINT_CLASSES[palette]
            },
            topic: None,
            series_index: None,
            style_index: explicit.then_some(entry.global),
            tooltip: Some(text),
        }));
        if detail == Detail::Full && spec.show_values {
            elements.push(Element::Text(Text {
                x,
                y: y - 10.0,
                class: "chartlet-value",
                anchor: TextAnchor::Middle,
                content: format_value(*value, frame.style),
            }));
        }
    }
}

/// The classes of a line: palette or declared color, its pattern and its weight. The legend
/// draws its sample with the same classes, so it looks exactly like the line.
fn line_class(spec: &ChartSpec, entry: LayerRef) -> &'static str {
    let layer = entry.layer;
    let color = if layer.resolved_color().is_some() {
        0
    } else {
        spec.palette_index(layer) + 1
    };
    let dash = match (layer.effective_dash(), layer.modeled) {
        (Dash::Solid, _) => 0,
        (Dash::Dashed, true) => 1,
        (Dash::Dashed, false) => 2,
        (Dash::Dotted, _) => 3,
    };
    let weight = match layer.stroke {
        Stroke::Regular => 0,
        Stroke::Thin => 1,
        Stroke::Bold => 2,
    };
    let classes = LINE_CLASSES.get_or_init(|| {
        let mut classes = Vec::new();
        for color in 0..=MAX_SERIES {
            let color = if color == 0 {
                String::new()
            } else {
                format!(" chartlet-line-series-{color}")
            };
            for dash in LINE_DASHES {
                for weight in LINE_WEIGHTS {
                    classes.push(format!("chartlet-line{weight}{dash}{color}"));
                }
            }
        }
        classes
    });
    &classes[(color * LINE_DASHES.len() + dash) * LINE_WEIGHTS.len() + weight]
}

/// The horizontal grid lines and value ticks of one plot.
fn push_value_grid(frame: &TimeFrame, elements: &mut Vec<Element>) {
    let plot = frame.plot;
    for value in frame.scale.ticks() {
        let y = frame.y(value);
        elements.push(Element::Line(Line {
            x1: plot.left,
            y1: y,
            x2: plot.left + plot.width,
            y2: y,
            class: if value.abs() < frame.scale.step / 100.0 {
                "chartlet-zero"
            } else {
                "chartlet-grid"
            },
        }));
        elements.push(Element::Text(Text {
            x: plot.left - 10.0,
            y: y + 4.0,
            class: "chartlet-tick",
            anchor: TextAnchor::End,
            content: format_tick(value, frame.scale.step, frame.style),
        }));
    }
}

/// The vertical grid lines and time ticks of one plot, the tick labels only with `labels`. A label
/// that would reach past the right edge of the chart moves left until it fits, since a tick can
/// sit at the very end of the span.
fn push_time_ticks(
    frame: &TimeFrame,
    max_ticks: usize,
    labels: bool,
    canvas_width: f64,
    metrics: &impl TextMetrics,
    elements: &mut Vec<Element>,
) {
    let plot = frame.plot;
    for tick in time::ticks(
        frame.span.0,
        frame.span.1,
        frame.zone,
        max_ticks,
        frame.precision == Precision::Minute,
    ) {
        let x = frame.x(tick.epoch);
        elements.push(Element::Line(Line {
            x1: x,
            y1: plot.top,
            x2: x,
            y2: plot.top + plot.height,
            class: "chartlet-grid",
        }));
        if !labels {
            continue;
        }
        let half = metrics.width(&tick.label, LABEL_SIZE) / 2.0;
        elements.push(Element::Text(Text {
            x: x.min(canvas_width - 4.0 - half),
            y: plot.top + plot.height + 24.0,
            class: "chartlet-tick",
            anchor: TextAnchor::Middle,
            content: tick.label,
        }));
    }
}

/// Space between two stacked panes; it holds the value axis title of the lower one.
const PANE_GAP: f64 = 36.0;
/// A pane lower than this cannot show its value axis in a readable way.
const MIN_PANE_HEIGHT: f64 = 40.0;

/// One frame per pane of a time chart, stacked from top to bottom within `plot`: every pane
/// shares the time span and takes a share of the height by its `heightRatio`, and scales its
/// values on its own.
fn pane_frames(
    spec: &ChartSpec,
    zone: TimeZone,
    plot: PlotArea,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<TimeFrame> {
    let span = time_span(spec, zone);
    let precision = spec.time_precision(zone);
    let panes = spec.panes.len();
    let ratios: u32 = spec.panes.iter().map(|pane| pane.height_ratio).sum();
    let available = plot.height - PANE_GAP * count(panes - 1);
    let mut top = plot.top;
    let mut frames = Vec::new();
    for (pane_index, pane) in spec.panes.iter().enumerate() {
        let height = if panes == 1 {
            plot.height
        } else {
            available * f64::from(pane.height_ratio) / f64::from(ratios)
        };
        if panes > 1 && height < MIN_PANE_HEIGHT {
            warnings.push(ChartWarning::new(
                "dense_chart",
                format!("/panes/{pane_index}/heightRatio"),
                "the pane is less than 40 pixels tall; raise height or its heightRatio",
            ));
        }
        frames.push(TimeFrame {
            plot: PlotArea {
                top,
                height,
                ..plot
            },
            span,
            zone,
            precision,
            scale: time_scale(spec, zone, Some(pane_index)),
            style: spec.pane_style(pane_index),
        });
        top += height + PANE_GAP;
    }
    frames
}

/// The value grid, the vertical time grid and the value axis title of one pane of a time chart;
/// the time tick labels only below the bottom pane, since all panes share one time axis.
fn push_pane_axes(
    spec: &ChartSpec,
    pane_index: usize,
    frame: &TimeFrame,
    bottom_pane: bool,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let plot = frame.plot;
    push_value_grid(frame, elements);
    let max_ticks =
        usize::try_from((plot_pixels(spec.width) / time_tick_spacing(frame.precision)).max(2))
            .expect("a usize is at least 32 bits wide");
    push_time_ticks(
        frame,
        max_ticks,
        bottom_pane,
        f64::from(spec.width),
        metrics,
        elements,
    );

    if let Some(title) = spec.panes[pane_index].value_axis.title.as_deref() {
        elements.push(Element::Text(Text {
            x: plot.left,
            y: plot.top - 20.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Start,
            content: fit_text(
                title,
                plot.width,
                LABEL_SIZE,
                metrics,
                warnings,
                &format!("/panes/{pane_index}/valueAxis/title"),
            ),
        }));
    }
}

/// One entry of a time chart's legend: the layer it shows, its text, and where it sits.
struct LegendEntry<'a> {
    entry: LayerRef<'a>,
    name: String,
    x: f64,
    row: usize,
}

/// One legend entry per series name, in order of first appearance, placed in rows: an entry that
/// would reach past `available_width` starts a new row. A modeled series says so in its entry,
/// because the dashing alone is not a legend.
fn legend_entries<'a>(
    spec: &'a ChartSpec,
    left: f64,
    available_width: f64,
    metrics: &impl TextMetrics,
) -> Vec<LegendEntry<'a>> {
    let mut layers: Vec<LayerRef> = Vec::new();
    for entry in spec.data_layers() {
        if !layers
            .iter()
            .any(|known| known.layer.name == entry.layer.name)
        {
            layers.push(entry);
        }
    }
    let words = spec.locale.words();
    let sample = LEGEND_LINE + 8.0;
    let (mut x, mut row) = (left, 0);
    let mut entries = Vec::new();
    for entry in layers {
        let name = entry.layer.name.as_deref().unwrap_or(words.value);
        let name = if entry.layer.modeled {
            format!("{name} ({})", words.modeled)
        } else if entry.layer.mark == Mark::Ohlc {
            format!("{name} ({})", words.candle_key)
        } else {
            name.to_owned()
        };
        let width = metrics
            .width(&name, LABEL_SIZE)
            .min(available_width - sample);
        if x > left && x + sample + width > left + available_width {
            x = left;
            row += 1;
        }
        entries.push(LegendEntry {
            entry,
            name,
            x,
            row,
        });
        x += sample + width + 20.0;
    }
    entries
}

/// The number of rows the legend of a time chart or of small multiples takes.
fn legend_rows(spec: &ChartSpec, available_width: f64, metrics: &impl TextMetrics) -> usize {
    legend_entries(spec, 0.0, available_width, metrics)
        .last()
        .map_or(1, |entry| entry.row + 1)
}

/// Draws the legend of a time chart or of small multiples, see [`legend_entries`]. Every sample is
/// a short piece of the line itself, over a swatch of its fill for an area.
fn add_layer_legend(
    spec: &ChartSpec,
    row: f64,
    left: f64,
    available_width: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    for LegendEntry {
        entry,
        name,
        x,
        row: line,
    } in legend_entries(spec, left, available_width, metrics)
    {
        let y = row + count(line) * LEGEND_HEIGHT;
        let explicit = entry.layer.resolved_color().is_some();
        if entry.layer.mark == Mark::Ohlc {
            crate::ohlc::push_legend_sample(x, y, elements);
        } else if entry.layer.mark == Mark::Area {
            elements.push(Element::Rect(Rect {
                x,
                y: y + 5.0,
                width: LEGEND_LINE,
                height: 8.0,
                class: if explicit {
                    "chartlet-area"
                } else {
                    AREA_CLASSES[spec.palette_index(entry.layer)]
                },
                series_index: None,
                style_index: explicit.then_some(entry.global),
                tooltip: None,
            }));
        }
        // A short piece of the line itself: color, pattern and weight, so that a legend entry
        // never rests on color alone.
        if entry.layer.mark != Mark::Ohlc {
            elements.push(Element::Polyline(Polyline {
                points: vec![(x, y + 5.0), (x + LEGEND_LINE, y + 5.0)],
                class: line_class(spec, entry),
                topic: None,
                series_index: None,
                style_index: explicit.then_some(entry.global),
                tooltip: None,
            }));
        }
        let label = fit_text(
            &name,
            left + available_width - x - LEGEND_LINE - 8.0,
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/panes/{}/layers/{}/name", entry.pane, entry.local),
        );
        elements.push(Element::Text(Text {
            x: x + LEGEND_LINE + 8.0,
            y: y + 9.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: label,
        }));
    }
}

/// The first and last timestamp any data layer, vertical reference line, point marker or zone
/// edge uses, a missing value included. Validation guarantees every data layer holds at least two observations that
/// increase, and a zoom window at least two of one layer, so the span is never empty.
fn time_span(spec: &ChartSpec, zone: TimeZone) -> (i64, i64) {
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    let data = spec
        .data_layers()
        .flat_map(|entry| entry.layer.resolved_times(zone));
    let rules = spec
        .layers()
        .flat_map(|layer| [&layer.time, &layer.from, &layer.to])
        .filter_map(Option::as_ref)
        .filter_map(|time| time.resolve(zone).ok());
    for epoch in data.chain(rules) {
        min = min.min(epoch);
        max = max.max(epoch);
    }
    (min, max)
}

/// Maps a timestamp onto the time axis.
fn time_x(epoch: i64, span: (i64, i64), plot: PlotArea) -> f64 {
    // The specification allows 1700 to 2200, so a span stays below 2^53 seconds and the ratio
    // keeps full precision even in 64-bit seconds.
    #[allow(clippy::cast_precision_loss)]
    let width = (span.1 - span.0).max(1) as f64;
    #[allow(clippy::cast_precision_loss)]
    let ratio = (epoch - span.0) as f64 / width;
    plot.left + TIME_INSET + ratio * (plot.width - 2.0 * TIME_INSET)
}

fn add_line_data(
    spec: &ChartSpec,
    plot: PlotArea,
    scale: &NumericScale,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let last_index = spec.data.len().saturating_sub(1);
    let divisor = f64::from(u32::try_from(last_index.max(1)).expect("data count is limited"));
    let spacing = if last_index == 0 {
        plot.width
    } else {
        plot.width / divisor
    };
    let mut segment = Vec::new();

    for (index, point) in spec.data.iter().enumerate() {
        let item_index = f64::from(u32::try_from(index).expect("data count is limited"));
        let x = if last_index == 0 {
            plot.left + plot.width / 2.0
        } else {
            plot.left + 6.0 + item_index / divisor * (plot.width - 12.0)
        };

        if let Some(value) = point.value {
            let y = scale.map(value, plot.top + plot.height, plot.top);
            segment.push((x, y));
            elements.push(Element::Circle(Circle {
                cx: x,
                cy: y,
                radius: 4.0,
                class: "chartlet-point",
                topic: None,
                series_index: None,
                style_index: None,
                tooltip: Some(tooltip(&point.label, value, spec.number_style(), None)),
            }));
            if spec.show_values {
                elements.push(Element::Text(Text {
                    x,
                    y: y - 10.0,
                    class: "chartlet-value",
                    anchor: TextAnchor::Middle,
                    content: format_value(value, spec.number_style()),
                }));
            }
        } else {
            flush_line_segment(elements, &mut segment);
        }

        let label = fit_text(
            &point.label,
            (spacing - 8.0).max(20.0),
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/data/{index}/label"),
        );
        elements.push(Element::Text(Text {
            x,
            y: plot.top + plot.height + 24.0,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content: label,
        }));
    }
    flush_line_segment(elements, &mut segment);
}

fn flush_line_segment(elements: &mut Vec<Element>, segment: &mut Vec<(f64, f64)>) {
    if segment.len() >= 2 {
        elements.push(Element::Polyline(Polyline {
            points: std::mem::take(segment),
            class: "chartlet-line",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    } else {
        segment.clear();
    }
}

pub(crate) fn base_elements(
    spec: &ChartSpec,
    scale: &NumericScale,
    plot: PlotArea,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Element> {
    let title = fit_text(&spec.title, plot.width, 22.0, metrics, warnings, "/title");
    let mut elements = vec![Element::Text(Text {
        x: plot.left,
        y: 30.0,
        class: "chartlet-title",
        anchor: TextAnchor::Start,
        content: title,
    })];

    for value in scale.ticks() {
        if plot.vertical_bars {
            let y = scale.map(value, plot.top + plot.height, plot.top);
            elements.push(Element::Line(Line {
                x1: plot.left,
                y1: y,
                x2: plot.left + plot.width,
                y2: y,
                class: if value.abs() < scale.step / 100.0 {
                    "chartlet-zero"
                } else {
                    "chartlet-grid"
                },
            }));
            elements.push(Element::Text(Text {
                x: plot.left - 10.0,
                y: y + 4.0,
                class: "chartlet-tick",
                anchor: TextAnchor::End,
                content: format_tick(value, scale.step, spec.number_style()),
            }));
        } else {
            let x = scale.map(value, plot.left, plot.left + plot.width);
            elements.push(Element::Line(Line {
                x1: x,
                y1: plot.top,
                x2: x,
                y2: plot.top + plot.height,
                class: if value.abs() < scale.step / 100.0 {
                    "chartlet-zero"
                } else {
                    "chartlet-grid"
                },
            }));
            elements.push(Element::Text(Text {
                x,
                y: plot.top + plot.height + 22.0,
                class: "chartlet-tick",
                anchor: TextAnchor::Middle,
                content: format_tick(value, scale.step, spec.number_style()),
            }));
        }
    }

    if let Some(title) = &spec.value_axis.title {
        let title = fit_text(
            title,
            plot.width,
            LABEL_SIZE,
            metrics,
            warnings,
            "/valueAxis/title",
        );
        if plot.vertical_bars {
            elements.push(Element::Text(Text {
                x: plot.left,
                y: plot.top - 20.0,
                class: "chartlet-axis-title",
                anchor: TextAnchor::Start,
                content: title.clone(),
            }));
        } else {
            elements.push(Element::Text(Text {
                x: plot.left + plot.width / 2.0,
                y: plot.top + plot.height + 44.0,
                class: "chartlet-axis-title",
                anchor: TextAnchor::Middle,
                content: title,
            }));
        }
    }

    if !plot.vertical_bars
        && let Some(title) = &spec.category_axis.title
    {
        let title = fit_text(
            title,
            plot.left - 24.0,
            LABEL_SIZE,
            metrics,
            warnings,
            "/categoryAxis/title",
        );
        elements.push(Element::Text(Text {
            x: plot.left - 12.0,
            y: plot.top - 20.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::End,
            content: title,
        }));
    }

    elements
}

/// Centers the category axis title below a plot whose categories run horizontally.
pub(crate) fn add_bottom_category_title(
    spec: &ChartSpec,
    left: f64,
    plot_width: f64,
    chart_height: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    if let Some(title) = &spec.category_axis.title {
        let title = fit_text(
            title,
            plot_width,
            LABEL_SIZE,
            metrics,
            warnings,
            "/categoryAxis/title",
        );
        elements.push(Element::Text(Text {
            x: left + plot_width / 2.0,
            y: chart_height - 14.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: title,
        }));
    }
}

/// Places a value label above a positive bar or below a negative one.
fn vertical_value_label(value: f64, center_x: f64, value_y: f64, content: String) -> Text {
    Text {
        x: center_x,
        y: if value >= 0.0 {
            value_y - 8.0
        } else {
            value_y + 16.0
        },
        class: "chartlet-value",
        anchor: TextAnchor::Middle,
        content,
    }
}

fn horizontal_value_label(
    value: f64,
    value_x: f64,
    baseline: f64,
    center_y: f64,
    style: NumberStyle,
    metrics: &impl TextMetrics,
) -> Text {
    let content = format_value(value, style);
    // Inverse text is only readable on the bar itself; a short negative bar gets its label
    // outside, left of the bar end.
    let fits_inside = metrics.width(&content, LABEL_SIZE) + 16.0 <= (baseline - value_x).abs();
    let (x, class, anchor) = if value >= 0.0 {
        (value_x + 8.0, "chartlet-value", TextAnchor::Start)
    } else if fits_inside {
        (value_x + 8.0, "chartlet-value-inverse", TextAnchor::Start)
    } else {
        (value_x - 8.0, "chartlet-value", TextAnchor::End)
    };
    Text {
        x,
        y: center_y + 4.0,
        class,
        anchor,
        content,
    }
}

fn series_text(text: Text, dataset: &Dataset, series_index: usize) -> Element {
    if dataset.series.len() > 1 {
        Element::SeriesText(text, series_index)
    } else {
        Element::Text(text)
    }
}

/// The bars of one category: a single bar, or one slot per series side by side.
#[derive(Debug, Clone, Copy)]
struct Group {
    width: f64,
    slot: f64,
    gap: f64,
    single: bool,
}

impl Group {
    fn new(band: f64, series_count: usize) -> Self {
        if series_count == 1 {
            let width = (band * 0.64).max(2.0);
            return Self {
                width,
                slot: width,
                gap: 0.0,
                single: true,
            };
        }
        let series = count(series_count);
        let width = (band * 0.8).max(2.0 * series);
        let slot = width / series;
        Self {
            width,
            slot,
            gap: (slot * 0.15).min(4.0),
            single: false,
        }
    }

    /// Offset from the group's leading edge and drawn thickness of one series' bar.
    fn slot(self, series_index: usize) -> (f64, f64) {
        (
            count(series_index) * self.slot + self.gap / 2.0,
            self.slot - self.gap,
        )
    }

    /// Whether a value label of the given extent fits without reaching into the neighbouring
    /// series. A single bar always keeps its label.
    fn fits(self, extent: f64) -> bool {
        self.single || extent <= self.slot
    }
}

fn legend_space(dataset: &Dataset) -> f64 {
    if dataset.series.len() > 1 {
        LEGEND_HEIGHT
    } else {
        0.0
    }
}

fn add_legend(
    dataset: &Dataset,
    left: f64,
    available_width: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    if dataset.series.len() < 2 {
        return;
    }
    let entry_width = available_width / count(dataset.series.len());
    let mut x = left;
    for (index, series) in dataset.series.iter().enumerate() {
        let name = series
            .name
            .as_deref()
            .expect("multi-series charts name every series");
        elements.push(Element::Rect(Rect {
            x,
            y: 46.0,
            width: 10.0,
            height: 10.0,
            class: SERIES_BAR_CLASSES[index],
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        let label = fit_text(
            name,
            entry_width - 36.0,
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/series/{index}/name"),
        );
        let label_width = metrics.width(&label, LABEL_SIZE);
        elements.push(Element::Text(Text {
            x: x + 16.0,
            y: 55.0,
            class: "chartlet-legend",
            anchor: TextAnchor::Start,
            content: label,
        }));
        x += 16.0 + label_width + 20.0;
    }
}

fn bar_class(dataset: &Dataset, series_index: usize) -> &'static str {
    if dataset.series.len() == 1 {
        "chartlet-bar"
    } else {
        SERIES_BAR_CLASSES[series_index]
    }
}

pub(crate) fn tooltip(
    category: &str,
    value: f64,
    style: NumberStyle,
    series_name: Option<&str>,
) -> String {
    let formatted = format_value(value, style);
    match series_name {
        Some(name) => format!("{category} – {name}: {formatted}"),
        None => format!("{category}: {formatted}"),
    }
}

pub(crate) fn warn_if_labels_omitted(omitted: bool, warnings: &mut Vec<ChartWarning>) {
    if omitted {
        warnings.push(ChartWarning::new(
            "value_labels_omitted",
            "/showValues",
            "some value labels do not fit next to their bars and were left out; every value remains in the HTML data alternative",
        ));
    }
}

pub(crate) fn count(value: usize) -> f64 {
    f64::from(u32::try_from(value).expect("counts are limited by validation"))
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotArea {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub vertical_bars: bool,
}

pub(crate) fn fit_text(
    text: &str,
    max_width: f64,
    font_size: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) -> String {
    if metrics.width(text, font_size) <= max_width {
        return text.to_owned();
    }

    let ellipsis_width = metrics.width("…", font_size);
    let mut output = String::new();
    for character in text.chars() {
        let candidate = format!("{output}{character}");
        if metrics.width(&candidate, font_size) + ellipsis_width > max_width {
            break;
        }
        output.push(character);
    }
    output.push('…');

    if !warnings
        .iter()
        .any(|warning| warning.code == "text_truncated" && warning.path == path)
    {
        warnings.push(ChartWarning::new(
            "text_truncated",
            path,
            "text was shortened in the visual chart; the full value remains in the specification and HTML data alternative",
        ));
    }
    output
}

/// Writes a value for reading: percent scaled and suffixed, fixed decimals if the style asks for
/// them, a true minus sign (U+2212, which screen readers announce as “minus”), and the locale's
/// decimal separator.
pub(crate) fn format_value(value: f64, style: impl Into<NumberStyle>) -> String {
    let style = style.into();
    let value = match style.format {
        ValueFormat::Number => value,
        ValueFormat::Percent => value * 100.0,
    };
    let digits = match style.decimals {
        Some(decimals) => format!("{:.*}", usize::from(decimals), tidy(value)),
        None => format!("{}", tidy(value)),
    };
    // Rounding to fixed decimals can turn a small negative value into `-0.0`.
    let digits = if digits.starts_with('-') && digits.trim_start_matches(['-', '0', '.']).is_empty()
    {
        digits[1..].to_owned()
    } else {
        digits
    };
    let digits = digits.replace('-', "\u{2212}");
    let digits = match style.locale {
        Locale::En => digits,
        Locale::De => digits.replace('.', ","),
    };
    let suffix = match (style.format, style.locale) {
        (ValueFormat::Number, _) => "",
        (ValueFormat::Percent, Locale::En) => "%",
        (ValueFormat::Percent, Locale::De) => "\u{202f}%",
    };
    format!("{digits}{suffix}")
}

/// Writes an axis tick with as many decimals as the tick step has, so that every tick of an axis
/// carries the same number of digits: `0.0, 0.5, 1.0` rather than `0, 0.5, 1`.
pub(crate) fn format_tick(value: f64, step: f64, style: NumberStyle) -> String {
    let step = match style.format {
        ValueFormat::Number => step,
        ValueFormat::Percent => step * 100.0,
    };
    let decimals = format!("{}", tidy(step))
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    format_value(
        value,
        NumberStyle {
            decimals: Some(u8::try_from(decimals).unwrap_or(u8::MAX)),
            ..style
        },
    )
}

/// Removes binary floating-point noise (`0.07 * 100`, `0.1 + 0.2`) by keeping 12 significant
/// digits, and normalizes `-0.0`. Rust's float formatting does not depend on the platform, so
/// the result stays deterministic.
fn tidy(value: f64) -> f64 {
    let rounded: f64 = format!("{value:.11e}")
        .parse()
        .expect("a formatted float parses back");
    if rounded == 0.0 { 0.0 } else { rounded }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NumericScale {
    min: f64,
    max: f64,
    pub step: f64,
}

impl NumericScale {
    pub(crate) fn from_values(mut values: impl Iterator<Item = f64>, include_zero: bool) -> Self {
        let first = values.next().expect("validated charts contain values");
        let mut min = if include_zero { first.min(0.0) } else { first };
        let mut max = if include_zero { first.max(0.0) } else { first };
        for value in values {
            min = min.min(value);
            max = max.max(value);
        }
        if (max - min).abs() < f64::EPSILON {
            if max.abs() < f64::EPSILON {
                min = -1.0;
                max = 1.0;
            } else {
                let padding = max.abs() * 0.2;
                min -= padding;
                max += padding;
            }
        } else if !include_zero {
            let padding = (max - min) * 0.05;
            min -= padding;
            max += padding;
        }
        let raw_step = (max - min) / 5.0;
        let magnitude = 10.0_f64.powf(raw_step.log10().floor());
        let fraction = raw_step / magnitude;
        let nice_fraction = if fraction <= 1.0 {
            1.0
        } else if fraction <= 2.0 {
            2.0
        } else if fraction <= 5.0 {
            5.0
        } else {
            10.0
        };
        let step = nice_fraction * magnitude;
        Self {
            min: tidy(tidy(min / step).floor() * step),
            max: tidy(tidy(max / step).ceil() * step),
            step,
        }
    }

    pub(crate) fn map(self, value: f64, output_min: f64, output_max: f64) -> f64 {
        let ratio = (value - self.min) / (self.max - self.min);
        output_min + ratio * (output_max - output_min)
    }

    /// Ticks are computed from their index instead of by repeated addition, so rounding errors
    /// do not accumulate along the axis.
    pub(crate) fn ticks(self) -> impl Iterator<Item = f64> {
        std::iter::successors(Some(0.0_f64), |index| Some(index + 1.0))
            .map(move |index| tidy(self.min + index * self.step))
            .take_while(move |tick| *tick <= self.max + self.step / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEPTH_BAND_STEP, HALO_WIDTH, MAX_WOBBLE, MIN_WOBBLE, NumericScale, PLOT_MARGIN, cap_links,
        coastline_profile, count, distance_to, format_tick, format_value, plan_furniture,
        topicmap_positions, unit_polygon_area,
    };
    use crate::{
        metrics::BuiltinMetrics,
        spec::{
            CartoucheSpec, Corner, Locale, NumberStyle, TopicLinkSpec, TopicMapSpec, TopicSpec,
            ValueFormat,
        },
    };

    #[test]
    fn scale_includes_zero_and_uses_nice_ticks() {
        let scale = NumericScale::from_values([12.0, 18.0, 15.0].into_iter(), true);
        assert!(scale.min.abs() < f64::EPSILON);
        assert!((scale.max - 20.0).abs() < f64::EPSILON);
        assert_eq!(
            scale.ticks().collect::<Vec<_>>(),
            vec![0.0, 5.0, 10.0, 15.0, 20.0]
        );
    }

    #[test]
    fn formats_percent_without_duplicate_suffix() {
        assert_eq!(format_value(0.125, ValueFormat::Percent), "12.5%");
        assert_eq!(format_value(1.0, ValueFormat::Percent), "100%");
    }

    #[test]
    fn removes_floating_point_noise_from_labels() {
        assert_eq!(format_value(0.07, ValueFormat::Percent), "7%");
        assert_eq!(format_value(0.1 + 0.2, ValueFormat::Number), "0.3");
        assert_eq!(format_value(-0.0, ValueFormat::Number), "0");
    }

    #[test]
    fn writes_german_numbers_with_comma_and_true_minus() {
        let de = NumberStyle {
            locale: Locale::De,
            ..NumberStyle::default()
        };
        assert_eq!(format_value(-1.25, de), "\u{2212}1,25");
        assert_eq!(format_value(-1.25, ValueFormat::Number), "\u{2212}1.25");
        let fixed = NumberStyle {
            decimals: Some(2),
            ..de
        };
        assert_eq!(format_value(1.547, fixed), "1,55");
        assert_eq!(format_value(-0.001, fixed), "0,00");
        let percent = NumberStyle {
            format: ValueFormat::Percent,
            ..de
        };
        assert_eq!(format_value(0.125, percent), "12,5\u{202f}%");
    }

    #[test]
    fn ticks_share_the_decimals_of_their_step() {
        let style = NumberStyle::default();
        assert_eq!(format_tick(1.0, 0.5, style), "1.0");
        assert_eq!(format_tick(-0.5, 0.5, style), "\u{2212}0.5");
        assert_eq!(format_tick(20.0, 5.0, style), "20");
        let percent = NumberStyle::from(ValueFormat::Percent);
        assert_eq!(format_tick(0.1, 0.05, percent), "10%");
        assert_eq!(format_tick(0.1, 0.025, percent), "10.0%");
    }

    #[test]
    fn ticks_do_not_accumulate_rounding_errors() {
        let scale = NumericScale::from_values([0.12, -0.04, 0.15].into_iter(), true);
        assert_eq!(
            scale.ticks().collect::<Vec<_>>(),
            vec![-0.05, 0.0, 0.05, 0.1, 0.15]
        );
        let scale = NumericScale::from_values([0.07, -0.005, 0.3].into_iter(), true);
        assert_eq!(
            scale
                .ticks()
                .map(|tick| format_value(tick, ValueFormat::Percent))
                .collect::<Vec<_>>(),
            vec!["−10%", "0%", "10%", "20%", "30%"]
        );
    }

    #[test]
    fn topicmap_places_many_circles_without_overlap_or_canvas_overflow() {
        let topics: Vec<TopicSpec> = (0..24)
            .map(|i| TopicSpec {
                label: format!("Topic {i}"),
                value: 20.0 + (count(i) * 37.0) % 260.0,
                points: 0,
                tooltip: None,
            })
            .collect();
        let islands: Vec<TopicSpec> = (0..6)
            .map(|i| TopicSpec {
                label: format!("Island {i}"),
                value: 5.0,
                points: 0,
                tooltip: None,
            })
            .collect();
        let links: Vec<TopicLinkSpec> = (0..23)
            .map(|i| TopicLinkSpec {
                from: format!("Topic {i}"),
                to: format!("Topic {}", i + 1),
                weight: 0.5,
            })
            .collect();
        let topicmap = TopicMapSpec {
            topics,
            links,
            islands,
            seed: 0,
            cartouche: None,
            graticule: true,
            compass: true,
            depth_bands: 2,
        };

        let margin = f64::from(PLOT_MARGIN);
        let top = 78.0;
        let plot_width = 800.0 - margin * 2.0;
        let plot_height = 450.0 - top - margin;
        let mut ordered: Vec<(usize, &TopicSpec)> = topicmap.topics.iter().enumerate().collect();
        ordered.sort_by(|a, b| b.1.value.total_cmp(&a.1.value));
        let mut warnings = Vec::new();
        let kept_links = cap_links(&topicmap, &mut warnings);
        let positions = topicmap_positions(
            &topicmap,
            &ordered,
            &kept_links,
            (margin, top, plot_width, plot_height),
            &[],
        );

        // What is drawn, not the bare circle: the coastline at its widest, pushed out by the
        // depth lines around it and by half the halo stroke.
        let drawn: Vec<(f64, f64, f64, f64)> = positions
            .values()
            .map(|placed| {
                let widest = placed.profile.iter().copied().fold(0.0, f64::max);
                let bands = 1.0 + DEPTH_BAND_STEP * f64::from(topicmap.depth_bands);
                (
                    placed.center.0,
                    placed.center.1,
                    placed.radius * widest,
                    placed.radius * widest * bands + HALO_WIDTH / 2.0,
                )
            })
            .collect();
        assert_eq!(drawn.len(), 30, "every topic and island got a position");

        for i in 0..drawn.len() {
            for j in (i + 1)..drawn.len() {
                let (x1, y1, coast1, _) = drawn[i];
                let (x2, y2, coast2, _) = drawn[j];
                let distance = (x2 - x1).hypot(y2 - y1);
                assert!(
                    distance >= coast1 + coast2,
                    "coastlines overlap: {:?} vs {:?}, distance {distance}",
                    drawn[i],
                    drawn[j]
                );
            }
        }
        for &(x, y, _, reach) in &drawn {
            assert!(x - reach >= margin - 0.5, "runs off the left edge: {x},{y}");
            assert!(
                x + reach <= margin + plot_width + 0.5,
                "runs off the right edge: {x},{y}"
            );
            assert!(y - reach >= top - 0.5, "runs off the top edge: {x},{y}");
            assert!(
                y + reach <= top + plot_height + 0.5,
                "runs off the bottom edge: {x},{y}"
            );
        }
    }

    #[test]
    fn a_coastline_encloses_the_area_its_value_asks_for() {
        // The area is the whole claim the chart makes, so the noise must not quietly change it:
        // the ratio of two enclosed areas has to be the ratio of their values.
        let reference = coastline_profile(90.0, 1);
        let reference_area = 90.0 * 90.0 * unit_polygon_area(&reference);
        for (value, seed) in [(20.0, 2_u64), (75.0, 3), (140.0, 4), (300.0, 5)] {
            let radius = 90.0 * (value / 180.0_f64).sqrt();
            let profile = coastline_profile(radius, seed);
            let area = radius * radius * unit_polygon_area(&profile);
            let drift = (area / reference_area) / (value / 180.0) - 1.0;
            assert!(
                drift.abs() <= 0.03,
                "value {value} drew an area off by {:.1}%",
                drift * 100.0
            );
        }
    }

    #[test]
    fn a_coastline_stays_within_the_wobble_the_collision_distance_assumes() {
        for seed in 0..40 {
            for radius in [12.0, 55.0, 140.0] {
                for wobble in coastline_profile(radius, seed) {
                    assert!(
                        (MIN_WOBBLE..=MAX_WOBBLE).contains(&wobble),
                        "seed {seed} at radius {radius} left the clamp: {wobble}"
                    );
                }
            }
        }
    }

    #[test]
    fn areas_stay_out_of_the_water_the_map_furniture_claims() {
        // Twelve areas on a small canvas: without reserved water, one of them ends up under the
        // cartouche or the compass rose.
        let topics: Vec<TopicSpec> = (0..12)
            .map(|index| TopicSpec {
                label: format!("Area {index}"),
                value: 40.0 + count(index) * 11.0,
                points: 0,
                tooltip: None,
            })
            .collect();
        let topicmap = TopicMapSpec {
            topics,
            links: Vec::new(),
            islands: (0..4)
                .map(|index| TopicSpec {
                    label: format!("Isle {index}"),
                    value: 4.0,
                    points: 0,
                    tooltip: None,
                })
                .collect(),
            seed: 7,
            cartouche: Some(CartoucheSpec {
                heading: "Topic map".to_owned(),
                meta: "a line of metadata".to_owned(),
                corner: Corner::BottomRight,
            }),
            graticule: true,
            compass: true,
            depth_bands: 2,
        };

        let margin = f64::from(PLOT_MARGIN);
        let top = 78.0;
        let plot = (margin, top, 800.0 - margin * 2.0, 450.0 - top - margin);
        let mut ordered: Vec<(usize, &TopicSpec)> = topicmap.topics.iter().enumerate().collect();
        ordered.sort_by(|a, b| b.1.value.total_cmp(&a.1.value));
        let mut warnings = Vec::new();
        let furniture = plan_furniture(&topicmap, plot, &BuiltinMetrics, &mut warnings);
        let reserved = furniture.reserved();
        assert_eq!(
            reserved.len(),
            2,
            "a compass rose and a cartouche were planned"
        );
        let positions = topicmap_positions(&topicmap, &ordered, &[], plot, &reserved);

        for placed in positions.values() {
            let widest = placed.profile.iter().copied().fold(0.0, f64::max);
            let reach = placed.radius * widest;
            for rect in &reserved {
                assert!(
                    distance_to(placed.center, *rect) >= reach,
                    "an area reaches into reserved water at {:?}",
                    placed.center
                );
            }
        }
    }

    #[test]
    fn the_compass_rose_moves_when_the_cartouche_wants_its_corner() {
        let base = TopicMapSpec {
            topics: vec![TopicSpec {
                label: "Only".to_owned(),
                value: 10.0,
                points: 0,
                tooltip: None,
            }],
            links: Vec::new(),
            islands: Vec::new(),
            seed: 0,
            cartouche: None,
            graticule: true,
            compass: true,
            depth_bands: 2,
        };
        let plot = (24.0, 78.0, 752.0, 348.0);
        let mut warnings = Vec::new();

        let default_corner = plan_furniture(&base, plot, &BuiltinMetrics, &mut warnings)
            .compass
            .expect("a compass rose was asked for");
        let contested = TopicMapSpec {
            cartouche: Some(CartoucheSpec {
                heading: "Topic map".to_owned(),
                meta: "metadata".to_owned(),
                corner: Corner::TopLeft,
            }),
            ..base
        };
        let moved = plan_furniture(&contested, plot, &BuiltinMetrics, &mut warnings)
            .compass
            .expect("a compass rose was asked for");
        assert!(
            default_corner.0 < moved.0,
            "the rose gave up the left corner"
        );
    }
}
