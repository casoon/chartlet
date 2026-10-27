// The playground: edit a specification as JSON or in a small form, render it with chartlet's own
// WebAssembly compiler in a worker, read the diagnostics at the field they name, and download the
// exact specification and its output. Everything stays in this browser tab: the only request the
// page makes is for the compiler itself. No second renderer lives here.
import { locate } from './pointer.mjs';

type Json = Record<string, any>;

interface Example {
  id: string;
  label: string;
  json: string;
}
interface Diagnostic {
  code: string;
  path: string | null;
  message: string;
  severity: 'error' | 'warning';
}
interface Remedy {
  id: string;
  summary: string;
  kind: 'layout' | 'content';
  patch: { op: 'add' | 'replace'; path: string; value: unknown }[];
  validated: boolean;
  resolves: boolean;
}
interface Help {
  code: string;
  path: string;
  docs: string;
  field: string;
  note?: string;
  remedies: Remedy[];
}

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const dataElement = $('pg-examples') as HTMLTemplateElement;
const base = dataElement.dataset.base ?? '/';
const examples: Example[] = JSON.parse(dataElement.content.textContent ?? '[]');

const el = {
  example: $<HTMLSelectElement>('pg-example'),
  theme: $<HTMLSelectElement>('pg-theme'),
  variant: $<HTMLSelectElement>('pg-variant'),
  modeJson: $<HTMLButtonElement>('pg-mode-json'),
  modeGuided: $<HTMLButtonElement>('pg-mode-guided'),
  editor: $<HTMLTextAreaElement>('pg-editor'),
  guided: $<HTMLDivElement>('pg-guided'),
  preview: $<HTMLDivElement>('pg-preview'),
  stale: $<HTMLParagraphElement>('pg-stale'),
  status: $<HTMLParagraphElement>('pg-status'),
  diagnostics: $<HTMLOListElement>('pg-diagnostics'),
  alternative: $<HTMLDivElement>('pg-alternative'),
  undo: $<HTMLButtonElement>('pg-undo'),
  redo: $<HTMLButtonElement>('pg-redo'),
  reset: $<HTMLButtonElement>('pg-reset'),
  copy: $<HTMLButtonElement>('pg-copy'),
  downloadJson: $<HTMLButtonElement>('pg-download-json'),
  downloadSvg: $<HTMLButtonElement>('pg-download-svg'),
  downloadHtml: $<HTMLButtonElement>('pg-download-html'),
  failure: $<HTMLDivElement>('pg-failure'),
  payload: $<HTMLSpanElement>('pg-payload'),
};

// ---- the compiler, in a worker -------------------------------------------------------------

let worker: Worker;
let ready = false;
let loadFailed = false;
let wasmBytes = 0;
let nextId = 1;
const pending = new Map<number, { resolve: (value: any) => void; reject: (error: Error) => void }>();
const RENDER_TIMEOUT_MS = 8000;

function startWorker(): void {
  ready = false;
  worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
  worker.onmessage = (event: MessageEvent) => {
    const message = event.data;
    if (message.type === 'ready') {
      ready = true;
      wasmBytes = message.bytes;
      el.payload.textContent = `compiler ${(message.bytes / 1000).toFixed(0)} kB, loaded when this page opened`;
      el.failure.hidden = true;
      scheduleRender(0);
    } else if (message.type === 'failed') {
      loadFailed = true;
      showFailure(message.message);
    } else if (message.type === 'result' || message.type === 'crash') {
      const waiting = pending.get(message.id);
      pending.delete(message.id);
      if (message.type === 'result') waiting?.resolve(message);
      else waiting?.reject(new Error(message.message));
    }
  };
  worker.onerror = (event) => {
    loadFailed = !ready;
    if (!ready) showFailure(event.message || 'the worker could not start');
  };
  worker.postMessage({ type: 'init', wasmUrl: `${base}playground/chartlet.wasm` });
}

function showFailure(reason: string): void {
  el.failure.hidden = false;
  $('pg-failure-reason').textContent = reason;
  setStatus('The compiler could not be loaded, so nothing can be rendered. You can still edit and download the JSON.');
}

