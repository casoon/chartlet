//! The SVG of a scene: its root with title, description and stylesheet, the hatch patterns, and
//! every element.

use std::fmt::Write;

use super::{
    escape,
    hooks::{Hooks, emit_data_block, emit_hook},
    number,
    styles::{base_style, layer_style, print_stylesheet, scope_stylesheet, type_class},
};
use crate::{
    Styles,
    scene::{Element, Scene, TextAnchor},
    spec::{ChartSpec, ChartType, Theme},
};

/// Where a chart's stylesheet goes: all of it inline, only its own declared colors inline with the
/// rest in the shared stylesheet, or all of it inline and resolved for the print variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StyleMode {
    Inline,
    External,
    /// Resolved for renderers outside the browser, see [`print_stylesheet`].
    Print,
}

impl From<Styles> for StyleMode {
    fn from(styles: Styles) -> Self {
        match styles {
            Styles::Inline => Self::Inline,
            Styles::External => Self::External,
        }
    }
}

/// Serializes a laid-out chart with its stylesheet placed as `styles` says.
pub(crate) fn svg(
    scene: &Scene,
    spec: &ChartSpec,
    description: &str,
    id_prefix: &str,
    styles: StyleMode,
    hooks: Hooks,
) -> String {
    let title_id = format!("{id_prefix}-title");
    let description_id = format!("{id_prefix}-description");
    let is_multiples = spec.chart_type == ChartType::Multiples;
    let is_blueprint = spec.theme == Theme::Blueprint;
    let is_dark = spec.theme == Theme::Dark || is_blueprint;
    let mut output = String::new();
    write!(
        output,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" id=\"{id_prefix}\" role=\"img\" aria-labelledby=\"{title_id} {description_id}\" class=\"chartlet-root{}{}\"{}{}>",
        scene.width,
        scene.height,
        scene.width,
        scene.height,
        match (is_dark, is_multiples) {
            (true, true) => " chartlet-theme-dark chartlet-multiples",
            (true, false) if is_blueprint => " chartlet-theme-dark chartlet-blueprint",
            (true, false) => " chartlet-theme-dark",
            (false, true) => " chartlet-multiples",
            (false, false) => "",
        },
        // The shared stylesheet addresses each chart type by this class.
        if styles == StyleMode::External {
            format!(" {}", type_class(spec.chart_type))
        } else {
            String::new()
        },
        if hooks == Hooks::Off {
            String::new()
        } else {
            format!(
                " data-chartlet-type=\"{}\" data-chartlet-id=\"{id_prefix}\" data-chartlet-locale=\"{}\"",
                crate::spec::type_name(spec.chart_type),
                spec.locale.code()
            )
        },
        // Stretched stripes take whatever box the page gives them.
        if spec.stripes.as_ref().is_some_and(|stripes| stripes.stretch) {
            " preserveAspectRatio=\"none\""
        } else {
            ""
        }
    )
    .expect("writing to String cannot fail");
    let print = styles == StyleMode::Print;
    let stylesheet = match styles {
        StyleMode::Inline => base_style(spec, false) + &layer_style(spec),
        StyleMode::External => layer_style(spec),
        StyleMode::Print => print_stylesheet(
            &(base_style(spec, true) + &layer_style(spec)),
            (is_dark, is_blueprint),
        ),
    };
    write!(
        output,
        "<title id=\"{title_id}\">{}</title><desc id=\"{description_id}\">{}</desc>",
        escape(&spec.title),
        escape(description),
    )
    .expect("writing to String cannot fail");
    if !stylesheet.is_empty() {
        write!(
            output,
            "<style>{}</style>",
            scope_stylesheet(&stylesheet, &format!("#{id_prefix}"), print),
        )
        .expect("writing to String cannot fail");
    }
    if hooks == Hooks::WithData {
        emit_data_block(spec, &mut output);
    }
    emit_hatches(spec, id_prefix, &mut output);
    crate::symbols::emit_defs(scene, id_prefix, &mut output);

    // A dark chart cannot know the color of the page behind it, so it paints its own surface.
    if is_dark {
        write!(
            output,
            "<rect class=\"chartlet-background\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\"/>",
            scene.width, scene.height
        )
        .expect("writing to String cannot fail");
    }
    let hook_spec = (hooks != Hooks::Off).then_some(spec);
    for element in &scene.elements {
        emit_element(element, id_prefix, hook_spec, &mut output);
    }
    output.push_str("</svg>");
    output
}

/// The hatch patterns a chart refers to: one per modeled band, drawn in the color of its line,
/// and one for the modeled spans of a range chart. Pattern content cannot inherit the color of
/// the shape that uses it, which is why every band has its own.
fn emit_hatches(spec: &ChartSpec, id_prefix: &str, output: &mut String) {
    let mut patterns = Vec::new();
    for entry in spec.data_layers() {
        if entry.layer.modeled && entry.layer.has_band() {
            let class = if entry.layer.resolved_color().is_some() {
                format!("chartlet-style-{}", entry.global)
            } else {
                format!(
                    "chartlet-line-series-{}",
                    spec.palette_index(entry.layer) + 1
                )
            };
            patterns.push((format!("hatch-{}", entry.global), class));
        }
    }
    if spec.chart_type == ChartType::Rangebar && spec.ranges.iter().any(|range| range.modeled) {
        patterns.push(("hatch-range".to_owned(), String::new()));
    }
    if patterns.is_empty() {
        return;
    }
    output.push_str("<defs>");
    for (suffix, class) in patterns {
        let class = if class.is_empty() {
            String::new()
        } else {
            format!(" {class}")
        };
        write!(
            output,
            "<pattern id=\"{id_prefix}-{suffix}\" width=\"6\" height=\"6\" patternUnits=\"userSpaceOnUse\" patternTransform=\"rotate(45)\"><line x1=\"0\" y1=\"0\" x2=\"0\" y2=\"6\" class=\"chartlet-hatch-line{class}\"/></pattern>"
        )
        .expect("writing to String cannot fail");
    }
    output.push_str("</defs>");
}

