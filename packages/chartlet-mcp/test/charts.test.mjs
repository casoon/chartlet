import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
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

test("advice adds checked help for a narrow canvas and only when asked for", () => {
  const narrow = { ...example("order-flow"), width: 320 };
  assert.equal("help" in validateSpec(narrow), false);
  const { help } = validateSpec(narrow, { advice: true });
  const width = help.find((entry) => entry.code === "canvas_too_small" && entry.path === "/width");
  assert.equal(width.field, "/width");
  assert.equal(width.remedies[0].validated, true);
  assert.equal(width.remedies[0].patch[0].path, "/width");
  assert.equal(narrow.width, 320, "the specification passed in is not changed");
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

test("renders the view of an architecture model it is asked for", () => {
  const spec = example("shop-views");
  const checkout = renderSpec({ spec, format: "svg", view: "checkout" });
  assert.equal(checkout.ok, true);
  assert.equal(checkout.content, renderChart(spec, { format: "svg", view: "checkout" }).content);
  assert.equal(checkout.manifest.view, "checkout");
  assert.equal(renderSpec({ spec, format: "svg" }).manifest.view, "overview");
  const unknown = renderSpec({ spec, view: "deploy" });
  assert.equal(unknown.ok, false);
  assert.deepEqual([unknown.error.code, unknown.error.path], ["unknown_view", "/render/view"]);
});

test("shows the perspective of an architecture model it is asked for", () => {
  const spec = example("service-perspectives");
  const debt = renderSpec({ spec, format: "svg", view: "owners", perspective: "debt" });
  assert.equal(debt.ok, true);
  assert.equal(debt.content, renderChart(spec, { format: "svg", view: "owners", perspective: "debt" }).content);
  assert.equal(debt.manifest.perspective, "debt");
  assert.equal(renderSpec({ spec, format: "svg", view: "owners" }).manifest.perspective, "owner");
  const unknown = renderSpec({ spec, perspective: "cost" });
  assert.deepEqual([unknown.error.code, unknown.error.path], ["unknown_perspective", "/render/perspective"]);
});

test("renders all views of an architecture model on one page", () => {
  const spec = example("shop-views");
  const page = renderSpec({ spec, allViews: true });
  assert.equal(page.ok, true);
  assert.equal(page.content, renderChart(spec, { allViews: true }).content);
  assert.deepEqual(page.manifest.views, ["overview", "checkout"]);
  const svg = renderSpec({ spec, format: "svg", allViews: true });
  assert.deepEqual([svg.error.code, svg.error.path], ["option_not_supported", "/render/allViews"]);
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
    symlinkSync(join(outside), join(root, "sub", "deeper"));
    writeFileSync(join(outside, "target.svg"), "");
    symlinkSync(join(outside, "target.svg"), join(root, "file.svg"));

    for (const [path, message] of [
      ["/etc/passwd", /relative to CHARTLET_MCP_ROOT/],
      [join(root, "chart.svg"), /relative to CHARTLET_MCP_ROOT/],
      ["C:\\chart.svg", /relative to CHARTLET_MCP_ROOT/],
      ["", /relative to CHARTLET_MCP_ROOT/],
      ["../chart.svg", /must not contain ".."/],
      ["..", /must not contain ".."/],
      ["sub/../../chart.svg", /must not contain ".."/],
      ["sub/../chart.svg", /must not contain ".."/],
      ["sub\\..\\..\\chart.svg", /must not contain ".."/],
      [".", /inside CHARTLET_MCP_ROOT/],
      ["link/chart.svg", /symbolic link/],
      ["link/new/dir/chart.svg", /symbolic link/],
      ["sub/deeper/chart.svg", /symbolic link/],
      ["file.svg", /symbolic link/],
    ]) {
      assert.throws(() => resolveOutputPath(path, root), message, path);
    }
    assert.throws(() => renderSpec({ spec: bar, outputPath: "../x.svg" }, root), /must not contain/);
    assert.deepEqual(resolveOutputPath("sub/./chart.svg", root), resolveOutputPath("sub/chart.svg", root));
    assert.deepEqual(readdirSync(outside), ["target.svg"]);
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(outside, { recursive: true, force: true });
  }
});

test("refuses outputPath without a usable CHARTLET_MCP_ROOT", () => {
  const root = mkdtempSync(join(tmpdir(), "chartlet-mcp-"));
  try {
    for (const missing of [undefined, ""]) {
      assert.throws(
        () => renderSpec({ spec: bar, outputPath: "chart.svg" }, missing),
        /outputPath is disabled: .*CHARTLET_MCP_ROOT/,
      );
    }
    assert.equal(renderSpec({ spec: bar, format: "svg" }).ok, true);
    assert.throws(() => resolveOutputPath("chart.svg", join(root, "missing")), /does not exist/);
    writeFileSync(join(root, "plain"), "");
    assert.throws(() => resolveOutputPath("chart.svg", join(root, "plain")), /not a directory/);
    assert.throws(() => resolveOutputPath("chart.svg", "/"), /file system root/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("resolves the root through symbolic links", () => {
  const real = mkdtempSync(join(tmpdir(), "chartlet-mcp-real-"));
  const alias = join(tmpdir(), `chartlet-mcp-alias-${process.pid}`);
  try {
    symlinkSync(real, alias);
    const result = renderSpec({ spec: bar, format: "svg", outputPath: "chart.svg" }, alias);
    assert.equal(result.path, "chart.svg");
    assert.equal(readFileSync(join(real, "chart.svg")).length, result.bytes);
  } finally {
    rmSync(alias, { force: true });
    rmSync(real, { recursive: true, force: true });
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

  assert.equal(explainSpec({ ...bar, type: "pie" }).error.path, "/type");
});

test("a project theme in the root applies unless a theme is given, and the result names it", () => {
  const root = mkdtempSync(join(tmpdir(), "chartlet-mcp-theme-"));
  try {
    const plain = renderSpec({ spec: bar, format: "svg" }, root);
    assert.equal(plain.theme, undefined);
    assert.equal(plain.manifest.theme, undefined);
    assert.equal(plain.notes.length, 1);
    assert.match(plain.notes[0], /default look is used/);
    assert.match(plain.notes[0], /chartlet_theme_from_brand/);
    assert.equal(renderSpec({ spec: bar, format: "svg" }).notes, undefined, "no root, no note");

    writeFileSync(join(root, "chartlet.theme.json"), JSON.stringify({ tokens: { accent: "#123456" } }));
    const themed = renderSpec({ spec: bar, format: "svg" }, root);
    assert.match(themed.content, /#123456/);
    assert.equal(themed.theme, "chartlet.theme.json");
    assert.equal(themed.manifest.theme, "chartlet.theme.json");
    assert.equal(themed.notes, undefined);

    const dark = renderSpec({ spec: bar, format: "svg", theme: "dark" }, root);
    assert.doesNotMatch(dark.content, /#123456/);
    assert.equal(dark.theme, "dark");
    assert.equal(dark.manifest.theme, "dark");
    assert.equal(renderSpec({ spec: bar, format: "svg", theme: { tokens: { accent: "#654321" } } }, root).theme, "inline");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
