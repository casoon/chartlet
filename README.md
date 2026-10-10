# chartlet

chartlet compiles a small JSON specification into a finished, accessible chart or software
diagram at build time. You get plain SVG, or an HTML figure with caption, source and data table.
Nothing runs in the browser: no chart JavaScript, no hydration, no layout shift.

**Website and documentation:** [casoon.github.io/chartlet](https://casoon.github.io/chartlet/)

![Grouped bar chart comparing budget and actual costs from January to April](examples/budget-vs-actual.svg)

![Flow chart of a release process in three lanes, laid out by chartlet](examples/release-flow.svg)

- **Accessible by default:** every chart carries a title and a generated description of the
  data, and the HTML output always includes the complete data as a table.
- **Deterministic:** the same specification produces the same bytes, so charts can be reviewed,
  diffed and cached like any other build artifact.
- **Honest about problems:** invalid input is rejected with a code, a path and a fix; layout
  compromises such as shortened labels are reported as warnings instead of happening silently.
- **Diagrams laid out for you:** sequence, flow, state, architecture and tree diagrams come from a
  description of what is connected; chartlet places every box and routes every edge, in portrait
  or landscape, and lists the whole structure in the data table.

> **Status:** 0.12. Bar charts (single, grouped, colored by group and stacked, vertical and horizontal),
> categorical line charts, time series with uncertainty bands, reference lines, points and stacked
> areas, warming stripes, calendar heatmaps, range bars, box plots and small multiples (of time series and of
> bars) are supported, waterfalls, waffle charts, parliament charts, treemaps, Sankey diagrams, Kaplan-Meier curves, scatter plots and timelines, and sequence, architecture and tree diagrams, plus experimental flow and state diagrams. Until 1.0, a minor release (0.12, 0.13, …) may still change the specification; a
> patch release never does.

## Quick start

Install the CLI from crates.io (Rust 1.88 or newer):

```sh
cargo install chartlet
```

Save a minimal specification as `spec.json`:

```json
{
  "schemaVersion": 1,
  "type": "bar",
  "title": "Monthly revenue",
  "data": [
    { "label": "January", "value": 120 },
    { "label": "February", "value": 180 },
    { "label": "March", "value": 150 }
  ]
}
```

Render it:

```sh
chartlet render spec.json --format html -o chart.html
```

To build from a clone of this repository instead, run `cargo install --path .`.

## Website and documentation

The [project website](https://casoon.github.io/chartlet/) has the documentation, every example
chart with its specification next to the rendered output, and the support matrix for embedding
contexts, browsers, and screen readers.

The site is built with Astro on the shared CASOON Pages theme and renders every chart with
`@casoon/chartlet` at build time. Its source lives in [`site/`](site/), the documentation in
[`docs/`](docs/).

## Chart types

| Chart | Specification | Example |
|---|---|---|
| Bar, vertical | `"type": "bar"` with `data` | [monthly-revenue](examples/monthly-revenue.json) |
| Bar, horizontal, with negative values | `"orientation": "horizontal"` | [quarterly-change](examples/quarterly-change.json) |
| Grouped bar, up to four series | `categories` and `series` instead of `data` | [budget-vs-actual](examples/budget-vs-actual.json) |
| Line with gaps for missing values | `"type": "line"`, `null` values | [monthly-trend](examples/monthly-trend.json) |
| Time series on a calendar axis | `"type": "time"` with `panes` and `layers` | [daily-orders](examples/daily-orders.json) |
| Time series, dark theme, declared colors | `"theme": "dark"`, `color` per layer | [revenue-vs-forecast](examples/revenue-vs-forecast.json) |
| Topic map: separate landmasses, area by value (experimental) | `"type": "topicmap"` with `topics` | [topicmap-sample](examples/topicmap-sample.json) |
| Knowledge landscape: one land, position by kinship (experimental) | `"type": "atlas"` with `realms` | [knowledge-landscape](examples/knowledge-landscape.json) |
| Time series with uncertainty band, modeled (hatched, dashed) | `lower`/`upper` per point, `"modeled": true` per layer | [temperature-projection](examples/temperature-projection.json) |
| Reference lines: threshold and date marker | `"mark": "annotation"` with `value` or `time` and `label` | [annual-mean-threshold](examples/annual-mean-threshold.json) |
| Time series with area, gaps, line patterns and zoom | `"mark": "area"`, `null` values, `dash`, `"stroke": "bold"`, `zoomSteps` by time | [sensor-readings](examples/sensor-readings.json) |
| Stacked areas, named where they end | `"stack": "normal"` on a pane of `area` layers, `"legend": "end"` | [generation-mix](examples/generation-mix.json) |
| Readings as dots around a trend line | `"mark": "point"` next to a `line` layer | [resting-heart-rate](examples/resting-heart-rate.json) |
| Zones and point markers | `"mark": "band"` with `from`/`to` or `bottom`/`top`; `"mark": "annotation"` with `time`, `value` and `shape` | [release-incidents](examples/release-incidents.json) |
| Candlesticks with a volume pane on one time axis, weekends closed up | `"mark": "ohlc"` with `data`; up to four `panes` with `heightRatio`; `"gaps": "collapse"` | [share-price](examples/share-price.json) |
| Warming stripes on a diverging scale | `"type": "stripes"` with `stripes` | [warming-stripes](examples/warming-stripes.json) |
| Calendar heatmap, by month or by week | `"type": "calendar"` with `calendar` | [daily-anomaly-calendar](examples/daily-anomaly-calendar.json) |
| Forest plot: squares by weight, a summary diamond, the line of no effect | `"type": "rangebar"` with `weight`, `summary`, `references` | [trial-effects](examples/trial-effects.json) |
| Timeline: phases, milestones, today's marker and dependencies (Gantt, roadmap) | `"type": "timeline"` with `items`, `markers` | [product-roadmap](examples/product-roadmap.json) |
| Waterfall: a total made up of rises, falls and subtotals | `"type": "waterfall"` with `waterfall.steps` | [revenue-to-profit](examples/revenue-to-profit.json) |
| Waffle chart: shares of a whole as squares, with the rest | `"type": "waffle"` with `waffle.parts` and `total` | [energy-sources](examples/energy-sources.json) |
| Parliament chart: seats by party, the majority line, a coalition ringed | `"type": "parliament"` with `parliament.parties`, `majority`, `coalition` | [election-result](examples/election-result.json) |
| Scatter plot: points in groups with threshold lines and names (volcano, Manhattan) | `"type": "scatter"` with `scatter.points`, `lines` | [gene-expression](examples/gene-expression.json) |
| Scatter plot with a regression line and logarithmic axes | `"regression": true`, `"xScale": "log"` on a `scatter` | [allometry](examples/allometry.json) |
| Kaplan-Meier curves: survival by group, censoring marks, confidence band, number at risk | `"type": "survival"` with `survival.groups` | [overall-survival](examples/overall-survival.json) |
| Sankey diagram: flows between stages as bands | `"type": "sankey"` with `sankey.links` | [energy-flow](examples/energy-flow.json) |
| Study selection (PRISMA) and chains of courts as flow charts (presets) | `"type": "flow"` | [study-selection](examples/study-selection.json), [court-instances](examples/court-instances.json) |
| Sankey diagram with fixed columns, order and colors (voter movement) | `sankey.nodes` with `column`, `color`, `"order": "listed"` | [voter-movement](examples/voter-movement.json) |
| Treemap: many parts of a whole as rectangles by value, in groups | `"type": "treemap"` with `treemap.items` | [budget-by-department](examples/budget-by-department.json) |
| Box plot from observations, with a box given by its five numbers | `"type": "boxplot"` with `boxes` | [response-times](examples/response-times.json) |
| Violins and strips from the same observations | `"boxDisplay": "violin"` on a `boxplot` | [response-distribution](examples/response-distribution.json) |
| Bars with error bars (confidence intervals) | `lower` and `upper` on every `data` point | [satisfaction-scores](examples/satisfaction-scores.json) |
| Range bars with central value, modeled hatched | `"type": "rangebar"` with `ranges` | [warming-contributions](examples/warming-contributions.json) |
| Range bars in groups on a logarithmic axis | `group` per range, `"valueAxis": { "scale": "log" }` | [soil-animals](examples/soil-animals.json) |
| Small multiples with a shared value axis | `"type": "multiples"` with titled `panes` | [emission-pathways](examples/emission-pathways.json) |
| Small multiples of bars: one panel per measure over shared categories, each with its own value axis | `"type": "multiples"` with `categories` and a `values` array per pane | [framework-benchmarks](examples/framework-benchmarks.json) |
| Bars colored by group, one legend entry per group | `"group"` on every `data` point | [benefit-and-harm](examples/benefit-and-harm.json) |
| Small multiples with a finding under each panel, one column on phones | `note` and `noteEmphasis` per pane, `"mobile": { "columns": 1 }` | [warming-causes](examples/warming-causes.json) |
| Sequence diagram, portrait | `"type": "sequence"` with `participants`, `messages`, `fragments` | [cache-lookup](examples/cache-lookup.json) |
| Sequence diagram, landscape | `"orientation": "landscape"` | [async-export](examples/async-export.json) |
| Flow chart in lanes with loops back and a group (experimental) | `"type": "flow"` with `nodes`, `edges`, `lanes`, `groups`, `mainPath` | [release-flow](examples/release-flow.json) |
| Flow chart turned landscape by a wide canvas (experimental) | `"orientation": "auto"` | [order-flow](examples/order-flow.json) |
| State diagram with a composite state, a choice and final states (experimental) | `"type": "state"` with `states`, `transitions`, `initial` | [ticket-states](examples/ticket-states.json) |
| Architecture diagram in nested boundaries | `"type": "architecture"` with `components`, `connections`, `boundaries` | [shop-architecture](examples/shop-architecture.json) |
| Sequence diagram with a phone variant: sign-in with a backend for frontend | `"type": "sequence"`, `"mobile"` | [sign-in-sequence](examples/sign-in-sequence.json) |
| Architecture diagram with a phone variant: a multi-tenant application | `"type": "architecture"`, `"mobile"` | [tenant-architecture](examples/tenant-architecture.json) |
| Tree: an ownership structure with shares, with a phone variant | `"type": "tree"` with `nodes`, each with a `parent` and a `link` | [ownership-structure](examples/ownership-structure.json) |
| Family tree: couples side by side, persons and an external node | `"type": "tree"` with `partner` and `kind` | [family-tree](examples/family-tree.json) |
| Mobile variant for narrow containers, any type | `"mobile": { "width": 360 }` | [mobile-revenue](examples/mobile-revenue.json) |
| Compact chart for a panel in a grid of columns | `"width": 240`, `"height": 180` (from 200 × 160 px: narrower gutter, same text size), `"timeAxis": { "step": 50, "min": "1850" }` for the ticks the panel's claim needs, `"stroke": "medium"`, `"valueAxis": { "unit": "W/m²" }` at the top tick instead of an axis title | – |

Each example has its rendered `.svg` and `.html` next to it. The SVG files are also the
reference output of the test suite.

### Missing values

In a line chart, `null` marks an observation that does not exist. chartlet leaves a visible gap
instead of drawing a line across it. In a grouped bar chart, `null` leaves out that bar. The data
table shows these cells as “Missing”. A single-series bar chart requires a value for every
category.

### Several series

Replace `data` with `categories` and `series`. Every series needs exactly one value per category.
A legend is added automatically, and the data table gets one column per series.

### Time series

`"type": "time"` replaces categories with timestamps and draws lines on a calendar axis. Each
observation is `{ "time": …, "value": … }`, where `time` is Unix seconds or an ISO 8601 date; a
timestamp without its own offset is read as wall-clock time in `timeAxis.timezone` (a fixed UTC
offset, `UTC` by default). Layers live in `panes` and may name their own color. See the
[time series guide](docs/guides/time-series.md) for the contract, the theme, and the measured
limits for dense series.

A line layer can carry an uncertainty band: give every point `lower` and `upper`. With
`"modeled": true` the line is dashed, its band hatched, and legend, description and data table
say “modeled”. An `"annotation"` layer draws a labelled reference line: `value` for a horizontal
threshold, `time` for a vertical marker. A bare year such as `"1850"` is a valid timestamp, and
annual data is labelled by year. `null` as a value breaks the line, its band and its area.
`"mark": "area"` fills the region between a line and zero, and `"stack": "normal"` on a pane
stacks its areas; `"mark": "point"` draws dots without a line, `"markers": false` a line without
dots; `"curve": "step"` with `stepEnd` draws period values such as annual means; `precision` per
layer names yearly means by their year beside monthly values; `timeAxis.step` sets the distance
between time ticks and `timeAxis.min`/`max` extend the axis to a round position; `dash` (`solid`, `dashed`, `dotted`)
and `stroke` (`thin`, `medium`, `regular`, `bold`) tell lines apart beyond color. A pane holds up to six
data layers, four of them in palette colors, and `zoomSteps` take timestamps on a time chart.
`"mark": "ohlc"` draws candlesticks from `data: [{ time, open, high, low, close }]` — hollow when
rising, filled when falling — and up to four `panes` stack on one shared time axis, each with its
own value axis and a share of the height by `heightRatio`; volume goes in a second pane as an
`area` layer.
`"type": "multiples"` lays out 2 to 12 titled panes as small
time charts in a grid (`columns`), sharing one value axis and one legend.

### Stripes, calendars and range bars

`"type": "stripes"` draws one stripe per year (`stripes.firstYear`, `stripes.values`);
`"type": "calendar"` one cell per day of `calendar.year` (`layout`: `months` or `weeks`). Both
color on a diverging scale of eight equal steps either side of `reference` (default 0), whose
outermost steps begin at `min` and `max` (default: the largest distance from the reference). A
missing value leaves an empty stripe or an outlined cell. `"type": "rangebar"` draws one span per
entry of `ranges` (`low`, `high`, optional `mid` and `modeled`), vertical or horizontal.

```json
{
  "schemaVersion": 1,
  "type": "bar",
  "title": "Budget and actual costs",
  "categories": ["January", "February", "March"],
  "series": [
    { "name": "Budget", "values": [120, 150, 140] },
    { "name": "Actual", "values": [130, 145, null] }
  ]
}
```

## Software diagrams

Four diagram types describe software rather than data. The specification says only what exists
and what is connected; chartlet lays the diagram out — layers along the flow, edges routed around
the boxes, the main path kept straight, cycles drawn back against the flow — and writes the same
accessible output as for a chart.

| Type | Elements |
|---|---|
| `sequence` | participants (`service`, `actor`, `database`, `queue`, `external`), messages (`call`, `reply`, `async`), activation bars, fragments (`alt` with `else`, `opt`, `loop`, `par`, `critical`, `break`), numbered messages |
| `flow` | steps (`process`, `start`, `end`, `decision`, `io`, `subprocess`, `store`, `external`), labelled edges, lanes, groups, a main path |
| `state` | states, choices, composite and final states, an initial state, transitions `event [guard] / action` |
| `architecture` | components (`person`, `frontend`, `service`, `database`, `queue`, `storage`, `cache`, `security`, `external`), connections with a technology, nested boundaries |
| `tree` | a root and the nodes below it, each naming its parent, with an optional link label such as a share; kinds (`unit`, `person`, `external`) and couples for family trees |

```json
{
  "schemaVersion": 1,
  "type": "flow",
  "title": "Publishing a post",
  "flow": {
    "nodes": [
      { "id": "draft", "label": "Draft", "kind": "start" },
      { "id": "check", "label": "Looks good?", "kind": "decision" },
      { "id": "edit", "label": "Edit" },
      { "id": "live", "label": "Published", "kind": "end" }
    ],
    "edges": [
      { "from": "draft", "to": "check" },
      { "from": "check", "to": "live", "label": "yes" },
      { "from": "check", "to": "edit", "label": "no" },
      { "from": "edit", "to": "check", "label": "again" }
    ],
    "mainPath": ["draft", "check", "live"]
  }
}
```

- **Portrait or landscape:** `orientation` runs a diagram down or right; `auto` picks what fits
  the canvas, and a mobile variant decides again. Below 480 pixels diagrams turn compact.
- **Shapes and roles:** every kind has its own shape and a role color on top of it, so meaning
  never rests on color alone; the colors are CSS custom properties a page can override.
- **Text alternative:** the description and the data table list every message, step, transition
  or component in reading order.
- **Focus:** in the HTML output a click on a node, or a choice in the "Focus" list, keeps the node
  and its neighbours and fades the rest — without a script.

See [Chart types](docs/guides/chart-types.md#sequence-diagrams) for every field.

## Interaction without JavaScript

The HTML output can add native controls and CSS-based interaction. No chart JavaScript is
shipped.

- **Series filter:** a grouped chart gets a checkbox per series. Deselecting one hides its bars
  and value labels with CSS `:has()`; the axis does not rescale. The data table and description always
  show the full data. Browsers without `:has()` support simply keep every series visible.
- **Zoom steps:** add two to four `zoomSteps` to pre-compute narrower views of the same chart. The HTML
  output renders one variant per step and switches between them with radio buttons. Each step
  costs its own SVG in the output file. A bar or line chart names categories by index; a `time`
  chart names a window by timestamps (`"from": "2026-03-22", "to": "2026-03-28"`).
- **Tooltips:** every bar and point carries a native `<title>` (`Month: value`, or
  `Month – Series: value`) that browsers can show on hover. The chart description and data table,
  rather than these hover-only tooltips, remain the assistive-technology alternative.

The pure SVG profile stays a single static chart; filtering and stepped zoom are HTML-only.

For what CSS cannot do, the npm package has an optional, additive browser module,
`@casoon/chartlet/interactive`: a crosshair that reads out values by pointer and keyboard, series
toggles on time charts, scroll stations and playback. It reads charts rendered with `hooks: true`
(CLI `--hooks`) and lays nothing out again; without it the output is unchanged. See
[Interaction](docs/guides/interaction.md#optional-javascript).

```json
{
  "schemaVersion": 1,
  "type": "bar",
  "title": "Quarterly revenue",
  "data": [
    { "label": "Q1", "value": 320 },
    { "label": "Q2", "value": 345 },
    { "label": "Q3", "value": 380 },
    { "label": "Q4", "value": 410 }
  ],
  "zoomSteps": [
    { "label": "First half", "from": 0, "to": 1 },
    { "label": "All", "from": 0, "to": 3 }
  ]
}
```

## Specification

[`schema/chartlet.schema.json`](schema/chartlet.schema.json) is the complete contract and can be
used for editor validation. Every field, per chart type, and every limit is listed in the
[specification reference](docs/reference/specification.md).

Limits: up to 100 categories and four series. Labels must be unique. Values must be zero or have
a magnitude between `1e-100` and `1e100`. A `time` chart carries up to four panes, each with up to six data layers of
2000 observations or candles (at most four layers of the chart in palette colors) and up to six
zones, reference lines and point markers; markers and value labels are drawn
up to 60 observations per layer, and beyond one observation per plot pixel the rendering is
reported as `dense_chart`. Small multiples take 2 to 12 panes.

## Output

| Option | Effect |
|---|---|
| `--format svg` | Standalone SVG with `<title>` and `<desc>` and the title drawn in the chart (default). |
| `--format html` | `<figure>` with caption, SVG, source and data table; the caption is the visible title, the SVG draws none. With `mobile` in the specification, both variants behind a container query. |
| `--format png` | PNG of the print, social or mobile variant with a bundled font, `--scale` 0.25–4. Optional: `cargo install chartlet --features png`. |
| `--table details` | Puts the HTML data table in a native, initially closed `<details>` (default). |
| `--table visible` | Shows the data table permanently. |
| `--id-prefix <prefix>` | Stable ID of the chart root and prefix for its other IDs; needed when the same chart appears twice on one page. |
| `--variant mobile` | Renders the mobile variant alone as SVG, for a `<picture>` source; requires `mobile` in the specification (`missing_mobile`) and the SVG format. `desktop` (default) renders the chart at `width` × `height`. |
| `--variant print` | Renders the chart as SVG with literal colors instead of CSS custom properties, for PDF pipelines and renderers outside the browser; IDs end in `-p`. SVG format only. See [Print and PDF](docs/guides/print.md). |
| `--variant social` | Renders the chart on a 1200 × 630 canvas for Open Graph images, title drawn large, literal colors; IDs end in `-s`. SVG format only. See [Social images and PNG](docs/guides/social-and-png.md). |
| `-o <path>` | Writes to a file instead of standard output. |
| `--manifest <path>` | Also writes a provenance manifest as JSON: chartlet version, SHA-256 of the canonical specification and of the output, format, variant, ID prefix and warnings; no timestamp. |
| `--strict` | Fails on any warning. Useful in CI. |
| `--diagnostics json` | Writes errors and warnings to standard error as one JSON document instead of text lines. |

Pass `-` instead of a file name to read the specification from standard input.

The SVG scales with its container, uses CSS classes for all styling and embeds no fonts, scripts
or external resources.

## Accessibility

- The SVG is exposed as a single image (`role="img"`). Its accessible name is the title, and its
  description is generated from the data, for example: “Bar chart with 4 categories and 2 series
  (Budget, Actual). Highest: 160 (Budget in April). Lowest: 120 (Budget in January). 1 value is
  missing.”
- The HTML output adds a `<figure>` with caption and a real `<table>` containing every value,
  including values whose visual label had to be left out. The caption is the visible title: the
  SVG inside the figure draws none, so the title is shown and announced once, while the SVG keeps it
  as its accessible name.
- Series colors stay distinguishable for the common forms of color-vision deficiency and have at
  least 4.5:1 contrast against white — and against the dark theme's background, where the lowest
  series color reaches 7.65:1. The legend lists series in the same order as the bars.
- Text is never removed silently: shortened labels and omitted value labels produce warnings, and
  the full text stays in the specification and the data table.

Testing with VoiceOver and NVDA is still pending. Until then, treat the output as designed for
accessibility, not as verified.

## Warnings and errors

Errors stop rendering and name a code, the location in the specification as a JSON Pointer
(`/` for the whole document) and a way to fix it:

```text
chartlet: series_length_mismatch at /series/0/values: expected 2 values, one per category; use null for a missing value
```

Warnings are written to standard error, and the chart is still produced:

| Code | Meaning |
|---|---|
| `text_truncated` | A label or title was shortened to fit. Titles (drawn only in the SVG profile) and the category labels of bar and range bar charts wrap onto a second line first; what does not fit on two lines is shortened. |
| `value_labels_omitted` | Some value labels had no room next to their bars. On a bar chart a category shows all of its value labels or none: none when one of them would overlap another value label or another bar of its own or a neighbouring category. |
| `labels_thinned` | More than 16 categories whose labels do not fit their bands even on two lines: only every few categories are labelled, each with the room of several bands. The data table lists every category. |
| `dense_chart` | More than 16 categories, or more observations in a time layer than the plot has horizontal pixels; the chart may be hard to read at this size. |
| `color_not_supported` | A layer's `color` was outside the contract and replaced by the neutral gray. |
| `color_not_resolved` | Print and social variants only: a layer declares its color as `var(--name)`, which they cannot resolve; it is drawn in the text color. Declare a literal color so that layers stay distinguishable. |
| `topic_too_small_for_label` | A topic map area is too small to hold its own name. |
| `label_does_not_fit` | A region of a knowledge landscape has no room for its name; the area keeps its tooltip. |
| `places_did_not_fit` | A region declares more places than it has ground. |
| `realm_without_structure` | A realm holds a single region, so it has no inner structure to show. |
| `more_places_than_value` | A region lists more places than its value. |
| `label_overlap` | The label of a zone, reference line or point marker overlaps another annotation label, crosses a data line, or reaches outside the plot; the label is kept. |
| `canvas_too_small` | A diagram does not fit its canvas in either orientation and was drawn larger; the message names the size it needs. |
| `group_overlap` | A step outside a group or boundary lies inside its frame. |
| `unreachable_state` | A state diagram has an initial state, and no transition leads to this state. |

## Astro

The npm package [`@casoon/chartlet`](packages/chartlet/README.md) renders charts while Astro
builds the site and ships no JavaScript to the browser. It carries the renderer as WebAssembly,
so the package is all you install:

```sh
npm install @casoon/chartlet
```

```astro
---
import Chart from '@casoon/chartlet/astro';
import revenue from '../data/monthly-revenue.json';
---

<Chart id="monthly-revenue" spec={revenue} />
```

To render with an installed CLI instead, set `CHARTLET_BIN` to its path. For Node.js without
Astro, Cloudflare Workers and Vite, see [JavaScript runtimes](docs/guides/javascript.md).

## AI agents

[`@casoon/chartlet-mcp`](packages/chartlet-mcp/README.md) is an MCP server for AI assistants:
the assistant inspects the data and drafts a specification, chartlet validates and compiles it.
No model runs in the server and the data is never changed. For software diagrams,
`chartlet_diagram_starter` hands the assistant a valid starting specification per type. See [AI agents](docs/guides/ai-agents.md).

## Rust

```sh
cargo add chartlet
```

```rust
use chartlet::{render_json, RenderFormat, RenderOptions};

let spec = std::fs::read_to_string("examples/monthly-revenue.json")?;
let output = render_json(&spec, RenderFormat::Html, &RenderOptions::default())?;
for warning in &output.warnings {
    eprintln!("{} at {}: {}", warning.code, warning.path, warning.message);
}
std::fs::write("chart.html", output.content)?;
```

`render_with_metrics` accepts your own `TextMetrics` implementation if your pages use a font whose
widths differ noticeably from the built-in profile.

## When chartlet is not the right tool

- You need continuous zooming, panning, cross-filtering or live updates. (A crosshair with values, series toggles, scroll stations and step-by-step playback come from the optional `@casoon/chartlet/interactive` module, which only reads the static output.)
- The data changes at runtime rather than at build time.
- You need geographic maps, 3D charts or chart types beyond the ones listed above, or diagrams
  you place by hand, with more than about 40 boxes.
- You want to explore data rather than publish a finished chart.

## License

[MIT](LICENSE)
