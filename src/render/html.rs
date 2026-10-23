//! The HTML figure around one or more SVGs: the series filter, zoom steps, topic picker, focus
//! of a diagram and mobile variant, without a script, and the page of all views of a model.

use std::fmt::Write;

use super::{escape, number, table::render_data_table};
use crate::{TableMode, spec::ChartSpec};

/// CSS rules for radio-selectable zoom panels. Only the panel whose radio is checked shows;
/// the first radio carries `checked`, so the first step is the default view.
const ZOOM_STYLE: &str = ".chartlet-zoom{border:none;padding:0;margin:0 0 12px 0}.chartlet-zoom legend{font-size:14px;font-weight:650;margin-bottom:4px}.chartlet-zoom label{display:inline-flex;align-items:center;min-height:44px;font-size:13px;margin-right:14px;cursor:pointer;white-space:nowrap}.chartlet-zoom input{margin-right:4px}.chartlet-zoom input:focus-visible{outline:2px solid #2563eb;outline-offset:2px}.chartlet-panel{display:none}.chartlet-wrapper:has(.chartlet-zoom input.zoom-0:checked) .chartlet-panel-0{display:block}.chartlet-wrapper:has(.chartlet-zoom input.zoom-1:checked) .chartlet-panel-1{display:block}.chartlet-wrapper:has(.chartlet-zoom input.zoom-2:checked) .chartlet-panel-2{display:block}.chartlet-wrapper:has(.chartlet-zoom input.zoom-3:checked) .chartlet-panel-3{display:block}";

/// Chrome for the area picker. The selection rules themselves are generated per chart, because
/// there is one of them per area.
const PICKER_STYLE: &str = ".chartlet-topic-picker{border:none;padding:0;margin:0 0 12px 0}.chartlet-topic-picker legend{font-size:14px;font-weight:650;margin-bottom:4px}.chartlet-topic-picker label{display:inline-flex;align-items:center;min-height:44px;font-size:13px;margin-right:14px;cursor:pointer;white-space:nowrap}.chartlet-topic-picker input{margin-right:4px}.chartlet-topic-picker input:focus-visible{outline:2px solid #2563eb;outline-offset:2px}";

/// The selection rules for one topic map.
///
/// Every rule is guarded by a `:has()` on a checked radio, which is what makes this degrade the
/// way it should: a browser that does not understand `:has()` drops all of them, leaving the map
/// in its plain state with every area in the accent colour and every panel visible. Nothing is
/// hidden that cannot be shown again.
fn picker_style(topics: usize, roots: &[&str]) -> String {
    let mut style = String::from(PICKER_STYLE);
    // The quiet state applies only once something is actually selected. The rules for the map
    // name each SVG's root ID, because the chart's own rules are scoped to it and would
    // otherwise outrank them.
    for root in roots {
        write!(
            style,
            ".chartlet-wrapper:has(.chartlet-topic-picker input:checked) #{root} .chartlet-topic-area{{fill:var(--chartlet-quiet)}}.chartlet-wrapper:has(.chartlet-topic-picker input:checked) #{root} .chartlet-topic-label,.chartlet-wrapper:has(.chartlet-topic-picker input:checked) #{root} .chartlet-topic-value{{fill:var(--chartlet-text)}}.chartlet-wrapper:has(.chartlet-topic-picker input:checked) #{root} .chartlet-topic-point{{fill:var(--chartlet-muted)}}"
        )
        .expect("writing to String cannot fail");
    }
    // A host page receives the wrapper as one block, so its own detail panels can only be
    // siblings of it — but a page that does place them inside should work too. Both forms are
    // written out rather than guessing which one the integration will use.
    style.push_str(".chartlet-wrapper:has(.chartlet-topic-picker input:checked) .chartlet-topic-panel,.chartlet-wrapper:has(.chartlet-topic-picker input:checked) ~ .chartlet-topic-panel{display:none}");
    for topic in 0..topics {
        for root in roots {
            write!(
                style,
                ".chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) #{root} .chartlet-topic-area.chartlet-topic-{topic}{{fill:var(--chartlet-accent)}}.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) #{root} .chartlet-topic-label.chartlet-topic-{topic},.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) #{root} .chartlet-topic-value.chartlet-topic-{topic}{{fill:var(--chartlet-background)}}.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) #{root} .chartlet-topic-point.chartlet-topic-{topic}{{fill:var(--chartlet-background)}}.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) #{root} .chartlet-topic-outside.chartlet-topic-{topic}{{fill:var(--chartlet-accent)}}"
            )
            .expect("writing to String cannot fail");
        }
        write!(
            style,
            ".chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) .chartlet-topic-panel-{topic},.chartlet-wrapper:has(.chartlet-topic-picker input.topic-{topic}:checked) ~ .chartlet-topic-panel-{topic}{{display:block}}"
        )
        .expect("writing to String cannot fail");
    }
    style
}

