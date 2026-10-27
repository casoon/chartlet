//! The stylesheets: the style groups of every chart type, a chart's own stylesheet and the
//! shared one, both built from the groups in one order, and the print variant.

use std::{collections::BTreeMap, fmt::Write};

use crate::spec::{ChartSpec, ChartType, Dash, Mark, RangeDisplay, Stroke, Theme};

/// The light palette. Every color is a CSS custom property so a host page can override a single
/// color, and so the dark palette below only has to restate the values.
const STYLE: &str = ".chartlet-root{--chartlet-text:#172033;--chartlet-muted:#344054;--chartlet-grid:#d9dee8;--chartlet-zero:#667085;--chartlet-accent:#2563eb;--chartlet-background:#fff;--chartlet-color-1:#2563eb;--chartlet-color-2:#c2410c;--chartlet-color-3:#475569;--chartlet-color-4:#111827;max-width:100%;height:auto;font-family:Inter,ui-sans-serif,system-ui,sans-serif;color:var(--chartlet-text)}.chartlet-title{font-size:22px;font-weight:650;fill:var(--chartlet-text)}.chartlet-label,.chartlet-tick,.chartlet-value,.chartlet-value-inverse,.chartlet-axis-title{font-size:12px;fill:var(--chartlet-muted)}.chartlet-value,.chartlet-value-inverse{font-weight:600}.chartlet-value{fill:var(--chartlet-text)}.chartlet-value-inverse{fill:var(--chartlet-background)}.chartlet-axis-title{font-weight:600}.chartlet-grid{stroke:var(--chartlet-grid);stroke-width:1}.chartlet-zero{stroke:var(--chartlet-zero);stroke-width:1.5}.chartlet-bar,.chartlet-point{fill:var(--chartlet-accent)}.chartlet-line{fill:none;stroke:var(--chartlet-accent);stroke-width:3;stroke-linecap:round;stroke-linejoin:round}.chartlet-background{fill:var(--chartlet-background)}";

/// The dark palette. A dark chart paints its own background: unlike the light profile it cannot
/// rely on the page behind it, which may be any color.
const DARK_STYLE: &str = ".chartlet-theme-dark{--chartlet-text:#e8edf6;--chartlet-muted:#a9b4c7;--chartlet-grid:#2a3342;--chartlet-zero:#7b8697;--chartlet-accent:#7ea6ff;--chartlet-background:#0e131c;--chartlet-color-1:#7ea6ff;--chartlet-color-2:#ff9c72;--chartlet-color-3:#9aa7bd;--chartlet-color-4:#e2e8f4}";

