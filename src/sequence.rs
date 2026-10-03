//! Sequence diagrams: participants and the messages they exchange, in the order they are sent.
//!
//! The diagram runs along a time axis, down in portrait and right in landscape. Every message
//! takes a slot on that axis, fragments add a head and a foot around their messages, and the
//! participants spread evenly across the other axis. The description and the data table list
//! every message in order, so that the diagram is never the only place it is said.

use std::fmt::Write as _;

use crate::{
    DataTable,
    diagram::{
        self, BADGE, CHIP_REACH, HEAD, arrowhead, badge, chip, cylinder, pixels, rounded,
        warn_growth, with_shadow, wrap,
    },
    error::ChartWarning,
    layout::{NARROW, fit_text, push_title, title_extra},
    metrics::TextMetrics,
    scene::{Circle, Element, Line, Polyline, Rect, Scene, Text, TextAnchor},
    spec::{
        ChartSpec, DiagramOrientation, FragmentSpec, Locale, MessageKind, ParticipantKind,
        SequenceSpec,
    },
    text,
};

/// Margin around the diagram.
const MARGIN: f64 = 24.0;
/// The margin, and the widest text in a participant's box, of a diagram narrower than
/// [`NARROW`], such as a mobile variant: there a participant's name wraps onto two lines.
const COMPACT_MARGIN: f64 = 12.0;
const COMPACT_TEXT: f64 = 64.0;
/// Space kept free below the diagram.
const BOTTOM: f64 = 16.0;
const LABEL_SIZE: f64 = 13.0;
const SUBLABEL_SIZE: f64 = 11.0;
const MESSAGE_SIZE: f64 = 12.0;
const TAG_SIZE: f64 = 11.0;
/// Distance between the baselines of a message label on two lines.
const LINE: f64 = 15.0;
/// Text inset inside a participant's box.
const PAD: f64 = 12.0;
/// Narrowest and widest box of a participant.
const MIN_BOX: f64 = 72.0;
const COMPACT_BOX: f64 = 56.0;
/// How far a numbered message's label rises so that it clears the badge on its arrow.
const BADGE_LIFT: f64 = 7.0;
const MAX_BOX: f64 = 180.0;
/// Height of the figure drawn above an actor's name.
const FIGURE: f64 = 26.0;
/// Half the width of an activation bar, and how far each nested bar moves aside.
const BAR: f64 = 5.0;
const NESTED_BAR: f64 = 4.0;
/// The loop a message to its sender draws: how far out and how far along it goes.
const LOOP_OUT: f64 = 24.0;
const LOOP_ALONG: f64 = 16.0;
/// The widest message label in landscape, before it wraps.
const LANDSCAPE_LABEL: f64 = 160.0;
/// Growth of the slots on the time axis when the canvas leaves room.
const MAX_STRETCH: f64 = 1.5;
/// How far each nested fragment frame moves in from the one around it.
const FRAME_INSET: f64 = 5.0;
const TAG_HEIGHT: f64 = 16.0;

/// The whole diagram, laid out in whichever orientation fits.
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let sequence = sequence(spec);
    let width = f64::from(spec.width);
    let compact = spec.width < NARROW;
    let margin = if compact { COMPACT_MARGIN } else { MARGIN };
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let model = Model::new(sequence, compact, metrics);
    let mut ignored = Vec::new();
    let landscape = match sequence.orientation {
        DiagramOrientation::Portrait => false,
        DiagramOrientation::Landscape => true,
        DiagramOrientation::Auto => {
            let portrait = Portrait::new(spec, &model, top, metrics, &mut ignored);
            let landscape = Landscape::new(spec, &model, top, metrics, &mut ignored);
            diagram::prefers_landscape(
                spec,
                (portrait.width, portrait.height),
                (landscape.width, landscape.height),
            )
        }
    };
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        width - 2.0 * margin,
        metrics,
        warnings,
    );
    let (scene_width, scene_height) = if landscape {
        Landscape::new(spec, &model, top, metrics, warnings).draw(
            spec,
            &model,
            metrics,
            warnings,
            &mut elements,
        )
    } else {
        Portrait::new(spec, &model, top, metrics, warnings).draw(
            spec,
            &model,
            metrics,
            warnings,
            &mut elements,
        )
    };
    Scene {
        width: scene_width,
        height: scene_height,
        elements,
    }
}

fn sequence(spec: &ChartSpec) -> &SequenceSpec {
    spec.sequence
        .as_ref()
        .expect("validated sequence diagrams carry a sequence block")
}

/// A stretch of one participant's lifeline during which it is busy: from the call that reaches
/// it to the reply it sends, or else to the last message it takes part in.
struct Activation {
    participant: usize,
    from: usize,
    to: usize,
    /// How many of the participant's activations are still open around this one.
    level: usize,
}

