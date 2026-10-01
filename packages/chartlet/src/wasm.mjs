// The renderer compiled to WebAssembly. No Node.js imports: runtimes that cannot compile
// WebAssembly from bytes at run time (workerd, some bundlers) import `chartlet.wasm` as a module
// and pass it in.

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function createRenderer(module) {
  const { memory, alloc, dealloc, render, result_len } = new WebAssembly.Instance(module, {})
    .exports;

  return {
    renderChart(spec, options = {}) {
      const request = encoder.encode(
        JSON.stringify({
          spec: JSON.stringify(spec),
          options: {
            format: options.format ?? "html",
            table: options.table ?? "details",
            idPrefix: options.idPrefix || undefined,
            variant: options.variant,
            strict: options.strict ?? false,
          },
        }),
      );
      const input = alloc(request.length);
      new Uint8Array(memory.buffer, input, request.length).set(request);
      const output = render(input, request.length);
      const length = result_len();
      const response = JSON.parse(decoder.decode(new Uint8Array(memory.buffer, output, length)));
      dealloc(input, request.length);
      dealloc(output, length);

      // The same lines the CLI writes to stderr.
      const warnings = response.warnings.map(
        ({ code, path, message }) => `warning[${code}] at ${path}: ${message}`,
      );
      if (!response.ok) {
        const { code, path, message } = response.error;
        const failure = path == null ? `chartlet: ${message}` : `chartlet: ${code} at ${path}: ${message}`;
        throw new Error([...warnings, failure].join("\n"));
      }
      return { content: response.content, warnings };
    },
  };
}
