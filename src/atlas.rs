//! The geometry of a knowledge landscape (`atlas`).
//!
//! A `topicmap` packs separate landmasses and lets their size carry the whole statement. An
//! `atlas` tiles one continuous land instead, so that *where* a region lies says as much as how
//! large it is.
//!
//! Everything here is hierarchical, and deliberately so. The first attempt placed all regions at
//! once and derived each realm from the centre of gravity of its own: a realm then arrived on the
//! map in three pieces with foreign regions wedged between them, and some regions ended up
//! outside the realm they belong to. A map whose realms are not places cannot be walked into. So
//! the realms are laid out first, as discs, and every region is placed *inside* its own realm.
//! The same order repeats when the ground is divided.
//!
//! The plan is `plan/09-wissenslandschaft-atlas.md`.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ops::Range;

use crate::noise::value_noise;
use crate::spec::{AtlasSpec, PlaceSpec, RegionSpec};

/// How much of the plot the realms ask for together, and how much of a realm its regions ask for.
/// The rest is the sea and the room borders need to wander; asking for all of it would press
/// everything flat against its frame.
const FILL_RATIO: f64 = 0.62;
const INNER_FILL: f64 = 0.66;

/// The golden angle: a phyllotaxis spiral spreads starting positions evenly without randomness,
/// so two runs of the same specification agree bit for bit.
const GOLDEN_ANGLE: f64 = 2.399_963_23;

/// Fixed numbers of rounds, not "until it settles": a convergence test would make the output
/// depend on floating-point luck and the golden files would drift between platforms.
const REALM_ROUNDS: usize = 260;
const REGION_ROUNDS: usize = 200;

/// Room between two neighbours, as a multiple of their radii. Just over 1: they touch, they do
/// not overlap.
const SPACING: f64 = 1.04;

/// How far a link lets two bodies sit, on top of `SPACING`, and how hard it pulls per round. A
/// link of weight 1 wants them adjacent, a link of weight 0 merely in the same part of the map.
const LINK_SLACK: f64 = 0.6;
const LINK_PULL: f64 = 0.08;

/// How hard a region with kin in another realm drifts towards that realm. It cannot leave its own
/// — containment sees to that — so what this buys is the border position, which is the whole
/// point of allowing kinship across realms at all.
const BORDER_PULL: f64 = 0.06;

/// A weak pull back to the middle, so that separation cannot walk the landscape into a corner.
const CENTRE_PULL: f64 = 0.02;

/// How far the realms are spread towards filling each axis of the plot on its own, once they have
/// settled. Zero keeps the arrangement exactly as relaxed and wastes a wide frame; one fills the
/// frame and shears every neighbourhood.
const STRETCH: f64 = 0.65;

/// Two bodies can start in the same spot when a specification repeats a value; pushing them apart
/// needs a direction, and this is the one they get.
const COINCIDENT: (f64, f64, f64) = (1.0, 0.0, 1.0);

/// How many cells across the plot is divided. Everything else follows: cells are square, so the
/// row count comes from the plot's own proportions.
const GRID_COLUMNS: usize = 180;

/// How much of the plot inside the shore is land.
///
/// The coast is found by halving: how far out, as a multiple of each realm's own radius, the land
/// has to reach for this much of the plot to be land. Two quantities had to be right at once, and
/// getting either alone was not enough. One width for every realm is simpler and wrong — it gives
/// a small realm proportionally more ground than a large one, and the areas stop meaning what the
/// values said. A fixed multiple is right about the areas and leaves the land a third of a wide
/// plot, because circles that may not overlap cannot fill a rectangle. Halving for the multiple
/// keeps the proportions and fills the frame.
const LAND_RATIO: f64 = 0.66;

/// Rounds of that search. Twenty halvings resolve it far finer than a cell, and the range it
/// searches starts at 1: at one radius every realm is exactly its own disc, so none can be cut
/// away entirely however small it is.
const CUT_ROUNDS: usize = 20;
const FURTHEST_COAST: f64 = 4.0;

/// How much of the plot the realms leave free on every side, and how much of it can never be land
/// at all. Without them the arrangement is fitted flush to the frame, the coast grows outward from
/// it, and the map arrives with its edges sliced off — a straight coastline is the one thing that
/// gives away that this is a rectangle. The inset gives the coast room to wander; the shore keeps
/// it off the frame whatever it does.
const INSET: f64 = 0.08;
const SHORE: f64 = 0.035;

/// How far a border wanders, as a share of the area's radius, and over how many cycles across the
/// plot. Enough to look drawn rather than computed, not enough to tear an area apart.
const COAST_AMPLITUDE: f64 = 0.085;
const COAST_FREQUENCY: f64 = 2.4;

/// How wide a region's hill spreads, as a share of its radius, and the levels the contour lines
/// are drawn at — as quantiles of the land, not as fractions of the highest point, so that a map
/// with one very dense region still gets contours everywhere else.
const HILL_SPREAD: f64 = 0.62;
const CONTOUR_COUNT: usize = 8;
const CONTOUR_FROM: f64 = 0.3;
const CONTOUR_TO: f64 = 0.96;

/// How much the terrain wanders on top of those hills, and over how many cycles across the plot.
/// Without it the height is a sum of smooth bells and the contours come out as concentric rings —
/// a target, not a landscape. Two octaves are enough to break the circles without inventing
/// detail the data does not have.
const RELIEF_WOBBLE: f64 = 0.1;
const RELIEF_FREQUENCY: f64 = 2.9;

/// Mixed into the noise seed per area, so every border wobbles on its own instead of the whole
/// map shifting together; and again per pass, so region borders do not trace realm borders.
const AREA_SALT: u64 = 0x9E37_79B9_7F4A_7C15;
const PASS_SALT: u64 = 0xC2B2_AE3D_27D4_EB4F;

/// A count as a measured value. One place for the cast, so there is one place to look at it.
#[allow(clippy::cast_precision_loss)]
fn count(value: usize) -> f64 {
    value as f64
}

/// A position in a sorted run, from a fraction of its length.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn place(value: f64) -> usize {
    value as usize
}

/// A measured length as a number of cells, never zero.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn steps(value: f64) -> usize {
    value.round().max(1.0) as usize
}

