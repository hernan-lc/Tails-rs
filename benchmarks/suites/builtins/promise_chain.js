// Suite: builtins
// Benchmark: promise_chain
// Measures: Promise resolution and .then() chaining

function chain(depth) {
  let p = Promise.resolve(0);
  for (let i = 1; i <= depth; i++) {
    p = p.then(v => v + i);
  }
  return p;
}

const ITER = 500;
const t0 = Date.now();
let last = 0;
for (let i = 0; i < ITER; i++) {
  last = await chain(20);
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(last);
