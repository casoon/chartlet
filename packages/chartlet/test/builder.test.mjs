import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";
import test from "node:test";

import { architecture, id } from "@casoon/chartlet/builder";

import { renderChart, renderChartDetailed } from "../src/render.mjs";

delete process.env.CHARTLET_BIN;

const packageDirectory = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const examples = resolve(packageDirectory, "../../examples");
const readExample = (name) => JSON.parse(readFileSync(join(examples, name), "utf8"));

function shop() {
  const a = architecture();
  const region = a.boundary("region", "Cloud region eu-central-1");
  const network = a.boundary("network", "Private network", { in: region });
  const data = a.boundary("data", "Data zone", { in: network });

  const customer = a.component("customer", "Customer", { kind: "person" });
  const web = a.component("web", "Web shop", { sublabel: "browser", kind: "frontend" });
  const cdn = a.component("cdn", "Content network", { kind: "external" });
  const api = a.component("api", "Shop API", { sublabel: "Node.js", in: network });
  const cache = a.component("cache", "Session cache", {
    sublabel: "Redis",
    kind: "cache",
    in: network,
  });
  const events = a.component("events", "Order events", { kind: "queue", in: network });
  const worker = a.component("worker", "Order worker", { in: network });
  const db = a.component("db", "Orders", { sublabel: "Postgres", kind: "database", in: data });
  const files = a.component("files", "Invoices", {
    sublabel: "object storage",
    kind: "storage",
    in: region,
  });
  const pay = a.component("pay", "Payment provider", { kind: "external" });
  const auth = a.component("auth", "Sign-in", {
    sublabel: "identity provider",
    kind: "security",
    in: network,
  });

  a.connect(customer, web, { label: "browses" })
    .connect(web, cdn, { label: "loads assets", technology: "HTTPS", dash: "dashed" })
    .connect(web, api, { label: "calls", technology: "HTTPS/JSON" })
    .connect(api, cache, { label: "sessions", technology: "RESP" })
    .connect(api, auth, { label: "checks token", technology: "OIDC" })
    .connect(api, db, { label: "reads, writes", technology: "SQL/TLS" })
    .connect(api, events, { label: "publishes", technology: "AMQP" })
    .connect(events, worker, { label: "delivers" })
    .connect(worker, pay, { label: "charges", technology: "HTTPS" })
    .connect(worker, files, { label: "stores PDF", technology: "S3 API" })
    .connect(worker, db, { label: "updates", technology: "SQL/TLS" })
    .mainPath([customer, web, api, events, worker, pay]);

  return a.spec({
    title: "Web shop on one cloud region",
    source: "Architecture handbook, overview",
    width: 1820,
    height: 540,
  });
}

test("builds the shop example as its JSON, and renders it byte-identical", () => {
  const expected = readExample("shop-architecture.json");
  const built = shop();
  assert.deepEqual(built, expected);
  for (const format of ["svg", "html"]) {
    assert.equal(renderChart(built, { format }).content, renderChart(expected, { format }).content);
  }
});

test("builds the tenant example with its settings and phone variant", () => {
  const a = architecture({ orientation: "portrait" });
  const rz = a.boundary("rz", "Data center");
  const netz = a.boundary("netz", "Application network", { in: rz });
  const daten = a.boundary("daten", "Data zone", { in: netz });
  const nutzer = a.component("nutzer", "Teacher, parent", { kind: "person" });
  const browser = a.component("browser", "Browser", { kind: "frontend" });
  const proxy = a.component("proxy", "Reverse proxy", {
    sublabel: "TLS 1.3, WAF",
    kind: "security",
    in: rz,
  });
  const web = a.component("web", "Web / BFF", { sublabel: "SvelteKit", in: netz });
  const idp = a.component("idp", "Sign-in", {
    sublabel: "Keycloak, OIDC",
    kind: "security",
    in: netz,
  });
  const api = a.component("api", "Domain API", { sublabel: "Rust", in: netz });
  const db = a.component("db", "Domain data", {
    sublabel: "Postgres, RLS per tenant",
    kind: "database",
    in: daten,
  });
  const audit = a.component("audit", "Audit log", {
    sublabel: "append only",
    kind: "database",
    in: daten,
  });
  const backup = a.component("backup", "Backup", { sublabel: "second site", kind: "storage" });
  const alt = a.component("alt", "School records", { sublabel: "legacy system", kind: "external" });
  const soc = a.component("soc", "SIEM / SOC", { kind: "external" });
  a.connect(nutzer, browser, { label: "uses" })
    .connect(browser, proxy, { label: "calls", technology: "HTTPS" })
    .connect(proxy, web, { label: "forwards" })
    .connect(web, idp, { label: "signs in", technology: "OIDC" })
    .connect(web, api, { label: "queries", technology: "user context" })
    .connect(api, db, { label: "reads, writes", technology: "SQL/TLS" })
    .connect(api, audit, { label: "logs" })
    .connect(alt, api, { label: "master data", technology: "REST/JSON" })
    .connect(api, soc, { label: "events", technology: "Syslog", dash: "dashed" })
    .connect(db, backup, { label: "backs up", dash: "dashed" })
    .mainPath([nutzer, browser, proxy, web, api, db]);
  const built = a.spec({
    title: "Multi-tenant line-of-business application",
    source: "Illustrative reference architecture",
    width: 1200,
    height: 1100,
    mobile: { width: 464, height: 1180 },
  });

  const expected = readExample("tenant-architecture.json");
  assert.deepEqual(built, expected);
  for (const variant of ["desktop", "mobile"]) {
    assert.equal(
      renderChart(built, { format: "svg", variant }).content,
      renderChart(expected, { format: "svg", variant }).content,
    );
  }
});

