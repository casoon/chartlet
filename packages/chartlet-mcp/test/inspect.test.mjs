import assert from "node:assert/strict";
import test from "node:test";

import { DISTINCT_CAP, inspectData } from "../src/inspect.mjs";

const column = (summary, name) => summary.columns.find((entry) => entry.name === name);
const types = (summary) => summary.suggestions.map(({ type, mark }) => (mark ? `${type}/${mark}` : type));

test("infers types, missing values and ranges from CSV", () => {
  const summary = inspectData({
    csv: [
      "region,revenue,units,launched,active,note",
      "North,12.5,3,2026-01-05,true,",
      "South,-4,10,2026-02-01T08:30:00Z,false,a",
      "East,,7,2026-03-01,TRUE,b",
    ].join("\n"),
  });
  assert.equal(summary.rowCount, 3);
  assert.deepEqual(column(summary, "region"), { name: "region", type: "string", missing: 0, distinct: 3 });
  assert.deepEqual(column(summary, "revenue"), { name: "revenue", type: "number", missing: 1, min: -4, max: 12.5 });
  assert.deepEqual(column(summary, "units"), { name: "units", type: "integer", missing: 0, min: 3, max: 10 });
  assert.deepEqual(column(summary, "launched"), {
    name: "launched",
    type: "date-time",
    missing: 0,
    timeFormat: "date-time",
    first: "2026-01-05",
    last: "2026-03-01",
    monotonic: "increasing",
  });
  assert.deepEqual(column(summary, "active"), { name: "active", type: "boolean", missing: 0 });
  assert.equal(column(summary, "note").missing, 1);
});

test("does not read locale numbers, invalid dates or mixed columns as numbers or times", () => {
  const summary = inspectData({
    rows: [
      { amount: "1,234", day: "2026-02-30", mixed: 1 },
      { amount: "5", day: "2026-02-01", mixed: "x" },
    ],
  });
  assert.equal(column(summary, "amount").type, "string");
  assert.equal(column(summary, "day").type, "string");
  assert.equal(column(summary, "mixed").type, "string");
});

test("reads rows with typed values and absent keys as missing", () => {
  const summary = inspectData({ rows: [{ a: 1, b: null }, { a: 2.5 }, { b: "x", c: true }] });
  assert.deepEqual(
    summary.columns.map(({ name, type, missing }) => [name, type, missing]),
    [
      ["a", "number", 1],
      ["b", "string", 2],
      ["c", "boolean", 2],
    ],
  );
  assert.equal(summary.rowCount, 3);
});

test("treats four-digit integers as years only by name or strict increase", () => {
  const rising = inspectData({ csv: "t,v\n1990,1\n1991,2\n1993,3\n" });
  assert.equal(column(rising, "t").type, "date-time");
  assert.equal(column(rising, "t").timeFormat, "year");
  const named = inspectData({ rows: [{ Year: 2001, v: 1 }, { Year: 1999, v: 2 }] });
  assert.equal(column(named, "Year").monotonic, "decreasing");
  const prices = inspectData({ csv: "price\n1200\n1100\n1500\n" });
  assert.equal(column(prices, "price").type, "integer");
});

test("orders date-times with offsets in UTC and flags unsorted times", () => {
  const summary = inspectData({
    csv: "t,v\n2026-01-01T10:00:00+02:00,1\n2026-01-01T09:00:00Z,2\n2026-01-01T08:00:00Z,3\n",
  });
  assert.equal(column(summary, "t").monotonic, "none");
  const unsorted = inspectData({ csv: "t,v\n2026-01-03,1\n2026-01-01,2\n2026-01-02,3\n" });
  assert.match(unsorted.suggestions[0].reason, /Sort the rows by time/);
});

test("caps the distinct count of string columns", () => {
  const rows = Array.from({ length: DISTINCT_CAP + 5 }, (_, index) => ({ id: `id-${index}` }));
  assert.deepEqual(column(inspectData({ rows }), "id"), {
    name: "id",
    type: "string",
    missing: 0,
    distinct: DISTINCT_CAP,
    distinctCapped: true,
  });
});