/// The blueprint look of a diagram: white-blue lines on a deep blue ground. It rests on the dark
/// palette, so it only restates what differs; the roles keep their order of hue by lightness.
const BLUEPRINT_STYLE: &str = ".chartlet-blueprint{--chartlet-text:#eaf4ff;--chartlet-muted:#b8d4f2;--chartlet-grid:#3d73ab;--chartlet-zero:#8fb8e0;--chartlet-accent:#7fd3ff;--chartlet-background:#0b3a68;--chartlet-node:#12497f;--chartlet-lane:#0e4073;--chartlet-lane-head:#14518d;--chartlet-shadow:rgba(0,0,0,0);--chartlet-role-blue:#8ecbff;--chartlet-role-blue-fill:#12497f;--chartlet-role-green:#9ae6b4;--chartlet-role-green-fill:#0f4d63;--chartlet-role-amber:#ffd98a;--chartlet-role-amber-fill:#1a4a6e;--chartlet-role-teal:#7fe3d6;--chartlet-role-teal-fill:#0c4b66;--chartlet-role-violet:#cbbaff;--chartlet-role-violet-fill:#1d4378;--chartlet-role-gray:#b8c9dc;--chartlet-role-gray-fill:#133f6e;--chartlet-role-red:#ffa198;--chartlet-role-red-fill:#183f6a}";

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
/// Box plots: the box, its median, whiskers and the points beyond them.
/// Timelines: phases in the colors of their groups, milestones, markers and arrows.
const TIMELINE_STYLE: &str = ".chartlet-timeline-phase{fill:var(--chartlet-accent);stroke:none;rx:3px}.chartlet-timeline-milestone{fill:var(--chartlet-accent);stroke:var(--chartlet-background);stroke-width:1.5}.chartlet-timeline-group-1{fill:var(--chartlet-color-1)}.chartlet-timeline-group-2{fill:var(--chartlet-color-2)}.chartlet-timeline-group-3{fill:var(--chartlet-color-3)}.chartlet-timeline-group-4{fill:var(--chartlet-color-4)}.chartlet-timeline-weekend{fill:var(--chartlet-grid);fill-opacity:.55;stroke:none}.chartlet-timeline-done{fill:var(--chartlet-text);fill-opacity:.35;stroke:none;rx:3px}.chartlet-timeline-marker{stroke:var(--chartlet-text);stroke-width:1.5;stroke-dasharray:5 4}.chartlet-timeline-marker-label{font-size:12px;font-weight:600;fill:var(--chartlet-text)}.chartlet-timeline-link{fill:none;stroke:var(--chartlet-zero);stroke-width:1.4;stroke-linejoin:round}.chartlet-timeline-head{fill:var(--chartlet-zero);stroke:var(--chartlet-zero);stroke-width:1;stroke-linejoin:round}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Waterfalls: bars that rise, fall and total.
const WATERFALL_STYLE: &str = ".chartlet-wf-up{fill:var(--chartlet-color-1);stroke:none}.chartlet-wf-down{fill:var(--chartlet-color-2);stroke:none}.chartlet-wf-total{fill:var(--chartlet-color-3);stroke:none}.chartlet-wf-link{stroke:var(--chartlet-zero);stroke-width:1;stroke-dasharray:3 3}";
/// Polar charts: rings, wedges or lollipops in one color, and the values of the rings.
const POLAR_STYLE: &str = ".chartlet-polar-ring{fill:none;stroke:var(--chartlet-grid);stroke-width:1}.chartlet-polar-bar{fill:var(--chartlet-color-1);stroke:none}.chartlet-polar-stem{stroke:var(--chartlet-color-1);stroke-width:2}.chartlet-polar-dot{fill:var(--chartlet-color-1);stroke:var(--chartlet-background);stroke-width:1.5}.chartlet-polar-tick{font-size:11px;fill:var(--chartlet-muted);paint-order:stroke;stroke:var(--chartlet-background);stroke-width:3px;stroke-linejoin:round}";
/// Mosaic charts: segments in the palette colors with their share written on them.
const MOSAIC_STYLE: &str = ".chartlet-mosaic-1{fill:var(--chartlet-color-1)}.chartlet-mosaic-2{fill:var(--chartlet-color-2)}.chartlet-mosaic-3{fill:var(--chartlet-color-3)}.chartlet-mosaic-4{fill:var(--chartlet-color-4)}.chartlet-mosaic-cell{stroke:var(--chartlet-background);stroke-width:1}.chartlet-mosaic-value{font-size:12px;font-weight:600;fill:var(--chartlet-background)}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Radar charts: grid, polygons told apart by color and dash pattern, markers and ring values.
const RADAR_STYLE: &str = ".chartlet-radar-ring{fill:none;stroke:var(--chartlet-grid);stroke-width:1}.chartlet-radar-spoke{stroke:var(--chartlet-grid);stroke-width:1}.chartlet-radar-area{fill-opacity:.14;stroke-width:2;stroke-linejoin:round}.chartlet-radar-key{stroke-width:2}.chartlet-radar-a1{fill:var(--chartlet-color-1);stroke:var(--chartlet-color-1)}.chartlet-radar-a2{fill:var(--chartlet-color-2);stroke:var(--chartlet-color-2);stroke-dasharray:7 4}.chartlet-radar-a3{fill:var(--chartlet-color-3);stroke:var(--chartlet-color-3);stroke-dasharray:2 3}.chartlet-radar-a4{fill:var(--chartlet-color-4);stroke:var(--chartlet-color-4);stroke-dasharray:8 3 2 3}.chartlet-radar-dot{stroke:var(--chartlet-background);stroke-width:1.5}.chartlet-radar-d1{fill:var(--chartlet-color-1)}.chartlet-radar-d2{fill:var(--chartlet-color-2)}.chartlet-radar-d3{fill:var(--chartlet-color-3)}.chartlet-radar-d4{fill:var(--chartlet-color-4)}.chartlet-radar-tick{font-size:11px;fill:var(--chartlet-muted);paint-order:stroke;stroke:var(--chartlet-background);stroke-width:3px;stroke-linejoin:round}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Funnels and pyramids: the bars or levels, and the rate between two stages.
const FUNNEL_STYLE: &str = ".chartlet-funnel-bar{fill:var(--chartlet-color-1);stroke:none}.chartlet-funnel-rate{font-size:12px;font-weight:600;fill:var(--chartlet-text)}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Waffles: squares in the palette colors, and the rest.
/// Heatmaps: a sequential and a diverging scale and their text colors, the palette colors of
/// categories with their symbols, and the cells that have no value.
const HEATMAP_STYLE: &str = ".chartlet-hm-s0{fill:#cde0f5}.chartlet-hm-s1{fill:#a7c8ea}.chartlet-hm-s2{fill:#81afdf}.chartlet-hm-s3{fill:#5b97d3}.chartlet-hm-s4{fill:#0f66ac}.chartlet-hm-s5{fill:#064f89}.chartlet-hm-d0{fill:#9c4a05}.chartlet-hm-d1{fill:#e0852a}.chartlet-hm-d2{fill:#f6c389}.chartlet-hm-d3{fill:#f4f4f4}.chartlet-hm-d4{fill:#a9cfe6}.chartlet-hm-d5{fill:#5b97d3}.chartlet-hm-d6{fill:#0f66ac}.chartlet-hm-c1{fill:var(--chartlet-color-1)}.chartlet-hm-c2{fill:var(--chartlet-color-2)}.chartlet-hm-c3{fill:var(--chartlet-color-3)}.chartlet-hm-c4{fill:var(--chartlet-color-4)}.chartlet-hm-cell{stroke:var(--chartlet-background);stroke-width:1}.chartlet-hm-key{stroke:var(--chartlet-grid);stroke-width:1}.chartlet-hm-missing{fill:none;stroke:var(--chartlet-zero);stroke-width:1;stroke-dasharray:3 3}.chartlet-hm-value{font-size:12px;font-weight:600}.chartlet-hm-on-light{fill:#172033}.chartlet-hm-on-dark{fill:#fff}.chartlet-hm-symbol{fill:var(--chartlet-background);stroke:var(--chartlet-text);stroke-width:1.5}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
const WAFFLE_STYLE: &str = ".chartlet-waffle-1{fill:var(--chartlet-color-1)}.chartlet-waffle-2{fill:var(--chartlet-color-2)}.chartlet-waffle-3{fill:var(--chartlet-color-3)}.chartlet-waffle-4{fill:var(--chartlet-color-4)}.chartlet-waffle-rest{fill:var(--chartlet-grid)}.chartlet-waffle-cell{stroke:none;rx:2px}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Fishbone diagrams: spine, head, bones with their category boxes, ribs and the labels on them.
const FISHBONE_STYLE: &str = ".chartlet-fish-spine{stroke:var(--chartlet-text);stroke-width:3;stroke-linecap:round}.chartlet-fish-arrow{fill:var(--chartlet-text);stroke:var(--chartlet-text);stroke-width:1;stroke-linejoin:round}.chartlet-fish-bone{stroke:var(--chartlet-role-teal);stroke-width:2;stroke-linecap:round}.chartlet-fish-rib{stroke:var(--chartlet-zero);stroke-width:1.2}.chartlet-fish-head{fill:var(--chartlet-role-blue-fill);stroke:var(--chartlet-role-blue);stroke-width:2;rx:8px}.chartlet-fish-head-text{font-size:13px;font-weight:700;fill:var(--chartlet-text)}.chartlet-fish-category{fill:var(--chartlet-role-teal-fill);stroke:var(--chartlet-role-teal);stroke-width:1.5;rx:6px}.chartlet-fish-category-text{font-size:13px;font-weight:650;fill:var(--chartlet-text)}.chartlet-fish-cause{font-size:12px;fill:var(--chartlet-text)}.chartlet-fish-subcause{font-size:12px;fill:var(--chartlet-muted)}";
/// Venn diagrams: circles, region text and leader lines.
const VENN_STYLE: &str = ".chartlet-venn{stroke-width:2;fill-opacity:.2}.chartlet-venn-1{fill:var(--chartlet-color-1);stroke:var(--chartlet-color-1)}.chartlet-venn-2{fill:var(--chartlet-color-2);stroke:var(--chartlet-color-2)}.chartlet-venn-3{fill:var(--chartlet-color-3);stroke:var(--chartlet-color-3)}.chartlet-venn-set{font-size:13px;font-weight:700;fill:var(--chartlet-text)}.chartlet-venn-label{font-size:12px;fill:var(--chartlet-text)}.chartlet-venn-count{font-size:14px;font-weight:700;fill:var(--chartlet-text)}.chartlet-venn-leader{stroke:var(--chartlet-zero);stroke-width:1}.chartlet-venn-dot{fill:var(--chartlet-text);stroke:none}";
/// Kanban boards: lanes, cards, limits and signs.
const KANBAN_STYLE: &str = ".chartlet-kan-lane{fill:var(--chartlet-lane);stroke:var(--chartlet-grid);stroke-width:1;rx:8px}.chartlet-kan-over{stroke:var(--chartlet-role-red);stroke-width:2;stroke-dasharray:6 3}.chartlet-kan-head{font-size:13px;font-weight:700;fill:var(--chartlet-text)}.chartlet-kan-load{font-size:12px;font-weight:600;fill:var(--chartlet-muted)}.chartlet-kan-over-text{fill:var(--chartlet-role-red);font-weight:700}.chartlet-kan-card{fill:var(--chartlet-background);stroke:var(--chartlet-grid);stroke-width:1;rx:6px}.chartlet-kan-blocked{stroke:var(--chartlet-role-red);stroke-width:2}.chartlet-kan-title{font-size:12px;fill:var(--chartlet-text)}.chartlet-kan-owner{font-size:11px;fill:var(--chartlet-muted)}.chartlet-kan-tag{fill:var(--chartlet-role-blue-fill);stroke:var(--chartlet-role-blue);stroke-width:1;rx:8px}.chartlet-kan-tag-text{font-size:11px;fill:var(--chartlet-text)}.chartlet-kan-sign{fill:var(--chartlet-background);stroke:var(--chartlet-role-red);stroke-width:1.5}.chartlet-kan-sign-bar{stroke:var(--chartlet-role-red);stroke-width:2}.chartlet-kan-sign-text{font-size:11px;font-weight:800;fill:var(--chartlet-role-red)}.chartlet-kan-blocked-text{font-size:11px;font-weight:700;fill:var(--chartlet-role-red)}";
/// Story maps: backbone, tasks, cards and cut lines.
const STORY_STYLE: &str = ".chartlet-story-activity{fill:var(--chartlet-role-blue-fill);stroke:var(--chartlet-role-blue);stroke-width:1.5;rx:6px}.chartlet-story-activity-text{font-size:12px;font-weight:700;fill:var(--chartlet-text)}.chartlet-story-task{fill:var(--chartlet-role-teal-fill);stroke:var(--chartlet-role-teal);stroke-width:1.5;rx:6px}.chartlet-story-task-text{font-size:12px;font-weight:600;fill:var(--chartlet-text)}.chartlet-story-card{fill:var(--chartlet-background);stroke:var(--chartlet-grid);stroke-width:1;rx:5px}.chartlet-story-card-text{font-size:12px;fill:var(--chartlet-text)}.chartlet-story-cut{stroke:var(--chartlet-zero);stroke-width:1.2;stroke-dasharray:6 4}.chartlet-story-release{font-size:12px;font-weight:700;fill:var(--chartlet-muted)}";
/// Journeys and blueprints: stages, cells, lines and the sentiment.
const JOURNEY_STYLE: &str = ".chartlet-journey-stage{fill:var(--chartlet-role-blue-fill);stroke:var(--chartlet-role-blue);stroke-width:1.5;rx:6px}.chartlet-journey-stage-text{font-size:12px;font-weight:700;fill:var(--chartlet-text)}.chartlet-journey-cell{fill:var(--chartlet-background);stroke:var(--chartlet-grid);stroke-width:1;rx:4px}.chartlet-journey-text{font-size:12px;fill:var(--chartlet-text)}.chartlet-journey-row{font-size:12px;font-weight:700;fill:var(--chartlet-muted)}.chartlet-journey-line{stroke:var(--chartlet-zero);stroke-width:1.5;stroke-dasharray:7 4}.chartlet-journey-line-label{font-size:11px;font-weight:600;fill:var(--chartlet-muted)}.chartlet-journey-scale{stroke:var(--chartlet-grid);stroke-width:1}.chartlet-journey-zero{stroke:var(--chartlet-zero);stroke-width:1}.chartlet-journey-scale-text{font-size:11px;fill:var(--chartlet-muted)}.chartlet-journey-sentiment{fill:none;stroke:var(--chartlet-accent);stroke-width:2.5;stroke-linejoin:round}.chartlet-journey-dot{fill:var(--chartlet-background);stroke:var(--chartlet-accent);stroke-width:2.5}.chartlet-journey-value{font-size:12px;font-weight:700;fill:var(--chartlet-text)}.chartlet-journey-step{fill:none;stroke:var(--chartlet-accent);stroke-width:1.5;rx:3px}.chartlet-journey-step-on{fill:var(--chartlet-accent)}";
/// Value chain maps: frame, stages, dots, links and movement.
const VMAP_STYLE: &str = ".chartlet-vmap-frame{fill:none;stroke:var(--chartlet-grid);stroke-width:1}.chartlet-vmap-stage-line{stroke:var(--chartlet-grid);stroke-width:1;stroke-dasharray:4 4}.chartlet-vmap-stage{font-size:12px;font-weight:600;fill:var(--chartlet-muted)}.chartlet-vmap-axis{font-size:12px;font-weight:700;fill:var(--chartlet-muted)}.chartlet-vmap-tick{font-size:11px;fill:var(--chartlet-muted)}.chartlet-vmap-link{stroke:var(--chartlet-zero);stroke-width:1.2}.chartlet-vmap-dot{fill:var(--chartlet-role-blue-fill);stroke:var(--chartlet-role-blue);stroke-width:2}.chartlet-vmap-ghost{fill:none;stroke:var(--chartlet-role-blue);stroke-width:1.5;stroke-dasharray:3 2}.chartlet-vmap-move{fill:none;stroke:var(--chartlet-role-blue);stroke-width:1.5;stroke-dasharray:5 3}.chartlet-vmap-head{fill:var(--chartlet-role-blue);stroke:var(--chartlet-role-blue);stroke-width:1}.chartlet-vmap-inertia{stroke:var(--chartlet-role-red);stroke-width:3}.chartlet-vmap-label{font-size:12px;fill:var(--chartlet-text);paint-order:stroke;stroke:var(--chartlet-background);stroke-width:3px;stroke-linejoin:round}";
/// Parliaments: seats by party, the majority line and the ring of a coalition.
const PARLIAMENT_STYLE: &str = ".chartlet-root{--chartlet-parl-5:#0f766e;--chartlet-parl-6:#6d28d9;--chartlet-parl-7:#b45309;--chartlet-parl-8:#be185d}.chartlet-root.chartlet-theme-dark{--chartlet-parl-5:#2dd4bf;--chartlet-parl-6:#b69cff;--chartlet-parl-7:#fbbf24;--chartlet-parl-8:#f472b6}.chartlet-parl-1{fill:var(--chartlet-color-1)}.chartlet-parl-2{fill:var(--chartlet-color-2)}.chartlet-parl-3{fill:var(--chartlet-color-3)}.chartlet-parl-4{fill:var(--chartlet-color-4)}.chartlet-parl-5{fill:var(--chartlet-parl-5)}.chartlet-parl-6{fill:var(--chartlet-parl-6)}.chartlet-parl-7{fill:var(--chartlet-parl-7)}.chartlet-parl-8{fill:var(--chartlet-parl-8)}.chartlet-parl-seat{stroke:none}.chartlet-parl-coalition{fill:none;stroke:var(--chartlet-text);stroke-width:2}.chartlet-parl-majority{stroke:var(--chartlet-text);stroke-width:1.5;stroke-dasharray:4 3}.chartlet-parl-total{font-size:28px;font-weight:700;fill:var(--chartlet-text)}.chartlet-parl-note{font-size:12px;fill:var(--chartlet-muted)}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Treemaps: the rectangles in the palette colors.
const TREEMAP_STYLE: &str = ".chartlet-tm-0{fill:var(--chartlet-color-1)}.chartlet-tm-1{fill:var(--chartlet-color-1)}.chartlet-tm-2{fill:var(--chartlet-color-2)}.chartlet-tm-3{fill:var(--chartlet-color-3)}.chartlet-tm-4{fill:var(--chartlet-color-4)}.chartlet-tm-cell{stroke:none}.chartlet-tm-frame{fill-opacity:.12;stroke-width:1.5}.chartlet-tm-frame.chartlet-tm-0,.chartlet-tm-frame.chartlet-tm-1{stroke:var(--chartlet-color-1)}.chartlet-tm-frame.chartlet-tm-2{stroke:var(--chartlet-color-2)}.chartlet-tm-frame.chartlet-tm-3{stroke:var(--chartlet-color-3)}.chartlet-tm-frame.chartlet-tm-4{stroke:var(--chartlet-color-4)}.chartlet-tm-head{font-size:12px;font-weight:600;fill:var(--chartlet-text)}.chartlet-tm-label{font-size:12px;font-weight:600;fill:var(--chartlet-background)}.chartlet-tm-value{font-size:11px;fill:var(--chartlet-background);opacity:.9}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Sankey diagrams: the bands and nodes in the palette colors.
const SANKEY_STYLE: &str = ".chartlet-sk-1{fill:var(--chartlet-color-1)}.chartlet-sk-2{fill:var(--chartlet-color-2)}.chartlet-sk-3{fill:var(--chartlet-color-3)}.chartlet-sk-4{fill:var(--chartlet-color-4)}.chartlet-sk-node{stroke:none}.chartlet-sk-link{stroke:none;opacity:.35}.chartlet-sk-label{font-size:12px;font-weight:600;fill:var(--chartlet-text)}.chartlet-sk-value{font-size:11px;fill:var(--chartlet-muted)}";
/// Kaplan-Meier curves: step lines, bands, marks and swatches in the palette colors.
const SURVIVAL_STYLE: &str = ".chartlet-km-1{fill:var(--chartlet-color-1);stroke:var(--chartlet-color-1)}.chartlet-km-2{fill:var(--chartlet-color-2);stroke:var(--chartlet-color-2)}.chartlet-km-3{fill:var(--chartlet-color-3);stroke:var(--chartlet-color-3)}.chartlet-km-4{fill:var(--chartlet-color-4);stroke:var(--chartlet-color-4)}.chartlet-km-line{fill:none;stroke-width:2.5;stroke-linejoin:round}.chartlet-km-band{stroke:none;fill-opacity:.15}.chartlet-km-mark{stroke-width:1.5}.chartlet-km-swatch{stroke:none}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Scatter plots: dots and swatches in the palette colors, threshold lines and names.
/// The bubbles of a scatter plot with sizes: a ring in the color of the ground, and the circles of
/// the size legend.
const SCATTER_BUBBLE_STYLE: &str = ".chartlet-sc-bubble{stroke:var(--chartlet-background);stroke-width:1.5}.chartlet-sc-ref{fill:none;stroke:var(--chartlet-muted);stroke-width:1.5}";
const SCATTER_STYLE: &str = ".chartlet-sc-1{fill:var(--chartlet-color-1)}.chartlet-sc-2{fill:var(--chartlet-color-2)}.chartlet-sc-3{fill:var(--chartlet-color-3)}.chartlet-sc-4{fill:var(--chartlet-color-4)}.chartlet-sc-dot{fill-opacity:.65;stroke:none}.chartlet-sc-swatch{stroke:none;rx:5px}.chartlet-sc-fit{stroke-width:2;fill:none}.chartlet-sc-fit.chartlet-sc-1{stroke:var(--chartlet-color-1)}.chartlet-sc-fit.chartlet-sc-2{stroke:var(--chartlet-color-2)}.chartlet-sc-fit.chartlet-sc-3{stroke:var(--chartlet-color-3)}.chartlet-sc-fit.chartlet-sc-4{stroke:var(--chartlet-color-4)}.chartlet-sc-line{stroke:var(--chartlet-zero);stroke-width:1.5;stroke-dasharray:5 4}.chartlet-sc-label{font-size:11px;font-weight:600;fill:var(--chartlet-text);paint-order:stroke;stroke:var(--chartlet-background);stroke-width:3px;stroke-linejoin:round}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Class diagrams, on top of [`DIAGRAM_STYLE`] and [`FLOW_STYLE`]: the members of a class, the
/// hollow and solid marks at the ends of relations, and their cardinalities.
const CLASSES_STYLE: &str = ".chartlet-class-member{font-size:12px;fill:var(--chartlet-text)}.chartlet-flow-card{font-size:11px;fill:var(--chartlet-muted)}.chartlet-flow-hollow{fill:var(--chartlet-background)}.chartlet-flow-solid{fill:var(--chartlet-zero)}.chartlet-flow-open{fill:none;stroke:var(--chartlet-zero);stroke-width:1.4;stroke-linejoin:round}";
/// The error bars of a bar chart.
const ERROR_STYLE: &str =
    ".chartlet-error{stroke:var(--chartlet-text);stroke-width:1.5;stroke-linecap:round}";
