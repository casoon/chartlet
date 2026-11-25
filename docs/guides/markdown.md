---
title: Markdown and documentation sites
description: Write a chart as a fenced code block in Markdown and get the rendered figure in Astro, Starlight, Docusaurus, VitePress, mdBook or any remark or markdown-it pipeline.
order: 3
---

A specification can live in the Markdown itself, as a fenced block in the language `chartlet`.
While the site is built, the block becomes the chart's HTML figure, with its caption, description
and data table. The page loads no chart script.

````md
```chartlet id=revenue
{
  "schemaVersion": 1,
  "type": "bar",
  "title": "Monthly revenue",
  "data": [
    { "label": "January", "value": 120 },
    { "label": "February", "value": 180 }
  ]
}
```
````

| Site generator | Plugin | Import |
| --- | --- | --- |
| Astro, Starlight, Docusaurus, Next.js with MDX, any remark pipeline | remark | `@casoon/chartlet/remark` |
| VitePress, Slidev, Marp, any markdown-it pipeline | markdown-it | `@casoon/chartlet/markdown-it` |
| mdBook | preprocessor of the CLI | `chartlet mdbook` |

Both plugins come with `npm install @casoon/chartlet`; the preprocessor with
`cargo install chartlet`.

## Fence settings

After the language, a block may set three things for itself, as `key=value`, quoted when the value
has spaces:

| Setting | Effect |
| --- | --- |
| `id` | The ID prefix of the chart. Set it when the same specification appears twice on a page, and to keep IDs stable when the specification changes. |
| `view` | The view of an architecture model to render. |
| `perspective` | The perspective of an architecture diagram to show. |

Any other setting stops the build with the list of the three.

## remark

```js
import remarkChartlet from '@casoon/chartlet/remark';

// in the pipeline
.use(remarkChartlet, { table: 'visible' })
```

The options are those of [`renderChart`](../javascript/) for every block, such as `table`, `theme`,
`styles`, `strict` and `allowWarnings`, plus `lang` (the fence language, default `chartlet`) and
`react` (below). The chart enters the tree as elements, not as raw HTML, so the plugin works in
Markdown and in MDX.

An invalid specification fails the file at the block's position with the error code and path. A
warning becomes a message of the file, as other remark plugins report theirs.

**Astro and Starlight.** Astro 7 runs remark plugins on the unified processor from
`@astrojs/markdown-remark`, which has to be installed next to it:

```js
// astro.config.mjs
import { defineConfig } from 'astro/config';
import { unified } from '@astrojs/markdown-remark';
import remarkChartlet from '@casoon/chartlet/remark';

export default defineConfig({
  markdown: { processor: unified({ remarkPlugins: [remarkChartlet] }) },
});
```

The same processor renders `.mdx` pages with `@astrojs/mdx`. For a chart whose data come from a
file or a script, the [Astro component](../astro/) stays the better fit.

**Docusaurus.** Docusaurus renders Markdown with React and keeps the page alive in the browser.
Set `react: true`, so that React leaves the radio buttons and checkboxes of the CSS-only controls
to the browser:

```js
// docusaurus.config.js
import remarkChartlet from '@casoon/chartlet/remark';

// presets → classic → docs (and blog, pages)
remarkPlugins: [[remarkChartlet, { react: true }]],
```

Without it the static page is the same, but after the page loads the focus control and the series
filter no longer react to clicks.

## markdown-it

```js
import MarkdownIt from 'markdown-it';
import markdownItChartlet from '@casoon/chartlet/markdown-it';

const md = new MarkdownIt().use(markdownItChartlet);
```

The options are those of the remark plugin, with `vue` in place of `react` and `onWarning`, which
receives every warning with the line of its block (by default, `console.warn`). Fences in other
languages keep the renderer they had. An invalid specification throws with the line of the block.

**VitePress.** VitePress turns each page into a Vue template, and Vue drops `<style>` elements when
it renders a page in the browser, after a click on a link. Set `vue: true`: the chart is then
passed to Vue as finished HTML.

```js
// .vitepress/config.mjs
import markdownItChartlet from '@casoon/chartlet/markdown-it';

export default {
  markdown: { config: (md) => md.use(markdownItChartlet, { vue: true }) },
};
```

## mdBook

```toml
# book.toml
[preprocessor.chartlet]
command = "chartlet mdbook"
```

The preprocessor renders every block of every chapter for the HTML renderer; other renderers get
the Markdown unchanged. A `chartlet.theme.json` next to `book.toml` applies to every chart. Two
settings in the same table apply to all charts of the book:

| Key | Effect |
| --- | --- |
| `table` | `"details"` (default) or `"visible"` for the data table. |
| `theme` | A bundled theme (`light`, `dark`, `blueprint`, `classic`, `editorial`) or a theme file relative to `book.toml`. |

An invalid specification stops the build with the chapter, the line of the block and the error.
Warnings go to standard error with the same place. Blocks inside block quotes and lists stay as
they are; put the block at the top level of the chapter.

## What stays the same

Each block renders exactly as `chartlet render --format html` with the same settings: the same
bytes, the same accessible figure, the same warnings. A chart in Markdown has no source file of
its own, so `chartlet inspect` and [`--watch`](../../reference/cli/#watching-and-preview) need the
specification in a file; copy it out to work on it, or keep it in a file and use the
[Astro component](../astro/) or [`renderChart`](../javascript/).
