# @casoon/chartlet-mcp

A [Model Context Protocol](https://modelcontextprotocol.io) server around the chartlet compiler.
An AI assistant proposes a chart specification; chartlet validates and compiles it
deterministically into static, accessible SVG or HTML.

- No language model runs inside the server. Every answer comes from the compiler or from
  arithmetic on the data you pass in.
- Data is never changed: values are read as written, nothing is rounded, sorted, aggregated or
  dropped behind your back. Where chartlet needs a different shape, the tools say so.
- No invented insights: `chartlet_explain` returns computed facts and labels them as such.
- The same specification and options always produce the same bytes, recorded in a provenance
  manifest with SHA-256 hashes.

It needs Node.js 22.12 or newer and talks over stdio.

## Install

### Claude Code

```sh
claude mcp add --env CHARTLET_MCP_ROOT="$PWD" chartlet -- npx -y @casoon/chartlet-mcp
```

or in a project's `.mcp.json`:

```json
{
  "mcpServers": {
    "chartlet": {
      "command": "npx",
      "args": ["-y", "@casoon/chartlet-mcp"],
      "env": { "CHARTLET_MCP_ROOT": "/path/to/project" }
    }
  }
}
```

`CHARTLET_MCP_ROOT` is the only directory `chartlet_render` may write into (see
[Safety](#safety)). Leave it out and the server never writes files; it returns the chart instead.

### Claude Desktop

In `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "chartlet": {
      "command": "npx",
      "args": ["-y", "@casoon/chartlet-mcp"]
    }
  }
}
```

Without `CHARTLET_MCP_ROOT` the server refuses `outputPath` and returns the chart content; add
`"env": { "CHARTLET_MCP_ROOT": "/path/to/charts" }` to let it write there.

## Tools

| Tool | Does | Changes anything |
| --- | --- | --- |
| `chartlet_inspect_data` | Takes CSV (RFC 4180, header row) or an array of row objects. Returns the row count; per column the inferred type (number, integer, date-time, boolean, string), missing values, min/max, first/last and order of times, distinct count of strings; and the chart types that fit (bar, time, multiples, rangebar, stripes, calendar, scatter, boxplot, treemap, waterfall, sankey, survival, timeline), with the columns to use and a reason. | No |
| `chartlet_validate_spec` | Renders the specification with the compiler and discards the output. Returns `ok`, the error with `code`, JSON Pointer `path` and `message`, and all warnings. | No |
| `chartlet_render` | Renders SVG or HTML (`format`, `variant`: `desktop`, `mobile` or `print`, `idPrefix`, `table`). Returns the content, warnings, CSP `styleHashes` and the provenance `manifest`; with `outputPath`, writes the file and returns its path and byte size instead. In a client that shows MCP Apps, the chart also appears in the chat ([below](#in-the-chat)). | Writes `outputPath` |
| `chartlet_explain` | Returns the accessible description chartlet generates, the chart type, and per series or layer the count, missing values, min, max, first and last value with their labels or times — computed, not interpreted. For a diagram, `structure` gives the counts of its elements and every row of its data table in reading order. | No |
| `chartlet_diagram_starter` | For `sequence`, `flow`, `state` or `architecture`: a valid starting specification, the kinds every element can take with the shape each is drawn in, and notes on ids, layout and orientation. | No |
| `chartlet_chart_starter` | For `waterfall`, `waffle`, `parliament`, `treemap`, `sankey`, `survival`, `scatter`, `timeline`, `forest`, `violin`, `prisma` or `instances`: a valid starting specification and notes on what the type takes and refuses. | No |

Resources: `chartlet://schema` is the JSON Schema of the specification; `ui://chartlet/figure`
is the view of `chartlet_render` for MCP Apps.

A typical session: `chartlet_inspect_data` on the user's table, draft a specification from a suggestion,
`chartlet_validate_spec` until `ok` and the warnings are acceptable, `chartlet_render` with an `outputPath`,
and `chartlet_explain` when the chart needs to be described in words.
For a software diagram, start from `chartlet_diagram_starter` instead of a table: chartlet lays the
diagram out itself, so the specification only says what is connected.

## In the chat

`chartlet_render` declares the view `ui://chartlet/figure` (`text/html;profile=mcp-app`,
[MCP Apps](https://github.com/modelcontextprotocol/ext-apps)). A client that shows MCP Apps
displays the chart in the conversation: the figure with caption, source and the data table in a
disclosure, exactly as the compiler renders the HTML profile. The view draws nothing itself and
loads no chart script; its only script is the bridge to the client, which receives the result,
reports the view's height and passes on the actions.

- Theme: when the specification sets no `theme`, the server renders the figure light and dark,
  and the view shows the one that matches the client's theme, switching when the client does.
  A `theme` in the specification wins. Without a theme from the client the view stays light.
- Width: a specification with a `mobile` layout switches to it by container query when the view
  is narrow.
- Actions, reachable by keyboard: "Download SVG" and "Download HTML" hand the files to the client
  (`ui/download-file`; shown only when the client supports downloads), "Copy specification"
  copies the JSON. No action changes data or calls a tool.
- Styles and CSP: the view resource carries the shared chartlet stylesheet once, and the figures
  are rendered with `styles: "external"`, so most charts bring no `<style>` of their own. Charts
  with declared colors or a `mobile` layout still carry a small inline `<style>`, which the
  default policy of MCP Apps allows (inline styles and scripts, nothing from the network). The
  resource declares no domains: it loads nothing and connects nowhere. `styleHashes` in the result
  are for pages you embed the content in, not for the view.
- The view data travels in the result's `_meta`, which the model does not read, and only to
  clients that declare MCP Apps support. Every other client — Claude Code, scripts, inspectors —
  gets the same result as before: text and `structuredContent`.

## With opengrid-mcp

`chartlet_inspect_data` takes the row objects that opengrid hands over unchanged: the selection
of opengrid-mcp and the JSON of opengrid's `exportRows(…, { format: "json" })` are both arrays of
objects keyed by field name. opengrid's wire notation fits: decimals arrive as strings such as
`"250.00"` and are read as numbers, dates and timestamps as date-time, `null` as missing;
non-finite floats (`"Infinity"`) are not taken for numbers.

A typical flow: filter and sort the grid to the view you want, select the rows, pass the
selection to `chartlet_inspect_data`, build the specification from a suggestion (with numbers as
JSON numbers), check it with `chartlet_validate_spec`, and render it with `chartlet_render`.

The two servers are not coupled and know nothing of each other. The chart is static output: a
click in the chart does not filter the grid, and changing the grid does not update the chart —
that is intended.

## Safety

- Files are written only below `CHARTLET_MCP_ROOT`, an existing directory set in the client
  configuration. Without it `outputPath` is disabled and the tool says how to enable it. The
  working directory the client starts the server in plays no part.
- `outputPath` must be relative to `CHARTLET_MCP_ROOT`. Absolute paths, any `..` segment,
  symbolic links that lead outside the directory (checked on the real path), and an existing
  symbolic link as the target are refused. Missing directories are created; an existing file is
  overwritten. A root that is the file system root is refused.
- No network access, no shell, no other file reads than the package's own schema and view.
- `chartlet_inspect_data` recognises numbers only in plain notation (`-1234.5`, `1e3`); `1,234` stays a
  string rather than being guessed. Four-digit integers count as years only when the column name
  contains "year" or "jahr" or the values strictly increase.

## Development

```sh
cd packages/chartlet-mcp && npm install && npm test
```

The package depends on the released `@casoon/chartlet`. To try it against a change in the
compiler, point the dependency at the checkout for a while (`npm install ../chartlet`, after
`cd packages/chartlet && npm run build:wasm`) and switch back before publishing. `npm test` and `npm pack` copy
`schema/chartlet.schema.json` from the repository into the package first.

`npm test` includes `test/view.e2e.test.mjs`, which opens the view in Chromium (Playwright)
inside a sandboxed iframe under the default MCP Apps CSP, plays the client side of the protocol
with a real `chartlet_render` result, and runs axe on it in both themes. It needs a Playwright
Chromium (`npx playwright install chromium`).

## License

MIT
