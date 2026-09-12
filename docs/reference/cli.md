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
| `--format svg` | Standalone SVG with `<title>` and `<desc>` (default). |
| `--format html` | `<figure>` with caption, SVG, source and data table. |
| `--table details` | Puts the HTML data table in a native, initially closed `<details>` (default). |
| `--table visible` | Shows the data table permanently. |
| `--id-prefix <prefix>` | Stable prefix for the accessibility IDs; needed when the same chart appears twice on one page. |
| `-o <path>` | Writes to a file instead of standard output. |
| `--strict` | Fails on any warning. Useful in CI. |

Pass `-` instead of a file name to read the specification from standard input.

The SVG scales with its container, uses CSS classes for all styling and embeds no fonts, scripts or
external resources.
