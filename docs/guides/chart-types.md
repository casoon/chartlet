---
title: Chart types
description: Bars, grouped bars, lines with gaps, time series with bands and reference lines, candlesticks and stacked panes, small multiples, warming stripes, calendar heatmaps, range bars, two kinds of map, sequence diagrams and flow charts.
order: 1
---

| Chart | Specification | Example |
| --- | --- | --- |
| Bar, vertical | `"type": "bar"` with `data` | [Monthly revenue](../../../showcase/monthly-revenue/) |
| Bar, horizontal, with negative values | `"orientation": "horizontal"` | [Quarterly change](../../../showcase/quarterly-change/) |
| Bar with a reference line, thousands separated | `references` with `value` and `label`; `valueAxis.thousandsSeparator` | [Open support tickets](../../../showcase/ticket-backlog/) |
| Stacked bar, by value or as 100 % | `"stack": "normal"` or `"percent"` with `series` | [Electricity generation](../../../showcase/energy-mix/), [Accessibility checks](../../../showcase/audit-outcomes/) |
| Grouped bar, series told apart by form as well as color | `"patterns": true` | [Population and emissions](../../../showcase/population-and-emissions/) |
| Grouped bar, up to four series | `categories` and `series` instead of `data` | [Budget vs. actual](../../../showcase/budget-vs-actual/) |
| Line with several series, color and pattern per series | `"type": "line"` with `categories` and `series` | [Weekly visitors](../../../showcase/visitors-by-channel/) |
| Line on a logarithmic axis | `"valueAxis": { "scale": "log" }` | [Earthquakes per year](../../../showcase/quake-frequency/) |
| Line with gaps for missing values | `"type": "line"`, `null` values | [Monthly trend](../../../showcase/monthly-trend/) |
| Time series on a calendar axis | `"type": "time"` with `panes` and `layers` | [Daily orders](../../../showcase/daily-orders/) |
| Time series, dark theme, declared colors | `"theme": "dark"`, `color` per layer | [Revenue vs. forecast](../../../showcase/revenue-vs-forecast/) |
| Separate landmasses, area by value (experimental) | `"type": "topicmap"` with `topics` | [Insights themes](../../../showcase/topicmap-sample/) |
| One continuous land, position by kinship (experimental) | `"type": "atlas"` with `realms` | [Documentation by area](../../../showcase/knowledge-landscape/) |
| Time series with uncertainty band, modeled | `lower`/`upper` per point, `"modeled": true` | [Temperature projection](../../../showcase/temperature-projection/) |
| Threshold and date marker | `"mark": "annotation"` with `value` or `time` | [Annual mean and threshold](../../../showcase/annual-mean-threshold/) |
| Time series with area, gaps and zoom | `"mark": "area"`, `null` values, `dash`, `zoomSteps` by time | [Data hall power draw](../../../showcase/sensor-readings/) |
| Readings around a trend (scatter) | `"mark": "point"` next to a `line` layer | [Resting heart rate](../../../showcase/resting-heart-rate/) |
| Stacked areas | `"stack": "normal"` on a pane of `area` layers, `"legend": "end"` | [Generation by source](../../../showcase/generation-mix/) |
| Zones and point markers | `"mark": "band"` with `from`/`to` or `bottom`/`top`; `"mark": "annotation"` with `time`, `value` and `shape` | [Checkout API error rate](../../../showcase/release-incidents/) |
| Candlesticks with a volume pane | `"mark": "ohlc"` with `data`; several `panes` with `heightRatio`; `"gaps": "collapse"` | [Daily share price](../../../showcase/share-price/) |
| Profile or deep time on a numeric axis, reversed | `"timeAxis": { "kind": "number", "reverse": true }` | [Deep-sea oxygen isotopes](../../../showcase/deep-sea-isotopes/) |
| Small multiples, each panel on its own value axis | `"independentAxes": true` | [Indicators of growth](../../../showcase/acceleration-indicators/) |
| Small multiples, shared value axis | `"type": "multiples"` with titled `panes` | [Emission pathways](../../../showcase/emission-pathways/) |
| Warming stripes | `"type": "stripes"` with `stripes` | [Warming stripes](../../../showcase/warming-stripes/) |
| Calendar heatmap, by month or week | `"type": "calendar"` with `calendar` | [Daily anomaly calendar](../../../showcase/daily-anomaly-calendar/) |
| Range bars with central value | `"type": "rangebar"` with `ranges` | [Warming contributions](../../../showcase/warming-contributions/) |
| Small multiples with a finding under each panel | `note` and `noteEmphasis` per pane, `"mobile": { "columns": 1 }` | [Which cause matches the warming?](../../../showcase/warming-causes/) |
| Range bars in groups, on a logarithmic axis | `group` per range, `"valueAxis": { "scale": "log" }` | [Soil animals](../../../showcase/soil-animals/) |
| Sequence diagram, portrait (experimental) | `"type": "sequence"` with `participants`, `messages` and `fragments` | [Reading an item through the cache](../../../showcase/cache-lookup/) |
| Sequence diagram, landscape (experimental) | `"orientation": "landscape"` in `sequence` | [Exporting a report in the background](../../../showcase/async-export/) |
| Flow chart in lanes, with a loop back and a group (experimental) | `"type": "flow"` with `nodes`, `edges`, `lanes`, `groups`, `mainPath` | [From commit to release](../../../showcase/release-flow/) |
| Flow chart, turned landscape by a wide canvas (experimental) | `"type": "flow"`, `"orientation": "auto"` | [Handling an order](../../../showcase/order-flow/) |
| Mobile variant for narrow containers, any type | `"mobile": { "width": 360 }` | [Monthly revenue by sales channel](../../../showcase/mobile-revenue/) |

