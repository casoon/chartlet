//! The one projection of the diagrams drawn at an angle: 2:1 dimetric. The `x` axis of the grid
//! runs right and down, the `y` axis left and down, both at a slope of one to two, and `z` runs
//! straight up in pixels. Everything an exploded stack or a plan draws goes through [`project`], so
//! that every corner of every face is computed the same way and the output is deterministic.

/// A point of the grid at `(x, y)` grid units and `z` pixels above the ground, on the page, relative
/// to the point `(0, 0, 0)`; `unit` is the length of one grid unit along an axis, in pixels of the
/// page's horizontal.
pub(crate) fn project(unit: f64, x: f64, y: f64, z: f64) -> (f64, f64) {
    ((x - y) * unit, (x + y) * unit * 0.5 - z)
}

/// The corners of the horizontal rectangle `(x, y, width, depth)` at height `z`, clockwise on the
/// page from the back corner: back, right, front, left.
pub(crate) fn rectangle(
    unit: f64,
    (x, y, width, depth): (f64, f64, f64, f64),
    z: f64,
) -> [(f64, f64); 4] {
    [
        project(unit, x, y, z),
        project(unit, x + width, y, z),
        project(unit, x + width, y + depth, z),
        project(unit, x, y + depth, z),
    ]
}

/// A closed outline: the points, with the first repeated at the end.
pub(crate) fn closed(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut outline = points.to_vec();
    outline.push(points[0]);
    outline
}

/// The faces of a box `(x, y, width, depth)` standing on the ground at `z0` and `height` tall, as
/// closed outlines in the order they are drawn: the face on the left of the page, the face on the
/// right, the top. The back faces are never seen from above.
pub(crate) struct Faces {
    pub left: Vec<(f64, f64)>,
    pub right: Vec<(f64, f64)>,
    pub top: Vec<(f64, f64)>,
}

pub(crate) fn faces(
    unit: f64,
    (x, y, width, depth): (f64, f64, f64, f64),
    (z0, height): (f64, f64),
) -> Faces {
    let low = rectangle(unit, (x, y, width, depth), z0);
    let high = rectangle(unit, (x, y, width, depth), z0 + height);
    Faces {
        // The front left face joins the left corner and the front corner.
        left: closed(&[low[3], low[2], high[2], high[3]]),
        // The front right face joins the front corner and the right corner.
        right: closed(&[low[2], low[1], high[1], high[2]]),
        top: closed(&high),
    }
}
