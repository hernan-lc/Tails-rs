// Suite: async
// Benchmark: promises
// Measures: Promise creation and resolution

const ITER = 100000;
const t0 = Date.now();
let sum = 0;
for (let i = 0; i < ITER; i++) {
  await new Promise(resolve => resolve(i));
  sum += i;
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(sum);
