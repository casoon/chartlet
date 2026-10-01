//! Turning a grid of labelled cells into outlines that can be drawn.
//!
//! The landscape decides ownership cell by cell, which is a staircase. What a map wants is a line.
//! Three steps get from one to the other: trace the edges between a cell that belongs and one that
//! does not, chain them into closed rings, then round the corners off until the staircase reads as
//! a coast.
//!
//! Nothing here knows what an atlas is. It works on any grid of cells that either belong or do not.

/// A corner of the grid: a lattice of `columns + 1` by `rows + 1` points.
type Corner = (usize, usize);

/// Rounds of corner cutting. Each one quarters the corners that are left; three take a staircase
/// to something that looks drawn.
const ROUNDS: usize = 3;

/// Every closed ring around the cells that `inside` accepts, in corner coordinates.
///
/// The edges come out directed, with the inside always on the right. That is what makes chaining
/// them unambiguous: at a corner where two rings pinch together, only one continuation keeps the
/// inside on the same hand, so the rings never swap halves.
pub(crate) fn rings(
    columns: usize,
    rows: usize,
    inside: &dyn Fn(usize, usize) -> bool,
) -> Vec<Vec<Corner>> {
    let mut edges: Vec<(Corner, Corner)> = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            if !inside(column, row) {
                continue;
            }
            if row == 0 || !inside(column, row - 1) {
                edges.push(((column, row), (column + 1, row)));
            }
            if column + 1 == columns || !inside(column + 1, row) {
                edges.push(((column + 1, row), (column + 1, row + 1)));
            }
            if row + 1 == rows || !inside(column, row + 1) {
                edges.push(((column + 1, row + 1), (column, row + 1)));
            }
            if column == 0 || !inside(column - 1, row) {
                edges.push(((column, row + 1), (column, row)));
            }
        }
    }
    chain(&edges)
}

/// Walks the directed edges into rings, taking each one exactly once.
fn chain(edges: &[(Corner, Corner)]) -> Vec<Vec<Corner>> {
    let mut leaving: std::collections::HashMap<Corner, Vec<usize>> =
        std::collections::HashMap::new();
    for (index, (from, _)) in edges.iter().enumerate() {
        leaving.entry(*from).or_default().push(index);
    }

    let mut used = vec![false; edges.len()];
    let mut found = Vec::new();
    for start in 0..edges.len() {
        if used[start] {
            continue;
        }
        let mut ring = Vec::new();
        let mut current = start;
        loop {
            used[current] = true;
            ring.push(edges[current].0);
            let Some(candidates) = leaving.get(&edges[current].1) else {
                break;
            };
            let Some(next) = candidates.iter().copied().find(|index| !used[*index]) else {
                break;
            };
            current = next;
        }
        // Three corners cannot enclose anything on a square lattice.
        if ring.len() > 3 {
            found.push(ring);
        }
    }
    found
}

/// Drops the middle of three points in a line. A staircase has long straight runs along the frame
/// and wherever an area is wide, and carrying every cell of them through the smoothing would cost
/// points without changing the curve.
pub(crate) fn straighten(ring: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if ring.len() < 3 {
        return ring.to_vec();
    }
    let mut kept: Vec<(f64, f64)> = Vec::with_capacity(ring.len());
    for index in 0..ring.len() {
        let before = ring[(index + ring.len() - 1) % ring.len()];
        let point = ring[index];
        let after = ring[(index + 1) % ring.len()];
        let (ax, ay) = (point.0 - before.0, point.1 - before.1);
        let (bx, by) = (after.0 - point.0, after.1 - point.1);
        if (ax * by - ay * bx).abs() > f64::EPSILON {
            kept.push(point);
        }
    }
    if kept.len() < 3 { ring.to_vec() } else { kept }
}

/// Chaikin's corner cutting on a closed ring: every corner becomes two points a quarter of the
/// way along each of its edges. Repeated, it converges on a curve; three rounds are enough to
/// lose the grid and keep the shape.
pub(crate) fn smooth(ring: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut points = ring.to_vec();
    for _ in 0..ROUNDS {
        if points.len() < 3 {
            break;
        }
        let mut next = Vec::with_capacity(points.len() * 2);
        for index in 0..points.len() {
            let (from, to) = (points[index], points[(index + 1) % points.len()]);
            next.push((
                from.0 + (to.0 - from.0) * 0.25,
                from.1 + (to.1 - from.1) * 0.25,
            ));
            next.push((
                from.0 + (to.0 - from.0) * 0.75,
                from.1 + (to.1 - from.1) * 0.75,
            ));
        }
        points = next;
    }
    points
}

