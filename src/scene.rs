#[derive(Debug, Clone)]
pub(crate) struct Scene {
    pub width: u32,
    pub height: u32,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone)]
pub(crate) enum Element {
    Circle(Circle),
    Line(Line),
    Polyline(Polyline),
    Rect(Rect),
    SeriesText(Text, usize),
    /// Text the layout decides more about than its class can say: a topic map sizes an area's
    /// name by how much room that area has, and ties it to the area it names.
    StyledText(Text, TextStyle),
    Text(Text),
}

/// What the layout knows about a piece of text beyond its class.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TextStyle {
    /// Font size in pixels, when the layout rather than the stylesheet decides it.
    pub size: Option<f64>,
    /// The area this text belongs to, see [`Polyline::topic`].
    pub topic: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct Circle {
    pub cx: f64,
    pub cy: f64,
    pub radius: f64,
    pub class: &'static str,
    /// The area this belongs to, see [`Polyline::topic`].
    pub topic: Option<usize>,
    /// Series index (0-based) for multi-series charts; used for CSS filtering.
    pub series_index: Option<usize>,
    /// Index into the specification-declared colors, see [`Polyline`].
    pub style_index: Option<usize>,
    /// Accessible tooltip text displayed on hover via `<title>`.
    pub tooltip: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Line {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub class: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub class: &'static str,
    /// Series index (0-based) for multi-series charts; used for CSS filtering.
    pub series_index: Option<usize>,
    /// Index into the specification-declared colors, see [`Polyline`].
    pub style_index: Option<usize>,
    /// Accessible tooltip text displayed on hover via `<title>`.
    pub tooltip: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Polyline {
    pub points: Vec<(f64, f64)>,
    pub class: &'static str,
    /// The index of the topic map area this belongs to, written out as a `chartlet-topic-N`
    /// class. It is what lets the selection styles address one area without any scripting.
    pub topic: Option<usize>,
    /// Series index (0-based) for multi-series charts; used for CSS filtering.
    pub series_index: Option<usize>,
    /// Index into the specification-declared colors: a polyline is painted by
    /// `.chartlet-style-N` (stroke), a rect by `.chartlet-style-N-swatch` (fill).
    pub style_index: Option<usize>,
    /// Accessible tooltip text displayed on hover via `<title>`.
    pub tooltip: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TextAnchor {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone)]
pub(crate) struct Text {
    pub x: f64,
    pub y: f64,
    pub class: &'static str,
    pub anchor: TextAnchor,
    pub content: String,
}
