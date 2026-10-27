// The MCP server: eight tools, three prompts, the specification schema and the type guide as
// resources, and the MCP App view of chartlet_render. The tools only read data, call the chartlet compiler and compute; there is no
// model and no interpretation here.

import { readFileSync } from "node:fs";

import { registerAppResource, registerAppTool, RESOURCE_MIME_TYPE } from "@modelcontextprotocol/ext-apps/server";
import { CLIENT_CAPABILITIES_META_KEY, McpServer } from "@modelcontextprotocol/server";
import { z } from "zod";

import { inspectChart } from "@casoon/chartlet";

import { explainSpec, renderSpec, resolveOutputPath, validateSpec } from "./charts.mjs";
import { checkCoverage, DETAILS } from "./coverage.mjs";
import { ARCHITECTURE_TEMPLATES, DIAGRAM_TYPES, diagramStarter } from "./diagrams.mjs";
import { promptText, typeGuide } from "./guide.mjs";
import { themeFromBrand, writeThemeFiles } from "./theme.mjs";
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
    view: z.string().optional().describe("The key of the view of an architecture model that was rendered."),
    views: z.array(z.string()).optional().describe("The keys of the views shown on one page with allViews, in page order."),
    perspective: z.string().optional().describe("The key of the perspective an architecture diagram showed."),
    theme: z.string().optional().describe("The theme that was applied: a bundled name, inline, or chartlet.theme.json from the project root."),
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

When no theme is given and CHARTLET_MCP_ROOT holds a chartlet.theme.json, that theme is applied like the theme option; the result and the manifest name the theme that was applied. Without such a file, notes says that the default look is used and how to set one (chartlet_theme_from_brand).

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
        view: z
          .string()
          .optional()
          .describe("The key of a view of an architecture model (architecture.views) to render; its default view when omitted. Each view is a diagram of its own with its own title, description and table; the manifest names the view."),
        perspective: z
          .string()
          .optional()
          .describe("The key of a perspective of an architecture model (architecture.perspectives) to show, in place of the one the model or the view selects; the manifest names it."),
        theme: z
          .union([
            z.enum(["light", "dark", "blueprint"]),
            z.object({
              theme: z.enum(["light", "dark", "blueprint"]).optional(),
              tokens: z.record(z.string()).optional(),
            }),
          ])
          .optional()
          .describe("Restyles the chart without changing its data: a bundled theme by name, or a theme object {theme, tokens} whose tokens are colors named like the --chartlet-* custom properties without the prefix (accent, text, background, color-1 to color-4, role-blue, ...), as hex values. The spec's own theme and tokens win; the same object works for print, social and PNG."),
        allViews: z
          .boolean()
          .optional()
          .describe("HTML only, without view: every view of an architecture model on one page, the default view first, with a navigation of links to the views and each view as a section with its own figure and table; the manifest lists the views."),
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
        theme: z.string().optional().describe("The theme that was applied; absent when the default look was used."),
        notes: z.array(z.string()).optional().describe("Hints for the assistant, e.g. that no project theme was found and how to set one."),
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

For ohlc layers min is the lowest low, max the highest high, first the first open and last the last close. Zones, reference lines and markers are not series. Nothing here judges causes, trends or significance: state such conclusions only from the data and say they are yours.`,
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
    "chartlet_inspect",
    {
      title: "Inspect a model for gaps",
      description: `Run chartlet's inspections of a model for gaps in its documentation, without rendering it — the same findings as \`chartlet inspect\`. Available for architecture diagrams (connections without a label or technology, components without a sublabel, isolated components, components no view shows, views that add nothing), flow charts (unlabelled branches of a decision, steps no start leads to, a start without an end) and class diagrams (classes without members, isolated classes); for all three, a diagram_over_budget hint when it has more nodes or edges than read well at its size (split it into views or two diagrams).

Each finding has a stable code, a severity, the JSON Pointer of the part it is about, what was found and how to improve it. A gap is missing documentation the specification has a field for; a hint is something to look at, which may well be intended — never treat a hint as an error. Findings never stop a render. Pass the codes of inspections that do not apply in disable.

