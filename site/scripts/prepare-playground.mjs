// Copies the compiled WebAssembly renderer next to the playground page, so that GitHub Pages
// serves it from the same origin as the page: the browser makes no request to anyone else.
// The module is built from the Rust sources with `node packages/chartlet/scripts/build-wasm.mjs`
// (the Pages workflow does that). Without it the build still succeeds and the playground shows
// its "compiler could not be loaded" state, which is also what a visitor sees when it is missing.
import { copyFileSync, existsSync, mkdirSync, rmSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const site = join(dirname(fileURLToPath(import.meta.url)), "..");
const source = join(site, "../packages/chartlet/src/chartlet.wasm");
const target = join(site, "public/playground/chartlet.wasm");

if (existsSync(source)) {
  mkdirSync(dirname(target), { recursive: true });
  copyFileSync(source, target);
  console.log(`playground: copied chartlet.wasm (${statSync(target).size} bytes)`);
} else {
  rmSync(target, { force: true });
  console.warn(
    "playground: packages/chartlet/src/chartlet.wasm is missing, so the playground will show its load-failure state. Build it with `npm run build:wasm` in packages/chartlet.",
  );
}
