use std::fmt::Write;

use crate::{
    DataTable, Styles, TableMode,
    layout::format_value,
    scene::{Element, Hook, Scene, TextAnchor, TextStyle},
    spec::{ChartSpec, ChartType, Dash, Mark, Stroke, Theme},
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
const LINE_SERIES_STYLE: &str = ".chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}.chartlet-line-series-1{stroke:var(--chartlet-color-1)}.chartlet-line-series-2{stroke:var(--chartlet-color-2)}.chartlet-line-series-3{stroke:var(--chartlet-color-3)}.chartlet-line-series-4{stroke:var(--chartlet-color-4)}.chartlet-point-series-1{fill:var(--chartlet-color-1)}.chartlet-hit{fill:transparent;stroke:none}";

/// CSS rules that hide series when their checkbox is deselected (HTML profile only).
/// Browsers that don't understand `:has()` ignore these rules; the chart stays fully visible.
/// Bands, modeled lines, reference lines and further palette markers of time charts and small
/// multiples. Only included when a chart uses one of them, so that existing charts keep their
/// bytes.
/// Outlined series of a bar chart with `patterns`: the background inside, the series color
/// around it.
const OUTLINE_STYLE: &str = ".chartlet-outline{fill:var(--chartlet-background);stroke-width:2}.chartlet-series-2.chartlet-outline{stroke:var(--chartlet-color-2)}.chartlet-series-4.chartlet-outline{stroke:var(--chartlet-color-4)}";
/// The smaller title of a narrow chart, such as a mobile variant.
const SMALL_TITLE_STYLE: &str = ".chartlet-title-small{font-size:18px}";
/// The reference lines of a bar chart, drawn like the reference lines of a time chart in
/// [`LAYER_EXTRA_STYLE`].
const REFERENCE_STYLE: &str = ".chartlet-rule{fill:none;stroke:var(--chartlet-zero);stroke-width:1.5;stroke-dasharray:5 4}.chartlet-rule-label{font-size:12px;font-weight:600;fill:var(--chartlet-text);paint-order:stroke;stroke:var(--chartlet-background);stroke-width:3px;stroke-linejoin:round}";
const LAYER_EXTRA_STYLE: &str = ".chartlet-band{stroke:none;fill-opacity:.18}.chartlet-band-series-1{fill:var(--chartlet-color-1)}.chartlet-band-series-2{fill:var(--chartlet-color-2)}.chartlet-band-series-3{fill:var(--chartlet-color-3)}.chartlet-band-series-4{fill:var(--chartlet-color-4)}.chartlet-hatch{stroke:none}.chartlet-hatch-line{stroke-width:1.2;opacity:.75}.chartlet-line-modeled{stroke-dasharray:7 5}.chartlet-line-thin{stroke-width:1}.chartlet-point-series-2{fill:var(--chartlet-color-2)}.chartlet-point-series-3{fill:var(--chartlet-color-3)}.chartlet-point-series-4{fill:var(--chartlet-color-4)}.chartlet-rule{fill:none;stroke:var(--chartlet-zero);stroke-width:1.5;stroke-dasharray:5 4}.chartlet-rule-label{font-size:12px;font-weight:600;fill:var(--chartlet-text);paint-order:stroke;stroke:var(--chartlet-background);stroke-width:3px;stroke-linejoin:round}";

/// Areas, line patterns other than the modeled dash, and bold lines. Only included when a chart
/// uses one of them, for the same reason as [`LAYER_EXTRA_STYLE`].
const MARK_EXTRA_STYLE: &str = ".chartlet-area{stroke:none;fill-opacity:.18}.chartlet-line-dashed{stroke-dasharray:7 5}.chartlet-line-dotted{stroke-dasharray:0 7}.chartlet-line-thin.chartlet-line-dotted{stroke-dasharray:1 3}.chartlet-line-bold{stroke-width:4.5}.chartlet-multiples .chartlet-line-bold{stroke-width:3}";
/// Medium lines, in the charts that draw one.
const MEDIUM_LINE_STYLE: &str = ".chartlet-line-medium{stroke-width:2}.chartlet-multiples .chartlet-line-medium{stroke-width:1.5}";

/// Zones and point markers of time charts. Only included when a chart uses one of them, for the
/// same reason as [`LAYER_EXTRA_STYLE`]. A marker is ringed in the background color so that it
/// stands apart from the line it sits on.
const ANNOTATION_EXTRA_STYLE: &str = ".chartlet-zone{fill:var(--chartlet-zero);fill-opacity:.14;stroke:none}.chartlet-marker{fill:var(--chartlet-text);stroke:var(--chartlet-background);stroke-width:1.5;stroke-linejoin:round}";

/// Candlesticks. A rising candle is hollow and a falling one filled, so the direction does not
/// rest on the two colors; both keep at least 4.5:1 against the background of their theme, and
/// they differ in lightness too (about 1.9:1 light, 3.1:1 dark), which grayscale print and
/// color-vision deficiencies keep. Only
/// included when a chart draws candles, for the same reason as [`LAYER_EXTRA_STYLE`].
const OHLC_STYLE: &str = ".chartlet-root{--chartlet-rise:#0a4f49;--chartlet-fall:#d03a0f}.chartlet-theme-dark{--chartlet-rise:#99f6e4;--chartlet-fall:#e5484d}.chartlet-wick{stroke-width:1.5}.chartlet-wick-rise{stroke:var(--chartlet-rise)}.chartlet-wick-fall{stroke:var(--chartlet-fall)}.chartlet-candle{stroke-width:1.5}.chartlet-candle-rise{fill:var(--chartlet-background);stroke:var(--chartlet-rise)}.chartlet-candle-fall{fill:var(--chartlet-fall);stroke:var(--chartlet-fall)}";

/// Small multiples draw thinner lines, because their plots are small.
const MULTIPLES_STYLE: &str = ".chartlet-multiples .chartlet-line{stroke-width:2}.chartlet-panel-title{font-size:13px;font-weight:650;fill:var(--chartlet-text)}";
/// The note under a panel title, in small multiples that have one.
const PANEL_NOTE_STYLE: &str = ".chartlet-panel-note{font-size:12px;fill:var(--chartlet-muted)}.chartlet-panel-note-strong{fill:var(--chartlet-text);font-weight:600}";

/// Cells of a calendar without a value: outlined, not filled.
const CALENDAR_STYLE: &str =
    ".chartlet-calendar-empty{fill:none;stroke:var(--chartlet-grid);stroke-width:1}";

/// Range bars: a translucent span in the accent color, a hatch on top of a modeled one, and a
/// strong mark for the central value.
const RANGEBAR_STYLE: &str = ".chartlet-range{fill:var(--chartlet-accent);fill-opacity:.3;stroke:var(--chartlet-accent);stroke-width:1}.chartlet-range-hatch{stroke:none}.chartlet-hatch-line{stroke:var(--chartlet-accent);stroke-width:1.2;opacity:.75}.chartlet-range-mid{stroke:var(--chartlet-text);stroke-width:3}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Range bars in groups: each group's spans in its palette color.
const RANGE_GROUP_STYLE: &str = ".chartlet-range-series-1{fill:var(--chartlet-color-1);stroke:var(--chartlet-color-1)}.chartlet-range-series-2{fill:var(--chartlet-color-2);stroke:var(--chartlet-color-2)}.chartlet-range-series-3{fill:var(--chartlet-color-3);stroke:var(--chartlet-color-3)}.chartlet-range-series-4{fill:var(--chartlet-color-4);stroke:var(--chartlet-color-4)}";

/// What every diagram shares: the colors of its roles, shadows, chips and number badges. Each
/// role is a stroke and a light fill, at least 3:1 against the background in both themes; the
/// role never rests on its color alone, the shape of a step says it too. The custom properties
/// are declared here rather than in the shared palette for the same reason as `--chartlet-sea`.
const DIAGRAM_STYLE: &str = ".chartlet-root{--chartlet-node:#eef3fd;--chartlet-lane:#f7f9fc;--chartlet-lane-head:#eef1f6;--chartlet-shadow:rgba(16,24,40,.09);--chartlet-role-blue:#2563eb;--chartlet-role-blue-fill:#eef3fd;--chartlet-role-green:#15803d;--chartlet-role-green-fill:#ecfdf3;--chartlet-role-amber:#b45309;--chartlet-role-amber-fill:#fff7e8;--chartlet-role-teal:#0f766e;--chartlet-role-teal-fill:#e9faf7;--chartlet-role-violet:#6d28d9;--chartlet-role-violet-fill:#f3efff;--chartlet-role-gray:#667085;--chartlet-role-gray-fill:#f7f8fa;--chartlet-role-red:#b42318;--chartlet-role-red-fill:#fef3f2}.chartlet-theme-dark{--chartlet-node:#1a2436;--chartlet-lane:#121925;--chartlet-lane-head:#18212f;--chartlet-shadow:rgba(0,0,0,.45);--chartlet-role-blue:#7ea6ff;--chartlet-role-blue-fill:#1a2436;--chartlet-role-green:#4ade80;--chartlet-role-green-fill:#132a1d;--chartlet-role-amber:#fbbf24;--chartlet-role-amber-fill:#2b2210;--chartlet-role-teal:#2dd4bf;--chartlet-role-teal-fill:#0f2a27;--chartlet-role-violet:#b69cff;--chartlet-role-violet-fill:#231c38;--chartlet-role-gray:#9aa7bd;--chartlet-role-gray-fill:#161d29;--chartlet-role-red:#f97066;--chartlet-role-red-fill:#2d1616}.chartlet-role-blue{fill:var(--chartlet-role-blue-fill);stroke:var(--chartlet-role-blue)}.chartlet-role-green{fill:var(--chartlet-role-green-fill);stroke:var(--chartlet-role-green)}.chartlet-role-amber{fill:var(--chartlet-role-amber-fill);stroke:var(--chartlet-role-amber)}.chartlet-role-teal{fill:var(--chartlet-role-teal-fill);stroke:var(--chartlet-role-teal)}.chartlet-role-violet{fill:var(--chartlet-role-violet-fill);stroke:var(--chartlet-role-violet)}.chartlet-role-gray{fill:var(--chartlet-role-gray-fill);stroke:var(--chartlet-role-gray)}.chartlet-role-red{fill:var(--chartlet-role-red-fill);stroke:var(--chartlet-role-red)}.chartlet-diagram-rim{fill:none;stroke:var(--chartlet-role-teal);stroke-width:1.5}.chartlet-diagram-shadow{fill:var(--chartlet-shadow);stroke:none;rx:7px}.chartlet-diagram-chip{fill:var(--chartlet-background);stroke:var(--chartlet-grid);stroke-width:1;rx:4px}.chartlet-diagram-chip-text{font-size:12px;fill:var(--chartlet-text)}.chartlet-diagram-badge{fill:var(--chartlet-accent);stroke:var(--chartlet-background);stroke-width:2}.chartlet-diagram-badge-text{font-size:10px;font-weight:700;fill:var(--chartlet-background)}";

/// Sequence diagrams, on top of [`DIAGRAM_STYLE`]. Branch conditions are ringed in the background
/// color, so that they stay legible where they cross a lifeline.
const SEQUENCE_STYLE: &str = ".chartlet-seq-box{stroke-width:1.5;stroke-linejoin:round;rx:7px}.chartlet-seq-external{stroke-dasharray:5 3}.chartlet-seq-hit{fill:transparent;stroke:none}.chartlet-seq-actor{stroke:var(--chartlet-text);stroke-width:1.5;stroke-linecap:round}.chartlet-seq-actor-head{fill:var(--chartlet-background);stroke:var(--chartlet-text);stroke-width:1.5}.chartlet-seq-label{font-size:13px;font-weight:600;fill:var(--chartlet-text)}.chartlet-seq-sublabel{font-size:11px;fill:var(--chartlet-muted)}.chartlet-seq-lifeline{stroke:var(--chartlet-zero);stroke-width:1;stroke-dasharray:3 4}.chartlet-seq-activation{fill:var(--chartlet-node);stroke:var(--chartlet-accent);stroke-width:1;rx:2px}.chartlet-seq-message{fill:none;stroke:var(--chartlet-zero);stroke-width:1.4;stroke-linejoin:round}.chartlet-seq-reply{stroke-dasharray:5 4}.chartlet-seq-head{fill:var(--chartlet-zero);stroke:var(--chartlet-zero);stroke-width:1;stroke-linejoin:round}.chartlet-seq-head-open{fill:none;stroke:var(--chartlet-zero);stroke-width:1.4;stroke-linecap:round;stroke-linejoin:round}.chartlet-seq-frame{fill:none;stroke:var(--chartlet-zero);stroke-width:1;rx:6px}.chartlet-seq-tab{fill:var(--chartlet-node);stroke:var(--chartlet-zero);stroke-width:1;rx:4px}.chartlet-seq-tag{font-size:11px;font-weight:650;fill:var(--chartlet-text)}.chartlet-seq-guard{font-size:11px;fill:var(--chartlet-muted);paint-order:stroke;stroke:var(--chartlet-background);stroke-width:3px;stroke-linejoin:round}.chartlet-seq-branch{stroke:var(--chartlet-zero);stroke-width:1;stroke-dasharray:5 4}";

/// Flow charts, on top of [`DIAGRAM_STYLE`]: `--chartlet-lane` fills every other lane, and the
/// main path is drawn in the accent color and thicker, so that it stands out by weight as well as
/// by color.
const FLOW_STYLE: &str = ".chartlet-flow-node{stroke-width:1.5;stroke-linejoin:round;rx:7px}.chartlet-flow-start{stroke-width:2}.chartlet-flow-end{stroke-width:3}.chartlet-flow-external{stroke-dasharray:5 3}.chartlet-flow-inner{stroke:var(--chartlet-role-blue);stroke-width:1}.chartlet-flow-label{font-size:13px;font-weight:600;fill:var(--chartlet-text)}.chartlet-flow-sublabel{font-size:11px;fill:var(--chartlet-muted)}.chartlet-flow-edge{fill:none;stroke:var(--chartlet-zero);stroke-width:1.4;stroke-linejoin:round}.chartlet-flow-dashed{stroke-dasharray:5 4}.chartlet-flow-dotted{stroke-dasharray:1 4;stroke-linecap:round}.chartlet-flow-main{stroke:var(--chartlet-accent);stroke-width:2.2}.chartlet-flow-head{fill:var(--chartlet-zero);stroke:var(--chartlet-zero);stroke-width:1;stroke-linejoin:round}.chartlet-flow-main-head{fill:var(--chartlet-accent);stroke:var(--chartlet-accent)}.chartlet-flow-lane{fill:var(--chartlet-lane);stroke:var(--chartlet-grid);stroke-width:1}.chartlet-flow-lane-alt{fill:var(--chartlet-background)}.chartlet-flow-lane-head{fill:var(--chartlet-lane-head);stroke:none}.chartlet-flow-lane-label{font-size:11px;font-weight:650;fill:var(--chartlet-muted);letter-spacing:.04em}.chartlet-flow-group{fill:var(--chartlet-accent);fill-opacity:.04;stroke:var(--chartlet-zero);stroke-width:1;stroke-dasharray:4 3;rx:10px}.chartlet-flow-group-label{font-size:11px;font-weight:600;fill:var(--chartlet-muted)}";

/// State diagrams, on top of [`DIAGRAM_STYLE`] and [`FLOW_STYLE`]: states with well rounded
/// corners, the inner outline of a final state, and the initial dot.
const STATE_STYLE: &str = ".chartlet-state{rx:14px}.chartlet-state-inner{fill:none;stroke:var(--chartlet-role-green);stroke-width:1.5;rx:11px}.chartlet-state-initial{fill:var(--chartlet-text);stroke:var(--chartlet-background);stroke-width:2}";

/// Architecture diagrams, on top of [`DIAGRAM_STYLE`] and [`FLOW_STYLE`]: the head of a person
/// and the dots of a window bar.
const ARCHITECTURE_STYLE: &str = ".chartlet-arch-head{fill:var(--chartlet-role-violet-fill);stroke:var(--chartlet-role-violet);stroke-width:1.5}.chartlet-arch-dot{fill:var(--chartlet-role-blue);stroke:none}";

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

/// The ID of an SVG's root element, which [`svg`] always writes as the root's first `id`.
fn root_id(svg: &str) -> Option<&str> {
    let start = svg.find(" id=\"")? + 5;
    let end = start + svg[start..].find('"')?;
    Some(&svg[start..end])
}

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
    let is_dark = spec.theme == Theme::Dark;
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
        StyleMode::Print => {
            print_stylesheet(&(base_style(spec, true) + &layer_style(spec)), is_dark)
        }
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

/// The class by which the shared stylesheet addresses a chart type.
fn type_class(chart_type: ChartType) -> String {
    format!("chartlet-type-{}", crate::spec::type_name(chart_type))
}

/// The stylesheets of the diagram types: what they share, then what each adds.
fn diagram_style(chart_type: ChartType) -> String {
    let parts: &[&str] = match chart_type {
        ChartType::Sequence => &[DIAGRAM_STYLE, SEQUENCE_STYLE],
        ChartType::Flow | ChartType::Tree => &[DIAGRAM_STYLE, FLOW_STYLE],
        ChartType::State => &[DIAGRAM_STYLE, FLOW_STYLE, STATE_STYLE],
        ChartType::Architecture => &[DIAGRAM_STYLE, FLOW_STYLE, ARCHITECTURE_STYLE],
        _ => &[],
    };
    parts.concat()
}

/// The style groups a chart uses, in the order of [`shared_stylesheet`], so that rules cascade
/// alike in a chart's own stylesheet and in the shared one. The print variant has no series
/// filter to style.
fn base_style(spec: &ChartSpec, print: bool) -> String {
    let has_series = spec.series.len() > 1;
    let is_time = matches!(spec.chart_type, ChartType::Time | ChartType::Multiples);
    // A categorical line chart with several series draws its lines in the palette and patterns of
    // a time chart's lines.
    let several_lines = spec.chart_type == ChartType::Line && has_series;
    let is_multiples = spec.chart_type == ChartType::Multiples;
    let is_diverging = matches!(spec.chart_type, ChartType::Stripes | ChartType::Calendar);
    let is_calendar = spec.chart_type == ChartType::Calendar;
    let is_rangebar = spec.chart_type == ChartType::Rangebar;
    let is_topicmap = spec.chart_type == ChartType::Topicmap;
    let is_atlas = spec.chart_type == ChartType::Atlas;
    let is_dark = spec.theme == Theme::Dark;
    format!(
        "{STYLE}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
        if is_dark { DARK_STYLE } else { "" },
        if has_series { SERIES_STYLE } else { "" },
        if has_series && !print {
            FILTER_STYLE
        } else {
            ""
        },
        if is_time || several_lines {
            LINE_SERIES_STYLE
        } else {
            ""
        },
        if (is_time && needs_layer_extras(spec)) || several_lines {
            LAYER_EXTRA_STYLE
        } else {
            ""
        },
        if (is_time && needs_mark_extras(spec)) || several_lines {
            MARK_EXTRA_STYLE
        } else {
            ""
        },
        if is_time && spec.layers().any(|layer| layer.stroke == Stroke::Medium) {
            MEDIUM_LINE_STYLE
        } else {
            ""
        },
        if is_time && needs_annotation_extras(spec) {
            ANNOTATION_EXTRA_STYLE
        } else {
            ""
        },
        if is_multiples { MULTIPLES_STYLE } else { "" },
        if is_multiples && spec.panes.iter().any(|pane| pane.note.is_some()) {
            PANEL_NOTE_STYLE
        } else {
            ""
        },
        if is_diverging {
            crate::diverging::STYLE
        } else {
            ""
        },
        if is_calendar { CALENDAR_STYLE } else { "" },
        if is_rangebar { RANGEBAR_STYLE } else { "" },
        if is_rangebar && spec.ranges.iter().any(|range| range.group.is_some()) {
            RANGE_GROUP_STYLE
        } else {
            ""
        },
        if spec.references.is_empty() {
            ""
        } else {
            REFERENCE_STYLE
        },
        if spec.patterns { OUTLINE_STYLE } else { "" },
        if spec.width < crate::layout::NARROW {
            SMALL_TITLE_STYLE
        } else {
            ""
        },
        if is_topicmap { TOPICMAP_STYLE } else { "" },
        if is_atlas { ATLAS_STYLE } else { "" },
        diagram_style(spec.chart_type),
        if is_time && spec.layers().any(|layer| layer.mark == Mark::Ohlc) {
            OHLC_STYLE
        } else {
            ""
        },
    )
}

/// Every style group for each of `chart_types` that can use it, in the order of [`base_style`].
/// The light palette and the dark theme apply to every chart; every other group is scoped to the
/// chart types that use it, so that two groups styling the same class, such as the hatching of a
/// time chart and of a range bar chart, never meet.
/// The shared stylesheet: with `common`, first the part every chart relies on, then the part of
/// each of `chart_types` in their order. A type's part styles only charts of that type, so the
/// parts of two types never style the same element and may be concatenated in any order.
pub(crate) fn shared_stylesheet(chart_types: &[ChartType], common: bool) -> String {
    use ChartType::{
        Architecture, Atlas, Bar, Calendar, Flow, Line, Multiples, Rangebar, Sequence, State,
        Stripes, Time, Topicmap, Tree,
    };
    let groups: [(&str, &[ChartType]); 26] = [
        (STYLE, &[]),
        (DARK_STYLE, &[]),
        (SMALL_TITLE_STYLE, &[]),
        (SERIES_STYLE, &[Bar]),
        (FILTER_STYLE, &[Bar]),
        (LINE_SERIES_STYLE, &[Line, Time, Multiples]),
        (LAYER_EXTRA_STYLE, &[Line, Time, Multiples]),
        (MARK_EXTRA_STYLE, &[Line, Time, Multiples]),
        (MEDIUM_LINE_STYLE, &[Time, Multiples]),
        (ANNOTATION_EXTRA_STYLE, &[Time, Multiples]),
        (MULTIPLES_STYLE, &[Multiples]),
        (PANEL_NOTE_STYLE, &[Multiples]),
        (crate::diverging::STYLE, &[Stripes, Calendar]),
        (CALENDAR_STYLE, &[Calendar]),
        (RANGEBAR_STYLE, &[Rangebar]),
        (RANGE_GROUP_STYLE, &[Rangebar]),
        (REFERENCE_STYLE, &[Bar]),
        (OUTLINE_STYLE, &[Bar]),
        (TOPICMAP_STYLE, &[Topicmap]),
        (ATLAS_STYLE, &[Atlas]),
        (DIAGRAM_STYLE, &[Sequence, Flow, State, Architecture, Tree]),
        (SEQUENCE_STYLE, &[Sequence]),
        (FLOW_STYLE, &[Flow, State, Architecture, Tree]),
        (STATE_STYLE, &[State]),
        (ARCHITECTURE_STYLE, &[Architecture]),
        (OHLC_STYLE, &[Time]),
    ];
    let mut stylesheet = String::new();
    // An empty list of types: the group applies to every chart.
    if common {
        for (group, _) in groups.iter().filter(|(_, types)| types.is_empty()) {
            stylesheet.push_str(&scope_stylesheet(group, ".chartlet-root", false));
        }
    }
    for (index, chart_type) in chart_types.iter().enumerate() {
        if chart_types[..index].contains(chart_type) {
            continue;
        }
        let scope = format!(".{}", type_class(*chart_type));
        for (group, _) in groups
            .iter()
            .filter(|(_, types)| types.contains(chart_type))
        {
            stylesheet.push_str(&scope_stylesheet(group, &scope, false));
        }
    }
    stylesheet
}

/// The stylesheet of the print variant. Renderers outside the browser, such as those of print and
/// PDF pipelines, often support class selectors in `<style>` but not CSS custom properties, and
/// would draw every mark that takes its color from one in black or not at all. This resolves
/// every `var()` to the value the chart's theme gives it, and a color declared as a variable to
/// its fallback, the chart's text color, and drops the declarations of the custom properties,
/// together with the rules left empty by that. The attribute selector for the realms of a
/// landscape, which resvg does not match, is written out as the classes it matches.
fn print_stylesheet(stylesheet: &str, is_dark: bool) -> String {
    let realms = crate::layout::REALM_CLASSES
        .iter()
        .chain(&["chartlet-atlas-realm-label"])
        .map(|class| format!(".{class}"))
        .collect::<Vec<_>>()
        .join(",");
    // Renderers outside the browser read font weights in hundreds only and would draw 650 as
    // regular.
    let stylesheet = &stylesheet
        .replace("[class^='chartlet-atlas-realm-']", &realms)
        .replace("font-weight:650", "font-weight:600");
    let mut properties: Vec<(&str, &str)> = Vec::new();
    for rule in stylesheet.split_inclusive('}') {
        let Some((selector, body)) = rule.split_once('{') else {
            continue;
        };
        if selector != ".chartlet-root" && !(is_dark && selector == ".chartlet-theme-dark") {
            continue;
        }
        for declaration in body.trim_end_matches('}').split(';') {
            if let Some((name, value)) = declaration.split_once(':')
                && name.starts_with("--")
            {
                properties.retain(|(known, _)| *known != name);
                properties.push((name, value));
            }
        }
    }
    let lookup = |name: &str| {
        properties
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, value)| *value)
    };
    let mut resolved = String::with_capacity(stylesheet.len());
    for rule in stylesheet.split_inclusive('}') {
        let Some((selector, body)) = rule.split_once('{') else {
            resolved.push_str(rule);
            continue;
        };
        let declarations: Vec<String> = body
            .trim_end_matches('}')
            .split(';')
            .filter(|declaration| !declaration.starts_with("--"))
            .map(|declaration| resolve_variables(declaration, &lookup))
            .collect();
        if declarations.is_empty() {
            continue;
        }
        write!(resolved, "{selector}{{{}}}", declarations.join(";"))
            .expect("writing to String cannot fail");
    }
    resolved
}

