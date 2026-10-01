# Fonts

The `png` feature rasterizes charts with these faces and no system fonts, so the same
specification renders to the same PNG on every machine.

| File | Face | Unicode subset |
| --- | --- | --- |
| `Inter-Regular-latin.ttf` | Inter 400 | Latin |
| `Inter-SemiBold-latin.ttf` | Inter 600 | Latin |
| `Inter-Regular-latin-ext.ttf` | Inter 400 | Latin Extended |
| `Inter-SemiBold-latin-ext.ttf` | Inter 600 | Latin Extended |

Inter is copyright 2016 The Inter Project Authors (https://github.com/rsms/inter) and licensed
under the SIL Open Font License 1.1, see `OFL.txt`. The license declares no Reserved Font Name.

The files are the `inter-latin-*-normal.woff` and `inter-latin-ext-*-normal.woff` subsets of the
npm package `@fontsource/inter` 5.2.8, unpacked from WOFF into the TrueType data they wrap; the
font data itself is unchanged.

Text outside these subsets, such as Greek or Cyrillic, has no glyphs in a PNG. The SVG and HTML
output do not use these files.
