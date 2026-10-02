---
title: Time series
description: Line charts and candlesticks on a real time axis — timestamps, timezone, layers, stacked panes, colors, theme, and the limits for dense data.
order: 2
---

`"type": "time"` draws lines on a calendar axis. An observation carries a timestamp instead of a
category, so the horizontal position is a real distance in time and the ticks land on calendar
boundaries rather than in the middle of a band.

```json
{
  "schemaVersion": 1,
  "type": "time",
  "title": "Daily orders",
  "timeAxis": { "timezone": "UTC", "title": "Day" },
  "panes": [
    {
      "valueAxis": { "title": "Orders" },
      "layers": [
        {
          "mark": "line",
          "name": "Orders",
          "points": [
            { "time": "2026-03-01", "value": 412 },
            { "time": "2026-03-02", "value": 438 }
          ]
        }
      ]
    }
  ]
}
```

## Timestamps

`time` is either Unix seconds or an ISO 8601 string: `1772323200`, `"2026-03-01"`,
`"2026-03-01T12:00:00Z"`, or `"2026-03-01T12:00:00+02:00"`; a month such as `"2026-03"` stands for its first day. A bare year such as `"1850"` stands
for January 1st; when every observation falls on January 1st, tooltips, description and table
label them by year.

A timestamp that does not carry its own offset is read as wall-clock time in
`timeAxis.timezone` — a fixed UTC offset such as `"+02:00"`, `UTC` by default. `"2026-03-01"`
therefore means midnight in that zone, and shifting the zone moves absolute timestamps but not
bare dates.

Timestamps must increase within a layer and lie between 1700-01-01 and 2200-01-01.

## Layers

A pane holds up to six data layers (eight in a stacked pane), each a `line`, an `area`, a `point` or an `ohlc` candlestick layer, and up
to six annotation layers: zones, reference lines and point markers together.
Four palette colors are available for the whole chart: at most four line and area layers go
without a `color` of their own, and a fifth one has to bring one (`too_many_layers` otherwise).
Candles take no palette color. A layer without a color takes the palette color of its position
among the chart's line and area layers, counted across all panes; from the fifth position on it
takes the first palette color no earlier layer holds. Give each layer a `name` as soon as the
chart has more than one, and keep the names unique across all panes: they label the one legend
and the data table. A legend that does not fit one
row wraps into a second.
A layer may name its own `color` in the same forms chartlet accepts anywhere — `#rgb`,
`#rrggbb`, `#rrggbbaa`, or `var(--your-variable)`, optionally with a hex fallback:
`var(--your-variable, #0f766e)`. A `var()` reference keeps the host page in charge of the value
and falls back to its fallback, or else to the chart's text color, when the page defines nothing,
so a series never disappears silently. The print variant draws the fallback. A value outside the contract becomes the neutral gray `#667085`
and is reported as a `color_not_supported` warning.

`"stroke": "thin"` draws a line at one pixel instead of three, for example single years under
their running mean; `"stroke": "bold"` draws it at 4.5 pixels to bring it forward. `"dash"` sets
the pattern: `solid`, `dashed`, or `dotted`. A modeled layer is dashed unless `dash` says
otherwise. The legend draws a short piece of each line — color, weight and pattern — so two lines
never differ by color alone.

## Missing values

`"value": null` marks an observation that is missing, for example while a sensor was offline. The
line breaks there instead of bridging the gap, and so do its area and its band; a single value
between two gaps keeps its marker but has no line to draw. The data table shows the timestamp with
“Missing”, and the description counts the missing values. A point without a value carries no
`lower` or `upper` (`invalid_band`).

## Gaps in time

By default the time axis keeps the distances in time: a weekend between two trading days takes
two days of room, with nothing drawn in it. `"timeAxis": { "gaps": "collapse" }` closes those gaps.
Every distinct timestamp that an observation uses — in any layer of any pane, a missing value and a
candle included — becomes one slot, and the slots sit at equal distances, so Friday and Monday are
neighbours and a holiday takes no space.

