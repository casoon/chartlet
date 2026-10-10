---
title: Chart types
description: Bars, grouped bars, lines with gaps, time series with bands and reference lines, candlesticks and stacked panes, small multiples, warming stripes, calendar heatmaps, range bars, two kinds of map, sequence diagrams, flow charts, state diagrams and architecture diagrams.
order: 1
---

| Chart | Specification | Example |
| --- | --- | --- |
| Bar, vertical | `"type": "bar"` with `data` | [Monthly revenue](../../../showcase/monthly-revenue/) |
| Bar, horizontal, with negative values | `"orientation": "horizontal"` | [Quarterly change](../../../showcase/quarterly-change/) |
| Bar with a reference line, thousands separated | `references` with `value` and `label`; `valueAxis.thousandsSeparator` | [Open support tickets](../../../showcase/ticket-backlog/) |
| Stacked bar, by value or as 100 % | `"stack": "normal"` or `"percent"` with `series` | [Electricity generation](../../../showcase/energy-mix/), [Accessibility checks](../../../showcase/audit-outcomes/) |
| Grouped bar, series told apart by form as well as color | `"patterns": true` | [Population and emissions](../../../showcase/population-and-emissions/) |
| Bar chart with error bars | `lower` and `upper` on every `data` point | [Satisfaction scores](../../../showcase/satisfaction-scores/) |
| Forest plot: squares by weight, a diamond, the line of no effect | `"type": "rangebar"` with `weight`, `summary` and `references` | [Effect of an intervention](../../../showcase/trial-effects/) |
| Timeline: phases, milestones, markers and arrows | `"type": "timeline"` with `items` and `markers` | [Product roadmap](../../../showcase/product-roadmap/) |
| Box plot from observations and from five numbers | `"type": "boxplot"` with `boxes` | [Response times](../../../showcase/response-times/) |
| Bars colored by group, one legend entry per group | `group` on every `data` point | [Effects of an evening without a phone](../../../showcase/benefit-and-harm/) |
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
| Small multiples of bars, one panel per measure with its own value axis | `"type": "multiples"` with `categories` and `values` per pane | [Web frameworks compared](../../../showcase/framework-benchmarks/) |
| Small multiples, shared value axis | `"type": "multiples"` with titled `panes` | [Emission pathways](../../../showcase/emission-pathways/) |
| Warming stripes | `"type": "stripes"` with `stripes` | [Warming stripes](../../../showcase/warming-stripes/) |
| Calendar heatmap, by month or week | `"type": "calendar"` with `calendar` | [Daily anomaly calendar](../../../showcase/daily-anomaly-calendar/) |
| Range bars with central value | `"type": "rangebar"` with `ranges` | [Warming contributions](../../../showcase/warming-contributions/) |
| Small multiples with a finding under each panel | `note` and `noteEmphasis` per pane, `"mobile": { "columns": 1 }` | [Which cause matches the warming?](../../../showcase/warming-causes/) |
| Range bars in groups, on a logarithmic axis | `group` per range, `"valueAxis": { "scale": "log" }` | [Soil animals](../../../showcase/soil-animals/) |
| Sequence diagram, portrait | `"type": "sequence"` with `participants`, `messages` and `fragments` | [Reading an item through the cache](../../../showcase/cache-lookup/) |
| Sequence diagram, landscape | `"orientation": "landscape"` in `sequence` | [Exporting a report in the background](../../../showcase/async-export/) |
| Flow chart in lanes, with a loop back and a group (experimental) | `"type": "flow"` with `nodes`, `edges`, `lanes`, `groups`, `mainPath` | [From commit to release](../../../showcase/release-flow/) |
| PRISMA study selection as a flow chart (preset, experimental) | `"type": "flow"` with `io`, `external` and `end` steps and `groups` | [Study selection](../../../showcase/study-selection/) |
| Chain of courts as a flow chart (preset, experimental) | `"type": "flow"`, `"locale": "de"` | [Instanzenzug](../../../showcase/court-instances/) |
| Flow chart, turned landscape by a wide canvas (experimental) | `"type": "flow"`, `"orientation": "auto"` | [Handling an order](../../../showcase/order-flow/) |
| State diagram with a choice, a loop and two final states (experimental) | `"type": "state"` with `states`, `transitions`, `initial` | [Life of a support ticket](../../../showcase/ticket-states/) |
| Architecture diagram with nested boundaries | `"type": "architecture"` with `components`, `connections`, `boundaries` | [Web shop on one cloud region](../../../showcase/shop-architecture/) |
| Tree with a share on every link | `"type": "tree"` with `nodes`, `parent`, `link` | [Ownership structure](../../../showcase/ownership-structure/) |
| Family tree with couples | `"type": "tree"` with `partner` and `kind` | [Three generations](../../../showcase/family-tree/) |
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

