---
title: JavaScript runtimes
description: Render charts from Node.js, Cloudflare Workers or a Vite build with the WebAssembly renderer in the npm package.
order: 3
---

`@casoon/chartlet` contains the renderer compiled to WebAssembly. It produces the same bytes as
the CLI and the crate, and it needs no Rust toolchain.

## Node.js

`renderChart` loads `chartlet.wasm` from the package on first use and renders synchronously:

```js
import { renderChart } from '@casoon/chartlet';

const { content, warnings } = renderChart(spec, { format: 'svg', idPrefix: 'revenue' });
```

The options are those of the CLI: `format`, `table`, `idPrefix`, `variant` and `strict`. When
`options.binary` or the environment variable `CHARTLET_BIN` names a `chartlet` executable,
`renderChart` calls that CLI instead.

## Runtimes that import WebAssembly

Cloudflare Workers (workerd) do not compile WebAssembly from bytes at run time; the module has to
be imported so that the bundler compiles it ahead of time. `@casoon/chartlet/wasm` takes that
module and has no Node.js imports:

```js
import wasm from '@casoon/chartlet/chartlet.wasm';
import { createRenderer } from '@casoon/chartlet/wasm';

const { renderChart } = createRenderer(wasm);

export default {
  fetch() {
    const { content } = renderChart(spec, { idPrefix: 'revenue' });
    return new Response(content, { headers: { 'content-type': 'text/html' } });
  },
};
```

Wrangler bundles the `.wasm` import as a compiled module without further configuration.

## Vite

In a browser bundle, let Vite emit the file and compile it once before rendering:

```js
import wasmUrl from '@casoon/chartlet/chartlet.wasm?url';
import { createRenderer } from '@casoon/chartlet/wasm';

const module = await WebAssembly.compileStreaming(fetch(wasmUrl));
const { renderChart } = createRenderer(module);
```

`renderChart` from `createRenderer` takes the same options and returns the same result as the
Node.js entry; `binary` is ignored.
