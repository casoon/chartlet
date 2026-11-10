use super::{
    LABEL_SIZE,
    axis::format_value,
    count, fit_text,
    labels::{Collision, LabelBox, LabelSpace, boxes_overlap, label_box, warn_label_overlap},
    timechart::TimeFrame,
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Circle, Element, Hook, Polyline, Rect, Text, TextAnchor},
    spec::{ChartSpec, LayerRef, LayerSpec, NumberStyle, Shape},
    time::{Precision, TimeZone},
};

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
        let precision = Precision::of(epochs.into_iter(), zone).at_least(spec.time_precision(zone));
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
pub(crate) fn marker_position(
    layer: &LayerSpec,
    zone: TimeZone,
    chart: Precision,
    style: NumberStyle,
) -> String {
    let epoch = layer
        .time
        .as_ref()
        .and_then(|time| time.resolve(zone).ok())
        .expect("validated point markers carry a time");
    let value = layer.value.expect("validated point markers carry a value");
    format!(
        "{}, {}",
        Precision::of(std::iter::once(epoch), zone)
            .at_least(chart)
            .format(epoch, zone),
        format_value(value, style)
    )
}

/// A zone: a shaded rectangle behind the data between its edges, each missing edge taken from
/// the plot. In a chart with candles its time edges lie at the outer edges of the candles' room
/// instead of their middles, so `from` equal to `to` marks a single candle. Its label sits inside
/// the zone's top left corner when the zone is tall enough; where another label or a data line
/// is in the way there, at its bottom, or a line further in from either. A zone too low for a
/// line of text takes its label just outside its upper or lower edge.
#[allow(clippy::too_many_arguments)]
pub(super) fn push_zone(
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
    let (start, end) = frame.edges();
    // Among candles a time is as wide as a candle: the zone covers the candles at its edges in
    // full, half the room of a candle beyond their middles, and stays inside the plot.
    let reach = crate::ohlc::half_slot(spec, entry.pane, frame)
        .map_or(0.0, |half| half.copysign(end - start));
    let inside = |x: f64| x.clamp(plot.left, plot.left + plot.width);
    let from = resolve(&layer.from).map_or(start, |epoch| inside(frame.x(epoch) - reach));
    let to = resolve(&layer.to).map_or(end, |epoch| inside(frame.x_until(epoch) + reach));
    // On an axis that runs from right to left, the zone starts at its right edge.
    let (left, right) = (from.min(to), from.max(to));
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
    let at = |y: f64| Text {
        x,
        y,
        class: "chartlet-rule-label",
        anchor,
        content: content.clone(),
    };
    // A zone too low for a line of text takes its label just above it, or below.
    let candidates = if bottom - top < 20.0 {
        let (above, below) = (at(top - 5.0), at(bottom + 13.0));
        if top - 17.0 >= plot.top {
            vec![above, below]
        } else {
            vec![below]
        }
    } else {
        // Inside the zone: at its top, then at its bottom, then a line further in from either.
        let rows = label_rows(bottom - top - 8.0).min(count(STAGGER_ROWS));
        (0..STAGGER_ROWS)
            .map(count)
            .take_while(|row| *row < rows.max(1.0))
            .flat_map(|row| [at(top + 14.0 + row * LINE), at(bottom - 6.0 - row * LINE)])
            .collect()
    };
    labels.push(Element::Text(
        space.settle(candidates, entry, metrics, warnings),
    ));
}

/// How many lines of label text fit into `height` pixels.
fn label_rows(height: f64) -> f64 {
    (height / LINE).floor()
}

