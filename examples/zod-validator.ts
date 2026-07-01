// Run from project root:
//   cargo run --bin tails -- build -p tails-validator
//   cargo run --bin tails -- run examples/zod-validator.ts
//
// ============================================================================

import * as v from "../dist/tails-validator.native";

function test(name: string, schema: v.Schema, value: unknown, expectPass: boolean) {
  const raw = v.validate(schema, value);
  const result = JSON.parse(raw) as v.ValidateResult;
  const pass = result.success === expectPass;
  const status = pass ? "PASS" : "FAIL";
  console.log(`  [${status}] ${name}`);
  if (!pass) {
    console.log(`    Expected: ${expectPass ? "success" : "failure"}`);
    console.log(`    Got: ${result.success ? "success" : "failure"}`);
    if (!result.success) {
      console.log(`    Error: ${JSON.stringify(result.error.issues, null, 2)}`);
    }
  }
  return pass;
}

let total = 0;
let passed = 0;

function run(name: string, fn: () => void) {
  console.log(`\n=== ${name} ===`);
  const before = total;
  fn();
  passed += total - before;
}

// ============================================================================
// String validators
// ============================================================================

run("stringMin", () => {
  const schema = v.stringMin(3);
  total += 3;
  test("valid min", schema, "hello", true);
  test("too short", schema, "ab", false);
  test("exact min", schema, "abc", true);
});

run("stringMax", () => {
  const schema = v.stringMax(5);
  total += 2;
  test("valid max", schema, "hello", true);
  test("too long", schema, "helloo", false);
});

run("stringLength", () => {
  const schema = v.stringLength(5);
  total += 2;
  test("exact length", schema, "hello", true);
  test("wrong length", schema, "hi", false);
});

run("stringPattern", () => {
  const schema = v.stringPattern("^[A-Z][a-z]+$");
  total += 2;
  test("matches pattern", schema, "Hello", true);
  test("no match", schema, "hello", false);
});

run("stringEmail", () => {
  const schema = v.stringEmail();
  total += 2;
  test("valid email", schema, "user@example.com", true);
  test("invalid email", schema, "not-an-email", false);
});

run("stringUrl", () => {
  const schema = v.stringUrl();
  total += 2;
  test("valid url", schema, "https://example.com", true);
  test("invalid url", schema, "not-a-url", false);
});

run("stringUuid", () => {
  const schema = v.stringUuid();
  total += 2;
  test("valid uuid", schema, "550e8400-e29b-41d4-a716-446655440000", true);
  test("invalid uuid", schema, "not-a-uuid", false);
});

run("stringDatetime", () => {
  const schema = v.stringDatetime();
  total += 2;
  test("valid datetime", schema, "2024-01-15", true);
  test("invalid datetime", schema, "not-a-date", false);
});

run("stringIPv4", () => {
  const schema = v.stringIPv4();
  total += 2;
  test("valid ipv4", schema, "192.168.1.1", true);
  test("invalid ipv4", schema, "999.999.999.999", false);
});

run("stringIPv6", () => {
  const schema = v.stringIPv6();
  total += 2;
  test("valid ipv6", schema, "2001:0db8:85a3:0000:0000:8a2e:0370:7334", true);
  test("invalid ipv6", schema, "not-ipv6", false);
});

run("stringPhone", () => {
  const schema = v.stringPhone();
  total += 2;
  test("valid phone", schema, "+1-234-567-8900", true);
  test("invalid phone", schema, "123", false);
});

run("stringBase64", () => {
  const schema = v.stringBase64();
  total += 2;
  test("valid base64", schema, "SGVsbG8=", true);
  test("invalid base64", schema, "not-base64!", false);
});

// ============================================================================
// Number validators
// ============================================================================

run("numberMin", () => {
  const schema = v.numberMin(0);
  total += 2;
  test("valid min", schema, 5, true);
  test("below min", schema, -1, false);
});

run("numberMax", () => {
  const schema = v.numberMax(100);
  total += 2;
  test("valid max", schema, 50, true);
  test("above max", schema, 101, false);
});

run("numberInt", () => {
  const schema = v.numberInt();
  total += 2;
  test("integer", schema, 42, true);
  test("float", schema, 3.14, false);
});

run("numberPositive", () => {
  const schema = v.numberPositive();
  total += 2;
  test("positive", schema, 5, true);
  test("negative", schema, -5, false);
});

