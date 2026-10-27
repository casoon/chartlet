import assert from "node:assert/strict";
import { test } from "node:test";

import { locate } from "../src/playground/pointer.mjs";

const text = `{
  "title": "A \\"quoted\\" title",
  "data": [
    { "label": "a", "value": 1 },
    { "label": "b", "value": 22.5 }
  ],
  "~key/x": true
}`;

const at = (pointer) => {
  const range = locate(text, pointer);
  return range && text.slice(range.start, range.end);
};

test("locates scalars, nested values and array elements", () => {
  assert.equal(at("/title"), '"A \\"quoted\\" title"');
  assert.equal(at("/data/1/value"), "22.5");
  assert.equal(at("/data/0"), '{ "label": "a", "value": 1 }');
  assert.equal(at("/~0key~1x"), "true");
});

test("a missing member names the nearest parent that exists", () => {
  assert.equal(at("/data/1/missing"), '{ "label": "b", "value": 22.5 }');
  assert.equal(at("/data/7/value"), text.slice(text.indexOf("["), text.indexOf("]") + 1));
});

test("says whether the value exists", () => {
  assert.equal(locate(text, "/data/0/value").exact, true);
  assert.equal(locate(text, "/data/0/nope").exact, false);
});

test("the root pointer is the whole document and broken text gives null", () => {
  assert.equal(at("/"), text);
  assert.equal(locate('{ "a": ', "/a"), null);
  assert.equal(locate("", "/a"), null);
});
