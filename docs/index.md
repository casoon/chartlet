---
title: Overview
description: What chartlet does, what it supports, and when another tool is the better choice.
order: 0
---

chartlet compiles a small JSON chart specification into a finished, accessible chart at build
time: plain SVG, or an HTML figure with caption, source and data table. Nothing runs in the
browser – no chart JavaScript, no hydration, no layout shift.

**Status:** 0.7. Bar charts (single and grouped, vertical and horizontal), categorical
line charts, time series with uncertainty bands and reference lines, small multiples, warming
stripes, calendar heatmaps and range bars are supported, plus two map-like types
(topic map and knowledge landscape, both experimental). Until 1.0, a minor release (0.8, 0.9, …) may still change
the specification; a patch release never does.

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
- You need geographic maps, networks, 3D charts, scatter plots or pie charts.
- You want to explore data rather than publish a finished chart.

## How the docs are organised

- **Getting started:** [installation](getting-started/installation/) and a
  [first chart](getting-started/quickstart/).
- **Guides:** chart types, interaction without JavaScript, Astro, accessibility, warnings and
  errors.
- **Reference:** the [specification](reference/specification/), [CLI options](reference/cli/),
  the [Rust crate](reference/rust/) and the [support matrix](reference/support/).

Every chart in the [showcase](../showcase/) is rendered by chartlet during the site build.
