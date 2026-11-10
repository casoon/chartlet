---
title: Responsive charts
description: A second layout for narrow containers – switched by a container query in the HTML output, or rendered alone for a picture element.
order: 3
---

A chart is laid out at `width` × `height` and scales down with its container. On a phone, an
800-pixel chart shrinks to less than half its size, and 12-pixel labels become too small to read.
The `mobile` field adds a second layout at a size of its own, so a narrow container gets a chart
composed for it rather than a smaller copy.

```json
{
  "schemaVersion": 1,
  "type": "time",
  "title": "Monthly revenue by sales channel",
  "mobile": { "width": 360, "height": 420, "breakpoint": 640 },
  "panes": [ … ]
}
```

| Field | Default | Description |
| --- | --- | --- |
| `mobile.width` | required | Width of the mobile layout in pixels, 200–600. |
| `mobile.height` | 360 | Height in pixels, 160–1600. It does not follow the aspect ratio of the chart: a narrow chart usually needs to be taller, not shorter. |
| `mobile.breakpoint` | 640 | Container width in CSS pixels below which the mobile variant is shown, 320–1600. |

The mobile variant is available for every chart type. It is laid out by the same rules as the chart
itself, which already depend on the width: fewer ticks, a smaller drawn title below 480 pixels (SVG profile) and category
labels that wrap onto a second line, legends that wrap into further rows, and labels shortened only where two lines do
not suffice. On a horizontal range bar chart the title and the hatching legend start at the left
edge instead of above the plot when a wide gutter of category labels leaves them too little room.
A bar chart's category shows all of its value labels or none, see `value_labels_omitted`. A label
that only the mobile layout has to shorten is reported with its usual code and path, and a message
beginning with `mobile variant: `.

## HTML output

With `mobile` set, the HTML figure carries both layouts in one wrapper and switches between them
with a CSS container query on the wrapper's width:

- The chart at `width` × `height` shows by default; the mobile variant shows only below the
  breakpoint. A browser without container queries keeps the full-size chart.
- The hidden variant is `display: none`, so exactly one SVG is in the accessibility tree at any
  width.
- Caption, source and data table appear once; the caption is the visible title, neither SVG draws
  one. Series filters and zoom steps are shared and switch both variants.
- The IDs of the mobile variant end in `-m` (`revenue-m`, `revenue-m-title`, …), so both variants
  and their scoped styles stay apart on one page.

The wrapper takes the full width of its container, because the container query measures it. The
switch follows the container, not the viewport: a chart in a narrow sidebar of a wide page gets the
mobile layout too.

### Where the rules live

The rules that switch the variants are these, here for a breakpoint of 420:

```css
.chartlet-wrapper.chartlet-responsive{display:block;width:100%;container-type:inline-size}
.chartlet-variant-mobile{display:none}
@container (max-width:419px){
  .chartlet-bp-420 .chartlet-variant-desktop{display:none}
  .chartlet-bp-420 .chartlet-variant-mobile{display:block}
}
```

By default the figure brings them along in a `<style>` element at the start of its wrapper, like
the rest of its stylesheet: the output stands alone. A `<style>` element inside the page body is
not valid HTML, though, and a Content Security Policy without `'unsafe-inline'` blocks it unless
it is named by hash. With [`styles: "external"`](../javascript/#shared-stylesheet) the figure
carries no `<style>` outside its SVGs and no `style` attribute; the page loads the rules with the
shared stylesheet:

```html
<link rel="stylesheet" href="/chartlet.css" />
…
<div class="chartlet-wrapper chartlet-responsive chartlet-bp-420">
  <figure class="chartlet-figure" data-viz="chart">
    <figcaption>…</figcaption>
    <div class="chartlet-variant-desktop"><svg …>…</svg></div>
    <div class="chartlet-variant-mobile"><svg …>…</svg></div>
    …
  </figure>
</div>
```

The shared stylesheet has the first two rules and the container query of the default breakpoint,
640. Every other breakpoint is one more container query, which the site adds to the file it
serves:

```js
import { stylesheet } from '@casoon/chartlet';

const css = stylesheet({ types: ['time'], breakpoints: [420] });
```

```sh
chartlet stylesheet --types time --breakpoints 420 > public/chartlet.css
```

`stylesheet({ types: [], common: false, breakpoints: [420, 560] })` returns the container queries
alone, for a file of their own; Rust has `stylesheet_breakpoints(&[420, 560])`. Each query
addresses its breakpoint by the class on the wrapper, so the queries of several breakpoints load
in any order. A chart whose breakpoint has no rule on the page keeps showing its full-size layout.
The rules of the zoom steps are part of the shared stylesheet too. A layer that declares a color
of its own still brings a `<style>` inside its SVG, see
[Content Security Policy](../javascript/#content-security-policy).

## SVG output

The SVG profile is unchanged by `mobile` and renders the chart at `width` × `height`. To render the
mobile variant alone, pass `--variant mobile`:

```sh
chartlet render revenue.json -o revenue.svg
chartlet render revenue.json --variant mobile -o revenue.mobile.svg
```

A `<picture>` element then chooses by viewport width:

```html
<picture>
  <source media="(max-width: 639px)" srcset="revenue.mobile.svg" />
  <img src="revenue.svg" alt="Monthly revenue by sales channel" />
</picture>
```

An SVG loaded as an image does not expose its title and description to assistive technology, so the
`alt` text has to carry the chart's message; the HTML output keeps them.

`--variant mobile` on a specification without `mobile` fails with `missing_mobile`; with
`--format html`, which always carries both layouts, it fails with `option_not_supported`.

## Astro and the render API

The Astro component passes the HTML through and needs no option for this. `renderChart` takes
`variant: 'mobile'` together with `format: 'svg'`:

```js
const { content } = renderChart(spec, { format: 'svg', variant: 'mobile' });
```