/// The share of the whole that each value asks for, once damped.
fn shares_of(values: &[f64], damping: f64) -> Vec<f64> {
    let damped: Vec<f64> = values.iter().map(|value| value.powf(damping)).collect();
    let total = damped.iter().sum::<f64>().max(f64::EPSILON);
    damped.iter().map(|value| value / total).collect()
}

/// Which slice of the regions belongs to each realm. Regions are flattened realm by realm, so
/// each realm owns one run of them.
pub(crate) fn realm_ranges(atlas: &AtlasSpec) -> Vec<Range<usize>> {
    let mut start = 0;
    atlas
        .realms
        .iter()
        .map(|realm| {
            let range = start..start + realm.regions.len();
            start = range.end;
            range
        })
        .collect()
}

/// A realm: the ground every one of its regions lies within.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Disc {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

/// Where a region's land grows from, and how much room it asks for.
#[derive(Debug, Clone)]
pub(crate) struct Site<'a> {
    /// Index of the realm this region belongs to.
    pub realm: usize,
    pub region: &'a RegionSpec,
    pub radius: f64,
    pub x: f64,
    pub y: f64,
}

/// A body under the relaxation. Both levels use the same physics; only what they mean differs.
#[derive(Debug, Clone, Copy)]
struct Body {
    x: f64,
    y: f64,
    radius: f64,
}

// --- placement ------------------------------------------------------------------------------

/// The direction from `a` to `b` and how far it is, never zero.
fn delta(a: &Body, b: &Body) -> (f64, f64, f64) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let distance = dx.hypot(dy);
    if distance < 1e-9 {
        return COINCIDENT;
    }
    (dx / distance, dy / distance, distance)
}

fn move_apart(bodies: &mut [Body], a: usize, b: usize, amount: f64) {
    let (ux, uy, _) = delta(&bodies[a], &bodies[b]);
    bodies[a].x -= ux * amount;
    bodies[a].y -= uy * amount;
    bodies[b].x += ux * amount;
    bodies[b].y += uy * amount;
}

/// Nobody sits on anybody: the one rule that must hold when the rounds are over, so it runs first
/// in each of them.
fn separate(bodies: &mut [Body]) {
    for a in 0..bodies.len() {
        for b in (a + 1)..bodies.len() {
            let want = (bodies[a].radius + bodies[b].radius) * SPACING;
            let (_, _, distance) = delta(&bodies[a], &bodies[b]);
            if distance < want {
                move_apart(bodies, a, b, (want - distance) / 2.0);
            }
        }
    }
}

/// Kinship pulls two bodies together, down to touching for a link of full weight.
fn attract(bodies: &mut [Body], links: &[(usize, usize, f64)]) {
    for &(a, b, weight) in links {
        let reach = SPACING + (1.0 - weight) * LINK_SLACK;
        let want = (bodies[a].radius + bodies[b].radius) * reach;
        let (_, _, distance) = delta(&bodies[a], &bodies[b]);
        if distance > want {
            move_apart(bodies, a, b, -(distance - want) * LINK_PULL);
        }
    }
}

fn recentre(bodies: &mut [Body], centre: (f64, f64)) {
    let divisor = count(bodies.len()).max(1.0);
    let mean_x = bodies.iter().map(|body| body.x).sum::<f64>() / divisor;
    let mean_y = bodies.iter().map(|body| body.y).sum::<f64>() / divisor;
    let (dx, dy) = (
        (centre.0 - mean_x) * CENTRE_PULL,
        (centre.1 - mean_y) * CENTRE_PULL,
    );
    for body in bodies.iter_mut() {
        body.x += dx;
        body.y += dy;
    }
}

/// Turn the arrangement so that its long axis runs along the plot's long axis.
///
/// The relaxation has no idea which way is wide. Left alone it settles into a diagonal band,
/// which then has to be scaled down hard to fit a landscape-shaped plot and leaves half of it
/// empty. A rigid turn keeps every distance and buys most of that room back.
fn orient(bodies: &mut [Body]) {
    let divisor = count(bodies.len()).max(1.0);
    let mean_x = bodies.iter().map(|body| body.x).sum::<f64>() / divisor;
    let mean_y = bodies.iter().map(|body| body.y).sum::<f64>() / divisor;

    // Weighted by area: where the map has its mass should decide which way it lies, not how many
    // small areas happen to trail off in some direction.
    let (mut xx, mut yy, mut xy) = (0.0, 0.0, 0.0);
    for body in bodies.iter() {
        let weight = body.radius * body.radius;
        let (dx, dy) = (body.x - mean_x, body.y - mean_y);
        xx += weight * dx * dx;
        yy += weight * dy * dy;
        xy += weight * dx * dy;
    }
    if xy.abs() < 1e-9 && (xx - yy).abs() < 1e-9 {
        return;
    }
    let (sin, cos) = (-0.5 * (2.0 * xy).atan2(xx - yy)).sin_cos();
    for body in bodies.iter_mut() {
        let (dx, dy) = (body.x - mean_x, body.y - mean_y);
        body.x = mean_x + dx * cos - dy * sin;
        body.y = mean_y + dx * sin + dy * cos;
    }
}

/// The arrangement is only ever relative; this fits it into the plot.
///
/// Mostly, not entirely, with one factor for both axes. A handful of realms relax into a compact
/// clump, and a clump fitted uniformly into a wide frame leaves half of it empty. Spreading the
/// *positions* the rest of the way, while the discs keep the smaller factor, fills the frame
/// without letting them overlap: the gaps that opens between them are closed by the coast, and
/// the areas are held by their quotas either way.
fn normalize(bodies: &mut [Body], plot: (f64, f64, f64, f64)) {
    let (left, top, width, height) = plot;
    let least = |pick: fn(&Body) -> f64| bodies.iter().map(pick).fold(f64::INFINITY, f64::min);
    let most = |pick: fn(&Body) -> f64| bodies.iter().map(pick).fold(f64::NEG_INFINITY, f64::max);
    let min_x = least(|body| body.x - body.radius);
    let min_y = least(|body| body.y - body.radius);
    let span_x = (most(|body| body.x + body.radius) - min_x).max(f64::EPSILON);
    let span_y = (most(|body| body.y + body.radius) - min_y).max(f64::EPSILON);

    let (fit_x, fit_y) = (width / span_x, height / span_y);
    let scale = fit_x.min(fit_y);
    let spread_x = scale + (fit_x - scale) * STRETCH;
    let spread_y = scale + (fit_y - scale) * STRETCH;
    let offset_x = left + (width - span_x * spread_x) / 2.0;
    let offset_y = top + (height - span_y * spread_y) / 2.0;
    for body in bodies.iter_mut() {
        body.x = offset_x + (body.x - min_x) * spread_x;
        body.y = offset_y + (body.y - min_y) * spread_y;
        body.radius *= scale;
    }
}