function compile(spec: string, options: Json): Promise<any> {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    const timer = setTimeout(() => {
      pending.delete(id);
      worker.terminate();
      startWorker();
      reject(new Error(`The render took longer than ${RENDER_TIMEOUT_MS / 1000} seconds and was stopped. The compiler restarted; shrink the specification.`));
    }, RENDER_TIMEOUT_MS);
    pending.set(id, {
      resolve: (value) => (clearTimeout(timer), resolve(value)),
      reject: (error) => (clearTimeout(timer), reject(error)),
    });
    worker.postMessage({ type: 'render', id, spec, options });
  });
}

// ---- state: one specification text, a history of it ---------------------------------------------

let text = '';
let initial = '';
let history: string[] = [];
let cursor = -1;
let mode: 'json' | 'guided' = 'json';
let lastValid = false;
let generation = 0;
let renderTimer: ReturnType<typeof setTimeout> | undefined;
let commitTimer: ReturnType<typeof setTimeout> | undefined;
let diagnostics: Diagnostic[] = [];
let help: Help[] = [];

function parse(source: string): Json | undefined {
  try {
    const value = JSON.parse(source);
    return value && typeof value === 'object' && !Array.isArray(value) ? value : undefined;
  } catch {
    return undefined;
  }
}

function setStatus(message: string): void {
  el.status.textContent = message;
}

function commit(now = false): void {
  clearTimeout(commitTimer);
  const push = () => {
    if (history[cursor] === text) return;
    history = history.slice(0, cursor + 1);
    history.push(text);
    if (history.length > 100) history.shift();
    cursor = history.length - 1;
    syncHistoryButtons();
  };
  if (now) push();
  else commitTimer = setTimeout(push, 600);
}

function syncHistoryButtons(): void {
  el.undo.disabled = cursor <= 0;
  el.redo.disabled = cursor >= history.length - 1;
}

/** Replaces the whole text (undo, example, reset): the editors follow. */
function setText(next: string, { record = true } = {}): void {
  text = next;
  el.editor.value = next;
  if (record) commit(true);
  syncControls();
  if (mode === 'guided') renderGuided();
  scheduleRender();
}

function onTextChanged(): void {
  commit();
  syncControls();
  scheduleRender();
}

function syncControls(): void {
  const spec = parse(text);
  el.theme.value = spec && typeof spec.theme === 'string' ? spec.theme : 'light';
  el.modeGuided.disabled = !spec;
  el.modeGuided.title = spec ? '' : 'Fix the JSON first: the form needs a valid specification.';
}

// ---- rendering ---------------------------------------------------------------------------------

function scheduleRender(delay = 350): void {
  clearTimeout(renderTimer);
  renderTimer = setTimeout(render, delay);
}

function renderOptions(extra: Json = {}): Json {
  const variant = el.variant.value;
  return {
    format: variant === 'desktop' ? 'html' : 'svg',
    variant,
    idPrefix: 'pg',
    alternative: true,
    advice: true,
    ...extra,
  };
}

async function render(): Promise<void> {
  const mine = ++generation;
  const spec = parse(text);
  if (!spec) {
    let message = 'The text is not a JSON object.';
    try {
      JSON.parse(text);
    } catch (error) {
      message = String((error as Error).message);
    }
    diagnostics = [{ code: 'invalid_json', path: '/', message, severity: 'error' }];
    help = [];
    markStale(true);
    showDiagnostics();
    setStatus(`The JSON is not valid: ${message}`);
    return;
  }
  if (!ready) {
    if (!loadFailed) setStatus('Loading the compiler…');
    return;
  }
  setStatus('Rendering…');
  let message;
  try {
    message = await compile(text, renderOptions());
  } catch (error) {
    if (mine !== generation) return;
    diagnostics = [{ code: 'render_failed', path: null, message: (error as Error).message, severity: 'error' }];
    help = [];
    markStale(true);
    showDiagnostics();
    setStatus((error as Error).message);
    return;
  }
  if (mine !== generation) return;
  const response = message.response;
  diagnostics = (response.warnings ?? []).map((w: Diagnostic) => ({ ...w, severity: 'warning' }));
  help = response.help ?? [];
  if (response.ok) {
    lastValid = true;
    el.preview.innerHTML = response.content;
    markStale(false);
    renderAlternative(response.alternative);
    setStatus(
      diagnostics.length === 0
        ? `Rendered in ${message.ms.toFixed(0)} ms. No warnings.`
        : `Rendered in ${message.ms.toFixed(0)} ms with ${diagnostics.length} warning${diagnostics.length === 1 ? '' : 's'}.`,
    );
  } else {
    diagnostics.unshift({ ...response.error, severity: 'error' });
    markStale(true);
    setStatus(
      `${response.error.code}${response.error.path ? ` at ${response.error.path}` : ''}: ${response.error.message}`,
    );
  }
  showDiagnostics();
}

