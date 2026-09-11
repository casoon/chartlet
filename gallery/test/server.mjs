import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, resolve } from 'node:path';

const base = '/chartlet/';
const root = resolve('dist');
const contentTypes = {
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.svg': 'image/svg+xml',
};

createServer(async (request, response) => {
  const pathname = new URL(request.url ?? '/', 'http://127.0.0.1').pathname;
  if (!pathname.startsWith(base)) {
    response.writeHead(404).end();
    return;
  }

  const relative = pathname.slice(base.length);
  const candidate = resolve(
    root,
    relative === '' || relative.endsWith('/') ? `${relative}index.html` : relative,
  );
  if (!candidate.startsWith(`${root}/`)) {
    response.writeHead(404).end();
    return;
  }

  try {
    const content = await readFile(candidate);
    response.writeHead(200, { 'content-type': contentTypes[extname(candidate)] ?? 'application/octet-stream' });
    response.end(content);
  } catch {
    response.writeHead(404).end();
  }
}).listen(4321, '127.0.0.1');
