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
- **Diagrams laid out for you:** sequence, flow, state, architecture, class, tree and network
  diagrams come from a description of what is connected; chartlet places every box and routes
  every edge, in portrait or landscape, and lists the whole structure in the data table.
- **Models, not only pictures:** one architecture model yields several views, deployments per
  environment and perspectives such as ownership, and `chartlet inspect` points out what its
  documentation leaves open.

> **Status:** before 1.0. Until 1.0, a minor release may still change the specification; a patch
> release never does. Flow, state and class diagrams are
> experimental: their layout may change between minor releases.

## Quick start

Install the CLI from crates.io (Rust 1.88 or newer), or build it from a clone with
`cargo install --path .`:

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

Render it, or keep a live preview open while you edit:

```sh
chartlet render spec.json --format html -o chart.html
chartlet render spec.json --format html --preview
```

## Website and documentation

The [project website](https://casoon.github.io/chartlet/) has the documentation, every example
chart with its specification next to the rendered output, and the support matrix for embedding
contexts, browsers and screen readers. It is built with Astro and renders every chart with
`@casoon/chartlet` at build time; its source lives in [`site/`](site/), the documentation in
[`docs/`](docs/).

## Chart types

| Chart | Specification | Example |
|---|---|---|
| Bar, vertical | `"type": "bar"` with `data` | [monthly-revenue](examples/monthly-revenue.json) |
| Bar, horizontal, with negative values | `"orientation": "horizontal"` | [quarterly-change](examples/quarterly-change.json) |
| Grouped bar, up to four series | `categories` and `series` instead of `data` | [budget-vs-actual](examples/budget-vs-actual.json) |
| Bars colored by group, one legend entry per group | `"group"` on every `data` point | [benefit-and-harm](examples/benefit-and-harm.json) |
| Grouped bars with error bars | `lower` and `upper` arrays on each `series` | [satisfaction-by-quarter](examples/satisfaction-by-quarter.json) |
| Bars with error bars (confidence intervals) | `lower` and `upper` on every `data` point | [satisfaction-scores](examples/satisfaction-scores.json) |
| Small multiples with a shared value axis | `"type": "multiples"` with titled `panes` | [emission-pathways](examples/emission-pathways.json) |
| Small multiples of bars: one panel per measure over shared categories, each with its own value axis | `"type": "multiples"` with `categories` and a `values` array per pane | [framework-benchmarks](examples/framework-benchmarks.json) |
| Small multiples with a finding under each panel, one column on phones | `note` and `noteEmphasis` per pane, `"mobile": { "columns": 1 }` | [warming-causes](examples/warming-causes.json) |
| Line with gaps for missing values | `"type": "line"`, `null` values | [monthly-trend](examples/monthly-trend.json) |
| Line chart with error bars | `lower` and `upper` on every `data` point of a `line` chart | [monthly-satisfaction](examples/monthly-satisfaction.json) |
| Time series on a calendar axis | `"type": "time"` with `panes` and `layers` | [daily-orders](examples/daily-orders.json) |
| Time series, dark theme, declared colors | `"theme": "dark"`, `color` per layer | [revenue-vs-forecast](examples/revenue-vs-forecast.json) |
| Own colors for one chart or for all of them | `tokens` in the specification, or a theme file with `--theme` | [brand theme](examples/themes/brand.json) |
| Time series with uncertainty band, modeled (hatched, dashed) | `lower`/`upper` per point, `"modeled": true` per layer | [temperature-projection](examples/temperature-projection.json) |
| Reference lines: threshold and date marker | `"mark": "annotation"` with `value` or `time` and `label` | [annual-mean-threshold](examples/annual-mean-threshold.json) |
| Time series with area, gaps, line patterns and zoom | `"mark": "area"`, `null` values, `dash`, `"stroke": "bold"`, `zoomSteps` by time | [sensor-readings](examples/sensor-readings.json) |
| Stacked areas, named where they end | `"stack": "normal"` on a pane of `area` layers, `"legend": "end"` | [generation-mix](examples/generation-mix.json) |
| Readings as dots around a trend line | `"mark": "point"` next to a `line` layer | [resting-heart-rate](examples/resting-heart-rate.json) |
| Zones and point markers | `"mark": "band"` with `from`/`to` or `bottom`/`top`; `"mark": "annotation"` with `time`, `value` and `shape` | [release-incidents](examples/release-incidents.json) |
| Candlesticks with a volume pane on one time axis, weekends closed up | `"mark": "ohlc"` with `data`; up to four `panes` with `heightRatio`; `"gaps": "collapse"` | [share-price](examples/share-price.json) |
| Warming stripes on a diverging scale | `"type": "stripes"` with `stripes` | [warming-stripes](examples/warming-stripes.json) |
| Calendar heatmap, by month or by week | `"type": "calendar"` with `calendar` | [daily-anomaly-calendar](examples/daily-anomaly-calendar.json) |
| Range bars with central value, modeled hatched | `"type": "rangebar"` with `ranges` | [warming-contributions](examples/warming-contributions.json) |
| Range bars in groups on a logarithmic axis | `group` per range, `"valueAxis": { "scale": "log" }` | [soil-animals](examples/soil-animals.json) |
| Forest plot: squares by weight, a summary diamond, the line of no effect | `"type": "rangebar"` with `weight`, `summary`, `references` | [trial-effects](examples/trial-effects.json) |
| Timeline: phases, milestones, today's marker and dependencies (Gantt, roadmap) | `"type": "timeline"` with `items`, `markers` | [product-roadmap](examples/product-roadmap.json) |
| Waterfall: a total made up of rises, falls and subtotals | `"type": "waterfall"` with `waterfall.steps` | [revenue-to-profit](examples/revenue-to-profit.json) |
| Waffle chart: shares of a whole as squares, with the rest | `"type": "waffle"` with `waffle.parts` and `total` | [energy-sources](examples/energy-sources.json) |
| Parliament chart: seats by party, the majority line, a coalition ringed | `"type": "parliament"` with `parliament.parties`, `majority`, `coalition` | [election-result](examples/election-result.json) |
| Nested treemap: sectors and departments | `children` on `treemap.items` | [budget-by-sector](examples/budget-by-sector.json) |
| Treemap: many parts of a whole as rectangles by value, in groups | `"type": "treemap"` with `treemap.items` | [budget-by-department](examples/budget-by-department.json) |
| Sankey diagram: flows between stages as bands | `"type": "sankey"` with `sankey.links` | [energy-flow](examples/energy-flow.json) |
| Sankey diagram with a flow back (recycling) as a loop | `"back": true` on a Sankey link | [water-cycle](examples/water-cycle.json) |
| Sankey diagram with fixed columns, order and colors (voter movement) | `sankey.nodes` with `column`, `color`, `"order": "listed"` | [voter-movement](examples/voter-movement.json) |
| Scatter plot: points in groups with threshold lines and names (volcano, Manhattan) | `"type": "scatter"` with `scatter.points`, `lines` | [gene-expression](examples/gene-expression.json) |
| Scatter plot with a regression line and logarithmic axes | `"regression": true`, `"xScale": "log"` on a `scatter` | [allometry](examples/allometry.json) |
| Manhattan plot: scatter with named x ticks and fixed ranges | `scatter.xTicks`, `xRange` | [genome-scan](examples/genome-scan.json) |
| Impact and effort matrix: scatter in four named quadrants | `scatter.quadrants` | [feature-priorities](examples/feature-priorities.json) |
| Kaplan-Meier curves: survival by group, censoring marks, confidence band, number at risk | `"type": "survival"` with `survival.groups` | [overall-survival](examples/overall-survival.json) |
| Box plot from observations, with a box given by its five numbers | `"type": "boxplot"` with `boxes` | [response-times](examples/response-times.json) |
| Grouped box plots: boxes colored by group with a legend | `group` on every box | [latency-by-region](examples/latency-by-region.json) |
| Violins and strips from the same observations | `"boxDisplay": "violin"` on a `boxplot` | [response-distribution](examples/response-distribution.json) |
| Mobile variant for narrow containers, any type | `"mobile": { "width": 360 }` | [mobile-revenue](examples/mobile-revenue.json) |

Each example has its rendered `.svg` and `.html` next to it; the SVG files are also the reference
output of the test suite. In a line chart, `null` leaves a visible gap instead of a line across a
missing observation; in a grouped bar chart it leaves out the bar, and the data table says
“Missing”. Time series take Unix seconds or ISO 8601 dates on a calendar axis with a fixed time
zone, up to four panes on one time axis and up to six layers per pane. See
[Chart types](docs/guides/chart-types.md) and the [time series guide](docs/guides/time-series.md)
for every field.

## Software diagrams

Seven diagram types describe software and structure rather than data. The specification says only
what exists and what is connected; chartlet lays the diagram out — layers along the flow, edges
routed around the boxes, the main path kept straight, cycles drawn back against the flow — and
writes the same accessible output as for a chart.

| Type | Elements | Examples |
|---|---|---|
| `sequence` | participants (`service`, `actor`, `database`, `queue`, `external`), messages (`call`, `reply`, `async`), activation bars, fragments (`alt`, `opt`, `loop`, `par`, `critical`, `break`), numbered messages | [cache-lookup](examples/cache-lookup.json), [async-export](examples/async-export.json), [sign-in-sequence](examples/sign-in-sequence.json) |
| `flow` | steps (`process`, `start`, `end`, `decision`, `io`, `subprocess`, `store`, `external`), labelled edges, lanes, groups, a main path; presets for study selection (PRISMA) and chains of courts | [release-flow](examples/release-flow.json), [order-flow](examples/order-flow.json), [contract-review](examples/contract-review.json), [order-fulfilment](examples/order-fulfilment.json), [study-selection](examples/study-selection.json), [court-instances](examples/court-instances.json) |
| `state` | states, choices, composite and final states, an initial state, transitions `event [guard] / action` | [ticket-states](examples/ticket-states.json) |
| `architecture` | components (`person`, `frontend`, `service`, `database`, `queue`, `storage`, `cache`, `security`, `external`, `node`, `artifact`), connections with a technology, a direction and an optional `step` number, nested boundaries; `theme: "blueprint"` | [shop-architecture](examples/shop-architecture.json), [checkout-calls](examples/checkout-calls.json), [shop-deployment](examples/shop-deployment.json), [tenant-architecture](examples/tenant-architecture.json) |
| `classes` | classes, interfaces, abstract classes, enums and entities with attribute and method compartments, relations with their marks, cardinalities and navigability, nested packages | [order-model](examples/order-model.json), [shop-schema](examples/shop-schema.json), [shop-modules](examples/shop-modules.json), [invoice-model](examples/invoice-model.json) |
| `tree` | a root and the nodes below it with optional link labels such as shares; couples for family trees; a radial mind map; a dendrogram from a distance matrix | [ownership-structure](examples/ownership-structure.json), [family-tree](examples/family-tree.json), [product-strategy](examples/product-strategy.json), [climate-similarity](examples/climate-similarity.json) |
| `network` | nodes as dots on a circle or in rings around a center, groups, weighted and directed edges | [co-authors](examples/co-authors.json), [service-dependencies](examples/service-dependencies.json) |

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

Modeling an architecture:

- **Views:** one model, several diagrams. A view lists the components it shows, or selects them
  by `tags`; a dynamic view numbers `steps` along existing connections. `--view` renders one,
  `--all-views` every view on one HTML page — [shop-views](examples/shop-views.json).
- **Deployments:** a view with an `environment` places `instances` of components on nested
  `hosts`, with instance connections listed or derived from the model —
  [api-environments](examples/api-environments.json),
  [booking-deployment](examples/booking-deployment.json).
- **Perspectives:** stated categories such as the owning team or technical debt, shown as a
  colored, named stripe with a legend, per view or with `--perspective` —
  [service-perspectives](examples/service-perspectives.json).
- **Interfaces and symbols:** a component `provides` interfaces drawn as balls, a connection with
  `toInterface` requires one; 16 generic `symbol`s mark gateways, caches, brokers and the like —
  [payment-interfaces](examples/payment-interfaces.json), [shop-symbols](examples/shop-symbols.json).
- **Levels:** the MCP starters `context`, `containers` and `deployment` begin a model at one level
  of abstraction — [booking-context](examples/booking-context.json),
  [booking-containers](examples/booking-containers.json).

In every diagram that has them:

- **Notes** explain an element, a frame or the whole diagram without adding an edge (flow, class
  and architecture) — [payment-notes](examples/payment-notes.json),
  [subscription-model](examples/subscription-model.json).
- **Roles** `critical`, `planned` and `deprecated` look alike across flow, state, architecture,
  class, tree and network diagrams, with a sign that does not rest on color —
  [billing-migration](examples/billing-migration.json),
  [payment-redesign](examples/payment-redesign.json).
- **Layout hints:** `mainPath` (as a list or from one step to another), `sameRank`, the sides an
  edge leaves and reaches (`fromSide`, `toSide`), `spacing`, `weight` for line width and
  `sharedTrunks` for fan-outs on one branching line —
  [data-tier](examples/data-tier.json), [request-volume](examples/request-volume.json),
  [service-fan-out](examples/service-fan-out.json).
- **Values and parts** fill flow and tree boxes by value or with a stripe of shares —
  [department-budgets](examples/department-budgets.json),
  [claim-handling](examples/claim-handling.json).
- **Portrait or landscape:** `orientation` runs a diagram down or right; `auto` picks what fits
  the canvas, and a mobile variant decides again. Below 480 pixels diagrams turn compact.
- **Text alternative:** the description lists every message, step, transition or component in
  reading order, and names the shape of the graph where it says more; the table holds the whole
  structure.
- **Focus:** in the HTML output, a click on a node or a choice in the "Focus" list keeps the node
  and its neighbours and fades the rest — without a script.
- **Hooks:** with `hooks: true`, nodes, frames, lanes and architecture connections carry their
  specification id as `data-chartlet-*` attributes, so a page finds them by id, not by position.

See [Chart types](docs/guides/chart-types.md#architecture-diagrams) for every field.

## Interaction without JavaScript

The HTML output can add native controls and CSS-based interaction. No chart JavaScript is
shipped.

- **Series filter:** a grouped chart gets a checkbox per series that hides its marks with CSS
  `:has()`; the data table and description always show the full data.
- **Zoom steps:** two to four `zoomSteps` pre-compute narrower views of the same chart, switched
  with radio buttons.
- **Tooltips:** every bar, point and diagram element carries a native `<title>`; the description
  and data table remain the assistive-technology alternative.

The pure SVG profile stays a single static chart. For what CSS cannot do, the optional browser
module `@casoon/chartlet/interactive` adds a crosshair that reads out values by pointer and
keyboard, series toggles on time charts, scroll stations and playback; it reads charts rendered
with `hooks: true` and lays nothing out again. See [Interaction](docs/guides/interaction.md).

## Specification

[`schema/chartlet.schema.json`](schema/chartlet.schema.json) is the complete contract and can be
used for editor validation. Every field, per chart type, and every limit is listed in the
[specification reference](docs/reference/specification.md).

## Command line

```sh
chartlet render <spec.json | -> [options]
chartlet inspect <spec.json | -> [--format text|json] [--strict]
chartlet stylesheet [--types bar,time,...] [--no-common | --common]
```

| Option of `render` | Effect |
|---|---|
| `--format svg\|html\|png` | Standalone SVG (default); an HTML `<figure>` with caption, SVG, source and data table, both layouts behind a container query when the specification has `mobile`; or a PNG (`cargo install chartlet --features png`, `--scale` 0.25–4). |
| `--variant mobile\|print\|social` | The phone layout alone, literal colors for PDF and print, or a 1200 × 630 Open Graph image. |
| `--view <key>`, `--all-views`, `--perspective <key>` | One view of an architecture model, every view on one HTML page, or a perspective to show. |
| `--theme <name\|file>` | A bundled theme or a theme file of your own colors, applied to the specification; see the [guide](docs/guides/customize.md). |
| `--table details\|visible` | The HTML data table in a closed `<details>` (default) or always shown. |
| `--id-prefix <prefix>` | Stable IDs, for the same chart twice on one page. |
| `--hooks`, `--styles external` | `data-*` hooks for scripts and tests; a chart without its shared stylesheet, which `chartlet stylesheet` prints once per page. |
| `--manifest <path>` | A provenance manifest: version, hashes of specification and output, warnings; no timestamp. |
| `--strict`, `--allow-warning <code>`, `--diagnostics json` | Fail on any warning except the codes allowed; errors and warnings as one JSON document. |
| `--watch`, `--preview [--port <port>]` | Render again on every change of the file; also serve the latest chart with its warnings on a local page that reloads. |

`chartlet inspect` checks an architecture model, a flow chart or a class diagram for gaps in its
documentation without rendering it — unlabelled connections and decision branches, components in
no view, unreachable steps — each with a JSON Pointer and a hint. See
[CLI options](docs/reference/cli.md).

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

Warnings go to standard error and the chart is still produced: shortened text
(`text_truncated`), omitted value labels, thinned category labels, dense charts, overlapping
annotation labels, a diagram that needed a larger canvas (`canvas_too_small`) and more. Every
code is listed in [Warnings and errors](docs/guides/warnings-and-errors.md).

## Astro and JavaScript

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

`renderChart` and `inspectChart` do the same outside Astro — in Node.js, Cloudflare Workers and
Vite, see [JavaScript runtimes](docs/guides/javascript.md) — and `@casoon/chartlet/builder`
assembles architecture and flow specifications in code, with ids checked at the call. To render
with an installed CLI instead, set `CHARTLET_BIN` to its path.

## AI agents

[`@casoon/chartlet-mcp`](packages/chartlet-mcp/README.md) is an MCP server for AI assistants:
the assistant inspects the data and drafts a specification, chartlet validates and compiles it.
No model runs in the server and the data is never changed. `chartlet_inspect_data` suggests the
types that fit a table, `chartlet_chart_starter` and `chartlet_diagram_starter` hand over a valid
starting specification per type (for architecture at the context, container or deployment level),
`chartlet_validate_spec`, `chartlet_render`, `chartlet_explain` and `chartlet_inspect` check,
compile, describe and inspect it. See [AI agents](docs/guides/ai-agents.md).

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
widths differ noticeably from the built-in profile; `inspect_json` runs the inspections, and
`chartlet::qr` draws a QR code as SVG. See [Rust](docs/reference/rust.md).

## When chartlet is not the right tool

- You need continuous zooming, panning, cross-filtering or live updates. (A crosshair with values,
  series toggles, scroll stations and step-by-step playback come from the optional
  `@casoon/chartlet/interactive` module, which only reads the static output.)
- The data changes at runtime rather than at build time.
- You need geographic maps, 3D charts or chart types beyond the ones listed above, or diagrams
  you place by hand, with more than about 40 boxes.
- You want to explore data rather than publish a finished chart.

## License

[MIT](LICENSE)
