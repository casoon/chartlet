import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";
import test from "node:test";

import { createRenderer } from "@casoon/chartlet/wasm";

import { renderChart } from "../src/render.mjs";

// Without a CLI configured, renderChart uses the bundled WebAssembly build.
delete process.env.CHARTLET_BIN;

const packageDirectory = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repository = resolve(packageDirectory, "../..");
const examples = join(repository, "examples");
const readExample = (name) => readFileSync(join(examples, name), "utf8");
const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
    cwd: repository,
    encoding: "utf8",
  }),
);
execFileSync("cargo", ["build", "--quiet"], { cwd: repository });
const binary = join(metadata.target_directory, "debug", "chartlet");

for (const file of readdirSync(examples).filter((name) => name.endsWith(".json"))) {
  const base = file.slice(0, -".json".length);
  test(`renders ${base} byte-identical to the committed examples`, () => {
    const spec = JSON.parse(readExample(file));
    assert.equal(renderChart(spec, { format: "svg" }).content, readExample(`${base}.svg`));
    assert.equal(renderChart(spec, { format: "html" }).content, readExample(`${base}.html`));
    if (existsSync(join(examples, `${base}.mobile.svg`))) {
      assert.equal(
        renderChart(spec, { format: "svg", variant: "mobile" }).content,
        readExample(`${base}.mobile.svg`),
      );
    }
  });
}

test("reports warnings and errors in the words of the CLI", () => {
  const spec = JSON.parse(readExample("monthly-trend.json"));
  const dense = {
    schemaVersion: 1,
    type: "time",
    title: "Hourly load",
    timeAxis: { timezone: "UTC" },
    panes: [
      {
        valueAxis: { title: "Load" },
        layers: [
          {
            mark: "line",
            name: "Load",
            points: Array.from({ length: 1000 }, (_, index) => ({
              time: 1770000000 + index * 3600,
              value: (index % 5) + 1,
            })),
          },
        ],
      },
    ],
  };
  const cases = [
    [dense, { format: "svg" }],
    [dense, { format: "svg", strict: true }],
    [spec, { format: "svg", variant: "mobile" }],
    [spec, { format: "html", table: "visible", idPrefix: "trend" }],
    [{ ...spec, type: "pie" }, {}],
  ];
  for (const [input, options] of cases) {
    const outcome = (render) => {
      try {
        return render();
      } catch (error) {
        return { error: error.message };
      }
    };
    assert.deepEqual(
      outcome(() => renderChart(input, options)),
      outcome(() => renderChart(input, { ...options, binary })),
    );
  }
});

test("exports the renderer and the module for runtimes that import WebAssembly", () => {
  const wasm = readFileSync(new URL(import.meta.resolve("@casoon/chartlet/chartlet.wasm")));
  const { renderChart: render } = createRenderer(new WebAssembly.Module(wasm));
  const spec = JSON.parse(readExample("monthly-revenue.json"));

  assert.equal(render(spec).content, readExample("monthly-revenue.html"));
});
