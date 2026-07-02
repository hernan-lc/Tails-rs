// HTTP server module example for Tails-rs
// Run: cargo run --release --bin tails -- run examples/http-server.ts
//
// Demonstrates the `http` native module: createServer + listen + close.
// The server stays alive until explicitly closed or the process exits.
//
// Try:
//   curl http://127.0.0.1:9877/
//   curl -d hello http://127.0.0.1:9877/echo
//   curl http://127.0.0.1:9877/json

import http from "http";

const PORT = 9877;

const server = http.createServer((req, res) => {
  console.log(`${req.method} ${req.url}`);

  if (req.url === "/json") {
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ message: "Hello from Tails!", timestamp: Date.now() }));
    return;
  }

  if (req.url === "/echo") {
    let body = "";
    req.on("data", (chunk: string) => (body += chunk));
    req.on("end", () => {
      res.writeHead(200);
      res.end("echo: " + body);
    });
    return;
  }

  // Default response
  res.writeHead(200);
  res.end("Hello from Tails-rs HTTP server!\nTry: /json, /echo");
});

// Listen without timeout — server stays alive until process exits
server.listen(PORT, () => {
  console.log(`Server listening on http://127.0.0.1:${PORT}`);
  console.log("Endpoints: /, /json, /echo");
  console.log("Press Ctrl+C to stop.");
}, { timeoutMs: 86400000 }); // 24 hours
