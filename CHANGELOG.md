# Changelog

All notable changes to chartlet are listed here. chartlet follows
[Semantic Versioning](https://semver.org/): until 1.0, a minor release (0.2, 0.3, …) may change the
specification; a patch release never does. Each entry starts with the area it concerns; the details
are in the [documentation](https://casoon.github.io/chartlet/docs/).

## [Unreleased]

### Added

- **Venn diagrams:** `"type": "venn"` draws two or three sets as circles at fixed places, with a
  label and a count for each region (`only A`, `A and B`, …); text sits inside where it fits and on
  leader lines otherwise. More than three sets are refused. Details:
  [Venn diagrams](https://casoon.github.io/chartlet/docs/guides/chart-types/#venn-diagrams).

- **Value chain maps:** `"type": "valuemap"` places components by visibility to the user (up) and
  evolution (right, in the stages genesis, custom, product and commodity) at positions given in the
  specification, with dependencies as lines, planned movement as dashed arrows and an inertia mark.
  Details: [Value chain maps](https://casoon.github.io/chartlet/docs/guides/chart-types/#value-chain-maps).

- **User journeys and service blueprints:** `"type": "journey"` follows a customer through stages with
  rows of actions, touchpoints and thoughts and a sentiment line from -2 to 2 with every value
  written; `"display": "blueprint"` adds the frontstage, backstage and support rows between the
  lines of interaction and visibility. On a narrow canvas every stage is a block of its own.
  Details: [User journeys and service blueprints](https://casoon.github.io/chartlet/docs/guides/chart-types/#user-journeys-and-service-blueprints).

- **Story maps:** `"type": "storymap"` draws a backbone of activities, the user tasks under them and
  the stories in release slices parted by labeled cut lines (`MVP`, `Release 2`); on a narrow
  canvas it reads as a list with the release beside each story. The table lists every story by
  activity, task and release. Details: [Story maps](https://casoon.github.io/chartlet/docs/guides/chart-types/#story-maps).

- **Kanban boards:** `"type": "kanban"` draws the states of work as columns with cards. A column
  can carry a limit, written as `3/4` and marked with a sign and its own outline when exceeded; a
  card has an owner, a tag and a `blocked` sign. The description counts the cards per column and
  names the blocked ones. Details:
  [Kanban boards](https://casoon.github.io/chartlet/docs/guides/chart-types/#kanban-boards).
- **Documentation:** [Updating a diagram](https://casoon.github.io/chartlet/docs/reference/layout-stability/)
  reports, measured on paired before/after diagrams, how much moves when a node or connection changes:
  the order of the declarations is a layout input, so append new nodes and keep the existing order.
  `scripts/layout-stability.mjs` reproduces it.

- **Limits:** a specification of more than 4 MiB is refused with `spec_too_large` before it is parsed.
  [Limits and performance](https://casoon.github.io/chartlet/docs/reference/limits/) lists what is
  enforced and what is only advice, with measured render times, memory and output sizes, a
  reproducible benchmark (`scripts/bench.mjs`) and budgets. The seeded diagram tests now also check
  that everything is drawn on the canvas.

- **Playground:** the site has a [playground](https://casoon.github.io/chartlet/playground/) that runs
  the compiler as WebAssembly in the browser, with JSON and form editing, diagnostics at the field
  they name, checked corrections, undo and download of the specification, SVG and HTML.

- **Diagnostics:** `--advice` (CLI), `advice: true` (WebAssembly, MCP `chartlet_validate_spec`) and
  `advise_json` (Rust) add structured help to `canvas_too_small` and `text_truncated`: the field,
  the documentation and a layout correction the compiler checked by rendering it, as a JSON Patch
  that is never applied for you. A correction of content is guidance without a patch. Details:
  [Actionable diagnostics](https://casoon.github.io/chartlet/docs/guides/warnings-and-errors/#actionable-diagnostics).

- **Brand typography:** `typography` names the font `family`, the `technicalFamily` for sublabels and a
  `widthScale` the layout reserves for them, so a brand font no longer needs a color-only theme to
  fit. `chartlet render --font brand.ttf` measures text with a font file you supply, `--embed-font`
  carries it in the SVG and `--format png` rasterizes with it; nothing is fetched, and a family that
  is only named is disclosed with `font_not_embedded`. Missing glyphs are listed as
  `font_glyphs_missing`. Details:
  [Typography](https://casoon.github.io/chartlet/docs/guides/customize/#typography).

- **Fishbone diagrams:** `"type": "fishbone"` draws one effect at the head of a spine and up to six
  categories as bones above and below it, each with causes and sub-causes on horizontal lines;
  labels are never set on a diagonal. The description and the table list the effect, the
  categories and the causes in order. Details:
  [Fishbone diagrams](https://casoon.github.io/chartlet/docs/guides/chart-types/#fishbone-diagrams).
- **Readability budget:** a diagram with more than 9 steps, states, components or classes or more
  than 12 edges, transitions, connections or relations (a sequence diagram: more than 6
  participants or 16 messages, a tree: more than 24 nodes) is reported as `diagram_over_budget`
  with the advice to split it into an overview and details. The budget grows with the canvas, the
  counts are guidance and not a limit, `--allow-warning diagram_over_budget` lets it through, and
  `chartlet inspect` reports it as a hint. Details:
  [Warnings and errors](https://casoon.github.io/chartlet/docs/guides/warnings-and-errors/).

- **Project theme:** a `chartlet.theme.json` in the working directory (the MCP server: in
  `CHARTLET_MCP_ROOT`) is applied like `--theme` unless another theme is given. The provenance
  manifest and the `chartlet_render` result name the theme that was applied, and without a theme
  file `chartlet_render` notes that the default look is used. Details:
  [Customize the look](https://casoon.github.io/chartlet/docs/guides/customize/).

- **MCP server:** `chartlet_theme_from_brand` turns brand colors an assistant read from a site into
  a light and a dark theme file with a receipt of every adjustment for contrast; fonts are
  recorded and not applied. `chartlet_check_coverage` compares a redrawn diagram with the
  inventory of its source: kept, changed, merged, ambiguous, missing and added, by ids, endpoints
  and members rather than labels alone. Details:
  [AI agents](https://casoon.github.io/chartlet/docs/guides/ai-agents/).

- **Agent skill:** `skills/chartlet/` is a portable skill (a `SKILL.md` with references) that leads
  an assistant from a request through data inspection, a starter, validation and rendering to the
  delivered file, with the MCP tools when they exist and the CLI otherwise. It never aggregates,
  invents data or drops relationships to make a figure fit. Install, update, uninstall and
  troubleshooting for Claude Code and Codex are in the skill's references; the part that has been
  run is listed there. Details:
  [AI agents](https://casoon.github.io/chartlet/docs/guides/ai-agents/#agent-skill).

- **Recipes:** five task-oriented examples (a report from CSV, a blog explanation, architecture in
  views, a release flow, a slide and print figure) with a shared brand theme, sample data marked as
  such, the exact commands and a Node, Astro and Markdown route, in `examples/recipes/` and in the
  showcase. Details:
  [Recipes](https://casoon.github.io/chartlet/docs/guides/recipes/).

- **Time charts:** `"stack": "stream"` on a pane of area layers centers the stack around a moving
  baseline instead of zero (a streamgraph); the value axis becomes a scale bar for thickness and
  each stream is named at its right end. Details:
  [Streamgraphs](https://casoon.github.io/chartlet/docs/guides/chart-types/#streamgraphs).

- **Box plots:** `"boxDisplay": "beeswarm"` draws every observation as a dot moved sideways just
  far enough that no two overlap, optionally over the box (`"showBox": true`). Details:
  [Box plots](https://casoon.github.io/chartlet/docs/guides/chart-types/#box-plots).

- **Box plots:** `"boxDisplay": "ridgeline"` draws one density curve per group, stacked with a
  slight overlap along a shared value axis, on one scale so that heights compare. Details:
  [Box plots](https://casoon.github.io/chartlet/docs/guides/chart-types/#box-plots).

- **Range bars:** `"display": "dumbbell"` draws each span as an open dot at the low value and a
  filled dot at the high value joined by a line, with both values written beside their dots, and
  `"sort": "change"` puts the widest span first. Details:
  [Dumbbells](https://casoon.github.io/chartlet/docs/guides/chart-types/#dumbbells).

- **Hand-drawn strokes:** `"stroke": "sketch"` draws lines and shapes slightly wobbly, for essays and
  slides. The displacement is seeded from the specification (same bytes every time), text stays
  crisp, the print variant and PNG stay clean. Details:
  [Customize the look](https://casoon.github.io/chartlet/docs/guides/customize/#hand-drawn-strokes).

- **Cycles:** `"type": "cycle"` draws three to eight steps on a ring, clockwise from twelve
  o'clock, with arrows along the ring from each step to the next and from the last back to the
  first. The labels stand outside the ring beside numbered dots or inside boxes on it; an optional
  `hub` in the middle is fed by every step through a link that can carry a `hubLabel`. Roles, focus
  links and the blueprint theme work as in other diagrams, and the description reads the cycle in
  order. Details: [Cycles](https://casoon.github.io/chartlet/docs/guides/chart-types/#cycles).
  Examples `improvement-cycle`, `growth-flywheel`.

- **Slope and rank charts:** `"slope": true` on a `line` chart of two categories draws one line per
  series from its first value to its second, and `"rank": true` plots the rank of each series
  across several categories, 1 at the top. The names stand at both ends instead of a legend,
  overlapping names are spread apart, and the description names the biggest rise and fall.
  Details: [Slope and rank charts](https://casoon.github.io/chartlet/docs/guides/chart-types/#slope-and-rank-charts).

- **Heatmaps:** `"type": "heatmap"` shows values in a grid of rows and columns on a sequential or
  a diverging scale, with the value written into the cell where it fits and a key below; or
  categories, each with a fill and a symbol. Cells that are not listed are drawn as missing, rows
  and columns can be sorted by label or value, and the table has every cell. Details:
  [Heatmaps](https://casoon.github.io/chartlet/docs/guides/chart-types/#heatmaps).

- **MCP server:** the resource `chartlet://guide/types` maps what to show to the type and its fields
  for every type of the schema, and the prompts `diagram`, `chart` and `redraw` lead from a request
  through validation to the rendered result. Details:
  [AI agents](https://casoon.github.io/chartlet/docs/guides/ai-agents/).

- **Size presets:** `size` in the specification and `chartlet render --size` pick a canvas by name
  (`doc`, `doc-wide`, `slide-16x9`, `slide-4x3`, `social-og`, `social-square`, `print-a4-landscape`,
  `print-a3-landscape`, `print-letter-landscape`); an explicit `width` or `height` still wins. See the
  [specification](https://casoon.github.io/chartlet/docs/reference/specification/).

### Changed

- **Releases:** both npm packages are published by a Release workflow on the version tag, with npm Trusted Publishing (provenance, no stored token), after approval in the `npm` environment.
- **Docs:** [Support matrix](https://casoon.github.io/chartlet/docs/reference/support/#versioning-and-compatibility)
  states the compatibility contract: reproducibility, byte stability, supported runtimes,
  experimental features, pinning before 1.0 and the migration policy with an upgrade example.
- **Site:** the start page leads with a Sankey diagram rendered during the build, a terminal recording of
  real `chartlet render` runs (a silent success, an error with code and path, a warning that names
  the fix) and three charts from the showcase; the CLI and Astro snippets are in
  [Quickstart](https://casoon.github.io/chartlet/docs/getting-started/quickstart/) and
  [Astro](https://casoon.github.io/chartlet/docs/guides/astro/).
- **Contributors:** `scripts/smoke-install.mjs` installs the packed npm packages into an empty project
  and checks rendering, assets and the MCP server; it is part of the release checklist in
  `CONTRIBUTING.md`, which no longer says the crate has no `rust-version`.

### Fixed

- **Flow groups:** groups whose steps interleave in the layers no longer blow the canvas up (a
  grouped crate diagram asked for about 514 million pixels). Where the frames cannot all be cleared,
  the layout stays compact and `group_overlap` names the groups.

## [0.21.0] - 2026-10-24

### Breaking

- **Topic map and knowledge landscape:** the experimental types `topicmap` and `atlas` are gone,
  with their blocks, the area picker of the HTML figure, their example files, the stylesheet part
  and their warnings. A specification with either type is refused with `invalid_spec`.

### Added

- **Themes:** `tokens` in a specification set single colors of the palette by the name of the
  `--chartlet-*` custom property without its prefix; they reach the print and social variants and
  PNG. Details: [Customize the look](https://casoon.github.io/chartlet/docs/guides/customize/).
- **Themes:** a theme file of your own colors is applied to any number of specifications with
  `--theme <name|file>`, the `theme` option of `@casoon/chartlet` and `<Chart>`, the `theme` input
  of `chartlet_render` and `chartlet::apply_theme`. `light`, `dark` and `blueprint` ship as such
  files in `themes/`.
- **Themes:** a color set in `tokens` that lies on another with less contrast than WCAG 2.2 asks for is
  reported as `low_contrast` at the token. Details: [Contrast](https://casoon.github.io/chartlet/docs/guides/customize/#contrast).
- **CLI:** `chartlet stylesheet` starts its output with `/* generated by chartlet - do not edit */`.
  Details: [Formatters and linters](https://casoon.github.io/chartlet/docs/guides/javascript/#formatters-and-linters).
- **Schema:** `chartlet-theme.schema.json` for theme files, and `tokens` in the specification schema.
- **Site:** the Customize page shows five ways to restyle a chart, each rendered.
- **Site:** the showcase gains the sequence diagram `sign-in-sequence`, the architecture diagram
  `tenant-architecture` and a QR code, the first example of `chartlet::qr`.

### Changed

- **Rust:** the crate declares `rust-version = "1.88"`, which README and the installation page already
  stated; it builds with that version, with and without `png`.
- **Site:** the showcase is grouped by topic with a jump list and shows one example per variant.

## [0.20.0] - 2026-10-24

### Added

- **Architecture diagrams:** `views` show one model as several diagrams: a view has a `key`, a
  `title`, a `description` and the `components` it shows, with their connections (or those listed
  by their new `id`), boundaries and main path; a dynamic view lists `steps`, numbered. Details:
  [Views](https://casoon.github.io/chartlet/docs/reference/specification/#views). Example `shop-views`.
- **Architecture diagrams:** one view is rendered — `--view`, `view` in npm and MCP,
  `RenderOptions::view`, else `defaultView`, else the first — never the whole model; description and
  table show what the view shows and how much of the model that is, and the manifest records `view`.
- **Architecture diagrams:** `--all-views` (`allViews` in npm, MCP and the Astro `Chart`,
  `RenderOptions::all_views`) renders every view on one HTML page without a script, with a
  navigation and one section per view. Details:
  [All views on one page](https://casoon.github.io/chartlet/docs/guides/chart-types/#all-views-on-one-page).
  Example `shop-views.all.html`.
- **Architecture diagrams:** with `--all-views`, the manifest lists the views in `views` and npm
  returns every view's text alternative in `alternatives` with `alternative: true`; SVG, the other
  variants and `--view` are refused (`option_not_supported`). Single-view exports are unchanged.
- **Architecture diagrams:** a view may set its own `width`, `height` and `mobile` in place of the
  model's, in ordinary and deployment views; they are checked like the model's, with diagnostics and
  layout warnings at the view.
- **Architecture diagrams:** `tags` on components and connections (never drawn) and
  `include`/`exclude` on a view select what the view shows besides its lists; what a view lists by
  id is always shown. Details:
  [Views](https://casoon.github.io/chartlet/docs/reference/specification/#views).
- **Architecture diagrams:** `perspectives` keep categorical metadata, such as the owning team, apart
  from the drawing: `categories` that list `components` and connections by `id`. Only the one
  selected with `perspective` is shown, in colors with names and a legend. Details:
  [Perspectives](https://casoon.github.io/chartlet/docs/guides/chart-types/#perspectives).
  Example `service-perspectives`.
- **Architecture diagrams:** `--perspective <key>` (`perspective` in npm and MCP `chartlet_render`,
  `RenderOptions::perspective`, `ChartSpec::with_perspective`) picks the perspective at render time
  in every view, variant and format; the manifest records it, `specHash` stays that of the model.
- **Architecture diagrams:** a view with an `environment` shows where the model runs, with nested
  `hosts`, `instances` of components and the `instanceConnections` it lists; `chartlet inspect`
  counts a component run there as shown. Details:
  [Environments and instances](https://casoon.github.io/chartlet/docs/guides/chart-types/#environments-and-instances).
  Example `api-environments`.
- **Architecture diagrams:** `"deriveConnections": true` on a deployment view derives the instance
  connections from the model, to the nearest instances by shared hosts; a listed instance connection
  now also takes the sides it leaves out from its model connection. Details:
  [Environments and instances](https://casoon.github.io/chartlet/docs/guides/chart-types/#environments-and-instances).
- **Architecture diagrams:** provided and required interfaces: `provides` on a component draws each
  interface as a ball on a stem, and a connection with `toInterface` ends in a socket around it;
  description, table, views and hooks follow. Details:
  [Architecture diagrams](https://casoon.github.io/chartlet/docs/guides/chart-types/#architecture-diagrams).
  Example `payment-interfaces`.
- **Architecture diagrams:** `symbol` draws one of 16 generic stroke symbols (`browser`,
  `api-gateway`, `message-broker`, …) above a component's label; the kind keeps its shape. Details:
  [Architecture diagrams](https://casoon.github.io/chartlet/docs/reference/specification/#architecture-diagrams).
  Example `shop-symbols`.
- **Architecture diagrams:** `connections[].direction` — `forward` (default), `both` (arrowheads at
  either end) or `none` (no arrowhead); tooltip, description and table say which. Details:
  [Architecture diagrams](https://casoon.github.io/chartlet/docs/guides/chart-types/#architecture-diagrams).
- **Class diagrams:** `packages` frame the classes that name them with `in`, nested up to four deep;
  relations still cross package borders. Details:
  [Class diagrams](https://casoon.github.io/chartlet/docs/reference/specification/#class-diagrams).
  Example `shop-modules`.
- **Class diagrams:** `orientation` on a package lays out its classes that way, as one block;
  `package_orientation_ignored` says when it cannot. Details:
  [Class diagrams](https://casoon.github.io/chartlet/docs/reference/specification/#class-diagrams).
  Example `invoice-model`.
- **Class diagrams:** `relations[].direction` on an association — `forward`, `both` or `none` — says
  which way it can be navigated; other kinds refuse it (`option_not_supported`).
- **Trees:** `distances` (labels and a distance matrix) instead of `nodes` builds a dendrogram, with
  `linkage` `average` (default), `complete` or `single`. Details:
  [Dendrograms](https://casoon.github.io/chartlet/docs/guides/chart-types/#dendrograms). Example
  `climate-similarity`.
- **Networks:** `"type": "network"`: nodes as dots joined by edges, on a circle or in rings around
  a `center`, with groups, `dash`, `weight` and `directed`; no force layout, so the same input gives
  the same picture. Details: [Networks](https://casoon.github.io/chartlet/docs/guides/chart-types/#networks).
  Examples `co-authors`, `service-dependencies`.
- **Diagrams:** `role` — `critical`, `planned` or `deprecated` — on nodes and edges of flow, state,
  network, tree, class and architecture diagrams: a role color plus a sign that needs no color,
  named in legend, description and table. Details:
  [Diagram fields](https://casoon.github.io/chartlet/docs/reference/specification/#diagram-fields).
  Examples `billing-migration`, `payment-redesign`.
- **Diagrams:** `notes` on flow, class and architecture diagrams: a note `on` a step, class or
  component stands beside it, one on a package or boundary inside that frame, one without `on` below
  the diagram; none adds an edge. Description and table name them. Examples `subscription-model`,
  `payment-notes`.
- **Diagrams:** `mainPath` of flow charts and architecture diagrams also takes
  `{ "from": "a", "to": "b" }`, the shortest way between the two (`main_path_unreachable` when there
  is none).
- **Diagrams:** `weight` on a flow edge or architecture connection draws its line from 1 px to 5 px
  for the heaviest; description and table give every weight. Example `request-volume`.
- **Diagrams:** `sameRank` in flow charts and architecture diagrams puts sets of steps or components
  at one position along the flow, such as parallel reviews; a layout hint that adds no edge. Example
  `data-tier`.
- **Diagrams:** `fromSide` and `toSide` on a flow edge or architecture connection pin the side it
  leaves and reaches, named after the flow so that they hold in either orientation. Details:
  [Port sides](https://casoon.github.io/chartlet/docs/guides/chart-types/#port-sides). Example
  `contract-review`.
- **Diagrams:** `"sharedTrunks": true` in flow charts and architecture diagrams draws edges that
  leave or reach one node together on one branching line, where they read the same; each is still
  listed. Examples `service-fan-out`, `order-fulfilment`.
- **Diagrams:** `spacing` — `compact`, `normal` (default) or `roomy` — scales the gaps of flow,
  state, class and architecture diagrams. Details:
  [Spacing](https://casoon.github.io/chartlet/docs/guides/chart-types/#spacing).
- **Diagrams:** `value` on a node of a flow chart or tree fills its box from a scale of six blues,
  with a key named by `valueLabel` and text that keeps 4.5:1 contrast. Example `department-budgets`.
- **Diagrams:** `parts` on a node of a flow chart or tree draws a stripe of shares at the bottom of
  its box, with one legend for the diagram. Example `claim-handling`.
- **Interaction:** with `hooks: true`, diagrams carry their ids for a page script —
  `data-chartlet-node`, `data-chartlet-edge`, `data-chartlet-frame` and `data-chartlet-lane`.
  Details: [The hooks](https://casoon.github.io/chartlet/docs/guides/interaction/#the-hooks).
- **CLI:** `chartlet inspect` checks architecture, flow and class diagrams for gaps in their
  documentation without rendering, as text or `--format json`, with `--disable` and `--strict`; in
  Rust `inspect_json`. Details:
  [Inspections](https://casoon.github.io/chartlet/docs/guides/warnings-and-errors/#inspections).
- **CLI:** `chartlet render --watch` renders again whenever the file changes, and `--preview`
  serves the latest chart on a local page that reloads. Details:
  [Watching and preview](https://casoon.github.io/chartlet/docs/reference/cli/#watching-and-preview).
- **npm:** `@casoon/chartlet/builder` builds architecture and flow specifications in code
  (`component`, `node`, `connect`, `fanOut`, `chain`, …) and throws at the call on a wrong id. Details:
  [Architecture builder](https://casoon.github.io/chartlet/docs/guides/javascript/#architecture-builder).
  `order-flow` and `release-flow` rebuild with it byte for byte.
- **npm:** `inspectChart(spec, { disable })` runs the inspections of `chartlet inspect` without the
  CLI and returns `{ ok, findings }` or `{ ok: false, error }`.
- **MCP:** `chartlet_inspect` returns the findings of `chartlet inspect` with code, severity, path,
  message and hint.
- **MCP:** `chartlet_diagram_starter` takes `template` with `type: "architecture"` — `context`,
  `containers` or `deployment` — each a valid specification with the question it answers. Examples
  `booking-context`, `booking-containers`, `booking-deployment`.
- **Docs:** the architecture guide maps system, container, component and deployment node onto
  chartlet and says where the starters stop short of a complete architecture model. Details:
  [Context, containers and deployment](https://casoon.github.io/chartlet/docs/guides/chart-types/#context-containers-and-deployment).

### Changed

- **Diagrams:** the description of flow charts, architecture and class diagrams names the shape of
  the graph before the steps — the most connected nodes, loops, unconnected parts, bottlenecks —
  where that says more than the steps. Details:
  [Graph structure](https://casoon.github.io/chartlet/docs/guides/chart-types/#graph-structure).
- **npm:** the WebAssembly module is about 34 KB smaller (700 KB to 666 KB gzip): chartlet finds the
  JSON Pointer of a specification error itself and no longer depends on `serde_path_to_error`.
  Error codes, paths and messages are unchanged.
- **Contributors:** `cargo test --test golden visual_report -- --ignored` writes a local HTML
  report of every reviewed example against its current rendering, without ever accepting a change.
  Details: [CONTRIBUTING.md](https://github.com/casoon/chartlet/blob/main/CONTRIBUTING.md).
- **Contributors:** `tests/generated.rs` renders flow charts, architecture diagrams, trees and
  networks from fixed seeds and checks that each renders the same twice as well-formed SVG.

### Fixed

- **Architecture diagrams:** in a deployment view, `text_too_long` for an instance label over 60
  characters points to the instance's `id`, not to a `label` field instances do not have, and says
  to shorten the id or the component's label.
- **Trees:** a distance matrix with a short row is refused with `invalid_matrix` at that row instead
  of stopping the render.
- **Diagrams:** in flow charts, architecture and class diagrams, a layer that lists a step between
  the members of a group no longer keeps that order; the frame took the step in, or pushed frames
  apart until the canvas grew many times its width.
- **Line charts:** `lower` and `upper` on a point without a value are refused with `invalid_bounds`
  instead of stopping the render.
- **Schema:** line charts with several series, small multiples of bars and fractional values on a
  numeric time axis validate against `schema/chartlet.schema.json`, as the compiler accepted them.

## [0.19.0] - 2026-10-12

### Added

- **Architecture diagrams:** `connections[].step` (1–99) numbers a connection in a sequence of
  calls (`2. places the order`); the description lists the steps in order. Example `checkout-calls`.
- **Architecture diagrams:** the component kinds `node` (a machine or cluster node) and `artifact`
  (something deployed on it) make deployment diagrams, with boundaries as the hosts. Example
  `shop-deployment`.
- **Trees:** `"layout": "radial"` draws a mind map: the root in the middle, every level on a ring,
  each subtree in a sector by its share of the leaves. Details:
  [Trees](https://casoon.github.io/chartlet/docs/guides/chart-types/#trees). Example
  `product-strategy`.
- **Diagrams:** `theme: "blueprint"` draws sequence, flow, state, architecture, tree and class
  diagrams in pale lines on a deep blue ground; other charts refuse it (`option_not_supported`).
- **Scatter plots:** `quadrants` divides the plot at an x and a y value into four named areas, such
  as an impact and effort matrix; description and table name the area of every point. Details:
  [Scatter plots](https://casoon.github.io/chartlet/docs/guides/chart-types/#scatter-plots).
  Example `feature-priorities`, with a phone variant.

### Fixed

- **Scatter plots:** description and table named a regression line even without
  `regression: true`; now only when it is drawn. In German, the count of points above a reference
  line reads „N Punkt(e) darüber“.

## [0.18.0] - 2026-10-12

### Added

- **Trees:** `collapsed` on a node draws it without what hangs below it, with a `+n` for the nodes
  left out; description and table still list every node (`nothing_to_collapse` warns when nothing
  hangs below).
- **Timelines:** `timezone` also takes about 55 common IANA names such as `Europe/Berlin`, with the
  daylight saving rules in force today. Details:
  [Timelines](https://casoon.github.io/chartlet/docs/reference/specification/#timelines).

### Changed

- **Text alternative:** small multiples open with “Time chart in 3 panels …” and “Bar chart in 2
  panels …” instead of “Small multiples of …” (German: „Zeitreihe in 3 Feldern …“), singular for
  one; several bands read “Lines have a shaded band from their lower to their upper bound”.
- **Text alternative:** English output spells “modelled” (legend, tooltips, description, table);
  the field `modeled` and the CSS class `chartlet-line-modeled` stay as they are.
- **Text alternative:** equal ends are named once (“x at 3”, “for 2020”, “on 3 Mar 2026”); a German
  scatter description says „von … bis …“, and Kaplan-Meier groups say “1 event” / „1 Ereignis“.

## [0.17.0] - 2026-10-11

### Added

- **Class diagrams:** `"type": "classes"` for class and entity-relationship diagrams: classes with
  stereotype, attributes and methods, joined by six kinds of relations with UML marks and
  cardinalities. Details:
  [Class and entity diagrams](https://casoon.github.io/chartlet/docs/guides/chart-types/#class-and-entity-diagrams).
  Examples `order-model`, `shop-schema`.
- **Timelines:** a summary phase takes `rollup`, the ids of the phases it sums up, and shows their
  progress weighted by length.
- **Sankey diagrams:** `"back": true` on a link marks a flow back to an earlier column, drawn as a
  loop under the diagram (`link_not_back` when it does not lead back). Example `water-cycle`.

### Changed

- **Timelines:** arrows between rows turn where no bar or diamond of the rows between them stands,
  instead of running through it.

## [0.16.0] - 2026-10-11

### Added

- **Bar and line charts:** a series of a grouped bar chart or an unstacked line chart takes `lower`
  and `upper` arrays for error bars, with tooltips and an interval column per series. Example
  `satisfaction-by-quarter`, with a phone variant.
- **Timelines:** `timezone` (UTC or a fixed offset such as `+02:00`) sets the zone the days and the
  shaded weekends are read and written in.

## [0.15.0] - 2026-10-11

### Added

- **Box plots:** `group` on every box colors the boxes of a group alike (up to four), with a legend
  and a Group column; for boxes, violins and strips. Example `latency-by-region`, with a phone
  variant.
- **Timelines:** `"weekends": true` shades Saturdays and Sundays while the axis spans at most 120
  days; a longer axis warns with `weekends_hidden`.

## [0.14.0] - 2026-10-10

### Added

- **Box plots:** `"showMean": true` marks the mean of each box with a diamond and adds a Mean
  column; added to the example `response-distribution`.
- **Box plots:** `"violinScale": "area"` gives every violin the same area and `"count"` an area by
  its number of observations, so widths compare between violins.
- **Timelines:** `progress` (0 to 1) on a phase draws that share of its bar darker, and `groupRows`
  puts the rows of a group together; progress added to the example `product-roadmap`.
- **Scatter plots:** `xTicks` puts named labels on the x axis (chromosomes of a Manhattan plot), and
  `xRange`/`yRange` fix the ends of an axis. Example `genome-scan`, with a phone variant.
- **Survival curves:** with `logRank` and two groups, the description adds the hazard ratio of the
  second group with its 95 % interval, checked against an independent implementation.

## [0.13.0] - 2026-10-10

### Added

- **Flow charts:** two presets as examples, each with a phone variant: `study-selection` (a PRISMA
  2020 flow) and `court-instances` (an Instanzenzug in German); neither is a type of its own.
- **Line charts:** a line chart of one series takes error bars, `lower` and `upper` on every point.
  Example `monthly-satisfaction`, with a phone variant.
- **Scatter plots:** `"regression": true` draws a least-squares line per group, and
  `xScale`/`yScale` set logarithmic axes. Example `allometry`, with a phone variant.
- **Survival curves:** `"logRank": true` runs the log-rank test of two to four groups, with the p
  value in the plot and the statistic in the description; added to the example `overall-survival`.
- **Sankey diagrams:** `nodes` pins a node to a `column` and a palette `color`, and
  `"order": "listed"` keeps the listed order (`column_too_early` when links lead past the column).
  Example `voter-movement`, with a phone variant.
- **Treemaps:** an item may have `children` instead of a value, nested up to three levels and drawn
  as frames. Details:
  [Treemaps](https://casoon.github.io/chartlet/docs/reference/specification/#treemaps). Example
  `budget-by-sector`, with a phone variant.

## [0.12.2] - 2026-10-06

### Fixed

- **Sankey diagrams:** nodes take the palette colors in turn instead of all taking the color of the
  first source, and neighbouring nodes in a column are swapped where that lets fewer bands cross.
- **Site:** the showcase follows the site's theme: a chart on a dark site uses chartlet's dark
  palette.

## [0.12.0] - 2026-10-06

Eight chart types and forest plots. The additions extend the specification, so they belong to 0.12.

### Added

- **Range bars:** forest plots: a `weight` on a span (a square whose area follows it),
  `"summary": true` (a diamond) and `references` (the line of no effect); value labels keep off the
  lines. Details: [Forest plots](https://casoon.github.io/chartlet/docs/guides/chart-types/#forest-plots). Example `trial-effects`, with a phone variant.
- **Box plots:** `"boxDisplay": "violin"` or `"strip"` draws the observations as a density or as
  points; every box needs `values` (`values_required`). Details: [Box plots](https://casoon.github.io/chartlet/docs/guides/chart-types/#box-plots). Example `response-distribution`, with a
  phone variant.
- **Timelines:** `"type": "timeline"` for Gantt charts, roadmaps and procedures: phases, milestones,
  marker lines, groups and dependencies (`after`) drawn as arrows. Details: [Timelines](https://casoon.github.io/chartlet/docs/guides/chart-types/#timelines). Example `product-roadmap`, with a
  phone variant.
- **Scatter plots:** `"type": "scatter"`, up to 5000 points in up to four groups, with reference
  lines and named points, for volcano and Manhattan plots. Details: [Scatter plots](https://casoon.github.io/chartlet/docs/guides/chart-types/#scatter-plots). Example `gene-expression`, with a phone
  variant.
- **Survival curves:** `"type": "survival"`, Kaplan-Meier curves of up to four groups with censoring
  ticks, an optional 95 % band and the number at risk. Details: [Survival curves](https://casoon.github.io/chartlet/docs/guides/chart-types/#survival-curves). Example `overall-survival`, with a phone
  variant.
- **Sankey diagrams:** `"type": "sankey"`, up to 100 links between up to 40 nodes in columns,
  ordered to cross little; cycles, self links and duplicates are refused. Details: [Sankey diagrams](https://casoon.github.io/chartlet/docs/guides/chart-types/#sankey-diagrams). Example `energy-flow`,
  with a phone variant.
- **Treemaps:** `"type": "treemap"`, up to 100 items as rectangles by value, in up to four groups
  with a legend. Details: [Treemaps](https://casoon.github.io/chartlet/docs/guides/chart-types/#treemaps). Example `budget-by-department`, with a phone variant.
- **Parliament charts:** `"type": "parliament"`, the seats of an assembly in a semicircle by party,
  with the majority line and a ringed coalition. Details: [Parliament charts](https://casoon.github.io/chartlet/docs/guides/chart-types/#parliament-charts). Example `election-result`, with a phone variant.
- **Waffle charts:** `"type": "waffle"`, up to four parts as squares by the largest remainder, and
  the rest of a larger total. Details: [Waffle charts](https://casoon.github.io/chartlet/docs/guides/chart-types/#waffle-charts). Example `energy-sources`, with a phone variant.
- **Waterfalls:** `"type": "waterfall"`, a running total with starts, deltas and checked totals,
  vertical or horizontal. Details: [Waterfalls](https://casoon.github.io/chartlet/docs/guides/chart-types/#waterfalls). Example `revenue-to-profit`, with a phone variant.

### Changed

- **Range bars:** the range bar style gains two rules, so range bar charts differ in their style
  block.

## [0.11.0] - 2026-10-05

Box plots and error bars, and canvas sizes that hold: the size a diagram asks for with
`canvas_too_small` is now one it can be drawn at. Both additions extend the specification; every
specification that rendered with 0.10.0 still renders, and every example is byte-identical.

### Added

- **Bar charts:** `lower` and `upper` on every point of a bar chart without groups draw error bars;
  the value label moves beyond the bar and the table gains an Interval column. Example
  `satisfaction-scores`.
- **Box plots:** `"type": "boxplot"`, one box per category, computed from `values` or drawn from
  the five numbers and `outliers`, vertical or horizontal. Details:
  [Box plots](https://casoon.github.io/chartlet/docs/reference/specification/#box-plots). Example
  `response-times`, with a phone variant.

### Fixed

- **Sequence diagrams:** a diagram up to 600 pixels wide takes the compact layout, so that a mobile
  variant that grows keeps its layout.
- **Diagrams:** a diagram that asks for a larger canvas (`canvas_too_small`) asks for a size at
  which it raises no further warning; the size for an edge label past the last layer, or a
  self-message of the last participant, grew with every attempt.

## [0.10.0] - 2026-10-05

Bars colored by group, small multiples of bars, the fixes found when casoon.de was moved to 0.9,
and a smaller WebAssembly renderer. Both additions extend the specification; every specification
that rendered with 0.9.0 still renders. Charts with a logarithmic axis, with a reference line
through a value label, with a span of one value, and flow and sequence diagrams with the cases
fixed below come out differently, so their examples changed; every other example is
byte-identical.

### Added

- **Bar charts:** `group` on the points of a single-series bar chart gives every group a palette
  color and a legend entry (at most 4), and the table a Group column.
- **Bar charts:** small multiples of bars: `multiples` with `categories`, and in every pane `values`
  and a `valueAxis` of its own, for measures in different units. Examples `benefit-and-harm`,
  `framework-benchmarks` (with a phone variant).

### Changed

- **Value axis:** a logarithmic axis ends at the next 1, 2 or 5 times a power of ten instead of the
  next power of ten: values up to 13.75 give an axis to 20, not 100. The examples with a
  logarithmic axis changed.
- **npm:** the WebAssembly renderer is 18 % smaller (1276 KB / 478 KB gzip to 1044 KB / 409 KB
  gzip) with the same output: the layouts share one sort routine, and the build is optimized for
  size (`opt-level = "z"`, `wasm-opt -Oz`) at about 15 % more render time (1.3 ms instead of
  1.15 ms).
- **npm:** `npm run build:wasm` prints the size of the module.

### Fixed

- **Sequence diagrams:** a message of the last participant to itself finds room: its label wraps
  and the canvas grows on the right (`canvas_too_small`) instead of `text_truncated`.
- **Diagrams:** in architecture and flow diagrams, an edge label beyond the last layer widens the
  canvas with `canvas_too_small` instead of running out of the picture.
- **Diagrams:** in architecture and flow diagrams, the label of an edge that runs back from a
  diamond stays at its edge, and a frame is as wide as its own name instead of shortening it.
- **Bar charts:** on horizontal bars, a value label that a reference line would run through moves
  to the far side of the line.
- **Time charts:** a value label that would run into the previous one of its layer is left out with
  `value_labels_omitted`, as with bars; under `--strict` such a chart now fails.
- **Range bars:** a span with `low` equal to `high` is labeled with the one value (`3`, not
  `3 to 3`).

## [0.9.0] - 2026-10-05

Trees, and finished sequence and architecture diagrams, laid out for a real reference architecture
and a sign-in sequence on a desktop canvas and at phone width. Every specification that rendered
with 0.8.0 still renders. Sequence, flow, state and architecture diagrams may come out slightly
differently, so their examples changed; every other example is byte-identical.

### Added

- **Architecture diagrams:** example `tenant-architecture`, with phone and print variants.
- **Sequence diagrams:** example `sign-in-sequence`, with phone and print variants.
- **Trees:** `"type": "tree"` for organization charts, ownership structures and family trees:
  nodes naming their `parent`, an optional `link` label such as a share, node `kind` and `partner`
  for couples. Details: [Trees](https://casoon.github.io/chartlet/docs/guides/chart-types/#trees).
  Examples `ownership-structure`, `family-tree`.

### Changed

- **Flow charts:** lanes leave a little room before their first and after their last step, so
  diagrams with lanes come out slightly longer.
- **Diagrams:** sequence and architecture diagrams are no longer experimental; flow and state
  diagrams stay experimental.

### Fixed

- **Sequence diagrams:** a participant's name is no longer shortened when its column has room;
  boxes are a pixel wider where that happened.
- **Sequence diagrams:** on a narrow canvas, message labels may reach over neighbouring lifelines
  and wrap, and boxes and gaps are tighter (five participants in about 400 pixels instead of 480);
  a frame grows to hold its label.
- **Diagrams:** in architecture and flow diagrams, edges that leave a step side by side no longer
  cross, a label of a shared port stands under its own stretch, labels keep off group frames in
  portrait, and a person's head sits inside its box.
- **Diagrams:** in architecture and flow diagrams, a step's name and an edge's technology are no
  longer shortened on a narrow canvas.

## [0.8.0] - 2026-10-03

Software diagrams, laid out by chartlet from a description of what is connected, with the same
accessible output as a chart. All four types are experimental: their layout may change between
minor releases. Every specification that rendered with 0.7.1 still renders the same; the
examples from before are byte-identical.

### Added

- **Architecture diagrams:** `"type": "architecture"`: components in eight kinds with their own
  shapes, connections with a label and a technology, and boundaries nested up to four deep that
  keep outside components out of their frame. Details:
  [Architecture diagrams](https://casoon.github.io/chartlet/docs/reference/specification/#architecture-diagrams).
  Example `shop-architecture`.
- **Architecture diagrams:** the component kind `security`, a shield in a new red role color.
- **Flow charts:** `"type": "flow"`, laid out in layers along the flow: steps in eight kinds, edges,
  cycles drawn back, lanes, groups and a main path kept in line. Details:
  [Flow charts](https://casoon.github.io/chartlet/docs/reference/specification/#flow-charts).
  Examples `release-flow`, `order-flow`.
- **State diagrams:** `"type": "state"` on the layout of flow charts, with transitions written
  `event [guard] / action`, a table row per transition and the warning `unreachable_state`.
  Details: [State diagrams](https://casoon.github.io/chartlet/docs/reference/specification/#state-diagrams).
  Example `ticket-states`.
- **State diagrams:** composite states (`"kind": "composite"`, other states lie `"in"` it), drawn as
  frames.
- **Sequence diagrams:** `"type": "sequence"`: participants in five kinds, calls, replies and
  asynchronous messages, activation bars, numbering and nested fragments. Details:
  [Sequence diagrams](https://casoon.github.io/chartlet/docs/reference/specification/#sequence-diagrams).
  Examples `cache-lookup`, `async-export`.
- **Diagrams:** `orientation` draws a diagram in portrait or landscape; `auto` picks the one that
  fits the canvas, or the one that grows it less, again for a mobile variant. A canvas too small
  grows, with `canvas_too_small`.
- **Diagrams:** edge labels of flow, state and architecture diagrams are drawn above all edges and
  stack instead of covering each other; a node that only receives from or sends into a frame stands
  after or before it.
- **Diagrams:** below 480 pixels, as in mobile variants, diagrams are compact: smaller margins and
  gaps, narrower steps, names on two lines instead of shortened, labels wrapped and kept on the
  page; a numbered message's label rises above its badge.
- **Diagrams:** one look for all types: a role color per kind (CSS custom properties
  `--chartlet-role-*`), soft shadows, rounded corners, labels on chips and message numbers in
  badges.
- **Interaction:** focus without a script: in the HTML profile, a click on a diagram's node or a
  choice in the "Focus" radio list keeps the node, its neighbours and the edges between them and
  fades the rest; the SVG stays an image.
- **MCP:** `chartlet_diagram_starter` returns a valid starting specification per diagram type, and
  `chartlet_explain` gives a diagram's element counts and table rows; both need `@casoon/chartlet` with the diagram
  types.

### Fixed

- **Rendering:** the social variant scales a chart down into its frame when the chart was drawn
  larger than the size it was laid out for.

## [0.7.1] - 2026-10-02

Fixes only, from a render sweep across sizes, languages and variants and three code reviews of
0.7.0. Specifications that rendered with 0.7.0 still render, except a declared axis `step` that
would give more than 50 ticks (or none), which is now refused instead of drawing an unreadable or
endless axis.

### Fixed

- **All charts:** a plot keeps at least 40 pixels of height and reports `dense_chart` instead of
  turning upside down on a small chart.
- **Bar charts:** the value label of a short negative horizontal bar no longer runs into the
  category labels; its category leaves out its value labels (`value_labels_omitted`).
- **Bar charts:** stacked bar charts use a declared `valueAxis.step`, which they ignored.
- **Bar and line charts:** `labels_thinned` names `/data` for charts with `data`, and its message
  reads "one category in n".
- **Value axis:** a declared `valueAxis.step` gives at most 50 ticks (an exact axis at least 2), and
  `timeAxis.step` 1 to 50; a tiny step could hang the renderer.
- **Time charts:** a time axis counts the ticks a step places and spaces them by the plot's real
  width, so a narrow chart no longer ends up with a single tick and times of day get their room.
- **Time charts:** a low pane labels only the value ticks that stand a label's height apart; its
  gridlines stay.
- **Time charts:** tooltips and the description write a value axis `unit` after the values, and an
  unnamed layer's table column carries it.
- **Time charts:** the value-axis gutter grows past 72 pixels where long labels need it (up to 40 %
  of the chart); small multiples measure their gutter the same way and shorten or drop a unit
  without room.
- **Time charts:** zoom windows show their own span: `timeAxis.min`, `max` and `step` apply to the
  whole axis only, and `stepEnd` only to a window that holds the last observation.
- **Time charts:** `stepEnd` after trailing missing values no longer draws over the gap.
- **Time charts:** a declared coarser `timeAxis.precision` no longer removes ticks within a day.
- **Time charts:** a sparkline refuses `valueAxis.unit`, `valueAxis.step` and `timeAxis.step`,
  which need axes.
- **Calendars:** a calendar reports cells smaller than 4 pixels.
- **Range bars:** the legend of range bar groups wraps instead of running off a narrow chart.
- **Rendering:** PNG output warns with `glyph_missing` about characters its fonts have no glyph for.
- **Rendering:** a PNG of a chart without a mobile variant fails with `missing_mobile`, like the
  SVG.

## [0.7.0] - 2026-10-02

Wider plots for time charts and a unit at the top tick. The specification keeps
`schemaVersion: 1`; every specification that rendered with 0.6.0 still renders.

### Added

- **Time charts:** `valueAxis.unit` writes the unit after the top tick label and in the table's
  column names, instead of a title above the plot; also in small multiples.

### Changed

- **Time charts:** the value-axis gutter is as wide as the widest tick label needs, at most the
  former 72 pixels, so the plot is wider; time tick labels at the left edge move inward like those
  at the right.

## [0.6.0] - 2026-10-02

Small panels, continued, and semibold text in PNG output. The specification keeps
`schemaVersion: 1`; every specification that rendered with 0.5.0 still renders.

### Added

- **Time charts:** `timeAxis.min` and `timeAxis.max` extend the time axis, such as to a round year
  before the first observation.
- **Time charts:** `"stroke": "medium"`, a two-pixel line for small panels.

### Changed

- **Time charts:** a compact time chart (below 320 pixels) makes its value-axis gutter as wide as
  its widest tick label needs.
- **Time charts:** without a drawn title, legend or value axis title above the first pane, the plot
  starts near the top instead of keeping the title's room free.

### Fixed

- **Rendering:** PNG output draws semibold text semibold: the bundled semibold faces join the Inter
  family, so titles, values and emphasized notes no longer fall back to regular.

## [0.5.0] - 2026-10-02

Small panels and their ticks. The specification keeps `schemaVersion: 1`; every specification
that rendered with 0.4.0 still renders.

### Added

- **All charts:** charts from 160 pixels tall, mobile variants too, for small panels.
- **Time charts:** `timeAxis.step`, the distance between time ticks — whole years on a calendar
  axis, units on a numeric one.
- **Small multiples:** `noteEmphasis` sets a panel's note apart, in the text color and semibold.
- **Rendering:** PNG of the mobile variant (`--variant mobile --format png`), with the literal
  colors of the print variant.

### Fixed

- **Time charts:** a time tick label that would run into the one before it is left out; its
  gridline stays.

## [0.4.0] - 2026-10-02

Compact charts, panel notes and step ends. The specification keeps `schemaVersion: 1`; every
specification that rendered with 0.3.0 still renders.

### Added

- **All charts:** compact charts from 200 pixels wide, mobile variants too: below 320 pixels the
  value-axis gutter and the margin are narrower, so a panel keeps its text size.
- **Time charts:** `stepEnd` on step lines, where the last step ends without an observation of its
  own.
- **Small multiples:** `panes[].note`, a short finding under a panel title.

### Changed

- **Time charts:** a declared `timeAxis.precision` changes how times are named, no longer the tick
  spacing.
- **Small multiples:** panel titles take two lines before they are shortened.

## [0.3.0] - 2026-10-02

Fixes and additions from moving a large site to 0.2.0. The specification keeps
`schemaVersion: 1`; every specification that rendered with 0.2.0 still renders.

### Added

- **Time charts:** `precision` per data layer, and every layer writes its times as finely as its own
  observations need (yearly means beside monthly values say only the year, and a fine helper line
  adds no decimals to the others); a table row is as fine as its finest layer with a value.
- **Time charts:** `markers: false` on a line or area layer, such as a fitted line; its tooltips
  stay on invisible targets.
- **Time charts:** a stacked pane takes up to eight area layers.

### Changed

- **Value axis:** an exact value axis puts its ticks on multiples of the step, with the zero line:
  −2.5 to 3.5 with step 1 ticks −2, −1, 0 … 3.

### Fixed

- **Value axis:** a horizontal value axis labels only the ticks whose labels fit, so the wide labels
  of a logarithmic axis no longer run into each other.
- **Range bars:** a horizontal range bar chart measures its value labels with the fallback reserve,
  so the label at the right end is no longer cut off.

## [0.2.0] - 2026-10-02

Stacked areas, points and range groups, exact value axes and the stylesheet in parts. The
specification keeps `schemaVersion: 1`; every specification that rendered with 0.1.0 still
renders.

### Added

- **Value axis:** `step` sets the tick interval and `exact` keeps `min` and `max` as the ends.
- **Time charts:** `"mark": "point"` draws a dot per observation and no line.
- **Time charts:** `"stack": "normal"` on a pane stacks its area layers; the crosshair of the
  interactive module places stacked values on top of the stack.
- **Time charts:** `"tooltips": "markers"` keeps tooltips on drawn markers only, so dense lines stay
  small.
- **Time charts:** months (`"2026-03"`) as times, and `timeAxis.precision` (`year`, `month`, `day`,
  `minute`) for how tooltips, table and description write times.
- **Range bars:** `group` per range, in a palette color with a legend entry per group.
- **Rendering:** the shared stylesheet in parts: `stylesheet_common()` and `stylesheet_types()`,
  `stylesheet({ types: [] })` and `stylesheet({ types, common: false })`, and
  `chartlet stylesheet --common` and `--no-common`.

### Changed

- **All charts:** charts narrower than 480 pixels draw their title at 18 pixels.
- **Bar and line charts:** more than 16 categories whose labels do not fit on a horizontal axis
  label every n-th category, with the room of n bands, and warn `labels_thinned`.
- **Rendering:** the shared stylesheet groups its rules type by type; the common part comes first,
  and a type's part styles only charts of that type.

### Fixed

- **Time charts:** the markers of the first series in a chart with several layers take its palette
  color.
- **Range bars:** a horizontal range bar chart keeps room for its last tick label.

## [0.1.0] - 2026-10-01

The first release without the alpha tag. The specification keeps `schemaVersion: 1`; every
specification that rendered with 0.1.0-alpha.7 still renders.

### Added

- **Bar charts:** `stack: "normal"` (with the total beyond each stack) and `stack: "percent"`
  (shares of each category's total);
  `patterns: true` draws every other series as an outline; `references` draws reference lines such
  as an average or a target.
- **Line charts:** `categories` and `series`, each series with its own color and line pattern.
- **Value axis:** `min` and `max`, `thousandsSeparator`, `scale: "log"` and `reverse`.
- **Time charts:** `timeAxis.kind: "number"` for profiles and deep time, `timeAxis.reverse`,
  `curve: "step"`, `legend: "end"` (names at the end of the lines) and `sparkline: true`.
- **Time charts:** zones with a single edge, and layer colors as `var(--name, #hex)` whose fallback
  the print variant draws.
- **Small multiples:** `independentAxes` gives every panel its own value axis; `mobile.columns`.
- **Stripes:** `stretch: true` draws a band of color that takes any box.
- **Text alternative:** `alternative: true` and `chartlet::text_alternative` return the description
  and the data table as data.
- **Interaction:** `hooks: true` and the optional module `@casoon/chartlet/interactive`: crosshair
  with values, series toggles, scroll stations and playback, reading only the static output.
- **Rendering:** `--variant print` (literal colors for PDF and print renderers) and
  `--variant social` (1200 × 630 for Open Graph images).
- **Rendering:** PNG output behind the optional cargo feature `png`, with Inter bundled (OFL-1.1).
- **Rendering:** `styles: "external"` with the shared stylesheet `@casoon/chartlet/chartlet.css`
  (`chartlet stylesheet [--types …]`), so a page with many charts loads their CSS once.
- **Rendering:** `allowWarnings` / `--allow-warning` lets reviewed warning codes through strict
  mode.
- **Rendering:** the HTML figure carries `data-viz="chart"` and marks its data table as the text
  layer.
- **MCP:** `@casoon/chartlet-mcp`: tools named `chartlet_*`, `CHARTLET_MCP_ROOT` confining
  `outputPath`, and a view that shows the chart in the chat of hosts that support MCP Apps.

### Changed

- **Value axis:** a line, time series or range bar whose values are all zero or more gets no margin
  below zero.
- **Time charts:** one tooltip rule for lines and candles: a tooltip for every observation while
  they stand at least 4 pixels apart, on an invisible target where no marker is drawn.
- **Time charts:** candle colors differ in lightness as well as in hue.
- **Topic maps:** topic maps and knowledge landscapes are marked experimental.
- **Rendering:** German output writes dates as `31.08.2026`; a range bar chart is a
  „Spannweitendiagramm“, a realm of a knowledge landscape a „Bereich“.
- **Rendering:** print and social variants write `font-weight: 600`, which renderers outside the
  browser read.
- **npm:** the package renders with its bundled WebAssembly build; a CLI is used only with `binary`
  or `CHARTLET_BIN`.
