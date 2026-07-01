let total = 0;
let passed = 0;
let failed = 0;

function check(name: string, ok: boolean) {
  total++;
  if (ok) {
    passed++;
    console.log("  [PASS] ", name);
  } else {
    failed++;
    console.error("  [FAIL] ", name);
  }
}

// ============================================================================
// Number.prototype methods
// ============================================================================

console.log("\n=== Number.prototype ===");

check("toFixed basic", (3.14159).toFixed(2) === "3.14");
check("toFixed 0 decimals", (3.7).toFixed(0) === "4");
check("toFixed many decimals", (0.1 + 0.2).toFixed(10) === "0.3000000000");
check("toFixed negative", (-3.14).toFixed(1) === "-3.1");
check("toFixed integer", (42).toFixed(2) === "42.00");
check("toFixed NaN", NaN.toFixed(2) === "NaN");
check("toFixed Infinity", Infinity.toFixed(2) === "Infinity");
check("toFixed -Infinity", (-Infinity).toFixed(2) === "-Infinity");

check("toString base 10", (255).toString(10) === "255");
check("toString base 16", (255).toString(16) === "ff");
check("toString base 2", (10).toString(2) === "1010");
check("toString base 8", (64).toString(8) === "100");
check("toString negative hex", (-255).toString(16) === "-ff");

check("valueOf number", (42).valueOf() === 42);
check("toExponential", (1234.5).toExponential(2) === "1.23e+3");
check("toExponential no args", (1234.5).toExponential() === "1.2345e+3");
check("toPrecision", (123.456).toPrecision(4) === "123.5");

check("Number.isInteger(42)", Number.isInteger(42) === true);
check("Number.isInteger(3.14)", Number.isInteger(3.14) === false);
check("Number.isInteger(NaN)", Number.isInteger(NaN) === false);
check("Number.isInteger(Infinity)", Number.isInteger(Infinity) === false);
check("Number.isSafeInteger(42)", Number.isSafeInteger(42) === true);
check("Number.isSafeInteger(2^53)", Number.isSafeInteger(2 ** 53) === false);

// ============================================================================
// Boolean.prototype methods
// ============================================================================

console.log("\n=== Boolean.prototype ===");

check("true.toString()", true.toString() === "true");
check("false.toString()", false.toString() === "false");
check("true.valueOf()", true.valueOf() === true);
check("false.valueOf()", false.valueOf() === false);

// ============================================================================
// String.prototype.matchAll
// ============================================================================

console.log("\n=== String.prototype.matchAll ===");

check(
  "matchAll global regex",
  "test1test2test3".matchAll(/test(\d)/g).length === 3,
);
check("matchAll no matches", "hello".matchAll(/xyz/g).length === 0);
check("matchAll capture groups", "aab".matchAll(/(a)b/g).length === 1);

// ============================================================================
// Math methods
// ============================================================================

console.log("\n=== Math methods ===");

check("Math.floor", Math.floor(3.7) === 3 && Math.floor(-3.7) === -4);
check("Math.ceil", Math.ceil(3.2) === 4 && Math.ceil(-3.2) === -3);
check("Math.round", Math.round(3.5) === 4 && Math.round(3.4) === 3);
check("Math.abs", Math.abs(-5) === 5 && Math.abs(5) === 5);
check("Math.sqrt", Math.sqrt(9) === 3);
check("Math.pow", Math.pow(2, 3) === 8);

// ============================================================================
// Summary
// ============================================================================

console.log("\n==================================================");
console.log(
  "Total: " + total + " tests, Passed: " + passed + ", Failed: " + failed,
);
console.log("==================================================");

if (failed > 0) {
  console.log("\nSome tests failed!");
} else {
  console.log("\nAll tests passed!");
}
