//! Where an edge label may stand: beside or on a stretch of its own edge, clear of every other
//! edge, and closer to its own line than to any other, so that it reads as the label of its edge.

use super::{edges::Label, place::Plan};
use crate::{
    diagram::{CHIP_REACH, HEAD, chip_box_tech},
    metrics::TextMetrics,
    scene::TextAnchor,
};

/// A box on the page: left, top, width, height.
pub(super) type Rect = (f64, f64, f64, f64);
/// A straight piece of an edge on the page.
type Segment = ((f64, f64), (f64, f64));

/// The least distance between a label and an edge that is not its own.
const CLEAR: f64 = 2.0;
/// The most a label may stand away from its own edge and still read as its label: the gap of
/// the roomiest look, and a little more.
const NEAR: f64 = 12.0;
/// Half the width of an arrowhead.
const HEAD_HALF: f64 = 3.5;
/// How far the candidates for a label lie apart along a stretch of its edge.
const STEP: f64 = 8.0;
/// The room a label leaves at both ends of the stretch it stands on or beside.
const LEAD: f64 = 4.0;

/// The ways of all edges on the page, and the arrowheads at their ends, against which the place
/// of a label is checked.
pub(super) struct Lines {
    /// Per edge, the corners of its way from the step it leaves to the step it reaches; empty
    /// for an edge from a step to itself.
    ways: Vec<Vec<(f64, f64)>>,
    /// The box around the arrowhead of every edge that has a way.
    heads: Vec<Rect>,
}

impl Lines {
    pub(super) fn new(plan: &Plan) -> Self {
        let ways: Vec<Vec<(f64, f64)>> = plan
            .routes
            .iter()
            .map(|route| {
                route.as_ref().map_or_else(Vec::new, |route| {
                    route.points.iter().map(|point| plan.page(*point)).collect()
                })
            })
            .collect();
        let heads = ways
            .iter()
            .filter(|way| way.len() >= 2)
            .map(|way| {
                let (tip, before) = (way[way.len() - 1], way[way.len() - 2]);
                let length = (tip.0 - before.0).hypot(tip.1 - before.1).max(1e-9);
                let back = (
                    tip.0 - (tip.0 - before.0) / length * HEAD,
                    tip.1 - (tip.1 - before.1) / length * HEAD,
                );
                let (left, top) = (tip.0.min(back.0) - HEAD_HALF, tip.1.min(back.1) - HEAD_HALF);
                let (right, bottom) =
                    (tip.0.max(back.0) + HEAD_HALF, tip.1.max(back.1) + HEAD_HALF);
                (left, top, right - left, bottom - top)
            })
            .collect();
        Self { ways, heads }
    }

