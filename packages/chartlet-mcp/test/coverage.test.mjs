import assert from "node:assert/strict";
import test from "node:test";

import { checkCoverage } from "../src/coverage.mjs";

const flow = (nodes, edges = [], groups = []) => ({
  schemaVersion: 1,
  type: "flow",
  title: "Process",
  flow: { nodes, edges, groups },
});
const node = (id, label) => ({ id, label });
const edge = (from, to, label) => ({ from, to, ...(label ? { label } : {}) });

/** Every source element is in exactly one place of the ledger. */
function assertLedger(result) {
  const { counts } = result;
  assert.equal(
    counts.kept + counts.changed + counts.merged + counts.ambiguous + counts.missing,
    counts.source.total,
  );
  assert.equal(result.kept.length, counts.kept);
  assert.equal(result.missing.length, counts.missing);
}

test("matches nodes by label, ignoring case and whitespace, and reports missing and added", () => {
  const result = checkCoverage({
    spec: flow([node("a", "Receive  order"), node("b", "Ship"), node("c", "Archive")], [edge("a", "b")]),
    source: [
      { label: "receive order", kind: "node" },
      { label: " SHIP ", kind: "node" },
      { label: "Invoice", kind: "node" },
      { kind: "edge", from: "Receive order", to: "Ship" },
    ],
  });
  assertLedger(result);
  assert.deepEqual(result.kept.map(({ source }) => source.label ?? `${source.from}>${source.to}`), [
    "receive order",
    " SHIP ",
    "Receive order>Ship",
  ]);
  assert.deepEqual(result.missing.map(({ source }) => source.label), ["Invoice"]);
  assert.deepEqual(result.added.map(({ label }) => label), ["Archive"]);
  assert.match(result.report, /missing: node "Invoice"/);
  assert.equal(result.inventory, "agent-extracted");
});

test("a repeated label is ambiguous, not kept, until an id mapping settles it", () => {
  const spec = flow([node("w1", "Worker"), node("w2", "Worker"), node("q", "Queue")]);
  const source = [
    { id: "s1", label: "Worker", kind: "node" },
    { id: "s2", label: "Worker", kind: "node" },
    { id: "s3", label: "Queue", kind: "node" },
  ];
  const first = checkCoverage({ spec, source });
  assertLedger(first);
  assert.equal(first.counts.ambiguous, 2);
  assert.equal(first.counts.kept, 1);
  assert.equal(first.added.length, 0, "candidates of an ambiguity are not reported as added");
  assert.equal(first.ambiguous[0].candidates.length, 2);

  const second = checkCoverage({
    spec,
    source,
    mappings: [
      { source: "s1", to: "w1" },
      { source: "s2", to: "w2" },
    ],
  });
  assertLedger(second);
  assert.equal(second.counts.kept, 3);
  assert.equal(second.counts.ambiguous, 0);
});

test("a sourceLabel on the specification's element names the source", () => {
  const result = checkCoverage({
    spec: flow([{ id: "a", label: "Order received", sourceLabel: "Bestellung" }]),
    source: [{ label: "Bestellung", kind: "node" }],
  });
  assert.equal(result.counts.kept, 1);
});

test("edges match by endpoints and direction; a reversed edge is changed, a missing one is missing", () => {
  const spec = flow(
    [node("a", "A"), node("b", "B"), node("c", "C")],
    [edge("b", "a"), edge("a", "c", "yes")],
  );
  const result = checkCoverage({
    spec,
    source: [
      { label: "A", kind: "node" },
      { label: "B", kind: "node" },
      { label: "C", kind: "node" },
      { kind: "edge", from: "A", to: "B" },
      { kind: "edge", from: "A", to: "C", label: "no" },
      { kind: "edge", from: "B", to: "C" },
    ],
  });
  assertLedger(result);
  assert.equal(result.counts.kept, 3);
  assert.deepEqual(
    result.changed.map(({ reason }) => reason),
    ["the direction is reversed", 'the label differs: source "no", specification "yes"'],
  );
  assert.equal(result.missing.length, 1);
  assert.match(result.missing[0].reason, /no connection from "B" to "C"/);
});

test("parallel and opposite edges are matched one to one", () => {
  const spec = flow(
    [node("a", "A"), node("b", "B")],
    [edge("a", "b", "y"), edge("a", "b", "x"), edge("b", "a", "back")],
  );
  const result = checkCoverage({
    spec,
    source: [
      { id: "1", label: "A", kind: "node" },
      { id: "2", label: "B", kind: "node" },
      { kind: "edge", from: "1", to: "2", label: "x" },
      { kind: "edge", from: "1", to: "2", label: "y" },
      { kind: "edge", from: "2", to: "1", label: "back" },
    ],
  });
  assertLedger(result);
  assert.equal(result.counts.kept, 5);
  assert.equal(result.added.length, 0);
});

