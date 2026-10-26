#!/usr/bin/env node
// Install smoke test: packs the npm packages, installs the tarballs into an empty temporary
// project outside the repository and uses them the way a user would, without the CLI or a Rust
// toolchain on the path of the checks. Run it before a release: `node scripts/smoke-install.mjs`.
// `npm pack` runs each package's `prepack`, so the WebAssembly build and the schema copy happen
// here and need the toolchain of a checkout; the installed project does not.
import { spawn, execFileSync } from "node:child_process";
import { copyFileSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packages = ["chartlet", "chartlet-mcp"];
const work = mkdtempSync(join(tmpdir(), "chartlet-smoke-"));
const project = join(work, "project");
mkdirSync(project);

const run = (command, args, cwd) =>
  execFileSync(command, args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] });
let failed = false;
const check = (name, ok, detail = "") => {
  console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? `: ${detail}` : ""}`);
  failed ||= !ok;
};

try {
  const tarballs = packages.map((name) => {
    const directory = join(repository, "packages", name);
    // The prepack output shares stdout, so the file name is the last line.
    const filename = run("npm", ["pack", "--pack-destination", work], directory).trim().split("\n").pop();
    console.log(`packed ${filename}`);
    return join(work, filename);
  });

  writeFileSync(
    join(project, "package.json"),
    '{ "name": "chartlet-smoke", "private": true, "type": "module" }\n',
  );
  run("npm", ["install", "--no-audit", "--no-fund", ...tarballs], project);

  const versions = packages.map(
    (name) =>
      JSON.parse(readFileSync(join(repository, "packages", name, "package.json"), "utf8")).version,
  );
  const cargo = readFileSync(join(repository, "Cargo.toml"), "utf8").match(/^version = "(.+)"/m)[1];
  check(
    "package and crate versions agree",
    new Set([...versions, cargo]).size === 1,
    `${versions.join(", ")}, crate ${cargo}`,
  );

  // The checks run in the installed project, so every import resolves from node_modules.
  const probe = `
    import { readFileSync, existsSync } from "node:fs";
    import { fileURLToPath } from "node:url";
    import { renderChartDetailed } from "@casoon/chartlet";
    const out = {};
    const bar = { schemaVersion: 1, type: "bar", title: "Revenue", data: [{ label: "Q1", value: 12 }, { label: "Q2", value: 15 }] };
    const flow = JSON.parse(readFileSync("release-flow.json", "utf8"));
    const render = (spec, options) => renderChartDetailed(spec, { format: "svg", ...options }).content;
    const svg = render(bar);
    out.chart = svg.startsWith("<svg") && svg.includes("Revenue");
    out.chartRepeatable = render(bar) === svg;
    const diagram = renderChartDetailed(flow, { format: "svg" });
    out.diagram = diagram.ok === true && diagram.content.startsWith("<svg");
    out.diagramGolden = diagram.ok && diagram.content.trimEnd() === readFileSync("release-flow.svg", "utf8").trimEnd();
    out.diagramError = diagram.ok ? "" : JSON.stringify(diagram.error);
    out.html = render(bar, { format: "html" }).includes("<figure");
    const asset = (specifier) => fileURLToPath(import.meta.resolve(specifier));
    const wasm = asset("@casoon/chartlet/chartlet.wasm");
    const css = asset("@casoon/chartlet/chartlet.css");
    const cssText = readFileSync(css, "utf8");
    out.wasm = { path: wasm, magic: readFileSync(wasm).subarray(0, 4).toString("hex") };
    out.css = { path: css, size: cssText.length, hasToken: cssText.includes("--chartlet-") };
    const mcp = asset("@casoon/chartlet-mcp/package.json").replace(/package\\.json$/, "");
    const schema = mcp + "schema/chartlet.schema.json";
    out.schema = { path: schema, ok: existsSync(schema) && JSON.parse(readFileSync(schema, "utf8")).$schema !== undefined };
    out.mcpRoot = mcp;
    console.log(JSON.stringify(out));
  `;
  // A committed example and its golden SVG: the installed WASM must reproduce the checked-in bytes.
  for (const file of ["release-flow.json", "release-flow.svg"]) {
    copyFileSync(join(repository, "examples", file), join(project, file));
  }
  writeFileSync(join(project, "probe.mjs"), probe);
  const out = JSON.parse(run("node", ["probe.mjs"], project));
  check("chart renders through WASM", out.chart && out.html);
  check("render is repeatable", out.chartRepeatable);
  check("diagram renders through WASM", out.diagram, out.diagramError);
  check("diagram matches the committed golden SVG", out.diagramGolden);
  const outside = (path) => !path.startsWith(repository) && path.includes("node_modules");
  check(
    "chartlet.wasm resolves from node_modules",
    outside(out.wasm.path) && out.wasm.magic === "0061736d",
    out.wasm.path,
  );
  check(
    "chartlet.css resolves from node_modules",
    outside(out.css.path) && out.css.hasToken,
    `${out.css.size} chars`,
  );
  check("schema ships in chartlet-mcp", outside(out.schema.path) && out.schema.ok, out.schema.path);

  // The MCP server over stdio, from the installed package: initialize, list tools and resources.
  const server = spawn(process.execPath, [join(out.mcpRoot, "src/server.mjs")], {
    cwd: project,
    env: { ...process.env, CHARTLET_BIN: "", CHARTLET_MCP_ROOT: "" },
    stdio: ["pipe", "pipe", "inherit"],
  });
  const pending = new Map();
  let buffer = "";
  server.stdout.on("data", (chunk) => {
    buffer += chunk;
    for (let newline; (newline = buffer.indexOf("\n")) >= 0; ) {
      const line = buffer.slice(0, newline).trim();
      buffer = buffer.slice(newline + 1);
      if (line) {
        const message = JSON.parse(line);
        pending.get(message.id)?.(message);
      }
    }
  });
  let id = 0;
  const send = (method, params = {}) =>
    new Promise((resolveMessage, reject) => {
      const timer = setTimeout(() => reject(new Error(`${method}: no answer in 30 s`)), 30_000);
      pending.set(++id, (message) => {
        clearTimeout(timer);
        resolveMessage(message);
      });
      server.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
    });
  try {
    await send("initialize", {
      protocolVersion: "2025-06-18",
      capabilities: {},
      clientInfo: { name: "smoke", version: "0" },
    });
    server.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);
    const tools = (await send("tools/list")).result.tools.map(({ name }) => name);
    const expected = [
      "chartlet_render",
      "chartlet_validate_spec",
      "chartlet_inspect_data",
      "chartlet_explain",
      "chartlet_inspect",
      "chartlet_diagram_starter",
      "chartlet_chart_starter",
    ];
    check("MCP lists its tools", expected.every((name) => tools.includes(name)), tools.join(", "));
    const uris = (await send("resources/list")).result.resources.map(({ uri }) => uri);
    check(
      "MCP lists the schema and the view",
      uris.includes("chartlet://schema") && uris.includes("ui://chartlet/figure"),
      uris.join(", "),
    );
    const schema = (await send("resources/read", { uri: "chartlet://schema" })).result.contents[0].text;
    check("MCP serves the bundled schema", JSON.parse(schema).$schema !== undefined);
    const view = (await send("resources/read", { uri: "ui://chartlet/figure" })).result.contents[0].text;
    check("MCP serves the view", /<html|<!doctype/i.test(view), `${view.length} chars`);
  } finally {
    server.kill();
  }
} catch (error) {
  check("smoke run", false, error.message);
} finally {
  rmSync(work, { recursive: true, force: true });
}
console.log(failed ? "smoke FAILED" : "smoke passed");
process.exit(failed ? 1 : 0);
