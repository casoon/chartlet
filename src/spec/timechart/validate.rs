//! Validation of the layers of a pane: observations, bands, stacks, annotations and zones.

use std::collections::BTreeSet;

use super::{
    Curve, LayerSpec, MAX_ANNOTATIONS, MAX_STACKED_LAYERS, MAX_TIME_LAYERS,
    MAX_TIME_POINTS_PER_LAYER, Mark, PaneSpec, Stroke, TimePoint,
};
use crate::{
    error::{ChartError, ChartWarning},
    spec::{AxisScale, MAX_SERIES, Stack, validate_number, validate_text},
    time::TimeValue,
};

/// What the layers of a pane are checked against: the chart's time zone, the width of its plot
/// in pixels, and whether every data layer needs a name.
pub(crate) struct LayerContext {
    pub zone: crate::time::TimeZone,
    pub plot_pixels: usize,
    pub named: bool,
}

/// A layer's own color has to be one of the forms in the contract; anything else is replaced by
/// the neutral tone, and the specification says which layer it was.
fn check_layer_color(layer: &LayerSpec, path: &str, warnings: &mut Vec<ChartWarning>) {
    if layer
        .color
        .as_deref()
        .is_some_and(|color| crate::color::sanitize(color).is_none())
    {
        let color = layer.color.as_deref().unwrap_or_default();
        warnings.push(ChartWarning::new(
            "color_not_supported",
            format!("{path}/color"),
            format!(
                "{color:?} is not #rgb, #rrggbb, #rrggbbaa or var(--name); the layer is painted in the neutral tone"
            ),
        ));
    }
}

/// The end of the last step lies after the last observation.
fn validate_step_end(
    layer: &LayerSpec,
    path: &str,
    zone: crate::time::TimeZone,
) -> Result<(), ChartError> {
    let Some(step_end) = &layer.step_end else {
        return Ok(());
    };
    let step_path = format!("{path}/stepEnd");
    let end = step_end
        .resolve(zone)
        .map_err(|message| ChartError::new("invalid_time", step_path.clone(), message))?;
    if layer
        .resolved_times(zone)
        .last()
        .is_some_and(|last| end <= *last)
    {
        return Err(ChartError::new(
            "unordered_time",
            step_path,
            "stepEnd must lie after the last observation",
        ));
    }
    Ok(())
}

/// Every observation resolves to a timestamp and follows the one before it; a `null` value marks
/// a gap and carries no band.
fn validate_points(
    layer: &LayerSpec,
    path: &str,
    zone: crate::time::TimeZone,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    let banded = layer.has_band();
    let mut previous: Option<i64> = None;
    for (point_index, point) in layer.points.iter().enumerate() {
        let point_path = format!("{path}/points/{point_index}");
        let epoch = point.time.resolve(zone).map_err(|message| {
            ChartError::new("invalid_time", format!("{point_path}/time"), message)
        })?;
        if let Some(value) = point.value {
            validate_number(value, &format!("{point_path}/value"))?;
            validate_band_point(point, &point_path, banded, value, warnings)?;
        } else if point.lower.is_some() || point.upper.is_some() {
            let edge = if point.lower.is_some() {
                "lower"
            } else {
                "upper"
            };
            return Err(ChartError::new(
                "invalid_band",
                format!("{point_path}/{edge}"),
                "a missing value has no band; leave out lower and upper where value is null",
            ));
        }
        if previous.is_some_and(|previous| epoch <= previous) {
            return Err(ChartError::new(
                "unordered_time",
                format!("{point_path}/time"),
                "timestamps must increase from point to point",
            ));
        }
        previous = Some(epoch);
    }
    Ok(())
}

