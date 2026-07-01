const fs = require('fs');
const path = require('path');

const fixturesDir = path.join(__dirname, '..', 'fixtures');
fs.mkdirSync(fixturesDir, { recursive: true });

const smallObj = { a: 1, b: 2, c: 3 };
const mediumObj = { data: Array.from({ length: 15000 }, (_, i) => ({ id: i, name: `item_${i}`, value: Math.random() })) };
const largeObj = { data: Array.from({ length: 100000 }, (_, i) => ({ id: i, name: `item_${i}`, value: Math.random() })) };

fs.writeFileSync(path.join(fixturesDir, 'small.json'), JSON.stringify(smallObj));
fs.writeFileSync(path.join(fixturesDir, 'medium.json'), JSON.stringify(mediumObj));
fs.writeFileSync(path.join(fixturesDir, 'large.json'), JSON.stringify(largeObj));

const lines = Array.from({ length: 10000 }, (_, i) => `Line ${i}: The quick brown fox jumps over the lazy dog. ${'x'.repeat(50)}`);
fs.writeFileSync(path.join(fixturesDir, 'medium_file.txt'), lines.join('\n'));

console.log('Fixtures generated in', fixturesDir);
