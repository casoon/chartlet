---
title: Specification
description: Every field of a chart specification, and the limits the renderer enforces.
order: 1
---

[`schema/chartlet.schema.json`](https://github.com/casoon/chartlet/blob/main/schema/chartlet.schema.json)
is the complete contract and can be used for editor validation.

| Field | Required | Description |
| --- | --- | --- |
| `schemaVersion` | yes | Always `1`. |
| `type` | yes | `bar`, `line`, `time`, `topicmap`, `atlas`, `stripes`, `calendar`, `rangebar`, `boxplot`, `timeline`, `parliament`, `treemap`, `sankey`, `survival`, `scatter`, `waffle`, `waterfall`, `multiples`, `sequence`, `flow`, `state`, `architecture`, or `tree`. |
| `title` | yes | Visible title; also the accessible name of the chart (`<title>`). The SVG profile draws it in the chart: a title wider than the chart wraps onto a second line at a space, and the chart below it moves down; only what does not fit on two lines is shortened (`text_truncated`). The HTML profile shows it as the `<figcaption>` and draws no title in its SVGs, so it never shortens it. |
| `data` | one of | Single series: `[{ "label": "…", "value": 1, "group": "…", "lower": 0.5, "upper": 1.5 }]`. `group` (bar charts) gives every bar of a group one palette color and a legend entry; either every point names a group or none does (`missing_group`), at most 4 groups (`too_many_series`). `lower` and `upper` (bar charts, no groups) draw an error bar through the end of the bar: on every point or on none (`missing_bounds`), the value between them (`invalid_bounds`). |
| `categories` + `series` | one of | Several series: unique category labels, and `[{ "name": "…", "values": […] }]`. On `multiples`, `categories` with `values` in each pane makes small multiples of bars. |
| `orientation` | no | `vertical` (default) or `horizontal`; bar charts only. |
| `theme` | no | `light` (default) or `dark`; both palettes are CSS custom properties on the root. |
| `timeAxis.timezone` | no | Fixed UTC offset such as `"+02:00"`, or `UTC` (default); `time` charts only. |
| `timeAxis.title` | no | Title of the time axis. |
| `timeAxis.gaps` | no | `show` (default) keeps the distances in time; `collapse` places every observed timestamp at the same distance from the next, so weekends and holidays take no space. Reference lines, point markers and zones then take the slot of an observation and must lie within the observed range (`time_out_of_range`). |
| `timeAxis.precision` | no | `year`, `month`, `day` or `minute`: how tooltips, the data table and the description name an observation. Absent, the coarsest form that tells the observations apart is chosen: a year when all fall on January 1, a month when all fall on the first of a month. Set it for annual values placed mid-year (`1950-07-01` → `1950`). Each data layer is named by its own observations (or its own `precision`), and a table row as finely as the finest layer with a value there. |
| `timeAxis.step` | no | The distance between time ticks instead of one chosen from the width: whole years (1–1000) on a calendar axis, units on a numeric axis; ticks sit on its multiples (`50` → 1850, 1900, 1950, 2000). For a narrow panel whose claim needs a tick the automatic choice leaves out. A label that would run into the one before it is left out; its gridline stays. |
| `timeAxis.min`, `timeAxis.max` | no | Positions the time axis reaches at least, such as `"1990"` before data from 1991, so that a round tick is drawn. They extend the axis and never cut an observation off; they are no observations (no table row, not in the description). Not with `"gaps": "collapse"`. |
| `panes[].layers[].stepEnd` | no | `line` and `area` with `"curve": "step"`: a time after the last observation where its step ends — annual means drawn as steps end at the end of their last year. It is no observation: no marker, tooltip, table row or description. Not with collapsed gaps or in a stacked pane. |
| `panes[].layers[].markers` | no | `line` and `area` only: `false` draws no markers, for a fitted or reference line whose observations are not data; tooltips stay on invisible targets unless the chart sets `"tooltips": "markers"`. |
| `panes[].layers[].precision` | no | Data layers on a calendar axis: the same values as `timeAxis.precision`, for one layer — yearly means beside monthly values. On a numeric axis each layer writes its positions with the decimals its own positions need. |
| `timeAxis.kind` | no | `calendar` (default) reads every `time` as a timestamp. `number` reads it as a plain number — a distance along a profile, a depth, an age in millions of years — with ticks at round numbers, values written in the chart's `locale`, and the axis `title` as the head of the table's first column. No `timezone`, `gaps: "collapse"` or `zoomSteps` with it. |
| `timeAxis.reverse` | no | `true` runs the axis from right to left, the largest position at the left: ages before present read from the oldest on the left to today on the right. |
| `panes` | no | `time`: 1–4 panes stacked on one shared time axis, each with its own `valueAxis` and `layers`. `multiples`: 2–12 panes, each with a unique `title`, sharing the top-level `valueAxis`; with `categories`, each pane has its own `values` (one per category, `null` for none) and its own `valueAxis` instead of `layers` (1–20 categories). Up to six data layers per pane — on a `time` chart at most four of the whole chart without their own `color` — and six annotation layers (zones, reference lines and point markers together). |
| `panes[].note` | no | `multiples` only: a short finding under the panel title, such as a verdict, up to 160 characters on up to two lines. Panel titles also take two lines before they are shortened; every panel keeps the same head height, so the plots stay aligned. The notes are part of the description. |
| `panes[].noteEmphasis` | no | `true` draws the panel's note in the text color and semibold instead of muted, to set one finding apart from the others by weight, not by color alone. |
| `panes[].title` | multiples | Heading of a small-multiples panel; not allowed on a `time` chart, whose panes are named by their `valueAxis.title`. |
| `panes[].heightRatio` | no | `time` only: the pane's share of the plot height against the other panes, 1–10, default 1. |
| `panes[].stack` | no | `time` only: `"normal"` stacks the pane's `area` layers in layer order, each on top of the ones before it, so that the top edge shows their total; line and point layers in the pane stay as they are. The areas need the same times and `curve`, a value of zero or more at every time (`unaligned_stack`, `negative_in_stack`) and no `lower`/`upper`. Markers and the end labels sit on top of the stack; tooltips and the data table give each layer's own value. |
| `panes[].valueAxis` | no | `time` only: `title`, `format`, `decimals`, `min`, `max`, `step`, `exact`, `thousandsSeparator`, `scale`, `reverse` and `unit` of the pane's own value axis. |
| `columns` | no | `multiples` only: grid columns, 1–6; default up to three. |
| `sparkline` | no | `time` only: `true` draws the lines and areas of one pane across the whole canvas, without axes, title, legend or value labels, with a dot at the end of each line; `width` 60–600 and `height` 16–200. The title and description remain the accessible name and description, and the HTML profile keeps its caption and table. Only line and area layers; no zoom steps or mobile variant. |
| `tooltips` | no | `time` only: `"observations"` (default) gives every observation a tooltip while observations stand at least 4 pixels apart, on an invisible target where no marker is drawn; `"markers"` gives tooltips only to drawn markers, so a line too dense for markers has none and its SVG stays small. |
| `legend` | no | `time` only: `top` (default) names the series in a legend above the plot; `end` writes each name at the last observation of its line, right of the plot, which makes room for the widest name (at most a third of the chart). Names that would overlap move apart. Not with candles, whose legend explains hollow and filled bodies. |
| `independentAxes` | no | `multiples` only: `true` gives every panel its own value axis, scaled to its own values, for panels in different units or sizes. Their heights can then no longer be compared across panels; the description says so. |
| `panes[].layers[].mark` | no | `line`, `area`, `point`, `ohlc` (candlesticks, `time` only), `band` (a zone), or `annotation`. An `area` is a line whose region down to zero is filled in its color; the value axis then always includes zero. A `point` layer draws a dot for every observation and no line, as in a scatter plot; it takes no `dash`, `stroke`, `curve`, `modeled` or `lower`/`upper`. The area between two lines is an uncertainty band (`lower`/`upper`), not a separate mark. |
| `panes[].layers[].name` | one of | Legend and table label; required once a pane — or a `time` chart across its panes — has several data layers, and unique within the pane; on a `time` chart unique across all panes, since they share one legend. |
| `panes[].layers[].data` | ohlc | `[{ "time": "2026-03-02", "open": 10, "high": 12, "low": 9, "close": 11 }]`, 2–2000 candles with increasing `time`; `low` at or below `open` and `close`, `high` at or above them. A candlestick layer takes no `color`, `modeled`, `stroke` or `dash`. |
| `panes[].layers[].points` | line, area | `[{ "time": 1772323200, "value": 1 }]`; ISO 8601 dates and bare years (`"1850"`) are accepted too. `"value": null` marks a missing observation: the line, its area and its band break there, and table and description say “Missing”. Add `lower` and `upper` to every point with a value for an uncertainty band; a point without a value carries neither. |
| `panes[].layers[].modeled` | no | Line and area layers: the line is dashed (unless `dash` says otherwise), its band hatched, and legend, description and table say “modeled”. |
| `panes[].layers[].stroke` | no | Line and area layers: `regular` (default, 3 px), `thin` (1 px), `medium` (2 px, for small panels) or `bold` (4.5 px); the legend sample shows the same weight, so two lines differ in more than color. |
| `panes[].layers[].dash` | no | Line and area layers: `solid`, `dashed`, or `dotted`. Defaults to `dashed` for a modeled layer and `solid` otherwise; the legend sample shows the same pattern. |
| `panes[].layers[].value` / `time` | annotation | A horizontal reference line at `value`, a vertical one at `time`, or a point marker at both. Both widen the axes. |
| `panes[].layers[].shape` | no | Point markers only: `circle` (default), `square`, `diamond`, `triangle-up`, or `triangle-down`. A reference line refuses it. |
| `panes[].layers[].bottom` / `top` | band | Value edges of a zone; `bottom` must lie below `top`. A missing edge is the edge of the plot. Given edges widen the value axis. |
| `panes[].layers[].from` / `to` | band | Time edges of a zone in the forms a point's `time` takes; `from` must lie before `to`. A missing edge is the edge of the plot. At least `bottom` and `top`, or `from` and `to`, are required. |
| `panes[].layers[].label` | annotation, band | Required text of a reference line, point marker or zone. |
| `stripes` | stripes | `{ "firstYear": 1850, "values": [..], "reference": 0, "min": .., "max": .., "yearLabels": true, "stretch": false }`; `null` leaves a year empty. `stretch: true` draws the stripes alone across the whole canvas, without margins, title or year labels, and the SVG stretches to whatever box the page gives it (`preserveAspectRatio="none"`): give it a height in CSS, such as a 6-pixel band. Title and description remain its accessible name and description. |
| `calendar` | calendar | `{ "year": 2024, "layout": "months" \| "weeks", "days": [{ "date": "2024-03-01", "value": 1 }], "reference", "min", "max" }`. |
| `ranges` | rangebar | `[{ "label": "…", "low": 0, "high": 1, "mid": 0.5, "modeled": false, "group": "…", "weight": 12, "summary": false }]`; `orientation` applies. `group` draws the spans of each group in its own palette color, with a legend entry per group; either every range names a group or none does (`missing_group`), and at most 4 groups (`too_many_series`). `weight` (above zero, with `mid`) turns the central mark into a square whose area follows it; `summary: true` (with `mid`) draws the span as a diamond, widest at `mid`: together with `references` a forest plot (`invalid_weight`, `summary_without_mid`). See [Forest plots](../guides/chart-types.md#forest-plots). |
| `boxes` | boxplot | `[{ "label": "…", "values": [3, 4, 5, 6, 9] }` or `{ "label": "…", "min": 1, "q1": 2, "median": 3, "q3": 4, "max": 6, "outliers": [9] }]`; `orientation` applies. A box is computed from 5–1000 `values` (quartiles by linear interpolation, whiskers to the last value within 1.5 × the box, every other value a point) or drawn from the five numbers; never both (`conflicting_data_shape`), never part of them (`incomplete_box`), in order (`invalid_box`). 1–100 boxes with unique labels. `boxDisplay` (`box`, `violin`, `strip`) draws the observations as boxes (the default), as violins or as strips; the last two need `values` on every box (`values_required`). See [Box plots](../guides/chart-types.md#box-plots). |
| `timeline` | timeline | `{ "items": [{ "id": "a", "label": "…", "start": "2027-01-04", "end": "2027-02-26", "group": "…", "after": ["b"] }, { "label": "…", "at": "2027-06-15" }], "markers": [{ "label": "Today", "at": "2027-02-10" }] }`. An item is a phase (`start` and `end`) or a milestone (`at`), never a mix (`invalid_item`); days are ISO 8601 dates or Unix seconds; `after` names the ids of items it follows (`unknown_node`, `circular_dependency`; a warning `follows_overlap` when it starts before them); groups on all items or none, at most 4; 1–60 items, up to 6 markers. See [Timelines](../guides/chart-types.md#timelines). |
| `parliament` | parliament | `{ "parties": [{ "label": "Red", "seats": 40 }, { "label": "Blue", "seats": 35 }], "majority": true, "coalition": ["Red"] }`. Up to 8 parties with unique labels and at least one seat, at most 800 seats; the dots are in blocks by party from left to right in the order of the list. `majority` marks the seat that makes a majority; `coalition` (labels of parties, `unknown_node`) rings their seats and sums them. See [Parliament charts](../guides/chart-types.md#parliament-charts). |
| `treemap` | treemap | `{ "items": [{ "label": "Schools", "value": 310, "group": "Society" }] }`. Up to 100 items with unique labels and values above zero (`invalid_value`); `group` goes on every item or on none (`missing_group`), up to 4 groups, and colors the rectangles with a legend. The areas follow the values by the squarified method. See [Treemaps](../guides/chart-types.md#treemaps). |
| `sankey` | sankey | `{ "links": [{ "from": "Gas", "to": "Power", "value": 120 }] }`. Up to 100 links and 40 nodes; the nodes are the labels the links name, in the order they first appear. Values above zero (`invalid_value`), no link from a node to itself (`self_link`), one link for each pair (`duplicate_link`), no cycle (`link_cycle`). Columns follow the longest path to a node; a band takes the color of the first node it descends from. See [Sankey diagrams](../guides/chart-types.md#sankey-diagrams). |
| `survival` | survival | `{ "groups": [{ "label": "Standard", "observations": [{ "time": 4.2 }, { "time": 9, "event": false }] }], "confidence": true, "atRisk": true, "timeTitle": "Months" }`. 1–4 groups with unique labels and 2–2000 observations each; a time is zero or more (`invalid_value`), `event: false` cuts the observation off (censored). The curve is the Kaplan-Meier estimate, `confidence` draws the 95 % band (log-log, Greenwood variance), `atRisk` (default true) writes the number at risk under the time axis. See [Survival curves](../guides/chart-types.md#survival-curves). |
| `scatter` | scatter | `{ "points": [{ "x": 1.2, "y": 3.4, "group": "Up", "label": "Gene A" }], "lines": [{ "axis": "y", "value": 1.3, "label": "p = 0.05" }], "xTitle": "…", "yTitle": "…" }`. 1–5000 points with finite numbers; `group` on every point or none (`missing_group`), up to 4 groups with a legend; up to 8 `lines` across the plot at a value of `x` or `y`; a `label` writes a name beside the point where there is room. From 300 points on the dots are small and have no tooltip; from 100 the data table lists only the labeled points, or the 20 highest. See [Scatter plots](../guides/chart-types.md#scatter-plots). |
| `waffle` | waffle | `{ "parts": [{ "label": "Wind", "value": 31 }, { "label": "Gas", "value": 28 }], "cells": 100, "columns": 10, "total": 100 }`. Up to 4 parts with unique labels and values above zero; `cells` 10–400 (default 100) squares in rows of `columns` (default about square), each part with the share of the squares by the largest remainder and at least one; a `total` above the sum of the parts leaves squares for the rest (`invalid_total` below it). See [Waffle charts](../guides/chart-types.md#waffle-charts). |
| `waterfall` | waterfall | `{ "steps": [{ "label": "Revenue", "value": 1250, "kind": "start" }, { "label": "Costs", "value": -420 }, { "label": "Margin", "kind": "total" }] }`; `orientation` and `valueAxis` apply. A `delta` (default) adds its value, a `start` sets the total as a bar from zero, a `total` shows the running total and takes no value, or the matching one (`total_mismatch`); `missing_value` otherwise. 2–40 steps with unique labels. See [Waterfalls](../guides/chart-types.md#waterfalls). |
| `sequence` | sequence | `{ "participants": [{ "id": "api", "label": "…", "sublabel": "…", "kind": "service" }], "messages": [{ "from": "api", "to": "db", "label": "…", "kind": "call" }], "fragments": [{ "kind": "alt", "label": "…", "from": 0, "to": 2, "else": [{ "from": 1, "label": "…" }] }], "numbered": false, "orientation": "auto" }`. `kind` of a participant: `service` (default), `actor`, `database`, `queue`, `external`; of a message: `call` (default), `reply`, `async`; of a fragment: `alt`, `opt`, `loop`, `par`, `critical`, `break`. `orientation`: `auto` (default), `portrait` or `landscape`. See [Sequence diagrams](../guides/chart-types.md#sequence-diagrams). |
| `flow` | flow | `{ "nodes": [{ "id": "a", "label": "…", "sublabel": "…", "kind": "process", "lane": "x" }], "edges": [{ "from": "a", "to": "b", "label": "…", "dash": "solid" }], "lanes": [{ "id": "x", "label": "…" }], "groups": [{ "label": "…", "nodes": ["a"] }], "mainPath": ["a", "b"], "orientation": "auto" }`. `kind` of a step: `process` (default), `start`, `end`, `decision`, `io`, `subprocess`, `store`, `external`. See [Flow charts](../guides/chart-types.md#flow-charts). |
| `state` | state | `{ "states": [{ "id": "a", "label": "…", "sublabel": "…", "kind": "state", "final": false, "in": "c" }], "transitions": [{ "from": "a", "to": "b", "event": "…", "guard": "…", "action": "…", "dash": "solid" }], "initial": "a", "mainPath": ["a", "b"], "orientation": "auto" }`. `kind`: `state` (default), `choice` or `composite`. See [State diagrams](../guides/chart-types.md#state-diagrams). |
| `architecture` | architecture | `{ "components": [{ "id": "a", "label": "…", "sublabel": "…", "kind": "service", "in": "net" }], "connections": [{ "from": "a", "to": "b", "label": "…", "technology": "…", "dash": "solid" }], "boundaries": [{ "id": "net", "label": "…", "in": "region" }], "mainPath": ["a", "b"], "orientation": "auto" }`. `kind`: `person`, `frontend`, `service` (default), `database`, `queue`, `storage`, `cache`, `security`, `external`. See [Architecture diagrams](../guides/chart-types.md#architecture-diagrams). |
| `tree` | tree | `{ "nodes": [{ "id": "a", "label": "…", "sublabel": "…", "parent": "root", "link": "60 %", "kind": "unit", "partner": "b" }], "orientation": "auto" }`. Exactly one node has neither a parent nor a partner: the root. The children of a node keep the order of the list. `kind`: `unit` (default), `person` or `external`. A `partner` joins two nodes as a couple, side by side; the children of either hang from the middle of the line between them. See [Trees](../guides/chart-types.md#trees). |
| `patterns` | no | `bar` with `series` only: `true` draws every other series as an outline — the background inside, the series color around it — in the bars, the legend and stacks, so that series differ in form as well as in color. |
| `stack` | no | `bar` with `series` only: `"normal"` stacks the series of a category by value, positive ones up and negative ones down, with the total beyond each stack; `"percent"` stacks shares of each category's total (values of zero or more, axis in percent). A stack has no series filter. |
| `references` | no | `bar` and `rangebar`: up to four reference lines across the bars, `[{ "value": 48, "label": "EU average" }]`. A line widens the value axis to reach its value; the description names it. |
| `panes[].layers[].curve` | no | Line and area: `linear` (default) joins the observations directly; `step` holds each value until the next observation, for values that stand for a whole period such as an annual mean. |
| `panes[].layers[].color` | no | `#rgb`, `#rrggbb`, `#rrggbbaa`, `var(--name)`, or `var(--name, #hex)` with a hex fallback, which the print variant draws. |
| `description` | no | Replaces the generated description. |
| `source` | no | Shown below the chart in the HTML output. |
| `categoryAxis.title` | no | Title of the category axis. |
| `valueAxis.title` | no | Title of the value axis. |
| `valueAxis.format` | no | `number` (default) or `percent`; `0.12` is shown as `12%`. |
| `valueAxis.decimals` | no | Fixed decimal places, 0–6, for values in tooltips, value labels, description and table. Axis ticks always carry as many decimals as their step (`0.0, 0.5, 1.0`). On a `time` chart every pane sets it on its own `valueAxis`. |
| `valueAxis.min`, `valueAxis.max` | no | The axis reaches at least down to `min` and up to `max`; values beyond them widen it further, and each end is rounded outward to a tick. Without them, a line, time series or range bar whose values are all zero or more never gets a margin below zero (and one whose values are all zero or less none above it). Bars always start at zero. On a `time` chart every pane sets them on its own `valueAxis`. |
| `valueAxis.thousandsSeparator` | no | `true` separates thousands in ticks, value labels, tooltips, description and table: `12,500`, or `12.500` with `locale: "de"`. Off by default, because four-digit values are often years. On a `time` chart every pane sets it on its own `valueAxis`. |
| `valueAxis.scale` | no | `linear` (default) or `log`: powers of ten evenly spaced, ticks at each power and, over two decades or fewer, at 2 and 5 times it. Every value, band and zone edge, reference line and declared `min`/`max` must be above zero; bars start at the bottom of the axis. Not with `stack` or an `area` layer. On a `time` chart every pane sets it on its own `valueAxis`. |
| `valueAxis.step`, `valueAxis.exact` | no | `step` sets the distance between ticks instead of a round step chosen from the values. `exact: true` makes `min` and `max` the ends of the axis — both are required, every value has to lie between them (`value_outside_axis` otherwise), and the ticks sit on multiples of the step between them: `{ "min": -2.5, "max": 3.5, "step": 1, "exact": true }` runs from −2.5 to 3.5 with ticks at −2, −1, 0 … 3 and the zero line. A bar chart's exact axis includes zero. Not with a logarithmic axis or a stack. |
| `valueAxis.unit` | no | `time` (per pane) and `multiples`: the unit of the values, such as `W/m²`, up to 20 characters. It follows the top tick label (`4 W/m²`) and the column names of the data table, and costs no height, unlike a title above the plot. |
| `valueAxis.reverse` | no | `true` runs the axis the other way, larger values down (or left on horizontal bars), as records such as δ18O are conventionally drawn. |
| `locale` | no | `en` (default) or `de`: language of the generated texts (description, legend additions, tooltips, calendar month and weekday names, HTML caption and table) the number format (`1,5`) and dates: ISO 8601 (`2026-08-31`) in English, `31.08.2026` and `08.2026` in German. Available for every chart type. Negative numbers always use the true minus sign `−`. |
| `showTitle` | no | `false` leaves the drawn title out of the SVG profile when the page heads the chart, and gives its space to the chart; it stays the accessible name. Every chart type. The HTML profile never draws the title (its `<figcaption>` is the visible title), so it ignores `showTitle`. Default `true`. |
| `width`, `height` | no | Size in pixels: 200–2400 × 160–1600, default 800 × 450. Below 320 pixels a chart is compact, for a panel in a grid of columns: a narrower value-axis gutter and margin keep room for the plot, and the text keeps its size when the chart is shown at the width it was rendered for. Text shrinks with the chart when a page scales it down, so render at the width of its place. |
| `mobile` | no | A second layout for narrow containers: `{ "width": 360, "height": 360, "breakpoint": 640 }`. `width` is required, 200–600; `height` 160–1600, default 360; `breakpoint` 320–1600, default 640, is the container width in CSS pixels below which the HTML profile shows the mobile variant; `columns`, for small multiples only, sets the grid columns of the mobile variant, usually 1. Available for every chart type. The SVG profile is unchanged; `--variant mobile` renders the mobile variant alone. See [Responsive charts](../guides/responsive.md). |
| `showValues` | no | Value labels on bars and points, default `true`. On a time pane with several layers the labels can overlap; the values stay in the tooltips and the data table. |
| `zoomSteps` | no | Two to four `{ "label", "from", "to" }` variants, selectable in the HTML output. Bar and line charts: `from` and `to` are category indices. `time` charts: timestamps in the forms a point's `time` takes; each window must hold at least two observations of one layer, and the chart is redrawn from the observations inside it. |

## Limits

Up to 100 categories and four series. Labels must be unique. Values must be zero or have a
magnitude between `1e-100` and `1e100`.

A `time` chart carries one to four panes (`too_many_panes`), each with a `heightRatio` from 1 to 10
(`invalid_height_ratio`) and up to six data layers (eight in a stacked pane) of 2000 observations or candles each. At most
four data layers of the whole chart take a palette color — candles take none — and a further layer
needs its own `color` (`too_many_layers`). Layer names are unique across the panes
(`duplicate_series`), and every data layer needs one once the chart has more than one
(`missing_name`). A candle whose `low` lies above its open or close, or whose `high` lies below
them, is `invalid_candle`; more candles than a third of the plot's horizontal pixels are drawn as
wicks only and reported as `dense_chart`. A zoom window has to hold two observations of a layer in
every pane (`zoom_out_of_range`). With `"gaps": "collapse"`, a reference line or point marker outside
the observed range and a zone that encloses no observation are `time_out_of_range`. Markers and
value labels are drawn up to 60 observations per layer; a denser layer is a line only, and the
values stay in the data table. More observations than the plot has horizontal pixels produce a
`dense_chart` warning. These numbers come from a measurement at the default size — see
[the time series guide](../guides/time-series.md).

A mobile variant is laid out from the same specification at `mobile.width` × `mobile.height`. A
size outside its limits is `invalid_dimension` with the path of the field (`/mobile/width`,
`/mobile/height`, `/mobile/breakpoint`); a `mobile` without `width` is `invalid_spec` at `/mobile`.
Warnings that only the mobile layout raises are reported with the same code and path and a message
that begins with `mobile variant: `.

Small multiples take 2 to 12 panes in up to six columns, with at most four distinct layer names
across all panels (one palette color each). Stripes take up to 500 yearly values; a calendar
covers one year between 1700 and 2199. Range bars take up to 100 ranges. A sequence diagram takes 1–12
participants (`too_many_participants`), 1–60 messages (`too_many_messages`) and up to 12 fragments
(`too_many_fragments`); participant `id`s are identifiers (`invalid_id`) and unique
(`duplicate_id`), every message names existing participants (`unknown_participant`), and fragments
lie within the messages, nest at most three deep and never cross (`invalid_fragment`,
`fragments_cross`). A flow chart takes 1–40 steps (`too_many_nodes`), up to 80 edges
(`too_many_edges`), 8 lanes (`too_many_lanes`) and 8 groups (`too_many_groups`); step and lane
`id`s are identifiers and unique, edges, groups and the main path name existing steps
(`unknown_node`), every step names a declared lane once there are lanes (`missing_lane`,
`unknown_lane`), a step belongs to one group at most (`duplicate_member`) and a group to one lane
(`group_spans_lanes`), and the main path follows edges (`main_path_gap`). A sequence diagram or
flow chart that does not fit its canvas is drawn larger and reported as `canvas_too_small` at
`/width` or `/height`; a step that lands inside the frame of a group it does not belong to is
`group_overlap`. A state diagram takes 1–40 states (`too_many_states`) and up to 80 transitions
(`too_many_transitions`); transitions, `initial` and the main path name existing states
(`unknown_state`), a choice or composite state cannot be final (`option_not_supported`), states lie in composite states (`not_composite`) without a circle (`state_cycle`) at most four deep (`states_too_deep`), a composite state holds a state (`empty_composite`) and is never named by a transition, `initial` or the main path (`composite_not_allowed`), and with an `initial`
state every other state needs a transition into it (`unreachable_state`, a warning). An architecture diagram takes 1–40 components (`too_many_nodes`),
up to 80 connections (`too_many_edges`) and 12 boundaries (`too_many_boundaries`) nested at most
four deep (`boundaries_too_deep`); identifiers are unique among components and boundaries
(`duplicate_id`), components and boundaries lie in existing boundaries (`unknown_boundary`)
without a circle (`boundary_cycle`), every boundary holds a component (`empty_boundary`), and
connections and the main path name existing components (`unknown_node`, `main_path_gap`). A tree takes 1–150 nodes (`too_many_nodes`) down to twelve levels below the root (`tree_too_deep`); identifiers are unique (`duplicate_id`), exactly one node has no parent (`invalid_root`), parents exist (`unknown_node`) and run up to the root without a circle (`circular_parent`), the root has no `link` (`option_not_supported`), and a partner exists, is another node, has no parent and is named by one node only (`unknown_node`, `invalid_partner`).

A pane takes up to six annotation layers — zones, reference lines and point markers together
(`too_many_annotations`). A zone needs `bottom` and `top`, `from` and `to`, or both
(`missing_position`); `bottom` at or above `top`, or `from` at or after `to`, is `invalid_band`; a
zone, reference line or point marker without `label` is `missing_label`. A field that belongs to
another mark — `name`, `points`, `value` or `shape` on a zone, `shape` on a reference line — is
`option_not_supported` with its path. A label that overlaps another annotation label, crosses a
data line, or leaves the plot is reported as a `label_overlap` warning.

Fields that belong to a later milestone are refused with a named error rather than ignored:
`zoomSteps` on small multiples, the `ohlc` mark on small multiples (`option_not_supported`), `title`
on the pane of a `time` chart, and the bar and line fields `data`, `categories`, `series`, `orientation`, and `categoryAxis` on a
`time` chart (`option_not_supported`). A block that belongs to another type (`stripes`,
`calendar`, `ranges`, `boxes`, `timeline`, `parliament`, `treemap`, `sankey`, `survival`, `scatter`, `waffle`, `waterfall`, `sequence`, `flow`, `state`, `architecture`, `tree`, `columns`) is refused the same way.

## Diverging scale

Stripes and calendars share one color scale: a neutral middle at `reference` (default 0) and
eight equally wide steps on either side. `min` and `max` set where the outermost steps begin and
default to the largest distance of any value from the reference; values beyond them take the
outermost step. The 17 colors are CSS custom properties `--chartlet-diverging-0` to `-16` and can
be overridden by the host page.
