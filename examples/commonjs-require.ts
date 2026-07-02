// CommonJS require() example for Tails-rs
// Run: tails run examples/commonjs-require.ts
//
// Demonstrates: require(), module.exports, exports, __dirname, __filename,
// and CJS module caching.

import path from "path";

// ---------------------------------------------------------------------------
// Basic require
// ---------------------------------------------------------------------------

console.log("=== Basic require ===");

// Use path.resolve to get absolute path (required for require)
const mathPath = path.resolve(__dirname, "cjs", "math.cjs");
const math = require(mathPath);
console.log("math.PI:", math.PI);
console.log("math.add(2, 3):", math.add(2, 3));
console.log("math.greeting:", math.greeting);

// ---------------------------------------------------------------------------
// Module that exports a function
// ---------------------------------------------------------------------------

console.log("\n=== Module exporting a function ===");

const greeterPath = path.resolve(__dirname, "cjs", "greeter.cjs");
const greet = require(greeterPath);
console.log("greet('World'):", greet("World"));
console.log("greet('Tails'):", greet("Tails"));

// ---------------------------------------------------------------------------
// Module caching — require returns the same object
// ---------------------------------------------------------------------------

console.log("\n=== Module caching ===");

const mathAgain = require(mathPath);
console.log("Same module?", math === mathAgain);

// ---------------------------------------------------------------------------
// __dirname and __filename
// ---------------------------------------------------------------------------

console.log("\n=== __dirname and __filename ===");
console.log("__dirname:", __dirname);
console.log("__filename:", __filename);

// ---------------------------------------------------------------------------
// exports object with state
// ---------------------------------------------------------------------------

console.log("\n=== exports object ===");

const counterPath = path.resolve(__dirname, "cjs", "counter.cjs");
const counter = require(counterPath);
console.log("counter.getValue():", counter.getValue());
counter.increment();
console.log("after increment:", counter.getValue());
counter.increment();
console.log("after second increment:", counter.getValue());

console.log("\n=== All CommonJS patterns completed ===");
