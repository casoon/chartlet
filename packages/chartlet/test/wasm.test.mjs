import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";
import test from "node:test";

import { createRenderer } from "@casoon/chartlet/wasm";

import { renderChart, renderChartDetailed } from "../src/render.mjs";

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
    if (existsSync(join(examples, `${base}.print.svg`))) {
      assert.equal(
        renderChart(spec, { format: "svg", variant: "print" }).content,
        readExample(`${base}.print.svg`),
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
    [dense, { format: "svg", strict: true, allowWarnings: ["dense_chart"] }],
    [spec, { format: "svg", variant: "mobile" }],
    [spec, { format: "html", variant: "print" }],
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

test("reports the outcome as structured diagnostics without throwing", () => {
  const spec = JSON.parse(readExample("monthly-trend.json"));
  const truncated = structuredClone(spec);
  truncated.data[1].label = "A category label far too long to fit under its bar";
  const cases = [
    [spec, { format: "svg" }],
    [spec, { format: "html", idPrefix: "trend", manifest: true }],
    [truncated, { format: "svg" }],
    [truncated, { format: "svg", strict: true }],
    [{ ...spec, title: "x".repeat(300) }, { format: "svg" }],
    [{ ...spec, type: "pie" }, {}],
    ['{"schemaVersion": 1,', {}],
  ];
  for (const [input, options] of cases) {
    const fromWasm = renderChartDetailed(input, options);
    assert.deepEqual(fromWasm, renderChartDetailed(input, { ...options, binary }));
    for (const warning of fromWasm.warnings) {
      assert.deepEqual(Object.keys(warning).sort(), ["code", "message", "path"]);
    }
    if (fromWasm.ok) {
      const thrown = renderChart(input, options);
      assert.equal(fromWasm.content, thrown.content);
      assert.deepEqual(fromWasm.styleHashes, thrown.styleHashes);
      assert.deepEqual(fromWasm.manifest, thrown.manifest);
    } else {
      assert.deepEqual(Object.keys(fromWasm.error).sort(), ["code", "message", "path"]);
    }
  }

  const allowed = renderChartDetailed(truncated, {
    format: "svg",
    strict: true,
    allowWarnings: ["text_truncated"],
  });
  assert.equal(allowed.ok, true);
  assert.deepEqual(
    allowed.warnings.map(({ code }) => code),
    ["text_truncated"],
  );

  const described = renderChart(spec, { format: "svg", alternative: true });
  assert.ok(described.content.includes(`<desc id="`));
  assert.ok(described.content.includes(described.alternative.description));
  assert.equal(described.alternative.table.columns.length, 2);
  assert.equal(described.alternative.table.rows.length, spec.data.length);
  assert.deepEqual(
    renderChart(spec, { format: "svg", alternative: true, binary }).alternative,
    described.alternative,
  );

  const pie = renderChartDetailed({ ...spec, type: "pie" });
  assert.equal(pie.ok, false);
  assert.equal(pie.error.path, "/type");
  assert.equal(renderChartDetailed('{"schemaVersion": 1,').error.code, "invalid_json");
  assert.deepEqual(
    renderChartDetailed(truncated, { format: "svg" }).warnings.map(({ code }) => code),
    ["text_truncated"],
  );
});

test("exports the renderer and the module for runtimes that import WebAssembly", () => {
  const wasm = readFileSync(new URL(import.meta.resolve("@casoon/chartlet/chartlet.wasm")));
  const { renderChart: render } = createRenderer(new WebAssembly.Module(wasm));
  const spec = JSON.parse(readExample("monthly-revenue.json"));

  assert.equal(render(spec).content, readExample("monthly-revenue.html"));
});

test("returns the CSP hash of every distinct inline style", () => {
  const wasm = readFileSync(new URL(import.meta.resolve("@casoon/chartlet/chartlet.wasm")));
  const { renderChart: render } = createRenderer(new WebAssembly.Module(wasm));
  const expected = (content) => [
    ...new Set(
      [...content.matchAll(/<style>(.*?)<\/style>/gs)].map(
        ([, style]) => `'sha256-${createHash("sha256").update(style).digest("base64")}'`,
      ),
    ),
  ];
  for (const name of ["topicmap-sample", "mobile-revenue", "monthly-revenue"]) {
    const spec = JSON.parse(readExample(`${name}.json`));
    for (const format of ["svg", "html"]) {
      const results = [
        renderChart(spec, { format }),
        renderChart(spec, { format, binary }),
        render(spec, { format }),
      ];
      const hashes = expected(results[0].content);
      assert.ok(hashes.length > 0);
      for (const result of results) {
        assert.deepEqual(result.styleHashes, hashes);
      }
    }
  }
});

test("returns the same provenance manifest from WebAssembly and the CLI", () => {
  const spec = JSON.parse(readExample("mobile-revenue.json"));
  assert.equal(renderChart(spec).manifest, undefined);
  assert.equal(renderChart(spec, { binary }).manifest, undefined);

  for (const options of [
    { format: "svg", manifest: true },
    { format: "html", idPrefix: "revenue", manifest: true },
    { format: "svg", variant: "mobile", manifest: true },
  ]) {
    const fromWasm = renderChart(spec, options);
    const fromCli = renderChart(spec, { ...options, binary });
    assert.deepEqual(fromWasm.manifest, fromCli.manifest);

    const { manifest } = fromWasm;
    const hash = createHash("sha256").update(fromWasm.content).digest("hex");
    assert.equal(manifest.outputHash, `sha256:${hash}`);
    assert.match(manifest.specHash, /^sha256:[0-9a-f]{64}$/);
    assert.equal(manifest.schemaVersion, 1);
    assert.equal(manifest.format, options.format);
    assert.equal(manifest.variant, options.variant ?? "desktop");
    assert.match(manifest.idPrefix, options.idPrefix ? /^revenue$/ : /^chartlet-[0-9a-f]{16}$/);
    assert.deepEqual(Object.keys(manifest).sort(), [
      "chartlet",
      "format",
      "idPrefix",
      "outputHash",
      "schemaVersion",
      "specHash",
      "variant",
      "warnings",
    ]);
  }
});
