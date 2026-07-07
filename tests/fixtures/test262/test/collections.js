/*---
includes: [assert.js]
description: Map and Set conformance tests
---*/

// Map
var m = new Map();
m.set("a", 1);
m.set("b", 2);
assert.sameValue(m.size, 2);
assert.sameValue(m.get("a"), 1);
assert.sameValue(m.has("c"), false);
m.delete("a");
assert.sameValue(m.size, 1);
m.clear();
assert.sameValue(m.size, 0);

// Set
var s = new Set();
s.add(1);
s.add(2);
s.add(1); // duplicate
assert.sameValue(s.size, 2);
assert.sameValue(s.has(1), true);
s.delete(1);
assert.sameValue(s.has(1), false);
