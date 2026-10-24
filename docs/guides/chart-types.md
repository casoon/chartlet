---
title: Chart types
description: Bars, grouped bars, lines with gaps, time series with bands and reference lines, candlesticks and stacked panes, small multiples, warming stripes, calendar heatmaps, range bars, sequence diagrams, flow charts, state diagrams, architecture diagrams and networks.
order: 1
---

| Chart | Specification | Example |
| --- | --- | --- |
| Bar, vertical | `"type": "bar"` with `data` | [Monthly revenue](../../../showcase/monthly-revenue/) |
| Bar, horizontal, with negative values | `"orientation": "horizontal"` | [Quarterly change](../../../showcase/quarterly-change/) |
| Bar with a reference line, thousands separated | `references` with `value` and `label`; `valueAxis.thousandsSeparator` | [Open support tickets](../../../showcase/ticket-backlog/) |
| Stacked bar, by value or as 100 % | `"stack": "normal"` or `"percent"` with `series` | [Electricity generation](../../../showcase/energy-mix/), [Accessibility checks](../../../showcase/audit-outcomes/) |
| Grouped bar, series told apart by form as well as color | `"patterns": true` | [Population and emissions](../../../showcase/population-and-emissions/) |
| Grouped bars with error bars | `lower` and `upper` arrays on each `series` | [Satisfaction by quarter](../../../showcase/satisfaction-by-quarter/) |
| Line chart with error bars | `lower` and `upper` on every `data` point of a `line` chart | [Monthly satisfaction](../../../showcase/monthly-satisfaction/) |
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
| Class diagram or entity-relationship diagram | `"type": "classes"` with `classes` and `relations` | [Order model](../../../showcase/order-model/), [Shop database](../../../showcase/shop-schema/) |
| Flow chart in lanes, with a loop back and a group (experimental) | `"type": "flow"` with `nodes`, `edges`, `lanes`, `groups`, `mainPath` | [From commit to release](../../../showcase/release-flow/) |
| PRISMA study selection as a flow chart (preset, experimental) | `"type": "flow"` with `io`, `external` and `end` steps and `groups` | [Study selection](../../../showcase/study-selection/) |
| Chain of courts as a flow chart (preset, experimental) | `"type": "flow"`, `"locale": "de"` | [Instanzenzug](../../../showcase/court-instances/) |
| Flow chart, turned landscape by a wide canvas (experimental) | `"type": "flow"`, `"orientation": "auto"` | [Handling an order](../../../showcase/order-flow/) |
| State diagram with a choice, a loop and two final states (experimental) | `"type": "state"` with `states`, `transitions`, `initial` | [Life of a support ticket](../../../showcase/ticket-states/) |
| Architecture diagram with nested boundaries | `"type": "architecture"` with `components`, `connections`, `boundaries` | [Web shop on one cloud region](../../../showcase/shop-architecture/) |
| Tree with a share on every link | `"type": "tree"` with `nodes`, `parent`, `link` | [Ownership structure](../../../showcase/ownership-structure/) |
| Family tree with couples | `"type": "tree"` with `partner` and `kind` | [Three generations](../../../showcase/family-tree/) |
| Tree with boxes colored by value | `value` on the nodes, `valueLabel` | [Budget by department](../../../showcase/department-budgets/) |
| Flow chart with the shares of each step | `parts` on the steps | [How insurance claims were settled](../../../showcase/claim-handling/) |
| Dendrogram built from a distance matrix | `"type": "tree"` with `distances` and `linkage` | [How alike the climates are](../../../showcase/climate-similarity/) |
| Network on a circle, groups together, edges by weight | `"type": "network"` with `nodes`, `edges`, `group`, `weight` | [Joint papers](../../../showcase/co-authors/) |
| Network in rings around one node, with arrowheads | `"layout": "rings"`, `center`, `"directed": true` | [What the checkout service depends on](../../../showcase/service-dependencies/) |
| Architecture diagram with weighted connections | `connections[].weight`, `"mainPath": { "from", "to" }` | [Requests per second](../../../showcase/request-volume/) |
| Architecture diagram with its databases on one rank | `"sameRank": [["catalogDb", "ordersDb", "billingDb"]]` | [Order platform and its data stores](../../../showcase/data-tier/) |
| Mobile variant for narrow containers, any type | `"mobile": { "width": 360 }` | [Monthly revenue by sales channel](../../../showcase/mobile-revenue/) |

Each example in the repository's `examples/` folder has its rendered `.svg` and `.html` next to it;
one with a mobile variant also has its `.mobile.svg`, and two have their [print variant](../print/)
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
table gains an Interval column. A chart takes error bars or groups, not both. A single-series
`line` chart takes them the same way: a stroke through each point, the scale reaching every limit.
With several series, each series takes `lower` and `upper` arrays next to its `values` (one entry
per category, `null` where the value is `null`; `missing_bounds`, `series_length_mismatch`,
`invalid_bounds`), on grouped bars and on lines, not on a stack: the bars of a group each get
their own error bar, tooltips give each interval, and the table gains an interval column per
series.

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
  the quartiles by linear interpolation between the two nearest observations, draws the whiskers to the last
  value within 1.5 times the box and every value beyond as a point. With `min`, `q1`, `median`,
  `q3` and `max` it draws a box that was computed elsewhere, and `outliers` adds the points. Giving
  both, or only some of the five numbers, or numbers out of order, is an error by name.
- **Orientation** is vertical by default, `"orientation": "horizontal"` lays the boxes on their
  side. `showValues` writes each median; the value axis can be linear or logarithmic.
- **Text alternative.** The description names the highest and the lowest median and the boxes with
  outliers; the data table has the five numbers of every box and its outliers; a tooltip adds the
  number of observations. 1–100 boxes with unique labels.
- **Groups.** `group` on every box (or none) gives the boxes of one group a palette color and a
  legend entry, up to four (`too_many_series`, `missing_group`); it colors boxes, violins and strips
  alike and adds a Group column to the table.
- **Mean.** `"showMean": true` marks the mean of each box's observations with a diamond (on boxes,
  violins and strips alike), adds it to the tooltip and a Mean column to the table. It needs
  `values` on every box (`values_required`).
