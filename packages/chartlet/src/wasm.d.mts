import type { RenderChartOptions, RenderChartResult } from "./render.mjs";

export interface ChartRenderer {
  /** Renders in WebAssembly; `options.binary` is ignored. */
  renderChart(spec: Record<string, unknown>, options?: RenderChartOptions): RenderChartResult;
}

/** Instantiates the renderer from the compiled `@casoon/chartlet/chartlet.wasm` module. */
export declare function createRenderer(module: WebAssembly.Module): ChartRenderer;
