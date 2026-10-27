import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { BUNDLED, contrast, MARK, parseColor, PROJECT_THEME, readProjectTheme, TEXT, themeFromBrand, writeThemeFiles } from "../src/theme.mjs";

const brand = { paper: "#faf6ef", ink: "#2b2a28", muted: "#8a8680", accent: "#e8590c", link: "#1c7ed6" };
const schema = JSON.parse(readFileSync(new URL("../../../schema/chartlet-theme.schema.json", import.meta.url), "utf8"));
const names = new RegExp(schema.properties.tokens.propertyNames.pattern);
const values = new RegExp(schema.properties.tokens.additionalProperties.pattern);

const ratio = (tokens, a, b) => contrast(parseColor(tokens[a]), parseColor(tokens[b]));

/** The pairs the compiler measures: text, marks and the roles of a diagram. */
function assertMeasuredPairs(tokens) {
  assert.ok(ratio(tokens, "text", "background") >= TEXT, "text");
  assert.ok(ratio(tokens, "muted", "background") >= TEXT, "muted");
  for (const mark of ["accent", "color-1", "color-2", "color-3", "color-4", "rise", "fall"]) {
    assert.ok(ratio(tokens, mark, "background") >= MARK, mark);
  }
  for (const role of ["blue", "green", "amber", "teal", "violet", "gray", "red"]) {
    assert.ok(ratio(tokens, `role-${role}`, "background") >= MARK, `${role} on background`);
    assert.ok(ratio(tokens, `role-${role}`, `role-${role}-fill`) >= MARK, `${role} on fill`);
    assert.ok(ratio(tokens, "text", `role-${role}-fill`) >= TEXT, `text on ${role} fill`);
  }
}

test("a brand becomes a light and a dark theme that pass every measured pair", () => {
  const { files } = themeFromBrand(brand);
  assert.equal(files.light.theme, "light");
  assert.equal(files.dark.theme, "dark");
  assertMeasuredPairs(files.light.tokens);
  assertMeasuredPairs(files.dark.tokens);
  assert.equal(files.light.tokens.background, "#faf6ef");
  assert.equal(files.light.tokens.accent, "#e8590c", "a value that passes is not touched");
  assert.equal(files.light.tokens["role-blue"], "#1c7ed6", "the link color is the blue role");
});

test("the files are valid against the theme schema", () => {
  const { files } = themeFromBrand(brand);
  for (const file of [files.light, files.dark]) {
    assert.deepEqual(Object.keys(file).sort(), ["theme", "tokens"]);
    for (const [name, value] of Object.entries(file.tokens)) {
      assert.match(name, names);
      assert.match(value, values);
    }
  }
});

test("the receipt keeps the original-to-proposed mapping and names every adjustment with its pair", () => {
  const { receipt } = themeFromBrand(brand);
  const muted = receipt.roles.find((entry) => entry.scheme === "light" && entry.token === "muted");
  assert.equal(muted.input, "#8a8680");
  assert.notEqual(muted.output, muted.input);
  assert.equal(muted.adjusted, true);
  assert.match(muted.reason, /text 4\.5:1/);
  assert.match(muted.reason, /measured 3\.36:1/);
  assert.ok(muted.ratio >= TEXT);
  assert.ok(receipt.adjusted.includes(muted));
  const accent = receipt.roles.find((entry) => entry.scheme === "light" && entry.token === "accent");
  assert.equal(accent.adjusted, false);
  assert.equal(accent.origin, "sampled");
  const paper = receipt.roles.find((entry) => entry.scheme === "light" && entry.role === "paper");
  assert.equal(paper.input, "#faf6ef");
  assert.ok(receipt.adjusted.every((entry) => entry.reason && entry.pair && entry.input !== entry.output));
});

test("the dark values are derived and marked so unless the brand gives its own", () => {
  const derived = themeFromBrand(brand);
  assert.equal(derived.darkDerived, true);
  const dark = derived.receipt.roles.filter((entry) => entry.scheme === "dark" && ["background", "text", "accent"].includes(entry.token));
  assert.ok(dark.every((entry) => entry.origin === "derived"));
  assert.ok(derived.receipt.notes.some((note) => /derived/.test(note)));

  const given = themeFromBrand({ ...brand, dark: { paper: "#101820", ink: "#f2f2f2", accent: "#ff7a2f" } });
  assert.equal(given.darkDerived, false);
  assert.equal(given.files.dark.tokens.background, "#101820");
  assert.equal(given.files.dark.tokens.accent, "#ff7a2f");
  assertMeasuredPairs(given.files.dark.tokens);
});