/// The symbol of a point marker, in its declared shape. Shape and label carry its meaning, so a
/// marker never depends on its color alone.
pub(super) fn push_marker(
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
        marker_position(layer, frame.zone, frame.precision, frame.style)
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
pub(super) fn push_marker_label(
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

/// A reference line across the plot: horizontal at a value, vertical at a time. Its label is
/// collected separately and drawn after the data. It takes the first place that is clear of the
/// plot's edges, the labels before it and the data lines; a collision that no place avoids is
/// reported.
///
/// A vertical line writes its label beside its top end, on the right, or on the left in the right
/// part of the plot. Labels that would overlap step down line by line, so that neighbouring
/// lines read as a staircase, each label beside its own line; a label that finds no line of
/// text free there tries the other side of its line.
///
/// A horizontal line writes its label above its left end; then right of the labels in its way,
/// above its right end, and below the line at either end.
pub(super) fn push_rule(
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
    let text = |x: f64, y: f64, anchor: TextAnchor, content: &str| Text {
        x,
        y,
        class: "chartlet-rule-label",
        anchor,
        content: content.to_owned(),
    };
    let (points, candidates, tooltip) = if let Some(value) = layer.value {
        let y = frame.y(value);
        let content = fit_text(
            label,
            plot.width / 2.0,
            LABEL_SIZE,
            metrics,
            warnings,
            &path,
        );
        let (start, end) = (plot.left + 4.0, plot.left + plot.width - 4.0);
        let mut candidates = vec![text(start, y - 6.0, TextAnchor::Start, &content)];
        // Right of whatever label is in the way, as long as there is one and room for it.
        for _ in 0..STAGGER_ROWS {
            let last = candidates.last().expect("there is a first candidate");
            let Some(taken) = space.right_of_labels(label_box(last, metrics)) else {
                break;
            };
            candidates.push(text(taken + 8.0, y - 6.0, TextAnchor::Start, &content));
        }
        candidates.push(text(end, y - 6.0, TextAnchor::End, &content));
        candidates.push(text(start, y + LINE, TextAnchor::Start, &content));
        candidates.push(text(end, y + LINE, TextAnchor::End, &content));
        (
            vec![(plot.left, y), (plot.left + plot.width, y)],
            candidates,
            format!("{label}: {}", format_value(value, frame.style)),
        )
    } else {
        let epoch = layer
            .time
            .as_ref()
            .and_then(|time| time.resolve(frame.zone).ok())
            .expect("validated annotations carry a value or a time");
        let x = frame.x(epoch);
        let content = fit_text(
            label,
            plot.width * 0.4,
            LABEL_SIZE,
            metrics,
            warnings,
            &path,
        );
        let width = metrics.width(&content, LABEL_SIZE);
        // A line in the right part of the plot takes its label on its left, so the label stays
        // inside the chart; so does one whose label would leave the plot on the right.
        let left = x > plot.left + plot.width * 0.7 || x + 4.0 + width > plot.left + plot.width;
        let sides = if left && x - 4.0 - width >= plot.left {
            [(x - 4.0, TextAnchor::End), (x + 4.0, TextAnchor::Start)]
        } else {
            [(x + 4.0, TextAnchor::Start), (x - 4.0, TextAnchor::End)]
        };
        // Down to the middle of the plot at most: further down a label no longer reads as the
        // head of its line. The other side of the line comes only after every line of text on
        // the first: side by side, two labels between two close lines could not be told apart.
        let rows = label_rows(plot.height / 2.0).clamp(1.0, count(STAGGER_ROWS));
        let candidates = sides
            .into_iter()
            .flat_map(|(label_x, anchor)| {
                let content = &content;
                (0..STAGGER_ROWS)
                    .map(count)
                    .take_while(move |row| *row < rows)
                    .map(move |row| text(label_x, plot.top + 12.0 + row * LINE, anchor, content))
            })
            .collect();
        let precision = Precision::of(std::iter::once(epoch), frame.zone).at_least(frame.precision);
        (
            vec![(x, plot.top), (x, bottom)],
            candidates,
            format!("{label}: {}", precision.format(epoch, frame.zone)),
        )
    };
    elements.push(Element::Polyline(Polyline {
        points,
        class: "chartlet-rule",
        series_index: None,
        style_index: layer.resolved_color().is_some().then_some(entry.global),
        tooltip: Some(tooltip),
    }));
    labels.push(Element::Text(
        space.settle(candidates, entry, metrics, warnings),
    ));
}

/// What a measure spans, for its tooltip and the description: both values and their difference.
pub(crate) fn measure_extent(spec: &ChartSpec, layer: &LayerSpec, style: NumberStyle) -> String {
    let (bottom, top) = (
        layer.bottom.expect("validated measures carry a bottom"),
        layer.top.expect("validated measures carry a top"),
    );
    crate::text::measure(
        spec.locale,
        &format_value(bottom, style),
        &format_value(top, style),
        &format_value(top - bottom, style),
    )
}

/// Half the width of the strokes that end a measure.
const MEASURE_END: f64 = 4.0;
/// Space between a measure and what it stands beside: the candle, its own label, the next column.
const MEASURE_GAP: f64 = 8.0;
/// The distance between two lines of label text: those of a label that is broken, and those of
/// labels that step out of each other's way.
const LINE: f64 = 14.0;
/// How many lines a label steps away from its first place before it gives up.
const STAGGER_ROWS: usize = 4;

/// One measure as it is placed: its layer, the time and position it measures at, the pixels it
/// spans, the column it stands in and, once its column is laid out, the width of its label.
struct Measure<'a> {
    entry: LayerRef<'a>,
    epoch: i64,
    x: f64,
    span: (f64, f64),
    column: usize,
    width: f64,
}

impl Measure<'_> {
    /// The baseline of the label: centred on the span, and kept inside the plot.
    fn baseline(&self, frame: &TimeFrame) -> f64 {
        let plot = frame.plot;
        let highest = plot.top + 9.0;
        (f64::midpoint(self.span.0, self.span.1) + 4.0)
            .clamp(highest, (plot.top + plot.height - 3.0).max(highest))
    }

    /// Whether this measure and `other` can share a column: neither their spans nor their
    /// labels overlap. Spans that only touch, such as a body and the wick above it, do not.
    fn fits_beside(&self, other: &Self, frame: &TimeFrame) -> bool {
        let apart = |a: (f64, f64), b: (f64, f64)| a.1 <= b.0 || b.1 <= a.0;
        let rows = |measure: &Self| {
            let baseline = measure.baseline(frame);
            (baseline - 9.0, baseline + 3.0)
        };
        apart(self.span, other.span) && apart(rows(self), rows(other))
    }

    /// Whether the measure stands right of its position. Columns alternate between the two
    /// sides, starting on the right; in the right part of the plot the left side comes first, so
    /// the labels stay inside.
    fn right(&self, frame: &TimeFrame) -> bool {
        let right_first = self.x <= frame.plot.left + frame.plot.width * 0.7;
        self.column.is_multiple_of(2) == right_first
    }

    /// Whether `other` stands at the same position, on the same side and nearer to it.
    fn is_behind(&self, other: &Self) -> bool {
        other.epoch == self.epoch
            && other.column % 2 == self.column % 2
            && other.column < self.column
    }
}

