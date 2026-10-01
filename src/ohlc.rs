//! Candlesticks: one wick from low to high and one body from open to close per observation. A
//! rising candle (close at or above open) has a hollow body and a falling one a filled body, so
//! the direction never rests on the rise and fall colors alone. Once the candles sit closer than
//! three pixels apart, only the wicks are drawn.

use std::collections::BTreeSet;

use crate::{
    error::{ChartError, ChartWarning},
    layout::{TimeFrame, format_value, plot_pixels, tooltip_name, tooltips_fit},
    scene::{Element, Line, Rect},
    spec::{
        ChartSpec, LayerContext, LayerRef, LayerSpec, MAX_TIME_POINTS_PER_LAYER, NumberStyle,
        Stroke, ValueFormat, validate_layer_name, validate_number,
    },
    text::{self, CandleSummary},
    time::{Precision, TimeZone},
};

/// A body is never wider than this, however far apart the candles are.
const MAX_BODY: f64 = 16.0;
/// Share of the distance between two candles their bodies take; the rest is the gap.
const BODY_SHARE: f64 = 0.7;
/// Below this many pixels per candle the bodies cannot be told apart.
const PIXELS_PER_CANDLE: u32 = 3;

/// The number of candles above which a chart of `width` draws wicks only.
pub(crate) const fn dense_limit(width: u32) -> u32 {
    plot_pixels(width) / PIXELS_PER_CANDLE
}

/// An `ohlc` layer: no field of another mark, a name by the rules of every data layer, and two to
/// 2000 candles whose low lies at or below open and close, whose high lies at or above them, and
/// whose timestamps increase.
pub(crate) fn validate(
    layer: &LayerSpec,
    path: &str,
    context: &LayerContext,
    names: &mut BTreeSet<String>,
    warnings: &mut Vec<ChartWarning>,
) -> Result<(), ChartError> {
    for (field, present, reason) in [
        (
            "points",
            !layer.points.is_empty(),
            "belongs to a line or area layer; candles go in data",
        ),
        (
            "color",
            layer.color.is_some(),
            "does not apply to candles; they take --chartlet-rise and --chartlet-fall, which a page can override",
        ),
        ("from", layer.from.is_some(), "belongs to a band layer"),
        ("to", layer.to.is_some(), "belongs to a band layer"),
        ("top", layer.top.is_some(), "belongs to a band layer"),
        ("bottom", layer.bottom.is_some(), "belongs to a band layer"),
        (
            "time",
            layer.time.is_some(),
            "belongs to an annotation layer",
        ),
        (
            "value",
            layer.value.is_some(),
            "belongs to an annotation layer",
        ),
        (
            "label",
            layer.label.is_some(),
            "belongs to an annotation layer",
        ),
        (
            "shape",
            layer.shape.is_some(),
            "belongs to an annotation layer",
        ),
        ("modeled", layer.modeled, "belongs to a line layer"),
        (
            "stroke",
            layer.stroke != Stroke::Regular,
            "belongs to a line layer",
        ),
        ("dash", layer.dash.is_some(), "belongs to a line layer"),
    ] {
        if present {
            return Err(ChartError::new(
                "option_not_supported",
                format!("{path}/{field}"),
                format!("{field} {reason}"),
            ));
        }
    }
    validate_layer_name(layer, path, context.named, names)?;

    if layer.data.len() > MAX_TIME_POINTS_PER_LAYER {
        return Err(ChartError::new(
            "too_many_data_points",
            format!("{path}/data"),
            format!("at most {MAX_TIME_POINTS_PER_LAYER} candles are supported per layer"),
        ));
    }
    validate_candles(layer, path, context.zone)?;
    if layer.data.len() < 2 {
        return Err(ChartError::new(
            "empty_series",
            format!("{path}/data"),
            "a candlestick layer needs at least two candles",
        ));
    }
    let limit = context.plot_pixels / usize::try_from(PIXELS_PER_CANDLE).expect("a small number");
    if layer.data.len() > limit {
        warnings.push(ChartWarning::new(
            "dense_chart",
            format!("{path}/data"),
            format!(
                "{} candles leave less than {PIXELS_PER_CANDLE} pixels each; only their wicks from low to high are drawn",
                layer.data.len()
            ),
        ));
    }
    Ok(())
}

