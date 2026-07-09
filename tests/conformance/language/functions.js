/*---
description: Tests function declarations, expression, scope, closures, and recursion.
---*/

// Basic function declaration & call
function add(x, y) {
    return x + y;
}
assert.sameValue(add(10, 20), 30, "Function declaration & execution");

// Closures & Lexical Scope
fnOuter = function(outerVal) {
    return function(innerVal) {
        return outerVal + innerVal;
    };
};
let addFive = fnOuter(5);
assert.sameValue(addFive(10), 15, "Closure captures outer variable");

// Recursive function
function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
assert.sameValue(factorial(5), 120, "Recursive factorial function");

// Arrow functions
const double = (x) => x * 2;
assert.sameValue(double(21), 42, "Arrow function execution");

// Default parameters
function greet(name = "World") {
    return "Hello " + name;
}
assert.sameValue(greet(), "Hello World", "Default parameter fallback");
assert.sameValue(greet("Alice"), "Hello Alice", "Default parameter override");