/// Replaces every `var(--name)` and `var(--name,fallback)` in a declaration by the value of the
/// property, or by its fallback when the chart does not define it; `currentColor` as a fallback
/// becomes the chart's text color.
fn resolve_variables<'a>(declaration: &str, lookup: &impl Fn(&str) -> Option<&'a str>) -> String {
    let mut output = String::new();
    let mut rest = declaration;
    while let Some(start) = rest.find("var(") {
        output.push_str(&rest[..start]);
        let end = start + rest[start..].find(')').expect("a var() is closed");
        let (name, fallback) = match rest[start + 4..end].split_once(',') {
            Some((name, fallback)) => (name, Some(fallback)),
            None => (&rest[start + 4..end], None),
        };
        let value = lookup(name)
            .or_else(|| match fallback {
                Some("currentColor") => lookup("--chartlet-text"),
                fallback => fallback,
            })
            .expect("every custom property chartlet uses is defined or has a fallback");
        output.push_str(value);
        rest = &rest[end + 1..];
    }
    output.push_str(rest);
    output
}

/// Scopes a chart's stylesheet to its root element. An inline SVG's stylesheet applies to the
/// whole page, so without this a later chart's rule such as `.chartlet-line` would recolor the
/// series of an earlier one. Rules that only style the root keep their low specificity, because
/// the `--chartlet-*` custom properties on it are what a host page overrides; the dark theme
/// doubles its class so that a later light chart's defaults cannot win over it. Rules for the
/// HTML controls around the SVG stay as they are. The print variant has no custom properties to
/// override, so its rules on the root are scoped by ID too.
fn scope_stylesheet(stylesheet: &str, scope: &str, print: bool) -> String {
    let mut scoped = String::with_capacity(stylesheet.len() * 2);
    for rule in stylesheet.split_inclusive('}') {
        let Some((selectors, body)) = rule.split_once('{') else {
            scoped.push_str(rule);
            continue;
        };
        let selectors: Vec<String> = selectors
            .split(',')
            .map(|selector| scope_selector(selector, scope, print))
            .collect();
        scoped.push_str(&selectors.join(","));
        scoped.push('{');
        scoped.push_str(body);
    }
    scoped
}

