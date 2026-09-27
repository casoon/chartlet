use std::fmt::Write;

use crate::{
    TableMode,
    layout::format_value,
    scene::{Element, Scene, TextAnchor, TextStyle},
    spec::{ChartSpec, ChartType, Mark, Theme},
};

/// The light palette. Every color is a CSS custom property so a host page can override a single
/// color, and so the dark palette below only has to restate the values.
const STYLE: &str = ".chartlet-root{--chartlet-text:#172033;--chartlet-muted:#344054;--chartlet-grid:#d9dee8;--chartlet-zero:#667085;--chartlet-accent:#2563eb;--chartlet-background:#fff;--chartlet-color-1:#2563eb;--chartlet-color-2:#c2410c;--chartlet-color-3:#475569;--chartlet-color-4:#111827;max-width:100%;height:auto;font-family:Inter,ui-sans-serif,system-ui,sans-serif;color:var(--chartlet-text)}.chartlet-title{font-size:22px;font-weight:650;fill:var(--chartlet-text)}.chartlet-label,.chartlet-tick,.chartlet-value,.chartlet-value-inverse,.chartlet-axis-title{font-size:12px;fill:var(--chartlet-muted)}.chartlet-value,.chartlet-value-inverse{font-weight:600}.chartlet-value{fill:var(--chartlet-text)}.chartlet-value-inverse{fill:var(--chartlet-background)}.chartlet-axis-title{font-weight:600}.chartlet-grid{stroke:var(--chartlet-grid);stroke-width:1}.chartlet-zero{stroke:var(--chartlet-zero);stroke-width:1.5}.chartlet-bar,.chartlet-point{fill:var(--chartlet-accent)}.chartlet-line{fill:none;stroke:var(--chartlet-accent);stroke-width:3;stroke-linecap:round;stroke-linejoin:round}.chartlet-background{fill:var(--chartlet-background)}";

/// The dark palette. A dark chart paints its own background: unlike the light profile it cannot
/// rely on the page behind it, which may be any color.
const DARK_STYLE: &str = ".chartlet-theme-dark{--chartlet-text:#e8edf6;--chartlet-muted:#a9b4c7;--chartlet-grid:#2a3342;--chartlet-zero:#7b8697;--chartlet-accent:#7ea6ff;--chartlet-background:#0e131c;--chartlet-color-1:#7ea6ff;--chartlet-color-2:#ff9c72;--chartlet-color-3:#9aa7bd;--chartlet-color-4:#e2e8f4}";

/// Only emitted for multi-series charts. The palette stays distinguishable under simulated
/// protanopia, deuteranopia and tritanopia and keeps at least 4.5:1 contrast against the
/// theme background.
const SERIES_STYLE: &str = ".chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}.chartlet-series-1{fill:var(--chartlet-color-1)}.chartlet-series-2{fill:var(--chartlet-color-2)}.chartlet-series-3{fill:var(--chartlet-color-3)}.chartlet-series-4{fill:var(--chartlet-color-4)}";

/// Strokes for time layers that declare no color of their own; the same palette as the bars.
const LINE_SERIES_STYLE: &str = ".chartlet-line-series-1{stroke:var(--chartlet-color-1)}.chartlet-line-series-2{stroke:var(--chartlet-color-2)}.chartlet-line-series-3{stroke:var(--chartlet-color-3)}.chartlet-line-series-4{stroke:var(--chartlet-color-4)}";

