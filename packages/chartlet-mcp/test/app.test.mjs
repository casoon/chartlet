import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { renderChart, stylesheet } from "@casoon/chartlet";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { EXTENSION_ID, RESOURCE_MIME_TYPE, RESOURCE_URI_META_KEY } from "@modelcontextprotocol/ext-apps/server";

const server = fileURLToPath(new URL("../src/server.mjs", import.meta.url));
const { version } = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));

const spec = {
  schemaVersion: 1,
  type: "bar",
  title: "Revenue by quarter",
  data: [
    { label: "Q1", value: 12 },
    { label: "Q2", value: 15 },
  ],
};

/**
 * @param {boolean} apps whether the client declares the MCP Apps extension
 */
async function connect(apps) {
  const env = { ...process.env };
  delete env.CHARTLET_BIN;
  delete env.CHARTLET_MCP_ROOT;
  const client = new Client(
    { name: "chartlet-mcp-test", version: "0.0.0" },
    apps ? { capabilities: { extensions: { [EXTENSION_ID]: { mimeTypes: [RESOURCE_MIME_TYPE] } } } } : {},
  );
  await client.connect(
    new StdioClientTransport({ command: process.execPath, args: [server], cwd: tmpdir(), env }),
  );
  return client;
}

test("chartlet_render points to the view resource", async (t) => {
  const client = await connect(false);
  t.after(() => client.close());
  const { tools } = await client.listTools();
  const render = tools.find(({ name }) => name === "chartlet_render");
  assert.equal(render._meta.ui.resourceUri, "ui://chartlet/figure");
  assert.equal(render._meta[RESOURCE_URI_META_KEY], "ui://chartlet/figure");
  for (const tool of tools.filter(({ name }) => name !== "chartlet_render")) {
    assert.equal(tool._meta?.ui, undefined, tool.name);
  }
});

test("serves the view as an MCP App resource without network access", async (t) => {
  const client = await connect(true);
  t.after(() => client.close());
  const { resources } = await client.listResources();
  const listed = resources.find(({ uri }) => uri === "ui://chartlet/figure");
  assert.equal(listed.mimeType, "text/html;profile=mcp-app");

  const { contents } = await client.readResource({ uri: "ui://chartlet/figure" });
  assert.equal(contents.length, 1);
  const [view] = contents;
  assert.equal(view.mimeType, "text/html;profile=mcp-app");
  assert.deepEqual(view._meta.ui.csp, { connectDomains: [], resourceDomains: [] });
  assert.deepEqual(view._meta.ui.permissions, { clipboardWrite: {} });
  assert.equal(view._meta.ui.prefersBorder, true);

  // The shared stylesheet once, one inline script (the bridge), nothing loaded from elsewhere.
  assert.equal(view.text.split(stylesheet()).length, 2);
  assert.equal(view.text.match(/<script\b/g).length, 1);
  assert.doesNotMatch(view.text, /<script[^>]*\ssrc=|<link\b|@import|https?:\/\//);
  assert.match(view.text, new RegExp(`version: "${version.replace(/\./g, "\\.")}"`));
  assert.doesNotMatch(view.text, /\/\*chartlet-/);
});

test("adds the view payload only for clients that show MCP Apps", async (t) => {
  const plain = await connect(false);
  const apps = await connect(true);
  t.after(async () => {
    await plain.close();
    await apps.close();
  });

  const text = await plain.callTool({ name: "chartlet_render", arguments: { spec, format: "svg" } });
  assert.equal(text._meta?.["chartlet/view"], undefined);

  const shown = await apps.callTool({ name: "chartlet_render", arguments: { spec, format: "svg" } });
  // The model-facing result is the same as without the view.
  assert.deepEqual(shown.content, text.content);
  assert.deepEqual(shown.structuredContent, text.structuredContent);
  const view = shown._meta["chartlet/view"];
  const idPrefix = shown.structuredContent.manifest.idPrefix;
  assert.equal(view.title, "Revenue by quarter");
  assert.deepEqual(Object.keys(view.figures), ["light", "dark"]);
  assert.equal(
    view.figures.light,
    renderChart(spec, { format: "html", styles: "external", idPrefix }).content,
  );
  assert.equal(
    view.figures.dark,
    renderChart({ ...spec, theme: "dark" }, { format: "html", styles: "external", idPrefix }).content,
  );
  assert.doesNotMatch(view.figures.light, /<style/);
  assert.deepEqual(view.files.svg, {
    name: "revenue-by-quarter.svg",
    mimeType: "image/svg+xml",
    text: shown.structuredContent.content,
  });
  assert.equal(view.files.html.name, "revenue-by-quarter.html");
  assert.equal(view.files.html.text, renderChart(spec, { format: "html", idPrefix }).content);
  assert.deepEqual(JSON.parse(view.spec), spec);
});

test("keeps the theme a specification sets", async (t) => {
  const client = await connect(true);
  t.after(() => client.close());
  const result = await client.callTool({
    name: "chartlet_render",
    arguments: { spec: JSON.stringify({ ...spec, theme: "dark" }) },
  });
  const view = result._meta["chartlet/view"];
  assert.deepEqual(Object.keys(view.figures), ["dark"]);
  assert.equal(view.files.html.text, result.structuredContent.content);

  const invalid = await client.callTool({ name: "chartlet_render", arguments: { spec: { ...spec, type: "pie" } } });
  assert.equal(invalid.isError, true);
  assert.equal(invalid._meta?.["chartlet/view"], undefined);
});
