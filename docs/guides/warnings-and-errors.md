---
title: Warnings and errors
description: Errors stop rendering with a code, a location and a fix; warnings report layout compromises while the chart is still produced.
order: 5
---

## Errors

Errors stop rendering and name a code, the location in the specification as a JSON Pointer (`/` for
the whole document) and a way to fix it:

```text
chartlet: series_length_mismatch at /series/0/values: expected 2 values, one per category; use null for a missing value
```

`invalid_json` at `/`: the specification is not JSON, or not an object. `spec_too_large` at `/`: the
text is more than 4 MiB; see [Limits and performance](../../reference/limits/). `invalid_spec`: a field has
the wrong type or an unknown name; the path points at it. `invalid_id_prefix` at `/render/idPrefix`:
an ID prefix that is not 1–64 ASCII letters, digits, hyphens or underscores starting with a letter.
`png_failed` at `/render/format`: the PNG could not be read from the SVG or encoded, with the
cause in the message. `serialization_failed`: the specification could not be written in its
canonical form for the generated IDs.

## Actionable diagnostics

A warning has a code, a path and a message. With **advice** it also comes with structured help, so
that a person or an agent need not parse the message: where to look, the documentation, and a few
remedies. Advice is off by default, and a response that did not ask for it is unchanged.

| Where | How |
| --- | --- |
| CLI | `chartlet render spec.json --diagnostics json --advice` adds `help` to the JSON report; `--advice` alone prints `help[...]` lines after the warnings |
| JavaScript (WebAssembly) | `renderChartDetailed(spec, { advice: true })` adds `help` to a successful response |
| MCP | `chartlet_validate_spec` with `advice: true` |
| Rust | `chartlet::advise_json(spec, &options)` |

```json
{
  "code": "canvas_too_small",
  "path": "/width",
  "field": "/width",
  "docs": "docs/guides/warnings-and-errors/",
  "note": "Changing the canvas is a layout change: no label, value or connection is touched. …",
  "remedies": [
    {
      "id": "raise-canvas",
      "kind": "layout",
      "summary": "Set width to 499 pixels, the size the diagram needs.",
      "validated": true,
      "resolves": true,
      "patch": [{ "op": "replace", "path": "/width", "value": 499 }]
    }
  ]
}
```

- **Which warnings:** `canvas_too_small` (raise `width` or `height` to what the render asked for) and
  `text_truncated` (widen the canvas to the first of 1.25, 1.5 and 2 times the width, up to 2400,
  at which the text fits). `label_overlap` and `dense_chart` come with guidance only, because
  chartlet cannot move labels or thin data without changing what they say. Other warnings have no
  help.
- **Checked, not guessed:** a `patch` is only offered after the compiler rendered the patched
  specification. `validated` says that happened; `resolves` says the render no longer had the
  warning and no layout warning that the original did not have. A remedy that does not resolve is
  shown as such.
- **Layout is not content:** a remedy of `kind` `layout` changes the canvas. A remedy of kind
  `content` needs a decision only the author can make, such as shortening a label; it carries no
  patch. chartlet never deletes, aggregates, renames or reorders data, labels or the relationships of a
  model to silence a warning.
- **Nothing is applied:** the patch is a proposal in JSON Patch form (`add` or `replace` with a JSON
  Pointer). The specification you passed in is not changed; the [playground](../playground/) applies a
  patch only when you press the button and offers *Undo*. The path of the patch is the field the
  warning is about, so a UI can link them.
- **Limits:** a warning in the mobile layout gets guidance, not a patch; there is no correction for
  overlaps. Computing help renders the specification once per candidate, so it costs a few extra
  renders and is only done when asked.

## Warnings

Warnings are written to standard error, and the chart is still produced:

