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

/// CSS rules that hide series when their checkbox is deselected (HTML profile only).
/// Browsers that don't understand `:has()` ignore these rules; the chart stays fully visible.
const FILTER_STYLE: &str = ".chartlet-wrapper{display:inline-block;max-width:100%}.chartlet-filter{border:none;padding:0;margin:0 0 12px 0}.chartlet-filter legend{font-size:14px;font-weight:650;margin-bottom:4px}.chartlet-filter label{display:inline-flex;align-items:center;min-height:44px;font-size:13px;margin-right:14px;cursor:pointer;white-space:nowrap}.chartlet-filter input{margin-right:4px}.chartlet-filter input:focus-visible{outline:2px solid #2563eb;outline-offset:2px}.chartlet-wrapper:has(.chartlet-filter input.series-0:not(:checked)) .chartlet-root [data-series=\"0\"]{display:none}.chartlet-wrapper:has(.chartlet-filter input.series-1:not(:checked)) .chartlet-root [data-series=\"1\"]{display:none}.chartlet-wrapper:has(.chartlet-filter input.series-2:not(:checked)) .chartlet-root [data-series=\"2\"]{display:none}.chartlet-wrapper:has(.chartlet-filter input.series-3:not(:checked)) .chartlet-root [data-series=\"3\"]{display:none}";

/// CSS rules for radio-selectable zoom panels. Only the panel whose radio is checked shows;
/// the first radio carries `checked`, so the first step is the default view.
const ZOOM_STYLE: &str = ".chartlet-zoom{border:none;padding:0;margin:0 0 12px 0}.chartlet-zoom legend{font-size:14px;font-weight:650;margin-bottom:4px}.chartlet-zoom label{display:inline-flex;align-items:center;min-height:44px;font-size:13px;margin-right:14px;cursor:pointer;white-space:nowrap}.chartlet-zoom input{margin-right:4px}.chartlet-zoom input:focus-visible{outline:2px solid #2563eb;outline-offset:2px}.chartlet-panel{display:none}.chartlet-wrapper:has(.chartlet-zoom input.zoom-0:checked) .chartlet-panel-0{display:block}.chartlet-wrapper:has(.chartlet-zoom input.zoom-1:checked) .chartlet-panel-1{display:block}.chartlet-wrapper:has(.chartlet-zoom input.zoom-2:checked) .chartlet-panel-2{display:block}.chartlet-wrapper:has(.chartlet-zoom input.zoom-3:checked) .chartlet-panel-3{display:block}";

pub(crate) fn svg(scene: &Scene, spec: &ChartSpec, description: &str, id_prefix: &str) -> String {
    let title_id = format!("{id_prefix}-title");
    let description_id = format!("{id_prefix}-description");
    let has_series = spec.series.len() > 1;
    let mut output = String::new();
    write!(
        output,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" role=\"img\" aria-labelledby=\"{title_id} {description_id}\" class=\"chartlet-root\">",
        scene.width, scene.height, scene.width, scene.height
    )
    .expect("writing to String cannot fail");
    write!(
        output,
        "<title id=\"{title_id}\">{}</title><desc id=\"{description_id}\">{}</desc><style>{STYLE}{}{}</style>",
        escape(&spec.title),
        escape(description),
        if has_series { SERIES_STYLE } else { "" },
        if has_series { FILTER_STYLE } else { "" },
    )
    .expect("writing to String cannot fail");

    for element in &scene.elements {
        emit_element(element, &mut output);
    }
    output.push_str("</svg>");
    output
}

/// Writes a scene element into the SVG with optional `data-series` and `<title>` child.
fn emit_element(element: &Element, output: &mut String) {
    match element {
        Element::Circle(circle) => {
            open("circle", circle.series_index, output);
            write!(
                output,
                " cx=\"{}\" cy=\"{}\" r=\"{}\" class=\"{}\"",
                number(circle.cx),
                number(circle.cy),
                number(circle.radius),
                circle.class
            )
            .expect("write");
            close("circle", circle.tooltip.as_deref(), output);
        }
        Element::Line(line) => write!(
            output,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"{}\"/>",
            number(line.x1),
            number(line.y1),
            number(line.x2),
            number(line.y2),
            line.class
        )
        .expect("write"),
        Element::Rect(rect) => {
            open("rect", rect.series_index, output);
            write!(
                output,
                " x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" class=\"{}\"",
                number(rect.x),
                number(rect.y),
                number(rect.width),
                number(rect.height),
                rect.class
            )
            .expect("write");
            close("rect", rect.tooltip.as_deref(), output);
        }
        Element::SeriesText(text, series_index) => emit_text(text, Some(*series_index), output),
        Element::Polyline(polyline) => {
            open("polyline", polyline.series_index, output);
            let points = polyline
                .points
                .iter()
                .map(|(x, y)| format!("{},{}", number(*x), number(*y)))
                .collect::<Vec<_>>()
                .join(" ");
            write!(
                output,
                " points=\"{}\" class=\"{}\"",
                points, polyline.class
            )
            .expect("write");
            close("polyline", polyline.tooltip.as_deref(), output);
        }
        Element::Text(text) => emit_text(text, None, output),
    }
}