/// Writes a scene element into the SVG with optional `data-series` and `<title>` child.
/// A hatched shape refers to its pattern by an attribute rather than by a stylesheet rule: the
/// pattern id carries the chart's `idPrefix`, and a rule would reach every chart on the page.
fn hatch_fill(class: &str, style_index: Option<usize>, id_prefix: &str) -> String {
    match (class, style_index) {
        ("chartlet-hatch", Some(index)) => format!(" fill=\"url(#{id_prefix}-hatch-{index})\""),
        ("chartlet-range-hatch", _) => format!(" fill=\"url(#{id_prefix}-hatch-range)\""),
        _ => String::new(),
    }
}

fn emit_element(
    element: &Element,
    id_prefix: &str,
    hooks: Option<&ChartSpec>,
    output: &mut String,
) {
    match element {
        Element::Circle(circle) => {
            open("circle", circle.series_index, output);
            write!(
                output,
                " cx=\"{}\" cy=\"{}\" r=\"{}\" class=\"{}{}\"",
                number(circle.cx),
                number(circle.cy),
                number(circle.radius),
                circle.class,
                point_class(circle.style_index),
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
                " x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{} class=\"{}{}\"",
                number(rect.x),
                number(rect.y),
                number(rect.width),
                number(rect.height),
                hatch_fill(rect.class, rect.style_index, id_prefix),
                rect.class,
                swatch_class(rect.style_index),
            )
            .expect("write");
            close("rect", rect.tooltip.as_deref(), output);
        }
        Element::SeriesText(text, series_index) => {
            emit_text(text, Some(*series_index), output);
        }
        Element::Polyline(polyline) => emit_polyline(polyline, id_prefix, output),
        Element::Text(text) => emit_text(text, None, output),
        Element::Hook(hook) => {
            if let Some(spec) = hooks {
                emit_hook(hook, spec, output);
            }
        }
        Element::Group(classes) => {
            write!(output, "<g class=\"{classes}\">").expect("write");
        }
        Element::NodeGroup(classes, id) => {
            write!(output, "<g class=\"{classes}\"").expect("write");
            if hooks.is_some() {
                write!(output, " data-chartlet-node=\"{}\"", escape(id)).expect("write");
            }
            output.push('>');
        }
        Element::EdgeGroup(classes, id) => {
            write!(output, "<g class=\"{classes}\"").expect("write");
            if hooks.is_some() {
                write!(output, " data-chartlet-edge=\"{}\"", escape(id)).expect("write");
            }
            output.push('>');
        }
        Element::GroupEnd => output.push_str("</g>"),
        Element::Hotspot(_) => {}
        Element::Symbol(symbol) => write!(
            output,
            "<use href=\"#{id_prefix}-symbol-{}\" x=\"{}\" y=\"{}\" width=\"{size}\" height=\"{size}\" class=\"chartlet-arch-symbol\"/>",
            symbol.name,
            number(symbol.x),
            number(symbol.y),
            size = number(crate::symbols::SIZE),
        )
        .expect("write"),
    }
}

/// Writes a polyline.
fn emit_polyline(polyline: &crate::scene::Polyline, id_prefix: &str, output: &mut String) {
    open("polyline", polyline.series_index, output);
    let points = polyline
        .points
        .iter()
        .map(|(x, y)| format!("{},{}", number(*x), number(*y)))
        .collect::<Vec<_>>()
        .join(" ");
    write!(
        output,
        " points=\"{}\"{} class=\"{}{}\"",
        points,
        hatch_fill(polyline.class, polyline.style_index, id_prefix),
        polyline.class,
        style_class(polyline.style_index),
    )
    .expect("write");
    close("polyline", polyline.tooltip.as_deref(), output);
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

/// The class that carries a specification-declared stroke color, with the leading space, or
/// nothing at all when the element has none.
fn style_class(style_index: Option<usize>) -> String {
    style_index.map_or_else(String::new, |index| format!(" chartlet-style-{index}"))
}

/// The same, for the fill of a legend swatch.
fn swatch_class(style_index: Option<usize>) -> String {
    style_index.map_or_else(String::new, |index| {
        format!(" chartlet-style-{index}-swatch")
    })
}

/// The same, for the fill of a data point.
fn point_class(style_index: Option<usize>) -> String {
    style_index.map_or_else(String::new, |index| {
        format!(" chartlet-style-{index}-point")
    })
}

fn close(tag: &str, tooltip: Option<&str>, output: &mut String) {
    if let Some(text) = tooltip {
        write!(output, "><title>{}</title></{tag}>", escape(text)).expect("write");
    } else {
        output.push_str("/>");
    }
}

const fn anchor(anchor: TextAnchor) -> &'static str {
    match anchor {
        TextAnchor::Start => "start",
        TextAnchor::Middle => "middle",
        TextAnchor::End => "end",
    }
}
