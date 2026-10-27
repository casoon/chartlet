#!/usr/bin/env node
// Measures the compiler on realistic and limit-sized workloads and prints a Markdown report.
//
//   cargo build --release
//   node scripts/bench.mjs --binary target/release/chartlet [--runs 15] [--check]
//
// The workloads are generated here, deterministically, so a report can be reproduced on another
// machine; the numbers are not comparable across machines, which is why the report names the
// hardware. WebAssembly and MCP figures need `packages/chartlet/src/chartlet.wasm` (built with
// `npm run build:wasm`); without it those sections say so and are skipped rather than guessed.
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { cpus, platform, release, tmpdir, totalmem } from "node:os";
import { join, resolve } from "node:path";
import { gzipSync } from "node:zlib";
import { pathToFileURL } from "node:url";

const root = resolve(new URL("..", import.meta.url).pathname);
const argument = (name, fallback) => {
  const at = process.argv.indexOf(name);
  return at < 0 ? fallback : process.argv[at + 1];
};
const binary = resolve(argument("--binary", join(root, "target/release/chartlet")));
const runs = Number(argument("--runs", "15"));
if (!existsSync(binary)) {
  console.error(`No chartlet executable at ${binary}; build one with cargo build --release.`);
  process.exit(1);
}

const example = (name) => JSON.parse(readFileSync(join(root, "examples", `${name}.json`), "utf8"));
const pad = (n) => String(n).padStart(2, "0");

// ---- workloads --------------------------------------------------------------------------------

function timeSeries(layers, points) {
  const start = Date.UTC(2020, 0, 1);
  return {
    schemaVersion: 1,
    type: "time",
    title: "Dense time series at the declared limit",
    width: 1200,
    height: 600,
    timeAxis: { timezone: "UTC" },
    panes: [
      {
        layers: Array.from({ length: layers }, (_, layer) => ({
          mark: "line",
          name: `Series ${layer + 1}`,
          color: ["#2563eb", "#c2410c", "#475569", "#111827", "#15803d", "#7c3aed"][layer],
          points: Array.from({ length: points }, (_, index) => {
            const date = new Date(start + index * 86_400_000);
            const time = `${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())}`;
            return { time, value: Math.round((100 + 40 * Math.sin(index / (9 + layer)) + layer * 20) * 100) / 100 };
          }),
        })),
      },
    ],
  };
}

function nestedBoundaries() {
  // Four levels of boundaries, 40 components: the shape that #6 reported as an enormous canvas.
  const boundaries = [{ id: "b0", label: "Zone 0" }];
  for (let depth = 1; depth < 4; depth += 1) boundaries.push({ id: `b${depth}`, label: `Zone ${depth}`, in: `b${depth - 1}` });
  for (let extra = 4; extra < 12; extra += 1) boundaries.push({ id: `b${extra}`, label: `Zone ${extra}`, in: `b${extra % 3}` });
  const components = Array.from({ length: 40 }, (_, index) => ({
    id: `c${index}`,
    label: `Component ${index}`,
    sublabel: "service",
    in: `b${index % 12}`,
  }));
  const connections = Array.from({ length: 60 }, (_, index) => ({
    from: `c${index % 40}`,
    to: `c${(index * 7 + 3) % 40}`,
    label: index % 3 === 0 ? "calls" : undefined,
  })).filter((connection) => connection.from !== connection.to);
  return {
    schemaVersion: 1,
    type: "architecture",
    title: "Nested boundaries, 40 components",
    width: 2400,
    height: 1600,
    architecture: { boundaries, components, connections },
  };
}

function groupedFlow() {
  const nodes = Array.from({ length: 40 }, (_, index) => ({ id: `n${index}`, label: `Step ${index + 1}` }));
  const edges = Array.from({ length: 39 }, (_, index) => ({ from: `n${index}`, to: `n${index + 1}` }));
  const groups = Array.from({ length: 8 }, (_, group) => ({
    label: `Group ${group + 1}`,
    nodes: nodes.slice(group * 5, group * 5 + 5).map((node) => node.id),
  }));
  return { schemaVersion: 1, type: "flow", title: "40 steps in 8 groups", width: 2400, height: 1600, flow: { nodes, edges, groups } };
}

