---
title: Support matrix
description: What has been verified, what is designed but not yet tested, and what is planned or out of scope.
order: 4
---

chartlet is before 1.0; screen reader and user testing are still pending. Statuses: **Verified**, **Experimental** (works, but may change between minor releases), **Designed, not yet verified**, **Planned**,
**Not supported**.

## Embedding

| Context | Status | Notes |
| --- | --- | --- |
| Standalone SVG | Verified | Byte-identical on macOS and Linux; covered by the golden-file tests. |
| HTML figure (caption, source, data table) | Verified | The data table is always present, in a native disclosure by default. |
| Astro component | Verified | Renders at build time and ships no chart JavaScript to the browser. |
| CLI and Rust API | Verified | The same renderer backs the CLI, the crate, and the npm package. |
| Mobile variant (`mobile`) | Verified | HTML profile: both variants in one wrapper, switched by a container query; the hidden one is `display:none`. Checked in Chromium at a narrow and a wide container, with zoom steps and series filter. Browsers without container queries keep the full-size chart. |
| `<img src="chart.svg">` | Designed, not yet verified | Title and description survive; the figure wrapper and data table do not apply. |
| Markdown | Planned | Not yet evaluated. |
| PDF and print (`--variant print`) | Verified | SVG with literal colors. Checked with resvg through Typst 0.15 (PNG and PDF) and with WebKit for bars, lines, a dark chart with declared colors, hatched bands, range bars, candlesticks, and a calendar heatmap. librsvg, Inkscape, Prince and wkhtmltopdf not yet tested. |
| E-mail clients | Planned | Not yet evaluated. |

## Browsers

| Context | Status | Notes |
| --- | --- | --- |
| Chromium (Chrome, Edge) | Verified | Automated accessibility scan (axe) and manual review during development. |
| Firefox | Planned | Standard SVG is expected to work. The Firefox build of Playwright did not start on the development machine, so the accessibility tree is not yet tested. |
| Safari (WebKit) | Verified for rendering | Checked 2026-10-12 with the WebKit build of Playwright (the engine of Safari, without Safari itself): all 76 example HTML profiles draw, every chart is a `role="img"` with a name and a description, the data table is present, the series filter hides series and the container query switches to the phone variant, with the same results as in Chromium. Not checked with VoiceOver. |

## Screen readers

| Context | Status | Notes |
| --- | --- | --- |
| VoiceOver (macOS) | Planned | The title and description are exposed as the accessible name and description; not yet tested. |
| NVDA (Windows) | Planned | Not yet tested. |
| JAWS (Windows) | Planned | Not yet tested. |

## Themes