test("suggests chart types by rule", () => {
  assert.deepEqual(types(inspectData({ csv: "region,revenue\nNorth,1\nSouth,2\n" })), ["bar"]);
  assert.deepEqual(types(inspectData({ csv: "day,orders,returns\n2026-01-01,1,2\n2027-01-02,3,4\n" })), [
    "time/line",
  ]);
  assert.deepEqual(
    types(inspectData({ csv: "Date,Open,High,Low,Close\n2026-01-01,1,3,0.5,2\n2026-01-02,2,4,1,3\n" })),
    ["time/ohlc"],
  );
  assert.deepEqual(
    types(inspectData({ csv: "team,low,mid,high\nA,1,2,3\nB,2,3,5\n" })),
    ["rangebar"],
  );
  assert.deepEqual(types(inspectData({ csv: "year,anomaly\n1850,-0.3\n1851,-0.2\n" })), [
    "time/line",
    "stripes",
  ]);
  assert.deepEqual(types(inspectData({ csv: "date,value\n2024-01-01,1\n2024-01-02,2\n" })), [
    "time/line",
    "calendar",
  ]);
  assert.deepEqual(
    types(inspectData({ csv: "t,v,lower,upper\n2026-01-01,1,0,2\n2026-01-02,2,1,3\n" })),
    ["time/line", "time/line", "calendar"],
  );
  assert.deepEqual(
    types(inspectData({ csv: "t,sector,v\n2026-01-01,A,1\n2026-01-01,B,2\n2026-01-02,A,3\n2026-01-02,B,4\n" })),
    ["time/line", "multiples"],
  );
});

test("explains limits in the suggestion reasons", () => {
  const bar = inspectData({ csv: "region,revenue\nNorth,1\nNorth,2\n" }).suggestions[0];
  assert.deepEqual(bar.columns, { label: "region", series: ["revenue"] });
  assert.match(bar.reason, /does not aggregate/);
  const gaps = inspectData({ csv: "year,v\n1850,1\n1852,2\n" }).suggestions.find(
    ({ type }) => type === "stripes",
  );
  assert.match(gaps.reason, /not consecutive/);
  const years = inspectData({ csv: "year,v\n1850,1\n1851,2\n" }).suggestions[0];
  assert.match(years.reason, /as strings such as "1990"/);
});

test("rejects input that is neither CSV nor rows", () => {
  assert.throws(() => inspectData({}), /exactly one of/);
  assert.throws(() => inspectData({ csv: "a", rows: [] }), /exactly one of/);
});

test("suggests the block types by the shape of the table", () => {
  const rows = (n, make) => Array.from({ length: n }, (_, i) => make(i));
  const csv = (header, lines) => [header, ...lines].join("\n") + "\n";
  const has = (summary, type) => summary.suggestions.some((suggestion) => suggestion.type === type);

  const points = inspectData({ csv: csv("x,y,class", rows(8, (i) => `${i},${i * 2},${i % 2 ? "up" : "down"}`)) });
  const scatter = points.suggestions.find(({ type }) => type === "scatter");
  assert.deepEqual(scatter.columns, { x: "x", y: "y", group: "class" });

  const times = inspectData({
    csv: csv("endpoint,ms", rows(20, (i) => `${i % 2 ? "search" : "checkout"},${40 + i}`)),
  });
  const box = times.suggestions.find(({ type }) => type === "boxplot");
  assert.deepEqual(box.columns, { label: "endpoint", values: "ms" });
  assert.match(box.reason, /violin/);

  const budget = inspectData({ csv: csv("item,amount", rows(8, (i) => `part${i},${10 + i}`)) });
  assert.ok(has(budget, "treemap"));
  assert.ok(!has(budget, "waterfall"));

  const bridge = inspectData({ csv: csv("step,change", ["Revenue,100", "Costs,-40", "Gain,10", "Tax,-5"]) });
  assert.ok(has(bridge, "waterfall"));

  const flows = inspectData({ csv: csv("source,target,amount", ["a,b,3", "a,c,2", "b,c,1"]) });
  assert.deepEqual(flows.suggestions.find(({ type }) => type === "sankey").columns, {
    from: "source",
    to: "target",
    value: "amount",
  });

  const trial = inspectData({
    csv: csv("months,event,arm", rows(8, (i) => `${i + 1},${i % 3 === 0 ? 0 : 1},${i % 2 ? "A" : "B"}`)),
  });
  assert.deepEqual(trial.suggestions.find(({ type }) => type === "survival").columns, {
    time: "months",
    event: "event",
    group: "arm",
  });

  const plan = inspectData({
    csv: csv("task,start,end", ["Research,2027-01-11,2027-02-19", "Build,2027-02-22,2027-05-28"]),
  });
  assert.deepEqual(plan.suggestions.find(({ type }) => type === "timeline").columns, {
    label: "task",
    start: "start",
    end: "end",
  });
});
