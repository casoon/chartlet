export type ChartFormat = "svg" | "html";
export type TableMode = "details" | "visible";

export interface RenderChartOptions {
  format?: ChartFormat;
  table?: TableMode;
  idPrefix?: string;
  strict?: boolean;
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
