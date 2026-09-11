use std::fmt::Write;

use crate::{
    TableMode,
    layout::format_value,
    scene::{Element, Scene, TextAnchor},
    spec::ChartSpec,
};

const STYLE: &str = ".chartlet-root{max-width:100%;height:auto;font-family:Inter,ui-sans-serif,system-ui,sans-serif;color:#172033}.chartlet-title{font-size:22px;font-weight:650;fill:#172033}.chartlet-label,.chartlet-tick,.chartlet-value,.chartlet-value-inverse,.chartlet-axis-title{font-size:12px;fill:#344054}.chartlet-value,.chartlet-value-inverse{font-weight:600}.chartlet-value{fill:#172033}.chartlet-value-inverse{fill:#fff}.chartlet-axis-title{font-weight:600}.chartlet-grid{stroke:#d9dee8;stroke-width:1}.chartlet-zero{stroke:#667085;stroke-width:1.5}.chartlet-bar,.chartlet-point{fill:#2563eb}.chartlet-line{fill:none;stroke:#2563eb;stroke-width:3;stroke-linecap:round;stroke-linejoin:round}";

/// Only emitted for multi-series charts. The palette stays distinguishable under simulated
/// protanopia, deuteranopia and tritanopia and keeps at least 4.5:1 contrast on white.
const SERIES_STYLE: &str = ".chartlet-legend{font-size:12px;fill:#344054}.chartlet-series-1{fill:#2563eb}.chartlet-series-2{fill:#c2410c}.chartlet-series-3{fill:#475569}.chartlet-series-4{fill:#111827}";

pub(crate) fn svg(scene: &Scene, spec: &ChartSpec, description: &str, id_prefix: &str) -> String {
    let title_id = format!("{id_prefix}-title");
    let description_id = format!("{id_prefix}-description");
    let mut output = String::new();
    write!(
        output,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" role=\"img\" aria-labelledby=\"{} {}\" class=\"chartlet-root\">",
        scene.width, scene.height, scene.width, scene.height, title_id, description_id
    )
    .expect("writing to String cannot fail");
    write!(
        output,
        "<title id=\"{}\">{}</title><desc id=\"{}\">{}</desc><style>{STYLE}{}</style>",
        title_id,
        escape(&spec.title),
        description_id,
        escape(description),
        if spec.series.len() > 1 {
            SERIES_STYLE
        } else {
            ""
        }
    )
    .expect("writing to String cannot fail");

    for element in &scene.elements {
        match element {
            Element::Circle(circle) => write!(
                output,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" class=\"{}\"/>",
                number(circle.cx),
                number(circle.cy),
                number(circle.radius),
                circle.class
            ),
            Element::Line(line) => write!(
                output,
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"{}\"/>",
                number(line.x1),
                number(line.y1),
                number(line.x2),
                number(line.y2),
                line.class
            ),
            Element::Rect(rect) => write!(
                output,
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" class=\"{}\"/>",
                number(rect.x),
                number(rect.y),
                number(rect.width),
                number(rect.height),
                rect.class
            ),
            Element::Polyline(polyline) => {
                let points = polyline
                    .points
                    .iter()
                    .map(|(x, y)| format!("{},{}", number(*x), number(*y)))
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(
                    output,
                    "<polyline points=\"{}\" class=\"{}\"/>",
                    points, polyline.class
                )
            }
            Element::Text(text) => write!(
                output,
                "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\" class=\"{}\">{}</text>",
                number(text.x),
                number(text.y),
                anchor(text.anchor),
                text.class,
                escape(&text.content)
            ),
        }
        .expect("writing to String cannot fail");
    }
    output.push_str("</svg>");
    output
}

pub(crate) fn html(svg: &str, spec: &ChartSpec, table_mode: TableMode) -> String {
    let mut output = String::from("<figure class=\"chartlet-figure\">");
    write!(output, "<figcaption>{}</figcaption>", escape(&spec.title))
        .expect("writing to String cannot fail");
    output.push_str(svg);
    if let Some(source) = &spec.source {
        write!(
            output,
            "<p class=\"chartlet-source\">Source: {}</p>",
            escape(source)
        )
        .expect("writing to String cannot fail");
    }
    if table_mode == TableMode::Details {
        output.push_str("<details class=\"chartlet-data\"><summary>Show chart data</summary>");
    } else {
        output.push_str("<div class=\"chartlet-data\">");
    }
    let dataset = spec.dataset();
    output.push_str("<table><thead><tr><th scope=\"col\">Category</th>");
    for series in &dataset.series {
        write!(
            output,
            "<th scope=\"col\">{}</th>",
            escape(series.name.unwrap_or("Value"))
        )
        .expect("writing to String cannot fail");
    }
    output.push_str("</tr></thead><tbody>");
    for (index, category) in dataset.categories.iter().enumerate() {
        write!(output, "<tr><th scope=\"row\">{}</th>", escape(category))
            .expect("writing to String cannot fail");
        for series in &dataset.series {
            write!(
                output,
                "<td>{}</td>",
                series.values[index].map_or_else(
                    || "Missing".to_owned(),
                    |value| escape(&format_value(value, spec.value_axis.format)),
                )
            )
            .expect("writing to String cannot fail");
        }
        output.push_str("</tr>");
    }
    output.push_str("</tbody></table>");
    if table_mode == TableMode::Details {
        output.push_str("</details>");
    } else {
        output.push_str("</div>");
    }
    output.push_str("</figure>");
    output
}

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

const fn anchor(anchor: TextAnchor) -> &'static str {
    match anchor {
        TextAnchor::Start => "start",
        TextAnchor::Middle => "middle",
        TextAnchor::End => "end",
    }
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
