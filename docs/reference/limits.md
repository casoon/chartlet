---
title: Limits and performance
description: What chartlet accepts, what is enforced, and measured render times, memory and output sizes, so you can estimate whether your input fits.
order: 5
---

This page separates three kinds of limit:

- **Enforced limits** refuse input with a code, a path and a fix. Nothing is truncated silently.
- **Readability advice** (`dense_chart`, `diagram_over_budget`, `text_truncated`) warns; the figure is
  still produced. It is about whether a person can read the result, not about safety.
- **Measured behavior** is how long a render takes and how big the output is. It is a measurement
  on named hardware, not a promise.

## Enforced limits

| Limit | Value | Refusal |
| --- | --- | --- |
| Size of the specification | 4 MiB (`spec_too_large` at `/`) | before parsing; the largest figure at the declared limits is about a tenth of it |
| Length of a text field | 200 characters for titles and labels (`text_too_long`), see the schema for the others | with the path of the field |
| JSON nesting | 128 levels, the parser's recursion limit | `invalid_json` or `invalid_spec` at the position; the process does not overflow its stack |
| Bar and line data points | 100 per series (`too_many_data_points`) | |
| Time chart | 2000 points per layer, 6 layers (8 stacked), 4 panes, 60 markers | |
| Scatter | 5000 points, 8 lines | |
| Box plot, survival | 1000 observations per box, 2000 observations | |
| Flow, state, architecture | 40 nodes, 80 edges or connections, 8 lanes and 8 groups; 12 boundaries nested up to 4 deep | |
| Sequence | 12 participants, 60 messages, 12 fragments nested up to 3 deep | |
| Tree | 150 nodes, 12 deep, 40 leaves in a dendrogram | |
| Class, network, Sankey | 40 classes and 100 relations; 40 nodes and 120 edges; 100 links | |
| Architecture views | 20 views, 8 perspectives | |
| Canvas | 200 to 2400 pixels wide | `canvas_too_small` is not a refusal: the diagram is drawn larger, with a warning |
| MCP output | 16 MiB of output per render through the CLI path | |

The exact numbers per type are in the [specification reference](../specification/) next to each
field; the counts above are the constants of the compiler.

A failed render has no effect on the next one: the compiler holds no state between renders, the CLI
exits, the WebAssembly instance is re-created by the [playground](../../guides/playground/) after a
trap, and the MCP server returns the error as a result. This is tested (`src/tests/limits.rs`,
`tests/generated.rs`).

Text is escaped everywhere it is written; a title or label with markup stays text in SVG and HTML
(tested with `<script>`, quotes and ampersands).

## Reading the measurements

`node scripts/bench.mjs --binary target/release/chartlet` generates the workloads below
deterministically, runs the executable, and prints the table; the checked-in numbers are from the
machine named above the table. WebAssembly and MCP figures need `npm run build:wasm` first; the
script says so when they are missing instead of guessing.

Measured with `node scripts/bench.mjs --runs 15` on Apple M4 Pro (14 cores), 48 GB, darwin 27.2.0, Node v24.22.1, chartlet 0.21.0 (release build).

| Workload | Input | Cold start | Warm (median of 15) | Worst | Peak memory | Output | Output gzip | Warnings |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| small editorial bar chart | 0.4 kB | 5 ms | 4 ms | 4 ms | 4.2 MB | 4.3 kB | 1.2 kB | none |
| dense time series, 6 x 2000 points | 441.2 kB | 25 ms | 26 ms | 27 ms | 10.0 MB | 455.3 kB | 119.0 kB | dense_chart |
| multi-view architecture, all views | 2.7 kB | 6 ms | 5 ms | 6 ms | 6.4 MB | 61.7 kB | 8.1 kB | none |
| architecture, 4 nested levels, 40 components (#6) | 5.1 kB | 312 ms | 305 ms | 314 ms | 6.2 MB | 61.9 kB | 8.0 kB | diagram_over_budget, group_overlap, canvas_too_small |
| flow, 40 steps in 8 groups (#6) | 2.8 kB | 7 ms | 6 ms | 8 ms | 5.6 MB | 37.8 kB | 4.6 kB | diagram_over_budget, canvas_too_small |
| long German labels, 16 categories | 2.0 kB | 4 ms | 4 ms | 4 ms | 4.4 MB | 11.2 kB | 1.8 kB | text_truncated |
| escaped text everywhere | 1.8 kB | 4 ms | 4 ms | 4 ms | 4.4 MB | 10.6 kB | 1.5 kB | text_truncated |

Times include starting the process (cold start is the first run after the file was written; a warm run is the median of the following ones), so they bound the CLI from above.

WebAssembly and MCP figures: not measured, `packages/chartlet/src/chartlet.wasm` is not built here (`npm run build:wasm` in packages/chartlet).

### What the numbers say

- **A figure within the limits renders in single-digit milliseconds** through the CLI; a dense time
  series at the limit (6 layers of 2000 points, 441 kB of JSON) takes about 26 ms, at 10 MB of memory.
- **Output size is the number to watch for MCP and prompts:** that time series is 455 kB of HTML
  (119 kB gzipped). An assistant that reads it back spends that much of its context. Use
  `outputPath` in `chartlet_render`, and ask for the SVG rather than the HTML profile when the data
  table is not needed; a typical figure is 4 to 60 kB.
- **One outlier:** an architecture with four nested boundary levels and 40 components takes about
  300 ms, 60 times as long as the other workloads. It is the slowest layout measured, still well
  below one second, and it is within the declared limits. Its warnings (`diagram_over_budget`,
  `group_overlap`) are what to heed: that diagram is past what reads well anyway.
- **Issue 6 (groups and boundaries asking for an absurd canvas) did not reproduce** on 0.21.0: the
  grouped flow of 40 steps asks for a height of 2924 px and the nested architecture for a width of
  2403 px, and rendering at that size resolves the warning (`src/tests/limits.rs` keeps a smaller
  fixture of both).
- **Cold start** is the first process start after the specification was written, on a machine where
  the executable is cached; the first start after a boot can take a few hundred milliseconds more.
  The CLI builds with no startup work of its own.

### Budgets

Set from these measurements with room for slower machines, and checked by
`node scripts/bench.mjs --check` (not part of CI; run it before a release):

| Budget | Value |
| --- | --- |
| Warm render of a figure within the declared limits | 250 ms, and 2 s for the nested architecture workload |
| Peak memory of one render | 64 MB |
| Output of one render | 1 MB |
| Specification | 4 MiB (enforced) |

### WebAssembly, browser and MCP

Not measured in this report: the WebAssembly module was not built where the report was made. What
is known without measuring: the build script (`npm run build:wasm`) enforces a budget of 720 kB
gzipped for `chartlet.wasm` and prints the size, which is also the lazy-loaded payload of the
[playground](../../guides/playground/). Warm renders in WebAssembly run the same code as the CLI;
expect them within a small factor of the CLI times above, without the process start. The model-visible
size of an MCP result is the size of the `content` above plus the warnings and the CSP hashes, so
the output column is the figure to plan with. `node scripts/bench.mjs` fills the WebAssembly and MCP
columns when the module is built; run it and replace this paragraph with its output.

