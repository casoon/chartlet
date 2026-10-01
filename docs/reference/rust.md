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

With the `png` feature (`cargo add chartlet --features png`), `render_png` rasterizes the print or
the social variant, see [Social images and PNG](../guides/social-and-png.md).

`render_with_metrics` accepts your own `TextMetrics` implementation if your pages use a font whose
widths differ noticeably from the built-in profile.

Every type and function is documented on [docs.rs/chartlet](https://docs.rs/chartlet).
