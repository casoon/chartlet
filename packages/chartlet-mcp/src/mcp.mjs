// The MCP server: six tools, the specification schema as a resource, and the MCP App view of
// chartlet_render. The tools only read data, call the chartlet compiler and compute; there is no
// model and no interpretation here.

import { readFileSync } from "node:fs";

import { registerAppResource, registerAppTool, RESOURCE_MIME_TYPE } from "@modelcontextprotocol/ext-apps/server";
import { CLIENT_CAPABILITIES_META_KEY, McpServer } from "@modelcontextprotocol/server";
import { z } from "zod";

import { explainSpec, renderSpec, validateSpec } from "./charts.mjs";
import { DIAGRAM_TYPES, diagramStarter } from "./diagrams.mjs";
import { CHART_STARTER_TYPES, chartStarter } from "./starters.mjs";
import { DISTINCT_CAP, inspectData } from "./inspect.mjs";
import { supportsView, VIEW_META_KEY, VIEW_RESOURCE_META, VIEW_URI, viewHtml, viewPayload } from "./view.mjs";

const packageJson = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));
const SCHEMA_URL = new URL("../schema/chartlet.schema.json", import.meta.url);

const spec = z
  .union([z.record(z.string(), z.unknown()), z.string()])
  .describe(
    'A chartlet specification: a JSON object, or the same as JSON text. It needs schemaVersion 1, type and title, e.g. {"schemaVersion":1,"type":"bar","title":"Revenue","data":[{"label":"Q1","value":12}]}. The full JSON Schema is the resource chartlet://schema.',
  );

const diagnostic = z.object({
  code: z.string().describe("Stable error or warning code, e.g. invalid_spec, text_truncated."),
  path: z.string().nullable().describe("JSON Pointer into the specification, or null."),
  message: z.string(),
});

const manifest = z
  .object({
    chartlet: z.string(),
    schemaVersion: z.number(),
    specHash: z.string(),
    outputHash: z.string(),
    format: z.string(),
    variant: z.string(),
    idPrefix: z.string(),
    warnings: z.array(diagnostic),
  })
  .describe("Provenance of the render: compiler version and SHA-256 of specification and output.");

const point = z.object({ value: z.number(), at: z.union([z.string(), z.number()]) });
const extreme = z.object({ value: z.number(), at: z.array(z.union([z.string(), z.number()])) });

/**
 * @param {unknown} output
 */
function success(output) {
  return {
    content: [{ type: /** @type {const} */ ("text"), text: JSON.stringify(output, null, 2) }],
    structuredContent: /** @type {Record<string, unknown>} */ (output),
  };
}

/**
 * @param {string} text
 */
function failure(text) {
  return { content: [{ type: /** @type {const} */ ("text"), text }], isError: true };
}

/**
 * @param {{ error: { code: string, path: string | null, message: string }, warnings: unknown[] }} result
 */
function specFailure({ error, warnings }) {
  return failure(
    `chartlet rejected the specification: ${error.code}${error.path == null ? "" : ` at ${error.path}`}: ${error.message}\n` +
      `Fix the field the path points to (chartlet_validate_spec reports the same diagnostics; the schema is chartlet://schema).` +
      (warnings.length > 0 ? `\nWarnings: ${JSON.stringify(warnings)}` : ""),
  );
}

/**
 * @param {{ root?: string }} [options] `root`: the sandbox directory for `outputPath`; defaults to
 *   the environment variable `CHARTLET_MCP_ROOT`. Without it, `outputPath` is refused.
 */
