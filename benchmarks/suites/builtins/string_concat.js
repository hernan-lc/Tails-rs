// Suite: builtins
// Benchmark: string_concat
// Measures: repeated string concatenation

let s = 'hello';
const ITER = 50000;
const t0 = Date.now();
for (let i = 0; i < ITER; i++) {
  s = s + 'x';
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(s.length);
