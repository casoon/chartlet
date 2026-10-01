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

test("serves the tools and the schema over stdio", async (t) => {
  const root = mkdtempSync(join(tmpdir(), "chartlet-mcp-e2e-"));
  const env = { ...process.env };
  delete env.CHARTLET_BIN;
  const client = new Client({ name: "chartlet-mcp-test", version: "0.0.0" });
  await client.connect(
    new StdioClientTransport({ command: process.execPath, args: [server], cwd: root, env }),
  );
  t.after(async () => {
    await client.close();
    rmSync(root, { recursive: true, force: true });
  });

  const { tools } = await client.listTools();
  assert.deepEqual(tools.map(({ name }) => name).sort(), [
    "explain_chart",
    "inspect_data",
    "render_chart",
    "validate_spec",
  ]);
  for (const tool of tools) {
    assert.ok(tool.description.length > 100, tool.name);
    assert.equal(tool.inputSchema.type, "object");
    assert.ok(tool.outputSchema, tool.name);
  }
  const annotations = Object.fromEntries(tools.map(({ name, annotations }) => [name, annotations]));
  assert.equal(annotations.inspect_data.readOnlyHint, true);
  assert.equal(annotations.render_chart.readOnlyHint, false);

  const rendered = await client.callTool({ name: "render_chart", arguments: { spec, format: "svg" } });
  assert.equal(rendered.isError, undefined);
  assert.match(rendered.structuredContent.content, /^<svg/);
  assert.equal(rendered.structuredContent.manifest.format, "svg");
  assert.deepEqual(JSON.parse(rendered.content[0].text), rendered.structuredContent);

  const written = await client.callTool({
    name: "render_chart",
    arguments: { spec: JSON.stringify(spec), outputPath: "charts/revenue.html" },
  });
  assert.equal(written.structuredContent.path, join("charts", "revenue.html"));
  assert.equal(
    written.structuredContent.bytes,
    readFileSync(join(root, "charts/revenue.html")).length,
  );

  const escape = await client.callTool({
    name: "render_chart",
    arguments: { spec, outputPath: "../escape.html" },
  });
  assert.equal(escape.isError, true);
  assert.match(escape.content[0].text, /must not contain/);

  const invalid = await client.callTool({
    name: "render_chart",
    arguments: { spec: { ...spec, type: "pie" } },
  });
  assert.equal(invalid.isError, true);
  assert.match(invalid.content[0].text, /at \/type/);

  const validation = await client.callTool({
    name: "validate_spec",
    arguments: { spec: { ...spec, type: "pie" } },
  });
  assert.equal(validation.structuredContent.ok, false);
  assert.equal(validation.structuredContent.error.path, "/type");

  const inspected = await client.callTool({
    name: "inspect_data",
    arguments: { csv: "quarter,revenue\nQ1,12\nQ2,15\n" },
  });
  assert.equal(inspected.structuredContent.suggestions[0].type, "bar");
  const badCsv = await client.callTool({ name: "inspect_data", arguments: { csv: 'a\n"open' } });
  assert.equal(badCsv.isError, true);

  const explained = await client.callTool({ name: "explain_chart", arguments: { spec } });
  assert.deepEqual(explained.structuredContent.series[0].max, { value: 15, at: ["Q2"] });

  const { contents } = await client.readResource({ uri: "chartlet://schema" });
  assert.equal(contents[0].text, schema);
});
