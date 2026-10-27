#!/usr/bin/env node
// Layout stability of an updated diagram: renders paired before/after specifications with the
// CLI and measures what moved. Prints a Markdown report.
//
//   node scripts/layout-stability.mjs --binary target/release/chartlet [--sorted]
//
// `--sorted` sorts the node declarations by id before every render, which emulates identity-based
// tie-breaking (the order of nodes would no longer be the author's) without changing the compiler.
//
// Measured per pair: how many nodes that the edit did not touch (neither edited nor connected to
// an edited node) moved by more than a pixel, how far, whether the relative order of unrelated
// node pairs flipped on either axis, the change in edge crossings and the change of the canvas.
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";

const root = resolve(new URL("..", import.meta.url).pathname);
const at = process.argv.indexOf("--binary");
const binary = resolve(at < 0 ? join(root, "target/release/chartlet") : process.argv[at + 1]);
const load = (name) => JSON.parse(readFileSync(join(root, "examples", `${name}.json`), "utf8"));
const clone = (value) => JSON.parse(JSON.stringify(value));

// ---- the edits ---------------------------------------------------------------------------------

const flowEdits = (insertAfter) => ({
  "reverse the order of the node declarations": (s) => {
    s.flow.nodes.reverse();
    return [];
  },
  "add a leaf": (s) => {
    s.flow.nodes.push({ id: "audit", label: "Audit log", ...(s.flow.lanes ? { lane: "ci" } : {}) });
    s.flow.edges.push({ from: insertAfter.leafFrom, to: "audit" });
    return ["audit", insertAfter.leafFrom];
  },
  "rename a node (longer label)": (s) => {
    s.flow.nodes.find((n) => n.id === insertAfter.rename).label = "A considerably longer label for this step";
    return [insertAfter.rename];
  },
  "add an edge": (s) => {
    s.flow.edges.push({ ...insertAfter.edge, label: "extra" });
    return [insertAfter.edge.from, insertAfter.edge.to];
  },
  "insert a node into an existing path": (s) => {
    const [from, to] = insertAfter.path;
    const index = s.flow.edges.findIndex((e) => e.from === from && e.to === to);
    const [old] = s.flow.edges.splice(index, 1);
    s.flow.nodes.push({ id: "scan", label: "Security scan", ...(s.flow.lanes ? { lane: "ci" } : {}) });
    s.flow.edges.push({ from, to: "scan", ...(old.label ? { label: old.label } : {}) }, { from: "scan", to });
    if (s.flow.mainPath) {
      const position = s.flow.mainPath.indexOf(from);
      if (position >= 0 && s.flow.mainPath[position + 1] === to) s.flow.mainPath.splice(position + 1, 0, "scan");
    }
    return ["scan", from, to];
  },
  "change a group": (s) => {
    if (s.flow.groups?.length) s.flow.groups[0].nodes = s.flow.groups[0].nodes.slice(0, -1);
    else s.flow.groups = [{ label: "Fulfilment", nodes: insertAfter.group }];
    return s.flow.groups[0].nodes.length ? insertAfter.group : [];
  },
});

const archEdits = {
  "reverse the order of the node declarations": (s) => {
    s.architecture.components.reverse();
    return [];
  },
  "add a leaf": (s) => {
    s.architecture.components.push({ id: "cache", label: "Cache", in: "netz" });
    s.architecture.connections.push({ from: "api", to: "cache" });
    return ["cache", "api"];
  },
  "rename a node (longer label)": (s) => {
    s.architecture.components.find((c) => c.id === "api").label = "Domain API with a considerably longer label";
    return ["api"];
  },
  "add an edge": (s) => {
    s.architecture.connections.push({ from: "idp", to: "db", label: "extra" });
    return ["idp", "db"];
  },
  "insert a node into an existing path": (s) => {
    const index = s.architecture.connections.findIndex((c) => c.from === "web" && c.to === "api");
    s.architecture.connections.splice(index, 1);
    s.architecture.components.push({ id: "gateway", label: "API gateway", in: "netz" });
    s.architecture.connections.push({ from: "web", to: "gateway" }, { from: "gateway", to: "api" });
    const path = s.architecture.mainPath;
    if (path) path.splice(path.indexOf("web") + 1, 0, "gateway");
    return ["gateway", "web", "api"];
  },
  "change a boundary": (s) => {
    s.architecture.components.find((c) => c.id === "idp").in = "rz";
    return ["idp"];
  },
};

