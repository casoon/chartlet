---
title: Rust crate
description: Render specifications from Rust code. Item-level documentation lives on docs.rs.
order: 3
---

```sh
cargo add chartlet
```

```rust
use chartlet::{render_json, RenderFormat, RenderOptions};

let spec = std::fs::read_to_string("examples/monthly-revenue.json")?;
let output = render_json(&spec, RenderFormat::Html, &RenderOptions::default())?;
for warning in &output.warnings {
    eprintln!("{} at {}: {}", warning.code, warning.path, warning.message);
}
std::fs::write("chart.html", output.content)?;
```

With `RenderOptions { manifest: true, .. }`, `output.manifest` holds the provenance of the render
(`Manifest::to_json` writes the JSON of `--manifest`). `chartlet::sha256` is the SHA-256 it hashes
with.

An architecture model with `views` renders its default view, or the one in
`RenderOptions { view: Some("checkout".into()), .. }`. `ChartSpec::view` resolves a view to a
specification of its own, for `text_alternative` or `render_png`; `ChartSpec::view_keys` lists
the keys. `RenderOptions { all_views: true, .. }` renders every view on one HTML page, and
`Manifest::views` lists them. `RenderOptions { perspective: Some("debt".into()), .. }` shows a
perspective in place of the one the model or the view selects; `ChartSpec::with_perspective`
does the same to a specification, before `view`. `inspect_json` runs the [inspections](../../guides/warnings-and-errors/#inspections)
of an architecture model, flow chart or class diagram, leaving out the codes it is given
(`INSPECTIONS` lists them all).

`ChartSpec::tokens` and `theme` restyle a chart; `apply_theme(spec_json, theme_json)` applies a theme
file to a specification given as JSON text, and `bundled_theme("dark")` returns the text of a
bundled one, see [Customize the look](../../guides/customize/).

With the `png` feature (`cargo add chartlet --features png`), `render_png` rasterizes the print or
the social variant, see [Social images and PNG](../../guides/social-and-png/).

`render_with_metrics` accepts your own `TextMetrics` implementation if your pages use a font whose
widths differ noticeably from the built-in profile.

Every type and function is documented on [docs.rs/chartlet](https://docs.rs/chartlet).