/// The measures of a pane, each with the first column whose measures at the same time leave
/// room for it; in the order in which their columns are laid out, from the position outward.
fn measure_columns<'a>(entries: &[LayerRef<'a>], frame: &TimeFrame) -> Vec<Measure<'a>> {
    let mut measures: Vec<Measure> = Vec::new();
    for entry in entries.iter().filter(|entry| entry.layer.is_measure()) {
        let layer = entry.layer;
        let epoch = layer
            .time
            .as_ref()
            .and_then(|time| time.resolve(frame.zone).ok())
            .expect("validated measures carry a time");
        let (top, bottom) = (
            frame.y(layer.top.expect("validated measures carry a top")),
            frame.y(layer.bottom.expect("validated measures carry a bottom")),
        );
        let mut measure = Measure {
            entry: *entry,
            epoch,
            x: frame.x(epoch),
            // On a reversed value axis the top lies below the bottom.
            span: (top.min(bottom), top.max(bottom)),
            column: 0,
            width: 0.0,
        };
        while measures.iter().any(|other| {
            other.epoch == measure.epoch
                && other.column == measure.column
                && !measure.fits_beside(other, frame)
        }) {
            measure.column += 1;
        }
        measures.push(measure);
    }
    measures.sort_by_key(|measure| measure.column);
    measures
}