```json
"timeAxis": { "title": "Trading day", "gaps": "collapse" }
```

The ticks still mark calendar boundaries, chosen as on any time axis: each sits on the first
observation on or after its boundary and is labelled with that observation's date — a weekly tick
after a holiday Monday reads Tuesday. Boundaries that fall into one gap give a single tick.

Reference lines, point markers and zones are not observations, so they take the slot of one: a
reference line or a point marker the first observation at or after its `time`, a zone the
observations from the first at or after `from` to the last at or before `to`. A reference line or
marker before the first or after the last observation, and a zone that encloses no observation,
have no slot and are refused with `time_out_of_range` at their path; a zone that reaches beyond the
observations is cut at the first and the last of them. Zoom windows work the same way and close the
gaps of the observations inside them; an annotation that has no observation to stand on in a
window is left out of it.

Tooltips, data table and description keep the real timestamps; the description adds “Gaps in time
are closed up.” Small multiples share one set of slots across their panels. The density limits
below still count observations per layer.

## Areas

`"mark": "area"` is a line whose region down to zero is filled in the line's color at low
opacity; the line is drawn on top at full strength and carries the markers and tooltips. An area
always starts at zero, so a chart with an area layer includes zero on its value axis. The legend
draws a swatch of the fill under the line sample, and the description names the filled layers.
The area *between* two lines — a range, a corridor, an uncertainty — is not a second area mark:
give the line `lower` and `upper` on every point, as described below.

`"stack": "normal"` on a pane stacks its areas in layer order, each on top of the ones before it,
so that the top edge shows their total — generation by source, visitors by channel. The areas
need the same times and curve and a value of zero or more at every time; use 0 where a source
has nothing. Markers and end labels sit on top of the stack, while tooltips, the data table and
the description give each layer's own value. A line in the same pane, such as demand, is not
stacked. `legend: "end"` names the layers where they end, from the bottom up. The series toggles
of the interactive module hide a stacked area without moving the others, so its band stays empty.

```json
"panes": [{
  "stack": "normal",
  "layers": [
    { "mark": "area", "name": "Fossil", "points": [ … ] },
    { "mark": "area", "name": "Wind and solar", "points": [ … ] }
  ]
}]
```

## Points

`"mark": "point"` draws a dot for every observation and no line between them: readings that
scatter around a trend, or measurements that are not a continuous series. A dot carries the
tooltip of its observation; above 60 observations the dots are drawn smaller and lose their value
labels, but are never left out. The legend shows a dot instead of a line sample. A point layer
takes no line options — `dash`, `stroke`, `curve`, `modeled` — and no `lower`/`upper` band.

```json
"layers": [
  { "mark": "point", "name": "Morning reading", "points": [ … ] },
  { "mark": "line", "name": "Weekly mean", "points": [ … ] }
]
```

## Candlesticks

`"mark": "ohlc"` draws one candle per observation in `data`: a wick from `low` to `high` and a body
from `open` to `close`.

```json
{ "mark": "ohlc", "name": "KSTL", "data": [
  { "time": "2026-03-02", "open": 50.2, "high": 51.9, "low": 49.8, "close": 51.4 },
  { "time": "2026-03-03", "open": 51.4, "high": 51.6, "low": 50.1, "close": 50.6 }
] }
```

A rising candle — `close` at or above `open` — has a hollow body, a falling one a filled body, so
the direction never rests on color alone. The two colors are CSS custom properties,
`--chartlet-rise` and `--chartlet-fall`, with values for the light and the dark theme that keep at
least 4.5:1 against the background; a page can override them like any other chart color. A
candlestick layer therefore takes no `color`, and no `modeled`, `stroke` or `dash` either
(`option_not_supported`). The bodies take 70 % of the median distance between two candles, at
least one pixel and at most 16; a candle whose open and close are equal keeps a body one pixel
high.