/// What both orientations need from the specification.
struct Model<'a> {
    sequence: &'a SequenceSpec,
    ends: Vec<(usize, usize)>,
    /// Each message's label as drawn, with its number when the diagram is numbered.
    labels: Vec<String>,
    activations: Vec<Activation>,
    /// How many fragments enclose each fragment.
    depths: Vec<usize>,
    /// The width a participant's box needs for its label and sublabel.
    box_widths: Vec<f64>,
    /// Height of the band that holds the participants' names.
    band: f64,
    /// Whether the diagram is narrower than [`NARROW`]: participant names wrap, margins shrink.
    compact: bool,
    margin: f64,
    has_actor: bool,
}

impl<'a> Model<'a> {
    fn new(sequence: &'a SequenceSpec, compact: bool, metrics: &impl TextMetrics) -> Self {
        let ends = sequence.ends();
        let labels = sequence
            .messages
            .iter()
            .map(|message| message.label.clone())
            .collect();
        let mut two_lines = false;
        let box_widths = sequence
            .participants
            .iter()
            .map(|participant| {
                let mut label = metrics.width(&participant.label, LABEL_SIZE);
                if compact && label > COMPACT_TEXT {
                    // Never narrower than the longest word, which cannot wrap.
                    let word = participant
                        .label
                        .split(' ')
                        .map(|word| metrics.width(word, LABEL_SIZE))
                        .fold(0.0, f64::max);
                    let mut ignored = Vec::new();
                    let lines = wrap(
                        &participant.label,
                        COMPACT_TEXT.max(word),
                        LABEL_SIZE,
                        metrics,
                        &mut ignored,
                        "",
                    );
                    two_lines |= lines.len() > 1;
                    label = lines
                        .iter()
                        .map(|line| metrics.width(line, LABEL_SIZE))
                        .fold(0.0, f64::max);
                }
                let sublabel = participant
                    .sublabel
                    .as_ref()
                    .map_or(0.0, |sublabel| metrics.width(sublabel, SUBLABEL_SIZE));
                // A compact box is as wide as its widest line, even a long word.
                let low = if compact { COMPACT_BOX } else { MIN_BOX };
                (label.max(sublabel) + 2.0 * PAD).clamp(low, MAX_BOX)
            })
            .collect();
        let has_sublabel = sequence
            .participants
            .iter()
            .any(|participant| participant.sublabel.is_some());
        Self {
            activations: activations(sequence, &ends),
            depths: (0..sequence.fragments.len())
                .map(|index| sequence.depth(index))
                .collect(),
            ends,
            labels,
            sequence,
            box_widths,
            band: if has_sublabel { 48.0 } else { 34.0 } + if two_lines { LINE } else { 0.0 },
            compact,
            margin: if compact { COMPACT_MARGIN } else { MARGIN },
            has_actor: sequence
                .participants
                .iter()
                .any(|participant| participant.kind == ParticipantKind::Actor),
        }
    }

    fn participants(&self) -> usize {
        self.sequence.participants.len()
    }

    /// The widest box any participant needs.
    fn widest_box(&self) -> f64 {
        self.box_widths.iter().copied().fold(MIN_BOX, f64::max)
    }

    /// Whether `participant` is busy while message `index` is sent.
    fn active(&self, participant: usize, index: usize) -> bool {
        self.activations.iter().any(|activation| {
            activation.participant == participant
                && activation.from <= index
                && index <= activation.to
        })
    }

    /// The participants that send or receive a message inside `fragment`.
    fn span(&self, fragment: &FragmentSpec) -> (usize, usize) {
        self.ends[fragment.from..=fragment.to]
            .iter()
            .flat_map(|(from, to)| [*from, *to])
            .fold((usize::MAX, 0), |(low, high), participant| {
                (low.min(participant), high.max(participant))
            })
    }
}

/// The activations of a sequence: a call opens one on its receiver, a reply closes the last one
/// its sender has open, and one left open ends with the last message its participant takes part
/// in. A message to oneself and an asynchronous message open none.
fn activations(sequence: &SequenceSpec, ends: &[(usize, usize)]) -> Vec<Activation> {
    let mut activations: Vec<Activation> = Vec::new();
    let mut open: Vec<Vec<usize>> = vec![Vec::new(); sequence.participants.len()];
    let mut closed = Vec::new();
    for (index, (message, (from, to))) in sequence.messages.iter().zip(ends).enumerate() {
        match message.kind {
            MessageKind::Call if from != to => {
                open[*to].push(activations.len());
                activations.push(Activation {
                    participant: *to,
                    from: index,
                    to: index,
                    level: open[*to].len() - 1,
                });
            }
            MessageKind::Reply => {
                if let Some(activation) = open[*from].pop() {
                    activations[activation].to = index;
                    closed.push(activation);
                }
            }
            MessageKind::Call | MessageKind::Async => {}
        }
    }
    for (index, activation) in activations.iter_mut().enumerate() {
        if closed.contains(&index) {
            continue;
        }
        activation.to = ends
            .iter()
            .rposition(|(from, to)| {
                *from == activation.participant || *to == activation.participant
            })
            .expect("the call that opened it involves the participant")
            .max(activation.from);
    }
    activations
}

