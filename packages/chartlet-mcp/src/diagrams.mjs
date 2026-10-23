// Software diagrams: a starting specification and the kinds of every element per diagram type,
// and the structural facts of a diagram. Diagrams are drawn from structure, not from data, so
// they need neither chartlet_inspect_data nor the series facts of chartlet_explain.

/** The diagram types chartlet draws. */
export const DIAGRAM_TYPES = /** @type {const} */ (["sequence", "flow", "state", "architecture", "tree", "classes"]);

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
  classes: {
    "schemaVersion": 1,
    "type": "classes",
    "title": "Order model",
    "width": 900,
    "height": 720,
    "classes": {
      "classes": [
        {
          "id": "customer",
          "label": "Customer",
          "attributes": [
            "+ id: Int",
            "+ name: String",
            "+ email: String"
          ],
          "methods": [
            "+ placeOrder(): Order"
          ]
        },
        {
          "id": "order",
          "label": "Order",
          "attributes": [
            "+ id: Int",
            "+ placedAt: Date",
            "+ status: Status"
          ],
          "methods": [
            "+ total(): Money",
            "+ cancel()"
          ]
        },
        {
          "id": "line",
          "label": "OrderLine",
          "attributes": [
            "+ quantity: Int",
            "+ price: Money"
          ]
        },
        {
          "id": "payment",
          "label": "Payment",
          "kind": "interface",
          "methods": [
            "+ charge(amount: Money): Receipt"
          ]
        },
        {
          "id": "card",
          "label": "CardPayment",
          "attributes": [
            "- number: String"
          ],
          "methods": [
            "+ charge(amount: Money): Receipt"
          ]
        }
      ],
      "relations": [
        {
          "from": "customer",
          "to": "order",
          "fromCardinality": "1",
          "toCardinality": "0..*",
          "label": "places"
        },
        {
          "from": "order",
          "to": "line",
          "kind": "composition",
          "fromCardinality": "1",
          "toCardinality": "1..*"
        },
        {
          "from": "card",
          "to": "payment",
          "kind": "realization"
        },
        {
          "from": "order",
          "to": "payment",
          "kind": "dependency",
          "label": "paid by"
        }
      ]
    }
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
      node: "box seen from a corner: a machine or cluster node that runs other components (deployment)",
      artifact: "box with a folded corner: something deployed, such as an image or a package",
    },
    boundary: { boundary: "frame; boundaries nest with in, up to four deep" },
    connection: {
      label: "what it does",
      technology: "how, in brackets below the label",
      step: "1-99, each once: numbers the connection in a sequence of calls, written before the label",
      direction: "forward (default, arrowhead at to), both (one connection both ways, arrowheads at either end) or none (no direction, no arrowhead)",
      sharedTrunks: "on architecture: true draws connections that leave or reach one component together on one branching line; only unlabeled ones with the same dash, weight and role whose other ends share a boundary",
      id: "optional stable name by which views select the connection",
    },
    view: {
      components: "architecture.views[]: a key, a title and the components shown; connections between them come along, or only the listed connections",
      steps: "a dynamic view: connection ids in the order they occur, each with optional step text, numbered from 1; a connection may occur again",
    },
    note: { note: "on a component id, text up to 160 characters: a pale box with a folded corner beside it, joined by a dotted line that is no connection; up to 12" },
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
  classes: {
    class: {
      class: "box with the name on top, attributes and methods in compartments below (default)",
      interface: "box with «interface» above the name",
      abstract: "box with «abstract» above the name",
      enum: "box with «enumeration» above the name; its attributes are the values",
      entity: "box of an entity-relationship diagram: no stereotype",
    },
    relation: {
      association: "plain line",
      inheritance: "hollow triangle at to, the parent",
      realization: "dashed line with a hollow triangle at to",
      composition: "filled diamond at from, the whole",
      aggregation: "hollow diamond at from",
      dependency: "dashed line with an open arrowhead at to",
      direction: "on an association only: forward (navigable to to, open arrowhead there), both or none (plain line)",
    },
    package: { package: "frame around the classes that name it with in; packages nest with in, up to four deep; relations join classes, never packages" },
    note: { note: "on a class id, text up to 160 characters: a pale box with a folded corner beside it, joined by a dotted line that is no relation; up to 12" },
  },
};

