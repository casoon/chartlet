---
title: Updating a diagram
description: How much of a flow or architecture diagram moves when one node or connection changes, measured, and what to do to keep a diagram recognizable between versions.
order: 6
---

The same specification gives the same bytes. That says nothing about what a reader sees when the
specification changes by one node: a diagram that is regenerated on every edit of a model can
rearrange itself, and the reader's mental map goes with it. This page reports what chartlet does, as
measured, and what to do about it.

## What was measured

`node scripts/layout-stability.mjs --binary target/release/chartlet` renders paired before and
after specifications of three diagrams (`release-flow`, `order-flow`, `tenant-architecture`) and
compares them: nodes the edit did not touch (neither edited nor connected to an edited node), how far
they moved, whether the order of pairs of them flipped, the change in edge crossings and in the size
of the canvas. The edits are the ones a living model sees: add a leaf, rename a node to a longer
label, add an edge, insert a node into an existing path, change the membership of a group or boundary.

| Diagram | Edit | Unrelated nodes moved | Mean / max move | Order flips (pairs) | Crossings (after) | Canvas change |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| release-flow (flow, 3 lanes, 1 group) | reverse the order of the node declarations | 10 of 10 | 299 / 656 px | 15 (45) | -2 (1) | +0 x +0 px |
| release-flow (flow, 3 lanes, 1 group) | add a leaf | 5 of 7 | 4 / 7 px | 0 (21) | +3 (6) | +0 x +5 px |
| release-flow (flow, 3 lanes, 1 group) | rename a node (longer label) | 6 of 6 | 32 / 62 px | 0 (15) | +0 (3) | +0 x +11 px |
| release-flow (flow, 3 lanes, 1 group) | add an edge | 4 of 4 | 20 / 38 px | 0 (6) | +5 (8) | +0 x +56 px |
| release-flow (flow, 3 lanes, 1 group) | insert a node into an existing path | 4 of 5 | 56 / 71 px | 0 (10) | -1 (2) | +0 x +69 px |
| release-flow (flow, 3 lanes, 1 group) | change a group | 0 of 4 | 0 / 0 px | 0 (6) | +0 (3) | +0 x +0 px |
| order-flow (flow, no lanes) | reverse the order of the node declarations | 10 of 10 | 171 / 836 px | 20 (45) | -1 (0) | +0 x +0 px |
| order-flow (flow, no lanes) | add a leaf | 5 of 7 | 4 / 10 px | 0 (21) | +0 (1) | +0 x +0 px |
| order-flow (flow, no lanes) | rename a node (longer label) | 6 of 6 | 41 / 46 px | 0 (15) | +0 (1) | +37 x +0 px |
| order-flow (flow, no lanes) | add an edge | 3 of 3 | 5 / 5 px | 0 (3) | +2 (3) | +1 x +0 px |
| order-flow (flow, no lanes) | insert a node into an existing path | 3 of 3 | 148 / 170 px | 2 (3) | -1 (0) | +136 x +0 px |
| order-flow (flow, no lanes) | change a group | 3 of 4 | 16 / 30 px | 0 (6) | +0 (1) | +5 x +0 px |
| order-flow with a mainPath | reverse the order of the node declarations | 10 of 10 | 171 / 836 px | 20 (45) | -1 (0) | +0 x +0 px |
| order-flow with a mainPath | add a leaf | 5 of 7 | 4 / 10 px | 0 (21) | +0 (1) | +0 x +0 px |
| order-flow with a mainPath | rename a node (longer label) | 6 of 6 | 41 / 46 px | 0 (15) | +0 (1) | +37 x +0 px |
| order-flow with a mainPath | add an edge | 3 of 3 | 5 / 5 px | 0 (3) | +2 (3) | +1 x +0 px |
| order-flow with a mainPath | insert a node into an existing path | 3 of 3 | 148 / 170 px | 2 (3) | -1 (0) | +136 x +0 px |
| order-flow with a mainPath | change a group | 3 of 4 | 16 / 30 px | 0 (6) | +0 (1) | +5 x +0 px |
| tenant-architecture (architecture, 3 nested boundaries) | reverse the order of the node declarations | 11 of 11 | 129 / 347 px | 25 (55) | +0 (0) | +0 x +3 px |
| tenant-architecture (architecture, 3 nested boundaries) | add a leaf | 5 of 5 | 72 / 112 px | 0 (10) | +0 (0) | +0 x +8 px |
| tenant-architecture (architecture, 3 nested boundaries) | rename a node (longer label) | 5 of 5 | 19 / 22 px | 0 (10) | +0 (0) | +0 x +0 px |
| tenant-architecture (architecture, 3 nested boundaries) | add an edge | 6 of 6 | 2 / 2 px | 0 (15) | +0 (0) | +0 x +0 px |
| tenant-architecture (architecture, 3 nested boundaries) | insert a node into an existing path | 3 of 3 | 43 / 87 px | 0 (3) | +0 (0) | +0 x +86 px |
| tenant-architecture (architecture, 3 nested boundaries) | change a boundary | 9 of 9 | 52 / 81 px | 5 (36) | +0 (0) | +0 x +76 px |

