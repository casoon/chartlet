---
title: Rust crate
description: Render specifications from Rust code. Item-level documentation lives on docs.rs.
order: 3
---

```sh
cargo add chartlet@0.1.0-alpha.3
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

`render_with_metrics` accepts your own `TextMetrics` implementation if your pages use a font whose
widths differ noticeably from the built-in profile.

Every type and function is documented on [docs.rs/chartlet](https://docs.rs/chartlet).
