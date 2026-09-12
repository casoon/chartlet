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
  including values whose visual label had to be left out.
- Series colours stay distinguishable for the common forms of colour-vision deficiency and have at
  least 4.5:1 contrast against white. The legend lists series in the same order as the bars.
- Text is never removed silently: shortened labels and omitted value labels produce warnings, and
  the full text stays in the specification and the data table.

## What is verified

Every page of this site, including all showcase charts, is checked with axe (WCAG 2.2 AA) in CI.
Testing with VoiceOver and NVDA is still pending. Until then, treat the output as designed for
accessibility, not as verified. The [support matrix](../../reference/support/) lists the details.

The default palette is designed for light backgrounds; a dark palette is planned. This site places
charts on a light surface in its dark theme for that reason.
