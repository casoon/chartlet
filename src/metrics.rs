pub trait TextMetrics {
    fn width(&self, text: &str, font_size: f64) -> f64;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BuiltinMetrics;

impl BuiltinMetrics {
    pub const VERSION: &'static str = "builtin-latin-v1";
}

impl TextMetrics for BuiltinMetrics {
    fn width(&self, text: &str, font_size: f64) -> f64 {
        let units = text.chars().map(character_width).sum::<f64>();
        units * font_size
    }
}

/// Texts of this size or smaller are the technical text of an editorial diagram: sublabels,
/// edge labels, tags and members are set in monospace; names (13 px and up) stay in the sans
/// font.
const MONO_UP_TO: f64 = 12.5;

/// The advance of a monospace character in em: that of the system monospace faces and of the
/// bundled `JetBrains Mono`.
pub(crate) const MONO_ADVANCE: f64 = 0.6;

/// The metrics of an editorial diagram: monospace widths for small text, the wrapped metrics for
/// the rest. Without `mono` it is the wrapped metrics alone.
pub(crate) struct Face<'a, M> {
    sans: &'a M,
    mono: bool,
}

impl<'a, M: TextMetrics> Face<'a, M> {
    pub(crate) const fn new(sans: &'a M, mono: bool) -> Self {
        Self { sans, mono }
    }
}

impl<M: TextMetrics> TextMetrics for Face<'_, M> {
    fn width(&self, text: &str, font_size: f64) -> f64 {
        if self.mono && font_size <= MONO_UP_TO {
            // A text has far fewer characters than f64 has mantissa bits.
            #[allow(clippy::cast_precision_loss)]
            let characters = text.chars().count() as f64;
            characters * MONO_ADVANCE * font_size
        } else {
            self.sans.width(text, font_size)
        }
    }
}

fn character_width(character: char) -> f64 {
    match character {
        'i' | 'j' | 'l' | 'I' | '1' | '|' | '!' | '.' | ',' | ':' | ';' => 0.30,
        'm' | 'w' | 'M' | 'W' | '@' | '%' => 0.90,
        ' ' => 0.32,
        character if character.is_ascii_uppercase() => 0.67,
        character if character.is_ascii_digit() => 0.56,
        _ => 0.56,
    }
}
