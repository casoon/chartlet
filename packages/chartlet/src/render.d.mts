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
  /** Also return a provenance manifest of the render in `manifest`. */
  manifest?: boolean;
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
}

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
