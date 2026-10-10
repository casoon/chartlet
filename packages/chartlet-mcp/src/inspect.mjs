// Describes a table deterministically: column types, missing values, ranges, and which chartlet
// chart types fit. Reads values only; nothing is converted in place, rounded or dropped.

import { parseCsv } from "./csv.mjs";

/** Distinct values of a string column are counted up to this many. */
export const DISTINCT_CAP = 1000;

const NUMBER = /^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/;
const INTEGER = /^[+-]?\d+$/;
const YEAR = /^\d{4}$/;
const DATE = /^(\d{4})-(\d{2})-(\d{2})$/;
const DATE_TIME =
  /^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})(?::(\d{2})(?:\.\d+)?)?(Z|[+-]\d{2}:\d{2})?$/;
const YEAR_NAME = /year|jahr/i;

/**
 * @typedef {string | number | boolean | null | undefined} Cell
 * @typedef {"number" | "integer" | "date-time" | "boolean" | "string" | "empty"} ColumnType
 * @typedef {"year" | "date" | "date-time"} TimeFormat
 * @typedef {{ value: string | number, row: number }} Located
 * @typedef {{
 *   name: string,
 *   type: ColumnType,
 *   missing: number,
 *   min?: number,
 *   max?: number,
 *   timeFormat?: TimeFormat,
 *   first?: string | number,
 *   last?: string | number,
 *   monotonic?: "increasing" | "decreasing" | "none",
 *   distinct?: number,
 *   distinctCapped?: boolean,
 * }} ColumnSummary
 * @typedef {{ type: string, mark?: string, columns: Record<string, string | string[]>, reason: string }} Suggestion
 * @typedef {{ rowCount: number, columns: ColumnSummary[], suggestions: Suggestion[] }} DataSummary
 */

/**
 * Reads CSV text or an array of row objects into column names and cells.
 *
 * @param {{ csv?: string, rows?: Record<string, Cell>[] }} input
 * @returns {{ names: string[], cells: Cell[][] }} cells per column
 */
export function readTable({ csv, rows }) {
  if ((csv === undefined) === (rows === undefined)) {
    throw new Error("Pass exactly one of `csv` (text with a header row) or `rows` (array of objects).");
  }
  if (csv !== undefined) {
    const { header, records } = parseCsv(csv);
    return { names: header, cells: header.map((_, column) => records.map((record) => record[column])) };
  }
  /** @type {string[]} */
  const names = [];
  const seen = new Set();
  for (const row of /** @type {Record<string, Cell>[]} */ (rows)) {
    for (const name of Object.keys(row)) {
      if (!seen.has(name)) {
        seen.add(name);
        names.push(name);
      }
    }
  }
  const all = /** @type {Record<string, Cell>[]} */ (rows);
  return { names, cells: names.map((name) => all.map((row) => row[name])) };
}

/**
 * @param {{ csv?: string, rows?: Record<string, Cell>[] }} input
 * @returns {DataSummary}
 */
export function inspectData(input) {
  const { names, cells } = readTable(input);
  const rowCount = cells[0]?.length ?? input.rows?.length ?? 0;
  const columns = names.map((name, index) => summarize(name, cells[index]));
  return { rowCount, columns, suggestions: suggest(rowCount, columns) };
}

/**
 * @param {Cell} cell
 */
function isMissing(cell) {
  return cell === undefined || cell === null || cell === "";
}

/**
 * The time of a date or date-time string in milliseconds, or `undefined` for an invalid one.
 * A date-time without an offset is ordered as UTC.
 *
 * @param {string} text
 */
function timeOf(text) {
  const match = DATE_TIME.exec(text) ?? DATE.exec(text);
  if (!match) {
    return undefined;
  }
  const [, year, month, day, hour = "00", minute = "00", second = "00"] = match;
  const date = new Date(Date.UTC(+year, +month - 1, +day, +hour, +minute, +second));
  if (
    date.getUTCFullYear() !== +year ||
    date.getUTCMonth() !== +month - 1 ||
    date.getUTCDate() !== +day ||
    +hour > 23 ||
    +minute > 59 ||
    +second > 59
  ) {
    return undefined;
  }
  return Date.parse(match[7] || !match[4] ? text.replace(" ", "T") : `${text.replace(" ", "T")}Z`);
}

