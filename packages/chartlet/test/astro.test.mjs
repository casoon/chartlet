import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";
import test from "node:test";

const packageDirectory = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repository = resolve(packageDirectory, "../..");
const fixture = join(packageDirectory, "test-fixture");
const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
    cwd: repository,
    encoding: "utf8",
  }),
);
execFileSync("cargo", ["build", "--quiet"], { cwd: repository });
const binary = join(metadata.target_directory, "debug", "chartlet");

// Without CHARTLET_BIN the component renders with the bundled WebAssembly build.
for (const [renderer, chartletBin] of [
  ["WebAssembly", undefined],
  ["the CLI", binary],
]) {
  test(`Astro builds a static chart without client JavaScript through ${renderer}`, () => {
    const env = { ...process.env, CHARTLET_BIN: chartletBin };
    if (chartletBin === undefined) {
      delete env.CHARTLET_BIN;
    }
    execFileSync("astro", ["build", "--root", fixture], {
      cwd: packageDirectory,
      env,
      stdio: "pipe",
    });
    const html = readFileSync(join(fixture, "dist/index.html"), "utf8");

    assert.match(html, /smoke-chart-title/);
    assert.match(html, /class="chartlet-line"/);
    assert.match(html, /<table>/);
    // `hooks` adds the data-* hooks for the optional module, and still no script.
    assert.match(html, /data-chartlet-type="line" data-chartlet-id="hooked-chart"/);
    assert.match(html, /<table data-chartlet-table="hooked-chart">/);
    assert.doesNotMatch(html, /<script/);
  });
}