/// Box plots: the box, its median, whiskers and the points beyond them.
const BOXPLOT_STYLE: &str = ".chartlet-box{fill:var(--chartlet-accent);fill-opacity:.3;stroke:var(--chartlet-accent);stroke-width:1.5}.chartlet-box-median{stroke:var(--chartlet-text);stroke-width:3}.chartlet-box-whisker{stroke:var(--chartlet-accent);stroke-width:1.5;fill:none}.chartlet-box-outlier{fill:none;stroke:var(--chartlet-accent);stroke-width:1.5}.chartlet-violin{fill:var(--chartlet-accent);fill-opacity:.3;stroke:var(--chartlet-accent);stroke-width:1.5;stroke-linejoin:round}.chartlet-violin-box{fill:var(--chartlet-text);stroke:none}.chartlet-violin-median{fill:var(--chartlet-background);stroke:none}.chartlet-box-mean{fill:var(--chartlet-background);stroke:var(--chartlet-text);stroke-width:1.5}.chartlet-box-g1.chartlet-box,.chartlet-box-g1.chartlet-violin{fill:var(--chartlet-color-1);stroke:var(--chartlet-color-1)}.chartlet-box-g1.chartlet-box-whisker,.chartlet-box-g1.chartlet-box-outlier{stroke:var(--chartlet-color-1)}.chartlet-box-g1.chartlet-box-point{fill:var(--chartlet-color-1)}.chartlet-box-g2.chartlet-box,.chartlet-box-g2.chartlet-violin{fill:var(--chartlet-color-2);stroke:var(--chartlet-color-2)}.chartlet-box-g2.chartlet-box-whisker,.chartlet-box-g2.chartlet-box-outlier{stroke:var(--chartlet-color-2)}.chartlet-box-g2.chartlet-box-point{fill:var(--chartlet-color-2)}.chartlet-box-g3.chartlet-box,.chartlet-box-g3.chartlet-violin{fill:var(--chartlet-color-3);stroke:var(--chartlet-color-3)}.chartlet-box-g3.chartlet-box-whisker,.chartlet-box-g3.chartlet-box-outlier{stroke:var(--chartlet-color-3)}.chartlet-box-g3.chartlet-box-point{fill:var(--chartlet-color-3)}.chartlet-box-g4.chartlet-box,.chartlet-box-g4.chartlet-violin{fill:var(--chartlet-color-4);stroke:var(--chartlet-color-4)}.chartlet-box-g4.chartlet-box-whisker,.chartlet-box-g4.chartlet-box-outlier{stroke:var(--chartlet-color-4)}.chartlet-box-g4.chartlet-box-point{fill:var(--chartlet-color-4)}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}.chartlet-box-point{fill:var(--chartlet-accent);fill-opacity:.55;stroke:none}";
const RANGEBAR_STYLE: &str = ".chartlet-range{fill:var(--chartlet-accent);fill-opacity:.3;stroke:var(--chartlet-accent);stroke-width:1}.chartlet-range-hatch{stroke:none}.chartlet-hatch-line{stroke:var(--chartlet-accent);stroke-width:1.2;opacity:.75}.chartlet-range-mid{stroke:var(--chartlet-text);stroke-width:3}.chartlet-range-weight{fill:var(--chartlet-text);stroke:none}.chartlet-range-summary{fill:var(--chartlet-text);fill-opacity:.85;stroke:var(--chartlet-text);stroke-width:1}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";
/// Dumbbells of range bars: a line between an open dot at the low value and a filled one at the
/// high value, in the accent color or in the color of the span's group.
const DUMBBELL_STYLE: &str = ".chartlet-dumbbell-line{fill:none;stroke:var(--chartlet-accent);stroke-width:3;stroke-linecap:round;stroke-opacity:.6}.chartlet-dumbbell-low{fill:var(--chartlet-background);stroke:var(--chartlet-accent);stroke-width:2.5}.chartlet-dumbbell-high{fill:var(--chartlet-accent);stroke:var(--chartlet-accent);stroke-width:2.5}.chartlet-dumbbell-low.chartlet-range-series-1,.chartlet-dumbbell-low.chartlet-range-series-2,.chartlet-dumbbell-low.chartlet-range-series-3,.chartlet-dumbbell-low.chartlet-range-series-4{fill:var(--chartlet-background)}.chartlet-dumbbell-line.chartlet-range-series-1,.chartlet-dumbbell-line.chartlet-range-series-2,.chartlet-dumbbell-line.chartlet-range-series-3,.chartlet-dumbbell-line.chartlet-range-series-4{fill:none}";
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

