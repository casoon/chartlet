import { spawnSync } from "node:child_process";

const MAX_OUTPUT_BYTES = 16 * 1024 * 1024;

export function renderChart(spec, options = {}) {
  const format = options.format ?? "html";
  const table = options.table ?? "details";
  const binary = options.binary ?? process.env.CHARTLET_BIN ?? "chartlet";
  const args = ["render", "-", "--format", format, "--table", table];

  if (options.idPrefix) {
    args.push("--id-prefix", options.idPrefix);
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
        `chartlet executable not found at ${JSON.stringify(binary)}. Install the CLI or set CHARTLET_BIN.`,
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
