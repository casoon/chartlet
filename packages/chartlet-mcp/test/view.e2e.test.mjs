// The MCP App view in a browser: a minimal host page embeds the view resource in a sandboxed
// iframe under the restrictive default CSP of MCP Apps, speaks the host side of the protocol and
// hands it a real chartlet_render result. axe checks the view in both themes.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import test from "node:test";

import axe from "axe-core";
import { Client } from "@modelcontextprotocol/client";
import { StdioClientTransport } from "@modelcontextprotocol/client/stdio";
import { EXTENSION_ID, RESOURCE_MIME_TYPE } from "@modelcontextprotocol/ext-apps/server";
import { chromium } from "playwright";

const server = fileURLToPath(new URL("../src/server.mjs", import.meta.url));

const mobileSpec = JSON.parse(readFileSync(new URL("../../../examples/mobile-revenue.json", import.meta.url), "utf8"));

const spec = {
  schemaVersion: 1,
  type: "bar",
  title: "Open tickets by team",
  source: "Illustrative values",
  data: [
    { label: "Platform", value: 12 },
    { label: "Payments", value: 7 },
    { label: "Search", value: 15 },
  ],
};

// What the MCP Apps specification lets a host enforce by default for a view that declares no
// domains: inline script and style, nothing from the network.
const CSP =
  "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src data:; font-src data:; connect-src 'none'";

const HOST = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Test host</title></head>
<body>
<iframe id="view" title="chartlet figure" sandbox="allow-scripts" style="width:640px;height:600px;border:0"></iframe>
<script>
  window.host = { sizes: [], downloads: [], requests: [] };
  const frame = document.getElementById("view");
  const post = (message) => frame.contentWindow.postMessage({ jsonrpc: "2.0", ...message }, "*");
  window.addEventListener("message", (event) => {
    if (event.source !== frame.contentWindow) return;
    const message = event.data;
    host.requests.push(message.method);
    if (message.method === "ui/initialize") {
      post({ id: message.id, result: {
        protocolVersion: "2026-01-26",
        hostInfo: { name: "test-host", version: "0.0.0" },
        hostCapabilities: { downloadFile: {} },
        hostContext: { theme: host.theme },
      } });
    } else if (message.method === "ui/notifications/initialized") {
      post({ method: "ui/notifications/tool-result", params: host.result });
    } else if (message.method === "ui/notifications/size-changed") {
      host.sizes.push(message.params);
    } else if (message.method === "ui/download-file") {
      host.downloads.push(message.params);
      post({ id: message.id, result: {} });
    }
  });
  window.setHostTheme = (theme) => post({ method: "ui/notifications/host-context-changed", params: { theme } });
  window.openView = (html, result, theme, width) => {
    frame.style.width = width + "px";
    host.result = result;
    host.theme = theme;
    frame.srcdoc = html;
  };
