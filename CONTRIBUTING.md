# Contributing

## Rust version

`rust-toolchain.toml` names the Rust version for development and CI; rustup installs it with
its components and the WebAssembly target on the first `cargo` call (or with
`rustup toolchain install`). A new stable release can bring new lints, so the version moves only
on purpose: change `channel`, run `cargo fmt --check`, the three `cargo clippy` calls and
`cargo test` from `.github/workflows/ci.yml`, fix what they report, and commit that alone.

The crate declares `rust-version = "1.88"` in `Cargo.toml`, the oldest compiler it is meant to build
with; development and CI use the newer pinned version. Raise it only when the code needs a newer
language or library feature, and then update the installation docs and the README too.

## Reviewing changes to the rendered examples

`tests/golden.rs` checks that every example in `examples/` renders to exactly its committed SVG,
for the desktop, mobile, print and social variants it lists, and to its committed HTML. When a
change moves pixels, those tests fail. `tests/schema.rs`, also part of `cargo test`, validates
every `examples/*.json` against `schema/chartlet.schema.json`. To see what changed, write the
visual report:

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

`tests/routing.rs` measures every diagram example: crossings, total edge length, runs closer than
12 px, edges behind a foreign step and steps off the 4 px grid. A layout change may lower the
numbers in its table, never raise them; `cargo test --test routing -- --ignored --nocapture`
prints the new rows.

## The diagram pipeline

A diagram passes the same stages on every render. Knowing which stage owns a feature tells you
where else it has to be carried.

1. **Views and deployments.** An architecture model with `views` never renders as a whole:
   `ChartSpec::resolve_view` (`src/spec/view.rs`) turns the chosen view into a plain
   architecture specification of its own — the selected components in model order, the
   connections between them, the boundaries that hold them, the notes and shared ranks it keeps.
   A view with an `environment` goes through `resolved_deployment` (`src/spec/deployment.rs`)
   instead, which expands instances into components, hosts into boundaries and instance
   connections (listed or derived) into connections. The `ResolvedView` remembers where every
   element came from; `model_path` turns a JSON Pointer into the resolved diagram back into one
   into the model, so that errors and warnings point at what the author wrote.
2. **Validation.** `ChartSpec::validate` (`src/spec/mod.rs`) runs the cross-cutting checks of
   `src/spec/validate.rs` — size, theme, texts, numbers, value axes, blocks that belong to another
   type — and then the module of the type (`src/spec/flow.rs`, `architecture.rs`, `classes.rs`,
   …). Every error is a `ChartError` with a stable code, the JSON Pointer of the field and a fix.
   Every view is resolved and validated as a diagram of its own. Everything after this stage
   trusts the specification and does not check again.
3. **Adapters.** `src/flow/mod.rs` (flow charts), `src/state.rs`, `src/classes.rs` and
   `src/architecture.rs` translate their specification into the engine's `Diagram`
   (`src/flow/model.rs`): nodes with a `Shape` and texts, edges with their ends and marks, lanes,
   groups and frames, notes (`src/flow/notes.rs`) and blocks (`src/flow/blocks.rs`, a package laid
   out in an orientation of its own). Sequence diagrams (`src/sequence.rs`), trees (`src/tree.rs`)
   and networks (`src/network.rs`) have layouts of their own and share only shapes, focus classes
   and roles with the engine.
4. **The layered engine** (`src/flow/`). `measure.rs` wraps every text and builds the layered
   graph of `graph.rs` (cycles broken, layers assigned, placeholders inserted, layers ordered for
   few crossings); `place.rs` places the nodes across the flow, keeps frames clear of nodes they
   do not hold (`frames.rs`) and lines up the main path (`straighten.rs`); `ports.rs` decides
   where edges meet their nodes, honouring `fromSide` and `toSide`; `gaps.rs` sizes the spaces
   between layers and their tracks; `route.rs` gives every edge its way and its label a place;
   `trunks.rs` merges edges into shared trunks. `draw.rs`, `edges.rs`, `nodes.rs` and `shapes.rs`
   turn the plan into scene elements. `layout_with_trunks` tries both orientations for `auto` and
   grows the canvas until the edge labels fit (`canvas_too_small`).
5. **Scene and render.** The adapter adds what lies outside the engine — the note band, the
   legends of roles (`src/roles.rs`), parts and values — to the `Scene` (`src/scene.rs`), a flat
   list of elements. `src/render.rs` writes it as SVG; `base_style` assembles the stylesheet from
   groups that apply only when the chart needs them, each guarded by `when(condition, STYLE)`.