/// The band edges of one point with a value: both or neither, matching the rest of the layer, and the
/// lower edge not above the upper one. A value outside its own band is allowed but reported: a
/// median can leave a percentile band only when the two come from different sources.
fn validate_band_point(
    point: &TimePoint,
    point_path: &str,
    banded: bool,
    value: f64,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    for (edge, present) in [
        ("lower", point.lower.is_some()),
        ("upper", point.upper.is_some()),
    ] {
        if present != banded {
            return Err(ChartError::new(
                "incomplete_band",
                format!("{point_path}/{edge}"),
                "give every point of a layer both lower and upper, or none of them",
            ));
        }
    }
    let (Some(lower), Some(upper)) = (point.lower, point.upper) else {
        return Ok(());
    };
    validate_number(lower, &format!("{point_path}/lower"))?;
    validate_number(upper, &format!("{point_path}/upper"))?;
    if lower > upper {
        return Err(ChartError::new(
            "invalid_band",
            format!("{point_path}/upper"),
            "upper must not be below lower",
        ));
    }
    if !(lower..=upper).contains(&value) {
        warnings.push(ChartWarning::new(
            "value_outside_band",
            format!("{point_path}/value"),
            "the value lies outside its own band; check that line and band describe the same quantity",
        ));
    }
    Ok(())
}

/// A stacked pane: at least two area layers, by value, with the same times and curve, a value at
/// every time, none below zero and no band.
pub(super) fn validate_pane_stack(
    pane: &PaneSpec,
    path: &str,
    zone: crate::time::TimeZone,
) -> Result<(), ChartError> {
    let Some(stack) = pane.stack else {
        return Ok(());
    };
    let refuse = |path: String, code: &'static str, message: &str| {
        Err(ChartError::new(code, path, message.to_owned()))
    };
    if stack == Stack::Percent {
        return refuse(
            format!("{path}/stack"),
            "option_not_supported",
            "a time pane stacks its areas by value only; use \"normal\" or \"stream\"",
        );
    }
    if stack == Stack::Stream {
        validate_stream(pane, path)?;
    }
    let areas: Vec<(usize, &LayerSpec)> = pane
        .layers
        .iter()
        .enumerate()
        .filter(|(_, layer)| layer.mark == Mark::Area)
        .collect();
    let Some((first_index, first)) = areas.first().copied().filter(|_| areas.len() >= 2) else {
        return refuse(
            format!("{path}/stack"),
            "option_not_supported",
            "stack needs at least two area layers in the pane",
        );
    };
    let times = first.resolved_times(zone);
    for (index, layer) in &areas {
        let layer_path = format!("{path}/layers/{index}");
        if let Some(point) = layer.points.iter().position(|point| point.value.is_none()) {
            return refuse(
                format!("{layer_path}/points/{point}/value"),
                "unaligned_stack",
                "a stacked area needs a value at every time; use 0 where there is none",
            );
        }
        if let Some(point) = layer
            .points
            .iter()
            .position(|point| point.value.is_some_and(|value| value < 0.0))
        {
            return refuse(
                format!("{layer_path}/points/{point}/value"),
                "negative_in_stack",
                "stacked areas take values of zero or more",
            );
        }
        if layer.has_band() {
            return refuse(
                format!("{layer_path}/points"),
                "option_not_supported",
                "a stacked area has no lower/upper band",
            );
        }
        if layer.resolved_times(zone) != times {
            return refuse(
                format!("{layer_path}/points"),
                "unaligned_stack",
                &format!(
                    "a stacked area needs the same times as the first, {path}/layers/{first_index}"
                ),
            );
        }
        if layer.curve != first.curve {
            return refuse(
                format!("{layer_path}/curve"),
                "option_not_supported",
                "the stacked areas of a pane share one curve",
            );
        }
    }
    Ok(())
}

/// A stream has no position on its value axis, only thickness: the pane holds areas and rules of
/// time, no lines, bands, markers or rules of value, and its axis takes no range, direction or
/// logarithm.
fn validate_stream(pane: &PaneSpec, path: &str) -> Result<(), ChartError> {
    let refuse = |path: String, message: &str| {
        Err(ChartError::new(
            "option_not_supported",
            path,
            message.to_owned(),
        ))
    };
    let axis = &pane.value_axis;
    for (field, present) in [
        ("min", axis.min.is_some()),
        ("max", axis.max.is_some()),
        ("exact", axis.exact),
        ("reverse", axis.reverse),
        ("scale", axis.scale == AxisScale::Log),
    ] {
        if present {
            return refuse(
                format!("{path}/valueAxis/{field}"),
                "a stream is centered on a moving baseline, so its value axis shows thickness only; remove this field",
            );
        }
    }
    for (index, layer) in pane.layers.iter().enumerate() {
        let allowed = match layer.mark {
            Mark::Area => true,
            Mark::Annotation => layer.value.is_none(),
            Mark::Line | Mark::Point | Mark::Ohlc | Mark::Band => false,
        };
        if !allowed {
            return refuse(
                format!("{path}/layers/{index}/mark"),
                "a stream holds area layers and reference lines of time only: a position on its value axis means nothing",
            );
        }
    }
    Ok(())
}

