---
title: Time series
description: Line charts on a real time axis — timestamps, timezone, layers, colors, theme, and the limits for dense data.
order: 2
---

`"type": "time"` draws lines on a calendar axis. An observation carries a timestamp instead of a
category, so the horizontal position is a real distance in time and the ticks land on calendar
boundaries rather than in the middle of a band.

```json
{
  "schemaVersion": 1,
  "type": "time",
  "title": "Daily orders",
  "timeAxis": { "timezone": "UTC", "title": "Day" },
  "panes": [
    {
      "valueAxis": { "title": "Orders" },
      "layers": [
        {
          "mark": "line",
          "name": "Orders",
          "points": [
            { "time": "2026-03-01", "value": 412 },
            { "time": "2026-03-02", "value": 438 }
          ]
        }
      ]
    }
  ]
}
```

## Timestamps

`time` is either Unix seconds or an ISO 8601 string: `1772323200`, `"2026-03-01"`,
`"2026-03-01T12:00:00Z"`, or `"2026-03-01T12:00:00+02:00"`. A bare year such as `"1850"` stands
for January 1st; when every observation falls on January 1st, tooltips, description and table
label them by year.

A timestamp that does not carry its own offset is read as wall-clock time in
`timeAxis.timezone` — a fixed UTC offset such as `"+02:00"`, `UTC` by default. `"2026-03-01"`
therefore means midnight in that zone, and shifting the zone moves absolute timestamps but not
bare dates.

Timestamps must increase within a layer and lie between 1700-01-01 and 2200-01-01.

## Layers

A pane holds up to four data layers, each a line, and up to six reference lines. Give each layer a `name` as soon as a
pane has more than one, and keep the names unique: they label the legend and the data table.
A layer may name its own `color` in the same forms chartlet accepts anywhere — `#rgb`,
`#rrggbb`, `#rrggbbaa`, or `var(--your-variable)`. A `var()` reference keeps the host page in
charge of the value and falls back to the chart's text color when the page defines nothing, so a
series never disappears silently. A value outside the contract becomes the neutral gray `#667085`
and is reported as a `color_not_supported` warning.

## Uncertainty bands and modeled lines

Give every point of a line `lower` and `upper` to draw a band around it, in the line's color. A
layer with `"modeled": true` is drawn dashed, its band hatched, and its legend entry and the
description say “(modeled)”, so the distinction does not rest on color. The data table gets a
`lower` and an `upper` column for the layer. Both edges on every point, or on none, is required
(`incomplete_band`); `lower` above `upper` is `invalid_band`; a value outside its own band is
reported as `value_outside_band`.

## Reference lines

An `"annotation"` layer with `value` and `label` draws a dashed horizontal line across the plot,
for a threshold such as 1.5 °C; with `time` and `label` it draws a vertical marker. Reference lines
widen the axes so they are always visible, are named in the description, and carry a tooltip; they
are not series, so they have no legend entry and no table column. A point marker with both `time`
and `value` arrives with the markers of a later milestone.

## Small multiples

`"type": "multiples"` draws 2 to 12 panes as small time charts in a grid of `columns`. Every panel
needs a unique `title`; the panels share the top-level `valueAxis`, the time span and one legend,
in which a layer name keeps its color across all panels. Panels draw smaller markers and no value
labels; the tooltips and the table (one column per panel and layer) carry the values.

## Theme

`"theme": "dark"` switches the palette. Both themes are drawn from CSS custom properties on the
chart root, so a page can override any single value; the dark theme additionally paints its own
background, which makes the SVG self-contained.

## Dense series

chartlet draws markers and value labels only while they stay readable — up to 60 observations per
layer. Above that the layer is drawn as a line only; the values remain in the data table, which is
always part of the HTML profile.

The limits are measured, not guessed. At the default width of 800 pixels, four layers of 2000
observations each render to 131 KB of SVG (355 KB of HTML) in about 14 ms. Beyond one observation
per plot pixel — 704 of them at the default width — the drawing cannot show every point, and
chartlet reports that as a `dense_chart` warning.

| Limit | Value |
| --- | --- |
| Observations per layer | 2000 |
| Data layers per pane | 4 |
| Reference lines per pane | 6 |
| Panes | 1 (`time`), 2–12 (`multiples`) |
| Observations with markers and value labels | 60 per layer |

## Value labels

`showValues` draws a label above every point. A single line stays readable that way; with two or
more lines the labels of neighbouring series can land on top of each other, because collision
handling arrives with the later mark milestones. The dark example in the showcase therefore draws
its two lines without labels and leaves the values to the tooltips and the data table.

## Not here yet

These are part of the next milestones and are refused with a named error instead of being
ignored: the `area`, `ohlc`, and zone `band` marks, point annotations, more than one pane on a
`time` chart, gaps for missing observations (`null`), and `zoomSteps`.