Read it with the necessary changes in mind: a longer label widens a node and its neighbours shift to
make room; a node inserted into a path adds a rank, so everything after it moves by about that
rank (the canvas grows by the same amount); a node that changes boundary changes what is next to what.
Those moves are the topology or the content changing, not the layout being unsteady.

## What follows from it

- **Adding to a diagram is mostly steady.** Adding a leaf at the end of the declarations moved the
  other nodes of the flows by 4 px on average (up to 10) and added no order flips. Adding an edge
  moved nodes by 5 to 20 px. Adding a leaf to the architecture moved its nodes by 72 px on average
  although the canvas grew by only 8 px: that is incidental movement, the one clear case in the set.
- **The order of the declarations is a layout input.** Reversing the order of `nodes` (or
  `components`) without changing a single connection moved every node, by 129 to 299 px on average,
  and flipped 15 to 25 of 45 to 55 pairs. Nothing is hidden: the order is in the specification, and
  the layout reads it as the tie-breaker wherever the structure leaves a choice. It is also the
  tool: **append new nodes at the end and keep the existing order**, and an edit stays local.
- **Identity-based tie-breaking would be worse, not better.** Sorting the declarations by `id`
  before rendering (an emulation with `--sorted`, without changing the compiler) makes the layout
  independent of the declaration order, as intended (reversing moves nothing). But it makes edits
  *less* steady, because a new id lands among the others by its name: adding an edge in the
  architecture moved nodes by 89 px on average (up to 353) and flipped 7 of 15 pairs, and changing a group
  of `order-flow` moved every node by 580 px on average. Authors lose the control that declaration
  order gives them, and nothing is gained.
- **`mainPath` did not change these results.** Adding a `mainPath` to `order-flow` gave the same
  numbers: the layout already follows the first path it finds. Pin a path when it is the story
  of the diagram, not as a stability measure. `sameRank` and views keep their place for keeping parts of
  a model side by side; they were not needed to make any of these edits steady.

## Decision

No new layout mode, no layout cache and no previous-layout input. The measured instability is the
declaration order, which is already an explicit, versioned input (the specification itself), and the
alternative that would remove it makes edits worse. What chartlet does instead:

- It documents the rule on this page: **keep the order of existing declarations, append new ones**, and
  review a changed diagram by its figure next to the previous one.
- Geometry and readability stay ahead of positional stability: a longer label, an inserted node or a
  requested orientation changes the arrangement, and should.
- The layout inputs for reproducibility are exactly the specification and the options
  (`orientation`, size, `view`, `perspective`, `theme`); there is no hidden state, no dependence on
  an earlier render, edit history or platform. The manifest's `specHash` covers all of it.

A later option would be an explicit, versioned `layoutHint` that pins positions of named nodes. It
is not planned: the measurements above show no case that the existing mechanisms leave unsolved
except the architecture leaf (72 px of incidental movement), which is within what a reader recognizes.

## Not covered

- **Pilot readers.** The question whether stability helps people understand and review updates needs
  paired examples shown to real users; none were. This page measures positions, not understanding.
- **Larger and portrait diagrams, class and network diagrams.** Three diagrams and five edits are an
  assessment, not a survey. The script takes more subjects.
- **Mobile orientation.** A requested orientation changes the arrangement by design.
