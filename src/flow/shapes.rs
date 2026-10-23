//! The shape of every kind of step.

use super::{BALL, model::Shape};
use crate::{
    diagram::{self, cylinder, with_shadow},
    scene::{Circle, Element, Line, Rect},
};

/// The shape of a step of `kind` in the box at `x`, `y`, with its shadow: every kind has its own,
/// and its role color only repeats what the shape says.
#[allow(clippy::too_many_lines)]
pub(crate) fn shape(
    kind: Shape,
    (x, y, width, height): (f64, f64, f64, f64),
    tooltip: Option<String>,
    elements: &mut Vec<Element>,
) {
    let shape = |points: Vec<(f64, f64)>, class: &'static str, tooltip: Option<String>| {
        Element::Polyline(diagram::polyline(points, class, tooltip))
    };
    let rect = |class: &'static str, tooltip: Option<String>| {
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
    let (middle_x, middle_y) = (x + width / 2.0, y + height / 2.0);
    // Every kind has its own shape; its role color only repeats what the shape says.
    match kind {
        Shape::Process => with_shadow(
            rect("chartlet-flow-node chartlet-role-blue", tooltip),
            elements,
        ),
        Shape::External => with_shadow(
            rect(
                "chartlet-flow-node chartlet-flow-external chartlet-role-gray",
                tooltip,
            ),
            elements,
        ),
        Shape::Subprocess => {
            with_shadow(
                rect("chartlet-flow-node chartlet-role-blue", tooltip),
                elements,
            );
            for side in [x + 7.0, x + width - 7.0] {
                elements.push(Element::Line(Line {
                    x1: side,
                    y1: y,
                    x2: side,
                    y2: y + height,
                    class: "chartlet-flow-inner",
                }));
            }
        }
        Shape::Start | Shape::End => {
            let class = if kind == Shape::Start {
                "chartlet-flow-node chartlet-flow-start chartlet-role-green"
            } else {
                "chartlet-flow-node chartlet-flow-end chartlet-role-green"
            };
            with_shadow(shape(pill(x, y, width, height), class, tooltip), elements);
        }
        Shape::Decision => with_shadow(
            shape(
                vec![
                    (middle_x, y),
                    (x + width, middle_y),
                    (middle_x, y + height),
                    (x, middle_y),
                    (middle_x, y),
                ],
                "chartlet-flow-node chartlet-role-amber",
                tooltip,
            ),
            elements,
        ),
        Shape::Io => with_shadow(
            shape(
                vec![
                    (x + 10.0, y),
                    (x + width, y),
                    (x + width - 10.0, y + height),
                    (x, y + height),
                    (x + 10.0, y),
                ],
                "chartlet-flow-node chartlet-role-violet",
                tooltip,
            ),
            elements,
        ),
        Shape::Store => cylinder(
            (x, y, width, height),
            (
                "chartlet-flow-node chartlet-role-teal",
                "chartlet-diagram-rim",
            ),
            tooltip,
            elements,
        ),
        Shape::Class => with_shadow(
            rect(
                "chartlet-flow-node chartlet-role-blue chartlet-class",
                tooltip,
            ),
            elements,
        ),
        Shape::State | Shape::Final | Shape::Initial => {
            state_shape(kind, (x, y, width, height), tooltip, elements);
        }
        // A block draws what it holds instead of a shape.
        Shape::Block => {}
        Shape::Interface => elements.push(Element::Circle(Circle {
            cx: x + width / 2.0,
            cy: y + height / 2.0,
            radius: BALL,
            class: "chartlet-arch-ball",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip,
        })),
        Shape::Note => {
            // A sheet without a shadow, its top right corner folded over.
            elements.push(shape(
                vec![
                    (x, y),
                    (x + width - 10.0, y),
                    (x + width, y + 10.0),
                    (x + width, y + height),
                    (x, y + height),
                    (x, y),
                ],
                "chartlet-flow-node chartlet-role-amber",
                tooltip,
            ));
            elements.push(shape(
                vec![
                    (x + width - 10.0, y),
                    (x + width - 10.0, y + 10.0),
                    (x + width, y + 10.0),
                ],
                "chartlet-flow-node chartlet-role-amber",
                None,
            ));
        }
        Shape::Person
        | Shape::Frontend
        | Shape::Queue
        | Shape::Bucket
        | Shape::Cache
        | Shape::Node
        | Shape::Artifact
        | Shape::Shield => {
            component_shape(kind, (x, y, width, height), tooltip, elements);
        }
    }
}

