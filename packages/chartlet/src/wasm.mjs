// The renderer compiled to WebAssembly. No Node.js imports: runtimes that cannot compile
// WebAssembly from bytes at run time (workerd, some bundlers) import `chartlet.wasm` as a module
// and pass it in.

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function createRenderer(module) {
  const { memory, alloc, dealloc, render, result_len } = new WebAssembly.Instance(module, {})
    .exports;

  // Sends one request to the renderer and returns its parsed JSON response.
  function send(message) {
    const request = encoder.encode(JSON.stringify(message));
    const input = alloc(request.length);
    new Uint8Array(memory.buffer, input, request.length).set(request);
    const output = render(input, request.length);
    const length = result_len();
    const response = JSON.parse(decoder.decode(new Uint8Array(memory.buffer, output, length)));
    dealloc(input, request.length);
    dealloc(output, length);
    return response;
  }

  // Renders one chart and returns the renderer's response.
  function respond(spec, options) {
    return send({
      spec,
      options: {
        format: options.format ?? "html",
        table: options.table ?? "details",
        idPrefix: options.idPrefix || undefined,
        variant: options.variant,
        strict: options.strict ?? false,
        allowWarnings: options.allowWarnings ?? [],
        manifest: options.manifest ?? false,
        alternative: options.alternative ?? false,
        styles: options.styles ?? "inline",
      },
    });
  }

  return {
    renderChart(spec, options = {}) {
      const response = respond(JSON.stringify(spec), options);

      // The same lines the CLI writes to stderr.
      const warnings = response.warnings.map(
        ({ code, path, message }) => `warning[${code}] at ${path}: ${message}`,
      );
      if (!response.ok) {
        const { code, path, message } = response.error;
        const failure = path == null ? `chartlet: ${message}` : `chartlet: ${code} at ${path}: ${message}`;
        throw new Error([...warnings, failure].join("\n"));
      }
      const result = { content: response.content, styleHashes: response.styleHashes, warnings };
      if (response.manifest) {
        result.manifest = response.manifest;
      }
      if (response.alternative) {
        result.alternative = response.alternative;
      }
      return result;
    },

    // The renderer's response as it is: `ok`, `error` or `content`, `styleHashes`, `manifest`
    // and `alternative`, and the warnings as `{ code, path, message }`. A string is taken as JSON text.
    renderChartDetailed(spec, options = {}) {
      return respond(typeof spec === "string" ? spec : JSON.stringify(spec), options);
    },

    // The shared stylesheet that charts rendered with `styles: "external"` rely on.
    stylesheet() {
      return send({ stylesheet: true }).stylesheet;
    },
  };
}
