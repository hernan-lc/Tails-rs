// Suite: core
// Benchmark: generators
// Measures: generator function creation, next(), and return()

function* gen(limit) {
  for (let i = 0; i < limit; i++) {
    yield i * 2;
  }
}

const ITER = 2000;
const t0 = Date.now();
let count = 0;
for (let i = 0; i < ITER; i++) {
  const g = gen(100);
  for (const v of g) {
    count += v;
  }
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(count);
