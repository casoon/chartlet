import type { RenderChartOptions } from "./render.mjs";

/**
 * Render options for every `chartlet` block. A block sets its own `idPrefix`, `view` and
 * `perspective` after the fence language: ```chartlet id=revenue view=checkout
 */
export interface RemarkChartletOptions
  extends Omit<RenderChartOptions, "idPrefix" | "view" | "perspective" | "alternative" | "manifest" | "advice"> {
  /** The fence language that marks a specification. Default: `"chartlet"`. */
  lang?: string;
  /**
   * For MDX rendered with React and hydrated on the client, as in Docusaurus: checked inputs of
   * the CSS-only controls become `defaultChecked`, so React leaves them uncontrolled.
   */
  react?: boolean;
}

/**
 * A remark plugin: every fenced block in the language `chartlet` becomes the rendered chart, as
 * hast elements, so it works in Markdown and MDX. An invalid specification fails the file with the
 * position of the block; warnings become messages of the file.
 */
declare function remarkChartlet(options?: RemarkChartletOptions): (tree: unknown, file: unknown) => void;

export default remarkChartlet;
