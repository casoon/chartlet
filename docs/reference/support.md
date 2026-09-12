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
| Light (default) | Verified | The default palette keeps at least 4.5:1 contrast on white. |
| Dark | Planned | Planned via CSS custom properties; not yet shipped. |
| Forced colours / high contrast | Planned | Planned; not yet shipped. |

## Distribution and interactions

| Context | Status | Notes |
| --- | --- | --- |
| Browser WASM build | Not supported | Planned after the MVP. |
| Native Node bindings | Not supported | Planned; the alpha npm package shells out to the CLI. |
| Series filtering and stepped zoom | Designed, not yet verified | Available in the HTML profile with native controls and CSS; broader browser and assistive-technology verification is pending. |
| Mark tooltips | Designed, not yet verified | Native SVG titles provide hover hints; the description and table remain the accessible alternatives. |