## Bars colored by group

A single-series bar chart can color its bars by group: give every `data` point a `group`. Each
group takes one palette color and one legend entry (at most four), and either every point names a
group or none does.

```json
{ "data": [
  { "label": "Sleep quality", "value": 4.5, "group": "Benefit" },
  { "label": "Missed messages", "value": 2, "group": "Harm" }
] }
```

The bars keep their full width and their own row: the groups are not series, so nothing is
stacked or missing. The description names the groups with their categories, and the data table
keeps one value per category and gains a Group column. The groups do not toggle with a series
filter. Bar charts only: a line chart refuses `group` (`option_not_supported`), and with a single
group the bars are drawn as they are.

## Small multiples of bars

Small multiples can also compare categories on several measures that have nothing in common, such
as requests per second, megabytes and milliseconds: give the chart `categories`, and each pane a
`title` and its `values`, one per category (`null` for none), with a `valueAxis` of its own (`unit`,
`scale`, `format`, `min`, `max`, …).

```json
{
  "type": "multiples",
  "categories": ["Rust", "Go", "Node"],
  "columns": 3,
  "panes": [
    { "title": "Requests per second", "note": "higher is better",
      "values": [118000, 96000, 64000], "valueAxis": { "unit": "req/s" } },
    { "title": "Cold start", "values": [3, 5, 45], "valueAxis": { "unit": "ms", "scale": "log" } }
  ]
}
```

Every panel draws horizontal bars with the category names at its left, its own ticks and its unit
under the axis; `showValues` writes the values at the bar ends. The top-level `valueAxis`,
`timeAxis`, `orientation`, `stack` and `layers` do not apply (`option_not_supported`). The
description names, for each panel, its highest and its lowest category; the data table has one
column per panel, headed by its title and unit. A phone variant with `"columns": 1` stacks the
panels. At most 20 categories.

## Error bars

A bar chart of one series draws an error bar through the end of each bar when every `data` point
has a `lower` and an `upper`: a stroke between them with a cap at either end, such as the
confidence interval of an estimate. The value lies between them (`invalid_bounds`). The value
label moves beyond the error bar, the tooltip reads `Team: 4.1 (3.9 to 4.3)`, and the data
table gains an Interval column. A chart takes error bars or groups, not both.

```json
{ "data": [
  { "label": "Free", "value": 3.4, "lower": 3.1, "upper": 3.7 },
  { "label": "Team", "value": 4.1, "lower": 3.9, "upper": 4.3 }
] }
```

## Box plots

`"type": "boxplot"` puts the distributions of several categories side by side: one box per
category, from its first to its third quartile, with the median across it, whiskers to the most
extreme values within reach and a point for every value beyond.

```json
{
  "schemaVersion": 1,
  "type": "boxplot",
  "title": "Response time by endpoint",
  "boxes": [
    { "label": "Search", "values": [42, 45, 47, 48, 50, 52, 53, 55, 58, 61, 64, 140] },
    { "label": "Report export", "min": 90, "q1": 130, "median": 160, "q3": 210, "max": 300, "outliers": [380, 450] }
  ]
}
```

- **From observations or from five numbers.** With `values` (5 to 1000 of them), chartlet computes
  the quartiles by linear interpolation, the way R and NumPy do, draws the whiskers to the last
  value within 1.5 times the box and every value beyond as a point. With `min`, `q1`, `median`,
  `q3` and `max` it draws a box that was computed elsewhere, and `outliers` adds the points. Giving
  both, or only some of the five numbers, or numbers out of order, is an error by name.
- **Orientation** is vertical by default, `"orientation": "horizontal"` lays the boxes on their
  side. `showValues` writes each median; the value axis can be linear or logarithmic.
- **Text alternative.** The description names the highest and the lowest median and the boxes with
  outliers; the data table has the five numbers of every box and its outliers; a tooltip adds the
  number of observations. 1–100 boxes with unique labels.