function markStale(stale: boolean): void {
  el.stale.hidden = !(stale && lastValid);
  el.preview.classList.toggle('stale', stale && lastValid);
  el.preview.toggleAttribute('aria-busy', false);
}

function renderAlternative(alternative: any): void {
  el.alternative.textContent = '';
  if (!alternative) return;
  const description = document.createElement('p');
  description.textContent = alternative.description;
  el.alternative.append(description);
  const table = alternative.table;
  if (table?.columns) {
    const node = document.createElement('table');
    const caption = document.createElement('caption');
    caption.textContent = table.caption;
    node.append(caption);
    const head = node.createTHead().insertRow();
    for (const column of table.columns) {
      const cell = document.createElement('th');
      cell.scope = 'col';
      cell.textContent = column;
      head.append(cell);
    }
    const body = node.createTBody();
    for (const row of table.rows) {
      const tr = body.insertRow();
      row.forEach((value: string, index: number) => {
        const cell = index === 0 ? document.createElement('th') : document.createElement('td');
        if (index === 0) (cell as HTMLTableCellElement).scope = 'row';
        cell.textContent = String(value);
        tr.append(cell);
      });
    }
    el.alternative.append(node);
  }
}

// ---- diagnostics at the field they name ---------------------------------------------------------

function showDiagnostics(): void {
  el.diagnostics.textContent = '';
  for (const diagnostic of diagnostics) {
    const item = document.createElement('li');
    item.className = diagnostic.severity;
    const title = document.createElement('p');
    const badge = document.createElement('strong');
    badge.textContent = `${diagnostic.severity === 'error' ? 'Error' : 'Warning'} ${diagnostic.code}`;
    title.append(badge, document.createTextNode(diagnostic.path ? ` at ${diagnostic.path}` : ''));
    const body = document.createElement('p');
    body.textContent = diagnostic.message;
    item.append(title, body);
    const actions = document.createElement('p');
    actions.className = 'actions';
    if (diagnostic.path && diagnostic.path !== '/') {
      const go = document.createElement('button');
      go.type = 'button';
      go.textContent = 'Go to field';
      go.addEventListener('click', () => goToField(diagnostic.path as string));
      actions.append(go);
    }
    const docs = document.createElement('a');
    docs.href = `${base}docs/guides/warnings-and-errors/`;
    docs.textContent = 'Warnings and errors reference';
    const schema = document.createElement('a');
    schema.href = `${base}docs/reference/specification/`;
    schema.textContent = 'Specification reference';
    actions.append(docs, schema);
    item.append(actions);
    for (const entry of help.filter((h) => h.code === diagnostic.code && h.path === diagnostic.path)) {
      item.append(helpBlock(entry));
    }
    el.diagnostics.append(item);
  }
  el.diagnostics.hidden = diagnostics.length === 0;
}

function helpBlock(entry: Help): HTMLElement {
  const block = document.createElement('div');
  block.className = 'help';
  if (entry.note) {
    const note = document.createElement('p');
    note.textContent = entry.note;
    block.append(note);
  }
  for (const remedy of entry.remedies) {
    const row = document.createElement('p');
    row.className = 'remedy';
    const summary = document.createElement('span');
    summary.textContent = `${remedy.summary} `;
    row.append(summary);
    if (remedy.patch.length > 0) {
      const state = document.createElement('em');
      state.textContent = remedy.validated
        ? remedy.resolves
          ? '(checked with the compiler: resolves the warning) '
          : '(checked with the compiler: does not resolve it) '
        : '(not checked) ';
      row.append(state);
      const apply = document.createElement('button');
      apply.type = 'button';
      apply.textContent = 'Apply this change';
      apply.addEventListener('click', () => applyPatch(remedy));
      row.append(apply);
    } else {
      const state = document.createElement('em');
      state.textContent = '(changes your content; chartlet does not apply it)';
      row.append(state);
    }
    block.append(row);
  }
  return block;
}