/// The layers of one pane: data layers within their limit and at most as many without a color of
/// their own as the palette has colors, reference lines within their own limit, and a name on
/// every data layer once there is more than one.
pub(super) fn validate_pane_layers(
    pane: &PaneSpec,
    pane_path: &str,
    context: &LayerContext,
    names: &mut BTreeSet<String>,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    let zone = context.zone;
    let data_layers = pane.layers.iter().filter(|layer| layer.is_data()).count();
    let annotations = pane.layers.len() - data_layers;
    if data_layers == 0 {
        return Err(ChartError::new(
            "empty_data",
            format!("{pane_path}/layers"),
            "provide at least one line layer; a reference line alone has nothing to refer to",
        ));
    }
    let limit = if pane.stack.is_some() {
        MAX_STACKED_LAYERS
    } else {
        MAX_TIME_LAYERS
    };
    if data_layers > limit {
        return Err(ChartError::new(
            "too_many_layers",
            format!("{pane_path}/layers"),
            format!(
                "at most {MAX_TIME_LAYERS} data layers are supported, {MAX_STACKED_LAYERS} in a stacked pane"
            ),
        ));
    }
    if let Some((layer_index, _)) = pane
        .layers
        .iter()
        .enumerate()
        .filter(|(_, layer)| layer.takes_palette())
        .nth(MAX_SERIES)
    {
        return Err(ChartError::new(
            "too_many_layers",
            format!("{pane_path}/layers/{layer_index}/color"),
            format!(
                "at most {MAX_SERIES} data layers take a palette color; give this layer a color of its own"
            ),
        ));
    }
    if annotations > MAX_ANNOTATIONS {
        return Err(ChartError::new(
            "too_many_annotations",
            format!("{pane_path}/layers"),
            format!("at most {MAX_ANNOTATIONS} annotation layers are supported per pane"),
        ));
    }
    for (layer_index, layer) in pane.layers.iter().enumerate() {
        let layer_path = format!("{pane_path}/layers/{layer_index}");
        match layer.mark {
            Mark::Annotation => validate_annotation(layer, &layer_path, zone, warnings)?,
            Mark::Band => validate_zone(layer, &layer_path, zone, warnings)?,
            Mark::Ohlc => crate::ohlc::validate(layer, &layer_path, context, names, warnings)?,
            Mark::Line | Mark::Area | Mark::Point => {
                validate_layer(layer, &layer_path, context, names, warnings)?;
            }
        }
    }
    Ok(())
}

/// An annotation: a horizontal reference line at a `value`, a vertical one at a `time`, or a
/// point marker at both, always with a label. Only a point marker takes a `shape`.
fn validate_annotation(
    layer: &LayerSpec,
    path: &str,
    zone: crate::time::TimeZone,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    check_layer_color(layer, path, warnings);
    for (field, present, reason) in [
        (
            "points",
            !layer.points.is_empty(),
            "belongs to a line layer",
        ),
        ("data", !layer.data.is_empty(), "belongs to an ohlc layer"),
        ("from", layer.from.is_some(), "belongs to a band layer"),
        ("to", layer.to.is_some(), "belongs to a band layer"),
        ("top", layer.top.is_some(), "belongs to a band layer"),
        ("bottom", layer.bottom.is_some(), "belongs to a band layer"),
        ("modeled", layer.modeled, "belongs to a line layer"),
        (
            "stroke",
            layer.stroke != Stroke::Regular,
            "belongs to a line layer",
        ),
        ("dash", layer.dash.is_some(), "belongs to a line layer"),
        (
            "curve",
            layer.curve != Curve::Linear,
            "belongs to a line or area layer",
        ),
        (
            "name",
            layer.name.is_some(),
            "is not used; an annotation is named by its label",
        ),
        (
            "shape",
            layer.shape.is_some() && !layer.is_marker(),
            "belongs to a point marker with both time and value; a reference line has no shape",
        ),
    ] {
        if present {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                format!("{field} {reason}"),
            ));
        }
    }
    if layer.time.is_none() && layer.value.is_none() {
        return Err(ChartError::new(
            "missing_position",
            path,
            "give time for a vertical reference line, value for a horizontal one, or both for a point marker",
        ));
    }
    if let Some(time) = &layer.time {
        time.resolve(zone)
            .map_err(|message| ChartError::new("invalid_time", format!("{path}/time"), message))?;
    }
    if let Some(value) = layer.value {
        validate_number(value, &format!("{path}/value"))?;
    }
    let Some(label) = &layer.label else {
        return Err(ChartError::new(
            "missing_label",
            format!("{path}/label"),
            if layer.is_marker() {
                "a point marker needs a label that says what it marks"
            } else {
                "a reference line needs a label that says what it marks"
            },
        ));
    };
    validate_text(label, &format!("{path}/label"), 100)
}

