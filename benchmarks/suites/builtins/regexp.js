// Suite: builtins
// Benchmark: regexp
// Measures: RegExp.exec on repeated input

const re = /([a-z]+)\s+([a-z]+)/g;
const text = 'the quick brown fox jumps over the lazy dog '.repeat(200);
const ITER = 1000;
const t0 = Date.now();
let count = 0;
for (let i = 0; i < ITER; i++) {
  re.lastIndex = 0;
  let m;
  while ((m = re.exec(text)) !== null) {
    count++;
  }
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(count);