/**
 * @param {string} name
 * @param {Cell[]} column
 * @returns {ColumnSummary}
 */
function summarize(name, column) {
  /** @type {Located[]} */
  const present = [];
  for (const [row, cell] of column.entries()) {
    if (!isMissing(cell)) {
      present.push({ value: /** @type {string | number} */ (cell), row });
    }
  }
  const missing = column.length - present.length;
  if (present.length === 0) {
    return { name, type: "empty", missing };
  }
  const values = present.map(({ value }) => value);

  if (values.every((value) => typeof value === "boolean" || /^(true|false)$/i.test(String(value)))) {
    return { name, type: "boolean", missing };
  }

  const isYear = (/** @type {string|number} */ value) =>
    typeof value === "string" ? YEAR.test(value) : Number.isInteger(value) && value >= 1000 && value <= 9999;
  if (values.every(isYear)) {
    const years = values.map(Number);
    // Four-digit integers are years when the column name says so or they strictly increase.
    if (YEAR_NAME.test(name) || years.every((year, index) => index === 0 || year > years[index - 1])) {
      return { name, type: "date-time", missing, timeFormat: "year", ...timeRange(values, years) };
    }
  }

  if (values.every((value) => typeof value === "string" && timeOf(value) !== undefined)) {
    const texts = /** @type {string[]} */ (values);
    const timeFormat = texts.every((text) => DATE.test(text)) ? "date" : "date-time";
    return {
      name,
      type: "date-time",
      missing,
      timeFormat,
      ...timeRange(values, texts.map((text) => /** @type {number} */ (timeOf(text)))),
    };
  }

  const numeric = (/** @type {string|number} */ value) =>
    typeof value === "number" ? Number.isFinite(value) : NUMBER.test(value);
  if (values.every(numeric)) {
    const numbers = values.map(Number);
    const integer = values.every((value) =>
      typeof value === "number" ? Number.isInteger(value) : INTEGER.test(value),
    );
    let min = numbers[0];
    let max = numbers[0];
    for (const number of numbers) {
      min = Math.min(min, number);
      max = Math.max(max, number);
    }
    return { name, type: integer ? "integer" : "number", missing, min, max };
  }

  const distinct = new Set();
  let distinctCapped = false;
  for (const value of values) {
    distinct.add(String(value));
    if (distinct.size > DISTINCT_CAP) {
      distinctCapped = true;
      break;
    }
  }
  return {
    name,
    type: "string",
    missing,
    distinct: distinctCapped ? DISTINCT_CAP : distinct.size,
    ...(distinctCapped ? { distinctCapped } : {}),
  };
}

/**
 * First and last value as written, and whether the times strictly rise or fall in row order.
 *
 * @param {(string|number)[]} values
 * @param {number[]} times
 */
function timeRange(values, times) {
  let increasing = true;
  let decreasing = true;
  for (let index = 1; index < times.length; index += 1) {
    increasing &&= times[index] > times[index - 1];
    decreasing &&= times[index] < times[index - 1];
  }
  return {
    first: values[0],
    last: values[values.length - 1],
    monotonic: /** @type {"increasing" | "decreasing" | "none"} */ (
      increasing ? "increasing" : decreasing && times.length > 1 ? "decreasing" : "none"
    ),
  };
}

/**
 * Chart types that fit the columns, each with the columns it would use and why. Rules only; no
 * column is judged by its values beyond its type.
 *
 * @param {number} rowCount
 * @param {ColumnSummary[]} columns
 * @returns {Suggestion[]}
 */
