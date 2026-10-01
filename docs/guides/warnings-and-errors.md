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
| `text_truncated` | A label or title was shortened to fit. |
| `value_labels_omitted` | Some value labels had no room next to their bars. |
| `dense_chart` | More than 16 categories; the chart may be hard to read at this size. |
| `topic_too_small_for_label` | A topic map area is too small to hold its own name; consider listing it as an island. |
| `label_does_not_fit` | A region of a knowledge landscape has no room for its name; the area keeps its tooltip. |
| `places_did_not_fit` | A region declares more places than it has ground, and some were left off. |
| `realm_without_structure` | A realm holds a single region, so it has no inner structure to show. |
| `more_places_than_value` | A region lists more places than its value; the two numbers come from different counts. |
| `value_outside_band` | A line's value lies outside its own uncertainty band. |
| `label_overlap` | The label of a zone, reference line or point marker overlaps the label of another annotation, the label of a zone or reference line crosses a data line, or a label reaches outside the plot. The path names the annotation layer; the label is still drawn. A point marker first tries its other sides before this is reported. |

A chart with a [mobile variant](responsive.md) is laid out twice in the HTML profile. A warning
that only the mobile layout raises keeps its code and path, and its message begins with
`mobile variant: `; one that the full-size chart already reports is not repeated.

Errors added with bands, reference lines, zones, point markers, stripes, calendars, range bars and small multiples:
`incomplete_band`, `invalid_band`, `missing_label`, `missing_position`, `too_many_annotations`,
`missing_stripes`, `missing_calendar`, `invalid_year`, `invalid_scale`, `invalid_date`,
`date_outside_year`, `duplicate_date`, `invalid_range`, `mid_outside_range`, `not_enough_panes`,
`invalid_columns`, `missing_title`, `duplicate_title`.

`missing_mobile` at `/mobile`: `--variant mobile` was asked for a specification without `mobile`.

Pass `--strict` to fail on any warning, for example in CI.