fn scope_selector(selector: &str, scope: &str, print: bool) -> String {
    const ROOT_CLASSES: [&str; 3] = [
        ".chartlet-root",
        ".chartlet-theme-dark",
        ".chartlet-multiples",
    ];
    if selector.starts_with('#')
        || selector.starts_with(".chartlet-wrapper")
        || selector.starts_with(".chartlet-filter")
    {
        return selector.to_owned();
    }
    let on_root = ROOT_CLASSES.iter().any(|class| selector.starts_with(class));
    match (on_root, selector.contains(' ')) {
        (true, false) if selector == ".chartlet-theme-dark" => {
            ".chartlet-root.chartlet-theme-dark".to_owned()
        }
        (true, false) if print => format!("{scope}{selector}"),
        (true, false) => selector.to_owned(),
        (true, true) => format!("{scope}{selector}"),
        (false, _) => format!("{scope} {selector}"),
    }
}

/// Whether a time chart uses anything beyond plain lines in the first palette colors.
fn needs_layer_extras(spec: &ChartSpec) -> bool {
    spec.chart_type == ChartType::Multiples
        || spec.layers().any(|layer| {
            matches!(layer.mark, Mark::Annotation | Mark::Area | Mark::Band)
                || layer.modeled
                || layer.has_band()
        })
        || spec.data_layers().any(|entry| {
            entry.layer.resolved_color().is_none()
                && (spec.palette_index(entry.layer) > 0 || spec.series_names().len() > 1)
        })
}