/** The change a remedy proposes, applied to the text only when the person chooses it. */
function applyPatch(remedy: Remedy): void {
  let next = text;
  for (const operation of remedy.patch) {
    const range = locate(next, operation.path);
    const value = JSON.stringify(operation.value);
    if (range?.exact) {
      next = next.slice(0, range.start) + value + next.slice(range.end);
    } else {
      const spec = parse(next);
      if (!spec) return;
      setPointer(spec, operation.path, operation.value);
      next = JSON.stringify(spec, null, 2);
    }
  }
  setText(next);
  setStatus(`Applied: ${remedy.summary} Undo restores the previous specification.`);
}

function setPointer(root: Json, pointer: string, value: unknown): void {
  const steps = pointer.split('/').slice(1).map((s) => s.replaceAll('~1', '/').replaceAll('~0', '~'));
  let node: any = root;
  for (const step of steps.slice(0, -1)) node = node[step];
  node[steps[steps.length - 1]] = value;
}

function goToField(pointer: string): void {
  if (mode === 'guided') {
    const field = el.guided.querySelector<HTMLElement>(`[data-pointer="${CSS.escape(pointer)}"]`);
    if (field) {
      field.focus();
      field.setAttribute('aria-invalid', 'true');
      return;
    }
    setMode('json');
  }
  const range = locate(text, pointer);
  el.editor.focus();
  if (range) el.editor.setSelectionRange(range.start, range.end);
  const line = text.slice(0, range?.start ?? 0).split('\n').length - 1;
  el.editor.scrollTop = Math.max(0, line - 3) * parseFloat(getComputedStyle(el.editor).lineHeight || '20');
}

// ---- the guided editor: bar chart and flow diagram ------------------------------------------------

function setMode(next: 'json' | 'guided'): void {
  if (next === 'guided' && !parse(text)) return;
  mode = next;
  el.modeJson.setAttribute('aria-pressed', String(mode === 'json'));
  el.modeGuided.setAttribute('aria-pressed', String(mode === 'guided'));
  el.editor.hidden = mode !== 'json';
  el.guided.hidden = mode !== 'guided';
  if (mode === 'guided') renderGuided();
  else el.editor.value = text;
}

function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attributes: Record<string, string> = {},
  ...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [name, value] of Object.entries(attributes)) node.setAttribute(name, value);
  node.append(...children);
  return node;
}

/** Changes the canonical specification from the form; every field the form does not know stays. */
function edit(change: (spec: Json) => void, { rebuild = false } = {}): void {
  const spec = parse(text);
  if (!spec) return;
  change(spec);
  text = JSON.stringify(spec, null, 2);
  el.editor.value = text;
  commit();
  syncControls();
  scheduleRender();
  if (rebuild) renderGuided();
}

function input(
  label: string,
  pointer: string,
  value: string | number | undefined,
  apply: (raw: string, field: HTMLInputElement) => void,
  type: 'text' | 'number' = 'text',
): HTMLInputElement {
  const field = h('input', {
    type,
    'aria-label': label,
    'data-pointer': pointer,
    value: value === undefined ? '' : String(value),
  });
  if (type === 'number') field.setAttribute('step', 'any');
  field.addEventListener('input', () => {
    field.removeAttribute('aria-invalid');
    apply(field.value, field);
  });
  return field;
}

function numberApply(set: (spec: Json, value: number) => void) {
  return (raw: string, field: HTMLInputElement) => {
    const value = Number(raw);
    if (raw.trim() === '' || Number.isNaN(value)) {
      // An incomplete edit stays in the field; the specification and the preview keep the last number.
      field.setAttribute('aria-invalid', 'true');
      markStale(true);
      setStatus('Enter a number. The preview shows the last valid value.');
      return;
    }
    edit((spec) => set(spec, value));
  };
}

