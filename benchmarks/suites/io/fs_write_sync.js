// Suite: io
// Benchmark: fs_write_sync
// Measures: fs.writeFileSync throughput

const fs = require('fs');
const path = 'benchmarks/fixtures/.write_tmp';

const ITER = 200;
const payload = 'x'.repeat(1024);
const t0 = Date.now();
let totalBytes = 0;
for (let i = 0; i < ITER; i++) {
  fs.writeFileSync(path, payload);
  totalBytes += payload.length;
}
const elapsed = Date.now() - t0;
try { fs.unlinkSync(path); } catch (_) {}
console.log(elapsed);
console.log(totalBytes);
