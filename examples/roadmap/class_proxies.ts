class Vec {
  #x;
  #y;
  constructor(x, y) {
    this.#x = x;
    this.#y = y;
  }
  get length() {
    return Math.hypot(this.#x, this.#y);
  }
}
const audited = [];
const v = new Proxy(new Vec(3, 4), {
  get(t, k) {
    audited.push(String(k));
    return Reflect.get(t, k);
  },
});
console.log("length = " + v.length);
"audited keys: " + audited.join(", ");
