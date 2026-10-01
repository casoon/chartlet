use std::collections::HashMap;

use super::{
    PLOT_MARGIN,
    axis::format_value,
    count, fit_text,
    title::{push_title, title_extra},
};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    noise,
    scene::{Circle, Element, Line, Polyline, Rect, Scene, Text, TextAnchor, TextStyle},
    spec::{ChartSpec, Corner, NumberStyle, TopicLinkSpec, TopicMapSpec, TopicSpec},
};

/// Fraction of the plot area topic circles fill; the rest is sea. Matches the design brief.
const TOPIC_FILL_RATIO: f64 = 0.42;
/// Island radius, relative to the smallest topic's radius.
const ISLAND_RADIUS_FACTOR: f64 = 0.55;
/// Gap enforced between two circle edges, relative to the smaller of the two radii.
const SEA_GAP_FACTOR: f64 = 0.14;
/// The golden angle: successive spiral points never align radially, so the starting layout has
/// no seams for the relaxation to get stuck on.
const SPIRAL_ANGLE: f64 = 2.399_96;
/// Fixed iteration count, so the same specification always relaxes to the same layout.
const RELAXATION_ITERATIONS: u32 = 120;
/// Extra collision-only passes after the main relaxation: a single separation pass per iteration
/// leaves a residual overlap of a few percent of the smaller radius (attraction and centering
/// keep pulling circles back together while collisions are resolved one pair at a time); these
/// passes settle that residual without the competing forces that caused it.
const SETTLE_ITERATIONS: u32 = 60;
/// Share of the distance to a link's target length pulled back per iteration.
const LINK_PULL_FACTOR: f64 = 0.08;
/// Share of the distance to the center corrected per iteration, so the cluster does not drift.
const CENTERING_FACTOR: f64 = 0.01;
/// At most this many links stay attached to any one topic or island; weaker ones are dropped
/// with a warning rather than cluttering that area with routes.
const MAX_LINKS_PER_TOPIC: usize = 3;
const ISLAND_GRID_COLUMNS: usize = 16;
const ISLAND_GRID_ROWS: usize = 10;
/// How far a coastline may dip inside and bulge outside its nominal radius. The relaxation keeps
/// areas [`MAX_WOBBLE`] apart rather than one nominal radius, so however the noise falls, two
/// coastlines can never touch; labels are measured against [`MIN_WOBBLE`], the narrowest the
/// coast can be where the text sits.
const MIN_WOBBLE: f64 = 0.84;
const MAX_WOBBLE: f64 = 1.16;
/// The two noise octaves: the first one cuts bays, the second adds the small jags.
const COAST_AMPLITUDES: [f64; 2] = [0.16, 0.05];
const COAST_FREQUENCIES: [f64; 2] = [1.7, 4.3];
/// Each depth line sits this much further out than the coast before it.
const DEPTH_BAND_STEP: f64 = 0.05;
/// Stroke width of the coastal halo. Must match `chartlet-topic-halo` in the stylesheet: half of
/// it is kept free at the canvas edge so the halo is never cut off.
const HALO_WIDTH: f64 = 10.0;
/// Path points stay inside this share of the nominal radius, which is well within the narrowest
/// the coastline can be.
const PATH_POINT_DISC: f64 = 0.7;
/// Radius of a single path point, and the distance two of them keep from each other.
const PATH_POINT_RADIUS: f64 = 2.2;
const PATH_POINT_SPACING: f64 = 6.0;
/// An area's name grows with the room it has, between these two sizes. The count below it stays
/// put: it carries information rather than decoration, so it is never shrunk to fit.
const TOPIC_LABEL_MIN: f64 = 19.0;
const TOPIC_LABEL_MAX: f64 = 34.0;
const TOPIC_LABEL_RATIO: f64 = 0.34;
const TOPIC_VALUE_SIZE: f64 = 18.0;
/// Distance from the name's baseline down to the count's.
const TOPIC_VALUE_DROP: f64 = 28.0;
/// Size of a label that had to move out of its area. Not smaller than this: a chart authored at
/// 1200 and shown in an 880 pixel container renders text at 0.73 of its size, and 12 effective
/// pixels is the floor.
const TOPIC_OUTSIDE_SIZE: f64 = 17.0;
/// Distance the name of an outside label keeps from its count, and from the next label below it.
const TOPIC_OUTSIDE_DROP: f64 = 20.0;
const TOPIC_OUTSIDE_SPACING: f64 = 46.0;
/// Clear space kept between the end of a label and the coast beside it.
const LABEL_PADDING: f64 = 4.0;
/// Cells the graticule divides the sea into. Enough to read as a map grid, few enough that it
/// stays behind the areas rather than competing with them.
const GRATICULE_COLUMNS: usize = 5;
const GRATICULE_ROWS: usize = 3;
/// Radius of the compass rose, and the clear water it keeps around itself.
const COMPASS_RADIUS: f64 = 26.0;
const FURNITURE_MARGIN: f64 = 10.0;
/// Type sizes and padding of the cartouche.
const CARTOUCHE_HEADING_SIZE: f64 = 15.0;
const CARTOUCHE_META_SIZE: f64 = 12.0;
const CARTOUCHE_PADDING: f64 = 12.0;
const CARTOUCHE_HEIGHT: f64 = 58.0;

/// Keeps at most [`MAX_LINKS_PER_TOPIC`] links per topic or island, strongest weight first;
/// weaker links beyond that are reported as `link_dropped` rather than drawn.
fn cap_links<'a>(
    topicmap: &'a TopicMapSpec,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<&'a TopicLinkSpec> {
    let mut by_weight: Vec<(usize, &TopicLinkSpec)> = topicmap.links.iter().enumerate().collect();
    by_weight.sort_by(|a, b| b.1.weight.total_cmp(&a.1.weight));

    let mut incident: HashMap<&str, usize> = HashMap::new();
    let mut kept = Vec::new();
    for (index, link) in by_weight {
        let from_count = *incident.get(link.from.as_str()).unwrap_or(&0);
        let to_count = *incident.get(link.to.as_str()).unwrap_or(&0);
        if from_count < MAX_LINKS_PER_TOPIC && to_count < MAX_LINKS_PER_TOPIC {
            *incident.entry(link.from.as_str()).or_insert(0) += 1;
            *incident.entry(link.to.as_str()).or_insert(0) += 1;
            kept.push(link);
        } else {
            warnings.push(ChartWarning::new(
                "link_dropped",
                format!("/topicmap/links/{index}"),
                "dropped: more than 3 routes would meet at one of its ends",
            ));
        }
    }
    kept
}

/// Phyllotaxis starting positions, largest topic first, around `center`.
fn spiral_start(count_topics: usize, mean_radius: f64, center: (f64, f64)) -> Vec<(f64, f64)> {
    let spiral_step = 2.0 * mean_radius;
    (0..count_topics)
        .map(|i| {
            let angle = count(i) * SPIRAL_ANGLE;
            let r = spiral_step * count(i).sqrt();
            (center.0 + angle.cos() * r, center.1 + angle.sin() * r)
        })
        .collect()
}

