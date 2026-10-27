# CLI route (no MCP server)

Prerequisite: the `chartlet` binary (`cargo install chartlet`, Rust 1.88 or newer), or Node 22.12+
with `npm install @casoon/chartlet` and `renderChartDetailed` from it. Check with
`chartlet --help`.

| Step | Command |
| --- | --- |
| Starting points | Copy and adapt a file from `examples/` of the repository, or write the specification from the schema (`schema/chartlet.schema.json`) |
| Validate and see warnings | `chartlet render spec.json --diagnostics json -o /dev/null` (stderr is one JSON document with `ok`, `error` and `warnings`) |
| Render SVG | `chartlet render spec.json -o chart.svg` |
| Render HTML with data table | `chartlet render spec.json --format html -o chart.html` |
| Print, social, PNG | `--variant print`, `--variant social`, `--format png` (PNG needs a build with the `png` feature) |
| Provenance | `--manifest chart.manifest.json` |
| Fail on warnings | `--strict`, with `--allow-warning <code>` for reviewed ones |
| Gaps in a diagram's model | `chartlet inspect spec.json` |
| Theme | `--theme light\|dark\|blueprint\|theme.json`; a `chartlet.theme.json` in the working directory applies by itself |

The loop is the same as with MCP: draft, validate, fix the field the error path names, render.
There is no data inspection or coverage tool on this route: read the data yourself, and for a
redraw list the source's elements and compare them with the specification by hand, reporting
what is kept, changed and missing.
