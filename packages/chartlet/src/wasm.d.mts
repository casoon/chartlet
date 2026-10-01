import type {
  RenderChartDetailedResult,
  RenderChartOptions,
  RenderChartResult,
  StylesheetOptions,
} from "./render.mjs";

export interface ChartRenderer {
  /** Renders in WebAssembly; `options.binary` is ignored. */
  renderChart(spec: Record<string, unknown>, options?: RenderChartOptions): RenderChartResult;
  /** Renders in WebAssembly and reports the outcome as data; see `renderChartDetailed`. */
  renderChartDetailed(
    spec: Record<string, unknown> | string,
    options?: RenderChartOptions,
  ): RenderChartDetailedResult;
  /** The shared stylesheet that charts rendered with `styles: "external"` rely on. */
  stylesheet(options?: StylesheetOptions): string;
}

/** Instantiates the renderer from the compiled `@casoon/chartlet/chartlet.wasm` module. */
export declare function createRenderer(module: WebAssembly.Module): ChartRenderer;