function field(label: string, control: HTMLElement): HTMLElement {
  const id = `pg-f-${Math.random().toString(36).slice(2, 8)}`;
  control.id = id;
  return h('div', { class: 'field' }, h('label', { for: id }, label), control);
}

function renderGuided(): void {
  const spec = parse(text);
  el.guided.textContent = '';
  if (!spec) return;
  const known = new Set(['schemaVersion', 'type', 'title', 'source', 'width', 'height', 'theme', 'data', 'flow', 'mobile']);
  const kept = Object.keys(spec).filter((key) => !known.has(key));

  const heading = h('h3', {}, spec.type === 'bar' && Array.isArray(spec.data) ? 'Bar chart' : spec.type === 'flow' ? 'Flow diagram' : 'Not supported here');
  el.guided.append(heading);

  const supportedBar = spec.type === 'bar' && Array.isArray(spec.data) && !spec.series;
  const supportedFlow = spec.type === 'flow' && spec.flow && Array.isArray(spec.flow.nodes);
  if (!supportedBar && !supportedFlow) {
    el.guided.append(
      h(
        'p',
        { class: 'note' },
        `The form edits a simple bar chart (one series in "data") and a flow diagram. This specification is "${String(spec.type)}"${spec.series ? ' with series' : ''}: edit it in the JSON view. Nothing was changed.`,
      ),
    );
    return;
  }

  el.guided.append(
    field('Title', input('Title', '/title', spec.title, (raw) => edit((s) => (s.title = raw)))),
    field('Source', input('Source', '/source', spec.source, (raw) => edit((s) => (raw ? (s.source = raw) : delete s.source)))),
    h(
      'div',
      { class: 'row' },
      field('Width', input('Width in pixels', '/width', spec.width, numberApply((s, v) => (s.width = v)), 'number')),
      field('Height', input('Height in pixels', '/height', spec.height, numberApply((s, v) => (s.height = v)), 'number')),
    ),
  );
  if (supportedBar) guidedBar(spec);
  else guidedFlow(spec);
  if (kept.length > 0) {
    el.guided.append(
      h(
        'p',
        { class: 'note' },
        `Kept as they are, edit them in the JSON view: ${kept.map((k) => `"${k}"`).join(', ')}. The theme and the size are set with the controls above.`,
      ),
    );
  }
}

function tableOf(caption: string, headings: string[]): { table: HTMLTableElement; body: HTMLTableSectionElement } {
  const table = h('table', { class: 'guided-table' }, h('caption', {}, caption));
  const row = table.createTHead().insertRow();
  for (const heading of headings) row.append(h('th', { scope: 'col' }, heading));
  return { table, body: table.createTBody() };
}

function guidedBar(spec: Json): void {
  const { table, body } = tableOf('Values', ['Label', 'Value', 'Row']);
  (spec.data as Json[]).forEach((point, index) => {
    const row = body.insertRow();
    const remove = h('button', { type: 'button' }, 'Remove');
    remove.setAttribute('aria-label', `Remove row ${index + 1}`);
    remove.addEventListener('click', () => edit((s) => s.data.splice(index, 1), { rebuild: true }));
    row.append(
      h('td', {}, input(`Label of row ${index + 1}`, `/data/${index}/label`, point.label, (raw) => edit((s) => (s.data[index].label = raw)))),
      h('td', {}, input(`Value of row ${index + 1}`, `/data/${index}/value`, point.value, numberApply((s, v) => (s.data[index].value = v)), 'number')),
      h('td', {}, remove),
    );
  });
  const add = h('button', { type: 'button' }, 'Add a row');
  add.addEventListener('click', () => edit((s) => s.data.push({ label: `Category ${s.data.length + 1}`, value: 0 }), { rebuild: true }));
  el.guided.append(table, add);
}

const KINDS = ['', 'start', 'end', 'decision', 'subprocess', 'io', 'store'];

