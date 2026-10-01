# @casoon/chartlet-mcp

A [Model Context Protocol](https://modelcontextprotocol.io) server around the chartlet compiler.
An AI assistant proposes a chart specification; chartlet validates and compiles it
deterministically into static, accessible SVG or HTML.

- No language model runs inside the server. Every answer comes from the compiler or from
  arithmetic on the data you pass in.
- Data is never changed: values are read as written, nothing is rounded, sorted, aggregated or
  dropped behind your back. Where chartlet needs a different shape, the tools say so.
- No invented insights: `explain_chart` returns computed facts and labels them as such.
- The same specification and options always produce the same bytes, recorded in a provenance
  manifest with SHA-256 hashes.

It needs Node.js 22.12 or newer and talks over stdio.

## Install

### Claude Code

```sh
claude mcp add chartlet -- npx -y @casoon/chartlet-mcp@alpha
```

or in a project's `.mcp.json`:

```json
{
  "mcpServers": {
    "chartlet": {
      "command": "npx",
      "args": ["-y", "@casoon/chartlet-mcp@alpha"]
    }
  }
}
```

Claude Code starts the server in the project directory, so `render_chart` can write into it.

### Claude Desktop

In `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "chartlet": {
      "command": "npx",
      "args": ["-y", "@casoon/chartlet-mcp@alpha"]
    }
  }
}
```

Claude Desktop may start servers in the file system root. The server then refuses `outputPath`
and returns the chart content instead.

## Tools

| Tool | Does | Changes anything |
| --- | --- | --- |
| `inspect_data` | Takes CSV (RFC 4180, header row) or an array of row objects. Returns the row count; per column the inferred type (number, integer, date-time, boolean, string), missing values, min/max, first/last and order of times, distinct count of strings; and the chart types that fit, with the columns to use and a reason. | No |
| `validate_spec` | Renders the specification with the compiler and discards the output. Returns `ok`, the error with `code`, JSON Pointer `path` and `message`, and all warnings. | No |
| `render_chart` | Renders SVG or HTML (`format`, `variant`: `desktop`, `mobile` or `print`, `idPrefix`, `table`). Returns the content, warnings, CSP `styleHashes` and the provenance `manifest`; with `outputPath`, writes the file and returns its path and byte size instead. | Writes `outputPath` |
| `explain_chart` | Returns the accessible description chartlet generates, the chart type, and per series or layer the count, missing values, min, max, first and last value with their labels or times — computed, not interpreted. | No |

Resource: `chartlet://schema` is the JSON Schema of the specification.

A typical session: `inspect_data` on the user's table, draft a specification from a suggestion,
`validate_spec` until `ok` and the warnings are acceptable, `render_chart` with an `outputPath`,
and `explain_chart` when the chart needs to be described in words.

## Safety

- `outputPath` must be relative to the server's working directory. Absolute paths, `..`
  segments, symbolic links that lead outside the directory, and an existing symbolic link as the
  target are refused. Missing directories are created; an existing file is overwritten. When the
  working directory is the file system root, nothing is written.
- No network access, no shell, no other file reads than the package's own schema.
- `inspect_data` recognises numbers only in plain notation (`-1234.5`, `1e3`); `1,234` stays a
  string rather than being guessed. Four-digit integers count as years only when the column name
  contains "year" or "jahr" or the values strictly increase.

## Development

```sh
cd packages/chartlet-mcp && npm install && npm test
```

The package depends on the published `@casoon/chartlet` release. To run it against the
checked-out compiler instead, build its `chartlet.wasm` and install it without saving:
`npm install ../chartlet --no-save`. Both packages are released together.
`npm test` and `npm pack` copy `schema/chartlet.schema.json` from the
repository into the package first.

## License

MIT