/// Starting positions on a phyllotaxis spiral around a centre.
fn spiral(sizes: &[f64], centre: (f64, f64), spread: f64) -> Vec<Body> {
    sizes
        .iter()
        .enumerate()
        .map(|(index, radius)| {
            let step = count(index);
            let angle = step * GOLDEN_ANGLE;
            Body {
                x: centre.0 + angle.cos() * step.sqrt() * spread,
                y: centre.1 + angle.sin() * step.sqrt() * spread,
                radius: *radius,
            }
        })
        .collect()
}

/// Radii for a set of values sharing an area: damped, so the largest does not swallow the rest.
fn radii(values: &[f64], damping: f64, area: f64) -> Vec<f64> {
    let scale = (area / std::f64::consts::PI).sqrt();
    shares_of(values, damping)
        .iter()
        .map(|share| scale * share.sqrt())
        .collect()
}

/// Which realm a region label belongs to, and its index among all regions.
fn locate(atlas: &AtlasSpec, ranges: &[Range<usize>], label: &str) -> Option<(usize, usize)> {
    atlas.realms.iter().enumerate().find_map(|(realm, entry)| {
        entry
            .regions
            .iter()
            .position(|region| region.label == label)
            .map(|index| (realm, ranges[realm].start + index))
    })
}

/// The realms, laid out first. Every region is placed inside one of these, which is what makes a
/// realm one landscape rather than a scatter that happens to share a name.
fn realm_discs(atlas: &AtlasSpec, plot: (f64, f64, f64, f64)) -> Vec<Disc> {
    let (left, top, width, height) = plot;
    let totals: Vec<f64> = atlas
        .realms
        .iter()
        .map(|realm| realm.regions.iter().map(|region| region.value).sum())
        .collect();
    let sizes = radii(&totals, atlas.area_damping, width * height * FILL_RATIO);
    let centre = (left + width / 2.0, top + height / 2.0);
    let spread = (width * height * FILL_RATIO / count(sizes.len()).max(1.0)).sqrt() * 1.7;
    let mut bodies = spiral(&sizes, centre, spread);

    // Region links, added up to the realms they connect: two realms that share many subjects
    // belong next to each other.
    let ranges = realm_ranges(atlas);
    let mut links: Vec<(usize, usize, f64)> = Vec::new();
    for link in &atlas.links {
        let (Some(from), Some(to)) = (
            locate(atlas, &ranges, &link.from),
            locate(atlas, &ranges, &link.to),
        ) else {
            continue;
        };
        if from.0 == to.0 {
            continue;
        }
        let pair = (from.0.min(to.0), from.0.max(to.0));
        if let Some(found) = links.iter_mut().find(|(a, b, _)| (*a, *b) == pair) {
            found.2 = (found.2 + link.weight).min(1.0);
        } else {
            links.push((pair.0, pair.1, link.weight));
        }
    }

    for _ in 0..REALM_ROUNDS {
        separate(&mut bodies);
        attract(&mut bodies, &links);
        recentre(&mut bodies, centre);
    }
    orient(&mut bodies);
    normalize(
        &mut bodies,
        (
            left + width * INSET,
            top + height * INSET,
            width * (1.0 - INSET * 2.0),
            height * (1.0 - INSET * 2.0),
        ),
    );

    bodies
        .iter()
        .map(|body| Disc {
            x: body.x,
            y: body.y,
            radius: body.radius,
        })
        .collect()
}

/// Pull a body back inside its realm, keeping its own radius clear of the edge.
fn contain(body: &mut Body, disc: &Disc) {
    let (dx, dy) = (body.x - disc.x, body.y - disc.y);
    let distance = dx.hypot(dy);
    let room = (disc.radius - body.radius).max(0.0);
    if distance <= room || distance < 1e-9 {
        return;
    }
    body.x = disc.x + dx / distance * room;
    body.y = disc.y + dy / distance * room;
}

/// The regions of one realm, placed inside its disc.
///
/// Kinship inside the realm pulls two regions together as usual. Kinship across realms cannot
/// take a region out of its own — containment sees to that — so it does the one thing left: it
/// pulls it towards the other realm, which leaves it on the border facing it. That is how a
/// subject mediating between two realms ends up between them without anyone placing it there.
fn place_regions<'a>(atlas: &'a AtlasSpec, discs: &[Disc]) -> Vec<Site<'a>> {
    let ranges = realm_ranges(atlas);
    let mut sites = Vec::new();

    for (realm, (entry, disc)) in atlas.realms.iter().zip(discs).enumerate() {
        let held: Vec<f64> = entry.regions.iter().map(|region| region.value).collect();
        let area = std::f64::consts::PI * disc.radius * disc.radius * INNER_FILL;
        let room = radii(&held, atlas.area_damping, area);
        let spread = (area / count(room.len()).max(1.0)).sqrt() * 1.6;
        let mut bodies = spiral(&room, (disc.x, disc.y), spread);

        // Links, split into the ones that stay inside this realm and the ones that leave it.
        let mut inner: Vec<(usize, usize, f64)> = Vec::new();
        let mut outward: Vec<(usize, usize, f64)> = Vec::new();
        for link in &atlas.links {
            let (Some(from), Some(to)) = (
                locate(atlas, &ranges, &link.from),
                locate(atlas, &ranges, &link.to),
            ) else {
                continue;
            };
            let start = ranges[realm].start;
            match (from.0 == realm, to.0 == realm) {
                (true, true) if from.1 != to.1 => {
                    inner.push((from.1 - start, to.1 - start, link.weight));
                }
                (true, false) => outward.push((from.1 - start, to.0, link.weight)),
                (false, true) => outward.push((to.1 - start, from.0, link.weight)),
                _ => {}
            }
        }

        for _ in 0..REGION_ROUNDS {
            separate(&mut bodies);
            attract(&mut bodies, &inner);
            for &(index, towards, weight) in &outward {
                let target = discs[towards];
                let (dx, dy) = (target.x - bodies[index].x, target.y - bodies[index].y);
                let distance = dx.hypot(dy).max(f64::EPSILON);
                bodies[index].x += dx / distance * disc.radius * BORDER_PULL * weight;
                bodies[index].y += dy / distance * disc.radius * BORDER_PULL * weight;
            }
            for body in &mut bodies {
                contain(body, disc);
            }
        }

        sites.extend(
            entry
                .regions
                .iter()
                .zip(&bodies)
                .map(|(region, body)| Site {
                    realm,
                    region,
                    radius: body.radius,
                    x: body.x,
                    y: body.y,
                }),
        );
    }
    sites
}