function longLabels() {
  const word = "Zahlungsverkehrsüberwachungsvorschriften";
  return {
    schemaVersion: 1,
    type: "bar",
    title: `${word} ${word} ${word}`,
    locale: "de",
    width: 900,
    height: 500,
    data: Array.from({ length: 16 }, (_, index) => ({ label: `${word} ${index + 1} ${word}`, value: index + 1 })),
  };
}

function moreEscapes() {
  const text = `<script>alert("&")</script> & ' " ${"äöü☃".repeat(8)}`;
  return {
    schemaVersion: 1,
    type: "bar",
    title: text,
    data: Array.from({ length: 12 }, (_, index) => ({ label: `${text} ${index}`, value: index })),
  };
}

const workloads = [
  { name: "small editorial bar chart", spec: example("monthly-revenue"), args: ["--format", "html"] },
  { name: "dense time series, 6 x 2000 points", spec: timeSeries(6, 2000), args: ["--format", "html"] },
  { name: "multi-view architecture, all views", spec: example("shop-views"), args: ["--format", "html", "--all-views"] },
  { name: "architecture, 4 nested levels, 40 components (#6)", spec: nestedBoundaries(), args: ["--format", "svg"] },
  { name: "flow, 40 steps in 8 groups (#6)", spec: groupedFlow(), args: ["--format", "svg"] },
  { name: "long German labels, 16 categories", spec: longLabels(), args: ["--format", "html"] },
  { name: "escaped text everywhere", spec: moreEscapes(), args: ["--format", "html"] },
];

// ---- measuring --------------------------------------------------------------------------------

const directory = mkdtempSync(join(tmpdir(), "chartlet-bench-"));
const isMac = platform() === "darwin";

function once(file, args) {
  const timed = isMac ? ["/usr/bin/time", "-l"] : ["/usr/bin/time", "-v"];
  const started = process.hrtime.bigint();
  const result = spawnSync(timed[0], [timed[1], binary, "render", file, ...args], {
    encoding: "buffer",
    maxBuffer: 256 * 1024 * 1024,
  });
  const milliseconds = Number(process.hrtime.bigint() - started) / 1e6;
  const stderr = result.stderr.toString("utf8");
  let peak = null;
  const mac = stderr.match(/(\d+)\s+maximum resident set size/);
  const linux = stderr.match(/Maximum resident set size \(kbytes\): (\d+)/);
  if (mac) peak = Number(mac[1]);
  if (linux) peak = Number(linux[1]) * 1024;
  const warnings = [...stderr.matchAll(/^warning\[([a-z_]+)\]/gm)].map((match) => match[1]);
  const failure = result.status === 0 ? "" : (stderr.split("\n").find((line) => line.startsWith("chartlet:")) ?? stderr.slice(0, 200));
  return { milliseconds, peak, size: result.stdout.length, status: result.status, warnings, stdout: result.stdout, failure };
}

const median = (values) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
const kilobytes = (bytes) => `${(bytes / 1000).toFixed(1)} kB`;
const megabytes = (bytes) => (bytes === null ? "n/a" : `${(bytes / 1e6).toFixed(1)} MB`);

const rows = [];
for (const workload of workloads) {
  const file = join(directory, "spec.json");
  const text = JSON.stringify(workload.spec);
  writeFileSync(file, text);
  const first = once(file, workload.args);
  const warm = Array.from({ length: runs }, () => once(file, workload.args));
  rows.push({
    name: workload.name,
    input: Buffer.byteLength(text),
    status: first.status,
    cold: first.milliseconds,
    warm: median(warm.map((run) => run.milliseconds)),
    worst: Math.max(...warm.map((run) => run.milliseconds)),
    peak: Math.max(first.peak ?? 0, ...warm.map((run) => run.peak ?? 0)) || null,
    output: first.size,
    gzip: gzipSync(first.stdout).length,
    warnings: [...new Set(first.warnings)],
    failure: first.failure,
  });
}

// ---- report -----------------------------------------------------------------------------------

const version = spawnSync(binary, ["--help"], { encoding: "utf8" }).status === 0
  ? JSON.parse(readFileSync(join(root, "packages/chartlet/package.json"), "utf8")).version
  : "unknown";
