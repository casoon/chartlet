// A page that renders the monthly-revenue chart as SVG and HTML, the way
// packages/chartlet/src/wasm.mjs talks to the module: copy the request in, render, read, free.
import { readFileSync } from "node:fs";

const spec = readFileSync(new URL("../../examples/monthly-revenue.json", import.meta.url), "utf8");
const encoder = new TextEncoder();
const decoder = new TextDecoder();

export default async function ({ memory, alloc, dealloc, render, result_len }) {
  for (const format of ["svg", "html"]) {
    const request = encoder.encode(JSON.stringify({ spec, options: { format } }));
    const input = alloc(request.length);
    new Uint8Array(memory.buffer, input, request.length).set(request);
    const output = render(input, request.length);
    const length = result_len();
    const response = JSON.parse(decoder.decode(new Uint8Array(memory.buffer, output, length)));
    dealloc(input, request.length);
    dealloc(output, length);
    if (!response.ok) throw new Error(`chartlet: ${response.error.message}`);
  }
}
