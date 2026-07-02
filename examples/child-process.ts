// child_process usage example for Tails-rs
// Run: tails run examples/child-process.ts
//
// Demonstrates: execSync, exec, spawn for running shell commands.

import child_process from "child_process";

// ---------------------------------------------------------------------------
// execSync — synchronous command execution
// ---------------------------------------------------------------------------

console.log("=== execSync ===");

// Simple command
const hostname = child_process.execSync("hostname");
console.log("hostname:", hostname);

// Command with output
const date = child_process.execSync("date -Iseconds");
console.log("date:", date);

// ---------------------------------------------------------------------------
// execSync — capture output
// ---------------------------------------------------------------------------

console.log("\n=== execSync output ===");

const ls = child_process.execSync("ls -la /tmp | head -5");
console.log("ls output:\n" + ls);

// ---------------------------------------------------------------------------
// execSync — working directory
// ---------------------------------------------------------------------------

console.log("\n=== execSync working directory ===");

const pwd = child_process.execSync("pwd");
console.log("current dir:", pwd);

// ---------------------------------------------------------------------------
// execSync — whoami
// ---------------------------------------------------------------------------

console.log("\n=== execSync whoami ===");

const whoami = child_process.execSync("whoami");
console.log("whoami:", whoami);

// ---------------------------------------------------------------------------
// spawn — asynchronous process spawning
// ---------------------------------------------------------------------------

console.log("\n=== spawn ===");

const spawned = child_process.spawn("echo", ["hello", "from", "spawn"]);
console.log("spawn type:", typeof spawned);

// ---------------------------------------------------------------------------
// exec — asynchronous command execution
// ---------------------------------------------------------------------------

console.log("\n=== exec ===");

console.log("exec type:", typeof child_process.exec);

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

console.log("\n=== All child_process patterns completed ===");
