class Vec {
  #x;
  #y;
  constructor(x: number, y: number) {
    this.#x = x;
    this.#y = y;
  }
  get length() {
    return Math.hypot(this.#x, this.#y);
  }
}
const audited: string[] = [];
const v = new Proxy(new Vec(3, 4), {
  get(t, k) {
    audited.push(String(k));
    return Reflect.get(t, k);
  },
});
console.log("length = " + v.length);
console.log("audited keys: " + audited.join(", "));
