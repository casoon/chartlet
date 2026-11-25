import type { ChartDiagnostic, RenderChartOptions } from "./render.mjs";

/** A warning of one `chartlet` block, with the line its fence starts on. */
export interface MarkdownItChartletWarning extends ChartDiagnostic {
  line?: number;
}

/**
 * Render options for every `chartlet` block. A block sets its own `idPrefix`, `view` and
 * `perspective` after the fence language: ```chartlet id=revenue view=checkout
 */
export interface MarkdownItChartletOptions
  extends Omit<RenderChartOptions, "idPrefix" | "view" | "perspective" | "alternative" | "manifest" | "advice"> {
  /** The fence language that marks a specification. Default: `"chartlet"`. */
  lang?: string;
  /**
   * For Markdown that becomes a Vue template, as in VitePress: the chart is passed as `v-html`,
   * so Vue keeps its `<style>` elements when it renders the page on the client.
   */
  vue?: boolean;
  /** Called for each warning; by default it is written with `console.warn`. */
  onWarning?: (warning: MarkdownItChartletWarning, env: unknown) => void;
}

/**
 * A markdown-it plugin: every fenced block in the language `chartlet` becomes the rendered chart.
 * An invalid specification throws with the line of the block.
 */
declare function markdownItChartlet(md: unknown, options?: MarkdownItChartletOptions): void;

export default markdownItChartlet;