/// Whether a time chart draws a zone or a point marker.
fn needs_annotation_extras(spec: &ChartSpec) -> bool {
    spec.layers()
        .any(|layer| layer.mark == Mark::Band || layer.is_marker())
}

/// Whether a time chart draws an area, a line pattern other than solid or the modeled dash, or a
/// bold line.
fn needs_mark_extras(spec: &ChartSpec) -> bool {
    spec.layers().any(|layer| {
        layer.mark == Mark::Area
            || layer.stroke == Stroke::Bold
            || (layer.is_data()
                && !matches!(
                    (layer.effective_dash(), layer.modeled),
                    (Dash::Solid, _) | (Dash::Dashed, true)
                ))
    })
}

/// CSS rules for the colors a time layer declares itself, including the fill of its band. Only colors that passed the contract reach this point, and the contract admits no
/// character that could end a declaration, so the values are safe to interpolate. The rules are
/// scoped to the chart's root ID: an inline SVG's stylesheet applies to the whole page, and two
/// charts would otherwise recolor each other's layers.
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
            if layer.is_marker() {
                write!(
                    style,
                    ".chartlet-marker.chartlet-style-{index}{{fill:{color};stroke:var(--chartlet-background)}}"
                )
                .expect("writing to String cannot fail");
            }
            if layer.mark == Mark::Area {
                write!(
                    style,
                    ".chartlet-area.chartlet-style-{index}{{fill:{color};stroke:none}}"
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
        Element::Polyline(polyline) => emit_polyline(polyline, None, id_prefix, output),
        Element::TopicArea(polyline, center) => {
            emit_polyline(polyline, Some(*center), id_prefix, output);
        }
        Element::Text(text) => emit_text(text, None, TextStyle::default(), output),
        Element::Hook(hook) => {
            if let Some(spec) = hooks {
                emit_hook(hook, spec, output);
            }
        }
        Element::Group(classes) => {
            write!(output, "<g class=\"{classes}\">").expect("write");
        }
        Element::GroupEnd => output.push_str("</g>"),
        Element::Hotspot(_) => {}
    }
}

