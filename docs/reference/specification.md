---
title: Specification
description: Every field of a chart specification, and the limits the renderer enforces.
order: 1
---

[`schema/chartlet.schema.json`](https://github.com/casoon/chartlet/blob/main/schema/chartlet.schema.json)
is the complete contract and can be used for editor validation. This page describes every field by
chart type, and the rules the renderer checks beyond the schema. An error names its code and the
place in the specification as a JSON Pointer (`/flow/edges/2/to`); see
[Warnings and errors](../../guides/warnings-and-errors/).

| `type` | Section | Drawn from |
| --- | --- | --- |
| `bar`, `line` | [Bar and line charts](#bar-and-line-charts) | `data`, or `categories` and `series` |
| `time` | [Time charts](#time-charts) | `timeAxis`, `panes` |
| `multiples` | [Small multiples](#small-multiples) | `panes` |
| `stripes` | [Stripes](#stripes) | `stripes` |
| `calendar` | [Calendars](#calendars) | `calendar` |
| `rangebar` | [Range bars](#range-bars) | `ranges` |
| `boxplot` | [Box plots](#box-plots) | `boxes` |
| `timeline` | [Timelines](#timelines) | `timeline` |
| `parliament` | [Parliament charts](#parliament-charts) | `parliament` |
| `treemap` | [Treemaps](#treemaps) | `treemap` |
| `sankey` | [Sankey diagrams](#sankey-diagrams) | `sankey` |
| `survival` | [Survival curves](#survival-curves) | `survival` |
| `scatter` | [Scatter plots](#scatter-plots) | `scatter` |
| `waffle` | [Waffle charts](#waffle-charts) | `waffle` |
| `funnel` | [Funnels and pyramids](#funnels-and-pyramids) | `funnel` |
| `radar` | [Radar charts](#radar-charts) | `radar` |
| `polar` | [Polar charts](#polar-charts) | `polar` |
| `mosaic` | [Mosaic charts](#mosaic-charts) | `mosaic` |
| `heatmap` | [Heatmaps](#heatmaps) | `heatmap` |
| `waterfall` | [Waterfalls](#waterfalls) | `waterfall` |
| `sequence` | [Sequence diagrams](#sequence-diagrams) | `sequence` |
| `flow` | [Flow charts](#flow-charts) | `flow` |
| `state` | [State diagrams](#state-diagrams) | `state` |
| `classes` | [Class diagrams](#class-diagrams) | `classes` |
| `architecture` | [Architecture diagrams](#architecture-diagrams) | `architecture` |
| `tree` | [Trees](#trees) | `tree` |
| `network` | [Networks](#networks) | `network` |
| `venn` | [Venn diagrams](#venn-diagrams) | `venn` |
| `valuemap` | [Value chain maps](#value-chain-maps) | `valuemap` |
| `journey` | [User journeys](#user-journeys-and-service-blueprints) | `journey` |
| `storymap` | [Story maps](#story-maps) | `storymap` |
| `kanban` | [Kanban boards](#kanban-boards) | `kanban` |
| `fishbone` | [Fishbone diagrams](#fishbone-diagrams) | `fishbone` |
| `cycle` | [Cycles](#cycles) | `cycle` |
| `layers` | [Layers](#layers) | `layers` |
| `plan` | [Plans](#plans) | `plan` |

Fields every type takes are in [Common fields](#common-fields), the value axis of the charts that
have one in [Value axis](#value-axis), and what the diagrams share in
[Diagram fields](#diagram-fields).

## Common fields

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `schemaVersion` | `1` | yes | Always `1`. |
| `type` | string | yes | One of the types in the table above. |
| `title` | string, up to 200 characters | yes | Visible title; also the accessible name of the chart (`<title>`). The SVG profile draws it in the chart: a title wider than the chart wraps onto a second line at a space, and the chart below it moves down; only what does not fit on two lines is shortened (`text_truncated`). The HTML profile shows it as the `<figcaption>` and draws no title in its SVGs, so it never shortens it. |
| `showTitle` | boolean | no | `false` leaves the drawn title out of the SVG profile when the page heads the chart, and gives its space to the chart; it stays the accessible name. Every chart type. The HTML profile never draws the title (its `<figcaption>` is the visible title), so it ignores `showTitle`. Default `true`. |
| `description` | string, up to 1000 characters | no | Replaces the generated description. |
| `source` | string, up to 300 characters | no | Shown below the chart in the HTML output. |
| `theme` | `light`, `dark`, `blueprint` | no | `light` (default), `dark` or, for diagrams, `blueprint`; the palettes are CSS custom properties on the root. |
| `tokens` | object | no | Single colors of the palette by the name of the `--chartlet-*` custom property without its prefix, such as `{ "accent": "#c2410c" }`; hex or `rgb()`/`rgba()` values. They win over the theme and carry into the print and social variants and PNG. `invalid_token` at `/tokens/<name>` for another name or value. See [Customize the look](../../guides/customize/). |
| `stroke` | `clean`, `sketch` | no | `clean` (default) or `sketch`: lines and shapes are displaced by up to about a pixel and a half and bowed slightly, for essays and slides. The displacement is seeded from the specification, so the bytes are the same every time; text, colors and contrast are untouched. The print variant and PNG draw clean strokes. See [Customize the look](../../guides/customize/#hand-drawn-strokes). |
| `typography` | object | no | `family` (titles and labels), `technicalFamily` (technical sublabels such as the technology of a node) and `widthScale` (0.7 to 1.5, the measured width of the family relative to the built-in metrics). The families come first in the font stack, before Inter; the output names them but does not carry them (`font_not_embedded`). `invalid_typography` at `/typography/<field>` for a value that is not a plain family name or scale. See [Typography](../../guides/customize/#typography). |
| `locale` | `en`, `de` | no | `en` (default) or `de`: language of the generated texts (description, legend additions, tooltips, calendar month and weekday names, HTML caption and table) the number format (`1,5`) and dates: ISO 8601 (`2026-08-31`) in English, `31.08.2026` and `08.2026` in German. Available for every chart type. Negative numbers always use the true minus sign `−`. |
| `size` | `doc`, `doc-wide`, `slide-16x9`, `slide-4x3`, `social-og`, `social-square`, `print-a4-landscape`, `print-a3-landscape`, `print-letter-landscape` | no | A named canvas size that sets `width` and `height`: 800 × 450, 1200 × 500, 1920 × 1080, 1600 × 1200, 1200 × 630, 1080 × 1080, 1123 × 794, 1587 × 1123 and 1056 × 816 CSS pixels (the paper sizes at 96 dpi). A `width` or `height` in the specification overrides that side of the preset; without `size` nothing changes. `chartlet render --size <preset>` sets it from the command line. The preset sets the canvas only: type size, margins and colors are not scaled with it, and the `social` and `print` variants stay as they are. An unknown name is `invalid_spec` at `/size`, and the message lists the valid names. |
| `width`, `height` | integer | no | Size in pixels: 200–2400 × 160–1600, default 800 × 450. Below 320 pixels a chart is compact, for a panel in a grid of columns: a narrower value-axis gutter and margin keep room for the plot, and the text keeps its size when the chart is shown at the width it was rendered for. Text shrinks with the chart when a page scales it down, so render at the width of its place. |
| `mobile` | object | no | A second layout for narrow containers: `{ "width": 360, "height": 360, "breakpoint": 640 }`. Available for every chart type. The SVG profile is unchanged; `--variant mobile` renders the mobile variant alone. See [Responsive charts](../../guides/responsive/). |
| `mobile.width` | integer, 200–600 | with `mobile` | Width of the mobile variant. |
| `mobile.height` | integer, 160–1600 | no | Height of the mobile variant, default 360. |
| `mobile.breakpoint` | integer, 320–1600 | no | The container width in CSS pixels below which the HTML profile shows the mobile variant, default 640. |
| `mobile.columns` | integer, 1–6 | no | Small multiples only: the grid columns of the mobile variant, usually 1. |
| `showValues` | boolean | no | Value labels on bars and points, default `true`. On a time pane with several layers the labels can overlap; the values stay in the tooltips and the data table. |

Rules:

- `schemaVersion` other than 1 is `unsupported_schema_version`.
- A size outside its limits is `invalid_dimension` with the path of the field (`/width`, `/height`,
  `/mobile/width`, `/mobile/height`, `/mobile/breakpoint`); a `mobile` without `width` is
  `invalid_spec` at `/mobile`.
- A mobile variant is laid out from the same specification at `mobile.width` × `mobile.height`.
  Warnings that only the mobile layout raises are reported with the same code and path and a
  message that begins with `mobile variant: `. Asking for the mobile variant of a specification
  without `mobile` is `missing_mobile` at `/mobile`.
- `"theme": "blueprint"` is for sequence, flow, state, architecture, tree, class, network, cycle, layers and plan
  diagrams; any other type refuses it (`option_not_supported` at `/theme`).
- Texts must contain visible text (`empty_text`), stay within their length (`text_too_long`) and
  hold no control character that XML 1.0 cannot represent (`invalid_xml_character`).
- Numbers must be finite (`non_finite_value`) and zero or of a magnitude between `1e-100` and
  `1e100` (`unsupported_numeric_range`).
- A block that belongs to another type (`stripes`, `calendar`, `ranges`, `boxes`, `timeline`,
  `parliament`, `treemap`, `sankey`, `survival`, `scatter`, `classes`, `waffle`, `funnel`, `radar`, `polar`, `mosaic`, `heatmap`, `waterfall`,
  `sequence`, `flow`, `state`, `architecture`, `tree`, `network`, `cycle`, `fishbone`, `kanban`, `storymap`, `journey`, `valuemap`, `venn`, `layers`, `plan`, `columns`),
  and a type's own field on another type (`showMean`, `violinScale`, `boxDisplay`, `references`,
  `stack`, `patterns`, `slope`, `rank`, `legend`, `sparkline`, `tooltips`, `independentAxes`, `mobile.columns`), is
  refused with `option_not_supported` rather than ignored.
- A type whose block is absent is refused with the block's own code: `missing_stripes`,
  `missing_calendar`, `missing_timeline`, `missing_parliament`, `missing_treemap`,
  `missing_sankey`, `missing_survival`, `missing_scatter`, `missing_waffle`, `missing_funnel`, `missing_radar`, `missing_polar`, `missing_mosaic`, `missing_heatmap`, `missing_waterfall`,
  `missing_sequence`, `missing_flow`, `missing_state`,
  `missing_classes`, `missing_architecture`, `missing_tree`, `missing_network`, `missing_cycle`, `missing_fishbone`, `missing_kanban`, `missing_storymap`, `missing_journey`, `missing_valuemap`, `missing_layers`, `missing_plan` and `missing_venn`. An empty list
  where a type needs at least one entry is `empty_data`.
- Stripes, calendars, timelines and every diagram are drawn from their block alone:
  `data`, `categories`, `series`, `zoomSteps`, `panes`, `orientation`, `categoryAxis`,
  `valueAxis` and `timeAxis` are `option_not_supported` there.

## Value axis

Bar, line, range bar, box plot and waterfall charts take a top-level `valueAxis`, small multiples one shared
by every panel (or one per panel, see [Small multiples](#small-multiples)), and every pane of a
`time` chart its own. Bar and line charts also take `categoryAxis`.

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `categoryAxis.title` | string, up to 100 characters | no | Title of the category axis. |
| `valueAxis.title` | string, up to 100 characters | no | Title of the value axis. |
| `valueAxis.format` | `number`, `percent` | no | `number` (default) or `percent`; `0.12` is shown as `12%`. |
| `valueAxis.decimals` | integer, 0–6 | no | Fixed decimal places for values in tooltips, value labels, description and table. Axis ticks always carry as many decimals as their step (`0.0, 0.5, 1.0`). On a `time` chart every pane sets it on its own `valueAxis`. |
| `valueAxis.min`, `valueAxis.max` | number | no | The axis reaches at least down to `min` and up to `max`; values beyond them widen it further, and each end is rounded outward to a tick. Without them, a line, time series or range bar whose values are all zero or more never gets a margin below zero (and one whose values are all zero or less none above it). Bars always start at zero. On a `time` chart every pane sets them on its own `valueAxis`. |
| `valueAxis.thousandsSeparator` | boolean | no | `true` separates thousands in ticks, value labels, tooltips, description and table: `12,500`, or `12.500` with `locale: "de"`. Off by default, because four-digit values are often years. On a `time` chart every pane sets it on its own `valueAxis`. |
| `valueAxis.scale` | `linear`, `log` | no | `linear` (default) or `log`: powers of ten evenly spaced, ticks at each power and, over two decades or fewer, at 2 and 5 times it. Every value, band and zone edge, reference line and declared `min`/`max` must be above zero; bars start at the bottom of the axis. Not with `stack` or an `area` layer. On a `time` chart every pane sets it on its own `valueAxis`. |
| `valueAxis.step` | number above 0 | no | The distance between ticks instead of a round step chosen from the values. |
| `valueAxis.exact` | boolean | no | `true` makes `min` and `max` the ends of the axis — both are required, every value has to lie between them, and the ticks sit on multiples of the step between them: `{ "min": -2.5, "max": 3.5, "step": 1, "exact": true }` runs from −2.5 to 3.5 with ticks at −2, −1, 0 … 3 and the zero line. A bar chart's exact axis includes zero. Not with a logarithmic axis or a stack. |
| `valueAxis.unit` | string, up to 20 characters | no | `time` (per pane) and `multiples`: the unit of the values, such as `W/m²`. It follows the top tick label (`4 W/m²`) and the column names of the data table, and costs no height, unlike a title above the plot. |
| `valueAxis.reverse` | boolean | no | `true` runs the axis the other way, larger values down (or left on horizontal bars), as records such as δ18O are conventionally drawn. |

Rules:

- `min` at or above `max` is `invalid_axis_range` at `max`; `decimals` above 6 is
  `invalid_decimals`.
- A `step` of zero or less is `invalid_value`; a step that gives more than 50 ticks across the
  values and declared ends, or fewer than two on an exact axis, is `invalid_axis_range` at `step`.
- `exact` without both `min` and `max`, or a bar chart's exact axis that leaves out zero, is
  `invalid_axis_range`; a value outside an exact axis is `value_outside_axis`.
- On a logarithmic axis a value, edge or bound of zero or less is `invalid_value`; `log` with a
  stack or an `area` layer, `step` or `exact` with `log`, `exact` with a stack and `unit` on any
  other type than `time` or `multiples` are `option_not_supported`.

## Bar and line charts

`"type": "bar"` and `"type": "line"`. A chart has `data` (one series) or `categories` with
`series`, never both (`conflicting_data_shape`). [Value axis](#value-axis) and `categoryAxis` apply.

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `data` | array, 1–100 | one of | Single series: `[{ "label": "…", "value": 1, "group": "…", "lower": 0.5, "upper": 1.5 }]`. |
| `data[].label` | string, up to 200 characters | yes | The category. Labels are unique. |
| `data[].value` | number or `null` | yes | The value; `null` marks a missing value (line charts). |
| `data[].group` | string, up to 60 characters | no | Bar charts: gives every bar of a group one palette color and a legend entry; either every point names a group or none does. |
| `data[].lower`, `data[].upper` | number | no | Bar charts without groups, and single-series line charts: an error bar through the end of the bar or the point, on every point or on none, the value between them. |
| `categories` | array of strings, 1–100 | one of | Unique category labels shared by all series. |
| `series` | array, 1–4 | with `categories` | `[{ "name": "…", "values": […] }]`. |
| `series[].name` | string, up to 100 characters | yes | Legend and table label; unique. |
| `series[].values` | array of numbers or `null` | yes | One value per category; `null` marks a missing value. |
| `series[].lower`, `series[].upper` | array of numbers or `null` | no | A series of a bar or line chart without a stack: error bars, one entry per category, `null` where the value is `null`. |
| `orientation` | `vertical`, `horizontal` | no | `vertical` (default) or `horizontal`; bar charts only. |
| `stack` | `normal`, `percent` | no | `bar` with `series` only: `"normal"` stacks the series of a category by value, positive ones up and negative ones down, with the total beyond each stack; `"percent"` stacks shares of each category's total (values of zero or more, axis in percent). A stack has no series filter. |
| `slope` | boolean | no | `line` with `categories` and `series` only: `true` draws a slope chart, one line per series from its first value to its second, with the name and value at both ends instead of a legend. Exactly two categories (`invalid_categories`), a value in every category (`missing_value`), no error bars or zoom steps; names that would overlap are spread apart. |
| `rank` | boolean | no | `line` with `categories` and `series` only: `true` plots the rank of each series in every category, 1 for the largest value at the top, with the names and ranks at both ends instead of a legend. At least two categories and two series (`invalid_categories`), a value in every category (`missing_value`), no value axis; equal values keep the order of the series. Not together with `slope`. |
| `patterns` | boolean | no | `bar` with `series` only: `true` draws every other series as an outline — the background inside, the series color around it — in the bars, the legend and stacks, so that series differ in form as well as in color. |
| `references` | array, up to 4 | no | `bar` and `rangebar`: reference lines across the bars, `[{ "value": 48, "label": "EU average" }]`. A line widens the value axis to reach its value; the description names it. |
| `references[].value` | number | yes | Where the line stands. |
| `references[].label` | string, up to 60 characters | yes | Its text. |
| `zoomSteps` | array, 2–4 | no | Variants selectable in the HTML output, each `{ "label", "from", "to" }`: `from` and `to` are category indices. |
| `zoomSteps[].label` | string, up to 40 characters | yes | Name of the variant. |
| `zoomSteps[].from`, `zoomSteps[].to` | integer, from 0 | yes | First and last category of the variant. |

Rules:

- Up to 100 categories and four series (`too_many_data_points`, `too_many_series`). Labels must be
  unique (`duplicate_label`), and so must series names (`duplicate_series`); a series has one
  value per category (`series_length_mismatch`).
- A bar chart needs a value for every category of `data` (`missing_bar_value`); a chart whose
  values are all `null` is `empty_series`.
- Groups: on every point or none (`missing_group`), at most 4 (`too_many_series`), bar charts only
  and not together with error bars (`option_not_supported`).
- Error bars: `lower` and `upper` on every point or value or on none (`missing_bounds`), the value
  between them (`invalid_bounds`); on another type or with a stack `option_not_supported`.
- `stack` needs categories with at least two series (`option_not_supported`); a percent stack
  takes no negative values (`invalid_value`).
- More than four reference lines are `too_many_references`.
- Zoom steps: one alone is `not_enough_zoom_steps`, more than four `too_many_zoom_steps`; an index
  that is not a whole number from 0 is `invalid_spec`, `from` after `to` is `invalid_zoom_step`,
  `to` beyond the last category `zoom_out_of_range`.
- `orientation` on a line chart is `option_not_supported`. More than 16 categories warn with
  `dense_chart`; when their labels do not fit even on two lines, only every few categories are
  labelled (`labels_thinned`). A bar chart category whose value labels have no room shows none
  (`value_labels_omitted`).

## Time charts

`"type": "time"`: one to four panes stacked on one shared time axis, each with its own `valueAxis`
(see [Value axis](#value-axis)) and `layers`. See [the time series guide](../../guides/time-series/).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `timeAxis.kind` | `calendar`, `number` | no | `calendar` (default) reads every `time` as a timestamp. `number` reads it as a plain number — a distance along a profile, a depth, an age in millions of years — with ticks at round numbers, values written in the chart's `locale`, and the axis `title` as the head of the table's first column. No `timezone`, `gaps: "collapse"` or `zoomSteps` with it. |
| `timeAxis.timezone` | string | no | Fixed UTC offset such as `"+02:00"`, or `UTC` (default). Timestamps without their own offset are read as wall-clock time in this zone. |
| `timeAxis.title` | string, up to 100 characters | no | Title of the time axis. |
| `timeAxis.gaps` | `show`, `collapse` | no | `show` (default) keeps the distances in time; `collapse` places every observed timestamp at the same distance from the next, so weekends and holidays take no space. Reference lines, point markers and zones then take the slot of an observation and must lie within the observed range. |
| `timeAxis.precision` | `year`, `month`, `day`, `minute` | no | How tooltips, the data table and the description name an observation. Absent, the coarsest form that tells the observations apart is chosen: a year when all fall on January 1, a month when all fall on the first of a month. Set it for annual values placed mid-year (`1950-07-01` → `1950`). Each data layer is named by its own observations (or its own `precision`), and a table row as finely as the finest layer with a value there. |
| `timeAxis.step` | number above 0 | no | The distance between time ticks instead of one chosen from the width: whole years (1–1000) on a calendar axis, units on a numeric axis; ticks sit on its multiples (`50` → 1850, 1900, 1950, 2000). For a narrow panel whose claim needs a tick the automatic choice leaves out. A label that would run into the one before it is left out; its gridline stays. |
| `timeAxis.min`, `timeAxis.max` | time | no | Positions the time axis reaches at least, such as `"1990"` before data from 1991, so that a round tick is drawn. They extend the axis and never cut an observation off; they are no observations (no table row, not in the description). Not with `"gaps": "collapse"`. |
| `timeAxis.reverse` | boolean | no | `true` runs the axis from right to left, the largest position at the left: ages before present read from the oldest on the left to today on the right. |
| `panes` | array, 1–4 | yes | The panes, top to bottom. |
| `panes[].heightRatio` | integer, 1–10 | no | The pane's share of the plot height against the other panes, default 1. |
| `panes[].valueAxis` | object | no | `title`, `format`, `decimals`, `min`, `max`, `step`, `exact`, `thousandsSeparator`, `scale`, `reverse` and `unit` of the pane's own value axis. |
| `panes[].stack` | `normal`, `stream` | no | Stacks the pane's `area` layers in layer order, each on top of the ones before it, so that the top edge shows their total; line and point layers in the pane stay as they are. Markers and the end labels sit on top of the stack; tooltips and the data table give each layer's own value. `stream` stacks them around a moving baseline instead of zero, so that the stack is centered (a streamgraph): the value axis becomes a scale bar for thickness, the areas are named at their right ends, and the pane takes area layers and reference lines of time only, with no `min`, `max`, `exact`, `reverse` or logarithmic scale on its axis (`option_not_supported`). |
| `panes[].layers` | array | yes | Up to six data layers (eight in a stacked pane) and six annotation layers. |
| `sparkline` | boolean | no | `true` draws the lines and areas of one pane across the whole canvas, without axes, title, legend or value labels, with a dot at the end of each line; `width` 60–600 and `height` 16–200. The title and description remain the accessible name and description, and the HTML profile keeps its caption and table. Only line and area layers; no zoom steps or mobile variant. |
| `tooltips` | `observations`, `markers` | no | `"observations"` (default) gives every observation a tooltip while observations stand at least 4 pixels apart, on an invisible target where no marker is drawn; `"markers"` gives tooltips only to drawn markers, so a line too dense for markers has none and its SVG stays small. |
| `legend` | `top`, `end` | no | `top` (default) names the series in a legend above the plot; `end` writes each name at the last observation of its line, right of the plot, which makes room for the widest name (at most a third of the chart). Names that would overlap move apart. Not with candles, whose legend explains hollow and filled bodies. |
| `zoomSteps` | array, 2–4 | no | `{ "label", "from", "to" }` windows selectable in the HTML output; `from` and `to` are times. Each window must hold at least two observations of one layer, and the chart is redrawn from the observations inside it. |

A time is Unix seconds between 1700-01-01 and 2200-01-01, an ISO 8601 date such as `2026-03-01` or
`2026-03-01T12:00:00Z`, or a bare year such as `"1850"` (January 1st).

### Layers

Small multiples take the same layers, without `ohlc`.

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `mark` | `line`, `area`, `point`, `ohlc`, `band`, `annotation` | yes | An `area` is a line whose region down to zero is filled in its color; the value axis then always includes zero. A `point` layer draws a dot for every observation and no line, as in a scatter plot; it takes no `dash`, `stroke`, `curve`, `modeled` or `lower`/`upper`. `ohlc` draws candlesticks (`time` only), `band` a zone, `annotation` a reference line or point marker. The area between two lines is an uncertainty band (`lower`/`upper`), not a separate mark. |
| `name` | string, up to 100 characters | one of | Legend and table label; required once a pane — or a `time` chart across its panes — has several data layers, and unique within the pane; on a `time` chart unique across all panes, since they share one legend. |
| `points` | array, 1–2000 | line, area, point | `[{ "time": 1772323200, "value": 1 }]`; ISO 8601 dates and bare years (`"1850"`) are accepted too, with increasing `time`. `"value": null` marks a missing observation: the line, its area and its band break there, and table and description say “Missing”. |
| `points[].lower`, `points[].upper` | number | no | An uncertainty band: on every point with a value, or on none; a point without a value carries neither. |
| `data` | array, 2–2000 | ohlc | `[{ "time": "2026-03-02", "open": 10, "high": 12, "low": 9, "close": 11 }]`, candles with increasing `time`; `low` at or below `open` and `close`, `high` at or above them. A candlestick layer takes no `color`, `modeled`, `stroke` or `dash`. |
| `color` | string, up to 100 characters | no | `#rgb`, `#rrggbb`, `#rrggbbaa`, `var(--name)`, or `var(--name, #hex)` with a hex fallback, which the print variant draws. Anything else is painted in the neutral tone and warns with `color_not_supported`; a `var(--name)` without fallback is drawn in the text color by the print and social variants, with `color_not_resolved`. |
| `modeled` | boolean | no | Line and area layers: the line is dashed (unless `dash` says otherwise), its band hatched, and legend, description and table say “modelled”. |
| `stroke` | `regular`, `thin`, `medium`, `bold` | no | Line and area layers: `regular` (default, 3 px), `thin` (1 px), `medium` (2 px, for small panels) or `bold` (4.5 px); the legend sample shows the same weight, so two lines differ in more than color. |
| `dash` | `solid`, `dashed`, `dotted` | no | Line and area layers. Defaults to `dashed` for a modeled layer and `solid` otherwise; the legend sample shows the same pattern. |
| `curve` | `linear`, `step` | no | Line and area: `linear` (default) joins the observations directly; `step` holds each value until the next observation, for values that stand for a whole period such as an annual mean. |
| `stepEnd` | time | no | `line` and `area` with `"curve": "step"`: a time after the last observation where its step ends — annual means drawn as steps end at the end of their last year. It is no observation: no marker, tooltip, table row or description. Not with collapsed gaps or in a stacked pane. |
| `markers` | boolean | no | `line` and `area` only: `false` draws no markers, for a fitted or reference line whose observations are not data; tooltips stay on invisible targets unless the chart sets `"tooltips": "markers"`. |
| `precision` | `year`, `month`, `day`, `minute` | no | Data layers on a calendar axis: the same values as `timeAxis.precision`, for one layer — yearly means beside monthly values. On a numeric axis each layer writes its positions with the decimals its own positions need. |
| `value`, `time` | number, time | annotation | A horizontal reference line at `value`, a vertical one at `time`, or a point marker at both. Both widen the axes. |
| `shape` | `circle`, `square`, `diamond`, `triangle-up`, `triangle-down` | no | Point markers only, `circle` by default. |
| `bottom`, `top` | number | band | Value edges of a zone; `bottom` must lie below `top`. A missing edge is the edge of the plot. Given edges widen the value axis. |
| `from`, `to` | time | band | Time edges of a zone; `from` must lie before `to`. A missing edge is the edge of the plot. At least `bottom` and `top`, or `from` and `to`, are required. |
| `label` | string, up to 100 characters | annotation, band | Required text of a reference line, point marker or zone. |

Rules:

- One to four panes (`too_many_panes`), each with a `heightRatio` from 1 to 10
  (`invalid_height_ratio`) and up to six data layers (eight in a stacked pane) of 2000
  observations or candles each (`too_many_data_points`). At most four data layers of the whole
  chart take a palette color — candles take none — and a further layer needs its own `color`
  (`too_many_layers`). Layer names are unique across the panes (`duplicate_series`), and every
  data layer needs one once the chart has more than one (`missing_name`).
- Observations: a time that cannot be read is `invalid_time`; times must increase
  (`unordered_time`); a layer without observations, or a line, area or point layer with fewer
  than two, is `empty_series`; a candlestick layer needs at least two candles (`empty_series`). A `stepEnd`
  at or before the last observation is `unordered_time`.
- Bands: `lower` and `upper` on every point with a value or on none (`incomplete_band`), `upper`
  not below `lower` (`invalid_band`); a value outside its own band warns with
  `value_outside_band`.
- A candle whose `low` lies above its open or close, or whose `high` lies below them, is
  `invalid_candle`; more candles than a third of the plot's horizontal pixels are drawn as wicks
  only and reported as `dense_chart`.
- A stacked pane needs area layers with the same times and `curve`, a value of zero or more at
  every time (`unaligned_stack`, `negative_in_stack`) and no `lower`/`upper`.
- A pane takes up to six annotation layers — zones, reference lines and point markers together
  (`too_many_annotations`). A zone needs `bottom` and `top`, `from` and `to`, or both, and a
  reference line `time`, `value` or both (`missing_position`); `bottom` at or above `top`, or
  `from` at or after `to`, is `invalid_band`; a zone, reference line or point marker without
  `label` is `missing_label`. A field that belongs to another mark — `name`, `points`, `value` or
  `shape` on a zone, `shape` on a reference line — is `option_not_supported` with its path. A
  label that overlaps another annotation label, crosses a data line, or leaves the plot is
  reported as a `label_overlap` warning.
- With `"gaps": "collapse"`, a reference line or point marker outside the observed range and a
  zone that encloses no observation are `time_out_of_range`.
- `timeAxis`: `max` at or before `min` is `invalid_axis_range`; a `step` that is no whole number of
  years from 1 to 1000 on a calendar axis, not above zero on a numeric one, or that places no tick
  or more than 50, is `invalid_axis_range` at `/timeAxis/step`. A `timezone` that is neither `UTC`
  nor a fixed offset is `invalid_timezone`.
- Zoom steps: one alone is `not_enough_zoom_steps`, more than four `too_many_zoom_steps`, `from`
  at or after `to` is `invalid_zoom_step`, and a window has to hold two observations of a layer in
  every pane (`zoom_out_of_range`).
- `title` on a pane, and the bar and line fields `data`, `categories`, `series`, `orientation` and
  `categoryAxis`, are `option_not_supported` on a time chart.
- Markers and value labels are drawn up to 60 observations per layer; a denser layer is a line
  only, and the values stay in the data table. More observations than the plot has horizontal
  pixels produce a `dense_chart` warning. These numbers come from a measurement at the default
  size — see [the time series guide](../../guides/time-series/).

## Small multiples

`"type": "multiples"`: 2–12 panes in a grid, each with a unique `title`, sharing the top-level
`valueAxis`. A pane either has `layers` (as on a [time chart](#layers), without `ohlc`), or, with
top-level `categories`, its own `values` — small multiples of bars.

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `panes` | array, 2–12 | yes | The panels, in reading order. |
| `panes[].title` | string, up to 100 characters | yes | Heading of the panel. Panel titles take two lines before they are shortened; every panel keeps the same head height, so the plots stay aligned. |
| `panes[].note` | string, up to 160 characters | no | A short finding under the panel title, such as a verdict, on up to two lines. The notes are part of the description. |
| `panes[].noteEmphasis` | boolean | no | `true` draws the panel's note in the text color and semibold instead of muted, to set one finding apart from the others by weight, not by color alone. Needs a `note`. |
| `panes[].layers` | array | one of | Up to six data layers and six annotation layers. |
| `panes[].values` | array of numbers or `null`, up to 100 | one of | With `categories`: one value per category, `null` for none. |
| `panes[].valueAxis` | object | no | With `categories`: the panel's own value axis. |
| `categories` | array of strings, 1–20 | no | The categories of small multiples of bars. |
| `columns` | integer, 1–6 | no | Grid columns; default up to three. |
| `independentAxes` | boolean | no | `true` gives every panel its own value axis, scaled to its own values, for panels in different units or sizes. Their heights can then no longer be compared across panels; the description says so. |

Rules:

- Fewer than two panes is `not_enough_panes`, more than twelve `too_many_panes`; `columns` outside
  1–6 is `invalid_columns`. A pane without `title` is `missing_title`, two alike
  `duplicate_title`.
- At most four distinct layer names across all panels (one palette color each).
- Small multiples of bars: at most 20 categories (`too_many_data_points`), unique
  (`duplicate_label`); a panel of bars takes `values`, no `layers` or `stack`
  (`option_not_supported`), and draws horizontal bars (`orientation` is `option_not_supported`).
- `zoomSteps` and the `ohlc` mark are `option_not_supported`, and so are `heightRatio` and `stack`,
  which belong to a time chart; `note` and `noteEmphasis` belong to small multiples only.

## Stripes

`"type": "stripes"`. Stripes and calendars share one [diverging scale](#diverging-scale).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `stripes.firstYear` | integer | yes | The year of the first value; the values follow year by year. |
| `stripes.values` | array of numbers or `null`, 1–500 | yes | One value per year; `null` leaves a year empty. |
| `stripes.reference` | number | no | The neutral middle of the scale, default 0. |
| `stripes.min`, `stripes.max` | number | no | Where the outermost steps begin; default the largest distance of any value from the reference. |
| `stripes.yearLabels` | boolean | no | Label the first and the last year under the stripes, default `true`. |
| `stripes.stretch` | boolean | no | `true` draws the stripes alone across the whole canvas, without margins, title or year labels, and the SVG stretches to whatever box the page gives it (`preserveAspectRatio="none"`): give it a height in CSS, such as a 6-pixel band. Title and description remain its accessible name and description. |

Rules: up to 500 yearly values (`too_many_data_points`); every year between 1 and 9999
(`invalid_year`); at least one value that is not `null` (`empty_series`); `min` below and `max`
above the reference (`invalid_scale`).

## Calendars

`"type": "calendar"`: a heatmap of one year on the [diverging scale](#diverging-scale).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `calendar.year` | integer, 1700–2199 | yes | The year shown. |
| `calendar.layout` | `months`, `weeks` | no | `months` (default): twelve rows of up to 31 days; `weeks`: seven weekday rows, one column per week. |
| `calendar.days` | array, 1–366 | yes | `[{ "date": "2024-03-01", "value": 1 }]`. A day that is absent or `null` stays an empty cell. |
| `calendar.days[].date` | string | yes | ISO 8601 date in the chart's year. |
| `calendar.days[].value` | number or `null` | yes | The day's value. |
| `calendar.reference`, `calendar.min`, `calendar.max` | number | no | As for stripes. |

Rules: a year outside 1700–2199 is `invalid_year`; a date that is no ISO 8601 date is
`invalid_date`, one in another year `date_outside_year`, a day given twice `duplicate_date`; at
least one value that is not `null` (`empty_series`); `min` below and `max` above the reference
(`invalid_scale`).

## Range bars

`"type": "rangebar"`: one span per category. `orientation`, [Value axis](#value-axis) and
`references` apply. See [Forest plots](../../guides/chart-types/#forest-plots).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `ranges` | array, 1–100 | yes | `[{ "label": "…", "low": 0, "high": 1, "mid": 0.5, "modeled": false, "group": "…", "weight": 12, "summary": false }]`. |
| `ranges[].label` | string, up to 200 characters | yes | The category; unique. |
| `ranges[].low`, `ranges[].high` | number | yes | The ends of the span. |
| `ranges[].mid` | number | no | A central mark between `low` and `high`. |
| `ranges[].modeled` | boolean | no | Draws the span as modelled. |
| `ranges[].group` | string, up to 60 characters | no | Draws the spans of each group in its own palette color, with a legend entry per group. |
| `ranges[].weight` | number above 0 | no | With `mid`: turns the central mark into a square whose area follows it. |
| `ranges[].summary` | boolean | no | With `mid`: draws the span as a diamond, widest at `mid` — together with `references` a forest plot. |

| `display` | `bar`, `dumbbell` | no | Draws each span as a bar (the default) or as a dumbbell: an open dot at `low`, a filled dot at `high` and a line between them, both values written beside their dots. |
| `sort` | `none`, `change` | no | `change` puts the widest span first, in the picture, the table and the description; `none` (default) keeps the order as listed. |

Rules: a dumbbell takes no `mid`, `weight`, `summary` or `modeled` on a range
(`option_not_supported`, with the path of the field), and `display` and `sort` belong to range bars
only (`option_not_supported`); up to 100 ranges (`too_many_data_points`) with unique labels (`duplicate_label`); `low`
above `high` is `invalid_range`, a `mid` outside them `mid_outside_range`; groups on every range or
none (`missing_group`), at most 4 (`too_many_series`); a `weight` of zero or less is
`invalid_weight`, and `weight` or `summary` without `mid` is `invalid_weight` or
`summary_without_mid`. More than 16 ranges warn with `dense_chart`.

## Box plots

`"type": "boxplot"`. `orientation` and [Value axis](#value-axis) apply. See
[Box plots](../../guides/chart-types/#box-plots).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `boxes` | array, 1–100 | yes | `[{ "label": "…", "values": [3, 4, 5, 6, 9] }` or `{ "label": "…", "min": 1, "q1": 2, "median": 3, "q3": 4, "max": 6, "outliers": [9] }]`. |
| `boxes[].label` | string, up to 200 characters | yes | The category; unique. |
| `boxes[].values` | array of numbers, 5–1000 | one of | Observations from which the box is computed: quartiles by linear interpolation, whiskers to the last value within 1.5 × the box, every other value a point. |
| `boxes[].min`, `boxes[].q1`, `boxes[].median`, `boxes[].q3`, `boxes[].max` | number | one of | The five numbers of a box computed elsewhere, in this order. |
| `boxes[].outliers` | array of numbers | no | Points beyond the whiskers of a box given by its five numbers. |
| `boxes[].group` | string, up to 60 characters | no | On every box (up to four groups): colors the boxes of a group alike, with a legend. |
| `boxDisplay` | `box`, `violin`, `strip`, `beeswarm`, `ridgeline` | no | Draws the observations as boxes (the default), as violins, as strips, as a beeswarm (dots moved sideways until none overlap) or as a ridgeline (one density curve per box, stacked with a slight overlap); every display but `box` needs `values` on every box. |
| `showBox` | boolean | no | Beeswarms only: draws the box and its whiskers under the dots. |
| `violinScale` | `width`, `area`, `count` | no | Violins only: `width` (default) makes every violin as wide as the others at its widest, `area` gives them the same area, `count` an area that follows their number of observations. |
| `showMean` | boolean | no | Marks the mean of every box with a diamond and adds a Mean column to the table; needs `values`. |

Rules: 1–100 boxes (`too_many_data_points`) with unique labels (`duplicate_label`); `values` and
the five numbers never both (`conflicting_data_shape`), never part of the five numbers
(`incomplete_box`), the five numbers in order (`invalid_box`), and 5–1000 `values`
(`invalid_values`); violins, strips, beeswarms, ridgelines and `showMean` need `values` on every
box (`values_required`); `showBox` belongs to beeswarms and `violinScale` to violins
(`option_not_supported`); groups on every box or none (`missing_group`), at most 4
(`too_many_series`). More than 16 boxes warn with `dense_chart`.

## Timelines

`"type": "timeline"`: phases, milestones, marker lines and which items follow which. See
[Timelines](../../guides/chart-types/#timelines).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `timeline.items` | array, 1–60 | yes | `[{ "id": "a", "label": "…", "start": "2027-01-04", "end": "2027-02-26", "group": "…", "after": ["b"] }, { "label": "…", "at": "2027-06-15" }]`. |
| `timeline.items[].id` | identifier | no | What `after` and `rollup` name; unique. |
| `timeline.items[].label` | string, up to 100 characters | yes | The row's name. |
| `timeline.items[].start`, `timeline.items[].end` | day | phase | A phase runs from `start` to `end`. |
| `timeline.items[].at` | day | milestone | A milestone lies at `at`. |
| `timeline.items[].progress` | number, 0–1 | no | How much of a phase is done: that share of its bar is drawn darker. |
| `timeline.items[].rollup` | array of ids | no | The ids of the phases this phase sums up: its progress is theirs, each weighted by its length. Instead of `progress`. |
| `timeline.items[].group` | string, up to 60 characters | no | Groups on all items or none, at most 4. |
| `timeline.items[].after` | array of ids | no | The ids of the items this item follows. |
| `timeline.markers` | array, up to 6 | no | `[{ "label": "Today", "at": "2027-02-10" }]`: dashed marker lines for days that matter. |
| `timeline.markers[].label` | string, up to 60 characters | yes | The marker's text. |
| `timeline.markers[].at` | day | yes | Its day. |
| `timeline.groupRows` | boolean | no | Puts the rows of a group together, in the order the groups first appear (needs groups). |
| `timeline.timezone` | string | no | Reads and writes the days in UTC (default), a fixed offset such as `+02:00` or one of about 55 IANA names such as `Europe/Berlin`, with the daylight saving rules in force today. |
| `timeline.weekends` | boolean | no | Shades Saturdays and Sundays on an axis of at most 120 days. |

Days are ISO 8601 dates, optionally with a time, or Unix seconds.

Rules: 1–60 items (`too_many_nodes`) and up to 6 markers (`too_many_markers`); ids are identifiers
(`invalid_id`) and unique (`duplicate_id`); a day that cannot be read is `invalid_time`, an `end`
before its `start` `invalid_range`; an item is a phase or a milestone, never a mix
(`invalid_item`); `progress` outside 0–1 is `invalid_value`, on a milestone (or a milestone with
`rollup`) `progress_on_milestone`, together with `rollup` `conflicting_progress`; a `rollup`
names existing items (`unknown_node`) that are phases summing up no others, not itself
(`invalid_rollup`); `after` names existing items (`unknown_node`) and runs in no circle
(`circular_dependency`), and an item that starts before one it follows ends warns with
`follows_overlap`; groups on all items or none (`missing_group`), at most 4 (`too_many_series`),
and `groupRows` without groups is `option_not_supported`; an unknown `timezone` is
`invalid_timezone`; `weekends` beyond 120 days warns with `weekends_hidden`; more than 30 items
warn with `dense_chart`.

## Parliament charts

`"type": "parliament"`: the seats of an assembly as dots in a semicircle, in blocks by party from
left to right in the order of the list. See
[Parliament charts](../../guides/chart-types/#parliament-charts).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `parliament.parties` | array, 1–8 | yes | `[{ "label": "Red", "seats": 40 }, { "label": "Blue", "seats": 35 }]`. |
| `parliament.parties[].label` | string, up to 100 characters | yes | The party; unique. |
| `parliament.parties[].seats` | integer, 1–800 | yes | Its seats. |
| `parliament.majority` | boolean | no | Marks the seat that makes a majority. |
| `parliament.coalition` | array of labels | no | Labels of parties whose seats are ringed and summed. |

Rules: up to 8 parties (`too_many_series`) with unique labels (`duplicate_label`) and at least
one seat (`invalid_value`), at most 800 seats (`too_many_data_points`); `coalition` names
existing parties (`unknown_node`).

## Treemaps

`"type": "treemap"`: the areas follow the values by the squarified method. See
[Treemaps](../../guides/chart-types/#treemaps).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `treemap.items` | array | yes | `[{ "label": "Schools", "value": 310, "group": "Society" }]`. |
| `items[].label` | string, up to 100 characters | yes | The item; unique, parts included. |
| `items[].value` | number above 0 | leaf | The value of a leaf; an item with `children` has none. |
| `items[].group` | string, up to 60 characters | no | On every item or on none, up to 4 groups: colors the rectangles with a legend. |
| `items[].children` | array | no | The parts of the item, drawn inside its rectangle, nested up to three levels; colored by the item at the top. |

Rules: up to 100 items, parts included (`too_many_data_points`), with unique labels
(`duplicate_label`) and values above zero (`invalid_value`); an item with `children` takes no
`value` (`value_with_children`), nested at most three levels (`too_deep`), and a nested treemap
takes no groups (`group_with_children`); groups on every item or none (`missing_group`), at most 4
(`too_many_series`).

## Sankey diagrams

`"type": "sankey"`. Columns follow the longest path to a node; a band takes the color of the node
it leaves. See [Sankey diagrams](../../guides/chart-types/#sankey-diagrams).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `sankey.links` | array, 1–100 | yes | `[{ "from": "Gas", "to": "Power", "value": 120 }]`. The nodes are the labels the links name, in the order they first appear. |
| `sankey.links[].from`, `sankey.links[].to` | string, up to 100 characters | yes | The nodes the flow leaves and reaches. |
| `sankey.links[].value` | number above 0 | yes | How much flows. |
| `sankey.links[].back` | boolean | no | A flow back to an earlier column, drawn as a loop under the diagram; the columns and their order do not follow it. |
| `sankey.nodes` | array, up to 40 | no | Nodes whose column or color the specification fixes; they come first, in this order. |
| `sankey.nodes[].label` | string, up to 100 characters | yes | A label the links name. |
| `sankey.nodes[].column` | integer, 0–7 | no | Pins the node's column. |
| `sankey.nodes[].color` | integer, 1–4 | no | Pins its palette color. |
| `sankey.order` | `auto`, `listed` | no | `auto` (default) orders the nodes of a column to cross few bands; `listed` keeps the order of `nodes`, then of the links. |

Rules: up to 100 links (`too_many_data_points`) and 40 nodes (`too_many_nodes`); values above zero
(`invalid_value`), no link from a node to itself (`self_link`), one link for each pair
(`duplicate_link`), no cycle (`link_cycle`) except flows marked `"back": true`, which must return
to an earlier column (`link_not_back`); listed nodes have unique labels (`duplicate_label`) that
the links name (`unknown_node`), and a pinned column no earlier than the links into the node allow
(`column_too_early`).

## Survival curves

`"type": "survival"`: Kaplan-Meier estimates. See
[Survival curves](../../guides/chart-types/#survival-curves).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `survival.groups` | array, 1–4 | yes | `[{ "label": "Standard", "observations": [{ "time": 4.2 }, { "time": 9, "event": false }] }]`. |
| `survival.groups[].label` | string, up to 100 characters | yes | The group; unique. |
| `survival.groups[].observations` | array, 2–2000 | yes | The observed times. |
| `survival.groups[].observations[].time` | number, 0 or more | yes | When the event happened, or the observation ended. |
| `survival.groups[].observations[].event` | boolean | no | `false` cuts the observation off (censored), marked on the curve; default `true`. |
| `survival.confidence` | boolean | no | Draws the 95 % band (log-log, Greenwood variance). |
| `survival.atRisk` | boolean | no | Writes the number at risk under the time axis, default `true`. |
| `survival.logRank` | boolean | no | Tests whether the groups differ (log-rank, Mantel-Cox) and writes the statistic and p in the plot and the description. |
| `survival.timeTitle` | string, up to 100 characters | no | Title of the time axis, such as `Months`. |

Rules: 1–4 groups (`too_many_series`) with unique labels (`duplicate_label`) and 2–2000
observations each (`invalid_values`); a time is zero or more (`invalid_value`); `logRank` needs
two or more groups (`log_rank_needs_groups`).

## Scatter plots

`"type": "scatter"`. See [Scatter plots](../../guides/chart-types/#scatter-plots).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `scatter.points` | array, 1–5000 | yes | `[{ "x": 1.2, "y": 3.4, "group": "Up", "label": "Gene A" }]`. From 300 points on the dots are small and have no tooltip; from 100 the data table lists only the labeled points, or the 20 highest. |
| `scatter.points[].x`, `scatter.points[].y` | number | yes | The position. |
| `scatter.points[].group` | string, up to 60 characters | no | On every point or none, up to 4 groups with a legend. |
| `scatter.points[].label` | string, up to 60 characters | no | Writes a name beside the point where there is room. |
| `scatter.points[].size` | number above 0 | no | On every point or none (`missing_size`): the area of the point is proportional to it, with a size legend of three reference circles; adds a column to the table. |
| `scatter.sizeTitle` | string, up to 60 characters | no | What the sizes are, named in the size legend, tooltips and table; default the word for a size. Needs sizes on the points (`option_not_supported`). |
| `scatter.lines` | array, up to 8 | no | `[{ "axis": "y", "value": 1.3, "label": "p = 0.05" }]`: lines across the plot. |
| `scatter.lines[].axis` | `x`, `y` | yes | The axis the value lies on. |
| `scatter.lines[].value` | number | yes | Where the line stands. |
| `scatter.lines[].label` | string, up to 100 characters | no | Its text. |
| `scatter.xTitle`, `scatter.yTitle` | string, up to 100 characters | no | Axis titles. |
| `scatter.xRange`, `scatter.yRange` | `[min, max]` | no | Fix the ends of an axis instead of ends rounded outward to a tick; everything drawn along it must lie within. |
| `scatter.xTicks` | array, up to 40 | no | `[{ "at": 1, "label": "chr1" }]`: named labels at chosen places on the x axis instead of numbered ticks; a label that would run into the one before it is left out. |
| `scatter.xTicks[].at` | number | yes | The position. |
| `scatter.xTicks[].label` | string, up to 40 characters | yes | The name. |
| `scatter.quadrants` | object | no | `{ "x": 5, "y": 5, "labels": { "topLeft": "Quick wins" } }` divides the plot into four areas by a line at `x` and one at `y`, and says in the description and the table in which area each point lies. |
| `scatter.quadrants.x`, `scatter.quadrants.y` | number | yes | The dividing lines. |
| `scatter.quadrants.labels.topLeft`, `.topRight`, `.bottomLeft`, `.bottomRight` | string, up to 40 characters | no | The name of each area, each optional; top is the side of the larger y, right the side of the larger x. |
| `scatter.regression` | boolean | no | Draws the least-squares line for each group; on a logarithmic axis it is fitted to the logarithms. |
| `scatter.xScale`, `scatter.yScale` | `linear`, `log` | no | `log` needs values above zero. |

Rules: 1–5000 points (`too_many_data_points`) with finite numbers; groups on every point or none
(`missing_group`), at most 4 (`too_many_series`); more than 8 lines or 40 tick labels are
`too_many_references`; with a logarithmic scale every value is above zero (`invalid_value`); a
range whose ends are not in order is `invalid_range`, and what lies outside a fixed range —
points, lines, ticks, the dividing lines of `quadrants` — is `out_of_range`. A dividing line on the
end of an axis, or an area too small for its name, warns with `quadrant_without_room`.

## Waffle charts

`"type": "waffle"`: a grid of squares that each stand for the same share of a whole, filled by the
parts in reading order. See [Waffle charts](../../guides/chart-types/#waffle-charts).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `waffle.parts` | array, 1–4 | yes | `[{ "label": "Wind", "value": 31 }, { "label": "Gas", "value": 28 }]`; each part gets the share of the squares by the largest remainder, and at least one. |
| `waffle.parts[].label` | string, up to 100 characters | yes | The part; unique. |
| `waffle.parts[].value` | number above 0 | yes | Its amount. |
| `waffle.cells` | integer, 10–400 | no | The number of squares, default 100. |
| `waffle.columns` | integer, 2–40 | no | Squares per row; default about square. |
| `waffle.total` | number above 0 | no | A total above the sum of the parts leaves squares for the rest. |

Rules: up to 4 parts (`too_many_series`) with unique labels (`duplicate_label`) and values above
zero (`invalid_value`); `cells` outside 10–400, or no more cells than parts, is
`invalid_cells`; `columns` outside 2–40 is `invalid_columns`; a `total` below the sum of the parts
is `invalid_total`.

## Funnels and pyramids

`"type": "funnel"`: stages that narrow as centered bars, or ranked levels as a pyramid. See
[Funnels and pyramids](../../guides/chart-types/#funnels-and-pyramids).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `funnel.stages` | array, 2–12 | yes | `[{ "label": "Visits", "value": 12400 }]`, first stage first (at the top). |
| `funnel.stages[].label` | string, up to 100 characters | yes | The stage; unique. |
| `funnel.stages[].value` | number above 0 | funnel: yes; pyramid: all or none | Its amount. |
| `funnel.display` | `"funnel"` or `"pyramid"` | no | Default `funnel`: bars as wide as the values, with the rate between two stages. `pyramid`: levels of a triangle, the widest at the bottom. |

Rules: labels are unique (`duplicate_label`); a value is above zero, on every stage of a funnel and
on all or none of the levels of a pyramid (`invalid_value`); fewer than 2 stages is `empty_data`,
more than 12 is `too_many_data_points`. A funnel whose stage is larger than the one before it is
drawn and reported with the warning `stage_exceeds_previous`.

## Radar charts

`"type": "radar"`: one axis per measure, one closed polygon per series. See
[Radar charts](../../guides/chart-types/#radar-charts).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `radar.axes` | array, 3–16 | yes | `[{ "label": "Speed", "max": 100 }]` |
| `radar.axes[].label` | string, up to 100 characters | yes | The axis; unique. |
| `radar.axes[].min` | number | no | The value at the center; default 0. |
| `radar.axes[].max` | number | no | The value at the end of the axis; default the next step of four grid rings above the largest value. Must be greater than `min`. |
| `radar.series` | array, 1–4 | yes | `[{ "name": "Engine A", "values": [92, 58, 74] }]` |
| `radar.series[].name` | string, up to 100 characters | yes | The series; unique. |
| `radar.series[].values` | array of numbers | yes | One value for every axis, in the order of `axes`, within the range of its axis. |

Rules: fewer than 3 axes or no series is `empty_data`, more than 16 axes `too_many_data_points`,
more than 4 series `too_many_series`; labels and names are unique (`duplicate_label`,
`duplicate_series`); a series has one value per axis (`series_length_mismatch`) inside the range of
its axis (`value_outside_axis`); `max` above `min` (`invalid_axis_range`). More than 8 axes or more
than 3 series is drawn and reported with the warning `dense_chart`.

## Polar charts

`"type": "polar"`: one value per category of a cycle, as wedges or lollipops around a center. See
[Polar charts](../../guides/chart-types/#polar-charts).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `polar.points` | array, 3–60 | yes | `[{ "label": "Jan", "value": 52 }]`, the first at the top, the others clockwise. |
| `polar.points[].label` | string, up to 100 characters | yes | The category; unique. |
| `polar.points[].value` | number, 0 or more | yes | Its value; zero is the center. |
| `polar.display` | `"bars"` or `"lollipops"` | no | Default `bars`. |
| `polar.max` | number above 0 | no | The value at the outer ring; default the next step of four grid rings above the largest value. |
| `polar.valueLabel` | string | no | What the values are, named in the table; default the word for a value. |

Rules: fewer than 3 points is `empty_data`, more than 60 `too_many_data_points`; labels are unique
(`duplicate_label`); a value is not negative (`invalid_value`) and not above `max`
(`value_outside_axis`); `max` is above zero (`invalid_axis_range`). Category names that do not fit
around the chart are left out and reported with the warning `labels_omitted`.

## Mosaic charts

`"type": "mosaic"`: columns as wide as their share of the whole, split by the share of their
segments. See [Mosaic charts](../../guides/chart-types/#mosaic-charts).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `mosaic.columns` | array, 1–12 | yes | `[{ "label": "Europe", "segments": [{ "label": "Hardware", "value": 120 }] }]` |
| `mosaic.columns[].label` | string, up to 100 characters | yes | The column; unique. |
| `mosaic.columns[].segments` | array | yes | At least one segment per column. |
| `mosaic.columns[].segments[].label` | string, up to 100 characters | yes | The segment; unique within its column. Segments of one label share a color across columns. |
| `mosaic.columns[].segments[].value` | number above 0 | yes | Its amount. |

Rules: no column is `empty_data`, more than 12 `too_many_data_points`; a column without segments is
`empty_data`; labels are unique within their level (`duplicate_label`); values are above zero
(`invalid_value`); more than 4 different segment labels in all is `too_many_series`.

## Heatmaps

`"type": "heatmap"`: values, or categories, in a grid of rows and columns. See
[Heatmaps](../../guides/chart-types/#heatmaps).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `heatmap.rows` | array of strings, 1–60 | yes | The rows; unique labels. |
| `heatmap.columns` | array of strings, 1–60 | yes | The columns; unique labels. |
| `heatmap.cells` | array | yes | `[{ "row": "Mon", "column": "AM", "value": 42 }]`; a cell that is not listed has no value. With `categories`, `category` takes the place of `value`. |
| `heatmap.categories` | array of strings, up to 4 | no | Makes the heatmap categorical: each category has a palette color and a symbol. |
| `heatmap.scale` | `"sequential"` or `"diverging"` | no | Default `sequential`: six steps from the smallest value to the largest. `diverging` has seven, parted at the midpoint. |
| `heatmap.midpoint` | number | no | Where a diverging scale turns; default 0. |
| `heatmap.valueLabel` | string | no | What the values are, named in the key and the table; default the word for a value. |
| `heatmap.rowOrder`, `heatmap.columnOrder` | `"listed"`, `"label"` or `"value"` | no | The order of the rows or columns: as listed (default), by label, or by the mean of their values, the largest first. |

Rules: a cell names a row and a column that exist (`unknown_row`, `unknown_column`) once
(`duplicate_cell`), and holds a `value` or, with `categories`, a `category` of the list
(`invalid_cell`, `unknown_category`); labels are unique (`duplicate_label`); up to 4 categories
(`too_many_series`); a categorical heatmap takes no `scale`, `midpoint` or `valueLabel`
(`option_not_supported`) and no ordering by `value` (`invalid_order`); a `midpoint` belongs to a
diverging scale.

## Waterfalls

`"type": "waterfall"`: a running total that rises and falls step by step. `orientation` and
[Value axis](#value-axis) apply. See [Waterfalls](../../guides/chart-types/#waterfalls).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `waterfall.steps` | array, 2–40 | yes | `[{ "label": "Revenue", "value": 1250, "kind": "start" }, { "label": "Costs", "value": -420 }, { "label": "Margin", "kind": "total" }]`. |
| `waterfall.steps[].label` | string, up to 100 characters | yes | The step; unique. |
| `waterfall.steps[].value` | number | depends | The change, or the start. |
| `waterfall.steps[].kind` | `delta`, `start`, `total` | no | A `delta` (default) adds its value, a `start` sets the total as a bar from zero, a `total` shows the running total and takes no value, or the matching one. |

Rules: 2–40 steps (`too_many_data_points`) with unique labels (`duplicate_label`); a `delta` or
`start` without value is `missing_value`; a `total` with a value other than the running total is
`total_mismatch`.

## Fishbone diagrams

`"type": "fishbone"`: the causes of one effect on bones. See
[Fishbone diagrams](../../guides/chart-types/#fishbone-diagrams).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `fishbone.effect` | string, up to 100 characters | yes | The effect at the head of the spine. |
| `fishbone.bones` | array, 1–6 | yes | The categories of causes; they alternate above and below the spine. |
| `fishbone.bones[].label` | string, up to 100 characters | yes | The category; unique. |
| `fishbone.bones[].causes` | array, up to 8 | no | The causes on the bone. |
| `fishbone.bones[].causes[].label` | string, up to 100 characters | yes | The cause. |
| `fishbone.bones[].causes[].causes` | array, up to 4 | no | Sub-causes, `[{ "label": "No route training" }]`; the last level. |

Rules: 1–6 bones, up to 8 causes per bone and 4 sub-causes per cause (`too_many_nodes`); bone
labels are unique (`duplicate_label`); a cause below a sub-cause is not part of the specification
and is refused as an unknown field. The canvas grows for the bones with a `canvas_too_small`
warning.

## Kanban boards

`"type": "kanban"`: columns of cards. See [Kanban boards](../../guides/chart-types/#kanban-boards).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `kanban.columns` | array, 1–6 | yes | The states, left to right. |
| `kanban.columns[].label` | string, up to 60 characters | yes | The state; unique. |
| `kanban.columns[].limit` | integer, 1 or more | no | The most cards the column should hold; written as `3/4`, marked when exceeded. |
| `kanban.columns[].cards` | array, up to 20 | no | The cards, top to bottom. |
| `kanban.columns[].cards[].title` | string, up to 100 characters | yes | The card. |
| `kanban.columns[].cards[].owner` | string, up to 40 characters | no | Who holds it. |
| `kanban.columns[].cards[].tag` | string, up to 30 characters | no | A kind of work, drawn as a chip. |
| `kanban.columns[].cards[].blocked` | boolean | no | Default `false`; drawn with a sign and the word. |

Rules: 1–6 columns and up to 20 cards per column (`too_many_nodes`); column labels are unique
(`duplicate_label`); a `limit` of 0 is `invalid_limit`. A column over its limit is drawn, not
refused. The canvas grows for long boards with a `canvas_too_small` warning.

## Story maps

`"type": "storymap"`: activities, tasks and stories in release slices. See
[Story maps](../../guides/chart-types/#story-maps).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `storymap.releases` | array of strings, 1–5 | yes | The slices from the top down, such as `["MVP", "Release 2"]`; unique. |
| `storymap.activities` | array, 1–8 | yes | The backbone, left to right; unique labels. |
| `storymap.activities[].label` | string, up to 100 characters | yes | The activity. |
| `storymap.activities[].tasks` | array, at least 1 | yes | The user tasks under it; 16 in all. |
| `storymap.activities[].tasks[].label` | string, up to 100 characters | yes | The task. |
| `storymap.activities[].tasks[].stories` | array, up to 12 | no | `[{ "title": "Search by name", "release": "MVP" }]`. |

Rules: up to 5 releases, 8 activities and 16 tasks (`too_many_nodes`); an activity needs a task
(`empty_data`); release names and activity labels are unique (`duplicate_label`); a story names a
release of the map (`unknown_release`).

## User journeys and service blueprints

`"type": "journey"`: stages with rows of actions, touchpoints, thoughts and sentiment. See
[User journeys and service blueprints](../../guides/chart-types/#user-journeys-and-service-blueprints).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `journey.stages` | array, 2–8 | yes | The stages, left to right. |
| `journey.stages[].label` | string, up to 60 characters | yes | The stage; unique. |
| `journey.stages[].action`, `touchpoint`, `thought` | string, up to 160 characters | no | What the customer does, where, and thinks. |
| `journey.stages[].sentiment` | integer, -2 to 2 | no | How the customer feels, drawn as a line with the value written. |
| `journey.display` | `"journey"` or `"blueprint"` | no | Default `journey`; `blueprint` allows the three rows below and draws the lines of interaction and visibility. |
| `journey.stages[].frontstage`, `backstage`, `support` | string, up to 160 characters | no | What the service does where the customer sees it, out of sight, and the systems under it. Blueprint only. |

Rules: 2–8 stages (`empty_data`, `too_many_nodes`) with unique labels (`duplicate_label`); a
sentiment outside -2…2 is `invalid_value`; `frontstage`, `backstage` and `support` without
`"display": "blueprint"` are `option_not_supported`. The canvas grows for long texts with a
`canvas_too_small` warning.

## Value chain maps

`"type": "valuemap"`: components by visibility and evolution. See
[Value chain maps](../../guides/chart-types/#value-chain-maps).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `valuemap.components` | array, 1–40 | yes | The components of the map. |
| `valuemap.components[].id` | string, up to 60 characters | yes | Identifies it in `dependencies`; unique. |
| `valuemap.components[].label` | string, up to 100 characters | yes | The name drawn beside the dot; unique. |
| `valuemap.components[].visibility` | number, 0–1 | yes | 1 is the user's need itself, 0 is hidden; drawn up. |
| `valuemap.components[].evolution` | number, 0–1 | yes | 0 is genesis, 1 commodity; drawn right. Stages begin at 0, 0.17, 0.4 and 0.7. |
| `valuemap.components[].movesTo` | object | no | `{ "evolution": 0.6, "visibility": 0.5 }`; a dashed arrow to the place; `visibility` defaults to the component's own. |
| `valuemap.components[].inertia` | boolean | no | Default `false`; a bar beside the dot. |
| `valuemap.dependencies` | array, up to 80 | no | `[{ "from": "shop", "to": "hosting" }]`: `from` needs `to`. |

Rules: 1–40 components (`too_many_nodes`) with unique ids (`duplicate_id`) and labels
(`duplicate_label`); positions between 0 and 1 and a movement to more evolution
(`invalid_position`); dependencies name components (`unknown_node`), none to itself
(`self_loop`) and each once (`duplicate_edge`).

## Venn diagrams

`"type": "venn"`: two or three sets and the regions where they overlap. See
[Venn diagrams](../../guides/chart-types/#venn-diagrams).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `venn.sets` | array of strings, 2–3 | yes | The names of the sets; unique, up to 60 characters. |
| `venn.regions` | array, up to 7 | no | `[{ "sets": ["Design", "Code"], "label": "Prototyping", "count": 4 }]`; the region that lies in exactly the sets it names. |
| `venn.regions[].sets` | array of set names | yes | One to three of `venn.sets`, each once. |
| `venn.regions[].label` | string, up to 100 characters | no | Written in the region, or beside the diagram on a leader line. |
| `venn.regions[].count` | number, 0 or more | no | Written under the label; formatted like other numbers (`locale`). |

Rules: fewer than two sets is `not_enough_sets`, more than three `too_many_sets`; set names are
unique (`duplicate_label`); a region names sets that exist (`unknown_set`) and is given once
(`duplicate_region`); a negative `count` is `invalid_value`. The table lists every region, also
those that are not given.

## Diagram fields

Sequence, flow, state, class, architecture, tree and network diagrams set their orientation inside
their block; the top-level `orientation` is refused (see [Common fields](#common-fields)).

An identifier — `id`, and every field that names one — is a letter followed by up to 63 letters,
digits, hyphens or underscores (`invalid_id`).

| Field | Type | Diagrams | Meaning |
| --- | --- | --- | --- |
| `orientation` | `auto`, `portrait`, `landscape` | sequence, flow, state, classes, architecture, tree | `auto` (default) takes portrait where that fits the canvas, landscape where only that fits, and otherwise the one that grows the canvas less. |
| `spacing` | `compact`, `normal`, `roomy` | flow, state, classes, architecture | `compact`, `normal` (default) or `roomy` scales the gaps between layers and between neighbours in a layer — never steps, texts, margins or the distance between parallel lines; see [Spacing](../../guides/chart-types/#spacing). |
| `mainPath` | array of ids, or `{ "from", "to" }` | flow, state (array only), architecture | The usual way through the diagram, as ids joined by edges, or the shortest way along the edges between `from` and `to` (at a tie the first in the order of the edges). Kept in line and drawn stronger. |
| `sameRank` | array of id sets | flow, architecture | Sets of at least two ids that lie at one position along the flow; a layout hint that adds no edge. |
| `fromSide`, `toSide` | `start`, `end`, `left`, `right` | flow edges, architecture connections | Pin the side of a node an edge leaves and reaches, named after the flow so that they hold in either orientation: `start` faces the nodes before, `end` the nodes after, `left` and `right` lie across the flow (left and right in portrait, top and bottom in landscape); see [Port sides](../../guides/chart-types/#port-sides). An edge with a side never joins a shared trunk. |
| `notes` | array, up to 12 | flow, classes, architecture | `[{ "on": "a", "text": "…" }, { "text": "…" }]`. |
| `notes[].on` | identifier | — | The step, class, package, component or boundary the note explains; flow charts take notes on steps only. |
| `notes[].text` | string, up to 160 characters | — | Required; wraps onto as many lines as it needs. |
| `role` | `critical`, `planned`, `deprecated` | flow, state, classes, architecture, tree, network, cycle, layers | What a node or an edge means beyond its kind, drawn alike in every diagram. |

**Notes.** A note on an element is drawn beside it and joined to it by a dotted line that is no
edge, relation or connection; a note on a package or boundary stands inside that frame, at the
head of its contents, without a line; a note without `on` explains the whole diagram and stands in
a band below it, notes side by side and wrapping onto further rows. The description gives every
note (`Note on Card: …`, `Note: …`); the table gives notes on elements in a Notes column of their
rows and the others, which belong to no row, after its caption. More than 12 notes are
`too_many_notes`, a longer text `text_too_long`, an `on` that names nothing `unknown_node` at
`/…/notes/0/on`.

**Roles.** On a step, component, state, node or class, and on an edge, connection, transition or
relation. `critical`: a 3 px red outline or a 2.6 px red line and a badge with `!`; `planned`:
dashed and violet, badge `+`; `deprecated`: dotted and gray, badge `−`. The badge sits on the top
left corner of a box. Colors are the theme's role colors (`--chartlet-role-red`, `-violet`,
`-gray`), so light, dark, blueprint and print each resolve them. A role keeps the shape and fill of
the kind (and a `value`'s fill); it overrides the color of the main path, which keeps its place in
the layout, while a `weight` keeps its line width; `planned` and `deprecated` have a pattern of
their own, so an edge, connection or transition with one of them takes no `dash`
(`role_conflict`); `critical` keeps a `dash`. A legend below the diagram names only the roles used;
the description lists what has each role, and the table adds a Role column when a step or
component has one and names the role of an edge in its notes. A state diagram's table, one row per
transition, adds a Role column for transitions and names a state with its role,
`Locked (critical)`; a tree's and a network's table add a Role column for nodes, a class diagram's
for classes, and name the role of a relation or a network edge in its notes. A network node is a
dot too small for a badge: a ring around it in the role's color and pattern says the role, and the
dot keeps the color of its group. A composite state is a frame and takes no role
(`option_not_supported`). The marks at the ends of a relation keep their form, a hollow mark stays
hollow; a realization or a dependency is dashed by its kind, so it takes `critical` only
(`role_conflict` at its `role`). An unknown role is `invalid_spec`.

**Layout.** A sequence diagram or flow chart — and every diagram laid out like one — that does not
fit its canvas is drawn larger and reported as `canvas_too_small` at `/width` or `/height`. A
`fromSide` or `toSide` along the flow that would turn the edge back over its step, such as `start`
on an edge that leads on to a later step, is ignored and reported as `port_side_ignored` at that
field (flow charts and architecture diagrams); an edge from a node to itself takes no `fromSide`
or `toSide` (`option_not_supported`). A main path must follow edges (`main_path_gap`) or, given
as `{ "from": "a", "to": "b" }`, have a way along them (`main_path_unreachable`). In a `sameRank`
set, each id appears in one set at most, and no two of a set are joined by an edge
(`same_rank_edge`).

## Sequence diagrams

`"type": "sequence"`. See [Sequence diagrams](../../guides/chart-types/#sequence-diagrams).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `sequence.participants` | array, 1–12 | yes | `[{ "id": "api", "label": "…", "sublabel": "…", "kind": "service" }]`. |
| `participants[].id` | identifier | yes | Unique. |
| `participants[].label`, `participants[].sublabel` | string, up to 60 characters | label | Name and second line. |
| `participants[].kind` | `service`, `actor`, `database`, `queue`, `external` | no | Each kind has its own shape; default `service`. |
| `sequence.messages` | array, 1–60 | yes | `[{ "from": "api", "to": "db", "label": "…", "kind": "call" }]`, in the order they are sent. |
| `messages[].from`, `messages[].to` | identifier | yes | Sender and receiver; a participant may send to itself. |
| `messages[].label` | string, up to 80 characters | yes | The message. |
| `messages[].kind` | `call`, `reply`, `async` | no | Default `call`; a call activates its receiver until the receiver replies. |
| `sequence.fragments` | array, up to 12 | no | `[{ "kind": "alt", "label": "…", "from": 0, "to": 2, "else": [{ "from": 1, "label": "…" }] }]`: frames around runs of consecutive messages. |
| `fragments[].kind` | `alt`, `opt`, `loop`, `par`, `critical`, `break` | yes | The kind of frame. |
| `fragments[].from`, `fragments[].to` | integer, from 0 | yes | The first and last message inside, counted from 0 and included. |
| `fragments[].label` | string, up to 60 characters | no | Its condition. |
| `fragments[].else` | array | no | Only `alt`: further branches, each starting at a later message inside the frame (`from`) with an optional `label`. |
| `sequence.numbered` | boolean | no | Numbers the messages in round badges at the start of their arrows. |
| `sequence.orientation` | `auto`, `portrait`, `landscape` | no | See [Diagram fields](#diagram-fields). |

Rules: 1–12 participants (`too_many_participants`), 1–60 messages (`too_many_messages`) and up to
12 fragments (`too_many_fragments`); participant `id`s are identifiers (`invalid_id`) and unique
(`duplicate_id`), every message names existing participants (`unknown_participant`), and
fragments lie within the messages, nest at most three deep and never cross (`invalid_fragment`,
`fragments_cross`).

## Flow charts

`"type": "flow"`: steps laid out in layers along the flow, down in portrait and right in landscape.
Edges may form cycles; an edge back is drawn against the flow. See
[Flow charts](../../guides/chart-types/#flow-charts).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `flow.nodes` | array, 1–40 | yes | The steps: `[{ "id": "a", "label": "…", "sublabel": "…", "kind": "process", "lane": "x", "value": 3, "parts": [{ "label": "…", "value": 2 }], "role": "critical" }]`. |
| `nodes[].id` | identifier | yes | Unique among steps and lanes. |
| `nodes[].label`, `nodes[].sublabel` | string, up to 60 characters | label | Name and second line. |
| `nodes[].kind` | `process`, `start`, `end`, `decision`, `io`, `subprocess`, `store`, `external` | no | Each kind has its own shape; default `process`. |
| `nodes[].lane` | identifier | with lanes | The lane the step lies in. |
| `nodes[].value` | number | no | Colors the step's fill, from light for the smallest value to dark for the largest. |
| `nodes[].parts` | array, up to 6 | no | `[{ "label", "value" }]` (label up to 40 characters, value 0 or more): the parts of what the step holds, drawn as a stripe inside the bottom of its box. |
| `nodes[].role` | role | no | See [Diagram fields](#diagram-fields). |
| `flow.edges` | array, up to 80 | no | `[{ "from": "a", "to": "b", "label": "…", "dash": "solid", "weight": 120, "role": "planned", "fromSide": "right", "toSide": "start" }]`. |
| `edges[].from`, `edges[].to` | identifier | yes | The steps it joins. |
| `edges[].label` | string, up to 60 characters | no | Its text. |
| `edges[].dash` | `solid`, `dashed`, `dotted` | no | Default `solid`. |
| `edges[].weight` | number, 0 or more | no | How much the edge carries: the line is drawn from 1 px for nothing to 5 px for the heaviest edge. |
| `edges[].role`, `edges[].fromSide`, `edges[].toSide` | | no | See [Diagram fields](#diagram-fields). |
| `flow.lanes` | array, up to 8 | no | `[{ "id": "x", "label": "…" }]` (label up to 40 characters): bands across the flow; once there are lanes, every step names one. |
| `flow.groups` | array, up to 8 | no | `[{ "label": "…", "nodes": ["a"] }]` (label up to 40 characters): frames around steps of one lane. |
| `flow.mainPath`, `flow.sameRank`, `flow.notes`, `flow.orientation`, `flow.spacing` | | no | See [Diagram fields](#diagram-fields). |
| `flow.valueLabel` | string, up to 40 characters | no | What the `value` of the steps stands for: the name of the color key and of the table column. |
| `flow.sharedTrunks` | boolean | no | Draws edges that leave a step together, or reach one together, on one line that branches, where they read the same (no label, same `dash`, `weight` and `role`, off the main path, other ends in the same lane and group, not out of a decision; note lines never). Every edge is still drawn and listed. |

Rules: 1–40 steps (`too_many_nodes`), up to 80 edges (`too_many_edges`), 8 lanes
(`too_many_lanes`) and 8 groups (`too_many_groups`); step and lane `id`s are identifiers
(`invalid_id`) and unique (`duplicate_id`); edges, groups and the main path name existing steps
(`unknown_node`); every step names a declared lane once there are lanes (`missing_lane`,
`unknown_lane`); a step belongs to one group at most (`duplicate_member`) and a group to one lane
(`group_spans_lanes`); a `weight` is at least 0 (`invalid_edge_weight`); `parts` (flow charts and
trees) has at most 6 entries per node and 8 names per diagram (`too_many_parts`) with values of
at least 0 that add up to more than 0 (`invalid_value`) and stays off decisions
(`option_not_supported`). A step that lands inside the frame of a group it does not belong to is
`group_overlap`. The rules in [Diagram fields](#diagram-fields) apply.

## State diagrams

`"type": "state"`: laid out like a flow chart. Transitions are written `event [guard] / action`.
See [State diagrams](../../guides/chart-types/#state-diagrams).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `state.states` | array, 1–40 | yes | `[{ "id": "a", "label": "…", "sublabel": "…", "kind": "state", "final": false, "in": "c", "role": "critical" }]`. |
| `states[].id` | identifier | yes | Unique. |
| `states[].label`, `states[].sublabel` | string, up to 60 characters | label | Name and second line. |
| `states[].kind` | `state`, `choice`, `composite` | no | `state` (default); a `choice` is a diamond; a `composite` is a frame around the states that name it with `in`. |
| `states[].final` | boolean | no | A final state, drawn with a double outline. |
| `states[].in` | identifier | no | The composite state it lies in. |
| `states[].role` | role | no | Not on a composite state. |
| `state.transitions` | array, up to 80 | no | `[{ "from": "a", "to": "b", "event": "…", "guard": "…", "action": "…", "dash": "solid", "role": "planned" }]`. |
| `transitions[].from`, `transitions[].to` | identifier | yes | The states it joins. |
| `transitions[].event`, `transitions[].guard`, `transitions[].action` | string, up to 40 characters | no | The parts of its label. |
| `transitions[].dash` | `solid`, `dashed`, `dotted` | no | Default `solid`. |
| `transitions[].role` | role | no | See [Diagram fields](#diagram-fields). |
| `state.initial` | identifier | no | The state the machine starts in: a dot with an arrow into it. |
| `state.mainPath` | array of ids | no | The usual way through the states, joined by transitions. |
| `state.orientation`, `state.spacing` | | no | See [Diagram fields](#diagram-fields). |

Rules: 1–40 states (`too_many_states`) and up to 80 transitions (`too_many_transitions`); ids are
identifiers (`invalid_id`) and unique (`duplicate_id`); transitions, `initial` and the main path
name existing states (`unknown_state`) and follow transitions (`main_path_gap`); a choice or
composite state cannot be final (`option_not_supported`); states lie in composite states
(`not_composite`) without a circle (`state_cycle`) at most four deep (`states_too_deep`), a
composite state holds a state (`empty_composite`) and is never named by a transition, `initial` or
the main path (`composite_not_allowed`); with an `initial` state every other state needs a
transition into it (`unreachable_state`, a warning).

## Class diagrams

`"type": "classes"`: a class or entity-relationship diagram, laid out like a flow chart. See
[Class and entity diagrams](../../guides/chart-types/#class-and-entity-diagrams).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `classes.classes` | array, 1–40 | yes | `[{ "id": "order", "label": "Order", "kind": "class", "attributes": ["+ id: Int"], "methods": ["+ total(): Money"], "role": "critical" }]`. |
| `classes[].id` | identifier | yes | Unique among classes and packages. |
| `classes[].label` | string, up to 60 characters | yes | The name. |
| `classes[].kind` | `class`, `interface`, `abstract`, `enum`, `entity` | no | `class` (default), `interface`, `abstract`, `enum` (a stereotype above the name) or `entity` (an entity of an entity-relationship diagram). |
| `classes[].attributes`, `classes[].methods` | array of strings, up to 30, each up to 100 characters | no | Lines of text in their own compartments. |
| `classes[].in` | identifier | no | The package the class lies in; classes outside every package are allowed. |
| `classes[].role` | role | no | See [Diagram fields](#diagram-fields). |
| `classes.relations` | array, up to 100 | no | `[{ "from": "order", "to": "line", "kind": "composition", "fromCardinality": "1", "toCardinality": "1..*", "label": "has", "role": "planned" }]`. |
| `relations[].from`, `relations[].to` | identifier | yes | The classes it joins; relations join classes, never packages. |
| `relations[].kind` | `association`, `inheritance`, `realization`, `composition`, `aggregation`, `dependency` | no | `association` (default, a plain line), `inheritance` (hollow triangle at `to`), `realization` (dashed, hollow triangle at `to`), `composition` (filled diamond at `from`), `aggregation` (hollow diamond at `from`) or `dependency` (dashed, open arrowhead at `to`). |
| `relations[].label` | string, up to 60 characters | no | Its text. |
| `relations[].fromCardinality`, `relations[].toCardinality` | string, up to 12 characters | no | Stand at the ends, such as `0..*`. |
| `relations[].direction` | `forward`, `both`, `none` | no | Associations only: `forward` (navigable to `to`, an open arrowhead there), `both` (open arrowheads at either end) or `none` (a plain line). |
| `relations[].role` | role | no | See [Diagram fields](#diagram-fields). |
| `classes.packages` | array, up to 12 | no | `[{ "id": "ordering", "label": "Ordering", "in": "shop", "orientation": "landscape" }]`: frames that group classes into modules. |
| `packages[].id` | identifier | yes | Unique among classes and packages. |
| `packages[].label` | string, up to 60 characters | yes | The name. |
| `packages[].in` | identifier | no | The package it lies in. |
| `packages[].orientation` | `portrait`, `landscape` | no | Lays out the package's classes that way as one block, whichever way the diagram runs; relations from outside run on straight to their class. |
| `classes.notes`, `classes.orientation`, `classes.spacing` | | no | See [Diagram fields](#diagram-fields). |

The description names every package with what it holds, and the table gains a Package column
(`Shop › Ordering`).

Rules: up to 40 classes (`too_many_nodes`) with unique ids (`duplicate_id`, `invalid_id`), at most
30 attributes and 30 methods each (`too_many_data_points`), and 100 relations (`too_many_edges`)
between existing classes (`unknown_node`, also for a package); `direction` on another kind than an
association is `option_not_supported`. Packages: up to 12 (`too_many_packages`) nested at most
four deep (`packages_too_deep`), classes and packages lie in existing packages
(`unknown_package`) without a circle (`package_cycle`), and every package holds a class
(`empty_package`). `auto` as a package's `orientation`, and a package with one inside another with
one, are `option_not_supported`; a package whose block a relation from outside would have to cross
follows the diagram (`package_orientation_ignored`, a warning).

## Architecture diagrams

`"type": "architecture"`: components joined by connections inside boundaries, laid out like a flow
chart. See [Architecture diagrams](../../guides/chart-types/#architecture-diagrams).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `architecture.components` | array, 1–40 | yes | `[{ "id": "a", "label": "…", "sublabel": "…", "kind": "service", "in": "net", "role": "critical", "tags": ["public"], "provides": [{ "id": "api", "label": "…" }] }]`. |
| `components[].id` | identifier | yes | Unique among components and boundaries. |
| `components[].label`, `components[].sublabel` | string, up to 60 characters | label | Name and second line, such as its technology or host. |
| `components[].kind` | `person`, `frontend`, `service`, `database`, `queue`, `storage`, `cache`, `security`, `external`, `node`, `artifact` | no | Each with its own shape; default `service`. `node` and `artifact` are for deployments. |
| `components[].symbol` | `browser`, `mobile`, `api-gateway`, `load-balancer`, `cdn`, `firewall`, `identity`, `container`, `orchestrator`, `function`, `cloud`, `cache`, `object-storage`, `message-broker`, `search-index`, `monitoring` | no | Draws a generic symbol above the label and names a technology category in the description and the data table, after the kind; the kind keeps its shape. |
| `components[].in` | identifier | no | The boundary it lies in. |
| `components[].role` | role | no | See [Diagram fields](#diagram-fields). |
| `components[].tags` | array of identifiers, up to 8 | no | Names of groups the component belongs to, such as `public`, by which views include or exclude it; not drawn. |
| `components[].provides` | array, up to 4 | no | Interfaces the component provides, each a ball on a stem ahead of its box with the label beside it. Not on `person` or `node`. |
| `provides[].id` | identifier | yes | Unique among the interfaces of the component; with the component's id it forms the stable id `component.interface`. |
| `provides[].label` | string, up to 40 characters | yes | Its name. |
| `architecture.connections` | array, up to 80 | no | `[{ "id": "c1", "from": "a", "to": "b", "label": "…", "technology": "…", "dash": "solid", "weight": 120, "role": "planned", "direction": "forward", "toInterface": "api", "fromSide": "left", "toSide": "start", "tags": ["sync"] }]`. |
| `connections[].id` | identifier | no | A stable name by which views and perspectives select the connection; unique among connections. |
| `connections[].from`, `connections[].to` | identifier | yes | The components it joins. |
| `connections[].label` | string, up to 60 characters | no | What it does. |
| `connections[].technology` | string, up to 40 characters | no | How, written in brackets on a line below the label. |
| `connections[].step` | integer, 1–99 | no | Numbers the connection in a sequence of calls; the number is written before the label, and the description lists the steps in order. |
| `connections[].dash` | `solid`, `dashed`, `dotted` | no | Default `solid`. |
| `connections[].weight` | number, 0 or more | no | How much it carries: the line is drawn from 1 px for nothing to 5 px for the heaviest. |
| `connections[].direction` | `forward`, `both`, `none` | no | `forward` (default, an arrowhead at `to`), `both` (one connection both ways, arrowheads at either end), `none` (no direction, no arrowhead). |
| `connections[].toInterface` | identifier | no | An interface `to` provides, which `from` requires: the connection ends in a socket around the ball, runs forward, takes no `toSide` and never joins a shared trunk. |
| `connections[].tags` | array of identifiers, up to 8 | no | Groups the connection belongs to, by which views include or exclude it; not drawn. |
| `connections[].role`, `connections[].fromSide`, `connections[].toSide` | | no | See [Diagram fields](#diagram-fields). |
| `architecture.delta` | object | no | `{ "before": "today", "after": "target" }`: the `key`s of two views drawn side by side as before and after, with a ledger of what was added, removed, changed, moved and rewired. See [Architecture delta](../../guides/chart-types/#architecture-delta). `unknown_view`, `same_view`, `option_not_supported`. |
| `components[].replaces` | identifier | no | In a delta, the component this one stands in for in the state after (`unknown_component`). |
| `architecture.boundaries` | array, up to 12 | no | `[{ "id": "net", "label": "…", "in": "region" }]` (label up to 60 characters): frames such as a cloud region, a network or a system. |
| `architecture.mainPath`, `architecture.sameRank`, `architecture.notes`, `architecture.orientation`, `architecture.spacing` | | no | See [Diagram fields](#diagram-fields), with components and connections. |
| `architecture.sharedTrunks` | boolean | no | Draws connections that leave a component together, or reach one together, on one line that branches, where they read the same (no label, technology or step, usual direction, same `dash`, `weight` and `role`, off the main path, other ends in the same boundary). |
| `architecture.views` | array, up to 20 | no | Views on the model; see [Views](#views). |
| `architecture.defaultView` | identifier | no | The key of the view rendered when none is asked for; without it, the first view. |
| `architecture.perspectives` | array, up to 8 | no | See [Perspectives](#perspectives). |
| `architecture.perspective` | identifier | no | The key of the perspective the diagram shows, and every view without one of its own. |

Rules: 1–40 components (`too_many_nodes`), up to 80 connections (`too_many_edges`) and 12
boundaries (`too_many_boundaries`) nested at most four deep (`boundaries_too_deep`); identifiers
are unique among components and boundaries (`duplicate_id`), components and boundaries lie in
existing boundaries (`unknown_boundary`) without a circle (`boundary_cycle`), every boundary holds
a component (`empty_boundary`); connections and the main path name existing components
(`unknown_node`, `main_path_gap`, `main_path_unreachable`); connection ids are unique among
connections (`duplicate_id`); a `symbol` is one of the names above (`unknown_symbol`); a `step` lies
from 1 to 99 (`invalid_step`) and every number is used once (`duplicate_step`). Tags: up to 8 per
component or connection (`too_many_tags`), written like ids (`invalid_tag`), each once
(`duplicate_tag`). Interfaces: at most four per component (`too_many_interfaces`), ids unique
within it (`duplicate_id`), none on a `person` or `node`, and a connection to an interface runs
forward and takes no `toSide` (`option_not_supported`); a `toInterface` that `to` does not provide
is `unknown_interface`. A component that lands inside the frame of a boundary it does not belong to
is `group_overlap`. The rules in [Diagram fields](#diagram-fields) apply.

### Views

With `views`, one view is rendered — the one asked for, else `defaultView`, else the first — as a
diagram of its own, never the whole model. See
[Views on one model](../../guides/chart-types/#views-on-one-model).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `views[].key` | identifier | yes | A stable name of the view; unique among views. |
| `views[].title` | string, up to 200 characters | yes | The title of the diagram the view renders. |
| `views[].description` | string, up to 1000 characters | no | Its description; without one it is generated from what the view shows. |
| `views[].components` | array of ids | no | The components the view shows. Required unless the view has `steps`, `include` or `exclude`. |
| `views[].connections` | array of ids | no | The connections it shows, each between shown components; without it, every connection between them. |
| `views[].mainPath` | array of ids, or `{ "from", "to" }` | no | The way the view draws stronger; without one, the model's main path when it lies in the view. |
| `views[].include` | array of tags | no | Shows every component carrying one of these tags and none of `exclude`; with `connections`, also the connections between shown components that carry one. |
| `views[].exclude` | array of tags | no | Leaves out components and connections carrying one, unless the view lists them. With neither `components` nor `include`, the view starts from every component. |
| `views[].steps` | array, up to 99 | no | A dynamic view: `[{ "connection": "c1", "label": "…" }]`, connections of the model in the order they occur, numbered from 1 in place of their own steps: `connection` names one by `id`, the same may occur again, and `label` (up to 60 characters) replaces the connection's. |
| `views[].perspective` | identifier | no | The perspective the view shows, in place of the model's. |
| `views[].width`, `views[].height`, `views[].mobile` | | no | Take the place of the model's and are checked like them, at the view (`invalid_dimension` at `/architecture/views/0/width`). |
| `views[].environment` | string, up to 60 characters | no | Makes the view a deployment view of this environment, such as `Production`. |
| `views[].hosts` | array | deployment | Machines, clusters or zones, nested like boundaries (`id`, `label`, `in`). |
| `views[].instances` | array | deployment | `[{ "id": "a-1", "of": "a", "in": "node" }]`: running instances of the model's components, drawn as `Label (id)`. `id` is unique among the instances and hosts of the view, `of` names the component, `in` the host. |
| `views[].instanceConnections` | array | no | Connections between instances, with the fields of a connection and `from`/`to` naming instances. One with the `id` of a model connection takes the `label`, `technology`, `dash`, `role`, `fromSide` and `toSide` it leaves out from it. |
| `views[].deriveConnections` | boolean | no | Deployment views only: adds an instance connection for every model connection the view does not name by `id`, from each instance of its `from` component to the nearest instances of its `to` component by shared hosts, all of them at a tie; see [Environments and instances](../../guides/chart-types/#environments-and-instances). |

Rules: up to 20 views (`too_many_views`) with unique keys (`duplicate_id`); a view lists
components or steps (`empty_view`) that exist (`unknown_node`, `unknown_connection`), each listed
connection or step joins shown components (`connection_outside_view`), a view with `steps` takes
no `connections` (`option_not_supported`) and at most 99 steps (`too_many_steps`), and
`defaultView` and the key asked for name a view (`unknown_view`). `include` and `exclude` name tags
the model carries (`unknown_tag`), each once in both together (`duplicate_tag`), select at least
one component (`empty_view`), and are not taken by a view with `steps` or an `environment`
(`option_not_supported`). A deployment view lists at least one instance (`empty_view`), each `of`
an existing component (`unknown_node`) and drawn as `Label (id)` in at most 60 characters
(`text_too_long` at the instance's `id`), and takes no `components`, `connections`, `steps` or
`mainPath`; another view takes no `hosts`, `instances`, `instanceConnections` or
`deriveConnections` (`option_not_supported`); an instance connection with an `id` names a model
connection (`unknown_connection`) between the components its instances run
(`instance_mismatch`). Instances, hosts and instance connections are then checked like components,
boundaries and connections, after expansion, with diagnostics in the view. Every view must resolve
to a valid diagram; its diagnostics point into the model or the view. A view keeps the notes
without `on` and those on the components and boundaries it shows; a deployment view keeps only the
notes without `on`.

### Perspectives

`perspectives` name questions such as the team or the technical debt; the diagram shows one only
when `perspective` (on the model or a view) selects it. See
[Perspectives](../../guides/chart-types/#perspectives).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `perspectives[].key` | identifier | yes | A stable name; unique among perspectives. |
| `perspectives[].title` | string, up to 40 characters | yes | Its name in the legend, the description and the table, such as `Team`. |
| `perspectives[].description` | string, up to 200 characters | no | A sentence added to the description. |
| `perspectives[].categories` | array, 1–8 | yes | `[{ "label": "Payments", "components": ["a"], "connections": ["c1"] }]`, in the order that fixes their colors. |
| `categories[].label` | string, up to 40 characters | yes | The category, such as a team; unique in the perspective. |
| `categories[].components`, `categories[].connections` | array of ids | no | What lies in the category; connections by `id`. |

Rules: up to 8 perspectives (`too_many_perspectives`) with well-formed keys of their own
(`invalid_id`), each with 1–8 categories (`invalid_categories`) with distinct labels
(`duplicate_category`) that list existing components (`unknown_node`) and connections
(`unknown_connection`), each in at most one category of a perspective (`duplicate_member`);
`perspective` on the model or a view names an existing perspective (`unknown_perspective`).

## Trees

`"type": "tree"`: one root and nodes that each name their parent, or a dendrogram built from
distances. See [Trees](../../guides/chart-types/#trees) and
[Dendrograms](../../guides/chart-types/#dendrograms).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `tree.nodes` | array, 1–150 | one of | `[{ "id": "a", "label": "…", "sublabel": "…", "parent": "root", "link": "60 %", "kind": "unit", "partner": "b", "value": 3, "parts": [{ "label": "…", "value": 2 }], "role": "critical" }]`. The children of a node keep the order of the list. |
| `nodes[].id` | identifier | yes | Unique. |
| `nodes[].label`, `nodes[].sublabel` | string, up to 60 characters | label | Name and second line. |
| `nodes[].parent` | identifier | no | The node it hangs from; exactly one node has neither a parent nor a partner: the root. |
| `nodes[].link` | string, up to 40 characters | no | What joins the node to its parent, such as a share. |
| `nodes[].kind` | `unit`, `person`, `external` | no | Default `unit`. |
| `nodes[].partner` | identifier | no | Joins two nodes as a couple, side by side; the children of either hang from the middle of the line between them. |
| `nodes[].collapsed` | boolean | no | Draws the node without what hangs below it (and below its partner), with a `+n` for the nodes left out; the description and the table still list them. |
| `nodes[].value` | number | no | Colors the node's fill, from light for the smallest value to dark for the largest. |
| `nodes[].parts` | array, up to 6 | no | As on a flow chart step. |
| `nodes[].role` | role | no | See [Diagram fields](#diagram-fields). |
| `tree.orientation` | `auto`, `portrait`, `landscape` | no | See [Diagram fields](#diagram-fields). |
| `tree.layout` | `layered`, `radial` | no | `layered` (default): one row per level (a column in landscape). `radial` draws a mind map: the root in the middle, every level on a ring around it, each subtree in a sector as wide as its share of the leaves. |
| `tree.valueLabel` | string, up to 40 characters | no | What the `value` of the nodes stands for: the name of the color key and of the table column. |
| `tree.distances` | object | one of | For a dendrogram: `{ "labels": ["a", "b"], "matrix": [[0, 1], [1, 0]] }`, the distances between items, from which the tree is built by agglomerative clustering. |
| `distances.labels` | array of strings, 2–40, each up to 60 characters | yes | The items, in the order of the matrix. |
| `distances.matrix` | array of arrays of numbers | yes | A square, symmetric matrix with 0 on its diagonal and no value below 0. |
| `tree.linkage` | `average`, `complete`, `single` | no | With `distances`: how far apart two clusters are — the mean distance between their items (`average`, default), the largest (`complete`) or the smallest (`single`). |

Rules: 1–150 nodes (`too_many_nodes`) down to twelve levels below the root (`tree_too_deep`);
identifiers are unique (`duplicate_id`), exactly one node has no parent (`invalid_root`), parents
exist (`unknown_node`) and run up to the root without a circle (`circular_parent`), the root has
no `link` (`option_not_supported`), a radial tree has no `orientation`, `partner` or `link`
(`option_not_supported`), and a partner exists, is another node, has no parent and is named by one
node only (`unknown_node`, `invalid_partner`); `collapsed` on a node with nothing below it warns
with `nothing_to_collapse`. `parts` follows the rules of flow charts. A dendrogram takes 2–40
labels (`empty_data`, `too_many_nodes`) that are unique (`duplicate_id`) and a square, symmetric
matrix with 0 on its diagonal and no value below 0 (`invalid_matrix`), and no `nodes`, `layout`,
`orientation` or `valueLabel`; `linkage` without `distances` is refused (`option_not_supported`).

## Networks

`"type": "network"`: nodes as dots joined by edges, with no direction of flow. See
[Networks](../../guides/chart-types/#networks).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `network.nodes` | array, 1–40 | yes | `[{ "id": "a", "label": "…", "group": "…", "role": "critical" }]`. |
| `nodes[].id` | identifier | yes | Unique. |
| `nodes[].label` | string, up to 60 characters | yes | The name. |
| `nodes[].group` | string, up to 40 characters | no | Its color and a legend entry. |
| `nodes[].role` | role | no | Drawn as a ring around the dot. |
| `network.edges` | array, up to 120 | no | `[{ "from": "a", "to": "b", "dash": "solid", "weight": 3, "role": "planned" }]`. |
| `edges[].from`, `edges[].to` | identifier | yes | The nodes it joins. |
| `edges[].dash` | `solid`, `dashed`, `dotted` | no | Default `solid`. |
| `edges[].weight` | number, 0 or more | no | The line is drawn from 1 px for nothing to 5 px for the heaviest edge. |
| `edges[].role` | role | no | See [Diagram fields](#diagram-fields). |
| `network.layout` | `circle`, `rings` | no | `circle` (default): the nodes clockwise from twelve o'clock, groups together; `rings` around `center`, every other node on the ring of its distance in edges. |
| `network.center` | identifier | rings | The node in the middle of the rings. |
| `network.order` | `crossings`, `listed` | no | Circle only: `crossings` (default, groups together with few crossings) or `listed`. |
| `network.directed` | boolean | no | Edges lead from one node to the other, drawn with an arrowhead. |

Rules: 1–40 nodes (`too_many_nodes`), up to 120 edges (`too_many_edges`) and 8 groups
(`too_many_groups`); identifiers are unique (`duplicate_id`), edges join two different existing
nodes (`unknown_node`, `self_loop`) and carry a `weight` of at least 0 (`invalid_edge_weight`);
rings need a `center` that exists (`missing_center`, `unknown_node`) and take no `order`, and a
circle takes no `center` (`option_not_supported`).

## Cycles

`"type": "cycle"`: steps on a ring where the last step feeds the first, optionally around a hub.
See [Cycles](../../guides/chart-types/#cycles).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `cycle.steps` | array, 3–8 | yes | `[{ "id": "a", "label": "…", "note": "…", "hubLabel": "…", "role": "critical" }]`, clockwise from twelve o'clock. |
| `steps[].id` | identifier | yes | Unique. |
| `steps[].label` | string, up to 60 characters | yes | The name. |
| `steps[].note` | string, up to 80 characters | no | A second, smaller line. |
| `steps[].hubLabel` | string, up to 24 characters | no | The label of the link to the hub; needs a hub. |
| `steps[].role` | role | no | See [Diagram fields](#diagram-fields). |
| `cycle.hub` | object | no | `{ "label": "…", "note": "…" }` (label up to 60, note up to 80 characters): what the steps feed, in the middle. |
| `cycle.labels` | `outside`, `inside` | no | `outside` (default): numbered dots with the labels beside them; `inside`: boxes that hold the labels. |

Rules: 3 to 8 steps (`too_few_steps`, `too_many_steps`); identifiers are unique (`duplicate_id`);
`hubLabel` needs a `hub` (`option_not_supported`).

## Layers

`"type": "layers"`: abstraction levels stacked on each other, or scopes drawn inside each other.
See [Layers](../../guides/chart-types/#layers).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `layers.layers` | array, 1–10 (6 nested) | yes | `[{ "id": "a", "label": "…", "items": ["…"], "note": "…", "role": "critical" }]`: from the bottom up in a stack, from the outermost inwards in nested scopes. |
| `layers[].id` | identifier | yes | Unique. |
| `layers[].label` | string, up to 60 characters | yes | The name. |
| `layers[].items` | array of up to 12 strings of up to 40 characters | no | What lies in the layer, drawn as small boxes. |
| `layers[].note` | string, up to 120 characters | no | A remark at the side of the layer (under its label on a narrow canvas). |
| `layers[].role` | role | no | See [Diagram fields](#diagram-fields). |
| `layers.display` | `stack`, `nested` | no | `stack` (default): bands, the first at the bottom; `nested`: frames inside each other, the first the outermost. |
| `layers.projection` | `flat`, `exploded` | no | `flat` (default); `exploded` draws a stack as slabs in 2:1 dimetric projection, lifted apart, with the labels on leader lines (a flat stack on a narrow canvas). Not with `nested` (`option_not_supported`). |

Rules: at least one layer, at most 10 in a stack and 6 nested scopes (`too_many_layers`); at most
12 items per layer (`too_many_items`); identifiers are unique (`duplicate_id`).

## Plans

`"type": "plan"`: rooms, buildings and open ground on a grid, seen at an angle. See
[Plans](../../guides/chart-types/#plans).

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `plan.areas` | array, 1–30 | yes | `[{ "id": "a", "label": "…", "kind": "room", "x": 0, "y": 0, "w": 6, "h": 4, "phase": "…", "items": [] }]`. |
| `areas[].id` | identifier | yes | Unique. |
| `areas[].label` | string, up to 60 characters | yes | The name. |
| `areas[].kind` | `room`, `building`, `outdoor` | no | `room` (default); `building` has a heavier outline; `outdoor` has no walls and may lie under other areas. |
| `areas[].x`, `areas[].y` | number, 0–100 | yes | The corner of the area, in grid units. |
| `areas[].w`, `areas[].h` | number above 0, up to 100 | yes | The width along `x` and the depth along `y`. |
| `areas[].phase` | string, up to 40 characters | no | When the area exists or is built; up to four phases, each with a pattern. |
| `areas[].items` | array, up to 12 | no | `[{ "kind": "desk", "x": 0.5, "y": 0.5 }]`; kinds `desk`, `shelf`, `table`, `bed`, `sofa`, `plant`; `x` and `y` in grid units from the corner of the area. |

Rules: 1–30 areas (`too_many_areas`); unique identifiers (`duplicate_id`); `x`, `y`, `w` and `h`
numbers in range (`invalid_area`); no two rooms or buildings overlap (`area_overlap`); at most four
phases (`too_many_phases`); an item lies inside its area (`item_outside_area`, `too_many_items`).

## Diverging scale

Stripes and calendars share one color scale: a neutral middle at `reference` (default 0) and
eight equally wide steps on either side. `min` and `max` set where the outermost steps begin and
default to the largest distance of any value from the reference; values beyond them take the
outermost step. The 17 colors are CSS custom properties `--chartlet-diverging-0` to `-16` and can
be overridden by the host page.
