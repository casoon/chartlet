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
| `type` | yes | `bar`, `line`, `time`, `topicmap`, `atlas`, `stripes`, `calendar`, `rangebar`, or `multiples`. |
| `title` | yes | Visible title; also the accessible name of the chart (`<title>`). The SVG profile draws it in the chart: a title wider than the chart wraps onto a second line at a space, and the chart below it moves down; only what does not fit on two lines is shortened (`text_truncated`). The HTML profile shows it as the `<figcaption>` and draws no title in its SVGs, so it never shortens it. |
| `data` | one of | Single series: `[{ "label": "…", "value": 1 }]`. |
| `categories` + `series` | one of | Several series: unique category labels, and `[{ "name": "…", "values": […] }]`. |
| `orientation` | no | `vertical` (default) or `horizontal`; bar charts only. |
| `theme` | no | `light` (default) or `dark`; both palettes are CSS custom properties on the root. |
| `timeAxis.timezone` | no | Fixed UTC offset such as `"+02:00"`, or `UTC` (default); `time` charts only. |
| `timeAxis.title` | no | Title of the time axis. |
| `timeAxis.gaps` | no | `show` (default) keeps the distances in time; `collapse` places every observed timestamp at the same distance from the next, so weekends and holidays take no space. Reference lines, point markers and zones then take the slot of an observation and must lie within the observed range (`time_out_of_range`). |
| `panes` | no | `time`: 1–4 panes stacked on one shared time axis, each with its own `valueAxis` and `layers`. `multiples`: 2–12 panes, each with a unique `title`, sharing the top-level `valueAxis`. Up to six data layers per pane — on a `time` chart at most four of the whole chart without their own `color` — and six annotation layers (zones, reference lines and point markers together). |
| `panes[].title` | multiples | Heading of a small-multiples panel; not allowed on a `time` chart, whose panes are named by their `valueAxis.title`. |
| `panes[].heightRatio` | no | `time` only: the pane's share of the plot height against the other panes, 1–10, default 1. |
| `panes[].valueAxis` | no | `time` only: `title`, `format`, `decimals`, `min` and `max` of the pane's own value axis. |
| `columns` | no | `multiples` only: grid columns, 1–6; default up to three. |
| `panes[].layers[].mark` | no | `line`, `area`, `ohlc` (candlesticks, `time` only), `band` (a zone), or `annotation`. An `area` is a line whose region down to zero is filled in its color; the value axis then always includes zero. The area between two lines is an uncertainty band (`lower`/`upper`), not a separate mark. |
| `panes[].layers[].name` | one of | Legend and table label; required once a pane — or a `time` chart across its panes — has several data layers, and unique within the pane; on a `time` chart unique across all panes, since they share one legend. |
| `panes[].layers[].data` | ohlc | `[{ "time": "2026-03-02", "open": 10, "high": 12, "low": 9, "close": 11 }]`, 2–2000 candles with increasing `time`; `low` at or below `open` and `close`, `high` at or above them. A candlestick layer takes no `color`, `modeled`, `stroke` or `dash`. |
| `panes[].layers[].points` | line, area | `[{ "time": 1772323200, "value": 1 }]`; ISO 8601 dates and bare years (`"1850"`) are accepted too. `"value": null` marks a missing observation: the line, its area and its band break there, and table and description say “Missing”. Add `lower` and `upper` to every point with a value for an uncertainty band; a point without a value carries neither. |
| `panes[].layers[].modeled` | no | Line and area layers: the line is dashed (unless `dash` says otherwise), its band hatched, and legend, description and table say “modeled”. |
| `panes[].layers[].stroke` | no | Line and area layers: `regular` (default, 3 px), `thin` (1 px), or `bold` (4.5 px); the legend sample shows the same weight, so two lines differ in more than color. |
| `panes[].layers[].dash` | no | Line and area layers: `solid`, `dashed`, or `dotted`. Defaults to `dashed` for a modeled layer and `solid` otherwise; the legend sample shows the same pattern. |
| `panes[].layers[].value` / `time` | annotation | A horizontal reference line at `value`, a vertical one at `time`, or a point marker at both. Both widen the axes. |
| `panes[].layers[].shape` | no | Point markers only: `circle` (default), `square`, `diamond`, `triangle-up`, or `triangle-down`. A reference line refuses it. |
| `panes[].layers[].bottom` / `top` | band | Value edges of a zone; `bottom` must lie below `top`. A missing edge is the edge of the plot. Given edges widen the value axis. |
| `panes[].layers[].from` / `to` | band | Time edges of a zone in the forms a point's `time` takes; `from` must lie before `to`. A missing edge is the edge of the plot. At least `bottom` and `top`, or `from` and `to`, are required. |
| `panes[].layers[].label` | annotation, band | Required text of a reference line, point marker or zone. |
| `stripes` | stripes | `{ "firstYear": 1850, "values": [..], "reference": 0, "min": .., "max": .., "yearLabels": true }`; `null` leaves a year empty. |
| `calendar` | calendar | `{ "year": 2024, "layout": "months" \| "weeks", "days": [{ "date": "2024-03-01", "value": 1 }], "reference", "min", "max" }`. |
| `ranges` | rangebar | `[{ "label": "…", "low": 0, "high": 1, "mid": 0.5, "modeled": false }]`; `orientation` applies. |
| `panes[].layers[].color` | no | `#rgb`, `#rrggbb`, `#rrggbbaa`, or `var(--name)` without a fallback. |
| `description` | no | Replaces the generated description. |
| `source` | no | Shown below the chart in the HTML output. |
| `categoryAxis.title` | no | Title of the category axis. |
| `valueAxis.title` | no | Title of the value axis. |
| `valueAxis.format` | no | `number` (default) or `percent`; `0.12` is shown as `12%`. |
| `valueAxis.decimals` | no | Fixed decimal places, 0–6, for values in tooltips, value labels, description and table. Axis ticks always carry as many decimals as their step (`0.0, 0.5, 1.0`). On a `time` chart every pane sets it on its own `valueAxis`. |
| `valueAxis.min`, `valueAxis.max` | no | The axis reaches at least down to `min` and up to `max`; values beyond them widen it further, and each end is rounded outward to a tick. Without them, a line, time series or range bar whose values are all zero or more never gets a margin below zero (and one whose values are all zero or less none above it). Bars always start at zero. On a `time` chart every pane sets them on its own `valueAxis`. |
| `locale` | no | `en` (default) or `de`: language of the generated texts (description, legend additions, tooltips, calendar month and weekday names, HTML caption and table) and the number format (`1,5`). Available for every chart type. Negative numbers always use the true minus sign `−`. |
| `showTitle` | no | `false` leaves the drawn title out of the SVG profile when the page heads the chart, and gives its space to the chart; it stays the accessible name. Every chart type. The HTML profile never draws the title (its `<figcaption>` is the visible title), so it ignores `showTitle`. Default `true`. |
| `width`, `height` | no | Size in pixels: 320–2400 × 240–1600, default 800 × 450. |
| `mobile` | no | A second layout for narrow containers: `{ "width": 360, "height": 360, "breakpoint": 640 }`. `width` is required, 280–600; `height` 240–1600, default 360; `breakpoint` 320–1600, default 640, is the container width in CSS pixels below which the HTML profile shows the mobile variant. Available for every chart type. The SVG profile is unchanged; `--variant mobile` renders the mobile variant alone. See [Responsive charts](../guides/responsive.md). |
| `showValues` | no | Value labels on bars and points, default `true`. On a time pane with several layers the labels can overlap; the values stay in the tooltips and the data table. |
| `zoomSteps` | no | Two to four `{ "label", "from", "to" }` variants, selectable in the HTML output. Bar and line charts: `from` and `to` are category indices. `time` charts: timestamps in the forms a point's `time` takes; each window must hold at least two observations of one layer, and the chart is redrawn from the observations inside it. |

## Limits

Up to 100 categories and four series. Labels must be unique. Values must be zero or have a
magnitude between `1e-100` and `1e100`.

A `time` chart carries one to four panes (`too_many_panes`), each with a `heightRatio` from 1 to 10
(`invalid_height_ratio`) and up to six data layers of 2000 observations or candles each. At most
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
covers one year between 1700 and 2199. Range bars take up to 100 ranges.

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
`calendar`, `ranges`, `columns`) is refused the same way.

## Diverging scale

Stripes and calendars share one color scale: a neutral middle at `reference` (default 0) and
eight equally wide steps on either side. `min` and `max` set where the outermost steps begin and
default to the largest distance of any value from the reference; values beyond them take the
outermost step. The 17 colors are CSS custom properties `--chartlet-diverging-0` to `-16` and can
be overridden by the host page.