Each example in the repository's `examples/` folder has its rendered `.svg` and `.html` next to it;
one with a mobile variant also has its `.mobile.svg`, and two have their [print variant](print.md)
as `.print.svg`.
The SVG files are also the reference output of the test suite.

## Several series

For bars and lines, replace `data` with `categories` and `series`. Every series needs exactly one value per category.
A legend is added automatically, and the data table gets one column per series. Lines tell their series apart by color and by pattern — solid, dashed, dotted, thin — so that
color is never the only difference; with several lines, value labels are left out and the tooltips
and the data table carry the values.

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

## Stacked bars

With `categories` and `series`, `"stack": "normal"` stacks the series of each category into one
bar instead of setting them side by side: positive values stack upward from zero, negative ones
downward, and the total is written beyond each stack. `"stack": "percent"` shows each category as
100 %, every segment a share of the category's total; its values must be zero or more. A segment
carries its value or share inside when it fits, and every segment keeps its tooltip. The
description names the highest and lowest total; the data table lists the values as given.

```json
{
  "schemaVersion": 1,
  "type": "bar",
  "stack": "percent",
  "title": "Accessibility checks by page",
  "categories": ["Home", "Search"],
  "series": [
    { "name": "Failed", "values": [4, 9] },
    { "name": "Passed", "values": [38, 30] }
  ]
}
```

## Reference lines

A bar chart takes up to four reference lines, such as an average or a target. Each runs across
the plot at its value, behind the bars, with its label on top; the value axis widens to reach it,
and the description names it.

```json
"references": [{ "value": 2500, "label": "Target" }]
```

## Missing values

In a line chart, `null` marks an observation that does not exist. chartlet leaves a visible gap
instead of drawing a line across it. In a grouped bar chart, `null` leaves out that bar. The data
table shows these cells as “Missing”. A single-series bar chart requires a value for every
category.

## Maps

Two chart types draw subjects as land, and they answer different questions.

Both map types are **experimental**: their layout is tuned to one production site so far and may
change more between minor releases than the other types do.

A **topic map** (`"type": "topicmap"`) packs one landmass per subject, with the area of each
proportional to its value. It answers *how much is there of what*. Subjects sit where they fit, so
position carries nothing; what does not belong to the main systematic is listed under `islands` and
drawn beside the rest.

A **knowledge landscape** (`"type": "atlas"`) tiles one continuous land instead, so that *where* a
region lies says as much as how large it is. Use it when the relationships between subjects matter
as much as their sizes.

```json
{
  "schemaVersion": 1,
  "type": "atlas",
  "title": "Documentation by area",
  "atlas": {
    "realms": [
      {
        "label": "Platform",
        "regions": [
          { "label": "Runtime", "value": 148, "places": [{ "label": "Process model", "weight": 2.4 }] },
          { "label": "Storage", "value": 96 }
        ]
      },
      {
        "label": "Interfaces",
        "regions": [{ "label": "HTTP API", "value": 112 }]
      }
    ],
    "links": [{ "from": "Storage", "to": "HTTP API", "weight": 0.5 }]
  }
}
```

