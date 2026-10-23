//! Drawing steps: their shape and their texts, from a step's label to the members of a class box
//! and the name of an interface.

use super::{
    BALL, INTERFACE_GAP, INTERFACE_LINE, INTERFACE_PAD, MEMBER_LINE, MEMBER_PAD, NODE_LINE, PAD,
    measure::{Model, NOTE_PAD, class_header},
    model::Shape,
    place::Plan,
    shapes::shape,
};
use crate::{
    layout::count,
    parts,
    scene::{Element, Line, Rect, SymbolUse, Text, TextAnchor},
    symbols,
};

impl Plan {
    pub(super) fn draw_node(&self, model: &Model, node: usize, elements: &mut Vec<Element>) {
        let spec = &model.diagram.nodes[node];
        let (x, y, width, height) = self.node_box(model, node);
        let tooltip = Some(match &spec.sublabel {
            Some(sublabel) => format!("{} – {sublabel}", spec.label),
            None => spec.label.clone(),
        });
        shape(spec.shape, (x, y, width, height), tooltip, elements);
        if spec.shape == Shape::Class {
            Self::class_text(model, node, (x, y, width), elements);
            return;
        }
        if spec.shape == Shape::Interface {
            self.interface_text(model, node, (x, y, width, height), elements);
            return;
        }
        if spec.shape == Shape::Note {
            for (index, line) in model.labels[node].iter().enumerate() {
                elements.push(Element::Text(Text {
                    x: x + PAD,
                    y: y + NOTE_PAD + 11.0 + MEMBER_LINE * count(index),
                    class: "chartlet-diagram-chip-text",
                    anchor: TextAnchor::Start,
                    content: line.clone(),
                }));
            }
            return;
        }
        Self::node_text(model, node, (x, y, width, height), elements);
        let lift = if spec.shape == Shape::Store { 6.0 } else { 0.0 };
        if let Some((class, name)) = &spec.category {
            parts::push_band(class, name, (x, y, width, height), lift, elements);
        }
        if !spec.parts.is_empty() {
            let names = parts::names(model.diagram.nodes.iter().map(|node| node.parts.as_slice()));
            parts::push_stripe(
                model.number_style,
                (&spec.parts, &names),
                (x, y, width, height),
                lift,
                elements,
            );
        }
    }

