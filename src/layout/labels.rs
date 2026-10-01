use super::{LABEL_SIZE, PlotArea, timechart::TimeFrame};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Text, TextAnchor},
    spec::LayerRef,
};

/// A label's box on the canvas: left, top, right and bottom edge.
pub(super) type LabelBox = (f64, f64, f64, f64);

/// What the label of an annotation has to keep clear of: the plot's edges, the labels placed
/// before it, the data lines of its pane and the point marker symbols.
pub(super) struct LabelSpace {
    plot: PlotArea,
    pub(super) labels: Vec<LabelBox>,
    lines: Vec<Vec<(f64, f64)>>,
    pub(super) symbols: Vec<LabelBox>,
}

/// Why a label collides, if it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Collision {
    Outside,
    Label,
    Line,
}

impl LabelSpace {
    /// The data lines of a pane as drawn, but only when the pane carries an annotation whose
    /// label could run into them.
    pub(super) fn new(frame: &TimeFrame, entries: &[LayerRef]) -> Self {
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

    pub(super) fn hits_symbol(&self, label: LabelBox) -> bool {
        self.symbols
            .iter()
            .any(|other| boxes_overlap(label, *other))
    }

    /// The first collision of a label, checked against the data lines only when `lines` asks for
    /// it.
    pub(super) fn collision(&self, label: LabelBox, lines: bool) -> Option<Collision> {
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
    pub(super) fn place(
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
pub(super) fn label_box(text: &Text, metrics: &impl TextMetrics) -> LabelBox {
    let width = metrics.width(&text.content, LABEL_SIZE);
    let left = match text.anchor {
        TextAnchor::Start => text.x,
        TextAnchor::Middle => text.x - width / 2.0,
        TextAnchor::End => text.x - width,
    };
    (left, text.y - 9.0, left + width, text.y + 3.0)
}

/// The built-in widths follow Inter, the first font the chart's CSS asks for. A browser without
/// Inter falls back to a system font, whose text can run up to about 8 % wider; the value labels
/// of bar charts and the hatching legend of range bars measure with this reserve, so that such
/// text still keeps clear of its neighbours.
const FALLBACK_RESERVE: f64 = 1.08;

/// Text metrics with [`FALLBACK_RESERVE`] added to every width.
pub(crate) struct WithReserve<'a, M>(pub &'a M);

impl<M: TextMetrics> TextMetrics for WithReserve<'_, M> {
    fn width(&self, text: &str, font_size: f64) -> f64 {
        self.0.width(text, font_size) * FALLBACK_RESERVE
    }
}

pub(super) fn boxes_overlap(a: LabelBox, b: LabelBox) -> bool {
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

pub(super) fn warn_label_overlap(
    entry: LayerRef,
    collision: Collision,
    warnings: &mut Vec<ChartWarning>,
) {
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
