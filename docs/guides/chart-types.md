---
title: Chart types
description: Bars, grouped bars, lines with gaps, time series with bands and reference lines, candlesticks and stacked panes, small multiples, warming stripes, calendar heatmaps, range bars, and two kinds of map.
order: 1
---

| Chart | Specification | Example |
| --- | --- | --- |
| Bar, vertical | `"type": "bar"` with `data` | [Monthly revenue](../../../showcase/monthly-revenue/) |
| Bar, horizontal, with negative values | `"orientation": "horizontal"` | [Quarterly change](../../../showcase/quarterly-change/) |
| Grouped bar, up to four series | `categories` and `series` instead of `data` | [Budget vs. actual](../../../showcase/budget-vs-actual/) |
| Line with gaps for missing values | `"type": "line"`, `null` values | [Monthly trend](../../../showcase/monthly-trend/) |
| Time series on a calendar axis | `"type": "time"` with `panes` and `layers` | [Daily orders](../../../showcase/daily-orders/) |
| Time series, dark theme, declared colors | `"theme": "dark"`, `color` per layer | [Revenue vs. forecast](../../../showcase/revenue-vs-forecast/) |
| Separate landmasses, area by value | `"type": "topicmap"` with `topics` | [Insights themes](../../../showcase/topicmap-sample/) |
| One continuous land, position by kinship | `"type": "atlas"` with `realms` | [Documentation by area](../../../showcase/knowledge-landscape/) |
| Time series with uncertainty band, modeled | `lower`/`upper` per point, `"modeled": true` | [Temperature projection](../../../showcase/temperature-projection/) |
| Threshold and date marker | `"mark": "annotation"` with `value` or `time` | [Annual mean and threshold](../../../showcase/annual-mean-threshold/) |
| Time series with area, gaps and zoom | `"mark": "area"`, `null` values, `dash`, `zoomSteps` by time | [Data hall power draw](../../../showcase/sensor-readings/) |
| Zones and point markers | `"mark": "band"` with `from`/`to` or `bottom`/`top`; `"mark": "annotation"` with `time`, `value` and `shape` | [Checkout API error rate](../../../showcase/release-incidents/) |
| Candlesticks with a volume pane | `"mark": "ohlc"` with `data`; several `panes` with `heightRatio` | [Daily share price](../../../showcase/share-price/) |
| Small multiples, shared value axis | `"type": "multiples"` with titled `panes` | [Emission pathways](../../../showcase/emission-pathways/) |
| Warming stripes | `"type": "stripes"` with `stripes` | [Warming stripes](../../../showcase/warming-stripes/) |
| Calendar heatmap, by month or week | `"type": "calendar"` with `calendar` | [Daily anomaly calendar](../../../showcase/daily-anomaly-calendar/) |
| Range bars with central value | `"type": "rangebar"` with `ranges` | [Warming contributions](../../../showcase/warming-contributions/) |
| Mobile variant for narrow containers, any type | `"mobile": { "width": 360 }` | [Monthly revenue by sales channel](../../../showcase/mobile-revenue/) |

Each example in the repository's `examples/` folder has its rendered `.svg` and `.html` next to it;
one with a mobile variant also has its `.mobile.svg`.
The SVG files are also the reference output of the test suite.

## Several series

Replace `data` with `categories` and `series`. Every series needs exactly one value per category.
A legend is added automatically, and the data table gets one column per series.

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

## Missing values

In a line chart, `null` marks an observation that does not exist. chartlet leaves a visible gap
instead of drawing a line across it. In a grouped bar chart, `null` leaves out that bar. The data
table shows these cells as “Missing”. A single-series bar chart requires a value for every
category.

## Maps

Two chart types draw subjects as land, and they answer different questions.

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

## Time series

`"type": "time"` drops categories altogether: an observation carries a timestamp, the ticks land
on calendar boundaries, and the layer list lives in `panes`. Bars and grouped series do not apply
to it, and a `time` chart refuses them by name instead of ignoring them. Its `zoomSteps` name a
window of time instead of a range of categories.
The [time series guide](time-series.md) covers timestamps, timezone, per-layer colors, the dark
theme, and the limits for dense data.
