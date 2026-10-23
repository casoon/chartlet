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
    /// The filled outline of a topic map area and the center it was placed around. The SVG
    /// carries that center as `data-cx`/`data-cy`, so a host page need not recompute it from the
    /// outline.
    TopicArea(Polyline, (f64, f64)),
    Rect(Rect),
    SeriesText(Text, usize),
    /// Text the layout decides more about than its class can say: a topic map sizes an area's
    /// name by how much room that area has, and ties it to the area it names.
    StyledText(Text, TextStyle),
    Text(Text),
    /// Draws nothing: a `data-*` hook for the optional interactive module, written only with the
    /// `hooks` render option.
    Hook(Hook),
    /// Opens a group whose classes tie what it holds to the diagram nodes it belongs to, for the
    /// focus of the HTML profile; [`Element::GroupEnd`] closes it.
    Group(String),
    /// Opens a group like [`Element::Group`] around a diagram node that has an ID in the
    /// specification: the classes, then that ID, which the `hooks` render option writes out as
    /// `data-chartlet-node`. [`Element::GroupEnd`] closes it.
    NodeGroup(String, String),
    /// Opens a group like [`Element::Group`] around a diagram edge that has an ID in the
    /// specification: the classes, then that ID, which the `hooks` render option writes out as
    /// `data-chartlet-edge`. [`Element::GroupEnd`] closes it.
    EdgeGroup(String, String),
    GroupEnd,
    /// Draws nothing: where a diagram node stands, for the focus link the HTML profile lays over
    /// it.
    Hotspot(Hotspot),
    /// A symbol of an architecture component, see [`crate::symbols`].
    Symbol(SymbolUse),
}

/// Where a symbol stands: its name and the top left corner of its square.
#[derive(Debug, Clone)]
pub(crate) struct SymbolUse {
    pub name: &'static str,
    pub x: f64,
    pub y: f64,
}

/// A diagram node the HTML profile can focus: its index, its name, and its box.
#[derive(Debug, Clone)]
pub(crate) struct Hotspot {
    pub node: usize,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The geometry and grouping the interactive module reads from a chart, see [`Element::Hook`].
#[derive(Debug, Clone)]
pub(crate) enum Hook {
    /// The plot of one pane: `x` maps the time or category axis as `(value, pixel)` pairs, linear
    /// between neighbours; `y` maps the value axis by its two ends, linear or, when `log`, linear
    /// in the logarithm of the value.
    Plot {
        pane: usize,
        x: Vec<(f64, f64)>,
        y: [(f64, f64); 2],
        log: bool,
    },
    /// Opens a group around what one layer draws, by its index among all layers of all panes.
    Layer(usize),
    /// Opens a group around a diagram object that has an ID in the specification but no focus
    /// group of its own, such as a frame: the name of the attribute that carries the ID, then the
    /// ID.
    Object(&'static str, String),
    /// Closes the group of the last [`Hook::Layer`] or [`Hook::Object`].
    End,
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