Every candle needs `low` at or below its open and close and `high` at or above them
(`invalid_candle` with the path of the field), finite numbers, and a later `time` than the candle
before it (`unordered_time`); a layer takes 2 to 2000 candles. The legend draws a hollow and a
filled candle and says “hollow: rising, filled: falling”. Each body has a tooltip with open,
high, low and close; the data table has one column for each of the four, and the description
names the first open, the last close, the change between them in absolute terms and in percent,
and the highest high and lowest low with their times.

More candles than a third of the plot's horizontal pixels — 234 at the default width — leave
less than three pixels per candle. The layer is then drawn as wicks only, from low to high, in the
rise and fall colors, without bodies and tooltips, and the render reports `dense_chart`. A zoom
step with fewer candles draws the bodies again. Candles closer than 4 pixels keep their bodies but, like the
observations of a line, lose their tooltips.

A candlestick pane may carry line layers such as a moving average, which are drawn over the
candles, as well as zones, reference lines and point markers. Candles are drawn on `time` charts
only; small multiples refuse them.

## Panes

A `time` chart takes up to four `panes`. They stack from top to bottom and share one time axis:
its tick labels and its title sit below the bottom pane, and its vertical grid lines run through
every pane. Each pane has its own `valueAxis` — title, `format` and `decimals` — and its own value
scale, fitted to its layers and annotations. `heightRatio` (1–10, default 1) sets the share of the
height a pane takes against the others: `3` and `1` give the upper pane three quarters.

```json
"panes": [
  { "heightRatio": 3, "valueAxis": { "title": "Price (USD)", "decimals": 2 }, "layers": [ … ] },
  { "heightRatio": 1, "valueAxis": { "title": "Volume (million shares)" }, "layers": [
    { "mark": "area", "name": "Volume", "stroke": "thin", "points": [ … ] }
  ] }
]
```

There is no volume mark: trading volume is an `area` layer (or a `line`) in a pane of its own.
All panes share one legend at the top, which is why layer names are unique across the whole chart.
Reference lines, zones and point markers belong to their pane and stay inside it. Zoom steps
show the same window in every pane, and each window has to hold two observations of a layer in
every pane (`zoom_out_of_range`). The data table stays one table with a column for every layer
of every pane, each written in its pane's format. The description names the panes by their
`valueAxis.title` — “Pane 2” when a pane has none — and gives the extremes of each pane
separately. A pane lower than 40 pixels is reported as `dense_chart` at its `heightRatio`. A pane
takes no `title`; that heads the panels of small multiples.

## Uncertainty bands and modeled lines

Give every point of a line `lower` and `upper` to draw a band around it, in the line's color. A
layer with `"modeled": true` is drawn dashed, its band hatched, and its legend entry and the
description say “(modeled)”, so the distinction does not rest on color. The data table gets a
`lower` and an `upper` column for the layer. Both edges on every point with a value, or on none, is
required (`incomplete_band`); `lower` above `upper` is `invalid_band`; a value outside its own band is
reported as `value_outside_band`.

## Zoom steps

`zoomSteps` pre-render two to four windows of time, switched by radio buttons in the HTML output.
On a time chart `from` and `to` are timestamps in the same forms as a point's `time`:

```json
"zoomSteps": [
  { "label": "Four weeks", "from": "2026-03-01", "to": "2026-03-28" },
  { "label": "Last week", "from": "2026-03-22", "to": "2026-03-28" }
]
```

Each window is drawn from the observations inside it, with both axes fitted to them; reference
lines and point markers at a time outside the window are left out, zones are cut at its edges or
left out when they miss it, and every layer keeps its color and its legend
entry. `from` must lie before `to` (`invalid_zoom_step`), and the window must hold at least two
observations of one layer (`zoom_out_of_range`). The data table always lists every observation,
and the SVG profile stays a single chart. Small multiples do not take zoom steps yet.

## Reference lines

An `"annotation"` layer with `value` and `label` draws a dashed horizontal line across the plot,
for a threshold such as 1.5 °C; with `time` and `label` it draws a vertical marker. Reference lines
widen the axes so they are always visible, are named in the description, and carry a tooltip; they
are not series, so they have no legend entry and no table column.

## Zones

