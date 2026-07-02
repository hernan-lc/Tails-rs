// Suite: io
// Benchmark: http_server
// Spawns an internal HTTP server and measures round-trip latency over TCP.
//
// Works with both event-loop runtimes (Node, Bun) and synchronous runtimes
// (tails).  The server listen call blocks until all connections are handled,
// so client connections are created inside the ready callback.

let http;
let net;
try {
  http = require('http');
  net = require('net');
} catch (e) {
  console.log('SKIP');
  process.exit(0);
}

if (!http || !net || !http.createServer || !net.createConnection) {
  console.log('SKIP');
  process.exit(0);
}

const PORT = 9877;
const ITER = 1000;

const server = http.createServer((req, res) => {
  res.writeHead(200);
  res.end('ok');
});

const t0 = Date.now();
server.listen(PORT, () => {
  for (let i = 0; i < ITER; i++) {
    const c = net.createConnection(PORT);
    c.write('hello');
    c.end();
  }
}, { maxConnections: ITER, timeoutMs: 60000 });
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(ITER);
server.close();
