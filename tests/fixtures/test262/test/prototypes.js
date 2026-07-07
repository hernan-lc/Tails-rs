/*---
includes: [assert.js]
description: String and Array prototype conformance tests
---*/

// String
var s = "hello";
assert.sameValue(s.toUpperCase(), "HELLO");
assert.sameValue(s.slice(1, 3), "el");
assert.sameValue("  trim  ".trim(), "trim");

// Array
var a = [1, 2, 3];
var doubled = a.map(function(x) { return x * 2; });
assert.sameValue(doubled.length, 3);
assert.sameValue(doubled[0], 2);
assert.sameValue(doubled[1], 4);
assert.sameValue(doubled[2], 6);

var filtered = a.filter(function(x) { return x > 1; });
assert.sameValue(filtered.length, 2);
assert.sameValue(filtered[0], 2);
assert.sameValue(filtered[1], 3);

assert.sameValue(a.reduce(function(acc, x) { return acc + x; }, 0), 6);
