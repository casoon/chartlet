---
title: Recipes
description: Start from the job at hand - a report from CSV, a blog explanation, architecture in several views, a release flow, a slide or print figure - with a complete specification, theme and tested commands for each.
order: 2
---

Each recipe is a complete example you can copy and change: a specification, the brand theme, the
commands and what they produce. The files are in
[`examples/recipes/`](https://github.com/casoon/chartlet/tree/main/examples/recipes). **All data
in them are made-up samples**; the `source` of each specification says so, and a real chart
needs its own source line. `tests/recipes.rs` renders every recipe with both themes, in every
output a recipe names, and fails on any warning.

## Find a recipe

| You want | Recipe | Family | Outputs |
| --- | --- | --- | --- |
| A figure for a report, from a table | [Report from CSV](#report-from-csv) | bar | HTML, narrow screen, print SVG |
| A diagram that explains one idea in an article | [Blog explanation](#blog-explanation) | flow | HTML with caption and table, narrow screen |
| Software architecture at several levels | [Architecture in views](#architecture-in-views) | architecture | one page of all views, or one SVG per view |
| A process with a decision and loops | [Release flow](#release-flow) | flow | SVG, strict build, narrow screen |
| A figure for a slide or a print layout | [Slide and print figure](#slide-and-print-figure) | bar | 16:9 SVG, print SVG, PNG |

Starting points for other types: the `chartlet_chart_starter` and `chartlet_diagram_starter`
tools of the [MCP server](../ai-agents/), and the [showcase](https://casoon.github.io/chartlet/showcase/),
which lists every example by chart family and use case.

## The brand theme

All recipes use [`brand.theme.json`](https://github.com/casoon/chartlet/blob/main/examples/recipes/brand.theme.json)
(and `brand.theme.dark.json`), made with `chartlet_theme_from_brand` from five made-up colors:
paper `#fbf8f3`, ink `#26231f`, muted `#6b655c`, accent `#0f766e`, link `#1d4ed8`. The tool
adjusted one value, the derived zero line (`#9b9894` to `#928f8b`, 2.71:1 to 3.04:1); its
receipt lists each change. Copy the
file to `chartlet.theme.json` in your project and the CLI and the MCP server apply it without
`--theme`. See [Customize the look](../customize/).

## Report from CSV

Data: [`revenue.csv`](https://github.com/casoon/chartlet/blob/main/examples/recipes/report-from-csv/revenue.csv)
(three regions, four quarters, one value missing). chartlet does not read CSV itself: a script or
an assistant (`chartlet_inspect_data`) turns the rows into `categories` and `series`, and a
missing cell becomes `null` so that table and description say "Missing" instead of 0.

```json
{
  "schemaVersion": 1,
  "type": "bar",
  "title": "Revenue by region and quarter",
  "source": "Sample data for this recipe, not real measurements",
  "categories": ["Q1", "Q2", "Q3", "Q4"],
  "series": [
    { "name": "North", "values": [120, 135, 142, 150] },
    { "name": "South", "values": [98, 104, null, 121] },
    { "name": "West", "values": [76, 81, 90, 97] }
  ],
  "mobile": { "width": 360 }
}
```

(The file has axis titles and `thousandsSeparator` too.) Commands, from the repository root:

```sh
S=examples/recipes
chartlet render $S/report-from-csv/spec.json --theme $S/brand.theme.json --format html -o revenue.html
chartlet render $S/report-from-csv/spec.json --theme $S/brand.theme.json --variant print -o revenue.print.svg
chartlet render $S/report-from-csv/spec.json --theme $S/brand.theme.json --strict --manifest revenue.manifest.json -o revenue.svg
```

The HTML carries the desktop and the narrow layout and switches by the width of its container.
Warnings: none. The manifest names the theme.

Through MCP: `chartlet_render` with `{ "spec": ..., "format": "html", "outputPath": "figures/revenue.html" }`
and, with the theme file as `chartlet.theme.json` in `CHARTLET_MCP_ROOT`, no `theme` at all.

## Blog explanation

A flow chart for one idea, with the explaining text around it. The specification
(`blog-explanation/spec.json`) gives `description` its own wording, because the generated one
says what the diagram contains, not why it matters. The HTML profile is a `<figure>` with caption,
the graphic, the source and the data table; a headline and a paragraph are page content around it,
so **no compiler field is needed for an editorial figure**:

```html
<article>
  <h2>Why the second request is fast</h2>
  <p>The first visitor pays for rendering; everyone after gets the stored page until it goes stale.</p>
  <!-- chartlet render ... --format html -->
  <figure class="chartlet-root" ...>...</figure>
  <p>The table below the figure lists every step and where it leads.</p>
</article>
```

```sh
chartlet render examples/recipes/blog-explanation/spec.json --theme examples/recipes/brand.theme.json --format html --id-prefix cache -o figure.html
```

Give each chart on a page its own `--id-prefix`. Warnings: none.

## Architecture in views

One model, several diagrams: `architecture-views/spec.json` has a context view and a dynamic view
that numbers the steps of one booking. Each view is a diagram of its own with its own title,
description and table.

```sh
R=examples/recipes
chartlet render $R/architecture-views/spec.json --theme $R/brand.theme.json --view context --format svg -o context.svg
chartlet render $R/architecture-views/spec.json --theme $R/brand.theme.json --view booking --format svg -o booking.svg
chartlet render $R/architecture-views/spec.json --theme $R/brand.theme.json --all-views --format html -o views.html
chartlet inspect $R/architecture-views/spec.json
```

`inspect` lists what the model leaves undocumented. A model of more than about nine components is
better split into views than drawn at once (`diagram_over_budget`).

## Release flow

`release-flow/spec.json`: two lanes, a decision and a loop back. A pipeline can fail on any
warning, and allow reviewed ones by code:

```sh
chartlet render examples/recipes/release-flow/spec.json --theme examples/recipes/brand.theme.json --strict -o release.svg
chartlet render examples/recipes/release-flow/spec.json --variant mobile -o release.mobile.svg
```

## Slide and print figure

`slide-figure/spec.json` sets `"size": "slide-16x9"`. The print variant has literal colors for
PDF pipelines; PNG needs the `png` feature of the CLI.

```sh
R=examples/recipes
chartlet render $R/slide-figure/spec.json --theme $R/brand.theme.json --format svg -o slide.svg
chartlet render $R/slide-figure/spec.json --theme $R/brand.theme.json --variant print -o slide.print.svg
chartlet render $R/slide-figure/spec.json --theme $R/brand.theme.json --format png -o slide.png
```

`--size print-a4-landscape` replaces the canvas without editing the file.

## Build and publish

**Framework-neutral Node build.** One script, no browser, the same bytes as the CLI:

```js
import { readFileSync, writeFileSync } from 'node:fs';
import { renderChart } from '@casoon/chartlet';

const spec = JSON.parse(readFileSync('examples/recipes/report-from-csv/spec.json', 'utf8'));
const brand = JSON.parse(readFileSync('examples/recipes/brand.theme.json', 'utf8'));
const { content, warnings, manifest } = renderChart(spec, { format: 'html', theme: brand, idPrefix: 'revenue', manifest: true });
if (warnings.length > 0) throw new Error(warnings.join('\n'));
writeFileSync('revenue.html', content);
```

**Astro.** `<Chart id="revenue" spec={revenue} theme={brand} />` from `@casoon/chartlet/astro`;
see [Astro](../astro/). No script reaches the browser.

**Markdown and README.** An SVG file carries its text alternative, but most Markdown renderers
treat it as an image, so give it alt text that states the message and link the HTML figure for the
table:

```md
![Revenue rose in every region; the South has no value for Q3.](revenue.svg)

[Data table and full description](revenue.html)
```

**Optional browser behavior** (crosshair, series toggle) is opt-in: render with `hooks` and load
`@casoon/chartlet/interactive`; without it the figure is complete. See
[Interaction](../interaction/).
