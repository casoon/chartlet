---
title: AI agents
description: Let an AI assistant draft charts while chartlet validates and compiles them, through the MCP server @casoon/chartlet-mcp.
order: 7
---

An assistant is good at reading a request and drafting a specification; it is not reliable at
drawing a correct chart or describing data exactly. `@casoon/chartlet-mcp` splits the work: the
assistant proposes, chartlet validates and compiles deterministically. The server runs no
language model, never changes the data, and reports only facts it can compute.

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
  value warming stripes, dates of one year and one value a calendar.
- `chartlet_validate_spec` renders the specification and discards the output. It returns `ok`, the error
  with `code`, JSON Pointer `path` and `message`, and every warning, for the desktop and the
  `mobile` layout. An invalid specification is a regular result, so the assistant can fix the
  field the path points to and try again.
- `chartlet_render` returns the SVG or HTML with warnings, CSP `styleHashes` and the provenance
  [manifest](warnings-and-errors.md#provenance). With `outputPath` it writes the file below the
  directory named by `CHARTLET_MCP_ROOT` instead and returns the path and size.
- `chartlet_explain` returns the description chartlet generates for the chart and, per series or
  layer, count, missing values, minimum, maximum, first and last value with their labels or
  times. The result says that it is computed, not interpreted; conclusions about causes or
  trends stay with the reader.

The resource `chartlet://schema` holds the [specification](../reference/specification.md)
schema.

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
[JavaScript runtimes](javascript.md#structured-diagnostics).
