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