function suggest(rowCount, columns) {
  /** @type {Suggestion[]} */
  const suggestions = [];
  const numbers = columns.filter(({ type }) => type === "number" || type === "integer");
  const times = columns.filter(({ type }) => type === "date-time");
  const strings = columns.filter(({ type }) => type === "string");
  const byName = (/** @type {string} */ wanted) =>
    numbers.find(({ name }) => name.toLowerCase() === wanted);
  const names = (/** @type {ColumnSummary[]} */ list) => list.map(({ name }) => name);
  const list = (/** @type {ColumnSummary[]} */ list) => names(list).join(", ");

  const ohlc = ["open", "high", "low", "close"].map(byName);
  const low = byName("low");
  const high = byName("high");
  const mid = byName("mid");
  const lower = byName("lower");
  const upper = byName("upper");
  const reserved = new Set(
    [...ohlc, low, high, mid, lower, upper].filter(Boolean).map((column) => column?.name),
  );
  const plain = numbers.filter(({ name }) => !reserved.has(name));

  for (const time of times) {
    const order =
      (time.monotonic === "increasing"
        ? ""
        : " Sort the rows by time first: chartlet expects increasing times.") +
      (time.timeFormat === "year"
        ? ' Give years to chartlet as strings such as "1990": an integer time is read as Unix seconds.'
        : "");
    const many = rowCount > 2000 ? " More than 2000 rows: a layer holds at most 2000 points." : "";

    if (ohlc.every(Boolean)) {
      suggestions.push({
        type: "time",
        mark: "ohlc",
        columns: {
          time: time.name,
          open: /** @type {ColumnSummary} */ (ohlc[0]).name,
          high: /** @type {ColumnSummary} */ (ohlc[1]).name,
          low: /** @type {ColumnSummary} */ (ohlc[2]).name,
          close: /** @type {ColumnSummary} */ (ohlc[3]).name,
        },
        reason: `Time column ${time.name} with open, high, low and close columns: one candlestick per row in an ohlc layer.${order}${many}`,
      });
    }
    if (lower && upper && plain.length > 0) {
      suggestions.push({
        type: "time",
        mark: "line",
        columns: { time: time.name, value: plain[0].name, lower: lower.name, upper: upper.name },
        reason: `Time column ${time.name}, value column ${plain[0].name} and lower/upper columns: a line layer whose points carry an uncertainty band.${order}${many}`,
      });
    }
    if (plain.length > 0) {
      const layers = plain.slice(0, 4);
      const more =
        plain.length > 4
          ? ` ${plain.length} numeric columns: at most 4 line layers per chart take palette colors; split into panes or pick columns.`
          : "";
      suggestions.push({
        type: "time",
        mark: "line",
        columns: { time: time.name, layers: names(layers) },
        reason: `Time column ${time.name} and numeric column(s) ${list(plain)}: one line layer per numeric column on a time axis.${more}${order}${many}`,
      });
    }
    for (const group of strings) {
      const count = group.distinct ?? 0;
      if (plain.length >= 1 && count >= 2 && count <= 12 && !group.distinctCapped) {
        suggestions.push({
          type: "multiples",
          columns: { time: time.name, group: group.name, value: plain[0].name },
          reason: `Time column ${time.name}, group column ${group.name} with ${count} distinct values and value column ${plain[0].name}: one pane per group (2–12) with a shared value axis${count <= 4 ? ", or one line layer per group in a time chart" : ""}.${order}`,
        });
      }
    }
    if (time.timeFormat === "year" && plain.length === 1) {
      const span =
        typeof time.first === "string" || typeof time.first === "number"
          ? Number(time.last) - Number(time.first) + 1
          : 0;
      const gaps =
        time.monotonic === "increasing" && span === rowCount - time.missing
          ? ""
          : " The years are not consecutive: stripes take one value per year from firstYear on, with null for a missing year.";
      suggestions.push({
        type: "stripes",
        columns: { year: time.name, value: plain[0].name },
        reason: `Year column ${time.name} and one value column ${plain[0].name}: one stripe per year on a diverging scale.${gaps}`,
      });
    }
    if (
      time.timeFormat === "date" &&
      plain.length === 1 &&
      String(time.first).slice(0, 4) === String(time.last).slice(0, 4) &&
      time.monotonic === "increasing"
    ) {
      suggestions.push({
        type: "calendar",
        columns: { date: time.name, value: plain[0].name },
        reason: `Date column ${time.name} within ${String(time.first).slice(0, 4)} and one value column ${plain[0].name}: one cell per day in a calendar heatmap of that year.`,
      });
    }
  }

  for (const label of strings) {
    const unique = label.distinct === rowCount - label.missing && label.missing === 0;
    const repeat = unique
      ? ""
      : " Labels repeat or are missing: chartlet does not aggregate, so give each category one row first.";
    if (low && high) {
      suggestions.push({
        type: "rangebar",
        columns: { label: label.name, low: low.name, high: high.name, ...(mid ? { mid: mid.name } : {}) },
        reason: `Category column ${label.name} with low and high columns${mid ? " and mid" : ""}: one span per category.${rowCount > 100 ? " More than 100 rows: a rangebar holds at most 100 spans." : ""}${repeat}`,
      });
    }
    if (plain.length > 0 && times.length === 0) {
      const series = plain.slice(0, 4);
      suggestions.push({
        type: "bar",
        columns: { label: label.name, series: names(series) },
        reason: `Category column ${label.name} and numeric column(s) ${list(plain)}: ${plain.length === 1 ? "one bar per category" : "grouped bars, one series per numeric column (at most 4)"}.${rowCount > 100 ? " More than 100 rows: a bar chart holds at most 100 categories." : ""}${repeat}`,
      });
    }
  }
  suggestions.push(...suggestFromBlocks(rowCount, { numbers: plain, times, strings, columns }));
  return suggestions;
}