test("an edge whose endpoint is missing is missing; balanced marks it expected, faithful does not", () => {
  const input = {
    spec: flow([node("a", "A")]),
    source: [
      { label: "A", kind: "node" },
      { label: "B", kind: "node" },
      { kind: "edge", from: "A", to: "B" },
    ],
  };
  const balanced = checkCoverage({ ...input, detail: "balanced" });
  assert.equal(balanced.counts.missing, 2);
  const byKind = Object.fromEntries(balanced.missing.map((row) => [row.source.kind, row.expected]));
  assert.deepEqual(byKind, { node: false, edge: true });
  const faithful = checkCoverage({ ...input, detail: "faithful" });
  assert.ok(faithful.missing.every((row) => row.expected === false));
  const simplified = checkCoverage({ ...input, detail: "simplified" });
  assert.ok(simplified.missing.every((row) => row.expected === true));
  for (const result of [balanced, faithful, simplified]) {
    assert.equal(result.counts.missing, 2, "the detail hint never hides an omission");
    assert.equal(result.counts.kept, 1);
  }
});

test("groups keep their members; a changed membership and an explicit merge are not kept", () => {
  const spec = flow(
    [node("a", "A"), node("b", "B"), node("c", "C")],
    [],
    [
      { label: "Team", nodes: ["a"] },
      { label: "Ops", nodes: ["c"] },
    ],
  );
  const nodes = ["A", "B", "C"].map((label) => ({ label, kind: "node" }));
  const result = checkCoverage({
    spec,
    source: [...nodes, { kind: "group", label: "Team", members: ["A", "B"] }, { kind: "group", label: "Ops", members: ["C"] }],
  });
  assertLedger(result);
  assert.deepEqual(result.changed.map(({ source }) => source.label), ["Team"]);
  assert.match(result.changed[0].reason, /members not in the group: B/);
  assert.deepEqual(result.kept.filter(({ source }) => source.kind === "group").map(({ source }) => source.label), ["Ops"]);

  const merged = checkCoverage({
    spec: flow([node("a", "A")], [], [{ label: "Everything", nodes: ["a"] }]),
    source: [
      { label: "A", kind: "node" },
      { id: "g1", label: "Front", kind: "group", members: ["A"] },
      { id: "g2", label: "Back", kind: "group", members: [] },
    ],
    mappings: [
      { source: "g1", to: "/flow/groups/0" },
      { source: "g2", to: "/flow/groups/0" },
    ],
  });
  assertLedger(merged);
  assert.equal(merged.counts.merged, 2);
  assert.equal(merged.counts.kept, 1);
  assert.equal(merged.merged[0].sources.length, 2);
});

test("a merged node makes the edges at it changed", () => {
  const spec = flow([node("ab", "AB"), node("c", "C")], [edge("ab", "c")]);
  const result = checkCoverage({
    spec,
    source: [
      { id: "a", label: "A", kind: "node" },
      { id: "b", label: "B", kind: "node" },
      { label: "C", kind: "node" },
      { kind: "edge", from: "a", to: "C" },
    ],
    mappings: [
      { source: "a", to: "ab" },
      { source: "b", to: "ab" },
    ],
  });
  assertLedger(result);
  assert.equal(result.counts.merged, 2);
  assert.equal(result.counts.kept, 1);
  assert.equal(result.changed[0].reason, "an endpoint was merged with another source node");
});

test("other diagram types are read, unknown ones and bad input are refused", () => {
  const result = checkCoverage({
    spec: {
      type: "architecture",
      architecture: {
        components: [{ id: "api", label: "API", in: "sys" }],
        connections: [{ from: "api", to: "api", label: "retry" }],
        boundaries: [{ id: "sys", label: "System" }],
      },
    },
    source: [
      { label: "API", kind: "node" },
      { label: "System", kind: "group", members: ["API"] },
    ],
  });
  assert.equal(result.counts.kept, 2);
  assert.equal(result.added.length, 1);
  assert.throws(() => checkCoverage({ spec: { type: "bar" }, source: [] }), /not for type "bar"/);
  assert.throws(() => checkCoverage({ spec: flow([]), source: [{ kind: "node" }] }), /needs a label/);
  assert.throws(() => checkCoverage({ spec: flow([]), source: [], detail: "x" }), /detail must be/);
});

test("unused mappings are reported", () => {
  const result = checkCoverage({
    spec: flow([node("a", "A")]),
    source: [{ label: "A", kind: "node" }],
    mappings: [{ source: "nope", to: "a" }],
  });
  assert.equal(result.unusedMappings.length, 1);
});
