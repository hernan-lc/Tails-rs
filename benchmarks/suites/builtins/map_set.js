// Suite: builtins
// Benchmark: map_set
// Measures: Map set/get iteration

const ITER = 50000;
const t0 = Date.now();
const m = new Map();
for (let i = 0; i < ITER; i++) {
  m.set(i, i * 2);
}
let sum = 0;
for (const [k, v] of m) {
  sum += v;
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(sum);
