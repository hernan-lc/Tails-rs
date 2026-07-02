const http = require("http");
const net = require("net");
const PORT = 9881;
const N = 100;
const server = http.createServer((req, res) => {
  res.writeHead(200);
  res.end("ok");
});
const t0 = Date.now();
server.listen(PORT, () => {
  const t1 = Date.now();
  for (let i = 0; i < N; i++) {
    const c = net.createConnection(PORT);
    c.write("hello");
    c.end();
  }
  const t2 = Date.now();
  console.log("connections created in " + (t2 - t1) + "ms");
}, { maxConnections: N, timeoutMs: 60000 });
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(N);
server.close();
