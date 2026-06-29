export default {
  metadata: {
    name: "counter",
    version: "1.0.0"
  },
  count: 0,
  increment() {
    this.count++;
  },
  getCount() {
    return this.count;
  }
};
