import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { renderChart } from "@casoon/chartlet";

import { explainSpec, renderSpec, resolveOutputPath, validateSpec } from "../src/charts.mjs";

// The bundled WebAssembly build renders, not an installed CLI.
delete process.env.CHARTLET_BIN;

const examples = new URL("../../../examples/", import.meta.url);
const example = (name) => JSON.parse(readFileSync(new URL(`${name}.json`, examples), "utf8"));

const bar = {
  schemaVersion: 1,
  type: "bar",
  title: "Revenue",
  data: [
    { label: "Q1", value: 12 },
    { label: "Q2", value: 9 },
    { label: "Q3", value: 15 },
    { label: "Q4", value: 15 },
  ],
};

test("validates with structured errors and warnings", () => {
  assert.deepEqual(validateSpec(bar), { ok: true, warnings: [] });
  assert.deepEqual(validateSpec({ ...bar, type: "pie" }).error.path, "/type");
  assert.equal(validateSpec("{").error.code, "invalid_json");
  assert.equal(validateSpec(JSON.stringify(bar)).ok, true);

  const long = example("monthly-trend");
  long.data[0].label = "A category label far too long to fit under its bar";
  const result = validateSpec(long);
  assert.equal(result.ok, true);
  assert.deepEqual(
    result.warnings.map(({ code, path }) => [code, path]),
    [["text_truncated", "/data/0/label"]],
  );
});

test("reports warnings of the mobile variant", () => {
  const spec = example("mobile-revenue");
  spec.panes[0].layers[0].name =
    "A very long series name that cannot fit into a narrow mobile legend row at all";
  spec.mobile.width = 280;
  assert.ok(validateSpec(spec).warnings.some(({ message }) => message.startsWith("mobile variant")));
});

test("renders the same bytes as @casoon/chartlet with a manifest", () => {
  const spec = example("monthly-revenue");
  for (const format of ["svg", "html"]) {
    const result = renderSpec({ spec, format, idPrefix: "revenue" });
    assert.equal(result.ok, true);
    assert.equal(result.content, renderChart(spec, { format, idPrefix: "revenue" }).content);
    assert.equal(
      result.manifest.outputHash,
      `sha256:${createHash("sha256").update(result.content).digest("hex")}`,
    );
    assert.equal(result.manifest.format, format);
  }
  assert.equal(renderSpec({ spec: { ...spec, type: "pie" } }).error.code, "invalid_spec");
});

test("writes to outputPath inside the root and returns path and size", () => {
  const root = mkdtempSync(join(tmpdir(), "chartlet-mcp-"));
  try {
    const result = renderSpec({ spec: bar, format: "svg", outputPath: "out/charts/revenue.svg" }, root);
    assert.equal(result.content, undefined);
    assert.equal(result.path, join("out", "charts", "revenue.svg"));
    const written = readFileSync(join(root, "out/charts/revenue.svg"));
    assert.equal(result.bytes, written.length);
    assert.equal(result.manifest.outputHash, `sha256:${createHash("sha256").update(written).digest("hex")}`);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("refuses output paths outside the root", () => {
  const root = mkdtempSync(join(tmpdir(), "chartlet-mcp-"));
  const outside = mkdtempSync(join(tmpdir(), "chartlet-mcp-outside-"));
  try {
    mkdirSync(join(root, "sub"));
    symlinkSync(outside, join(root, "link"));
    writeFileSync(join(outside, "target.svg"), "");
    symlinkSync(join(outside, "target.svg"), join(root, "file.svg"));

    for (const [path, message] of [
      ["/etc/passwd", /relative path/],
      [join(root, "chart.svg"), /relative path/],
      ["C:\\chart.svg", /relative path/],
      ["", /relative path/],
      ["../chart.svg", /must not contain ".."/],
      ["sub/../../chart.svg", /must not contain ".."/],
      ["sub\\..\\..\\chart.svg", /must not contain ".."/],
      [".", /inside/],
      ["link/chart.svg", /symbolic link/],
      ["link/new/dir/chart.svg", /symbolic link/],
      ["file.svg", /symbolic link/],
    ]) {
      assert.throws(() => resolveOutputPath(path, root), message, path);
    }
    assert.throws(() => resolveOutputPath("chart.svg", "/"), /file system root/);
    assert.throws(() => renderSpec({ spec: bar, outputPath: "../x.svg" }, root), /must not contain/);
    assert.equal(resolveOutputPath("sub/./chart.svg", root), join(resolveOutputPath("sub/chart.svg", root)));
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(outside, { recursive: true, force: true });
  }
});

test("explains with computed facts only", () => {
  const result = explainSpec(bar);
  assert.match(result.note, /^Computed, not interpreted/);
  assert.equal(result.type, "bar");
  assert.match(result.generatedDescription, /Bar chart/);
  assert.deepEqual(result.series, [
    {
      name: null,
      count: 4,
      missing: 0,
      min: { value: 9, at: ["Q2"] },
      max: { value: 15, at: ["Q3", "Q4"] },
      first: { value: 12, at: "Q1" },
      last: { value: 15, at: "Q4" },
    },
  ]);
});

test("returns the generated description next to the spec's own", () => {
  const spec = example("monthly-trend");
  const result = explainSpec(spec);
  assert.equal(result.specDescription, spec.description);
  assert.notEqual(result.generatedDescription, spec.description);
  assert.match(result.generatedDescription, /^Line chart/);
  assert.equal(result.series[0].missing, 1);
});

test("explains every data layer of time charts, ohlc, stripes, calendar and ranges", () => {
  const share = explainSpec(example("share-price"));
  const ohlc = share.series.find(({ mark }) => mark === "ohlc");
  assert.equal(ohlc.pane, "Price (USD)");
  assert.deepEqual(ohlc.first, { value: 49.24, at: "2026-01-07" });
  assert.deepEqual(ohlc.min, { value: 48.93, at: ["2026-01-07"] });
  assert.equal(share.seriesCount, share.series.length);
  assert.ok(share.generatedDescription.includes("&") === false);

  const stripes = explainSpec(example("warming-stripes")).series[0];
  assert.deepEqual(stripes.first, { value: -0.38, at: 1850 });
  assert.equal(stripes.missing, 1);

  const calendar = explainSpec(example("daily-anomaly-calendar")).series[0];
  assert.deepEqual(calendar.max, { value: 2.39, at: ["2024-08-27", "2024-08-28"] });

  const multiples = explainSpec(example("emission-pathways"));
  assert.deepEqual(
    [...new Set(multiples.series.map(({ pane }) => pane))],
    ["Energy", "Industry", "Transport", "Buildings"],
  );

  assert.deepEqual(explainSpec(example("topicmap-sample")).series, []);
  assert.equal(explainSpec({ ...bar, type: "pie" }).error.path, "/type");
});
