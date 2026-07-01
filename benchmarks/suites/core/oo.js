// Suite: core
// Benchmark: oo
// Measures: object property assignment and method dispatch

function makeClass(seed) {
  function Cls(val) {
    this.x = val;
    this.data = {};
    for (let i = 0; i < 10; i++) {
      this.data['k' + i] = i * seed;
    }
  }
  Cls.prototype.inc = function() { this.x += 1; return this.x; };
  return Cls;
}

const C = makeClass(7);
const ITER = 100000;
const t0 = Date.now();
const arr = [];
for (let i = 0; i < ITER; i++) {
  const o = new C(i);
  o.inc();
  arr.push(o.x);
}
const elapsed = Date.now() - t0;
console.log(elapsed);
console.log(arr.length);
