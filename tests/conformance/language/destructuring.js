/*---
description: Tests array and object destructuring, spread, and defaults.
---*/

// Array destructuring basic
const [a, b, c] = [1, 2, 3];
assert.sameValue(a, 1, "array destructuring 1st");
assert.sameValue(b, 2, "array destructuring 2nd");
assert.sameValue(c, 3, "array destructuring 3rd");

// Array destructuring skip
const [d, , f] = [1, 2, 3];
assert.sameValue(d, 1, "array destructuring skip 1st");
assert.sameValue(f, 3, "array destructuring skip 3rd");

// Object destructuring basic
const {x, y} = {x: 10, y: 20};
assert.sameValue(x, 10, "object destructuring x");
assert.sameValue(y, 20, "object destructuring y");

// Object destructuring renamed
const {x: valA, y: valB} = {x: 10, y: 20};
assert.sameValue(valA, 10, "object destructuring renamed x");
assert.sameValue(valB, 20, "object destructuring renamed y");

// Destructuring with default
const {p1 = 10, p2 = 20} = {p1: 5};
assert.sameValue(p1, 5, "destructuring default override");
assert.sameValue(p2, 20, "destructuring default fallback");

// Nested destructuring
const {nested: {b: nestedB, c: nestedC}} = {nested: {b: 1, c: 2}};
assert.sameValue(nestedB, 1, "nested destructuring b");
assert.sameValue(nestedC, 2, "nested destructuring c");

// Array spread
const arr1 = [1, 2];
const arr2 = [...arr1, 3, 4];
assert.sameValue(arr2.length, 4, "spread array length");
assert.sameValue(arr2[0], 1, "spread array element 0");

// Object spread
const obj1 = {o1: 1, o2: 2};
const obj2 = {...obj1, o3: 3};
assert.sameValue(obj2.o1, 1, "spread object key o1");
assert.sameValue(obj2.o3, 3, "spread object key o3");
