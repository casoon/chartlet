// Validation, rendering and computed facts on top of `@casoon/chartlet`. Every result comes from
// the compiler or from arithmetic on the specification's own values.

import { lstatSync, mkdirSync, realpathSync, statSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, relative, resolve, sep } from "node:path";

import { renderChartDetailed } from "@casoon/chartlet";

/**
 * @typedef {import("@casoon/chartlet").ChartDiagnostic} ChartDiagnostic
 * @typedef {Record<string, unknown> | string} SpecInput
 * @typedef {{ ok: boolean, error?: ChartDiagnostic, warnings: ChartDiagnostic[] }} Validation
 */

/**
 * Validates by rendering the HTML profile, which lays out both the desktop and a `mobile`
 * variant, so warnings of either are reported. The content is discarded.
 *
 * @param {SpecInput} spec
 * @returns {Validation}
 */
export function validateSpec(spec) {
  const result = renderChartDetailed(spec, { format: "html", manifest: false });
  return result.ok
    ? { ok: true, warnings: result.warnings }
    : { ok: false, error: result.error, warnings: result.warnings };
}

/**
 * @typedef {{
 *   spec: SpecInput,
 *   format?: "svg" | "html",
 *   variant?: "desktop" | "mobile" | "print" | "social",
 *   idPrefix?: string,
 *   table?: "details" | "visible",
 *   outputPath?: string,
 * }} RenderInput
 */

/**
 * Renders with a provenance manifest. With `outputPath`, writes the content there (see
 * `resolveOutputPath`) and returns the path and byte size instead of the content.
 *
 * @param {RenderInput} input
 * @param {string} [root] the sandbox directory `outputPath` is relative to (`CHARTLET_MCP_ROOT`);
 *   without it, `outputPath` is refused
 */
export function renderSpec(input, root) {
  const target = input.outputPath === undefined ? undefined : resolveOutputPath(input.outputPath, root);
  const result = renderChartDetailed(input.spec, {
    format: input.format ?? "html",
    variant: input.variant,
    idPrefix: input.idPrefix,
    table: input.table,
    manifest: true,
  });
  if (!result.ok) {
    return result;
  }
  const { content, warnings, styleHashes, manifest } = result;
  if (target === undefined) {
    return { ok: true, content, warnings, styleHashes, manifest };
  }
  mkdirSync(dirname(target.path), { recursive: true });
  writeFileSync(target.path, content);
  return {
    ok: true,
    path: relative(target.root, target.path),
    bytes: Buffer.byteLength(content),
    warnings,
    styleHashes,
    manifest,
  };
}

/**
 * Resolves a relative output path inside the sandbox directory `root`. Refuses everything when
 * there is no `root`, a `root` that is not a directory or is the file system root, an absolute
 * path, any `..` segment, a path through a symbolic link that leaves `root`, and an existing
 * symbolic link as the target.
 *
 * @param {string} outputPath
 * @param {string | undefined} root
 * @returns {{ root: string, path: string }} the real path of `root` and the absolute target path
 */
