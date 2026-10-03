# Changelog

All notable changes to chartlet are listed here. chartlet follows [Semantic Versioning](https://semver.org/):
until 1.0, a minor release (0.2, 0.3, …) may change the specification; a patch release never
does.

## [Unreleased]

### Changed

- Flowcharts with lanes leave a little room between a lane's head and its first step, and after
  its last step, in portrait and landscape. Diagrams with lanes come out slightly longer.

## [0.8.0] - 2026-10-03

Software diagrams: sequence, flow, state and architecture diagrams, laid out by chartlet from a
description of what is connected, in portrait or landscape, with the same accessible output as a
chart and a focus on a node without a script. All four types are experimental: their layout may
change between minor releases. Every specification that rendered with 0.7.1 still renders the same;
the examples from before are byte-identical.

### Added

- Sequence diagrams (`"type": "sequence"`, experimental), the first of a family of software
  diagrams: participants in five kinds with their own shapes (service, actor, database, queue,
  external), calls, replies and asynchronous messages, messages to oneself, activation bars from
  call to reply, numbered messages, and `alt`/`opt`/`loop`/`par`/`critical`/`break` fragments that
  nest, with `else` branches. `orientation` draws a diagram in portrait (time running down) or
  landscape (time running right); `auto`, the default, picks the one that fits the canvas, and a
  mobile variant picks again at its size. A canvas too small grows, with the warning
  `canvas_too_small`. The description lists every message in order and the data table has a row
  per message. New examples `cache-lookup` and `async-export`.
- Flow charts (`"type": "flow"`, experimental), laid out by chartlet in layers along the flow:
  steps in eight kinds with their own shapes, labelled and dashed edges, cycles drawn back
  against the flow, loops on a step, lanes, groups and a main path kept in line. Portrait and
  landscape as for sequence diagrams. Errors and warnings name the field (`unknown_node`,
  `missing_lane`, `main_path_gap`, `group_overlap`, …). The description and the data table follow
  the steps in reading order. New examples `release-flow` and `order-flow`.
- State diagrams (`"type": "state"`, experimental) on the layout of flow charts: an initial dot,
  states, choices and final states, transitions written `event [guard] / action`, loops on a
  state, a main path. The warning `unreachable_state` names a state nothing leads to. The data
  table has one row per transition with event, guard and action in columns of their own. New
  example `ticket-states`.
- Architecture diagrams (`"type": "architecture"`, experimental): components in eight kinds with
  their own shapes (person, frontend, service, database, queue, storage, cache, external),
  connections with a label and a technology, and boundaries nested up to four deep, laid out like
  flow charts. Components outside a boundary are kept out of its frame. New example
  `shop-architecture`.
- Edge labels of flow, state and architecture diagrams are drawn above all edges; labels on the
  same side of a step stack instead of covering each other.
- Composite states (`"kind": "composite"`, other states lie `"in"` it) drawn as frames, and the
  component kind `security`, a shield in a new red role color.
- A component or step that hangs off a frame from outside — only receiving from it, or only
  sending into it — stands after or before the frame on the main axis instead of far beside it.
- Diagrams below 480 pixels, such as mobile variants, are compact: smaller margins and gaps,
  narrower steps, participant names on two lines instead of shortened, loop labels wrapped,
  frames and labels kept on the page. A numbered message's label rises above its badge instead
  of giving up width to it.
- Focus without a script: in the HTML profile a click on a diagram's node, or a choice in the
  "Focus" list of radio buttons above it, keeps the node, its neighbours and the edges between
  them and fades the rest. The SVG stays an image.
- MCP server: the tool `chartlet_diagram_starter` returns a valid starting specification per
  diagram type with the kinds of its elements, and `chartlet_explain` gives a diagram's structure:
  the counts of its elements and the rows of its data table. Needs `@casoon/chartlet` with the
  diagram types.
- Where neither orientation fits the canvas, `auto` takes the one that grows it less.
- The diagram types share one look: a role color per kind of step or participant on top of its
  shape (CSS custom properties `--chartlet-role-*`), soft shadows, rounded corners on edges, edge
  and message labels on chips, and message numbers in badges at the start of their arrows.

### Fixed

- The social variant scales a chart down into its frame when the chart was drawn larger than
  the size it was laid out for.

## [0.7.1] - 2026-10-02

Fixes only, from a render sweep across sizes, languages and variants and three code reviews of
0.7.0. Specifications that rendered with 0.7.0 still render, except a declared axis `step` that
would give more than 50 ticks (or none), which is now refused instead of drawing an unreadable or
endless axis.

### Fixed

- A time axis counts the ticks a step actually places instead of estimating them, and spaces its
  ticks by the width its plot actually has: a narrow chart no longer ends up with a single tick,
  and labels with a time of day get the room they need.
- A low pane of a time chart labels only the value ticks that stand a label's height apart; its
  gridlines stay.
- The value label of a short negative horizontal bar no longer runs into the category labels;
  its category leaves out its value labels instead (`value_labels_omitted`).
- PNG output warns with `glyph_missing` about characters its fonts have no glyph for.
- A declared `valueAxis.step` must give at most 50 ticks (an exact axis at least 2), and a
  declared `timeAxis.step` 1 to 50; a tiny step could draw millions and hang the renderer.
  Stacked bar charts now use a declared `valueAxis.step`, which they ignored.
- Tooltips and the description write a value axis `unit` after the values.
- A plot keeps at least 40 pixels of height and reports `dense_chart` instead of turning upside
  down on a small chart; a calendar reports cells smaller than 4 pixels.
- The value-axis gutter of time charts grows past 72 pixels where long labels, such as a value
  with its unit, need it (up to 40 % of the chart); small multiples measure their panel gutter
  the same way, report labels that do not fit, and shorten or leave out a unit that has no room.
- Zoom windows show their own span: `timeAxis.min`, `max` and `step` apply to the whole axis
  only, and `stepEnd` only to a window that holds the last observation.
- `stepEnd` after trailing missing values no longer draws over the gap.
- A declared coarser `timeAxis.precision` no longer removes ticks within a day.
- An unnamed layer's data table column carries the value axis `unit`.
- The legend of range bar groups wraps instead of running off a narrow chart.
- A sparkline refuses `valueAxis.unit`, `valueAxis.step` and `timeAxis.step`, which need axes.
- A PNG of a chart without a mobile variant fails with `missing_mobile`, like the SVG.
- `labels_thinned` names `/data` for charts with `data`, and its message reads "one category in
  n".

## [0.7.0] - 2026-10-02

Wider plots for time charts and a unit at the top tick. The specification keeps
`schemaVersion: 1`; every specification that rendered with 0.6.0 still renders, with the
changes in output listed under Changed.

### Added

- Time charts and small multiples: `valueAxis.unit`, the unit after the top tick label and the
  data table's column names, instead of a title above the plot.

### Changed

- Every time chart makes its value-axis gutter as wide as its widest tick label needs, at most
  the former 72 pixels, so the plot is wider; time tick labels at the left edge move inward like
  those at the right edge.

## [0.6.0] - 2026-10-02

Small panels, continued, and semibold text in PNG output. The specification keeps
`schemaVersion: 1`; every specification that rendered with 0.5.0 still renders, with the
changes in output listed under Changed and Fixed.

### Added

- Time charts: `timeAxis.min` and `timeAxis.max` extend the time axis to a position such as a
  round year before the first observation.
- Time charts: `"stroke": "medium"`, a two-pixel line for small panels.

### Changed

- A compact time chart (below 320 pixels) makes its value-axis gutter as wide as its widest
  tick label needs.
- A time chart with neither a drawn title, a legend nor a value axis title above its first pane
  starts its plot near the top instead of keeping the title's room free.

### Fixed

- PNG output draws semibold text semibold: the bundled semibold faces join the Inter family, so
  titles, axis titles, values and emphasized notes no longer fall back to the regular weight.

## [0.5.0] - 2026-10-02

Small panels and their ticks. The specification keeps `schemaVersion: 1`; every specification
that rendered with 0.4.0 still renders, with the changes in output listed under Fixed.

### Added

- Time charts: `timeAxis.step`, the distance between time ticks — whole years on a calendar
  axis, units on a numeric one.
- Small multiples: `noteEmphasis` sets a panel's note apart, in the text color and semibold.
- Charts from 160 pixels tall (mobile variants too), for small panels.
- PNG of the mobile variant: `--variant mobile --format png`, with the literal colors of the
  print variant.

### Fixed

- A time tick label that would run into the one before it is left out; its gridline stays.

## [0.4.0] - 2026-10-02

Compact charts, panel notes and step ends. The specification keeps `schemaVersion: 1`; every
specification that rendered with 0.3.0 still renders, with the changes in output listed under
Changed.

### Added

- Compact charts from 200 pixels wide (mobile variants too): below 320 pixels the value-axis
  gutter and the margin are narrower, so a panel rendered at its width keeps its text size.
- Small multiples: `panes[].note`, a short finding under a panel title.
- Step lines: `stepEnd`, where the last step ends without an observation of its own.

### Changed

- Small-multiples panel titles take two lines before they are shortened.
- A declared `timeAxis.precision` changes how times are named, no longer the tick spacing.

## [0.3.0] - 2026-10-02

Fixes and additions from moving a large site to 0.2.0. The specification keeps
`schemaVersion: 1`; every specification that rendered with 0.2.0 still renders, with the changes
in output listed under Changed and Fixed.

### Added

- Time charts: `precision` per data layer, and every layer writes its times as finely as its
  own observations need — yearly means beside monthly values say only the year, and on a
  numeric axis a finely spaced helper line no longer adds decimals to the other layers. A table
  row is labelled as finely as the finest layer with a value there.
- Time charts: `markers: false` on a line or area layer, such as a fitted line; its tooltips
  stay on invisible targets.
- A stacked pane takes up to eight area layers.

### Changed

- An exact value axis puts its ticks on multiples of the step, with the zero line:
  −2.5 to 3.5 with step 1 ticks −2, −1, 0 … 3.

### Fixed

- A horizontal value axis labels only the ticks whose labels fit, so the wide labels of a
  logarithmic axis no longer run into each other.
- A horizontal range bar chart measures its value labels with the fallback reserve, so the
  label at the right end is no longer cut off.

## [0.2.0] - 2026-10-02

Adds stacked areas, points and range groups, exact value axes and the stylesheet in parts. The
specification keeps `schemaVersion: 1`; every specification that rendered with 0.1.0 still
renders, with the changes in output listed under Changed and Fixed.

### Added

- Time charts: `"mark": "point"` draws a dot per observation and no line, as in a scatter plot.
- Time charts: `"stack": "normal"` on a pane stacks its area layers; the crosshair of the
  interactive module places stacked values on top of the stack.
- Time charts: `"tooltips": "markers"` keeps tooltips on drawn markers only, so dense lines stay
  small.
- Time charts: months (`"2026-03"`) as times, and `timeAxis.precision` (`year`, `month`, `day`,
  `minute`) for how tooltips, the table and the description write times.
- Range bars: `group` per range, drawn in a palette color with a legend entry per group.
- Value axis: `step` sets the tick interval and `exact` keeps `min` and `max` as the ends.
- The shared stylesheet in parts: `stylesheet_common()` and `stylesheet_types()`,
  `stylesheet({ types: [] })` and `stylesheet({ types, common: false })`, `chartlet stylesheet
  --common` and `--no-common`. The common part comes first; a type's part styles only charts of
  that type.

### Changed

- More than 16 categories on a horizontal axis whose labels do not fit: every n-th category is
  labelled, with the room of n bands, and the warning `labels_thinned`.
- Charts narrower than 480 pixels draw their title at 18 pixels.
- In the shared stylesheet, each type's rules now come together, type by type.

### Fixed

- The markers of the first series in a time chart with several layers take its palette color.
- A horizontal range bar chart keeps room for its last tick label.

## [0.1.0] - 2026-10-01

The first release without the alpha tag. The specification keeps `schemaVersion: 1`; every
specification that rendered with 0.1.0-alpha.7 still renders, with the changes in output listed
under Changed.

### Added

- Bar charts: `stack: "normal"` (by value, with the total beyond each stack) and `stack: "percent"`
  (shares of each category's total); `patterns: true` draws every other series as an outline;
  `references` draws reference lines such as an average or a target.
- Line charts take `categories` and `series`; each series has its own color and line pattern.
- Value axis: `min` and `max`, `thousandsSeparator`, `scale: "log"` and `reverse`.
- Time charts: `timeAxis.kind: "number"` for profiles and deep time, `timeAxis.reverse`,
  `curve: "step"`, `legend: "end"` (names at the end of the lines), `sparkline: true`, zones with
  a single edge, and layer colors as `var(--name, #hex)` whose fallback the print variant draws.
- Small multiples: `independentAxes` gives every panel its own value axis; `mobile.columns`.
- Stripes: `stretch: true` draws a band of color that takes any box.
- Variants: `--variant print` (literal colors for PDF and print renderers) and `--variant social`
  (1200 × 630 for Open Graph images).
- PNG output behind the optional cargo feature `png`, with Inter bundled (OFL-1.1).
- `styles: "external"` with the shared stylesheet `@casoon/chartlet/chartlet.css`
  (`chartlet stylesheet [--types …]`), so a page with many charts loads their CSS once.
- `alternative: true` and `chartlet::text_alternative`: the description and the data table as data.
- `allowWarnings` / `--allow-warning`: strict mode lets reviewed warning codes through.
- `hooks: true` and the optional module `@casoon/chartlet/interactive`: crosshair with values,
  series toggles, scroll stations and playback, reading only the static output.
- The HTML figure carries `data-viz="chart"` and marks its data table as the text layer.
- `@casoon/chartlet-mcp`: tools named `chartlet_*`, `CHARTLET_MCP_ROOT` confining `outputPath`,
  and a view that shows the chart in the chat of hosts that support MCP Apps.

### Changed

- A line, time series or range bar whose values are all zero or more gets no margin below zero.
- One tooltip rule for lines and candles: a tooltip for every observation while observations
  stand at least 4 pixels apart, on an invisible target where no marker is drawn.
- Candle colors differ in lightness as well as in hue.
- German output writes dates as `31.08.2026`; a range bar chart is a „Spannweitendiagramm“, a
  realm of a knowledge landscape a „Bereich“.
- Print and social variants write `font-weight: 600`, which renderers outside the browser read.
- Topic maps and knowledge landscapes are marked experimental.
- The npm package renders with its bundled WebAssembly build; a CLI is used only with `binary`
  or `CHARTLET_BIN`.
