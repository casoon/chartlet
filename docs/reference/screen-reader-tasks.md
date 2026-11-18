---
title: Screen reader test tasks
description: The task matrix for testing chartlet output with VoiceOver and NVDA, with the expected result of each task and a place to record what was found.
order: 7
---

This is the test plan behind the screen reader rows of the [support matrix](../support/). **No task has
been run yet**: every result below is "Not run". A result goes into this page only with the versions
it was observed with, and the support matrix changes only after that. An axe pass does not replace
these tasks: it checks the markup, not what a person hears.

## Setup to record

Fill this in before the first task, once per session.

| Item | Value |
| --- | --- |
| Date, tester | |
| Operating system and version | |
| Browser and version | |
| Screen reader and version | |
| chartlet version (`chartlet --version`) | |
| Page: files of `examples/` opened from a local copy, or the [showcase](https://casoon.github.io/chartlet/) | |

Pairs to cover: VoiceOver with Safari on macOS, NVDA with Firefox or Chromium on Windows. JAWS stays
a named follow-up.

## Tasks

Open the `.html` file of each example (the HTML figure), not only the `.svg`. A task passes when the
expected result holds without sighted help; write down what was announced word for word where it
differs.

| # | Task | Example | Expected result | VoiceOver / Safari | NVDA / Firefox |
| --- | --- | --- | --- | --- | --- |
| 1 | Find the chart and read it | `budget-vs-actual` | The chart is one image with the title as its name and the generated description (categories, series, highest and lowest value, missing values). The title is announced once. | Not run | Not run |
| 2 | Read the values | `budget-vs-actual` | The data table is reachable, announced as a table with its caption, and has a row for every value, including values whose label was left out. | Not run | Not run |
| 3 | Open and close the table disclosure | `budget-vs-actual` | The disclosure announces its state (collapsed, expanded) and works with the keyboard alone. | Not run | Not run |
| 4 | Filter series | `budget-vs-actual` | The filter is a checkbox per series: reachable by keyboard, announcing name and checked state; hiding a series removes it from the chart and from what is read, showing it again restores both. | Not run | Not run |
| 5 | Dense time series | `sensor-readings` | The description gives the number of points, the time span, the series, the highest and lowest value and the missing values; the table is complete and navigable by row and column. The zoom controls (radio buttons) announce their name and state. | Not run | Not run |
| 6 | Flow | `editorial-dense-flow` | The description names every step in reading order and says where each leads; the table has a row per step. The focus disclosure (radio buttons) announces its choices. | Not run | Not run |
| 7 | Sequence | `async-export` | Participants and the ten messages are named in order of time; the table has a row per message. | Not run | Not run |
| 8 | Architecture and focus | `booking-containers` | The description names every component; the focus control is a group of radio buttons that announces its choices, and choosing one keeps the reading order sensible. | Not run | Not run |
| 9 | Dark theme and forced colors | `budget-vs-actual` with `"theme": "dark"` | Nothing is announced differently; in forced colors every mark and label stays visible (note what is lost, forced colors are not yet supported). | Not run | Not run |
| 10 | Zoom to 200 % and 400 % | `budget-vs-actual` | No loss of content or function; the table stays reachable. | Not run | Not run |
| 11 | Phone width and keyboard only | `budget-by-department` (`mobile` variant) | Only one of the two variants is read; the hidden one is not announced. All controls are reachable without a pointer. | Not run | Not run |
| 12 | Reduced motion | any example with a control | No motion that the page does not need when the system asks for reduced motion. | Not run | Not run |

## Embedding contexts

Use `budget-vs-actual` in each context and record what the reader hears and loses.

| Context | Expected | What the host must provide | Result |
| --- | --- | --- | --- |
| Inline SVG in a page | One image with name and description | A caption or text next to it if the page wants the numbers | Not run |
| HTML figure (default) | Name, description, caption and the data table | Nothing further | Not run |
| `<img src="chart.svg">` | Title and description survive as the image's text; the figure wrapper, the table and the controls do not apply | `alt` text, because many screen readers ignore the SVG's own title for an image element | Not run |
| Markdown / README embedding | Not yet evaluated | `alt` text, and the numbers in the text if they matter | Not run |

## Recording defects

Log a defect as an issue with the task number, the pair, the versions and the exact announcement.
Do not widen a claim in the [accessibility guide](../../guides/accessibility/) while a task is open.
