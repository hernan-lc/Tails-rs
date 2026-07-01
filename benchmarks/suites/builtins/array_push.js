// Suite: builtins
// Benchmark: array_push
// Measures: Array.push growth

const ITER = 200000;
const t0 = Date.now();
const arr = [];
for (let i = 0; i < ITER; i++) {
  arr.push(i);
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(arr.length);
