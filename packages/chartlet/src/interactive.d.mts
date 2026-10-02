/** A pane's plot as its hook describes it: `[domain, range]` per axis. */
export interface ChartPlot {
  pane: number;
  x: [number[], number[]];
  y: [number[], number[]];
}

/** A value column of the data table. */
export interface ChartColumn {
  name: string;
  /** The series: its index among the data layers, or among the series of a category chart. */
  series: number;
  pane: number;
  part: "value" | "lower" | "upper" | "open" | "high" | "low" | "close";
  /** A stacked area, drawn on top of the stacked areas before it in its pane. */
  stacked?: boolean;
}

/** A row of the data table. */
export interface ChartRow {
  /** Unix seconds on a time axis, the row index otherwise. */
  x: number;
  label: string;
  /** Each value as the chart writes it. */
  text: string[];
  /** Each value unformatted; `null` where it is missing. */
  value: (number | null)[];
}

/** What a chart rendered with `hooks: true` holds, as `readChart` reads it. */
export interface Chart {
  root: Element;
  /** The ID prefix the chart was rendered with. */
  id: string | undefined;
  svgs: SVGSVGElement[];
  table: HTMLTableElement | null;
  columns: ChartColumn[];
  rows: ChartRow[];
  plots: Map<SVGSVGElement, ChartPlot[]>;
  /** Series switched off, by `ChartColumn.series`. */
  hidden: Set<number>;
  /** Registers a listener for `changed()`. */
  on(listener: () => void): void;
  /** Tells every feature that the chart's state changed, such as a series switched off. */
  changed(): void;
}

/** A feature: enhances the chart and returns what undoes it. */
export type ChartFeature = (chart: Chart) => (() => void) | undefined | void;

/** Maps `value` from `domain` onto `range`, piecewise linear between neighbouring pairs. */
export declare function toPixel(domain: number[], range: number[], value: number): number;
/** The inverse of `toPixel`: the value at `pixel`. */
export declare function toValue(domain: number[], range: number[], pixel: number): number;

/** Reads a chart rendered with `hooks: true` from its figure, its SVG or an element around either. */
export declare function readChart(root: Element): Chart;

/**
 * Enhances the chart in `root` (its figure, its SVG, or an element around either) with the
 * features passed in, e.g. `enhance(figure, { crosshair, toggle })`. Give a feature options with
 * a function: `{ play: (chart) => play(chart, { interval: 400 }) }`.
 */
export declare function enhance(
  root: Element,
  features?: Partial<Record<"crosshair" | "toggle" | "steps" | "play", ChartFeature>>,
): { chart: Chart; destroy(): void };

/** A vertical rule with the values of one observation, by pointer and keyboard. */
export declare function crosshair(chart: Chart): () => void;

export interface ToggleOptions {
  /** The legend of the checkbox group; `"Series"` by default. */
  legend?: string;
}
/** One checkbox per named series of a time chart or small multiples. */
export declare function toggle(chart: Chart, options?: ToggleOptions): (() => void) | undefined;

/** Shows only the data between `from` and `to` (axis values), or everything without arguments. */
export declare function reveal(chart: Chart, from?: number | null, to?: number | null): void;

export interface StepsOptions {
  /** The stations on the page; `[data-chartlet-step]` by default. */
  selector?: string;
}
/** Scroll stations that set the chart to the state they describe. */
export declare function steps(chart: Chart, options?: StepsOptions): (() => void) | undefined;

export interface PlayOptions {
  /** Milliseconds per observation; by default the whole chart plays in about six seconds. */
  interval?: number;
  /** Button and group labels, for charts in other languages. */
  labels?: Partial<Record<"group" | "play" | "pause" | "back" | "forward" | "all", string>>;
}
/** Reveals the data one observation at a time, with pause, steps and a way to show all. */
export declare function play(chart: Chart, options?: PlayOptions): (() => void) | undefined;
