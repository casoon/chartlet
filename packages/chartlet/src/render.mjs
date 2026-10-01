import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

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

function renderWithCli(binary, spec, options) {
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

  const result = spawnSync(binary, args, {
    input: JSON.stringify(spec),
    encoding: "utf8",
    maxBuffer: MAX_OUTPUT_BYTES,
  });

  if (result.error) {
    if (result.error.code === "ENOENT") {
      throw new Error(
        `chartlet executable not found at ${JSON.stringify(binary)}. Install the CLI, or unset CHARTLET_BIN to use the bundled WebAssembly build.`,
      );
    }
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(result.stderr.trim() || `chartlet exited with status ${result.status}`);
  }

  return {
    content: result.stdout.trimEnd(),
    warnings: result.stderr
      .split("\n")
      .map((warning) => warning.trim())
      .filter(Boolean),
  };
}
