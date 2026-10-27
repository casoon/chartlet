// Theme files for the project: a theme from brand values the assistant read from a site or a style
// guide, and the discovery of `chartlet.theme.json` in the server's root. Pure computation and
// reads below the root; nothing here reaches the network, fetching the site is the assistant's job.
//
// Every value the brand gives is kept as given unless a measured contrast pair fails; each change
// is in the receipt with the pair, the ratio before and after, and the original value.

import { existsSync, lstatSync, mkdirSync, readFileSync, realpathSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";

/** The file a project sets its look in. */
export const PROJECT_THEME = "chartlet.theme.json";

/** The ratios WCAG 2.2 asks of text (1.4.3) and of graphics (1.4.11), as the compiler measures. */
export const TEXT = 4.5;
export const MARK = 3;

/**
 * The bundled palettes' values for the tokens a brand does not give, mirroring
 * themes/light.json and themes/dark.json (a test compares them).
 */
export const BUNDLED = {
  light: {
    "color-2": "#c2410c",
    "color-3": "#475569",
    rise: "#0a4f49",
    fall: "#d03a0f",
    "role-green": "#15803d",
    "role-green-fill": "#ecfdf3",
    "role-amber": "#b45309",
    "role-amber-fill": "#fff7e8",
    "role-teal": "#0f766e",
    "role-teal-fill": "#e9faf7",
    "role-violet": "#6d28d9",
    "role-violet-fill": "#f3efff",
    "role-gray": "#667085",
    "role-gray-fill": "#f7f8fa",
    "role-red": "#b42318",
    "role-red-fill": "#fef3f2",
  },
  dark: {
    "color-2": "#ff9c72",
    "color-3": "#9aa7bd",
    rise: "#99f6e4",
    fall: "#e5484d",
    "role-green": "#4ade80",
    "role-green-fill": "#132a1d",
    "role-amber": "#fbbf24",
    "role-amber-fill": "#2b2210",
    "role-teal": "#2dd4bf",
    "role-teal-fill": "#0f2a27",
    "role-violet": "#b69cff",
    "role-violet-fill": "#231c38",
    "role-gray": "#9aa7bd",
    "role-gray-fill": "#161d29",
    "role-red": "#f97066",
    "role-red-fill": "#2d1616",
  },
};

const ROLES = ["green", "amber", "teal", "violet", "gray", "red"];

/** @typedef {[number, number, number]} Rgb */

/**
 * @param {string} value `#rgb`, `#rrggbb` (an `rgb()` is read too)
 * @returns {Rgb | undefined}
 */
export function parseColor(value) {
  const text = String(value).trim();
  let match = /^#([0-9a-f]{3})$/i.exec(text);
  if (match) {
    return [...match[1]].map((digit) => Number.parseInt(digit + digit, 16));
  }
  match = /^#([0-9a-f]{6})$/i.exec(text);
  if (match) {
    return [0, 2, 4].map((at) => Number.parseInt(match[1].slice(at, at + 2), 16));
  }
  match = /^rgb\(\s*(\d{1,3})[ ,]+(\d{1,3})[ ,]+(\d{1,3})\s*\)$/i.exec(text);
  if (match && match.slice(1).every((part) => Number(part) <= 255)) {
    return [Number(match[1]), Number(match[2]), Number(match[3])];
  }
  return undefined;
}

/** @param {Rgb} rgb */
export const hex = (rgb) => `#${rgb.map((part) => Math.round(part).toString(16).padStart(2, "0")).join("")}`;

/** @param {Rgb} rgb */
function luminance(rgb) {
  const [r, g, b] = rgb.map((part) => {
    const value = part / 255;
    return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** The WCAG contrast ratio of two colors. @param {Rgb} a @param {Rgb} b */
export function contrast(a, b) {
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (light + 0.05) / (dark + 0.05);
}

/** @param {Rgb} from @param {Rgb} to @param {number} t @returns {Rgb} */
const mix = (from, to, t) => /** @type {Rgb} */ (from.map((part, index) => part + (to[index] - part) * t));

const WHITE = /** @type {Rgb} */ ([255, 255, 255]);
const BLACK = /** @type {Rgb} */ ([0, 0, 0]);

/**
 * The smallest step from `color` away from `ground` (darker on a light ground, lighter on a dark
 * one) that reaches `needs`.
 *
 * @param {Rgb} color @param {Rgb} ground @param {number} needs @returns {Rgb}
 */
function toContrast(color, ground, needs) {
  const target = luminance(ground) > 0.18 ? BLACK : WHITE;
  for (let step = 0; step <= 100; step += 1) {
    const candidate = mix(color, target, step / 100);
    if (contrast(candidate, ground) >= needs) {
      return candidate;
    }
  }
  return target;
}

const FONT_REASON =
  "chartlet has no font setting yet (typography, #46): text is drawn and measured with its bundled sans-serif metrics, so this family is recorded here and not written to the theme file";

/**
 * Turns brand values into a light theme file and a dark one, with a receipt.
 *
 * @param {{
 *   paper: string, ink: string, muted?: string, accent: string, link?: string,
 *   fonts?: { names?: string, technical?: string },
 *   dark?: { paper: string, ink: string, muted?: string, accent: string, link?: string },
 * }} brand
 */
export function themeFromBrand(brand) {
  /** @type {Record<string, unknown>[]} */
  const roles = [];
  const notes = [];
  const read = (/** @type {string} */ name, /** @type {string | undefined} */ value) => {
    const color = parseColor(/** @type {string} */ (value));
    if (!color) {
      throw new Error(`${name} must be a hex color such as #1a2b3c or rgb(26, 43, 60), got ${JSON.stringify(value)}.`);
    }
    return color;
  };

  /**
   * One scheme's tokens: inputs are kept unless a pair fails; everything else is derived from them.
   *
   * @param {"light" | "dark"} scheme
   * @param {{ paper: Rgb, ink: Rgb, muted?: Rgb, accent: Rgb, link?: Rgb }} given what was sampled
   * @param {boolean} derived whether the given values were derived
   */
  const scheme_ = (scheme, given, derived) => {
    /** @type {Record<string, string>} */
    const tokens = {};
    const background = given.paper;
    tokens.background = hex(background);
    const record = (/** @type {Record<string, unknown>} */ entry) => roles.push({ scheme, ...entry });
    /**
     * Sets a token to `value` (the sampled or derived value), moved only as far as the pair needs.
     */
    const set = (
      /** @type {string} */ role,
      /** @type {string} */ token,
      /** @type {Rgb} */ value,
      /** @type {Rgb} */ ground,
      /** @type {number} */ needs,
      /** @type {string} */ pair,
      /** @type {"sampled" | "derived" | "default"} */ origin,
    ) => {
      const before = contrast(value, ground);
      const out = before >= needs ? value : toContrast(value, ground, needs);
      tokens[token] = hex(out);
      record({
        role,
        token,
        origin,
        input: hex(value),
        output: hex(out),
        ...(before >= needs
          ? { adjusted: false }
          : {
              adjusted: true,
              reason: `${pair} needs ${needs}:1, measured ${before.toFixed(2)}:1; now ${contrast(out, ground).toFixed(2)}:1`,
            }),
        pair,
        ratio: Number(contrast(out, ground).toFixed(2)),
      });
      return out;
    };
    record({ role: "paper", token: "background", origin: derived ? "derived" : "sampled", input: hex(background), output: tokens.background, adjusted: false });

    const sampledOrigin = derived ? "derived" : "sampled";
    const ink = set("ink", "text", given.ink, background, TEXT, "text on background (text 4.5:1)", sampledOrigin);
    set(
      "muted",
      "muted",
      given.muted ?? mix(background, ink, 0.72),
      background,
      TEXT,
      "muted text on background (text 4.5:1)",
      given.muted ? sampledOrigin : "derived",
    );
    const accent = set("accent", "accent", given.accent, background, MARK, "accent marks on background (marks 3:1)", sampledOrigin);
    tokens["color-1"] = hex(accent);
    record({ role: "accent", token: "color-1", origin: "derived", input: hex(accent), output: hex(accent), adjusted: false, note: "the first series color is the accent" });
    set("ink", "color-4", ink, background, MARK, "fourth series color on background (marks 3:1)", "derived");

    set("paper+ink", "grid", mix(background, ink, 0.14), background, 1, "grid lines (decorative, no ratio asked)", "derived");
    set("paper+ink", "zero", mix(background, ink, 0.45), background, MARK, "zero line on background (marks 3:1)", "derived");
    const node = mix(background, accent, scheme === "light" ? 0.07 : 0.14);
    tokens.node = hex(node);
    record({ role: "paper+accent", token: "node", origin: "derived", input: hex(node), output: hex(node), adjusted: false });
    for (const [token, share] of [["lane", 0.035], ["lane-head", 0.07]]) {
      const lane = mix(background, ink, share);
      tokens[token] = hex(lane);
      record({ role: "paper+ink", token, origin: "derived", input: hex(lane), output: hex(lane), adjusted: false });
    }
    const textOnFill = (/** @type {string} */ fill) => contrast(ink, /** @type {Rgb} */ (parseColor(fill)));

    const bundled = BUNDLED[scheme];
    for (const [role, token] of [["bundled", "color-2"], ["bundled", "color-3"], ["bundled", "rise"], ["bundled", "fall"]]) {
      set(role, token, read(token, bundled[/** @type {keyof typeof bundled} */ (token)]), background, MARK, `${token} marks on background (marks 3:1)`, "default");
    }

    // Roles: the blue role is the link color (or the accent); the others keep the bundled hues.
    /** @type {[string, Rgb, string][]} */
    const roleColors = [
      ["blue", given.link ?? accent, given.link ? sampledOrigin : "derived"],
      ...ROLES.map((name) => /** @type {[string, Rgb, string]} */ ([name, read(`role-${name}`, bundled[/** @type {keyof typeof bundled} */ (`role-${name}`)]), "default"])),
    ];
    for (const [name, color, origin] of roleColors) {
      const fillGiven =
        name === "blue"
          ? mix(background, given.link ?? accent, scheme === "light" ? 0.08 : 0.14)
          : read(`role-${name}-fill`, bundled[/** @type {keyof typeof bundled} */ (`role-${name}-fill`)]);
      // The label on the box is the text color; the fill moves toward the ground until it reads.
      let fill = fillGiven;
      if (textOnFill(hex(fill)) < TEXT) {
        const before = textOnFill(hex(fill));
        fill = toContrast(fill, ink, TEXT);
        record({
          scheme,
          role: `${name} fill`,
          token: `role-${name}-fill`,
          origin: name === "blue" ? "derived" : "default",
          input: hex(fillGiven),
          output: hex(fill),
          adjusted: true,
          reason: `text on a box needs ${TEXT}:1, measured ${before.toFixed(2)}:1; now ${textOnFill(hex(fill)).toFixed(2)}:1`,
          pair: "text on role fill (text 4.5:1)",
        });
      } else {
        record({ scheme, role: `${name} fill`, token: `role-${name}-fill`, origin: name === "blue" ? "derived" : "default", input: hex(fill), output: hex(fill), adjusted: false });
      }
      tokens[`role-${name}-fill`] = hex(fill);
      // The outline must stand on the ground and on its fill.
      let stroke = color;
      const reasons = [];
      for (const [ground, label] of [[background, "background"], [fill, "its fill"]]) {
        const before = contrast(stroke, ground);
        if (before < MARK) {
          stroke = toContrast(stroke, ground, MARK);
          reasons.push(`outline on ${label} needs ${MARK}:1, measured ${before.toFixed(2)}:1`);
        }
      }
      tokens[`role-${name}`] = hex(stroke);
      record({
        scheme,
        role: `${name} role`,
        token: `role-${name}`,
        origin,
        input: hex(color),
        output: hex(stroke),
        adjusted: reasons.length > 0,
        ...(reasons.length > 0 ? { reason: `${reasons.join("; ")}; now ${Math.min(contrast(stroke, background), contrast(stroke, fill)).toFixed(2)}:1` } : {}),
        pair: "role outline on background and on its fill (marks 3:1)",
        ratio: Number(Math.min(contrast(stroke, background), contrast(stroke, fill)).toFixed(2)),
      });
    }
    return tokens;
  };

  const light = {
    paper: read("paper", brand.paper),
    ink: read("ink", brand.ink),
    muted: brand.muted === undefined ? undefined : read("muted", brand.muted),
    accent: read("accent", brand.accent),
    link: brand.link === undefined ? undefined : read("link", brand.link),
  };
  const lightTokens = scheme_("light", light, false);

  let dark;
  let derived = false;
  if (brand.dark) {
    dark = {
      paper: read("dark.paper", brand.dark.paper),
      ink: read("dark.ink", brand.dark.ink),
      muted: brand.dark.muted === undefined ? undefined : read("dark.muted", brand.dark.muted),
      accent: read("dark.accent", brand.dark.accent),
      link: brand.dark.link === undefined ? undefined : read("dark.link", brand.dark.link),
    };
  } else {
    derived = true;
    let paper = mix(light.ink, BLACK, 0.5);
    while (luminance(paper) > 0.02) {
      paper = mix(paper, BLACK, 0.2);
    }
    dark = {
      paper,
      ink: mix(light.paper, paper, 0.08),
      muted: mix(light.paper, paper, 0.32),
      accent: mix(light.accent, WHITE, 0.3),
      link: light.link ? mix(light.link, WHITE, 0.3) : undefined,
    };
    notes.push(
      "The dark values were derived from the light ones (paper from the ink, ink from the paper, a lighter accent) and are marked derived in the receipt; give `dark` to use the brand's own dark values.",
    );
  }
  const darkTokens = scheme_("dark", dark, derived);

  const fonts = Object.entries(brand.fonts ?? {})
    .filter(([, family]) => family)
    .map(([role, family]) => ({ role, family, applied: false, metricsProfile: false, reason: FONT_REASON }));
  if (fonts.length > 0) {
    notes.push("Fonts were not written to the theme: the renderer cannot reproduce them, so no match is promised.");
  }
  notes.push(
    "Tokens not listed (diverging, heat and the extra series colors) keep the bundled palette. Only the contrast pairs above were measured; a chart that sets its own colors in the specification is not covered.",
    "Fetching the site or style guide is the assistant's job; these are the values it passed in.",
  );

  const adjusted = roles.filter((entry) => entry.adjusted === true);
  return {
    files: {
      light: { theme: "light", tokens: lightTokens },
      dark: { theme: "dark", tokens: darkTokens },
    },
    darkDerived: derived,
    receipt: { roles, adjusted, fonts, notes },
  };
}

/**
 * The theme file of a project: `chartlet.theme.json` directly in `root`. Returns `undefined` when
 * there is none; refuses a symbolic link, a file that is not JSON or not a theme object.
 *
 * @param {string | undefined} root the server's sandbox directory (`CHARTLET_MCP_ROOT`)
 * @returns {{ theme: { theme?: string, tokens?: Record<string, string> }, source: string } | undefined}
 */
export function readProjectTheme(root) {
  if (root === undefined || root === "") {
    return undefined;
  }
  let realRoot;
  try {
    realRoot = realpathSync(root);
    if (!statSync(realRoot).isDirectory()) {
      return undefined;
    }
  } catch {
    return undefined;
  }
  const path = join(realRoot, PROJECT_THEME);
  let stats;
  try {
    stats = lstatSync(path);
  } catch {
    return undefined;
  }
  if (stats.isSymbolicLink() || !stats.isFile() || dirname(path) !== realRoot) {
    throw new Error(`${PROJECT_THEME} in CHARTLET_MCP_ROOT must be a regular file, not a symbolic link.`);
  }
  let parsed;
  try {
    parsed = JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    throw new Error(`${PROJECT_THEME} is not valid JSON: ${/** @type {Error} */ (error).message}`);
  }
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`${PROJECT_THEME} must be a JSON object with "theme" and "tokens".`);
  }
  const { $schema: _schema, ...theme } = parsed;
  return { theme, source: PROJECT_THEME };
}

/**
 * Writes the light file to `path` and the dark one beside it (`.json` becomes `.dark.json`), below
 * the root. Nothing is written when one of them exists and `overwrite` is not set: a project's
 * theme is never replaced by accident.
 *
 * @param {{ light: object, dark: object }} files
 * @param {string} path relative to the root, ending in `.json`
 * @param {boolean} overwrite
 * @param {string | undefined} root
 * @param {(path: string, root: string | undefined) => { root: string, path: string }} resolve the
 *   sandbox check of `outputPath`
 * @returns {{ path: string, bytes: number }[]}
 */
export function writeThemeFiles(files, path, overwrite, root, resolve) {
  if (!path.endsWith(".json")) {
    throw new Error(`path must end in .json, got ${JSON.stringify(path)}.`);
  }
  const targets = [
    [path, files.light],
    [path.replace(/\.json$/, ".dark.json"), files.dark],
  ].map(([name, file]) => ({ name: /** @type {string} */ (name), file, target: resolve(/** @type {string} */ (name), root) }));
  const existing = targets.filter(({ target }) => existsSync(target.path)).map(({ name }) => name);
  if (existing.length > 0 && !overwrite) {
    throw new Error(
      `${existing.join(" and ")} already exist${existing.length === 1 ? "s" : ""}; nothing was written. Pass overwrite: true to replace ${existing.length === 1 ? "it" : "them"}, or another path for a theme of its own.`,
    );
  }
  return targets.map(({ file, target }) => {
    const text = `${JSON.stringify({ $schema: SCHEMA_URL, ...file }, null, 2)}\n`;
    mkdirSync(dirname(target.path), { recursive: true });
    writeFileSync(target.path, text);
    return { path: relative(target.root, target.path), bytes: Buffer.byteLength(text) };
  });
}

const SCHEMA_URL = "https://casoon.github.io/chartlet/schema/theme-v1.json";

/** The note `chartlet_render` adds when a root has no theme file. */
export const NO_THEME_NOTE = `No ${PROJECT_THEME} in CHARTLET_MCP_ROOT and no theme given: the default look is used. To set the project's look once, call chartlet_theme_from_brand with the brand's colors and write the file, or save a theme file as ${PROJECT_THEME} in the root.`;
