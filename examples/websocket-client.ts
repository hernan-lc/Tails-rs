// WebSocket client example for Tails-rs
// Run: tails run examples/websocket-client.ts
//
// Demonstrates the WebSocket API: constructor, send, close, event listeners.

// Create a WebSocket connection (this is a structural demo — actual connection
// requires a running WebSocket server like `wscat -l 9001`).

const ws = new WebSocket("ws://127.0.0.1:9001");

console.log("WebSocket created");
console.log("  URL:", ws.url);
console.log("  readyState:", ws.readyState, "(0=CONNECTING)");

// Register event listeners
ws.addEventListener("open", () => {
  console.log("Connected!");
  ws.send("Hello from Tails!");
});

ws.addEventListener("message", (event: any) => {
  console.log("Received:", event.data || event);
});

ws.addEventListener("close", (event: any) => {
  console.log("Disconnected:", event.code || "unknown");
});

ws.addEventListener("error", (event: any) => {
  console.log("Error:", event.message || "unknown");
});

// Close after 2 seconds
setTimeout(() => {
  console.log("Closing connection...");
  ws.close();
}, 2000);

console.log("Waiting for connection...");
