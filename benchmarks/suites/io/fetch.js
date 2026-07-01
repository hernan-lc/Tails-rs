// Suite: io
// Benchmark: fetch
// Measures: fetch() against httpbin.org (single-flight, no parallelism)
// Fetches a small JSON endpoint and reads response.text().

const ITER = 20;
const url = 'https://httpbin.org/get';
const t0 = Date.now();
let totalBytes = 0;
for (let i = 0; i < ITER; i++) {
  const res = await fetch(url);
  const text = await res.text();
  totalBytes += text.length;
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(totalBytes);
