// ============================================================
// Tails-rs — modular_demo.ts
// Run:  cargo run --bin tails -- examples/modular_demo.ts
//
// Demonstrates ES-module style imports/exports:
//   - Named imports
//   - Default imports
//   - Namespace imports (import *)
//   - Re-exporting
// ============================================================

// --- Named imports ---
import { add, multiply, PI } from "./math_utils";

console.log("--- Named Imports ---");
console.log("add:", add(3, 4));
console.log("multiply:", multiply(3, 4));
console.log("PI:", PI);

// --- Default import ---
import Greeter from "./greeter";

console.log("\n--- Default Import ---");
let g: Greeter = new Greeter("Hi");
console.log("greet:", g.greet("World"));

// --- Namespace import ---
import * as MathUtils from "./math_utils";

console.log("\n--- Namespace Import ---");
console.log("square:", MathUtils.square(9));
console.log("subtract:", MathUtils.subtract(10, 3));

// --- Mixed imports ---
import Greeter2, { shout } from "./greeter";

console.log("\n--- Mixed Default + Named ---");
let g2: Greeter2 = new Greeter2("Hey");
console.log("greet:", g2.greet("Tails"));
console.log("shout:", shout("hello"));

// --- Re-export ---
export { add as sum, multiply as product } from "./math_utils";

console.log("\n--- Re-export ---");
console.log("sum(1,2):", sum(1, 2));
console.log("product(1,2):", product(1, 2));

// --- Import for side-effects (console output from imported module runs) ---
import "./math_utils";

console.log("\n=== MODULAR DEMO DONE ===");
