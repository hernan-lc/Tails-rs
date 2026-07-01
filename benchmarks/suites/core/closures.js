// Suite: core
// Benchmark: closures
// Measures: function call overhead with closures

function makeAdder(n) {
  return function(x) { return x + n; };
}

const ITER = 1000000;
const t0 = Date.now();
let sum = 0;
for (let i = 0; i < ITER; i++) {
  const add5 = makeAdder(i);
  sum += add5(1);
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(sum);
