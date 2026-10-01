import assert from "node:assert/strict";
import test from "node:test";

import { parseCsv } from "../src/csv.mjs";

test("reads a header and records with LF, CRLF and CR line ends", () => {
  for (const end of ["\n", "\r\n", "\r"]) {
    assert.deepEqual(parseCsv(["a,b", "1,2", "3,4"].join(end) + end), {
      header: ["a", "b"],
      records: [
        ["1", "2"],
        ["3", "4"],
      ],
    });
  }
});

test("keeps quoted commas, line breaks and doubled quotes", () => {
  const { records } = parseCsv('name,note\r\n"Smith, J.","said ""hi""\r\nand left"\r\n"",x\n');
  assert.deepEqual(records, [
    ["Smith, J.", 'said "hi"\r\nand left'],
    ["", "x"],
  ]);
});

test("keeps values exactly as written", () => {
  const { records } = parseCsv("a,b,c\n 1 ,0012,1e3\n");
  assert.deepEqual(records, [[" 1 ", "0012", "1e3"]]);
});

test("reads empty fields, a last line without line break, and a trailing empty field", () => {
  assert.deepEqual(parseCsv("a,b,c\n,,\n1,,3").records, [
    ["", "", ""],
    ["1", "", "3"],
  ]);
  assert.deepEqual(parseCsv("a,b\n1,").records, [["1", ""]]);
  assert.deepEqual(parseCsv('a,b\n1,""').records, [["1", ""]]);
});

test("drops a byte order mark and blank lines at the end only", () => {
  assert.deepEqual(parseCsv("﻿a\n1\n\n2\n\n\n"), { header: ["a"], records: [["1"], [""], ["2"]] });
  assert.deepEqual(parseCsv('a\n""\n').records, [[""]]);
});

test("keeps a quote inside an unquoted field as a character", () => {
  assert.deepEqual(parseCsv('size\n5" screen\n').records, [['5" screen']]);
});

test("reports malformed CSV with its line", () => {
  assert.throws(() => parseCsv(""), /CSV is empty/);
  assert.throws(() => parseCsv('a,b\n1,"open\n2,3\n'), /line 2: quoted field is not closed/);
  assert.throws(() => parseCsv('a,b\n"x"y,2\n'), /line 2: unexpected "y" after a closing quote/);
  assert.throws(() => parseCsv("a,b\n1,2\n3\n"), /line 3: 1 field\(s\), but the header has 2/);
  assert.throws(() => parseCsv('a,b\n"multi\nline",2\n1,2,3\n'), /line 4: 3 field/);
  assert.throws(() => parseCsv("a,,c\n1,2,3\n"), /column 2 has no name/);
  assert.throws(() => parseCsv("a,a\n1,2\n"), /"a" appears twice/);
});