/// Lengthens the heads of fragments and branches in landscape, where a fragment's label and the
/// labels of its branches sit side by side along the top of the frame: each gets as far as the
/// next branch, or the end of the frame.
fn make_room_for_guards(model: &Model, slots: &mut Slots, metrics: &impl TextMetrics) {
    let track = Track::new(model, slots, 1.0);
    let guard = |label: Option<&str>| {
        label.map_or(0.0, |label| {
            metrics.width(&format!("[{label}]"), TAG_SIZE) + 12.0
        })
    };
    for (index, fragment) in model.sequence.fragments.iter().enumerate() {
        let end = track.frames[index].1;
        let starts = &track.branches[index];
        let needed = tab_width(fragment, metrics) + 6.0 + guard(fragment.label.as_deref());
        let room = starts.first().copied().unwrap_or(end) - track.frames[index].0;
        slots.head[index] += (needed - room).max(0.0);
        for (number, branch) in fragment.branches.iter().enumerate() {
            let next = starts.get(number + 1).copied().unwrap_or(end);
            let needed = 6.0 + guard(Some(branch.label.as_deref().unwrap_or("")));
            slots.branch[index][number] += (needed - (next - starts[number])).max(0.0);
        }
    }
}

/// The width of the tab that names a fragment's kind.
fn tab_width(fragment: &FragmentSpec, metrics: &impl TextMetrics) -> f64 {
    metrics.width(fragment.kind.keyword(), TAG_SIZE) + 12.0
}

/// Positions along the time axis, relative to where the messages begin.
struct Track {
    /// Where each message's arrow runs.
    arrows: Vec<f64>,
    /// Where each fragment's frame begins and ends.
    frames: Vec<(f64, f64)>,
    /// Where each further branch of each fragment begins.
    branches: Vec<Vec<f64>>,
    length: f64,
}

/// The extent of the slots on the time axis: before each arrow, the whole slot, each fragment's
/// head, the head of each of its branches, and a fragment's foot.
struct Slots {
    arrow: Vec<f64>,
    slot: Vec<f64>,
    head: Vec<f64>,
    branch: Vec<Vec<f64>>,
    foot: f64,
}

impl Slots {
    /// The heads of every fragment and branch at the same extent.
    fn uniform(sequence: &SequenceSpec, head: f64, branch: f64) -> (Vec<f64>, Vec<Vec<f64>>) {
        (
            vec![head; sequence.fragments.len()],
            sequence
                .fragments
                .iter()
                .map(|fragment| vec![branch; fragment.branches.len()])
                .collect(),
        )
    }
}

impl Track {
    fn new(model: &Model, slots: &Slots, scale: f64) -> Self {
        let sequence = model.sequence;
        let fragments = &sequence.fragments;
        let mut frames = vec![(0.0, 0.0); fragments.len()];
        let mut branches: Vec<Vec<f64>> = fragments
            .iter()
            .map(|fragment| vec![0.0; fragment.branches.len()])
            .collect();
        let mut arrows = Vec::with_capacity(sequence.messages.len());
        let mut at = 0.0;
        let mut by_depth: Vec<usize> = (0..fragments.len()).collect();
        by_depth.sort_by_key(|index| (model.depths[*index], *index));
        for index in 0..sequence.messages.len() {
            for &fragment in &by_depth {
                if fragments[fragment].from == index {
                    frames[fragment].0 = at;
                    at += slots.head[fragment] * scale;
                }
                if let Some(branch) = fragments[fragment]
                    .branches
                    .iter()
                    .position(|branch| branch.from == index)
                {
                    branches[fragment][branch] = at;
                    at += slots.branch[fragment][branch] * scale;
                }
            }
            arrows.push(at + slots.arrow[index] * scale);
            at += slots.slot[index] * scale;
            for &fragment in by_depth.iter().rev() {
                if fragments[fragment].to == index {
                    frames[fragment].1 = at;
                    at += slots.foot * scale;
                }
            }
        }
        Self {
            arrows,
            frames,
            branches,
            length: at,
        }
    }

    /// The track at the scale that fills `room` without growing slots by more than
    /// [`MAX_STRETCH`]; at scale 1 when it does not fit.
    fn fitted(model: &Model, slots: &Slots, room: f64) -> Self {
        let natural = Self::new(model, slots, 1.0);
        if natural.length >= room || natural.length <= 0.0 {
            return natural;
        }
        Self::new(model, slots, (room / natural.length).min(MAX_STRETCH))
    }
}

fn label_path(index: usize) -> String {
    format!("/sequence/messages/{index}/label")
}

/// Participants side by side, time running down.
struct Portrait {
    margin: f64,
    column: f64,
    /// Top of the band with the participants' names.
    band_top: f64,
    /// Where the messages begin.
    base: f64,
    lines: Vec<Vec<String>>,
    track: Track,
    width: f64,
    height: f64,
}

