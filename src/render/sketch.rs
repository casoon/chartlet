//! The sketch stroke style: lines and shapes displaced by a seeded, bounded amount. The seed is
//! the SHA-256 of the specification, the elements draw from it in scene order, so the same
//! specification gives the same bytes. Displacement stays within about a pixel and a half;
//! text and circles are not touched.

use std::fmt::Write;

use super::number;
use crate::spec::ChartSpec;

/// The state of the displacement: a `SplitMix64` sequence.
pub(super) struct Sketch(u64);

impl Sketch {
    pub(super) fn new(spec: &ChartSpec) -> Self {
        let json = serde_json::to_string(spec).unwrap_or_default();
        let hash = crate::sha256(json.as_bytes());
        let mut seed = [0u8; 8];
        seed.copy_from_slice(&hash[..8]);
        Self(u64::from_le_bytes(seed))
    }

    /// A number in -1..1.
    #[allow(clippy::cast_precision_loss)]
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 11) as f64 / (1u64 << 53) as f64).mul_add(2.0, -1.0)
    }

    /// The path of a line that bows a little between two slightly displaced ends.
    pub(super) fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> Option<String> {
        let (dx, dy) = (x2 - x1, y2 - y1);
        let length = dx.hypot(dy);
        if length < 3.0 {
            return None;
        }
        let amplitude = length.mul_add(0.012, 0.3).min(1.2);
        let (nx, ny) = (-dy / length, dx / length);
        let ax = self.next().mul_add(amplitude * 0.5, x1);
        let ay = self.next().mul_add(amplitude * 0.5, y1);
        let bx = self.next().mul_add(amplitude * 0.5, x2);
        let by = self.next().mul_add(amplitude * 0.5, y2);
        let bow = self.next() * amplitude * 1.5;
        let cx = f64::midpoint(ax, bx) + nx * bow;
        let cy = f64::midpoint(ay, by) + ny * bow;
        Some(format!(
            "M{} {}Q{} {} {} {}",
            number(ax),
            number(ay),
            number(cx),
            number(cy),
            number(bx),
            number(by)
        ))
    }

    /// The closed path of a rectangle with displaced corners and bowed sides.
    pub(super) fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> Option<String> {
        if width < 4.0 || height < 4.0 {
            return None;
        }
        let amplitude = width.min(height).mul_add(0.01, 0.4).min(1.2);
        let corners = [
            (x, y),
            (x + width, y),
            (x + width, y + height),
            (x, y + height),
        ]
        .map(|(cx, cy)| {
            (
                self.next().mul_add(amplitude * 0.6, cx),
                self.next().mul_add(amplitude * 0.6, cy),
            )
        });
        let mut path = format!("M{} {}", number(corners[0].0), number(corners[0].1));
        for index in 0..4 {
            let (ax, ay) = corners[index];
            let (bx, by) = corners[(index + 1) % 4];
            let (dx, dy) = (bx - ax, by - ay);
            let length = dx.hypot(dy).max(1.0);
            let bow = self.next() * amplitude;
            write!(
                path,
                "Q{} {} {} {}",
                number(f64::midpoint(ax, bx) - dy / length * bow),
                number(f64::midpoint(ay, by) + dx / length * bow),
                number(bx),
                number(by)
            )
            .expect("write");
        }
        path.push('Z');
        Some(path)
    }

    /// The points of a polyline, each displaced a little.
    pub(super) fn points(&mut self, points: &[(f64, f64)]) -> Vec<(f64, f64)> {
        points
            .iter()
            .map(|&(x, y)| (self.next().mul_add(0.8, x), self.next().mul_add(0.8, y)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{RenderFormat, RenderOptions, Variant, render_json};

    const FLOW: &str = include_str!("../../examples/sketch-pipeline.json");

    fn svg(input: &str, variant: Variant) -> String {
        let options = RenderOptions {
            variant,
            id_prefix: Some("t".to_owned()),
            ..RenderOptions::default()
        };
        render_json(input, RenderFormat::Svg, &options)
            .expect("renders")
            .content
    }

    fn clean(input: &str) -> String {
        input.replace("\"stroke\": \"sketch\",", "")
    }

    #[test]
    fn sketch_is_stable_and_leaves_text_alone() {
        let first = svg(FLOW, Variant::Desktop);
        assert_eq!(first, svg(FLOW, Variant::Desktop));
        let plain = svg(&clean(FLOW), Variant::Desktop);
        assert_ne!(first, plain);
        assert_eq!(
            first.matches("<text ").count(),
            plain.matches("<text ").count()
        );
    }

    #[test]
    fn print_variant_draws_clean_strokes() {
        assert_eq!(svg(FLOW, Variant::Print), svg(&clean(FLOW), Variant::Print));
    }

    #[test]
    fn another_specification_is_displaced_differently() {
        let other = FLOW.replace("From idea to shipped", "From idea to done");
        let shapes = |svg: String| svg.split("<path d=\"").nth(1).map(str::to_owned);
        assert_ne!(
            shapes(svg(FLOW, Variant::Desktop)),
            shapes(svg(&other, Variant::Desktop))
        );
    }
}
