//! Symbols for the components of an architecture diagram: a small, fixed set of generic stroke
//! icons for technology categories, drawn for chartlet and licensed with it. A symbol only
//! decorates a component and names its category; the kind keeps its shape and its meaning.
//!
//! Every symbol a chart uses is written once into its `<defs>`, under the chart's ID prefix, and
//! each component refers to it; a chart without symbols carries none.

use std::fmt::Write as _;

use crate::{
    scene::{Element, Scene},
    spec::Locale,
};

/// The size of a symbol on the page, and the room it takes above a component's label.
pub(crate) const SIZE: f64 = 18.0;
pub(crate) const ROOM: f64 = 22.0;

pub(crate) const STYLE: &str = ".chartlet-arch-symbol{fill:none;stroke:var(--chartlet-muted);stroke-width:1.75;stroke-linecap:round;stroke-linejoin:round}";

/// A symbol: its name in the specification, its outline on a 24 × 24 grid, and the category it
/// stands for in English and German.
#[derive(Debug)]
pub(crate) struct Symbol {
    pub name: &'static str,
    path: &'static str,
    en: &'static str,
    de: &'static str,
}

impl Symbol {
    pub(crate) const fn category(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::En => self.en,
            Locale::De => self.de,
        }
    }
}

const fn symbol(
    name: &'static str,
    path: &'static str,
    en: &'static str,
    de: &'static str,
) -> Symbol {
    Symbol { name, path, en, de }
}

/// Every symbol, in the order their definitions are written.
pub(crate) const SYMBOLS: [Symbol; 16] = [
    symbol(
        "browser",
        "M3 5h18v14H3zM3 9h18M6 7h.01M8.5 7h.01",
        "web browser",
        "Webbrowser",
    ),
    symbol(
        "mobile",
        "M8 2.5h8a1.5 1.5 0 0 1 1.5 1.5v16a1.5 1.5 0 0 1-1.5 1.5H8A1.5 1.5 0 0 1 6.5 20V4A1.5 1.5 0 0 1 8 2.5zM11 18h2",
        "mobile app",
        "Mobile App",
    ),
    symbol(
        "api-gateway",
        "M8 6l-5 6 5 6M16 6l5 6-5 6M10 12h4",
        "API gateway",
        "API-Gateway",
    ),
    symbol(
        "load-balancer",
        "M3 12h5M8 12l4-6h7M8 12h11M8 12l4 6h7M17 4l2 2-2 2M17 10l2 2-2 2M17 16l2 2-2 2",
        "load balancer",
        "Lastverteiler",
    ),
    symbol(
        "cdn",
        "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18zM3 12h18M12 3c-3.5 3-3.5 15 0 18M12 3c3.5 3 3.5 15 0 18",
        "content delivery network",
        "Content-Delivery-Netzwerk",
    ),
    symbol(
        "firewall",
        "M3 5h18v14H3zM3 9.7h18M3 14.3h18M9 5v4.7M15 5v4.7M6 9.7v4.6M12 9.7v4.6M18 9.7v4.6M9 14.3V19M15 14.3V19",
        "firewall",
        "Firewall",
    ),
    symbol(
        "identity",
        "M8 11a4 4 0 1 0 0 8a4 4 0 1 0 0-8zM10.9 12.1L20 3M16 7l2.5 2.5M13.5 9.5L15 11",
        "identity provider",
        "Identitätsdienst",
    ),
    symbol(
        "container",
        "M12 3l8 4.5v9L12 21l-8-4.5v-9zM4 7.5l8 4.5 8-4.5M12 12v9",
        "container",
        "Container",
    ),
    symbol(
        "orchestrator",
        "M12 2.5l8.2 4.75v9.5L12 21.5l-8.2-4.75v-9.5zM12 9.5a2.5 2.5 0 1 0 0 5a2.5 2.5 0 1 0 0-5zM12 9.5V5.5M14.2 13.25l3.4 2M9.8 13.25l-3.4 2",
        "container orchestrator",
        "Container-Orchestrierung",
    ),
    symbol(
        "function",
        "M7 4h2.5l8.5 16M5.5 20l6-10",
        "serverless function",
        "Serverless-Funktion",
    ),
    symbol(
        "cloud",
        "M7 18.5h10a4 4 0 0 0 .6-7.95A6 6 0 0 0 6.2 9.6 4.5 4.5 0 0 0 7 18.5z",
        "cloud service",
        "Cloud-Dienst",
    ),
    symbol(
        "cache",
        "M13 2.5L4.5 13.5H11l-1 8 8.5-11H12z",
        "in-memory cache",
        "In-Memory-Cache",
    ),
    symbol(
        "object-storage",
        "M3.5 6a8.5 2.5 0 1 0 17 0a8.5 2.5 0 1 0-17 0M3.5 6l2 13c.3 2 12.7 2 13 0l2-13",
        "object storage",
        "Objektspeicher",
    ),
    symbol(
        "message-broker",
        "M9 4h6v16H9zM2.5 8H9M2.5 16H9M15 12h6.5M19 9.5l2.5 2.5-2.5 2.5M6.5 5.5L9 8 6.5 10.5M6.5 13.5L9 16l-2.5 2.5",
        "message broker",
        "Message-Broker",
    ),
    symbol(
        "search-index",
        "M10.5 4a6.5 6.5 0 1 0 0 13a6.5 6.5 0 1 0 0-13zM15.3 15.3L20.5 20.5",
        "search index",
        "Suchindex",
    ),
    symbol(
        "monitoring",
        "M3 4h18v13H3zM8.5 20.5h7M12 17v3.5M6 11h3l2-4 2 7 2-3h3",
        "monitoring",
        "Monitoring",
    ),
];

/// The symbol named `name`.
pub(crate) fn find(name: &str) -> Option<&'static Symbol> {
    SYMBOLS.iter().find(|symbol| symbol.name == name)
}

/// The names of all symbols, for a diagnostic.
pub(crate) fn names() -> String {
    SYMBOLS
        .iter()
        .map(|symbol| symbol.name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// The definitions of the symbols `scene` uses, each once, in the order of [`SYMBOLS`]; nothing
/// when it uses none.
pub(crate) fn emit_defs(scene: &Scene, id_prefix: &str, output: &mut String) {
    let used: Vec<&Symbol> = SYMBOLS
        .iter()
        .filter(|symbol| {
            scene.elements.iter().any(
                |element| matches!(element, Element::Symbol(placed) if placed.name == symbol.name),
            )
        })
        .collect();
    if used.is_empty() {
        return;
    }
    output.push_str("<defs>");
    for symbol in used {
        write!(
            output,
            "<symbol id=\"{id_prefix}-symbol-{}\" viewBox=\"0 0 24 24\"><path d=\"{}\"/></symbol>",
            symbol.name, symbol.path
        )
        .expect("writing to String cannot fail");
    }
    output.push_str("</defs>");
}

#[cfg(test)]
mod tests {
    use super::SYMBOLS;

    #[test]
    fn the_schema_lists_every_symbol_in_order() {
        let names: Vec<String> = SYMBOLS
            .iter()
            .map(|symbol| format!("\"{}\"", symbol.name))
            .collect();
        let schema = include_str!("../schema/chartlet.schema.json");
        assert!(schema.contains(&format!("\"enum\": [{}]", names.join(", "))));
    }
}
