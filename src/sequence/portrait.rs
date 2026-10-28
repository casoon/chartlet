//! The portrait layout: participants side by side, time running down.

use super::{
    BADGE_LIFT, BAR, BOTTOM, COMPACT_LABEL, FIGURE, FRAME_INSET, LAST_SELF_LABEL, LINE, LOOP_ALONG,
    LOOP_OUT, MESSAGE_SIZE, NESTED_BAR, TAG_SIZE,
    draw::{Frame, Header, bar, focusable_participant, guard, head, message_tooltip, polyline},
    label_path,
    model::Model,
    pad,
    track::{Slots, Track, tab_width},
};
use crate::{
    diagram::{BADGE, CHIP_REACH, HEAD, badge, chip, chip_box, pixels, rounded, warn_growth, wrap},
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Element, Line, TextAnchor},
    spec::ChartSpec,
};

/// Participants side by side, time running down.
pub(super) struct Portrait {
    margin: f64,
    column: f64,
    /// Top of the band with the participants' names.
    band_top: f64,
    /// Where the messages begin.
    base: f64,
    lines: Vec<Vec<String>>,
    track: Track,
    pub(super) width: f64,
    pub(super) height: f64,
}

/// How far right of the last lifeline the label of a message of the last participant to itself
/// reaches: the strip that is kept free for it; zero without such a message.
fn last_self_tail(model: &Model, metrics: &impl TextMetrics) -> f64 {
    let last = model.participants() - 1;
    let tail = model
        .labels
        .iter()
        .zip(&model.ends)
        .filter(|(_, (from, to))| from == to && *from == last)
        .map(|(label, _)| {
            let mut ignored = Vec::new();
            wrap(
                label,
                LAST_SELF_LABEL,
                MESSAGE_SIZE,
                metrics,
                &mut ignored,
                "",
            )
            .iter()
            .map(|line| metrics.width(line, MESSAGE_SIZE))
            .fold(0.0, f64::max)
        })
        .fold(0.0, f64::max);
    if tail > 0.0 {
        BAR + 2.0 * CHIP_REACH + tail
    } else {
        0.0
    }
}

impl Portrait {
    pub(super) fn new(
        spec: &ChartSpec,
        model: &Model,
        top: f64,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
    ) -> Self {
        let participants = crate::layout::count(model.participants());
        let needed = model.widest_box() + if model.compact { 4.0 } else { 12.0 };
        let last = model.participants() - 1;
        // A message of the last participant to itself writes its label beyond the lifeline, into
        // the room the half column right of it leaves. When that is too little, a strip of the
        // size of the label is kept free on the right. It does not depend on the canvas that was
        // asked for, so the width the warning names is the width that works.
        let tail = last_self_tail(model, metrics);
        let narrowest = 2.0 * model.margin + needed * participants;
        let natural = f64::from(spec.width).max(narrowest);
        let reserve = if (natural - 2.0 * model.margin) / (2.0 * participants) < tail {
            tail
        } else {
            0.0
        };
        let grid = (f64::from(spec.width) - reserve).max(narrowest);
        let width = grid + reserve;
        let column = (grid - 2.0 * model.margin) / participants;
        let lines: Vec<Vec<String>> = model
            .labels
            .iter()
            .zip(&model.ends)
            .enumerate()
            .map(|(index, (label, (from, to)))| {
                let room = if from == to && *from == last {
                    LAST_SELF_LABEL
                } else if from == to && model.compact {
                    // Beside the loop, as far as the canvas goes.
                    let center = model.margin + column * (crate::layout::count(*from) + 0.5);
                    width - model.margin - center - CHIP_REACH - 8.0
                } else if from == to {
                    column - LOOP_OUT - 8.0
                } else {
                    let span = column * crate::layout::count(from.abs_diff(*to)) - 16.0;
                    if model.compact {
                        span.max(COMPACT_LABEL.min(width - 2.0 * model.margin - 16.0))
                    } else {
                        span
                    }
                };
                wrap(
                    label,
                    room,
                    MESSAGE_SIZE,
                    metrics,
                    warnings,
                    &label_path(index),
                )
            })
            .collect();
        let (head, branch) = Slots::uniform(model.sequence, 24.0, 22.0);
        // A numbered message's label rises above the badge on its arrow.
        let above = 10.0
            + model.gap
            + if model.sequence.numbered {
                BADGE_LIFT
            } else {
                0.0
            };
        let slots = Slots {
            arrow: lines
                .iter()
                .map(|lines| LINE * crate::layout::count(lines.len()) + above)
                .collect(),
            slot: lines
                .iter()
                .zip(&model.ends)
                .map(|(lines, (from, to))| {
                    LINE * crate::layout::count(lines.len())
                        + above
                        + if from == to { LOOP_ALONG + 12.0 } else { 10.0 }
                })
                .collect(),
            head,
            branch,
            foot: 10.0,
        };
        let band_top = top + if model.has_actor { FIGURE } else { 0.0 };
        let base = band_top + model.band + 14.0;
        let track = Track::fitted(model, &slots, f64::from(spec.height) - base - 8.0 - BOTTOM);
        let height = f64::from(spec.height).max(base + track.length + 8.0 + BOTTOM);
        Self {
            margin: model.margin,
            column,
            band_top,
            base,
            lines,
            track,
            width,
            height,
        }
    }

