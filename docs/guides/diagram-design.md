---
title: Diagram design
description: The editorial look of diagrams, a calmer alternative to the classic look. What it changes, the tokens, sizes and spacing it uses, and how it was chosen.
---

# Diagram design

Diagrams have two looks. `classic` is the default: every kind of node has a color of its own.
`editorial` is neutral paper and ink with one accent, small type tags on nodes, monospace for
technical text, and edge labels on a mask without a border. Both are in every release; you choose
per specification, per project or per run.

```json
{ "type": "architecture", "look": "editorial", "architecture": { "components": [] } }
```

```sh
chartlet render spec.json --theme editorial      # for one run
chartlet render spec.json --theme classic        # the other way, e.g. against a project theme file
```

A theme file may set `"look": "editorial"` next to its `tokens`, so a project can make it the look of
all its diagrams (`chartlet.theme.json`). The look combines with every palette: `"theme": "dark"`
gives dark paper, `"theme": "blueprint"` keeps its blue ground and takes the neutral boxes, hairlines
and masks. Charts that are not diagrams take the paper and ink palette only, so a figure and its
diagrams on one page belong together; see the `editorial-bars` and `editorial-trend` examples.

## What the look changes

| Part | Classic | Editorial |
| --- | --- | --- |
| Kind of a node | a fill and an outline color per kind | shape and a type tag; one neutral fill and outline for every kind |
| Accent | blue, also on badges and activation bars | the main path and its arrowheads only; elements an emphasis marks use it too |
| Outlines and lines | 1.5 px, a shadow below boxes | 1 px, no shadows; 2 px for the main path |
| Corners | 7 px | 6 px (states 8 px, frames 8 px) |
| Names | sans, 13 px, weight 600 | the same |
| Sublabels, frame names, the technology of an edge | sans | monospace, 11 px; names, edge labels and members stay sans |
| Edge labels | on a bordered plate, 4 px from the line | on a mask in the color of the paper, no border, 8 px from the line; a technology joins the label after a dot, the name in sans and the technology in monospace: `calls · HTTPS/JSON` |
| Number badges | accent disc | ink disc |
| Roles | color, line pattern and badge | the same; `critical` keeps a red outline, `planned` and `deprecated` are ink with their pattern and badge |

The type tag is a small plate with capital letters on the top edge of a node, 8 px from its left
corner. Every kind has a default (`DB`, `QUEUE`, `CACHE`, `FILES`, `SEC`, `UI`, `USER`, `EXT`,
`HOST`, `ARTIFACT`; `SVC` for an ordinary architecture component), `"tag": "API"` on a flow step or
component replaces it (at most 8 characters), and `"tags": false` leaves the tags off. Class diagrams tag
`IFACE`, `ABSTRACT`, `ENUM` and `ENTITY`; trees tag `USER` and `EXT` nodes. Tags are not drawn on
decisions, initial dots, notes and interfaces, and not on a canvas narrower than 480 px; the data
table and the description still name the kind. Sequence participants, network nodes and cycle steps
have no kinds to tag, and a sequence participant's shape tells what it is.

Tokens, tags and masks never carry meaning alone: the kind is also the shape, the role is also a line
pattern and a badge, the main path is also thicker.

## Tokens

The editorial look uses the existing tokens, so every token of a specification or a theme file still
wins. In the vocabulary of the design brief: `background` is the paper, `node` and `lane-head` are
the second paper, `text` is the ink, `grid` is the rule, `zero` is the stroke of outlines and lines.

| Token | Light | Dark | Used for |
| --- | --- | --- | --- |
| `background` (paper) | `#fcfbf8` | `#17161a` | the ground, the label masks, the tag plates |
| `node` (paper 2) | `#f2efe8` | `#222127` | the fill of every node |
| `text` (ink) | `#1d1c1a` | `#ece8df` | names, number badges |
| `muted` | `#4d4a44` | `#b7b2a7` | captions, legends |
| `zero` | `#6b665c` | `#8f8a80` | outlines, lines, frames, tag plates |
| `grid` (rule) | `#d8d4ca` | `#3a3840` | lane borders, grid lines (decoration only) |
| `accent` | `#0b6a88` | `#62b8d4` | the main path |
| `role-red` | `#a8281c` | `#f08a7e` | the `critical` role |
| `role-violet` | `#4d4a44` | `#c9c4b8` | the `planned` role |
| `color-1` to `color-4` | accent, `#a8431a`, `#6b665c`, ink | `#62b8d4`, `#e0916a`, `#9a958a`, ink | series of charts |
| `shadow` | transparent | transparent | no shadows |