/// The shapes of a state diagram: a state, a final state with its double outline, and the
/// initial dot.
fn state_shape(
    kind: Shape,
    (x, y, width, height): (f64, f64, f64, f64),
    tooltip: Option<String>,
    elements: &mut Vec<Element>,
) {
    let rect = |class: &'static str, tooltip: Option<String>| {
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
    match kind {
        Shape::State => with_shadow(
            rect(
                "chartlet-flow-node chartlet-state chartlet-role-blue",
                tooltip,
            ),
            elements,
        ),
        Shape::Final => {
            with_shadow(
                rect(
                    "chartlet-flow-node chartlet-state chartlet-role-green",
                    tooltip,
                ),
                elements,
            );
            elements.push(Element::Rect(Rect {
                x: x + 4.0,
                y: y + 4.0,
                width: width - 8.0,
                height: height - 8.0,
                class: "chartlet-state-inner",
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
        Shape::Initial => elements.push(Element::Circle(Circle {
            cx: x + width / 2.0,
            cy: y + height / 2.0,
            radius: width / 2.0,
            class: "chartlet-state-initial",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip,
        })),
        _ => unreachable!("only the shapes of a state diagram come here"),
    }
}

/// The shapes of an architecture diagram beyond those of a flow chart.
fn component_shape(
    kind: Shape,
    (x, y, width, height): (f64, f64, f64, f64),
    tooltip: Option<String>,
    elements: &mut Vec<Element>,
) {
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
    let polygon = |points: Vec<(f64, f64)>, class: &'static str, tooltip: Option<String>| {
        Element::Polyline(diagram::polyline(points, class, tooltip))
    };
    let middle = y + height / 2.0;
    match kind {
        Shape::Person => {
            with_shadow(
                rect(x, y, "chartlet-flow-node chartlet-role-violet", tooltip),
                elements,
            );
            head(x + width / 2.0, y + 11.0, elements);
        }
        Shape::Frontend => {
            with_shadow(
                rect(x, y, "chartlet-flow-node chartlet-role-blue", tooltip),
                elements,
            );
            window_bar(x, y, width, elements);
        }
        Shape::Queue => {
            with_shadow(
                rect(
                    x + 4.0,
                    y - 4.0,
                    "chartlet-flow-node chartlet-role-violet",
                    None,
                ),
                elements,
            );
            with_shadow(
                rect(x, y, "chartlet-flow-node chartlet-role-violet", tooltip),
                elements,
            );
        }
        Shape::Bucket => with_shadow(
            polygon(
                vec![
                    (x, y),
                    (x + width, y),
                    (x + width - 8.0, y + height),
                    (x + 8.0, y + height),
                    (x, y),
                ],
                "chartlet-flow-node chartlet-role-teal",
                tooltip,
            ),
            elements,
        ),
        Shape::Cache => with_shadow(
            polygon(
                vec![
                    (x + 12.0, y),
                    (x + width - 12.0, y),
                    (x + width, middle),
                    (x + width - 12.0, y + height),
                    (x + 12.0, y + height),
                    (x, middle),
                    (x + 12.0, y),
                ],
                "chartlet-flow-node chartlet-role-amber",
                tooltip,
            ),
            elements,
        ),
        Shape::Shield => with_shadow(
            polygon(
                vec![
                    (x, y),
                    (x + width, y),
                    (x + width, y + height * 0.6),
                    (x + width / 2.0, y + height),
                    (x, y + height * 0.6),
                    (x, y),
                ],
                "chartlet-flow-node chartlet-role-red",
                tooltip,
            ),
            elements,
        ),
        Shape::Node | Shape::Artifact => {
            deployment_shape(kind, (x, y, width, height), tooltip, elements);
        }
        _ => unreachable!("only the shapes of an architecture diagram come here"),
    }
}

/// The shapes of a deployment: a node that runs other components, and what is deployed on it.
fn deployment_shape(
    kind: Shape,
    (x, y, width, height): (f64, f64, f64, f64),
    tooltip: Option<String>,
    elements: &mut Vec<Element>,
) {
    let rect = |class: &'static str, tooltip: Option<String>| {
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
    let polygon = |points: Vec<(f64, f64)>, class: &'static str, tooltip: Option<String>| {
        Element::Polyline(diagram::polyline(points, class, tooltip))
    };
    match kind {
        Shape::Node => {
            // The lid and the side of the box, drawn first so that the front covers their edges.
            with_shadow(
                polygon(
                    vec![
                        (x, y),
                        (x + 8.0, y - 6.0),
                        (x + width + 8.0, y - 6.0),
                        (x + width + 8.0, y + height - 6.0),
                        (x + width, y + height),
                        (x + width, y),
                        (x, y),
                    ],
                    "chartlet-flow-node chartlet-role-gray",
                    None,
                ),
                elements,
            );
            with_shadow(
                rect("chartlet-flow-node chartlet-role-gray", tooltip),
                elements,
            );
            elements.push(Element::Line(Line {
                x1: x + width,
                y1: y,
                x2: x + width + 8.0,
                y2: y - 6.0,
                class: "chartlet-flow-inner",
            }));
        }
        Shape::Artifact => {
            with_shadow(
                polygon(
                    vec![
                        (x, y),
                        (x + width - 12.0, y),
                        (x + width, y + 12.0),
                        (x + width, y + height),
                        (x, y + height),
                        (x, y),
                    ],
                    "chartlet-flow-node chartlet-role-blue",
                    tooltip,
                ),
                elements,
            );
            for (from, to) in [
                ((x + width - 12.0, y), (x + width - 12.0, y + 12.0)),
                ((x + width - 12.0, y + 12.0), (x + width, y + 12.0)),
            ] {
                elements.push(Element::Line(Line {
                    x1: from.0,
                    y1: from.1,
                    x2: to.0,
                    y2: to.1,
                    class: "chartlet-flow-inner",
                }));
            }
        }
        _ => unreachable!("only a node and an artifact come here"),
    }
}

/// The head on top of a person.
fn head(x: f64, y: f64, elements: &mut Vec<Element>) {
    elements.push(Element::Circle(Circle {
        cx: x,
        cy: y,
        radius: 6.0,
        class: "chartlet-arch-head",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
}

/// The window bar along the top of a frontend: a line and three dots.
fn window_bar(x: f64, y: f64, width: f64, elements: &mut Vec<Element>) {
    elements.push(Element::Line(Line {
        x1: x,
        y1: y + 12.0,
        x2: x + width,
        y2: y + 12.0,
        class: "chartlet-flow-inner",
    }));
    for dot in 0..3 {
        elements.push(Element::Circle(Circle {
            cx: x + 8.0 + 6.0 * f64::from(dot),
            cy: y + 6.0,
            radius: 1.8,
            class: "chartlet-arch-dot",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
}

/// A box with fully rounded ends.
fn pill(x: f64, y: f64, width: f64, height: f64) -> Vec<(f64, f64)> {
    const STEPS: u32 = 8;
    let radius = height / 2.0;
    let arc = |center: f64, from: f64| -> Vec<(f64, f64)> {
        (0..=STEPS)
            .map(|step| {
                let angle = from + std::f64::consts::PI * f64::from(step) / f64::from(STEPS);
                (
                    center + radius * angle.cos(),
                    y + radius + radius * angle.sin(),
                )
            })
            .collect()
    };
    let half = std::f64::consts::FRAC_PI_2;
    let mut points = arc(x + width - radius, -half);
    points.extend(arc(x + radius, half));
    points.push(points[0]);
    points
}
