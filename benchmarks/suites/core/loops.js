// Suite: core
// Benchmark: loops
// Measures: tight for-loop with integer addition

const ITER = 5000000;
const t0 = Date.now();
let x = 0;
for (let i = 0; i < ITER; i++) {
  x = x + i;
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(x);
