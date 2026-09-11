# chartlet gallery

The public gallery for [chartlet](../README.md): a static site, built with Astro, that
dogfoods the `@casoon/chartlet` integration. Every chart on the site is rendered at build
time; the browser receives plain SVG and no chart JavaScript.

## What it contains

- **Examples** (`/examples/<slug>/`): the canonical specs from `<repo>/examples/`, each with
  the rendered chart, the specification next to it, the generated accessible description, and
  the full data table.
- **Support matrix** (`/support/`): what has been verified, what is designed but untested, and
  what is planned or out of scope.

The example specs are imported directly from `<repo>/examples/`, so there is exactly one
source of truth for every chart: the same files are the golden-file test inputs.

## Prerequisites

1. The `chartlet` CLI must be buildable and available. From the repository root:

   ```sh
   cargo build --release
   ```

2. The CLI must be on `PATH`, or `CHARTLET_BIN` must point to it.

## Build and preview

```sh
cd gallery
npm install
CHARTLET_BIN=../target/release/chartlet npm run build
npm run preview
```

`npm run dev` works the same way and picks up `CHARTLET_BIN` from the environment.

## Accessibility

The built output is checked in Chromium with [axe-core](https://github.com/dequelabs/axe-core). All
rendered examples expose an accessible name (the title) and an accessible description (the
generated data summary) via `aria-labelledby`, and the HTML figure always contains the data as
a real table.

Run the gallery checks with `npm test`. They also verify that the site ships no scripts and that
all internal links remain below the configured `/chartlet/` GitHub Pages base path.

## Deployment

The site is published to GitHub Pages at `https://casoon.github.io/chartlet/` by the
[`Gallery` workflow](../.github/workflows/pages.yml) on every push to `main`. The configured
`base` in `astro.config.mjs` matches that path.

## Notes

- Charts on the index page use the `@casoon/chartlet/astro` `<Chart>` component.
- Detail pages call the package's `renderChart()` API directly so they can surface the
  generated description and any build-time warnings.
