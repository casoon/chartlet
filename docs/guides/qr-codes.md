---
title: QR codes
description: Encode a link or any text as a QR code and draw it as a small, deterministic SVG — from Rust.
order: 6
---

`chartlet::qr` turns text into a QR code. It is separate from the chart specification: there is
no `"type"` for it and no CLI command, only the Rust API.

```rust
use chartlet::qr::{ErrorCorrection, QrCode};

let code = QrCode::encode("https://example.com", ErrorCorrection::Medium)?;
let svg = code.to_svg("Link to example.com", 4);
```

`to_svg` draws one unit per module with a quiet zone of the given width (four modules is the
usual minimum) and sets `role="img"` with the title as the accessible name. Scale the SVG with
CSS; `shape-rendering="crispEdges"` keeps the edges sharp.

To draw the code with something else, read the modules directly: `size()` is the number of
modules per side, `is_dark(x, y)` tells whether a module is dark.

## What gets encoded

- Text is encoded as UTF-8 bytes (`encode_bytes` takes raw bytes), in the smallest version that
  fits, from version 1 (21 × 21 modules) to version 40 (177 × 177).
- The error correction level decides how much of the code may be damaged: `Low` about 7 %,
  `Medium` (the default) about 15 %, `Quartile` about 25 %, `High` about 30 %. Higher levels need
  a larger code for the same text.
- Of the eight mask patterns, the one with the lowest penalty score is used, so the same input
  always gives the same code and the same SVG bytes.
- Data that does not fit into version 40 returns `DataTooLong` with the largest length that
  would: 2953 bytes at `Low`, 1273 at `High`.