    fn segments(&self, edge: usize) -> impl Iterator<Item = Segment> + '_ {
        self.ways[edge].windows(2).map(|pair| (pair[0], pair[1]))
    }

    /// Whether a label of `edge` in `rect` reads as that edge's: it stands at its own edge, covers
    /// no other edge and no arrowhead, and no other edge lies closer to it than its own.
    pub(super) fn reads_as(&self, edge: usize, rect: Rect) -> bool {
        // A plate that only brushes the corner of an arrowhead does not cover it.
        if self.heads.iter().any(|head| overlap(rect, *head, -2.0)) {
            return false;
        }
        let own = self
            .segments(edge)
            .map(|segment| distance(rect, segment))
            .fold(f64::INFINITY, f64::min);
        own <= NEAR
            && (0..self.ways.len())
                .filter(|other| *other != edge)
                .flat_map(|other| self.segments(other))
                .all(|segment| {
                    let away = distance(rect, segment);
                    away >= CLEAR && away > own + 0.5
                })
    }

    /// The places a label of `edge` with a plate of `size` may take, best first: along its way
    /// from the step it leaves, on each stretch first beside the line on either side, then on
    /// the line itself. Each is the first baseline of the text and its anchor. `gap` is the
    /// distance the look keeps between a plate and its line.
    pub(super) fn candidates(
        &self,
        edge: usize,
        (width, height): (f64, f64),
        gap: f64,
    ) -> Vec<((f64, f64), TextAnchor)> {
        // From the top of a plate to the first baseline, and from its side to the text.
        const BASELINE: f64 = 14.0;
        const INSET: f64 = CHIP_REACH - 2.0;
        let reach = CHIP_REACH + 2.0 + gap;
        let beside = reach - INSET;
        let way = &self.ways[edge];
        let last = way.len().saturating_sub(2);
        let mut candidates = Vec::new();
        for (index, pair) in way.windows(2).enumerate() {
            let (from, to) = (pair[0], pair[1]);
            // The last stretch ends in the arrowhead.
            let trail = if index == last { HEAD + 6.0 } else { LEAD };
            if (from.0 - to.0).abs() < 0.5 {
                let direction = (to.1 - from.1).signum();
                let room = (to.1 - from.1).abs() - LEAD - trail - height;
                let tops = slide(room).map(|along| {
                    let start = from.1 + direction * (LEAD + along);
                    if direction > 0.0 {
                        start
                    } else {
                        start - height
                    }
                });
                // A stretch shorter than the label takes it beside its middle, not on it.
                let middle = f64::midpoint(from.1, to.1) - height / 2.0;
                for (x, anchor) in [
                    (from.0 + reach, TextAnchor::Start),
                    (from.0 - reach, TextAnchor::End),
                    (from.0, TextAnchor::Middle),
                ] {
                    candidates.extend(tops.clone().map(|top| ((x, top + BASELINE), anchor)));
                    if room < 0.0 && anchor != TextAnchor::Middle {
                        candidates.push(((x, middle + BASELINE), anchor));
                    }
                }
            } else if (from.1 - to.1).abs() < 0.5 {
                let direction = (to.0 - from.0).signum();
                let room = (to.0 - from.0).abs() - LEAD - trail - width;
                let middles = slide(room).map(|along| {
                    let start = from.0 + direction * (LEAD + along);
                    start + direction * width / 2.0
                });
                let on = from.1 - height / 2.0;
                for top in [from.1 + beside, from.1 - beside - height, on] {
                    candidates.extend(
                        middles
                            .clone()
                            .map(|middle| ((middle, top + BASELINE), TextAnchor::Middle)),
                    );
                    if room < 0.0 && (top - on).abs() > 0.5 {
                        let middle = f64::midpoint(from.0, to.0);
                        candidates.push(((middle, top + BASELINE), TextAnchor::Middle));
                    }
                }
            }
        }
        candidates
    }
}

/// The distances from the start of a stretch with `room` to spare at which a label is tried.
fn slide(room: f64) -> impl Iterator<Item = f64> + Clone {
    // A stretch is at most a canvas long.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let steps = if room < 0.0 {
        0
    } else {
        (room / STEP).floor() as usize + 1
    };
    (0..steps).map(|step| crate::layout::count(step) * STEP)
}

/// Whether two boxes lie closer than `air` to each other.
pub(super) fn overlap(a: Rect, b: Rect, air: f64) -> bool {
    a.0 < b.0 + b.2 + air && b.0 < a.0 + a.2 + air && a.1 < b.1 + b.3 + air && b.1 < a.1 + a.3 + air
}

/// The distance between a box and a straight piece of an edge that runs along or across the
/// page; 0 where the piece touches or crosses the box.
fn distance((x, y, width, height): Rect, (a, b): Segment) -> f64 {
    let gap = |low: f64, high: f64, from: f64, to: f64| {
        let (from, to) = (from.min(to), from.max(to));
        (low - to).max(from - high).max(0.0)
    };
    gap(x, x + width, a.0, b.0).hypot(gap(y, y + height, a.1, b.1))
}

