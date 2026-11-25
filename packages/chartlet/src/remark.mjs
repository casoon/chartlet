// A remark plugin: every fenced block in the language `chartlet` becomes the rendered chart while
// the Markdown is built. The chart enters the tree as hast elements, not as raw HTML, so it works
// in Markdown and MDX pipelines alike.

import { htmlToHast, renderFence } from "./markdown.mjs";

export default function remarkChartlet(options = {}) {
  const { lang = "chartlet", react = false, ...renderOptions } = options;
  return (tree, file) => {
    visit(tree, (node, index, parent) => {
      if (node.type !== "code" || node.lang !== lang) {
        return;
      }
      let rendered;
      try {
        rendered = renderFence(node.value, node.meta, renderOptions);
      } catch (error) {
        file.fail(error.message, node);
      }
      for (const { code, path, message } of rendered.warnings) {
        file.message(`warning[${code}] at ${path}: ${message}`, node);
      }
      parent.children[index] = chartNode(htmlToHast(rendered.content, { react }), node.position);
    });
  };
}

// An mdast node that turns into the chart's hast: its single top-level element, or a `div`
// around several.
function chartNode(hast, position) {
  const elements = hast.children.filter(
    (child) => child.type === "element" || child.value.trim() !== "",
  );
  const [element] = elements;
  const data =
    elements.length === 1 && element.type === "element"
      ? { hName: element.tagName, hProperties: element.properties, hChildren: element.children }
      : { hName: "div", hProperties: {}, hChildren: hast.children };
  return { type: "chartlet", data, position };
}

function visit(node, callback) {
  if (!node.children) {
    return;
  }
  node.children.forEach((child, index) => {
    callback(child, index, node);
    visit(child, callback);
  });
}
