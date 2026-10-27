import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { renderSpec, validateSpec } from "../src/charts.mjs";
import { promptText } from "../src/guide.mjs";

const read = (path) => JSON.parse(readFileSync(fileURLToPath(new URL(path, import.meta.url)), "utf8"));
const { tasks } = read("../eval/tasks.json");
const reference = (task) => {
  const { example, spec, patch } = task.reference;
  const merged = { ...(example ? read(`../../../examples/${example}.json`) : spec), ...patch };
  // A null in a patch removes the field.
  return Object.fromEntries(Object.entries(merged).filter(([, value]) => value !== null));
};

test("the evaluation set covers chart, diagram, repair and change tasks", () => {
  const kinds = new Set(tasks.map((task) => task.kind));
  assert.deepEqual([...kinds].sort(), ["change", "chart", "diagram", "repair"]);
  assert.equal(new Set(tasks.map((task) => task.id)).size, tasks.length);
  for (const task of tasks) {
    assert.ok(promptText(task.prompt, task.request).includes(task.request), task.id);
  }
});

for (const task of tasks) {
  test(`reference solution of ${task.id} validates and renders`, () => {
    if (task.broken) {
      const broken = validateSpec(task.broken);
      assert.equal(broken.ok, false);
      assert.equal(broken.error.code, task.expectedError.code, JSON.stringify(broken.error));
    }
    const spec = reference(task);
    const checked = validateSpec(spec);
    assert.equal(checked.ok, true, JSON.stringify(checked.error));
    const result = renderSpec({ spec, format: "svg" });
    assert.equal(result.ok, true);
    if (task.expectedType) {
      assert.equal(spec.type, task.expectedType);
    }
    if (task.expectInSvg) {
      assert.ok(result.content.includes(task.expectInSvg), task.expectInSvg);
    }
  });
}