An invalid specification, or a type without inspections (inspections_unavailable), returns a tool error with code, path and message.`,
      inputSchema: z.object({
        spec,
        disable: z
          .array(z.string())
          .optional()
          .describe('Inspection codes to leave out, as findings report them, e.g. ["connection_without_technology"].'),
      }),
      outputSchema: z.object({
        ok: z.literal(true),
        findings: z.array(
          z.object({
            code: z.string().describe("Stable inspection code, e.g. unlabelled_connection."),
            severity: z.enum(["gap", "hint"]),
            path: z.string().describe("JSON Pointer into the specification."),
            message: z.string(),
            hint: z.string().describe("How to improve it."),
          }),
        ),
      }),
      annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    },
    async ({ spec, disable }) => {
      const result = inspectChart(spec, { disable });
      return result.ok ? success(result) : specFailure({ error: result.error, warnings: [] });
    },
  );

  const color = z.string().describe("A hex color such as #1a2b3c (or rgb(26, 43, 60)).");
  const scheme = {
    paper: color.describe("The page or card background."),
    ink: color.describe("The body text color."),
    muted: color.optional().describe("Secondary text, such as axis labels; derived from paper and ink when omitted."),
    accent: color.describe("The brand's main color: bars, lines, highlights; also the first series color."),
    link: color.optional().describe("The link color; becomes the blue role of diagrams, the accent when omitted."),
  };
  server.registerTool(
    "chartlet_theme_from_brand",
    {
      title: "Make a theme from brand colors",
      description: `Turn brand values you read from a site or a style guide into a chartlet theme file. The server reads no site and has no network access: fetching the colors is your job, and you pass them in as sampled. Fonts are accepted and recorded, but chartlet has no font setting yet, so they are listed as not applied and nothing about them is written to the theme.

Returns a light theme file and a dark one, and a receipt: every role with the value you gave (input), the value written (output), whether it was adjusted and why (the pair, the measured ratio and the one reached; text needs 4.5:1, marks 3:1, in light and dark), and whether it was sampled, derived or a bundled default. Your values are never changed unless a pair fails. When you give only light values, the dark ones are derived and marked derived; pass dark to use the brand's own. A theme file holds one palette: use the light file as chartlet.theme.json and apply the dark one with chartlet_render theme.

With write: true the files are written below CHARTLET_MCP_ROOT (chartlet.theme.json and chartlet.theme.dark.json, or path and its .dark.json twin), which chartlet_render then applies to every chart without a theme of its own. Existing files are not overwritten unless overwrite is true; one project's theme never replaces another's. Render a chart afterwards and check the result.`,
      inputSchema: z.object({
        ...scheme,
        fonts: z
          .object({ names: z.string().optional(), technical: z.string().optional() })
          .optional()
          .describe("Font families for names and for technical text, as the brand names them. Recorded in the receipt only."),
        dark: z
          .object({
            paper: scheme.paper,
            ink: scheme.ink,
            muted: scheme.muted,
            accent: scheme.accent,
            link: scheme.link,
          })
          .optional()
          .describe("The brand's own dark values; derived from the light ones when omitted."),
        write: z.boolean().default(false).describe("Write the files below CHARTLET_MCP_ROOT."),
        path: z.string().default("chartlet.theme.json").describe("Where to write the light file, relative to CHARTLET_MCP_ROOT; must end in .json."),
        overwrite: z.boolean().default(false).describe("Replace files that exist."),
      }),
      outputSchema: z.object({
        ok: z.literal(true),
        files: z.object({
          light: z.record(z.string(), z.unknown()),
          dark: z.record(z.string(), z.unknown()),
        }),
        darkDerived: z.boolean(),
        receipt: z.object({
          roles: z.array(z.record(z.string(), z.unknown())),
          adjusted: z.array(z.record(z.string(), z.unknown())),
          fonts: z.array(z.record(z.string(), z.unknown())),
          notes: z.array(z.string()),
        }),
        written: z.array(z.object({ path: z.string(), bytes: z.number().int() })).optional(),
      }),
      annotations: { readOnlyHint: false, destructiveHint: false, idempotentHint: true, openWorldHint: false },
    },
    async ({ write, path, overwrite, ...brand }) => {
      try {
        const result = themeFromBrand(brand);
        if (!write) {
          return success(result);
        }
        return success({ ...result, written: writeThemeFiles(result.files, path, overwrite, root, resolveOutputPath) });
      } catch (error) {
        return failure(/** @type {Error} */ (error).message);
      }
    },
  );

  server.registerTool(
    "chartlet_check_coverage",
    {
      title: "Check what a redrawn diagram kept",
      description: `Compare a redrawn diagram with the inventory of its source: what survived, what is missing and what was added. chartlet reads no foreign formats; pass the elements you read from the source, labelled as your own extraction. Available for flow, state, architecture, classes, tree, network and sequence diagrams. Nothing is rendered and nothing is fetched.