function guidedFlow(spec: Json): void {
  const flow = spec.flow as Json;
  const nodes = flow.nodes as Json[];
  const edges = (flow.edges ?? []) as Json[];
  const { table, body } = tableOf('Steps', ['Id', 'Label', 'Kind', 'Row']);
  nodes.forEach((node, index) => {
    const row = body.insertRow();
    const kind = h('select', { 'aria-label': `Kind of step ${index + 1}`, 'data-pointer': `/flow/nodes/${index}/kind` });
    for (const value of KINDS) kind.append(h('option', { value }, value || 'process'));
    kind.value = node.kind ?? '';
    kind.addEventListener('change', () => edit((s) => (kind.value ? (s.flow.nodes[index].kind = kind.value) : delete s.flow.nodes[index].kind)));
    const remove = h('button', { type: 'button' }, 'Remove');
    remove.setAttribute('aria-label', `Remove step ${index + 1}`);
    remove.addEventListener('click', () => {
      const id = node.id;
      edit(
        (s) => {
          s.flow.nodes.splice(index, 1);
          s.flow.edges = (s.flow.edges ?? []).filter((e: Json) => e.from !== id && e.to !== id);
        },
        { rebuild: true },
      );
      setStatus(`Removed step ${id} and the connections that used it.`);
    });
    row.append(
      h(
        'td',
        {},
        input(`Id of step ${index + 1}`, `/flow/nodes/${index}/id`, node.id, (raw) => {
          const old = nodes[index].id;
          edit((s) => {
            s.flow.nodes[index].id = raw;
            for (const e of s.flow.edges ?? []) {
              if (e.from === old) e.from = raw;
              if (e.to === old) e.to = raw;
            }
          });
        }),
      ),
      h('td', {}, input(`Label of step ${index + 1}`, `/flow/nodes/${index}/label`, node.label, (raw) => edit((s) => (s.flow.nodes[index].label = raw)))),
      h('td', {}, kind),
      h('td', {}, remove),
    );
  });
  const addNode = h('button', { type: 'button' }, 'Add a step');
  addNode.addEventListener('click', () => {
    const used = new Set(nodes.map((n) => n.id));
    let n = nodes.length + 1;
    while (used.has(`step-${n}`)) n += 1;
    edit((s) => s.flow.nodes.push({ id: `step-${n}`, label: `Step ${n}` }), { rebuild: true });
  });
  el.guided.append(table, addNode);

  const { table: edgeTable, body: edgeBody } = tableOf('Connections', ['From', 'To', 'Label', 'Row']);
  edges.forEach((edge, index) => {
    const row = edgeBody.insertRow();
    const select = (which: 'from' | 'to') => {
      const control = h('select', { 'aria-label': `${which === 'from' ? 'From' : 'To'} step of connection ${index + 1}`, 'data-pointer': `/flow/edges/${index}/${which}` });
      for (const node of nodes) control.append(h('option', { value: node.id }, `${node.id}`));
      if (!nodes.some((n) => n.id === edge[which])) control.append(h('option', { value: edge[which] ?? '' }, `${edge[which] ?? ''} (unknown)`));
      control.value = edge[which] ?? '';
      control.addEventListener('change', () => edit((s) => (s.flow.edges[index][which] = control.value)));
      return control;
    };
    const remove = h('button', { type: 'button' }, 'Remove');
    remove.setAttribute('aria-label', `Remove connection ${index + 1}`);
    remove.addEventListener('click', () => edit((s) => s.flow.edges.splice(index, 1), { rebuild: true }));
    row.append(
      h('td', {}, select('from')),
      h('td', {}, select('to')),
      h('td', {}, input(`Label of connection ${index + 1}`, `/flow/edges/${index}/label`, edge.label, (raw) => edit((s) => (raw ? (s.flow.edges[index].label = raw) : delete s.flow.edges[index].label)))),
      h('td', {}, remove),
    );
  });
  const addEdge = h('button', { type: 'button' }, 'Add a connection');
  addEdge.addEventListener('click', () => {
    if (nodes.length < 2) return setStatus('Add at least two steps first.');
    edit((s) => (s.flow.edges = [...(s.flow.edges ?? []), { from: nodes[0].id, to: nodes[1].id }]), { rebuild: true });
  });
  el.guided.append(edgeTable, addEdge);
}

// ---- export ---------------------------------------------------------------------------------------

