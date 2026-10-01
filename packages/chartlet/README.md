# @casoon/chartlet

Build static, accessible SVG or HTML charts during an Astro build. The component runs only on
the server and ships no chart JavaScript to the browser.

The package carries the renderer compiled to WebAssembly, so it is all you install. It needs
Node.js 22.12 or newer.

```sh
npm install @casoon/chartlet@alpha
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
SVG) and `strict`. `warnings` holds the lines the CLI would write; an invalid specification
throws with the CLI's message. The output is byte-identical to the `chartlet` CLI.

To render with an installed CLI instead of WebAssembly, set `CHARTLET_BIN` to its absolute path
or pass `binary`.

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
