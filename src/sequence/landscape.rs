//! The landscape layout: participants one above the other, time running right.

use super::{
    BAR, BOTTOM, FIGURE, FRAME_INSET, LANDSCAPE_LABEL, LINE, LOOP_ALONG, LOOP_OUT, MESSAGE_SIZE,
    NESTED_BAR,
    draw::{Frame, Header, bar, focusable_participant, guard, head, message_tooltip, polyline},
    label_path,
    model::Model,
    pad,
    track::{Slots, Track, make_room_for_guards},
};
use crate::{
    diagram::{BADGE, CHIP_REACH, HEAD, badge, chip, pixels, rounded, warn_growth, wrap},
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Element, Line, TextAnchor},
    spec::{ChartSpec, ParticipantKind},
};

/// Participants one below the other, time running right.
pub(super) struct Landscape {
    /// Width of the participants' boxes in the gutter on the left.
    gutter: f64,
    lane: f64,
    top: f64,
    /// Where the messages begin.
    base: f64,
    lines: Vec<Vec<String>>,
    track: Track,
    pub(super) width: f64,
    pub(super) height: f64,
}

impl Landscape {
    pub(super) fn new(
        spec: &ChartSpec,
        model: &Model,
        top: f64,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
    ) -> Self {
        let participants = crate::layout::count(model.participants());
        // An actor's figure stands left of its name.
        let gutter = model.widest_box() + if model.has_actor { FIGURE } else { 0.0 };
        let min_lane = model.band + 30.0;
        let room = f64::from(spec.height) - top - BOTTOM;
        let lane = (room / participants).min(140.0).max(min_lane);
        let height = f64::from(spec.height).max(top + lane * participants + BOTTOM);
        let lines: Vec<Vec<String>> = model
            .labels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                wrap(
                    label,
                    LANDSCAPE_LABEL,
                    MESSAGE_SIZE,
                    metrics,
                    warnings,
                    &label_path(index),
                )
            })
            .collect();
        let label_width = |lines: &[String]| {
            lines
                .iter()
                .map(|line| metrics.width(line, MESSAGE_SIZE))
                .fold(0.0, f64::max)
        };
        let (head, branch) = Slots::uniform(model.sequence, 14.0, 14.0);
        let mut slots = Slots {
            arrow: vec![6.0; lines.len()],
            slot: lines
                .iter()
                .zip(&model.ends)
                .map(|(lines, (from, to))| {
                    let out = if from == to { LOOP_OUT + 6.0 } else { 6.0 };
                    (6.0 + out + label_width(lines) + 2.0 * CHIP_REACH + 14.0).max(36.0)
                })
                .collect(),
            head,
            branch,
            foot: 12.0,
        };
        make_room_for_guards(model, &mut slots, metrics);
        let base = model.margin + gutter + 16.0;
        let track = Track::fitted(
            model,
            &slots,
            f64::from(spec.width) - model.margin - base - 8.0,
        );
        let width = f64::from(spec.width).max(base + track.length + 8.0 + model.margin);
        Self {
            gutter,
            lane,
            top,
            base,
            lines,
            track,
            width,
            height,
        }
    }

    fn center(&self, participant: usize) -> f64 {
        self.top + self.lane * (crate::layout::count(participant) + 0.5)
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
            let y = self.center(participant);
            elements.push(Element::Line(Line {
                x1: model.margin + self.gutter,
                y1: y,
                x2: end,
                y2: y,
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
                from,
                self.center(activation.participant) - BAR
                    + NESTED_BAR * crate::layout::count(activation.level),
                (to - from).max(12.0),
                2.0 * BAR,
            ));
            elements.push(Element::GroupEnd);
        }
        for (index, fragment) in sequence.fragments.iter().enumerate() {
            let (low, high) = model.span(fragment);
            let inset = FRAME_INSET * crate::layout::count(model.depths[index]);
            let (left, right) = self.track.frames[index];
            let branches = &self.track.branches[index];
            let frame = Frame {
                left: self.base + left,
                top: self.center(low) - self.lane / 2.0 + 4.0 + inset,
                right: self.base + right + 4.0,
                bottom: self.center(high) + self.lane / 2.0 - 4.0 - inset,
                label_end: self.base + branches.first().map_or(right + 4.0, |at| at + 4.0),
            };
            frame.draw(index, fragment, metrics, warnings, elements);
            for (number, (branch, at)) in fragment
                .branches
                .iter()
                .zip(&self.track.branches[index])
                .enumerate()
            {
                let path = format!("/sequence/fragments/{index}/else/{number}/label");
                let x = self.base + at + 4.0;
                elements.push(Element::Line(Line {
                    x1: x,
                    y1: frame.top,
                    x2: x,
                    y2: frame.bottom,
                    class: "chartlet-seq-branch",
                }));
                if let Some(label) = &branch.label {
                    let next = branches.get(number + 1).map_or(right, |at| *at);
                    let room = self.base + next + 4.0 - x - 12.0;
                    elements.push(guard(
                        (x + 6.0, frame.top + 12.0),
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
        for (index, participant) in sequence.participants.iter().enumerate() {
            let center = self.center(index);
            let indent = if participant.kind == ParticipantKind::Actor {
                FIGURE
            } else {
                0.0
            };
            let header = Header {
                index,
                x: model.margin + indent,
                y: center - model.band / 2.0,
                width: self.gutter - indent,
                height: model.band,
                figure: (model.margin + FIGURE / 2.0, center - FIGURE / 2.0),
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
        let x = self.base + self.track.arrows[index];
        let tooltip = Some(message_tooltip(model, index));
        let lines = &self.lines[index];
        let offset = |participant: usize| {
            if model.active(participant, index) {
                BAR
            } else {
                0.0
            }
        };
        let block = LINE * crate::layout::count(lines.len() - 1);
        if from == to {
            let y = self.center(from) + offset(from);
            elements.push(Element::Polyline(polyline(
                rounded(&[
                    (x, y),
                    (x, y + LOOP_ALONG),
                    (x + LOOP_OUT, y + LOOP_ALONG),
                    (x + LOOP_OUT, y + HEAD),
                ]),
                kind,
                tooltip,
            )));
            elements.push(Element::Polyline(head(
                (x + LOOP_OUT, y),
                (0.0, -1.0),
                kind,
            )));
            chip(
                lines,
                (x + LOOP_OUT + 4.0 + model.gap + CHIP_REACH, y + 14.0),
                TextAnchor::Start,
                metrics,
                elements,
            );
            if model.sequence.numbered {
                badge(index + 1, (x + LOOP_OUT / 2.0, y + LOOP_ALONG), elements);
            }
            return;
        }
        let direction = if to > from { 1.0 } else { -1.0 };
        let y1 = self.center(from) + direction * offset(from);
        let y2 = self.center(to) - direction * offset(to);
        elements.push(Element::Polyline(polyline(
            vec![(x, y1), (x, y2 - direction * HEAD)],
            kind,
            tooltip,
        )));
        elements.push(Element::Polyline(head((x, y2), (0.0, direction), kind)));
        // Between the sender's lane and the next one, where no other lifeline crosses it.
        let gap = self.center(from) + direction * self.lane / 2.0;
        chip(
            lines,
            (x + 2.0 + model.gap + CHIP_REACH, gap + 4.0 - block / 2.0),
            TextAnchor::Start,
            metrics,
            elements,
        );
        if model.sequence.numbered {
            badge(index + 1, (x, y1 + direction * (BADGE + 8.0)), elements);
        }
    }
}