/// Weighted edges of flow charts and architecture diagrams: one width per step of
/// [`crate::flow::WEIGHT_STEPS`], half a pixel apart, from 1 px to 5 px.
const WEIGHT_STYLE: &str = ".chartlet-flow-w0 .chartlet-flow-edge{stroke-width:1}.chartlet-flow-w1 .chartlet-flow-edge{stroke-width:1.5}.chartlet-flow-w2 .chartlet-flow-edge{stroke-width:2}.chartlet-flow-w3 .chartlet-flow-edge{stroke-width:2.5}.chartlet-flow-w4 .chartlet-flow-edge{stroke-width:3}.chartlet-flow-w5 .chartlet-flow-edge{stroke-width:3.5}.chartlet-flow-w6 .chartlet-flow-edge{stroke-width:4}.chartlet-flow-w7 .chartlet-flow-edge{stroke-width:4.5}.chartlet-flow-w8 .chartlet-flow-edge{stroke-width:5}";

/// Networks, on top of [`DIAGRAM_STYLE`] and [`FLOW_STYLE`]: dots in the color of their group,
/// labels beside them, and the rings around the center.
const NETWORK_STYLE: &str = ".chartlet-root{--chartlet-net-5:#0f766e;--chartlet-net-6:#6d28d9;--chartlet-net-7:#b45309;--chartlet-net-8:#be185d}.chartlet-root.chartlet-theme-dark{--chartlet-net-5:#2dd4bf;--chartlet-net-6:#b69cff;--chartlet-net-7:#fbbf24;--chartlet-net-8:#f472b6}.chartlet-net-node{fill:var(--chartlet-role-blue);stroke:var(--chartlet-background);stroke-width:2}.chartlet-net-g1{fill:var(--chartlet-color-1)}.chartlet-net-g2{fill:var(--chartlet-color-2)}.chartlet-net-g3{fill:var(--chartlet-color-3)}.chartlet-net-g4{fill:var(--chartlet-color-4)}.chartlet-net-g5{fill:var(--chartlet-net-5)}.chartlet-net-g6{fill:var(--chartlet-net-6)}.chartlet-net-g7{fill:var(--chartlet-net-7)}.chartlet-net-g8{fill:var(--chartlet-net-8)}.chartlet-net-center{stroke:var(--chartlet-text)}.chartlet-net-label{font-size:12px;font-weight:600;fill:var(--chartlet-text)}.chartlet-net-ring{fill:none;stroke:var(--chartlet-grid);stroke-width:1;stroke-dasharray:3 4}.chartlet-legend{font-size:12px;fill:var(--chartlet-muted)}";

