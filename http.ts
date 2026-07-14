// HTTP server example — uses the native http module (no Node deps)
// Run: cargo run --bin tails -- run http.ts
//
// Try:
//   curl http://127.0.0.1:3000/
//   curl -d hello http://127.0.0.1:3000/echo
//
// This replaces the original fastify-based version, which cannot run on
// Tails because fastify 5.x depends on Node.js-only stream/symbol APIs that
// are not (yet) provided by the runtime. See docs/compat.md for the
// current framework-compat matrix.

import http from "http";

const PORT = 3000;

const server = http.createServer((req, res) => {
  console.log(`${req.method} ${req.url}`);

  if (req.url === "/echo") {
    let body = "";
    req.on("data", (chunk: string) => (body += chunk));
    req.on("end", () => {
      res.writeHead(200);
      res.end("echo: " + body);
    });
    return;
  }

  res.writeHead(200);
  res.end("Hello from Tails-rs!\nTry: /echo\n");
});

server.listen(PORT, () => {
  console.log(`Server listening on http://127.0.0.1:${PORT}`);

  // Kick a request to ourselves so the event loop has something to flush.
  fetch(`http://localhost:${PORT}/`)
    .then((r) => r.text())
    .then((body) => {
      console.log("self-fetch body:\n" + body);
      console.log("shutting down…");
      server.close();
    })
    .catch((err) => {
      console.error("self-fetch failed:", err);
      process.exit(1);
    });
});