/// Which `data-*` hooks of the interactive module a chart carries: none, the hooks alone (the
/// HTML profile, whose data table carries the values), or the hooks with the values in a JSON
/// data block (the SVG profile).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hooks {
    Off,
    Attributes,
    WithData,
}

/// Writes a hook: the plot of a pane as an empty group, or the group around one layer.
fn emit_hook(hook: &Hook, spec: &ChartSpec, output: &mut String) {
    let pairs = |pairs: &[(f64, f64)]| {
        let (values, pixels): (Vec<String>, Vec<String>) = pairs
            .iter()
            .map(|(value, pixel)| (number(*value), number(*pixel)))
            .unzip();
        (values.join(" "), pixels.join(" "))
    };
    match hook {
        Hook::Plot { pane, x, y, log } => {
            let (x_domain, x_range) = pairs(x);
            let (y_domain, y_range) = pairs(y);
            let scale = if *log { " data-y-scale=\"log\"" } else { "" };
            write!(
                output,
                "<g data-chartlet-plot=\"\" data-pane=\"{pane}\" data-x-domain=\"{x_domain}\" data-x-range=\"{x_range}\" data-y-domain=\"{y_domain}\" data-y-range=\"{y_range}\"{scale}/>"
            )
            .expect("write");
        }
        Hook::Layer(global) => {
            let entry = spec
                .indexed_layers()
                .nth(*global)
                .expect("a hook names an existing layer");
            if entry.layer.is_data() {
                let series = spec
                    .data_layers()
                    .position(|data| data.global == *global)
                    .expect("a data layer is among the data layers");
                write!(
                    output,
                    "<g data-series=\"{series}\" data-pane=\"{}\"",
                    entry.pane
                )
                .expect("write");
                if let Some(name) = &entry.layer.name {
                    write!(output, " data-name=\"{}\"", escape(name)).expect("write");
                }
                output.push('>');
            } else {
                write!(
                    output,
                    "<g data-annotation=\"{global}\" data-pane=\"{}\">",
                    entry.pane
                )
                .expect("write");
            }
        }
        Hook::End => output.push_str("</g>"),
    }
}