// --- the ground -----------------------------------------------------------------------------

/// The grid the landscape is decided on: which region owns which patch, and where the ground ends.
#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub columns: usize,
    pub rows: usize,
    /// Side of one cell, in user units. Cells are square.
    pub cell: f64,
    /// Centre of the cell at column 0, row 0.
    pub origin: (f64, f64),
    /// Row-major, one entry per cell: the region that owns it, or `None` for the sea.
    pub owner: Vec<Option<usize>>,
}

impl Field {
    pub fn at(&self, column: usize, row: usize) -> Option<usize> {
        self.owner[row * self.columns + column]
    }

    pub fn centre(&self, column: usize, row: usize) -> (f64, f64) {
        (
            self.origin.0 + count(column) * self.cell,
            self.origin.1 + count(row) * self.cell,
        )
    }
}

/// How high the land stands, cell by cell, and the levels the contour lines run at.
pub(crate) struct Relief {
    /// Row-major like the field, in `0.0..=1.0`, and zero at sea.
    pub height: Vec<f64>,
    pub levels: Vec<f64>,
}

/// The relief, from how densely each region is written on.
///
/// Height is entries per unit of ground, and that is the whole idea: the area was damped so that
/// the largest subject would not swallow the map, and damping throws away exactly the part of the
/// quantity that the terrain now carries. With `areaDamping` at 0.5 a region's area follows the
/// square root of its value, so its density follows the square root too — the statement is not
/// lost, it moves from how wide a region is to how pronounced it is.
fn relief(sites: &[Site], field: &Field, noise_seed: u64) -> Relief {
    let mut held = vec![0usize; sites.len()];
    for owner in field.owner.iter().flatten() {
        held[*owner] += 1;
    }
    let ground = field.cell * field.cell;
    let density: Vec<f64> = sites
        .iter()
        .zip(&held)
        .map(|(site, cells)| site.region.value / (count(*cells).max(1.0) * ground))
        .collect();
    let tallest = density.iter().fold(0.0_f64, |most, one| most.max(*one));

    let frequency = RELIEF_FREQUENCY / (field.cell * count(field.columns)).max(f64::EPSILON);
    let mut height: Vec<f64> = (0..field.owner.len())
        .map(|cell| {
            if field.owner[cell].is_none() {
                return 0.0;
            }
            let point = field.centre(cell % field.columns, cell / field.columns);
            let hills: f64 = sites
                .iter()
                .zip(&density)
                .map(|(site, tall)| {
                    let spread = (site.radius * HILL_SPREAD).max(f64::EPSILON);
                    let away = (point.0 - site.x).hypot(point.1 - site.y) / spread;
                    tall / tallest.max(f64::EPSILON) * (-0.5 * away * away).exp()
                })
                .sum();
            let wander = value_noise(point.0 * frequency, point.1 * frequency, noise_seed)
                + 0.45
                    * value_noise(
                        point.0 * frequency * 2.3,
                        point.1 * frequency * 2.3,
                        noise_seed ^ AREA_SALT,
                    );
            hills + wander * RELIEF_WOBBLE
        })
        .collect();

    // Scaled between the lowest and the highest point of the land, not from zero: the wandering
    // pushes parts of it below nothing, and clamping those to zero would leave a third of the map
    // exactly flat, with the lowest contour running along the coast.
    let low = field
        .owner
        .iter()
        .zip(&height)
        .filter_map(|(owner, one)| owner.map(|_| *one))
        .fold(f64::INFINITY, f64::min);
    let high = field
        .owner
        .iter()
        .zip(&height)
        .filter_map(|(owner, one)| owner.map(|_| *one))
        .fold(f64::NEG_INFINITY, f64::max);
    let span = (high - low).max(f64::EPSILON);
    for (owner, one) in field.owner.iter().zip(&mut height) {
        *one = if owner.is_some() {
            ((*one - low) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }

    // Levels as quantiles of the land: a map whose highest point is one dense region would
    // otherwise put every contour around it and leave the rest flat.
    let mut land: Vec<f64> = field
        .owner
        .iter()
        .zip(&height)
        .filter_map(|(owner, one)| owner.map(|_| *one))
        .collect();
    land.sort_by(f64::total_cmp);
    let levels = (0..CONTOUR_COUNT)
        .map(|step| {
            if land.is_empty() {
                return 0.0;
            }
            let share =
                CONTOUR_FROM + (CONTOUR_TO - CONTOUR_FROM) * count(step) / count(CONTOUR_COUNT - 1);
            land[place(count(land.len() - 1) * share)]
        })
        .collect();

    Relief { height, levels }
}

/// One place, put down on the ground of its region.
#[derive(Debug, Clone)]
pub(crate) struct Spot<'a> {
    pub place: &'a PlaceSpec,
    pub region: usize,
    pub x: f64,
    pub y: f64,
}

/// Every declared place, inside its own region and off its borders.
///
/// Farthest-point first: each place goes where it is furthest from the ones already down. That
/// spreads them evenly without any randomness and, unlike a spiral, it follows whatever shape the
/// region actually has. The heaviest places go first, so the ones that matter get the open ground
/// and the rest fill in around them.
fn places<'a>(sites: &[Site<'a>], field: &Field) -> Vec<Spot<'a>> {
    let mut spots = Vec::new();
    for (region, site) in sites.iter().enumerate() {
        if site.region.places.is_empty() {
            continue;
        }
        // Off the borders: a point on the line reads as belonging to the neighbour.
        let inner: Vec<usize> = (0..field.owner.len())
            .filter(|cell| {
                if field.owner[*cell] != Some(region) {
                    return false;
                }
                let (column, row) = (cell % field.columns, cell / field.columns);
                [
                    (column > 0).then(|| cell - 1),
                    (column + 1 < field.columns).then(|| cell + 1),
                    (row > 0).then(|| cell - field.columns),
                    (row + 1 < field.rows).then(|| cell + field.columns),
                ]
                .into_iter()
                .flatten()
                .all(|near| field.owner[near] == Some(region))
            })
            .collect();
        let ground: Vec<usize> = if inner.is_empty() {
            (0..field.owner.len())
                .filter(|cell| field.owner[*cell] == Some(region))
                .collect()
        } else {
            inner
        };
        if ground.is_empty() {
            continue;
        }

        let mut order: Vec<usize> = (0..site.region.places.len()).collect();
        order.sort_by(|a, b| {
            site.region.places[*b]
                .weight
                .total_cmp(&site.region.places[*a].weight)
                .then(a.cmp(b))
        });

        let point = |cell: usize| field.centre(cell % field.columns, cell / field.columns);
        // How far each candidate cell is from the nearest place already down. Infinite while
        // there is none, so the first pick is decided by the line below instead.
        let mut apart: Vec<f64> = vec![f64::INFINITY; ground.len()];

        for (taken, index) in order.into_iter().take(ground.len()).enumerate() {
            // The first place goes nearest the region's own centre; every one after it goes
            // wherever it would be furthest from the places already there.
            let best = if taken == 0 {
                (0..ground.len())
                    .min_by(|a, b| {
                        let away = |slot: &usize| {
                            let (x, y) = point(ground[*slot]);
                            (x - site.x).hypot(y - site.y)
                        };
                        away(a).total_cmp(&away(b)).then(a.cmp(b))
                    })
                    .unwrap_or(0)
            } else {
                (0..ground.len())
                    .max_by(|a, b| apart[*a].total_cmp(&apart[*b]).then(b.cmp(a)))
                    .unwrap_or(0)
            };
            let (x, y) = point(ground[best]);
            spots.push(Spot {
                place: &site.region.places[index],
                region,
                x,
                y,
            });
            for (slot, cell) in ground.iter().enumerate() {
                let (other_x, other_y) = point(*cell);
                apart[slot] = apart[slot].min((other_x - x).hypot(other_y - y));
            }
        }
    }
    spots
}

