---
title: CLI options
sidebarLabel: CLI options
description: Output formats and options of chartlet render, chartlet inspect and chartlet stylesheet.
order: 2
---

```sh
chartlet render <spec.json | -> [options]
```

| Option | Effect |
| --- | --- |
| `--format svg` | Standalone SVG with `<title>` and `<desc>` and the title drawn in the chart (default). |
| `--format html` | `<figure>` with caption, SVG, source and data table; the caption is the visible title, the SVG draws none. With `mobile` in the specification, both variants behind a container query. |
| `--format png` | PNG of the print variant, of the social variant with `--variant social`, or of the mobile layout with `--variant mobile`, drawn with the bundled Inter font. Needs chartlet built with the `png` feature (`cargo install chartlet --features png`). |
| `--scale <factor>` | With `--format png`: image pixels per SVG pixel, 0.25–4 (default 1). |
| `--table details` | Puts the HTML data table in a native, initially closed `<details>` (default). |
| `--table visible` | Shows the data table permanently. |
| `--id-prefix <prefix>` | Stable ID of the chart root and prefix for its other IDs; needed when the same chart appears twice on one page. |
| `--variant mobile` | Renders the mobile variant alone as SVG, for a `<picture>` source; requires `mobile` in the specification (`missing_mobile`) and the SVG format. `desktop` (default) renders the chart at `width` × `height`. |
| `--variant print` | Renders the chart as SVG with literal colors instead of CSS custom properties, for PDF pipelines and renderers outside the browser; IDs end in `-p`. SVG format only. See [Print and PDF](../../guides/print/). |
| `--variant social` | Renders the chart on a 1200 × 630 canvas for link previews such as Open Graph images: title drawn large, source below, literal colors, no tooltips; IDs end in `-s`. SVG format only. See [Social images and PNG](../../guides/social-and-png/). |
| `--view <key>` | Renders this view of an architecture model with `views`; without it, its default view. Every variant and format renders the same view; `unknown_view` at `/render/view` for a key no view has, or a specification without views. See [Views on one model](../../guides/chart-types/#views-on-one-model). |
| `--all-views` | Renders every view of an architecture model on one HTML page: the model's title, a navigation of links to the views and each view as a section with its own figure, description and data table, the default view first. HTML and the desktop variant only, not together with `--view`; otherwise, and for a specification without views, `option_not_supported` at `/render/allViews`. One SVG per view stays the way to get the views as SVG. See [All views on one page](../../guides/chart-types/#all-views-on-one-page). |
| `-o <path>` | Writes to a file instead of standard output. |
| `--manifest <path>` | Also writes a provenance manifest of the render as JSON to `<path>`, see [Provenance](../../guides/warnings-and-errors/#provenance). |
| `--strict` | Fails on any warning. Useful in CI. |
| `--allow-warning <code>` | With `--strict`, lets warnings with this code through; they are still reported. Repeatable, for example `--allow-warning dense_chart`. |
| `--styles inline\|external` | `inline` (default): the chart carries its whole stylesheet. `external`: only its own declared colors; the page loads the output of `chartlet stylesheet` once. |
| `--hooks` | Adds the `data-*` hooks that the optional `@casoon/chartlet/interactive` module reads; see [Interaction](../../guides/interaction/#optional-javascript). Without it the output is unchanged. |
| `--diagnostics json` | Writes errors and warnings to standard error as one JSON document instead of text lines. |
| `--watch` | Renders again whenever the specification file changes, until Ctrl-C; see [Watching](#watching-and-preview). |
| `--preview` | Watches and shows the latest render on a local page; see [Watching](#watching-and-preview). |
| `--port <port>` | With `--preview`: the port to serve on (default: a free one). |

Pass `-` instead of a file name to read the specification from standard input.

With `--diagnostics json`, standard error carries exactly one JSON document, whether rendering
succeeds or not. The exit code is unchanged: 0 on success, 1 on failure.

```json
{
  "ok": false,
  "error": { "code": "strict_warnings", "path": null, "message": "strict mode rejected 1 warning(s)" },
  "warnings": [
    { "code": "text_truncated", "path": "/data/2/label", "message": "…" }
  ]
}
```

`error.path` is a JSON Pointer into the specification for specification errors, and `null` for
command-line and file errors (code `cli_error`) and for `strict_warnings`.

With `--manifest`, the manifest is written after the chart, and only when rendering succeeds:

```sh
chartlet render revenue.json -o revenue.svg --manifest revenue.manifest.json
```

```json
{
  "chartlet": "0.19.0",
  "schemaVersion": 1,
  "specHash": "sha256:…",
  "outputHash": "sha256:…",
  "format": "svg",
  "variant": "desktop",
  "idPrefix": "chartlet-…",
  "warnings": []
}
```

A render of a view of an architecture model also records `"view": "<key>"`, after `idPrefix`.
The fields are described under [Provenance](../../guides/warnings-and-errors/#provenance).

The SVG scales with its container, uses CSS classes for all styling and embeds no fonts, scripts or
external resources.

## Watching and preview

```sh
chartlet render revenue.json -o revenue.svg --watch
chartlet render revenue.json --format html --preview
```

`--watch` renders once and again after every change of the specification file, with all other
options, until Ctrl-C. It needs a file, not `-`, and `-o`. Saves that come in several writes, and
editors that replace the file, count as one change. Each render reports its warnings and errors as
a single render does — with `--diagnostics json`, one JSON document per line and render. A render
that fails, also through `--strict`, leaves the last output and manifest in place; a successful one
replaces them in one step, so a reader never sees half a file. The output is byte for byte what a
single `chartlet render` writes.

`--preview` watches too, and serves a page on `127.0.0.1` only, at the address printed to standard
output. The page shows the latest chart with its warnings and reloads after every render. After a
failed render it shows the error and marks the previous chart as stale. `-o` is optional with it;
the page itself is never part of the output. It shows SVG and HTML, not `--format png`; with
`--styles external` it loads the shared stylesheet itself. Ctrl-C stops the watch and the page.

## chartlet inspect

```sh
chartlet inspect <spec.json | -> [--disable <code>]... [--strict] [--format text|json]
```

Checks an architecture model, a flow chart or a class diagram for gaps in its documentation,
without rendering it; see
[Inspections](../../guides/warnings-and-errors/#inspections) for the rules. The findings go to
standard output, one line each, or with `--format json` as one document:

```text
gap[unlabelled_connection] at /architecture/connections/0: the connection from "Customer" to "Web shop" has no label; add a label that says what it does, such as "reads orders"
hint[isolated_component] at /architecture/components/2: "Audit log" has no connection to any other component; connect it, or keep it if it stands apart on purpose
```

```json
{ "findings": [{ "code": "isolated_component", "severity": "hint", "path": "/architecture/components/2", "message": "…", "hint": "…" }] }
```

| Option | Effect |
| --- | --- |
| `--disable <code>` | Leaves out one inspection; repeatable. An unknown code is an error. |
| `--strict` | Fails (exit code 1, `strict_inspections`) when a finding of severity `gap` remains; hints never fail. |
| `--format text\|json` | Text lines (default) or one JSON document on standard output. |
| `--diagnostics json` | Writes an error as the JSON document of `render` to standard error. |

Findings alone leave the exit code at 0. An invalid specification fails exactly as with
`chartlet render`, with its code and path, and any other chart type with
`inspections_unavailable` at `/type`. `chartlet render` never runs inspections: its output,
warnings, `--strict` and `--allow-warning` stay as they are. The npm package runs the same
inspections with `inspectChart`, the MCP server with the tool `chartlet_inspect`.

## chartlet stylesheet

```sh
chartlet stylesheet [--types bar,time,...] [--no-common]
chartlet stylesheet --common
```

Prints the shared stylesheet that charts rendered with `--styles external` rely on: for every
chart type, or with `--types` for the listed ones. `--no-common` leaves out the common part,
`--common` prints only that part. The common part comes first; each type's part styles only charts
of that type, so separately printed parts load in any order after the common part.