/// Runs the fixed-iteration relaxation: pairwise collision, spring attraction along kept links
/// between two topics, and a light pull back towards the center.
///
/// That pull is stronger along the plot's short side than along its long one, in proportion to
/// `aspect`. A cluster relaxed with an even pull comes out round, and normalizing a round cluster
/// into a wide canvas fits it to the height and leaves the width empty.
fn relax(
    positions: &mut [(f64, f64)],
    radii: &[f64],
    center: (f64, f64),
    links: &[(usize, usize, f64)],
    aspect: f64,
    reserved: &[(f64, f64, f64, f64)],
) {
    let pull = (CENTERING_FACTOR / aspect, CENTERING_FACTOR * aspect);
    for _ in 0..RELAXATION_ITERATIONS {
        for a in 0..positions.len() {
            for b in (a + 1)..positions.len() {
                separate(positions, radii, a, b);
            }
        }
        for &(from, to, weight) in links {
            attract(positions, radii, from, to, weight);
        }
        for position in positions.iter_mut() {
            position.0 += (center.0 - position.0) * pull.0;
            position.1 += (center.1 - position.1) * pull.1;
        }
        keep_out(positions, radii, reserved);
    }
    // The settling passes repeat the reserved areas as well: a late correction between two
    // coastlines must not put one of them back under the cartouche.
    for _ in 0..SETTLE_ITERATIONS {
        for a in 0..positions.len() {
            for b in (a + 1)..positions.len() {
                separate(positions, radii, a, b);
            }
        }
        keep_out(positions, radii, reserved);
    }
}

/// Pushes every area out of the rectangles the map furniture has claimed.
fn keep_out(positions: &mut [(f64, f64)], radii: &[f64], reserved: &[(f64, f64, f64, f64)]) {
    for (position, radius) in positions.iter_mut().zip(radii) {
        for rect in reserved {
            push_out(position, radius * MAX_WOBBLE, *rect);
        }
    }
}

/// How far a point lies from a rectangle; zero while it is inside one.
fn distance_to(point: (f64, f64), rect: (f64, f64, f64, f64)) -> f64 {
    let (left, top, width, height) = rect;
    let nearest = (
        point.0.clamp(left, left + width),
        point.1.clamp(top, top + height),
    );
    (point.0 - nearest.0).hypot(point.1 - nearest.1)
}

/// Moves one area clear of a reserved rectangle, by the shortest way out.
fn push_out(position: &mut (f64, f64), reach: f64, rect: (f64, f64, f64, f64)) {
    let (left, top, width, height) = rect;
    let (right, bottom) = (left + width, top + height);
    let nearest = (position.0.clamp(left, right), position.1.clamp(top, bottom));
    let (dx, dy) = (position.0 - nearest.0, position.1 - nearest.1);
    let distance = dx.hypot(dy);
    if distance >= reach {
        return;
    }
    if distance > 1e-9 {
        let correction = (reach - distance) / distance;
        position.0 += dx * correction;
        position.1 += dy * correction;
        return;
    }
    // The center sits inside the rectangle, so there is no direction to push along: leave by
    // whichever edge is closest.
    let exits = [
        (left - reach - position.0, 0.0),
        (right + reach - position.0, 0.0),
        (0.0, top - reach - position.1),
        (0.0, bottom + reach - position.1),
    ];
    let (dx, dy) = exits
        .into_iter()
        .min_by(|a, b| a.0.hypot(a.1).total_cmp(&b.0.hypot(b.1)))
        .expect("a rectangle has four edges");
    position.0 += dx;
    position.1 += dy;
}

/// The distance two areas keep between their centers: the coastlines at their widest, plus the
/// strip of sea between them. Measuring with the nominal radius instead would let a bulge of one
/// coastline reach into its neighbour.
fn keep_apart(radii: &[f64], a: usize, b: usize) -> f64 {
    (radii[a] + radii[b]) * MAX_WOBBLE + SEA_GAP_FACTOR * radii[a].min(radii[b])
}

/// Pushes two overlapping circles apart along their connecting axis, half the correction each.
fn separate(positions: &mut [(f64, f64)], radii: &[f64], a: usize, b: usize) {
    let (dx, dy) = (
        positions[b].0 - positions[a].0,
        positions[b].1 - positions[a].1,
    );
    let distance = dx.hypot(dy);
    let min_distance = keep_apart(radii, a, b);
    if distance >= min_distance {
        return;
    }
    // Coincident starting points cannot be normalized into a direction; nudge deterministically.
    let (ux, uy) = if distance > 1e-9 {
        (dx / distance, dy / distance)
    } else {
        (1.0, 0.0)
    };
    let correction = (min_distance - distance) / 2.0;
    positions[a].0 -= ux * correction;
    positions[a].1 -= uy * correction;
    positions[b].0 += ux * correction;
    positions[b].1 += uy * correction;
}

/// Pulls (or pushes) two linked topics toward the distance at which their areas just about
/// touch, weighted by how strong the declared neighborhood is.
fn attract(positions: &mut [(f64, f64)], radii: &[f64], a: usize, b: usize, weight: f64) {
    let (dx, dy) = (
        positions[b].0 - positions[a].0,
        positions[b].1 - positions[a].1,
    );
    let distance = dx.hypot(dy);
    if distance <= 1e-9 {
        return;
    }
    // One sea gap further out than the collision distance, so a linked pair settles just clear of
    // each other instead of fighting the separation above for the rest of the iterations.
    let target = keep_apart(radii, a, b) + SEA_GAP_FACTOR * radii[a].min(radii[b]);
    let pull = (distance - target) * weight * LINK_PULL_FACTOR / 2.0;
    let (ux, uy) = (dx / distance, dy / distance);
    positions[a].0 += ux * pull;
    positions[a].1 += uy * pull;
    positions[b].0 -= ux * pull;
    positions[b].1 -= uy * pull;
}

/// How far beyond its nominal radius an area actually reaches: the widest the coastline can be,
/// pushed out once more by every depth line drawn around it.
fn outer_extent(depth_bands: u8) -> f64 {
    MAX_WOBBLE * (1.0 + DEPTH_BAND_STEP * f64::from(depth_bands))
}

/// Scales and centers the relaxed cluster so everything drawn fits the plot: only here does the
/// map's final size emerge, since the physics above works at an arbitrary scale.
///
/// `extents` says how far past its nominal radius each area actually reaches — its widest
/// coastline point, pushed out by the depth lines around it — so the fit is measured against what
/// is drawn rather than against the bare circle. The plot keeps half the halo width free at its
/// edge, otherwise the outermost ring would be cut off by the canvas.
fn normalize(
    positions: &mut [(f64, f64)],
    radii: &mut [f64],
    plot: (f64, f64, f64, f64),
    extents: &[f64],
) {
    let (margin, top, plot_width, plot_height) = plot;
    let target_center = (margin + plot_width / 2.0, top + plot_height / 2.0);
    let inset = HALO_WIDTH / 2.0;
    let (usable_width, usable_height) = (plot_width - inset * 2.0, plot_height - inset * 2.0);
    let mut bounds = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for ((&(x, y), &radius), &extent) in positions.iter().zip(radii.iter()).zip(extents) {
        let reach = radius * extent;
        bounds.0 = bounds.0.min(x - reach);
        bounds.1 = bounds.1.max(x + reach);
        bounds.2 = bounds.2.min(y - reach);
        bounds.3 = bounds.3.max(y + reach);
    }
    let bbox_width = (bounds.1 - bounds.0).max(1.0);
    let bbox_height = (bounds.3 - bounds.2).max(1.0);
    let fit = (usable_width / bbox_width).min(usable_height / bbox_height);
    let bbox_center = (
        f64::midpoint(bounds.0, bounds.1),
        f64::midpoint(bounds.2, bounds.3),
    );
    for ((x, y), r) in positions.iter_mut().zip(radii.iter_mut()) {
        *x = target_center.0 + (*x - bbox_center.0) * fit;
        *y = target_center.1 + (*y - bbox_center.1) * fit;
        *r *= fit;
    }
}