function download(name: string, content: string, type: string): void {
  const url = URL.createObjectURL(new Blob([content], { type }));
  const link = h('a', { href: url, download: name });
  document.body.append(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

async function exportOutput(format: 'svg' | 'html'): Promise<void> {
  if (!ready) return setStatus('The compiler is not loaded.');
  try {
    const { response } = await compile(
      text,
      renderOptions({ format, alternative: false, advice: false, idPrefix: undefined }),
    );
    if (!response.ok) return setStatus(`Nothing to download: ${response.error.code} at ${response.error.path}.`);
    download(`chartlet.${format}`, response.content, format === 'svg' ? 'image/svg+xml' : 'text/html');
    setStatus(`Downloaded chartlet.${format}. PNG is not offered in the browser; the CLI renders it.`);
  } catch (error) {
    setStatus((error as Error).message);
  }
}

// ---- wiring -----------------------------------------------------------------------------------------

function applyTheme(value: string): void {
  const spec = parse(text);
  if (!spec) return;
  edit((s) => (value === 'light' ? delete s.theme : (s.theme = value)), { rebuild: mode === 'guided' });
}

function init(): void {
  for (const example of examples) el.example.append(h('option', { value: example.id }, example.label));
  const choose = (id: string) => {
    const example = examples.find((e) => e.id === id) ?? examples[0];
    initial = example.json;
    history = [];
    cursor = -1;
    lastValid = false;
    el.preview.textContent = '';
    setText(example.json);
    setStatus(`Loaded the example "${example.label}".`);
  };
  el.example.addEventListener('change', () => {
    if (text !== initial && !confirm('Replace your changes with the chosen example? Undo can bring them back.')) {
      el.example.value = examples.find((e) => e.json === initial)?.id ?? examples[0].id;
      return;
    }
    const previous = text;
    choose(el.example.value);
    // Keep what was there reachable by undo.
    history = [previous, text];
    cursor = 1;
    syncHistoryButtons();
  });
  el.editor.addEventListener('input', () => {
    text = el.editor.value;
    onTextChanged();
  });
  el.modeJson.addEventListener('click', () => setMode('json'));
  el.modeGuided.addEventListener('click', () => setMode('guided'));
  el.theme.addEventListener('change', () => applyTheme(el.theme.value));
  el.variant.addEventListener('change', () => {
    const spec = parse(text);
    if (el.variant.value === 'mobile' && spec && !spec.mobile) {
      edit((s) => (s.mobile = { width: 360 }));
      setStatus('Added "mobile": { "width": 360 } to the specification: the mobile layout needs it.');
    }
    scheduleRender(0);
  });
  el.undo.addEventListener('click', () => {
    if (cursor > 0) {
      cursor -= 1;
      setText(history[cursor], { record: false });
      syncHistoryButtons();
    }
  });
  el.redo.addEventListener('click', () => {
    if (cursor < history.length - 1) {
      cursor += 1;
      setText(history[cursor], { record: false });
      syncHistoryButtons();
    }
  });
  el.reset.addEventListener('click', () => {
    if (confirm('Discard all changes and go back to the example as it was loaded?')) {
      const previous = text;
      setText(initial);
      history = [previous, initial];
      cursor = 1;
      syncHistoryButtons();
      setStatus('Reset to the example. Undo restores your changes.');
    }
  });
  el.copy.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(text);
      setStatus('Copied the specification JSON.');
    } catch {
      setStatus('Copying was not allowed here; use Download JSON.');
    }
  });
  el.downloadJson.addEventListener('click', () => {
    download('chartlet.json', text.endsWith('\n') ? text : `${text}\n`, 'application/json');
    setStatus('Downloaded chartlet.json: the exact specification of this figure.');
  });
  el.downloadSvg.addEventListener('click', () => exportOutput('svg'));
  el.downloadHtml.addEventListener('click', () => exportOutput('html'));
  choose(new URLSearchParams(location.search).get('example') ?? examples[0].id);
  el.example.value = examples.find((e) => e.json === initial)?.id ?? examples[0].id;
  setMode('json');
  if (typeof WebAssembly === 'undefined' || typeof Worker === 'undefined') {
    loadFailed = true;
    showFailure('This browser has no WebAssembly or no module workers.');
  } else {
    setStatus('Loading the compiler…');
    startWorker();
  }
  void wasmBytes;
}

init();