    fn center(&self, participant: usize) -> f64 {
        self.margin + self.column * (crate::layout::count(participant) + 0.5)
    }

    pub(super) fn draw(
        &self,
        spec: &ChartSpec,
        model: &Model,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
        elements: &mut Vec<Element>,
    ) -> (u32, u32) {
        warn_growth(spec, self.width, self.height, warnings);
        let sequence = model.sequence;
        let end = self.base + self.track.length + 8.0;
        for participant in 0..model.participants() {
            elements.push(Element::Group(model.participant_classes(participant)));
            let x = self.center(participant);
            elements.push(Element::Line(Line {
                x1: x,
                y1: self.band_top + model.band,
                x2: x,
                y2: end,
                class: "chartlet-seq-lifeline",
            }));
            elements.push(Element::GroupEnd);
        }
        for activation in &model.activations {
            elements.push(Element::Group(
                model.participant_classes(activation.participant),
            ));
            let from = self.base + self.track.arrows[activation.from];
            let to = self.base + self.track.arrows[activation.to];
            elements.push(bar(
                self.center(activation.participant) - BAR
                    + NESTED_BAR * crate::layout::count(activation.level),
                from,
                2.0 * BAR,
                (to - from).max(12.0),
            ));
            elements.push(Element::GroupEnd);
        }
        for (index, fragment) in sequence.fragments.iter().enumerate() {
            let (low, high) = model.span(fragment);
            let half = (self.column / 2.0 - 4.0).min(72.0);
            let inset = FRAME_INSET * crate::layout::count(model.depths[index]);
            let (top, bottom) = self.track.frames[index];
            let left = self.center(low) - half + inset;
            // A frame over few participants grows to the right, as far as the canvas allows, so
            // that its label is not shortened.
            let needed = fragment.label.as_ref().map_or(0.0, |label| {
                tab_width(fragment, metrics) + 12.0 + metrics.width(&format!("[{label}]"), TAG_SIZE)
            });
            let right = (self.center(high) + half - inset)
                .max((left + needed).min(self.width - model.margin));
            let frame = Frame {
                left,
                top: self.base + top,
                right,
                bottom: self.base + bottom + 4.0,
                label_end: right,
            };
            frame.draw(index, fragment, metrics, warnings, elements);
            for (number, (branch, at)) in fragment
                .branches
                .iter()
                .zip(&self.track.branches[index])
                .enumerate()
            {
                let path = format!("/sequence/fragments/{index}/else/{number}/label");
                let y = self.base + at + 4.0;
                elements.push(Element::Line(Line {
                    x1: frame.left,
                    y1: y,
                    x2: frame.right,
                    y2: y,
                    class: "chartlet-seq-branch",
                }));
                if let Some(label) = &branch.label {
                    let room = frame.right - frame.left - 16.0;
                    elements.push(guard(
                        (frame.left + 8.0, y + 14.0),
                        label,
                        room,
                        metrics,
                        warnings,
                        &path,
                    ));
                }
            }
        }
        for index in 0..sequence.messages.len() {
            elements.push(Element::Group(model.message_classes(index)));
            self.draw_message(model, index, metrics, elements);
            elements.push(Element::GroupEnd);
        }
        let gap = if model.compact { 4.0 } else { 12.0 };
        let box_width = |participant: usize| model.box_widths[participant].min(self.column - gap);
        for index in 0..sequence.participants.len() {
            let width = box_width(index);
            let header = Header {
                index,
                x: self.center(index) - width / 2.0,
                y: self.band_top,
                width,
                height: model.band,
                figure: (self.center(index), self.band_top - FIGURE),
                wrap: model.compact,
                pad: pad(model.compact),
            };
            focusable_participant(model, &header, metrics, warnings, elements);
        }
        (pixels(self.width), pixels(self.height))
    }