Source elements are { kind: "node" | "edge" | "group", label, id?, from?, to?, directed?, members? }. Give ids when you have them: a label alone is not an identity, so a repeated label is reported as ambiguous, not kept. An edge is matched by its endpoints (from and to are source ids or labels) and direction, a group by label and members. mappings ({ source: id or label, to: id or JSON Pointer of the specification's element }) settle ambiguity and declare merges. An element of the specification may carry sourceLabel or sourceId as the source's name for it; remove them before rendering.

The report has kept, changed (a reversed direction, another label, other members, an endpoint that was merged), merged, ambiguous, missing and added, with counts: every source element is in exactly one of kept, changed, merged, ambiguous and missing. detail (faithful, balanced, simplified) only marks which missing elements are expected omissions; it never removes one from the report. Show the report to the user.`,
      inputSchema: z.object({
        spec,
        source: z
          .array(
            z.object({
              kind: z.enum(["node", "edge", "group"]),
              label: z.string().optional().describe("Required for a node or group; optional for an edge."),
              id: z.string().optional().describe("The source's own id, if it has one."),
              from: z.string().optional().describe("Edge: source id or label of the start."),
              to: z.string().optional().describe("Edge: source id or label of the end."),
              directed: z.boolean().optional().describe("Edge: false when direction carries no meaning; default true."),
              members: z.array(z.string()).optional().describe("Group: source ids or labels of its nodes."),
            }),
          )
          .describe("The elements read from the source."),
        mappings: z
          .array(
            z.object({
              source: z.string().describe("Source id, or label when it has no id."),
              to: z.string().describe("Id or JSON Pointer of the specification's element."),
              kind: z.enum(["node", "edge", "group"]).optional(),
            }),
          )
          .optional(),
        detail: z.enum(DETAILS).default("balanced").describe("How faithful the redraw should be; only marks missing elements as expected."),
        inventory: z.string().default("agent-extracted").describe('How the inventory was made, e.g. "agent-extracted" or "parsed".'),
      }),
      outputSchema: z.object({
        ok: z.literal(true),
        type: z.string(),
        inventory: z.string(),
        detail: z.string(),
        counts: z.record(z.string(), z.unknown()),
        kept: z.array(z.record(z.string(), z.unknown())),
        changed: z.array(z.record(z.string(), z.unknown())),
        merged: z.array(z.record(z.string(), z.unknown())),
        ambiguous: z.array(z.record(z.string(), z.unknown())),
        missing: z.array(z.record(z.string(), z.unknown())),
        added: z.array(z.record(z.string(), z.unknown())),
        unusedMappings: z.array(z.record(z.string(), z.unknown())),
        report: z.string(),
      }),
      annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    },
    async (input) => {
      try {
        return success(checkCoverage(input));
      } catch (error) {
        return failure(/** @type {Error} */ (error).message);
      }
    },
  );

  server.registerTool(
    "chartlet_diagram_starter",
    {
      title: "Start a software diagram",
      description: `Return a valid starting specification for a chartlet software diagram, the kinds its elements can take with the shape each is drawn in, and notes on ids, layout and orientation. Diagrams are drawn from structure, not data: sequence (participants exchanging messages, with fragments), flow (steps and edges, with lanes, groups and a main path), state (states and transitions event [guard] / action, with choices, composite and final states), architecture (components and connections inside nested boundaries), tree (an organization or ownership structure, or a radial mind map) and classes (class and entity-relationship diagrams: classes with attribute and method compartments, joined by relations with cardinalities).

For an architecture diagram, template picks a starter at one level of abstraction, modeled in the usual architecture fields: context (the system as one box, its users and the external systems it calls), containers (the separately running or deployable parts inside the system boundary; a container is such a part, not a container image) or deployment (a logical model of the deployable parts plus a deployment view of production: hosts nested in region and cluster, two instances of the API, managed queue, database and storage). The result names the question the template answers and its level. These are starting points, not a complete architecture model: each is one diagram, and elements are not kept in sync across levels.

chartlet lays a diagram out itself; the specification only says what is connected. Adapt the starter, then check it with chartlet_validate_spec and render it with chartlet_render.`,
      inputSchema: z.object({
        type: z.enum(DIAGRAM_TYPES).describe("The diagram type to start from."),
        template: z
          .enum(ARCHITECTURE_TEMPLATES)
          .optional()
          .describe("Only with type architecture: a starter at one level of abstraction (context, containers, deployment)."),
      }),
      outputSchema: z.object({
        ok: z.literal(true),
        type: z.string(),
        spec: z.record(z.string(), z.unknown()),
        kinds: z.record(z.string(), z.record(z.string(), z.string())),
        notes: z.array(z.string()),
        template: z.string().optional(),
        question: z.string().optional(),
        level: z.string().optional(),
      }),
      annotations: { readOnlyHint: true, idempotentHint: true, openWorldHint: false },
    },
    async ({ type, template }) =>
      template && type !== "architecture"
        ? failure(`template ${template} is an architecture template; use it with type architecture.`)
        : success(diagramStarter(type, template)),
  );

  server.registerTool(
    "chartlet_chart_starter",
    {
      title: "Start a chart from a block",
      description: `Return a valid starting specification and notes for a chartlet chart that is drawn from a block of its own rather than from table rows: waterfall (a running total of deltas, starts and totals), waffle (shares as squares), heatmap (values or categories in a grid of rows and columns), parliament (seats by party with majority and coalition), treemap (rectangles by value, in groups), sankey (flows between nodes), survival (Kaplan-Meier curves from times with censoring), scatter (points with groups, threshold lines and names: volcano and Manhattan plots), timeline (phases, milestones and dependencies), cycle (steps on a ring that feed each other, optionally around a hub), forest (a range bar chart as a forest plot) violin (a box plot drawn as violins), prisma (a PRISMA/CONSORT study-selection flow) and instances (a chain of courts or authorities); the last two are presets on the flow chart.

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

  server.registerResource(
    "type-guide",
    "chartlet://guide/types",
    {
      title: "chartlet type selection guide",
      description:
        "Which chartlet type and fields to use for what you want to show, when not to draw, readability budgets and common mistakes. Built from the schema's types.",
      mimeType: "text/markdown",
    },
    async (uri) => ({
      contents: [{ uri: uri.href, mimeType: "text/markdown", text: typeGuide() }],
    }),
  );

  const request = z.string().describe("What should be shown, in the user's words.");
  for (const [name, title, description, argsSchema] of /** @type {const} */ ([
    ["diagram", "Draw a diagram", "From a description to a rendered chartlet diagram: pick the type, validate, render.", { request }],
    [
      "chart",
      "Chart some data",
      "From data to a rendered chartlet chart: inspect the data, pick the type, validate, render.",
      { request, data: z.string().optional().describe("The data as CSV text, if it is not already in the conversation.") },
    ],
    ["redraw", "Redraw an existing diagram", "Redraw a diagram from text, an image or a file as a chartlet specification, with a coverage report.", { request }],
  ])) {
    server.registerPrompt(name, { title, description, argsSchema: z.object(argsSchema) }, async (args) => ({
      messages: [
        {
          role: "user",
          content: {
            type: "text",
            text: promptText(name, "data" in args && args.data ? `${args.request}\n\nData (CSV):\n${args.data}` : args.request),
          },
        },
      ],
    }));
  }

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