/// Every candle resolves to a timestamp after the one before it, carries finite numbers, and
/// keeps its low and its high outside the body.
fn validate_candles(layer: &LayerSpec, path: &str, zone: TimeZone) -> Result<(), ChartError> {
    let mut previous: Option<i64> = None;
    for (index, candle) in layer.data.iter().enumerate() {
        let candle_path = format!("{path}/data/{index}");
        let epoch = candle.time.resolve(zone).map_err(|message| {
            ChartError::new("invalid_time", format!("{candle_path}/time"), message)
        })?;
        for (field, value) in [
            ("open", candle.open),
            ("high", candle.high),
            ("low", candle.low),
            ("close", candle.close),
        ] {
            validate_number(value, &format!("{candle_path}/{field}"))?;
        }
        if candle.low > candle.open.min(candle.close) {
            return Err(ChartError::new(
                "invalid_candle",
                format!("{candle_path}/low"),
                "low must not lie above open or close",
            ));
        }
        if candle.high < candle.open.max(candle.close) {
            return Err(ChartError::new(
                "invalid_candle",
                format!("{candle_path}/high"),
                "high must not lie below open or close",
            ));
        }
        if previous.is_some_and(|previous| epoch <= previous) {
            return Err(ChartError::new(
                "unordered_time",
                format!("{candle_path}/time"),
                "timestamps must increase from candle to candle",
            ));
        }
        previous = Some(epoch);
    }
    Ok(())
}

/// The width of every body: a share of the median distance between neighbouring candles, at
/// least one pixel and at most [`MAX_BODY`].
fn body_width(xs: &[f64]) -> f64 {
    let mut distances: Vec<f64> = xs
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .collect();
    if distances.is_empty() {
        return MAX_BODY;
    }
    distances.sort_by(f64::total_cmp);
    let middle = distances.len() / 2;
    let median = if distances.len().is_multiple_of(2) {
        f64::midpoint(distances[middle - 1], distances[middle])
    } else {
        distances[middle]
    };
    (median * BODY_SHARE).clamp(1.0, MAX_BODY)
}

/// Draws the candles of one layer: every wick, and while there is room for them, the bodies with
/// a tooltip each.
pub(crate) fn push_candles(
    spec: &ChartSpec,
    entry: LayerRef,
    frame: &TimeFrame,
    elements: &mut Vec<Element>,
) {
    let candles = entry.layer.resolved_candles(frame.zone);
    let xs: Vec<f64> = candles.iter().map(|(epoch, _)| frame.x(*epoch)).collect();
    let dense = candles.len()
        > usize::try_from(dense_limit(spec.width)).expect("a usize is at least 32 bits wide");
    let width = body_width(&xs);
    let with_tooltips = tooltips_fit(&xs);
    let name = tooltip_name(spec, entry);
    for ((epoch, [open, high, low, close]), x) in candles.iter().zip(xs) {
        let rising = close >= open;
        elements.push(Element::Line(Line {
            x1: x,
            y1: frame.y(*high),
            x2: x,
            y2: frame.y(*low),
            class: if rising {
                "chartlet-wick chartlet-wick-rise"
            } else {
                "chartlet-wick chartlet-wick-fall"
            },
        }));
        if dense {
            continue;
        }
        let top = frame.y(open.max(*close));
        let bottom = frame.y(open.min(*close));
        // Open and close alike still leave a visible line.
        let (y, height) = if bottom - top < 1.0 {
            (f64::midpoint(top, bottom) - 0.5, 1.0)
        } else {
            (top, bottom - top)
        };
        let values = [open, high, low, close].map(|value| format_value(*value, frame.style));
        let values = text::candle_values(spec.locale, &values);
        let time = frame.precision.format(*epoch, frame.zone);
        elements.push(Element::Rect(Rect {
            x: x - width / 2.0,
            y,
            width,
            height,
            class: if rising {
                "chartlet-candle chartlet-candle-rise"
            } else {
                "chartlet-candle chartlet-candle-fall"
            },
            series_index: None,
            style_index: None,
            tooltip: with_tooltips.then(|| match &name {
                Some(name) => format!("{time} – {name}: {values}"),
                None => format!("{time}: {values}"),
            }),
        }));
    }
}

