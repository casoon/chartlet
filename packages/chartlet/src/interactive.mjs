// The optional interactive layer for charts rendered with `hooks: true`. It reads what the static
// output already holds — the plot geometry from the `data-*` hooks, the values from the data
// table or, in the SVG profile, from the JSON data block — and never lays anything out again.
// No dependencies, no inline script, no `eval`; styles are set through the CSSOM only.

const SVG = "http://www.w3.org/2000/svg";
const numbers = (text) => (text ?? "").trim().split(/\s+/).filter(Boolean).map(Number);

// Maps `value` from `domain` onto `range`, linear between neighbouring pairs and beyond the ends.
// Both lists have the same length; `domain` ascends.
export function toPixel(domain, range, value) {
  return interpolate(domain, range, value);
}

// The inverse of `toPixel`: the value at `pixel`. `range` may ascend or descend.
export function toValue(domain, range, pixel) {
  return interpolate(range, domain, pixel);
}

function interpolate(from, to, value) {
  const last = from.length - 1;
  if (last < 1) return to[0];
  const direction = Math.sign(from[last] - from[0]) || 1;
  let index = 1;
  while (index < last && (value - from[index]) * direction > 0) index++;
  const [a, b] = [from[index - 1], from[index]];
  return to[index - 1] + (b === a ? 0 : ((value - a) / (b - a)) * (to[index] - to[index - 1]));
}

// What a chart holds: its SVGs with their plots, and the data table as columns and rows.
export function readChart(root) {
  const svgs = root.matches?.("svg[data-chartlet-type]")
    ? [root]
    : [...root.querySelectorAll("svg[data-chartlet-type]")];
  const table = root.querySelector?.("table[data-chartlet-table]");
  let columns;
  let rows;
  if (table) {
    columns = [...table.tHead.rows[0].cells].slice(1).map((cell) => ({
      name: cell.textContent.trim(),
      series: Number(cell.dataset.series),
      pane: Number(cell.dataset.pane),
      part: cell.dataset.part,
    }));
    rows = [...table.tBodies[0].rows].map((row) => {
      const cells = [...row.cells].slice(1);
      return {
        x: Number(row.dataset.x),
        label: row.cells[0].textContent.trim(),
        text: cells.map((cell) => cell.textContent.trim()),
        value: cells.map((cell) => (cell.dataset.value == null ? null : Number(cell.dataset.value))),
      };
    });
  } else {
    const block = svgs[0]?.querySelector("script[data-chartlet-data]");
    ({ columns, rows } = block ? JSON.parse(block.textContent) : { columns: [], rows: [] });
  }
  const plots = new Map(
    svgs.map((svg) => [
      svg,
      [...svg.querySelectorAll("[data-chartlet-plot]")].map((plot) => {
        // A logarithmic value axis is linear in the logarithm of its values.
        const log = plot.dataset.yScale === "log";
        const yDomain = numbers(plot.dataset.yDomain);
        return {
          pane: Number(plot.dataset.pane),
          x: [numbers(plot.dataset.xDomain), numbers(plot.dataset.xRange)],
          y: [log ? yDomain.map(Math.log10) : yDomain, numbers(plot.dataset.yRange)],
          log,
        };
      }),
    ]),
  );
  const listeners = [];
  return {
    root,
    id: table?.dataset.chartletTable ?? svgs[0]?.dataset.chartletId,
    svgs,
    table,
    columns,
    rows,
    plots,
    // Series switched off, by their index among the data layers.
    hidden: new Set(),
    on: (listener) => listeners.push(listener),
    changed: () => listeners.forEach((listener) => listener()),
  };
}

// Enhances the chart in `root` — its figure, its SVG, or an element around either — with the
// features passed in, such as `{ crosshair, toggle }`. A feature is a function of the chart; pass
// `(chart) => play(chart, { interval: 400 })` to give it options. Returns `destroy()`.
export function enhance(root, features = {}) {
  const chart = readChart(root);
  const cleanups = Object.values(features)
    .filter((feature) => typeof feature === "function")
    .map((feature) => feature(chart));
  return {
    chart,
    destroy: () => cleanups.forEach((cleanup) => cleanup?.()),
  };
}

// The rows a plot shows: a zoom step covers only part of the table.
function visibleRows(chart, plot) {
  const [domain] = plot.x;
  const [low, high] = [domain[0], domain[domain.length - 1]];
  return chart.rows.map((_, index) => index).filter((index) => {
    const { x } = chart.rows[index];
    return x >= low && x <= high;
  });
}

