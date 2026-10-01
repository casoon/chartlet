export type ChartFormat = "svg" | "html";
export type TableMode = "details" | "visible";
/** Which layout the SVG profile renders; `mobile` needs a `mobile` field in the spec. */
export type ChartVariant = "desktop" | "mobile";

export interface RenderChartOptions {
  format?: ChartFormat;
  table?: TableMode;
  idPrefix?: string;
  variant?: ChartVariant;
  strict?: boolean;
  /** A `chartlet` executable to render with instead of the bundled WebAssembly build. */
  binary?: string;
}

export interface RenderChartResult {
  content: string;
  warnings: string[];
}

export declare function renderChart(
  spec: Record<string, unknown>,
  options?: RenderChartOptions,
): RenderChartResult;
