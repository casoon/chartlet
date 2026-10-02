# Changelog

All notable changes to chartlet are listed here. chartlet follows [Semantic Versioning](https://semver.org/):
until 1.0, a minor release (0.2, 0.3, …) may change the specification; a patch release never
does.

## [Unreleased]

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
