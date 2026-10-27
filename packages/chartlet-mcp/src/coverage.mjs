// Coverage of a redrawn diagram: what of the source survived in the specification. chartlet reads
// no foreign formats; the assistant passes the inventory it read from the source. Everything here
// is a comparison of two lists: no network, no model, no rendering.
//
// Identity is never a label alone. A label that occurs more than once on either side is reported
// as ambiguous unless an explicit mapping or a `sourceId`/`sourceLabel` on the specification's
// element settles it. Edges are matched by their endpoints and direction, groups by label and
// membership. Every source element lands in exactly one of kept, changed, merged, ambiguous and
// missing; every specification element in kept, changed, merged, ambiguous (as a candidate) or
// added. The detail hint only annotates the missing elements; it never removes one from the list.

export const DETAILS = ["faithful", "balanced", "simplified"];
export const KINDS = ["node", "edge", "group"];

/**
 * @typedef {{ id?: string, label?: string, kind: "node" | "edge" | "group", from?: string, to?: string, directed?: boolean, members?: string[] }} SourceElement
 * @typedef {{ source: string, to: string, kind?: "node" | "edge" | "group" }} Mapping
 * @typedef {{ kind: "node" | "edge" | "group", ref: string, id?: string, label: string, from?: string, to?: string, members?: string[], key: string }} SpecElement
 */

/** @param {unknown} label */
export function normalize(label) {
  return String(label ?? "")
    .normalize("NFKC")
    .toLowerCase()
    .replace(/\s+/g, " ")
    .trim();
}

/** @param {unknown} value @returns {Record<string, any>[]} */
const list = (value) => (Array.isArray(value) ? value : []);

/**
 * The nodes, edges and groups of a specification, each with the JSON Pointer of where it is.
 * `sourceLabel` and `sourceId` on an element are read as the source's name for it; they are not
 * part of the schema, so remove them before rendering.
 *
 * @param {Record<string, any>} spec
 * @returns {SpecElement[]}
 */
export function specInventory(spec) {
  /** @type {SpecElement[]} */
  const out = [];
  /** @param {"node" | "edge" | "group"} kind @param {string} ref @param {Record<string, any>} item @param {string} label @param {Partial<SpecElement>} [extra] */
  const add = (kind, ref, item, label, extra = {}) =>
    out.push({
      kind,
      ref,
      id: typeof item.id === "string" ? item.id : undefined,
      label,
      key: normalize(item.sourceLabel ?? label),
      ...(typeof item.sourceId === "string" ? { sourceId: item.sourceId } : {}),
      ...extra,
    });
  const nodes = (block, path, field, labelOf = (/** @type {any} */ item) => item.label) =>
    list(block?.[field]).forEach((item, index) => add("node", `/${path}/${field}/${index}`, item, labelOf(item)));
  const edges = (block, path, field, labelOf = (/** @type {any} */ item) => item.label ?? "") =>
    list(block?.[field]).forEach((item, index) =>
      add("edge", `/${path}/${field}/${index}`, item, labelOf(item), { from: item.from, to: item.to }),
    );

  switch (spec.type) {
    case "flow": {
      const flow = spec.flow;
      nodes(flow, "flow", "nodes");
      edges(flow, "flow", "edges");
      list(flow?.groups).forEach((group, index) =>
        add("group", `/flow/groups/${index}`, group, group.label, { members: list(group.nodes) }),
      );
      list(flow?.lanes).forEach((lane, index) =>
        add("group", `/flow/lanes/${index}`, lane, lane.label, {
          members: list(flow.nodes)
            .filter((node) => node.lane === lane.id)
            .map((node) => node.id),
        }),
      );
      break;
    }
    case "state": {
      const state = spec.state;
      nodes(state, "state", "states");
      edges(state, "state", "transitions", (t) =>
        [t.event, t.guard ? `[${t.guard}]` : "", t.action ? `/ ${t.action}` : ""].filter(Boolean).join(" "),
      );
      break;
    }
    case "architecture": {
      const architecture = spec.architecture;
      nodes(architecture, "architecture", "components");
      edges(architecture, "architecture", "connections");
      list(architecture?.boundaries).forEach((boundary, index) =>
        add("group", `/architecture/boundaries/${index}`, boundary, boundary.label, {
          members: list(architecture.components)
            .filter((component) => component.in === boundary.id)
            .map((component) => component.id),
        }),
      );
      break;
    }
    case "classes": {
      const classes = spec.classes;
      nodes(classes, "classes", "classes");
      edges(classes, "classes", "relations");
      list(classes?.packages).forEach((pack, index) =>
        add("group", `/classes/packages/${index}`, pack, pack.label, {
          members: list(classes.classes)
            .filter((item) => item.package === pack.id)
            .map((item) => item.id),
        }),
      );
      break;
    }
    case "tree":
      list(spec.tree?.nodes).forEach((node, index) => {
        add("node", `/tree/nodes/${index}`, node, node.label);
        if (typeof node.parent === "string") {
          add("edge", `/tree/nodes/${index}/parent`, {}, "", { from: node.parent, to: node.id });
        }
      });
      break;
    case "network":
      nodes(spec.network, "network", "nodes");
      edges(spec.network, "network", "edges");
      break;
    case "sequence":
      nodes(spec.sequence, "sequence", "participants");
      edges(spec.sequence, "sequence", "messages");
      list(spec.sequence?.fragments).forEach((fragment, index) =>
        add("group", `/sequence/fragments/${index}`, fragment, fragment.label ?? fragment.kind ?? "fragment"),
      );
      break;
    default:
      throw new Error(
        `Coverage is available for flow, state, architecture, classes, tree, network and sequence diagrams, not for type ${JSON.stringify(spec.type)}.`,
      );
  }
  return out;
}