export function createServer({ root = process.env.CHARTLET_MCP_ROOT } = {}) {
  const server = new McpServer({ name: "chartlet", version: packageJson.version });

  server.registerTool(
    "chartlet_inspect_data",
    {
      title: "Inspect tabular data",
      description: `Describe a table before charting it: row count, per column the inferred type, missing values and range, and which chartlet chart types fit the columns. Deterministic; values are read, never changed.

Pass exactly one of:
  - csv: RFC 4180 text with a header row (comma separator; quote fields containing commas, quotes or line breaks; "" escapes a quote).
  - rows: an array of objects, one per row; columns are the keys in order of first appearance.

Per column: type is number, integer, date-time, boolean, string or empty (no values). Missing means an absent key, null or "". Numbers get min/max. date-time columns get timeFormat (year, date or date-time), first/last as written, and monotonic (increasing, decreasing or none, strictly, in row order). A column of four-digit integers counts as years when its name contains "year"/"jahr" or the values strictly increase. Strings get the distinct count (capped at ${DISTINCT_CAP}). Numbers are recognised only in plain notation such as -1234.5 or 1e3; "1,234" stays a string.

suggestions lists fitting chart types with the columns to use and a reason, by fixed rules: category + numbers -> bar; time + numbers -> time (line layers); time + open/high/low/close -> time with an ohlc layer; time + value + lower/upper -> line with uncertainty band; time + group + value -> multiples; category + low/high -> rangebar; category + value + lower/upper -> bar (or line) with error bars; year + one value -> stripes; dates within one year + one value -> calendar; two numbers -> scatter; a repeating category + numbers -> boxplot (violin or strip by boxDisplay); unique labels + positive numbers (6-100 rows) -> treemap; unique labels + values of both signs -> waterfall; from/to columns + a positive number -> sankey; a time-like column + a 0/1 event column -> survival; unique labels + two date columns -> timeline. Use them as starting points, then build the spec yourself and check it with chartlet_validate_spec.`,
      inputSchema: z.object({
        csv: z.string().optional().describe("CSV text with a header row (RFC 4180)."),
        rows: z
          .array(z.record(z.string(), z.union([z.string(), z.number(), z.boolean(), z.null()])))
          .optional()
          .describe('Rows as objects, e.g. [{"month":"2026-01","revenue":12.5}].'),
      }),
      outputSchema: z.object({
        rowCount: z.number().int(),
        columns: z.array(
          z.object({
            name: z.string(),
            type: z.enum(["number", "integer", "date-time", "boolean", "string", "empty"]),
            missing: z.number().int(),
            min: z.number().optional(),
            max: z.number().optional(),
            timeFormat: z.enum(["year", "date", "date-time"]).optional(),
            first: z.union([z.string(), z.number()]).optional(),
            last: z.union([z.string(), z.number()]).optional(),
            monotonic: z.enum(["increasing", "decreasing", "none"]).optional(),
            distinct: z.number().int().optional(),
            distinctCapped: z.boolean().optional(),
          }),
        ),
        suggestions: z.array(
          z.object({
            type: z.string(),
            mark: z.string().optional(),
            columns: z.record(z.string(), z.union([z.string(), z.array(z.string())])),
            reason: z.string(),
          }),
        ),
      }),
      annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    },
    async (input) => {
      try {
        return success(inspectData(input));
      } catch (error) {
        return failure(/** @type {Error} */ (error).message);
      }
    },
  );

  server.registerTool(
    "chartlet_validate_spec",
    {
      title: "Validate a chartlet specification",
      description: `Check a chartlet specification with the real compiler: it is rendered (HTML profile, which also lays out a mobile variant) and the output discarded. Returns ok, the first error with code, JSON Pointer path and message, and all layout warnings (e.g. text_truncated, label_overlap, dense_chart).

Use after drafting or editing a spec and before chartlet_render. An invalid spec is a normal result with ok: false, not a tool error. Fix the field the error path points to and validate again.`,
      inputSchema: z.object({ spec }),
      outputSchema: z.object({
        ok: z.boolean(),
        error: diagnostic.optional(),
        warnings: z.array(diagnostic),
      }),
      annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    },
    async ({ spec }) => success(validateSpec(spec)),
  );

  registerAppTool(
    server,
    "chartlet_render",
    {
      title: "Render a chartlet chart",
      description: `Compile a chartlet specification into static, accessible SVG or HTML: data charts, and software diagrams (sequence, flow, state, architecture, tree; chartlet_diagram_starter gives a starting point). The same spec and options always give the same bytes; the manifest records the compiler version and SHA-256 of spec and output.

Returns content (or, with outputPath, the written path and byte size instead), warnings, styleHashes (CSP 'sha256-…' sources for the inline styles) and manifest. Charts can be 10–200 KB: prefer outputPath when the content does not need to be read.

outputPath writes the file relative to the directory the server was given in CHARTLET_MCP_ROOT and is refused when that variable is not set. Absolute paths, ".." segments and symbolic links leading outside it are refused; missing directories are created and an existing file is overwritten. An invalid spec returns a tool error with code, path and message; run chartlet_validate_spec first.

In a client that shows MCP Apps, the chart also appears in the conversation as a static figure with its data table, following the client's light or dark theme when the spec sets none.`,
      inputSchema: z.object({
        spec,
        format: z
          .enum(["svg", "html"])
          .default("html")
          .describe("html: figure with caption, source and data table (default); svg: the graphic alone."),
        variant: z
          .enum(["desktop", "mobile", "print", "social"])
          .optional()
          .describe("SVG only: mobile renders the spec's mobile layout; needs a mobile field in the spec. print renders the chart with literal colors for PDF and print renderers (resvg, Typst, librsvg). social renders a 1200×630 image for Open Graph previews, with literal colors."),
        idPrefix: z
          .string()
          .optional()
          .describe("Stable, page-unique prefix for element IDs; derived from the spec when omitted."),
        table: z
          .enum(["details", "visible"])
          .optional()
          .describe("HTML only: data table in a disclosure (details, default) or always visible."),
        outputPath: z
          .string()
          .optional()
          .describe('File path relative to CHARTLET_MCP_ROOT to write to, e.g. "charts/revenue.html". Only available when the server has CHARTLET_MCP_ROOT set.'),
      }),
      outputSchema: z.object({
        ok: z.literal(true),
        content: z.string().optional(),
        path: z.string().optional(),
        bytes: z.number().int().optional(),
        warnings: z.array(diagnostic),
        styleHashes: z.array(z.string()),
        manifest,
      }),
      annotations: {
        readOnlyHint: false,
        destructiveHint: true,
        idempotentHint: true,
        openWorldHint: false,
      },
      _meta: { ui: { resourceUri: VIEW_URI } },
    },
    async (input, ctx) => {
      try {
        const result = renderSpec(input, root);
        if ("error" in result) {
          return specFailure(result);
        }
        // Clients of protocol 2026-07-28 send their capabilities with each request; older ones
        // declared them once at initialization.
        const capabilities =
          /** @type {import("@modelcontextprotocol/server").ClientCapabilities | undefined} */ (
            ctx.mcpReq.envelope?.[CLIENT_CAPABILITIES_META_KEY]
          ) ?? server.server.getClientCapabilities();
        return supportsView(capabilities)
          ? { ...success(result), _meta: { [VIEW_META_KEY]: viewPayload(input, result) } }
          : success(result);
      } catch (error) {
        return failure(/** @type {Error} */ (error).message);
      }
    },
  );

  server.registerTool(
    "chartlet_explain",
    {
      title: "Compute facts about a chart",
      description: `Report facts about a chartlet chart, computed and not interpreted: the accessible description chartlet generates (as in the SVG <desc>; the spec's own description is returned separately), the chart type, and per data series or layer its count, missing values, min and max (with every label or time that has that value), and first and last value with label or time.

Diagrams (sequence, flow, state, architecture, tree) have no series; for them structure gives the counts of their elements and every row of their data table in reading order — messages, steps with where they lead, transitions with event, guard and action, or components with their boundaries and connections.

For ohlc layers min is the lowest low, max the highest high, first the first open and last the last close. Zones, reference lines and markers are not series. topicmap and atlas charts have no series. Nothing here judges causes, trends or significance: state such conclusions only from the data and say they are yours.`,
      inputSchema: z.object({ spec }),
      outputSchema: z.object({
        ok: z.literal(true),
        note: z.string(),
        type: z.string(),
        title: z.string(),
        generatedDescription: z.string().nullable(),
        specDescription: z.string().optional(),
        seriesCount: z.number().int(),
        series: z.array(
          z.object({
            pane: z.string().optional(),
            name: z.string().nullable(),
            mark: z.string().optional(),
            count: z.number().int(),
            missing: z.number().int(),
            min: extreme.optional(),
            max: extreme.optional(),
            first: point.optional(),
            last: point.optional(),
          }),
        ),
        structure: z
          .object({
            counts: z.record(z.string(), z.number().int()),
            columns: z.array(z.string()),
            rows: z.array(z.array(z.string())),
          })
          .optional()
          .describe("Diagrams only: the counts of their elements and the rows of their data table, in reading order."),
        warnings: z.array(diagnostic),
      }),
      annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    },
    async ({ spec }) => {
      const result = explainSpec(spec);
      return "error" in result ? specFailure(result) : success(result);
    },
  );

  server.registerTool(
    "chartlet_diagram_starter",
    {
      title: "Start a software diagram",
      description: `Return a valid starting specification for a chartlet software diagram, the kinds its elements can take with the shape each is drawn in, and notes on ids, layout and orientation. Diagrams are drawn from structure, not data: sequence (participants exchanging messages, with fragments), flow (steps and edges, with lanes, groups and a main path), state (states and transitions event [guard] / action, with choices, composite and final states) and architecture (components and connections inside nested boundaries).

chartlet lays a diagram out itself; the specification only says what is connected. Adapt the starter, then check it with chartlet_validate_spec and render it with chartlet_render.`,
      inputSchema: z.object({
        type: z.enum(DIAGRAM_TYPES).describe("The diagram type to start from."),
      }),
      outputSchema: z.object({
        ok: z.literal(true),
        type: z.string(),
        spec: z.record(z.string(), z.unknown()),
        kinds: z.record(z.string(), z.record(z.string(), z.string())),
        notes: z.array(z.string()),
      }),
      annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    },
    async ({ type }) => success(diagramStarter(type)),
  );

  server.registerTool(
    "chartlet_chart_starter",
    {
      title: "Start a chart from a block",
      description: `Return a valid starting specification and notes for a chartlet chart that is drawn from a block of its own rather than from table rows: waterfall (a running total of deltas, starts and totals), waffle (shares as squares), parliament (seats by party with majority and coalition), treemap (rectangles by value, in groups), sankey (flows between nodes), survival (Kaplan-Meier curves from times with censoring), scatter (points with groups, threshold lines and names: volcano and Manhattan plots), timeline (phases, milestones and dependencies), forest (a range bar chart as a forest plot) violin (a box plot drawn as violins), prisma (a PRISMA/CONSORT study-selection flow) and instances (a chain of courts or authorities); the last two are presets on the flow chart.

Adapt the starter, then check it with chartlet_validate_spec and render it with chartlet_render.`,
      inputSchema: z.object({
        type: z.enum(CHART_STARTER_TYPES).describe("The chart to start from."),
      }),
      outputSchema: z.object({
        ok: z.literal(true),
        type: z.string(),
        spec: z.record(z.string(), z.unknown()),
        notes: z.array(z.string()),
      }),
      annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    },
    async ({ type }) => success(chartStarter(type)),
  );

  server.registerResource(
    "schema",
    "chartlet://schema",
    {
      title: "chartlet specification JSON Schema",
      description: "JSON Schema (draft 2020-12) of the chartlet v1 specification that the tools accept.",
      mimeType: "application/schema+json",
    },
    async (uri) => ({
      contents: [
        { uri: uri.href, mimeType: "application/schema+json", text: readFileSync(SCHEMA_URL, "utf8") },
      ],
    }),
  );

  registerAppResource(
    server,
    "figure",
    VIEW_URI,
    {
      title: "chartlet figure",
      description: "MCP App view of chartlet_render: the rendered figure with caption, source and data table.",
      mimeType: RESOURCE_MIME_TYPE,
      _meta: VIEW_RESOURCE_META,
    },
    async (uri) => ({
      contents: [
        { uri: uri.href, mimeType: RESOURCE_MIME_TYPE, text: viewHtml(packageJson.version), _meta: VIEW_RESOURCE_META },
      ],
    }),
  );

  return server;
}