const subjects = [
  {
    name: "release-flow (flow, 3 lanes, 1 group)",
    spec: load("release-flow"),
    edits: flowEdits({
      leafFrom: "tests",
      rename: "review",
      edge: { from: "tests", to: "prod" },
      path: ["build", "tests"],
      group: ["build", "tests", "green"],
    }),
    nodes: (s) => s.flow.nodes.map((n) => n.id),
    links: (s) => s.flow.edges.map((e) => [e.from, e.to]),
  },
  {
    name: "order-flow (flow, no lanes)",
    spec: load("order-flow"),
    edits: flowEdits({
      leafFrom: "ship",
      rename: "paid",
      edge: { from: "reserve", to: "cancelled" },
      path: ["paid", "reserve"],
      group: ["reserve", "stock", "available"],
    }),
    nodes: (s) => s.flow.nodes.map((n) => n.id),
    links: (s) => s.flow.edges.map((e) => [e.from, e.to]),
  },
  {
    name: "order-flow with a mainPath",
    spec: { ...load("order-flow"), flow: { ...load("order-flow").flow, mainPath: ["received", "paid", "reserve", "available", "ship", "delivered"] } },
    edits: flowEdits({
      leafFrom: "ship",
      rename: "paid",
      edge: { from: "reserve", to: "cancelled" },
      path: ["paid", "reserve"],
      group: ["reserve", "stock", "available"],
    }),
    nodes: (s) => s.flow.nodes.map((n) => n.id),
    links: (s) => s.flow.edges.map((e) => [e.from, e.to]),
  },
  {
    name: "tenant-architecture (architecture, 3 nested boundaries)",
    spec: load("tenant-architecture"),
    edits: archEdits,
    nodes: (s) => s.architecture.components.map((c) => c.id),
    links: (s) => s.architecture.connections.map((c) => [c.from, c.to]),
  },
];

// ---- measuring ---------------------------------------------------------------------------------

const sorted = process.argv.includes("--sorted");
function render(input) {
  const spec = clone(input);
  if (sorted) {
    const byId = (a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0);
    spec.flow?.nodes.sort(byId);
    spec.architecture?.components.sort(byId);
  }
  const result = spawnSync(binary, ["render", "-", "--format", "svg", "--hooks"], {
    input: JSON.stringify(spec),
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.status !== 0) throw new Error(result.stderr.split("\n")[0]);
  return result.stdout;
}

function geometry(svg) {
  const box = svg.match(/viewBox="0 0 ([\d.]+) ([\d.]+)"/);
  const nodes = new Map();
  for (const match of svg.matchAll(/<g [^>]*data-chartlet-node="([^"]+)"[^>]*>(.*?)<\/g>/gs)) {
    const shapes = [...match[2].matchAll(/<(polyline|rect)[^>]*class="([^"]*)"[^>]*>/g)].filter(
      (shape) => !shape[2].includes("shadow") && shape[2].includes("chartlet-flow-node"),
    );
    const points = shapes.flatMap((shape) => {
      const text = shape[0];
      const list = text.match(/points="([^"]+)"/)?.[1];
      if (list) return list.split(" ").map((pair) => pair.split(",").map(Number));
      const [x, y, w, h] = ["x", "y", "width", "height"].map((name) => Number(text.match(new RegExp(` ${name}="([^"]+)"`))?.[1]));
      return [[x, y], [x + w, y + h]];
    });
    if (points.length === 0) continue;
    const xs = points.map((p) => p[0]);
    const ys = points.map((p) => p[1]);
    nodes.set(match[1], { x: (Math.min(...xs) + Math.max(...xs)) / 2, y: (Math.min(...ys) + Math.max(...ys)) / 2 });
  }
  const edges = [...svg.matchAll(/<polyline points="([^"]+)" class="chartlet-flow-edge[^"]*"/g)].map((m) =>
    m[1].split(" ").map((pair) => pair.split(",").map(Number)),
  );
  return { width: Number(box[1]), height: Number(box[2]), nodes, edges };
}

