---
title: Print and PDF
description: An SVG with literal colors for PDF pipelines and renderers outside the browser.
order: 3
---

The chart's SVG takes its colors from CSS custom properties, so that a host page can override a
single color. Browsers resolve them; many renderers outside the browser do not. A PDF pipeline
built on such a renderer draws every bar, line and label that takes its color from a custom
property in black, or not at all.

The print variant draws the same chart for these renderers:

```sh
chartlet render revenue.json --variant print -o revenue.print.svg
```

## What changes

- The layout is that of the SVG profile at `width` × `height`; only the stylesheet differs.
- Every `var(--chartlet-…)` is replaced by the value of the chart's theme, light or dark. A layer
  color declared as `var(--name, #0f766e)` is drawn in its fallback color; one declared as
  `var(--name)` without a fallback becomes the chart's text color and is reported as
  `color_not_resolved`.
- The stylesheet keeps class selectors in a `<style>` element, scoped to the chart's root ID. It
  contains no custom properties, no `currentColor`, no `:has()` and no attribute selectors.
- The font stack is unchanged and ends in the generic `sans-serif`. A renderer without Inter
  draws the text in its own fallback font, which may be narrower or wider than the widths the
  layout measured with.
- Hatched bands and ranges keep their patterns, drawn in literal colors.
- The native tooltips (`<title>` per mark) stay; a PDF does not show them.
- IDs end in `-p` (`revenue-p`, `revenue-p-title`, …), so the print and the screen SVG can share
  a page.

A host page cannot recolor the print variant through custom properties. The series filter and
zoom steps belong to the HTML profile and are not part of it. `--variant print` with
`--format html` fails with `option_not_supported` at `/render/variant`.

## Renderers

| Renderer | Status |
| --- | --- |
| resvg, through Typst 0.15 (`#image("revenue.print.svg")`, PNG and PDF) | Verified: bars, grouped bars, lines with declared colors on a dark chart, hatched bands, range bars, candlesticks, a calendar heatmap, a topic map and a landscape match the screen SVG. |
| WebKit (macOS Quick Look) | Verified: same as in the browser. |
| librsvg (`rsvg-convert`), Inkscape, Prince, wkhtmltopdf | Not yet verified. All support class selectors in `<style>`. |

The screen SVG in the same Typst document draws every mark black.

## Astro and the render API

`renderChart` takes `variant: 'print'` together with `format: 'svg'`:

```js
const { content } = renderChart(spec, { format: 'svg', variant: 'print' });
```
