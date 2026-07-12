import http from "http";
import { setInterval, setTimeout } from "timers";

// Define the port the server will listen on
const PORT = 3000;

// Create the HTTP server
const server = http.createServer((req, res) => {
  // Get the request method and URL path
  const { method, url } = req;

  console.log(`[${new Date().toISOString()}] ${method} ${url}`);

  // Route 1: Home Page
  if (url === "/" && method === "GET") {
    res.writeHead(200, { "Content-Type": "text/html" });
    res.end("<h1>Welcome to my Node.js HTTP Server!</h1><p>Try visiting <a href='/api/user'>/api/user</a></p>");
  }

  // Route 2: JSON API Endpoint
  else if (url === "/api/user" && method === "GET") {
    const userData = {
      id: 1,
      name: "Alex Dev",
      role: "Full Stack Engineer",
      status: "Coding"
    };

    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify(userData));
  }

  // Route 3: Catch-all for 404 Not Found
  else {
    res.writeHead(404, { "Content-Type": "text/plain" });
    res.end("404 Not Found - The page you are looking for does not exist.");
  }
});

// Start the server
server.listen(PORT, () => {
  console.log(`🚀 Server is running and listening on http://localhost:${PORT}`);
});
setInterval(() => {
  console.log("tick");
}, 5000);
setTimeout(() => {
  console.log("exiting");
  process.exit();
}, 10000);