- **Violins and strips.** `"boxDisplay": "violin"` draws the estimated density of the
  observations (a Gaussian kernel, Silverman's bandwidth, from the lowest to the highest value) as
  an outline mirrored about the middle, every violin as wide as the others at its widest (compare
  shapes; `"violinScale": "area"` gives them the same area, so widths compare, and `"count"` an area
  that follows the number of observations), with
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
- **Group rows.** `"groupRows": true` puts the rows of one group together, in the order the groups
  first appear, instead of in the order of `items` (it needs groups); the description and the
  table keep the listed order.
- **Time zone.** `"timezone"` names the zone the days are read and written in: `UTC`, a fixed
  offset such as `"+02:00"`, or one of about 55 common IANA names such as `"Europe/Berlin"`,
  `"America/New_York"`, `"Asia/Tokyo"` or `"Australia/Sydney"` (`invalid_timezone` otherwise).
  A bare date or a time without an offset is the time on the wall there, as written; an instant —
  Unix seconds, or a time with its own offset — is shifted by the offset in force then, an hour
  more in summer. The axis is laid out in wall-clock time, so every day is equally wide, and the
  ticks, the dates and the weekends are in the zone. The daylight saving rules are those in force
  today (Europe since 1996, the USA since 2007): earlier dates may be an hour off. Names outside
  the table are refused; a fixed offset always works.
- **Weekends.** `"weekends": true` shades Saturdays and Sundays (in the timeline's zone) as pale columns behind the
  rows while the axis spans at most 120 days; on a longer axis they would be slivers, so they are
  left out and `weekends_hidden` says so.
- **Progress.** `progress` (0 to 1) on a phase draws that share of its bar darker, adds it to the
  tooltip, the description and a Progress column of the table (`progress_on_milestone` for a
  milestone, `invalid_value` outside 0 to 1).
- **Summary phases.** `rollup` on a phase names the ids of the phases it sums up (`unknown_node`,
  `invalid_rollup` for a milestone, itself or another summary): its progress is theirs, each
  weighted by its length, a phase without progress counting as not begun. It replaces `progress`
  (`conflicting_progress`).
- **Markers** are dashed lines across all rows with their label on top: today, a deadline.
- **Arrows.** `after` names the `id`s of the items an item follows; an arrow runs from the end of
  each to the start of the item, below the bars, and turns where no bar or diamond of the rows
  between them stands: just after the item it leaves, else beside the bars in the way. Following in a circle is an error
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
- **Nested items.** An item with `children` has no value of its own (`value_with_children`): it is
  as big as its parts together, drawn as a frame with its name above the parts. Items nest up to
  three levels (`too_deep`), labels stay unique across all levels, and the 100-item limit counts
  every level. A nested treemap takes its colors from the item at the top an item belongs to and
  has no `group` (`group_with_children`); the legend names the items at the top when there are
  four or fewer. The table lists every item with its path (`Society › Schools`).
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
- **Flows back.** `"back": true` on a link marks a flow that returns to an earlier column, such
  as recycled water. A Sankey diagram has no cycles (`link_cycle`): the columns and their order
  follow the other links, and a flow back is drawn as a loop of its own thickness under the
  nodes, out of the right of its source and into the left of its target. It must lead to a
  column left of the one it leaves (`link_not_back`). Nodes count it in their height, the total
  does not.
- **Fixed columns, order and colors.** `nodes` lists nodes you want to pin: a `column` (from 0;
  a node cannot stand left of the column its incoming links lead to, `column_too_early`), a
  `color` (1–4, so that the same party has the same color on both sides) and, through their
  position, the order — with `"order": "listed"` the nodes of a column keep the listed order
  (then the order of the links) instead of being sorted to cross few bands. A listed node must be
  named by a link (`unknown_node`).
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
- **Log-rank test.** `"logRank": true` tests whether the groups' curves differ (Mantel-Cox, with
  the chi-squared distribution of groups − 1 degrees of freedom) and writes `Log-rank p = 0.013`
  in the plot and the statistic in the description. With exactly two groups the description adds
  the hazard ratio of the second group against the first with its 95 % interval (the one-step
  estimate from observed and expected events). It needs two or more groups
  (`log_rank_needs_groups`).
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
- **Regression.** `"regression": true` draws the least-squares line through the points, one for
  each group with at least three points (and some spread in x). The description gives the slope,
  the intercept and r².
- **Log axes.** `"xScale": "log"` and `"yScale": "log"` space powers of ten evenly and need values
  above zero, on points and lines alike (`invalid_value`). A regression on a log axis is fitted to
  the logarithms (`ln y = a + b ln x`, a power law), so it is straight on the page.
- **Ranges and named ticks.** `xRange` and `yRange` (`[min, max]`) fix the ends of an axis
  instead of rounding them outward to a tick; everything drawn along it must lie within
  (`out_of_range`, `invalid_range`). `xTicks` (`[{ "at": 1180, "label": "6" }]`) replaces the
  numbered ticks of the x axis by labels at chosen places: a Manhattan plot labels its
  chromosomes this way, and colors them alternately with two groups. A label that would run into
  the one before it is left out.
- **Quadrants.** `"quadrants": { "x": 5, "y": 5, "labels": { "topLeft": "Quick wins", "topRight":
  "Major projects", "bottomLeft": "Fill-ins", "bottomRight": "Thankless tasks" } }` divides the
  plot by a line at each value into four areas, as in an impact and effort matrix, and writes the
  name of each area quietly in its outer corner — or in another corner of the area when points or
  their names are there (`label_overlap` when every corner is taken). Each name is optional; an
  area without one is called by where it lies. Top is the side of the larger y, right the side of
  the larger x. With `xRange` or `yRange` a dividing line must lie inside the range
  (`out_of_range`); without, the axis reaches it, and one that ends up on the end of the axis is
  reported as `quadrant_without_room`. The description names the points of every area, and the
  data table gains a Quadrant column; a point on a dividing line belongs to no area and is said to
  be on it.
- **Names.** A point with a `label` gets its name beside it, in the order of the list; a name that
  would run over one already written is left out (`value_labels_omitted`).
- **Many points.** Up to 300 points have a tooltip each; beyond that the dots are small and plain.
  The data table lists every point up to 100; for more, only the labeled ones, or the 20 highest
  if none is labeled, and its caption says how many of how many.
- **Text alternative.** The description gives the number of points, the span of both axes, the
  points per group, how many lie above each line and, with quadrants, the points of every area.

## Class and entity diagrams

`"type": "classes"` draws a class diagram or an entity-relationship diagram: boxes with a name and
compartments for attributes and methods, joined by relations with the marks of their kind and
their cardinalities. chartlet lays it out, like a flow chart; the specification only says what is
related.

```json
{
  "schemaVersion": 1,
  "type": "classes",
  "title": "Order model",
  "classes": {
    "classes": [
      { "id": "order", "label": "Order", "attributes": ["+ id: Int"], "methods": ["+ total(): Money"] },
      { "id": "line", "label": "OrderLine", "attributes": ["+ quantity: Int"] },
      { "id": "payment", "label": "Payment", "kind": "interface", "methods": ["+ charge(amount: Money)"] },
      { "id": "card", "label": "CardPayment" }
    ],
    "relations": [
      { "from": "order", "to": "line", "kind": "composition", "fromCardinality": "1", "toCardinality": "1..*" },
      { "from": "card", "to": "payment", "kind": "realization" }
    ]
  }
}
```

- **Classes** have a unique `id` and a `label`. `kind` writes a stereotype above the name:
  `interface`, `abstract` or `enum`; `entity` is an entity of an entity-relationship diagram and
  has none. `attributes` and `methods` are lines of text — `+ id: Int`, `id (PK)` — each in a
  compartment of its own; an empty compartment is left out. Up to 40 classes and 30 lines in a
  compartment. The box is as wide as its widest line.
- **Relations** join two classes by id (`unknown_node`): `association` is a plain line,
  `inheritance` a hollow triangle at `to` (the parent), `realization` the same dashed,
  `composition` a filled diamond at `from` (the whole), `aggregation` a hollow one,
  `dependency` a dashed line with an open arrowhead at `to`. A relation may have a `label`,
  and `fromCardinality` and `toCardinality` (such as `1` or `0..*`) stand at its ends. The parent
  of an inheritance is laid out before its child.
- **Navigability.** An association may say which way it can be navigated: `"direction":
  "forward"` draws an open arrowhead at `to` (`from` knows `to`), `"both"` one at either end,
  and `"none"` a plain line. The description adds “navigable to Order” or “navigable both ways”.
  The other kinds keep their fixed ends and refuse a `direction` (`option_not_supported`).
- **Packages** group classes into modules: `"packages": [{ "id": "ordering", "label": "Ordering",
  "in": "shop" }]` draws a frame named `Ordering` inside the frame of `shop`, and a class lies in
  it with `"in": "ordering"`. Packages nest up to four deep, at most 12; every package holds a
  class, directly or in a package inside it, and classes outside every package stay free.
  Relations still join classes, also across package borders; a relation that names a package is
  `unknown_node`. Example `shop-modules`.
- **Orientation of a package.** `"orientation": "landscape"` (or `"portrait"`) on a package lays
  out its classes that way whichever way the diagram runs — a chain of value objects side by
  side in a diagram that runs down. The package is laid out on its own, with the packages inside
  it and the relations between its classes, and the diagram places it as one block. A relation
  from outside meets the block where its class lies inside and runs on straight to the side of
  that class. Where that way would cross another class, a relation or a label of the block in
  every orientation the diagram may take, chartlet lays the package out with the diagram instead
  and reports `package_orientation_ignored`; with `"orientation": "auto"` on the diagram it first
  tries the orientation where every way is open. A package with an orientation of its own holds
  no package that has one too, and takes `portrait` or `landscape` only (`option_not_supported`).
  Example `invoice-model`.
- **Notes** explain a class: `"notes": [{ "on": "card", "text": "Stores only a token" }]`
  draws a pale box with a folded corner beside the class, joined to it by a dotted line without
  marks. The line only says “explains”: it is no relation, so the most connected classes,
  bottlenecks and relations read the same with and without notes. The text (up to 160
  characters) wraps onto as many lines as it needs; up to 12 notes, each on an existing class
  or package (`unknown_node`). A note on a package (`"on": "ordering"`) stands inside its frame,
  at the head of its classes, without a line; a note without `on` explains the whole diagram and
  stands in a band below it. Architecture diagrams take the same `notes` on components,
  boundaries and the whole diagram, flow charts on steps and the whole chart. Examples
  `subscription-model`, `payment-notes`.
- **Layout.** The layered layout and the canvas rules of flow charts apply: `orientation`
  (`auto`, `portrait`, `landscape`) and the warning `canvas_too_small` when the diagram needs a
  bigger canvas.
- **Text alternative.** The description gives every class with the number of its attributes and
  methods and the relations it starts, in words (`Dog inherits from Animal`), after the most
  connected classes and the bottlenecks (see [Graph structure](#graph-structure)); the data table
  has one row per class with its kind, attributes, methods and relations. With packages, the
  description names every package with what it holds (`Ordering (in Shop): Order, OrderLine.`)
  and the table gains a Package column with the path of packages (`Shop › Ordering`). Notes
  follow the classes in the description (`Note on Card: Stores only a token.`, `Note: …` for one
  on the whole diagram) and fill a Notes column of the table; notes on packages and on the whole
  diagram belong to no row and follow the table's caption.

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
- **The main path** names the usual way through the flow as step ids joined by edges, or only its
  ends, `"mainPath": {"from": "draft", "to": "live"}`: chartlet then takes the shortest way along
  the edges (at a tie the first in the order of the edges). It keeps the path in line where the
  lanes leave room and draws it thicker in the accent color; `main_path_gap` reports two
  consecutive steps no edge joins, `main_path_unreachable` two ends no way joins.
- **Shared ranks.** chartlet puts every step one layer after the latest step that leads to it, so
  steps that belong side by side — parallel reviews, the ends of a flow — can land a layer
  apart. `"sameRank": [["legal", "finance", "security"]]` puts each set at one position along the
  flow, in portrait and landscape and in the mobile variant; the order within the layer and the
  lanes stay chartlet's. It is a layout hint and adds no edge. An edge that would have to run
  against it is drawn back like the edge that closes a loop. A set names at least two steps
  (`empty_data`), a step lies in one set at most (`duplicate_member`), and no edge joins two steps
  of one set (`same_rank_edge`), since an edge always crosses from one layer to another.
- **Notes** work as in class diagrams: `"notes": [{ "on": "edit", "text": "At most two rounds" },
  { "text": "Applies to every article" }]` puts the first beside the step, joined by a dotted line
  that is no edge, and the second, without `on`, in a band below the chart. The description ends
  with them (`Note on Edit: …`, `Note: …`); a note on a step fills a Notes column of the table, a
  note on the whole chart follows the table's caption.
- **Values.** A node with a `value` is filled from a scale of six blues, light for the smallest
  value of the diagram and dark for the largest; `valueLabel` names what the values stand for
  (`Budget (m €)`) in the key below the diagram and in the table. Every step fixes its text color,
  so labels keep at least 4.5:1 in the light and the dark theme; the colors are the custom
  properties `--chartlet-heat-0` to `--chartlet-heat-5`. The description says what the colors
  show and gives every value, so the color is never the only sign.
- **Parts.** `parts: [{ "label": "kept", "value": 180 }, …]` draws a stripe inside the bottom of a
  box, split into the shares of its parts; each name keeps one color across the diagram and the
  legend below names them. The description and a Parts column give every part with its value
  and share. At most 6 parts per node and 8 names per diagram (`too_many_parts`), values of at
  least 0 that add up to more than 0 (`invalid_value`); a decision has no room for a stripe
  (`option_not_supported`).
- **Weights.** An edge with a `weight` (a number of at least 0, such as cases per month) is drawn
  as much stronger as it carries more: from 1 px for no weight to 5 px for the heaviest edge of
  the chart, in steps of half a pixel; the arrowheads keep their size and the main path its
  color. The description names the heaviest edge, and every weight stands in the description and
  the table beside where the edge leads. Edges without a weight keep the usual line;
  `invalid_edge_weight` reports a negative weight. Architecture diagrams take the same `weight`
  on their connections.
- **Lanes** are bands across the flow, one per role or system: columns in portrait, rows in
  landscape. Once a chart has `lanes`, every step names its `lane` (`missing_lane`).
- **Groups** frame steps of one lane under a name (`group_spans_lanes` otherwise). A step outside
  a group that ends up inside its frame is reported as `group_overlap`.
- **Shared trunks.** With `"sharedTrunks": true`, edges that leave one step for steps of the next
  layer, or reach one from steps of the layer before — a step that splits into parallel tasks, the
  tasks that all lead to one step — are drawn on one line that branches on a track of its own,
  with a dot at every branch, as in architecture diagrams. Only edges that read the same on the
  shared part share it: no `label`, the same `dash`, `weight` and `role`, off the main path, and
  their other ends in the same lane and group. The branches of a decision keep a line each, even
  without a label, and so does a second edge between the same two steps. Every edge is still drawn
  from end to end with its tooltip, and the description and the table are as before. Example
  `order-fulfilment`.
- **Orientation** works as for sequence diagrams: portrait runs the flow down, landscape right,
  and `auto` picks by the canvas.
- **Port sides.** `fromSide` and `toSide` on an edge say which side of its steps it leaves and
  reaches, such as a way back that leaves a decision sideways; see [Port sides](#port-sides).
- **Spacing.** `"spacing": "compact"` or `"roomy"` tightens or loosens the gaps between the
  layers and the steps; see [Spacing](#spacing).
- **Text alternative.** The description names the steps in reading order — layer by layer — and
  where each one leads, with the main path, the lanes and the groups. Before the steps it names
  the shape of the graph where there is one to tell (see [Graph structure](#graph-structure));
  the data table has one row per step with its kind, lane, group and next steps.

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

### Port sides

An edge leaves its step on the side the flow leads to and reaches the next on the side it comes
from. `fromSide` and `toSide` pin the side instead, named after the flow so that they hold in
portrait, in landscape and in the mobile variant: `start` faces the steps before, `end` the steps
after, and `left` and `right` lie across the flow — left and right in portrait, top and bottom in
landscape.

```json
{ "from": "compliant", "to": "draft", "label": "no", "fromSide": "right", "toSide": "right" }
```

- An edge with `left` or `right` leaves the side of its step level with its middle, runs along
  the flow on a short stub beside it and changes course between the layers like any other edge;
  the layout keeps room beside the step for it. A way back that leaves and reaches the same side
  runs straight past the steps between. Several edges on one side of a step spread along it, each
  on a stub of its own, so they do not cross there.
- `start` and `end` are where an edge meets its step anyway when they agree with its direction:
  `end` on `fromSide` and `start` on `toSide` of an edge that leads on, the other way round for a
  way back that closes a loop. The opposite would turn the edge back over its own step; chartlet
  ignores it and reports `port_side_ignored` at the field.
- An edge with a side keeps a line of its own: it never joins a shared trunk. On the main path it
  is still drawn thicker, but the steps it joins are not lined up, since it cannot run straight.
- An edge from a step to itself is drawn as a loop beside the step and takes no side
  (`option_not_supported`).
- Architecture diagrams take the same `fromSide` and `toSide` on their connections. Example
  `contract-review`.

### Spacing

`spacing` sets how much room a flow chart, a state diagram, a class diagram or an architecture
diagram leaves between its layers and between the steps of a layer:

| `spacing` | Gaps |
| --- | --- |
| `compact` | about 70 %, and kept at that on a canvas that leaves room, instead of spreading the layers over it; never below 16 px between steps and 20 px between layers |
| `normal` | the default, unchanged |
| `roomy` | 150 % |

Only the gaps change. Steps, their texts and where those wrap, the margins, the padding of frames
and lanes and the distance between two parallel edges keep their size, in every orientation and
in the mobile variant, which scales its own smaller gaps the same way.

### Graph structure

Flow charts, architecture diagrams, class diagrams and networks describe the shape of their graph in a few
sentences before the steps, as far as it says more than the steps themselves — a single chain
says nothing here:

- the most connected nodes, when they have at least three connections and no more than three
  share the top;
- where nothing leads in and where nothing leads out (flow charts, architecture and directed
  networks only; start and end steps are not named again, nor are more than three);
- loops: one per tangle of edges that leads back into itself, the shortest way back to its first
  node (`Loop: Review → Build → Review.`), at most three (as above);
- unconnected parts, when there are several;
- bottlenecks: nodes without which the graph falls into at least two pieces of more than one
  node each, when there are one or two.

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
  `security` (a shield, for an identity provider, a firewall or a vault), `external` (a dashed
  box) and, for a deployment, `node` (a machine or a cluster node: a box seen from a corner) and
  `artifact` (something deployed on it, such as an image or a package: a box with a folded
  corner). A deployment is an architecture diagram whose boundaries are the hosts.
- **Boundaries** are frames — a cloud region, a network, a zone, a system. A component lies `in`
  a boundary, and a boundary `in` another one, up to four deep. Every boundary holds at least one
  component (`empty_boundary`), and boundaries cannot lie in each other in a circle
  (`boundary_cycle`). chartlet keeps components that do not belong to a boundary out of its frame.
- **Shared ranks** work as in flow charts: `"sameRank": [["catalogDb", "ordersDb",
  "billingDb"]]` puts the databases of a data tier at one position along the flow. Without it,
  each database follows its own service, and a `Data tier` boundary around them would stretch
  over three layers and the services between them; with it, the frame is one layer deep.
- **Notes** explain a component, a boundary or the whole diagram, as in class diagrams:
  `"notes": [{ "on": "vault", "text": "…" }, { "on": "pci", "text": "…" }, { "text": "…" }]`.
  The first stands beside the component, joined by a dotted line that is no connection; the
  second inside the boundary, at the head of its components, without a line; the third in a band
  below the diagram. A view keeps the notes without `on` and those on what it shows. Example
  `payment-notes`.
- **Connections** carry a `label` for what they do and a `technology` for how, written in
  brackets on a line below it. A `step` from 1 to 99 numbers the connection in a sequence of
  calls; the number is written before the label, every number is used once (`duplicate_step`,
  `invalid_step`), and the description lists the steps in order.
- **Direction.** A connection runs from `from` to `to`, with an arrowhead at `to`. With
  `"direction": "both"` it runs both ways — an arrowhead at either end, such as a socket or a
  synchronisation — and with `"direction": "none"` it has no direction and no arrowhead, such as
  replication between peers or a shared link. Either stays one connection: it is listed once, under
  its `from` component, as “both ways” or “without direction”, and the tooltip joins its ends with
  `↔` or `—`. Once a connection has no single direction, the description no longer names where
  nothing leads in or out, nor loops.
- **Shared trunks.** With `"sharedTrunks": true`, connections that leave one component for
  components of the next layer, or reach one from components of the layer before, are drawn on
  one line that branches on a track of its own, with a dot at every branch — a gateway that
  calls many services, services that all log to one store. Only connections that read the same on
  the shared part share it: no `label`, `technology` or `step`, the usual direction, the same
  `dash`, `weight` and `role`, off the main path, and their other ends in the same boundary, so that they
  cross the same frames. Anything else, such as a second connection between the same two
  components, keeps a line of its own. Every connection is still drawn from end to end with its
  tooltip, and the description and the table list every one as before. Example
  `service-fan-out`.
- **Interfaces.** A component lists the interfaces it `provides`, such as
  `"provides": [{ "id": "payments-api", "label": "Payments API" }]`; each is drawn as a ball on a
  stem ahead of the component's box, with its label beside it (on a narrow canvas, on a chip
  below it). A connection with
  `"toInterface": "payments-api"` says that `from` requires that interface of `to`: it ends in a
  socket around the ball instead of an arrowhead, and every connection that requires the same
  interface meets the same ball. The description and the table say it in words — “Checkout
  (service): connects to Payments (requires Payments API, charges)”, “Payments provides Payments
  API and Refunds API” — and the table gains a Provides column. An interface's id is unique within
  its component; with the component's id it forms the stable id `payments.payments-api`, which the
  hooks carry. A component provides at most four (`too_many_interfaces`); a `person` or a `node`
  provides none, and a connection to an interface runs forward (`option_not_supported`); an id
  that `to` does not provide is `unknown_interface`. An interface follows its component into every
  view and environment, and a connection to an interface never joins a shared trunk. Without
  `provides` and `toInterface`, nothing changes. Example `payment-interfaces`.
- **Roles.** `role` on a component or a connection says what it means in this picture, beyond its
  kind: `critical` (a strong red outline or line, badge `!`), `planned` (dashed and violet, badge
  `+`) or `deprecated` (dotted and gray, badge `−`). Flow charts take the same roles on steps and
  edges, state diagrams on states and transitions, trees on nodes, networks on nodes and edges,
  and class diagrams on classes and relations. Shape and fill stay those of the kind, so a role
  adds meaning rather than replacing it; `external` stays a kind. The colors are the theme's role
  colors in light, dark, blueprint and print, and the line or badge carries the role without
  color. A legend below the diagram names only the roles used, the description lists what has
  each role, and the table gains a Role column. A planned or deprecated connection, edge or
  transition takes no `dash` (`role_conflict`). Example `billing-migration` shows a migration: the
  old path deprecated, the new one planned.
  - A network's dot is too small for a badge: a ring around it, solid, dashed or dotted in the
    role's color, says the role, and the dot keeps the color of its group.
  - A composite state is a frame and takes no role (`option_not_supported`); give the states
    inside it one.
  - A relation keeps the marks of its kind: a hollow triangle or diamond stays hollow, and only
    its outline and the line take the role's color. A realization or a dependency is dashed by
    its kind, so it takes `critical` only; `planned` or `deprecated` would draw their own pattern
    over it (`role_conflict`). Example `payment-redesign`: a planned payment class, a deprecated
    invoice and the critical dependency of the order on payment.
- **Blueprint look.** `"theme": "blueprint"` draws a sequence, flow, state, architecture, tree or
  classes diagram in pale lines on a deep blue ground. Charts with axes keep `light` and `dark`
  (`option_not_supported`).
- **Text alternative.** The description says what each boundary holds, the shape of the graph
  (see [Graph structure](#graph-structure)) and where each component connects to; the data table has one row per component with its kind, its boundaries
  (`Cloud › Private network`) and its connections.

Limits: 1–40 components, up to 80 connections and 12 boundaries; identifiers are unique among
components and boundaries together.

### Views on one model

One architecture model can be shown as several diagrams: an overview, the way a checkout takes, a
detail of one zone. The model keeps its components, connections and boundaries once; `views`
lists what each diagram shows. A specification with views always renders one of them — the one
asked for by its `key` (`--view checkout`, `view: 'checkout'`), else `defaultView`, else the
first — and never the whole model, so a view never passes for the complete system.

```json
"connections": [
  { "id": "call", "from": "web", "to": "api", "label": "calls" },
  { "id": "orders", "from": "api", "to": "db", "label": "reads, writes" }
],
"views": [
  { "key": "overview", "title": "Web shop: overview", "components": ["web", "api", "db"] },
  {
    "key": "checkout",
    "title": "Web shop: checkout",
    "steps": [
      { "connection": "call", "label": "submits order" },
      { "connection": "orders", "label": "reserves stock" },
      { "connection": "orders", "label": "saves order" }
    ]
  }
]
```

- **Selection.** `components` lists the components a view shows, by id; they keep the order of
  the model. The connections between them come along, or only those listed in `connections`,
  by the optional `id` a connection carries. A view draws no connection the model does not have
  and leaves out every connection to a component it hides; it does not merge components into
  coarser ones.
- **Tags.** Components and connections may carry `tags`, such as `"tags": ["public"]` — up to
  eight, written like ids, never drawn. A view selects by them with `include` and `exclude`, in
  addition to its lists, by one rule:
  1. A component is shown when the view lists it in `components`, or when it carries a tag of
     `include` and none of `exclude`. A view with `exclude` but neither `components` nor
     `include` starts from every component.
  2. Without `connections`, every connection between shown components comes along, except one
     that carries a tag of `exclude`. With `connections`, the view shows those, and every
     connection between shown components that carries a tag of `include` and none of `exclude`.

  What a view lists by id is always shown; tags never take it away. The model's order stays.
  `unknown_tag` for a tag no component or connection carries, `duplicate_tag` for one named
  twice in `include` and `exclude` together, `invalid_tag`, `too_many_tags`, and `empty_view`
  when the tags select no component. A dynamic or deployment view takes no tags
  (`option_not_supported`).
- **Boundaries** come along when they hold a shown component, directly or further in; the others
  are left out.
- **Highlighting.** A view's `mainPath` is drawn stronger, along its own connections. Without
  one, the view keeps the model's main path when it shows all of it.
- **Dynamic views.** `steps` lists connections in the order they occur, each with an optional
  text in place of the connection's label; the same connection may occur again. The view
  numbers them from 1, shows only them and the components at their ends, and draws a repeated
  connection once per occurrence — a static numbered diagram, without timing or parallel steps.
- **Text.** A view has its own `title` and, optionally, `description`; the model's description
  is never used. Without one the description is generated from what the view shows and says
  how much of the model that is (“It is a view that shows 8 of the model's 11 components and 8
  of its 11 connections.”). The data table lists the shown components only.
- **Size.** A view may set its own `width`, `height` and `mobile`, in place of the model's — a
  narrow detail beside a wide overview. They are checked like the model's, with diagnostics at
  the view; a view without them keeps the model's size. `spacing` always comes from the model.
- **Variants and provenance.** Every variant — HTML, mobile, print, social, PNG — renders the
  chosen view. The manifest names it in `view`; `specHash` is the hash of the whole model. Each
  view gets its own derived ID prefix, so several views can share a page.
- **Diagnostics** point into the model: `unknown_view` for a key no view has (at `/render/view`,
  or `/architecture/defaultView`), `empty_view` for a view without components or steps,
  `unknown_node` and `unknown_connection` for ids the model does not have,
  `connection_outside_view` for a listed connection to a hidden component, and the usual
  diagnostics of the resolved diagram at the place in the model or the view they come from.

Up to 20 views and 99 steps per view. Example `shop-views` (overview, and checkout as
`shop-views.checkout.svg`).

#### All views on one page

`--all-views` (`allViews: true` in npm and MCP, `RenderOptions::all_views`) renders every view
of a model as one HTML page, without a script: the model's title as an `h2`, a navigation
of links to the views, and each view as a section headed by its title as an `h3`, with the figure,
description and data table of its own export — the default view first, then the others in
the order of `views`. The links are ordinary in-page links, reachable with the keyboard; in
print every view after the first starts a new page.

- **IDs.** The page takes the ID prefix (`--id-prefix`, or one derived from the model); the
  views take that prefix followed by `-v1`, `-v2`, … in the order of the page, their sections
  `<prefix>-v1-view`, …, which the navigation links to. Every ID on the page is unique.
- **Only HTML.** The SVG format, the mobile, print and social variants, `--view`, a
  specification without views and PNG are refused (`option_not_supported` at
  `/render/allViews`); each view as SVG stays a file of its own. A single view's export is
  unchanged by this option.
- **Size and perspective.** Each section has the `width`, `height` and `mobile` of its view,
  or the model's. `--perspective` shows the perspective asked for in every section.
- **Provenance.** The manifest lists the views in `views`, in the order of the page, and the
  perspective asked for in `perspective`.

Example `shop-views.all.html`.

### Perspectives

The same diagram can answer different questions: which team runs each component, where
technical debt is high. A model keeps the answers as stated metadata, apart from its components
and from how it is drawn: `perspectives` names the questions, each with its categories and the
components in each. Nothing is drawn until a perspective is selected — by `perspective` on the
model, or on a view, which takes the place of the model's, or at render time
(`--perspective debt`, `perspective: 'debt'`), which takes the place of both in every view.

```json
"perspectives": [
  { "key": "owner", "title": "Team", "description": "The team named in the service catalogue runs each component.",
    "categories": [
      { "label": "Bookings", "components": ["api"] },
      { "label": "Platform", "components": ["db"] }
    ] },
  { "key": "debt", "title": "Technical debt",
    "categories": [{ "label": "low", "components": ["db"] }, { "label": "high", "components": ["api"] }] }
],
"views": [
  { "key": "owners", "title": "Who runs what", "components": ["api", "db"], "perspective": "owner" },
  { "key": "debt", "title": "Technical debt", "components": ["api", "db"], "perspective": "debt" }
]
```

- **Drawing.** Every box carries a stripe in the color of its category and names the category
  after its sublabel, so the meaning does not rest on color alone. The colors follow the order of
  `categories`, the same in every view. Shapes, roles and the layout stay as they are.
- **Not stated.** A component that no category of the shown perspective lists is drawn with an
  outlined stripe and named “not stated” — never as the lowest category. A component lies in at
  most one category of a perspective. A view keeps in each category only the components it shows.
- **Connections.** A category may also list `connections` by their `id`. Such a line is drawn in
  the category's color, arrowheads too, and carries the category's name after its label (`reads ·
  Data`, or the name alone) so the meaning does not rest on color; a role keeps its own color.
  A connection no category lists is drawn as without the perspective, never as not stated. A
  view keeps the categories of the connections it shows; every occurrence of a step and every
  instance connection that carries a listed model connection lies in its category. The description names them after the
  components of their category (`Data: Posts, API → Posts`), the table after the connection.
- **Legend.** Below the diagram, after the perspective's title: the categories used by components
  or connections, in their order, and “not stated” when it occurs.
- **Text.** The description ends with the perspective: each category with the components that
  have it, then the perspective's own description. The data table has a column named after the
  perspective. The manifest names it in `perspective`.
- **Nothing derived.** Categories are only what the specification states; chartlet does not
  rate components or infer categories from the topology or the names. A model with perspectives
  but none selected renders exactly as without them.

Up to 8 perspectives with 1–8 categories each. Example `service-perspectives` (who runs what,
and technical debt as `service-perspectives.debt.svg`).

### Environments and instances

A model describes each component once; a deployment view shows where it runs in one
environment — once on a developer machine, twice behind a load balancer in production. A view
with an `environment` is such a view: it lists `hosts`, nested like boundaries, the `instances`
running on them, each `of` one component of the model, and the `instanceConnections` between
them.

```json
"views": [
  { "key": "logical", "title": "Orders API: components", "components": ["lb", "api", "db"] },
  {
    "key": "production",
    "title": "Orders API in production",
    "environment": "Production",
    "hosts": [
      { "id": "node-a", "label": "Node A" },
      { "id": "node-b", "label": "Node B" }
    ],
    "instances": [
      { "id": "lb-prod", "of": "lb" },
      { "id": "api-1", "of": "api", "in": "node-a" },
      { "id": "api-2", "of": "api", "in": "node-b" }
    ],
    "instanceConnections": [
      { "id": "route", "from": "lb-prod", "to": "api-1" },
      { "id": "route", "from": "lb-prod", "to": "api-2" }
    ]
  }
]
```

- **One component, many instances.** An instance is drawn as its component — kind, symbol,
  sublabel, role — labelled with the component's label and the instance's id, `Orders API
  (api-1)`, so replicas stay apart in the drawing, the description and the table. A label changed
  in the model reaches every environment.
- **Only listed connections.** Nothing joins instances but `instanceConnections`; chartlet never
  connects replicas by itself unless the view asks for it with `deriveConnections`. An instance connection with the `id` of a model connection must
  join instances of that connection's ends (`instance_mismatch`) and takes the label, technology,
  dash, role, `fromSide` and `toSide` it leaves out from it.
- **Derived connections.** `"deriveConnections": true` adds instance connections from the model,
  by one stated rule, after the listed ones:
  1. Every model connection whose `id` no instance connection names is derived; one the view
     names is left to its instance connections entirely, so the listed ones take precedence.
  2. Each instance of its `from` component, in the order of `instances`, joins the nearest
     instances of its `to` component: those whose hosts share the most enclosing hosts with its
     own, counted from the outermost (an instance without a host shares none). At a tie it joins
     all of them, and never itself.
  3. The derived connection is the model connection — label, technology, dash, role, weight,
     direction, `fromSide` and `toSide` — with the two instances as its ends.

  So two API replicas on their own nodes each join the database on their node, and a load
  balancer outside the nodes joins both replicas. Derived connections count toward the 80
  connections of the view (`too_many_edges` at `…/deriveConnections`), and a diagnostic about
  one points to its model connection. Only a deployment view takes `deriveConnections`
  (`option_not_supported`).
- **Hosts** replace the model's boundaries in the view; the model's main path, shared ranks and
  notes stay out of it, except notes without `on`, which every view keeps.
- **Text.** The description names the environment and how many instances run how many
  components; the table's first column is the instance.
- **Limits and diagnostics.** The resolved diagram is checked like any other — at most 40
  instances, 80 instance connections and 12 hosts nested four deep — and every diagnostic points
  into the view: `unknown_node` for an `of`, `from` or `to` that names nothing, `unknown_boundary`
  for a host, `duplicate_id` for an id used twice among instances and hosts,
  `unknown_connection` for an `id` the model does not have, `text_too_long` at an instance's `id`
  when its label `Label (id)` exceeds 60 characters. A deployment view takes no
  `components`, `connections`, `steps` or `mainPath`, and another view no `hosts`, `instances`
  or `instanceConnections` (`option_not_supported`).

`chartlet inspect` counts a component as shown when an environment runs it, and does not report
a deployment view as adding nothing. Example `api-environments` (components, and production as
`api-environments.production.svg`).

### Context, containers and deployment

Three starters (MCP: `chartlet_diagram_starter` with `type: "architecture"` and `template`)
describe one system, a course booking service, at three levels of abstraction.
Each answers one question and stays at its level; all use the ordinary `architecture` fields,
render without `symbol` and carry their meaning in labels, sublabels and the data table.

| Template | Question | Shows | Example |
| --- | --- | --- | --- |
| `context` | Who uses the system, and which other systems does it depend on? | The system in scope as one component, people (`person`) and external systems (`external`). No boundary, nothing inside the system. | `booking-context` |
| `containers` | Which applications and data stores make up the system, and how do they talk to each other? | One boundary for the system around its separately running or deployable parts — web app, API, worker, queue, database, file store — with people and external systems outside it. | `booking-containers` |
| `deployment` | Where does each part run, and how many instances of it run there? | A logical model of the deployable parts — the images (`artifact`), the managed queue, database and object storage — and a [deployment view](#environments-and-instances) of production: region, cluster, two nodes and a worker pool as hosts, two instances of the API behind the ingress. The production view renders by default, the view `logical` shows the model. | `booking-deployment` |

The terms map onto chartlet like this, and must not be read as one another:

- **Software system** — the thing the team builds and owns. At context level it is one
  component; at container level it is a boundary.
- **Container** — a separately running or deployable part of a system: an application, a
  database, a queue, a file store. It is not a container image or a container process; whether
  it is shipped as an image is a deployment fact.
- **Component** — a part inside one container, such as a module of the API. chartlet's
  `components` array holds whatever the diagram shows at its level, so a component in the
  specification is a component in this sense only in a diagram of one container's inside; no
  starter covers that level.
- **Deployment node** — where something runs: a region, a cluster, a node pool. The starter
  lists them as `hosts` of its production view and places `instances` of the model's components
  on them, one per replica; `kind: "node"` draws a single machine as a component instead.

Where the starters stop short of a complete architecture model:

- Each starter is a separate specification. An element that appears at two levels — the course
  booking system as a box and as a boundary — is written twice, and nothing checks that the two
  agree. One model with several `views` can show different selections of the same elements, but a
  view does not merge containers into a system or open a container into components.
- Technology and responsibility are free text: `sublabel` on a component, `technology` on a
  connection. There are no separate fields for them, and no element types for system, container
  or component beyond the `kind` shapes.
- The deployment starter describes one environment. Another one, such as staging, is another
  view with an `environment` on the same model; see
  [Environments and instances](#environments-and-instances).

## Trees

A tree (`"type": "tree"`) draws a root and the nodes below it: an organization chart, an
ownership structure, a family, a hierarchy of norms or components, or, laid out radially, a mind
map. Every node but the root names its `parent`; the children of a
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
- **Collapsed nodes.** `"collapsed": true` draws a node without what hangs below it (and below
  its partner): the node says how many nodes it leaves out, `+2` after its sublabel, and the
  tree is laid out without them, so a big tree can be shown at a glance and unfolded by turning
  the flag off. The output is static, so this is a choice made in the specification. The
  description names the collapsed nodes and the data table keeps every node, with a Collapsed
  column giving the count. Collapsing a node with nothing below it warns
  (`nothing_to_collapse`).
- **Couples.** `"partner": "id"` joins a node to another as a couple: the two stand side by side
  (one above the other in landscape) with a short line between them, and the children of either
  hang from the middle of that line, as in a family tree. A partner has no `parent` of its own
  (`invalid_partner`); the node it names is the head of the couple.
- **Links** are the lines from a parent to its children. A node's `link` is written on the line
  to its parent, for a share, a weight or a relation; the root has none.
- **Orientation.** `portrait` runs down from the root, `landscape` to the right; `auto` takes the
  one that grows the canvas less, as in the other diagrams.
- **Mind maps.** `"layout": "radial"` puts the root in the middle and every level on a ring
  around it: the branches run clockwise from twelve o'clock, and every subtree keeps a sector of
  its own, as wide as its share of the leaves, so that its spokes never reach into a neighbour's.
  The rings are ellipses in the proportions that grow the canvas least; a ring lies so far out
  that its boxes keep clear of every box inside it and of one another. A radial tree has no
  direction, no room for couples and no stretch of line for a link's text, so `orientation`,
  `partner` and `link` are refused (`option_not_supported`); `kind` and `collapsed` work as in the
  layered tree. It needs room in both directions — on a phone a layered tree reads better. The
  description opens with "Mind map (radial tree)" and names the root as the centre.
- **Values and parts** work as in flow charts (see [Flow charts](#flow-charts)): `value` with
  `valueLabel` colors the boxes, `parts` draws a stripe of shares in them.
- **Text alternative.** The description names the root and, for every node with children, what
  hangs below it; the data table has one row per node in reading order, with its level, its
  parent, its link and its children.

Limits: 1–150 nodes. A node has one parent; a node with two parents (a shared holding) is not a
tree — use an architecture or a flow diagram. The table of a tree with couples gains a Partner
column.

### Dendrograms

With `distances` instead of `nodes`, chartlet builds the tree itself: it joins the two closest
items, then again and again the two closest clusters, until one is left (agglomerative
clustering), and draws a dendrogram — the items in rows on the left in the order the tree gives
them, every join as a bracket at the distance where it happens, on a distance axis below.

```json
"tree": {
  "distances": {
    "labels": ["A", "B", "C"],
    "matrix": [[0, 2, 6], [2, 0, 5], [6, 5, 0]]
  },
  "linkage": "average"
}
```

- **The matrix** is square in the order of `labels`, symmetric, 0 on its diagonal and nowhere below
  0 (`invalid_matrix` with the path of the first value that is not). 2–40 items with unique labels
  (`duplicate_id`).
- **Linkage** says how far apart two clusters are: `average` (default, the mean distance between
  their items), `complete` (the largest) or `single` (the smallest). At a tie the pair whose items
  come first in `labels` joins first, so the same matrix always gives the same tree.
- A dendrogram takes no `nodes`, `layout`, `orientation` or `valueLabel`
  (`option_not_supported`); `linkage` belongs to distances only.
- **Text alternative.** The description names the linkage, the closest pair, the two groups of
  the last join and the items in the order of the tree; the table has one row per join, with what
  it joins (`{A, B} + C`), the distance and how many items the new cluster holds.

## Networks

`"type": "network"` draws nodes as dots joined by edges when the graph has no direction of flow to
lay it out by — who works with whom, which services call each other, links between topics — and
loops and dense cross-links would tangle a flow chart. There is deliberately no force layout: the
same input always gives the same picture.

```json
{
  "schemaVersion": 1,
  "type": "network",
  "title": "Who works with whom",
  "network": {
    "nodes": [
      { "id": "ana", "label": "Ana", "group": "Design" },
      { "id": "ben", "label": "Ben", "group": "Code" },
      { "id": "cem", "label": "Cem", "group": "Design" }
    ],
    "edges": [
      { "from": "ana", "to": "ben", "weight": 4 },
      { "from": "ben", "to": "cem", "weight": 1 }
    ]
  }
}
```

- **On a circle** (`"layout": "circle"`, the default) the nodes stand clockwise from twelve
  o'clock. With `"order": "crossings"` (the default) groups stand together in the order they first
  appear, and neighbours within a group swap places as long as that lets fewer edges cross;
  `"order": "listed"` keeps the order of `nodes`. Edges bend a little towards the middle, so that
  two of them between neighbours stay apart.
- **In rings** (`"layout": "rings"` with `center`) the center stands in the middle and every other
  node on the ring of its distance from it in edges, read in either direction: an ego network, the
  dependencies of one service. Ring 1 follows the order of `nodes`, every further ring the angle
  of each node's neighbours on the ring before. Nodes no way reaches stand on one more ring
  outside.
- **Groups.** A node's `group` gives it one of eight colors and an entry in the legend below; the
  group also stands in the description and the table, so the color is never the only sign.
- **Edges** take `dash` and `weight` as in flow charts: the heaviest edge is drawn 5 px wide, the
  others in proportion. `"directed": true` draws an arrowhead at the `to` end; the description
  then says where each node leads instead of whom it is connected with. An edge joins two
  different nodes (`self_loop`).
- **Labels** stand beside their dot on the outer side; where that would cover another dot or
  label, above or below it, or on the inner side.
- **Text alternative.** The description gives the number of nodes and connections, the groups,
  the rings with their nodes, the shape of the graph (see [Graph structure](#graph-structure)),
  the heaviest edge, and every node in reading order with its neighbours and weights. The data
  table has one row per node with its group, its ring (`Center`, the distance, or
  `not connected`) and its neighbours.

Limits: 1–40 nodes, up to 120 edges and 8 groups. A network that does not fit its canvas is
drawn larger and reported as `canvas_too_small`. With `layout: "rings"` a `center` is required
(`missing_center`) and `order` is refused; on a circle `center` is refused
(`option_not_supported`).

## Time series

`"type": "time"` drops categories altogether: an observation carries a timestamp, the ticks land
on calendar boundaries, and the layer list lives in `panes`. Bars and grouped series do not apply
to it, and a `time` chart refuses them by name instead of ignoring them. Its `zoomSteps` name a
window of time instead of a range of categories.
The [time series guide](../time-series/) covers timestamps, timezone, per-layer colors, the dark
theme, and the limits for dense data.
