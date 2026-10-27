---
title: Size presets
description: A named canvas for slides, print and social images that scales type, margins and wrapping with it.
order: 4
---

`size` in the specification, `--size` on the command line and the `size` option of the MCP tool
`chartlet_render` pick a canvas for a destination:

```sh
chartlet render spec.json --size slide-16x9 -o slide.svg
chartlet render spec.json --size print-a4-landscape -o page.svg
chartlet render spec.json --size slide-16x9 --format png -o slide.png
```

A preset is more than a viewBox. The chart is laid out at the canvas divided by the preset's
scale and drawn that much larger, so type, strokes, margins, label wrapping, legend rows and the
minimum geometry of a diagram are decided at the size they are read at: a long title wraps and a
legend breaks into rows for the room the canvas gives, not for the document size.

| Preset | Canvas (CSS pixels) | Scale | Layout size | Colors |
| --- | --- | --- | --- | --- |
| `doc` | 800 × 450 | 1 | 800 × 450 | custom properties |
| `doc-wide` | 1200 × 500 | 1 | 1200 × 500 | custom properties |
| `slide-16x9` | 1920 × 1080 | 1.6 | 1200 × 675 | custom properties |
| `slide-4x3` | 1600 × 1200 | 1.6 | 1000 × 750 | custom properties |
| `social-og` | 1200 × 630 | 1.25 | 960 × 504 | custom properties |
| `social-square` | 1080 × 1080 | 1.2 | 900 × 900 | custom properties |
| `print-a4-landscape` | 1123 × 794 | 1 | 1123 × 794 | literal |
| `print-a3-landscape` | 1587 × 1123 | 1 | 1587 × 1123 | literal |
| `print-letter-landscape` | 1056 × 816 | 1 | 1056 × 816 | literal |

The print canvases are the paper at 96 dpi, so the document type size (12 px labels) is 9 pt on
paper. On a slide the 12 px label becomes about 19 px at 1920 pixels wide, and the text of a diagram
grows by the same factor. The SVG has the canvas as `width` and `height` and the layout size as
`viewBox`, so a page that scales the SVG scales all of it together.

## Precedence

From the strongest to the weakest:

1. The **`mobile` and `social` variants** (`--variant mobile`, `--variant social`) ignore `size`:
   they have a size of their own.
2. An explicit **`width` or `height`** in the specification overrides that side of the canvas.
   The scale of the preset still applies, so `"size": "slide-16x9", "width": 1000` is 1000 × 1080
   pixels laid out at 625 × 675.
3. **`size`** (the specification's, or `--size`, which replaces it).
4. The default size of 800 × 450 without a `size`.

The `print` variant (`--variant print`) with a `size` draws the preset's canvas with literal
colors. A print preset in the default variant does the same for SVG, with the IDs of the print
variant (ending in `-p`) and no hooks; the HTML format ignores the literal colors and keeps the
custom properties. PNG always rasterizes with literal colors and the bundled Inter font (see
[Social images and PNG](../social-and-png/)), so a PNG of a preset has exactly the preset's pixels at
`--scale 1`.

Without `size`, or with `doc` or `doc-wide`, the output is byte-identical to a specification
without it.

## When the content does not fit

Nothing is clipped and nothing is dropped. A chart is laid out at exactly the layout size and
shortens or wraps labels with their usual warnings (`text_truncated`, `labels_thinned`,
`value_labels_omitted`). A diagram that needs more room than the canvas keeps the room it needs:
the SVG is larger than the preset and the warning `size_exceeded` at `/size` names the size it
needs. Shorten labels, remove elements, split the diagram, or choose a larger preset.

`diagram_over_budget` uses the layout size, so a slide reads the same number of steps as a
document of that layout size, with larger type.

## Rust, JavaScript and MCP

`size` is a field of the specification, so every API takes it. In the Rust API a specification
read with `ChartSpec::from_json` resolves it; `SizePreset::dimensions`, `scale` and `is_print`
describe a preset. `chartlet_render` takes a `size` option that sets the specification's `size`.
The schema lists the names in `size`.