function element(name, attributes = {}, namespace) {
  const node = namespace ? document.createElementNS(namespace, name) : document.createElement(name);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, value);
  return node;
}

// Where an SVG's user units lie on the screen.
function screenScale(svg) {
  const box = svg.getBoundingClientRect();
  return { box, scale: box.width / svg.viewBox.baseVal.width };
}

// A vertical rule and the values of one observation, by pointer and keyboard, synchronized over
// every pane. The values appear in an HTML overlay beside the chart that screen readers announce
// politely; it stays while hovered or focused and closes with Escape (WCAG 1.4.13).
export function crosshair(chart) {
  const cleanups = [];
  for (const svg of chart.svgs) {
    const plots = chart.plots.get(svg);
    if (!plots.length) continue;
    const rows = visibleRows(chart, plots[0]);
    const layer = svg.appendChild(
      element("g", { class: "chartlet-crosshair", "aria-hidden": "true", "pointer-events": "none" }, SVG),
    );
    const overlay = element("div", { class: "chartlet-crosshair-value", role: "status", "aria-atomic": "true" });
    Object.assign(overlay.style, {
      position: "absolute",
      zIndex: 1,
      padding: "4px 8px",
      font: "13px/1.4 system-ui, sans-serif",
      background: "Canvas",
      color: "CanvasText",
      border: "1px solid GrayText",
      borderRadius: "4px",
      whiteSpace: "nowrap",
    });
    overlay.hidden = true;
    const parent = svg.parentElement;
    if (getComputedStyle(parent).position === "static") parent.style.position = "relative";
    svg.after(overlay);
    if (!svg.hasAttribute("tabindex")) svg.setAttribute("tabindex", "0");
    svg.setAttribute("aria-keyshortcuts", "ArrowLeft ArrowRight Home End Escape");
    let active = -1;
    let current = plots[0];

    const show = (position) => {
      active = position;
      layer.replaceChildren();
      overlay.replaceChildren();
      if (position < 0) {
        overlay.hidden = true;
        return;
      }
      const row = chart.rows[rows[position]];
      const left = toPixel(...current.x, row.x);
      const top = current.y[1][1];
      for (const plot of plots) {
        const x = toPixel(...plot.x, row.x);
        const [bottom, upper] = plot.y[1];
        layer.append(element("line", { x1: x, x2: x, y1: bottom, y2: upper, stroke: "currentColor", "stroke-width": 1 }, SVG));
        chart.columns.forEach((column, index) => {
          const value = row.value[index];
          if (column.pane !== plot.pane || chart.hidden.has(column.series) || value == null) return;
          if (column.part !== "value" && column.part !== "close") return;
          const y = toPixel(...plot.y, plot.log ? Math.log10(value) : value);
          layer.append(element("circle", { cx: x, cy: y, r: 4, fill: "Canvas", stroke: "currentColor", "stroke-width": 2 }, SVG));
        });
      }
      const heading = element("strong");
      heading.textContent = row.label;
      overlay.append(heading);
      // Small multiples list the panel under the pointer; a time chart's panes share one list.
      const multiples = svg.dataset.chartletType === "multiples";
      chart.columns.forEach((column, index) => {
        if (chart.hidden.has(column.series) || (multiples && column.pane !== current.pane)) return;
        const line = element("div");
        line.textContent = `${column.name}: ${row.text[index]}`;
        overlay.append(line);
      });
      overlay.hidden = false;
      const { box, scale } = screenScale(svg);
      const origin = parent.getBoundingClientRect();
      const offset = box.left - origin.left + left * scale;
      const right = left > svg.viewBox.baseVal.width / 2;
      overlay.style.top = `${box.top - origin.top + top * scale}px`;
      overlay.style.left = right ? "" : `${offset + 12}px`;
      overlay.style.right = right ? `${origin.width - offset + 12}px` : "";
    };

    // The observation nearest the pointer, in the pane under it.
    const onPointer = (event) => {
      const { box, scale } = screenScale(svg);
      const x = (event.clientX - box.left) / scale;
      const y = (event.clientY - box.top) / scale;
      const plot = plots.find(({ x: [, range], y: [, height] }) =>
        x >= Math.min(...range) - 8 && x <= Math.max(...range) + 8 &&
        y >= Math.min(...height) && y <= Math.max(...height));
      if (!plot || !rows.length) return;
      current = plot;
      let nearest = 0;
      rows.forEach((index, position) => {
        const distance = (row) => Math.abs(toPixel(...plot.x, chart.rows[row].x) - x);
        if (distance(index) < distance(rows[nearest])) nearest = position;
      });
      if (nearest !== active) show(nearest);
    };
    const onKey = (event) => {
      const last = rows.length - 1;
      const next = {
        ArrowRight: active < 0 ? 0 : Math.min(active + 1, last),
        ArrowLeft: active < 0 ? last : Math.max(active - 1, 0),
        Home: 0,
        End: last,
        Escape: -1,
      }[event.key];
      if (next === undefined || (event.key === "Escape" && active < 0)) return;
      event.preventDefault();
      show(next);
    };
    const onLeave = (event) => {
      if (!overlay.contains(event.relatedTarget) && event.relatedTarget !== svg && document.activeElement !== svg) show(-1);
    };
    const onEscape = (event) => event.key === "Escape" && active >= 0 && show(-1);
    const listeners = [
      [svg, "pointermove", onPointer],
      [svg, "pointerdown", onPointer],
      [svg, "keydown", onKey],
      [svg, "pointerleave", onLeave],
      [overlay, "pointerleave", onLeave],
      [svg, "blur", () => show(-1)],
      [document, "keydown", onEscape],
    ];
    for (const [target, type, listener] of listeners) target.addEventListener(type, listener);
    chart.on(() => active >= 0 && show(active));
    cleanups.push(() => {
      for (const [target, type, listener] of listeners) target.removeEventListener(type, listener);
      layer.remove();
      overlay.remove();
    });
  }
  return () => cleanups.forEach((cleanup) => cleanup());
}

