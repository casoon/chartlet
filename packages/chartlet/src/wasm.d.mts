import type {
  InspectChartOptions,
  InspectChartResult,
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
  /** Runs the inspections in WebAssembly; `options.binary` is ignored. See `inspectChart`. */
  inspectChart(spec: Record<string, unknown> | string, options?: InspectChartOptions): InspectChartResult;
}

/** Instantiates the renderer from the compiled `@casoon/chartlet/chartlet.wasm` module. */
export declare function createRenderer(module: WebAssembly.Module): ChartRenderer;
