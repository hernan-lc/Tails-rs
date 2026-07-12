import http from "http";
import fs from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";

const PORT = 3000;

// Replicate __dirname functionality in ES Modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Mock Database Memory Store
const usersDb = [
  { id: 1, name: "Alice", role: "Admin" },
  { id: 2, name: "Bob", role: "Developer" }
];

// Helper function to handle JSON responses cleanly
const sendJSON = (res, statusCode, data) => {
  res.writeHead(statusCode, { "Content-Type": "application/json" });
  res.end(JSON.stringify(data));
};

// Main Server Request Listener
const server = http.createServer(async (req, res) => {
  // Parse the URL and extract query string params (e.g., /api/users?role=Admin)
  const parsedUrl = new URL(req.url, `http://${req.headers.host}`);
  const pathname = parsedUrl.pathname;
  const method = req.method;

  console.log(`[${new Date().toLocaleTimeString()}] ${method} ${pathname}`);

  try {
    // ---------------------------------------------------------
    // ROUTE 1: GET / -> Serve Static HTML File
    // ---------------------------------------------------------
    if (pathname === "/" && method === "GET") {
      const filePath = path.join(__dirname, "examples", "public", "index.html");
      const htmlContent = await fs.readFile(filePath, "utf-8");

      res.writeHead(200, { "Content-Type": "text/html" });
      res.end(htmlContent);
    }

    // ---------------------------------------------------------
    // ROUTE 2: GET /api/users -> Supports filtering via Query Parameters
    // ---------------------------------------------------------
    else if (pathname === "/api/users" && method === "GET") {
      const roleFilter = parsedUrl.searchParams.get("role");

      if (roleFilter) {
        const filteredUsers = usersDb.filter(
          u => u.role.toLowerCase() === roleFilter.toLowerCase()
        );
        return sendJSON(res, 200, filteredUsers);
      }

      sendJSON(res, 200, usersDb);
    }

    // ---------------------------------------------------------
    // ROUTE 3: POST /api/users -> Parse JSON Request Body & Add User
    // ---------------------------------------------------------
    else if (pathname === "/api/users" && method === "POST") {
      // This runtime buffers the full request body on `req.body` before the
      // handler runs, so we read it directly instead of streaming via
      // `req.on("data")`. The body is a string here (no Buffer wrapper).
      try {
        const payload = JSON.parse(req.body || "{}");

        // Simple Validation
        if (!payload.name || !payload.role) {
          return sendJSON(res, 400, { error: "Missing 'name' or 'role' fields" });
        }

        const newUser = {
          id: usersDb.length + 1,
          name: payload.name,
          role: payload.role
        };

        usersDb.push(newUser);
        sendJSON(res, 201, { message: "User created successfully", user: newUser });
      } catch (parseError) {
        sendJSON(res, 400, { error: "Invalid JSON payload format" });
      }
    }

    // ---------------------------------------------------------
    // ROUTE 4: Catch-all 404 Not Found
    // ---------------------------------------------------------
    else {
      sendJSON(res, 404, { error: `Route ${method} ${pathname} not found.` });
    }

  } catch (globalError) {
    // Top-level catch block to prevent the server from crashing on unhandled errors (e.g., file missing)
    console.error("Server Error:", globalError);
    sendJSON(res, 500, { error: "Internal Server Error" });
  }
});

// Start the server instance
server.listen(PORT, () => {
  console.log(`HTTP Server live on port ${PORT}`);
});
setInterval(() => {
  console.log("tick");
}, 5000);
fetch("http://localhost:3000/")
  .then((res) => res.text())
  .then((body) => console.log(body))
  .catch((err) => console.error(err));
setTimeout(() => {
  console.log("exiting");
  process.exit();
}, 15000);
