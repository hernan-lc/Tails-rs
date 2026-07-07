/*---
includes: [assert.js]
flags: [onlyStrict]
---*/

var x = 1;
assert.sameValue(x, 1, "Basic assignment");
assert.sameValue(typeof x, "number");