/// CSS rules that hide series when their checkbox is deselected (HTML profile only).
/// Browsers that don't understand `:has()` ignore these rules; the chart stays fully visible.
/// Bands, modeled lines, reference lines and further palette markers of time charts and small
/// multiples. Only included when a chart uses one of them, so that existing charts keep their
/// bytes.
const LAYER_EXTRA_STYLE: &str = ".chartlet-band{stroke:none;fill-opacity:.18}.chartlet-band-series-1{fill:var(--chartlet-color-1)}.chartlet-band-series-2{fill:var(--chartlet-color-2)}.chartlet-band-series-3{fill:var(--chartlet-color-3)}.chartlet-band-series-4{fill:var(--chartlet-color-4)}.chartlet-hatch{stroke:none}.chartlet-hatch-line{stroke-width:1.2;opacity:.75}.chartlet-line-modeled{stroke-dasharray:7 5}.chartlet-point-series-2{fill:var(--chartlet-color-2)}.chartlet-point-series-3{fill:var(--chartlet-color-3)}.chartlet-point-series-4{fill:var(--chartlet-color-4)}.chartlet-legend-swatch.chartlet-series-1{fill:var(--chartlet-color-1)}.chartlet-legend-swatch.chartlet-series-2{fill:var(--chartlet-color-2)}.chartlet-legend-swatch.chartlet-series-3{fill:var(--chartlet-color-3)}.chartlet-legend-swatch.chartlet-series-4{fill:var(--chartlet-color-4)}.chartlet-rule{fill:none;stroke:var(--chartlet-zero);stroke-width:1.5;stroke-dasharray:5 4}.chartlet-rule-label{font-size:12px;font-weight:600;fill:var(--chartlet-text);paint-order:stroke;stroke:var(--chartlet-background);stroke-width:3px;stroke-linejoin:round}";

/// Small multiples draw thinner lines, because their plots are small.
const MULTIPLES_STYLE: &str = ".chartlet-multiples .chartlet-line{stroke-width:2}.chartlet-panel-title{font-size:13px;font-weight:650;fill:var(--chartlet-text)}";

/// Cells of a calendar without a value: outlined, not filled.
const CALENDAR_STYLE: &str =
    ".chartlet-calendar-empty{fill:none;stroke:var(--chartlet-grid);stroke-width:1}";

/// Range bars: a translucent span in the accent color, a hatch on top of a modeled one, and a
/// strong mark for the central value.
const RANGEBAR_STYLE: &str = ".chartlet-range{fill:var(--chartlet-accent);fill-opacity:.3;stroke:var(--chartlet-accent);stroke-width:1}.chartlet-range-hatch{stroke:none}.chartlet-hatch-line{stroke:var(--chartlet-accent);stroke-width:1.2;opacity:.75}.chartlet-range-mid{stroke:var(--chartlet-text);stroke-width:3}";

const FILTER_STYLE: &str = ".chartlet-wrapper{display:inline-block;max-width:100%}.chartlet-filter{border:none;padding:0;margin:0 0 12px 0}.chartlet-filter legend{font-size:14px;font-weight:650;margin-bottom:4px}.chartlet-filter label{display:inline-flex;align-items:center;min-height:44px;font-size:13px;margin-right:14px;cursor:pointer;white-space:nowrap}.chartlet-filter input{margin-right:4px}.chartlet-filter input:focus-visible{outline:2px solid #2563eb;outline-offset:2px}.chartlet-wrapper:has(.chartlet-filter input.series-0:not(:checked)) .chartlet-root [data-series=\"0\"]{display:none}.chartlet-wrapper:has(.chartlet-filter input.series-1:not(:checked)) .chartlet-root [data-series=\"1\"]{display:none}.chartlet-wrapper:has(.chartlet-filter input.series-2:not(:checked)) .chartlet-root [data-series=\"2\"]{display:none}.chartlet-wrapper:has(.chartlet-filter input.series-3:not(:checked)) .chartlet-root [data-series=\"3\"]{display:none}";

/// CSS rules for radio-selectable zoom panels. Only the panel whose radio is checked shows;
/// the first radio carries `checked`, so the first step is the default view.
const ZOOM_STYLE: &str = ".chartlet-zoom{border:none;padding:0;margin:0 0 12px 0}.chartlet-zoom legend{font-size:14px;font-weight:650;margin-bottom:4px}.chartlet-zoom label{display:inline-flex;align-items:center;min-height:44px;font-size:13px;margin-right:14px;cursor:pointer;white-space:nowrap}.chartlet-zoom input{margin-right:4px}.chartlet-zoom input:focus-visible{outline:2px solid #2563eb;outline-offset:2px}.chartlet-panel{display:none}.chartlet-wrapper:has(.chartlet-zoom input.zoom-0:checked) .chartlet-panel-0{display:block}.chartlet-wrapper:has(.chartlet-zoom input.zoom-1:checked) .chartlet-panel-1{display:block}.chartlet-wrapper:has(.chartlet-zoom input.zoom-2:checked) .chartlet-panel-2{display:block}.chartlet-wrapper:has(.chartlet-zoom input.zoom-3:checked) .chartlet-panel-3{display:block}";