/// Ramer–Douglas–Peucker: keeps the points that carry the shape and drops the rest. Smoothing
/// quadruples the count, and most of what it adds sits on a line the eye already follows.
///
/// The tolerance belongs to the caller because it belongs to the grid: a fraction of a cell keeps
/// every turn the cells actually make, whatever size those cells are.
pub(crate) fn simplify(ring: &[(f64, f64)], tolerance: f64) -> Vec<(f64, f64)> {
    if ring.len() < 4 {
        return ring.to_vec();
    }
    let mut keep = vec![false; ring.len()];
    keep[0] = true;
    keep[ring.len() - 1] = true;
    let mut spans = vec![(0, ring.len() - 1)];
    while let Some((from, to)) = spans.pop() {
        if to <= from + 1 {
            continue;
        }
        let (mut worst, mut at) = (0.0, from);
        for (index, point) in ring.iter().enumerate().take(to).skip(from + 1) {
            let away = off_line(*point, ring[from], ring[to]);
            if away > worst {
                worst = away;
                at = index;
            }
        }
        if worst > tolerance {
            keep[at] = true;
            spans.push((from, at));
            spans.push((at, to));
        }
    }
    ring.iter()
        .zip(&keep)
        .filter_map(|(point, held)| held.then_some(*point))
        .collect()
}

/// How far a point lies off the line through two others.
fn off_line(point: (f64, f64), from: (f64, f64), to: (f64, f64)) -> f64 {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx.hypot(dy);
    if length < f64::EPSILON {
        return (point.0 - from.0).hypot(point.1 - from.1);
    }
    ((point.0 - from.0) * dy - (point.1 - from.1) * dx).abs() / length
}

/// How far every cell that belongs lies from the nearest cell that does not, counted in cells.
///
/// Cells that do not belong are zero, and so is everything beyond the grid: an area pressed
/// against the frame is not deep there, which is exactly what a label wants to know.
pub(crate) fn depth(
    columns: usize,
    rows: usize,
    inside: &dyn Fn(usize, usize) -> bool,
) -> Vec<u32> {
    let mut depth = vec![u32::MAX; columns * rows];
    let mut wave: Vec<usize> = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            let cell = row * columns + column;
            if inside(column, row) {
                if row == 0 || column == 0 || row + 1 == rows || column + 1 == columns {
                    depth[cell] = 1;
                    wave.push(cell);
                }
            } else {
                depth[cell] = 0;
                wave.push(cell);
            }
        }
    }

    let mut head = 0;
    while head < wave.len() {
        let cell = wave[head];
        head += 1;
        let (column, row) = (cell % columns, cell / columns);
        let near = [
            (column > 0).then(|| cell - 1),
            (column + 1 < columns).then(|| cell + 1),
            (row > 0).then(|| cell - columns),
            (row + 1 < rows).then(|| cell + columns),
        ];
        for next in near.into_iter().flatten() {
            if depth[next] == u32::MAX {
                depth[next] = depth[cell] + 1;
                wave.push(next);
            }
        }
    }
    depth
}