/// State diagrams, on top of [`DIAGRAM_STYLE`] and [`FLOW_STYLE`]: states with well rounded
/// corners, the inner outline of a final state, and the initial dot.
const STATE_STYLE: &str = ".chartlet-state{rx:14px}.chartlet-state-inner{fill:none;stroke:var(--chartlet-role-green);stroke-width:1.5;rx:11px}.chartlet-state-initial{fill:var(--chartlet-text);stroke:var(--chartlet-background);stroke-width:2}";

/// Architecture diagrams, on top of [`DIAGRAM_STYLE`] and [`FLOW_STYLE`]: the head of a person
/// and the dots of a window bar.
/// Provided and required interfaces of architecture diagrams: the ball, its name, and the socket
/// in the color of its connection's role.
const INTERFACE_STYLE: &str = ".chartlet-arch-ball{fill:var(--chartlet-background);stroke:var(--chartlet-role-blue);stroke-width:1.5}.chartlet-arch-interface{font-size:11px;fill:var(--chartlet-text)}.chartlet-arch-socket{fill:none;stroke:var(--chartlet-zero);stroke-width:1.5;stroke-linecap:round}.chartlet-arch-socket-main{stroke:var(--chartlet-accent)}.chartlet-flow-role-critical .chartlet-arch-socket{stroke:var(--chartlet-role-red)}.chartlet-flow-role-planned .chartlet-arch-socket{stroke:var(--chartlet-role-violet)}.chartlet-flow-role-deprecated .chartlet-arch-socket{stroke:var(--chartlet-role-gray)}";
/// The socket of an interface connection in the color of its category, where the perspective shown
/// gives connections categories.
const INTERFACE_CATEGORY_STYLE: &str = ".chartlet-flow-part-1 .chartlet-arch-socket{stroke:var(--chartlet-color-1)}.chartlet-flow-part-2 .chartlet-arch-socket{stroke:var(--chartlet-color-2)}.chartlet-flow-part-3 .chartlet-arch-socket{stroke:var(--chartlet-color-3)}.chartlet-flow-part-4 .chartlet-arch-socket{stroke:var(--chartlet-color-4)}.chartlet-flow-part-5 .chartlet-arch-socket{stroke:var(--chartlet-part-5)}.chartlet-flow-part-6 .chartlet-arch-socket{stroke:var(--chartlet-part-6)}.chartlet-flow-part-7 .chartlet-arch-socket{stroke:var(--chartlet-part-7)}.chartlet-flow-part-8 .chartlet-arch-socket{stroke:var(--chartlet-part-8)}";
/// Cycles, on top of [`DIAGRAM_STYLE`] and [`FLOW_STYLE`]: the number in a step's dot, the hub and
/// its links.
const CYCLE_STYLE: &str = ".chartlet-cycle-num{font-size:13px;font-weight:700;fill:var(--chartlet-text)}.chartlet-cycle-hub{stroke-width:2.5}.chartlet-cycle-link{stroke-width:1.2;stroke-dasharray:2 4;stroke-linecap:round}";

