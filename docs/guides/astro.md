---
title: Astro
description: Render charts while Astro builds the site – as a component or through the render API.
order: 3
---

The npm package `@casoon/chartlet` renders charts while Astro builds the site and ships no
JavaScript to the browser. In the alpha it calls the `chartlet` CLI: install both, and make sure
the CLI is on `PATH` or `CHARTLET_BIN` points to it.

```sh
npm install @casoon/chartlet@alpha
cargo install chartlet --version 0.1.0-alpha.3
```

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

`content` is the SVG or HTML; `warnings` lists the layout warnings the CLI reported. This site uses
exactly that to render its showcase.