6. **Text alternatives.** `automatic_description` (`src/describe.rs`) dispatches to the
   description of the type (`src/flow/describe.rs`, `architecture::description`, …), and each type
   has a `data_table`. The words themselves, per locale, live in `src/text/`; the facts about a
   graph's shape in `src/structure.rs`.
7. **Hooks.** With the `hooks` render option, `Element::NodeGroup`, `Element::EdgeGroup` and
   `Hook::Object` become `data-chartlet-node`, `data-chartlet-edge` and the attributes of frames
   and lanes; `tests/hooks.rs` checks that hooks change nothing else, and
   `docs/guides/interaction.md` lists them.

### Adding a diagram feature

- [ ] Specification: the field in `src/spec/` and in `schema/chartlet.schema.json`, with a
      description.
- [ ] Validation: every new rule with its own code and the JSON Pointer of the field, plus a test.
- [ ] Views and deployments: carry the field into the resolved diagram, leave out what refers to
      hidden elements, and map its paths back in `model_path`.
- [ ] Adapter: translate the field into the engine's `Diagram`, or draw it in the type's own
      layout.
- [ ] Engine: only when the layout itself has to change; both orientations, frames and trunks.
- [ ] Description and data table, in both locales (`src/text/`).
- [ ] Hooks: a new element with an `id` carries it.
- [ ] Roles: a new node or edge kind keeps its shape under every role.
- [ ] MCP: the kinds and notes in `packages/chartlet-mcp/src/diagrams.mjs`.
- [ ] Docs: `docs/reference/specification.md` and `docs/guides/chart-types.md`.
- [ ] `CHANGELOG.md`, under Unreleased.
- [ ] A golden example in `examples/`, listed in `tests/golden.rs`, with its SVG and HTML.

## Releasing

Before a version is published, from a clean checkout:

- [ ] `cargo fmt --check`, the three `cargo clippy` calls and the two `cargo test` calls (the second
      with `--features png`) from `.github/workflows/ci.yml`; the golden and schema tests are part
      of `cargo test`. CI runs on the version tag; run them locally before it.
- [ ] `npm test` in `packages/chartlet` and `packages/chartlet-mcp`.
- [ ] `node scripts/smoke-install.mjs`: packs both npm packages, installs the tarballs into an empty
      project outside the repository, renders a chart and a diagram through WebAssembly (the diagram
      against its golden SVG), checks that `chartlet.wasm`, `chartlet.css` and the schema resolve from
      `node_modules`, and starts the MCP server to list its tools and resources. It needs the Rust
      toolchain only to pack, not to install or run. It also checks the tarball contents, that the
      manifest names the installed version, that the bundled schema is the schema of the release,
      that the installed renderer satisfies the range of `chartlet-mcp`, and renders through the MCP
      tools. `--cli` adds `cargo install --locked --path .` into a temporary root and compares the
      CLI output with the WebAssembly output. The release workflow runs the default checks again on
      the tag, before the publish.
- [ ] Versions: crate, both packages and the `@casoon/chartlet` range of `@casoon/chartlet-mcp`
      agree; `CHANGELOG.md` moves `[Unreleased]` to the version; README and the site are current.
      `node scripts/check-versions.mjs` compares the crate, both packages, the Rust version and
      the statements in README and docs that repeat them, and the install paths of the README. The
      site takes its version from the same manifests; open the deployed page and the playground
      after the Pages build.
- [ ] After publishing: `node scripts/smoke-install.mjs --published` installs the published versions
      of both packages from the registry into an empty project and repeats the checks; run
      `cargo install chartlet` once in a clean environment as well.
- [ ] The compatibility contract in `docs/reference/support.md` still matches what the release does:
      the `@casoon/chartlet` range of `chartlet-mcp`, `engines`, `rust-version`, the migration
      example (use the removal or rename of the release before it).

Publishing, once the version is committed and merged to `main`:

- [ ] `cargo publish --locked` for the crate.
- [ ] Tag the merge commit: `git tag vX.Y.Z && git push origin vX.Y.Z`. The tag starts
      `.github/workflows/release.yml`: it checks that the tag matches the crate and both packages,
      runs the smoke install, then waits for approval in the `npm` environment (Required Reviewer)
      and publishes `@casoon/chartlet` and `@casoon/chartlet-mcp` through npm Trusted Publishing —
      no token, provenance attached. Never run `npm publish` from a machine: a package published
      there carries no provenance.
