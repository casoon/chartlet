---
title: Chart types
description: Vertical and horizontal bars, grouped bars with up to four series, and lines with gaps for missing values.
order: 1
---

| Chart | Specification | Example |
| --- | --- | --- |
| Bar, vertical | `"type": "bar"` with `data` | [Monthly revenue](../../../showcase/monthly-revenue/) |
| Bar, horizontal, with negative values | `"orientation": "horizontal"` | [Quarterly change](../../../showcase/quarterly-change/) |
| Grouped bar, up to four series | `categories` and `series` instead of `data` | [Budget vs. actual](../../../showcase/budget-vs-actual/) |
| Line with gaps for missing values | `"type": "line"`, `null` values | [Monthly trend](../../../showcase/monthly-trend/) |

Each example in the repository's `examples/` folder has its rendered `.svg` and `.html` next to it.
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
