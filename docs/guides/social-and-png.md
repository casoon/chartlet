---
title: Social images and PNG
description: A 1200 × 630 variant for link previews, and PNG output with a bundled font for the same bytes on every machine.
order: 3
---

## Social variant

A link to a page with a chart shows a preview image in chats and social networks, usually from the
page's `og:image`. The social variant draws the chart for that image from the same
specification:

```sh
chartlet render revenue.json --variant social -o revenue.social.svg
```

- The canvas is 1200 × 630 pixels, the common size of Open Graph images, and opaque in the
  chart's theme.
- The title is drawn at 44 pixels, on two lines if it needs them; a second line that is still
  too wide is shortened (`text_truncated` at `/title`). The source, when the specification has
  one, is the last line.
- The chart is laid out without its title at the size that remains, two thirds of it, and drawn
  one and a half times as large, so that its labels stay readable where a preview shows the image
  at half its size. A label that this size has to shorten is reported with its usual code.
- The stylesheet is resolved as in the [print variant](./print.md): literal colors, no custom
  properties, so that renderers outside the browser draw it. A layer color declared as
  `var(--name)` is drawn in the text color and reported as `color_not_resolved`.
- The marks carry no tooltips; the root keeps the title and description as its accessible name.
- IDs end in `-s` (`revenue-s`, `revenue-s-title`, …).

`--variant social` with `--format html` fails with `option_not_supported` at `/render/variant`.

Most networks do not accept SVG as `og:image`. Rasterize the social variant to PNG, with chartlet
as below or with another renderer that supports class selectors in `<style>`.

## PNG

PNG output is an optional feature of the crate. The default build, the WebAssembly build and the
npm package do not contain it.

```sh
cargo install chartlet --features png
chartlet render revenue.json --variant social --format png -o revenue.png
chartlet render revenue.json --format png --scale 2 -o revenue@2x.png
```

- A PNG is rasterized from the print variant, the chart at `width` × `height`, or with
  `--variant social` from the social variant. `--variant desktop` is taken as `print`;
  `--variant mobile` rasterizes the mobile layout with the same literal colors, and fails with
  `missing_mobile` for a chart without `mobile`.
- `--scale` sets the image pixels per SVG pixel, 0.25 to 4 (default 1); outside that range it
  fails with `invalid_scale`. `--scale 2` gives a 1600 × 900 image of an 800 × 450 chart.
- Text is drawn with Inter (regular and semibold, Latin and Latin Extended), which the crate
  bundles under the SIL Open Font License 1.1; system fonts are never used. Weights above 600
  are drawn semibold. Characters outside these subsets, such as Greek or Cyrillic, have no
  glyphs.
- The same specification, options and chartlet version give the same PNG bytes. A new version of
  chartlet may change them, as it may change the SVG.
- `--manifest` is not available with `--format png`. Warnings and `--strict` work as for SVG.

The PNG is an image without a text alternative: give the `<img>` an `alt` text, or the
`og:image:alt` property, that carries the chart's message.

### Rust

```rust
use chartlet::{ChartSpec, PngOptions, Variant, render_png};

let spec = ChartSpec::from_json(&std::fs::read_to_string("revenue.json")?)?;
let output = render_png(&spec, &PngOptions { variant: Variant::Social, scale: 1.0 })?;
std::fs::write("revenue.png", output.png)?;
```

with `chartlet = { version = "0.12", features = ["png"] }` in `Cargo.toml`.

## Astro and the render API

`renderChart` takes `variant: 'social'` together with `format: 'svg'`; it returns the SVG, not a
PNG:

```js
const { content } = renderChart(spec, { format: 'svg', variant: 'social' });
```