- **Violins and strips.** `"boxDisplay": "violin"` draws the estimated density of the
  observations (a Gaussian kernel, Silverman's bandwidth, from the lowest to the highest value) as
  an outline mirrored about the middle, every violin as wide as the others at its widest, with
  the quartile box and the median inside. `"boxDisplay": "strip"` draws every observation as a
  point, spread across the width by a fixed sequence so the same data always gives the same
  picture, with the median across. Both need `values` on every box (`values_required`); the
  value axis reaches every observation.

## Forest plots

A range bar chart becomes a forest plot — the usual picture of a meta-analysis — with three
additions: a `weight` on a span, a `summary` span and a reference line for the line of no effect.

```json
{
  "type": "rangebar",
  "orientation": "horizontal",
  "references": [{ "label": "No effect", "value": 0 }],
  "ranges": [
    { "label": "Study A (n=240)", "low": -7.8, "high": -1.2, "mid": -4.5, "weight": 24 },
    { "label": "Study B (n=120)", "low": -9.9, "high": 1.7, "mid": -4.1, "weight": 11 },
    { "label": "Overall", "low": -4.7, "high": -1.9, "mid": -3.3, "summary": true }
  ]
}
```

- A span with a `weight` is drawn as a thin line with a **square** on its `mid` whose area follows
  the weight; the heaviest span has the biggest square. A weight is above zero and needs a `mid`
  (`invalid_weight`).
- A span with `"summary": true` is an overall result: a **diamond** from `low` to `high`, widest at
  `mid` (`summary_without_mid`).
- `references` draws the line of no effect across the plot with its label; a value label never
  sits on it.
- With `showValues` each span writes its effect and interval, `−4.5 (−7.8 to −1.2)`; put the
  size of the study in its label. The data table has a Weight column, the description names the
  overall result and the reference line.

## Timelines

`"type": "timeline"` puts phases, milestones and the days that matter in rows over one time axis:
a Gantt chart, a roadmap, the course of a procedure with its deadlines.

```json
{
  "schemaVersion": 1,
  "type": "timeline",
  "title": "Product roadmap",
  "timeline": {
    "items": [
      { "id": "design", "label": "Design", "start": "2027-01-04", "end": "2027-02-26" },
      { "id": "build", "label": "Build", "start": "2027-03-01", "end": "2027-05-28", "after": ["design"] },
      { "label": "Launch", "at": "2027-06-15", "after": ["build"] }
    ],
    "markers": [{ "label": "Today", "at": "2027-02-10" }]
  }
}
```

- **Phases and milestones.** An item with `start` and `end` is a bar, an item with `at` a
  diamond; mixing them is an error (`invalid_item`). Days are ISO 8601 dates, optionally with a
  time, or Unix seconds, read in UTC. The axis runs over all items and markers with a little room
  at both ends, and its ticks are chosen like those of a time chart.
- **Groups** give items a color and a legend entry: on every item or none, at most four.
- **Markers** are dashed lines across all rows with their label on top: today, a deadline.
- **Arrows.** `after` names the `id`s of the items an item follows; an arrow runs from the end of
  each to the start of the item, below the bars. Following in a circle is an error
  (`circular_dependency`); an item that starts before the one it follows ends is a warning,
  `follows_overlap`, and its arrow runs round the long way.
- **Size.** Every item takes a row of 30 pixels; a canvas that is too low grows and says so with
  `canvas_too_small`. With `showValues` the dates stand beside each item; on a phone (a canvas
  narrower than 480 pixels) they stay in the tooltips and the table.
- **Text alternative.** The description lists every item with its days and what it follows, then
  the marked days; the data table has one row per item with its kind, start, end, group and
  predecessors. 1–60 items, up to six markers.

## Waterfalls

`"type": "waterfall"` shows how a total is made up: a running total that rises and falls step by
step, from revenue to profit, from a budget to its spending.

```json
{
  "schemaVersion": 1,
  "type": "waterfall",
  "title": "From revenue to profit",
  "waterfall": {
    "steps": [
      { "label": "Revenue", "value": 1250, "kind": "start" },
      { "label": "Materials", "value": -420 },
      { "label": "Staff", "value": -380 },
      { "label": "Gross margin", "kind": "total" }
    ]
  }
}
```

- **Kinds.** A `delta` (the default) moves the total by its value: a bar between the total before
  and after it, drawn in one color when it rises and another when it falls, and written `+60`
  or `−420`. A `start` sets the total to its value, as a bar from zero. A `total` is a bar from
  zero to the running total so far — a subtotal or the end result; it takes no value, or the
  value it names must be the running total (`total_mismatch`). A dashed line carries the total
  from one bar to the next.
- **Orientation** is vertical by default; `"orientation": "horizontal"` lays the steps on their
  side. The `valueAxis` applies as on a bar chart.
- **Text alternative.** The description gives the end result, the biggest rise and the biggest
  fall; the table lists every step with its kind, its change and the total after it. 2–40 steps
  with unique labels.

## Waffle charts

`"type": "waffle"` shows parts of a whole as squares: a grid of 100 squares in which each square is
one percent is read more easily than a pie.

```json
{
  "schemaVersion": 1,
  "type": "waffle",
  "title": "Where the electricity came from",
  "waffle": {
    "parts": [
      { "label": "Wind", "value": 31 },
      { "label": "Gas", "value": 28 }
    ],
    "total": 100
  }
}
```

- **Squares.** The whole has `cells` squares (100 by default) in rows of `columns` (the grid is about
  square by default). The parts take them by the largest remainder, in reading order, from the top
  left; every part gets at least one square, however small. With a `total` above the sum of the
  parts the squares that are left, in a pale color, stand for the rest.
- **At most four parts** (`too_many_series`), one palette color each, with unique labels and values
  above zero. The legend stands right of the grid, below it on a narrow canvas, and names every
  part with its value and share.
- **Text alternative.** The description says how many squares there are and what one stands for,
  and lists every part with its value, share and squares; the table has a row for each, with the
  rest.

## Parliament charts

`"type": "parliament"` shows who holds the seats of an assembly: one dot for every seat, in a
semicircle, in blocks by party from left to right.

```json
{
  "schemaVersion": 1,
  "type": "parliament",
  "title": "Seats after the election",
  "parliament": {
    "parties": [
      { "label": "Left alliance", "seats": 38 },
      { "label": "Social democrats", "seats": 41 },
      { "label": "Conservatives", "seats": 52 }
    ],
    "majority": true,
    "coalition": ["Left alliance", "Social democrats"]
  }
}
```

- **Parties** keep the order of the list from the left of the semicircle to the right, with one
  palette color each: up to eight parties (`too_many_series`) and 800 seats. chartlet picks the
  number of rows whose dots come out biggest.
- **Majority.** `"majority": true` draws a dashed line at the seat that makes a majority — more
  than half — and writes the number under the total.
- **Coalition.** `coalition` names the parties that govern together by their labels
  (`unknown_node` for a label that is not a party): their seats are ringed, and the legend adds
  their seats and whether they make a majority.
- **Text alternative.** The legend, the description and the table give every party with its seats
  and share, the table with a Coalition column; every dot has a tooltip with its party and its
  number.

## Treemaps

`"type": "treemap"` shows the parts of a whole as rectangles whose areas follow the values — for
many items, where a bar chart would run out of room.

```json
{
  "schemaVersion": 1,
  "type": "treemap",
  "title": "Where the budget goes",
  "treemap": {
    "items": [
      { "label": "Schools", "value": 310, "group": "Society" },
      { "label": "Roads", "value": 120, "group": "Infrastructure" },
      { "label": "Defence", "value": 150, "group": "Security" }
    ]
  }
}
```

- **Items** have unique labels and values above zero (`invalid_value`), up to 100. The rectangles
  are packed by the squarified method, the largest first, so they stay close to squares.
- **Groups.** `group` goes on every item or on none (`missing_group`): up to four groups, one
  palette color each, with a legend. Without groups all rectangles take the accent color.
- **Labels.** The name and the value stand inside a rectangle that has room for them; chartlet
  leaves them out of the smaller ones (`value_labels_omitted`) and shortens long names
  (`text_truncated`). Every rectangle has a tooltip with value and share.
- **Text alternative.** The description names the biggest items and the shares of the groups;
  the table lists every item from the largest to the smallest with value, share and group.

## Sankey diagrams

`"type": "sankey"` shows how a quantity flows from sources through stages to uses: nodes in
columns, joined by bands as thick as the flow.

```json
{
  "schemaVersion": 1,
  "type": "sankey",
  "title": "From energy source to use",
  "sankey": {
    "links": [
      { "from": "Gas", "to": "Power", "value": 120 },
      { "from": "Wind", "to": "Power", "value": 90 },
      { "from": "Power", "to": "Households", "value": 100 },
      { "from": "Power", "to": "Losses", "value": 110 }
    ]
  }
}
```

- **Links** name their ends by label; the nodes are the labels in the order they first appear. A
  value is above zero (`invalid_value`), a link joins two different nodes (`self_link`), a pair
  has one link (`duplicate_link`), and the links must not run in a circle (`link_cycle`). Up to
  100 links and 40 nodes.
- **Columns** follow the longest path from a node without incoming links; within a column the
  nodes are ordered so that the bands cross as little as they can. A node is as tall as the
  larger of what flows in and out; the flows need not balance.
- **Colors.** The nodes take the palette colors in turn, column by column from the top, so that
  neighbours differ (the four colors repeat beyond four nodes); a band takes the color of the node
  it leaves. After the barycenter sweeps, chartlet swaps neighbouring nodes while that lets fewer
  bands cross.
- **Text alternative.** The description gives the total and the biggest links; the table lists
  every link with value and share of the total.

## Survival curves

`"type": "survival"` draws Kaplan-Meier curves: for every group the share of subjects still
without the event, from the observed times.

```json
{
  "schemaVersion": 1,
  "type": "survival",
  "title": "Overall survival by treatment",
  "survival": {
    "groups": [
      { "label": "Standard", "observations": [{ "time": 3.1 }, { "time": 8, "event": false }, { "time": 12.4 }] },
      { "label": "New drug", "observations": [{ "time": 6.3 }, { "time": 14 }, { "time": 20, "event": false }] }
    ],
    "confidence": true,
    "timeTitle": "Months since randomization"
  }
}
```

- **Observations.** A `time` is zero or more (`invalid_value`); `"event": false` means the subject
  left the study before the event (censored) and is marked on the curve by a short vertical tick.
  Every group has 2 to 2000 observations (`invalid_values`), up to four groups, one palette color
  each.
- **The curve** drops at every time with events by the share of those still at risk, and runs to the
  last observed time. `"confidence": true` adds the 95 % band, computed from Greenwood's variance
  on the log-log scale, so that it stays between 0 % and 100 %.
- **Number at risk.** Under the time axis, on by default (`"atRisk": false` leaves it out): how
  many observations reach each tick of the axis or beyond, for every group.
- **Text alternative.** The description gives every group's observations, events, median and
  survival at the end (the median is "not reached" while the curve stays above 50 %); the table
  has the survival, the limits if drawn and the number at risk of every group at every tick.

## Scatter plots

`"type": "scatter"` draws points on two numeric axes — a volcano plot, a Manhattan plot, residuals,
any cloud of observations — with groups, threshold lines and names on the points that matter.

```json
{
  "schemaVersion": 1,
  "type": "scatter",
  "title": "Differential expression after treatment",
  "scatter": {
    "points": [
      { "x": -4.0, "y": 9.3, "group": "Down", "label": "BRCA1" },
      { "x": 2.5, "y": 6.5, "group": "Up", "label": "MYC" },
      { "x": 0.2, "y": 0.4, "group": "Not significant" }
    ],
    "lines": [
      { "axis": "x", "value": -1 },
      { "axis": "x", "value": 1 },
      { "axis": "y", "value": 1.3, "label": "p = 0.05" }
    ],
    "xTitle": "log2 fold change",
    "yTitle": "−log10 p"
  }
}
```

- **Points** have finite `x` and `y`, up to 5000. `group` goes on every point or on none
  (`missing_group`): up to four groups, one palette color each, with a legend — the first point of
  each group fixes the order and so the color. A Manhattan plot colors its chromosomes alternately
  with two groups.
- **Lines** run across the plot at a value of `x` (vertical) or `y` (horizontal), up to eight, with
  an optional label; the axes reach them.
- **Names.** A point with a `label` gets its name beside it, in the order of the list; a name that
  would run over one already written is left out (`value_labels_omitted`).
- **Many points.** Up to 300 points have a tooltip each; beyond that the dots are small and plain.
  The data table lists every point up to 100; for more, only the labeled ones, or the 20 highest
  if none is labeled, and its caption says how many of how many.
- **Text alternative.** The description gives the number of points, the span of both axes, the
  points per group and how many lie above each line.

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
order they are sent.

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
where anything goes. Flow charts are **experimental**: their layout may still change between minor
releases.

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

**Focus.** In the HTML profile a diagram can be focused on one node — a step, a state, a
component, a participant: a click on the node, or a choice in the "Focus" list above the figure,
keeps that node, its neighbours and the edges between them as they are and fades the rest; "Show
all" brings everything back. There is no script: the list is a group of radio buttons with the
usual keyboard, the nodes carry labels of those buttons laid over them, and CSS `:has()` does
the rest. The SVG stays an image to assistive technology. A browser without `:has()` shows the
diagram unchanged.

**Narrow canvases.** Below 480 pixels — a mobile variant, say — diagrams turn compact: smaller
margins and gaps, narrower steps whose labels wrap onto two lines, participant names of sequence
diagrams on two lines instead of shortened, and labels kept on the page by moving them to the
other side of their edge. A diagram still wider than its canvas grows and is scaled down by the
page; `canvas_too_small` names the width it needs.

**Look.** Both diagram types share one visual language: every kind of step or participant has
its own shape and a role color that repeats it — blue for steps and systems, green for start and
end, amber for decisions, teal for data stores, violet for input and output and for queues, red
for security, gray for anything external — each at least 3:1 against the background in both themes. Steps carry a
soft shadow, edges turn with rounded corners, and edge labels sit on small chips in the background
color. The colors are CSS custom properties (`--chartlet-role-blue`, `--chartlet-role-blue-fill`,
…) that a host page can override.

Limits: 1–40 steps with unique `id`s, up to 80 edges, 8 lanes and 8 groups.

## State diagrams

A state diagram (`"type": "state"`) shows the states something can be in and the transitions
between them. It uses the layout of flow charts — layers, the main path, cycles drawn against the
flow, loops on a state, portrait and landscape — and is **experimental** like them.

```json
{
  "schemaVersion": 1,
  "type": "state",
  "title": "Door",
  "state": {
    "initial": "closed",
    "states": [
      { "id": "closed", "label": "Closed" },
      { "id": "open", "label": "Open" },
      { "id": "locked", "label": "Locked" },
      { "id": "gone", "label": "Removed", "final": true }
    ],
    "transitions": [
      { "from": "closed", "to": "open", "event": "push", "guard": "unlocked" },
      { "from": "open", "to": "closed", "event": "release" },
      { "from": "closed", "to": "locked", "event": "lock", "action": "beep" },
      { "from": "locked", "to": "closed", "event": "unlock" },
      { "from": "locked", "to": "gone", "event": "dismantle" }
    ]
  }
}
```

- **States** are boxes with well rounded corners; `"final": true` gives a state a double outline,
  `"kind": "choice"` makes it a diamond the machine passes through at once, by its guards.
- **Composite states.** `"kind": "composite"` makes a state a frame around the states that name
  it with `"in"`, nested up to four deep. Transitions connect the states inside it; one that names
  the composite state itself is `composite_not_allowed`.
- **`initial`** names the state the machine starts in; it gets a dot with an arrow into it. Once
  it is set, a state no transition leads to is reported as `unreachable_state`.
- **Transitions** carry an `event`, a `guard` and an `action`, each optional and each up to 40
  characters, written `event [guard] / action`; `dash` works as on flow chart edges.
- **Text alternative.** The description says where the machine starts and ends and lists every
  transition in reading order; the data table has one row per transition, with event, guard and
  action in columns of their own.

Limits: 1–40 states with unique `id`s and up to 80 transitions; an unknown `id` is
`unknown_state`.

## Architecture diagrams

An architecture diagram (`"type": "architecture"`) shows the components of a system, how they
connect and where they run. It uses the layout of flow charts.

The canvas size is yours: a diagram that needs more room grows the canvas and says so with
`canvas_too_small`, but one that needs less is not shrunk, so a landscape diagram on a tall canvas
leaves space above and below. With `orientation` left at `auto`, chartlet takes the orientation that
grows the canvas least; set `orientation` to pin it, and `width` and `height` to what you want.

```json
{
  "schemaVersion": 1,
  "type": "architecture",
  "title": "Blog",
  "architecture": {
    "boundaries": [
      { "id": "cloud", "label": "Cloud" },
      { "id": "private", "label": "Private network", "in": "cloud" }
    ],
    "components": [
      { "id": "reader", "label": "Reader", "kind": "person" },
      { "id": "site", "label": "Site", "kind": "frontend" },
      { "id": "api", "label": "API", "in": "private" },
      { "id": "db", "label": "Posts", "kind": "database", "in": "private" }
    ],
    "connections": [
      { "from": "reader", "to": "site", "label": "reads" },
      { "from": "site", "to": "api", "label": "loads", "technology": "HTTPS" },
      { "from": "api", "to": "db", "technology": "SQL" }
    ]
  }
}
```

- **Components** have a `kind`, each with its own shape and role color: `person` (a box with a
  head), `frontend` (a box with a window bar), `service` (a box, the default), `database` (a
  cylinder), `queue` (a box with a stack behind it), `storage` (a bucket), `cache` (a hexagon),
  `security` (a shield, for an identity provider, a firewall or a vault) and `external` (a dashed
  box).
- **Boundaries** are frames — a cloud region, a network, a zone, a system. A component lies `in`
  a boundary, and a boundary `in` another one, up to four deep. Every boundary holds at least one
  component (`empty_boundary`), and boundaries cannot lie in each other in a circle
  (`boundary_cycle`). chartlet keeps components that do not belong to a boundary out of its frame.
- **Connections** carry a `label` for what they do and a `technology` for how, written in
  brackets on a line below it.
- **Text alternative.** The description says what each boundary holds and where each component
  connects to; the data table has one row per component with its kind, its boundaries
  (`Cloud › Private network`) and its connections.

Limits: 1–40 components, up to 80 connections and 12 boundaries; identifiers are unique among
components and boundaries together.

## Trees

A tree (`"type": "tree"`) draws a root and the nodes below it: an organization chart, an
ownership structure, a family, a hierarchy of norms or components. Every node but the root names its `parent`; the children of a
node keep the order of the list. chartlet lays the tree out by itself: one row per level, the
children of a node side by side, the node centered over them, and subtrees pushed together until
they are a gap apart at every level, so that a narrow subtree tucks in under a broad neighbour.

```json
{
  "schemaVersion": 1,
  "type": "tree",
  "title": "Group structure",
  "tree": {
    "nodes": [
      { "id": "holding", "label": "Holding", "sublabel": "parent company" },
      { "id": "retail", "label": "Retail", "parent": "holding", "link": "100 %" },
      { "id": "digital", "label": "Digital", "parent": "holding", "link": "60 %" },
      { "id": "labs", "label": "Labs", "parent": "digital", "link": "50 %" }
    ]
  }
}
```

- **Nodes** have a `label`, an optional `sublabel` (a role, a legal form) and a `parent`. Exactly
  one node has no parent: the root (`invalid_root`). Parents must exist (`unknown_node`) and run
  up to the root without a circle (`circular_parent`); the tree goes at most twelve levels
  below the root (`tree_too_deep`).
- **Kinds.** `kind` gives a node its shape: `unit` (a box, the default), `person` (a box with a
  head) and `external` (a dashed box).
- **Couples.** `"partner": "id"` joins a node to another as a couple: the two stand side by side
  (one above the other in landscape) with a short line between them, and the children of either
  hang from the middle of that line, as in a family tree. A partner has no `parent` of its own
  (`invalid_partner`); the node it names is the head of the couple.
- **Links** are the lines from a parent to its children. A node's `link` is written on the line
  to its parent, for a share, a weight or a relation; the root has none.
- **Orientation.** `portrait` runs down from the root, `landscape` to the right; `auto` takes the
  one that grows the canvas less, as in the other diagrams.
- **Text alternative.** The description names the root and, for every node with children, what
  hangs below it; the data table has one row per node in reading order, with its level, its
  parent, its link and its children.

Limits: 1–150 nodes. A node has one parent; a node with two parents (a shared holding) is not a
tree — use an architecture or a flow diagram. The table of a tree with couples gains a Partner
column.

## Time series

`"type": "time"` drops categories altogether: an observation carries a timestamp, the ticks land
on calendar boundaries, and the layer list lives in `panes`. Bars and grouped series do not apply
to it, and a `time` chart refuses them by name instead of ignoring them. Its `zoomSteps` name a
window of time instead of a range of categories.
The [time series guide](time-series.md) covers timestamps, timezone, per-layer colors, the dark
theme, and the limits for dense data.