/// The root IDs of every SVG in the panels, desktop and mobile.
fn panel_roots(panels: &[Panel]) -> Vec<&str> {
    panels
        .iter()
        .flat_map(|panel| std::iter::once(&panel.svg).chain(panel.mobile.as_ref()))
        .filter_map(|svg| root_id(svg))
        .collect()
}

/// The ID of an SVG's root element, which [`svg`](super::svg::svg) always writes as the root's first `id`.
fn root_id(svg: &str) -> Option<&str> {
    let start = svg.find(" id=\"")? + 5;
    let end = start + svg[start..].find('"')?;
    Some(&svg[start..end])
}

/// One chart of the HTML figure: a zoom step, or the only chart. `mobile` is its mobile variant,
/// when the specification has one.
pub(crate) struct Panel {
    pub label: String,
    pub svg: String,
    pub mobile: Option<String>,
    /// For a diagram, where its nodes stand in each layout, desktop first: the HTML profile lays
    /// a focus link over every node. Empty for every other chart.
    pub stages: Vec<Stage>,
}

/// The nodes of one laid-out diagram: the SVG's root ID, its size, and where each node stands.
#[derive(Debug, Clone)]
pub(crate) struct Stage {
    pub root: String,
    pub width: u32,
    pub height: u32,
    pub hotspots: Vec<crate::scene::Hotspot>,
}

pub(crate) fn html(
    panel: Panel,
    spec: &ChartSpec,
    table_mode: TableMode,
    id_prefix: &str,
    hooks: bool,
) -> String {
    html_document(&[panel], spec, table_mode, id_prefix, false, hooks)
}

pub(crate) fn html_zoom(
    panels: &[Panel],
    spec: &ChartSpec,
    table_mode: TableMode,
    id_prefix: &str,
    hooks: bool,
) -> String {
    html_document(panels, spec, table_mode, id_prefix, true, hooks)
}

/// One view of an architecture model on the page of all its views: the ID prefix of its figure,
/// which its section's ID extends, its title and its HTML figure.
pub(crate) struct ViewSection {
    pub id: String,
    pub title: String,
    pub content: String,
}

/// The page of all views: links stay visible and large enough to hit, a link target keeps a
/// margin above it, and in print every view after the first starts a page of its own.
const VIEWS_STYLE: &str = ".chartlet-views-nav ul{margin:0 0 16px 0;padding-left:20px}.chartlet-views-nav li{margin:4px 0}.chartlet-views-nav a{display:inline-block;min-height:24px;padding:2px 0}.chartlet-views-nav a:focus-visible{outline:2px solid #2563eb;outline-offset:2px}.chartlet-view{margin:0 0 32px 0;scroll-margin-top:16px}@media print{.chartlet-view+.chartlet-view{break-before:page}}";

/// Every view of a model on one page, without a script: the model's title, a navigation of links
/// to the views in the order of the page, and each view as a section headed by its title, with
/// its figure.
pub(crate) fn html_views(sections: &[ViewSection], spec: &ChartSpec, id_prefix: &str) -> String {
    let words = spec.locale.words();
    let mut output = String::new();
    write!(
        output,
        "<div class=\"chartlet-views\" id=\"{id_prefix}\"><style>{VIEWS_STYLE}</style><h2 id=\"{id_prefix}-title\">{}</h2><nav class=\"chartlet-views-nav\" aria-label=\"{}\"><ul>",
        escape(&spec.title),
        words.views
    )
    .expect("write");
    for section in sections {
        write!(
            output,
            "<li><a href=\"#{}-view\">{}</a></li>",
            section.id,
            escape(&section.title)
        )
        .expect("write");
    }
    output.push_str("</ul></nav>");
    for section in sections {
        write!(
            output,
            "<section class=\"chartlet-view\" id=\"{id}-view\" aria-labelledby=\"{id}-view-title\"><h3 id=\"{id}-view-title\">{}</h3>{}</section>",
            escape(&section.title),
            section.content,
            id = section.id,
        )
        .expect("write");
    }
    output.push_str("</div>");
    output
}