const NOTES = [
  "Ids are a letter followed by letters, digits, - or _, unique within the diagram.",
  "A tree has exactly one root: the node with neither parent nor partner; every other node names its parent, or its partner.",
  "chartlet lays the diagram out itself; the specification says what is connected, never where it goes.",
  'orientation "auto" (default) takes portrait — time or flow running down — where that fits the canvas, landscape — running right — where only that fits, and otherwise the one that grows the canvas less (warning canvas_too_small names the size needed). "portrait" and "landscape" force one.',
  'Flow and architecture diagrams take "sameRank": [["a", "b"]]: sets of nodes put at one position along the flow, such as parallel reviews or the databases of a data tier. A layout hint, not an edge; each node in one set at most, and no edge may join two nodes of one set (same_rank_edge).',
  "Every kind has its own shape and a role color; say what an edge means in its label, the line pattern only repeats it.",
  "The description and the data table list every element in reading order; check a spec with chartlet_validate_spec.",
  '"theme": "blueprint" draws a diagram in pale lines on a deep blue ground (diagrams only; charts keep light and dark). A deployment is an architecture diagram: boundaries are the hosts, components of kind node and artifact sit in them.',
  'An architecture component may take "symbol", a generic symbol above its label that names its technology category (the kind keeps its shape; put a product name in sublabel): browser, mobile, api-gateway, load-balancer, cdn, firewall, identity, container, orchestrator, function, cloud, cache, object-storage, message-broker, search-index, monitoring. Any other name fails with unknown_symbol.',
  'Flow and architecture diagrams take "role" on nodes and edges: "critical" (strong red outline or line, badge !), "planned" (dashed violet, badge +) or "deprecated" (dotted gray, badge −). The kind keeps its shape and fill; a legend names only the roles used. Use a role for what something means in this picture, not for its kind (external is a kind); planned and deprecated edges take no dash (role_conflict).',
  'An architecture model with "views" renders one view (chartlet_render view, or defaultView, or the first): only its components, the connections between them and the boundaries holding them; never the whole model.',
  'An architecture model may take "perspectives": [{"key": "owner", "title": "Team", "categories": [{"label": "Payments", "components": ["api"]}, {"label": "Platform", "components": ["db"]}]}] (up to 8, 1-8 categories each; a component in at most one category of a perspective). Only "perspective": "owner" on the model or a view shows it: a colored stripe and the category in every box, a legend, a description sentence and a table column; components without one show as not stated. Only stated categories are shown, nothing is derived.',
  'A tree takes "layout": "radial" for a mind map (root in the middle, subtrees in sectors); it refuses orientation, partner and link. "collapsed": true on a node hides what hangs below it.',
];

/** The architecture templates chartlet_diagram_starter knows besides the plain architecture starter. */
export const ARCHITECTURE_TEMPLATES = /** @type {const} */ (["context", "containers", "deployment"]);