/// Realms, the regions inside them, and the ground they hold.
pub(crate) struct Landscape<'a> {
    pub sites: Vec<Site<'a>>,
    pub field: Field,
    pub relief: Relief,
    pub spots: Vec<Spot<'a>>,
}

/// A point the ground gets divided around: where it pulls from, how far it reaches, and what
/// share of the ground it should end up with.
struct Seed {
    x: f64,
    y: f64,
    radius: f64,
    target: f64,
}

fn seeds_from(points: impl Iterator<Item = (f64, f64, f64)>, targets: &[f64]) -> Vec<Seed> {
    points
        .zip(targets)
        .map(|((x, y, radius), target)| Seed {
            x,
            y,
            radius,
            target: *target,
        })
        .collect()
}

/// The per-seed noise offset, so each border wanders on its own.
#[allow(clippy::cast_possible_truncation)]
fn salt(noise_seed: u64, index: usize) -> u64 {
    noise_seed ^ (index as u64).wrapping_add(1).wrapping_mul(AREA_SALT)
}

/// What it costs a seed to reach a point: the plain distance, pushed about by noise so that the
/// border it eventually meets wanders instead of running true.
fn cost(seed: &Seed, point: (f64, f64), frequency: f64, salt: u64) -> f64 {
    let wobble =
        COAST_AMPLITUDE * seed.radius * value_noise(point.0 * frequency, point.1 * frequency, salt);
    ((point.0 - seed.x).hypot(point.1 - seed.y) + wobble).max(0.0)
}

/// Where the land stops, in realm radii, found by halving.
fn coast_cut(reach: &[f64]) -> f64 {
    let wanted = LAND_RATIO * count(reach.len());
    let (mut low, mut high) = (1.0, FURTHEST_COAST);
    for _ in 0..CUT_ROUNDS {
        let middle = f64::midpoint(low, high);
        if count(reach.iter().filter(|out| **out <= middle).count()) < wanted {
            low = middle;
        } else {
            high = middle;
        }
    }
    high
}

/// How many cells each seed gets, adding up to exactly what there is to give.
fn quotas(seeds: &[Seed], total: usize) -> Vec<usize> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut shares: Vec<usize> = seeds
        .iter()
        .map(|seed| (seed.target * count(total)).floor().max(1.0) as usize)
        .collect();
    // Rounding leaves a handful of cells over or short. They go to, or come from, the largest
    // share — the one place where a cell either way cannot be noticed.
    let largest = shares
        .iter()
        .enumerate()
        .max_by_key(|(_, share)| **share)
        .map_or(0, |(index, _)| index);
    let given: usize = shares.iter().sum();
    shares[largest] = shares[largest] + total.saturating_sub(given) - given.saturating_sub(total);
    shares
}

/// The cell a seed starts from: its own, or the closest one it is allowed to have.
fn start_cell(seed: &Seed, field: &Field, allowed: &[bool]) -> Option<usize> {
    let mut best: Option<(f64, usize)> = None;
    for (cell, open) in allowed.iter().enumerate() {
        if !open {
            continue;
        }
        let point = field.centre(cell % field.columns, cell / field.columns);
        let distance = (point.0 - seed.x).hypot(point.1 - seed.y);
        if best.is_none_or(|(least, _)| distance < least) {
            best = Some((distance, cell));
        }
    }
    best.map(|(_, cell)| cell)
}

fn around(field: &Field, cell: usize) -> impl Iterator<Item = usize> {
    let (column, row) = (cell % field.columns, cell / field.columns);
    [
        (column > 0).then(|| cell - 1),
        (column + 1 < field.columns).then(|| cell + 1),
        (row > 0).then(|| cell - field.columns),
        (row + 1 < field.rows).then(|| cell + field.columns),
    ]
    .into_iter()
    .flatten()
}

