---
title: Interaction without JavaScript
description: Series filters, pre-rendered zoom steps and native tooltips – built from HTML controls and CSS.
order: 2
---

The HTML output can add native controls and CSS-based interaction. No chart JavaScript is shipped.
The pure SVG profile stays a single static chart; filtering and stepped zoom are HTML-only.

## Series filter

A grouped chart gets a checkbox per series. Deselecting one hides its bars and value labels with CSS
`:has()`; the axis does not rescale. The data table and description always show the full data.
Browsers without `:has()` support simply keep every series visible.

Try it on [Budget vs. actual](../../../showcase/budget-vs-actual/).

## Zoom steps

Add two to four `zoomSteps` to pre-compute narrower views of the same chart. The HTML output renders
one variant per step and switches between them with radio buttons. Each step costs its own SVG in
the output file.

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

Try it on [Headcount](../../../showcase/headcount/).

## Tooltips

Every bar and point carries a native `<title>` (`Month: value`, or `Month – Series: value`) that
browsers can show on hover. The chart description and data table, rather than these hover-only
tooltips, remain the assistive-technology alternative.