const lines = [];
lines.push(`Measured with \`node scripts/bench.mjs --runs ${runs}\` on ${cpus()[0].model} (${cpus().length} cores), ${Math.round(totalmem() / 2 ** 30)} GB, ${platform()} ${release()}, Node ${process.version}, chartlet ${version} (release build).`);
lines.push("");
lines.push("| Workload | Input | Cold start | Warm (median of " + runs + ") | Worst | Peak memory | Output | Output gzip | Warnings |");
lines.push("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |");
for (const row of rows) {
  lines.push(
    `| ${row.name}${row.status === 0 ? "" : " (failed)"} | ${kilobytes(row.input)} | ${row.cold.toFixed(0)} ms | ${row.warm.toFixed(0)} ms | ${row.worst.toFixed(0)} ms | ${megabytes(row.peak)} | ${kilobytes(row.output)} | ${kilobytes(row.gzip)} | ${row.failure || row.warnings.join(", ") || "none"} |`,
  );
}
lines.push("");
lines.push("Times include starting the process (cold start is the first run after the file was written; a warm run is the median of the following ones), so they bound the CLI from above.");

// WebAssembly and the MCP response, when the module is built.
const wasmPath = join(root, "packages/chartlet/src/chartlet.wasm");
if (existsSync(wasmPath)) {
  const { createRenderer } = await import(pathToFileURL(join(root, "packages/chartlet/src/wasm.mjs")).href);
  const bytes = readFileSync(wasmPath);
  const compileStart = performance.now();
  const module = new WebAssembly.Module(bytes);
  const renderer = createRenderer(module);
  const coldInstance = performance.now() - compileStart;
  lines.push("");
  lines.push(`WebAssembly: ${kilobytes(bytes.length)} (${kilobytes(gzipSync(bytes, { level: 9 }).length)} gzip -9); compile and instantiate ${coldInstance.toFixed(0)} ms.`);
  lines.push("");
  lines.push("| Workload | First render | Warm render (median) | Model-visible MCP result |");
  lines.push("| --- | ---: | ---: | ---: |");
  for (const workload of workloads) {
    const format = workload.args[1];
    const options = { format, allViews: workload.args.includes("--all-views"), alternative: false };
    const first = performance.now();
    const response = renderer.renderChartDetailed(workload.spec, options);
    const firstMs = performance.now() - first;
    const times = Array.from({ length: runs }, () => {
      const started = performance.now();
      renderer.renderChartDetailed(workload.spec, options);
      return performance.now() - started;
    });
    const visible = response.ok ? Buffer.byteLength(JSON.stringify({ content: response.content, warnings: response.warnings, styleHashes: response.styleHashes })) : 0;
    lines.push(`| ${workload.name} | ${firstMs.toFixed(0)} ms | ${median(times).toFixed(0)} ms | ${kilobytes(visible)} |`);
  }
} else {
  lines.push("");
  lines.push("WebAssembly and MCP figures: not measured, `packages/chartlet/src/chartlet.wasm` is not built here (`npm run build:wasm` in packages/chartlet).");
}

// Budgets, set from the measurements above with room for slower machines: a warm render of a
// figure within the declared limits stays under 250 ms and 64 MB, the nested architecture diagram
// (the slowest layout measured) under 2 s, and the output under 1 MB. `--check` fails on a breach,
// so a regression can be found locally or in a release check without making it part of every CI run.
const slow = "architecture, 4 nested levels";
const breaches = rows.flatMap((row) => {
  const limit = row.name.startsWith(slow) ? 2000 : 250;
  return [
    ...(row.status !== 0 ? [`${row.name}: the render failed (${row.failure})`] : []),
    ...(row.warm > limit ? [`${row.name}: ${row.warm.toFixed(0)} ms is over ${limit} ms`] : []),
    ...(row.peak !== null && row.peak > 64e6 ? [`${row.name}: ${megabytes(row.peak)} is over 64 MB`] : []),
    ...(row.output > 1e6 ? [`${row.name}: ${kilobytes(row.output)} output is over 1 MB`] : []),
  ];
});
rmSync(directory, { recursive: true, force: true });
console.log(lines.join("\n"));
if (process.argv.includes("--check") && breaches.length > 0) {
  console.error(breaches.map((breach) => `- ${breach}`).join("\n"));
  process.exit(1);
}
