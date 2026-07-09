/*---
description: Tests Map, Set, Object methods (keys, values, entries, assign), and Array methods.
---*/

// Map tests
let map = new Map();
map.set("a", 1);
map.set("b", 2);
assert.sameValue(map.get("a"), 1, "Map get");
assert.sameValue(map.size, 2, "Map size");
assert.sameValue(map.has("b"), true, "Map has true");
map.delete("a");
assert.sameValue(map.has("a"), false, "Map has false after delete");

// Set tests
let set = new Set();
set.add(1);
set.add(2);
set.add(1); // duplicate
assert.sameValue(set.size, 2, "Set size ignores duplicates");
assert.sameValue(set.has(2), true, "Set has");
set.delete(2);
assert.sameValue(set.has(2), false, "Set has false after delete");

// Object methods
let person = { name: "Alice", age: 30 };
assert.sameValue(Object.keys(person).length, 2, "Object.keys length");
assert.sameValue(Object.values(person).length, 2, "Object.values length");
assert.sameValue(Object.entries(person).length, 2, "Object.entries length");

let targetObj = { name: "Alice" };
Object.assign(targetObj, { city: "NYC" });
assert.sameValue(targetObj.city, "NYC", "Object.assign properties");

// Array methods
let arr = [1, 2, 3, 4, 5];
arr.push(6);
assert.sameValue(arr.pop(), 6, "Array push and pop");

let mapped = arr.map(function(x) { return x * 2; }).join(",");
assert.sameValue(mapped, "2,4,6,8,10", "Array map and join");

let filtered = arr.filter(function(x) { return x > 3; }).join(",");
assert.sameValue(filtered, "4,5", "Array filter");

let reduced = arr.reduce(function(a, b) { return a + b; }, 0);
assert.sameValue(reduced, 15, "Array reduce");

let found = arr.find(function(x) { return x > 3; });
assert.sameValue(found, 4, "Array find");

assert.sameValue(arr.some(function(x) { return x > 4; }), true, "Array some");
assert.sameValue(arr.every(function(x) { return x > 0; }), true, "Array every");
assert.sameValue(arr.includes(3), true, "Array includes");
assert.sameValue(arr.slice(1, 3).join(","), "2,3", "Array slice");
assert.sameValue([[1, 2], [3, 4]].flat().join(","), "1,2,3,4", "Array flat");
