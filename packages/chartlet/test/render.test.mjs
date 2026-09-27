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

const dailyOrders = JSON.parse(
  readFileSync(join(repository, "examples/daily-orders.json"), "utf8"),
);
const darkRevenue = JSON.parse(
  readFileSync(join(repository, "examples/revenue-vs-forecast.json"), "utf8"),
);

test("carries a time series through the wrapper with its time axis", () => {
  const result = renderChart(dailyOrders, {
    binary,
    format: "html",
    idPrefix: "time-example",
  });

  assert.match(result.content, /<polyline/);
  assert.match(result.content, /<th scope="col">Time<\/th>/);
  assert.match(result.content, /class="chartlet-tick">2026-02-14</);
  assert.deepEqual(result.warnings, []);
});

test("keeps the theme and the declared colors of a dark series", () => {
  const result = renderChart(darkRevenue, { binary, format: "svg" });

  assert.match(result.content, /chartlet-theme-dark/);
  assert.match(result.content, /stroke:var\(--chart-forecast,currentColor\)/);
  assert.deepEqual(result.warnings, []);
});

test("reports a series that outnumbers the plot pixels as a warning", () => {
  const points = Array.from({ length: 1000 }, (_, index) => ({
    time: 1770000000 + index * 3600,
    value: (index % 5) + 1,
  }));
  const result = renderChart(
    {
      schemaVersion: 1,
      type: "time",
      title: "Hourly load",
      timeAxis: { timezone: "UTC" },
      panes: [
        {
          valueAxis: { title: "Load" },
          layers: [{ mark: "line", name: "Load", points }],
        },
      ],
    },
    { binary, format: "svg" },
  );

  assert.equal(result.warnings.length, 1);
  assert.match(result.warnings[0], /warning\[dense_chart\]/);
  assert.match(result.warnings[0], /exceed the 704 horizontal pixels/);
});