A `"band"` layer shades a zone behind the data: a range of values between `bottom` and `top`, a
span of time between `from` and `to`, or a rectangle with both. A missing edge is the edge of the
plot, so `from` and `to` alone mark a window across the full height, for example a maintenance
window, and `bottom` and `top` alone a range across the whole time axis, such as a target range.

```json
{ "mark": "band", "label": "Maintenance", "from": "2026-02-16", "to": "2026-02-19" },
{ "mark": "band", "label": "Above the error budget", "bottom": 1, "top": 2.5 }
```

A zone is filled at low opacity in its `color`, or in the neutral gray, and drawn before every
line, so the data stays on top. Its `label` is required and sits in the zone's top left corner when
the zone is tall enough, otherwise just above or below it. Given edges widen the axes. A zone has
no legend entry and no table column; the description lists it under “Zones” with its extent, and
its tooltip says the same.

## Point markers

An `"annotation"` with both `time` and `value` draws a marker symbol at that point, for an event
such as a release or an incident:

```json
{ "mark": "annotation", "label": "Release 2.4", "time": "2026-02-24", "value": 2.14, "shape": "triangle-up" }
```

`shape` is `circle` (the default), `square`, `diamond`, `triangle-up`, or `triangle-down`; only a
point marker takes one. Shape and label carry the meaning together, so a marker never depends on
its color. The label sits to the right of the symbol; when that leaves the plot or runs into
another label, a data line or a marker, it tries the left, then above and below. The marker has a
native tooltip with label, time and value, widens both axes, and is listed under “Markers” in the
description.

## Label collisions

chartlet measures every annotation label with the same text metrics as the rest of the layout.
When the label of a zone, reference line or point marker overlaps another annotation label, when
the label of a zone or reference line crosses a data line, or when a label reaches outside the
plot, the render reports a `label_overlap` warning with the path of the annotation layer. The
label is never dropped; move the annotation or shorten its label.

## Sparklines

`"sparkline": true` turns a one-pane time chart into a word-sized graphic for a table cell or a
dashboard tile: only its lines and areas, a dot at the end of each line, no axes, title or
legend, down to 60 × 16 pixels. Its title and description still name and describe it, the
tooltips still carry the values, and the HTML profile keeps the caption and the data table.

```json
{ "schemaVersion": 1, "type": "time", "title": "Visitors, last 30 days",
  "sparkline": true, "width": 120, "height": 32, "panes": [{ "layers": [ … ] }] }
```

## Names at the end of the lines

With a few lines that end apart, a name beside each line reads faster than a legend: `"legend":
"end"` writes every series name at its last observation, right of the plot, and leaves out the
legend. The plot makes room for the widest name; names that would overlap move apart.

## Steps and areas between lines

A value that stands for a whole period — an annual mean, a tariff, a quota — reads best as a step:
`"curve": "step"` on a line or area layer holds each value until the next observation and then
jumps. The last value holds until `stepEnd`, when given — `"stepEnd": "2026"` ends the step of
the 2025 mean at the end of 2025 without adding an observation for 2026. The area between two
lines, such as a target and a projection, is a band: give the line
its points with `lower` and `upper`, and `"modeled": true` hatches it.

## Numbers instead of dates

Profiles and deep time have no calendar: elevation along a distance, temperature down a borehole,
an isotope record over millions of years. `"timeAxis": { "kind": "number" }` reads every `time`
as a plain number and keeps everything else of a time chart — bands, zones, reference lines,
point markers, panes. Ticks fall on round numbers, values are written in the chart's `locale`,
and the axis `title` names the first column of the data table. `"reverse": true` runs the axis
from right to left, so that ages before present read from the oldest on the left; the value axis
takes `"reverse": true` too, for records drawn with larger values downward.

```json
"timeAxis": { "kind": "number", "reverse": true, "title": "Million years ago" }
```

## Small multiples

