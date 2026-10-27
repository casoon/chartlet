import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";

import { readSchemaTypes, SHOWING, typeGuide } from "../src/guide.mjs";

const server = fileURLToPath(new URL("../src/server.mjs", import.meta.url));
const schema = JSON.parse(readFileSync(new URL("../../../schema/chartlet.schema.json", import.meta.url), "utf8"));
const types = /** @type {string[]} */ (schema.properties.type.enum);

test("the guide covers exactly the types of the schema", () => {
  assert.deepEqual(readSchemaTypes().types, types);
  assert.deepEqual(Object.keys(SHOWING).sort(), [...types].sort(), "SHOWING and the schema's types differ");
  const guide = typeGuide();
  for (const type of types) {
    assert.match(guide, new RegExp(`\\| \`${type}\` \\|`), `type ${type} missing from the guide`);
  }
  assert.doesNotMatch(guide, /not described/);
});

test("the guide says when not to draw, gives budgets, sizes and common mistakes", () => {
  const guide = typeGuide();
  for (const heading of ["## When not to draw", "## Readability budgets", "## Size", "## Common mistakes"]) {
    assert.ok(guide.includes(heading), heading);
  }
  assert.match(guide, /`slide-16x9`/);
  assert.match(guide, /`size_exceeded`/);
  assert.match(guide, /`diagram_over_budget`/);
});

test("the guide names the block of every type that has one", () => {
  const guide = typeGuide();
  const { blocks } = readSchemaTypes();
  assert.ok(Object.keys(blocks).length > 10);
  for (const [type, block] of Object.entries(blocks)) {
    assert.match(guide, new RegExp(`\\| \`${type}\` \\| \`${block}\`:`), `${type} -> ${block}`);
  }
});

test("lists prompts and the guide, and a prompt leads through validate and render", async (t) => {
  const env = { ...process.env };
  delete env.CHARTLET_BIN;
  const client = new Client({ name: "chartlet-mcp-test", version: "0.0.0" });
  await client.connect(new StdioClientTransport({ command: process.execPath, args: [server], cwd: tmpdir(), env }));
  t.after(() => client.close());

  const { prompts } = await client.listPrompts();
  assert.deepEqual(prompts.map(({ name }) => name).sort(), ["chart", "diagram", "redraw", "revise"]);

  const { resources } = await client.listResources();
  assert.ok(resources.some(({ uri }) => uri === "chartlet://guide/types"));

  const { tools } = await client.listTools();
  const toolNames = new Set(tools.map(({ name }) => name));
  for (const name of ["diagram", "chart", "redraw", "revise"]) {
    const { messages } = await client.getPrompt({ name, arguments: { request: "checkout flow" } });
    const text = messages[0].content.text;
    assert.match(text, /checkout flow/);
    assert.match(text, /chartlet:\/\/guide\/types/);
    assert.ok(text.indexOf("chartlet_validate_spec") < text.indexOf("chartlet_render"), name);
    for (const tool of text.match(/chartlet_[a-z_]+/g)) {
      assert.ok(toolNames.has(tool), `${name} names unknown tool ${tool}`);
    }
    assert.ok(text.length < 3000, `${name}: ${text.length} characters`);
  }
  const chart = await client.getPrompt({ name: "chart", arguments: { request: "revenue", data: "q,v\nQ1,1" } });
  assert.match(chart.messages[0].content.text, /Q1,1/);
  assert.match(chart.messages[0].content.text, /chartlet_inspect_data/);

  const { contents } = await client.readResource({ uri: "chartlet://guide/types" });
  assert.equal(contents[0].text, typeGuide());
  assert.ok(contents[0].text.length < 12000, `guide: ${contents[0].text.length} characters`);
});
