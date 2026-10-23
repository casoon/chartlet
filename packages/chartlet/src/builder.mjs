// An optional way to assemble architecture specifications in code. It only builds the regular
// JSON specification: `spec()` returns a plain object to inspect, serialize or pass to
// `renderChart`. No global state, no randomness, nothing rendered or written here. It checks only
// what it adds itself — identifiers used twice, references to elements not declared yet, lists
// where one element is expected; everything else is left to the validation of the core.

const IDENTIFIER = /^[A-Za-z][A-Za-z0-9_-]{0,63}$/;

// Joins parts into an identifier with hyphens, so a reused building block derives its ids from
// one name: `id("eu", "worker")` is `"eu-worker"`.
export function id(...parts) {
  const joined = parts.join("-");
  if (!IDENTIFIER.test(joined)) {
    throw new Error(
      `"${joined}" is not an identifier: a letter, then letters, digits, hyphens or underscores.`,
    );
  }
  return joined;
}

// A builder for one architecture diagram. `settings` go into the `architecture` object as they
// are, such as `{ orientation: "portrait" }`.
export function architecture(settings = {}) {
  const boundaries = [];
  const components = [];
  const connections = [];
  const declared = new Map();
  let mainPath;

  function declare(kind, elementId, label, fields) {
    id(elementId);
    if (declared.has(elementId)) {
      throw new Error(
        `"${elementId}" is already declared as a ${declared.get(elementId)}; identifiers are ` +
          "unique among components and boundaries.",
      );
    }
    if (fields.in !== undefined) {
      expect(fields.in, "boundary", `The boundary of "${elementId}"`);
    }
    declared.set(elementId, kind);
    return { id: elementId, label, ...fields };
  }

  function expect(ref, kind, what) {
    if (typeof ref !== "string") {
      throw new Error(`${what} must be one identifier, not ${JSON.stringify(ref)}.`);
    }
    const found = declared.get(ref);
    if (found === undefined) {
      throw new Error(`${what} is "${ref}", which is not declared (yet).`);
    }
    if (kind !== undefined && found !== kind) {
      throw new Error(`${what} is "${ref}", which is a ${found}, not a ${kind}.`);
    }
  }

  const builder = {
    // Declares a boundary and returns its id; `fields.in` names the boundary it lies in.
    boundary(boundaryId, label, fields = {}) {
      boundaries.push(declare("boundary", boundaryId, label, fields));
      return boundaryId;
    },

    // Declares a component and returns its id; `fields.in` names its boundary.
    component(componentId, label, fields = {}) {
      components.push(declare("component", componentId, label, fields));
      return componentId;
    },

    // One connection between two declared components.
    connect(from, to, fields = {}) {
      expect(from, "component", "The start of a connection");
      expect(to, "component", "The end of a connection");
      connections.push({ from, to, ...fields });
      return builder;
    },

    // One connection from `from` to each of `targets`, in their order, all with the same fields.
    fanOut(from, targets, fields = {}) {
      for (const to of list(targets, "fanOut")) builder.connect(from, to, fields);
      return builder;
    },

    // One connection from each of `sources` to `to`, in their order, all with the same fields.
    fanIn(sources, to, fields = {}) {
      for (const from of list(sources, "fanIn")) builder.connect(from, to, fields);
      return builder;
    },

    // A connection from each component to the next one, all with the same fields.
    chain(ids, fields = {}) {
      const chained = list(ids, "chain");
      for (let index = 1; index < chained.length; index++) {
        builder.connect(chained[index - 1], chained[index], fields);
      }
      return builder;
    },

    // The usual way a request takes: component ids, or `{ from, to }`.
    mainPath(path) {
      const ids = Array.isArray(path) ? path : [path.from, path.to];
      for (const ref of ids) expect(ref, "component", "A step of the main path");
      mainPath = Array.isArray(path) ? [...path] : { from: path.from, to: path.to };
      return builder;
    },

    // The specification as built so far, as a new plain object; `fields` are its top-level
    // fields, such as `title`, `width`, `height` or `mobile`.
    spec(fields = {}) {
      return structuredClone({
        schemaVersion: 1,
        type: "architecture",
        ...fields,
        architecture: {
          ...(boundaries.length > 0 && { boundaries }),
          components,
          ...(connections.length > 0 && { connections }),
          ...(mainPath !== undefined && { mainPath }),
          ...settings,
        },
      });
    },
  };
  return builder;
}

function list(value, method) {
  if (!Array.isArray(value)) {
    throw new Error(`${method} takes a list of identifiers, not ${JSON.stringify(value)}.`);
  }
  return value;
}