const TIME_LIKE = /^(time|duration|days?|months?|weeks?|years?|follow|survival)/i;
const EVENT_LIKE = /event|status|censor|death|dead|relapse/i;

/**
 * Chart types drawn from a block of their own: scatter, box plot (and violin or strip), treemap,
 * waterfall, sankey, survival curves and timeline. Each rule reads column types, names and ranges
 * only, and says what the table needs before it fits.
 *
 * @param {number} rowCount
 * @param {{ numbers: ColumnSummary[], times: ColumnSummary[], strings: ColumnSummary[], columns: ColumnSummary[] }} parts
 * @returns {Suggestion[]}
 */
function suggestFromBlocks(rowCount, { numbers, times, strings, columns }) {
  /** @type {Suggestion[]} */
  const found = [];
  const list = (/** @type {ColumnSummary[]} */ columns) => columns.map(({ name }) => name).join(", ");
  const few = (/** @type {ColumnSummary} */ column) =>
    !column.distinctCapped && (column.distinct ?? 0) >= 2 && (column.distinct ?? 0) <= 4;
  const unique = (/** @type {ColumnSummary} */ column) =>
    column.missing === 0 && column.distinct === rowCount;

  const eventColumn = columns.find(
    (column) =>
      EVENT_LIKE.test(column.name) &&
      (column.type === "boolean" || (column.type === "integer" && column.min === 0 && column.max === 1)),
  );
  const timeColumn = numbers.find(
    (column) => TIME_LIKE.test(column.name) && (column.min ?? -1) >= 0 && column !== eventColumn,
  );
  if (eventColumn && timeColumn) {
    const group = strings.find(few);
    found.push({
      type: "survival",
      columns: {
        time: timeColumn.name,
        event: eventColumn.name,
        ...(group ? { group: group.name } : {}),
      },
      reason: `Time column ${timeColumn.name} and event column ${eventColumn.name}${group ? ` with group column ${group.name} (${group.distinct} groups)` : ""}: Kaplan-Meier curves. Every row becomes an observation { time, event }; an event of 0 or false is censored. At most 4 groups and 2000 observations each.`,
    });
  }

  const plain = numbers.filter((column) => column !== timeColumn && column !== eventColumn);
  if (times.length === 0 && plain.length >= 2 && rowCount >= 5) {
    const [x, y] = [
      plain.find(({ name }) => name.toLowerCase() === "x") ?? plain[0],
      plain.find(({ name }) => name.toLowerCase() === "y") ?? plain[1],
    ];
    const group = strings.find(few);
    const label = strings.find((column) => column !== group && unique(column));
    found.push({
      type: "scatter",
      columns: {
        x: x.name,
        y: y.name,
        ...(group ? { group: group.name } : {}),
        ...(label ? { label: label.name } : {}),
      },
      reason: `Numeric columns ${x.name} and ${y.name}: one point per row${group ? `, colored by ${group.name}` : ""}${label ? `, named by ${label.name} where you set a label` : ""}.${rowCount > 5000 ? " More than 5000 rows: a scatter plot holds at most 5000 points." : ""}${rowCount > 300 ? " Above 300 points the dots are small without tooltips; label only the points that matter." : ""}`,
    });
  }

  if (times.length === 0 && plain.length >= 1) {
    for (const label of strings) {
      const count = label.distinct ?? 0;
      if (!label.distinctCapped && count >= 2 && count <= 100 && rowCount >= 5 * count) {
        found.push({
          type: "boxplot",
          columns: { label: label.name, values: plain[0].name },
          reason: `Category column ${label.name} repeats (${count} categories, about ${Math.round(rowCount / count)} rows each) with numeric column ${plain[0].name}: collect the values of each category into one box with "values" for a box plot, or set "boxDisplay": "violin" or "strip" to show their distribution. Each category needs at least 5 values.`,
        });
      }
    }
    for (const label of strings.filter(unique)) {
      const value = plain[0];
      if (rowCount >= 6 && rowCount <= 100 && (value.min ?? 0) > 0) {
        found.push({
          type: "treemap",
          columns: { label: label.name, value: value.name },
          reason: `Unique labels in ${label.name} and positive values in ${value.name}: rectangles by value, with an optional "group" (up to 4) per item for color. Fewer than 6 items read better as bars, waffle squares or parliament seats.`,
        });
      }
      if (rowCount >= 4 && rowCount <= 40 && (value.min ?? 0) < 0 && (value.max ?? 0) > 0) {
        found.push({
          type: "waterfall",
          columns: { label: label.name, value: value.name },
          reason: `Unique labels in ${label.name} and values of both signs in ${value.name}: a running total as a waterfall. Make the first row a "start", the rows that are changes deltas, and add a "total" step wherever the running total should show.`,
        });
      }
    }
  }

  const from = strings.find(({ name }) => /^(from|source|origin|src)/i.test(name));
  const to = strings.find(({ name }) => /^(to|target|dest)/i.test(name));
  const flow = numbers.find((column) => (column.min ?? 0) > 0);
  if (from && to && flow && from !== to) {
    found.push({
      type: "sankey",
      columns: { from: from.name, to: to.name, value: flow.name },
      reason: `Columns ${from.name} and ${to.name} name the ends of a flow and ${flow.name} its size: one link per row. A pair may appear once (add the values up first), a link must join two different nodes and the links must not run in a circle. At most 100 links and 40 nodes.${rowCount > 100 ? " More than 100 rows: too many links." : ""}`,
    });
  }

  const dates = times.filter(({ timeFormat }) => timeFormat === "date");
  const label = strings.find(unique);
  if (dates.length >= 2 && label) {
    const start = dates.find(({ name }) => /start|begin|from/i.test(name)) ?? dates[0];
    const end = dates.find(({ name }) => /end|finish|due|until/i.test(name) && name !== start.name) ?? dates.find((column) => column !== start);
    if (end) {
      found.push({
        type: "timeline",
        columns: { label: label.name, start: start.name, end: end.name },
        reason: `Unique labels in ${label.name} with date columns ${start.name} and ${end.name}: one phase per row on a timeline (a milestone takes a single "at" date instead). Name the ids that a row follows with "after" to draw dependencies, and group rows with "group".`,
      });
    }
  }
  return found;
}