/// Topic map styling. The label and its value stay fully opaque against the filled area, because
/// an opacity below one would take the text under the 4.5:1 contrast the palette is built for;
/// the decorative halo, depth lines and path points are the parts that fade.
///
/// `chartlet-topic-halo`'s stroke width is mirrored by `HALO_WIDTH` in the layout, which keeps
/// that much room free at the canvas edge.
///
/// `--chartlet-sea` is declared here rather than in the shared palette: only a topic map paints
/// a sea, and adding the property to every chart would change the bytes of all of them.
const TOPICMAP_STYLE: &str = ".chartlet-root{--chartlet-sea:#f2efe8;--chartlet-quiet:#fff}.chartlet-theme-dark{--chartlet-sea:#131a24;--chartlet-quiet:#1e2836}.chartlet-sea{fill:var(--chartlet-sea)}.chartlet-graticule{stroke:var(--chartlet-grid);stroke-width:1;opacity:.7}.chartlet-compass-ring{fill:var(--chartlet-background);stroke:var(--chartlet-grid);stroke-width:1}.chartlet-compass-needle{fill:var(--chartlet-muted);opacity:.55}.chartlet-compass-north{fill:var(--chartlet-accent)}.chartlet-compass-label{font-size:11px;font-weight:650;fill:var(--chartlet-muted)}.chartlet-cartouche{fill:var(--chartlet-background);stroke:var(--chartlet-grid);stroke-width:1}.chartlet-cartouche-frame{fill:none;stroke:var(--chartlet-grid);stroke-width:1}.chartlet-cartouche-heading{font-weight:650;fill:var(--chartlet-text)}.chartlet-cartouche-meta{fill:var(--chartlet-muted)}.chartlet-topic-area{fill:var(--chartlet-accent)}.chartlet-topic-halo{fill:none;stroke:var(--chartlet-accent);stroke-width:10;stroke-linejoin:round;opacity:.16}.chartlet-topic-band{fill:none;stroke:var(--chartlet-grid);stroke-width:1;stroke-dasharray:3 6;stroke-linejoin:round}.chartlet-topic-link{fill:none;stroke:var(--chartlet-grid);stroke-width:1.5;stroke-dasharray:4 5;stroke-linecap:round}.chartlet-topic-leader{stroke:var(--chartlet-zero);stroke-width:1}.chartlet-topic-point{fill:var(--chartlet-background);opacity:.6}.chartlet-topic-label{font-weight:650;fill:var(--chartlet-background)}.chartlet-topic-value{fill:var(--chartlet-background)}.chartlet-topic-outside{font-weight:650;fill:var(--chartlet-text)}.chartlet-topic-outside-value{fill:var(--chartlet-muted)}";

/// The landscape. It shares the sea, the graticule and the cartouche with the topic map, and adds
/// only what a nested map needs of its own.
const ATLAS_STYLE: &str = ".chartlet-root{--chartlet-sea:#f2efe8;--chartlet-quiet:#fff}.chartlet-sea{fill:var(--chartlet-sea)}.chartlet-theme-dark{--chartlet-sea:#131a24;--chartlet-quiet:#1e2836}[class^='chartlet-atlas-realm-']{fill:var(--chartlet-quiet);stroke:var(--chartlet-grid);stroke-width:1.2;stroke-linejoin:round}.chartlet-atlas-realm-1{opacity:.94}.chartlet-atlas-realm-2{opacity:.88}.chartlet-atlas-realm-3{opacity:.82}.chartlet-atlas-realm-4{opacity:.76}.chartlet-atlas-realm-5{opacity:.7}.chartlet-atlas-realm-6{opacity:.64}.chartlet-atlas-realm-7{opacity:.58}.chartlet-atlas-place{fill:var(--chartlet-text);opacity:.5}.chartlet-atlas-contour{fill:none;stroke:var(--chartlet-grid);stroke-width:.55;stroke-linejoin:round;opacity:.5}.chartlet-atlas-region{fill:none;stroke:var(--chartlet-grid);stroke-width:.8;stroke-linejoin:round;opacity:.75}.chartlet-atlas-coast{fill:none;stroke:var(--chartlet-accent);stroke-width:1.6;stroke-linejoin:round;opacity:.55}.chartlet-atlas-label{font-weight:650;fill:var(--chartlet-text)}.chartlet-atlas-realm-label{font-weight:650;fill:var(--chartlet-muted);letter-spacing:.08em;opacity:.55}";