/// A zone: a shaded area between `bottom` and `top`, between `from` and `to`, or both. A missing
/// edge is the edge of the plot, so a zone with only `bottom` covers every value above it; at
/// least one edge has to be given, and the zone needs a label.
fn validate_zone(
    layer: &LayerSpec,
    path: &str,
    zone: crate::time::TimeZone,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    check_layer_color(layer, path, warnings);
    for (field, present, reason) in [
        (
            "points",
            !layer.points.is_empty(),
            "belongs to a line layer; for an uncertainty band around a line, give its points lower and upper",
        ),
        ("data", !layer.data.is_empty(), "belongs to an ohlc layer"),
        (
            "time",
            layer.time.is_some(),
            "belongs to an annotation; a zone spans from and to",
        ),
        (
            "value",
            layer.value.is_some(),
            "belongs to an annotation; a zone spans bottom and top",
        ),
        ("shape", layer.shape.is_some(), "belongs to a point marker"),
        ("modeled", layer.modeled, "belongs to a line layer"),
        (
            "stroke",
            layer.stroke != Stroke::Regular,
            "belongs to a line layer",
        ),
        ("dash", layer.dash.is_some(), "belongs to a line layer"),
        (
            "curve",
            layer.curve != Curve::Linear,
            "belongs to a line or area layer",
        ),
        (
            "name",
            layer.name.is_some(),
            "is not used; a zone is named by its label",
        ),
    ] {
        if present {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                format!("{field} {reason}"),
            ));
        }
    }
    let resolve = |field: &str, time: Option<&TimeValue>| {
        time.map(|time| {
            time.resolve(zone).map_err(|message| {
                ChartError::new("invalid_time", format!("{path}/{field}"), message)
            })
        })
        .transpose()
    };
    let from = resolve("from", layer.from.as_ref())?;
    let to = resolve("to", layer.to.as_ref())?;
    if let Some(bottom) = layer.bottom {
        validate_number(bottom, &format!("{path}/bottom"))?;
    }
    if let Some(top) = layer.top {
        validate_number(top, &format!("{path}/top"))?;
    }
    let edges = [
        layer.bottom.is_some(),
        layer.top.is_some(),
        from.is_some(),
        to.is_some(),
    ];
    if !edges.contains(&true) {
        return Err(ChartError::new(
            "missing_position",
            path,
            "give bottom and/or top for a range of values, from and/or to for a span of time",
        ));
    }
    if let (Some(bottom), Some(top)) = (layer.bottom, layer.top)
        && bottom >= top
    {
        return Err(ChartError::new(
            "invalid_band",
            format!("{path}/top"),
            "top must lie above bottom",
        ));
    }
    if let (Some(from), Some(to)) = (from, to)
        && from >= to
    {
        return Err(ChartError::new(
            "invalid_band",
            format!("{path}/to"),
            "to must lie after from",
        ));
    }
    let Some(label) = &layer.label else {
        return Err(ChartError::new(
            "missing_label",
            format!("{path}/label"),
            "a zone needs a label that says what it marks",
        ));
    };
    validate_text(label, &format!("{path}/label"), 100)
}