/// The legend sample of a candlestick layer: a rising, hollow candle beside a falling, filled one.
/// `y` is the top of the legend row.
pub(crate) fn push_legend_sample(x: f64, y: f64, elements: &mut Vec<Element>) {
    for (center, top, height, rising) in [(x + 6.0, y, 8.0, true), (x + 18.0, y + 2.0, 7.0, false)]
    {
        let (wick, body) = if rising {
            (
                "chartlet-wick chartlet-wick-rise",
                "chartlet-candle chartlet-candle-rise",
            )
        } else {
            (
                "chartlet-wick chartlet-wick-fall",
                "chartlet-candle chartlet-candle-fall",
            )
        };
        elements.push(Element::Line(Line {
            x1: center,
            y1: y - 3.0,
            x2: center,
            y2: y + 12.0,
            class: wick,
        }));
        elements.push(Element::Rect(Rect {
            x: center - 3.0,
            y: top,
            width: 6.0,
            height,
            class: body,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
}

/// A change written with its sign: a plus for a rise, the true minus for a fall.
fn signed(value: f64, style: NumberStyle) -> String {
    let written = format_value(value, style);
    if value > 0.0 {
        format!("+{written}")
    } else {
        written
    }
}

/// The sentence about one candlestick layer: first open, last close, the change between them and
/// the extremes, each with its time.
pub(crate) fn describe(
    spec: &ChartSpec,
    entry: LayerRef,
    zone: TimeZone,
    precision: Precision,
    style: NumberStyle,
) -> String {
    let candles = entry.layer.resolved_candles(zone);
    let (first, last) = (
        candles
            .first()
            .expect("validated candle layers have candles"),
        candles
            .last()
            .expect("validated candle layers have candles"),
    );
    let pick = |index: usize, better: fn(f64, f64) -> bool| {
        candles.iter().fold(first, |best, candle| {
            if better(candle.1[index], best.1[index]) {
                candle
            } else {
                best
            }
        })
    };
    let high = pick(1, |candidate, best| candidate > best);
    let low = pick(2, |candidate, best| candidate < best);
    let (open, close) = (first.1[0], last.1[3]);
    let show = |value: f64| format_value(value, style);
    let at = |epoch: i64| precision.format(epoch, zone);
    let percent = (open != 0.0).then(|| {
        signed(
            (close - open) / open.abs(),
            NumberStyle {
                format: ValueFormat::Percent,
                decimals: Some(1),
                locale: spec.locale,
                thousands: false,
            },
        )
    });
    let values = [show(open), show(close), show(high.1[1]), show(low.1[2])];
    let times = [at(first.0), at(last.0), at(high.0), at(low.0)];
    text::candles(
        spec.locale,
        &CandleSummary {
            name: entry.layer.name.as_deref(),
            open: (&values[0], &times[0]),
            close: (&values[1], &times[1]),
            change: &signed(close - open, style),
            percent: percent.as_deref(),
            high: (&values[2], &times[2]),
            low: (&values[3], &times[3]),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::body_width;

    #[test]
    fn bodies_take_a_share_of_the_median_distance_within_their_bounds() {
        assert!((body_width(&[0.0, 10.0, 20.0, 50.0]) - 7.0).abs() < 1e-9);
        assert!((body_width(&[0.0, 100.0]) - 16.0).abs() < 1e-9);
        assert!((body_width(&[0.0, 1.0, 2.0]) - 1.0).abs() < 1e-9);
    }
}