/// Chrome for the area picker. The selection rules themselves are generated per chart, because
/// there is one of them per area.
const PICKER_STYLE: &str = ".chartlet-topic-picker{border:none;padding:0;margin:0 0 12px 0}.chartlet-topic-picker legend{font-size:14px;font-weight:650;margin-bottom:4px}.chartlet-topic-picker label{display:inline-flex;align-items:center;min-height:44px;font-size:13px;margin-right:14px;cursor:pointer;white-space:nowrap}.chartlet-topic-picker input{margin-right:4px}.chartlet-topic-picker input:focus-visible{outline:2px solid #2563eb;outline-offset:2px}";

/// The selection rules for one topic map.
///
/// Every rule is guarded by a `:has()` on a checked radio, which is what makes this degrade the
/// way it should: a browser that does not understand `:has()` drops all of them, leaving the map
/// in its plain state with every area in the accent colour and every panel visible. Nothing is
/// hidden that cannot be shown again.
fn picker_style(topics: usize) -> String {
    let mut style = String::from(PICKER_STYLE);
    // The quiet state applies only once something is actually selected.
    style.push_str(".chartlet-wrapper:has(.chartlet-topic-picker input:checked) .chartlet-topic-area{fill:var(--chartlet-quiet)}.chartlet-wrapper:has(.chartlet-topic-picker input:checked) .chartlet-topic-label,.chartlet-wrapper:has(.chartlet-topic-picker input:checked) .chartlet-topic-value{fill:var(--chartlet-text)}.chartlet-wrapper:has(.chartlet-topic-picker input:checked) .chartlet-topic-point{fill:var(--chartlet-muted)}");
    // A host page receives the wrapper as one block, so its own detail panels can only be
    // siblings of it — but a page that does place them inside should work too. Both forms are
    // written out rather than guessing which one the integration will use.
    style.push_str(".chartlet-wrapper:has(.chartlet-topic-picker input:checked) .chartlet-topic-panel,.chartlet-wrapper:has(.chartlet-topic-picker input:checked) ~ .chartlet-topic-panel{display:none}");
    for topic in 0..topics {
        write!(
            style,
            ".chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) .chartlet-topic-area.chartlet-topic-{topic}{{fill:var(--chartlet-accent)}}.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) .chartlet-topic-label.chartlet-topic-{topic},.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) .chartlet-topic-value.chartlet-topic-{topic}{{fill:var(--chartlet-background)}}.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) .chartlet-topic-point.chartlet-topic-{topic}{{fill:var(--chartlet-background)}}.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) .chartlet-topic-outside.chartlet-topic-{topic}{{fill:var(--chartlet-accent)}}.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) .chartlet-topic-panel-{topic},.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) ~ .chartlet-topic-panel-{topic}{{display:block}}"
        )
        .expect("writing to String cannot fail");
    }
    style
}

