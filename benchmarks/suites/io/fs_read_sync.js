// Suite: io
// Benchmark: fs_read_sync
// Measures: fs.readFileSync throughput

const fs = require('fs');
const path = 'benchmarks/fixtures/medium_file.txt';

if (!fs.existsSync(path)) {
  console.log('SKIP');
  process.exit(0);
}

const ITER = 200;
const t0 = Date.now();
let totalBytes = 0;
for (let i = 0; i < ITER; i++) {
  const buf = fs.readFileSync(path, 'utf8');
  totalBytes += buf.length;
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(totalBytes);