/// The measures of a pane: for each a vertical line from `bottom` to `top` with a short stroke
/// at either end and its label beside it. They stand beside the position they measure, clear of
/// a candle's body: the first to its right, one that would overlap it to its left, further ones
/// alternately further out on either side. Measures that do not overlap share a column, as the
/// body of a candle and its two wicks do. A label takes the room its side of the plot has left:
/// one line where it fits, two lines broken at a space where it does not, and shortened last.
#[allow(clippy::too_many_arguments)]
pub(super) fn push_measures(
    spec: &ChartSpec,
    entries: &[LayerRef],
    frame: &TimeFrame,
    elements: &mut Vec<Element>,
    labels: &mut Vec<Element>,
    space: &mut LabelSpace,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    let plot = frame.plot;
    let mut measures = measure_columns(entries, frame);
    let Some(pane) = measures.first().map(|measure| measure.entry.pane) else {
        return;
    };
    let clear = crate::ohlc::half_body(spec, pane, frame) + MEASURE_GAP;
    for index in 0..measures.len() {
        let measure = &measures[index];
        let inner = nearer_columns(&measures, index);
        let right = measure.right(frame);
        let side = if right { 1.0 } else { -1.0 };
        let x = measure.x + side * (clear + MEASURE_END + inner);
        let label_x = x + side * (MEASURE_END + MEASURE_GAP);
        let room = if right {
            plot.left + plot.width - label_x
        } else {
            label_x - plot.left
        };
        let (top, bottom) = measure.span;
        let entry = measure.entry;
        let label = entry
            .layer
            .label
            .as_deref()
            .expect("validated measures carry a label");
        let texts = measure_label(
            entry,
            (label_x, measure.baseline(frame), room),
            right,
            frame,
            warnings,
            metrics,
        );
        measures[index].width = texts
            .iter()
            .map(|text| metrics.width(&text.content, LABEL_SIZE))
            .fold(0.0, f64::max);

        elements.push(Element::Hook(Hook::Layer(entry.global)));
        elements.push(Element::Polyline(Polyline {
            points: vec![
                (x - MEASURE_END, top),
                (x + MEASURE_END, top),
                (x, top),
                (x, bottom),
                (x - MEASURE_END, bottom),
                (x + MEASURE_END, bottom),
            ],
            class: "chartlet-measure",
            series_index: None,
            style_index: entry
                .layer
                .resolved_color()
                .is_some()
                .then_some(entry.global),
            tooltip: Some(format!(
                "{label}: {}",
                measure_extent(spec, entry.layer, frame.style)
            )),
        }));
        elements.push(Element::Hook(Hook::End));
        // The line keeps the labels placed after it off itself, as a marker's symbol does.
        let line: LabelBox = (x - MEASURE_END, top, x + MEASURE_END, bottom);
        if space.labels.iter().any(|other| boxes_overlap(line, *other)) {
            warn_label_overlap(entry, Collision::Label, warnings);
        }
        for text in texts {
            space.place(&text, entry, true, metrics, warnings);
            labels.push(Element::Text(text));
        }
        space.symbols.push(line);
    }
}

/// What the columns nearer to its position take from the measure at `index`, on its side: their
/// strokes, their widest labels and the gaps between them. `measures` is ordered by column.
fn nearer_columns(measures: &[Measure], index: usize) -> f64 {
    let measure = &measures[index];
    let nearer: Vec<&Measure> = measures[..index]
        .iter()
        .filter(|other| measure.is_behind(other))
        .collect();
    let mut columns: Vec<usize> = nearer.iter().map(|other| other.column).collect();
    columns.dedup();
    columns
        .iter()
        .map(|column| {
            let widest = nearer
                .iter()
                .filter(|other| other.column == *column)
                .map(|other| other.width)
                .fold(0.0, f64::max);
            2.0 * MEASURE_END + MEASURE_GAP + widest + MEASURE_GAP
        })
        .sum()
}

/// The label of a measure at `(x, baseline)` with `room` beside it: one line where it fits, two
/// lines around that baseline where it fits broken, and shortened where it fits neither way.
fn measure_label(
    entry: LayerRef,
    (x, baseline, room): (f64, f64, f64),
    right: bool,
    frame: &TimeFrame,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Vec<Text> {
    let plot = frame.plot;
    let label = entry
        .layer
        .label
        .as_deref()
        .expect("validated measures carry a label");
    let lines = match broken_label(label, room, metrics) {
        Some(lines) => lines.to_vec(),
        None => vec![fit_text(
            label,
            room.max(0.0),
            LABEL_SIZE,
            metrics,
            warnings,
            &format!("/panes/{}/layers/{}/label", entry.pane, entry.local),
        )],
    };
    // Two lines stand half a line above and below the place of one, inside the plot.
    let shift = if lines.len() == 2 { LINE / 2.0 } else { 0.0 };
    let highest = plot.top + 9.0 + shift;
    let baseline = baseline.clamp(highest, (plot.top + plot.height - 3.0 - shift).max(highest));
    lines
        .into_iter()
        .enumerate()
        .map(|(line, content)| Text {
            x,
            y: baseline - shift + LINE * count(line),
            class: "chartlet-rule-label",
            anchor: if right {
                TextAnchor::Start
            } else {
                TextAnchor::End
            },
            content,
        })
        .collect()
}

/// A label too wide for `room` on two lines, broken at the last space after which both parts
/// fit; `None` when it fits on one line or on neither.
fn broken_label(label: &str, room: f64, metrics: &impl TextMetrics) -> Option<[String; 2]> {
    let fits = |text: &str| metrics.width(text, LABEL_SIZE) <= room;
    if fits(label) {
        return None;
    }
    label
        .match_indices(' ')
        .rev()
        .map(|(index, _)| (label[..index].trim_end(), label[index..].trim_start()))
        .find(|(first, second)| {
            !first.is_empty() && !second.is_empty() && fits(first) && fits(second)
        })
        .map(|(first, second)| [first.to_owned(), second.to_owned()])
}
