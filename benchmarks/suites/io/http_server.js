// Suite: io
// Benchmark: http_server
// Spawns an internal HTTP server and measures round-trip latency over TCP.

const http = require('http');
const net = require('net');

const PORT = 9877;
const ITER = 1000;
let serverReady = false;

const server = http.createServer((req, res) => {
  let body = '';
  req.on('data', chunk => body += chunk);
  req.on('end', () => {
    res.writeHead(200);
    res.end('ok:' + body.length);
  });
});

server.listen(PORT, () => {
  serverReady = true;
  run();
});

function run() {
  if (!serverReady) return;
  const t0 = Date.now();
  let done = 0;
  for (let i = 0; i < ITER; i++) {
    await new Promise((resolve, reject) => {
      const c = net.createConnection(PORT, () => {
        c.write('hello');
        c.end();
      });
      c.on('data', () => resolve());
      c.on('error', reject);
      setTimeout(() => reject(new Error('timeout')), 500);
    });
    done++;
  }
  const elapsed = Date.now() - t0;
  console.log(elapsed);
  console.log(done);
  server.close();
}

setTimeout(() => {
  if (!serverReady) {
    console.log('SKIP');
    process.exit(0);
  }
}, 2000);
