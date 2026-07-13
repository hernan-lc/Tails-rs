class LRUCache {
  constructor() {
    this.max = 1000;
    this.map = new Map();
  }
  set(k, v) {
    this.map.set(k, v);
    return this;
  }
  get(k) {
    return this.map.get(k);
  }
}
module.exports = LRUCache;
