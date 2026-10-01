//! The diverging color scale shared by warming stripes and calendar heatmaps: eight steps on
//! either side of a neutral middle, each step equally wide (see [`crate::spec::Diverging`]).
//!
//! The colors are two hues of equal lightness spacing, blue below the reference and red above
//! it, with a light grey middle. They are CSS custom properties, so a host page can replace any
//! step, and the same in the light and the dark theme: a scale that carries data must not change
//! its meaning with the page.

use crate::spec::Diverging;

/// One class per step, from the far end below the reference to the far end above it.
pub(crate) const CLASSES: [&str; 2 * Diverging::STEPS + 1] = [
    "chartlet-diverging-0",
    "chartlet-diverging-1",
    "chartlet-diverging-2",
    "chartlet-diverging-3",
    "chartlet-diverging-4",
    "chartlet-diverging-5",
    "chartlet-diverging-6",
    "chartlet-diverging-7",
    "chartlet-diverging-8",
    "chartlet-diverging-9",
    "chartlet-diverging-10",
    "chartlet-diverging-11",
    "chartlet-diverging-12",
    "chartlet-diverging-13",
    "chartlet-diverging-14",
    "chartlet-diverging-15",
    "chartlet-diverging-16",
];

pub(crate) const STYLE: &str = ".chartlet-root{--chartlet-diverging-0:#033761;--chartlet-diverging-1:#064f89;--chartlet-diverging-2:#0f66ac;--chartlet-diverging-3:#347ec4;--chartlet-diverging-4:#5b97d3;--chartlet-diverging-5:#81afdf;--chartlet-diverging-6:#a7c8ea;--chartlet-diverging-7:#cde0f5;--chartlet-diverging-8:#eeeeee;--chartlet-diverging-9:#f8d5d0;--chartlet-diverging-10:#f1b2aa;--chartlet-diverging-11:#e88f85;--chartlet-diverging-12:#dd6b60;--chartlet-diverging-13:#cd443d;--chartlet-diverging-14:#b32322;--chartlet-diverging-15:#901114;--chartlet-diverging-16:#690509}.chartlet-diverging-0{fill:var(--chartlet-diverging-0)}.chartlet-diverging-1{fill:var(--chartlet-diverging-1)}.chartlet-diverging-2{fill:var(--chartlet-diverging-2)}.chartlet-diverging-3{fill:var(--chartlet-diverging-3)}.chartlet-diverging-4{fill:var(--chartlet-diverging-4)}.chartlet-diverging-5{fill:var(--chartlet-diverging-5)}.chartlet-diverging-6{fill:var(--chartlet-diverging-6)}.chartlet-diverging-7{fill:var(--chartlet-diverging-7)}.chartlet-diverging-8{fill:var(--chartlet-diverging-8)}.chartlet-diverging-9{fill:var(--chartlet-diverging-9)}.chartlet-diverging-10{fill:var(--chartlet-diverging-10)}.chartlet-diverging-11{fill:var(--chartlet-diverging-11)}.chartlet-diverging-12{fill:var(--chartlet-diverging-12)}.chartlet-diverging-13{fill:var(--chartlet-diverging-13)}.chartlet-diverging-14{fill:var(--chartlet-diverging-14)}.chartlet-diverging-15{fill:var(--chartlet-diverging-15)}.chartlet-diverging-16{fill:var(--chartlet-diverging-16)}";

use crate::layout::{LABEL_SIZE, format_value};
use crate::scene::{Element, Rect, Text, TextAnchor};
use crate::spec::NumberStyle;

/// Width of one swatch of the color key.
const KEY_SWATCH: f64 = 14.0;

/// Width of the whole color key.
pub(crate) fn key_width() -> f64 {
    KEY_SWATCH * crate::layout::count(CLASSES.len())
}

/// A color key: one swatch per step, labelled with the outer ends and the reference below. The
/// outer labels read as "at most" and "at least", since values beyond them take the end steps.
pub(crate) fn push_key(
    elements: &mut Vec<Element>,
    left: f64,
    top: f64,
    scale: &Diverging,
    style: NumberStyle,
) {
    for (index, class) in CLASSES.iter().enumerate() {
        elements.push(Element::Rect(Rect {
            x: left + KEY_SWATCH * crate::layout::count(index),
            y: top,
            width: KEY_SWATCH,
            height: 10.0,
            class,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
    let baseline = top + 10.0 + LABEL_SIZE + 4.0;
    for (x, anchor, value) in [
        (left, TextAnchor::Start, scale.min),
        (
            left + key_width() / 2.0,
            TextAnchor::Middle,
            scale.reference,
        ),
        (left + key_width(), TextAnchor::End, scale.max),
    ] {
        elements.push(Element::Text(Text {
            x,
            y: baseline,
            class: "chartlet-tick",
            anchor,
            content: format_value(value, style),
        }));
    }
}
