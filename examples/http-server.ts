// HTTP server module example for Tails-rs
// Run: cargo run --release --bin tails -- run examples/http-server.ts
//
// Demonstrates the `http` native module: createServer + listen + close.
// Because Tails uses a single-threaded cooperative event loop, `listen` runs a
// bounded accept loop. Pass options { maxConnections, timeoutMs } to control it.

import http from "http";

const PORT = 9877;

const server = http.createServer((req, res) => {
  // req.body is the full request body (collected synchronously).
  // req.on('data'/'end') fire immediately for the same effect.
  let body = "";
  req.on("data", (chunk) => (body += chunk));
  req.on("end", () => {
    res.writeHead(200);
    res.end("ok:" + body.length);
  });
});

console.log("Listening on http://127.0.0.1:" + PORT);

server.listen(PORT, () => {
  console.log("Server ready — send a request with: curl -d hello http://127.0.0.1:" + PORT + "/echo");
}, { maxConnections: 5, timeoutMs: 8000 });

console.log("Server closed after handling requests (or timeout).");