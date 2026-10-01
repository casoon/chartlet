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

The options are those of the CLI: `format`, `table`, `idPrefix`, `variant` (`desktop`, `mobile`
or `print`), `strict`, `allowWarnings` and `manifest`. When
`options.binary` or the environment variable `CHARTLET_BIN` names a `chartlet` executable,
`renderChart` calls that CLI instead.

## Text alternative

A page that wraps the SVG profile in its own accessible figure needs the words the chart would
otherwise carry. `alternative: true` returns them as data, in the language of the
specification's `locale`:

```js
const { content, alternative } = renderChart(spec, { format: 'svg', alternative: true });
alternative.description; // the description the SVG carries in <desc>
alternative.table; // { caption, columns, rows }: the data table of the HTML profile as text
```

Each row starts with its category; values are written as the chart writes them, including a
declared `decimals` or `thousandsSeparator`. From Rust, `chartlet::text_alternative(&spec)`
returns the same.

## Structured diagnostics

`renderChart` throws on an invalid specification. `renderChartDetailed` takes the same arguments
and reports the outcome as data, the document the CLI writes with `--diagnostics json`:

```js
import { renderChartDetailed } from '@casoon/chartlet';

const result = renderChartDetailed(spec, { format: 'svg' });
if (!result.ok) {
  console.error(result.error.code, result.error.path, result.error.message);
}
for (const { code, path, message } of result.warnings) {
  console.warn(code, path, message);
}
```

A successful result also carries `content`, `styleHashes` and, with `manifest: true`, `manifest`.
`spec` may be JSON text instead of an object; malformed text is reported as `invalid_json`. A
missing WebAssembly build or CLI still throws. `createRenderer` returns `renderChartDetailed` too.

## Provenance

With `manifest: true`, the result also carries `manifest`, the object the CLI writes with
`--manifest`: chartlet version, schema version, SHA-256 of the canonical specification and of
`content`, format, variant, ID prefix and warnings, without a timestamp. See
[Provenance](warnings-and-errors.md#provenance) for the fields.

```js
const { content, manifest } = renderChart(spec, { format: 'svg', manifest: true });
// manifest.outputHash === `sha256:${sha256Hex(content)}`
```

The WebAssembly build and the CLI path return the same manifest; the CLI path reads it from a
temporary file that it removes again.

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

## Shared stylesheet

By default every chart carries its whole stylesheet, scoped to its root, so an SVG file stands
alone. On a page with many charts that repeats about 1.5–4 KB of identical CSS per chart, and a
mobile variant repeats it again. With `styles: 'external'` a chart carries only what is its own —
the colors its layers declare — and usually no `<style>` at all; the page loads the shared
stylesheet once:

```js
import '@casoon/chartlet/chartlet.css'; // or link the file; 16 KB, about 2 KB gzipped
const { content } = renderChart(spec, { format: 'html', styles: 'external' });
```

The stylesheet is the same for every chart, so it caches across pages, and as a file it needs no
CSP hash. `stylesheet()` returns it as a string; the CLI prints it with `chartlet stylesheet` and
renders with `--styles external`. A chart looks the same either way.

The full file covers every chart type. A site that renders only some of them can serve just
those: `stylesheet({ types: ['bar', 'time'] })` or `chartlet stylesheet --types bar,time` keeps
the common rules (about 4 KB) and the rules of the named types.

## Content Security Policy

A chart carries its styles in inline `<style>` elements, which a strict `style-src` blocks unless
it names them by hash. Both entries return those hashes with the content: `styleHashes` holds the
CSP source expression (`'sha256-<base64>'`) of every distinct `<style>` element in `content`, in
order of first appearance. The hash is taken over the text between the tags, exactly as written.

As a response header:

```js
const { content, styleHashes } = renderChart(spec, { idPrefix: 'revenue' });

return new Response(content, {
  headers: {
    'content-type': 'text/html',
    'content-security-policy': `default-src 'self'; style-src 'self' ${styleHashes.join(' ')}`,
  },
});
```

Astro (6 and newer, with `security.csp` enabled) writes the policy itself and takes hashes without
the quotes. Add them per page:

```astro
---
import { renderChart } from '@casoon/chartlet';
import spec from '../data/monthly-revenue.json';

const { content, styleHashes } = renderChart(spec, { idPrefix: 'monthly-revenue' });
for (const hash of styleHashes) {
  Astro.csp?.insertStyleHash(hash.slice(1, -1));
}
---

<Fragment set:html={content} />
```

or for every page in `astro.config.mjs`:

```js
import { defineConfig } from 'astro/config';
import { renderChart } from '@casoon/chartlet';
import spec from './src/data/monthly-revenue.json' with { type: 'json' };

const { styleHashes } = renderChart(spec, { idPrefix: 'monthly-revenue' });

export default defineConfig({
  security: {
    csp: {
      styleDirective: { hashes: styleHashes.map((hash) => hash.slice(1, -1)) },
    },
  },
});
```

The styles depend on the specification, the format and the `idPrefix`, so hash the same render
you embed. The WebAssembly build computes the hashes in the renderer, so `createRenderer` returns
them synchronously in every runtime; the CLI path computes the same values with `node:crypto`.