/// Divides `cells` among `seeds` by growing each area outward from its own core.
///
/// The obvious way is to hand every cell to whichever seed is nearest, and it was the way this
/// started: it gives areas of roughly the right size, and now and then it leaves one of them in
/// two pieces on opposite sides of a neighbour. Two patches under one name are not one place.
/// Growing instead — cheapest cell first, but only ever onto ground the area already holds —
/// makes a single piece a property of the construction rather than something to check for
/// afterwards. It also makes the sizes exact: an area stops when its quota is full.
fn grow(
    seeds: &[Seed],
    field: &Field,
    cells: &[usize],
    frequency: f64,
    noise_seed: u64,
) -> Vec<Option<usize>> {
    let mut allowed = vec![false; field.owner.len()];
    for cell in cells {
        allowed[*cell] = true;
    }
    let mut owner: Vec<Option<usize>> = vec![None; field.owner.len()];
    let mut left = quotas(seeds, cells.len());

    // Ordered by cost, then by area and cell, so that two cells that cost the same are always
    // taken in the same order. Costs travel as bits: they are never negative, and the bit pattern
    // of a non-negative f64 sorts the same way the number does.
    let mut frontier: BinaryHeap<Reverse<(u64, usize, usize)>> = BinaryHeap::new();
    for (index, seed) in seeds.iter().enumerate() {
        if let Some(cell) = start_cell(seed, field, &allowed) {
            frontier.push(Reverse((0, index, cell)));
        }
    }

    while let Some(Reverse((_, index, cell))) = frontier.pop() {
        if owner[cell].is_some() || left[index] == 0 {
            continue;
        }
        owner[cell] = Some(index);
        left[index] -= 1;
        for next in around(field, cell) {
            if allowed[next] && owner[next].is_none() {
                let point = field.centre(next % field.columns, next / field.columns);
                let reach = cost(&seeds[index], point, frequency, salt(noise_seed, index));
                frontier.push(Reverse((reach.to_bits(), index, next)));
            }
        }
    }

    // Cells nobody had quota left for. They go to whoever is already next to them, which keeps
    // every area in one piece and leaves no hole in the land.
    let mut stranded: Vec<usize> = cells
        .iter()
        .copied()
        .filter(|cell| owner[*cell].is_none())
        .collect();
    while !stranded.is_empty() {
        let mut again = Vec::new();
        let mut settled = false;
        for cell in stranded {
            match around(field, cell).find_map(|next| owner[next]) {
                Some(heir) => {
                    owner[cell] = Some(heir);
                    settled = true;
                }
                None => again.push(cell),
            }
        }
        if !settled {
            break;
        }
        stranded = again;
    }
    owner
}

/// The ground, decided in two passes: the realms divide the land, then each realm's own ground is
/// divided among its regions. A realm is one piece by construction, which is also what lets a
/// click on it zoom to one shape rather than to a scatter.
fn field(atlas: &AtlasSpec, realms: &[Disc], sites: &[Site], plot: (f64, f64, f64, f64)) -> Field {
    let (left, top, width, height) = plot;
    let cell = width / count(GRID_COLUMNS);
    let rows = steps(height / cell);
    let mut field = Field {
        columns: GRID_COLUMNS,
        rows,
        cell,
        origin: (left + cell / 2.0, top + cell / 2.0),
        owner: vec![None; GRID_COLUMNS * rows],
    };

    let frequency = COAST_FREQUENCY / width;
    let ranges = realm_ranges(atlas);
    let totals: Vec<f64> = ranges
        .iter()
        .map(|range| sites[range.clone()].iter().map(|s| s.region.value).sum())
        .collect();
    let realm_seeds = seeds_from(
        realms.iter().map(|disc| (disc.x, disc.y, disc.radius)),
        &shares_of(&totals, atlas.area_damping),
    );

    // The shore is off limits whatever else happens: it is what keeps a coastline from arriving
    // as a straight line along the frame.
    let shore = width.min(height) * SHORE;
    let open: Vec<usize> = (0..field.owner.len())
        .filter(|cell| {
            let (x, y) = field.centre(cell % field.columns, cell / field.columns);
            x - left > shore
                && left + width - x > shore
                && y - top > shore
                && top + height - y > shore
        })
        .collect();

    // How far out each realm lies from every open cell, in its own radii. The coast is one cut
    // through that, so every realm's saum is in proportion to it.
    let reach: Vec<f64> = open
        .iter()
        .map(|cell| {
            let point = field.centre(cell % field.columns, cell / field.columns);
            realm_seeds
                .iter()
                .enumerate()
                .map(|(index, seed)| {
                    cost(seed, point, frequency, salt(atlas.seed, index))
                        / seed.radius.max(f64::EPSILON)
                })
                .fold(f64::INFINITY, f64::min)
        })
        .collect();
    let cut = coast_cut(&reach);
    let land: Vec<usize> = open
        .iter()
        .zip(&reach)
        .filter_map(|(cell, out)| (*out <= cut).then_some(*cell))
        .collect();
    let realm_owner = grow(&realm_seeds, &field, &land, frequency, atlas.seed);

    for (realm, range) in ranges.iter().enumerate() {
        let mine: Vec<usize> = land
            .iter()
            .copied()
            .filter(|cell| realm_owner[*cell] == Some(realm))
            .collect();
        if mine.is_empty() {
            continue;
        }
        let mut values = Vec::new();
        let mut points = Vec::new();
        for site in &sites[range.clone()] {
            values.push(site.region.value);
            points.push((site.x, site.y, site.radius));
        }
        let seeds = seeds_from(points.into_iter(), &shares_of(&values, atlas.area_damping));
        let owner = grow(&seeds, &field, &mine, frequency, atlas.seed ^ PASS_SALT);
        for cell in mine {
            field.owner[cell] = owner[cell].map(|region| range.start + region);
        }
    }
    field
}

/// The whole landscape: the realms, the regions inside them, and the ground they hold.
pub(crate) fn landscape(atlas: &AtlasSpec, plot: (f64, f64, f64, f64)) -> Landscape<'_> {
    let realms = realm_discs(atlas, plot);
    let sites = place_regions(atlas, &realms);
    let field = field(atlas, &realms, &sites, plot);
    let relief = relief(&sites, &field, atlas.seed);
    let spots = places(&sites, &field);
    Landscape {
        sites,
        field,
        relief,
        spots,
    }
}

#[cfg(test)]
mod tests {
    use super::{Field, Landscape, count, landscape, place, realm_discs, realm_ranges, shares_of};
    use crate::spec::ChartSpec;

