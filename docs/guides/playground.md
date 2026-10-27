---
title: Playground
description: Edit a specification in the browser with chartlet's own compiler running locally as WebAssembly; what it does, what it needs and what it does not do.
order: 14
---

The [playground](../../../playground/) lets you open an example, change a value or a step, read the
diagnostics at the field they name and download the result, without installing Node or Rust. It is
the real compiler: the same WebAssembly module that `@casoon/chartlet/wasm` ships, run in a worker.
There is no second renderer on the page.

## What it does

- **Start from an example:** a bar chart, a flow diagram, a specification with an error to correct,
  a canvas that is too narrow, and an architecture model.
- **Two ways to edit, one specification:** the JSON view is the complete source. The form view edits
  a bar chart (one series in `data`) and a flow diagram: title, source, size, values and labels,
  steps with their kind, and connections. Fields the form does not know are kept as they are and
  listed; a specification of another type says so and stays untouched. Switching between the views
  keeps the content. Applying a form edit rewrites the JSON with two-space indentation.
- **Incomplete edits do not break the figure:** an empty number field is marked invalid and the
  preview keeps the last valid render, marked "out of date". The same holds for JSON that does not
  parse and for a specification the compiler refuses.
- **Diagnostics where they belong:** every error and warning shows its code, its JSON Pointer and a
  link to the reference. *Go to field* selects the value in the JSON or focuses the input of the
  form. Some warnings offer a checked correction, see
  [Warnings and errors](../warnings-and-errors/#actionable-diagnostics): it is applied only when you
  press the button, and *Undo* restores the previous text.
- **Undo, redo and reset:** a history of up to 100 states; *Reset…* asks first and is itself
  undoable.
- **The text alternative:** the description and the data table of the figure are shown beside the
  preview, as for the HTML profile.
- **Export:** *Download JSON* is the exact specification that produced the figure, *Download SVG*
  and *Download HTML* are the compiler's output for the chosen preview. The preview selection
  (figure, mobile layout, print variant) is a render option, not part of the specification; choosing
  the mobile layout adds `"mobile": { "width": 360 }` to the specification, visibly, because the
  compiler needs it.

## What it does not do

- **No PNG.** PNG needs the bundled fonts and a rasterizer that is not part of the WebAssembly
  module; `chartlet render --format png` makes it.
- **No upload, no account, no analytics.** The specification stays in the tab. The page makes one
  request of its own, for `chartlet.wasm`, from the same origin. Sharing a specification by link is
  not offered; copy or download the JSON.
- **No drag-and-drop canvas and no second layout engine.** The form is a table of fields.

## Browser support

The playground needs WebAssembly and module workers (`new Worker(url, { type: "module" })`):

| Browser | Version |
| --- | --- |
| Chrome, Edge | 80 and later |
| Firefox | 114 and later |
| Safari | 15 and later |

In a browser without them, or when the module cannot be fetched, the page says so, keeps the JSON
editor and the downloads of the JSON, and shows how to install the CLI instead. The support matrix
is derived from the features used; it has not been run against every listed version.

## Size and limits

The compiler is fetched once when the page opens and not on any other page of the site. Its size is
the size of `chartlet.wasm` of the release, shown on the page, with the gzip size reported by
`npm run build:wasm` (budget 720 kB gzipped). A render that takes longer than 8 seconds is stopped
and the compiler restarts, so an expensive specification cannot freeze the page; a render that traps
leaves the next one with a fresh instance.

## Accessibility

Every control has a label, the status line is a polite live region that announces what was
rendered, how many warnings there are or which error stopped it, the diagnostics are a list of
buttons and links in reading order, and everything works with the keyboard alone.