impl Portrait {
    fn new(
        spec: &ChartSpec,
        model: &Model,
        top: f64,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
    ) -> Self {
        let participants = crate::layout::count(model.participants());
        let needed = model.widest_box() + if model.compact { 8.0 } else { 12.0 };
        let width = f64::from(spec.width).max(2.0 * model.margin + needed * participants);
        let column = (width - 2.0 * model.margin) / participants;
        let lines: Vec<Vec<String>> = model
            .labels
            .iter()
            .zip(&model.ends)
            .enumerate()
            .map(|(index, (label, (from, to)))| {
                let room = if from == to {
                    column - LOOP_OUT - 8.0
                } else {
                    column * crate::layout::count(from.abs_diff(*to)) - 16.0
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

    fn draw(
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
            let x = self.center(participant);
            elements.push(Element::Line(Line {
                x1: x,
                y1: self.band_top + model.band,
                x2: x,
                y2: end,
                class: "chartlet-seq-lifeline",
            }));
        }
        for activation in &model.activations {
            let from = self.base + self.track.arrows[activation.from];
            let to = self.base + self.track.arrows[activation.to];
            elements.push(bar(
                self.center(activation.participant) - BAR
                    + NESTED_BAR * crate::layout::count(activation.level),
                from,
                2.0 * BAR,
                (to - from).max(12.0),
            ));
        }
        for (index, fragment) in sequence.fragments.iter().enumerate() {
            let (low, high) = model.span(fragment);
            let half = (self.column / 2.0 - 4.0).min(72.0);
            let inset = FRAME_INSET * crate::layout::count(model.depths[index]);
            let (top, bottom) = self.track.frames[index];
            let frame = Frame {
                left: self.center(low) - half + inset,
                top: self.base + top,
                right: self.center(high) + half - inset,
                bottom: self.base + bottom + 4.0,
                label_end: self.center(high) + half - inset,
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
            self.draw_message(model, index, metrics, elements);
        }
        let gap = if model.compact { 8.0 } else { 12.0 };
        let box_width = |participant: usize| model.box_widths[participant].min(self.column - gap);
        for (index, participant) in sequence.participants.iter().enumerate() {
            let width = box_width(index);
            draw_participant(
                &Header {
                    index,
                    x: self.center(index) - width / 2.0,
                    y: self.band_top,
                    width,
                    height: model.band,
                    figure: (self.center(index), self.band_top - FIGURE),
                    wrap: model.compact,
                },
                participant,
                metrics,
                warnings,
                elements,
            );
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
            let first = y - 9.0 - lift - LINE * crate::layout::count(lines.len() - 1);
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
        let first = y - 9.0 - lift - LINE * crate::layout::count(lines.len() - 1);
        chip(
            lines,
            (f64::midpoint(x1, x2), first),
            TextAnchor::Middle,
            metrics,
            elements,
        );
        if model.sequence.numbered {
            badge(index + 1, (x1 + direction * (BADGE + 8.0), y), elements);
        }
    }
}

/// Participants one below the other, time running right.
struct Landscape {
    /// Width of the participants' boxes in the gutter on the left.
    gutter: f64,
    lane: f64,
    top: f64,
    /// Where the messages begin.
    base: f64,
    lines: Vec<Vec<String>>,
    track: Track,
    width: f64,
    height: f64,
}

impl Landscape {
    fn new(
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

    fn draw(
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
            let y = self.center(participant);
            elements.push(Element::Line(Line {
                x1: model.margin + self.gutter,
                y1: y,
                x2: end,
                y2: y,
                class: "chartlet-seq-lifeline",
            }));
        }
        for activation in &model.activations {
            let from = self.base + self.track.arrows[activation.from];
            let to = self.base + self.track.arrows[activation.to];
            elements.push(bar(
                from,
                self.center(activation.participant) - BAR
                    + NESTED_BAR * crate::layout::count(activation.level),
                (to - from).max(12.0),
                2.0 * BAR,
            ));
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
            self.draw_message(model, index, metrics, elements);
        }
        for (index, participant) in sequence.participants.iter().enumerate() {
            let center = self.center(index);
            let indent = if participant.kind == ParticipantKind::Actor {
                FIGURE
            } else {
                0.0
            };
            draw_participant(
                &Header {
                    index,
                    x: model.margin + indent,
                    y: center - model.band / 2.0,
                    width: self.gutter - indent,
                    height: model.band,
                    figure: (model.margin + FIGURE / 2.0, center - FIGURE / 2.0),
                    wrap: model.compact,
                },
                participant,
                metrics,
                warnings,
                elements,
            );
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
                (x + LOOP_OUT + 4.0 + CHIP_REACH, y + 14.0),
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
            (x + 2.0 + CHIP_REACH, gap + 4.0 - block / 2.0),
            TextAnchor::Start,
            metrics,
            elements,
        );
        if model.sequence.numbered {
            badge(index + 1, (x, y1 + direction * (BADGE + 8.0)), elements);
        }
    }
}

fn message_tooltip(model: &Model, index: usize) -> String {
    let message = &model.sequence.messages[index];
    let (from, to) = model.ends[index];
    let participants = &model.sequence.participants;
    format!(
        "{}. {} → {}: {}",
        index + 1,
        participants[from].label,
        participants[to].label,
        message.label
    )
}

fn polyline(points: Vec<(f64, f64)>, kind: MessageKind, tooltip: Option<String>) -> Polyline {
    let class = if kind == MessageKind::Reply {
        "chartlet-seq-message chartlet-seq-reply"
    } else {
        "chartlet-seq-message"
    };
    diagram::polyline(points, class, tooltip)
}

/// The arrowhead of a message: filled for a call, open otherwise.
fn head(tip: (f64, f64), direction: (f64, f64), kind: MessageKind) -> Polyline {
    if kind == MessageKind::Call {
        arrowhead(tip, direction, true, "chartlet-seq-head")
    } else {
        arrowhead(tip, direction, false, "chartlet-seq-head-open")
    }
}

fn bar(x: f64, y: f64, width: f64, height: f64) -> Element {
    Element::Rect(Rect {
        x,
        y,
        width,
        height,
        class: "chartlet-seq-activation",
        series_index: None,
        style_index: None,
        tooltip: None,
    })
}

/// A fragment's frame.
struct Frame {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    /// How far right the fragment's label may run.
    label_end: f64,
}

impl Frame {
    /// The frame with its kind in a tab in the top left corner and its label beside the tab.
    fn draw(
        &self,
        index: usize,
        fragment: &FragmentSpec,
        metrics: &impl TextMetrics,
        warnings: &mut Vec<ChartWarning>,
        elements: &mut Vec<Element>,
    ) {
        elements.push(Element::Rect(Rect {
            x: self.left,
            y: self.top,
            width: self.right - self.left,
            height: self.bottom - self.top,
            class: "chartlet-seq-frame",
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        let keyword = fragment.kind.keyword();
        let tab = tab_width(fragment, metrics);
        elements.push(Element::Rect(Rect {
            x: self.left,
            y: self.top,
            width: tab,
            height: TAG_HEIGHT,
            class: "chartlet-seq-tab",
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
        elements.push(Element::Text(Text {
            x: self.left + 6.0,
            y: self.top + 12.0,
            class: "chartlet-seq-tag",
            anchor: TextAnchor::Start,
            content: keyword.to_owned(),
        }));
        if let Some(label) = &fragment.label {
            let room = self.label_end - self.left - tab - 12.0;
            let path = format!("/sequence/fragments/{index}/label");
            let content = fit_text(
                &format!("[{label}]"),
                room,
                TAG_SIZE,
                metrics,
                warnings,
                &path,
            );
            elements.push(Element::Text(Text {
                x: self.left + tab + 6.0,
                y: self.top + 12.0,
                class: "chartlet-seq-guard",
                anchor: TextAnchor::Start,
                content,
            }));
        }
    }
}

/// The condition of a branch, in brackets.
fn guard(
    (x, y): (f64, f64),
    label: &str,
    room: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) -> Element {
    Element::Text(Text {
        x,
        y,
        class: "chartlet-seq-guard",
        anchor: TextAnchor::Start,
        content: fit_text(
            &format!("[{label}]"),
            room,
            TAG_SIZE,
            metrics,
            warnings,
            path,
        ),
    })
}

/// Where a participant's box goes: the band that holds its name.
struct Header {
    index: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    /// Where an actor's figure stands: the middle of its head and the top of the figure.
    figure: (f64, f64),
    /// Whether a name too wide for the box wraps onto a second line rather than being shortened.
    wrap: bool,
}

fn draw_participant(
    header: &Header,
    participant: &crate::spec::ParticipantSpec,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    elements: &mut Vec<Element>,
) {
    let Header {
        x,
        y,
        width,
        height,
        figure: (figure_x, figure_y),
        ..
    } = *header;
    let tooltip = Some(match &participant.sublabel {
        Some(sublabel) => format!("{} – {sublabel}", participant.label),
        None => participant.label.clone(),
    });
    let rect = |x: f64, y: f64, class: &'static str, tooltip: Option<String>| {
        Element::Rect(Rect {
            x,
            y,
            width,
            height,
            class,
            series_index: None,
            style_index: None,
            tooltip,
        })
    };
    match participant.kind {
        // Every kind has its own shape; its role color only repeats what the shape says.
        ParticipantKind::Service => with_shadow(
            rect(x, y, "chartlet-seq-box chartlet-role-blue", tooltip),
            elements,
        ),
        ParticipantKind::External => with_shadow(
            rect(
                x,
                y,
                "chartlet-seq-box chartlet-seq-external chartlet-role-gray",
                tooltip,
            ),
            elements,
        ),
        ParticipantKind::Queue => {
            with_shadow(
                rect(
                    x + 4.0,
                    y - 4.0,
                    "chartlet-seq-box chartlet-role-violet",
                    None,
                ),
                elements,
            );
            with_shadow(
                rect(x, y, "chartlet-seq-box chartlet-role-violet", tooltip),
                elements,
            );
        }
        ParticipantKind::Database => cylinder(
            (x, y, width, height),
            (
                "chartlet-seq-box chartlet-role-teal",
                "chartlet-diagram-rim",
            ),
            tooltip,
            elements,
        ),
        ParticipantKind::Actor => {
            figure(figure_x, figure_y, elements);
            elements.push(rect(x, y, "chartlet-seq-hit", tooltip));
        }
    }
    participant_text(header, participant, metrics, warnings, elements);
}

/// A participant's label, and its sublabel below it.
fn participant_text(
    header: &Header,
    participant: &crate::spec::ParticipantSpec,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    elements: &mut Vec<Element>,
) {
    let Header {
        index,
        x,
        y,
        width,
        height,
        ..
    } = *header;
    let center = x + width / 2.0;
    let path = format!("/sequence/participants/{index}");
    let room = width - 2.0 * PAD;
    let label_path = format!("{path}/label");
    let lines = if header.wrap {
        wrap(
            &participant.label,
            room,
            LABEL_SIZE,
            metrics,
            warnings,
            &label_path,
        )
    } else {
        vec![fit_text(
            &participant.label,
            room,
            LABEL_SIZE,
            metrics,
            warnings,
            &label_path,
        )]
    };
    // A second line of the name moves both lines up by half its height.
    let lift = LINE * crate::layout::count(lines.len() - 1) / 2.0;
    let (label_y, sublabel_y) = match participant.sublabel {
        Some(_) => (
            y + height / 2.0 - 2.0 - lift,
            y + height / 2.0 + 13.0 + lift,
        ),
        None => (y + height / 2.0 + 4.5 - lift, 0.0),
    };
    for (row, line) in lines.into_iter().enumerate() {
        elements.push(Element::Text(Text {
            x: center,
            y: label_y + LINE * crate::layout::count(row),
            class: "chartlet-seq-label",
            anchor: TextAnchor::Middle,
            content: line,
        }));
    }
    if let Some(sublabel) = &participant.sublabel {
        elements.push(Element::Text(Text {
            x: center,
            y: sublabel_y,
            class: "chartlet-seq-sublabel",
            anchor: TextAnchor::Middle,
            content: fit_text(
                sublabel,
                room,
                SUBLABEL_SIZE,
                metrics,
                warnings,
                &format!("{path}/sublabel"),
            ),
        }));
    }
}

/// A person: head, body, arms and legs, standing on `top + FIGURE`.
fn figure(center: f64, top: f64, elements: &mut Vec<Element>) {
    elements.push(Element::Circle(Circle {
        cx: center,
        cy: top + 5.5,
        radius: 4.5,
        class: "chartlet-seq-actor-head",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    for (x1, y1, x2, y2) in [
        (0.0, 10.0, 0.0, 17.0),
        (-7.0, 12.5, 7.0, 12.5),
        (0.0, 17.0, -6.0, 23.0),
        (0.0, 17.0, 6.0, 23.0),
    ] {
        elements.push(Element::Line(Line {
            x1: center + x1,
            y1: top + y1,
            x2: center + x2,
            y2: top + y2,
            class: "chartlet-seq-actor",
        }));
    }
}

/// The diagram in sentences: its participants, then every message in order, then the fragments.
pub(crate) fn description(spec: &ChartSpec) -> String {
    let sequence = sequence(spec);
    let locale = spec.locale;
    let participants: Vec<String> = sequence
        .participants
        .iter()
        .map(|participant| text::participant(locale, &participant.label, participant.kind))
        .collect();
    let mut description = text::sequence_opening(locale, &participants, sequence.messages.len());
    for (index, (message, (from, to))) in sequence.messages.iter().zip(sequence.ends()).enumerate()
    {
        let sender = &sequence.participants[from].label;
        let receiver = (from != to).then(|| sequence.participants[to].label.as_str());
        write!(
            description,
            " {}",
            text::sequence_message(
                locale,
                index + 1,
                sender,
                receiver,
                &message.label,
                message.kind
            )
        )
        .expect("writing to String cannot fail");
    }
    for fragment in &sequence.fragments {
        description.push(' ');
        description.push_str(&text::fragment(locale, fragment));
    }
    description
}

/// One row per message: its number, sender, receiver, label and kind, and the fragments around
/// it when the diagram has any.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    let sequence = sequence(spec);
    let words = spec.locale.words();
    let mut columns: Vec<String> = [
        words.number,
        words.sender,
        words.receiver,
        words.message,
        words.message_kind,
    ]
    .iter()
    .map(|word| (*word).to_owned())
    .collect();
    let framed = !sequence.fragments.is_empty();
    if framed {
        columns.push(words.fragment.to_owned());
    }
    let rows = sequence
        .messages
        .iter()
        .zip(sequence.ends())
        .enumerate()
        .map(|(index, (message, (from, to)))| {
            let mut row = vec![
                (index + 1).to_string(),
                sequence.participants[from].label.clone(),
                sequence.participants[to].label.clone(),
                message.label.clone(),
                text::message_kind(spec.locale, message.kind).to_owned(),
            ];
            if framed {
                row.push(fragments_at(spec.locale, sequence, index));
            }
            row
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
    }
}

/// The fragments around message `index`, outermost first, each as its kind and the label of the
/// branch the message is in.
fn fragments_at(locale: Locale, sequence: &SequenceSpec, index: usize) -> String {
    let mut around: Vec<(usize, &FragmentSpec)> = sequence
        .fragments
        .iter()
        .enumerate()
        .filter(|(_, fragment)| fragment.from <= index && index <= fragment.to)
        .map(|(position, fragment)| (sequence.depth(position), fragment))
        .collect();
    around.sort_by_key(|(depth, _)| *depth);
    around
        .iter()
        .map(|(_, fragment)| {
            let label = fragment
                .branches
                .iter()
                .rev()
                .find(|branch| branch.from <= index)
                .map_or(fragment.label.as_deref(), |branch| {
                    Some(branch.label.as_deref().unwrap_or(text::otherwise(locale)))
                });
            match label {
                Some(label) => format!("{} [{label}]", fragment.kind.keyword()),
                None => fragment.kind.keyword().to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use crate::{ChartSpec, RenderFormat, RenderOptions, render_json, text_alternative};

    const SPEC: &str = r#"{
        "schemaVersion": 1,
        "type": "sequence",
        "title": "Login",
        "sequence": {
            "participants": [
                {"id": "user", "label": "User", "kind": "actor"},
                {"id": "app", "label": "App"},
                {"id": "db", "label": "Users", "kind": "database"}
            ],
            "messages": [
                {"from": "user", "to": "app", "label": "sign in"},
                {"from": "app", "to": "db", "label": "find user"},
                {"from": "db", "to": "app", "label": "user", "kind": "reply"},
                {"from": "app", "to": "app", "label": "check password"},
                {"from": "app", "to": "user", "label": "welcome", "kind": "reply"}
            ],
            "fragments": [{"kind": "opt", "label": "known user", "from": 2, "to": 3}]
        }
    }"#;

    fn spec(json: &str) -> ChartSpec {
        ChartSpec::from_json(json).expect("the test specification parses")
    }

    fn svg(json: &str) -> crate::RenderOutput {
        render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect("the test specification renders")
    }

    fn error(json: &str) -> (&'static str, String) {
        let error = render_json(json, RenderFormat::Svg, &RenderOptions::default())
            .expect_err("the test specification is invalid");
        (error.code, error.path)
    }

    #[test]
    fn every_message_is_an_arrow_with_its_label_and_number() {
        let output = svg(SPEC);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-seq-message\"")
                .count(),
            3
        );
        assert_eq!(
            output
                .content
                .matches("chartlet-seq-message chartlet-seq-reply")
                .count(),
            2
        );
        assert_eq!(
            output
                .content
                .matches("class=\"chartlet-seq-head\"")
                .count(),
            3
        );
        assert!(
            output
                .content
                .contains("<title>2. App → Users: find user</title>")
        );
        assert!(output.content.contains(">opt</text>"));
        assert!(output.content.contains(">[known user]</text>"));
    }

    #[test]
    fn a_call_activates_its_receiver_until_the_reply() {
        let sequence = spec(SPEC).sequence.expect("a sequence block");
        let activations = super::activations(&sequence, &sequence.ends());
        let spans: Vec<(usize, usize, usize)> = activations
            .iter()
            .map(|activation| (activation.participant, activation.from, activation.to))
            .collect();
        // The app is busy from the sign-in to its reply, the store from the lookup to its reply.
        assert_eq!(spans, vec![(1, 0, 4), (2, 1, 2)]);
    }

    #[test]
    fn an_activation_without_reply_ends_with_the_last_message_of_its_participant() {
        let json = SPEC.replace(
            r#""label": "user", "kind": "reply""#,
            r#""label": "user", "kind": "async""#,
        );
        let sequence = spec(&json).sequence.expect("a sequence block");
        let activations = super::activations(&sequence, &sequence.ends());
        assert_eq!((activations[1].from, activations[1].to), (1, 2));
    }

    #[test]
    fn auto_orientation_turns_landscape_when_only_that_fits() {
        let messages: Vec<String> = (0..30)
            .map(|index| format!(r#"{{"from": "a", "to": "b", "label": "m{index}"}}"#))
            .collect();
        let json = format!(
            r#"{{"schemaVersion": 1, "type": "sequence", "title": "Many", "width": 2400, "height": 300,
            "sequence": {{"participants": [{{"id": "a", "label": "A"}}, {{"id": "b", "label": "B"}}],
            "messages": [{}]}}}}"#,
            messages.join(",")
        );
        let output = svg(&json);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert!(output.content.starts_with(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"2400\" height=\"300\""
        ));
        // In landscape the lifelines run across: the first one starts and ends at the same height.
        let lifeline = output
            .content
            .split("<line ")
            .skip(1)
            .find(|line| line.contains("chartlet-seq-lifeline"))
            .expect("a lifeline");
        let y = |name: &str| {
            lifeline
                .split(&format!("{name}=\""))
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .expect("a coordinate")
                .to_owned()
        };
        assert_eq!(y("y1"), y("y2"));

        let portrait = json.replace(r#""messages""#, r#""orientation": "portrait", "messages""#);
        let output = svg(&portrait);
        assert_eq!(output.warnings[0].code, "canvas_too_small");
        assert_eq!(output.warnings[0].path, "/height");
    }

    #[test]
    fn the_text_alternative_lists_every_message_in_order() {
        let alternative = text_alternative(&spec(SPEC)).expect("valid");
        assert_eq!(
            alternative.description,
            "Sequence diagram with 3 participants: User (actor), App and Users (database). 5 messages, in order: 1. User to App: sign in. 2. App to Users: find user. 3. Users to App: user (reply). 4. App to itself: check password. 5. App to User: welcome (reply). Fragment opt \"known user\" spans messages 3–4."
        );
        assert_eq!(
            alternative.table.columns,
            ["No.", "From", "To", "Message", "Kind", "Fragment"]
        );
        assert_eq!(
            alternative.table.rows[2],
            ["3", "Users", "App", "user", "reply", "opt [known user]"]
        );
        assert_eq!(alternative.table.rows[0][5], "");
    }

    #[test]
    fn the_html_table_carries_text_hooks_without_values() {
        let output = render_json(
            SPEC,
            RenderFormat::Html,
            &RenderOptions {
                hooks: true,
                ..RenderOptions::default()
            },
        )
        .expect("renders");
        assert!(output.content.contains(
            "<th scope=\"col\" data-series=\"0\" data-pane=\"0\" data-part=\"text\">From</th>"
        ));
        assert!(!output.content.contains("data-value"));
    }

    #[test]
    fn invalid_sequences_name_the_field() {
        for (from, to, code, path) in [
            (
                r#""to": "db""#,
                r#""to": "nobody""#,
                "unknown_participant",
                "/sequence/messages/1/to",
            ),
            (
                r#""id": "db""#,
                r#""id": "app""#,
                "duplicate_id",
                "/sequence/participants/2/id",
            ),
            (
                r#""id": "db""#,
                r#""id": "2db""#,
                "invalid_id",
                "/sequence/participants/2/id",
            ),
            (
                r#""from": 2, "to": 3"#,
                r#""from": 3, "to": 2"#,
                "invalid_fragment",
                "/sequence/fragments/0/from",
            ),
            (
                r#""from": 2, "to": 3"#,
                r#""from": 2, "to": 5"#,
                "invalid_fragment",
                "/sequence/fragments/0/to",
            ),
            (
                r#""from": 2, "to": 3}"#,
                r#""from": 2, "to": 3, "else": [{"from": 3}]}"#,
                "option_not_supported",
                "/sequence/fragments/0/else",
            ),
            (
                r#""from": 2, "to": 3}]"#,
                r#""from": 2, "to": 3}, {"kind": "loop", "from": 3, "to": 4}]"#,
                "fragments_cross",
                "/sequence/fragments/1/from",
            ),
            (
                r#""type": "sequence""#,
                r#""type": "bar""#,
                "option_not_supported",
                "/sequence",
            ),
        ] {
            assert!(SPEC.contains(from), "{from}");
            assert_eq!(
                error(&SPEC.replace(from, to)),
                (code, path.to_owned()),
                "{to}"
            );
        }
        let missing = r#"{"schemaVersion": 1, "type": "sequence", "title": "Empty"}"#;
        assert_eq!(error(missing), ("missing_sequence", "/sequence".to_owned()));
    }

    #[test]
    fn nested_fragments_and_branches_are_accepted() {
        let json = SPEC.replace(
            r#"[{"kind": "opt", "label": "known user", "from": 2, "to": 3}]"#,
            r#"[{"kind": "alt", "label": "found", "from": 1, "to": 4, "else": [{"from": 3, "label": "unknown"}]},
                {"kind": "loop", "from": 1, "to": 2}]"#,
        );
        let sequence = spec(&json).sequence.expect("a sequence block");
        assert_eq!((sequence.depth(0), sequence.depth(1)), (0, 1));
        let output = svg(&json);
        assert!(output.warnings.is_empty(), "{:?}", output.warnings);
        assert!(output.content.contains(">[unknown]</text>"));
    }

    #[test]
    fn a_narrow_diagram_wraps_participant_names_instead_of_shortening_them() {
        let json = SPEC
            .replace(
                "\"title\": \"Login\",",
                "\"title\": \"Login\", \"width\": 320, \"height\": 600,",
            )
            .replace(
                r#""label": "Users", "kind": "database""#,
                r#""label": "Registered users", "kind": "database""#,
            );
        let output = svg(&json);
        assert!(
            output
                .warnings
                .iter()
                .all(|warning| !warning.path.starts_with("/sequence/participants")),
            "{:?}",
            output.warnings
        );
        assert!(output.content.contains(">Registered</text>"));
        assert!(output.content.contains(">users</text>"));
    }

    #[test]
    fn rendering_twice_gives_the_same_bytes() {
        assert_eq!(svg(SPEC).content, svg(SPEC).content);
    }
}
