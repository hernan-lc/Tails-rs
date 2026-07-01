// Simple example showing native module usage
// Run from project root: tails examples/simple-native.ts
// From examples/ directory, ../dist/ resolves to the project's dist/ folder
import fs from "../dist/fs.native";
import path from "../dist/path.native";
console.log(fs);
console.log(path);