Three levels, and each one is a thing on the finished map:

- A **realm** is one landscape. Its regions are laid out inside it, so it always arrives in one
  piece — which is what lets a host page zoom into it as a single shape.
- A **region** is an area within a realm. Its `value` decides how much of its realm it holds.
- A **place** is a single entry, drawn as a point inside its region. `weight` makes it stand out;
  `tooltip` gives it a line of detail on hover.

`links` name two **regions** and may cross realm borders. Kinship cannot move a region out of its
own realm, so what it does instead is leave it on the border facing its kin — which is how a
subject that mediates between two realms ends up between them, without anyone placing it there.

### Size, and why it is damped

`areaDamping` is the exponent a region's area follows: `1` makes it proportional to its value,
and the default `0.5` makes it follow the square root. Two subjects at 292 and 14 entries are a
factor of 21 apart; damped, their areas are about 4.6 apart. The larger one stays larger without
deciding the whole map.

Nothing is lost by that. The **contour lines** are drawn from entries per unit of ground, so the
part of the quantity the area gave up is exactly the part the terrain now carries: dense subjects
stand higher. Set `"contours": false` to leave the terrain out; the land itself does not change.

### What a host page can address

Every shape carries classes, and they are the contract for building an interaction on top:

| Class | On |
| --- | --- |
| `chartlet-atlas-realm-N` | the shape of realm *N*, in declaration order |
| `chartlet-atlas-region` + `chartlet-topic-N` | the outline of region *N*, counted across all realms |
| `chartlet-atlas-place` + `chartlet-topic-N` | every place, tagged with the region it sits in |
| `chartlet-atlas-coast` | the coastline |
| `chartlet-atlas-contour` | one contour line |
| `chartlet-atlas-realm-label`, `chartlet-atlas-label` | the names |

A realm's extent is its shape: take every element of its class and union their bounding boxes.
chartlet renders static SVG, so panning and zooming belong to the host — the classes are what it
needs to do that without re-deriving anything from the geometry.

A region whose area has no room for its own name at a readable size is left unnamed, and
`label_does_not_fit` says which one. The area keeps its tooltip either way.

### What a host page can address on a topic map

A topic map numbers its areas the same way: the topics in the order the specification lists
them, then the islands.

| Class or attribute | On |
| --- | --- |
| `chartlet-topic-area` + `chartlet-topic-N` | the filled outline of area *N* |
| `data-cx`, `data-cy` | on that outline: the center the area was placed around, in viewBox units |
| `chartlet-topic-label`, `chartlet-topic-value` + `chartlet-topic-N` | the name and value inside area *N* |
| `chartlet-topic-outside`, `chartlet-topic-outside-value` + `chartlet-topic-N` | the same, set beside the map when the area has no room |
| `chartlet-topic-link` | a line between two related areas |

The center is where a route between areas should start and end: it is rounded like every other
coordinate in the SVG, and reading it saves recomputing it from the wobbling outline.

## Sequence diagrams

A sequence diagram (`"type": "sequence"`) shows participants and the messages they exchange, in the
order they are sent. It is **experimental**, like the maps: it opens a family of software diagrams
whose shared parts may still change between minor releases.

```json
{
  "schemaVersion": 1,
  "type": "sequence",
  "title": "Signing in",
  "sequence": {
    "participants": [
      { "id": "user", "label": "User", "kind": "actor" },
      { "id": "app", "label": "App", "sublabel": "web" },
      { "id": "db", "label": "Accounts", "kind": "database" }
    ],
    "messages": [
      { "from": "user", "to": "app", "label": "sign in" },
      { "from": "app", "to": "db", "label": "find account" },
      { "from": "db", "to": "app", "label": "account", "kind": "reply" },
      { "from": "app", "to": "user", "label": "welcome", "kind": "reply" }
    ],
    "fragments": [{ "kind": "opt", "label": "known account", "from": 2, "to": 3 }],
    "numbered": true
  }
}
```

- **Participants** have a `kind`, and every kind its own shape, so that it never rests on color:
  `service` (a box, the default), `actor` (a figure), `database` (a cylinder), `queue` (a box with a
  stack behind it) and `external` (a dashed box).
