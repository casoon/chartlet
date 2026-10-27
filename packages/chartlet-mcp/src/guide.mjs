// The type selection guide (chartlet://guide/types) and the prompt texts. The list of types and the
// fields each one takes are read from the specification schema, so a new or removed type shows up
// here without an edit; only the plain-language "if you are showing" column is written by hand,
// and a test fails when it and the schema disagree.

import { readFileSync } from "node:fs";

import { ARCHITECTURE_TEMPLATES, DIAGRAM_TYPES } from "./diagrams.mjs";
import { CHART_STARTER_TYPES } from "./starters.mjs";

const SCHEMA_URL = new URL("../schema/chartlet.schema.json", import.meta.url);

/** What each type shows, by the question the reader asks. Keys must equal the schema's types. */
export const SHOWING = /** @type {Record<string, string>} */ ({
  bar: "values compared across categories; shares with `stack`; one value with a reference line",
  line: "a trend over ordered categories (months, versions) without a calendar axis",
  time: "values over real dates or numbers: gaps, bands, candlesticks (ohlc), several stacked panes",
  stripes: "one value per year as warming stripes",
  calendar: "one value per day inside a year, as a calendar heatmap",
  rangebar: "a range per item, as bars or dumbbells, or a forest plot (effects with intervals and a line of no effect)",
  multiples: "the same measure side by side in small panels, one per group",
  boxplot: "the spread of observations per group (box, violin, strip, beeswarm or ridgeline)",
  timeline: "phases, milestones and dependencies on a time axis, such as a roadmap",
  waterfall: "how a start value becomes an end value through additions and subtractions",
  waffle: "a few shares of one whole, as squares",
  funnel: "stages that narrow (visits to orders) with the rate between them, or ranked levels as a pyramid",
  radar: "a few series compared on 3 to 8 measures, as one closed polygon each",
  polar: "one value per category of a cycle (months, hours, directions) around a center",
  mosaic: "two shares at once: column widths by share of the whole, segments by share of the column",
  heatmap: "values, or categories such as a permission matrix, in a grid of rows and columns",
  parliament: "seats by party with a majority line and coalitions",
  treemap: "many positive parts of a whole, in groups, by area",
  sankey: "quantities flowing between stages or nodes",
  survival: "time to an event with censoring (Kaplan-Meier curves)",
  scatter: "two numeric measures per point; with `quadrants` a two-axis prioritization; volcano and Manhattan plots; with `size` a bubble chart (a third value as area)",
  sequence: "time-ordered messages between actors or systems",
  flow: "decision logic with branches (`decision` steps), a process in lanes, a PRISMA flow",
  state: "the states of one thing and the events that move it between them",
  architecture: "components of a system, their connections and boundaries, at one level of abstraction",
  tree: "a hierarchy: organization, ownership, family, a dendrogram, a mind map",
  classes: "classes with attributes and relations, or a database schema",
  network: "who is connected to whom, without a direction of flow or a hierarchy",
  cycle: "steps that feed each other in a loop, such as an improvement loop or a flywheel around a hub",
});

/** Fields of the types that have no block of their own in the schema. */
const TABLE_FIELDS = /** @type {Record<string, string>} */ ({
  bar: "`data` (label, value), or `categories` + `series`",
  line: "`data`, or `categories` + `series` (null for a gap)",
  time: "`panes` with `layers` (mark, data)",
  multiples: "`panes` (at least 2), each with its data",
});

/** @returns {{ types: string[], blocks: Record<string, string>, fields: Record<string, string[]> }} */
export function readSchemaTypes() {
  const schema = JSON.parse(readFileSync(SCHEMA_URL, "utf8"));
  const types = /** @type {string[]} */ (schema.properties.type.enum);
  /** @type {Record<string, string>} */
  const blocks = {};
  for (const clause of schema.allOf ?? []) {
    const type = clause.if?.properties?.type?.const;
    const block = clause.then?.required?.[0];
    if (type && block && schema.properties[block]) {
      blocks[type] = block;
    }
  }
  /** @type {Record<string, string[]>} */
  const fields = {};
  for (const [type, block] of Object.entries(blocks)) {
    const ref = schema.properties[block].$ref;
    const def = ref ? schema.$defs[ref.split("/").pop()] : schema.properties[block];
    fields[type] = Object.keys(def?.properties ?? {});
  }
  return { types, blocks, fields };
}

/** @param {string} type */
function startedBy(type) {
  if (/** @type {readonly string[]} */ (DIAGRAM_TYPES).includes(type)) {
    const templates =
      type === "architecture" ? `, template ${ARCHITECTURE_TEMPLATES.map((t) => `\`${t}\``).join(", ")}` : "";
    return `\`chartlet_diagram_starter\` (type \`${type}\`${templates})`;
  }
  if (/** @type {readonly string[]} */ (CHART_STARTER_TYPES).includes(type)) {
    return `\`chartlet_chart_starter\` (type \`${type}\`)`;
  }
  return "`chartlet_inspect_data` suggests the columns";
}