/// Places every island into the sea gap that keeps it farthest from every topic and from every
/// island placed before it: an exhaustive, and therefore deterministic, grid search.
///
/// Distances are measured between what is actually drawn — coastline plus depth lines — so an
/// island never lands on a neighbour's outermost ring.
fn place_islands<'a>(
    topicmap: &'a TopicMapSpec,
    topic_circles: &[(f64, f64, f64)],
    plot: (f64, f64, f64, f64),
    reserved: &[(f64, f64, f64, f64)],
) -> Vec<(&'a str, f64, f64, f64)> {
    let (margin, top, plot_width, plot_height) = plot;
    let extent = outer_extent(topicmap.depth_bands);
    let island_radius = topic_circles
        .iter()
        .map(|&(_, _, r)| r)
        .fold(f64::INFINITY, f64::min)
        * ISLAND_RADIUS_FACTOR;
    // The island's own rings have to stay on the canvas as much as a topic's do.
    let reach = island_radius * extent + HALO_WIDTH / 2.0;

    let mut placed: Vec<(f64, f64, f64)> = Vec::with_capacity(topicmap.islands.len());
    let mut labeled = Vec::with_capacity(topicmap.islands.len());
    for island in &topicmap.islands {
        let mut best = (margin + reach, top + reach, f64::NEG_INFINITY);
        for row in 0..ISLAND_GRID_ROWS {
            for column in 0..ISLAND_GRID_COLUMNS {
                let x = (margin + plot_width * (count(column) + 0.5) / count(ISLAND_GRID_COLUMNS))
                    .clamp(margin + reach, margin + plot_width - reach);
                let y = (top + plot_height * (count(row) + 0.5) / count(ISLAND_GRID_ROWS))
                    .clamp(top + reach, top + plot_height - reach);
                // Water the furniture has claimed is not water an island may take.
                if reserved
                    .iter()
                    .any(|rect| distance_to((x, y), *rect) < reach)
                {
                    continue;
                }
                let score = topic_circles
                    .iter()
                    .chain(placed.iter())
                    .map(|&(cx, cy, r)| (x - cx).hypot(y - cy) - (r + island_radius) * extent)
                    .fold(f64::INFINITY, f64::min);
                if score > best.2 {
                    best = (x, y, score);
                }
            }
        }
        placed.push((best.0, best.1, island_radius));
        labeled.push((island.label.as_str(), best.0, best.1, island_radius));
    }
    labeled
}

/// Where one area ended up and the coastline it will be drawn with.
struct Placement {
    center: (f64, f64),
    radius: f64,
    seed: u64,
    profile: Vec<f64>,
}

/// Places every topic and island, keyed by label: phyllotaxis starting positions,
/// fixed-iteration relaxation with the map furniture's rectangles as immovable water, a final
/// normalize to the plot, then islands into the sea that is left.
///
/// Coastlines are generated before the normalize so their real reach, rather than the worst case
/// the clamp allows, decides how much room the map needs. Their shape does not depend on the
/// scale, so generating them at the pre-normalize radius costs nothing.
fn topicmap_positions<'a>(
    topicmap: &'a TopicMapSpec,
    ordered: &[(usize, &'a TopicSpec)],
    kept_links: &[&TopicLinkSpec],
    plot: (f64, f64, f64, f64),
    reserved: &[(f64, f64, f64, f64)],
) -> HashMap<&'a str, Placement> {
    let (margin, top, plot_width, plot_height) = plot;
    let total_value: f64 = ordered.iter().map(|(_, topic)| topic.value).sum();
    let canvas_area = plot_width * plot_height;
    let area_scale = (canvas_area * TOPIC_FILL_RATIO / (std::f64::consts::PI * total_value)).sqrt();
    let mut radii: Vec<f64> = ordered
        .iter()
        .map(|(_, topic)| area_scale * topic.value.sqrt())
        .collect();
    // The seed follows a topic's position in the specification, not its rank by value: one more
    // entry must not redraw the coastline of every area it moved past in the order.
    let seeds: Vec<u64> = ordered
        .iter()
        .map(|(index, _)| seed_for(topicmap.seed, *index))
        .collect();
    let profiles: Vec<Vec<f64>> = radii
        .iter()
        .zip(&seeds)
        .map(|(radius, seed)| coastline_profile(*radius, *seed))
        .collect();
    let band_reach = 1.0 + DEPTH_BAND_STEP * f64::from(topicmap.depth_bands);
    let extents: Vec<f64> = profiles
        .iter()
        .map(|profile| profile.iter().copied().fold(0.0, f64::max) * band_reach)
        .collect();

    let label_index: HashMap<&str, usize> = ordered
        .iter()
        .enumerate()
        .map(|(position, (_, topic))| (topic.label.as_str(), position))
        .collect();
    let topic_links: Vec<(usize, usize, f64)> = kept_links
        .iter()
        .filter_map(|link| {
            let from = *label_index.get(link.from.as_str())?;
            let to = *label_index.get(link.to.as_str())?;
            Some((from, to, link.weight))
        })
        .collect();

    let center = (margin + plot_width / 2.0, top + plot_height / 2.0);
    let mean_radius = radii.iter().sum::<f64>() / count(radii.len().max(1));
    let mut positions = spiral_start(ordered.len(), mean_radius, center);
    relax(
        &mut positions,
        &radii,
        center,
        &topic_links,
        (plot_width / plot_height).sqrt(),
        reserved,
    );
    normalize(&mut positions, &mut radii, plot, &extents);
    // The normalize moves and scales everything, so the areas have to be shown the reserved
    // water once more afterwards.
    keep_out(&mut positions, &radii, reserved);

    let mut merged: HashMap<&str, Placement> = ordered
        .iter()
        .zip(positions.iter().zip(radii.iter()))
        .zip(seeds.iter().zip(profiles))
        .map(|(((_, topic), (&center, &radius)), (&seed, profile))| {
            (
                topic.label.as_str(),
                Placement {
                    center,
                    radius,
                    seed,
                    profile,
                },
            )
        })
        .collect();
    let topic_circles: Vec<(f64, f64, f64)> = merged
        .values()
        .map(|placed| (placed.center.0, placed.center.1, placed.radius))
        .collect();
    for (index, (label, x, y, radius)) in place_islands(topicmap, &topic_circles, plot, reserved)
        .into_iter()
        .enumerate()
    {
        let seed = seed_for(topicmap.seed, index);
        merged.insert(
            label,
            Placement {
                center: (x, y),
                radius,
                seed,
                profile: coastline_profile(radius, seed),
            },
        );
    }
    merged
}

/// The coastline seed of the area declared at `index`.
fn seed_for(base: u64, index: usize) -> u64 {
    base.wrapping_add(u64::try_from(index).expect("an index fits in a u64"))
}

