/**
 * The fields of an element beyond the ones the builder handles. They are copied into the
 * specification as they are and checked by the validation of the core against the schema
 * (`schema/chartlet.schema.json`), which stays the only contract: `kind`, `sublabel`,
 * `technology`, `dash`, `weight`, `step`, `role`, `value` and fields added later all pass through.
 */
export type Fields = Record<string, unknown>;

/** Joins parts into an identifier with hyphens; throws if the result is not one. */
export declare function id(...parts: (string | number)[]): string;

export interface ArchitectureBuilder {
  /** Declares a boundary and returns its id. `fields.in` names a boundary declared before. */
  boundary(id: string, label: string, fields?: Fields & { in?: string }): string;
  /** Declares a component and returns its id. `fields.in` names a boundary declared before. */
  component(id: string, label: string, fields?: Fields & { in?: string }): string;
  /** One connection between two components declared before. */
  connect(from: string, to: string, fields?: Fields): ArchitectureBuilder;
  /** One connection from `from` to each of `targets`, in their order, with the same fields. */
  fanOut(from: string, targets: readonly string[], fields?: Fields): ArchitectureBuilder;
  /** One connection from each of `sources` to `to`, in their order, with the same fields. */
  fanIn(sources: readonly string[], to: string, fields?: Fields): ArchitectureBuilder;
  /** A connection from each component to the next one, with the same fields. */
  chain(ids: readonly string[], fields?: Fields): ArchitectureBuilder;
  /** The usual way a request takes: component ids, or `{ from, to }`. */
  mainPath(path: readonly string[] | { from: string; to: string }): ArchitectureBuilder;
  /**
   * The specification built so far, as a new plain object. `fields` are its top-level fields,
   * such as `title`, `width`, `height` or `mobile`.
   */
  spec(fields?: Fields): Record<string, unknown>;
}

/**
 * A builder for one architecture diagram. `settings` go into the `architecture` object as they
 * are, such as `{ orientation: "portrait" }`. Identifiers used twice, references to elements not
 * declared yet and a list where one identifier is expected throw at once.
 */
export declare function architecture(settings?: Fields): ArchitectureBuilder;

export interface FlowBuilder {
  /** Declares a lane and returns its id. */
  lane(id: string, label: string): string;
  /** Declares a step and returns its id. `fields.lane` names a lane declared before. */
  node(id: string, label: string, fields?: Fields & { lane?: string }): string;
  /** A frame with `label` around steps declared before, in their order. */
  group(label: string, members: readonly string[]): FlowBuilder;
  /** One edge between two steps declared before. */
  connect(from: string, to: string, fields?: Fields): FlowBuilder;
  /** One edge from `from` to each of `targets`, in their order, with the same fields. */
  fanOut(from: string, targets: readonly string[], fields?: Fields): FlowBuilder;
  /** One edge from each of `sources` to `to`, in their order, with the same fields. */
  fanIn(sources: readonly string[], to: string, fields?: Fields): FlowBuilder;
  /** An edge from each step to the next one, with the same fields. */
  chain(ids: readonly string[], fields?: Fields): FlowBuilder;
  /** The usual way through the flow: step ids, or `{ from, to }`. */
  mainPath(path: readonly string[] | { from: string; to: string }): FlowBuilder;
  /** Steps that lie at one position along the flow; each call adds one set. */
  sameRank(ids: readonly string[]): FlowBuilder;
  /**
   * The specification built so far, as a new plain object. `fields` are its top-level fields,
   * such as `title`, `width`, `height` or `mobile`.
   */
  spec(fields?: Fields): Record<string, unknown>;
}

/**
 * A builder for one flow chart. `settings` go into the `flow` object as they are, such as
 * `{ orientation: "landscape" }`. Identifiers used twice, references to lanes or steps not
 * declared yet and a list where one identifier is expected throw at once.
 */
export declare function flow(settings?: Fields): FlowBuilder;
