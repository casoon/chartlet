//! Small constructors for the marks the outline diagrams (fishbone, Venn, kanban, story map,
//! journey and value map) are made of, so that their layouts read as geometry.

use crate::scene::{Element, Line, Rect, Text, TextAnchor};

/// A rectangle with a class and an optional tooltip.
pub(crate) fn rect(
    (x, y, width, height): (f64, f64, f64, f64),
    class: &'static str,
    tooltip: Option<String>,
) -> Element {
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
}

/// A straight line.
pub(crate) fn line((x1, y1): (f64, f64), (x2, y2): (f64, f64), class: &'static str) -> Element {
    Element::Line(Line {
        x1,
        y1,
        x2,
        y2,
        class,
    })
}

/// A text whose baseline starts at `x`, `y`.
pub(crate) fn text(
    (x, y): (f64, f64),
    anchor: TextAnchor,
    class: &'static str,
    content: impl Into<String>,
) -> Element {
    Element::Text(Text {
        x,
        y,
        class,
        anchor,
        content: content.into(),
    })
}