/// One laid-out area, ready to be drawn: which topic it stands for, and where it was placed. The
/// wobble profile is carried along because the halo, the fill and every depth line are the same
/// coastline at a different distance from the center.
struct Area<'a> {
    topic: &'a TopicSpec,
    path: String,
    /// Which `chartlet-topic-N` class this area carries. Topics come first, in the order the
    /// specification lists them, then the islands — so a host page can address every area, and
    /// the picker's indices still line up with the topics it offers.
    index: Option<usize>,
    placed: &'a Placement,
    style: NumberStyle,
}

impl Area<'_> {
    fn center(&self) -> (f64, f64) {
        self.placed.center
    }

    fn radius(&self) -> f64 {
        self.placed.radius
    }

    fn tooltip(&self) -> String {
        self.topic.tooltip.clone().unwrap_or_else(|| {
            format!(
                "{}: {}",
                self.topic.label,
                format_value(self.topic.value, self.style)
            )
        })
    }
}

/// How many points a coastline is sampled at: larger areas get more, but a small island gains
/// nothing from detail its size cannot show, and every point costs bytes in the output.
fn coastline_resolution(radius: f64) -> usize {
    let wanted = 12.0 + radius / 6.0;
    (24..=48)
        .find(|points| count(*points) >= wanted)
        .unwrap_or(48)
}

/// The wobble profile of one coastline: what the nominal radius is multiplied by at each sampled
/// angle.
///
/// The profile is normalized so the shape it describes encloses exactly the area of the nominal
/// circle — the area is what carries the value, so it must not drift with the noise — and clamped
/// afterwards, which is what makes the relaxation's collision distance an upper bound.
fn coastline_profile(radius: f64, seed: u64) -> Vec<f64> {
    let resolution = coastline_resolution(radius);
    let mut profile: Vec<f64> = (0..resolution)
        .map(|step| {
            let angle = count(step) / count(resolution) * std::f64::consts::TAU;
            let (sin, cos) = angle.sin_cos();
            let wobble = COAST_AMPLITUDES.iter().zip(COAST_FREQUENCIES).fold(
                1.0,
                |wobble, (amplitude, frequency)| {
                    wobble + amplitude * noise::value_noise(cos * frequency, sin * frequency, seed)
                },
            );
            wobble.clamp(MIN_WOBBLE, MAX_WOBBLE)
        })
        .collect();

    let enclosed = unit_polygon_area(&profile);
    if enclosed > 0.0 {
        let correction = (std::f64::consts::PI / enclosed).sqrt();
        for wobble in &mut profile {
            *wobble = (*wobble * correction).clamp(MIN_WOBBLE, MAX_WOBBLE);
        }
    }
    profile
}

/// The area enclosed by a profile drawn at radius 1: the shoelace formula, simplified because
/// every angular step is the same size.
fn unit_polygon_area(profile: &[f64]) -> f64 {
    let step = std::f64::consts::TAU / count(profile.len());
    0.5 * step.sin()
        * profile
            .iter()
            .enumerate()
            .map(|(index, radius)| radius * profile[(index + 1) % profile.len()])
            .sum::<f64>()
}

/// A coastline, or one of the depth lines around it, as a closed ring of points.
fn coastline(area: &Area, scale: f64) -> Vec<(f64, f64)> {
    let resolution = area.placed.profile.len();
    let mut points: Vec<(f64, f64)> = area
        .placed
        .profile
        .iter()
        .enumerate()
        .map(|(step, wobble)| {
            let angle = count(step) / count(resolution) * std::f64::consts::TAU;
            let (sin, cos) = angle.sin_cos();
            let distance = area.radius() * wobble * scale;
            (
                area.center().0 + cos * distance,
                area.center().1 + sin * distance,
            )
        })
        .collect();
    // A polyline is filled as if it were closed, but it is not stroked that way: without the
    // repeated first point the halo and the depth lines would show a gap.
    if let Some(&first) = points.first() {
        points.push(first);
    }
    points
}

/// Every area of the map: the topics first, in the order they were laid out, then the islands.
fn build_areas<'a>(
    topicmap: &'a TopicMapSpec,
    ordered: &[(usize, &'a TopicSpec)],
    positions: &'a HashMap<&str, Placement>,
    style: NumberStyle,
) -> (Vec<Area<'a>>, Vec<Area<'a>>) {
    let areas = ordered
        .iter()
        .map(|(index, topic)| build_area(topic, "topics", *index, *index, positions, style))
        .collect();
    let islands = topicmap
        .islands
        .iter()
        .enumerate()
        .map(|(index, island)| {
            let selectable = topicmap.topics.len() + index;
            build_area(island, "islands", index, selectable, positions, style)
        })
        .collect();
    (areas, islands)
}

/// Pairs a topic with the placement it was given, and with the path its warnings point at.
fn build_area<'a>(
    topic: &'a TopicSpec,
    group: &str,
    index: usize,
    selectable: usize,
    positions: &'a HashMap<&str, Placement>,
    style: NumberStyle,
) -> Area<'a> {
    Area {
        topic,
        path: format!("/topicmap/{group}/{index}"),
        index: Some(selectable),
        placed: positions
            .get(topic.label.as_str())
            .expect("every topic and island was placed"),
        style,
    }
}

/// A route as a gently curved polyline. A straight line between two neighbouring areas reads as
/// a connector; the curve reads as something drawn on a map.
fn route(from: (f64, f64), to: (f64, f64)) -> Vec<(f64, f64)> {
    const SEGMENTS: usize = 12;
    const BULGE: f64 = 0.1;
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let control = (
        f64::midpoint(from.0, to.0) - dy * BULGE,
        f64::midpoint(from.1, to.1) + dx * BULGE,
    );
    (0..=SEGMENTS)
        .map(|step| {
            let t = count(step) / count(SEGMENTS);
            let rest = 1.0 - t;
            (
                rest * rest * from.0 + 2.0 * rest * t * control.0 + t * t * to.0,
                rest * rest * from.1 + 2.0 * rest * t * control.1 + t * t * to.1,
            )
        })
        .collect()
}

/// The cartouche: the legend box printed on the map, with the text it will actually carry.
struct Cartouche {
    heading: String,
    meta: String,
    frame: (f64, f64, f64, f64),
}

/// The parts of the drawing that are not data. Their rectangles go to the relaxation as reserved
/// water: without that, an area eventually grows underneath the cartouche.
struct Furniture {
    compass: Option<(f64, f64)>,
    cartouche: Option<Cartouche>,
}

impl Furniture {
    /// The rectangles no area may reach into, each with a little clear water around it.
    fn reserved(&self) -> Vec<(f64, f64, f64, f64)> {
        let mut rects = Vec::new();
        if let Some((x, y)) = self.compass {
            let reach = COMPASS_RADIUS + FURNITURE_MARGIN;
            rects.push((x - reach, y - reach, reach * 2.0, reach * 2.0));
        }
        if let Some(cartouche) = &self.cartouche {
            let (x, y, width, height) = cartouche.frame;
            rects.push((
                x - FURNITURE_MARGIN,
                y - FURNITURE_MARGIN,
                width + FURNITURE_MARGIN * 2.0,
                height + FURNITURE_MARGIN * 2.0,
            ));
        }
        rects
    }
}

