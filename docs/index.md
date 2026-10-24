---
title: Overview
description: What chartlet does, what it supports, and when another tool is the better choice.
order: 0
---

chartlet compiles a small JSON specification into a finished, accessible chart or software
diagram at build time: plain SVG, or an HTML figure with caption, source and data table. Nothing runs in the
browser – no chart JavaScript, no hydration, no layout shift.

chartlet draws bar and line charts, time series with bands, areas, candlesticks and reference
lines, small multiples, warming stripes, calendar heatmaps, range bars and forest plots, box and
violin plots, waterfalls, waffle and parliament charts, treemaps, Sankey diagrams, Kaplan-Meier
curves, scatter plots and timelines, and
sequence, flow, state, architecture, class, tree and network diagrams that it lays out itself.
Architecture models carry several views, deployments per environment and perspectives, and
`chartlet inspect` checks a model for gaps in its documentation.

**Status:** before 1.0. Until 1.0, a minor release may still change the specification; a patch
release never does. Flow, state and class diagrams are
experimental: their layout may change between minor releases.

## Principles

- **Accessible by default:** every chart carries a title and a generated description of the data,
  and the HTML output always includes the complete data as a table.
- **Deterministic:** the same specification produces the same bytes, so charts can be reviewed,
  diffed and cached like any other build artifact.
- **Honest about problems:** invalid input is rejected with a code, a path and a fix; layout
  compromises such as shortened labels are reported as warnings instead of happening silently.

## When chartlet is not the right tool

- You need continuous zooming, panning, cross-filtering or live updates. A crosshair with
  values, series toggles, scroll stations and step-by-step playback come from the optional
  [`@casoon/chartlet/interactive`](guides/interaction/) module, which only reads the static output.
- The data changes at runtime rather than at build time.
- You need geographic maps, 3D charts or pie charts, or diagrams you place by hand, with more
  than about 40 boxes.
- You want to explore data rather than publish a finished chart.

## How the docs are organised

- **Getting started:** [installation](getting-started/installation/) and a
  [first chart](getting-started/quickstart/).
- **Guides:** chart types, time series, interaction without JavaScript, responsive charts,
  Astro, JavaScript runtimes, AI agents, accessibility, print, social images and PNG, QR codes,
  warnings and errors.
- **Reference:** the [specification](reference/specification/), [CLI options](reference/cli/),
  the [Rust crate](reference/rust/) and the [support matrix](reference/support/).

Every chart in the [showcase](../showcase/) is rendered by chartlet during the site build.