/// The cells an area has the most room in, each at least `apart` cells from the ones before it.
///
/// The first is the point furthest from anything that is not this area — where a label disturbs
/// least. The rest are fallbacks for when something is already written there.
pub(crate) fn roomiest(
    depth: &[u32],
    columns: usize,
    apart: usize,
    most: usize,
) -> Vec<(usize, usize, u32)> {
    let mut ranked: Vec<usize> = (0..depth.len()).filter(|cell| depth[*cell] > 0).collect();
    // By depth, deepest first, and by cell index where two are equally deep.
    ranked.sort_by(|a, b| depth[*b].cmp(&depth[*a]).then(a.cmp(b)));

    let mut found: Vec<(usize, usize, u32)> = Vec::new();
    for cell in ranked {
        if found.len() == most {
            break;
        }
        let (column, row) = (cell % columns, cell / columns);
        if found.iter().all(|(other_column, other_row, _)| {
            column.abs_diff(*other_column) + row.abs_diff(*other_row) >= apart
        }) {
            found.push((column, row, depth[cell]));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{depth, rings, roomiest, simplify, smooth, straighten};

    fn square(columns: usize, rows: usize, held: &[(usize, usize)]) -> Vec<Vec<(usize, usize)>> {
        rings(columns, rows, &|column, row| held.contains(&(column, row)))
    }

    #[test]
    fn one_cell_is_one_ring_of_four_corners() {
        let found = square(3, 3, &[(1, 1)]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].len(), 4);
    }

    #[test]
    fn two_separate_cells_are_two_rings() {
        let found = square(5, 3, &[(0, 0), (4, 2)]);
        assert_eq!(found.len(), 2);
    }

    /// A ring with a hole has an outer boundary and an inner one, and both have to come out.
    #[test]
    fn a_hole_is_a_ring_of_its_own() {
        let held: Vec<(usize, usize)> = (0..3)
            .flat_map(|row| (0..3).map(move |column| (column, row)))
            .filter(|cell| *cell != (1, 1))
            .collect();
        let found = square(3, 3, &held);
        assert_eq!(found.len(), 2, "outer boundary and hole");
        assert!(found.iter().any(|ring| ring.len() == 4), "the hole");
    }

    #[test]
    fn nothing_held_is_no_ring() {
        assert_eq!(square(4, 4, &[]), Vec::<Vec<(usize, usize)>>::new());
    }

    #[test]
    fn straightening_a_long_edge_keeps_only_its_corners() {
        let staircase: Vec<(f64, f64)> = vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (3.0, 1.0),
            (0.0, 1.0),
        ];
        assert_eq!(straighten(&staircase).len(), 4);
    }

    #[test]
    fn smoothing_stays_inside_what_it_rounds() {
        let box_ring = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let rounded = smooth(&box_ring);
        assert!(rounded.len() > box_ring.len());
        for (x, y) in rounded {
            assert!((0.0..=10.0).contains(&x) && (0.0..=10.0).contains(&y));
        }
    }

    #[test]
    fn simplifying_drops_what_lies_on_the_line() {
        let along: Vec<(f64, f64)> = (0..40).map(|step| (f64::from(step), 0.0)).collect();
        assert_eq!(simplify(&along, 0.4).len(), 2);
    }

    #[test]
    fn depth_counts_the_way_out() {
        // A 5 by 5 block in a 7 by 7 grid: the middle is three cells from the outside.
        let held: Vec<(usize, usize)> = (1..6)
            .flat_map(|row| (1..6).map(move |column| (column, row)))
            .collect();
        let found = depth(7, 7, &|column, row| held.contains(&(column, row)));
        assert_eq!(found[3 * 7 + 3], 3, "the middle");
        assert_eq!(found[7 + 1], 1, "a corner of the block");
        assert_eq!(found[0], 0, "outside");
    }

    /// An area pressed against the frame has no room there, whatever the grid says.
    #[test]
    fn the_frame_counts_as_outside() {
        let found = depth(5, 5, &|_, _| true);
        assert_eq!(found[0], 1);
        assert_eq!(found[2 * 5 + 2], 3);
    }

    #[test]
    fn the_roomiest_places_keep_their_distance() {
        let found = depth(9, 9, &|_, _| true);
        let spots = roomiest(&found, 9, 4, 3);
        assert_eq!(spots.len(), 3);
        for (index, (column, row, _)) in spots.iter().enumerate() {
            for (other_column, other_row, _) in &spots[index + 1..] {
                assert!(column.abs_diff(*other_column) + row.abs_diff(*other_row) >= 4);
            }
        }
    }

    #[test]
    fn the_whole_run_is_deterministic() {
        let held: Vec<(usize, usize)> = (0..6)
            .flat_map(|row| (0..7).map(move |column| (column, row)))
            .filter(|(column, row)| column + row > 2 && column * row < 20)
            .collect();
        let once = square(7, 6, &held);
        let twice = square(7, 6, &held);
        assert_eq!(once, twice);
        for ring in &once {
            let points: Vec<(f64, f64)> = ring
                .iter()
                .map(|(column, row)| {
                    (
                        f64::from(u32::try_from(*column).unwrap()) * 4.0,
                        f64::from(u32::try_from(*row).unwrap()) * 4.0,
                    )
                })
                .collect();
            assert_eq!(
                simplify(&smooth(&straighten(&points)), 0.4),
                simplify(&smooth(&straighten(&points)), 0.4)
            );
        }
    }
}