// The groups that draw data layers, by name.
const named = (chart) => chart.svgs.flatMap((svg) => [...svg.querySelectorAll("g[data-series][data-name]")]);

// One checkbox per named series of a time chart or small multiples, the way the series filter of
// a bar chart works: switching one off hides its lines, bands and markers; the axes stay.
export function toggle(chart, { legend = "Series" } = {}) {
  const groups = named(chart).sort((a, b) => a.dataset.series - b.dataset.series);
  const names = [...new Set(groups.map((group) => group.dataset.name))];
  if (names.length < 2 || chart.root.querySelector?.(".chartlet-filter")) return undefined;
  const fieldset = element("fieldset", { class: "chartlet-filter" });
  const caption = element("legend");
  caption.textContent = legend;
  fieldset.append(caption);
  for (const name of names) {
    const label = element("label");
    const input = element("input", { type: "checkbox" });
    input.checked = true;
    input.addEventListener("change", () => {
      for (const group of named(chart)) {
        if (group.dataset.name !== name) continue;
        if (input.checked) group.removeAttribute("display");
        else group.setAttribute("display", "none");
        chart.hidden[input.checked ? "delete" : "add"](Number(group.dataset.series));
      }
      chart.changed();
    });
    label.append(input, ` ${name}`);
    fieldset.append(label);
  }
  const anchor = chart.root.closest?.("figure") ?? chart.root;
  anchor.before(fieldset);
  return () => fieldset.remove();
}

// Shows only the data between `from` and `to` (axis values; `null` for either end of the plot)
// by clipping the data layers of every pane, or shows everything again without arguments.
export function reveal(chart, from, to) {
  for (const svg of chart.svgs) {
    for (const plot of chart.plots.get(svg)) {
      const id = `${svg.id}-reveal-${plot.pane}`;
      let clip = svg.querySelector(`[id="${id}"]`);
      const targets = svg.querySelector("g[data-series]")
        ? svg.querySelectorAll(`g[data-series][data-pane="${plot.pane}"]`)
        : svg.querySelectorAll(".chartlet-line,.chartlet-point,.chartlet-value");
      if (from === undefined) {
        clip?.remove();
        targets.forEach((target) => target.removeAttribute("clip-path"));
        continue;
      }
      if (!clip) {
        clip = svg.appendChild(element("clipPath", { id }, SVG));
        clip.append(element("rect", { y: 0, height: svg.viewBox.baseVal.height }, SVG));
      }
      const [domain, range] = plot.x;
      const start = from == null ? 0 : toPixel(domain, range, from) - 2;
      const end = to == null ? svg.viewBox.baseVal.width : toPixel(domain, range, to) + 2;
      clip.firstChild.setAttribute("x", start);
      clip.firstChild.setAttribute("width", Math.max(0, end - start));
      targets.forEach((target) => target.setAttribute("clip-path", `url(#${id})`));
    }
  }
}

