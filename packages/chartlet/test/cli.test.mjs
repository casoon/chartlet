import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const packageDirectory = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repository = resolve(packageDirectory, "../..");
const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
    cwd: repository,
    encoding: "utf8",
  }),
);
execFileSync("cargo", ["build", "--quiet"], { cwd: repository });
const rust = join(metadata.target_directory, "debug", "chartlet");
const node = [join(packageDirectory, "src/cli.mjs")];

function run(command, args, input) {
  const [file, ...prefix] = command === rust ? [rust] : [process.execPath, ...node];
  const result = spawnSync(file, [...prefix, ...args], { cwd: repository, input, encoding: "utf8" });
  return { stdout: result.stdout, stderr: result.stderr, status: result.status };
}

const cases = [
  ["render", "examples/monthly-revenue.json"],
  ["render", "examples/release-flow.json", "--format", "html", "--id-prefix", "flow", "--table", "visible"],
  ["render", "examples/shop-views.json", "--format", "html", "--all-views"],
  ["render", "examples/monthly-revenue.json", "--theme", "dark", "--styles", "external", "--hooks"],
  ["render", "examples/monthly-revenue.json", "--theme", "examples/themes/brand.json", "--variant", "mobile"],
  ["render", "examples/monthly-revenue.json", "--size", "slide-16x9", "--diagnostics", "json"],
  ["render", "examples/monthly-revenue.json", "--view"],
  ["inspect", "examples/shop-architecture.json"],
  ["inspect", "examples/shop-architecture.json", "--format", "json", "--strict"],
  ["inspect", "examples/monthly-revenue.json"],
  ["stylesheet", "--common", "--breakpoints", "420"],
  ["stylesheet", "--types", "bar,time", "--no-common"],
];

test("writes what the Rust CLI writes", () => {
  for (const args of cases) {
    assert.deepEqual(run(node, args), run(rust, args), args.join(" "));
  }
});

test("reads stdin and reports warnings and help as the Rust CLI does", () => {
  const spec = JSON.stringify({
    schemaVersion: 1,
    type: "bar",
    title: "Labels",
    width: 200,
    data: [
      { label: "A label much too long for a chart only two hundred pixels wide", value: 1 },
      { label: "B", value: 2 },
    ],
  });
  for (const args of [[], ["--advice"], ["--strict"], ["--diagnostics", "json", "--advice"]]) {
    assert.deepEqual(run(node, ["render", "-", ...args], spec), run(rust, ["render", "-", ...args], spec), args.join(" "));
  }
  for (const input of ['{"schemaVersion":1}', "{oops"]) {
    assert.deepEqual(run(node, ["render", "-"], input), run(rust, ["render", "-"], input), input);
  }
});

test("points to the Rust CLI for what needs it", () => {
  const png = run(node, ["render", "examples/monthly-revenue.json", "--format", "png"]);
  assert.equal(png.status, 1);
  assert.equal(png.stderr, "chartlet: --format png needs the Rust CLI: cargo install chartlet --features png\n");
  const watch = run(node, ["render", "examples/monthly-revenue.json", "--watch"]);
  assert.equal(watch.stderr, "chartlet: --watch needs the Rust CLI: cargo install chartlet\n");
});
