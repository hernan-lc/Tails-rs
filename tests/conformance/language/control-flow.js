/*---
description: Tests conditional statements, loops, switch case, and break/continue.
---*/

// If-else
let a = 5;
let ifResult = "";
if (a > 10) {
    ifResult = "big";
} else if (a > 3) {
    ifResult = "medium";
} else {
    ifResult = "small";
}
assert.sameValue(ifResult, "medium", "if-else-if condition");

// Ternary
let ternary = a > 3 ? "big" : "small";
assert.sameValue(ternary, "big", "ternary operator");

// For loop
let sum = 0;
for (let i = 1; i <= 5; i++) {
    sum = sum + i;
}
assert.sameValue(sum, 15, "for loop sum");

// While loop
let w = 5;
while (w > 0) {
    w = w - 1;
}
assert.sameValue(w, 0, "while loop decrement");

// Do-while loop
let d = 0;
do {
    d = d + 1;
} while (d < 3);
assert.sameValue(d, 3, "do-while loop increment");

// For-in loop
let obj = { a: 1, b: 2 };
let keys = "";
for (let k in obj) {
    keys = keys + k;
}
assert(keys === "ab" || keys === "ba", "for-in loop over object keys");

// Switch statement
let day = 2;
let dayName = "";
switch (day) {
    case 1:
        dayName = "Mon";
        break;
    case 2:
        dayName = "Tue";
        break;
    default:
        dayName = "Other";
}
assert.sameValue(dayName, "Tue", "switch statement matching case 2");

// Switch default
let otherDay = 99;
let otherDayName = "";
switch (otherDay) {
    case 1:
        otherDayName = "Mon";
        break;
    default:
        otherDayName = "Other";
}
assert.sameValue(otherDayName, "Other", "switch statement matching default case");

// Break and continue
let bcSum = 0;
for (let i = 0; i < 10; i++) {
    if (i === 3) continue;
    if (i === 7) break;
    bcSum = bcSum + i;
}
assert.sameValue(bcSum, 18, "break and continue in loop");
