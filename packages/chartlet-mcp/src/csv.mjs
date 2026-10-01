// An RFC 4180 CSV reader: comma-separated fields, records ended by CRLF, LF or CR, fields in
// double quotes may hold commas, line breaks and quotes doubled as `""`. Values stay strings
// exactly as written; nothing is trimmed or converted. A quote inside an unquoted field is kept
// as a literal character.

/**
 * @typedef {{ header: string[], records: string[][] }} CsvTable
 */

/**
 * Parses CSV text with a header row. Throws an `Error` naming the line of the first problem: an
 * unterminated quoted field, text after a closing quote, a record whose field count differs from
 * the header, or an empty or repeated column name.
 *
 * @param {string} text
 * @returns {CsvTable}
 */
export function parseCsv(text) {
  const rows = readRecords(text.startsWith("﻿") ? text.slice(1) : text);
  if (rows.length === 0) {
    throw new Error("CSV is empty: expected a header row with the column names.");
  }
  const [{ fields: header }, ...rest] = rows;
  const seen = new Set();
  for (const [index, name] of header.entries()) {
    if (name === "") {
      throw new Error(`CSV header: column ${index + 1} has no name.`);
    }
    if (seen.has(name)) {
      throw new Error(`CSV header: column name ${JSON.stringify(name)} appears twice.`);
    }
    seen.add(name);
  }
  for (const { fields, line } of rest) {
    if (fields.length !== header.length) {
      throw new Error(
        `CSV line ${line}: ${fields.length} field(s), but the header has ${header.length}. Quote fields that contain commas or line breaks.`,
      );
    }
  }
  return { header, records: rest.map(({ fields }) => fields) };
}

/**
 * Splits the text into records, each with the line it starts on. Blank lines at the end are
 * dropped; a blank line elsewhere is a record with one empty field.
 *
 * @param {string} text
 * @returns {{ fields: string[], line: number }[]}
 */
function readRecords(text) {
  /** @type {{ fields: string[], line: number, blank: boolean }[]} */
  const records = [];
  /** @type {string[]} */
  let fields = [];
  let field = "";
  let quoted = false;
  let line = 1;
  let recordLine = 1;
  let index = 0;

  const endField = () => {
    fields.push(field);
    field = "";
  };
  const endRecord = () => {
    const blank = fields.length === 0 && field === "" && !quoted;
    endField();
    records.push({ fields, line: recordLine, blank });
    fields = [];
    quoted = false;
  };

  while (index < text.length) {
    const char = text[index];

    if (char === '"' && field === "" && !quoted) {
      // A quoted field: read to the closing quote.
      quoted = true;
      const openLine = line;
      index += 1;
      for (;;) {
        if (index >= text.length) {
          throw new Error(`CSV line ${openLine}: quoted field is not closed.`);
        }
        const inner = text[index];
        if (inner === '"') {
          if (text[index + 1] === '"') {
            field += '"';
            index += 2;
            continue;
          }
          index += 1;
          break;
        }
        if (inner === "\n" || (inner === "\r" && text[index + 1] !== "\n")) {
          line += 1;
        }
        field += inner;
        index += 1;
      }
      const next = text[index];
      if (next !== undefined && next !== "," && next !== "\r" && next !== "\n") {
        throw new Error(
          `CSV line ${line}: unexpected ${JSON.stringify(next)} after a closing quote. A quote inside a quoted field is written as "".`,
        );
      }
      continue;
    }

    if (char === ",") {
      endField();
      quoted = false;
      index += 1;
      continue;
    }
    if (char === "\r" || char === "\n") {
      endRecord();
      index += char === "\r" && text[index + 1] === "\n" ? 2 : 1;
      line += 1;
      recordLine = line;
      continue;
    }
    field += char;
    index += 1;
  }
  // The last record, unless the text ends with a line break.
  if (fields.length > 0 || field !== "" || quoted) {
    endRecord();
  }

  while (records.length > 0 && records[records.length - 1].blank) {
    records.pop();
  }
  return records;
}