test("a failing brand is adjusted, a dark paper with weak ink is repaired", () => {
  const { files, receipt } = themeFromBrand({ paper: "#ffffff", ink: "#999999", accent: "#ffee00" });
  assertMeasuredPairs(files.light.tokens);
  assertMeasuredPairs(files.dark.tokens);
  const text = receipt.roles.find((entry) => entry.scheme === "light" && entry.token === "text");
  assert.equal(text.input, "#999999");
  assert.equal(text.adjusted, true);
  const accent = receipt.roles.find((entry) => entry.scheme === "light" && entry.token === "accent");
  assert.equal(accent.adjusted, true);
  assert.match(accent.reason, /marks 3:1|needs 3:1/);
});

test("fonts are recorded and not written, with the reason", () => {
  const { files, receipt } = themeFromBrand({ ...brand, fonts: { names: "Fraunces", technical: "JetBrains Mono" } });
  assert.deepEqual(
    receipt.fonts.map(({ role, family, applied, metricsProfile }) => [role, family, applied, metricsProfile]),
    [
      ["names", "Fraunces", false, false],
      ["technical", "JetBrains Mono", false, false],
    ],
  );
  assert.match(receipt.fonts[0].reason, /no font setting/);
  assert.ok(!JSON.stringify(files).includes("Fraunces"));
  assert.deepEqual(themeFromBrand(brand).receipt.fonts, []);
});

test("bad colors are named", () => {
  assert.throws(() => themeFromBrand({ ...brand, paper: "white" }), /paper must be a hex color/);
  assert.throws(() => themeFromBrand({ ...brand, dark: { paper: "#000", ink: "#fff", accent: "nope" } }), /dark\.accent/);
});

test("the bundled values it builds on mirror the bundled themes", () => {
  for (const scheme of ["light", "dark"]) {
    const bundled = JSON.parse(readFileSync(new URL(`../../../themes/${scheme}.json`, import.meta.url), "utf8")).tokens;
    for (const [token, value] of Object.entries(BUNDLED[scheme])) {
      assert.equal(bundled[token], value, `${scheme} ${token}`);
    }
  }
});

test("theme files are written below the root, the dark one beside the light one, and never replaced by accident", () => {
  const root = mkdtempSync(join(tmpdir(), "chartlet-theme-write-"));
  const resolve = (path, base) => ({ root: base, path: join(base, path) });
  try {
    const { files } = themeFromBrand(brand);
    const written = writeThemeFiles(files, "chartlet.theme.json", false, root, resolve);
    assert.deepEqual(written.map(({ path }) => path), ["chartlet.theme.json", "chartlet.theme.dark.json"]);
    const light = JSON.parse(readFileSync(join(root, "chartlet.theme.json"), "utf8"));
    assert.equal(light.theme, "light");
    assert.deepEqual(light.tokens, files.light.tokens);
    assert.equal(readProjectTheme(root).theme.tokens.background, "#faf6ef");

    assert.throws(() => writeThemeFiles(files, "chartlet.theme.json", false, root, resolve), /already exist.*overwrite: true/);
    assert.equal(writeThemeFiles(files, "chartlet.theme.json", true, root, resolve).length, 2);
    assert.equal(writeThemeFiles(files, "themes/acme.json", false, root, resolve)[1].path, join("themes", "acme.dark.json"));
    assert.throws(() => writeThemeFiles(files, "theme.txt", false, root, resolve), /must end in \.json/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("a project theme is read from the root, absent or refused otherwise", () => {
  const root = mkdtempSync(join(tmpdir(), "chartlet-theme-"));
  try {
    assert.equal(readProjectTheme(undefined), undefined);
    assert.equal(readProjectTheme(join(root, "missing")), undefined);
    assert.equal(readProjectTheme(root), undefined);

    writeFileSync(join(root, PROJECT_THEME), JSON.stringify({ $schema: "x", theme: "dark", tokens: { accent: "#123456" } }));
    assert.deepEqual(readProjectTheme(root), {
      theme: { theme: "dark", tokens: { accent: "#123456" } },
      source: PROJECT_THEME,
    });

    writeFileSync(join(root, PROJECT_THEME), "{ nope");
    assert.throws(() => readProjectTheme(root), /not valid JSON/);
    writeFileSync(join(root, PROJECT_THEME), "[1]");
    assert.throws(() => readProjectTheme(root), /must be a JSON object/);

    rmSync(join(root, PROJECT_THEME));
    writeFileSync(join(root, "elsewhere.json"), "{}");
    symlinkSync(join(root, "elsewhere.json"), join(root, PROJECT_THEME));
    assert.throws(() => readProjectTheme(root), /symbolic link/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
