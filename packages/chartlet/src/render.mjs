import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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
    return wasmRenderer().renderChart(spec, options);
  }
  return renderWithCli(binary, spec, options);
}

function wasmRenderer() {
  renderer ??= createRenderer(new WebAssembly.Module(readWasm()));
  return renderer;
}

// The CLI writes no text alternative; the bundled WebAssembly build derives the same one.
function withAlternative(rendered, spec, options) {
  if (options.alternative) {
    const { alternative, alternatives } = wasmRenderer().renderChartDetailed(spec, {
      format: options.allViews ? "html" : "svg",
      view: options.view,
      perspective: options.perspective,
      allViews: options.allViews,
      alternative: true,
    });
    rendered.alternative = alternative;
    if (alternatives) {
      rendered.alternatives = alternatives;
    }
  }
  return rendered;
}

// The shared stylesheet that charts rendered with `styles: "external"` rely on, for every chart
// type or for those in `options.types`. The package also ships the full one as
// `@casoon/chartlet/chartlet.css`.
export function stylesheet(options = {}) {
  return wasmRenderer().stylesheet(options);
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
    return wasmRenderer().renderChartDetailed(spec, options);
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
  return withAlternative(rendered, spec, options);
}

// The inspections of a model, as data: `{ ok: true, findings }`, or `{ ok: false, error }` for an
// invalid specification or a type without inspections. The CLI reports the same findings.
export function inspectChart(spec, options = {}) {
  const binary = options.binary ?? process.env.CHARTLET_BIN;
  if (!binary) {
    return wasmRenderer().inspectChart(spec, options);
  }
  const args = ["inspect", "-", "--format", "json", "--diagnostics", "json"];
  for (const code of options.disable ?? []) {
    args.push("--disable", code);
  }
  const result = spawnSync(binary, args, {
    input: typeof spec === "string" ? spec : JSON.stringify(spec),
    encoding: "utf8",
    maxBuffer: MAX_OUTPUT_BYTES,
  });
  if (result.error) {
    throw cliError(result.error, binary);
  }
  const diagnostics = JSON.parse(result.stderr);
  if (!diagnostics.ok) {
    return { ok: false, error: diagnostics.error };
  }
  return { ok: true, findings: JSON.parse(result.stdout).findings };
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
  return withAlternative(rendered, spec, options);
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
  if (options.view) {
    args.push("--view", options.view);
  }
  if (options.perspective) {
    args.push("--perspective", options.perspective);
  }
  if (options.allViews) {
    args.push("--all-views");
  }
  if (typeof options.theme === "string") {
    args.push("--theme", options.theme);
  }
  if (options.styles) {
    args.push("--styles", options.styles);
  }
  if (options.hooks) {
    args.push("--hooks");
  }
  if (options.strict) {
    args.push("--strict");
  }
  for (const code of options.allowWarnings ?? []) {
    args.push("--allow-warning", code);
  }
  args.push(...extraArgs);
  // The CLI writes the manifest to a file and reads a theme object from one; both live in a
  // directory that is removed afterwards.
  const directory =
    options.manifest || (options.theme && typeof options.theme === "object")
      ? mkdtempSync(join(tmpdir(), "chartlet-"))
      : undefined;
  const manifestPath = options.manifest && join(directory, "manifest.json");
  if (directory && typeof options.theme === "object") {
    const themePath = join(directory, "theme.json");
    writeFileSync(themePath, JSON.stringify(options.theme));
    args.push("--theme", themePath);
  }
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
    if (directory) {
      rmSync(directory, { recursive: true, force: true });
    }
  }

  if (result.error) {
    throw cliError(result.error, binary);
  }
  return { result, manifest };
}

function cliError(error, binary) {
  if (error.code === "ENOENT") {
    return new Error(
      `chartlet executable not found at ${JSON.stringify(binary)}. Install the CLI, or unset CHARTLET_BIN to use the bundled WebAssembly build.`,
    );
  }
  return error;
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