/// The top left corner of a box of this size, tucked into one corner of the plot.
fn corner_origin(
    corner: Corner,
    plot: (f64, f64, f64, f64),
    width: f64,
    height: f64,
) -> (f64, f64) {
    let (margin, top, plot_width, plot_height) = plot;
    let (left, right) = (
        margin + FURNITURE_MARGIN,
        margin + plot_width - width - FURNITURE_MARGIN,
    );
    let (upper, lower) = (
        top + FURNITURE_MARGIN,
        top + plot_height - height - FURNITURE_MARGIN,
    );
    match corner {
        Corner::TopLeft => (left, upper),
        Corner::TopRight => (right, upper),
        Corner::BottomLeft => (left, lower),
        Corner::BottomRight => (right, lower),
    }
}

/// Decides where the compass rose and the cartouche sit, and how wide the cartouche has to be for
/// its own text. The rose takes the top left corner, unless the cartouche was put there.
fn plan_furniture(
    topicmap: &TopicMapSpec,
    plot: (f64, f64, f64, f64),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Furniture {
    let (_, _, plot_width, _) = plot;
    let widest = plot_width * 0.4;
    let cartouche = topicmap.cartouche.as_ref().map(|spec| {
        let heading = fit_text(
            &spec.heading,
            widest,
            CARTOUCHE_HEADING_SIZE,
            metrics,
            warnings,
            "/topicmap/cartouche/heading",
        );
        let meta = fit_text(
            &spec.meta,
            widest,
            CARTOUCHE_META_SIZE,
            metrics,
            warnings,
            "/topicmap/cartouche/meta",
        );
        let width = metrics
            .width(&heading, CARTOUCHE_HEADING_SIZE)
            .max(metrics.width(&meta, CARTOUCHE_META_SIZE))
            + CARTOUCHE_PADDING * 2.0;
        let (x, y) = corner_origin(spec.corner, plot, width, CARTOUCHE_HEIGHT);
        Cartouche {
            heading,
            meta,
            frame: (x, y, width, CARTOUCHE_HEIGHT),
        }
    });

    let rose_corner = match topicmap.cartouche.as_ref().map(|spec| spec.corner) {
        Some(Corner::TopLeft) => Corner::TopRight,
        _ => Corner::TopLeft,
    };
    let compass = topicmap.compass.then(|| {
        let (x, y) = corner_origin(
            rose_corner,
            plot,
            COMPASS_RADIUS * 2.0,
            COMPASS_RADIUS * 2.0,
        );
        (x + COMPASS_RADIUS, y + COMPASS_RADIUS)
    });
    Furniture { compass, cartouche }
}

/// The sea the map sits in, and the grid over it.
pub(super) fn push_sea(elements: &mut Vec<Element>, plot: (f64, f64, f64, f64), graticule: bool) {
    let (margin, top, plot_width, plot_height) = plot;
    elements.push(Element::Rect(Rect {
        x: margin,
        y: top,
        width: plot_width,
        height: plot_height,
        class: "chartlet-sea",
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    if !graticule {
        return;
    }
    for column in 1..GRATICULE_COLUMNS {
        let x = margin + plot_width * count(column) / count(GRATICULE_COLUMNS);
        elements.push(Element::Line(Line {
            x1: x,
            y1: top,
            x2: x,
            y2: top + plot_height,
            class: "chartlet-graticule",
        }));
    }
    for row in 1..GRATICULE_ROWS {
        let y = top + plot_height * count(row) / count(GRATICULE_ROWS);
        elements.push(Element::Line(Line {
            x1: margin,
            y1: y,
            x2: margin + plot_width,
            y2: y,
            class: "chartlet-graticule",
        }));
    }
}

/// The compass rose: a ring, a four-point star, and the one direction worth naming.
fn push_compass(elements: &mut Vec<Element>, center: (f64, f64)) {
    elements.push(Element::Circle(Circle {
        cx: center.0,
        cy: center.1,
        radius: COMPASS_RADIUS,
        class: "chartlet-compass-ring",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    let star = |shape: &[(f64, f64)]| {
        let mut points: Vec<(f64, f64)> = shape
            .iter()
            .map(|(x, y)| (center.0 + x * COMPASS_RADIUS, center.1 + y * COMPASS_RADIUS))
            .collect();
        points.push(points[0]);
        points
    };
    elements.push(Element::Polyline(Polyline {
        points: star(&[
            (0.0, -0.74),
            (0.15, -0.15),
            (0.74, 0.0),
            (0.15, 0.15),
            (0.0, 0.74),
            (-0.15, 0.15),
            (-0.74, 0.0),
            (-0.15, -0.15),
        ]),
        class: "chartlet-compass-needle",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    elements.push(Element::Polyline(Polyline {
        points: star(&[(0.0, -0.74), (0.15, -0.15), (-0.15, -0.15)]),
        class: "chartlet-compass-north",
        topic: None,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    elements.push(Element::Text(Text {
        x: center.0,
        y: center.1 - COMPASS_RADIUS - 6.0,
        class: "chartlet-compass-label",
        anchor: TextAnchor::Middle,
        content: "N".to_owned(),
    }));
}

/// The cartouche: a framed plate carrying the heading and the metadata line the specification
/// gave it. Nothing in it is computed, so nothing in it can be out of date with the drawing.
fn push_cartouche(elements: &mut Vec<Element>, cartouche: &Cartouche) {
    let (x, y, width, height) = cartouche.frame;
    for (class, inset) in [
        ("chartlet-cartouche", 0.0),
        ("chartlet-cartouche-frame", 4.0),
    ] {
        elements.push(Element::Rect(Rect {
            x: x + inset,
            y: y + inset,
            width: width - inset * 2.0,
            height: height - inset * 2.0,
            class,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
    elements.push(Element::StyledText(
        Text {
            x: x + CARTOUCHE_PADDING,
            y: y + CARTOUCHE_PADDING + CARTOUCHE_HEADING_SIZE,
            class: "chartlet-cartouche-heading",
            anchor: TextAnchor::Start,
            content: cartouche.heading.clone(),
        },
        TextStyle {
            size: Some(CARTOUCHE_HEADING_SIZE),
            topic: None,
        },
    ));
    elements.push(Element::StyledText(
        Text {
            x: x + CARTOUCHE_PADDING,
            y: y + height - CARTOUCHE_PADDING,
            class: "chartlet-cartouche-meta",
            anchor: TextAnchor::Start,
            content: cartouche.meta.clone(),
        },
        TextStyle {
            size: Some(CARTOUCHE_META_SIZE),
            topic: None,
        },
    ));
}

/// Lays out a topic map. Areas are placed by a phyllotaxis spiral, relaxed by a fixed number of
/// iterations and normalized to the plot; their circles then become coastlines, carrying one
/// point per path through the topic.
///
/// Drawing order follows the design brief: depth lines, routes, halos, filled areas, path points,
/// labels, and the islands last, on top. The sea, the graticule, the compass rose and the
/// cartouche are not drawn yet; they arrive with their own step.
pub(super) fn layout_topicmap(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let topicmap = spec
        .topicmap
        .as_ref()
        .expect("validated topicmap charts carry a topicmap block");
    let margin = f64::from(PLOT_MARGIN);
    let plot_width = f64::from(spec.width) - margin * 2.0;
    let top = 78.0 + title_extra(spec, plot_width, metrics);
    let plot_height = f64::from(spec.height) - top - margin;

    let mut ordered: Vec<(usize, &TopicSpec)> = topicmap.topics.iter().enumerate().collect();
    ordered.sort_by(|a, b| b.1.value.total_cmp(&a.1.value));
    let kept_links = cap_links(topicmap, warnings);
    let plot = (margin, top, plot_width, plot_height);
    let furniture = plan_furniture(topicmap, plot, metrics, warnings);
    let positions =
        topicmap_positions(topicmap, &ordered, &kept_links, plot, &furniture.reserved());

    let (areas, islands) = build_areas(topicmap, &ordered, &positions, spec.number_style());

    // Labels are decided before anything is drawn, because a label that has to sit outside its
    // area needs its leader line laid down underneath the areas.
    let mut inside_labels = Vec::new();
    let mut island_labels = Vec::new();
    let mut outside: Vec<OutsideLabel> = Vec::new();
    for area in &areas {
        outside.extend(push_label(
            &mut inside_labels,
            area,
            plot,
            metrics,
            warnings,
        ));
    }
    for island in &islands {
        outside.extend(push_label(
            &mut island_labels,
            island,
            plot,
            metrics,
            warnings,
        ));
    }
    let outside = settle_outside_labels(outside, plot, warnings);

    let mut elements = Vec::new();
    push_title(&mut elements, spec, margin, plot_width, metrics, warnings);
    push_sea(&mut elements, plot, topicmap.graticule);
    for area in &areas {
        push_depth_lines(&mut elements, area, topicmap.depth_bands);
    }
    for link in &kept_links {
        if let (Some(from), Some(to)) = (
            positions.get(link.from.as_str()),
            positions.get(link.to.as_str()),
        ) {
            elements.push(Element::Polyline(Polyline {
                points: route(from.center, to.center),
                class: "chartlet-topic-link",
                topic: None,
                series_index: None,
                style_index: None,
                tooltip: None,
            }));
        }
    }
    push_leaders(&mut elements, &outside);
    for area in &areas {
        push_coast(&mut elements, area);
    }
    for area in &areas {
        push_path_points(&mut elements, area, warnings);
    }
    elements.append(&mut inside_labels);
    for island in &islands {
        push_depth_lines(&mut elements, island, topicmap.depth_bands);
        push_coast(&mut elements, island);
        push_path_points(&mut elements, island, warnings);
    }
    elements.append(&mut island_labels);
    push_outside_labels(&mut elements, &outside);
    if let Some(center) = furniture.compass {
        push_compass(&mut elements, center);
    }
    if let Some(cartouche) = &furniture.cartouche {
        push_cartouche(&mut elements, cartouche);
    }

    Scene {
        width: spec.width,
        height: spec.height,
        elements,
    }
}

/// The dashed depth lines around an area, the outermost one first.
fn push_depth_lines(elements: &mut Vec<Element>, area: &Area, depth_bands: u8) {
    for band in (1..=u32::from(depth_bands)).rev() {
        elements.push(Element::Polyline(Polyline {
            points: coastline(area, 1.0 + DEPTH_BAND_STEP * f64::from(band)),
            class: "chartlet-topic-band",
            topic: None,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
}

/// The coastal halo and the filled area itself, which share one outline.
fn push_coast(elements: &mut Vec<Element>, area: &Area) {
    let points = coastline(area, 1.0);
    elements.push(Element::Polyline(Polyline {
        points: points.clone(),
        class: "chartlet-topic-halo",
        topic: area.index,
        series_index: None,
        style_index: None,
        tooltip: None,
    }));
    elements.push(Element::TopicArea(
        Polyline {
            points,
            class: "chartlet-topic-area",
            topic: area.index,
            series_index: None,
            style_index: None,
            tooltip: Some(area.tooltip()),
        },
        area.center(),
    ));
}

/// One point per path through the topic, spread over the inner disc by the same golden angle the
/// spiral uses, so they never settle into rows.
///
/// An area only has room for so many before they merge into a smudge; beyond that the count is
/// reported rather than drawn, and the data table keeps carrying the exact number.
fn push_path_points(elements: &mut Vec<Element>, area: &Area, warnings: &mut Vec<ChartWarning>) {
    let usable = area.radius() * PATH_POINT_DISC;
    let drawn = (1..=area.topic.points)
        .take_while(|points| usable / f64::from(*points).sqrt() >= PATH_POINT_SPACING)
        .count();
    if drawn < usize::try_from(area.topic.points).expect("a path count fits in a usize") {
        warnings.push(ChartWarning::new(
            "dense_chart",
            format!("{}/points", area.path),
            format!(
                "{} path points do not stay apart in an area this size; {drawn} are drawn and the data table keeps the count",
                area.topic.points
            ),
        ));
    }
    let phase = noise::value_noise(0.5, 0.5, area.placed.seed) * std::f64::consts::TAU;
    for point in 0..drawn {
        let angle = count(point) * SPIRAL_ANGLE + phase;
        let distance = usable * ((count(point) + 0.5) / count(drawn)).sqrt();
        let (sin, cos) = angle.sin_cos();
        elements.push(Element::Circle(Circle {
            cx: area.center().0 + cos * distance,
            cy: area.center().1 + sin * distance,
            radius: PATH_POINT_RADIUS,
            class: "chartlet-topic-point",
            topic: area.index,
            series_index: None,
            style_index: None,
            tooltip: None,
        }));
    }
}

/// A name and count that did not fit inside their area and were moved out beside it.
struct OutsideLabel {
    name: String,
    value: String,
    path: String,
    /// The area this label names, see [`Area::index`].
    topic: Option<usize>,
    /// What the area is worth, so the least important labels are the ones dropped when a side
    /// runs out of room.
    weight: f64,
    /// `End` when the label sits to the left of its area, so the text grows away from the map.
    anchor: TextAnchor,
    x: f64,
    y: f64,
    center: (f64, f64),
}

/// The size an area's name is drawn at: as large as the area can carry, within the range the
/// design brief fixes.
fn label_size(radius: f64) -> f64 {
    (radius * TOPIC_LABEL_RATIO).clamp(TOPIC_LABEL_MIN, TOPIC_LABEL_MAX)
}

/// Draws the area's name and count inside it, or hands back a label to be placed outside.
///
/// The fit is measured against the largest circle that fits inside this coastline, so a bay in
/// the coast cannot cut a corner off the text. A label that has to move out is reported as
/// `label_outside_area` — it is not a failure, but it is a fact about the drawing that the
/// caller of chartlet can act on.
fn push_label(
    elements: &mut Vec<Element>,
    area: &Area,
    plot: (f64, f64, f64, f64),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
) -> Option<OutsideLabel> {
    let value = format_value(area.topic.value, area.style);
    let size = label_size(area.radius());
    let inscribed = area.radius()
        * area
            .placed
            .profile
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
    let name_fits = fits_inside(
        metrics.width(&area.topic.label, size),
        size * 0.8,
        inscribed,
    );
    let value_fits = fits_inside(
        metrics.width(&value, TOPIC_VALUE_SIZE),
        TOPIC_VALUE_DROP + TOPIC_VALUE_SIZE * 0.3,
        inscribed,
    );
    if !name_fits || !value_fits {
        warnings.push(ChartWarning::new(
            "label_outside_area",
            format!("{}/label", area.path),
            "the label did not fit inside the area and was placed beside it, with a leader line",
        ));
        return Some(outside_label(area, plot, value, metrics));
    }
    elements.push(Element::StyledText(
        Text {
            x: area.center().0,
            y: area.center().1,
            class: "chartlet-topic-label",
            anchor: TextAnchor::Middle,
            content: area.topic.label.clone(),
        },
        TextStyle {
            size: Some(size),
            topic: area.index,
        },
    ));
    elements.push(Element::StyledText(
        Text {
            x: area.center().0,
            y: area.center().1 + TOPIC_VALUE_DROP,
            class: "chartlet-topic-value",
            anchor: TextAnchor::Middle,
            content: value,
        },
        TextStyle {
            size: Some(TOPIC_VALUE_SIZE),
            topic: area.index,
        },
    ));
    None
}

/// Parks a label beside its area, clear of the outermost depth line.
///
/// It goes on the side facing the nearer edge of the map, which keeps it out of the crowded
/// middle — unless the text would then run off the canvas, in which case it goes on the other
/// side instead. The stacking pass afterwards is what settles the final height.
fn outside_label(
    area: &Area,
    plot: (f64, f64, f64, f64),
    value: String,
    metrics: &impl TextMetrics,
) -> OutsideLabel {
    let (margin, _, plot_width, _) = plot;
    let width = metrics
        .width(&area.topic.label, TOPIC_OUTSIDE_SIZE)
        .max(metrics.width(&value, TOPIC_OUTSIDE_SIZE));
    let widest = area.placed.profile.iter().copied().fold(0.0_f64, f64::max);
    let clearance = area.radius() * widest + HALO_WIDTH / 2.0 + LABEL_PADDING * 2.0;

    let fits_right = area.center().0 + clearance + width <= margin + plot_width;
    let fits_left = area.center().0 - clearance - width >= margin;
    let to_right = if area.center().0 >= margin + plot_width / 2.0 {
        fits_right || !fits_left
    } else {
        !fits_left && fits_right
    };
    // The clamp only bites when neither side has room for the text. A label that reaches over its
    // own coast still reads; a label half off the canvas does not.
    let x = if to_right {
        (area.center().0 + clearance).min(margin + plot_width - width)
    } else {
        (area.center().0 - clearance).max(margin + width)
    };

    OutsideLabel {
        name: area.topic.label.clone(),
        value,
        path: area.path.clone(),
        topic: area.index,
        weight: area.topic.value,
        anchor: if to_right {
            TextAnchor::Start
        } else {
            TextAnchor::End
        },
        x,
        y: area.center().1,
        center: area.center(),
    }
}

/// Whether one line of text, centered on the area and reaching `drop` pixels from its center,
/// keeps all four of its corners inside a circle of `inscribed` radius.
fn fits_inside(width: f64, drop: f64, inscribed: f64) -> bool {
    (width / 2.0 + LABEL_PADDING).hypot(drop) <= inscribed
}

/// Settles the labels that were moved out of their areas. Each side of the map is a column, and
/// two labels in one column may not sit closer than a label's own height.
fn settle_outside_labels(
    labels: Vec<OutsideLabel>,
    plot: (f64, f64, f64, f64),
    warnings: &mut Vec<ChartWarning>,
) -> Vec<OutsideLabel> {
    let (_, top, _, plot_height) = plot;
    let (left, right): (Vec<_>, Vec<_>) = labels
        .into_iter()
        .partition(|label| matches!(label.anchor, TextAnchor::End));
    let mut settled = settle_column(left, top, plot_height, warnings);
    settled.extend(settle_column(right, top, plot_height, warnings));
    settled
}

/// Spreads one column of labels so none overlaps the next, inside the plot. A column that cannot
/// hold them all keeps the largest areas' labels and reports the rest: a label drawn over another
/// label carries less than no information.
fn settle_column(
    mut labels: Vec<OutsideLabel>,
    top: f64,
    plot_height: f64,
    warnings: &mut Vec<ChartWarning>,
) -> Vec<OutsideLabel> {
    // Room for the name's ascender at the top and for the count's descender at the bottom: a
    // settled label has to stay inside the plot as a whole block.
    let first = top + TOPIC_OUTSIDE_SIZE;
    let last = top + plot_height - TOPIC_OUTSIDE_DROP - TOPIC_OUTSIDE_SIZE * 0.3;
    let capacity = (1..=labels.len())
        .take_while(|slots| count(*slots - 1) * TOPIC_OUTSIDE_SPACING <= last - first)
        .count();
    if labels.len() > capacity {
        labels.sort_by(|a, b| b.weight.total_cmp(&a.weight));
        for dropped in labels.split_off(capacity) {
            warnings.push(ChartWarning::new(
                "dense_chart",
                format!("{}/label", dropped.path),
                "there is no room beside the map for this label; the data table keeps the name",
            ));
        }
    }

    labels.sort_by(|a, b| a.y.total_cmp(&b.y));
    let mut lowest = first;
    for label in &mut labels {
        label.y = label.y.max(lowest);
        lowest = label.y + TOPIC_OUTSIDE_SPACING;
    }
    let mut highest = last;
    for label in labels.iter_mut().rev() {
        label.y = label.y.min(highest);
        highest = label.y - TOPIC_OUTSIDE_SPACING;
    }
    labels
}

/// The line from an outside label to the middle of the area it belongs to.
///
/// Drawn before the areas themselves: the fill covers everything but the stretch over open sea,
/// so the line always reaches the coast and never crosses it.
fn push_leaders(elements: &mut Vec<Element>, labels: &[OutsideLabel]) {
    for label in labels {
        elements.push(Element::Line(Line {
            x1: label.x,
            // Between the two lines of the label rather than on the name's baseline.
            y1: label.y + TOPIC_OUTSIDE_DROP / 2.0 - TOPIC_OUTSIDE_SIZE * 0.35,
            x2: label.center.0,
            y2: label.center.1,
            class: "chartlet-topic-leader",
        }));
    }
}

fn push_outside_labels(elements: &mut Vec<Element>, labels: &[OutsideLabel]) {
    for label in labels {
        elements.push(Element::StyledText(
            Text {
                x: label.x,
                y: label.y,
                class: "chartlet-topic-outside",
                anchor: label.anchor,
                content: label.name.clone(),
            },
            TextStyle {
                size: Some(TOPIC_OUTSIDE_SIZE),
                topic: label.topic,
            },
        ));
        elements.push(Element::StyledText(
            Text {
                x: label.x,
                y: label.y + TOPIC_OUTSIDE_DROP,
                class: "chartlet-topic-outside-value",
                anchor: label.anchor,
                content: label.value.clone(),
            },
            TextStyle {
                size: Some(TOPIC_OUTSIDE_SIZE),
                topic: label.topic,
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEPTH_BAND_STEP, HALO_WIDTH, MAX_WOBBLE, MIN_WOBBLE, cap_links, coastline_profile,
        distance_to, plan_furniture, topicmap_positions, unit_polygon_area,
    };
    use crate::{
        layout::{PLOT_MARGIN, count},
        metrics::BuiltinMetrics,
        spec::{CartoucheSpec, Corner, TopicLinkSpec, TopicMapSpec, TopicSpec},
    };

    #[test]
    fn topicmap_places_many_circles_without_overlap_or_canvas_overflow() {
        let topics: Vec<TopicSpec> = (0..24)
            .map(|i| TopicSpec {
                label: format!("Topic {i}"),
                value: 20.0 + (count(i) * 37.0) % 260.0,
                points: 0,
                tooltip: None,
            })
            .collect();
        let islands: Vec<TopicSpec> = (0..6)
            .map(|i| TopicSpec {
                label: format!("Island {i}"),
                value: 5.0,
                points: 0,
                tooltip: None,
            })
            .collect();
        let links: Vec<TopicLinkSpec> = (0..23)
            .map(|i| TopicLinkSpec {
                from: format!("Topic {i}"),
                to: format!("Topic {}", i + 1),
                weight: 0.5,
            })
            .collect();
        let topicmap = TopicMapSpec {
            topics,
            links,
            islands,
            seed: 0,
            cartouche: None,
            graticule: true,
            compass: true,
            depth_bands: 2,
        };

        let margin = f64::from(PLOT_MARGIN);
        let top = 78.0;
        let plot_width = 800.0 - margin * 2.0;
        let plot_height = 450.0 - top - margin;
        let mut ordered: Vec<(usize, &TopicSpec)> = topicmap.topics.iter().enumerate().collect();
        ordered.sort_by(|a, b| b.1.value.total_cmp(&a.1.value));
        let mut warnings = Vec::new();
        let kept_links = cap_links(&topicmap, &mut warnings);
        let positions = topicmap_positions(
            &topicmap,
            &ordered,
            &kept_links,
            (margin, top, plot_width, plot_height),
            &[],
        );

        // What is drawn, not the bare circle: the coastline at its widest, pushed out by the
        // depth lines around it and by half the halo stroke.
        let drawn: Vec<(f64, f64, f64, f64)> = positions
            .values()
            .map(|placed| {
                let widest = placed.profile.iter().copied().fold(0.0, f64::max);
                let bands = 1.0 + DEPTH_BAND_STEP * f64::from(topicmap.depth_bands);
                (
                    placed.center.0,
                    placed.center.1,
                    placed.radius * widest,
                    placed.radius * widest * bands + HALO_WIDTH / 2.0,
                )
            })
            .collect();
        assert_eq!(drawn.len(), 30, "every topic and island got a position");

        for i in 0..drawn.len() {
            for j in (i + 1)..drawn.len() {
                let (x1, y1, coast1, _) = drawn[i];
                let (x2, y2, coast2, _) = drawn[j];
                let distance = (x2 - x1).hypot(y2 - y1);
                assert!(
                    distance >= coast1 + coast2,
                    "coastlines overlap: {:?} vs {:?}, distance {distance}",
                    drawn[i],
                    drawn[j]
                );
            }
        }
        for &(x, y, _, reach) in &drawn {
            assert!(x - reach >= margin - 0.5, "runs off the left edge: {x},{y}");
            assert!(
                x + reach <= margin + plot_width + 0.5,
                "runs off the right edge: {x},{y}"
            );
            assert!(y - reach >= top - 0.5, "runs off the top edge: {x},{y}");
            assert!(
                y + reach <= top + plot_height + 0.5,
                "runs off the bottom edge: {x},{y}"
            );
        }
    }

    #[test]
    fn a_coastline_encloses_the_area_its_value_asks_for() {
        // The area is the whole claim the chart makes, so the noise must not quietly change it:
        // the ratio of two enclosed areas has to be the ratio of their values.
        let reference = coastline_profile(90.0, 1);
        let reference_area = 90.0 * 90.0 * unit_polygon_area(&reference);
        for (value, seed) in [(20.0, 2_u64), (75.0, 3), (140.0, 4), (300.0, 5)] {
            let radius = 90.0 * (value / 180.0_f64).sqrt();
            let profile = coastline_profile(radius, seed);
            let area = radius * radius * unit_polygon_area(&profile);
            let drift = (area / reference_area) / (value / 180.0) - 1.0;
            assert!(
                drift.abs() <= 0.03,
                "value {value} drew an area off by {:.1}%",
                drift * 100.0
            );
        }
    }

    #[test]
    fn a_coastline_stays_within_the_wobble_the_collision_distance_assumes() {
        for seed in 0..40 {
            for radius in [12.0, 55.0, 140.0] {
                for wobble in coastline_profile(radius, seed) {
                    assert!(
                        (MIN_WOBBLE..=MAX_WOBBLE).contains(&wobble),
                        "seed {seed} at radius {radius} left the clamp: {wobble}"
                    );
                }
            }
        }
    }

    #[test]
    fn areas_stay_out_of_the_water_the_map_furniture_claims() {
        // Twelve areas on a small canvas: without reserved water, one of them ends up under the
        // cartouche or the compass rose.
        let topics: Vec<TopicSpec> = (0..12)
            .map(|index| TopicSpec {
                label: format!("Area {index}"),
                value: 40.0 + count(index) * 11.0,
                points: 0,
                tooltip: None,
            })
            .collect();
        let topicmap = TopicMapSpec {
            topics,
            links: Vec::new(),
            islands: (0..4)
                .map(|index| TopicSpec {
                    label: format!("Isle {index}"),
                    value: 4.0,
                    points: 0,
                    tooltip: None,
                })
                .collect(),
            seed: 7,
            cartouche: Some(CartoucheSpec {
                heading: "Topic map".to_owned(),
                meta: "a line of metadata".to_owned(),
                corner: Corner::BottomRight,
            }),
            graticule: true,
            compass: true,
            depth_bands: 2,
        };

        let margin = f64::from(PLOT_MARGIN);
        let top = 78.0;
        let plot = (margin, top, 800.0 - margin * 2.0, 450.0 - top - margin);
        let mut ordered: Vec<(usize, &TopicSpec)> = topicmap.topics.iter().enumerate().collect();
        ordered.sort_by(|a, b| b.1.value.total_cmp(&a.1.value));
        let mut warnings = Vec::new();
        let furniture = plan_furniture(&topicmap, plot, &BuiltinMetrics, &mut warnings);
        let reserved = furniture.reserved();
        assert_eq!(
            reserved.len(),
            2,
            "a compass rose and a cartouche were planned"
        );
        let positions = topicmap_positions(&topicmap, &ordered, &[], plot, &reserved);

        for placed in positions.values() {
            let widest = placed.profile.iter().copied().fold(0.0, f64::max);
            let reach = placed.radius * widest;
            for rect in &reserved {
                assert!(
                    distance_to(placed.center, *rect) >= reach,
                    "an area reaches into reserved water at {:?}",
                    placed.center
                );
            }
        }
    }

    #[test]
    fn the_compass_rose_moves_when_the_cartouche_wants_its_corner() {
        let base = TopicMapSpec {
            topics: vec![TopicSpec {
                label: "Only".to_owned(),
                value: 10.0,
                points: 0,
                tooltip: None,
            }],
            links: Vec::new(),
            islands: Vec::new(),
            seed: 0,
            cartouche: None,
            graticule: true,
            compass: true,
            depth_bands: 2,
        };
        let plot = (24.0, 78.0, 752.0, 348.0);
        let mut warnings = Vec::new();

        let default_corner = plan_furniture(&base, plot, &BuiltinMetrics, &mut warnings)
            .compass
            .expect("a compass rose was asked for");
        let contested = TopicMapSpec {
            cartouche: Some(CartoucheSpec {
                heading: "Topic map".to_owned(),
                meta: "metadata".to_owned(),
                corner: Corner::TopLeft,
            }),
            ..base
        };
        let moved = plan_furniture(&contested, plot, &BuiltinMetrics, &mut warnings)
            .compass
            .expect("a compass rose was asked for");
        assert!(
            default_corner.0 < moved.0,
            "the rose gave up the left corner"
        );
    }
}
