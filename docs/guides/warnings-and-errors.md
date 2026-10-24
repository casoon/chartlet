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
| `warnings` | The warnings of the render, each with `code`, `path` and `message`. |

The canonical specification is the parsed form, serialized again: key order, whitespace and
number spelling of the input file do not change `specHash`, and defaults that the input leaves
out are part of it. The manifest carries no timestamp, so the same render always produces the
same manifest. The npm package returns the same object as `manifest` with `manifest: true`, see
[JavaScript runtimes](../javascript/#provenance); the Rust crate as `RenderOutput::manifest` with
`RenderOptions { manifest: true, .. }`.
