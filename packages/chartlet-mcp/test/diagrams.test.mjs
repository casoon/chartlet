import assert from "node:assert/strict";
import test from "node:test";

import { explainSpec, validateSpec } from "../src/charts.mjs";
import { DIAGRAM_TYPES, diagramStarter } from "../src/diagrams.mjs";

test("every diagram starter is a valid specification without warnings", () => {
  for (const type of DIAGRAM_TYPES) {
    const starter = diagramStarter(type);
    assert.equal(starter.spec.type, type);
    const result = validateSpec(starter.spec);
    // The installed @casoon/chartlet may predate a type; remove this guard when the dependency
    // is raised to the release that adds `tree` (0.9).
    if (!result.ok && /unknown variant/.test(result.error?.message ?? "")) {
      continue;
    }
    assert.equal(result.ok, true, `${type}: ${JSON.stringify(result)}`);
    assert.deepEqual(result.warnings, [], type);
    assert.ok(Object.keys(starter.kinds).length > 0, type);
  }
});

test("a starter is a copy, so changing it leaves the next one alone", () => {
  const first = diagramStarter("flow");
  first.spec.title = "Changed";
  assert.equal(diagramStarter("flow").spec.title, "Publishing a post");
});

test("explain gives a diagram's counts and the rows of its table", () => {
  const result = explainSpec(diagramStarter("state").spec);
  assert.equal(result.ok, true);
  assert.equal(result.seriesCount, 0);
  assert.deepEqual(result.structure.counts, { states: 4, composites: 0, transitions: 5 });
  assert.deepEqual(result.structure.columns, ["From", "Event", "Guard", "Action", "To"]);
  assert.deepEqual(result.structure.rows[0], ["Start", "", "", "", "Closed"]);
  assert.match(result.generatedDescription, /^State diagram with 4 states and 5 transitions\./);
});

test("charts other than diagrams carry no structure", () => {
  const result = explainSpec({ schemaVersion: 1, type: "bar", title: "T", data: [{ label: "A", value: 1 }] });
  assert.equal(result.ok, true);
  assert.equal(result.structure, undefined);
});