/// What the interactive module needs to know about the data table beyond its text: per value
/// column the series it belongs to (its index among the data layers, or among the series of a
/// category chart), the pane and which part of the series it holds; per row its position on the
/// axis (Unix seconds on a time axis, the row index otherwise); and every value unformatted.
/// The hooks of the data table: per value column its series, pane, part and whether it is a
/// stacked area, drawn on top of the stacked areas before it; per row its position and values.
struct TableHooks {
    columns: Vec<(usize, usize, &'static str, bool)>,
    x: Vec<f64>,
    values: Vec<Vec<Option<f64>>>,
}

fn table_hooks(spec: &ChartSpec) -> TableHooks {
    // A diagram's table holds text only: its columns belong to no series and its rows carry no
    // values.
    if matches!(
        spec.chart_type,
        ChartType::Sequence
            | ChartType::Flow
            | ChartType::State
            | ChartType::Architecture
            | ChartType::Tree
    ) {
        let table = data_table(spec);
        return TableHooks {
            columns: (1..table.columns.len())
                .map(|index| (index - 1, 0, "text", false))
                .collect(),
            x: (0..table.rows.len())
                .map(|row| f64::from(u32::try_from(row).expect("rows are limited")))
                .collect(),
            values: vec![vec![None; table.columns.len() - 1]; table.rows.len()],
        };
    }
    let dataset = spec.table_dataset();
    let rows = dataset.categories.len();
    let values = (0..rows)
        .map(|row| {
            dataset
                .series
                .iter()
                .map(|series| series.values[row])
                .collect()
        })
        .collect();
    if !matches!(spec.chart_type, ChartType::Time | ChartType::Multiples) {
        return TableHooks {
            columns: (0..dataset.series.len())
                .map(|index| (index, 0, "value", false))
                .collect(),
            x: (0..rows)
                .map(|row| f64::from(u32::try_from(row).expect("rows are limited")))
                .collect(),
            values,
        };
    }
    let zone = spec.time_zone().unwrap_or_default();
    let mut epochs: Vec<i64> = spec
        .data_layers()
        .flat_map(|entry| entry.layer.resolved_times(zone))
        .collect();
    epochs.sort_unstable();
    epochs.dedup();
    let mut columns = Vec::new();
    for (series, entry) in spec.data_layers().enumerate() {
        let parts: &[&'static str] = if entry.layer.mark == Mark::Ohlc {
            &["open", "high", "low", "close"]
        } else if entry.layer.has_band() {
            &["value", "lower", "upper"]
        } else {
            &["value"]
        };
        let stacked = spec.stack_base(entry, zone).is_some();
        columns.extend(
            parts
                .iter()
                .map(|part| (series, entry.pane, *part, stacked)),
        );
    }
    // The specification allows 1700 to 2200, well within the integers an f64 holds exactly.
    #[allow(clippy::cast_precision_loss)]
    let x = epochs.iter().map(|epoch| *epoch as f64).collect();
    TableHooks { columns, x, values }
}

/// The data table as a JSON data block for the SVG profile, which has no table to read: the
/// columns with their hooks, and per row its position, its text and its unformatted values. A
/// script element of type `application/json` is never executed, so a strict CSP allows it.
fn emit_data_block(spec: &ChartSpec, output: &mut String) {
    let table = data_table(spec);
    let hooks = table_hooks(spec);
    let columns: Vec<serde_json::Value> = table
        .columns
        .iter()
        .skip(1)
        .zip(&hooks.columns)
        .map(|(name, (series, pane, part, stacked))| {
            let mut column =
                serde_json::json!({ "name": name, "series": series, "pane": pane, "part": part });
            if *stacked {
                column["stacked"] = serde_json::Value::Bool(true);
            }
            column
        })
        .collect();
    let rows: Vec<serde_json::Value> = table
        .rows
        .iter()
        .zip(hooks.x.iter().zip(&hooks.values))
        .map(|(row, (x, values))| {
            serde_json::json!({ "x": x, "label": row[0], "text": row[1..], "value": values })
        })
        .collect();
    let block = serde_json::json!({ "columns": columns, "rows": rows });
    write!(
        output,
        "<script type=\"application/json\" data-chartlet-data=\"\">{}</script>",
        escape_text(&block.to_string())
    )
    .expect("write");
}

/// Escapes text content for XML and HTML alike; quotes may stay as they are.
fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Writes a polyline; `center`, when given, becomes its `data-cx`/`data-cy`.
fn emit_polyline(
    polyline: &crate::scene::Polyline,
    center: Option<(f64, f64)>,
    id_prefix: &str,
    output: &mut String,
) {
    open("polyline", polyline.series_index, output);
    let points = polyline
        .points
        .iter()
        .map(|(x, y)| format!("{},{}", number(*x), number(*y)))
        .collect::<Vec<_>>()
        .join(" ");
    let center = center.map_or_else(String::new, |(cx, cy)| {
        format!(" data-cx=\"{}\" data-cy=\"{}\"", number(cx), number(cy))
    });
    write!(
        output,
        " points=\"{}\"{} class=\"{}{}{}\"{center}",
        points,
        hatch_fill(polyline.class, polyline.style_index, id_prefix),
        polyline.class,
        style_class(polyline.style_index),
        topic_class(polyline.topic),
    )
    .expect("write");
    close("polyline", polyline.tooltip.as_deref(), output);
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

/// Writes the data table; with `hooks`, the chart's ID prefix, it also carries the
/// [`TableHooks`] as `data-*` attributes.
fn render_data_table(
    output: &mut String,
    spec: &ChartSpec,
    table_mode: TableMode,
    hooks: Option<&str>,
) {
    if table_mode == TableMode::Details {
        write!(
            output,
            "<details class=\"chartlet-data\" data-viz-text><summary>{}</summary>",
            spec.locale.words().show_data
        )
        .expect("write");
    } else {
        output.push_str("<div class=\"chartlet-data\" data-viz-text>");
    }
    let table = data_table(spec);
    let table_hooks = hooks.map(|_| table_hooks(spec));
    let table_id = hooks.map_or_else(String::new, |id_prefix| {
        format!(" data-chartlet-table=\"{id_prefix}\"")
    });
    write!(
        output,
        "<table{table_id}><caption>{}</caption><thead><tr>",
        escape(&table.caption)
    )
    .expect("write");
    for (index, column) in table.columns.iter().enumerate() {
        let column_hooks = match (&table_hooks, index.checked_sub(1)) {
            (Some(table_hooks), Some(index)) => {
                let (series, pane, part, stacked) = table_hooks.columns[index];
                let stacked = if stacked { " data-stacked=\"\"" } else { "" };
                format!(
                    " data-series=\"{series}\" data-pane=\"{pane}\" data-part=\"{part}\"{stacked}"
                )
            }
            _ => String::new(),
        };
        write!(
            output,
            "<th scope=\"col\"{column_hooks}>{}</th>",
            escape(column)
        )
        .expect("write");
    }
    output.push_str("</tr></thead><tbody>");
    for (row_index, row) in table.rows.iter().enumerate() {
        let (head, cells) = row.split_first().expect("a row starts with its category");
        match &table_hooks {
            Some(table_hooks) => write!(
                output,
                "<tr data-x=\"{}\"><th scope=\"row\">{}</th>",
                number(table_hooks.x[row_index]),
                escape(head)
            ),
            None => write!(output, "<tr><th scope=\"row\">{}</th>", escape(head)),
        }
        .expect("write");
        for (column, cell) in cells.iter().enumerate() {
            match table_hooks
                .as_ref()
                .and_then(|table_hooks| table_hooks.values[row_index][column])
            {
                Some(value) => write!(output, "<td data-value=\"{value}\">{}</td>", escape(cell)),
                None => write!(output, "<td>{}</td>", escape(cell)),
            }
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

/// The data table of a chart as text: its caption, the column heads and one row per category,
/// each starting with the category, with values written as the chart writes them.
pub(crate) fn data_table(spec: &ChartSpec) -> DataTable {
    match spec.chart_type {
        ChartType::Sequence => return crate::sequence::data_table(spec),
        ChartType::Flow => return crate::flow::data_table(spec),
        ChartType::State => return crate::state::data_table(spec),
        ChartType::Architecture => return crate::architecture::data_table(spec),
        ChartType::Tree => return crate::tree::data_table(spec),
        _ => {}
    }
    let words = spec.locale.words();
    let dataset = spec.table_dataset();
    // A numeric axis is named by its title; it holds no times.
    let numeric = spec.time_axis.kind == crate::spec::TimeAxisKind::Number;
    let first = match spec.chart_type {
        ChartType::Time | ChartType::Multiples if numeric => {
            spec.time_axis.title.as_deref().unwrap_or(words.position)
        }
        ChartType::Time | ChartType::Multiples => words.time,
        ChartType::Topicmap => words.topic,
        ChartType::Atlas => words.region,
        ChartType::Stripes => words.year,
        ChartType::Calendar => words.date,
        ChartType::Bar | ChartType::Line | ChartType::Rangebar => words.category,
        ChartType::Sequence
        | ChartType::Flow
        | ChartType::State
        | ChartType::Architecture
        | ChartType::Tree => {
            unreachable!("a diagram writes its own table")
        }
    };
    let columns = std::iter::once(first.to_owned())
        .chain(
            dataset
                .series
                .iter()
                .map(|series| series.name.as_deref().unwrap_or(words.value).to_owned()),
        )
        .collect();
    let rows = dataset
        .categories
        .iter()
        .enumerate()
        .map(|(index, category)| {
            std::iter::once(category.clone())
                .chain(dataset.series.iter().map(|series| {
                    series.values[index].map_or_else(
                        || words.missing.to_owned(),
                        |value| {
                            format_value(value, series.style.unwrap_or_else(|| spec.number_style()))
                        },
                    )
                }))
                .collect()
        })
        .collect();
    DataTable {
        caption: format!("{} {}", words.data_for, spec.title),
        columns,
        rows,
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