/// Switches between the chart and its mobile variant by the width of the wrapper. The mobile
/// variant is hidden unless the container query applies, so a browser without container queries
/// keeps showing the chart at its full size. `display:none` also takes the hidden variant out of
/// the accessibility tree. The rules carry the breakpoint in their selector, so charts with
/// different breakpoints on one page do not switch each other.
fn responsive_style(breakpoint: u32) -> String {
    format!(
        ".chartlet-wrapper.chartlet-responsive{{display:block;width:100%;container-type:inline-size}}.chartlet-variant-mobile{{display:none}}@container (max-width:{}px){{.chartlet-bp-{breakpoint} .chartlet-variant-desktop{{display:none}}.chartlet-bp-{breakpoint} .chartlet-variant-mobile{{display:block}}}}",
        breakpoint - 1
    )
}

/// Builds the HTML figure. With one panel and no zoom name this is the plain figure; with
/// several panels it adds a radio group that switches between pre-computed variants.
/// The radio group that picks an area of a topic map, with the rules that restyle the map.
fn emit_topic_picker(
    topics: usize,
    panels: &[Panel],
    spec: &ChartSpec,
    id_prefix: &str,
    output: &mut String,
) {
    write!(
        output,
        "<style>{}</style>",
        picker_style(topics, &panel_roots(panels))
    )
    .expect("write");
    write!(
        output,
        "<fieldset class=\"chartlet-topic-picker\"><legend>{}</legend>",
        spec.locale.words().area
    )
    .expect("write");
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

fn html_document(
    panels: &[Panel],
    spec: &ChartSpec,
    table_mode: TableMode,
    id_prefix: &str,
    zoomable: bool,
    hooks: bool,
) -> String {
    let mut output = String::new();
    // A stack shows every series as part of a whole; hiding one would leave a gap in it.
    let has_series = spec.series.len() > 1 && spec.stack.is_none();
    let zoom = zoomable && panels.len() > 1;
    let breakpoint = spec.mobile.as_ref().map(|mobile| mobile.breakpoint);
    // One area cannot be picked from, and islands are not what the map is about.
    let picker = spec
        .topicmap
        .as_ref()
        .map(|topicmap| topicmap.topics.len())
        .filter(|topics| *topics > 1);
    let focus = !zoom && panels.first().is_some_and(|panel| !panel.stages.is_empty());
    let wrapped = has_series || zoom || picker.is_some() || breakpoint.is_some() || focus;
    if let Some(breakpoint) = breakpoint {
        write!(
            output,
            "<div class=\"chartlet-wrapper chartlet-responsive chartlet-bp-{breakpoint}\"><style>{}</style>",
            responsive_style(breakpoint)
        )
        .expect("write");
    } else if wrapped {
        output.push_str("<div class=\"chartlet-wrapper\">");
    }
    if let Some(topics) = picker {
        emit_topic_picker(topics, panels, spec, id_prefix, &mut output);
    }
    if focus {
        emit_focus_controls(&panels[0], spec, id_prefix, &mut output);
    }
    if has_series {
        write!(
            output,
            "<fieldset class=\"chartlet-filter\"><legend>{}</legend>",
            spec.locale.words().series
        )
        .expect("write");
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
            "<style>{ZOOM_STYLE}</style><fieldset class=\"chartlet-zoom\"><legend>{}</legend>",
            spec.locale.words().view
        )
        .expect("write");
        for (i, Panel { label, .. }) in panels.iter().enumerate() {
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
    // `data-viz` and `data-viz-text` follow the display-mode convention of barrierlab: a chart
    // figure whose text layer is its data table.
    output.push_str("<figure class=\"chartlet-figure\" data-viz=\"chart\">");
    write!(output, "<figcaption>{}</figcaption>", escape(&spec.title)).expect("write");
    emit_panels(panels, zoom, id_prefix, &mut output);
    if let Some(source) = &spec.source {
        write!(
            output,
            "<p class=\"chartlet-source\">{}: {}</p>",
            spec.locale.words().source,
            escape(source)
        )
        .expect("write");
    }
    render_data_table(&mut output, spec, table_mode, hooks.then_some(id_prefix));
    output.push_str("</figure>");
    if wrapped {
        output.push_str("</div>");
    }
    output
}

/// The charts of the figure: one per zoom step, each with its mobile variant beside it.
fn emit_panels(panels: &[Panel], zoom: bool, id_prefix: &str, output: &mut String) {
    for (i, panel) in panels.iter().enumerate() {
        let desktop = staged(&panel.svg, panel.stages.first(), id_prefix);
        let variants = match &panel.mobile {
            Some(mobile) => format!(
                "<div class=\"chartlet-variant-desktop\">{desktop}</div><div class=\"chartlet-variant-mobile\">{}</div>",
                staged(mobile, panel.stages.get(1), id_prefix)
            ),
            None => desktop,
        };
        if zoom {
            write!(
                output,
                "<div class=\"chartlet-panel chartlet-panel-{i}\">{variants}</div>"
            )
            .expect("write");
        } else {
            output.push_str(&variants);
        }
    }
}

/// The focus of a diagram, without a script: a group of radio buttons, one per node and one for
/// all of them, in a disclosure before the figure, and over every node a label of its button, so
/// that a click on the node checks it. While a node is checked, CSS keeps it, its neighbours and
/// the edges between them as they are and fades the rest. The buttons are the controls, with
/// their names and the usual keyboard; the labels over the nodes are only a larger place to click
/// them. The SVG keeps its `role="img"`: the labels are HTML beside it, not content of it.
const FOCUS_STYLE: &str = ".chartlet-stage{position:relative;display:inline-block;max-width:100%;vertical-align:top}.chartlet-stage>svg{display:block}.chartlet-hotspot{position:absolute;border-radius:8px;cursor:pointer}.chartlet-focus{margin:0 0 12px 0;font-size:13px}.chartlet-focus summary{cursor:pointer;min-height:24px}.chartlet-focus fieldset{border:none;padding:0;margin:4px 0 0 0}.chartlet-focus label{display:inline-flex;align-items:center;min-height:44px;margin-right:14px;cursor:pointer;white-space:nowrap}.chartlet-focus input{margin-right:4px}.chartlet-focus input:focus-visible{outline:2px solid #2563eb;outline-offset:2px}";

/// The stylesheet and the radio buttons of the focus, before the figure.
fn emit_focus_controls(panel: &Panel, spec: &ChartSpec, id_prefix: &str, output: &mut String) {
    let words = spec.locale.words();
    let mut style = String::from(FOCUS_STYLE);
    let nodes = &panel.stages[0].hotspots;
    for stage in &panel.stages {
        let root = &stage.root;
        write!(
            style,
            ".chartlet-wrapper:has(.chartlet-focus-node:checked) #{root} .chartlet-f{{opacity:.18}}"
        )
        .expect("write");
        for hotspot in &stage.hotspots {
            write!(
                style,
                "#{}{{left:{}%;top:{}%;width:{}%;height:{}%}}",
                hotspot_id(root, hotspot.node),
                number(hotspot.x / f64::from(stage.width) * 100.0),
                number(hotspot.y / f64::from(stage.height) * 100.0),
                number(hotspot.width / f64::from(stage.width) * 100.0),
                number(hotspot.height / f64::from(stage.height) * 100.0),
            )
            .expect("write");
        }
        for hotspot in nodes {
            let node = hotspot.node;
            write!(
                style,
                ".chartlet-wrapper:has(#{id_prefix}-focus-{node}:checked) #{root} .chartlet-f-{node}{{opacity:1}}.chartlet-wrapper:has(#{id_prefix}-focus-{node}:checked) #{root} .chartlet-n-{node} .chartlet-flow-node,.chartlet-wrapper:has(#{id_prefix}-focus-{node}:checked) #{root} .chartlet-n-{node} .chartlet-seq-box{{stroke-width:3}}"
            )
            .expect("write");
        }
    }
    write!(
        output,
        "<style>{style}</style><details class=\"chartlet-focus\"><summary>{}</summary><fieldset><label><input type=\"radio\" name=\"{id_prefix}-focus\" checked> {}</label>",
        words.focus, words.show_all
    )
    .expect("write");
    for hotspot in nodes {
        write!(
            output,
            "<label><input type=\"radio\" name=\"{id_prefix}-focus\" class=\"chartlet-focus-node\" id=\"{id_prefix}-focus-{}\"> {}</label>",
            hotspot.node,
            escape(&hotspot.label)
        )
        .expect("write");
    }
    output.push_str("</fieldset></details>");
}

fn hotspot_id(root: &str, node: usize) -> String {
    format!("{root}-hs-{node}")
}

/// An SVG with, over every node of its stage, a label of that node's focus button; or the SVG
/// alone without a stage.
fn staged(svg: &str, stage: Option<&Stage>, id_prefix: &str) -> String {
    let Some(stage) = stage else {
        return svg.to_owned();
    };
    let mut output = format!("<div class=\"chartlet-stage\">{svg}");
    for hotspot in &stage.hotspots {
        write!(
            output,
            "<label class=\"chartlet-hotspot\" id=\"{}\" for=\"{id_prefix}-focus-{}\"></label>",
            hotspot_id(&stage.root, hotspot.node),
            hotspot.node,
        )
        .expect("write");
    }
    output.push_str("</div>");
    output
}