function crossings(edges) {
  const segments = edges.map((points) => points.slice(1).map((p, i) => [points[i], p]));
  const cross = ([a, b], [c, d]) => {
    const side = (p, q, r) => Math.sign((q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]));
    const near = (p, q) => Math.hypot(p[0] - q[0], p[1] - q[1]) < 1.5;
    if ([a, b].some((p) => [c, d].some((q) => near(p, q)))) return false;
    return side(a, b, c) * side(a, b, d) < 0 && side(c, d, a) * side(c, d, b) < 0;
  };
  let count = 0;
  for (let i = 0; i < segments.length; i += 1)
    for (let j = i + 1; j < segments.length; j += 1)
      for (const s of segments[i]) for (const t of segments[j]) if (cross(s, t)) count += 1;
  return count;
}

function compare(before, after, touched) {
  const common = [...before.nodes.keys()].filter((id) => after.nodes.has(id));
  const unrelated = common.filter((id) => !touched.has(id));
  const moves = unrelated.map((id) => {
    const a = before.nodes.get(id);
    const b = after.nodes.get(id);
    return Math.hypot(a.x - b.x, a.y - b.y);
  });
  let flipsX = 0;
  let flipsY = 0;
  let pairs = 0;
  for (let i = 0; i < unrelated.length; i += 1) {
    for (let j = i + 1; j < unrelated.length; j += 1) {
      const [a, b] = [unrelated[i], unrelated[j]];
      pairs += 1;
      const sign = (g, axis) => Math.sign(g.nodes.get(a)[axis] - g.nodes.get(b)[axis]);
      // A tie of a pair on an axis is an equal rank; going from tied to untied is not a flip.
      if (sign(before, "x") * sign(after, "x") < 0) flipsX += 1;
      if (sign(before, "y") * sign(after, "y") < 0) flipsY += 1;
    }
  }
  return {
    unrelated: unrelated.length,
    moved: moves.filter((m) => m > 1).length,
    mean: moves.length ? moves.reduce((s, m) => s + m, 0) / moves.length : 0,
    max: Math.max(0, ...moves),
    flips: flipsX + flipsY,
    pairs,
    crossings: crossings(after.edges) - crossings(before.edges),
    crossingsAfter: crossings(after.edges),
    dw: after.width - before.width,
    dh: after.height - before.height,
  };
}

const lines = [
  "| Diagram | Edit | Unrelated nodes moved | Mean / max move | Order flips (pairs) | Crossings (after) | Canvas change |",
  "| --- | --- | ---: | ---: | ---: | ---: | ---: |",
];
for (const subject of subjects) {
  const before = geometry(render(subject.spec));
  for (const [name, edit] of Object.entries(subject.edits)) {
    const spec = clone(subject.spec);
    const edited = new Set(edit(spec));
    // Neighbours of an edited node are part of the edit: their edges changed.
    const touched = new Set(edited);
    for (const [from, to] of subject.links(spec)) {
      if (edited.has(from)) touched.add(to);
      if (edited.has(to)) touched.add(from);
    }
    let row;
    try {
      const after = geometry(render(spec));
      const result = compare(before, after, touched);
      const crossingText = `${result.crossings >= 0 ? "+" : ""}${result.crossings} (${result.crossingsAfter})`;
      row = `| ${subject.name} | ${name} | ${result.moved} of ${result.unrelated} | ${result.mean.toFixed(0)} / ${result.max.toFixed(0)} px | ${result.flips} (${result.pairs}) | ${crossingText} | ${result.dw >= 0 ? "+" : ""}${result.dw.toFixed(0)} x ${result.dh >= 0 ? "+" : ""}${result.dh.toFixed(0)} px |`;
    } catch (error) {
      row = `| ${subject.name} | ${name} | failed: ${error.message} | | | | |`;
    }
    lines.push(row);
  }
}
console.log(lines.join("\n"));
