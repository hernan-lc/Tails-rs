// Suite: builtins
// Benchmark: date
// Measures: Date.now + Date parsing loop

const fmt = '2025-01-15T12:34:56.789Z';
const ITER = 500000;
const t0 = Date.now();
let sum = 0;
for (let i = 0; i < ITER; i++) {
  sum += Date.now();
  Date.parse(fmt);
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(sum);
