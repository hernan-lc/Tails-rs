// Suite: builtins
// Benchmark: json_parse
// Measures: JSON.parse on a ~100 KB payload

const fs = require('fs');
const path = 'benchmarks/fixtures/medium.json';

if (!fs.existsSync(path)) {
  console.log('SKIP');
  process.exit(0);
}

const raw = fs.readFileSync(path, 'utf8');
const ITER = 20;
const t0 = Date.now();
let last = null;
for (let i = 0; i < ITER; i++) {
  last = JSON.parse(raw);
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(last.data.length);
