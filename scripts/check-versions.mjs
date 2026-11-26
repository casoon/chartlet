#!/usr/bin/env node
// Catches the drift between the facts that the README, the documentation and the site state and
// the metadata they come from: the crate, the two npm packages, the Python package and the Rust version. Run it
// before a release (see CONTRIBUTING.md); it changes nothing.
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";

const root = resolve(new URL("..", import.meta.url).pathname);
const read = (file) => readFileSync(join(root, file), "utf8");
const problems = [];

const cargo = read("Cargo.toml");
const crate = cargo.match(/^version = "([^"]+)"/m)?.[1];
const rust = cargo.match(/^rust-version = "([^"]+)"/m)?.[1];
const npm = JSON.parse(read("packages/chartlet/package.json")).version;
const mcp = JSON.parse(read("packages/chartlet-mcp/package.json")).version;
const python = read("python/pyproject.toml").match(/^version = "([^"]+)"/m)?.[1];

if (new Set([crate, npm, mcp, python]).size !== 1) {
  problems.push(`versions differ: crate ${crate}, @casoon/chartlet ${npm}, @casoon/chartlet-mcp ${mcp}, Python package ${python}`);
}
const minor = crate.split(".").slice(0, 2).join(".");

// Statements of the current minor or Rust version that are written by hand.
const expectations = [
  ["README.md", new RegExp(`Rust ${rust.replace(/\./g, "\\.")} or newer`), `Rust ${rust} or newer`],
  ["docs/getting-started/installation.mdx", new RegExp(`Rust ${rust.replace(/\./g, "\\.")} or newer`), `Rust ${rust} or newer`],
  ["docs/getting-started/installation.mdx", new RegExp(`chartlet = "${minor.replace(/\./g, "\\.")}"`), `chartlet = "${minor}"`],
  ["docs/reference/support.md", new RegExp(`\\^${minor.replace(/\./g, "\\.")}\\.0`), `^${minor}.0`],
  ["docs/reference/cli.md", new RegExp(`"chartlet": "${crate.replace(/\./g, "\\.")}"`), `"chartlet": "${crate}" in the manifest example`],
];
for (const [file, pattern, what] of expectations) {
  if (!pattern.test(read(file))) problems.push(`${file} does not state ${what}`);
}

// The entry points must name the same install paths: the CLI, both packages and the playground.
const entry = read("README.md");
for (const needle of ["cargo install chartlet", "npm install @casoon/chartlet", "pip install chartlet", "@casoon/chartlet-mcp", "casoon.github.io/chartlet/playground"]) {
  if (!entry.includes(needle)) problems.push(`README.md does not mention ${needle}`);
}

if (problems.length > 0) {
  console.error(problems.map((problem) => `- ${problem}`).join("\n"));
  process.exit(1);
}
console.log(`ok: chartlet ${crate}, Rust ${rust}, crate and the three packages agree, README and docs match.`);