run("numberNegative", () => {
  const schema = v.numberNegative();
  total += 2;
  test("negative", schema, -5, true);
  test("positive", schema, 5, false);
});

run("numberMultipleOf", () => {
  const schema = v.numberMultipleOf(3);
  total += 2;
  test("multiple of 3", schema, 9, true);
  test("not multiple of 3", schema, 7, false);
});

run("numberFinite", () => {
  const schema = v.numberFinite();
  total += 1;
  test("finite number", schema, 42, true);
});

// ============================================================================
// Array validators
// ============================================================================

run("arrayMin", () => {
  const schema = v.arrayMin(v.string(), 2);
  total += 2;
  test("valid min", schema, ["a", "b", "c"], true);
  test("too few", schema, ["a"], false);
});

run("arrayMax", () => {
  const schema = v.arrayMax(v.string(), 3);
  total += 2;
  test("valid max", schema, ["a", "b"], true);
  test("too many", schema, ["a", "b", "c", "d"], false);
});

run("arrayLength", () => {
  const schema = v.arrayLength(v.number(), 3);
  total += 2;
  test("exact length", schema, [1, 2, 3], true);
  test("wrong length", schema, [1, 2], false);
});

run("arrayUnique", () => {
  const schema = v.arrayUnique(v.string());
  total += 2;
  test("unique items", schema, ["a", "b", "c"], true);
  test("duplicate items", schema, ["a", "a", "b"], false);
});

// ============================================================================
// Composable validators
// ============================================================================

run("optional", () => {
  const schema = v.optional(v.stringMin(3));
  total += 2;
  test("present and valid", schema, "hello", true);
  test("null (optional)", schema, null, true);
});

run("nullable", () => {
  const schema = v.nullable(v.stringMin(3));
  total += 2;
  test("present and valid", schema, "hello", true);
  test("null (nullable)", schema, null, true);
});

run("withDefault", () => {
  const schema = v.withDefault(v.numberMin(0), 10);
  total += 2;
  test("present value", schema, 5, true);
  test("null uses default", schema, null, true);
});

run("transform", () => {
  const schema = v.transform(v.string(), "uppercase");
  total += 1;
  test("transforms to uppercase", schema, "hello", true);
});

run("refine", () => {
  const schema = v.refine(v.numberMin(0), "Must be non-negative");
  total += 1;
  test("refined value", schema, 5, true);
});

run("pipe", () => {
  const schema = v.pipe([v.string(), v.stringMin(3)]);
  total += 2;
  test("valid pipeline", schema, "hello", true);
  test("fails pipeline", schema, "hi", false);
});

run("customError", () => {
  const schema = v.customError(v.stringMin(3), "Name is too short");
  total += 1;
  test("valid with custom error", schema, "hello", true);
});

run("lazy", () => {
  const schema = v.lazy("ref", v.stringMin(1));
  total += 1;
  test("lazy schema works", schema, "hello", true);
});

// ============================================================================
// Additional builders
// ============================================================================

run("literal", () => {
  const schema = v.literal("hello");
  total += 2;
  test("exact match", schema, "hello", true);
  test("no match", schema, "world", false);
});

run("enumValues", () => {
  const schema = v.enumValues(["a", "b", "c"]);
  total += 2;
  test("valid enum", schema, "a", true);
  test("invalid enum", schema, "d", false);
});

run("union", () => {
  const schema = v.union([v.string(), v.numberInt()]);
  total += 2;
  test("string variant", schema, "hello", true);
  test("number variant", schema, 42, true);
});

run("tuple", () => {
  const schema = v.tuple([v.string(), v.numberInt()]);
  total += 2;
  test("valid tuple", schema, ["hello", 42], true);
  test("wrong tuple", schema, [42, "hello"], false);
});

run("coerce", () => {
  const schema = v.coerce("number", v.numberMin(0));
  total += 2;
  test("coerce string to number", schema, "42", true);
  test("coerce invalid", schema, "abc", false);
});

run("record", () => {
  const schema = v.record(v.numberMin(0));
  total += 2;
  test("valid record", schema, { a: 1, b: 2 }, true);
  test("invalid record value", schema, { a: -1 }, false);
});

// ============================================================================
// Summary
// ============================================================================

console.log(`\n${"=".repeat(50)}`);
console.log(`Total: ${total} tests, Passed: ${passed}, Failed: ${total - passed}`);
console.log(`${"=".repeat(50)}`);
