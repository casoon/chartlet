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

Pass `--strict` to fail on any warning, for example in CI.