    fn draw_message(
        &self,
        model: &Model,
        index: usize,
        metrics: &impl TextMetrics,
        elements: &mut Vec<Element>,
    ) {
        let (from, to) = model.ends[index];
        let kind = model.sequence.messages[index].kind;
        let y = self.base + self.track.arrows[index];
        let tooltip = Some(message_tooltip(model, index));
        let lines = &self.lines[index];
        let offset = |participant: usize| {
            if model.active(participant, index) {
                BAR
            } else {
                0.0
            }
        };
        if from == to {
            let x = self.center(from) + offset(from);
            elements.push(Element::Polyline(polyline(
                rounded(&[
                    (x, y),
                    (x + LOOP_OUT, y),
                    (x + LOOP_OUT, y + LOOP_ALONG),
                    (x + HEAD, y + LOOP_ALONG),
                ]),
                kind,
                tooltip,
            )));
            elements.push(Element::Polyline(head(
                (x, y + LOOP_ALONG),
                (-1.0, 0.0),
                kind,
            )));
            let lift = if model.sequence.numbered {
                BADGE_LIFT
            } else {
                0.0
            };
            let first = y - 9.0 - model.gap - lift - LINE * crate::layout::count(lines.len() - 1);
            chip(
                lines,
                (x + CHIP_REACH, first),
                TextAnchor::Start,
                metrics,
                elements,
            );
            if model.sequence.numbered {
                badge(index + 1, (x + LOOP_OUT, y + LOOP_ALONG / 2.0), elements);
            }
            return;
        }
        let direction = if to > from { 1.0 } else { -1.0 };
        let x1 = self.center(from) + direction * offset(from);
        let x2 = self.center(to) - direction * offset(to);
        elements.push(Element::Polyline(polyline(
            vec![(x1, y), (x2 - direction * HEAD, y)],
            kind,
            tooltip,
        )));
        elements.push(Element::Polyline(head((x2, y), (direction, 0.0), kind)));
        let lift = if model.sequence.numbered {
            BADGE_LIFT
        } else {
            0.0
        };
        let first = y - 9.0 - model.gap - lift - LINE * crate::layout::count(lines.len() - 1);
        // A label wider than the span between its lifelines stays inside the canvas.
        let middle = f64::midpoint(x1, x2);
        let (chip_left, _, chip_width, _) =
            chip_box(lines, (middle, first), TextAnchor::Middle, metrics);
        let shift = (model.margin - chip_left).max(0.0)
            - (chip_left + chip_width - (self.width - model.margin)).max(0.0);
        chip(
            lines,
            (middle + shift, first),
            TextAnchor::Middle,
            metrics,
            elements,
        );
        if model.sequence.numbered {
            badge(index + 1, (x1 + direction * (BADGE + 8.0), y), elements);
        }
    }
}
