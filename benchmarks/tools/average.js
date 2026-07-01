const data = [];
process.stdin.setEncoding('utf8');
let buf = '';
process.stdin.on('data', chunk => buf += chunk);
process.stdin.on('end', () => {
  const vals = buf.trim().split(/[\s\n]+/).map(Number).filter(v => !Number.isNaN(v));
  if (vals.length === 0) {
    console.error('average.js: no numeric input');
    process.exit(1);
  }
  vals.sort((a, b) => a - b);
  const sum = vals.reduce((a, b) => a + b, 0);
  const mean = sum / vals.length;
  const median = vals.length % 2 === 0 ? (vals[vals.length / 2 - 1] + vals[vals.length / 2]) / 2 : vals[Math.floor(vals.length / 2)];
  const variance = vals.reduce((acc, v) => acc + (v - mean) ** 2, 0) / vals.length;
  const stdev = Math.sqrt(variance);
  console.log(JSON.stringify({
    mean_us: Math.round(mean * 1000),
    median_us: Math.round(median * 1000),
    stdev_us: Math.round(stdev * 1000),
    min_us: Math.round(vals[0] * 1000),
    max_us: Math.round(vals[vals.length - 1] * 1000),
    n: vals.length
  }));
});
