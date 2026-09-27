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
| Line on the time axis | Verified | Up to four layers of 2000 observations in one pane; markers and value labels up to 60 observations per layer. |
| Dense series | Verified | More observations than plot pixels is reported as `dense_chart`; measured at 131 KB of SVG for four layers of 2000 points. |
| `area`, `ohlc`, `band`, `annotation` marks | Planned | Refused with `mark_not_implemented`. |
| More than one pane | Planned | Refused with `too_many_panes`. |
| Gaps (`null`) and `zoomSteps` on a time chart | Planned | Refused with `option_not_supported`. |

## Distribution and interactions

| Context | Status | Notes |
| --- | --- | --- |
| Browser WASM build | Not supported | Planned after the MVP. |
| Native Node bindings | Not supported | Planned; the alpha npm package shells out to the CLI. |
| Series filtering and stepped zoom | Designed, not yet verified | Available in the HTML profile with native controls and CSS; broader browser and assistive-technology verification is pending. |
| Mark tooltips | Designed, not yet verified | Native SVG titles provide hover hints; the description and table remain the accessible alternatives. |
