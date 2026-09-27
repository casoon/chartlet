//! Deterministic value noise, used by the topic map to turn circles into coastlines.
//!
//! Written out rather than taken from a crate on purpose: chartlet promises byte-identical output
//! for the same specification, and a dependency would tie that promise to someone else's version
//! numbers. Every golden file would drift the day that crate changed a constant.

/// Lattice coordinates are offset into positive numbers, so a cell index is a plain `u64`.
const LATTICE_ORIGIN: f64 = 1024.0;

/// Smooth value noise in `-1.0..=1.0`: bilinear between four hashed lattice corners, with a
/// smoothstep on both axes so the result has bays rather than creases.
pub(crate) fn value_noise(x: f64, y: f64, seed: u64) -> f64 {
    let (grid_x, grid_y) = (x.floor(), y.floor());
    let (cell_x, cell_y) = (cell(grid_x), cell(grid_y));
    let (weight_x, weight_y) = (smoothstep(x - grid_x), smoothstep(y - grid_y));
    let near = lerp(
        corner(cell_x, cell_y, seed),
        corner(cell_x + 1, cell_y, seed),
        weight_x,
    );
    let far = lerp(
        corner(cell_x, cell_y + 1, seed),
        corner(cell_x + 1, cell_y + 1, seed),
        weight_x,
    );
    lerp(near, far, weight_y)
}

/// The lattice cell a coordinate falls into.
///
/// Callers sample the unit circle scaled by a small frequency, so the coordinate is a handful of
/// units away from zero: adding [`LATTICE_ORIGIN`] makes it positive, and neither the sign loss
/// nor the truncation of the conversion can reach a value that matters.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn cell(value: f64) -> u64 {
    (value + LATTICE_ORIGIN) as u64
}

/// Hashes one lattice corner into `-1.0..=1.0`.
fn corner(x: u64, y: u64, seed: u64) -> f64 {
    let mut state = seed
        .wrapping_add(x.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(y.wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    // SplitMix64's finalizer: stateless, and it mixes well enough that neighbouring corners show
    // no pattern of their own in the coastline.
    state = (state ^ (state >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state = (state ^ (state >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^= state >> 31;
    // The top half of the hash is the well-mixed half, and a `u32` converts to `f64` exactly.
    let top = u32::try_from(state >> 32).expect("a 64-bit value shifted by 32 fits in 32 bits");
    f64::from(top) / f64::from(u32::MAX) * 2.0 - 1.0
}

fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(from: f64, to: f64, t: f64) -> f64 {
    from + (to - from) * t
}

#[cfg(test)]
mod tests {
    use super::value_noise;

    #[test]
    fn the_same_point_always_returns_the_same_value() {
        // Compared bit for bit: the output has to be reproducible, not merely close.
        assert_eq!(
            value_noise(0.3, -1.2, 7).to_bits(),
            value_noise(0.3, -1.2, 7).to_bits()
        );
        assert_ne!(
            value_noise(0.3, -1.2, 7).to_bits(),
            value_noise(0.3, -1.2, 8).to_bits()
        );
    }

    #[test]
    fn stays_inside_the_declared_range() {
        for step in 0..500 {
            let angle = f64::from(step) / 500.0 * std::f64::consts::TAU;
            for frequency in [1.7, 4.3] {
                let value = value_noise(angle.cos() * frequency, angle.sin() * frequency, 3);
                assert!((-1.0..=1.0).contains(&value), "{value} at {angle}");
            }
        }
    }

    #[test]
    fn neighbouring_points_stay_close_together() {
        // A coastline built from this must not jump: over a step far smaller than the lattice,
        // the value may only move a fraction of its range.
        let mut previous = value_noise(0.0, 0.0, 11);
        for step in 1..200 {
            let x = f64::from(step) / 100.0;
            let value = value_noise(x, 0.0, 11);
            assert!((value - previous).abs() < 0.35, "jumped at {x}");
            previous = value;
        }
    }

    #[test]
    fn different_regions_do_not_repeat() {
        let first: Vec<f64> = (0..20).map(|i| value_noise(f64::from(i), 0.0, 1)).collect();
        let second: Vec<f64> = (0..20)
            .map(|i| value_noise(f64::from(i) + 40.0, 0.0, 1))
            .collect();
        assert_ne!(first, second);
    }
}