| Code | Meaning |
| --- | --- |
| `text_truncated` | A label or title was shortened to fit. A title is drawn only in the SVG profile and wraps onto a second line first, as do the category labels of bar and range bar charts; only what does not fit on two lines, or a single word wider than the room, is shortened. The legend of a bar chart wraps into further rows and shortens only a name wider than a whole row. |
| `value_labels_omitted` | Some value labels had no room next to their bars. On a bar chart a category shows all of its value labels or none: none when one of them would overlap another value label or another bar of its own or a neighbouring category, or reach into the category labels left of a horizontal plot. |
| `glyph_missing` | PNG only: the bundled fonts have no glyph for some characters of the drawn text, such as a subscript two, a Greek letter or ‰; the PNG draws a box for each, the SVG and HTML output show them. |
| `labels_thinned` | More than 16 categories whose labels do not fit their bands even on two lines: only every few categories are labelled, each with the room of several bands. The data table lists every category. |
| `dense_chart` | More than 16 categories; the chart may be hard to read at this size. |
| `value_outside_band` | A line's value lies outside its own uncertainty band. |
| `color_not_resolved` | Print and social variants only: a layer declares its color as `var(--name)` without a fallback, which they cannot resolve; it is drawn in the text color. Give it a fallback, `var(--name, #2563eb)`, or a literal color so that layers stay distinguishable. |
| `canvas_too_small` | A diagram does not fit its canvas in either orientation and was drawn larger; the path names `/width` or `/height`, the message the size it needs. A page that scales the SVG to its container still shows all of it. |
| `size_exceeded` | A size preset: the content needs more room than the preset's canvas, usually a diagram. The image has the size the content needs, larger than the preset, and nothing is cut; the message names it. Shorten labels, remove elements, or choose a larger preset. |
| `low_contrast` | A color set in `tokens` lies on a color it is drawn on top of with less contrast than WCAG 2.2 asks for: 4.5:1 for text, 3:1 for marks. The path is the token; the message names both colors and the ratio. See [Customize the look](../customize/#contrast). |
| `diagram_over_budget` | A diagram has more than reads well at its size, as an overview: more than 9 steps, states, components or classes or more than 12 edges, transitions, connections or relations (sequence: more than 6 participants or 16 messages; tree: more than 24 nodes). The budget grows with the area of the canvas. Split the diagram into an overview and details, in views for an architecture model, try the other orientation or a larger size, or keep it if it reads well. The counts are a heuristic: a large, clearly laid out diagram may be fine, and a crowded one below the budget is not caught. `chartlet inspect` and `chartlet_inspect` report it as a hint. |
| `group_overlap` | A step outside a group of a flow chart, or a component outside a boundary of an architecture diagram, lies inside its frame. |
| `package_orientation_ignored` | A class diagram package with an `orientation` of its own cannot be laid out as a block: a relation from outside would have to cross another class, relation or label of the package to reach its class, in every orientation the diagram may take. The package follows the diagram's orientation. |
| `port_side_ignored` | A `fromSide` or `toSide` of a flow chart edge or an architecture connection lies along the flow but against the edge, such as `start` on an edge that leads on to a later step, and would turn it back over its own step; the edge meets the step where the flow leads instead. |
| `unreachable_state` | A state diagram names an `initial` state, and no transition leads to this state. |
| `label_overlap` | The label of a zone, reference line or point marker overlaps the label of another annotation, the label of a zone or reference line crosses a data line, or a label reaches outside the plot. The path names the annotation layer; the label is still drawn. A point marker first tries its other sides before this is reported. On a scatter plot: every corner of a quadrant holds points or their names, so the quadrant's name is drawn over them. |
| `quadrant_without_room` | A dividing line of a scatter plot's `quadrants` falls on the end of an axis, so two areas have no room (path `/scatter/quadrants/x` or `y`; fix the range with `xRange` or `yRange`), or an area is too small for its name, which is left out of the drawing (path of the label). The description and the table still name every area. |

A chart with a [mobile variant](../responsive/) is laid out twice in the HTML profile. A warning
that only the mobile layout raises keeps its code and path, and its message begins with
`mobile variant: `; one that the full-size chart already reports is not repeated.

Errors added with bands, reference lines, zones, point markers, stripes, calendars, range bars and small multiples:
`incomplete_band`, `invalid_band`, `missing_label`, `missing_position`, `too_many_annotations`,
`missing_stripes`, `missing_calendar`, `invalid_year`, `invalid_scale`, `invalid_date`,
`date_outside_year`, `duplicate_date`, `invalid_range`, `mid_outside_range`, `not_enough_panes`,
`invalid_columns`, `missing_title`, `duplicate_title`, `missing_group`, `unaligned_stack`,
`negative_in_stack`.

`time_out_of_range` at an annotation's `time` or a zone's path: with `"gaps": "collapse"` a
reference line or point marker lies before the first or after the last observation, or a zone
encloses no observation, so the collapsed time axis has no place for it.

`missing_mobile` at `/mobile`: `--variant mobile` was asked for a specification without `mobile`.

`invalid_token` at `/tokens/<name>`: the name is not a [token](../customize/#which-tokens-exist) or the value is not a hex or `rgb()`/`rgba()` color. `invalid_theme`: a theme file that is not a JSON object with `theme` and `tokens` only; the path names the field.

Pass `--strict` to fail on any warning, for example in CI. To accept a warning you have
reviewed, such as `dense_chart` on a chart that really has 20 categories, let its code through
with `--allow-warning dense_chart` (npm: `allowWarnings: ["dense_chart"]`); it is still reported.

## Inspections

`chartlet inspect model.json` checks an architecture model, a flow chart or a class diagram for
gaps in its documentation — a
separate, explicitly requested profile, apart from errors and warnings. A specification that
renders keeps rendering: no inspection becomes a warning, and `--strict` and `--allow-warning` of
`chartlet render` do not see them. Each finding has a stable code, a severity, a JSON Pointer to
the component, connection, view, step, edge or class, a message and a hint. A `gap` is documentation the
specification has a field for; a `hint` is something to review that may well be intended.
Findings only restate what the model says; they judge neither the architecture nor its
security, nor whether a process or a class model is sound.

| Code | Severity | Reported for | Not reported for |
| --- | --- | --- | --- |
| `unlabelled_connection` | gap | A connection without `label`. | — |
| `connection_without_technology` | hint | A connection without `technology`. | — |
| `component_without_sublabel` | gap | A component without `sublabel` (its technology or host). | A `person`, which has neither. |
| `isolated_component` | hint | A component without any connection. | A model of a single component. |
| `component_in_no_view` | gap | A component of a model with `views` that no view shows, so it is never drawn. | A model without views, which shows every component. |
| `view_adds_nothing` | hint | A view whose components and connections another view shows as well; of two equal views the later one. | A dynamic view: the order of its steps is what it adds. |
| `unlabelled_decision_branch` | gap | Flow: an edge without `label` out of a `decision` with two or more ways on. | A decision with a single way on, which has no branches to tell apart. |
| `unreachable_step` | hint | Flow: a step that no chain of edges leads to from a `start` step. | A flow without a `start` step; a `store` or `external` step, which feeds in from outside. |
| `flow_without_end` | hint | Flow: a flow with a `start` step but no `end` step. | A flow with neither, a part of a larger one. |
| `class_without_members` | gap | Class diagram: a class without `attributes` and `methods`. | A diagram in which no class has members, an overview of names. |
| `isolated_class` | hint | Class diagram: a class without a relation to another class; notes are no relations. | A diagram of a single class. |
| `diagram_over_budget` | hint | Architecture, flow and class diagrams: more items or relations than the [`diagram_over_budget` warning](#warnings) allows for the canvas. | An architecture model with `views`, whose views are drawn one by one. |

A rule that does not fit a model is left out with `--disable <code>`, for example
`--disable connection_without_technology` for a model that leaves protocols to its components,
`--disable isolated_component` when a component stands apart on purpose, or `--disable
unreachable_step` for a flow with more than one way in. `chartlet inspect
--strict` fails only on gaps. The Rust crate runs them with `chartlet::inspect_json`, the npm
package with `inspectChart(spec, { disable })` and the MCP server with the tool
`chartlet_inspect`, all with the same findings.

## Provenance

`--manifest <path>` writes, next to the chart, a JSON record of how it was made, for example to
check in CI that a committed chart still matches its specification and the chartlet version:

| Field | Content |
| --- | --- |
| `chartlet` | Version of the chartlet crate that rendered. |
| `schemaVersion` | `schemaVersion` of the specification. |
| `specHash` | `sha256:` and the hexadecimal SHA-256 of the canonical specification. |
| `outputHash` | `sha256:` and the hexadecimal SHA-256 of the output as UTF-8, exactly as `-o` writes it (standard output adds a final line break). |
| `format` | `svg` or `html`. |
| `variant` | `desktop`, `mobile` or `print`. |
| `idPrefix` | The ID prefix the chart was rendered with: `--id-prefix`, or the one derived from the specification. The mobile variant appends `-m` to it, the print variant `-p`. |
| `view` | Only for an architecture model with `views`: the key of the view that was rendered (`--view`, or the default view). `specHash` stays the hash of the whole model. |
| `views` | Only with `--all-views`: the keys of the views on the page, in its order. `idPrefix` is then the prefix of the page. |
| `perspective` | Only for an architecture diagram that shows a perspective: its key, the one asked for with `--perspective` or the one the model or the view selects. |
| `theme` | Only when a theme was applied by the host: a bundled theme by name, or the path of the theme file, such as `chartlet.theme.json` found in the working directory. |
| `warnings` | The warnings of the render, each with `code`, `path` and `message`. |

The canonical specification is the parsed form, serialized again: key order, whitespace and
number spelling of the input file do not change `specHash`, and defaults that the input leaves
out are part of it. The manifest carries no timestamp, so the same render always produces the
same manifest. The npm package returns the same object as `manifest` with `manifest: true`, see
[JavaScript runtimes](../javascript/#provenance); the Rust crate as `RenderOutput::manifest` with
`RenderOptions { manifest: true, .. }`.
