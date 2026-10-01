# Changelog

All notable changes to chartlet are listed here. chartlet follows [Semantic Versioning](https://semver.org/):
until 1.0, a minor release (0.2, 0.3, …) may change the specification; a patch release never
does.

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
