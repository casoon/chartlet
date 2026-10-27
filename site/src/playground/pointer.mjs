// Finds where a JSON Pointer ("/flow/nodes/2/label") points inside the text of a JSON document,
// so that a diagnostic of the compiler can select the field it names. A small scanner, not a
// parser: it assumes nothing about the document except that it is JSON, and answers null when
// the text is not valid enough to reach the path.

/**
 * @param {string} text
 * @param {string} pointer
 * @returns {{ start: number, end: number, exact: boolean } | null} the range of the value the
 *   pointer names; when a step does not exist, the range of the nearest parent that does, with
 *   `exact` false.
 */
export function locate(text, pointer) {
  const steps =
    pointer === "" || pointer === "/"
      ? []
      : pointer
          .split("/")
          .slice(1)
          .map((step) => step.replaceAll("~1", "/").replaceAll("~0", "~"));
  let at = skipSpace(text, 0);
  const root = valueRange(text, at);
  if (!root) {
    return null;
  }
  let range = root;
  for (const step of steps) {
    const child = childRange(text, range, step);
    if (!child) {
      return { ...range, exact: false };
    }
    range = child;
  }
  return { ...range, exact: true };
}

function skipSpace(text, from) {
  let at = from;
  while (at < text.length && /\s/.test(text[at])) {
    at += 1;
  }
  return at;
}

function stringEnd(text, from) {
  let at = from + 1;
  while (at < text.length) {
    if (text[at] === "\\") {
      at += 2;
    } else if (text[at] === '"') {
      return at + 1;
    } else {
      at += 1;
    }
  }
  return -1;
}

/** The range of the value that starts at `from`, or null. */
function valueRange(text, from) {
  const first = text[from];
  if (first === undefined) {
    return null;
  }
  if (first === '"') {
    const end = stringEnd(text, from);
    return end < 0 ? null : { start: from, end };
  }
  if (first === "{" || first === "[") {
    let depth = 0;
    let at = from;
    while (at < text.length) {
      const character = text[at];
      if (character === '"') {
        const end = stringEnd(text, at);
        if (end < 0) {
          return null;
        }
        at = end;
        continue;
      }
      if (character === "{" || character === "[") {
        depth += 1;
      } else if (character === "}" || character === "]") {
        depth -= 1;
        if (depth === 0) {
          return { start: from, end: at + 1 };
        }
      }
      at += 1;
    }
    return null;
  }
  let at = from;
  while (at < text.length && !/[\s,}\]]/.test(text[at])) {
    at += 1;
  }
  return at === from ? null : { start: from, end: at };
}

/** The range of the member or element `step` of the object or array at `parent`. */
function childRange(text, parent, step) {
  const open = text[parent.start];
  if (open !== "{" && open !== "[") {
    return null;
  }
  let at = skipSpace(text, parent.start + 1);
  let index = 0;
  while (at < parent.end - 1) {
    let key = null;
    if (open === "{") {
      if (text[at] !== '"') {
        return null;
      }
      const end = stringEnd(text, at);
      if (end < 0) {
        return null;
      }
      try {
        key = JSON.parse(text.slice(at, end));
      } catch {
        return null;
      }
      at = skipSpace(text, end);
      if (text[at] !== ":") {
        return null;
      }
      at = skipSpace(text, at + 1);
    }
    const value = valueRange(text, at);
    if (!value) {
      return null;
    }
    if (open === "{" ? key === step : String(index) === step) {
      return value;
    }
    index += 1;
    at = skipSpace(text, value.end);
    if (text[at] === ",") {
      at = skipSpace(text, at + 1);
    } else {
      break;
    }
  }
  return null;
}