/** @param {SourceElement} element */
const nameOf = (element) => element.label ?? [element.from, element.to].filter(Boolean).join(" -> ");

/**
 * @param {SourceElement} element
 * @param {number} index
 */
function sourceView(element, index) {
  return {
    index,
    kind: element.kind,
    ...(element.id === undefined ? {} : { id: element.id }),
    ...(element.label === undefined ? {} : { label: element.label }),
    ...(element.from === undefined ? {} : { from: element.from, to: element.to }),
  };
}

/** @param {SpecElement} element */
function specView(element) {
  return {
    kind: element.kind,
    ref: element.ref,
    ...(element.id === undefined ? {} : { id: element.id }),
    ...(element.label === "" ? {} : { label: element.label }),
    ...(element.from === undefined ? {} : { from: element.from, to: element.to }),
  };
}

/**
 * Compares the source inventory with the specification.
 *
 * @param {{
 *   spec: Record<string, unknown> | string,
 *   source: SourceElement[],
 *   mappings?: Mapping[],
 *   detail?: "faithful" | "balanced" | "simplified",
 *   inventory?: string,
 * }} input
 */
export function checkCoverage({ spec, source, mappings = [], detail = "balanced", inventory = "agent-extracted" }) {
  const parsed = typeof spec === "string" ? JSON.parse(spec) : spec;
  if (!DETAILS.includes(detail)) {
    throw new Error(`detail must be one of ${DETAILS.join(", ")}.`);
  }
  source.forEach((element, index) => {
    if (!KINDS.includes(element.kind)) {
      throw new Error(`source[${index}].kind must be node, edge or group.`);
    }
    if (element.kind !== "edge" && !normalize(element.label)) {
      throw new Error(`source[${index}] is a ${element.kind} and needs a label.`);
    }
    if (element.kind === "edge" && !normalize(element.label) && (!element.from || !element.to)) {
      throw new Error(`source[${index}] is an edge and needs from and to (source ids or labels), or a label.`);
    }
  });
  const items = specInventory(parsed);

  /** @type {Map<number, number[]>} spec index -> source indices that claim it */
  const claims = new Map();
  /** @type {Map<number, string>} source index -> how it was matched */
  const matchedBy = new Map();
  /** @type {Map<number, string>} source index -> reason it changed */
  const changed = new Map();
  /** @type {Map<number, { candidates: number[], reason: string }>} */
  const ambiguous = new Map();
  /** @type {Map<number, string>} */
  const missing = new Map();
  /** @param {number} s @param {number} p @param {string} how */
  const claim = (s, p, how) => {
    claims.set(p, [...(claims.get(p) ?? []), s]);
    matchedBy.set(s, how);
  };
  const claimed = (/** @type {number} */ p) => claims.has(p);
  const open = (/** @type {number} */ s) => !matchedBy.has(s) && !ambiguous.has(s) && !missing.has(s);

  // Explicit mappings, by source id or label to the id or pointer of a specification element.
  /** @type {Mapping[]} */
  const unused = [];
  for (const mapping of mappings) {
    const wanted = normalize(mapping.source);
    const sources = source
      .map((element, index) => ({ element, index }))
      .filter(
        ({ element, index }) =>
          open(index) &&
          (mapping.kind === undefined || mapping.kind === element.kind) &&
          (element.id === mapping.source || (element.id === undefined && normalize(element.label) === wanted)),
      );
    const target = items.findIndex((item) => item.id === mapping.to || item.ref === mapping.to);
    const fits = sources.filter(({ element }) => items[target]?.kind === element.kind);
    if (target < 0 || fits.length === 0) {
      unused.push(mapping);
    } else {
      for (const { index } of fits) {
        claim(index, target, "mapping");
      }
    }
  }
  // sourceId on a specification element names the source element by id.
  items.forEach((item, p) => {
    const sourceId = /** @type {any} */ (item).sourceId;
    if (sourceId === undefined || claimed(p)) {
      return;
    }
    const s = source.findIndex((element, index) => open(index) && element.id === sourceId && element.kind === item.kind);
    if (s >= 0) {
      claim(s, p, "sourceId");
    }
  });

  // Nodes and groups, by label; a label that does not name exactly one on each side is ambiguous.
  for (const kind of /** @type {const} */ (["node", "group"])) {
    /** @type {Map<string, { sources: number[], specs: number[] }>} */
    const byKey = new Map();
    const slot = (/** @type {string} */ key) => {
      if (!byKey.has(key)) {
        byKey.set(key, { sources: [], specs: [] });
      }
      return /** @type {{ sources: number[], specs: number[] }} */ (byKey.get(key));
    };
    source.forEach((element, s) => {
      if (element.kind === kind && open(s)) {
        slot(normalize(element.label)).sources.push(s);
      }
    });
    items.forEach((item, p) => {
      if (item.kind === kind && !claimed(p)) {
        slot(item.key).specs.push(p);
      }
    });
    for (const { sources, specs } of byKey.values()) {
      if (sources.length === 0) {
        continue;
      }
      if (specs.length === 0) {
        sources.forEach((s) => missing.set(s, `no ${kind} labelled ${JSON.stringify(source[s].label)} in the specification`));
      } else if (sources.length === 1 && specs.length === 1) {
        claim(sources[0], specs[0], "label");
      } else {
        const reason = `the label ${JSON.stringify(source[sources[0]].label)} names ${sources.length} source ${kind}(s) and ${specs.length} specification ${kind}(s); map them with an id or mapping`;
        sources.forEach((s) => ambiguous.set(s, { candidates: specs, reason }));
      }
    }
  }

  // The specification node each source node stands for, and which spec nodes took several.
  /** @type {Map<number, number>} */
  const nodeOf = new Map();
  for (const [p, claimers] of claims) {
    if (items[p].kind === "node") {
      claimers.forEach((s) => nodeOf.set(s, p));
    }
  }
  const mergedNode = (/** @type {number} */ p) => (claims.get(p)?.length ?? 0) > 1;
  /** Resolves a source reference (id, else a unique label) to a source node index, -1 unknown, -2 ambiguous. */
  const resolve = (/** @type {string} */ ref) => {
    const byId = source.findIndex((element) => element.kind === "node" && element.id === ref);
    if (byId >= 0) {
      return byId;
    }
    const hits = source.flatMap((element, s) => (element.kind === "node" && normalize(element.label) === normalize(ref) ? [s] : []));
    return hits.length === 1 ? hits[0] : hits.length === 0 ? -1 : -2;
  };

  // Edges, in three passes: same endpoints and label, same endpoints, opposite direction.
  /** @type {Map<number, { from: number, to: number }>} resolved source edges */
  const ends = new Map();
  source.forEach((element, s) => {
    if (element.kind !== "edge" || !open(s)) {
      return;
    }
    if (!element.from || !element.to) {
      const hits = items.flatMap((item, p) => (item.kind === "edge" && !claimed(p) && item.key === normalize(element.label) ? [p] : []));
      if (hits.length === 1) {
        claim(s, hits[0], "label");
      } else if (hits.length === 0) {
        missing.set(s, `no connection labelled ${JSON.stringify(element.label)} in the specification`);
      } else {
        ambiguous.set(s, { candidates: hits, reason: `the label ${JSON.stringify(element.label)} names ${hits.length} connections; give from and to` });
      }
      return;
    }
    const [from, to] = [resolve(element.from), resolve(element.to)];
    if (from === -2 || to === -2) {
      ambiguous.set(s, { candidates: [], reason: "an endpoint label names several source nodes; use source ids in from and to" });
    } else if (from < 0 || to < 0) {
      missing.set(s, `an endpoint (${JSON.stringify(from < 0 ? element.from : element.to)}) is not a node of the source inventory`);
    } else if (!nodeOf.has(from) || !nodeOf.has(to)) {
      const lost = nodeOf.has(from) ? to : from;
      missing.set(s, `the endpoint ${JSON.stringify(nameOf(source[lost]))} is not in the specification (${ambiguous.has(lost) ? "ambiguous" : "missing"})`);
    } else {
      ends.set(s, { from: nodeOf.get(from) ?? -1, to: nodeOf.get(to) ?? -1 });
    }
  });
  const nodeIndex = (/** @type {string | undefined} */ id) => items.findIndex((item) => item.kind === "node" && item.id === id);
  const edgesBetween = (/** @type {number} */ a, /** @type {number} */ b, /** @type {boolean} */ directed) =>
    items.flatMap((item, p) => {
      if (item.kind !== "edge" || claimed(p)) {
        return [];
      }
      const [f, t] = [nodeIndex(item.from), nodeIndex(item.to)];
      return (f === a && t === b) || (!directed && f === b && t === a) ? [p] : [];
    });
  for (const pass of ["label", "any", "reverse"]) {
    for (const [s, { from, to }] of ends) {
      if (!open(s)) {
        continue;
      }
      const element = source[s];
      const directed = element.directed !== false;
      if (pass === "reverse") {
        if (directed) {
          const p = edgesBetween(to, from, true)[0];
          if (p !== undefined) {
            claim(s, p, "endpoints");
            changed.set(s, "the direction is reversed");
          }
        }
        continue;
      }
      const hits = edgesBetween(from, to, directed);
      const p = pass === "label" ? hits.find((hit) => items[hit].key === normalize(element.label)) : hits[0];
      if (p === undefined) {
        continue;
      }
      claim(s, p, pass === "label" ? "endpoints+label" : "endpoints");
      if (normalize(element.label) !== items[p].key && (normalize(element.label) || items[p].key)) {
        changed.set(s, `the label differs: source ${JSON.stringify(element.label ?? "")}, specification ${JSON.stringify(items[p].label)}`);
      }
    }
  }
  for (const [s] of ends) {
    if (open(s)) {
      missing.set(s, `no connection from ${JSON.stringify(source[s].from)} to ${JSON.stringify(source[s].to)} in the specification`);
    }
  }
  // An edge at a merged node is no longer the relationship the source drew.
  for (const [s, { from, to }] of ends) {
    if (matchedBy.has(s) && !changed.has(s) && (mergedNode(from) || mergedNode(to))) {
      changed.set(s, "an endpoint was merged with another source node");
    }
  }

  // Groups: the members the source names must be the members the specification's group has.
  for (const [p, claimers] of claims) {
    if (items[p].kind !== "group" || claimers.length !== 1) {
      continue;
    }
    const s = claimers[0];
    const named = source[s].members;
    if (!Array.isArray(named)) {
      continue;
    }
    const expected = named.map(resolve);
    const present = new Set(items[p].members ?? []);
    const absent = expected.filter((n) => n >= 0 && nodeOf.has(n) && !present.has(items[nodeOf.get(n) ?? 0].id ?? "")).map((n) => nameOf(source[n]));
    const wanted = new Set(expected.filter((n) => nodeOf.has(n)).map((n) => items[nodeOf.get(n) ?? 0].id));
    const extra = [...present].filter((id) => !wanted.has(id));
    const lost = expected.filter((n) => n < 0 || !nodeOf.has(n)).length;
    const reasons = [];
    if (absent.length > 0) {
      reasons.push(`members not in the group: ${absent.join(", ")}`);
    }
    if (extra.length > 0) {
      reasons.push(`members not in the source group: ${extra.join(", ")}`);
    }
    if (lost > 0) {
      reasons.push(`${lost} member(s) are not in the specification`);
    }
    if (reasons.length > 0) {
      changed.set(s, reasons.join("; "));
    }
  }
  // Source groups that share one specification group were merged.
  for (const [p, claimers] of claims) {
    if (items[p].kind === "group" && claimers.length > 1) {
      claimers.forEach((s) => changed.delete(s));
    }
  }

  // The ledger.
  const merged = [...claims].filter(([, claimers]) => claimers.length > 1);
  const mergedSources = new Set(merged.flatMap(([, claimers]) => claimers));
  const isKept = (/** @type {number} */ s) => matchedBy.has(s) && !mergedSources.has(s) && !changed.has(s);
  const candidatePool = new Set([...ambiguous.values()].flatMap(({ candidates }) => candidates));
  const expectation = (/** @type {number} */ s) => {
    if (detail === "faithful") {
      return { expected: false, expectedReason: "faithful: every omission needs the user's approval" };
    }
    if (detail === "simplified") {
      return { expected: true, expectedReason: "simplified: omissions are proposed; confirm them with the user" };
    }
    const element = source[s];
    if (element.kind === "edge" && /endpoint .* is not in the specification/.test(missing.get(s) ?? "")) {
      return { expected: true, expectedReason: "balanced: the connection follows from an omitted node" };
    }
    if (element.kind === "group" && Array.isArray(element.members) && element.members.length > 0 && element.members.every((member) => { const n = resolve(member); return n < 0 || missing.has(n); })) {
      return { expected: true, expectedReason: "balanced: every member of the group is omitted" };
    }
    return { expected: false, expectedReason: "balanced: nothing in the source explains this omission" };
  };
  const sources = (/** @type {(s: number) => boolean} */ pick) =>
    source.flatMap((_, s) => (pick(s) ? [s] : []));

  const keptList = sources(isKept).map((s) => ({
    source: sourceView(source[s], s),
    spec: specView(items[[...claims].find(([, c]) => c.includes(s))?.[0] ?? 0]),
    matchedBy: matchedBy.get(s),
  }));
  const changedList = sources((s) => changed.has(s) && !mergedSources.has(s)).map((s) => ({
    source: sourceView(source[s], s),
    spec: specView(items[[...claims].find(([, c]) => c.includes(s))?.[0] ?? 0]),
    reason: changed.get(s),
  }));
  const mergedList = merged.map(([p, claimers]) => ({
    spec: specView(items[p]),
    sources: claimers.map((s) => sourceView(source[s], s)),
    reason: `${claimers.length} source ${items[p].kind}s are one ${items[p].kind} in the specification`,
  }));
  const ambiguousList = sources((s) => ambiguous.has(s)).map((s) => ({
    source: sourceView(source[s], s),
    candidates: (ambiguous.get(s)?.candidates ?? []).map((p) => specView(items[p])),
    reason: ambiguous.get(s)?.reason,
  }));
  const missingList = sources((s) => missing.has(s)).map((s) => ({
    source: sourceView(source[s], s),
    reason: missing.get(s),
    ...expectation(s),
  }));
  const addedList = items.flatMap((item, p) => (claimed(p) || candidatePool.has(p) ? [] : [specView(item)]));

  const count = (/** @type {{ kind: string }[]} */ rows) => ({
    node: rows.filter((row) => row.kind === "node").length,
    edge: rows.filter((row) => row.kind === "edge").length,
    group: rows.filter((row) => row.kind === "group").length,
    total: rows.length,
  });
  const counts = {
    source: count(source),
    spec: count(items),
    kept: keptList.length,
    changed: changedList.length,
    merged: mergedSources.size,
    ambiguous: ambiguousList.length,
    missing: missingList.length,
    added: addedList.length,
  };
  const label = (/** @type {{ kind: string, label?: string, from?: string, to?: string }} */ row) =>
    row.kind === "edge" ? `${row.kind} ${row.from ?? ""}${row.from ? " -> " : ""}${row.to ?? ""}${row.label ? ` "${row.label}"` : ""}` : `${row.kind} "${row.label}"`;
  const report = [
    `Coverage of the redrawn ${parsed.type} diagram (inventory ${inventory}, detail ${detail}): ${counts.kept} of ${counts.source.total} source elements kept as they were, ${counts.changed} changed, ${counts.merged} merged, ${counts.ambiguous} ambiguous, ${counts.missing} missing; ${counts.added} added.`,
    ...changedList.map((row) => `- changed: ${label(row.source)}: ${row.reason}`),
    ...mergedList.map((row) => `- merged: ${row.sources.map(label).join(", ")} into ${label(row.spec)}`),
    ...ambiguousList.map((row) => `- ambiguous: ${label(row.source)}: ${row.reason}`),
    ...missingList.map((row) => `- missing${row.expected ? " (expected)" : ""}: ${label(row.source)}: ${row.reason}`),
    ...addedList.map((row) => `- added: ${label(row)}`),
  ].join("\n");

  return {
    ok: true,
    type: String(parsed.type),
    inventory,
    detail,
    counts,
    kept: keptList,
    changed: changedList,
    merged: mergedList,
    ambiguous: ambiguousList,
    missing: missingList,
    added: addedList,
    unusedMappings: unused,
    report,
  };
}
