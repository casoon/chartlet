// A markdown-it plugin: every fenced block in the language `chartlet` becomes the rendered chart
// while the Markdown is rendered. Other fences keep the renderer they had.

import { renderFence } from "./markdown.mjs";

export default function markdownItChartlet(md, options = {}) {
  const {
    lang = "chartlet",
    vue = false,
    onWarning = (warning) =>
      console.warn(`warning[${warning.code}] at ${warning.path}: ${warning.message} (line ${warning.line})`),
    ...renderOptions
  } = options;
  const fence = md.renderer.rules.fence;
  md.renderer.rules.fence = (tokens, index, markdownOptions, env, self) => {
    const token = tokens[index];
    const [language = "", ...meta] = token.info.trim().split(/(\s+)/);
    if (language !== lang) {
      return fence(tokens, index, markdownOptions, env, self);
    }
    const line = token.map ? token.map[0] + 1 : undefined;
    let rendered;
    try {
      rendered = renderFence(token.content, meta.join(""), renderOptions);
    } catch (error) {
      throw new Error(line === undefined ? error.message : `${error.message} (chartlet block at line ${line})`);
    }
    for (const warning of rendered.warnings) {
      onWarning({ ...warning, line }, env);
    }
    return vue ? `${vueHtml(rendered.content)}\n` : `${rendered.content}\n`;
  };
}

// Markdown that becomes a Vue template, as in VitePress, loses every `<style>` when Vue renders
// the page on the client. As `v-html`, the chart stays HTML that Vue does not compile.
function vueHtml(content) {
  const literal = JSON.stringify(content).replaceAll("&", "&amp;").replaceAll('"', "&quot;");
  return `<div class="chartlet-markdown" v-html="${literal}"></div>`;
}
