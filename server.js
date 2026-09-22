// Static host for the browser workbench.
//
// It serves `web/` — the shell, the renderer and the compiled Rust kernel — and
// nothing else. There is no engineering logic here: every number the page shows
// comes out of the WebAssembly kernel.

const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = process.env.PORT || 3000;
const ROOT = path.join(__dirname, 'web');

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.wasm': 'application/wasm',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
};

const server = http.createServer((request, response) => {
  // Cross-origin isolation: not required by the single-threaded build, but it keeps
  // the door open for a shared-memory kernel later without changing the host.
  response.setHeader('Cross-Origin-Opener-Policy', 'same-origin');
  response.setHeader('Cross-Origin-Embedder-Policy', 'require-corp');
  response.setHeader('Cross-Origin-Resource-Policy', 'cross-origin');

  const url = new URL(request.url, `http://${request.headers.host ?? 'localhost'}`);
  const requested = url.pathname === '/' ? '/index.html' : url.pathname;
  const filePath = path.join(ROOT, path.normalize(requested).replace(/^(\.\.[/\\])+/, ''));

  if (!filePath.startsWith(ROOT)) {
    response.writeHead(403, { 'Content-Type': 'text/plain' });
    response.end('Forbidden');
    return;
  }

  fs.readFile(filePath, (error, data) => {
    if (error) {
      response.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
      response.end('Not Found');
      return;
    }
    const contentType = MIME_TYPES[path.extname(filePath)] || 'application/octet-stream';
    response.writeHead(200, { 'Content-Type': contentType });
    response.end(data);
  });
});

server.listen(PORT, '0.0.0.0', () => {
  console.log(`Civil engineering workbench on http://0.0.0.0:${PORT}`);
});
