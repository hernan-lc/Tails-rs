// Suite: async
// Benchmark: async_await
// Measures: async function call overhead

async function work(i) {
  return i * 2;
}

(async () => {
  const ITER = 50000;
  const t0 = Date.now();
  let sum = 0;
  for (let i = 0; i < ITER; i++) {
    sum += await work(i);
  }
  const elapsed = Date.now() - t0;
  console.log(elapsed);
  console.log(sum);
})();