- **Messages** are a `call` (solid line, filled head; the default), a `reply` (dashed line, open
  head) or `async` (solid line, open head). A message whose `from` and `to` are the same draws a
  loop. A call activates its receiver until the receiver replies — or, without a reply, until the
  last message it takes part in — and the activation is drawn as a bar on its lifeline.
- **Fragments** frame a run of messages, `from` and `to` counted from 0: `alt`, `opt`, `loop`,
  `par`, `critical` and `break`, with an optional `label` as condition. Only `alt` takes `else`,
  further branches that each begin at a later message. Fragments nest up to three deep; two that
  overlap without one enclosing the other are `fragments_cross`.
- **Orientation.** `portrait` sets the participants side by side and runs time down, for tall
  formats; `landscape` sets them one below the other and runs time right, for wide formats. The
  default, `auto`, takes portrait where the diagram fits the canvas that way, landscape where only
  that fits, and otherwise the one that has to grow the canvas less — and decides again for a
  mobile variant at its own size.
  Spare room spreads the messages out a little; a canvas too small for the diagram grows to the
  size it needs, with the warning `canvas_too_small` naming that size.
- **Numbers.** `"numbered": true` puts each message's number in a round badge at the start of
  its arrow; description and table number the messages either way.
- **Text alternative.** The description names every participant and lists every message in
  order, then every fragment; the data table has one row per message with its number, sender,
  receiver, label, kind and the fragments around it. Arrows carry a tooltip such as
  `2. App → Accounts: find account`.

Limits: 1–12 participants with unique `id`s (a letter, then letters, digits, `-` or `_`), 1–60
messages, up to 12 fragments. An `id` no participant has is `unknown_participant`.

## Flow charts

A flow chart (`"type": "flow"`) draws steps joined by arrows. chartlet lays it out by itself, in
layers along the direction of the flow; the specification says only what is connected, never
where anything goes. Like the sequence diagram it is **experimental**.

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

- **Steps** have a `kind`, each with its own shape: `process` (a box, the default), `start` and
  `end` (pills, the end with a strong outline), `decision` (a diamond), `io` (a slanted box),
  `subprocess` (a box with double sides), `store` (a cylinder) and `external` (a dashed box). A
  `sublabel` adds a smaller second line; a long label wraps onto two.
- **Edges** may carry a `label` and a `dash` (`solid`, `dashed`, `dotted`). They may form cycles —
  a retry, a loop back for changes — and an edge from a step to itself is drawn as a loop beside
  it. Say what an edge means in its label; the line pattern only repeats it.
- **The main path** names the usual way through the flow as step ids joined by edges. chartlet
  keeps it in line where the lanes leave room and draws it thicker in the accent color;
  `main_path_gap` reports two consecutive steps no edge joins.
- **Lanes** are bands across the flow, one per role or system: columns in portrait, rows in
  landscape. Once a chart has `lanes`, every step names its `lane` (`missing_lane`).
- **Groups** frame steps of one lane under a name (`group_spans_lanes` otherwise). A step outside
  a group that ends up inside its frame is reported as `group_overlap`.
- **Orientation** works as for sequence diagrams: portrait runs the flow down, landscape right,
  and `auto` picks by the canvas.
- **Text alternative.** The description names the steps in reading order — layer by layer — and
  where each one leads, with the main path, the lanes and the groups; the data table has one row
  per step with its kind, lane, group and next steps.

**Look.** Both diagram types share one visual language: every kind of step or participant has
its own shape and a role color that repeats it — blue for steps and systems, green for start and
end, amber for decisions, teal for data stores, violet for input and output and for queues, gray
for anything external — each at least 3:1 against the background in both themes. Steps carry a
soft shadow, edges turn with rounded corners, and edge labels sit on small chips in the background
color. The colors are CSS custom properties (`--chartlet-role-blue`, `--chartlet-role-blue-fill`,
…) that a host page can override.

Limits: 1–40 steps with unique `id`s, up to 80 edges, 8 lanes and 8 groups.

## Time series

`"type": "time"` drops categories altogether: an observation carries a timestamp, the ticks land
on calendar boundaries, and the layer list lives in `panes`. Bars and grouped series do not apply
to it, and a `time` chart refuses them by name instead of ignoring them. Its `zoomSteps` name a
window of time instead of a range of categories.
The [time series guide](time-series.md) covers timestamps, timezone, per-layer colors, the dark
theme, and the limits for dense data.
