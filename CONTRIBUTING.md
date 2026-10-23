# Contributing

## Reviewing changes to the rendered examples

`tests/golden.rs` checks that every example in `examples/` renders to exactly its committed SVG,
for the desktop, mobile, print and social variants it lists, and to its committed HTML. When a
change moves pixels, those tests fail; to see what changed, write the visual report:

```sh
cargo test --test golden visual_report -- --ignored --nocapture
```

The test prints the path of `index.html`, below `tmp/golden-report/` in Cargo's target directory.
For every changed reference the report shows the reviewed SVG and the current rendering side by
side, as an overlay or as a difference, with the first differing bytes. Unchanged references and
examples without a reference in `tests/golden.rs` are hidden until you tick their boxes. Each SVG
is embedded as an image of its own, so IDs and styles of different charts never interact.

The report never accepts a change. Once a new rendering is right, write it over the reference
with the command the report gives, for example:

```sh
cargo run -- render examples/release-flow.json -o examples/release-flow.svg
cargo run -- render examples/tenant-architecture.json --variant mobile -o examples/tenant-architecture.mobile.svg
```

and commit the SVG with the change that caused it, together with the HTML
(`cargo run -- render examples/release-flow.json --format html -o examples/release-flow.html`).

`tests/generated.rs` renders diagrams generated from fixed seeds — long labels, cycles, nested
boundaries, narrow canvases — and checks that each renders, renders the same twice and is
well-formed SVG. A failure names the seed, so it can be reproduced.