    /// The name of an interface: beside its ball in portrait, below it in landscape.
    fn interface_text(
        &self,
        model: &Model,
        node: usize,
        (x, y, width, height): (f64, f64, f64, f64),
        elements: &mut Vec<Element>,
    ) {
        let lines = &model.labels[node];
        let (middle_x, middle_y) = (x + width / 2.0, y + height / 2.0);
        let compact = model.spacing.is_compact();
        if compact {
            elements.push(Element::Rect(Rect {
                x,
                y: middle_y + BALL + INTERFACE_PAD / 2.0,
                width,
                height: INTERFACE_LINE * count(lines.len()) + INTERFACE_PAD,
                class: "chartlet-diagram-chip",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
        for (index, line) in lines.iter().enumerate() {
            let (x, y, anchor) = if compact {
                (
                    middle_x,
                    middle_y + BALL + INTERFACE_PAD + 9.0 + INTERFACE_LINE * count(index),
                    TextAnchor::Middle,
                )
            } else if self.landscape {
                (
                    middle_x,
                    middle_y + BALL + INTERFACE_GAP + 9.0 + INTERFACE_LINE * count(index),
                    TextAnchor::Middle,
                )
            } else {
                (
                    middle_x + BALL + INTERFACE_GAP,
                    middle_y + 4.0 + INTERFACE_LINE * (count(index) - count(lines.len() - 1) / 2.0),
                    TextAnchor::Start,
                )
            };
            elements.push(Element::Text(Text {
                x,
                y,
                class: "chartlet-arch-interface",
                anchor,
                content: line.clone(),
            }));
        }
    }

    /// A class box: the stereotype above the name, centered, then a line and the members of every
    /// compartment, each compartment under a line of its own.
    fn class_text(
        model: &Model,
        node: usize,
        (x, y, width): (f64, f64, f64),
        elements: &mut Vec<Element>,
    ) {
        let spec = &model.diagram.nodes[node];
        let (names, stereotypes) = (&model.labels[node], &model.sublabels[node]);
        let middle = x + width / 2.0;
        let mut baseline = y + 12.0;
        for line in stereotypes {
            baseline += 14.0;
            elements.push(Element::Text(Text {
                x: middle,
                y: baseline - 3.0,
                class: "chartlet-flow-sublabel",
                anchor: TextAnchor::Middle,
                content: line.clone(),
            }));
        }
        for line in names {
            baseline += NODE_LINE;
            elements.push(Element::Text(Text {
                x: middle,
                y: baseline - 2.0,
                class: "chartlet-flow-label",
                anchor: TextAnchor::Middle,
                content: line.clone(),
            }));
        }
        let mut top = y + class_header(names.len(), stereotypes.len());
        for lines in spec.compartments.iter().filter(|lines| !lines.is_empty()) {
            elements.push(Element::Line(Line {
                x1: x,
                y1: top,
                x2: x + width,
                y2: top,
                class: "chartlet-flow-inner",
            }));
            for (index, line) in lines.iter().enumerate() {
                elements.push(Element::Text(Text {
                    x: x + PAD,
                    y: top + MEMBER_PAD + MEMBER_LINE * count(index) + 11.0,
                    class: "chartlet-class-member",
                    anchor: TextAnchor::Start,
                    content: line.clone(),
                }));
            }
            top += 2.0 * MEMBER_PAD + MEMBER_LINE * count(lines.len());
        }
    }

    /// A step's label, centered on one or two lines, and its sublabel below.
    fn node_text(
        model: &Model,
        node: usize,
        (x, y, width, height): (f64, f64, f64, f64),
        elements: &mut Vec<Element>,
    ) {
        let spec = &model.diagram.nodes[node];
        // The initial dot carries no text; its name is its tooltip.
        if spec.shape == Shape::Initial {
            return;
        }
        let (middle_x, middle_y) = (x + width / 2.0, y + height / 2.0);
        let lines = &model.labels[node];
        let sublines = &model.sublabels[node];
        let block = NODE_LINE * count(lines.len() - 1) + 14.0 * count(sublines.len());
        // Text clears what a shape draws at its top.
        let shift = match spec.shape {
            Shape::Store => 3.0,
            Shape::Person => 8.0,
            Shape::Frontend => 6.0,
            Shape::Shield => -6.0,
            _ => 0.0,
        } - if spec.has_stripe() {
            parts::ROOM / 2.0
        } else {
            0.0
        };
        // A symbol stands centered above the label, in the room the box grew by.
        let first = middle_y - block / 2.0
            + 4.5
            + shift
            + if spec.symbol.is_some() {
                symbols::ROOM / 2.0
            } else {
                0.0
            };
        if let Some(symbol) = spec.symbol {
            elements.push(Element::Symbol(SymbolUse {
                name: symbol.name,
                x: middle_x - symbols::SIZE / 2.0,
                y: first - 10.0 - symbols::ROOM,
            }));
        }
        for (index, line) in lines.iter().enumerate() {
            elements.push(Element::Text(Text {
                x: middle_x,
                y: first + NODE_LINE * count(index),
                class: "chartlet-flow-label",
                anchor: TextAnchor::Middle,
                content: line.clone(),
            }));
        }
        for (index, line) in sublines.iter().enumerate() {
            elements.push(Element::Text(Text {
                x: middle_x,
                y: first + NODE_LINE * count(lines.len() - 1) + 14.0 * count(index + 1),
                class: "chartlet-flow-sublabel",
                anchor: TextAnchor::Middle,
                content: line.clone(),
            }));
        }
    }
}