`"type": "multiples"` draws 2 to 12 panes as small time charts in a grid of `columns`. Every panel
needs a unique `title`; the panels share the top-level `valueAxis`, the time span and one legend,
in which a layer name keeps its color across all panels. Panels draw smaller markers and no value
labels; the tooltips and the table (one column per panel and layer) carry the values.

When the panels measure different things — population, energy use and water withdrawal, say —
one shared axis flattens all but the largest. `"independentAxes": true` gives every panel its own
value axis, scaled to its own values; the top-level `valueAxis` still sets format, decimals,
thousands separator and a logarithmic `scale` for all of them. Heights can then no longer be
compared across panels, and the description says that each panel has a value axis of its own.

## Theme

`"theme": "dark"` switches the palette. Both themes are drawn from CSS custom properties on the
chart root, so a page can override any single value; the dark theme additionally paints its own
background, which makes the SVG self-contained. To take the page's own tokens, set the properties
with a selector more specific than the chart's own, for example
`.report .chartlet-root { --chartlet-text: var(--color-text); }`. A dark chart defines its palette
on `.chartlet-root.chartlet-theme-dark`, so it keeps its colors next to a light chart on the same
page; to override it, add the theme class to your selector
(`.report .chartlet-root.chartlet-theme-dark`).

Every other rule in a chart's stylesheet is scoped to the chart's root ID (the `--id-prefix`), so
several inline charts on one page never restyle each other. The custom properties above are the
theming interface; the class names of individual marks are not.

## Language and numbers

`"locale": "de"` writes every generated text in German — description, legend additions, tooltips,
the HTML caption and data table — and writes numbers with a decimal comma. Text from the
specification is never translated. Every chart type takes a locale.

Axis ticks carry as many decimals as their step, so an axis reads `0.0, 0.5, 1.0`. For every other
value, `valueAxis.decimals` (0–6) fixes the decimals: `1.547` with two decimals is `1.55`. Negative
numbers use the true minus sign `−`, which screen readers announce as “minus”.

## Title

The title is the accessible name of the chart. The HTML profile shows it once, as the
`<figcaption>`, and draws no title inside the SVG. When the page already heads a standalone SVG,
`"showTitle": false` leaves the drawn title out of it too and gives its space to the plot; the title
stays in `<title>`.

## Dense series

chartlet draws markers and value labels only while they stay readable — up to 60 observations per
layer. Above that the layer is drawn as a line only. Tooltips follow one rule for lines and
candles: every observation keeps its tooltip, on an invisible target where no marker is drawn,
as long as neighbouring observations stand at least 4 pixels apart; denser layers have none. The
values always remain in the data table, which is part of the HTML profile.

The invisible targets cost bytes: a line of 365 daily values is several times larger with them.
`"tooltips": "markers"` keeps tooltips on drawn markers only, so a line too dense for markers
has none — the right choice for a page with many dense charts, or with the crosshair of the
interactive module, which reads the values from the data table anyway.

The limits are measured, not guessed. At the default width of 800 pixels, four layers of 2000
observations each render to 131 KB of SVG (355 KB of HTML) in about 14 ms; six layers, the
maximum, render to 196 KB of SVG. Beyond one observation
per plot pixel — 704 of them at the default width — the drawing cannot show every point, and
chartlet reports that as a `dense_chart` warning.

| Limit | Value |
| --- | --- |
| Observations or candles per layer | 2000 |
| Data layers per pane | 6; at most 4 of the chart in palette colors |
| Zones, reference lines and point markers per pane | 6 together |
| Panes | 1–4 (`time`), 2–12 (`multiples`) |
| `heightRatio` | 1–10 |
| Candles with bodies | a third of the plot's horizontal pixels per layer |
| Observations with markers and value labels | 60 per layer |

## Value labels

`showValues` draws a label above every point. A single line stays readable that way; with two or
more lines the labels of neighbouring series can land on top of each other: the collision check
covers annotation labels, not value labels. The dark example in the showcase therefore draws
its two lines without labels and leaves the values to the tooltips and the data table.

## Not here yet

These are part of the next milestones and are refused with a named error instead of being
ignored: `zoomSteps` and candlesticks on small multiples.