/** @type {Record<string, { question: string, level: string, spec: Record<string, unknown> }>} */
const TEMPLATES = {
  context: {
    question: "Who uses the system, and which other systems does it depend on?",
    level:
      "System context: the one software system in scope as a single box, the people who use it and the external systems it calls. Nothing inside the system is shown.",
    spec: {
      schemaVersion: 1,
      type: "architecture",
      title: "Course booking: system context",
      height: 480,
      architecture: {
        components: [
          { id: "learner", label: "Learner", sublabel: "books and attends courses", kind: "person" },
          { id: "trainer", label: "Trainer", sublabel: "runs courses", kind: "person" },
          { id: "booking", label: "Course booking", sublabel: "the system in scope" },
          { id: "payment", label: "Payment provider", sublabel: "external system", kind: "external" },
          { id: "mail", label: "E-mail service", sublabel: "external system", kind: "external" },
          { id: "directory", label: "Staff directory", sublabel: "external system", kind: "external" },
        ],
        connections: [
          { from: "learner", to: "booking", label: "books a course" },
          { from: "trainer", to: "booking", label: "plans dates" },
          { from: "booking", to: "payment", label: "takes payment" },
          { from: "booking", to: "mail", label: "sends confirmations" },
          { from: "booking", to: "directory", label: "looks up trainers" },
        ],
        mainPath: ["learner", "booking", "payment"],
      },
    },
  },
  containers: {
    question: "Which applications and data stores make up the system, and how do they talk to each other?",
    level:
      "Containers: the system as one boundary around its separately running or deployable parts (applications, databases, queues, file stores), with the people and external systems around it. A container here is such a part, not a container image or process; components inside one container belong in a diagram of their own.",
    spec: {
      schemaVersion: 1,
      type: "architecture",
      title: "Course booking: containers",
      height: 940,
      architecture: {
        boundaries: [{ id: "system", label: "Course booking (software system)" }],
        components: [
          { id: "learner", label: "Learner", kind: "person" },
          { id: "web", label: "Web app", sublabel: "single-page app", kind: "frontend", in: "system" },
          { id: "api", label: "Booking API", sublabel: "API application", in: "system" },
          { id: "jobs", label: "Job queue", sublabel: "message queue", kind: "queue", in: "system" },
          { id: "worker", label: "Worker", sublabel: "background jobs", in: "system" },
          { id: "db", label: "Bookings", sublabel: "relational database", kind: "database", in: "system" },
          { id: "files", label: "Certificates", sublabel: "file store", kind: "storage", in: "system" },
          { id: "payment", label: "Payment provider", kind: "external" },
          { id: "mail", label: "E-mail service", kind: "external" },
        ],
        connections: [
          { from: "learner", to: "web", label: "books a course", technology: "HTTPS" },
          { from: "web", to: "api", label: "calls", technology: "JSON/HTTPS" },
          { from: "api", to: "db", label: "reads, writes", technology: "SQL" },
          { from: "api", to: "payment", label: "takes payment", technology: "HTTPS" },
          { from: "api", to: "jobs", label: "queues jobs" },
          { from: "jobs", to: "worker", label: "delivers" },
          { from: "worker", to: "files", label: "stores certificates" },
          { from: "worker", to: "mail", label: "sends confirmations", technology: "SMTP" },
        ],
        mainPath: ["learner", "web", "api", "db"],
      },
    },
  },
  deployment: {
    question: "Where does each part run, and which managed services does it use?",
    level:
      "Deployment: one environment drawn as nested boundaries (region, cluster, node pools) with what is deployed in them (kind artifact) and the managed services beside them; a web tier hands work to workers through a queue, and the workers write to storage. Boundaries are hosts, not systems; the diagram describes one environment and does not model several environments or instances of one deployment.",
    spec: {
      schemaVersion: 1,
      type: "architecture",
      title: "Course booking: deployment",
      height: 1000,
      architecture: {
        boundaries: [
          { id: "region", label: "Cloud region" },
          { id: "cluster", label: "Container cluster", in: "region" },
          { id: "webPool", label: "Node pool: web", in: "cluster" },
          { id: "workerPool", label: "Node pool: workers", in: "cluster" },
        ],
        components: [
          { id: "learner", label: "Learner", kind: "person" },
          { id: "ingress", label: "Ingress", sublabel: "load balancer", in: "cluster" },
          { id: "web", label: "Web app and API", sublabel: "image, 3 replicas", kind: "artifact", in: "webPool" },
          { id: "worker", label: "Worker", sublabel: "image, 2 replicas", kind: "artifact", in: "workerPool" },
          { id: "jobs", label: "Job queue", sublabel: "managed queue", kind: "queue", in: "region" },
          { id: "db", label: "Bookings", sublabel: "managed database", kind: "database", in: "region" },
          { id: "files", label: "Certificates", sublabel: "object storage", kind: "storage", in: "region" },
        ],
        connections: [
          { from: "learner", to: "ingress", label: "opens", technology: "HTTPS" },
          { from: "ingress", to: "web", label: "routes" },
          { from: "web", to: "db", label: "reads, writes", technology: "SQL/TLS" },
          { from: "web", to: "jobs", label: "queues jobs" },
          { from: "jobs", to: "worker", label: "delivers" },
          { from: "worker", to: "files", label: "stores certificates" },
          { from: "worker", to: "db", label: "updates", technology: "SQL/TLS" },
        ],
        mainPath: ["learner", "ingress", "web", "jobs", "worker", "files"],
        sameRank: [["db", "files"]],
      },
    },
  },
};

/**
 * A valid starting specification for `type`, the kinds of its elements, and notes. With an
 * architecture `template`, the template's specification, the question it answers and its level.
 *
 * @param {string} type
 * @param {string} [template]
 */
export function diagramStarter(type, template) {
  const starter = {
    ok: /** @type {const} */ (true),
    type,
    spec: structuredClone(template ? TEMPLATES[template].spec : STARTERS[type]),
    kinds: KINDS[/** @type {keyof typeof KINDS} */ (type)],
    notes: NOTES,
  };
  if (!template) return starter;
  const { question, level } = TEMPLATES[template];
  return { ...starter, template, question, level };
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
        notes: length(spec.architecture?.notes),
      };
    case "tree":
      return { nodes: length(spec.tree?.nodes) };
    case "classes":
      return {
        classes: length(spec.classes?.classes),
        relations: length(spec.classes?.relations),
        packages: length(spec.classes?.packages),
        notes: length(spec.classes?.notes),
      };
    default:
      return null;
  }
}
