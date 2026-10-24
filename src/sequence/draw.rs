//! The parts both orientations draw alike: arrows and their heads, activation bars, fragment
//! frames with their guards, and participants with their boxes, figures and names.

use super::{
    LABEL_SIZE, LINE, SUBLABEL_SIZE, TAG_HEIGHT, TAG_SIZE, model::Model, track::tab_width,
};
use crate::{
    diagram::{self, arrowhead, cylinder, with_shadow, wrap},
    error::ChartWarning,
    layout::fit_text,
    metrics::TextMetrics,
    scene::{Circle, Element, Hotspot, Line, Polyline, Rect, Text, TextAnchor},
    spec::{FragmentSpec, MessageKind, ParticipantKind},
};

pub(super) fn message_tooltip(model: &Model, index: usize) -> String {
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

pub(super) fn polyline(
    points: Vec<(f64, f64)>,
    kind: MessageKind,
    tooltip: Option<String>,
) -> Polyline {
    let class = if kind == MessageKind::Reply {
        "chartlet-seq-message chartlet-seq-reply"
    } else {
        "chartlet-seq-message"
    };
    diagram::polyline(points, class, tooltip)
}

/// The arrowhead of a message: filled for a call, open otherwise.
pub(super) fn head(tip: (f64, f64), direction: (f64, f64), kind: MessageKind) -> Polyline {
    if kind == MessageKind::Call {
        arrowhead(tip, direction, true, "chartlet-seq-head")
    } else {
        arrowhead(tip, direction, false, "chartlet-seq-head-open")
    }
}

pub(super) fn bar(x: f64, y: f64, width: f64, height: f64) -> Element {
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
pub(super) struct Frame {
    pub(super) left: f64,
    pub(super) top: f64,
    pub(super) right: f64,
    pub(super) bottom: f64,
    /// How far right the fragment's label may run.
    pub(super) label_end: f64,
}

impl Frame {
    /// The frame with its kind in a tab in the top left corner and its label beside the tab.
    pub(super) fn draw(
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
pub(super) fn guard(
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

/// A participant in the group of its focus, and its focus link's place.
pub(super) fn focusable_participant(
    model: &Model,
    header: &Header,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    elements: &mut Vec<Element>,
) {
    let participant = &model.sequence.participants[header.index];
    elements.push(Element::NodeGroup(
        model.participant_classes(header.index),
        participant.id.clone(),
    ));
    draw_participant(header, participant, metrics, warnings, elements);
    elements.push(Element::GroupEnd);
    elements.push(Element::Hotspot(Hotspot {
        node: header.index,
        label: participant.label.clone(),
        x: header.x,
        y: header.y,
        width: header.width,
        height: header.height,
    }));
}

/// Where a participant's box goes: the band that holds its name.
pub(super) struct Header {
    pub(super) index: usize,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
    /// Where an actor's figure stands: the middle of its head and the top of the figure.
    pub(super) figure: (f64, f64),
    /// Whether a name too wide for the box wraps onto a second line rather than being shortened.
    pub(super) wrap: bool,
    /// Text inset inside the box.
    pub(super) pad: f64,
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
        pad,
        ..
    } = *header;
    let center = x + width / 2.0;
    let path = format!("/sequence/participants/{index}");
    let room = width - 2.0 * pad;
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
