// What the Markdown plugins share: rendering one fenced `chartlet` block, and turning chartlet's
// own HTML into a hast tree for pipelines such as MDX that take no raw HTML.

import { renderChartDetailed } from "./render.mjs";

// The settings a fence may carry after its language, such as ```chartlet id=revenue view=checkout
const FENCE_SETTINGS = { id: "idPrefix", view: "view", perspective: "perspective" };
const SETTING = /\s*([^\s=]+)=(?:"([^"]*)"|'([^']*)'|(\S+))/y;

// Renders the specification in a fenced block. Returns the HTML and the warnings as
// `{ code, path, message }`; an invalid specification or fence throws.
export function renderFence(source, meta, options = {}) {
  const result = renderChartDetailed(source, {
    format: "html",
    ...options,
    ...fenceSettings(meta ?? ""),
  });
  if (!result.ok) {
    const { code, path, message } = result.error;
    throw new Error(path == null ? `chartlet: ${message}` : `chartlet: ${code} at ${path}: ${message}`);
  }
  return { content: result.content, warnings: result.warnings };
}

function fenceSettings(meta) {
  const settings = {};
  let rest = meta.trim();
  while (rest !== "") {
    SETTING.lastIndex = 0;
    const match = SETTING.exec(rest);
    if (!match) {
      throw new Error(
        `chartlet: cannot read ${JSON.stringify(rest)} after the fence language; write key=value with ${Object.keys(FENCE_SETTINGS).join(", ")}`,
      );
    }
    const [, key, doubleQuoted, singleQuoted, bare] = match;
    if (!(key in FENCE_SETTINGS)) {
      throw new Error(
        `chartlet: unknown fence setting ${JSON.stringify(key)}; the settings are ${Object.keys(FENCE_SETTINGS).join(", ")}`,
      );
    }
    settings[FENCE_SETTINGS[key]] = doubleQuoted ?? singleQuoted ?? bare;
    rest = rest.slice(SETTING.lastIndex).trim();
  }
  return settings;
}

const VOID_ELEMENTS = new Set(["area", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr"]);
const RAW_TEXT_ELEMENTS = new Set(["script", "style"]);
const BOOLEAN_ATTRIBUTES = new Set(["checked", "disabled", "hidden", "open", "selected"]);
const OPEN_TAG = /<([a-zA-Z][\w:-]*)((?:\s+[^\s=>/]+(?:="[^"]*")?)*)\s*(\/?)>/y;
const ATTRIBUTE = /\s+([^\s=>/]+)(?:="([^"]*)")?/g;
const ENTITY = /&(?:#(\d+)|#x([0-9a-fA-F]+)|(amp|lt|gt|quot|apos));/g;
const NAMED_ENTITIES = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'" };

// Turns the HTML chartlet writes into a hast root. It reads chartlet's own output, which is
// well-formed with quoted attributes, not arbitrary HTML. With `react`, a checked input becomes
// `defaultChecked`, so React leaves the CSS-only controls uncontrolled after hydration.
export function htmlToHast(html, options = {}) {
  const root = { type: "root", children: [] };
  const stack = [root];
  let index = 0;
  while (index < html.length) {
    const start = html.indexOf("<", index);
    const end = start === -1 ? html.length : start;
    if (end > index) {
      append(stack, { type: "text", value: decode(html.slice(index, end)) });
    }
    if (start === -1) {
      break;
    }
    if (html.startsWith("<!--", start)) {
      index = html.indexOf("-->", start) + 3;
      continue;
    }
    if (html.startsWith("</", start)) {
      const close = html.indexOf(">", start);
      const name = html.slice(start + 2, close).trim();
      const element = stack.pop();
      if (element.tagName !== name) {
        throw new Error(`chartlet: unexpected </${name}> in the rendered HTML`);
      }
      index = close + 1;
      continue;
    }
    OPEN_TAG.lastIndex = start;
    const match = OPEN_TAG.exec(html);
    if (!match) {
      throw new Error(`chartlet: cannot read the rendered HTML at offset ${start}`);
    }
    const [, tagName, attributes, selfClosing] = match;
    const element = { type: "element", tagName, properties: properties(attributes, options), children: [] };
    append(stack, element);
    index = OPEN_TAG.lastIndex;
    if (RAW_TEXT_ELEMENTS.has(tagName)) {
      const close = html.indexOf(`</${tagName}>`, index);
      if (close > index) {
        element.children.push({ type: "text", value: html.slice(index, close) });
      }
      index = close + tagName.length + 3;
    } else if (!selfClosing && !VOID_ELEMENTS.has(tagName)) {
      stack.push(element);
    }
  }
  if (stack.length > 1) {
    throw new Error(`chartlet: <${stack.at(-1).tagName}> is not closed in the rendered HTML`);
  }
  return root;
}

function append(stack, node) {
  stack.at(-1).children.push(node);
}

function properties(attributes, options) {
  const result = {};
  for (const [, name, value] of attributes.matchAll(ATTRIBUTE)) {
    if (name === "class") {
      result.className = decode(value ?? "").split(/\s+/).filter(Boolean);
    } else if (name === "checked" && options.react) {
      result.defaultChecked = true;
    } else if (value === undefined) {
      result[name] = BOOLEAN_ATTRIBUTES.has(name) ? true : "";
    } else {
      result[name] = decode(value);
    }
  }
  return result;
}

function decode(text) {
  return text.replace(ENTITY, (_, decimal, hex, name) =>
    decimal ? String.fromCodePoint(Number(decimal)) : hex ? String.fromCodePoint(parseInt(hex, 16)) : NAMED_ENTITIES[name],
  );
}