pub(crate) fn svg(scene: &Scene, spec: &ChartSpec, description: &str, id_prefix: &str) -> String {
    let title_id = format!("{id_prefix}-title");
    let description_id = format!("{id_prefix}-description");
    let has_series = spec.series.len() > 1;
    let is_time = matches!(spec.chart_type, ChartType::Time | ChartType::Multiples);
    let is_multiples = spec.chart_type == ChartType::Multiples;
    let is_diverging = matches!(spec.chart_type, ChartType::Stripes | ChartType::Calendar);
    let is_calendar = spec.chart_type == ChartType::Calendar;
    let is_rangebar = spec.chart_type == ChartType::Rangebar;
    let is_topicmap = spec.chart_type == ChartType::Topicmap;
    let is_atlas = spec.chart_type == ChartType::Atlas;
    let is_dark = spec.theme == Theme::Dark;
    let mut output = String::new();
    write!(
        output,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" role=\"img\" aria-labelledby=\"{title_id} {description_id}\" class=\"chartlet-root{}\">",
        scene.width,
        scene.height,
        scene.width,
        scene.height,
        match (is_dark, is_multiples) {
            (true, true) => " chartlet-theme-dark chartlet-multiples",
            (true, false) => " chartlet-theme-dark",
            (false, true) => " chartlet-multiples",
            (false, false) => "",
        }
    )
    .expect("writing to String cannot fail");
    write!(
        output,
        "<title id=\"{title_id}\">{}</title><desc id=\"{description_id}\">{}</desc><style>{STYLE}{}{}{}{}{}{}{}{}{}{}{}{}</style>",
        escape(&spec.title),
        escape(description),
        if is_dark { DARK_STYLE } else { "" },
        if has_series { SERIES_STYLE } else { "" },
        if has_series { FILTER_STYLE } else { "" },
        if is_time { LINE_SERIES_STYLE } else { "" },
        if is_time && needs_layer_extras(spec) {
            LAYER_EXTRA_STYLE
        } else {
            ""
        },
        if is_multiples { MULTIPLES_STYLE } else { "" },
        if is_diverging { crate::diverging::STYLE } else { "" },
        if is_calendar { CALENDAR_STYLE } else { "" },
        if is_rangebar { RANGEBAR_STYLE } else { "" },
        if is_topicmap { TOPICMAP_STYLE } else { "" },
        if is_atlas { ATLAS_STYLE } else { "" },
        layer_style(spec),
    )
    .expect("writing to String cannot fail");
    emit_hatches(spec, id_prefix, &mut output);

    // A dark chart cannot know the color of the page behind it, so it paints its own surface.
    if is_dark {
        write!(
            output,
            "<rect class=\"chartlet-background\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\"/>",
            scene.width, scene.height
        )
        .expect("writing to String cannot fail");
    }
    for element in &scene.elements {
        emit_element(element, id_prefix, &mut output);
    }
    output.push_str("</svg>");
    output
}

/// Whether a time chart uses anything beyond plain lines in the first palette colors.
fn needs_layer_extras(spec: &ChartSpec) -> bool {
    spec.chart_type == ChartType::Multiples
        || spec
            .layers()
            .any(|layer| layer.mark == Mark::Annotation || layer.modeled || layer.has_band())
        || spec.data_layers().any(|entry| {
            entry.layer.resolved_color().is_none()
                && (spec.palette_index(entry.pane, entry.layer) > 0
                    || spec.series_names().len() > 1)
        })
}

/// CSS rules for the colors a time layer declares itself, including the fill of its band. Only colors that passed the contract reach this point, and the contract admits no
/// character that could end a declaration, so the values are safe to interpolate.
fn layer_style(spec: &ChartSpec) -> String {
    let mut style = String::new();
    for (index, layer) in spec.layers().enumerate() {
        if let Some(color) = layer.resolved_color() {
            let color = crate::color::emit(color);
            write!(
                style,
                ".chartlet-style-{index}{{stroke:{color}}}.chartlet-style-{index}-swatch{{fill:{color}}}.chartlet-style-{index}-point{{fill:{color}}}"
            )
            .expect("writing to String cannot fail");
            if layer.has_band() {
                write!(
                    style,
                    ".chartlet-band.chartlet-style-{index}{{fill:{color};stroke:none}}"
                )
                .expect("writing to String cannot fail");
            }
        }
    }
    style
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
                    spec.palette_index(entry.pane, entry.layer) + 1
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

fn emit_element(element: &Element, id_prefix: &str, output: &mut String) {
    match element {
        Element::Circle(circle) => {
            open("circle", circle.series_index, output);
            write!(
                output,
                " cx=\"{}\" cy=\"{}\" r=\"{}\" class=\"{}{}{}\"",
                number(circle.cx),
                number(circle.cy),
                number(circle.radius),
                circle.class,
                point_class(circle.style_index),
                topic_class(circle.topic),
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
            emit_text(text, Some(*series_index), TextStyle::default(), output);
        }
        Element::StyledText(text, style) => emit_text(text, None, *style, output),
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
                " points=\"{}\"{} class=\"{}{}{}\"",
                points,
                hatch_fill(polyline.class, polyline.style_index, id_prefix),
                polyline.class,
                style_class(polyline.style_index),
                topic_class(polyline.topic),
            )
            .expect("write");
            close("polyline", polyline.tooltip.as_deref(), output);
        }
        Element::Text(text) => emit_text(text, None, TextStyle::default(), output),
    }
}