// A year, a date or a number, as a position on the axis: Unix seconds on a time axis.
function axisValue(chart, text) {
  if (chart.svgs[0]?.dataset.chartletType !== "time" && chart.svgs[0]?.dataset.chartletType !== "multiples") {
    return Number(text);
  }
  return /^\d{4}$/.test(text) ? Date.UTC(Number(text), 0, 1) / 1000 : Date.parse(text) / 1000;
}

// Scroll stations: elements of the page with `data-chartlet-step` (empty, or the chart's ID)
// set the chart to the state they describe once they reach the middle of the viewport —
// `data-chartlet-highlight` (a series name), `data-chartlet-annotation` (an annotation's layer
// index), `data-chartlet-range` ("from to") and `data-chartlet-zoom` (a zoom step's index or
// label). States only emphasize, hide or switch what is rendered.
export function steps(chart, { selector = "[data-chartlet-step]" } = {}) {
  if (typeof IntersectionObserver === "undefined") return undefined;
  const stations = [...document.querySelectorAll(selector)].filter(
    (station) => !station.dataset.chartletStep || station.dataset.chartletStep === chart.id,
  );
  const dim = (groups, keep) =>
    groups.forEach((group) => (keep(group) ? group.removeAttribute("opacity") : group.setAttribute("opacity", "0.25")));
  const apply = (station) => {
    const { chartletHighlight: highlight, chartletAnnotation: annotation, chartletRange: range, chartletZoom: zoom } =
      station.dataset;
    for (const svg of chart.svgs) {
      dim([...svg.querySelectorAll("g[data-series]")], (group) => !highlight || group.dataset.name === highlight);
      dim([...svg.querySelectorAll("g[data-annotation]")], (group) => !annotation || group.dataset.annotation === annotation);
    }
    if (range) {
      const [from, to] = range.split(/\s+/).map((part) => axisValue(chart, part));
      reveal(chart, from, to);
    } else {
      reveal(chart);
    }
    if (zoom != null) {
      const radios = [...(chart.root.closest?.(".chartlet-wrapper") ?? chart.root).querySelectorAll(".chartlet-zoom input")];
      const radio =
        radios.find((input) => input.parentElement.textContent.trim() === zoom) ?? radios[Number(zoom)];
      if (radio) radio.checked = true;
    }
  };
  const observer = new IntersectionObserver(
    (entries) => entries.forEach((entry) => entry.isIntersecting && apply(entry.target)),
    { rootMargin: "-50% 0px -50% 0px" },
  );
  stations.forEach((station) => observer.observe(station));
  return () => observer.disconnect();
}

// Reveals the data one observation at a time, with play and pause, a step back and forward and
// a button that shows everything again (WCAG 2.2.2). With `prefers-reduced-motion: reduce`
// there is no playback, only the step buttons.
export function play(chart, { interval, labels = {} } = {}) {
  const plot = chart.plots.get(chart.svgs[0])?.[0];
  if (!plot) return undefined;
  const rows = visibleRows(chart, plot);
  const text = { group: "Playback", play: "Play", pause: "Pause", back: "Step back", forward: "Step forward", all: "Show all", ...labels };
  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const group = element("div", { class: "chartlet-play", role: "group", "aria-label": text.group });
  const status = element("span", { role: "status" });
  let position = -1;
  let timer;
  const button = (label, action) => {
    const node = element("button", { type: "button" });
    node.textContent = label;
    node.addEventListener("click", action);
    group.append(node);
    return node;
  };
  const go = (next) => {
    position = next;
    if (next < 0 || next >= rows.length - 1) {
      position = -1;
      stop();
      reveal(chart);
      status.textContent = "";
      return;
    }
    reveal(chart, null, chart.rows[rows[next]].x);
    status.textContent = timer ? "" : chart.rows[rows[next]].label;
  };
  const stop = () => {
    clearInterval(timer);
    timer = undefined;
    if (toggleButton) toggleButton.textContent = text.play;
    if (position >= 0) status.textContent = chart.rows[rows[position]].label;
  };
  const toggleButton = still
    ? undefined
    : button(text.play, () => {
        if (timer) return stop();
        toggleButton.textContent = text.pause;
        timer = setInterval(() => go(position + 1), interval ?? Math.max(40, 6000 / rows.length));
        if (position < 0) go(0);
      });
  button(text.back, () => (stop(), go(Math.max(position < 0 ? rows.length - 2 : position - 1, 0))));
  button(text.forward, () => (stop(), go(position + 1)));
  button(text.all, () => (stop(), go(-1)));
  group.append(status);
  const anchor = chart.root.closest?.("figure") ?? chart.root;
  anchor.after(group);
  return () => {
    stop();
    reveal(chart);
    group.remove();
  };
}
