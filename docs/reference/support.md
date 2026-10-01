---
title: Support matrix
description: What has been verified, what is designed but not yet tested, and what is planned or out of scope.
order: 4
---

chartlet is in early alpha. Statuses: **Verified**, **Designed, not yet verified**, **Planned**,
**Not supported**.

## Embedding

| Context | Status | Notes |
| --- | --- | --- |
| Standalone SVG | Verified | Byte-identical on macOS and Linux; covered by the golden-file tests. |
| HTML figure (caption, source, data table) | Verified | The data table is always present, in a native disclosure by default. |
| Astro component | Verified | Renders at build time and ships no chart JavaScript to the browser. |
| CLI and Rust API | Verified | The same renderer backs the CLI, the crate, and the npm package. |
| Mobile variant (`mobile`) | Verified | HTML profile: both variants in one wrapper, switched by a container query; the hidden one is `display:none`. Checked in Chromium at a narrow and a wide container, with zoom steps and series filter. Browsers without container queries keep the full-size chart. |
| `<img src="chart.svg">` | Designed, not yet verified | Title and description survive; the figure wrapper and data table do not apply. |
| Markdown | Planned | Not yet evaluated. |
| PDF and print | Planned | Not yet evaluated. |
| E-mail clients | Planned | Not yet evaluated. |

## Browsers

| Context | Status | Notes |
| --- | --- | --- |
| Chromium (Chrome, Edge) | Verified | Automated accessibility scan (axe) and manual review during development. |
| Firefox | Planned | Standard SVG is expected to work; the accessibility tree is not yet tested. |
| Safari | Planned | Standard SVG is expected to work; the accessibility tree is not yet tested. |

## Screen readers

| Context | Status | Notes |
| --- | --- | --- |
| VoiceOver (macOS) | Planned | The title and description are exposed as the accessible name and description; not yet tested. |
| NVDA (Windows) | Planned | Not yet tested. |
| JAWS (Windows) | Planned | Not yet tested. |

## Themes

| Context | Status | Notes |
| --- | --- | --- |
| Light (default) | Verified | The default palette keeps at least 4.5:1 contrast on white (lowest series color 5.17:1). |
| Dark | Verified | `"theme": "dark"` sets the palette through the same CSS custom properties and paints its own background. Lowest series color 7.65:1 against `#0e131c`. |
| Forced colours / high contrast | Planned | Planned; not yet shipped. |
| Host-page colors (`var(--name)` per layer) | Verified | A `var()` reference is passed through and falls back to the chart's text color when the page defines nothing. |

## Time series

| Context | Status | Notes |
| --- | --- | --- |
| Calendar axis (ticks, timezone) | Verified | Ticks snap to calendar boundaries; a fixed UTC offset such as `"+02:00"` shifts the labels and the data table. Covered by unit and golden-file tests. |
| Line on the time axis | Verified | Up to six layers of 2000 observations per pane, at most four of the chart in palette colors; markers and value labels up to 60 observations per layer. The legend wraps into further rows. |
| Dense series | Verified | More observations than plot pixels is reported as `dense_chart`; measured at 131 KB of SVG for four layers of 2000 points and 196 KB for six. |
| Uncertainty band and modeled layers | Verified | `lower`/`upper` per point draw a band in the line's color; `"modeled": true` dashes the line, hatches the band and says “modeled” in legend, description and table. Covered by unit and golden-file tests. |
| Reference lines (`annotation`) | Verified | Horizontal at `value` or vertical at `time`. A label that overlaps another annotation label, crosses a data line or leaves the plot is reported as `label_overlap`; it is not moved. |
| Zones (`band`) | Verified | A shaded range between `bottom` and `top`, `from` and `to`, or both, behind the data, with its label; described, no legend entry or table column. Covered by unit and golden-file tests. |
| Point markers (`annotation` with `time` and `value`) | Verified | Five shapes, a label beside the symbol that tries all four sides before `label_overlap` is reported, and a native tooltip. Zones, reference lines and markers share the limit of six per pane. |
| Small multiples | Verified | `"type": "multiples"`: 2–12 panes with a shared value axis and one legend. |
| German texts and numbers (`"locale": "de"`) | Verified | Every chart type. |
| `area` mark | Verified | Filled down to zero in the layer's color; the value axis includes zero. Covered by unit and golden-file tests. |
| Line patterns and weights (`dash`, `stroke`) | Verified | `solid`, `dashed`, `dotted`; `thin`, `regular`, `bold`; the legend sample shows both. |
| Gaps (`null`) on a time chart | Verified | Break line, area and band; counted as missing in description and table. Also in small multiples. |
| `zoomSteps` on a time chart | Verified | Windows by timestamp, HTML profile only; same radio-and-CSS mechanism as category zoom. |
| `ohlc` mark (candlesticks) | Verified | Wick from low to high, body from open to close; rising hollow, falling filled, in `--chartlet-rise` and `--chartlet-fall`. Beyond one candle per three plot pixels only the wicks are drawn and `dense_chart` is reported. Tooltips, table columns and description per candle layer. Covered by unit and golden-file tests. |
| Several panes on a `time` chart | Verified | Up to four panes stacked on one shared time axis, each with its own value axis and a share of the height by `heightRatio`; one legend with names unique across the chart. Covered by unit and golden-file tests. |
| `zoomSteps` on small multiples | Planned | Refused with `option_not_supported`. |

## Other chart types

| Context | Status | Notes |
| --- | --- | --- |
| Warming stripes, calendar heatmap | Verified | Shared diverging scale of 17 colors as CSS custom properties; covered by unit and golden-file tests. |
| Range bars | Verified | Vertical and horizontal, optional central value and hatched modeled ranges. |
| Topic map, knowledge landscape | Verified | Rendered and tested like every other type; the layout is tuned to one production site so far. |

## Distribution and interactions

| Context | Status | Notes |
| --- | --- | --- |
| WebAssembly build in Node.js | Verified | The npm package renders through it by default; tests compare every example with the CLI output byte for byte on Node.js 22. |
| WebAssembly build in Cloudflare Workers | Verified | `@casoon/chartlet/wasm` with an imported `chartlet.wasm`; checked locally in workerd, not in a deployed Worker. |
| WebAssembly build in the browser | Designed, not yet verified | Same entry point; the module is compiled by the page. |
| Native Node bindings | Not supported | The npm package renders through the WebAssembly build instead; `CHARTLET_BIN` selects the CLI. |
| Series filtering and stepped zoom | Designed, not yet verified | Available in the HTML profile with native controls and CSS; broader browser and assistive-technology verification is pending. |
| Mark tooltips | Designed, not yet verified | Native SVG titles provide hover hints; the description and table remain the accessible alternatives. |