/// The name of a data layer: required once the legend lists more than one layer, and unique
/// among the names it is checked against.
pub(crate) fn validate_layer_name(
    layer: &LayerSpec,
    path: &str,
    required: bool,
    names: &mut BTreeSet<String>,
) -> Result<(), ChartError> {
    match layer.name.as_deref() {
        Some(name) => {
            validate_text(name, &format!("{path}/name"), 100)?;
            if !names.insert(name.to_owned()) {
                return Err(ChartError::new(
                    "duplicate_series",
                    format!("{path}/name"),
                    "layer names must be unique within a pane, and across the panes of a time chart",
                ));
            }
        }
        None if required => {
            return Err(ChartError::new(
                "missing_name",
                format!("{path}/name"),
                "name every layer when a pane or a chart has more than one",
            ));
        }
        None => {}
    }
    Ok(())
}

fn validate_layer(
    layer: &LayerSpec,
    path: &str,
    context: &LayerContext,
    names: &mut BTreeSet<String>,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    let (zone, plot_pixels) = (context.zone, context.plot_pixels);
    let mark = match layer.mark {
        Mark::Area => "an area",
        Mark::Point => "a point",
        _ => "a line",
    };

    check_layer_color(layer, path, warnings);

    // Fields that belong to another mark are rejected, never ignored.
    for (field, present, owner) in [
        ("data", !layer.data.is_empty(), "ohlc"),
        ("from", layer.from.is_some(), "band"),
        ("to", layer.to.is_some(), "band"),
        ("top", layer.top.is_some(), "band"),
        ("bottom", layer.bottom.is_some(), "band"),
        ("time", layer.time.is_some(), "annotation"),
        ("value", layer.value.is_some(), "annotation"),
        ("label", layer.label.is_some(), "annotation"),
        ("shape", layer.shape.is_some(), "annotation"),
    ] {
        if present {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                format!("{field} belongs to a {owner} layer, not to {mark} layer"),
            ));
        }
    }
    // Points draw no line, so nothing that shapes a line applies to them.
    if layer.mark == Mark::Point {
        for (field, present) in [
            ("dash", layer.dash.is_some()),
            ("stroke", layer.stroke != Stroke::Regular),
            ("curve", layer.curve != Curve::Linear),
            ("modeled", layer.modeled),
        ] {
            if present {
                return Err(ChartError::new(
                    "option_not_supported",
                    format!("{path}/{field}"),
                    format!("{field} shapes a line; a point layer draws none"),
                ));
            }
        }
        if let Some(index) = layer
            .points
            .iter()
            .position(|point| point.lower.is_some() || point.upper.is_some())
        {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/points/{index}"),
                "lower and upper draw a band along a line; a point layer draws none",
            ));
        }
    }

    validate_layer_name(layer, path, context.named, names)?;

    if layer.points.is_empty() {
        return Err(ChartError::new(
            "empty_series",
            format!("{path}/points"),
            "provide at least one observation",
        ));
    }
    if layer.points.len() > MAX_TIME_POINTS_PER_LAYER {
        return Err(ChartError::new(
            "too_many_data_points",
            format!("{path}/points"),
            format!("at most {MAX_TIME_POINTS_PER_LAYER} observations are supported per layer"),
        ));
    }

    validate_points(layer, path, zone, warnings)?;
    validate_step_end(layer, path, zone)?;

    if !layer.has_two_points(zone) {
        return Err(ChartError::new(
            "empty_series",
            format!("{path}/points"),
            if layer.mark == Mark::Point {
                "a point layer needs at least two observations"
            } else {
                "a line needs at least two observations"
            },
        ));
    }
    // A time axis is meant to carry many observations, so the density warning starts where the
    // drawing stops resolving them instead of at a fixed count.
    if layer.points.len() > plot_pixels {
        warnings.push(ChartWarning::new(
            "dense_chart",
            format!("{path}/points"),
            format!(
                "{} observations exceed the {plot_pixels} horizontal pixels of the plot; the line cannot show every point at this width",
                layer.points.len()
            ),
        ));
    }
    Ok(())
}
