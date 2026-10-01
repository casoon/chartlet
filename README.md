# chartlet

chartlet compiles a small JSON chart specification into a finished, accessible chart at build
time. You get plain SVG, or an HTML figure with caption, source and data table. Nothing runs in
the browser: no chart JavaScript, no hydration, no layout shift.

**Website and documentation:** [casoon.github.io/chartlet](https://casoon.github.io/chartlet/)

![Grouped bar chart comparing budget and actual costs from January to April](examples/budget-vs-actual.svg)

- **Accessible by default:** every chart carries a title and a generated description of the
  data, and the HTML output always includes the complete data as a table.
- **Deterministic:** the same specification produces the same bytes, so charts can be reviewed,
  diffed and cached like any other build artifact.
- **Honest about problems:** invalid input is rejected with a code, a path and a fix; layout
  compromises such as shortened labels are reported as warnings instead of happening silently.

> **Status:** early alpha. Bar charts (single and grouped, vertical and horizontal), categorical
> line charts, time series with uncertainty bands and reference lines, warming stripes, calendar
> heatmaps, range bars and small multiples are supported. The specification may still change before the first
> stable release.

## Quick start

Install the CLI from crates.io (Rust 1.88 or newer). While chartlet is in alpha, name the version
explicitly:

```sh
cargo install chartlet --version 0.1.0-alpha.5
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
| Topic map: separate landmasses, area by value | `"type": "topicmap"` with `topics` | [topicmap-sample](examples/topicmap-sample.json) |
| Knowledge landscape: one land, position by kinship | `"type": "atlas"` with `realms` | [knowledge-landscape](examples/knowledge-landscape.json) |
| Time series with uncertainty band, modeled (hatched, dashed) | `lower`/`upper` per point, `"modeled": true` per layer | [temperature-projection](examples/temperature-projection.json) |
| Reference lines: threshold and date marker | `"mark": "annotation"` with `value` or `time` and `label` | [annual-mean-threshold](examples/annual-mean-threshold.json) |
| Time series with area, gaps, line patterns and zoom | `"mark": "area"`, `null` values, `dash`, `"stroke": "bold"`, `zoomSteps` by time | [sensor-readings](examples/sensor-readings.json) |
| Zones and point markers | `"mark": "band"` with `from`/`to` or `bottom`/`top`; `"mark": "annotation"` with `time`, `value` and `shape` | [release-incidents](examples/release-incidents.json) |
| Candlesticks with a volume pane on one time axis | `"mark": "ohlc"` with `data`; up to four `panes` with `heightRatio` | [share-price](examples/share-price.json) |
| Warming stripes on a diverging scale | `"type": "stripes"` with `stripes` | [warming-stripes](examples/warming-stripes.json) |
| Calendar heatmap, by month or by week | `"type": "calendar"` with `calendar` | [daily-anomaly-calendar](examples/daily-anomaly-calendar.json) |
| Range bars with central value, modeled hatched | `"type": "rangebar"` with `ranges` | [warming-contributions](examples/warming-contributions.json) |
| Small multiples with a shared value axis | `"type": "multiples"` with titled `panes` | [emission-pathways](examples/emission-pathways.json) |
| Mobile variant for narrow containers, any type | `"mobile": { "width": 360 }` | [mobile-revenue](examples/mobile-revenue.json) |

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
`"mark": "area"` fills the region between a line and zero; `dash` (`solid`, `dashed`, `dotted`)
and `stroke` (`thin`, `regular`, `bold`) tell lines apart beyond color. A pane holds up to six
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
| `--format svg` | Standalone SVG with `<title>` and `<desc>` (default). |
| `--format html` | `<figure>` with caption, SVG, source and data table; with `mobile` in the specification, both variants behind a container query. |
| `--table details` | Puts the HTML data table in a native, initially closed `<details>` (default). |
| `--table visible` | Shows the data table permanently. |
| `--id-prefix <prefix>` | Stable ID of the chart root and prefix for its other IDs; needed when the same chart appears twice on one page. |
| `--variant mobile` | Renders the mobile variant alone as SVG, for a `<picture>` source; requires `mobile` in the specification (`missing_mobile`) and the SVG format. `desktop` (default) renders the chart at `width` × `height`. |
| `-o <path>` | Writes to a file instead of standard output. |
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
  including values whose visual label had to be left out.
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
| `text_truncated` | A label or title was shortened to fit. |
| `value_labels_omitted` | Some value labels had no room next to their bars. |
| `dense_chart` | More than 16 categories, or more observations in a time layer than the plot has horizontal pixels; the chart may be hard to read at this size. |
| `color_not_supported` | A layer's `color` was outside the contract and replaced by the neutral gray. |
| `topic_too_small_for_label` | A topic map area is too small to hold its own name. |
| `label_does_not_fit` | A region of a knowledge landscape has no room for its name; the area keeps its tooltip. |
| `places_did_not_fit` | A region declares more places than it has ground. |
| `realm_without_structure` | A realm holds a single region, so it has no inner structure to show. |
| `more_places_than_value` | A region lists more places than its value. |
| `label_overlap` | The label of a zone, reference line or point marker overlaps another annotation label, crosses a data line, or reaches outside the plot; the label is kept. |

## Astro

The npm package [`@casoon/chartlet`](packages/chartlet/README.md) renders charts while Astro
builds the site and ships no JavaScript to the browser. It carries the renderer as WebAssembly,
so the package is all you install:

```sh
npm install @casoon/chartlet@alpha
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

## Rust

```sh
cargo add chartlet@0.1.0-alpha.5
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

- You need continuous zooming, panning, cross-filtering, live updates or keyboard-addressable details for individual marks.
- The data changes at runtime rather than at build time.
- You need maps, networks, 3D charts or chart types beyond the ones listed above.
- You want to explore data rather than publish a finished chart.

## License

[MIT](LICENSE)