fn emit_text(text: &crate::scene::Text, series_index: Option<usize>, output: &mut String) {
    open("text", series_index, output);
    write!(
        output,
        " x=\"{}\" y=\"{}\" text-anchor=\"{}\" class=\"{}\">{}</text>",
        number(text.x),
        number(text.y),
        anchor(text.anchor),
        text.class,
        escape(&text.content)
    )
    .expect("write");
}

fn open(tag: &str, series_index: Option<usize>, output: &mut String) {
    write!(output, "<{tag}").expect("write");
    if let Some(i) = series_index {
        write!(output, " data-series=\"{i}\"").expect("write");
    }
}

fn close(tag: &str, tooltip: Option<&str>, output: &mut String) {
    if let Some(text) = tooltip {
        write!(output, "><title>{}</title></{tag}>", escape(text)).expect("write");
    } else {
        output.push_str("/>");
    }
}

pub(crate) fn html(svg: &str, spec: &ChartSpec, table_mode: TableMode) -> String {
    html_document(&[(String::new(), svg.to_owned())], spec, table_mode, None)
}

pub(crate) fn html_zoom(
    panels: &[(String, String)],
    spec: &ChartSpec,
    table_mode: TableMode,
    zoom_name: &str,
) -> String {
    html_document(panels, spec, table_mode, Some(zoom_name))
}

/// Builds the HTML figure. With one panel and no zoom name this is the plain figure; with
/// several panels it adds a radio group that switches between pre-computed variants.
fn html_document(
    panels: &[(String, String)],
    spec: &ChartSpec,
    table_mode: TableMode,
    zoom_name: Option<&str>,
) -> String {
    let mut output = String::new();
    let has_series = spec.series.len() > 1;
    let zoom = zoom_name.is_some() && panels.len() > 1;
    if has_series || zoom {
        output.push_str("<div class=\"chartlet-wrapper\">");
    }
    if has_series {
        output.push_str("<fieldset class=\"chartlet-filter\"><legend>Series</legend>");
        for (i, series) in spec.series.iter().enumerate() {
            write!(
                output,
                "<label><input type=\"checkbox\" class=\"series-{i}\" checked> {}</label>",
                escape(&series.name)
            )
            .expect("write");
        }
        output.push_str("</fieldset>");
    }
    if zoom {
        let name = zoom_name.expect("zoom name is present");
        write!(
            output,
            "<style>{ZOOM_STYLE}</style><fieldset class=\"chartlet-zoom\"><legend>View</legend>"
        )
        .expect("write");
        for (i, (label, _)) in panels.iter().enumerate() {
            let checked = if i == 0 { " checked" } else { "" };
            write!(
                output,
                "<label><input type=\"radio\" name=\"zoom-{name}\" class=\"zoom-{i}\"{checked}> {}</label>",
                escape(label)
            )
            .expect("write");
        }
        output.push_str("</fieldset>");
    }
    output.push_str("<figure class=\"chartlet-figure\">");
    write!(output, "<figcaption>{}</figcaption>", escape(&spec.title)).expect("write");
    for (i, (_, svg)) in panels.iter().enumerate() {
        if zoom {
            write!(
                output,
                "<div class=\"chartlet-panel chartlet-panel-{i}\">{svg}</div>"
            )
            .expect("write");
        } else {
            output.push_str(svg);
        }
    }
    if let Some(source) = &spec.source {
        write!(
            output,
            "<p class=\"chartlet-source\">Source: {}</p>",
            escape(source)
        )
        .expect("write");
    }
    render_data_table(&mut output, spec, table_mode);
    output.push_str("</figure>");
    if has_series || zoom {
        output.push_str("</div>");
    }
    output
}

fn render_data_table(output: &mut String, spec: &ChartSpec, table_mode: TableMode) {
    if table_mode == TableMode::Details {
        output.push_str("<details class=\"chartlet-data\"><summary>Show chart data</summary>");
    } else {
        output.push_str("<div class=\"chartlet-data\">");
    }
    let dataset = spec.dataset();
    write!(
        output,
        "<table><caption>Data for {}</caption><thead><tr><th scope=\"col\">Category</th>",
        escape(&spec.title)
    )
    .expect("write");
    for series in &dataset.series {
        write!(
            output,
            "<th scope=\"col\">{}</th>",
            escape(series.name.unwrap_or("Value"))
        )
        .expect("write");
    }
    output.push_str("</tr></thead><tbody>");
    for (index, category) in dataset.categories.iter().enumerate() {
        write!(output, "<tr><th scope=\"row\">{}</th>", escape(category)).expect("write");
        for series in &dataset.series {
            write!(
                output,
                "<td>{}</td>",
                series.values[index].map_or_else(
                    || "Missing".to_owned(),
                    |value| escape(&format_value(value, spec.value_axis.format)),
                )
            )
            .expect("write");
        }
        output.push_str("</tr>");
    }
    output.push_str("</tbody></table>");
    if table_mode == TableMode::Details {
        output.push_str("</details>");
    } else {
        output.push_str("</div>");
    }
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
