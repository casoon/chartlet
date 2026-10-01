import assert from "node:assert/strict";
import test from "node:test";

import { toPixel, toValue } from "@casoon/chartlet/interactive";

test("maps a value onto a linear axis and back", () => {
  const domain = [1772323200, 1774656000];
  const range = [78, 770];
  assert.equal(toPixel(domain, range, domain[0]), 78);
  assert.equal(toPixel(domain, range, domain[1]), 770);
  assert.equal(toPixel(domain, range, (domain[0] + domain[1]) / 2), 424);
  assert.equal(toValue(domain, range, 424), (domain[0] + domain[1]) / 2);
});

test("maps a value axis whose pixels run upwards", () => {
  const domain = [0, 60];
  const range = [394, 58];
  assert.equal(toPixel(domain, range, 0), 394);
  assert.equal(toPixel(domain, range, 30), 226);
  assert.equal(toValue(domain, range, 226), 30);
  // Beyond the ends the mapping continues the nearest segment.
  assert.equal(toPixel(domain, range, 120), -278);
});

test("maps collapsed gaps piecewise, slot by slot", () => {
  // Friday, Monday, Tuesday: the weekend takes no room.
  const domain = [0, 3 * 86400, 4 * 86400];
  const range = [100, 200, 300];
  assert.equal(toPixel(domain, range, 3 * 86400), 200);
  assert.equal(toPixel(domain, range, 1.5 * 86400), 150);
  assert.equal(toPixel(domain, range, 3.5 * 86400), 250);
  assert.equal(toValue(domain, range, 250), 3.5 * 86400);
  for (const value of domain) {
    assert.equal(toValue(domain, range, toPixel(domain, range, value)), value);
  }
});

test("a single category sits at one pixel", () => {
  assert.equal(toPixel([0], [424], 0), 424);
  assert.equal(toValue([0], [424], 500), 0);
});