/// Layers, on top of [`DIAGRAM_STYLE`] and [`FLOW_STYLE`]: the boxes of items, the dotted line to a
/// note, the shaded faces of an exploded stack and the odd scopes of nested ones, which keep the
/// background so that neighbouring scopes differ.
const LAYERS_STYLE: &str = ".chartlet-layers-item{fill:var(--chartlet-background);stroke:var(--chartlet-grid);stroke-width:1;rx:5px}.chartlet-layers-item-text{font-size:12px;fill:var(--chartlet-text)}.chartlet-layers-note{font-size:12px;fill:var(--chartlet-muted)}.chartlet-layers-leader{fill:none;stroke:var(--chartlet-zero);stroke-width:1;stroke-dasharray:1 3;stroke-linecap:round}.chartlet-layers-odd{fill:var(--chartlet-background)}.chartlet-layers-guide{stroke:var(--chartlet-zero);stroke-width:1;stroke-dasharray:2 4}.chartlet-layers-shade-left{fill:var(--chartlet-text);fill-opacity:.1;stroke:none}.chartlet-layers-shade-right{fill:var(--chartlet-text);fill-opacity:.2;stroke:none}";

/// Plans, on top of [`DIAGRAM_STYLE`] and [`FLOW_STYLE`]: floors by kind, low walls with shaded
/// outer faces, furniture, the leader lines of the labels and the marks of the phase patterns.
const PLAN_STYLE: &str = ".chartlet-plan-floor{stroke:var(--chartlet-zero);stroke-width:1;stroke-linejoin:round}.chartlet-plan-room{fill:var(--chartlet-role-blue-fill)}.chartlet-plan-building{fill:var(--chartlet-role-violet-fill);stroke:var(--chartlet-role-violet);stroke-width:2.5}.chartlet-plan-outdoor{fill:var(--chartlet-role-green-fill);stroke:var(--chartlet-role-green);stroke-dasharray:5 3}.chartlet-plan-wall{fill:var(--chartlet-role-blue-fill);stroke:var(--chartlet-zero);stroke-width:1;stroke-linejoin:round}.chartlet-plan-wall-in{fill:var(--chartlet-background);stroke:var(--chartlet-zero);stroke-width:1;stroke-linejoin:round}.chartlet-plan-rim{fill:none;stroke:var(--chartlet-text);stroke-width:1.5;stroke-linejoin:round}.chartlet-plan-rim-building{stroke-width:3}.chartlet-plan-item{fill:var(--chartlet-role-amber-fill);stroke:var(--chartlet-role-amber);stroke-width:1;stroke-linejoin:round}.chartlet-plan-shade-left{fill:var(--chartlet-text);fill-opacity:.1;stroke:none}.chartlet-plan-shade-right{fill:var(--chartlet-text);fill-opacity:.2;stroke:none}.chartlet-plan-leader{fill:none;stroke:var(--chartlet-zero);stroke-width:1}.chartlet-plan-dot{fill:var(--chartlet-text)}.chartlet-plan-swatch{fill:var(--chartlet-role-blue-fill);stroke:var(--chartlet-zero);stroke-width:1}.chartlet-plan-phase-1,.chartlet-plan-phase-2,.chartlet-plan-phase-3{stroke:none}.chartlet-plan-mark{stroke:var(--chartlet-text);stroke-opacity:.55;stroke-width:1.2}.chartlet-plan-mark-dot{fill:var(--chartlet-text);fill-opacity:.55}";

/// The delta of an architecture model: the headings of the two panels, the rule between them and
/// the ledger below.
const DELTA_STYLE: &str = ".chartlet-delta-heading{font-size:14px;font-weight:700;fill:var(--chartlet-text)}.chartlet-delta-entry{font-size:12px;fill:var(--chartlet-text)}.chartlet-delta-sign{font-size:14px;font-weight:700;fill:var(--chartlet-text)}.chartlet-delta-rule{stroke:var(--chartlet-grid);stroke-width:1;stroke-dasharray:3 4}";

const ARCHITECTURE_STYLE: &str = ".chartlet-arch-head{fill:var(--chartlet-role-violet-fill);stroke:var(--chartlet-role-violet);stroke-width:1.5}.chartlet-arch-dot{fill:var(--chartlet-role-blue);stroke:none}";

const FILTER_STYLE: &str = ".chartlet-wrapper{display:inline-block;max-width:100%}.chartlet-filter{border:none;padding:0;margin:0 0 12px 0}.chartlet-filter legend{font-size:14px;font-weight:650;margin-bottom:4px}.chartlet-filter label{display:inline-flex;align-items:center;min-height:44px;font-size:13px;margin-right:14px;cursor:pointer;white-space:nowrap}.chartlet-filter input{margin-right:4px}.chartlet-filter input:focus-visible{outline:2px solid #2563eb;outline-offset:2px}.chartlet-wrapper:has(.chartlet-filter input.series-0:not(:checked)) .chartlet-root [data-series=\"0\"]{display:none}.chartlet-wrapper:has(.chartlet-filter input.series-1:not(:checked)) .chartlet-root [data-series=\"1\"]{display:none}.chartlet-wrapper:has(.chartlet-filter input.series-2:not(:checked)) .chartlet-root [data-series=\"2\"]{display:none}.chartlet-wrapper:has(.chartlet-filter input.series-3:not(:checked)) .chartlet-root [data-series=\"3\"]{display:none}";