// A building block is a plain function: it takes the builder and a name, and derives its ids
// from the name.
function workers(a, name, region) {
  const zone = a.boundary(id(name, "zone"), `Region ${name}`, { in: region });
  const queue = a.component(id(name, "queue"), "Jobs", { kind: "queue", in: zone });
  const pool = [1, 2, 3].map((n) =>
    a.component(id(name, "worker", n), `Worker ${n}`, { in: zone }),
  );
  a.fanOut(queue, pool, { label: "delivers" });
  return { queue, pool };
}

function regions() {
  const a = architecture();
  const cloud = a.boundary("cloud", "Cloud");
  const api = a.component("api", "API", { in: cloud });
  const db = a.component("db", "Orders", { kind: "database" });
  for (const name of ["eu", "us"]) {
    const { queue, pool } = workers(a, name, cloud);
    a.connect(api, queue, { label: "enqueues" }).fanIn(pool, db, { label: "writes" });
  }
  a.chain(["api", "eu-queue", "eu-worker-1"], { dash: "dotted" });
  return a.spec({ title: "Workers in two regions" });
}

test("derives the same ids in the same order from the same building blocks", () => {
  const first = regions();
  assert.deepEqual(regions(), first);
  assert.deepEqual(
    first.architecture.components.map((component) => component.id),
    [
      "api",
      "db",
      "eu-queue",
      "eu-worker-1",
      "eu-worker-2",
      "eu-worker-3",
      "us-queue",
      "us-worker-1",
      "us-worker-2",
      "us-worker-3",
    ],
  );
  assert.deepEqual(
    first.architecture.connections.slice(0, 5).map(({ from, to }) => `${from}>${to}`),
    [
      "eu-queue>eu-worker-1",
      "eu-queue>eu-worker-2",
      "eu-queue>eu-worker-3",
      "api>eu-queue",
      "eu-worker-1>db",
    ],
  );
  assert.equal(first.architecture.connections.length, 16);
  assert.equal(renderChartDetailed(first, { format: "svg" }).ok, true);
});

test("returns a copy that later calls do not change", () => {
  const a = architecture();
  a.component("one", "One");
  const before = a.spec();
  a.component("two", "Two");
  assert.deepEqual(before.architecture, { components: [{ id: "one", label: "One" }] });
  before.architecture.components[0].label = "Changed";
  assert.equal(a.spec().architecture.components[0].label, "One");
});

test("reports identifiers used twice and references to undeclared elements", () => {
  const a = architecture();
  const zone = a.boundary("zone", "Zone");
  a.component("api", "API", { in: zone });
  assert.throws(() => a.component("api", "Again"), /"api" is already declared as a component/);
  assert.throws(() => a.component("zone", "Zone"), /"zone" is already declared as a boundary/);
  assert.throws(
    () => a.component("db", "DB", { in: "nowhere" }),
    /"nowhere", which is not declared/,
  );
  assert.throws(() => a.component("db", "DB", { in: "api" }), /a component, not a boundary/);
  assert.throws(() => a.connect("api", "db"), /end of a connection is "db", which is not declared/);
  assert.throws(() => a.connect("api", zone), /a boundary, not a component/);
  assert.throws(() => a.mainPath({ from: "api", to: "db" }), /main path is "db"/);
  assert.throws(() => a.component("1st", "First"), /"1st" is not an identifier/);
  assert.throws(() => id("eu", "a b"), /"eu-a b" is not an identifier/);
  // No connection was recorded by the calls that failed.
  assert.equal(a.spec().architecture.connections, undefined);
});

test("takes no list where one component is expected", () => {
  const a = architecture();
  const [x, y, z] = ["x", "y", "z"].map((name) => a.component(name, name.toUpperCase()));
  assert.throws(() => a.connect([x, y], z), /must be one identifier, not \["x","y"\]/);
  assert.throws(() => a.fanOut([x, y], [z]), /must be one identifier/);
  assert.throws(() => a.fanIn(x, z), /fanIn takes a list of identifiers/);
});

test("leaves everything else to the validation of the core", () => {
  const a = architecture();
  const x = a.component("x", "X", { kind: "spaceship" });
  a.connect(x, x, { future: true });
  const result = renderChartDetailed(a.spec(), { format: "svg" });
  assert.equal(result.ok, false);
  assert.match(result.error.path, /^\/architecture\//);
});
