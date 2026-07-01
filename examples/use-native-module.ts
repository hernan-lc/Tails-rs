// Example TypeScript file showing how to import the native module
// From examples/ directory, the native module is at ../dist/
import myModule from "../dist/my-tails-module.native";

// Using the native functions
const greeting = myModule.greet("World");
console.log(greeting); // "Hello, World!"

const sum = myModule.add(1, 2);
console.log(sum); // 3

const product = myModule.multiply(3, 4);
console.log(product); // 12