| Context | Status | Notes |
| --- | --- | --- |
| Light (default) | Verified | The default palette keeps at least 4.5:1 contrast on white (lowest series color 5.17:1). |
| Dark | Verified | `"theme": "dark"` sets the palette through the same CSS custom properties and paints its own background. Lowest series color 7.65:1 against `#0e131c`. |
| Own colors (`tokens`, theme file) | Verified | Hex and `rgb()` values reach the stylesheet, the print and social variants and PNG. Pairs of colors that lie on top of each other are measured, and a ratio below WCAG 2.2's is reported as `low_contrast`; other chart types' scales are not measured. See [Customize the look](../../guides/customize/#contrast). Covered by unit tests. |
| Forced colours / high contrast | Planned | Planned; not yet shipped. |
| Host-page colors (`var(--name)` per layer) | Verified | A `var()` reference is passed through and falls back to the chart's text color when the page defines nothing. |

## Time series

| Context | Status | Notes |
| --- | --- | --- |
| Calendar axis (ticks, timezone) | Verified | Ticks snap to calendar boundaries; a fixed UTC offset such as `"+02:00"` shifts the labels and the data table. Covered by unit and golden-file tests. |
| Line on the time axis | Verified | Up to six layers of 2000 observations per pane, at most four of the chart in palette colors; markers and value labels up to 60 observations per layer. The legend wraps into further rows. |
| Dense series | Verified | More observations than plot pixels is reported as `dense_chart`; measured at 131 KB of SVG for four layers of 2000 points and 196 KB for six. |
| Uncertainty band and modeled layers | Verified | `lower`/`upper` per point draw a band in the line's color; `"modeled": true` dashes the line, hatches the band and says “modelled” in legend, description and table. Covered by unit and golden-file tests. |
| Reference lines (`annotation`) | Verified | Horizontal at `value` or vertical at `time`. A label that overlaps another annotation label, crosses a data line or leaves the plot is reported as `label_overlap`; it is not moved. |
| Zones (`band`) | Verified | A shaded range between `bottom` and `top`, `from` and `to`, or both, behind the data, with its label; described, no legend entry or table column. Covered by unit and golden-file tests. |
| Point markers (`annotation` with `time` and `value`) | Verified | Five shapes, a label beside the symbol that tries all four sides before `label_overlap` is reported, and a native tooltip. Zones, reference lines and markers share the limit of six per pane. |
| Small multiples | Verified | `"type": "multiples"`: 2–12 panes with a shared value axis and one legend. |
| German texts and numbers (`"locale": "de"`) | Verified | Every chart type. |
| `area` mark | Verified | Filled down to zero in the layer's color; the value axis includes zero. Covered by unit and golden-file tests. |
| Line patterns and weights (`dash`, `stroke`) | Verified | `solid`, `dashed`, `dotted`; `thin`, `regular`, `bold`; the legend sample shows both. |
| Gaps (`null`) on a time chart | Verified | Break line, area and band; counted as missing in description and table. Also in small multiples. |
| Gaps in time (`timeAxis.gaps: "collapse"`) | Verified | Every observed timestamp takes one evenly spaced slot, so weekends and holidays take no space; ticks sit on the first observation after each calendar boundary, annotations on the slot of an observation. Also in small multiples and zoom windows. Covered by unit and golden-file tests. |
| `zoomSteps` on a time chart | Verified | Windows by timestamp, HTML profile only; same radio-and-CSS mechanism as category zoom. |
| `ohlc` mark (candlesticks) | Verified | Wick from low to high, body from open to close; rising hollow, falling filled, in `--chartlet-rise` and `--chartlet-fall`. Beyond one candle per three plot pixels only the wicks are drawn and `dense_chart` is reported. Tooltips, table columns and description per candle layer. Covered by unit and golden-file tests. |
| Several panes on a `time` chart | Verified | Up to four panes stacked on one shared time axis, each with its own value axis and a share of the height by `heightRatio`; one legend with names unique across the chart. Covered by unit and golden-file tests. |
| `zoomSteps` on small multiples | Planned | Refused with `option_not_supported`. |

## Other chart types

| Context | Status | Notes |
| --- | --- | --- |
| Warming stripes, calendar heatmap | Verified | Shared diverging scale of 17 colors as CSS custom properties; covered by unit and golden-file tests. |
| Bars colored by group, small multiples of bars | Verified | `group` on `data` points (one color and legend entry per group, a Group column in the table) and `multiples` with `categories` and a `values` array and value axis per pane; covered by unit tests of validation, text and rendering and by golden-file tests, including the phone variant. |
| Box plots, error bars on bars | Verified | Quartiles by linear interpolation, whiskers within 1.5 boxes, outliers; unit tests of the statistics, validation, text and rendering in both orientations, and golden-file tests. |
| Forest plots, timelines | Verified | `weight`, `summary` and reference lines on range bars, and the `timeline` type with phases, milestones, markers, arrows and groups; unit tests of validation, text, arrows and rendering, and golden-file tests including the phone variant. |
| Waterfalls | Verified | Deltas, starts and totals with a check of the totals, both orientations; unit tests of the running total, text and validation, and golden-file tests including the phone variant. |
| Heatmaps | Verified | Sequential and diverging scales with text colors that keep 4.5:1, categories with fill and symbol, missing cells, ordering by label or value; unit tests of the cells, text, ordering and validation, and golden-file tests including the phone variant. |
| Cycle diagrams | Verified | Three to eight steps on a ring with the labels outside beside numbered dots or inside boxes, arrows along the ring, a hub with labelled links, roles and focus links; unit tests of the text and validation, a generated-specification test, and golden-file tests including the phone variant. |
| Layer stacks and nested scopes | Verified | Bands from the bottom up and frames inside each other, with items wrapped onto rows, notes at the side, roles and focus links; unit tests of the text, the geometry and validation, and golden-file tests including the phone variant. |
| Waffle charts | Verified | Apportioning of the squares (largest remainder, one square at least), the rest of a total, legend beside or below; unit tests of the squares, text and validation, and golden-file tests including the phone variant. |
| Scatter plots | Verified | Groups, threshold lines, collision-free names, small dots and a limited table for many points; unit tests of the text, the table limit and validation, and golden-file tests including the phone variant. |
| Survival curves | Verified | Kaplan-Meier estimate with Greenwood confidence limits, censoring marks, number at risk; unit tests of the estimate, text and validation, and golden-file tests including the phone variant. |
| Regression, log axes, log-rank test, pinned Sankey nodes, nested treemaps, error bars on lines | Verified | Least squares on the logarithms for log axes, the Mantel-Cox test checked against an independent implementation, `column_too_early`, nested frames, line error bars; unit tests of each and golden-file tests including the phone variants. |
| Timeline summary phases and arrow routing, Sankey flows back | Verified | `rollup` weighted by length, arrows that turn beside bars in the way, loops for flows back; unit tests of each and golden-file tests. |
| Class and entity-relationship diagrams | Experimental | The layered layout of flow charts with class boxes and the marks of six relation kinds; unit tests of the text and validation and golden-file tests; like the flow chart it stands on, experimental. |
| Error bars on several series, timeline time zones | Verified | `lower`/`upper` on each series of grouped bars and lines with an interval column per series; a fixed-offset zone for timeline days, ticks and weekends; unit tests of both and golden-file tests including the phone variant. |
| Grouped box plots, shaded timeline weekends | Verified | Palette colors and a legend by group on boxes, violins and strips; Saturdays and Sundays as columns on axes of up to 120 days; unit tests of both and golden-file tests including the phone variant. |
| Mean marker, violin scales, hazard ratio, timeline progress and grouped rows, named scatter ticks and ranges | Verified | The hazard ratio checked against an independent implementation, shared densities for violin areas, row order by group; unit tests of each and golden-file tests including the phone variants. |
| Flow presets (PRISMA, Instanzenzug) | Experimental | Examples on the flow chart, which is itself experimental. |
| Violins and strips | Verified | Kernel density with Silverman's bandwidth, fixed jitter; unit tests of both displays and validation, and golden-file tests including the phone variant. |
| Beeswarms and ridgelines | Verified | Dots placed from the lowest value on, each nearest the middle without touching another (checked for every pair); density curves on one shared scale; unit tests of both displays and validation, and golden-file tests including the phone variant. |
| Sankey diagrams | Verified | Columns by the longest path, barycenter ordering, bands with eased edges; unit tests of the text and validation, and golden-file tests including the phone variant. |
| Treemaps | Verified | Squarified rectangles with groups, labels where they fit, legend; unit tests of the layout text and validation, and golden-file tests including the phone variant. |
| Parliament charts | Verified | Seats in a semicircle with the number of rows chosen for the biggest dots, the line of the majority and a coalition; unit tests of the seats, text and validation, and golden-file tests including the phone variant. |
| Streamgraphs | Verified | Areas centered on a moving baseline with a scale bar for thickness and names in the middle of each band; unit tests of the baseline, text and validation, and golden-file tests including the phone variant. |
| Range bars | Verified | Vertical and horizontal, optional central value and hatched modeled ranges; dumbbells with values at both ends and ordering by change, covered by unit tests of layout, text and validation and golden-file tests including the phone variant. |
| Sequence and architecture diagrams | Verified | Laid out by chartlet; covered by unit tests of layout invariants (no overlapping boxes, main path straight, frames clear, no shortened names), and golden-file tests for portrait, landscape and the phone variant. Known limits: on a phone, five participants or a wide architecture still need 400–470 pixels (the canvas grows and says so); in landscape, edges that leave one component side by side keep their labels close to the component rather than on their own stretch. |
| Tree diagrams | Verified | Laid out by chartlet (subtrees pushed together, parents centered over their children, couples side by side); covered by unit tests of layout invariants (no overlapping boxes at 136 nodes with long names, parents centered, couples on one row, no shortened names) and golden-file tests for portrait, landscape and the phone variant. |
| Flow and state diagrams | Experimental | Laid out by chartlet on the same engine; covered by unit tests of layout invariants and golden-file tests. The layout may change between minor releases. |
| Diagram focus | Designed, not yet verified | Radio buttons and labels over the nodes, CSS `:has()`; checked with axe, not yet with screen readers. |

## Distribution and interactions

| Context | Status | Notes |
| --- | --- | --- |
| WebAssembly build in Node.js | Verified | The npm package renders through it by default; tests compare every example with the CLI output byte for byte on Node.js 22. |
| WebAssembly build in Cloudflare Workers | Verified | `@casoon/chartlet/wasm` with an imported `chartlet.wasm`; checked locally in workerd, not in a deployed Worker. |
| WebAssembly build in the browser | Designed, not yet verified | Same entry point; the module is compiled by the page. |
| MCP server (`@casoon/chartlet-mcp`) | Designed, not yet verified | stdio server with `chartlet_inspect_data`, `chartlet_validate_spec`, `chartlet_render`, `chartlet_explain`, `chartlet_inspect`, `chartlet_diagram_starter`, `chartlet_chart_starter`, the schema as a resource and the MCP App view `ui://chartlet/figure`; covered by unit tests, an end-to-end test with the reference MCP client and a browser test of the view with axe, not yet tried in an MCP host such as Claude Desktop or Claude Code. |
| Native Node bindings | Not supported | The npm package renders through the WebAssembly build instead; `CHARTLET_BIN` selects the CLI. |
| Series filtering and stepped zoom | Designed, not yet verified | Available in the HTML profile with native controls and CSS; broader browser and assistive-technology verification is pending. |
| Mark tooltips | Designed, not yet verified | Native SVG titles provide hover hints; the description and table remain the accessible alternatives. |

## Versioning and compatibility

chartlet follows [Semantic Versioning](https://semver.org/) and is before 1.0, so the contract
below is the one that applies to 0.x; the [changelog](https://github.com/casoon/chartlet/blob/main/CHANGELOG.md)
lists every change by release.

**One release, one version.** The crate, the CLI, `@casoon/chartlet` and `@casoon/chartlet-mcp` of
a release carry the same version number, and the schema in `schema/` belongs to it. The packages do
not need identical versions at run time: `@casoon/chartlet-mcp` depends on `@casoon/chartlet` with a
caret range, which before 1.0 means the same minor (`^0.21.0` accepts `0.21.x`). Its bundled schema
and the renderer are therefore always from the same minor, and a patch of the renderer never
changes the specification.

**Reproducibility.** A chart is a function of the compiler, the specification and the options that
reach the renderer (`format`, `variant`, `table`, `theme`, `view`, `perspective`, `idPrefix`), plus
the shipped stylesheet (`chartlet.css`, for `styles: "external"`).
Pin the chartlet version exactly, keep the specification and options in version control, and
`--manifest` records the version, `specHash` and `outputHash` to check it in CI (see
[Provenance](../../guides/warnings-and-errors/#provenance)).

**Byte stability.** Within a release, and across patch releases, the same compiler, specification
and options give the same bytes, on macOS and Linux, through the CLI, the crate and the WebAssembly
build alike; the golden-file tests (`tests/golden.rs`) enforce it. Between minor releases, output
may change on purpose: layouts, spacing, text of descriptions, stylesheet details. Such changes are
listed in the changelog, and the golden files of the new release show them. Re-render and review the
diff when you move to a new minor; do not expect committed output to stay identical.

**Supported runtimes.**

| Runtime | Supported |
| --- | --- |
| Node.js (`@casoon/chartlet`, `@casoon/chartlet-mcp`) | 22.12 or newer (`engines`); the tests run on Node.js 22. No Rust toolchain is needed. |
| Rust (crate, CLI) | 1.88 or newer (`rust-version`); development and CI use the version in `rust-toolchain.toml`. |
| Astro (`@casoon/chartlet/astro`) | 6.0 or newer (optional peer dependency). |
| Cloudflare Workers, browsers | Through `@casoon/chartlet/wasm`; see the status in the Distribution table above. |

**Experimental features.** Everything marked Experimental above (flow, state, class and
entity-relationship diagrams and the flow presets) may change its specification or its layout in any
minor release, and says so in the changelog.

**Before 1.0.** A minor release (0.21 to 0.22) may change the specification, remove a type or an
option, and change output; a patch release (0.21.0 to 0.21.1) fixes defects only. Pin `0.21.x`
(`~0.21.0`, or an exact version for reproducible output) and move between minors deliberately. The
`schemaVersion` of the specification stays `1` until a change cannot be expressed in it, and a
different value is refused with `unsupported_schema_version`.

**Migration policy.** A change that makes a valid specification invalid is listed under
**Breaking** in the changelog with what replaces it. A removed field or type is refused with
`invalid_spec` and the JSON Pointer of the field, never silently ignored, so a failed render after an
upgrade tells you where to look. Upgrade in this order:

1. Read the changelog entries between your version and the target, **Breaking** first.
2. Raise the pin of the packages (and the crate) together.
3. Render every specification (`chartlet render`, or `renderChartDetailed` in a script) and fix each `invalid_spec` at its path; add `--strict` to fail on warnings as well.
4. Re-render, review the output diff, and commit specifications and output together.

Example, from 0.20 to 0.21: 0.21.0 removed the experimental types `topicmap` and `atlas`. After
`npm install @casoon/chartlet@~0.21.0`, a specification with `"type": "topicmap"` fails with
`invalid_spec` at `/type`; remodel it as a `network` or `tree` diagram, then re-render. Every other
specification renders as before, and committed output is expected to differ only where the changelog
names a layout change.
