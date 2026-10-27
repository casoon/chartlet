// The compiler of the playground: chartlet's own WebAssembly renderer in a worker, so that an
// invalid or expensive specification can never freeze the page. A render that traps leaves the
// instance in an unknown state, so the worker instantiates the compiled module again before it
// answers the next request.
import { createRenderer } from '@casoon/chartlet/wasm';

type Renderer = ReturnType<typeof createRenderer>;

let module: WebAssembly.Module | undefined;
let renderer: Renderer | undefined;

self.onmessage = async (event: MessageEvent) => {
  const message = event.data;
  if (message.type === 'init') {
    try {
      const response = await fetch(message.wasmUrl);
      if (!response.ok) throw new Error(`${message.wasmUrl} answered ${response.status}`);
      const bytes = await response.arrayBuffer();
      module = await WebAssembly.compile(bytes);
      renderer = createRenderer(module);
      self.postMessage({ type: 'ready', bytes: bytes.byteLength });
    } catch (error) {
      self.postMessage({ type: 'failed', message: String(error instanceof Error ? error.message : error) });
    }
    return;
  }
  if (message.type === 'render') {
    if (!renderer || !module) {
      self.postMessage({ type: 'crash', id: message.id, message: 'The renderer is not loaded.' });
      return;
    }
    try {
      const started = performance.now();
      const response = renderer.renderChartDetailed(message.spec, message.options);
      self.postMessage({ type: 'result', id: message.id, response, ms: performance.now() - started });
    } catch (error) {
      renderer = createRenderer(module);
      self.postMessage({
        type: 'crash',
        id: message.id,
        message: String(error instanceof Error ? error.message : error),
      });
    }
  }
};
