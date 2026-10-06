// Starting specifications for the chart types that are drawn from a block of their own rather
// than from a table of rows: each is a valid spec without warnings, with notes on what the type
// takes and refuses.

/** The types chartlet_chart_starter knows. */
export const CHART_STARTER_TYPES = /** @type {const} */ ([
  "waterfall",
  "waffle",
  "parliament",
  "treemap",
  "sankey",
  "survival",
  "scatter",
  "timeline",
  "forest",
  "violin",
]);

/** @type {Record<string, Record<string, unknown>>} */
const STARTERS = {
  waterfall: {
    schemaVersion: 1,
    type: "waterfall",
    title: "From revenue to profit",
    showValues: true,
    waterfall: {
      steps: [
        { label: "Revenue", value: 1250, kind: "start" },
        { label: "Materials", value: -420 },
        { label: "Staff", value: -380 },
        { label: "Gross margin", kind: "total" },
        { label: "Licenses", value: 60 },
        { label: "Operating profit", kind: "total" },
      ],
    },
  },
  waffle: {
    schemaVersion: 1,
    type: "waffle",
    title: "Where the electricity came from",
    waffle: {
      parts: [
        { label: "Wind", value: 31 },
        { label: "Solar", value: 12 },
        { label: "Gas", value: 28 },
        { label: "Coal", value: 17 },
      ],
      total: 100,
    },
  },
  parliament: {
    schemaVersion: 1,
    type: "parliament",
    title: "Seats after the election",
    parliament: {
      parties: [
        { label: "Left alliance", seats: 38 },
        { label: "Social democrats", seats: 41 },
        { label: "Conservatives", seats: 52 },
      ],
      majority: true,
      coalition: ["Left alliance", "Social democrats"],
    },
  },
  treemap: {
    schemaVersion: 1,
    type: "treemap",
    title: "Where the budget goes",
    treemap: {
      items: [
        { label: "Schools", value: 310, group: "Society" },
        { label: "Hospitals", value: 280, group: "Society" },
        { label: "Roads", value: 120, group: "Infrastructure" },
        { label: "Defence", value: 150, group: "Security" },
        { label: "Police", value: 85, group: "Security" },
      ],
    },
  },
  sankey: {
    schemaVersion: 1,
    type: "sankey",
    title: "From energy source to use",
    sankey: {
      links: [
        { from: "Gas", to: "Power", value: 120 },
        { from: "Wind", to: "Power", value: 90 },
        { from: "Power", to: "Households", value: 100 },
        { from: "Power", to: "Losses", value: 110 },
      ],
    },
  },
  survival: {
    schemaVersion: 1,
    type: "survival",
    title: "Overall survival by treatment",
    survival: {
      groups: [
        {
          label: "Standard",
          observations: [{ time: 3.1 }, { time: 8, event: false }, { time: 12.4 }, { time: 15 }, { time: 20, event: false }],
        },
        {
          label: "New drug",
          observations: [{ time: 6.3 }, { time: 14 }, { time: 18, event: false }, { time: 22 }, { time: 30, event: false }],
        },
      ],
      confidence: true,
      timeTitle: "Months since randomization",
    },
  },
  scatter: {
    schemaVersion: 1,
    type: "scatter",
    title: "Differential expression",
    scatter: {
      points: [
        { x: -3, y: 6.2, group: "Down", label: "TP53" },
        { x: 2.4, y: 5.1, group: "Up", label: "MYC" },
        { x: 0.3, y: 0.6, group: "Not significant" },
        { x: -0.4, y: 1.1, group: "Not significant" },
      ],
      lines: [{ axis: "x", value: -1 }, { axis: "x", value: 1 }, { axis: "y", value: 1.3, label: "p = 0.05" }],
      xTitle: "log2 fold change",
      yTitle: "−log10 p",
    },
  },
  timeline: {
    schemaVersion: 1,
    type: "timeline",
    title: "Product roadmap",
    timeline: {
      items: [
        { id: "research", label: "Research", start: "2027-01-11", end: "2027-02-19", group: "Discovery" },
        { id: "build", label: "Build", start: "2027-02-22", end: "2027-05-28", group: "Build", after: ["research"] },
        { id: "release", label: "Release", at: "2027-06-15", group: "Build", after: ["build"] },
      ],
      markers: [{ label: "Today", at: "2027-03-08" }],
    },
  },
  forest: {
    schemaVersion: 1,
    type: "rangebar",
    orientation: "horizontal",
    title: "Effect of the intervention",
    valueAxis: { title: "Difference (negative favors the intervention)" },
    references: [{ label: "No effect", value: 0 }],
    ranges: [
      { label: "Study A (n=240)", low: -7.8, high: -1.2, mid: -4.5, weight: 24 },
      { label: "Study B (n=120)", low: -9.9, high: 1.7, mid: -4.1, weight: 11 },
      { label: "Overall", low: -4.7, high: -1.9, mid: -3.3, summary: true },
    ],
  },
  violin: {
    schemaVersion: 1,
    type: "boxplot",
    boxDisplay: "violin",
    title: "Response times by endpoint",
    valueAxis: { title: "Milliseconds" },
    boxes: [
      { label: "Search", values: [42, 45, 47, 48, 50, 52, 53, 55, 58, 61, 64, 140] },
      { label: "Checkout", values: [88, 92, 95, 101, 104, 108, 112, 118, 125, 131] },
    ],
  },
};

