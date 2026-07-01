// Simple example showing native module usage
// Run from project root: tails examples/simple-native.ts
// From examples/ directory, ../dist/ resolves to the project's dist/ folder
import myModule from "../dist/my-tails-module.native";

console.log("=== Tails Native Module Example ===");
// Test greet function
const greeting = myModule.greet("Tails");
console.log(`greet("Tails") = "${greeting}"`);

// Test add function
const sum = myModule.add(10, 20);
console.log(`add(10, 20) = ${sum}`);

// Test multiply function
const product = myModule.multiply(5, 6);
console.log(`multiply(5, 6) = ${product}`);

console.log("");
console.log("=== Counter Class Test ===");

// Test Counter class
const counter = new myModule.Counter(10);
console.log(`counter created with initial value 10`);

counter.increment();
counter.increment();
counter.increment();
console.log(`after 3 increments: ${counter.getCount()}`);

counter.decrement();
console.log(`after 1 decrement: ${counter.getCount()}`);

const counter2 = new myModule.Counter(0);
counter2.increment();
counter2.increment();
console.log(`second counter: ${counter2.getCount()}`);

console.log("");
console.log("=== All tests passed! ===");