</script>
</body></html>`;

/** Starts the server and returns the view HTML and render results as an MCP Apps client sees them. */
async function fromServer() {
  const env = { ...process.env };
  delete env.CHARTLET_BIN;
  delete env.CHARTLET_MCP_ROOT;
  const client = new Client(
    { name: "chartlet-mcp-e2e", version: "0.0.0" },
    { capabilities: { extensions: { [EXTENSION_ID]: { mimeTypes: [RESOURCE_MIME_TYPE] } } } },
  );
  await client.connect(
    new StdioClientTransport({ command: process.execPath, args: [server], cwd: tmpdir(), env }),
  );
  try {
    const { contents } = await client.readResource({ uri: "ui://chartlet/figure" });
    const result = await client.callTool({ name: "chartlet_render", arguments: { spec } });
    const failed = await client.callTool({
      name: "chartlet_render",
      arguments: { spec: { ...spec, type: "pie" } },
    });
    const html = contents[0].text.replace(
      "<head>",
      `<head><meta http-equiv="Content-Security-Policy" content="${CSP}">`,
    );
    const mobile = await client.callTool({ name: "chartlet_render", arguments: { spec: mobileSpec } });
    return { html, result, failed, mobile };
  } finally {
    await client.close();
  }
}

/**
 * @param {import("playwright").Page} page
 * @param {{ html: string, result: unknown, theme?: string }} options
 */
async function openView(page, { html, result, theme, width = 640 }) {
  await page.setContent(HOST);
  await page.evaluate(
    ([html, result, theme, width]) => window.openView(html, result, theme, width),
    [html, result, theme, width],
  );
  const frame = page.frameLocator("#view");
  await frame.locator("#figure:not([aria-busy])").waitFor();
  return frame;
}

/**
 * @param {import("playwright").Page} page
 */
async function axeViolations(page) {
  const frame = page.frames().find((candidate) => candidate !== page.mainFrame());
  await frame.evaluate(axe.source);
  const { violations } = await frame.evaluate(() =>
    window.axe.run(document, { runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21aa", "wcag22aa"] } }),
  );
  return violations.map(({ id, nodes }) => `${id}: ${nodes.map(({ target }) => target.join(" ")).join(", ")}`);
}

test("the view shows the rendered figure, follows the host theme and stays accessible", async (t) => {
  const { html, result, failed, mobile } = await fromServer();
  const browser = await chromium.launch();
  t.after(() => browser.close());
  const page = await browser.newPage();
  const problems = [];
  page.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning") problems.push(message.text());
  });
  page.on("pageerror", (error) => problems.push(error.message));

  const frame = await openView(page, { html, result, theme: "dark" });
  const view = result._meta["chartlet/view"];

  await t.test("shows the compiler's figure for the host theme", async () => {
    assert.equal(await frame.locator("figcaption").textContent(), "Open tickets by team");
    assert.equal(await frame.locator("html").getAttribute("data-theme"), "dark");
    assert.match(await frame.locator("svg").getAttribute("class"), /chartlet-theme-dark/);
    assert.equal(await frame.locator("details.chartlet-data table tbody tr").count(), 3);
    assert.equal(await frame.locator(".chartlet-source").textContent(), "Source: Illustrative values");
    // The shared stylesheet applies: the bars take the chart's accent color.
    const fill = await frame.locator(".chartlet-bar").first().evaluate((bar) => getComputedStyle(bar).fill);
    assert.notEqual(fill, "rgb(0, 0, 0)");
    assert.equal(await frame.locator("#figure > figure").count(), 1);
  });

  await t.test("passes axe in the dark theme", async () => {
    assert.deepEqual(await axeViolations(page), []);
  });

  await t.test("reports its size", async () => {
    await page.waitForFunction(() => window.host.sizes.length > 0);
    const sizes = await page.evaluate(() => window.host.sizes);
    assert.ok(sizes.at(-1).height > 300, JSON.stringify(sizes));
  });

  await t.test("offers keyboard-reachable actions", async () => {
    await frame.locator("summary").focus();
    await page.keyboard.press("Tab");
    assert.equal(await frame.locator(":focus").textContent(), "Download SVG");
    await page.keyboard.press("Enter");
    await page.keyboard.press("Tab");
    assert.equal(await frame.locator(":focus").textContent(), "Download HTML");
    await page.keyboard.press("Enter");
    await page.keyboard.press("Tab");
    assert.equal(await frame.locator(":focus").textContent(), "Copy specification");
    await page.keyboard.press("Enter");
    await page.waitForFunction(() => window.host.downloads.length === 2);
    const downloads = await page.evaluate(() => window.host.downloads);
    assert.deepEqual(
      downloads.map(({ contents: [{ resource }] }) => [resource.uri, resource.mimeType, resource.text]),
      [
        ["file:///open-tickets-by-team.svg", "image/svg+xml", view.files.svg.text],
        ["file:///open-tickets-by-team.html", "text/html", view.files.html.text],
      ],
    );
    await frame.locator("#status:not(:empty)").waitFor();
    // Nothing the view sends asks the host to change data or call tools.
    const requests = await page.evaluate(() => window.host.requests);
    assert.deepEqual(
      [...new Set(requests)].sort(),
      ["ui/download-file", "ui/initialize", "ui/notifications/initialized", "ui/notifications/size-changed"],
    );
  });

  await t.test("switches to the light figure when the host theme changes", async () => {
    await page.evaluate(() => window.setHostTheme("light"));
    await frame.locator("html[data-theme='light']").waitFor();
    assert.doesNotMatch(await frame.locator("svg").getAttribute("class"), /chartlet-theme-dark/);
    assert.deepEqual(await axeViolations(page), []);
  });

  await t.test("shows a tool error as text", async () => {
    const errorPage = await browser.newPage();
    const errorFrame = await openView(errorPage, { html, result: failed, theme: "light" });
    assert.match(await errorFrame.locator("#figure .error").textContent(), /invalid_spec at \/type/);
    assert.equal(await errorFrame.locator("#actions").isHidden(), true);
    assert.deepEqual(await axeViolations(errorPage), []);
  });

  await t.test("lays out the mobile variant in a narrow view", async () => {
    for (const [width, shown, hidden] of [
      [360, ".chartlet-variant-mobile", ".chartlet-variant-desktop"],
      [800, ".chartlet-variant-desktop", ".chartlet-variant-mobile"],
    ]) {
      const narrow = await openView(await browser.newPage(), { html, result: mobile, theme: "light", width });
      assert.equal(await narrow.locator(shown).isVisible(), true, `${width}: ${shown}`);
      assert.equal(await narrow.locator(hidden).isVisible(), false, `${width}: ${hidden}`);
    }
  });

  await t.test("runs under the default CSP without errors", () => {
    assert.deepEqual(problems, []);
  });
});
