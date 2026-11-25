import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { fromHtml } from "hast-util-from-html";
import { toHtml } from "hast-util-to-html";
import MarkdownIt from "markdown-it";
import { toHast } from "mdast-util-to-hast";

import { htmlToHast } from "../src/markdown.mjs";
import markdownItChartlet from "../src/markdown-it.mjs";
import { renderChart } from "../src/render.mjs";
import remarkChartlet from "../src/remark.mjs";

const examples = resolve(dirname(fileURLToPath(import.meta.url)), "../../../examples");
const example = (name) => readFileSync(join(examples, `${name}.json`), "utf8");

// A vfile as remark passes it: `fail` throws, `message` collects.
function file() {
  const messages = [];
  return {
    messages,
    message(reason, node) {
      messages.push({ reason, line: node.position?.start.line });
    },
    fail(reason, node) {
      const error = new Error(reason);
      error.line = node.position?.start.line;
      throw error;
    },
  };
}

function code(value, meta = null, lang = "chartlet") {
  return { type: "code", lang, meta, value, position: { start: { line: 3 }, end: { line: 9 } } };
}

function remark(tree, options) {
  const vfile = file();
  remarkChartlet(options)(tree, vfile);
  return { html: toHtml(toHast(tree)), messages: vfile.messages };
}

test("reads every example's HTML as a parser of HTML does", () => {
  for (const name of readdirSync(examples).filter((entry) => entry.endsWith(".json"))) {
    const spec = JSON.parse(readFileSync(join(examples, name), "utf8"));
    for (const options of [{}, { hooks: true }]) {
      const { content } = renderChart(spec, { format: "html", ...options });
      assert.equal(toHtml(htmlToHast(content)), toHtml(fromHtml(content, { fragment: true })), name);
    }
  }
});

test("remark: a chartlet block becomes the chart; other blocks stay", () => {
  const tree = {
    type: "root",
    children: [
      code(example("release-flow"), "id=flow"),
      { type: "blockquote", children: [code(example("monthly-revenue"))] },
      code("let a = 1;", null, "js"),
    ],
  };
  const { html } = remark(tree);
  const expected = renderChart(JSON.parse(example("release-flow")), { format: "html", idPrefix: "flow" });
  assert.ok(html.startsWith(toHtml(fromHtml(expected.content, { fragment: true }))));
  assert.match(html, /<blockquote>\s*<figure/);
  assert.match(html, /<code class="language-js">let a = 1;/);
});

test("remark: react turns checked inputs into defaultChecked", () => {
  const tree = { type: "root", children: [code(example("release-flow"))] };
  remarkChartlet({ react: true })(tree, file());
  const inputs = [];
  const walk = (node) => {
    if (node.tagName === "input") inputs.push(node.properties);
    (node.children ?? node.data?.hChildren ?? []).forEach(walk);
  };
  walk(tree.children[0]);
  assert.ok(inputs.some((properties) => properties.defaultChecked === true));
  assert.ok(inputs.every((properties) => !("checked" in properties)));
});

test("remark: an invalid block fails the file at the block; warnings become messages", () => {
  assert.throws(
    () => remark({ type: "root", children: [code('{"schemaVersion":1,"type":"bar"}')] }),
    (error) => error.message.startsWith("chartlet: invalid_spec at /") && error.line === 3,
  );
  assert.throws(
    () => remark({ type: "root", children: [code(example("monthly-revenue"), "idd=x")] }),
    /unknown fence setting "idd"; the settings are id, view, perspective/,
  );
  const long = JSON.stringify({
    schemaVersion: 1,
    type: "bar",
    title: "Labels",
    width: 200,
    data: [
      { label: "A label much too long for a chart only two hundred pixels wide", value: 1 },
      { label: "B", value: 2 },
    ],
  });
  const { messages } = remark({ type: "root", children: [code(long)] });
  assert.deepEqual(messages.map(({ reason, line }) => [reason.split(" at ")[0], line]), [["warning[text_truncated]", 3]]);
});

test("markdown-it: renders the chart's exact HTML and keeps other fences", () => {
  const md = new MarkdownIt().use(markdownItChartlet);
  const output = md.render(`\`\`\`chartlet id=flow\n${example("release-flow")}\n\`\`\`\n\n\`\`\`js\nlet a = 1;\n\`\`\`\n`);
  const expected = renderChart(JSON.parse(example("release-flow")), { format: "html", idPrefix: "flow" });
  assert.ok(output.startsWith(`${expected.content}\n`));
  assert.match(output, /<code class="language-js">let a = 1;/);
});

test("markdown-it: vue passes the chart as v-html", () => {
  const md = new MarkdownIt().use(markdownItChartlet, { vue: true });
  const output = md.render(`\`\`\`chartlet\n${example("monthly-revenue")}\n\`\`\`\n`);
  const [, literal] = output.match(/^<div class="chartlet-markdown" v-html="([^"]*)"><\/div>\n$/);
  const html = JSON.parse(literal.replaceAll("&quot;", '"').replaceAll("&amp;", "&"));
  assert.equal(html, renderChart(JSON.parse(example("monthly-revenue")), { format: "html" }).content);
});

test("markdown-it: errors name the line; warnings go to onWarning", () => {
  const md = new MarkdownIt().use(markdownItChartlet);
  assert.throws(
    () => md.render('Text\n\n```chartlet\n{"schemaVersion":1}\n```\n'),
    /^Error: chartlet: invalid_spec at \/: .* \(chartlet block at line 3\)$/,
  );
  const warnings = [];
  const quiet = new MarkdownIt().use(markdownItChartlet, { onWarning: (warning) => warnings.push(warning) });
  quiet.render(
    `\`\`\`chartlet\n${JSON.stringify({
      schemaVersion: 1,
      type: "bar",
      title: "Labels",
      width: 200,
      data: [
        { label: "A label much too long for a chart only two hundred pixels wide", value: 1 },
        { label: "B", value: 2 },
      ],
    })}\n\`\`\`\n`,
  );
  assert.deepEqual(warnings.map(({ code, line }) => [code, line]), [["text_truncated", 1]]);
});
