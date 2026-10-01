# @casoon/chartlet

Build static, accessible SVG or HTML charts during an Astro build. The component runs only on
the server and ships no chart JavaScript to the browser.

The package carries the renderer compiled to WebAssembly, so it is all you install. It needs
Node.js 22.12 or newer.

```sh
npm install @casoon/chartlet
```

```astro
---
import Chart from '@casoon/chartlet/astro';
import chart from '../data/monthly-revenue.json';
---

<Chart id="monthly-revenue" spec={chart} />
```

`id` must be stable and unique within the page. It prevents accessibility ID collisions when
the same specification is embedded more than once.

Use `format="svg"` for the graphic alone or `format="html"` for the default figure, caption,
source, and data-table output. The HTML table uses a native disclosure by default; set
`table="visible"` to show it permanently.

## Render API

```js
import { renderChart } from '@casoon/chartlet';

const { content, warnings } = renderChart(spec, { format: 'svg', idPrefix: 'revenue' });
```

Options: `format`, `table`, `idPrefix`, `variant` (`mobile` renders the mobile variant alone as
SVG, `print` the chart as SVG with literal colors for PDF and print renderers, `social` the chart
on a 1200 × 630 canvas for Open Graph images), `strict` and `manifest` (`true` adds a provenance `manifest` to the result: chartlet
version, SHA-256 of the canonical specification and of `content`, format, variant, ID prefix and
warnings, without a timestamp). `warnings` holds the lines the CLI would write; an invalid specification
throws with the CLI's message. The output is byte-identical to the `chartlet` CLI.

`renderChartDetailed(spec, options)` takes the same arguments but does not throw on an invalid
specification: it returns `{ ok, error?, warnings }` with `{ code, path, message }` diagnostics,
plus `content`, `styleHashes` and `manifest` when rendering succeeded. `spec` may also be JSON
text.

To render with an installed CLI instead of WebAssembly, set `CHARTLET_BIN` to its absolute path
or pass `binary`.

## Content Security Policy

A chart styles itself with inline `<style>` elements. `styleHashes` lists their CSP source
expressions (`'sha256-…'`), one per distinct element, in order of first appearance; a strict
`style-src` allows exactly those:

```js
const { content, styleHashes } = renderChart(spec, { idPrefix: 'revenue' });

headers.set('Content-Security-Policy', `style-src 'self' ${styleHashes.join(' ')}`);
```

Astro takes the hashes without quotes: per page through
`Astro.csp?.insertStyleHash(hash.slice(1, -1))`, or for all pages in
`security.csp.styleDirective.hashes`. `createRenderer` returns the same field.

## Interactive module

`@casoon/chartlet/interactive` is an optional browser module without dependencies for charts
rendered with `hooks: true`: crosshair with values (pointer and keyboard), series toggles, scroll
stations and playback. Import only the features a page uses:

```js
import { enhance, crosshair, toggle } from '@casoon/chartlet/interactive';

enhance(document.querySelector('figure.chartlet-figure'), { crosshair, toggle });
```

It reads values from the data table and geometry from `data-*` hooks, needs no inline script and
sets styles only through the CSSOM. See
[Interaction](https://github.com/casoon/chartlet/blob/main/docs/guides/interaction.md#optional-javascript).

## Cloudflare Workers and Vite

Runtimes that cannot compile WebAssembly from bytes, such as workerd, import the module and pass
it to `createRenderer` from `@casoon/chartlet/wasm`, which has no Node.js imports:

```js
import wasm from '@casoon/chartlet/chartlet.wasm';
import { createRenderer } from '@casoon/chartlet/wasm';

const { renderChart } = createRenderer(wasm);
```

In a Vite browser build, compile it from the emitted URL:

```js
import wasmUrl from '@casoon/chartlet/chartlet.wasm?url';
import { createRenderer } from '@casoon/chartlet/wasm';

const { renderChart } = createRenderer(
  await WebAssembly.compileStreaming(fetch(wasmUrl)),
);
```

## Development

`src/chartlet.wasm` is not committed. `npm run build:wasm` builds it from the `wasm/` crate of the
repository (target `wasm32-unknown-unknown`, optimized with `wasm-opt` when it is installed);
`npm pack` and `npm publish` run it first.
