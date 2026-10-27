---
title: AI agents
description: Let an AI assistant draft charts while chartlet validates and compiles them, through the MCP server @casoon/chartlet-mcp.
order: 7
---

An assistant is good at reading a request and drafting a specification; it is not reliable at
drawing a correct chart or describing data exactly. `@casoon/chartlet-mcp` splits the work: the
assistant proposes, chartlet validates and compiles deterministically. The server runs no
language model, never changes the data, and reports only facts it can compute.

## In practice

<figure>
<video controls preload="none" playsinline width="1920" height="1080" poster="../../../videos/chartlet-mcp-poster.webp" aria-label="Five requests to chartlet through MCP, as a chat with tool calls" style="width:100%;height:auto">
<source src="../../../videos/chartlet-mcp.mp4" type="video/mp4">
</video>
<figcaption>Five requests to an assistant with the chartlet server, 1:48, no sound. The chat is staged; every tool call and result is real output of the server, shortened.</figcaption>
</figure>

1. **A chart from a data file:** "Make a chart from `data/revenue-2026.csv` for the Q3 report and
   save it to `src/charts/`." `chartlet_inspect_data` suggests grouped bars;
   `chartlet_validate_spec` rejects the first draft with `series_length_mismatch` and the hint to
   use `null` for the missing Q4 value; `chartlet_render` writes the figure to
   `src/charts/revenue-q3.html`.
2. **A diagram from a description:** "Draw our checkout flow as a sequence diagram: browser, API,
   payment provider, database." `chartlet_diagram_starter` gives a valid sequence
   specification to start from; the assistant fills in participants and messages, and chartlet
   lays the diagram out.
3. **Fixing an existing chart:** "The labels in the plans chart get cut off – fix it."
   `chartlet_validate_spec` reports `text_truncated` for all five labels; with
   `"orientation": "horizontal"` the warnings are gone.
4. **Text for a report:** "Write two sentences about the revenue chart." `chartlet_explain`
   returns first, last, minimum, maximum and missing values; the sentences use only those.
5. **A timeline:** "Show the relaunch phases with the launch milestone as a timeline."
   `chartlet_validate_spec` warns with `follows_overlap` that the build phase starts before the
   design phase ends; the assistant moves the start and renders the timeline.

## Setup

The server speaks the [Model Context Protocol](https://modelcontextprotocol.io) over stdio. In
Claude Code:

```sh
claude mcp add --env CHARTLET_MCP_ROOT="$PWD" chartlet -- npx -y @casoon/chartlet-mcp
```

For Claude Desktop and project configuration, see the
[package README](https://github.com/casoon/chartlet/blob/main/packages/chartlet-mcp/README.md).

## Tools

- `chartlet_inspect_data` reads CSV or row objects and returns the row count, per column the inferred
  type, missing values and range, and which chart types fit, each with the columns to use and a
  reason. The rules are fixed: a category and numbers suggest a bar chart, a time column and
  numbers a time chart, open/high/low/close an `ohlc` layer, low/high a range bar, years and one
  value warming stripes, dates of one year and one value a calendar. Tables of other shapes
  get the types with a block of their own: two numbers a scatter plot, a repeating category and
  numbers a box plot (or violin), unique labels with positive numbers a treemap, with values of
  both signs a waterfall, `from`/`to` columns a Sankey diagram, a time column and a 0/1 event
  column survival curves, unique labels with two dates a timeline.
- `chartlet_validate_spec` renders the specification and discards the output. It returns `ok`, the error
  with `code`, JSON Pointer `path` and `message`, and every warning, for the desktop and the
  `mobile` layout. An invalid specification is a regular result, so the assistant can fix the
  field the path points to and try again.
- `chartlet_render` returns the SVG or HTML with warnings, CSP `styleHashes` and the provenance
  [manifest](../warnings-and-errors/#provenance). With `outputPath` it writes the file below the
  directory named by `CHARTLET_MCP_ROOT` instead and returns the path and size.
- `chartlet_explain` returns the description chartlet generates for the chart and, per series or
  layer, count, missing values, minimum, maximum, first and last value with their labels or
  times. The result says that it is computed, not interpreted; conclusions about causes or
  trends stay with the reader.

- `chartlet_diagram_starter` returns, for `sequence`, `flow`, `state`, `architecture`, `tree` or `classes`, a valid
  starting specification, the kinds every element can take with the shape each is drawn in, and
  notes on ids, layout and orientation. For a diagram, `chartlet_explain` adds its structure:
  the counts of its elements and every row of its data table. With `type: "architecture"`,
  `template` picks a starter at one level of abstraction — `context`, `containers` or
  `deployment` — and the result adds the `question` it answers and its `level`; see
  [Context, containers and deployment](../../guides/chart-types/#context-containers-and-deployment)
  for what each level shows and where it stops short of a complete architecture model.
- `chartlet_chart_starter` returns a valid starting specification and notes for the charts that
  are drawn from a block of their own: `waterfall`, `waffle`, `heatmap`, `parliament`, `treemap`, `sankey`,
  `survival`, `scatter`, `timeline`, `forest` (a range bar chart), `violin` (a box plot) and the
  flow presets `prisma` and `instances`.

The resource `chartlet://schema` holds the [specification](../../reference/specification/)
schema. The resource `chartlet://guide/types` maps what you want to show to the type and the
fields to use, for every type of the schema, with when not to draw, readability budgets and the
most common mistakes; it is built from the schema, so a new type appears in it.

## Prompts

Clients that support MCP prompts list them as slash commands. Each leads from a request through
the tools above: read the type guide, state the plan (type, size, what is left out), draft from a
starter, `chartlet_validate_spec` until `ok`, `chartlet_render`. They keep the data and the
relationships as given, propose any aggregation explicitly, stop with a diagnosis instead of
looping on the same error, and say when a table or a sentence serves better.

- `diagram` draws a software diagram from a description.
- `chart` makes a chart from data, starting with `chartlet_inspect_data`; the optional `data`
  argument takes CSV text.
- `redraw` redraws an existing diagram from text, an image or a file as a specification and ends
  with a report of what was covered and what was left out.

## In the chat

In a client that supports [MCP Apps](https://github.com/modelcontextprotocol/ext-apps),
`chartlet_render` also shows the chart in the conversation through the view
`ui://chartlet/figure`: the HTML profile as the compiler renders it, with
caption, source and the data table. The view draws nothing itself; its one script talks to the
client. It follows the client's light or dark theme when the specification sets no `theme`,
switches to a `mobile` layout when it is narrow, and offers "Download SVG", "Download HTML" and
"Copy specification" as keyboard-reachable buttons. Styles come from the shared stylesheet
(`styles: "external"`), which the view resource carries once; it loads nothing from the network.
Clients without MCP Apps support get the same text result as before.

## With opengrid

`chartlet_inspect_data` takes the row objects of an opengrid selection or of opengrid's JSON
export unchanged; decimals written as strings are read as numbers. The servers are not coupled:
the chart is static, and a click in it does not filter the grid.

## What the server refuses

- To guess: `1,234` is a string, not a number, and an unsorted time column is reported, not
  sorted. Suggestions name what has to change first, such as aggregating repeated categories.
- To write anywhere unless the client configuration names a directory in `CHARTLET_MCP_ROOT`.
  Inside it, absolute paths, `..`, and symbolic links that lead out are refused.

## With the npm package

The tools are a thin layer over `renderChartDetailed` from `@casoon/chartlet`, which returns the
compiler's diagnostics as data instead of throwing; see
[JavaScript runtimes](../javascript/#structured-diagnostics).