/// The class by which the shared stylesheet addresses a chart type.
pub(super) fn type_class(chart_type: ChartType) -> String {
    format!("chartlet-type-{}", crate::spec::type_name(chart_type))
}

/// The stylesheets of the diagram types: what they share, then what each adds.
fn diagram_style(chart_type: ChartType) -> String {
    let parts: &[&str] = match chart_type {
        ChartType::Sequence => &[DIAGRAM_STYLE, SEQUENCE_STYLE],
        ChartType::Flow => &[DIAGRAM_STYLE, FLOW_STYLE],
        ChartType::State => &[DIAGRAM_STYLE, FLOW_STYLE, STATE_STYLE],
        ChartType::Classes => &[DIAGRAM_STYLE, FLOW_STYLE, CLASSES_STYLE],
        ChartType::Network => &[DIAGRAM_STYLE, FLOW_STYLE, NETWORK_STYLE],
        ChartType::Cycle => &[DIAGRAM_STYLE, FLOW_STYLE, CYCLE_STYLE],
        ChartType::Layers => &[DIAGRAM_STYLE, FLOW_STYLE, LAYERS_STYLE],
        ChartType::Plan => &[DIAGRAM_STYLE, FLOW_STYLE, PLAN_STYLE],
        ChartType::Architecture | ChartType::Tree => {
            &[DIAGRAM_STYLE, FLOW_STYLE, ARCHITECTURE_STYLE]
        }
        ChartType::Fishbone
        | ChartType::Venn
        | ChartType::Kanban
        | ChartType::Storymap
        | ChartType::Journey
        | ChartType::Valuemap => &[DIAGRAM_STYLE],
        _ => &[],
    };
    parts.concat()
}

/// The style of the chart types that have a style group of their own and need nothing else.
const fn type_style(chart_type: ChartType) -> &'static str {
    match chart_type {
        ChartType::Boxplot => BOXPLOT_STYLE,
        ChartType::Timeline => TIMELINE_STYLE,
        ChartType::Waterfall => WATERFALL_STYLE,
        ChartType::Waffle => WAFFLE_STYLE,
        ChartType::Funnel => FUNNEL_STYLE,
        ChartType::Radar => RADAR_STYLE,
        ChartType::Polar => POLAR_STYLE,
        ChartType::Mosaic => MOSAIC_STYLE,
        ChartType::Heatmap => HEATMAP_STYLE,
        ChartType::Fishbone => FISHBONE_STYLE,
        ChartType::Venn => VENN_STYLE,
        ChartType::Kanban => KANBAN_STYLE,
        ChartType::Storymap => STORY_STYLE,
        ChartType::Journey => JOURNEY_STYLE,
        ChartType::Valuemap => VMAP_STYLE,
        ChartType::Parliament => PARLIAMENT_STYLE,
        ChartType::Treemap => TREEMAP_STYLE,
        ChartType::Sankey => SANKEY_STYLE,
        ChartType::Survival => SURVIVAL_STYLE,
        ChartType::Scatter => SCATTER_STYLE,
        _ => "",
    }
}

/// The style groups a chart uses, in the order of [`shared_stylesheet`], so that rules cascade
/// alike in a chart's own stylesheet and in the shared one. The print variant has no series
/// filter to style.
#[allow(clippy::too_many_lines)]
pub(super) fn base_style(spec: &ChartSpec, print: bool) -> String {
    let has_series = spec.series.len() > 1 || spec.data_groups().len() > 1;
    let is_time = matches!(spec.chart_type, ChartType::Time | ChartType::Multiples)
        && !spec.is_bar_multiples();
    // A categorical line chart with several series draws its lines in the palette and patterns of
    // a time chart's lines.
    let several_lines = spec.chart_type == ChartType::Line && has_series;
    let is_multiples = spec.chart_type == ChartType::Multiples;
    let is_diverging = matches!(spec.chart_type, ChartType::Stripes | ChartType::Calendar);
    let is_calendar = spec.chart_type == ChartType::Calendar;
    let is_rangebar = spec.chart_type == ChartType::Rangebar;
    let is_dark = matches!(spec.theme, Theme::Dark | Theme::Blueprint);
    let diagram = diagram_style(spec.chart_type);
    let roles = if spec.has_roles() {
        crate::roles::style(spec.chart_type)
    } else {
        String::new()
    };
    // The groups in their order; a group that does not apply is left out.
    [
        STYLE,
        when(is_dark, DARK_STYLE),
        when(has_series, SERIES_STYLE),
        when(has_series && !print, FILTER_STYLE),
        when(is_time || several_lines, LINE_SERIES_STYLE),
        when(
            (is_time && needs_layer_extras(spec)) || several_lines,
            LAYER_EXTRA_STYLE,
        ),
        when(
            (is_time && needs_mark_extras(spec)) || several_lines,
            MARK_EXTRA_STYLE,
        ),
        when(
            is_time && spec.layers().any(|layer| layer.stroke == Stroke::Medium),
            MEDIUM_LINE_STYLE,
        ),
        when(
            is_time && needs_annotation_extras(spec),
            ANNOTATION_EXTRA_STYLE,
        ),
        when(is_multiples, MULTIPLES_STYLE),
        when(
            is_multiples && spec.panes.iter().any(|pane| pane.note.is_some()),
            PANEL_NOTE_STYLE,
        ),
        when(is_diverging, crate::diverging::STYLE),
        when(is_calendar, CALENDAR_STYLE),
        when(is_rangebar, RANGEBAR_STYLE),
        when(
            is_rangebar && spec.display == RangeDisplay::Dumbbell,
            DUMBBELL_STYLE,
        ),
        type_style(spec.chart_type),
        when(
            spec.scatter
                .as_ref()
                .is_some_and(crate::spec::ScatterSpec::is_sized),
            SCATTER_BUBBLE_STYLE,
        ),
        when(spec.has_error_bars(), ERROR_STYLE),
        when(
            is_rangebar && spec.ranges.iter().any(|range| range.group.is_some()),
            RANGE_GROUP_STYLE,
        ),
        when(!spec.references.is_empty(), REFERENCE_STYLE),
        when(spec.patterns, OUTLINE_STYLE),
        when(spec.width < crate::layout::NARROW, SMALL_TITLE_STYLE),
        &diagram,
        // Before the roles: a role keeps its color on a connection in a category.
        when(
            spec.has_connection_categories(),
            crate::parts::CONNECTION_STYLE,
        ),
        &roles,
        when(spec.has_edge_weights(), WEIGHT_STYLE),
        when(spec.has_node_values(), crate::heat::STYLE),
        when(
            spec.has_node_parts() || spec.has_perspective(),
            crate::parts::STYLE,
        ),
        when(spec.has_perspective(), crate::parts::UNSTATED_STYLE),
        when(spec.has_component_symbols(), crate::symbols::STYLE),
        when(spec.has_interfaces(), INTERFACE_STYLE),
        when(
            spec.has_interfaces() && spec.has_connection_categories(),
            INTERFACE_CATEGORY_STYLE,
        ),
        when(spec.has_delta(), DELTA_STYLE),
        when(spec.theme == Theme::Blueprint, BLUEPRINT_STYLE),
        when(
            is_time && spec.layers().any(|layer| layer.mark == Mark::Ohlc),
            OHLC_STYLE,
        ),
    ]
    .concat()
}

