const fs = require('fs');
const path = require('path');

const resultsPath = process.argv[2] || path.join(__dirname, '..', 'results', 'latest.json');
const outPath = process.argv[3] || path.join(__dirname, '..', 'results', 'REPORT.md');

const data = JSON.parse(fs.readFileSync(resultsPath, 'utf8'));

const runtimes = [...new Set(data.entries.map(e => e.runtime))];
const scripts = [...new Set(data.entries.map(e => e.script))];

let md = '# Cross-Runtime Benchmark Report\n\n';
md += `Generated: ${new Date().toISOString()}\n\n`;

for (const script of scripts) {
  md += `## ${path.basename(script)}\n\n`;
  md += '| Runtime | Mean (ms) | Median (ms) | Stdev (ms) | Min (ms) | Max (ms) | Runs |\n';
  md += '|---------|-----------|-------------|------------|----------|----------|------|\n';

  const entries = data.entries.filter(e => e.script === script && e.mean_us !== undefined);
  const skipped = data.entries.filter(e => e.script === script && e.skipped);

  for (const rt of runtimes) {
    const e = entries.find(x => x.runtime === rt);
    if (!e) {
      const sk = skipped.find(x => x.runtime === rt);
      if (sk) {
        md += `| ${rt} | SKIP | — | — | — | — | — |\n`;
      } else {
        md += `| ${rt} | — | — | — | — | — | — |\n`;
      }
      continue;
    }
    const mean = (e.mean_us / 1000).toFixed(2);
    const median = (e.median_us / 1000).toFixed(2);
    const stdev = (e.stdev_us / 1000).toFixed(2);
    const min = (e.min_us / 1000).toFixed(2);
    const max = (e.max_us / 1000).toFixed(2);
    const n = e.n || e.runs_completed || 0;
    md += `| ${rt} | ${mean} | ${median} | ${stdev} | ${min} | ${max} | ${n} |\n`;
  }
  md += '\n';
}

fs.writeFileSync(outPath, md);
console.log('Report written to', outPath);