/** @type {Record<string, string[]>} */
const NOTES = {
  waterfall: [
    'A step is a delta (value, may be negative), a "start" (bar from zero) or a "total" (the running total; a value must match it, total_mismatch).',
    '"orientation": "horizontal" lays the bars on their side.',
  ],
  waffle: [
    "Up to four parts, each with at least one square; a total above their sum leaves the squares of the rest.",
    "cells (10–400, default 100) and columns (2–40) set the grid.",
  ],
  parliament: [
    "Up to 8 parties and 800 seats, in blocks from left to right in the order of the list.",
    "coalition names parties by label (unknown_node otherwise); majority draws the line at the seat that makes one.",
  ],
  treemap: [
    "Up to 100 items with unique labels and values above zero; group goes on every item or on none, up to four groups.",
    "Rectangles too small for a name leave it out (value_labels_omitted); the table lists every item.",
  ],
  sankey: [
    "Nodes are the labels the links name. A link joins two different nodes, once per pair, with a value above zero; links must not run in a circle.",
    "Columns follow the longest path; flows need not balance, a node is as tall as the larger of in and out.",
  ],
  survival: [
    "Every observation has a time (zero or more) and event (default true); event false is censored and marked on the curve.",
    "Up to four groups, 2–2000 observations each. confidence draws the 95% band, atRisk (default true) the number at risk under the axis.",
  ],
  scatter: [
    "Up to 5000 points; group on every point or none (up to four, the first point of each fixes its color). lines are thresholds at a value of x or y.",
    "Above 300 points the dots are small without tooltips; above 100 the table lists only labeled points, or the 20 highest. Label the points that matter.",
  ],
  timeline: [
    "An item is a phase (start and end) or a milestone (at); after names ids it follows; markers are dashed days such as today.",
    "Dates are ISO days; the canvas grows with the number of items.",
  ],
  forest: [
    "A rangebar with a weight on a span draws a square on mid sized by the weight (needs mid); summary true draws a diamond for the overall result.",
    "references draws the line of no effect.",
  ],
  violin: [
    'boxDisplay "violin" draws the density with the box inside; "strip" draws every observation as a point. Both need values on every box (values_required).',
    "Every violin is as wide as the others at its widest, so compare shapes, not areas.",
  ],
};

/**
 * A valid starting specification for `type` and notes on what it takes.
 *
 * @param {string} type
 */
export function chartStarter(type) {
  return {
    ok: /** @type {const} */ (true),
    type,
    spec: structuredClone(STARTERS[type]),
    notes: NOTES[type],
  };
}
