// Software diagrams: a starting specification and the kinds of every element per diagram type,
// and the structural facts of a diagram. Diagrams are drawn from structure, not from data, so
// they need neither chartlet_inspect_data nor the series facts of chartlet_explain.

/** The diagram types chartlet draws. */
export const DIAGRAM_TYPES = /** @type {const} */ (["sequence", "flow", "state", "architecture", "tree"]);

/** @type {Record<string, Record<string, unknown>>} */
const STARTERS = {
  sequence: {
    schemaVersion: 1,
    type: "sequence",
    title: "Signing in",
    sequence: {
      participants: [
        { id: "user", label: "User", kind: "actor" },
        { id: "app", label: "App", sublabel: "web" },
        { id: "db", label: "Accounts", kind: "database" },
      ],
      messages: [
        { from: "user", to: "app", label: "sign in" },
        { from: "app", to: "db", label: "find account" },
        { from: "db", to: "app", label: "account", kind: "reply" },
        { from: "app", to: "user", label: "welcome", kind: "reply" },
      ],
      fragments: [{ kind: "opt", label: "known account", from: 2, to: 3 }],
      numbered: true,
    },
  },
  flow: {
    schemaVersion: 1,
    type: "flow",
    title: "Publishing a post",
    flow: {
      nodes: [
        { id: "draft", label: "Draft", kind: "start" },
        { id: "check", label: "Looks good?", kind: "decision" },
        { id: "edit", label: "Edit" },
        { id: "live", label: "Published", kind: "end" },
      ],
      edges: [
        { from: "draft", to: "check" },
        { from: "check", to: "live", label: "yes" },
        { from: "check", to: "edit", label: "no" },
        { from: "edit", to: "check", label: "again" },
      ],
      mainPath: ["draft", "check", "live"],
    },
  },
  state: {
    schemaVersion: 1,
    type: "state",
    title: "Door",
    state: {
      initial: "closed",
      states: [
        { id: "closed", label: "Closed" },
        { id: "open", label: "Open" },
        { id: "locked", label: "Locked" },
        { id: "gone", label: "Removed", final: true },
      ],
      transitions: [
        { from: "closed", to: "open", event: "push", guard: "unlocked" },
        { from: "open", to: "closed", event: "release" },
        { from: "closed", to: "locked", event: "lock", action: "beep" },
        { from: "locked", to: "closed", event: "unlock" },
        { from: "locked", to: "gone", event: "dismantle" },
      ],
    },
  },
  architecture: {
    schemaVersion: 1,
    type: "architecture",
    title: "Blog",
    architecture: {
      boundaries: [
        { id: "cloud", label: "Cloud" },
        { id: "private", label: "Private network", in: "cloud" },
      ],
      components: [
        { id: "reader", label: "Reader", kind: "person" },
        { id: "site", label: "Site", kind: "frontend" },
        { id: "api", label: "API", in: "private" },
        { id: "db", label: "Posts", kind: "database", in: "private" },
      ],
      connections: [
        { from: "reader", to: "site", label: "reads" },
        { from: "site", to: "api", label: "loads", technology: "HTTPS" },
        { from: "api", to: "db", technology: "SQL" },
      ],
      mainPath: ["reader", "site", "api", "db"],
    },
  },
  tree: {
    schemaVersion: 1,
    type: "tree",
    title: "Group structure",
    tree: {
      nodes: [
        { id: "holding", label: "Holding", sublabel: "parent company" },
        { id: "retail", label: "Retail", parent: "holding", link: "100 %" },
        { id: "digital", label: "Digital", parent: "holding", link: "60 %" },
        { id: "labs", label: "Labs", parent: "digital", link: "50 %" },
      ],
    },
  },
};

