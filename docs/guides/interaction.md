---
title: Interaction without JavaScript
description: Series filters, pre-rendered zoom steps and native tooltips – built from HTML controls and CSS.
order: 2
---

The HTML output can add native controls and CSS-based interaction. No chart JavaScript is shipped;
interaction without JavaScript is the base, and the [optional module](#optional-javascript) is an
additive layer on top of it.
The pure SVG profile stays a single static chart; filtering and stepped zoom are HTML-only.

## Series filter

A grouped chart gets a checkbox per series. Deselecting one hides its bars and value labels with CSS
`:has()`; the axis does not rescale. The data table and description always show the full data.
Browsers without `:has()` support simply keep every series visible.

Try it on [Budget vs. actual](../../../showcase/budget-vs-actual/).

## Focus on a diagram node

Sequence, flow, state and architecture diagrams get a "Focus" disclosure above the figure with a
radio button per node, and a label of each button laid over its node. Choosing a node — in the
list, or with a click or tap on the node — keeps it, its neighbours and the edges between them
and fades the rest; "Show all" brings everything back. CSS `:has()` does the work; browsers
without it show the diagram unchanged.

Try it on [From commit to release](../../../showcase/release-flow/).

## Zoom steps

Add two to four `zoomSteps` to pre-compute narrower views of the same chart. The HTML output renders
one variant per step and switches between them with radio buttons. Each step costs its own SVG in
the output file.

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

Try it on [Headcount](../../../showcase/headcount/).

## Tooltips

Every bar and point carries a native `<title>` (`Month: value`, or `Month – Series: value`) that
browsers can show on hover. The chart description and data table, rather than these hover-only
tooltips, remain the assistive-technology alternative.

## Optional JavaScript

Everything above works without a script. For what CSS cannot do — a crosshair that reads out
values, series toggles on time charts, scroll stations and playback — the npm package has an
optional module, `@casoon/chartlet/interactive`. It is additive: a page that does not import it,
or a browser without JavaScript, gets the static chart with its caption, description and table.

Render the chart with `hooks: true` (CLI `--hooks`, Astro `hooks`), then enhance its figure from
a module the page loads itself:

```js
import { renderChart } from '@casoon/chartlet';
const { content } = renderChart(spec, { idPrefix: 'power', hooks: true });
```

```js
// a module of the page, bundled like any other import
import { enhance, crosshair, toggle, steps, play } from '@casoon/chartlet/interactive';

for (const figure of document.querySelectorAll('figure.chartlet-figure')) {
  enhance(figure, { crosshair, toggle, steps, play });
}
```

Each feature is a function of the chart, so a bundler keeps only those a page uses: the whole
module is about 3.7 KB minified and gzipped, the crosshair alone about 2.2 KB. To pass options,
wrap a feature: `{ play: (chart) => play(chart, { interval: 300 }) }`.

- **`crosshair`** (time charts, small multiples, line charts): a vertical rule with a marker on
  every value of the observation under the pointer, in every pane at once; small multiples list
  the values of the panel under the pointer. The chart becomes
  focusable; the arrow keys move from one observation to the next, Home and End jump to the ends,
  Escape closes. The values appear in an overlay beside the chart that screen readers announce
  politely. It stays while the pointer is on the chart or the overlay, or while the chart has
  focus, and closes with Escape (WCAG 1.4.13).
- **`toggle`** (time charts, small multiples): one checkbox per named series, like the series
  filter of a grouped bar chart. Switching one off hides its lines, bands and markers; the axes do
  not rescale, and the crosshair leaves it out. `toggle(chart, { legend: 'Reihen' })` names the
  group in another language.
- **`steps`**: scroll stations. An element of the page with `data-chartlet-step` (empty, or the
  chart's ID prefix) sets the chart to the state it describes once it reaches the middle of the
  viewport: `data-chartlet-highlight="Servers"` brings one series forward,
  `data-chartlet-annotation="2"` one annotation (by its index among all layers),
  `data-chartlet-range="2026-03-10 2026-03-20"` shows only that span (a year, a date, or a
  category index on a line chart), `data-chartlet-zoom="Last week"` switches to a pre-rendered
  zoom step by its label or index. A station without attributes resets the chart.
- **`play`**: reveals the data one observation at a time, with Play and Pause, a step back and
  forward, and Show all (WCAG 2.2.2). With `prefers-reduced-motion: reduce` there is no playback,
  only the step buttons. Labels are English by default; `labels` replaces them.

The module reads what the static output already holds and lays nothing out again. It needs no
inline script and no `eval`, and sets styles only through the CSSOM, so a strict
`script-src 'self'` and `style-src` with the chart's hashes keep working. A range slider over the
zoom steps is not part of it; the radio buttons above already switch them.

Try it on the [interactive demo](../../../interactive/).

### The hooks

`hooks: true` adds these attributes and nothing else; the drawing is the same byte for byte once
they are taken out again.

| Where | Hook |
|---|---|
| Root `<svg>` | `data-chartlet-type` (`time`, `multiples`, `line`, …), `data-chartlet-id` (its ID prefix) and `data-chartlet-locale` (`en` or `de`; the module labels its controls in that language) |
| Plot of each pane (time, multiples, line) | An empty `<g data-chartlet-plot="" data-pane>` with `data-x-domain`/`data-x-range` and `data-y-domain`/`data-y-range`: values and the pixels they map to, linear between neighbouring pairs. A time axis lists Unix seconds — both ends of the span, or every observation when gaps are collapsed; a numeric time axis lists its numbers in millionths; a line chart lists category indices. A logarithmic value axis adds `data-y-scale="log"`: the mapping is then linear in the logarithm of the value. |
| Data layers of a time chart or small multiples | `<g data-series data-pane data-name>` around a layer's line, area, band, candles and markers; `data-series` is the layer's index among all data layers |
| Diagram nodes (flow, state, architecture, classes, network, tree, sequence) | `data-chartlet-node` with the node's `id` from the specification on the group around the node's box and text — a step, state, component, class, network node, tree node or sequence participant; the start dot of a state diagram, which has no ID, carries none |
| Architecture connections | `data-chartlet-edge` with the connection's `id` on the group around its line, arrowheads and label; only connections that have an `id`, on shared trunks too (the branch dots carry none). A note's line, flow edges, transitions, relations and network or tree edges have no ID and carry none; in a dynamic view only the first step over a connection keeps its ID |
| Frames | `<g data-chartlet-frame>` around the frame and name of an architecture boundary or host, a class package or a composite state, with its `id` |
| Flow lanes | `<g data-chartlet-lane>` around a lane's band and name, with its `id` |
| Annotation layers | `<g data-annotation data-pane>` around a reference line, zone or point marker; `data-annotation` is its index among all layers |
| HTML data table | `data-chartlet-table` (the ID prefix) on `<table>`; `data-series`, `data-pane` and `data-part` (`value`, `lower`, `upper`, `open`, `high`, `low`, `close`) on each value column; `data-x` on each row (Unix seconds, or the row index); `data-value` with the unformatted number on each cell that has one |
| SVG profile | The same table as `<script type="application/json" data-chartlet-data="">` inside the SVG: `{ "columns": [{ "name", "series", "pane", "part" }], "rows": [{ "x", "label", "text", "value" }] }`. A JSON data block is never executed, so CSP allows it. |

The print variant carries no hooks.

### Finding a diagram node

`data-chartlet-node` is the `id` the specification gives a step, state, component, class, network
node, tree node or participant — not its label, its position or its place in the list — so two
classes both named `Order` stay apart, and reordering the specification changes nothing.
`data-chartlet-edge`, `data-chartlet-frame` and `data-chartlet-lane` work the same way for
connections, frames and lanes. None of them is a DOM ID: the desktop and the mobile variant, or
the same diagram embedded twice, carry the same value without a collision, so look it up within
one figure. A host page needs no module for this:

```js
// Highlight the class `sales-order` in the figure that holds the domain model.
const figure = document.querySelector('#model').closest('figure');
for (const node of figure.querySelectorAll('[data-chartlet-node="sales-order"]')) {
  node.classList.add('is-current');
}
```

An edge carries a hook only when it has an `id` of its own, which only architecture connections
take: a relation, a transition or a flow edge has none, and its place in the list is not a lasting
identity. The hook only marks a group; it does not make it a button or a link, so a page that
acts on a click also needs its own focusable control.
