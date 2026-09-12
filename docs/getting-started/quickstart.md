---
title: Quickstart
description: Write a minimal specification and render it as SVG or as an HTML figure.
order: 2
---

## Write a specification

Save a minimal specification as `spec.json`:

```json
{
  "schemaVersion": 1,
  "type": "bar",
  "title": "Monthly revenue",
  "data": [
    { "label": "January", "value": 120 },
    { "label": "February", "value": 180 },
    { "label": "March", "value": 150 }
  ]
}
```

## Render it

```sh
chartlet render spec.json --format html -o chart.html
```

`--format html` produces a `<figure>` with caption, SVG, source and data table. Leave the option out
for a standalone SVG. Pass `-` instead of a file name to read the specification from standard
input.

## Next steps

- All fields are in the [specification reference](../../reference/specification/).
- Grouped bars, horizontal bars and lines are covered in [Chart types](../../guides/chart-types/).
- To render charts while an Astro site builds, see [Astro](../../guides/astro/).
