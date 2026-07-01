import fs from "../dist/tails-fs.native";
console.log("fs keys:", Object.keys(fs));
console.log("mkdir type:", typeof fs.mkdir);
console.log("mkdir:", fs.mkdir);
const result = fs.mkdir("/tmp/tails-test-fs", true);
console.log("mkdir result:", result);