fn emit_text(
    text: &crate::scene::Text,
    series_index: Option<usize>,
    style: TextStyle,
    output: &mut String,
) {
    open("text", series_index, output);
    let size = style.size.map_or_else(String::new, |size| {
        format!(" font-size=\"{}\"", number(size))
    });
    write!(
        output,
        " x=\"{}\" y=\"{}\"{size} text-anchor=\"{}\" class=\"{}{}\">{}</text>",
        number(text.x),
        number(text.y),
        anchor(text.anchor),
        text.class,
        topic_class(style.topic),
        escape(&text.content)
    )
    .expect("write");
}

/// The class that ties an element to one topic map area, with the leading space, or nothing at
/// all when it belongs to no particular area.
fn topic_class(topic: Option<usize>) -> String {
    topic.map_or_else(String::new, |index| format!(" chartlet-topic-{index}"))
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

pub(crate) fn html(svg: &str, spec: &ChartSpec, table_mode: TableMode, id_prefix: &str) -> String {
    html_document(
        &[(String::new(), svg.to_owned())],
        spec,
        table_mode,
        id_prefix,
        false,
    )
}

pub(crate) fn html_zoom(
    panels: &[(String, String)],
    spec: &ChartSpec,
    table_mode: TableMode,
    id_prefix: &str,
) -> String {
    html_document(panels, spec, table_mode, id_prefix, true)
}

/// Builds the HTML figure. With one panel and no zoom name this is the plain figure; with
/// several panels it adds a radio group that switches between pre-computed variants.
fn html_document(
    panels: &[(String, String)],
    spec: &ChartSpec,
    table_mode: TableMode,
    id_prefix: &str,
    zoomable: bool,
) -> String {
    let mut output = String::new();
    let has_series = spec.series.len() > 1;
    let zoom = zoomable && panels.len() > 1;
    // One area cannot be picked from, and islands are not what the map is about.
    let picker = spec
        .topicmap
        .as_ref()
        .map(|topicmap| topicmap.topics.len())
        .filter(|topics| *topics > 1);
    if has_series || zoom || picker.is_some() {
        output.push_str("<div class=\"chartlet-wrapper\">");
    }
    if let Some(topics) = picker {
        write!(output, "<style>{}</style>", picker_style(topics)).expect("write");
        output.push_str("<fieldset class=\"chartlet-topic-picker\"><legend>Area</legend>");
        let names = spec
            .topicmap
            .as_ref()
            .expect("a picker means there is a topic map")
            .topics
            .iter()
            .map(|topic| topic.label.as_str());
        for (index, label) in names.enumerate() {
            let checked = if index == 0 { " checked" } else { "" };
            write!(
                output,
                "<label><input type=\"radio\" name=\"topic-{id_prefix}\" class=\"topic-{index}\"{checked}> {}</label>",
                escape(label)
            )
            .expect("write");
        }
        output.push_str("</fieldset>");
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
        write!(
            output,
            "<style>{ZOOM_STYLE}</style><fieldset class=\"chartlet-zoom\"><legend>View</legend>"
        )
        .expect("write");
        for (i, (label, _)) in panels.iter().enumerate() {
            let checked = if i == 0 { " checked" } else { "" };
            write!(
                output,
                "<label><input type=\"radio\" name=\"zoom-{id_prefix}\" class=\"zoom-{i}\"{checked}> {}</label>",
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
    if has_series || zoom || picker.is_some() {
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
    let dataset = spec.table_dataset();
    write!(
        output,
        "<table><caption>Data for {}</caption><thead><tr><th scope=\"col\">{}</th>",
        escape(&spec.title),
        match spec.chart_type {
            ChartType::Time | ChartType::Multiples => "Time",
            ChartType::Topicmap => "Topic",
            ChartType::Atlas => "Region",
            ChartType::Stripes => "Year",
            ChartType::Calendar => "Date",
            ChartType::Bar | ChartType::Line | ChartType::Rangebar => "Category",
        }
    )
    .expect("write");
    for series in &dataset.series {
        write!(
            output,
            "<th scope=\"col\">{}</th>",
            escape(series.name.as_deref().unwrap_or("Value"))
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
                    |value| {
                        escape(&format_value(
                            value,
                            series.format.unwrap_or_else(|| spec.value_format()),
                        ))
                    },
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
