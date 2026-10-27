---
title: Customize the look
description: Change the colors of a chart, or of every chart of a project, with one set of names – a bundled theme, tokens in the specification, a theme file, or the CSS of the host page.
order: 2
---

Every color of a chart is a CSS custom property on its root, `--chartlet-accent`, `--chartlet-text`
and so on. A **token** is such a property without its prefix: `accent`, `text`, `color-1`. All
the ways to restyle a chart set tokens; they differ in where you write them down.

| Way | Where | Use it for |
| --- | --- | --- |
| [Bundled theme](#bundled-themes) | `"theme": "dark"` or `--theme dark` | A ready-made palette |
| [Tokens in the specification](#tokens-in-the-specification) | `"tokens": { … }` | One chart that needs its own colors |
| [Theme file](#theme-file) | `--theme brand.json`, the `theme` option | Many charts that share a look |
| [Host-page CSS](#host-page-css) | `.report .chartlet-root { --chartlet-accent: … }` | Charts inlined in a page that owns its colors |
| [Layer colors](#layer-colors) | `"color"` on a layer of a time chart | One series in one chart |

Tokens change colors only. Layout, text, data and the accessible description stay as they are.

## Bundled themes

`light` (the default), `dark` and, for diagrams, `blueprint` ship with chartlet as theme files, the
same kind of file you write yourself. Choose one in the specification or when rendering:

```json
{ "schemaVersion": 1, "type": "bar", "title": "Monthly revenue", "theme": "dark", … }
```

```sh
chartlet render revenue.json --theme dark
```

The files are in [`themes/`](https://github.com/casoon/chartlet/tree/main/themes) of the
repository and list every token with its value. Copy one as the start of your own.

## Tokens in the specification

`tokens` sets single colors of one chart, on top of its theme:

```json
{
  "schemaVersion": 1,
  "type": "bar",
  "title": "Monthly revenue",
  "tokens": { "accent": "#c2410c", "grid": "#e7d9cf" },
  "data": [ … ]
}
```

A value is a hex color (`#rgb`, `#rrggbb`, `#rrggbbaa`) or `rgb()`/`rgba()` with numbers. An unknown
name or another value is refused: `invalid_token` at `/tokens/<name>`.

A chart with a `background` token paints that background itself, as a dark chart does, so that the
SVG stands alone on any page.

## Theme file

A theme file is a JSON file with a `theme` to start from and the `tokens` that differ:

```json
{
  "$schema": "https://raw.githubusercontent.com/casoon/chartlet/main/schema/chartlet-theme.schema.json",
  "theme": "light",
  "tokens": {
    "text": "#1f2a1f",
    "accent": "#0f766e",
    "color-1": "#0f766e",
    "color-2": "#c2410c"
  }
}
```

Render any number of specifications with it, and they share one look:

```sh
chartlet render revenue.json --theme brand.json
```

```js
import { renderChart } from '@casoon/chartlet';
import brand from './brand.json' with { type: 'json' };

renderChart(spec, { theme: brand });      // a theme file as an object
renderChart(spec, { theme: 'dark' });     // a bundled theme by its name
```

`<Chart theme={brand} … />` takes the same, and so does the `theme` input of the `chartlet_render`
tool. `--watch` reads the file again whenever the chart is rendered.

### A theme for the whole project

A file named `chartlet.theme.json` in the working directory is applied by `chartlet render` as if
`--theme chartlet.theme.json` were given, unless another theme is named. The MCP server looks for
it in `CHARTLET_MCP_ROOT`. The manifest names the theme that was applied (`"theme":
"chartlet.theme.json"`, or the name given), so a rendered figure shows where its look came from.
Without such a file nothing changes, and `chartlet_render` adds a note that the default look is
used. Projects keep their own file; nothing writes a theme outside the directory it is given, and
`chartlet_theme_from_brand` does not replace an existing one without `overwrite`.

### From brand colors

`chartlet_theme_from_brand` (MCP) builds a light and a dark theme file from `paper`, `ink`, `muted`,
`accent` and optionally `link`, which an assistant read from a site or a style guide; the server
does not fetch anything. Values stay as given unless a pair fails the contrast rules below, and
the receipt lists each role with the value given, the value written and the pair and ratios of
every change. When only light values are given, the dark ones are derived and marked so. Fonts
are recorded in the receipt and not written: a theme file has colors only. Tokens it does not set
keep the bundled palette, and only the pairs listed under [Contrast](#contrast) are measured. A
recipe from a site to a rendered chart is in [AI agents](../ai-agents/).

A theme file only fills in what a specification leaves open: a `theme` or a token the
specification names itself wins. A token that equals what the chart's theme already gives is left
out of the output, so `--theme dark` adds nothing to a dark chart.

## Host-page CSS

Charts rendered as HTML or inline SVG read their colors from the page, so a page can set the same
properties itself:

```css
.report .chartlet-root {
  --chartlet-accent: #0f766e;
  --chartlet-text: var(--color-text);
}
```

A dark chart keeps its palette on `.chartlet-root.chartlet-theme-dark`; add the class to your
selector to override it. A token written in the specification is addressed by the chart's ID and
wins over such a rule. A chart as an `<img>` or in a PDF cannot see the page's CSS; use tokens
there.

## Layer colors

A layer of a time chart names its own color, `"color": "#0f766e"`, or one of the page,
`"color": "var(--brand, #0f766e)"`. This colors that layer only and works next to everything above.

## Which tokens exist

| Tokens | Colors |
| --- | --- |
| `text`, `muted`, `grid`, `zero`, `accent`, `background` | Text, secondary text and axis labels, grid lines, the zero line, the accent of single-series marks, the chart's ground |
| `color-1` … `color-4` | The series palette: bars, lines and groups |
| `role-blue`, `role-green`, `role-amber`, `role-teal`, `role-violet`, `role-gray`, `role-red`, each also with `-fill` | Stroke and fill of the roles in diagrams |
| `node`, `lane`, `lane-head`, `shadow` | Boxes, lanes and shadows of diagrams |
| `heat-0` … `heat-5` | The scale of node values in diagrams |
| `diverging-0` … `diverging-16` | Warming stripes and calendar heatmaps |
| `rise`, `fall` | Candlesticks |
| `net-5` … `net-8`, `parl-5` … `parl-8`, `part-5` … `part-8` | Further groups of networks, parliaments and architecture perspectives |

A token a chart does not use has no effect. The JSON Schemas list the names:
[`chartlet.schema.json`](https://github.com/casoon/chartlet/blob/main/schema/chartlet.schema.json)
for `tokens` in a specification, and
[`chartlet-theme.schema.json`](https://github.com/casoon/chartlet/blob/main/schema/chartlet-theme.schema.json)
for a theme file.

## Typography

Themes and tokens change colors only. `typography` names the fonts of a chart and tells the layout
how much room they need:

```json
"typography": { "family": "Source Sans 3", "technicalFamily": "Fira Code", "widthScale": 1.06 }
```

- `family` is the font of every text, `technicalFamily` the font of technical sublabels (the
  technology of a node, the sublabel of a participant); it defaults to `family`.
- `widthScale` multiplies the width the layout assumes for text. Chartlet's built-in metrics are an
  approximation for Latin text; measure your family against it (or pass the file with `--font`, below)
  and enter the ratio. It is the same on every platform, so the layout is deterministic.

**What is guaranteed.** The layout, the generated description and the table are the same in SVG, HTML
and PNG. What is drawn depends on whether the font is available where the SVG is shown. chartlet
never fetches a font and does not look at the fonts of the machine: the family is named in the
stack in front of Inter and the system fonts, and every render with a named family says so with the
warning `font_not_embedded`.

**Supplying a font file.** With the CLI, `--font brand.ttf` (a plain TrueType or OpenType file) makes
chartlet measure text with the advance widths of that file, under the name in `typography.family`:

| Command | Result |
| --- | --- |
| `chartlet render chart.json --font brand.ttf` | layout measured with the font; SVG names the family; `font_not_embedded` is dropped, `font_glyphs_missing` lists characters the file lacks |
| `... --font brand.ttf --embed-font` | the SVG carries the file as a base64 `@font-face`, so it looks the same everywhere |
| `... --font brand.ttf --format png` | the PNG is rasterized with the file (single face: semibold text uses it too) |

The same is available in Rust as `render_with_font` and `render_png_with_font`. The file is yours to
license: embedding it in an SVG redistributes it. Without `--font`, PNG uses the bundled Inter.

**Boundaries.** Kerning and ligatures are not applied to the measurement; only the basic multilingual
plane is mapped; scripts that need shaping (Arabic, Indic scripts) and right-to-left text are not
laid out correctly, and WOFF or font collections are refused with `invalid_font`. Characters without
a glyph are reported, and a viewer draws them in a fallback font that was not measured.

**Byte-for-byte.** For a pinned chartlet version, a specification and, with `--font`, a given font file
(its SHA-256 is the identity), the SVG, HTML and PNG bytes are the same on every machine, except
PNG edge pixels across CPU architectures (see the PNG guide). The font file is not part of the
manifest; record its hash next to it.

## Hand-drawn strokes

`"stroke": "sketch"` draws lines and shapes slightly wobbly, for essays and slides. Each corner of a
rectangle is displaced by up to about a pixel, sides and straight lines bow a little, and the
points of a polyline move by under a pixel. Text, circles, colors and contrast stay as they are.

The displacement is seeded from the SHA-256 of the specification and drawn in scene order, so the
same specification gives the same bytes; changing any field changes the wobble. The print variant
and PNG draw clean strokes. A theme file carries colors only, so `stroke` is set in the
specification. Data positions move by less than 1 px, so do not use it where exact geometry is
read off the figure.

```json
{ "schemaVersion": 1, "type": "flow", "title": "From idea to shipped", "stroke": "sketch", "flow": { "nodes": [] } }
```

## Print, social images and PNG

Tokens are resolved like the theme's own colors, so the print and social variants and PNG output
carry them, which CSS of the host page cannot do:

```sh
chartlet render revenue.json --theme brand.json --variant print -o revenue.print.svg
```

## Contrast

The bundled palettes keep their contrast ratios on their own backgrounds (see
[Support](../../reference/support/#themes)). Colors you set are measured: a pair that lies on top of
each other in the chart, with at least one color from `tokens`, and a ratio below WCAG 2.2's is
reported as `low_contrast` at the token. The chart is still rendered.

| Pair | Needs | Measured |
| --- | --- | --- |
| `text`, `muted` on `background` | 4.5:1 | Always |
| `accent` on `background` | 3:1 | Bar and line charts with one series |
| `color-1` … `color-4` on `background` | 3:1 | The series a bar, line or time chart draws |
| A role's outline on its fill and on `background`, `text` on its fill | 3:1, 3:1, 4.5:1 | Diagrams, for the roles the diagram draws |

A translucent color is measured as it looks over the background. Colors of other chart types, such
as the scales of stripes and calendars, are not measured.
