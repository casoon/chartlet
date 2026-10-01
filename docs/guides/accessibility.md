---
title: Accessibility
description: How a chart is exposed to assistive technology, and what has been verified so far.
order: 4
---

- The SVG is exposed as a single image (`role="img"`). Its accessible name is the title, and its
  description is generated from the data, for example: “Bar chart with 4 categories and 2 series
  (Budget, Actual). Highest: 160 (Budget in April). Lowest: 120 (Budget in January). 1 value is
  missing.”
- The HTML output adds a `<figure>` with caption and a real `<table>` containing every value,
  including values whose visual label had to be left out. The caption is the visible title: the
  SVG inside the figure draws none, so the title is shown and announced once, while the SVG keeps it
  as its accessible name.
- Series colours stay distinguishable for the common forms of colour-vision deficiency and have at
  least 4.5:1 contrast against white — and against the dark theme's background, where the lowest
  series colour reaches 7.65:1. The legend lists series in the same order as the bars.
- The figure follows the display-mode convention for visualisations: it carries
  `data-viz="chart"`, and its data table is the text layer, marked `data-viz-text`. A page that
  switches visuals off can hide the graphic and keep the table. A host that wraps the SVG profile
  in its own figure gets the description and the table as data with `alternative: true` (see
  [JavaScript runtimes](../javascript/)).
- Text is never removed silently: shortened labels and omitted value labels produce warnings, and
  the full text stays in the specification and the data table.

## What is verified

Every page of this site, including all showcase charts, is checked with axe (WCAG 2.2 AA) in CI.
Testing with VoiceOver and NVDA is still pending. Until then, treat the output as designed for
accessibility, not as verified. The [support matrix](../../reference/support/) lists the details.

The default palette is designed for light backgrounds; `"theme": "dark"` switches to a dark
palette that paints its own background. This site places light charts on a light surface in its
dark theme for that reason.