export function resolveOutputPath(outputPath, root) {
  if (root === undefined || root === "") {
    throw new Error(
      "outputPath is disabled: the server writes files only below the directory named by the environment variable CHARTLET_MCP_ROOT, and it is not set. Set it in the client configuration (for example \"env\": { \"CHARTLET_MCP_ROOT\": \"/path/to/project\" }), or omit outputPath to get the content back.",
    );
  }
  let realRoot;
  try {
    realRoot = realpathSync(root);
  } catch {
    throw new Error(`CHARTLET_MCP_ROOT ${JSON.stringify(root)} does not exist; set it to an existing directory.`);
  }
  if (!statSync(realRoot).isDirectory()) {
    throw new Error(`CHARTLET_MCP_ROOT ${JSON.stringify(root)} is not a directory.`);
  }
  if (dirname(realRoot) === realRoot) {
    throw new Error("CHARTLET_MCP_ROOT is the file system root; set it to a project directory.");
  }
  if (outputPath === "" || isAbsolute(outputPath) || /^[a-zA-Z]:/.test(outputPath)) {
    throw new Error(
      `outputPath must be a path relative to CHARTLET_MCP_ROOT (${realRoot}), got ${JSON.stringify(outputPath)}.`,
    );
  }
  if (outputPath.split(/[\\/]/).includes("..")) {
    throw new Error(`outputPath must not contain "..", got ${JSON.stringify(outputPath)}.`);
  }
  const target = resolve(realRoot, outputPath);
  if (target === realRoot || !target.startsWith(realRoot + sep)) {
    throw new Error(`outputPath must name a file inside CHARTLET_MCP_ROOT (${realRoot}).`);
  }
  // The deepest directory that exists must resolve inside the root, so a symbolic link cannot
  // lead the write elsewhere.
  let existing = dirname(target);
  for (;;) {
    try {
      existing = realpathSync(existing);
      break;
    } catch {
      existing = dirname(existing);
    }
  }
  if (existing !== realRoot && !existing.startsWith(realRoot + sep)) {
    throw new Error(`outputPath leads outside CHARTLET_MCP_ROOT (${realRoot}) through a symbolic link.`);
  }
  try {
    if (lstatSync(target).isSymbolicLink()) {
      throw new Error(`outputPath ${JSON.stringify(outputPath)} is a symbolic link; refusing to write through it.`);
    }
  } catch (error) {
    if (/** @type {NodeJS.ErrnoException} */ (error).code !== "ENOENT") {
      throw error;
    }
  }
  return { root: realRoot, path: target };
}

const ENTITIES = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'" };

/**
 * @param {string} text
 */
function unescapeXml(text) {
  return text.replace(/&(#x[0-9a-fA-F]+|#\d+|amp|lt|gt|quot|apos);/g, (_, entity) => {
    if (entity.startsWith("#x")) {
      return String.fromCodePoint(Number.parseInt(entity.slice(2), 16));
    }
    if (entity.startsWith("#")) {
      return String.fromCodePoint(Number(entity.slice(1)));
    }
    return ENTITIES[/** @type {keyof typeof ENTITIES} */ (entity)];
  });
}

/**
 * @typedef {{ value: number, at: string | number }} Point
 * @typedef {{ value: number, at: (string | number)[] }} Extreme
 * @typedef {{
 *   pane?: string,
 *   name: string | null,
 *   mark?: string,
 *   count: number,
 *   missing: number,
 *   min?: Extreme,
 *   max?: Extreme,
 *   first?: Point,
 *   last?: Point,
 * }} SeriesFacts
 */

/**
 * The facts about a chart that can be computed without interpretation: the accessible description
 * chartlet generates, and per data series its count, missing values, minimum, maximum, first and
 * last value with the label or time they belong to.
 *
 * @param {SpecInput} input
 */
export function explainSpec(input) {
  const result = renderChartDetailed(input, { format: "svg" });
  if (!result.ok) {
    return result;
  }
  const spec = /** @type {Record<string, any>} */ (typeof input === "string" ? JSON.parse(input) : input);
  // A specification's own description replaces the generated one in <desc>; render without it to
  // read the generated text.
  const { description: specDescription, ...withoutDescription } = spec;
  const generated =
    specDescription === undefined
      ? result
      : renderChartDetailed(withoutDescription, { format: "svg" });
  const desc = generated.ok ? /<desc\b[^>]*>([\s\S]*?)<\/desc>/.exec(generated.content) : null;
  const series = seriesFacts(spec);
  return {
    ok: true,
    note: "Computed, not interpreted: every value below is generated by chartlet or calculated from the specification's data. It contains no judgement about causes, trends or significance.",
    type: spec.type,
    title: spec.title,
    generatedDescription: desc ? unescapeXml(desc[1]) : null,
    ...(specDescription === undefined ? {} : { specDescription }),
    seriesCount: series.length,
    series,
    warnings: result.warnings,
  };
}