    const PLOT: (f64, f64, f64, f64) = (24.0, 78.0, 752.0, 348.0);

    fn spec(json: &str) -> ChartSpec {
        serde_json::from_str(json).expect("the fixture parses")
    }

    /// Three realms of three regions. Crowded enough that placement has decisions to make — with
    /// four regions everything touches everything and nothing can be shown.
    fn crowded(links: &str) -> ChartSpec {
        let realms = ["North", "Middle", "South"]
            .iter()
            .enumerate()
            .map(|(realm, name)| {
                let regions = (0..3)
                    .map(|index| {
                        format!(
                            r#"{{ "label": "{name}{index}", "value": {} }}"#,
                            20 + realm * 30 + index * 7
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!(r#"{{ "label": "{name}", "regions": [{regions}] }}"#)
            })
            .collect::<Vec<_>>()
            .join(",");
        spec(&format!(
            r#"{{"schemaVersion": 1, "type": "atlas", "title": "Crowded",
                "atlas": {{"realms": [{realms}], "links": [{links}]}}}}"#
        ))
    }

    fn built(chart: &ChartSpec) -> Landscape<'_> {
        landscape(chart.atlas.as_ref().expect("an atlas fixture"), PLOT)
    }

    /// The largest connected run of cells that answer `same`, as a share of all of them. Four-way
    /// neighbours, because a landscape held together only at a corner is not held together.
    fn largest_piece(field: &Field, same: &dyn Fn(usize) -> bool) -> f64 {
        let mut seen = vec![false; field.columns * field.rows];
        let mut mine = 0usize;
        let mut biggest = 0usize;
        for row in 0..field.rows {
            for column in 0..field.columns {
                match field.at(column, row) {
                    Some(owner) if same(owner) => mine += 1,
                    _ => continue,
                }
                if seen[row * field.columns + column] {
                    continue;
                }
                let mut stack = vec![(column, row)];
                let mut size = 0usize;
                while let Some((x, y)) = stack.pop() {
                    let cell = y * field.columns + x;
                    if seen[cell] || !matches!(field.at(x, y), Some(owner) if same(owner)) {
                        continue;
                    }
                    seen[cell] = true;
                    size += 1;
                    if x > 0 {
                        stack.push((x - 1, y));
                    }
                    if x + 1 < field.columns {
                        stack.push((x + 1, y));
                    }
                    if y > 0 {
                        stack.push((x, y - 1));
                    }
                    if y + 1 < field.rows {
                        stack.push((x, y + 1));
                    }
                }
                biggest = biggest.max(size);
            }
        }
        if mine == 0 {
            return 0.0;
        }
        count(biggest) / count(mine)
    }

    #[test]
    fn the_landscape_is_deterministic() {
        let chart = crowded(r#"{ "from": "North0", "to": "South2", "weight": 0.5 }"#);
        let (first, second) = (built(&chart), built(&chart));
        assert_eq!(first.field.owner, second.field.owner);
        for (a, b) in first.sites.iter().zip(&second.sites) {
            assert_eq!(a.x.to_bits(), b.x.to_bits());
            assert_eq!(a.y.to_bits(), b.y.to_bits());
        }
        for (a, b) in first.relief.height.iter().zip(&second.relief.height) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }

    /// The promise the type is named for, and the one the first attempt broke.
    #[test]
    fn a_realm_is_one_piece() {
        let chart = crowded(r#"{ "from": "North0", "to": "South2", "weight": 0.5 }"#);
        let landscape = built(&chart);
        let realms: Vec<usize> = landscape.sites.iter().map(|site| site.realm).collect();
        for realm in 0..3 {
            let whole = largest_piece(&landscape.field, &|owner| realms[owner] == realm);
            assert!(
                whole > 0.95,
                "realm {realm} is split: largest piece {whole:.2}"
            );
        }
    }

    /// A region torn into islands by the noise would read as two places with one name.
    #[test]
    fn a_region_is_one_piece() {
        let chart = crowded("");
        let landscape = built(&chart);
        for region in 0..landscape.sites.len() {
            let whole = largest_piece(&landscape.field, &|owner| owner == region);
            assert!(
                whole > 0.9,
                "region {region} is split: largest piece {whole:.2}"
            );
        }
    }

    #[test]
    fn every_region_holds_ground() {
        let chart = crowded("");
        let landscape = built(&chart);
        for (region, site) in landscape.sites.iter().enumerate() {
            let held = landscape
                .field
                .owner
                .iter()
                .filter(|owner| **owner == Some(region))
                .count();
            assert!(held > 0, "{} holds no ground at all", site.region.label);
        }
    }

    #[test]
    fn every_region_lies_inside_its_realm() {
        let chart = crowded(r#"{ "from": "North0", "to": "South2", "weight": 1 }"#);
        let landscape = built(&chart);
        let discs = realm_discs(chart.atlas.as_ref().expect("an atlas fixture"), PLOT);
        for site in &landscape.sites {
            let disc = discs[site.realm];
            let over = (site.x - disc.x).hypot(site.y - disc.y) + site.radius - disc.radius;
            assert!(
                over <= 0.5,
                "{} sticks out of its realm by {over:.1}",
                site.region.label
            );
        }
    }

    /// Kinship across realms cannot move a region out of its own, so what it must do instead is
    /// leave it on the border facing its kin.
    #[test]
    fn kinship_across_realms_moves_a_region_to_the_border() {
        let towards = |chart: &ChartSpec| {
            let landscape = built(chart);
            let site = landscape
                .sites
                .iter()
                .find(|site| site.region.label == "North0")
                .expect("placed");
            let target = realm_discs(chart.atlas.as_ref().expect("an atlas fixture"), PLOT)[2];
            (site.x - target.x).hypot(site.y - target.y)
        };
        let loose = crowded("");
        let tight = crowded(r#"{ "from": "North0", "to": "South2", "weight": 1 }"#);
        let apart = towards(&loose);
        let pulled = towards(&tight);
        assert!(
            pulled < apart,
            "the link should move it towards the other realm: {pulled:.1} with, {apart:.1} without"
        );
    }

    /// Capacity control, and it is a two-level promise: a realm holds the share its regions add up
    /// to, and inside it every region holds the share its own value asks for. Damping applies at
    /// both levels, so a region's share of the whole map is the product of the two.
    #[test]
    fn each_level_holds_the_share_its_value_asks_for() {
        let chart = crowded("");
        let atlas = chart.atlas.as_ref().expect("an atlas fixture");
        let landscape = built(&chart);
        let land = count(landscape.field.owner.iter().flatten().count());
        let held = |keep: &dyn Fn(usize) -> bool| {
            count(
                landscape
                    .field
                    .owner
                    .iter()
                    .flatten()
                    .filter(|region| keep(**region))
                    .count(),
            )
        };

        let ranges = realm_ranges(atlas);
        let totals: Vec<f64> = ranges
            .iter()
            .map(|range| {
                landscape.sites[range.clone()]
                    .iter()
                    .map(|site| site.region.value)
                    .sum()
            })
            .collect();
        let realm_targets = shares_of(&totals, atlas.area_damping);

        for (realm, range) in ranges.iter().enumerate() {
            let mine = held(&|region| range.contains(&region));
            let want = realm_targets[realm];
            assert!(
                (mine / land - want).abs() < want * 0.3,
                "realm {realm} holds {:.3} of the land but asks for {want:.3}",
                mine / land
            );

            let values: Vec<f64> = landscape.sites[range.clone()]
                .iter()
                .map(|site| site.region.value)
                .collect();
            for (slot, target) in shares_of(&values, atlas.area_damping).iter().enumerate() {
                let region = range.start + slot;
                let share = held(&|owner| owner == region) / mine;
                assert!(
                    (share - target).abs() < target * 0.35,
                    "{} holds {share:.3} of its realm but asks for {target:.3}",
                    landscape.sites[region].region.label
                );
            }
        }
    }

    #[test]
    fn the_sea_surrounds_the_land() {
        let chart = crowded("");
        let landscape = built(&chart);
        let field = &landscape.field;
        let share = count(field.owner.iter().flatten().count()) / count(field.owner.len());
        assert!(
            (0.55..0.63).contains(&share),
            "the land should fill the plot without reaching its frame, got {share:.3}"
        );
        // The whole rim, not just the corners: a coast that runs along the frame is a cut.
        for column in 0..field.columns {
            for row in [0, field.rows - 1] {
                assert!(
                    field.at(column, row).is_none(),
                    "the land reaches the frame at column {column}, row {row}"
                );
            }
        }
        for row in 0..field.rows {
            for column in [0, field.columns - 1] {
                assert!(
                    field.at(column, row).is_none(),
                    "the land reaches the frame at column {column}, row {row}"
                );
            }
        }
    }

    /// The first attempt put every place of a region on the same cell: the "furthest from what is
    /// already there" numbers started out negative, so the minimum after each pick never took.
    #[test]
    fn places_spread_out_over_their_region() {
        let chart = spec(
            r#"{"schemaVersion": 1, "type": "atlas", "title": "Spread",
                "atlas": {"realms": [{"label": "Only", "regions": [
                    {"label": "Field", "value": 40, "places": [
                        {"label": "A"}, {"label": "B"}, {"label": "C"},
                        {"label": "D"}, {"label": "E"}, {"label": "F"}
                    ]}
                ]}]}}"#,
        );
        let landscape = built(&chart);
        assert_eq!(landscape.spots.len(), 6);
        for (index, spot) in landscape.spots.iter().enumerate() {
            for other in &landscape.spots[index + 1..] {
                let apart = (spot.x - other.x).hypot(spot.y - other.y);
                assert!(
                    apart > landscape.field.cell,
                    "{} and {} sit on top of each other",
                    spot.place.label,
                    other.place.label
                );
            }
        }
    }

    /// Every place lies on the ground of the region that declared it.
    #[test]
    fn every_place_lies_in_its_own_region() {
        let chart = crowded("");
        let landscape = built(&chart);
        for spot in &landscape.spots {
            let column = place((spot.x - landscape.field.origin.0) / landscape.field.cell + 0.5);
            let row = place((spot.y - landscape.field.origin.1) / landscape.field.cell + 0.5);
            assert_eq!(
                landscape.field.at(column, row),
                Some(spot.region),
                "{} sits outside its region",
                spot.place.label
            );
        }
    }

    #[test]
    fn the_sea_has_no_height() {
        let chart = crowded("");
        let landscape = built(&chart);
        for (owner, height) in landscape.field.owner.iter().zip(&landscape.relief.height) {
            if owner.is_none() {
                assert!(height.abs() < f64::EPSILON, "the sea stands {height} high");
            } else {
                assert!(
                    (0.0..=1.0).contains(height),
                    "height {height} is off the scale"
                );
            }
        }
    }

    /// The terrain carries what the damped area gave up: entries per unit of ground.
    #[test]
    fn a_denser_region_stands_higher() {
        let chart = crowded("");
        let landscape = built(&chart);
        let mean = |label: &str| {
            let region = landscape
                .sites
                .iter()
                .position(|site| site.region.label == label)
                .expect("placed");
            let mine: Vec<f64> = landscape
                .field
                .owner
                .iter()
                .zip(&landscape.relief.height)
                .filter_map(|(owner, height)| (*owner == Some(region)).then_some(*height))
                .collect();
            mine.iter().sum::<f64>() / count(mine.len()).max(1.0)
        };
        // 94 entries in the largest realm against 20 in the smallest.
        assert!(
            mean("South2") > mean("North0") * 1.5,
            "the denser region should stand clearly higher: {:.3} against {:.3}",
            mean("South2"),
            mean("North0")
        );
    }

    #[test]
    fn the_contour_levels_rise() {
        let chart = crowded("");
        let landscape = built(&chart);
        let levels = &landscape.relief.levels;
        assert_eq!(levels.len(), 8);
        for pair in levels.windows(2) {
            assert!(pair[1] > pair[0], "levels should rise: {levels:?}");
        }
        assert!(
            levels[0] > 0.0 && levels[4] < 1.0,
            "levels {levels:?} leave no room"
        );
    }

    /// Damping is what keeps the biggest subject from deciding the whole map.
    #[test]
    fn damping_narrows_the_gap_between_largest_and_smallest() {
        let shares = shares_of(&[292.0, 14.0], 0.5);
        let ratio = shares[0] / shares[1];
        assert!(
            (4.0..5.0).contains(&ratio),
            "292 against 14 is a factor of 21; damped it should be near its square root, got {ratio}"
        );
    }
}
