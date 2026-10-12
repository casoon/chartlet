// Builds the WebAssembly renderer from the workspace crate `wasm/` and copies it to
// src/chartlet.wasm. Runs wasm-opt when it is on PATH; the output is valid without it.
import { execFileSync, spawnSync } from "node:child_process";
import { copyFileSync, readFileSync, writeFileSync } from "node:fs";
import { gzipSync } from "node:zlib";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { createRenderer } from "../src/wasm.mjs";

const packageDirectory = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repository = resolve(packageDirectory, "../..");
const output = join(packageDirectory, "src/chartlet.wasm");

execFileSync(
  "cargo",
  ["build", "--profile", "wasm", "-p", "chartlet-wasm", "--target", "wasm32-unknown-unknown"],
  { cwd: repository, stdio: "inherit" },
);
const { target_directory: targetDirectory } = JSON.parse(
  execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
    cwd: repository,
    encoding: "utf8",
  }),
);
copyFileSync(join(targetDirectory, "wasm32-unknown-unknown/wasm/chartlet_wasm.wasm"), output);

// The WebAssembly features rustc enables by default for wasm32-unknown-unknown.
const features = [
  "bulk-memory",
  "bulk-memory-opt",
  "multivalue",
  "mutable-globals",
  "nontrapping-float-to-int",
  "reference-types",
  "sign-ext",
].map((feature) => `--enable-${feature}`);
const optimized = spawnSync("wasm-opt", ["-Oz", ...features, output, "-o", output], {
  stdio: "inherit",
});
if (optimized.error?.code === "ENOENT") {
  console.log("wasm-opt not found; keeping the unoptimized build.");
} else if (optimized.status !== 0) {
  process.exit(optimized.status ?? 1);
}

// The size of the module as shipped. The budget is what a Worker or a page would have to load;
// when a feature pushes the module over it, raise it on purpose in the same commit.
const budgetGzip = 600_000;
const shipped = readFileSync(output);
const gzipped = gzipSync(shipped, { level: 9 }).length;
const kilobytes = (bytes) => `${(bytes / 1000).toFixed(1)} KB`;
console.log(
  `chartlet.wasm: ${kilobytes(shipped.length)} (${shipped.length} B), gzip -9 ${kilobytes(gzipped)}`,
);
if (gzipped > budgetGzip) {
  console.error(`chartlet.wasm is over its budget of ${kilobytes(budgetGzip)} gzipped.`);
  process.exit(1);
}

// The shared stylesheet for `styles: "external"`, from the renderer just built.
const renderer = createRenderer(new WebAssembly.Module(readFileSync(output)));
writeFileSync(join(packageDirectory, "src/chartlet.css"), `${renderer.stylesheet()}\n`);
