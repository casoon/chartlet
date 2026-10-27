#!/usr/bin/env node
// Install smoke test: packs the npm packages, installs the tarballs into an empty temporary
// project outside the repository and uses them the way a user would, without the CLI or a Rust
// toolchain on the path of the checks. Run it before a release: `node scripts/smoke-install.mjs`.
// `npm pack` runs each package's `prepack`, so the WebAssembly build and the schema copy happen
// here and need the toolchain of a checkout; the installed project does not.
//
//   node scripts/smoke-install.mjs                 packed tarballs of this checkout
//   node scripts/smoke-install.mjs --published     the versions on the registry that this
//                                                  checkout declares (run it after publishing)
//   node scripts/smoke-install.mjs --cli           additionally `cargo install --locked --path .`
//                                                  into a temporary root and render with that CLI
import { spawn, execFileSync } from "node:child_process";
import { copyFileSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const published = process.argv.includes("--published");
const withCli = process.argv.includes("--cli");
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
  const manifestOf = (name) =>
    JSON.parse(readFileSync(join(repository, "packages", name, "package.json"), "utf8"));
  // The files each tarball carries, by package: what a user receives, not what the checkout has.
  const shipped = {};
  const tarballs = published
    ? packages.map((name) => `@casoon/${name}@${manifestOf(name).version}`)
    : packages.map((name) => {
        const directory = join(repository, "packages", name);
        // The prepack output shares stdout, so the file name is the last line. `--json` lists
        // the files of the tarball.
        const lines = run("npm", ["pack", "--json", "--pack-destination", work], directory).trim().split("\n");
        const start = lines.findIndex((line) => line === "[");
        const [packed] = JSON.parse(lines.slice(start).join("\n"));
        shipped[name] = packed.files.map((file) => file.path);
        console.log(`packed ${packed.filename}`);
        return join(work, packed.filename);
      });
  if (!published) {
    const has = (name, path) => shipped[name].includes(path);
    check(
      "tarball of chartlet carries wasm, stylesheet, types and the Astro component",
      ["src/chartlet.wasm", "src/chartlet.css", "src/index.d.ts", "src/Chart.astro"].every((path) => has("chartlet", path)),
    );
    check(
      "tarball of chartlet-mcp carries server, view and schema",
      ["src/server.mjs", "src/view.html", "schema/chartlet.schema.json"].every((path) => has("chartlet-mcp", path)),
    );
    check(
      "tarballs carry no tests, sources of the Rust crate or local state",
      Object.values(shipped).flat().every((path) => !/^(test|tests|node_modules|target|plan)\//.test(path)),
    );
  }

  writeFileSync(
    join(project, "package.json"),
    '{ "name": "chartlet-smoke", "private": true, "type": "module" }\n',
  );
  run("npm", ["install", "--no-audit", "--no-fund", ...tarballs], project);

  const versions = packages.map((name) => manifestOf(name).version);
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
    const manifest = renderChartDetailed(bar, { format: "svg", manifest: true }).manifest;
    out.manifest = { chartlet: manifest.chartlet, schemaVersion: manifest.schemaVersion, hasHashes: Boolean(manifest.specHash && manifest.outputHash) };
    const installed = (name) => JSON.parse(readFileSync(new URL("./node_modules/" + name + "/package.json", import.meta.url), "utf8"));
    out.installed = { chartlet: installed("@casoon/chartlet").version, mcp: installed("@casoon/chartlet-mcp").version, range: installed("@casoon/chartlet-mcp").dependencies["@casoon/chartlet"] };
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
  check(
    "manifest names the installed compiler version and the specification version",
    out.manifest.chartlet === out.installed.chartlet && out.manifest.schemaVersion === 1 && out.manifest.hasHashes,
    `compiler ${out.manifest.chartlet}, schemaVersion ${out.manifest.schemaVersion}`,
  );
  // The contract of docs/reference/support.md: chartlet-mcp needs a renderer of the same minor,
  // not the identical version. Before 1.0 a caret range is the same minor from the given patch.
  const [major, minor, patch] = out.installed.chartlet.split(".").map(Number);
  const [rMajor, rMinor, rPatch] = out.installed.range.replace(/^\^/, "").split(".").map(Number);
  check(
    "installed renderer satisfies the range of chartlet-mcp",
    out.installed.range.startsWith("^") &&
      major === rMajor &&
      (major > 0 || (minor === rMinor && patch >= rPatch)),
    `${out.installed.chartlet} for ${out.installed.range}`,
  );
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
  check(
    "bundled schema is the schema of this release",
    readFileSync(out.schema.path, "utf8") === readFileSync(join(repository, "schema/chartlet.schema.json"), "utf8"),
  );

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
    const call = async (name, args) => (await send("tools/call", { name, arguments: args })).result;
    const bar = { schemaVersion: 1, type: "bar", title: "Revenue", data: [{ label: "Q1", value: 12 }, { label: "Q2", value: 15 }] };
    const flow = JSON.parse(readFileSync(join(project, "release-flow.json"), "utf8"));
    const chart = await call("chartlet_render", { spec: bar, format: "svg", size: "slide-16x9" });
    check(
      "MCP renders a chart at a size preset",
      !chart.isError && chart.structuredContent?.content.includes('width="1920" height="1080"'),
    );
    const diagram = await call("chartlet_render", { spec: flow, format: "svg" });
    check(
      "MCP renders a diagram identical to the golden SVG",
      !diagram.isError &&
        diagram.structuredContent?.content.trimEnd() === readFileSync(join(project, "release-flow.svg"), "utf8").trimEnd(),
    );
    check(
      "MCP manifest names the installed compiler",
      chart.structuredContent?.manifest.chartlet === out.installed.chartlet,
      chart.structuredContent?.manifest.chartlet,
    );
    const invalid = await call("chartlet_validate_spec", { spec: { schemaVersion: 1, type: "bar" } });
    check("MCP reports an invalid spec with code and path", invalid.structuredContent?.ok === false && Boolean(invalid.structuredContent.error?.code && invalid.structuredContent.error?.path));
  } finally {
    server.kill();
  }
  if (withCli) {
    // The documented crate path, separately from the npm packages: install the crate the way
    // `cargo install chartlet` does, from the checked-in lock file, into an empty root.
    const root = join(work, "cli");
    run("cargo", ["install", "--locked", "--path", repository, "--root", root, "--target-dir", join(work, "cli-target")], work);
    const chartlet = join(root, "bin", "chartlet");
    const manifestPath = join(work, "cli-manifest.json");
    const svg = run(chartlet, ["render", join(project, "release-flow.json"), "--manifest", manifestPath], work);
    const version = JSON.parse(readFileSync(manifestPath, "utf8")).chartlet;
    check("CLI manifest names the release version", version === versions[0], version);
    check(
      "CLI output matches the golden SVG and the WebAssembly output",
      svg.trimEnd() === readFileSync(join(project, "release-flow.svg"), "utf8").trimEnd(),
    );
  }
} catch (error) {
  check("smoke run", false, error.message);
} finally {
  rmSync(work, { recursive: true, force: true });
}
console.log(failed ? "smoke FAILED" : "smoke passed");
process.exit(failed ? 1 : 0);