/** The kinds of each element per diagram type, with the shape that draws them. */
const KINDS = {
  sequence: {
    participant: {
      service: "box (default)",
      actor: "figure",
      database: "cylinder",
      queue: "box with a stack behind it",
      external: "dashed box",
    },
    message: {
      call: "solid line, filled head; activates the receiver until it replies (default)",
      reply: "dashed line, open head",
      async: "solid line, open head",
    },
    fragment: {
      alt: "alternatives; further branches in else",
      opt: "optional",
      loop: "repeats",
      par: "in parallel",
      critical: "uninterrupted",
      break: "ends the interaction",
    },
  },
  flow: {
    node: {
      process: "box (default)",
      start: "pill",
      end: "pill with a strong outline",
      decision: "diamond",
      io: "slanted box",
      subprocess: "box with double sides",
      store: "cylinder",
      external: "dashed box",
    },
    edge: { solid: "default", dashed: "dash", dotted: "dash" },
  },
  state: {
    state: {
      state: "box with well rounded corners (default)",
      choice: "diamond, passed through at once by its guards",
      composite: "frame around the states that name it with in",
      final: "a state with \"final\": true, double outline",
    },
    transition: { label: "event [guard] / action, each part optional" },
  },
  architecture: {
    component: {
      service: "box (default)",
      person: "box with a head",
      frontend: "box with a window bar",
      database: "cylinder",
      queue: "box with a stack behind it",
      storage: "bucket",
      cache: "hexagon",
      security: "shield",
      external: "dashed box",
    },
    boundary: { boundary: "frame; boundaries nest with in, up to four deep" },
    connection: { label: "what it does", technology: "how, in brackets below the label" },
  },
  tree: {
    node: {
      unit: "box (default)",
      person: "box with a head",
      external: "dashed box",
    },
    link: { link: "on a node: written on the line to its parent, such as a share; not on the root" },
    partner: { partner: "on a node: joins it to another as a couple, side by side; children of either hang from the middle of the line between them" },
  },
};

const NOTES = [
  "Ids are a letter followed by letters, digits, - or _, unique within the diagram.",
  "A tree has exactly one root: the node with neither parent nor partner; every other node names its parent, or its partner.",
  "chartlet lays the diagram out itself; the specification says what is connected, never where it goes.",
  'orientation "auto" (default) takes portrait — time or flow running down — where that fits the canvas, landscape — running right — where only that fits, and otherwise the one that grows the canvas less (warning canvas_too_small names the size needed). "portrait" and "landscape" force one.',
  "Every kind has its own shape and a role color; say what an edge means in its label, the line pattern only repeats it.",
  "The description and the data table list every element in reading order; check a spec with chartlet_validate_spec.",
];

/**
 * A valid starting specification for `type`, the kinds of its elements, and notes.
 *
 * @param {string} type
 */
export function diagramStarter(type) {
  return {
    ok: /** @type {const} */ (true),
    type,
    spec: structuredClone(STARTERS[type]),
    kinds: KINDS[/** @type {keyof typeof KINDS} */ (type)],
    notes: NOTES,
  };
}

/**
 * How many elements of each sort a diagram has.
 *
 * @param {Record<string, any>} spec
 * @returns {Record<string, number> | null}
 */
export function diagramCounts(spec) {
  const length = (/** @type {unknown} */ list) => (Array.isArray(list) ? list.length : 0);
  switch (spec.type) {
    case "sequence":
      return {
        participants: length(spec.sequence?.participants),
        messages: length(spec.sequence?.messages),
        fragments: length(spec.sequence?.fragments),
      };
    case "flow":
      return {
        steps: length(spec.flow?.nodes),
        edges: length(spec.flow?.edges),
        lanes: length(spec.flow?.lanes),
        groups: length(spec.flow?.groups),
      };
    case "state": {
      const states = spec.state?.states ?? [];
      const composites = states.filter((/** @type {any} */ state) => state.kind === "composite").length;
      return {
        states: states.length - composites,
        composites,
        transitions: length(spec.state?.transitions),
      };
    }
    case "architecture":
      return {
        components: length(spec.architecture?.components),
        connections: length(spec.architecture?.connections),
        boundaries: length(spec.architecture?.boundaries),
      };
    case "tree":
      return { nodes: length(spec.tree?.nodes) };
    default:
      return null;
  }
}
