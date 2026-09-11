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

test("Astro builds a static chart without client JavaScript", () => {
  execFileSync("astro", ["build", "--root", fixture], {
    cwd: packageDirectory,
    env: { ...process.env, CHARTLET_BIN: binary },
    stdio: "pipe",
  });
  const html = readFileSync(join(fixture, "dist/index.html"), "utf8");

  assert.match(html, /smoke-chart-title/);
  assert.match(html, /class="chartlet-line"/);
  assert.match(html, /<table>/);
  assert.doesNotMatch(html, /<script/);
});
