import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";

const server = fileURLToPath(new URL("../src/server.mjs", import.meta.url));
const schema = readFileSync(new URL("../../../schema/chartlet.schema.json", import.meta.url), "utf8");

const spec = {
  schemaVersion: 1,
  type: "bar",
  title: "Revenue",
  data: [
    { label: "Q1", value: 12 },
    { label: "Q2", value: 15 },
  ],
};

/**
 * Starts the server over stdio with the bundled WebAssembly renderer and the given environment.
 *
 * @param {Record<string, string>} extra
 */
async function connect(extra) {
  const env = { ...process.env, ...extra };
  delete env.CHARTLET_BIN;
  if (!("CHARTLET_MCP_ROOT" in extra)) {
    delete env.CHARTLET_MCP_ROOT;
  }
  const client = new Client({ name: "chartlet-mcp-test", version: "0.0.0" });
  await client.connect(
    new StdioClientTransport({ command: process.execPath, args: [server], cwd: tmpdir(), env }),
  );
  return client;
}

test("serves the tools and the schema over stdio", async (t) => {
  const root = mkdtempSync(join(tmpdir(), "chartlet-mcp-e2e-"));
  const client = await connect({ CHARTLET_MCP_ROOT: root });
  t.after(async () => {
    await client.close();
    rmSync(root, { recursive: true, force: true });
  });

  const { tools } = await client.listTools();
  assert.deepEqual(tools.map(({ name }) => name).sort(), [
    "chartlet_chart_starter",
    "chartlet_diagram_starter",
    "chartlet_explain",
    "chartlet_inspect_data",
    "chartlet_render",
    "chartlet_validate_spec",
  ]);
  for (const tool of tools) {
    assert.ok(tool.description.length > 100, tool.name);
    assert.equal(tool.inputSchema.type, "object");
    assert.ok(tool.outputSchema, tool.name);
  }
  const annotations = Object.fromEntries(tools.map(({ name, annotations }) => [name, annotations]));
  assert.equal(annotations.chartlet_inspect_data.readOnlyHint, true);
  assert.equal(annotations.chartlet_render.readOnlyHint, false);

  const rendered = await client.callTool({ name: "chartlet_render", arguments: { spec, format: "svg" } });
  assert.equal(rendered.isError, undefined);
  assert.match(rendered.structuredContent.content, /^<svg/);
  assert.equal(rendered.structuredContent.manifest.format, "svg");
  assert.deepEqual(JSON.parse(rendered.content[0].text), rendered.structuredContent);

  const written = await client.callTool({
    name: "chartlet_render",
    arguments: { spec: JSON.stringify(spec), outputPath: "charts/revenue.html" },
  });
  assert.equal(written.structuredContent.path, join("charts", "revenue.html"));
  assert.equal(
    written.structuredContent.bytes,
    readFileSync(join(root, "charts/revenue.html")).length,
  );

  const escape = await client.callTool({
    name: "chartlet_render",
    arguments: { spec, outputPath: "../escape.html" },
  });
  assert.equal(escape.isError, true);
  assert.match(escape.content[0].text, /must not contain/);

  const invalid = await client.callTool({
    name: "chartlet_render",
    arguments: { spec: { ...spec, type: "pie" } },
  });
  assert.equal(invalid.isError, true);
  assert.match(invalid.content[0].text, /at \/type/);

  const validation = await client.callTool({
    name: "chartlet_validate_spec",
    arguments: { spec: { ...spec, type: "pie" } },
  });
  assert.equal(validation.structuredContent.ok, false);
  assert.equal(validation.structuredContent.error.path, "/type");

  const inspected = await client.callTool({
    name: "chartlet_inspect_data",
    arguments: { csv: "quarter,revenue\nQ1,12\nQ2,15\n" },
  });
  assert.equal(inspected.structuredContent.suggestions[0].type, "bar");
  const badCsv = await client.callTool({ name: "chartlet_inspect_data", arguments: { csv: 'a\n"open' } });
  assert.equal(badCsv.isError, true);

  const explained = await client.callTool({ name: "chartlet_explain", arguments: { spec } });
  assert.deepEqual(explained.structuredContent.series[0].max, { value: 15, at: ["Q2"] });

  const { contents } = await client.readResource({ uri: "chartlet://schema" });
  assert.equal(contents[0].text, schema);
});

test("refuses outputPath when CHARTLET_MCP_ROOT is not set", async (t) => {
  const client = await connect({});
  t.after(() => client.close());
  const refused = await client.callTool({
    name: "chartlet_render",
    arguments: { spec, outputPath: "chart.html" },
  });
  assert.equal(refused.isError, true);
  assert.match(refused.content[0].text, /outputPath is disabled.*CHARTLET_MCP_ROOT/);
  const inline = await client.callTool({ name: "chartlet_render", arguments: { spec } });
  assert.match(inline.structuredContent.content, /^<figure/);
});

// Row objects as opengrid hands them over, from `exportRows(…, { format: "json" })` or the
// opengrid-mcp selection: keys are the field names in column order, values in opengrid's wire
// notation (decimals and timestamps as strings, null for missing). The rows follow the orders of
// the web-opengrid demo data set.
const orders = [
  { id: 1, order_date: "2026-04-12", customer: "Yarrow Works", country: "CH", category: "Services", qty: 2, unit_price: "250.00", discount: 0, amount: "500.00", express: false, shipped_at: "2026-04-17T12:00:00.000000Z", note: null },
  { id: 2, order_date: "2026-04-29", customer: "Alder Systems", country: "CH", category: "Services", qty: 1, unit_price: "890.00", discount: 0.1, amount: "801.00", express: true, shipped_at: "2026-05-01T16:00:00.000000Z", note: null },
  { id: 3, order_date: "2026-02-01", customer: "Pinecrest Systems", country: "GB", category: "Software", qty: 2, unit_price: "119.00", discount: 0, amount: "238.00", express: false, shipped_at: null, note: "Gift" },
];

test("chartlet_inspect_data takes opengrid row objects unchanged", async (t) => {
  const client = await connect({});
  t.after(() => client.close());
  const result = await client.callTool({ name: "chartlet_inspect_data", arguments: { rows: orders } });
  assert.equal(result.isError, undefined);
  const summary = result.structuredContent;
  assert.equal(summary.rowCount, 3);
  assert.deepEqual(summary.columns.map(({ name }) => name), Object.keys(orders[0]));
  const byName = Object.fromEntries(summary.columns.map((column) => [column.name, column]));
  assert.equal(byName.id.type, "integer");
  assert.deepEqual([byName.amount.type, byName.amount.min, byName.amount.max], ["number", 238, 801]);
  assert.equal(byName.unit_price.type, "number");
  assert.equal(byName.discount.type, "number");
  assert.equal(byName.express.type, "boolean");
  assert.deepEqual(
    [byName.order_date.type, byName.order_date.timeFormat, byName.order_date.monotonic],
    ["date-time", "date", "none"],
  );
  assert.deepEqual(
    [byName.shipped_at.type, byName.shipped_at.timeFormat, byName.shipped_at.missing],
    ["date-time", "date-time", 1],
  );
  assert.deepEqual([byName.note.type, byName.note.missing], ["string", 2]);
  assert.ok(summary.suggestions.some(({ type, columns }) => type === "time" && columns.time === "order_date"));

  // opengrid writes non-finite floats as strings; they are not read as numbers.
  const ratio = await client.callTool({
    name: "chartlet_inspect_data",
    arguments: { rows: [{ ratio: 1.5 }, { ratio: "Infinity" }, { ratio: null }] },
  });
  assert.equal(ratio.structuredContent.columns[0].type, "string");
});
