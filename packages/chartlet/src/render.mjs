import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { createRenderer } from "./wasm.mjs";

const MAX_OUTPUT_BYTES = 16 * 1024 * 1024;

let renderer;

// Renders with the bundled WebAssembly build, or with the CLI when `options.binary` or
// `CHARTLET_BIN` names one. Both produce the same bytes.
export function renderChart(spec, options = {}) {
  const binary = options.binary ?? process.env.CHARTLET_BIN;
  if (!binary) {
    renderer ??= createRenderer(new WebAssembly.Module(readWasm()));
    return renderer.renderChart(spec, options);
  }
  return renderWithCli(binary, spec, options);
}

// Resolved by package name, not relative to this file: bundlers such as Vite copy this module
// into a server chunk elsewhere, but leave the package where Node can resolve it.
function readWasm() {
  const url = new URL(import.meta.resolve("@casoon/chartlet/chartlet.wasm"));
  try {
    return readFileSync(url);
  } catch (error) {
    if (error.code === "ENOENT") {
      throw new Error(
        `chartlet.wasm is missing at ${url.pathname}. In a checkout, run \`npm run build:wasm\`; otherwise set CHARTLET_BIN to a chartlet executable.`,
      );
    }
    throw error;
  }
}

// Renders like `renderChart`, but reports the outcome as data instead of throwing: `ok`, the
// structured `error` of a failed render, and every warning as `{ code, path, message }`.
export function renderChartDetailed(spec, options = {}) {
  const binary = options.binary ?? process.env.CHARTLET_BIN;
  if (!binary) {
    renderer ??= createRenderer(new WebAssembly.Module(readWasm()));
    return renderer.renderChartDetailed(spec, options);
  }
  const { result, manifest } = runCli(
    binary,
    typeof spec === "string" ? spec : JSON.stringify(spec),
    options,
    ["--diagnostics", "json"],
  );
  const diagnostics = JSON.parse(result.stderr);
  if (!diagnostics.ok) {
    return { ok: false, error: diagnostics.error, warnings: diagnostics.warnings };
  }
  const content = result.stdout.trimEnd();
  const rendered = {
    ok: true,
    content,
    styleHashes: styleHashes(content),
    warnings: diagnostics.warnings,
  };
  if (manifest) {
    rendered.manifest = manifest;
  }
  return rendered;
}

function renderWithCli(binary, spec, options) {
  const { result, manifest } = runCli(binary, JSON.stringify(spec), options, []);
  if (result.status !== 0) {
    throw new Error(result.stderr.trim() || `chartlet exited with status ${result.status}`);
  }

  const content = result.stdout.trimEnd();
  const rendered = {
    content,
    styleHashes: styleHashes(content),
    warnings: result.stderr
      .split("\n")
      .map((warning) => warning.trim())
      .filter(Boolean),
  };
  if (manifest) {
    rendered.manifest = manifest;
  }
  return rendered;
}

// Runs `chartlet render` on `input` and returns the process result with the manifest it wrote.
function runCli(binary, input, options, extraArgs) {
  const format = options.format ?? "html";
  const table = options.table ?? "details";
  const args = ["render", "-", "--format", format, "--table", table];

  if (options.idPrefix) {
    args.push("--id-prefix", options.idPrefix);
  }
  if (options.variant) {
    args.push("--variant", options.variant);
  }
  if (options.strict) {
    args.push("--strict");
  }
  args.push(...extraArgs);
  // The CLI writes the manifest to a file; it is read back and removed with its directory.
  const manifestDirectory = options.manifest
    ? mkdtempSync(join(tmpdir(), "chartlet-manifest-"))
    : undefined;
  const manifestPath = manifestDirectory && join(manifestDirectory, "manifest.json");
  if (manifestPath) {
    args.push("--manifest", manifestPath);
  }

  let result;
  let manifest;
  try {
    result = spawnSync(binary, args, {
      input,
      encoding: "utf8",
      maxBuffer: MAX_OUTPUT_BYTES,
    });
    if (manifestPath && result.status === 0) {
      manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    }
  } finally {
    if (manifestDirectory) {
      rmSync(manifestDirectory, { recursive: true, force: true });
    }
  }

  if (result.error) {
    if (result.error.code === "ENOENT") {
      throw new Error(
        `chartlet executable not found at ${JSON.stringify(binary)}. Install the CLI, or unset CHARTLET_BIN to use the bundled WebAssembly build.`,
      );
    }
    throw result.error;
  }
  return { result, manifest };
}

// The same CSP source expressions the WebAssembly build returns: one per distinct `<style>`
// element, in order of first appearance.
function styleHashes(content) {
  const hashes = [];
  for (const [, style] of content.matchAll(/<style>(.*?)<\/style>/gs)) {
    const hash = `'sha256-${createHash("sha256").update(style).digest("base64")}'`;
    if (!hashes.includes(hash)) {
      hashes.push(hash);
    }
  }
  return hashes;
}
