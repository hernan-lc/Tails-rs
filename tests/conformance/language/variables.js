/*---
description: Tests let, const, var declarations, and primitive types.
---*/

// Variable declarations
let x = 10;
const y = 20;
var z = 30;
assert.sameValue(x + y + z, 60, "Basic let, const, var addition");

// Typeof and primitives
let s = "hello";
let b = true;
let n = null;
let u = undefined;

assert.sameValue(typeof s, "string", "typeof string");
assert.sameValue(typeof b, "boolean", "typeof boolean");
assert.sameValue(typeof n, "object", "typeof null is object");
assert.sameValue(typeof u, "undefined", "typeof undefined");

// Compound assignment
let w = 10;
w += 5;
assert.sameValue(w, 15, "Compound addition");
w *= 2;
assert.sameValue(w, 30, "Compound multiplication");

// Comparisons
assert(5 == 5, "5 == 5");
assert(5 === 5, "5 === 5");
assert(5 != 3, "5 != 3");
assert(5 !== "5", "5 !== '5'");
assert(5 < 10, "5 < 10");
assert(5 > 3, "5 > 3");

// Logical operators
assert.sameValue(true && false, false, "true && false");
assert.sameValue(true || false, true, "true || false");
assert.sameValue(!true, false, "!true");

// Void operator
assert.sameValue(void 0, undefined, "void 0");
assert.sameValue(typeof (void 0), "undefined", "typeof void 0");
