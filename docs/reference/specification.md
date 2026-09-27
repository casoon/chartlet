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
| `title` | yes | Visible title; also the accessible name of the chart. |
| `data` | one of | Single series: `[{ "label": "…", "value": 1 }]`. |
| `categories` + `series` | one of | Several series: unique category labels, and `[{ "name": "…", "values": […] }]`. |
| `orientation` | no | `vertical` (default) or `horizontal`; bar charts only. |
| `theme` | no | `light` (default) or `dark`; both palettes are CSS custom properties on the root. |
| `timeAxis.timezone` | no | Fixed UTC offset such as `"+02:00"`, or `UTC` (default); `time` charts only. |
| `timeAxis.title` | no | Title of the time axis. |
| `timeAxis.gaps` | no | `show` (default) or `collapse`. |
| `panes` | no | `time`: one pane with a `valueAxis` and its `layers`. `multiples`: 2–12 panes, each with a unique `title`, sharing the top-level `valueAxis`. Up to four data layers and six annotation layers per pane. |
| `panes[].title` | multiples | Heading of a small-multiples panel; not allowed on a `time` chart. |
| `columns` | no | `multiples` only: grid columns, 1–6; default up to three. |
| `panes[].layers[].mark` | no | `line` or `annotation`; `area`, `ohlc`, and the zone `band` are planned. |
| `panes[].layers[].name` | one of | Legend and table label; required and unique once a pane has several layers. |
| `panes[].layers[].points` | line | `[{ "time": 1772323200, "value": 1 }]`; ISO 8601 dates and bare years (`"1850"`) are accepted too. Add `lower` and `upper` to every point for an uncertainty band. |
| `panes[].layers[].modeled` | no | Line layers: the line is dashed, its band hatched, and legend, description and table say “modeled”. |
| `panes[].layers[].value` / `time` | annotation | A horizontal reference line at `value`, or a vertical one at `time`; exactly one of the two. |
| `panes[].layers[].label` | annotation | Required text of a reference line. |
| `stripes` | stripes | `{ "firstYear": 1850, "values": [..], "reference": 0, "min": .., "max": .., "yearLabels": true }`; `null` leaves a year empty. |
| `calendar` | calendar | `{ "year": 2024, "layout": "months" \| "weeks", "days": [{ "date": "2024-03-01", "value": 1 }], "reference", "min", "max" }`. |
| `ranges` | rangebar | `[{ "label": "…", "low": 0, "high": 1, "mid": 0.5, "modeled": false }]`; `orientation` applies. |
| `panes[].layers[].color` | no | `#rgb`, `#rrggbb`, `#rrggbbaa`, or `var(--name)` without a fallback. |
| `description` | no | Replaces the generated description. |
| `source` | no | Shown below the chart in the HTML output. |
| `categoryAxis.title` | no | Title of the category axis. |
| `valueAxis.title` | no | Title of the value axis. |
| `valueAxis.format` | no | `number` (default) or `percent`; `0.12` is shown as `12%`. |
| `width`, `height` | no | Size in pixels: 320–2400 × 240–1600, default 800 × 450. |
| `showValues` | no | Value labels on bars and points, default `true`. On a time pane with several layers the labels can overlap; the values stay in the tooltips and the data table. |
| `zoomSteps` | no | Two to four `{ "label", "from", "to" }` variants, selectable in the HTML output. |

## Limits

Up to 100 categories and four series. Labels must be unique. Values must be zero or have a
magnitude between `1e-100` and `1e100`.

A `time` chart carries one pane with up to four layers of 2000 observations each. Markers and
value labels are drawn up to 60 observations per layer; a denser layer is a line only, and the
values stay in the data table. More observations than the plot has horizontal pixels produce a
`dense_chart` warning. These numbers come from a measurement at the default size — see
[the time series guide](../guides/time-series.md).

Small multiples take 2 to 12 panes in up to six columns, with at most four distinct layer names
across all panels (one palette color each). Stripes take up to 500 yearly values; a calendar
covers one year between 1700 and 2199. Range bars take up to 100 ranges.

Fields that belong to a later milestone are refused with a named error rather than ignored: the
`area`, `ohlc`, and zone `band` marks (`mark_not_implemented`), a point annotation with both `time`
and `value`, more than one pane on a `time` chart (`too_many_panes`), `null` observations
(`option_not_supported`), and the bar and line fields `data`, `categories`, `series`,
`orientation`, `categoryAxis`, and `zoomSteps` on a `time` chart (`option_not_supported`). A block
that belongs to another type (`stripes`, `calendar`, `ranges`, `columns`) is refused the same way.

## Diverging scale

Stripes and calendars share one color scale: a neutral middle at `reference` (default 0) and
eight equally wide steps on either side. `min` and `max` set where the outermost steps begin and
default to the largest distance of any value from the reference; values beyond them take the
outermost step. The 17 colors are CSS custom properties `--chartlet-diverging-0` to `-16` and can
be overridden by the host page.
