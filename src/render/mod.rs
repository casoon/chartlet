//! Writing a laid-out chart: the SVG of its scene with its stylesheet, the hooks of the
//! interactive module, and the HTML figure around it with its data table.

mod hooks;
mod html;
mod sketch;
mod styles;
mod svg;
mod table;

pub(crate) use hooks::Hooks;
pub(crate) use html::{Panel, Stage, ViewSection, html, html_views, html_zoom};
pub(crate) use styles::shared_stylesheet;
pub(crate) use svg::{StyleMode, svg};
pub(crate) use table::data_table;

pub(crate) fn escape(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
    output
}

fn number(value: f64) -> String {
    let value = if value == 0.0 { 0.0 } else { value };
    let fixed = format!("{value:.3}");
    fixed.trim_end_matches('0').trim_end_matches('.').to_owned()
}

#[cfg(test)]
mod tests {
    use super::{escape, number};

    #[test]
    fn escapes_all_xml_special_characters() {
        assert_eq!(escape("<&>\"'"), "&lt;&amp;&gt;&quot;&#39;");
    }

    #[test]
    fn formats_coordinates_deterministically() {
        assert_eq!(number(-0.0), "0");
        assert_eq!(number(1.250_01), "1.25");
    }
}
