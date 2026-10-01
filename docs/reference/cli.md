---
title: CLI options
sidebarLabel: CLI options
description: Output formats and options of chartlet render.
order: 2
---

```sh
chartlet render <spec.json | -> [options]
```

| Option | Effect |
| --- | --- |
| `--format svg` | Standalone SVG with `<title>` and `<desc>` and the title drawn in the chart (default). |
| `--format html` | `<figure>` with caption, SVG, source and data table; the caption is the visible title, the SVG draws none. With `mobile` in the specification, both variants behind a container query. |
| `--table details` | Puts the HTML data table in a native, initially closed `<details>` (default). |
| `--table visible` | Shows the data table permanently. |
| `--id-prefix <prefix>` | Stable ID of the chart root and prefix for its other IDs; needed when the same chart appears twice on one page. |
| `--variant mobile` | Renders the mobile variant alone as SVG, for a `<picture>` source; requires `mobile` in the specification (`missing_mobile`) and the SVG format. `desktop` (default) renders the chart at `width` × `height`. |
| `-o <path>` | Writes to a file instead of standard output. |
| `--manifest <path>` | Also writes a provenance manifest of the render as JSON to `<path>`, see [Provenance](../guides/warnings-and-errors.md#provenance). |
| `--strict` | Fails on any warning. Useful in CI. |
| `--diagnostics json` | Writes errors and warnings to standard error as one JSON document instead of text lines. |

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
  "chartlet": "0.1.0-alpha.7",
  "schemaVersion": 1,
  "specHash": "sha256:…",
  "outputHash": "sha256:…",
  "format": "svg",
  "variant": "desktop",
  "idPrefix": "chartlet-…",
  "warnings": []
}
```

The fields are described under [Provenance](../guides/warnings-and-errors.md#provenance).

The SVG scales with its container, uses CSS classes for all styling and embeds no fonts, scripts or
external resources.
