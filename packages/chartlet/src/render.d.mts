export type ChartFormat = "svg" | "html";
export type TableMode = "details" | "visible";
/**
 * Which layout the SVG profile renders; `mobile` needs a `mobile` field in the spec, `print`
 * draws the chart with literal colors for print and PDF renderers.
 */
export type ChartVariant = "desktop" | "mobile" | "print";

export interface RenderChartOptions {
  format?: ChartFormat;
  table?: TableMode;
  idPrefix?: string;
  variant?: ChartVariant;
  /** Fail on any warning, except those whose code is listed in `allowWarnings`. */
  strict?: boolean;
  /** Warning codes that `strict` lets through, such as `"dense_chart"`. They are still reported. */
  allowWarnings?: string[];
  /** Also return a provenance manifest of the render in `manifest`. */
  manifest?: boolean;
  /**
   * `inline` (default): every chart carries its whole stylesheet. `external`: the page loads
   * `@casoon/chartlet/chartlet.css` once, and a chart carries only its own declared colors.
   */
  styles?: "inline" | "external";
  /** Also return the chart's text alternative in `alternative`. */
  alternative?: boolean;
  /** A `chartlet` executable to render with instead of the bundled WebAssembly build. */
  binary?: string;
}

export interface ChartManifestWarning {
  code: string;
  path: string;
  message: string;
}

/**
 * The provenance of one render. It carries no timestamp: the same render always yields the same
 * manifest.
 */
export interface ChartManifest {
  /** The version of the chartlet crate that rendered. */
  chartlet: string;
  schemaVersion: number;
  /** `sha256:` and the hex SHA-256 of the canonical (parsed and re-serialized) specification. */
  specHash: string;
  /** `sha256:` and the hex SHA-256 of `content` as UTF-8. */
  outputHash: string;
  format: ChartFormat;
  variant: ChartVariant;
  /** The ID prefix passed in, or the one derived from the specification. */
  idPrefix: string;
  warnings: ChartManifestWarning[];
}

export interface RenderChartResult {
  content: string;
  /**
   * The CSP source expressions (`'sha256-…'`) of every distinct inline `<style>` element in
   * `content`, in order of first appearance — what a strict `style-src` has to allow.
   */
  styleHashes: string[];
  warnings: string[];
  /** Present when `options.manifest` is set. */
  manifest?: ChartManifest;
  /** Present when `options.alternative` is set. */
  alternative?: ChartTextAlternative;
}

/**
 * What a chart says without its graphic: the description its SVG carries and its data table, as
 * text, for a host that builds its own accessible wrapper around the SVG.
 */
export interface ChartTextAlternative {
  description: string;
  table: {
    caption: string;
    /** The column heads; the first names the categories. */
    columns: string[];
    /** One row per category, starting with the category; values as the chart writes them. */
    rows: string[][];
  };
}

export interface StylesheetOptions {
  /** Only the rules of these chart types, such as `["bar", "time"]`; all types when absent. */
  types?: string[];
}

/** The shared stylesheet that charts rendered with `styles: "external"` rely on. */
export declare function stylesheet(options?: StylesheetOptions): string;

export declare function renderChart(
  spec: Record<string, unknown>,
  options?: RenderChartOptions,
): RenderChartResult;

/** An error or warning as the CLI reports it with `--diagnostics json`. */
export interface ChartDiagnostic {
  code: string;
  /** A JSON Pointer into the specification, or `null` for option errors and `strict_warnings`. */
  path: string | null;
  message: string;
}

export interface RenderChartDetailedSuccess {
  ok: true;
  content: string;
  styleHashes: string[];
  warnings: ChartDiagnostic[];
  /** Present when `options.manifest` is set. */
  manifest?: ChartManifest;
  /** Present when `options.alternative` is set. */
  alternative?: ChartTextAlternative;
}

export interface RenderChartDetailedFailure {
  ok: false;
  error: ChartDiagnostic;
  warnings: ChartDiagnostic[];
}

export type RenderChartDetailedResult = RenderChartDetailedSuccess | RenderChartDetailedFailure;

/**
 * Renders like `renderChart`, but returns a failed render as `{ ok: false, error }` instead of
 * throwing, and every warning as a structured diagnostic. A string `spec` is read as JSON text,
 * so malformed JSON is reported as `invalid_json`. A missing WebAssembly build or CLI still
 * throws.
 */
export declare function renderChartDetailed(
  spec: Record<string, unknown> | string,
  options?: RenderChartOptions,
): RenderChartDetailedResult;