/**
 * @param {{ value: unknown, at: string | number }[]} points
 * @returns {Omit<SeriesFacts, "name">}
 */
function facts(points) {
  const present = /** @type {Point[]} */ (points.filter(({ value }) => typeof value === "number"));
  if (present.length === 0) {
    return { count: points.length, missing: points.length };
  }
  const values = present.map(({ value }) => value);
  const extreme = (/** @type {number} */ value) => ({
    value,
    at: present.filter((point) => point.value === value).map(({ at }) => at),
  });
  return {
    count: points.length,
    missing: points.length - present.length,
    min: extreme(Math.min(...values)),
    max: extreme(Math.max(...values)),
    first: present[0],
    last: present[present.length - 1],
  };
}

/**
 * @param {Record<string, any>} spec
 * @returns {SeriesFacts[]}
 */
function seriesFacts(spec) {
  switch (spec.type) {
    case "bar":
    case "line":
      if (Array.isArray(spec.data)) {
        return [
          {
            name: null,
            ...facts(spec.data.map(({ label, value }) => ({ value, at: label }))),
          },
        ];
      }
      return (spec.series ?? []).map((/** @type {any} */ { name, values }) => ({
        name,
        ...facts(values.map((/** @type {unknown} */ value, /** @type {number} */ index) => ({ value, at: spec.categories[index] }))),
      }));
    case "time":
    case "multiples":
      return (spec.panes ?? []).flatMap((/** @type {any} */ pane, /** @type {number} */ index) =>
        (pane.layers ?? []).flatMap((/** @type {any} */ layer) => layerFacts(pane, index, layer)),
      );
    case "stripes":
      return [
        {
          name: null,
          ...facts(
            spec.stripes.values.map((/** @type {unknown} */ value, /** @type {number} */ index) => ({
              value,
              at: spec.stripes.firstYear + index,
            })),
          ),
        },
      ];
    case "calendar":
      return [
        {
          name: null,
          ...facts(spec.calendar.days.map((/** @type {any} */ { date, value }) => ({ value, at: date }))),
        },
      ];
    case "rangebar":
      return [
        { name: "low", ...facts(spec.ranges.map((/** @type {any} */ { label, low }) => ({ value: low, at: label }))) },
        { name: "high", ...facts(spec.ranges.map((/** @type {any} */ { label, high }) => ({ value: high, at: label }))) },
      ];
    default:
      return [];
  }
}

/**
 * @param {any} pane
 * @param {number} index
 * @param {any} layer
 * @returns {SeriesFacts[]}
 */
function layerFacts(pane, index, layer) {
  const paneName = pane.title ?? pane.valueAxis?.title ?? `pane ${index + 1}`;
  const name = layer.name ?? null;
  if ((layer.mark === "line" || layer.mark === "area") && Array.isArray(layer.points)) {
    return [
      {
        pane: paneName,
        name,
        mark: layer.mark,
        ...facts(layer.points.map((/** @type {any} */ { time, value }) => ({ value, at: time }))),
      },
    ];
  }
  if (layer.mark === "ohlc" && Array.isArray(layer.data)) {
    const data = layer.data;
    const low = facts(data.map((/** @type {any} */ { time, low }) => ({ value: low, at: time })));
    const high = facts(data.map((/** @type {any} */ { time, high }) => ({ value: high, at: time })));
    const open = facts(data.map((/** @type {any} */ { time, open }) => ({ value: open, at: time })));
    const close = facts(data.map((/** @type {any} */ { time, close }) => ({ value: close, at: time })));
    return [
      {
        pane: paneName,
        name,
        mark: "ohlc",
        count: data.length,
        missing: 0,
        min: low.min,
        max: high.max,
        first: open.first,
        last: close.last,
      },
    ];
  }
  return [];
}
