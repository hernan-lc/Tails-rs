import * as v from "./dist/tails-validator.native";

const schema1 = v.object({ a: v.string() }, ["a"], false);
const schema2 = v.object({ b: v.number() }, ["b"], false);
const intersected = v.intersection([schema1, schema2]);

// Using JSON strings to bypass potential handle registry issues between binary and cdylib
const input = { a: "hello", b: 42, c: "should be redacted" };
const raw = v.validate(intersected, JSON.stringify(input));
const result = JSON.parse(raw);

console.log("Result:", JSON.stringify(result, null, 2));

if (result.success) {
    if (result.data.c !== undefined) {
        console.log("FAIL: 'c' was NOT redacted");
    } else if (result.data.a === "hello" && result.data.b === 42) {
        console.log("PASS: 'a' and 'b' preserved, 'c' redacted");
    } else {
        console.log("FAIL: data mismatch", JSON.stringify(result.data));
    }
} else {
    console.log("FAIL: validation failed", JSON.stringify(result.error, null, 2));
}