/** The guide as Markdown. */
export function typeGuide() {
  const { types, blocks, fields } = readSchemaTypes();
  const rows = types.map((type) => {
    const names = fields[type] ?? [];
    const own = blocks[type]
      ? `\`${blocks[type]}\`: ${names.slice(0, 8).map((f) => `\`${f}\``).join(", ")}${names.length > 8 ? ", …" : ""}`
      : (TABLE_FIELDS[type] ?? "");
    return `| ${SHOWING[type] ?? "(not described)"} | \`${type}\` | ${own} | ${startedBy(type)} |`;
  });
  return `# Choosing a chartlet type

Every \`type\` of the specification, by what you are showing. Start from a starter when the last column names one, then check with \`chartlet_validate_spec\` and render with \`chartlet_render\`. The full schema is \`chartlet://schema\`.

| If you are showing | Use | Fields | Start from |
| --- | --- | --- | --- |
${rows.join("\n")}

## When not to draw

- A list of items with a few attributes each: a table. A chart adds nothing the rows do not say.
- One number, one comparison, one shape of a trend: a sentence ("Revenue rose from 12 to 15").
- More than about 12 categories on one bar chart, or more than 4 lines: group, aggregate or filter first, and say what you left out.
- Data that is mostly missing, or a diagram of more than about 15 elements: split it into views or levels, or give a table.

## Readability budgets

- Long labels are truncated or overlap: shorten them, use \`"orientation": "horizontal"\`, or move detail into \`description\`.
- One chart answers one question; the title states the message.
- Pass a \`mobile\` block when the figure is read on narrow screens.
- \`diagram_over_budget\` (more than about 9 nodes or 12 edges, a sequence more than 6 participants or 16 messages) asks for an overview and details: views, two diagrams or a larger size, never a silent cut.
- Warnings such as \`text_truncated\`, \`label_overlap\` and \`dense_chart\` mean the reader loses something; treat them as findings.

## Common mistakes

- Choosing by look, not by question: a few shares of a whole are \`waffle\`, many parts \`treemap\`, a change through steps \`waterfall\`.
- A time axis on text months: use \`time\` for real dates, \`line\` or \`bar\` for ordered labels.
- A series with fewer values than categories: write \`null\` for the gap (\`series_length_mismatch\`).
- Ids that are reused or point nowhere in a diagram: every message, edge or relation names ids that exist.
- Changing values or removing edges to make an error disappear: fix the field the error path points to, or tell the reader what was changed.
- Mixing blocks: a type takes its own block only (a \`flow\` has \`flow\`, never \`sequence\`).
`;
}

/**
 * The text of a prompt: a numbered procedure over the existing tools.
 *
 * @param {"diagram" | "chart" | "redraw"} kind
 * @param {string} request
 */
export function promptText(kind, request) {
  const rules = `Rules:
- Infer routine defaults (size, type, labels) from the request and state them in one short message; ask only when data are missing or two readings change the meaning.
- Keep the exact data and relationships. Propose any aggregation or cut explicitly; never fix a validation error by silently changing values or deleting edges.
- Stop after three rounds with the same error, or when no type fits: report a diagnosis and the partial specification instead of looping.
- If a table or a sentence serves better than a drawing (see the guide), say so instead of drawing.
- Deliver the editable specification (JSON), the rendered result (use outputPath when the server has a root) and the warnings that remain.`;
  const steps = {
    diagram: `1. Read the resource chartlet://guide/types and pick the type for what the request describes.
2. State your plan in one message: type, size, and what you will leave out.
3. Call chartlet_diagram_starter (chartlet_chart_starter for the block chart types) and adapt the specification to the request.`,
    chart: `1. Read the resource chartlet://guide/types.
2. Call chartlet_inspect_data with the data (csv or rows) and choose among its suggestions.
3. State your plan in one message: type, columns, and what you will aggregate or leave out. Draft the specification from the real values (chartlet_chart_starter for the block chart types).`,
    redraw: `1. Read the resource chartlet://guide/types and pick the chartlet type that carries the source's structure.
2. Read the source (text, image or a file the user names) and list its elements and relations before drafting; state what chartlet cannot show.
3. Call chartlet_diagram_starter or chartlet_chart_starter and rebuild the specification element by element.`,
  }[kind];
  const subject = {
    diagram: "Draw this as a chartlet diagram",
    chart: "Make a chartlet chart from this data",
    redraw: "Redraw this as a chartlet specification",
  }[kind];
  const coverage =
    kind === "redraw"
      ? "\n6. Call chartlet_check_coverage with the specification and the inventory you read in step 2 (source ids, from/to of edges, members of groups; say it is your extraction) and show its report to the user: what was kept, changed, merged, ambiguous, missing and added. Ask about every omission you did not propose."
      : "";
  return `${subject}:

${request}

${steps}
4. Call chartlet_validate_spec until ok is true; fix the field the error path points to.
5. Call chartlet_render (format html or svg) with the specification.${coverage}

${rules}`;
}
