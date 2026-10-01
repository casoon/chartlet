// The MCP App view of chartlet_render: one HTML resource that shows the figure the compiler
// rendered, in a host that supports MCP Apps. The server renders every byte; the view's script
// only talks to the host.

import { readFileSync } from "node:fs";

import { renderChartDetailed, stylesheet } from "@casoon/chartlet";
import { getUiCapability, RESOURCE_MIME_TYPE } from "@modelcontextprotocol/ext-apps/server";

export const VIEW_URI = "ui://chartlet/figure";
/** The key of the view payload in the tool result's `_meta`, which the model does not see. */
export const VIEW_META_KEY = "chartlet/view";
/** No network access and no outside resources: everything the view needs is in the resource. */
export const VIEW_RESOURCE_META = {
  ui: {
    csp: { connectDomains: [], resourceDomains: [] },
    permissions: { clipboardWrite: {} },
    prefersBorder: true,
  },
};

/**
 * The view's HTML: the shared chartlet stylesheet once, the view's own styles and the bridge
 * script. Charts arrive rendered with `styles: "external"`, so they rely on that stylesheet.
 *
 * @param {string} version
 */
export function viewHtml(version) {
  return readFileSync(new URL("./view.html", import.meta.url), "utf8")
    .replace("/*chartlet-stylesheet*/", () => stylesheet())
    .replace("/*chartlet-version*/", () => version);
}

/**
 * Whether the client declared that it shows MCP App views.
 *
 * @param {import("@modelcontextprotocol/server").ClientCapabilities | undefined} capabilities
 */
export function supportsView(capabilities) {
  return getUiCapability(capabilities)?.mimeTypes?.includes(RESOURCE_MIME_TYPE) ?? false;
}

/**
 * @param {Record<string, unknown> | string} spec
 * @param {import("@casoon/chartlet").RenderChartOptions} options
 */
function render(spec, options) {
  const result = renderChartDetailed(spec, options);
  if (!result.ok) {
    throw new Error(`chartlet could not render the view: ${result.error.code}: ${result.error.message}`);
  }
  return result.content;
}

/**
 * @param {unknown} title
 */
function fileStem(title) {
  const stem = String(title ?? "")
    .normalize("NFKD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 60)
    .replace(/-+$/, "");
  return stem === "" ? "chart" : stem;
}

/**
 * What the view shows for a successful render: the HTML profile per theme (both light and dark
 * when the spec sets no theme, so the view can follow the host), the SVG and HTML files the
 * download actions hand to the host, and the specification for the copy action.
 *
 * @param {import("./charts.mjs").RenderInput} input
 * @param {{ content?: string, manifest: import("@casoon/chartlet").ChartManifest }} result
 */
export function viewPayload(input, result) {
  const spec = /** @type {Record<string, unknown>} */ (
    typeof input.spec === "string" ? JSON.parse(input.spec) : input.spec
  );
  const format = input.format ?? "html";
  const idPrefix = result.manifest.idPrefix;
  const html = { format: /** @type {const} */ ("html"), idPrefix, table: input.table };
  const figures =
    spec.theme === undefined
      ? {
          light: render(spec, { ...html, styles: "external" }),
          dark: render({ ...spec, theme: "dark" }, { ...html, styles: "external" }),
        }
      : { [String(spec.theme)]: render(spec, { ...html, styles: "external" }) };
  const stem = fileStem(spec.title);
  const content = (/** @type {"svg" | "html"} */ kind) =>
    result.content !== undefined && format === kind
      ? result.content
      : kind === "svg"
        ? render(spec, { format: "svg", idPrefix, variant: format === "svg" ? input.variant : undefined })
        : render(spec, html);
  return {
    title: String(spec.title ?? ""),
    figures,
    files: {
      svg: { name: `${stem}.svg`, mimeType: "image/svg+xml", text: content("svg") },
      html: { name: `${stem}.html`, mimeType: "text/html", text: content("html") },
    },
    spec: JSON.stringify(spec, null, 2),
  };
}
