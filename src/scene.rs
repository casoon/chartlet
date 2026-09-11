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
    Text(Text),
}

#[derive(Debug, Clone)]
pub(crate) struct Circle {
    pub cx: f64,
    pub cy: f64,
    pub radius: f64,
    pub class: &'static str,
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
}

#[derive(Debug, Clone)]
pub(crate) struct Polyline {
    pub points: Vec<(f64, f64)>,
    pub class: &'static str,
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
