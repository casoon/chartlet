import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";
import test from "node:test";

import { renderChart } from "../src/render.mjs";

const packageDirectory = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repository = resolve(packageDirectory, "../..");
const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
    cwd: repository,
    encoding: "utf8",
  }),
);
execFileSync("cargo", ["build", "--quiet"], { cwd: repository });
const binary = join(metadata.target_directory, "debug", "chartlet");
const spec = JSON.parse(
  readFileSync(join(repository, "examples/monthly-trend.json"), "utf8"),
);

test("renders through the CLI without client-side runtime output", () => {
  const result = renderChart(spec, {
    binary,
    format: "html",
    idPrefix: "astro-example",
  });

  assert.match(result.content, /^<figure/);
  assert.match(result.content, /astro-example-title/);
  assert.match(result.content, /class="chartlet-line"/);
  assert.doesNotMatch(result.content, /<script/);
  assert.deepEqual(result.warnings, []);
});