The other role tokens are set to the neutral outline and fill, so a custom stylesheet that reads them
stays neutral. Contrast, measured and tested (`tests/editorial.rs`): text on paper and on the second
paper is at least 8:1 for ink and 7.5:1 for muted text; outlines, lines and the accent are at least
4.6:1 against the paper, far above the 3:1 that marks need. The tags and masks use the same pairs.
Print and PNG resolve the same values; the print variant carries no custom properties. The `low_contrast` warning for a token of your own measures it against this palette when the look is `editorial`.

## Sizes and spacing

All values are pixels at the canvas size; a diagram is never scaled by the look.

| Part | Value |
| --- | --- |
| Name | 13, weight 600, line height 16 |
| Sublabel, group name, technology of an edge | monospace 11 |
| Edge or message label, class member | sans 12, line height 15 |
| Type tag | monospace 9, tracking 0.5, plate 14 high, 5 padding left and right, 8 in from the corner |
| Node padding | 12 along the text, 24 height beyond the text lines (classic: 20) |
| Gap between nodes in a layer | 32; between layers 32; around frames 32 (classic: 28, 28, 30) |
| Gap between a label mask and its line | 8 (classic: 4); masks keep 3 clear of nodes and of each other |
| Outline, line | 1; main path 2; end node 2 |
| Corner radius | 6; states and frames 8 |

Distances that are multiples of four: the gaps, the corner of the tag, the label gap, the frame
radius and the node height beyond its text. The text sizes and the 15 px line height follow the
type, not the grid, and the lines of an edge keep their routing.

Monospace text (11 px and smaller) is measured as 0.6 times its size per character, which is what the system monospace
faces and the bundled face give; widths are the same in the web output, the print variant and PNG.
In PNG the monospace face is JetBrains Mono (SIL Open Font License, `fonts/`), next to Inter.

## How the look was chosen

Two treatments were drawn for the same specifications, flow (`release-flow`, `order-flow`), sequence
(`sign-in-sequence`), architecture (`tenant-architecture`, `shop-architecture`), state
(`ticket-states`) and classes (`order-model`), each in light, dark and at 360 px, and compared with
the classic look in a browser and as PNG.

- **A, paper.** Boxes filled with a second paper, tags on plates that cut the outline, one accent for
  the path. This is `editorial`.
- **B, ink.** No fills, ink outlines, tags as plain accent text on the outline, labels in the
  muted ink.

B reads as drawn with a pen and prints well, but its tags sit on the outline and are struck through by
it, boxes and frames stop being different things without a fill, and a main path in the accent is the
only thing that sets the eye. A keeps every kind readable by shape and tag, keeps frames, lanes and
nodes apart by fill and line, and is the one that survives dark paper. It was chosen.

What the review found, and what it changed:

- Monospace sublabels and technologies are wider than sans text. A diagram that fitted exactly grows a little
  (`canvas_too_small` names the size); the committed examples are sized for it.
- With the emphasis and the legend strip (`release-focus`, `shop-focus`, `sign-in-focus`) the emphasized
  elements take the accent over the neutral fills, and the legend swatches are neutral; checked on
  desktop, dark and at 360 px.
- The tag stands half outside the box. The first row of nodes therefore gets 7 px less room above;
  the title margin covers it.
- On a canvas narrower than 480 px the tags are dropped and labels wrap sooner. At 360 px a class
  diagram needs about 420 px with monospace members; keep members short or use a wider canvas.
- Sequence labels sit 8 px above their arrow; a numbered diagram keeps its badges on the arrow.
- Dense case: `editorial-dense-architecture` (11 components, long labels, 2052 px wide) and
  `editorial-dense-flow` (10 steps in three lanes) are drawn without clipped text or covered nodes;
  the test `label_masks_clear_nodes_and_each_other` checks that no mask covers a node or another mask.
- Rounded corners need a renderer that reads `rx` from CSS; the PNG output draws square corners for
  boxes in both looks.

The review is the author's, in a browser and in PNG, not a study; look at the examples yourself:
`editorial-flow`, `-sequence`, `-architecture`, `-states`, `-classes`, their `-dark` twins, the
`.mobile.svg` and `.print.svg` variants, and the companion charts `editorial-bars` and
`editorial-trend`.

## Making editorial the default

The classic look stays the default so that no existing output changes. To make `editorial` the
default, move `#[default]` from `Classic` to `Editorial` in `enum Look` in `src/spec/mod.rs`, swap the
test of `skip_serializing_if` (`Look::is_classic` becomes `is_editorial`) so that `classic` is the
value that is written out, and regenerate the reference files in `examples/`. Specifications that name
`"look": "classic"` or are rendered with `--theme classic` keep the old look. Nothing else reads the
default: the stylesheet, the layout and the schema key off `spec.look`; change `"default"` of `look`
in `schema/chartlet.schema.json` to match.