/// `style` where `condition` holds, nothing otherwise.
const fn when(condition: bool, style: &str) -> &str {
    if condition { style } else { "" }
}

/// Every style group for each of `chart_types` that can use it, in the order of [`base_style`].
/// The light palette and the dark theme apply to every chart; every other group is scoped to the
/// chart types that use it, so that two groups styling the same class, such as the hatching of a
/// time chart and of a range bar chart, never meet.
/// The shared stylesheet: with `common`, first the part every chart relies on, then the part of
/// each of `chart_types` in their order. A type's part styles only charts of that type, so the
/// parts of two types never style the same element and may be concatenated in any order.
#[allow(clippy::too_many_lines)]
pub(crate) fn shared_stylesheet(chart_types: &[ChartType], common: bool) -> String {
    use ChartType::{
        Architecture, Bar, Boxplot, Calendar, Classes, Cycle, Fishbone, Flow, Funnel, Heatmap,
        Journey, Kanban, Layers, Line, Mosaic, Multiples, Network, Parliament, Plan, Polar, Radar,
        Rangebar, Sankey, Scatter, Sequence, State, Storymap, Stripes, Survival, Time, Timeline,
        Tree, Treemap, Valuemap, Venn, Waffle, Waterfall,
    };
    let groups: &[(&str, &[ChartType])] = &[
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
        (DUMBBELL_STYLE, &[Rangebar]),
        (BOXPLOT_STYLE, &[Boxplot]),
        (TIMELINE_STYLE, &[Timeline]),
        (ERROR_STYLE, &[Bar]),
        (RANGE_GROUP_STYLE, &[Rangebar]),
        (REFERENCE_STYLE, &[Bar, Rangebar]),
        (OUTLINE_STYLE, &[Bar]),
        (
            DIAGRAM_STYLE,
            &[
                Sequence,
                Flow,
                State,
                Architecture,
                Tree,
                Classes,
                Network,
                Cycle,
            ],
        ),
        (SEQUENCE_STYLE, &[Sequence]),
        (
            FLOW_STYLE,
            &[
                Flow,
                State,
                Architecture,
                Tree,
                Classes,
                Network,
                Cycle,
                Layers,
            ],
        ),
        (STATE_STYLE, &[State]),
        (CLASSES_STYLE, &[Classes]),
        (ARCHITECTURE_STYLE, &[Architecture, Tree]),
        (crate::parts::CONNECTION_STYLE, &[Architecture]),
        crate::roles::STYLE_GROUPS[0],
        crate::roles::STYLE_GROUPS[1],
        crate::roles::STYLE_GROUPS[2],
        (WEIGHT_STYLE, &[Flow, Architecture, Network]),
        (NETWORK_STYLE, &[Network]),
        (CYCLE_STYLE, &[Cycle]),
        (LAYERS_STYLE, &[Layers]),
        (PLAN_STYLE, &[Plan]),
        (crate::heat::STYLE, &[Flow, Tree]),
        (crate::parts::STYLE, &[Flow, Tree, Architecture]),
        (crate::parts::UNSTATED_STYLE, &[Architecture]),
        (crate::symbols::STYLE, &[Architecture]),
        (INTERFACE_STYLE, &[Architecture]),
        (INTERFACE_CATEGORY_STYLE, &[Architecture]),
        (DELTA_STYLE, &[Architecture]),
        (
            BLUEPRINT_STYLE,
            &[
                Sequence,
                Flow,
                State,
                Architecture,
                Tree,
                Classes,
                Network,
                Cycle,
            ],
        ),
        (WATERFALL_STYLE, &[Waterfall]),
        (WAFFLE_STYLE, &[Waffle]),
        (FUNNEL_STYLE, &[Funnel]),
        (RADAR_STYLE, &[Radar]),
        (POLAR_STYLE, &[Polar]),
        (MOSAIC_STYLE, &[Mosaic]),
        (HEATMAP_STYLE, &[Heatmap]),
        (
            DIAGRAM_STYLE,
            &[Fishbone, Venn, Kanban, Storymap, Journey, Valuemap],
        ),
        (FISHBONE_STYLE, &[Fishbone]),
        (VENN_STYLE, &[Venn]),
        (KANBAN_STYLE, &[Kanban]),
        (STORY_STYLE, &[Storymap]),
        (JOURNEY_STYLE, &[Journey]),
        (VMAP_STYLE, &[Valuemap]),
        (PARLIAMENT_STYLE, &[Parliament]),
        (TREEMAP_STYLE, &[Treemap]),
        (SANKEY_STYLE, &[Sankey]),
        (SURVIVAL_STYLE, &[Survival]),
        (SCATTER_STYLE, &[Scatter]),
        (SCATTER_BUBBLE_STYLE, &[Scatter]),
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
/// together with the rules left empty by that.
pub(super) fn print_stylesheet(
    stylesheet: &str,
    (is_dark, is_blueprint): (bool, bool),
    tokens: &BTreeMap<String, String>,
) -> String {
    // Renderers outside the browser read font weights in hundreds only and would draw 650 as
    // regular.
    let stylesheet = &stylesheet.replace("font-weight:650", "font-weight:600");
    let mut properties: Vec<(&str, &str)> = Vec::new();
    for rule in stylesheet.split_inclusive('}') {
        let Some((selector, body)) = rule.split_once('{') else {
            continue;
        };
        if selector != ".chartlet-root"
            && !(is_dark && selector == ".chartlet-theme-dark")
            && !(is_blueprint && selector == ".chartlet-blueprint")
        {
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
    // The chart's own tokens replace what its theme gives the property.
    let lookup = |name: &str| {
        name.strip_prefix("--chartlet-")
            .and_then(|token| tokens.get(token))
            .map(String::as_str)
            .or_else(|| {
                properties
                    .iter()
                    .find(|(known, _)| *known == name)
                    .map(|(_, value)| *value)
            })
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
pub(super) fn scope_stylesheet(stylesheet: &str, scope: &str, print: bool) -> String {
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
    const ROOT_CLASSES: [&str; 4] = [
        ".chartlet-root",
        ".chartlet-theme-dark",
        ".chartlet-blueprint",
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
        (true, false) if selector == ".chartlet-blueprint" => {
            ".chartlet-root.chartlet-blueprint".to_owned()
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
pub(super) fn layer_style(spec: &ChartSpec) -> String {
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