impl Label {
    /// The plate of the label where it stands.
    pub(super) fn plate(&self, metrics: &impl TextMetrics) -> Rect {
        chip_box_tech(
            &self.lines,
            self.tech.as_deref(),
            self.at,
            self.anchor,
            metrics,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Lines, distance, overlap};
    use crate::scene::TextAnchor;

    /// Two edges: one down at x = 100 from 0 to 200, one that crosses it at y = 120.
    fn lines() -> Lines {
        Lines {
            ways: vec![
                vec![(100.0, 0.0), (100.0, 200.0)],
                vec![(40.0, 120.0), (160.0, 120.0), (160.0, 200.0)],
            ],
            heads: vec![(96.0, 188.0, 8.0, 16.0), (156.0, 188.0, 8.0, 16.0)],
        }
    }

    #[test]
    fn a_box_is_as_far_from_a_line_as_its_nearest_side() {
        let rect = (10.0, 10.0, 20.0, 10.0);
        assert!((distance(rect, ((40.0, 0.0), (40.0, 50.0))) - 10.0).abs() < 1e-9);
        assert!(distance(rect, ((0.0, 15.0), (50.0, 15.0))) < 1e-9);
        assert!((distance(rect, ((33.0, 24.0), (60.0, 24.0))) - 5.0).abs() < 1e-9);
        assert!(overlap(rect, (32.0, 10.0, 5.0, 5.0), 3.0));
        assert!(!overlap(rect, (34.0, 10.0, 5.0, 5.0), 3.0));
    }

    #[test]
    fn a_label_reads_as_its_edge_only_clear_of_the_others_and_nearest_to_its_own() {
        let lines = lines();
        // Beside its own line, far from the other.
        assert!(lines.reads_as(0, (108.0, 20.0, 40.0, 20.0)));
        // Across the line of the other edge.
        assert!(!lines.reads_as(0, (108.0, 110.0, 40.0, 20.0)));
        // Clear of both, but nearer to the other edge than to its own.
        assert!(!lines.reads_as(0, (114.0, 96.0, 40.0, 20.0)));
        // Clear of both and nearer to its own, but too far from it to belong to it.
        assert!(!lines.reads_as(0, (20.0, 20.0, 40.0, 20.0)));
        // On an arrowhead.
        assert!(!lines.reads_as(0, (80.0, 180.0, 40.0, 20.0)));
        // On its own line, where the other edge does not cross.
        assert!(lines.reads_as(1, (110.0, 110.0, 40.0, 20.0)));
        assert!(!lines.reads_as(1, (80.0, 110.0, 40.0, 20.0)));
    }

    #[test]
    fn candidates_follow_the_way_of_the_edge_from_its_start() {
        let lines = lines();
        let candidates = lines.candidates(1, (40.0, 20.0), 0.0);
        // First below the stretch across, at its start; the plate keeps 4 px from the line.
        assert_eq!(candidates[0], ((64.0, 138.0), TextAnchor::Middle));
        // Then along it, above it, and on it.
        assert!(candidates.contains(&((72.0, 138.0), TextAnchor::Middle)));
        assert!(candidates.contains(&((64.0, 110.0), TextAnchor::Middle)));
        assert!(candidates.contains(&((64.0, 124.0), TextAnchor::Middle)));
        // The last stretch leaves room for the arrowhead and takes the label beside it.
        assert!(candidates.contains(&((169.0, 138.0), TextAnchor::Start)));
        assert!(candidates.iter().all(|((_, y), _)| *y <= 200.0 - 14.0));
        // A stretch shorter than the label takes it below and above its middle, not on it.
        let wide = lines.candidates(1, (130.0, 20.0), 0.0);
        assert_eq!(wide[0], ((100.0, 138.0), TextAnchor::Middle));
        assert_eq!(wide[1], ((100.0, 110.0), TextAnchor::Middle));
        assert!(wide[2..].iter().all(|((x, _), _)| *x >= 151.0));
    }
}
