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
| `mobile.width` | required | Width of the mobile layout in pixels, 280–600. |
| `mobile.height` | 360 | Height in pixels, 240–1600. It does not follow the aspect ratio of the chart: a narrow chart usually needs to be taller, not shorter. |
| `mobile.breakpoint` | 640 | Container width in CSS pixels below which the mobile variant is shown, 320–1600. |

The mobile variant is available for every chart type. It is laid out by the same rules as the chart
itself, which already depend on the width: fewer ticks, shortened labels, and a time chart's legend
wrapping into further rows. A label that only the mobile layout has to shorten is reported with its
usual code and path, and a message beginning with `mobile variant: `.

## HTML output

With `mobile` set, the HTML figure carries both layouts in one wrapper and switches between them
with a CSS container query on the wrapper's width:

- The chart at `width` × `height` shows by default; the mobile variant shows only below the
  breakpoint. A browser without container queries keeps the full-size chart.
- The hidden variant is `display: none`, so exactly one SVG is in the accessibility tree at any
  width.
- Caption, source and data table appear once. Series filters, zoom steps and the area picker of a
  topic map are shared and switch both variants.
- The IDs of the mobile variant end in `-m` (`revenue-m`, `revenue-m-title`, …), so both variants
  and their scoped styles stay apart on one page.

The wrapper takes the full width of its container, because the container query measures it. The
switch follows the container, not the viewport: a chart in a narrow sidebar of a wide page gets the
mobile layout too.

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
