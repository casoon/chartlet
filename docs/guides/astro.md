---
title: Astro
description: Render charts while Astro builds the site – as a component or through the render API.
order: 3
---

The npm package `@casoon/chartlet` renders charts while Astro builds the site and ships no
JavaScript to the browser. It carries the renderer as a WebAssembly build, so the package is all
you install:

```sh
npm install @casoon/chartlet@alpha
```

To render with an installed `chartlet` CLI instead, set `CHARTLET_BIN` to its path or pass
`binary` to the component. Both produce the same bytes.

## The component

```astro
---
import Chart from '@casoon/chartlet/astro';
import revenue from '../data/monthly-revenue.json';
---

<Chart id="monthly-revenue" spec={revenue} />
```

- `id` must be stable and unique within the page. It prevents accessibility ID collisions when the
  same specification is embedded more than once.
- `format="svg"` renders the graphic alone; `format="html"` (the default) renders the figure with
  caption, source and data table.
- The HTML table uses a native disclosure by default; set `table="visible"` to show it permanently.
- A specification with `mobile` needs nothing extra: the component passes the HTML through, and
  the figure switches to the mobile variant by the width of its container. See
  [Responsive charts](responsive.md).

## The render API

When a page needs the output as a string – for example to show build-time warnings next to the
chart – call `renderChart` directly:

```js
import { renderChart } from '@casoon/chartlet';

const { content, warnings } = renderChart(spec, {
  format: 'html',
  table: 'details',
  idPrefix: 'revenue',
});
```

`content` is the SVG or HTML; `warnings` lists the layout warnings, in the same lines the CLI
writes. An invalid specification throws an error with the CLI's message. With
`format: 'svg'` and `variant: 'mobile'` it returns the mobile variant alone. This site uses
exactly that to render its showcase.

Outside Astro, and in runtimes that cannot read files, see [JavaScript runtimes](javascript.md).
